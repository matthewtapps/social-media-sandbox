use chrono;
use nalgebra::DVector;
use rand::{random, Rng, RngCore};
use std::collections::HashMap;

#[derive(Debug, Clone)]
enum AgentType {
    Individual,
    Bot,
    Organisation,
}

#[derive(Debug, Clone)]
struct Content {
    id: usize,
    creator_id: usize,
    timestamp: i64,
    tags: Vec<String>,
    engagement_score: f32,
    vector_representation: DVector<f32>,
}

#[derive(Debug, Clone)]
struct Agent {
    id: usize,
    agent_type: AgentType,
    content_generation_frequency: f32, // 1 = the most frequent, 0 = never posts
    created_content: Vec<usize>,

    // Interests determine the tags of content that is created by an agent, and the recommendations
    // provided to Individuals
    interests: HashMap<String, f32>,
    interest_vector: DVector<f32>,

    // These attributes only matter to Individuals as content consumers
    next_post_likelihood: f32, // 1 = will definitely keep scrolling, 0 = will stop scrolling now
    attention_span: f32,       // 1 = will read any length of post, 0 = just reads headlines
    preferred_creators: HashMap<usize, f32>, // The ID of the creator, and a f32 weight
    viewed_content: Vec<usize>,
    bias_factor: f32,
}

#[derive(Debug, Clone)]
struct RecommendationEngine {
    tag_to_index: HashMap<String, usize>,
    index_to_tag: HashMap<usize, String>,
    content_pool: Vec<Content>,
    vector_dimension: usize,
    diversity_weight: f32,
    recency_weight: f32,
    engagement_weight: f32,
}

impl RecommendationEngine {
    fn new() -> Self {
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

    fn vectorize_tags(&self, tags: &[String]) -> DVector<f32> {
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

    fn calculate_content_score(&self, content: &Content, agent: &Agent, current_time: i64) -> f32 {
        let interest_similarity = content.vector_representation.dot(&agent.interest_vector);

        let time_diff = current_time - content.timestamp;
        let recency_score: f32 = (-0.1 * time_diff as f32).exp() as f32;

        let diversity_score = self.calculate_diversity_score(content, agent);

        interest_similarity
            * (1.0 - self.diversity_weight - self.recency_weight - self.engagement_weight)
            + recency_score * self.recency_weight
            + diversity_score * self.diversity_weight
            + content.engagement_score * self.engagement_weight
    }

    fn calculate_diversity_score(&self, content: &Content, agent: &Agent) -> f32 {
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

    fn get_recommendations(&self, agent: &Agent, count: usize, current_time: i64) -> Vec<&Content> {
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

    fn update_agent_interests(&self, agent: &mut Agent, viewed_content: &Content) {
        for (tag, weight) in viewed_content.tags.iter().filter_map(|tag| {
            self.tag_to_index
                .get(tag)
                .map(|&i| (tag, agent.interest_vector[i]))
        }) {
            let current_interest = agent.interests.entry(tag.clone()).or_insert(0.0);
            *current_interest += agent.bias_factor * (weight - *current_interest);
        }

        // Update interest vector
        agent.interest_vector =
            self.vectorize_tags(&agent.interests.keys().cloned().collect::<Vec<_>>());
    }
}

impl Agent {
    fn new(id: usize, agent_type: AgentType) -> Self {
        Agent {
            id,
            agent_type,
            content_generation_frequency: random(),
            interests: HashMap::new(),
            interest_vector: DVector::zeros(100),
            next_post_likelihood: random(),
            attention_span: random(),
            preferred_creators: HashMap::new(),
            viewed_content: Vec::new(),
            created_content: Vec::new(),
            bias_factor: random(),
        }
    }

    fn generate_content(&self, engine: &RecommendationEngine) -> Content {
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
        }
    }
}

fn main() {
    // Initialize recommendation engine
    let mut engine = RecommendationEngine::new();

    // Set up initial tags
    let sample_tags = vec![
        "politics".to_string(),
        "technology".to_string(),
        "science".to_string(),
        "entertainment".to_string(),
        "sports".to_string(),
    ];

    // Initialize tag mappings
    for (i, tag) in sample_tags.iter().enumerate() {
        engine.tag_to_index.insert(tag.clone(), i);
        engine.index_to_tag.insert(i, tag.clone());
    }

    // Create content creator agent with strong interests in technology and science
    let mut creator = Agent::new(1, AgentType::Organisation);
    creator.interests.insert("technology".to_string(), 0.8);
    creator.interests.insert("science".to_string(), 0.7);
    let creator_tags: Vec<String> = creator.interests.keys().cloned().collect();
    creator.interest_vector = engine.vectorize_tags(&creator_tags);

    // Create content consumer agent with initial interests in politics and sports
    let mut consumer = Agent::new(2, AgentType::Individual);
    consumer.interests.insert("politics".to_string(), 0.6);
    consumer.interests.insert("sports".to_string(), 0.7);
    consumer.bias_factor = 0.3;
    let consumer_tags: Vec<String> = consumer.interests.keys().cloned().collect();
    consumer.interest_vector = engine.vectorize_tags(&consumer_tags);

    // Print initial state
    println!("Initial Consumer Interests:");
    for (tag, weight) in &consumer.interests {
        println!("  {}: {:.2}", tag, weight);
    }

    // Generate some content from creator
    let current_time = chrono::Utc::now().timestamp();

    for _ in 0..5 {
        let content = creator.generate_content(&engine);
        println!("\nCreated content with tags: {:?}", content.tags);
        engine.content_pool.push(content);
    }

    // Get recommendations for consumer
    println!("\nRecommended content for consumer:");
    let recommendations = engine.get_recommendations(&consumer, 3, current_time);
    for (i, content) in recommendations.iter().enumerate() {
        println!("Recommendation {}: {:?}", i + 1, content.tags);
    }

    // Simulate consumer viewing all recommended content
    for content in recommendations {
        engine.update_agent_interests(&mut consumer, content);
        consumer.viewed_content.push(content.id);
    }

    // Print final state
    println!("\nUpdated Consumer Interests:");
    for (tag, weight) in &consumer.interests {
        println!("  {}: {:.2}", tag, weight);
    }

    // Calculate interest changes
    println!("\nInterest Changes:");
    let initial_interests = vec![
        "politics",
        "sports",
        "technology",
        "science",
        "entertainment",
    ];

    for &tag in &initial_interests {
        let initial = consumer.interests.get(tag).copied().unwrap_or(0.0);
        let tag_string = tag.to_string();
        let final_interest = consumer.interests.get(&tag_string).copied().unwrap_or(0.0);
        let change = final_interest - initial;
        if change != 0.0 {
            println!("  {}: {:.2} ({:+.2})", tag, final_interest, change);
        }
    }
}
