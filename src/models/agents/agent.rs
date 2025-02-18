use rand::{random, RngCore};

use crate::models::InterestProfile;
use std::fmt::Debug;

#[derive(Debug, Clone)]
pub enum AgentType {
    Individual,
    Organisation,
    Bot,
}

#[derive(Debug, Clone)]
pub struct AgentCore {
    pub id: usize,
    pub content_creation_frequency: f32, // 1 = the most frequent, 0 = never posts
    pub created_content: Vec<usize>,
    pub create_speed: f32,

    // Determines the interest profile of any content created, which is used for
    // content recommendations and updates of consumer interests
    pub interest_profile: InterestProfile,
}

impl AgentCore {
    pub fn new() -> Self {
        Self {
            id: rand::thread_rng().next_u32() as usize,
            content_creation_frequency: random(),
            created_content: Vec::new(),
            create_speed: random(),
            interest_profile: InterestProfile::new(100),
        }
    }
}

pub trait AgentAccessors {
    fn id(&self) -> usize;
    fn interests(&self) -> &InterestProfile;
}
