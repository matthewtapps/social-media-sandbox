use social_media_sandbox::{models::SimulationConfig, Simulation};

fn main() {
    let config = SimulationConfig::default();
    let mut simulation = Simulation::new(config);
    simulation.run(100);
}
