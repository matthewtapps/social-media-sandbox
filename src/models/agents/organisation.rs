use super::{Agent, AgentCore, AgentState, AgentType};
use crate::{
    models::{InterestProfile, SimulationConfig, Topic},
    Post, RecommendationEngine,
};
use rand::{random, Rng, RngCore};

#[derive(Debug, Clone)]
pub struct Organisation {
    pub core: AgentCore,
}

impl Agent for Organisation {
    fn tick(&mut self, _engine: &RecommendationEngine, config: &SimulationConfig) -> Option<Post> {
        let (content_option, new_state) = match &self.core.state {
            AgentState::CreatingPost {
                post_id,
                ticks_spent,
                ticks_required,
            } => self.proceed_from_creating_post(config, *post_id, *ticks_spent, *ticks_required),
            _ => {
                // Organizations, like bots, should always be creating
                (None, self.start_creating_post())
            }
        };

        self.core.state = new_state;
        content_option
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> AgentType {
        AgentType::Organisation
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

impl Organisation {
    pub fn new(id: usize, config: &SimulationConfig) -> Self {
        let mut interest_profile = InterestProfile::new(100);

        // Organizations are focused - they typically have strong opinions about few topics
        let tag = &config.sample_tags[rand::thread_rng().gen_range(0..config.sample_tags.len())];

        // Organizations tend to have strong opinions (agreements closer to +1 or -1)
        let agreement = if random::<f32>() > 0.5 {
            0.7 + random::<f32>() * 0.3 // Strong positive (0.7 to 1.0)
        } else {
            -1.0 + random::<f32>() * 0.3 // Strong negative (-1.0 to -0.7)
        };

        interest_profile.interests.insert(
            tag.clone(),
            Topic {
                weighted_interest: 1.0, // Will be normalized
                agreement,
            },
        );

        interest_profile.normalise_weights();

        Self {
            core: AgentCore {
                id,
                content_creation_frequency: 1.0, // Organizations always create
                created_content: Vec::new(),
                create_speed: 1.0,
                state: AgentState::CreatingPost {
                    post_id: rand::thread_rng().next_u32() as usize,
                    ticks_spent: 0,
                    ticks_required: Self::calculate_post_ticks(),
                },
                interest_profile,
            },
        }
    }

    fn proceed_from_creating_post(
        &mut self,
        config: &SimulationConfig,
        post_id: usize,
        ticks_spent: i32,
        ticks_required: i32,
    ) -> (Option<Post>, AgentState) {
        let new_ticks_spent = ticks_spent + 1;

        if new_ticks_spent >= ticks_required {
            // Generate content and start new creation
            let content = self.core.generate_content(config);
            self.core.created_content.push(content.id);

            (Some(content), self.start_creating_post())
        } else {
            // Continue current creation
            (
                None,
                AgentState::CreatingPost {
                    post_id,
                    ticks_spent: new_ticks_spent,
                    ticks_required,
                },
            )
        }
    }

    fn start_creating_post(&self) -> AgentState {
        AgentState::CreatingPost {
            post_id: rand::thread_rng().next_u32() as usize,
            ticks_spent: 0,
            ticks_required: Self::calculate_post_ticks(),
        }
    }

    // Organizations take longer to create posts than bots
    fn calculate_post_ticks() -> i32 {
        (random::<f32>() * 30.0) as i32
    }
}
