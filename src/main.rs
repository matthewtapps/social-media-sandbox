use eframe::egui;
use egui::Vec2;
use social_media_sandbox::{
    models::{Activity, AgentType, SimulationConfig},
    Simulation,
};
pub struct SimulationApp {
    running: bool,
    simulation: Simulation,
    open_agent_windows: Vec<usize>, // Track multiple open windows
}

impl Default for SimulationApp {
    fn default() -> Self {
        Self {
            running: false,
            simulation: Simulation::new(SimulationConfig::default()),
            open_agent_windows: Vec::new(),
        }
    }
}

impl eframe::App for SimulationApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.ui(ctx);

        if self.running {
            ctx.request_repaint();
            self.simulation.tick()
        }
    }
}

impl SimulationApp {
    fn ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("Configuration");

            if ui
                .button(if self.running { "Stop" } else { "Start" })
                .clicked()
            {
                self.running = !self.running;
            }

            let current_individuals = self
                .simulation
                .agents
                .iter()
                .filter(|a| matches!(a.get_type(), AgentType::Individual))
                .count();
            if ui
                .add(
                    egui::Slider::new(&mut self.simulation.config.num_individuals, 0..=100)
                        .text("Num. Individuals"),
                )
                .changed()
            {
                self.handle_agent_count_change(
                    self.simulation.config.num_individuals,
                    current_individuals,
                    AgentType::Individual,
                );
            }

            let current_bots = self
                .simulation
                .agents
                .iter()
                .filter(|a| matches!(a.get_type(), AgentType::Bot))
                .count();
            if ui
                .add(
                    egui::Slider::new(&mut self.simulation.config.num_bots, 0..=100)
                        .text("Num. Bots"),
                )
                .changed()
            {
                self.handle_agent_count_change(
                    self.simulation.config.num_bots,
                    current_bots,
                    AgentType::Bot,
                );
            }

            let current_orgs = self
                .simulation
                .agents
                .iter()
                .filter(|a| matches!(a.get_type(), AgentType::Organisation))
                .count();
            if ui
                .add(
                    egui::Slider::new(&mut self.simulation.config.num_organisations, 0..=100)
                        .text("Num. Organisations"),
                )
                .changed()
            {
                self.handle_agent_count_change(
                    self.simulation.config.num_organisations,
                    current_orgs,
                    AgentType::Organisation,
                );
            }

            ui.add(
                egui::Slider::new(&mut self.simulation.config.base_content_length, 0..=100)
                    .text("Base content length"),
            );
            ui.add(
                egui::Slider::new(&mut self.simulation.config.max_content_length, 0..=200)
                    .text("Max content length"),
            );
            ui.add(
                egui::Slider::new(&mut self.simulation.config.bot_creation_ticks, 0..=20)
                    .text("Bot create time"),
            );

            ui.add(
                egui::Slider::new(&mut self.simulation.config.diversity_weight, 0.0..=1.0)
                    .text("Diversity Weight"),
            );
            ui.add(
                egui::Slider::new(&mut self.simulation.config.recency_weight, 0.0..=1.0)
                    .text("Recency Weight"),
            );
            ui.add(
                egui::Slider::new(&mut self.simulation.config.engagement_weight, 0.0..=1.0)
                    .text("Engagement Weight"),
            );
            ui.add(
                egui::Slider::new(&mut self.simulation.config.interest_decay_rate, 0.0..=1.0)
                    .text("Interest Decay Rate"),
            );
            ui.add(
                egui::Slider::new(&mut self.simulation.config.tick_rate_ms, 0..=10_000)
                    .text("Tick Rate (ms)"),
            );

