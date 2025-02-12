use crate::{models::SimulationConfig, Content, RecommendationEngine};
use rand::{random, Rng, RngCore};
use std::collections::HashMap;

use super::{Activity, Agent, AgentCore, AgentType, ContentCreationState};

#[derive(Debug, Clone)]
pub struct Bot {
    pub core: AgentCore,
}

impl Agent for Bot {
    fn tick(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
    ) -> Option<Content> {
        match self.core.activity {
            Activity::Creating(ref mut creation_state) => {
                creation_state.ticks_spent += 1;
                if creation_state.ticks_spent >= creation_state.ticks_required {
                    let content = self.core.generate_content(engine, config);
                    self.core.activity = self.new_creation(config);
                    return Some(content);
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> AgentType {
        AgentType::Bot
    }

    fn interests(&self) -> &HashMap<String, f32> {
        &self.core.interests
    }

    fn activity(&self) -> &Activity {
        &self.core.activity
    }

    fn id(&self) -> &usize {
        &self.core.id
    }
}

impl Bot {
    pub fn new(id: usize, config: &SimulationConfig, engine: &RecommendationEngine) -> Self {
        let create_speed = 1.0;
        let ticks_required = 1;
        let mut interests = HashMap::new();

        for _ in 0..config.starting_tags.bot {
            let tag =
                &config.sample_tags[rand::thread_rng().gen_range(0..config.sample_tags.len())];
            interests.insert(tag.to_string(), 1.0);
        }

        let agent_tags: Vec<String> = interests.keys().cloned().collect();
        let interest_vector = engine.vectorize_tags(&agent_tags);

        Self {
            core: AgentCore {
                id,
                content_creation_frequency: random(),
                created_content: Vec::new(),
                create_speed,
                activity: Activity::Creating(ContentCreationState {
                    content_id: rand::thread_rng().next_u32() as usize,
                    ticks_spent: 0,
                    ticks_required,
                }),
                interests,
                interest_vector,
            },
        }
    }

    fn new_creation(&self, config: &SimulationConfig) -> Activity {
        let ticks_required = config.bot_creation_ticks;
        return Activity::Creating(ContentCreationState {
            content_id: rand::thread_rng().next_u32() as usize,
            ticks_spent: 0,
            ticks_required,
        });
    }
}
