use crate::{models::AgentType, RecommendationEngine};
use nalgebra::DVector;
use std::time::{Duration, Instant};

use super::{Agent, Bot, Content, Individual, Organisation};

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub num_individuals: usize,
    pub num_bots: usize,
    pub num_organisations: usize,
    pub max_content_length: i32,
    pub creator_preference_increment: f32,
    pub long_content_interest_multiplier: f32,
    pub bot_creation_ticks: i32,
    pub sample_tags: Vec<String>,
    pub starting_tags: StartingTags,
    pub base_content_length: i32,
    pub starting_content: Vec<Content>,
    pub diversity_weight: f32,
    pub recency_weight: f32,
    pub engagement_weight: f32,
    pub tick_rate_ms: i32,
}

#[derive(Debug, Clone)]
pub struct StartingTags {
    pub individual: usize,
    pub bot: usize,
    pub organisation: usize,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        SimulationConfig {
            num_individuals: 1,
            num_bots: 5,
            num_organisations: 5,
            max_content_length: 60,
            creator_preference_increment: 0.1,
            long_content_interest_multiplier: 1.5,
            bot_creation_ticks: 2,
            sample_tags: vec![
                "politics".to_string(),
                "technology".to_string(),
                "science".to_string(),
                "entertainment".to_string(),
                "sports".to_string(),
                "health".to_string(),
                "education".to_string(),
                "business".to_string(),
            ],
            starting_tags: StartingTags {
                individual: 3,
                bot: 3,
                organisation: 3,
            },
            base_content_length: 20,
            starting_content: vec![Content {
                id: 99999,
                creator_id: 99999,
                timestamp: chrono::Utc::now().timestamp(),
                tags: vec!["initial".to_string()],
                engagement_score: 0.0,
                vector_representation: DVector::zeros(100),
                length: 10,
            }],
            diversity_weight: 0.2,
            recency_weight: 0.2,
            engagement_weight: 0.2,
            tick_rate_ms: 2_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Simulation {
    pub config: SimulationConfig,
    pub engine: RecommendationEngine,
    pub agents: Vec<Box<dyn Agent>>,
    pub current_tick: Instant,
    pub last_tick: Instant,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let mut engine = RecommendationEngine::new(&config);
        let mut agents: Vec<Box<dyn Agent>> = Vec::new();
        let mut id_counter = 0;

        let sample_tags = vec![
            "politics",
            "technology",
            "science",
            "entertainment",
            "sports",
            "health",
            "education",
            "business",
        ];

        for (i, tag) in sample_tags.iter().enumerate() {
            engine.tag_to_index.insert(tag.to_string(), i);
            engine.index_to_tag.insert(i, tag.to_string());
        }

        for _ in 0..config.num_individuals {
            let agent = Individual::new(id_counter, &config, &engine);
            agents.push(Box::new(agent));
            id_counter += 1;
        }

        for _ in 0..config.num_bots {
            let agent = Bot::new(id_counter, &config, &engine);
            agents.push(Box::new(agent));
            id_counter += 1;
        }

        for _ in 0..config.num_organisations {
            let agent = Organisation::new(id_counter, &config, &engine);
            agents.push(Box::new(agent));
            id_counter += 1;
        }

        Simulation {
            config,
            engine,
            agents,
            current_tick: Instant::now(),
            last_tick: Instant::now(),
        }
    }

    pub fn tick(&mut self) {
        self.current_tick = Instant::now();
        let elapsed = self.current_tick.duration_since(self.last_tick);

        if elapsed >= Duration::from_millis(self.config.tick_rate_ms as u64) {
            self.last_tick = self.current_tick;

            let mut new_content = Vec::new();

            for agent in self.agents.iter_mut() {
                if let Some(content) = agent.tick(&self.engine, &self.config) {
                    new_content.push(content);
                }
            }

            self.engine.content_pool.extend(new_content);
        }
    }

    pub fn print_statistics(&self) {
        println!("\nTick {:?}", self.current_tick);
        println!("Content pool size: {}", self.engine.content_pool.len());

        // Print some engagement statistics
        let avg_engagement: f32 = self
            .engine
            .content_pool
            .iter()
            .map(|c| c.engagement_score)
            .sum::<f32>()
            / self.engine.content_pool.len() as f32;

        println!("Average engagement score: {:.2}", avg_engagement);

        // Print statistics for different agent types
        let (individual_count, bot_count, organisation_count) = self.agents.iter().fold(
            (0, 0, 0),
            |(individual, bot, organisation), agent| match agent.get_type() {
                AgentType::Individual => (individual + 1, bot, organisation),
                AgentType::Bot => (individual, bot + 1, organisation),
                AgentType::Organisation => (individual, bot, organisation + 1),
            },
        );

        println!("Number of Individual agents: {}", individual_count);
        println!("Number of Bot agents: {}", bot_count);
        println!("Number of Organisation agents: {}", organisation_count);

        // Print some example agent states
        for agent in &self.agents {
            match agent.get_type() {
                AgentType::Individual => {
                    println!("\nIndividual State:");
                    println!("\nActivity: {:?}", agent.activity());
                    println!("Interests: {:?}", agent.interests());
                    println!(
                        "Preferred creators: {:?}",
                        agent.preferred_creators().unwrap()
                    );
                } // AgentType::Organisation => {
                //     println!("\nOrganisation State:");
                //     println!("Interests: {:?}", agent.interests());
                // }
                // AgentType::Bot => {
                //     println!("\nBot State:");
                //     println!("Interests: {:?}", agent.interests());
                // }
                _ => (),
            }
        }
    }
}
