use eframe::egui;
use egui::Vec2;
use social_media_sandbox::{
    models::{Activity, SimulationConfig},
    Simulation,
};
pub struct SimulationApp {
    running: bool,
    simulation: Simulation,
}

impl Default for SimulationApp {
    fn default() -> Self {
        let default_config = SimulationConfig::default();
        Self {
            running: false,
            simulation: Simulation::new(default_config),
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

            ui.add(
                egui::DragValue::new(&mut self.simulation.config.num_individuals)
                    .range(0..=100)
                    .suffix(" individuals"),
            );
            ui.add(
                egui::DragValue::new(&mut self.simulation.config.num_bots)
                    .range(0..=100)
                    .suffix(" bots"),
            );
            ui.add(
                egui::DragValue::new(&mut self.simulation.config.num_organisations)
                    .range(0..=100)
                    .suffix(" organisations"),
            );
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
                egui::Slider::new(&mut self.simulation.config.tick_rate_ms, 0..=10_000)
                    .text("Tick Rate (ms)"),
            );
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Agents");
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    for agent in &self.simulation.agents {
                        ui.allocate_ui(Vec2 { x: 150.0, y: 150.0 }, |ui| {
                            ui.group(|ui| {
                                ui.label(format!("Agent {:?}", agent.id()));
                                ui.label(format!("Type: {:?}", agent.get_type()));
                                for (interest, weight) in agent.interests() {
                                    ui.label(format!(
                                        "{:?} {}%",
                                        String::from(interest),
                                        *weight as f32 / 1.0
                                    ));
                                }
                                ui.label(match &agent.activity() {
                                    Activity::Creating(state) => format!(
                                        "Creating ({}%)",
                                        (state.ticks_spent as f32 / state.ticks_required as f32
                                            * 100.0) as i32
                                    ),
                                    Activity::Consuming(state) => format!(
                                        "Consuming ({}%)",
                                        (state.ticks_spent as f32 / state.ticks_required as f32
                                            * 100.0) as i32
                                    ),
                                    Activity::Offline => "Offline".to_string(),
                                });
                            });
                        });
                    }
                })
            })
        });

        egui::TopBottomPanel::bottom("Content Pool").show(ctx, |ui| {
            ui.heading("Content Pool");
            ui.set_min_height(ctx.available_rect().height() / 2.0);
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
