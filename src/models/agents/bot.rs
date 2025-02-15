use super::{Agent, AgentCore, AgentState, AgentType};
use crate::{
    models::{InterestProfile, SimulationConfig, Topic},
    RecommendationEngine,
};
use rand::{random, Rng, RngCore};

#[derive(Debug, Clone)]
pub struct Bot {
    pub core: AgentCore,
}

impl Agent for Bot {
    fn tick(&mut self, engine: &mut RecommendationEngine, config: &SimulationConfig) {
        // Extract data from current creation state
        let new_state = match &self.core.state {
            AgentState::CreatingPost {
                post_id,
                ticks_spent,
                ticks_required,
            } => self.proceed_from_creating_post(
                config,
                engine,
                *post_id,
                *ticks_spent,
                *ticks_required,
            ),
            _ => {
                // Bot should always be creating, so initialize creation if in any other state
                self.start_creating_post(config)
            }
        };

        self.core.state = new_state;
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> AgentType {
        AgentType::Bot
    }

    fn interest_profile(&self) -> &InterestProfile {
        &self.core.interest_profile
    }

    fn state(&self) -> &AgentState {
        &self.core.state
    }

    fn id(&self) -> &usize {
        &self.core.id
    }
}

impl Bot {
    pub fn new(id: usize, config: &SimulationConfig) -> Self {
        // Initialize interest profile
        let mut interest_profile = InterestProfile::new(100);

        // Add random starting interests
        for _ in 0..config.starting_tags.bot {
            let tag =
                &config.sample_tags[rand::thread_rng().gen_range(0..config.sample_tags.len())];
            interest_profile.interests.insert(
                tag.clone(),
                Topic {
                    weighted_interest: 1.0,                 // Will be normalized
                    agreement: random::<f32>() * 2.0 - 1.0, // Random agreement between -1 and 1
                },
            );
        }

        interest_profile.normalise_weights();

        Self {
            core: AgentCore {
                id,
                content_creation_frequency: 1.0, // Bots always create
                created_content: Vec::new(),
                create_speed: 1.0, // Bots create at full speed
                state: AgentState::CreatingPost {
                    post_id: rand::thread_rng().next_u32() as usize,
                    ticks_spent: 0,
                    ticks_required: config.bot_creation_ticks,
                },
                interest_profile,
            },
        }
    }

    fn proceed_from_creating_post(
        &mut self,
        config: &SimulationConfig,
        engine: &mut RecommendationEngine,
        post_id: usize,
        ticks_spent: i32,
        ticks_required: i32,
    ) -> AgentState {
        let new_ticks_spent = ticks_spent + 1;

        if new_ticks_spent >= ticks_required {
            // Generate content and start new creation
            let content = self.core.generate_content(config);
            self.core.created_content.push(content.id);

            engine.create_post(content);

            return self.start_creating_post(config);
        } else {
            // Continue current creation
            return AgentState::CreatingPost {
                post_id,
                ticks_spent: new_ticks_spent,
                ticks_required,
            };
        }
    }

    fn start_creating_post(&self, config: &SimulationConfig) -> AgentState {
        AgentState::CreatingPost {
            post_id: rand::thread_rng().next_u32() as usize,
            ticks_spent: 0,
            ticks_required: config.bot_creation_ticks,
        }
    }
}