            if ui.button("Reset Simulation").clicked() {
                self.simulation = Simulation::new(SimulationConfig::default());

                self.open_agent_windows.clear(); // Clear any open windows
            }
        });

        egui::TopBottomPanel::top("Agents").show(ctx, |ui| {
            ui.set_min_height(ctx.available_rect().height() / 2.0);
            ui.set_max_height(ctx.available_rect().height() / 2.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for agent in &self.simulation.agents {
                        let agent_id = *agent.id();
                        ui.allocate_ui(Vec2 { x: 150.0, y: 180.0 }, |ui| {
                            ui.vertical(|ui| {
                                ui.add_space(10.0);
                                // Top section for icon
                                ui.vertical_centered(|ui| {
                                    let response = match agent.get_type() {
                                        AgentType::Bot => draw_bot_icon(ui),
                                        AgentType::Organisation => draw_org_icon(ui),
                                        AgentType::Individual => draw_person_icon(ui),
                                    };
                                    if response.clicked()
                                        && !self.open_agent_windows.contains(&agent_id)
                                    {
                                        self.open_agent_windows.push(agent_id);
                                    }
                                });

                                ui.with_layout(
                                    egui::Layout::bottom_up(egui::Align::Center),
                                    |ui| {
                                        match &agent.activity() {
                                            Activity::Creating(state) => {
                                                let progress = state.ticks_spent as f32
                                                    / state.ticks_required as f32;
                                                ui.add(
                                                    egui::ProgressBar::new(progress)
                                                        .text("Creating"),
                                                );
                                            }
                                            Activity::Consuming(state) => {
                                                let progress = state.ticks_spent as f32
                                                    / state.ticks_required as f32;
                                                ui.add(
                                                    egui::ProgressBar::new(progress)
                                                        .text("Consuming"),
                                                );
                                            }
                                            Activity::Offline => {
                                                ui.add(egui::ProgressBar::new(0.0).text("Offline"));
                                            }
                                        }
                                        ui.add_space(10.0);
                                    },
                                );
                            });
                        });
                    }
                });
            });
        });

        self.open_agent_windows.retain(|&agent_id| {
            if let Some(agent) = self.simulation.agents.iter().find(|a| *a.id() == agent_id) {
                let mut window_open = true;
                egui::Window::new(format!("Agent {}", agent_id))
                    .open(&mut window_open)
                    .show(ctx, |ui| {
                        ui.label(format!("Type: {:?}", agent.get_type()));
                        ui.separator();
                        ui.label("Interests:");
                        for (interest, weight) in agent.interests() {
                            ui.label(format!(
                                "{:?}: {}%",
                                String::from(interest),
                                *weight as f32 / 1.0
                            ));
                        }
                        ui.separator();
                        ui.label("Activity:");
                        ui.label(match &agent.activity() {
                            Activity::Creating(state) => format!(
                                "Creating ({}%)",
                                (state.ticks_spent as f32 / state.ticks_required as f32 * 100.0)
                                    as i32
                            ),
                            Activity::Consuming(state) => format!(
                                "Consuming ({}%)",
                                (state.ticks_spent as f32 / state.ticks_required as f32 * 100.0)
                                    as i32
                            ),
                            Activity::Offline => "Offline".to_string(),
                        });
                    });
                window_open
            } else {
                false
            }
        });

        egui::TopBottomPanel::bottom("Content Pool").show(ctx, |ui| {
            ui.heading("Content Pool");
            ui.set_min_height(ctx.available_rect().height());
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for content in &self.simulation.engine.content_pool {
                        ui.allocate_ui(Vec2 { x: 150.0, y: 150.0 }, |ui| {
                            ui.group(|ui| {
                                ui.label(format!("Content {}", content.id));
                                ui.label(format!("Creator: {}", content.creator_id));
                                ui.label(format!("Time: {}", content.timestamp));
                                ui.label(format!("Length: {}", content.length));
                                ui.label(format!("Tags: {}", content.tags.join(", ")));
                                ui.label(format!("Engagement: {:.2}", content.engagement_score));
                            });
                        });
                    }
                });
            });
        });
    }

    fn handle_agent_count_change(
        &mut self,
        new_value: usize,
        current_count: usize,
        agent_type: AgentType,
    ) {
        match new_value.cmp(&current_count) {
            std::cmp::Ordering::Greater => {
                for _ in 0..(new_value - current_count) {
                    self.simulation.add_agent(agent_type);
                }
            }
            std::cmp::Ordering::Less => {
                for _ in 0..(current_count - new_value) {
                    self.simulation.remove_agent(agent_type);
                }
            }
            std::cmp::Ordering::Equal => {}
        }
    }
}

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Social Media Simulation",
        options,
        Box::new(|_cc| Ok(Box::new(SimulationApp::default()))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|_cc| Ok(Box::new(SimulationApp::default()))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}

