use crate::{models::AgentType, RecommendationEngine};

use super::{Agent, Bot, Individual, Organisation};

#[derive(Debug, Clone)]
pub struct SimulationConfig<'a> {
    pub num_individuals: usize,
    pub num_bots: usize,
    pub num_organisations: usize,
    pub max_content_length: i32,
    pub creator_preference_increment: f32,
    pub long_content_interest_multiplier: f32,
    pub bot_creation_ticks: i32,
    pub sample_tags: Vec<&'a str>,
    pub starting_tags: StartingTags,
}

#[derive(Debug, Clone)]
pub struct StartingTags {
    pub individual: usize,
    pub bot: usize,
    pub organisation: usize,
}

impl<'a> Default for SimulationConfig<'a> {
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
                "politics",
                "technology",
                "science",
                "entertainment",
                "sports",
                "health",
                "education",
                "business",
            ],
            starting_tags: StartingTags {
                individual: 3,
                bot: 3,
                organisation: 3,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Simulation<'a> {
    pub config: SimulationConfig<'a>,
    pub engine: RecommendationEngine,
    pub agents: Vec<Box<dyn Agent>>,
    pub current_tick: i64,
}

impl<'a> Simulation<'a> {
    pub fn new(config: SimulationConfig<'a>) -> Self {
        let mut engine = RecommendationEngine::new();
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
            current_tick: 0,
        }
    }

    pub fn tick(&mut self) {
        self.current_tick += 1;
        // let current_time = chrono::Utc::now().timestamp();

        let mut new_content = Vec::new();

        for agent in self.agents.iter_mut() {
            let content = agent.tick(&self.engine, &self.config);
            match content {
                Some(content) => new_content.push(content),
                _ => (),
            }
        }

        self.engine.content_pool.extend(new_content);
    }

    pub fn run(&mut self, num_ticks: i64) {
        for _ in 0..num_ticks {
            self.tick();

            // Print statistics every 10 ticks
            if self.current_tick % 10 == 0 {
                self.print_statistics();
            }
        }
    }

    pub fn print_statistics(&self) {
        println!("\nTick {}", self.current_tick);
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
                    println!("Preferred creators: {:?}", agent.preferred_creators());
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
