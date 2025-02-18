use rand::RngCore;

use crate::{
    engine::RecommendationsUtils,
    models::{content::Comment, SimulationConfig},
    InterestProfile, Post, RecommendationEngine,
};

#[derive(Debug, Clone)]
pub struct Offline;

#[derive(Debug, Clone)]
pub struct Scrolling {
    pub recommended_post_ids: Vec<usize>,
}

impl RecommendationsUtils for ReadingPost {}

#[derive(Debug, Clone)]
pub struct ReadingPost {
    pub post_id: usize,
    pub creator_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
    pub potential_interest_gain: f32,
}

impl ReadingPost {
    pub fn new(
        post: &Post,
        read_speed: f32,
        interest_profile: &InterestProfile,
        engine: &RecommendationEngine,
    ) -> Self {
        ReadingPost {
            post_id: post.id,
            creator_id: post.creator_id,
            ticks_spent: 0,
            ticks_required: Self::calculate_required_ticks(post.length, read_speed),
            potential_interest_gain: Self::calculate_interest_gain(
                interest_profile,
                &post.interest_profile,
                engine,
            ),
        }
    }
}

impl RecommendationsUtils for ReadingComments {}

#[derive(Debug, Clone)]
pub struct ReadingComments {
    pub post_id: usize,
    pub creator_id: usize,
    pub current_comment_ids: Vec<usize>,
    pub current_comment_index: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
    pub potential_interest_gain: f32,
}

impl ReadingComments {
    pub fn new(
        post: &Post,
        comments: Vec<&Comment>,
        read_speed: f32,
        interest_profile: &InterestProfile,
        engine: &RecommendationEngine,
    ) -> Self {
        ReadingComments {
            creator_id: post.creator_id,
            post_id: post.id,
            current_comment_ids: comments.iter().map(|comment| comment.id).collect(),
            current_comment_index: 0,
            ticks_spent: 0,
            ticks_required: (comments[0].length as f32 * (1.0 - read_speed)) as i32,
            potential_interest_gain: Self::calculate_interest_gain(
                interest_profile,
                &comments[0].interest_profile,
                engine,
            ),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreatingPost {
    pub post_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
}

impl CreatingPost {
    pub fn new(write_speed: f32, config: &SimulationConfig) -> Self {
        CreatingPost {
            post_id: rand::thread_rng().next_u32() as usize,
            ticks_spent: 0,
            ticks_required: (config.base_content_length as f32 * write_speed) as i32,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CreatingComment {
    pub post_id: usize,
    pub comment_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
}

impl CreatingComment {
    pub fn new(write_speed: f32, config: &SimulationConfig, post_id: usize) -> Self {
        CreatingComment {
            post_id,
            comment_id: rand::thread_rng().next_u32() as usize,
            ticks_spent: 0,
            ticks_required: (config.base_content_length as f32 * write_speed) as i32,
        }
    }
}
