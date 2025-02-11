use crate::{Content, RecommendationEngine};
use nalgebra::DVector;
use rand::{random, RngCore};
use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub trait Agent: Debug + Any {
    fn tick(&mut self, engine: &RecommendationEngine) -> Option<Content>;

    fn clone_box(&self) -> Box<dyn Agent>;

    fn get_type(&self) -> AgentType;

    fn interests(&self) -> &HashMap<String, f32>;

    fn preferred_creators(&self) -> Option<&HashMap<usize, f32>> {
        None
    }

    fn activity(&self) -> &Activity;
}

impl Clone for Box<dyn Agent> {
    fn clone(&self) -> Box<dyn Agent> {
        self.clone_box()
    }
}

#[derive(Debug, Clone)]
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
    pub activity: Activity,

    // Interests determine the tags of content that is created by an agent, and the recommendations
    // provided to Individuals
    pub interests: HashMap<String, f32>,
    pub interest_vector: DVector<f32>,
}

#[derive(Debug, Clone)]
pub enum Activity {
    Consuming(ContentConsumptionState),
    Creating(ContentCreationState),
    Offline,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentCreationState {
    pub content_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ContentConsumptionState {
    pub content_id: usize,
    pub creator_id: usize,
    pub ticks_spent: i32,
    pub ticks_required: i32,
}

impl AgentCore {
    pub fn generate_content(&self, engine: &RecommendationEngine) -> Content {
        let selected_tags: Vec<String> = self
            .interests
            .iter()
            .filter(|&(_, &weight)| random::<f32>() < weight)
            .map(|(tag, _)| tag.clone())
            .collect();

        Content {
            id: rand::thread_rng().next_u32() as usize,
            creator_id: self.id,
            tags: selected_tags.clone(),
            timestamp: chrono::Utc::now().timestamp(),
            engagement_score: 0.0,
            vector_representation: engine.vectorize_tags(&selected_tags),
            length: random(),
        }
    }
}