fn draw_bot_icon(ui: &mut egui::Ui) -> egui::Response {
    let rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(rect, egui::Sense::click());
    let painter = ui.painter();

    let center = rect.center();
    let size = rect.height().min(rect.width()) * 0.8;

    // Head
    painter.circle_filled(center, size / 2.0, egui::Color32::GRAY);

    // Antenna
    painter.line_segment(
        [
            center + Vec2::new(-size / 4.0, -size / 2.0),
            center + Vec2::new(-size / 4.0, -size / 1.5),
        ],
        egui::Stroke::new(2.0, egui::Color32::DARK_GRAY),
    );
    painter.circle_filled(
        center + Vec2::new(-size / 4.0, -size / 1.5),
        size / 10.0,
        egui::Color32::DARK_GRAY,
    );

    // Eyes
    painter.circle_filled(
        center + Vec2::new(-size / 4.0, -size / 6.0),
        size / 8.0,
        egui::Color32::LIGHT_BLUE,
    );
    painter.circle_filled(
        center + Vec2::new(size / 4.0, -size / 6.0),
        size / 8.0,
        egui::Color32::LIGHT_BLUE,
    );

    response
}

fn draw_org_icon(ui: &mut egui::Ui) -> egui::Response {
    let rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(rect, egui::Sense::click());
    let painter = ui.painter();

    let center = rect.center();
    let size = rect.height().min(rect.width()) * 0.8;

    painter.rect_filled(
        egui::Rect::from_center_size(center, Vec2::new(size, size)),
        0.0,
        egui::Color32::GRAY,
    );

    for x in [-size / 4.0, size / 4.0] {
        for y in [-size / 4.0, size / 4.0] {
            painter.rect_filled(
                egui::Rect::from_center_size(
                    center + Vec2::new(x, y),
                    Vec2::new(size / 4.0, size / 4.0),
                ),
                0.0,
                egui::Color32::LIGHT_BLUE,
            );
        }
    }

    response
}

fn draw_person_icon(ui: &mut egui::Ui) -> egui::Response {
    let rect = ui.available_rect_before_wrap();
    let response = ui.allocate_rect(rect, egui::Sense::click());
    let painter = ui.painter();

    let center = rect.center();
    let size = rect.height().min(rect.width()) * 0.8;

    // Head
    painter.circle_filled(
        center + Vec2::new(0.0, -size / 3.0),
        size / 4.0,
        egui::Color32::GRAY,
    );

    // Body
    painter.line_segment(
        [
            center + Vec2::new(0.0, -size / 6.0),
            center + Vec2::new(0.0, size / 3.0),
        ],
        egui::Stroke::new(2.0, egui::Color32::GRAY),
    );

    // Arms
    painter.line_segment(
        [
            center + Vec2::new(-size / 3.0, 0.0),
            center + Vec2::new(size / 3.0, 0.0),
        ],
        egui::Stroke::new(2.0, egui::Color32::GRAY),
    );

    // Legs
    painter.line_segment(
        [
            center + Vec2::new(0.0, size / 3.0),
            center + Vec2::new(-size / 4.0, size / 2.0),
        ],
        egui::Stroke::new(2.0, egui::Color32::GRAY),
    );
    painter.line_segment(
        [
            center + Vec2::new(0.0, size / 3.0),
            center + Vec2::new(size / 4.0, size / 2.0),
        ],
        egui::Stroke::new(2.0, egui::Color32::GRAY),
    );

    response
}
