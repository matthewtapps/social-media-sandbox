use rand::{random, RngCore};

use super::{InterestProfile, SimulationConfig};

#[derive(Debug, Clone)]
pub struct Content {
    pub id: usize,
    pub creator_id: usize,
    pub timestamp: i64,
    pub interest_profile: InterestProfile,
    pub length: i32,

    // Reader agent IDs, for deriving engagement score
    pub readers: Vec<usize>,
    // Comment IDs, for deriving engagement score
    pub comments: Vec<usize>,

    pub engagement_score: f32,
}

impl Content {
    pub fn new(
        creator_id: usize,
        interest_profile: InterestProfile,
        config: &SimulationConfig,
    ) -> Self {
        Self {
            id: rand::thread_rng().next_u32() as usize,
            creator_id,
            timestamp: chrono::Utc::now().timestamp(),
            interest_profile,
            length: (random::<f32>() * config.max_post_length as f32) as i32,
            readers: Vec::new(),
            comments: Vec::new(),
            engagement_score: 0.0,
        }
    }

    pub fn increase_engagement(&mut self) {
        self.engagement_score += 1.0;
    }
}

#[derive(Debug, Clone)]
pub struct Comment {
    pub id: usize,
    pub commentor_id: usize,
    pub timestamp: i64,
    pub interest_profile: InterestProfile,
    pub length: i32,
}

impl Comment {
    pub fn new(
        commentor_id: usize,
        interest_profile: InterestProfile,
        config: &SimulationConfig,
    ) -> Self {
        Self {
            id: rand::thread_rng().next_u32() as usize,
            commentor_id,
            timestamp: chrono::Utc::now().timestamp(),
            interest_profile,
            length: (random::<f32>() * config.max_comment_length as f32) as i32,
        }
    }
}
