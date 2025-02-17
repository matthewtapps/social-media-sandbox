use crate::{InterestProfile, Post, RecommendationEngine};

pub struct Offline;

pub struct Scrolling {
    pub recommended_post_ids: Vec<usize>,
}

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
            potential_interest_gain: Self::calculate_interest_gain(interest_profile, post, engine),
        }
    }

    fn calculate_required_ticks(length: i32, read_speed: f32) -> i32 {
        (length as f32 * (1.0 - read_speed)) as i32
    }

    fn calculate_interest_gain(
        interest_profile: &InterestProfile,
        post: &Post,
        engine: &RecommendationEngine,
    ) -> f32 {
        let base_gain = 0.2;

        let similarity = if interest_profile.interests.is_empty() {
            0.0
        } else {
            engine.calculate_vector_similarity(
                &interest_profile.vector_representation,
                &post.interest_profile.vector_representation,
            )
        };

        let similarity_multiplier = 1.0 + similarity.min(1.0);

        base_gain * similarity_multiplier
    }
}

pub struct ReadingComments {
    pub post_id: usize,
    pub creator_id: usize,
    pub current_comment_ids: Vec<usize>,
    pub current_comment_index: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
    pub potential_interest_gain: f32,
}

pub struct CreatingPost {
    pub post_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
}

pub struct CreatingComment {
    pub post_id: usize,
    pub comment_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
}
