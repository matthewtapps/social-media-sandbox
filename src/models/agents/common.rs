use crate::models::{InterestProfile, SimulationConfig};
use crate::{Post, RecommendationEngine};
use rand::{random, RngCore};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Agent: Debug + Any {
    fn tick(&mut self, engine: &mut RecommendationEngine, config: &SimulationConfig);

    fn clone_box(&self) -> Box<dyn Agent>;

    fn get_type(&self) -> AgentType;

    fn interest_profile(&self) -> &InterestProfile;

    fn preferred_creators(&self) -> Option<&HashMap<usize, f32>> {
        None
    }

    fn state(&self) -> &AgentState;

    fn id(&self) -> &usize;
}

impl Clone for Box<dyn Agent> {
    fn clone(&self) -> Box<dyn Agent> {
        self.clone_box()
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AgentType {
    Individual,
    Bot,
    Organisation,
}

#[derive(Debug, Clone)]
pub enum AgentState {
    Offline,
    Scrolling {
        recommended_post_ids: Vec<usize>,
    },
    ReadingPost {
        post_id: usize,
        creator_id: usize,
        ticks_spent: i32,
        ticks_required: i32,
        potential_interest_gain: f32,
    },
    ReadingComments {
        post_id: usize,
        creator_id: usize,
        current_comment_ids: Vec<usize>,
        current_comment_index: usize,
        ticks_spent: i32,
        ticks_required: i32,
        potential_interest_gain: f32,
    },
    CreatingPost {
        post_id: usize,
        ticks_spent: i32,
        ticks_required: i32,
    },
    CreatingComment {
        post_id: usize,
        comment_id: usize,
        ticks_spent: i32,
        ticks_required: i32,
    },
}

#[derive(Debug, Clone)]
pub struct AgentCore {
    pub id: usize,
    pub content_creation_frequency: f32, // 1 = the most frequent, 0 = never posts
    pub created_content: Vec<usize>,
    pub create_speed: f32,
    pub state: AgentState,

    // Determines the interest profile of any content created, which is used for
    // content recommendations and updates of consumer interests
    pub interest_profile: InterestProfile,
}

impl AgentCore {
    pub fn generate_content(&self, config: &SimulationConfig) -> Post {
        let selected_tags = self
            .interest_profile
            .select_content_tags(config.min_content_tags, config.max_content_tags);

        let content_profile = self.interest_profile.filtered_clone(&selected_tags);

        Post {
            id: rand::thread_rng().next_u32() as usize,
            creator_id: self.id,
            timestamp: chrono::Utc::now().timestamp(),
            interest_profile: content_profile,
            length: (random::<f32>() * config.max_post_length as f32) as i32,
            readers: Vec::new(),
            comments: Vec::new(),
            engagement_score: 0.0,
        }
    }
}
