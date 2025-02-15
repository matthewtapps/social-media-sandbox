use nalgebra::DVector;
use rand::{random, Rng};
use std::collections::HashMap;

use super::Post;

#[derive(Debug, Clone)]
pub struct Topic {
    // Represents the Agent's weighted interest in the Topic - an f32 between
    // 0.0 and 1.0 that adds up to 1.0 when combined with all the other Topic
    // weights of that Agent
    pub weighted_interest: f32,
    // A value from -1.0 to 1.0 that represents the Agent's level of disagreement
    // or agreement with the Topic
    pub agreement: f32,
}

#[derive(Debug, Clone)]
pub struct InterestProfile {
    // String representation attached to that Topic, which is like a tag
    pub interests: HashMap<String, Topic>,

    // Should add to 1.0 except when being updated and prior to being normalised
    pub total_weight: f32,

    // Vector representation of an Interest only considers the weights of the interests,
    // not the agreement level, so that these can be used separately within the
    // recommendation algorithm;
    // Agreement level only makes sense in context with the interest it belongs to,
    // so vectorising those separately is not useful
    pub vector_representation: DVector<f32>,
}

impl InterestProfile {
    pub fn new(dimension_size: usize) -> Self {
        Self {
            interests: HashMap::new(),
            total_weight: 0.0,
            vector_representation: DVector::zeros(dimension_size),
        }
    }

    pub fn filtered_clone(&self, selected_tags: &[String]) -> Self {
        let mut filtered = InterestProfile::new(self.vector_representation.len());

        for tag in selected_tags {
            if let Some(topic) = self.interests.get(tag) {
                filtered.interests.insert(
                    tag.clone(),
                    Topic {
                        weighted_interest: topic.weighted_interest,
                        agreement: topic.agreement,
                    },
                );
            }
        }

        filtered.normalise_weights();
        filtered
    }

    pub fn update_interest_from_post(&mut self, post: &Post, interest: f32) {
        for (tag, content_interest) in &post.interest_profile.interests {
            let weighted_addition = content_interest.weighted_interest * interest;

            let topic = self.interests.entry(tag.clone()).or_insert(Topic {
                weighted_interest: 0.0,
                agreement: 0.0,
            });

            topic.weighted_interest += weighted_addition;
        }

        self.normalise_weights();
    }

    pub fn normalise_weights(&mut self) {
        self.total_weight = self
            .interests
            .values()
            .map(|topic| topic.weighted_interest)
            .sum();

        if self.total_weight == 0.0 {
            return;
        }

        for topic in self.interests.values_mut() {
            topic.weighted_interest /= self.total_weight;
        }

        self.total_weight = 1.0;
    }

    pub fn select_content_tags(&self, min_tags: usize, max_tags: usize) -> Vec<String> {
        let mut interests: Vec<_> = self
            .interests
            .iter()
            .map(|(tag, topic)| (tag.clone(), topic.weighted_interest))
            .collect();

        interests.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        let mut selected_tags = Vec::new();
        let mut remaining_tags = interests.clone();

        let num_tags = rand::thread_rng().gen_range(min_tags..=max_tags.min(interests.len()));

        if !interests.is_empty() {
            let mut random_weight = random::<f32>();

            for (tag, weight) in interests.iter() {
                random_weight -= weight;
                if random_weight <= 0.0 {
                    selected_tags.push(tag.clone());
                    remaining_tags.retain(|(t, _)| t != tag);
                    break;
                }
            }

            if selected_tags.is_empty() {
                selected_tags.push(interests[0].0.clone());
                remaining_tags.remove(0);
            }
        }

        while selected_tags.len() < num_tags && !remaining_tags.is_empty() {
            let index = rand::thread_rng().gen_range(0..remaining_tags.len());
            selected_tags.push(remaining_tags.remove(index).0);
        }

        selected_tags
    }
}
