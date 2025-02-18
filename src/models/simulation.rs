use crate::RecommendationEngine;
use chrono::{DateTime, Utc};

use super::{AgentType, Individual};

#[derive(Debug, Clone)]
pub struct SimulationConfig {
    pub num_individuals: usize,
    pub num_bots: usize,
    pub num_organisations: usize,
    pub max_post_length: i32,
    pub max_comment_length: i32,
    pub bot_creation_ticks: i32,
    pub sample_tags: Vec<String>,
    pub starting_tags: StartingTags,
    pub base_content_length: i32,
    pub diversity_weight: f32,
    pub recency_weight: f32,
    pub engagement_weight: f32,
    pub tick_rate_ms: i32,
    pub interest_decay_rate: f32,
    pub min_content_tags: usize,
    pub max_content_tags: usize,
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
            num_individuals: 3,
            num_bots: 2,
            num_organisations: 2,
            max_post_length: 60,
            max_comment_length: 10,
            bot_creation_ticks: 4,
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
            diversity_weight: 0.2,
            recency_weight: 0.2,
            engagement_weight: 0.2,
            tick_rate_ms: 100,
            interest_decay_rate: 0.0,
            min_content_tags: 1,
            max_content_tags: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Simulation {
    pub config: SimulationConfig,
    pub engine: RecommendationEngine,
    pub individuals: Vec<Individual>,
    pub current_tick: DateTime<Utc>,
    pub last_tick: DateTime<Utc>,
}

impl Simulation {
    pub fn new(config: SimulationConfig) -> Self {
        let mut engine = RecommendationEngine::new();
        let mut individuals: Vec<Individual> = Vec::new();

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
            individuals.push(Individual::new());
        }

        let now = Utc::now();

        Simulation {
            config,
            engine,
            individuals,
            current_tick: now,
            last_tick: now,
        }
    }

    pub fn tick(&mut self) {
        self.current_tick = Utc::now();
        let elapsed = (self.current_tick - self.last_tick).num_milliseconds();

        if elapsed >= self.config.tick_rate_ms as i64 {
            let individuals = std::mem::take(&mut self.individuals);

            self.last_tick = self.current_tick;

            self.individuals = individuals
                .into_iter()
                .map(|individual| individual.tick(&self.engine))
                .collect();
        }
    }

    pub fn add_agent(&mut self, agent_type: AgentType) {
        match agent_type {
            AgentType::Individual => {
                self.add_individual();
            }
            _ => unimplemented!(),
        }
    }

    fn add_individual(&mut self) {
        self.individuals.push(Individual::new())
    }

    pub fn remove_agent(&mut self, agent_type: AgentType) {
        match agent_type {
            AgentType::Individual => {
                self.remove_individual();
            }
            _ => unimplemented!(),
        }
    }

    fn remove_individual(&mut self) {
        self.individuals.pop();
    }
}
