use crate::{
    models::{content::Comment, InterestProfile, SimulationConfig},
    Post, RecommendationEngine,
};
use rand::{random, RngCore};

use super::{Agent, AgentCore, AgentState, AgentType};

#[derive(Debug, Clone)]
pub struct Individual {
    pub core: AgentCore,

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

impl Agent for Individual {
    fn tick(&mut self, engine: &mut RecommendationEngine, config: &SimulationConfig) {
        self.session_length_ticks += 1;
        let new_state = match &self.core.state {
            AgentState::Offline => self.proceed_from_offline(engine, config),
            AgentState::Scrolling {
                recommended_post_ids,
            } => self.proceed_from_scrolling(engine, config, recommended_post_ids.clone()),
            AgentState::ReadingPost {
                post_id,
                creator_id,
                ticks_spent,
                ticks_required,
                potential_interest_gain,
            } => self.proceed_from_reading_post(
                engine,
                config,
                *post_id,
                *creator_id,
                *ticks_spent,
                *ticks_required,
                *potential_interest_gain,
            ),
            AgentState::CreatingPost {
                post_id,
                ticks_spent,
                ticks_required,
            } => self.proceed_from_creating_post(
                engine,
                config,
                *post_id,
                *ticks_spent,
                *ticks_required,
            ),
            AgentState::ReadingComments {
                post_id,
                creator_id,
                current_comment_ids,
                current_comment_index,
                ticks_spent,
                ticks_required,
                potential_interest_gain,
            } => self.proceed_from_reading_comments(
                engine,
                config,
                *post_id,
                *creator_id,
                current_comment_ids.clone(),
                *current_comment_index,
                *ticks_spent,
                *ticks_required,
                *potential_interest_gain,
            ),
            AgentState::CreatingComment {
                post_id,
                comment_id,
                ticks_spent,
                ticks_required,
            } => self.proceed_from_creating_comment(
                engine,
                config,
                *post_id,
                *comment_id,
                *ticks_spent,
                *ticks_required,
            ),
        };
        self.core.state = new_state;
    }

    fn clone_box(&self) -> Box<dyn Agent> {
        Box::new(self.clone())
    }

    fn get_type(&self) -> AgentType {
        AgentType::Individual
    }

    fn interest_profile(&self) -> &InterestProfile {
        &self.core.interest_profile
    }

    fn state(&self) -> &AgentState {
        &self.core.state
    }

    fn id(&self) -> &usize {
        &self.core.id
    }
}

impl Individual {
    pub fn new(id: usize, _config: &SimulationConfig, _engine: &RecommendationEngine) -> Self {
        Self {
            core: AgentCore {
                id,
                content_creation_frequency: random::<f32>().min(0.3),
                created_content: Vec::new(),
                create_speed: random(),
                state: AgentState::Offline,
                interest_profile: InterestProfile::new(100),
            },
            next_post_likelihood: random(),
            attention_span: random::<f32>().min(0.5),
            viewed_content: Vec::new(),
            read_speed: random(),
            session_length_ticks: 0,
        }
    }

    fn proceed_from_offline(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
    ) -> AgentState {
        // Uses next post likelihood to determine whether to come online
        if random::<f32>() < self.next_post_likelihood {
            self.proceed_to_scrolling(engine, config)
        } else {
            AgentState::Offline
        }
    }

    fn proceed_from_scrolling(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
        current_recommendations: Vec<usize>,
    ) -> AgentState {
        // First check if we should select a post to interact with
        if self.should_select_post() {
            if let Some(selected_post_id) =
                self.select_post_from_recommendations(current_recommendations, engine)
            {
                // Get the selected post
                if let Some(selected_post) = engine.get_content_by_id(selected_post_id) {
                    // Decide what to do with the selected post
                    if self.should_read_post() {
                        return AgentState::ReadingPost {
                            post_id: selected_post.id,
                            creator_id: selected_post.creator_id,
                            ticks_spent: 0,
                            ticks_required: (selected_post.length as f32 * (1.0 - self.read_speed))
                                as i32,
                            potential_interest_gain: self
                                .calculate_potential_interest_gain(selected_post, engine),
                        };
                    }

                    if self.should_read_comments() {
                        // Get initial batch of comments
                        if let Some(comment_ids) = engine.get_comment_recommendations(
                            selected_post.id,
                            Vec::new(), // No viewed comments yet
                            10,
                        ) {
                            if !comment_ids.is_empty() {
                                // Find the first comment to start reading
                                if let Some(first_comment) = selected_post
                                    .comments
                                    .iter()
                                    .find(|c| c.id == comment_ids[0])
                                {
                                    return AgentState::ReadingComments {
                                        post_id: selected_post.id,
                                        creator_id: selected_post.creator_id,
                                        current_comment_ids: comment_ids,
                                        current_comment_index: 0,
                                        ticks_spent: 0,
                                        ticks_required: (first_comment.length as f32
                                            * (1.0 - self.read_speed))
                                            as i32,
                                        potential_interest_gain: self
                                            .calculate_potential_interest_gain_from_comment(
                                                first_comment,
                                                engine,
                                            ),
                                    };
                                }
                            }
                        }
                    }

                    if self.should_write_comment() {
                        let comment_id = rand::thread_rng().next_u32() as usize;
                        return AgentState::CreatingComment {
                            post_id: selected_post.id,
                            comment_id,
                            ticks_spent: 0,
                            ticks_required: (config.max_comment_length as f32
                                * (1.0 - self.core.create_speed))
                                as i32,
                        };
                    }
                }
            }
        }

        // Check if we should go offline
        if self.should_go_offline() {
            return AgentState::Offline;
        }

        // If we didn't transition to any other state, keep scrolling
        self.proceed_to_scrolling(engine, config)
    }

