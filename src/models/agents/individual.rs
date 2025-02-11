use crate::{models::SimulationConfig, Content, RecommendationEngine};
use rand::{random, Rng, RngCore};
use std::collections::HashMap;

use super::{Activity, Agent, AgentCore, AgentType, ContentConsumptionState, ContentCreationState};

#[derive(Debug, Clone)]
pub struct Individual {
    pub core: AgentCore,
    pub next_post_likelihood: f32, // 1 = will definitely keep scrolling, 0 = will stop scrolling now
    pub attention_span: f32,       // 1 = will read any length of post, 0 = just reads headlines
    pub preferred_creators: HashMap<usize, f32>, // The ID of the creator, and a f32 weight
    pub viewed_content: Vec<usize>,
    pub bias_factor: f32,
    pub consume_speed: f32,
}

impl Agent for Individual {
    fn tick(&mut self, engine: &RecommendationEngine) -> Option<Content> {
        self.decay_preferences();

        match self.core.activity {
            Activity::Offline => {
                // Use next post likelihood for whether to come back online
                if random::<f32>() < self.next_post_likelihood {
                    self.core.activity = self.create_or_consume(engine);
                }
            }

            Activity::Creating(ref mut creation_state) => {
                creation_state.ticks_spent += 1;
                if creation_state.ticks_spent >= creation_state.ticks_required {
                    let content = self.core.generate_content(engine);
                    self.core.activity = self.create_or_consume(engine);
                    return Some(content);
                }
            }

            Activity::Consuming(ref mut consumption_state) => {
                consumption_state.ticks_spent += 1;
                if consumption_state.ticks_spent >= consumption_state.ticks_required {
                    let content_id = consumption_state.content_id;
                    let creator_id = consumption_state.creator_id;

                    self.viewed_content.push(consumption_state.creator_id);

                    self.update_interests(engine, content_id);
                    self.update_preferred_creators(creator_id);

                    self.core.activity = self.create_or_consume(engine)
                }
            }
        }
        None
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> AgentType {
        AgentType::Individual
    }

    fn interests(&self) -> &HashMap<String, f32> {
        &self.core.interests
    }

    fn preferred_creators(&self) -> Option<&HashMap<usize, f32>> {
        Some(&self.preferred_creators)
    }

    fn activity(&self) -> &Activity {
        &self.core.activity
    }
}

impl Individual {
    pub fn new(id: usize, config: &SimulationConfig, engine: &RecommendationEngine) -> Self {
        let mut interests = HashMap::new();

        for _ in 0..config.starting_tags.bot {
            let tag = config.sample_tags[rand::thread_rng().gen_range(0..config.sample_tags.len())];
            interests.insert(tag.to_string(), 1.0);
        }

        let agent_tags: Vec<String> = interests.keys().cloned().collect();
        let interest_vector = engine.vectorize_tags(&agent_tags);

        Self {
            core: AgentCore {
                id,
                content_creation_frequency: random(),
                created_content: Vec::new(),
                create_speed: random(),
                activity: Activity::Offline,
                interests,
                interest_vector,
            },
            next_post_likelihood: random(),
            attention_span: random(),
            preferred_creators: HashMap::new(),
            viewed_content: Vec::new(),
            bias_factor: random(),
            consume_speed: random(),
        }
    }
    fn should_generate_content(&self) -> bool {
        let roll = random::<f32>();
        let should_generate = random::<f32>() < self.core.content_creation_frequency;
        println!(
            "Generation roll: {:?}, Frequency: {:?}",
            roll, self.core.content_creation_frequency
        );
        return should_generate;
    }

    fn create_or_consume(&self, engine: &RecommendationEngine) -> Activity {
        if self.should_generate_content() {
            let ticks_required = (random::<f32>() * 30.0 / self.core.create_speed) as i32;
            return Activity::Creating(ContentCreationState {
                content_id: rand::thread_rng().next_u32() as usize,
                ticks_spent: 0,
                ticks_required,
            });
        } else {
            if let Some(content) = engine
                .get_recommendations(self, 1, chrono::Utc::now().timestamp())
                .first()
            {
                let ticks_required = (content.length as f32 / self.consume_speed) as i32;
                return Activity::Consuming(ContentConsumptionState {
                    content_id: content.id,
                    creator_id: content.creator_id,
                    ticks_spent: 0,
                    ticks_required,
                });
            } else {
                // TODO: No content was found so we go offline?
                return Activity::Offline;
            }
        }
    }

    fn update_interests(&mut self, engine: &RecommendationEngine, content_id: usize) {
        if let Some(content) = engine.get_content_by_id(content_id) {
            for tag in &content.tags {
                let interest = self.core.interests.entry(tag.to_string()).or_insert(0.0);
                *interest += 0.1;
            }
        }
    }

    fn update_preferred_creators(&mut self, creator_id: usize) {
        let weight = self.preferred_creators.entry(creator_id).or_insert(0.0);
        *weight += 0.1;
    }

    fn decay_preferences(&mut self) {
        for interest in self.core.interests.values_mut() {
            *interest *= 0.99;
        }

        for weight in self.preferred_creators.values_mut() {
            *weight *= 0.99;
        }
    }
}
