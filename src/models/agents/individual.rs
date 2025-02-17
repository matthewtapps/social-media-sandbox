use rand::{random, RngCore};

use crate::{
    models::{content::Comment, SimulationConfig},
    Post, RecommendationEngine,
};

use super::{
    AgentCore, CreatingComment, CreatingPost, Offline, ReadingComments, ReadingPost, Scrolling,
    TransitionError,
};

#[derive(Debug, Clone)]
pub struct IndividualCore {
    // 1 = will definitely keep scrolling, 0 = will stop scrolling now
    pub next_post_likelihood: f32,

    // 1 = will read any length of post, 0 = just reads headlines
    pub attention_span: f32,

    // 1 = fastest consume speed, 0 = will never finish
    pub read_speed: f32,

    // List of content IDs representing posts that have been previously
    // recommended while scrolling
    pub viewed_content: Vec<usize>,

    // How many ticks the current online session has run for
    pub session_length_ticks: i32,
}

#[derive(Debug, Clone)]
pub struct Individual<S> {
    pub individual_core: IndividualCore,
    pub core: AgentCore,
    pub state: S,
}

impl From<(Individual<Offline>, &RecommendationEngine)> for Individual<Scrolling> {
    fn from((agent, engine): (Individual<Offline>, &RecommendationEngine)) -> Self {
        let recommended_posts = engine.get_post_recommendations(
            &agent.core.interest_profile,
            agent.individual_core.viewed_content,
            10,
            chrono::Utc::now().timestamp(),
        );
        Individual {
            individual_core: agent.individual_core,
            core: agent.core,
            state: Scrolling {
                recommended_post_ids: recommended_posts,
            },
        }
    }
}

impl Individual<Scrolling> {
    fn select_post(&self, engine: &RecommendationEngine) -> Option<Post> {
        if self.state.recommended_post_ids.is_empty() {
            return None;
        };
        let recommended_post_ids = self.state.recommended_post_ids;

        let agent_vector = self.core.interest_profile.vector_representation;

        let scored_recommendations: Vec<_> = recommended_post_ids
            .iter()
            .map(|id| {
                let content = engine.get_content_by_id(*id).unwrap();
                let similarity = engine.calculate_vector_similarity(
                    &agent_vector,
                    &content.interest_profile.vector_representation,
                );
                (content, similarity)
            })
            .collect();

        let total_similarity: f32 = scored_recommendations
            .iter()
            .map(|(_, similarity)| similarity)
            .sum();

        let mut random_value = random::<f32>() * total_similarity;

        &scored_recommendations.iter().map(|(post, similarity)| {
            random_value -= similarity;
            if random_value < 0.0 {
                return Some(post);
            }
            None
        });
        None
    }

    fn select_comment_on_post(&self, engine: &RecommendationEngine) -> Option<ReadingComments> {
        let post = self.select_post(engine).unwrap();

        if let Some(comments) = engine.get_comment_recommendations(post.id, Vec::new(), 10) {
            if !comments.is_empty() {
                if let Some(first_comment) = post.comments.iter().find(|c| c.id == comments[0].id) {
                    return Some(ReadingComments {
                        post_id: post.id,
                        creator_id: post.creator_id,
                        current_comment_ids: comments.iter().map(|c| c.id).collect(),
                        current_comment_index: 0,
                        ticks_spent: 0,
                        ticks_required: (first_comment.length as f32
                            * (1.0 - self.individual_core.read_speed))
                            as i32,
                        potential_interest_gain: self
                            .calculate_potential_interest_gain_from_comment(first_comment, engine),
                    });
                } else {
                    return None;
                }
            } else {
                return None;
            }
        } else {
            return None;
        }
    }

    fn calculate_potential_interest_gain(&self, post: &Post, engine: &RecommendationEngine) -> f32 {
        let base_gain = 0.2;

        let similarity = if self.core.interest_profile.interests.is_empty() {
            0.0
        } else {
            engine.calculate_vector_similarity(
                &self.core.interest_profile.vector_representation,
                &post.interest_profile.vector_representation,
            )
        };

        let similarity_multiplier = 1.0 + similarity.min(1.0);

        base_gain * similarity_multiplier
    }

    fn calculate_potential_interest_gain_from_comment(
        &self,
        comment: &Comment,
        engine: &RecommendationEngine,
    ) -> f32 {
        let base_gain = 0.2;

        let similarity = if self.core.interest_profile.interests.is_empty() {
            0.0
        } else {
            engine.calculate_vector_similarity(
                &self.core.interest_profile.vector_representation,
                &comment.interest_profile.vector_representation,
            )
        };

        let similarity_multiplier = 1.0 + similarity.min(1.0);

        base_gain * similarity_multiplier
    }
}

impl TryFrom<(Individual<Scrolling>, &RecommendationEngine)> for Individual<ReadingPost> {
    type Error = TransitionError;

    fn try_from(
        (agent, engine): (Individual<Scrolling>, &RecommendationEngine),
    ) -> Result<Individual<ReadingPost>, Self::Error> {
        let post = agent
            .select_post(engine)
            .ok_or(TransitionError::NoPostAvailable)?;

        Ok(Individual {
            individual_core: agent.individual_core,
            core: agent.core,
            state: ReadingPost::new(
                &post,
                agent.individual_core.read_speed,
                &agent.core.interest_profile,
                engine,
            ),
        })
    }
}

impl TryFrom<(Individual<Scrolling>, &RecommendationEngine)> for Individual<ReadingComments> {
    type Error = TransitionError;

    fn try_from(
        (agent, engine): (Individual<Scrolling>, &RecommendationEngine),
    ) -> Result<Individual<ReadingComments>, Self::Error> {
        let post = agent
            .select_post(engine)
            .ok_or(TransitionError::NoPostAvailable)?;

        let selected_comments = engine
            .get_comment_recommendations(post.id, Vec::new(), 10)
            .ok_or(TransitionError::NoCommentsAvailable)?;

        Ok(Individual {
            individual_core: agent.individual_core,
            core: agent.core,
            state: ReadingComments::new(
                &post,
                selected_comments,
                agent.individual_core.read_speed,
                &agent.core.interest_profile,
                engine,
            ),
        })
    }
}

impl Individual<CreatingPost> {
    pub fn generate_content(&self, config: &SimulationConfig) -> Post {
        let selected_tags = self
            .core
            .interest_profile
            .select_content_tags(config.min_content_tags, config.max_content_tags);

        let content_profile = self.core.interest_profile.filtered_clone(&selected_tags);

        Post {
            id: rand::thread_rng().next_u32() as usize,
            creator_id: self.core.id,
            timestamp: chrono::Utc::now().timestamp(),
            interest_profile: content_profile,
            length: (random::<f32>() * config.max_post_length as f32) as i32,
            readers: Vec::new(),
            comments: Vec::new(),
            engagement_score: 0.0,
        }
    }
}

impl Individual<CreatingComment> {}
