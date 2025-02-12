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
    pub bias_factor: f32, // 1 = consumed content has full effect on interests, 0 = interests never change
    pub decay_factor: f32, // 1 = full interest decay speed of 1% per tick, 0 = no decay
    pub consume_speed: f32, // 1 = fastest consume speed, 0 = will never finish
    pub last_interaction_times: HashMap<String, i64>,
}

impl Agent for Individual {
    fn tick(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
    ) -> Option<Content> {
        match self.core.activity {
            Activity::Offline => {
                // Use next post likelihood for whether to come back online
                if random::<f32>() < self.next_post_likelihood {
                    self.core.activity = self.create_or_consume(engine, config);
                }
            }

            Activity::Creating(ref mut creation_state) => {
                creation_state.ticks_spent += 1;
                if creation_state.ticks_spent >= creation_state.ticks_required {
                    let content = self.core.generate_content(engine, config);
                    self.core.activity = self.create_or_consume(engine, config);
                    return Some(content);
                }
            }

            Activity::Consuming(ref mut consumption_state) => {
                consumption_state.ticks_spent += 1;
                if consumption_state.ticks_spent >= consumption_state.ticks_required
                    || random::<f32>() > self.attention_span
                {
                    let content_id = consumption_state.content_id;
                    let creator_id = consumption_state.creator_id;
                    let ticks_spent = consumption_state.ticks_spent;

                    self.viewed_content.push(consumption_state.creator_id);

                    self.update_interests(engine, content_id, ticks_spent);
                    self.update_preferred_creators(creator_id, ticks_spent);

                    self.core.activity = self.create_or_consume(engine, config)
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

    fn id(&self) -> &usize {
        &self.core.id
    }
}

impl Individual {
    pub fn new(id: usize, config: &SimulationConfig, engine: &RecommendationEngine) -> Self {
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
                content_creation_frequency: random::<f32>().min(0.3),
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
            decay_factor: random(),
            last_interaction_times: HashMap::new(),
        }
    }

    fn should_generate_content(&self) -> bool {
        return random::<f32>() < self.core.content_creation_frequency;
    }

    fn create_or_consume(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
    ) -> Activity {
        self.decay_interests(config);
        if self.should_generate_content() {
            let ticks_required = (random::<f32>()
                * config.base_content_length as f32
                * self.core.create_speed) as i32;
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

    fn update_interests(
        &mut self,
        engine: &RecommendationEngine,
        content_id: usize,
        ticks_spent: i32,
    ) {
        if let Some(content) = engine.get_content_by_id(content_id) {
            let now = chrono::Utc::now().timestamp();
            let experience_factor = 1.0 / (1.0 + self.viewed_content.len() as f32 * 0.1);

            for tag in &content.tags {
                let is_new_interest = !self.core.interests.contains_key(tag);
                let last_interaction = self.last_interaction_times.get(tag).unwrap_or(&0);
                let recency_factor = (-0.001 * (now - last_interaction) as f32).exp();

                let interest = self.core.interests.entry(tag.to_string()).or_insert(0.0);
                let update_magnitude = if is_new_interest { 0.2 } else { 0.05 };

                *interest += update_magnitude
                    * self.bias_factor
                    * experience_factor
                    * (1.0 + recency_factor);
                self.last_interaction_times.insert(tag.to_string(), now);
            }
            self.viewed_content.push(content_id)
        }
    }

    fn update_preferred_creators(&mut self, creator_id: usize, ticks_spent: i32) {
        let weight = self.preferred_creators.entry(creator_id).or_insert(0.0);
        *weight += 0.05 * self.bias_factor * ticks_spent as f32;
    }

    fn decay_interests(&mut self, config: &SimulationConfig) {
        for interest in self.core.interests.values_mut() {
            *interest *= (1.0 - config.interest_decay_rate) * (1.0 - self.decay_factor);
        }

        for weight in self.preferred_creators.values_mut() {
            *weight *= (1.0 - config.interest_decay_rate) * (1.0 - self.decay_factor);
        }
    }
}