    fn proceed_from_reading_post(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
        post_id: usize,
        creator_id: usize,
        mut ticks_spent: i32,
        ticks_required: i32,
        potential_interest_gain: f32,
    ) -> AgentState {
        ticks_spent += 1;

        if ticks_spent >= ticks_required || random::<f32>() > self.attention_span {
            self.viewed_content.push(post_id);

            // TODO: Update interests based on the post content

            if self.should_go_offline() {
                AgentState::Offline
            } else {
                self.proceed_to_scrolling(engine, config)
            }
        } else {
            let post = engine.get_content_by_id(post_id).unwrap();
            self.update_interests_from_profile(
                &post.interest_profile,
                ticks_spent,
                potential_interest_gain,
            );
            AgentState::ReadingPost {
                post_id,
                creator_id,
                ticks_spent,
                ticks_required,
                potential_interest_gain,
            }
        }
    }

    fn update_interests_from_profile(
        &mut self,
        interest_profile: &InterestProfile,
        ticks_spent: i32,
        potential_interest_gain: f32,
    ) {
        // Starts at 1/2 gained in first tick, then progressively approaches
        // 1/1 in subsequent ticks
        let interest_this_tick = potential_interest_gain / (ticks_spent + 1) as f32;

        self.core
            .interest_profile
            .update_interest_from_profile(&interest_profile, interest_this_tick);
    }

    fn proceed_from_reading_comments(
        &mut self,
        engine: &RecommendationEngine,
        config: &SimulationConfig,
        post_id: usize,
        creator_id: usize,
        current_comment_ids: Vec<usize>,
        current_comment_index: usize,
        mut ticks_spent: i32,
        ticks_required: i32,
        potential_interest_gain: f32,
    ) -> AgentState {
        // Progress reading current comment at start of tick
        ticks_spent += 1;

        if let Some(post) = engine.get_content_by_id(post_id) {
            if let Some(current_comment) = post
                .comments
                .iter()
                .find(|c| c.id == current_comment_ids[current_comment_index])
            {
                self.update_interests_from_profile(
                    &current_comment.interest_profile,
                    ticks_spent,
                    potential_interest_gain,
                );

                // If we're finished or bored
                if ticks_spent >= ticks_required || random::<f32>() > self.attention_span {
                    // Maybe read the post if we haven't yet
                    if self.should_read_post() && !self.viewed_content.contains(&post_id) {
                        return AgentState::ReadingPost {
                            post_id,
                            creator_id,
                            ticks_spent: 0,
                            ticks_required: (post.length as f32 * (1.0 - self.read_speed)) as i32,
                            potential_interest_gain: self
                                .calculate_potential_interest_gain(post, engine),
                        };
                    }

                    // Maybe write our own comment
                    if self.should_write_comment() {
                        let comment_id = rand::thread_rng().next_u32() as usize;
                        return AgentState::CreatingComment {
                            post_id,
                            comment_id,
                            ticks_spent: 0,
                            ticks_required: (config.max_comment_length as f32
                                * (1.0 - self.core.create_speed))
                                as i32,
                        };
                    }

                    // Maybe go offline
                    if self.should_go_offline() {
                        return AgentState::Offline;
                    }

                    // Maybe move to next comment
                    let next_index = current_comment_index + 1;
                    if next_index < current_comment_ids.len() {
                        if let Some(next_comment) = post
                            .comments
                            .iter()
                            .find(|c| c.id == current_comment_ids[next_index])
                        {
                            return AgentState::ReadingComments {
                                post_id,
                                creator_id,
                                current_comment_ids,
                                current_comment_index: next_index,
                                ticks_spent: 0,
                                ticks_required: (next_comment.length as f32
                                    * (1.0 - self.read_speed))
                                    as i32,
                                potential_interest_gain: self
                                    .calculate_potential_interest_gain_from_comment(
                                        next_comment,
                                        engine,
                                    ),
                            };
                        }
                    }

                    // Maybe go back to scrolling
                    if self.should_scroll() {
                        return self.proceed_to_scrolling(engine, config);
                    }
                }
            }
        }

        // Continue reading current comment if we haven't finished and didn't transition to another state
        AgentState::ReadingComments {
            post_id,
            creator_id,
            current_comment_ids,
            current_comment_index,
            ticks_spent,
            ticks_required,
            potential_interest_gain,
        }
    }

