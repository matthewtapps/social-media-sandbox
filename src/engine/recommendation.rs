use crate::models::content::Comment;
use crate::models::Agent;
use nalgebra::DVector;

use crate::models::Individual;
use crate::models::Post;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RecommendationEngine {
    pub tag_to_index: HashMap<String, usize>,
    pub index_to_tag: HashMap<usize, String>,
    pub content_pool: Vec<Post>,
    pub vector_dimension: usize,
    pub config: RecommendationEngineConfig,
}

#[derive(Debug, Clone)]
pub struct RecommendationEngineConfig {
    pub interest_weight: f32,
    pub recency_weight: f32,
    pub engagement_weight: f32,
    pub recency_decay_rate: f32,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        RecommendationEngine {
            tag_to_index: HashMap::new(),
            index_to_tag: HashMap::new(),
            content_pool: Vec::new(),
            vector_dimension: 100,
            config: RecommendationEngineConfig {
                interest_weight: 0.5,
                recency_weight: 0.3,
                engagement_weight: 0.2,
                recency_decay_rate: 0.05,
            },
        }
    }

    pub fn get_content_by_id(&self, content_id: usize) -> Option<&Post> {
        self.content_pool.iter().find(|c| c.id == content_id)
    }

    pub fn get_comments_by_post_id(&self, content_id: usize) -> Option<Vec<&Comment>> {
        self.content_pool
            .iter()
            .find(|c| c.id == content_id)
            .map(|post| post.comments.iter().collect())
    }

    pub fn calculate_content_score(
        &self,
        content: &Post,
        agent: &Individual,
        current_time: i64,
    ) -> f32 {
        let interest_alignment = self.calculate_vector_similarity(
            &agent.interest_profile().vector_representation,
            &content.interest_profile.vector_representation,
        );

        let hours_old = (current_time - content.timestamp) as f32 / 3600.0;
        let recency_score = (-0.05 * hours_old).exp(); // Decay by ~5% per hour

        let engagement_score = content.engagement_score; // Assuming this is already normalized 0.0-1.0

        let score = interest_alignment * self.config.interest_weight
            + recency_score * self.config.recency_weight
            + engagement_score * self.config.engagement_weight;

        score.clamp(0.0, 1.0)
    }

    pub fn calculate_vector_similarity(&self, vec1: &DVector<f32>, vec2: &DVector<f32>) -> f32 {
        let dot_product = vec1.dot(vec2);
        let magnitude1 = vec1.norm();
        let magnitude2 = vec2.norm();

        if magnitude1 == 0.0 || magnitude2 == 0.0 {
            return 0.0;
        }

        (dot_product / (magnitude1 * magnitude2)).clamp(0.0, 1.0)
    }

    pub fn get_post_recommendations(
        &self,
        agent: &Individual,
        count: usize,
        current_time: i64,
    ) -> Vec<usize> {
        let mut scored_posts: Vec<(usize, f32)> = self
            .content_pool
            .iter()
            .filter(|content| !agent.viewed_content.contains(&content.id))
            .map(|content| {
                let score = self.calculate_content_score(content, agent, current_time);
                (content.id, score)
            })
            .collect();

        scored_posts.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        scored_posts
            .into_iter()
            .take(count)
            .map(|(id, _)| id)
            .collect()
    }

    pub fn get_comment_recommendations(
        &self,
        post_id: usize,
        current_comment_ids: Vec<usize>,
        count: usize,
    ) -> Option<Vec<usize>> {
        self.get_content_by_id(post_id).map(|post| {
            let current_ids: std::collections::HashSet<_> =
                current_comment_ids.into_iter().collect();

            let mut comments: Vec<(&Comment, usize)> = post
                .comments
                .iter()
                .filter(|comment| !current_ids.contains(&comment.id))
                .map(|comment| (comment, comment.id))
                .collect();

            comments.sort_by(|(a, _), (b, _)| {
                b.engagement_score.partial_cmp(&a.engagement_score).unwrap()
            });
            comments.into_iter().take(count).map(|(_, id)| id).collect()
        })
    }

    pub fn increase_engagement_score(&mut self, content_id: usize) {
        let post: &mut Post = self
            .content_pool
            .iter_mut()
            .find(|c| c.id == content_id)
            .unwrap();

        post.increase_engagement();
    }

    pub fn add_comment_to_post(&mut self, post_id: usize, comment: Comment) {
        let post: &mut Post = self
            .content_pool
            .iter_mut()
            .find(|c| c.id == post_id)
            .unwrap();

        post.comments.push(comment);
    }

    pub fn create_post(&mut self, post: Post) {
        self.content_pool.push(post);
    }
}
