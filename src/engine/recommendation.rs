use crate::models::Content;
use crate::models::Individual;
use nalgebra::DVector;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct RecommendationEngine {
    pub tag_to_index: HashMap<String, usize>,
    pub index_to_tag: HashMap<usize, String>,
    pub content_pool: Vec<Content>,
    pub vector_dimension: usize,
    pub diversity_weight: f32,
    pub recency_weight: f32,
    pub engagement_weight: f32,
}

impl RecommendationEngine {
    pub fn new() -> Self {
        RecommendationEngine {
            tag_to_index: HashMap::new(),
            index_to_tag: HashMap::new(),
            content_pool: Vec::new(),
            vector_dimension: 100,
            diversity_weight: 0.2,
            recency_weight: 0.3,
            engagement_weight: 0.2,
        }
    }

    pub fn vectorize_tags(&self, tags: &[String]) -> DVector<f32> {
        let mut vector = DVector::zeros(self.vector_dimension);
        for tag in tags {
            if let Some(&index) = self.tag_to_index.get(tag) {
                vector[index] += 1.0;
            }
        }

        if vector.norm() > 0.0 {
            vector.normalize_mut();
        }
        vector
    }

    pub fn calculate_content_score(
        &self,
        content: &Content,
        agent: &Individual,
        current_time: i64,
    ) -> f32 {
        let interest_similarity = content
            .vector_representation
            .dot(&agent.core.interest_vector);

        let time_diff = current_time - content.timestamp;
        let recency_score: f32 = (-0.1 * time_diff as f32).exp() as f32;

        let diversity_score = self.calculate_diversity_score(content, agent);

        interest_similarity
            * (1.0 - self.diversity_weight - self.recency_weight - self.engagement_weight)
            + recency_score * self.recency_weight
            + diversity_score * self.diversity_weight
            + content.engagement_score * self.engagement_weight
    }

    pub fn calculate_diversity_score(&self, content: &Content, agent: &Individual) -> f32 {
        if agent.viewed_content.is_empty() {
            return 1.0;
        }

        let recent_views: Vec<&Content> = agent
            .viewed_content
            .iter()
            .take(5) // Consider last 5 viewed items
            .filter_map(|&id| self.content_pool.iter().find(|c| c.id == id))
            .collect();

        let avg_similarity: f32 = recent_views
            .iter()
            .map(|&viewed| {
                content
                    .vector_representation
                    .dot(&viewed.vector_representation)
            })
            .sum::<f32>()
            / recent_views.len() as f32;

        // Return inverse of similarity to promote diversity
        1.0 - avg_similarity
    }

    pub fn get_recommendations(
        &self,
        agent: &Individual,
        count: usize,
        current_time: i64,
    ) -> Vec<&Content> {
        let mut scored_content: Vec<(&Content, f32)> = self
            .content_pool
            .iter()
            .filter(|content| !agent.viewed_content.contains(&content.id))
            .map(|content| {
                let score = self.calculate_content_score(content, agent, current_time);
                (content, score)
            })
            .collect();

        // Sort by score in descending order
        scored_content.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Return top N recommendations
        scored_content
            .into_iter()
            .take(count)
            .map(|(content, _)| content)
            .collect()
    }

    pub fn update_agent_interests(&self, agent: &mut Individual, viewed_content: &Content) {
        for (tag, weight) in viewed_content.tags.iter().filter_map(|tag| {
            self.tag_to_index
                .get(tag)
                .map(|&i| (tag, agent.core.interest_vector[i]))
        }) {
            let current_interest = agent.core.interests.entry(tag.clone()).or_insert(0.0);
            *current_interest += agent.bias_factor * (weight - *current_interest);
        }

        // Update interest vector
        agent.core.interest_vector =
            self.vectorize_tags(&agent.core.interests.keys().cloned().collect::<Vec<_>>());
    }

    pub fn get_content_by_id(&self, content_id: usize) -> Option<&Content> {
        self.content_pool.iter().find(|c| c.id == content_id)
    }
}