    fn calculate_potential_interest_gain(
        &self,
        content: &Post,
        engine: &RecommendationEngine,
    ) -> f32 {
        let base_gain = 0.2;

        let similarity = if self.core.interest_profile.interests.is_empty() {
            0.0
        } else {
            engine.calculate_vector_similarity(
                &self.core.interest_profile.vector_representation,
                &content.interest_profile.vector_representation,
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

    fn proceed_from_creating_post(
        &mut self,
        engine: &mut RecommendationEngine,
        config: &SimulationConfig,
        post_id: usize,
        mut ticks_spent: i32,
        ticks_required: i32,
    ) -> AgentState {
        ticks_spent += 1;

        if ticks_spent >= ticks_required {
            let content = self.core.generate_content(config);

            self.core.created_content.push(content.id);

            engine.create_post(content);

            if self.should_go_offline() {
                return AgentState::Offline;
            } else {
                return self.proceed_to_scrolling(engine, config);
            };
        } else {
            // Continue creating post
            AgentState::CreatingPost {
                post_id,
                ticks_spent,
                ticks_required,
            }
        }
    }

    fn proceed_from_creating_comment(
        &mut self,
        engine: &mut RecommendationEngine,
        config: &SimulationConfig,
        post_id: usize,
        comment_id: usize,
        mut ticks_spent: i32,
        ticks_required: i32,
    ) -> AgentState {
        ticks_spent += 1;

        if ticks_spent >= ticks_required {
            let comment = Comment::new(comment_id, self.core.interest_profile.clone(), config);

            engine.add_comment_to_post(post_id, comment);

            // After creating a comment, we might:
            if self.should_go_offline() {
                return AgentState::Offline;
            }

            // Default to going back to scrolling
            self.proceed_to_scrolling(engine, config)
        } else {
            // Continue creating comment
            AgentState::CreatingComment {
                post_id,
                comment_id,
                ticks_spent,
                ticks_required,
            }
        }
    }

    // Encapsulates retrieving recommendations from engine and adding the retrieved
    // posts to viewed content
    fn proceed_to_scrolling(
        &mut self,
        engine: &RecommendationEngine,
        _config: &SimulationConfig,
    ) -> AgentState {
        let recommended_post_ids =
            engine.get_post_recommendations(&self, 10, chrono::Utc::now().timestamp());

        // Add retrieved recommendations to viewed content which the engine
        // filters out from future recommendations
        self.viewed_content.extend(recommended_post_ids.clone());

        AgentState::Scrolling {
            recommended_post_ids,
        }
    }

    fn select_post_from_recommendations(
        &self,
        recommended_post_ids: Vec<usize>,
        engine: &RecommendationEngine,
    ) -> Option<usize> {
        if recommended_post_ids.is_empty() {
            return None;
        }

        let agent_vector = &self.core.interest_profile.vector_representation;

        let scored_recommendations: Vec<_> = recommended_post_ids
            .iter()
            .map(|id| {
                // TODO: Unwrap bad, should properly handle when ID isn't found
                let content = engine.get_content_by_id(*id).unwrap();
                let similarity = engine.calculate_vector_similarity(
                    agent_vector,
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

        for (content, similarity) in &scored_recommendations {
            random_value -= similarity;
            if random_value <= 0.0 {
                return Some(content.id);
            }
        }

        scored_recommendations.last().map(|(content, _)| content.id)
    }

    // TODO: Proper probability calculations for these functions
    fn should_go_offline(&self) -> bool {
        // Should get higher probability the longer we've been scrolling for
        // if random::<f32>() > 0.9 {
        //     return true;
        // }
        false
    }

    fn should_select_post(&self) -> bool {
        // Higher interest alignment in the post should increase probability of
        // selecting that post
        if random::<f32>() > 0.5 {
            return true;
        }
        false
    }

    fn should_read_post(&self) -> bool {
        if random::<f32>() > 0.5 {
            return true;
        }
        false
    }

    fn should_read_comments(&self) -> bool {
        if random::<f32>() > 0.5 {
            return true;
        }
        false
    }

    fn should_write_comment(&self) -> bool {
        if random::<f32>() > 0.5 {
            return true;
        }
        false
    }

    fn should_scroll(&self) -> bool {
        if random::<f32>() > 0.5 {
            return true;
        }
        false
    }
}
