use crate::models::InterestProfile;
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum AgentType {
    Individual,
    Bot,
    Organisation,
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
