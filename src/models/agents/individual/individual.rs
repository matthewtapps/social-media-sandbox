use rand::{random, RngCore};

use crate::{
    engine::RecommendationsUtils,
    models::{
        content::Comment, AgentAccessors, AgentCore, CreatingComment, CreatingPost, Offline,
        ReadingComments, ReadingPost, Scrolling, SimulationConfig, TransitionError,
    },
    InterestProfile, Post, RecommendationEngine,
};

#[derive(Debug, Clone)]
pub enum IndividualWrapper {
    Offline(IndividualAgent<Offline>),
    Scrolling(IndividualAgent<Scrolling>),
    ReadingPost(IndividualAgent<ReadingPost>),
    ReadingComments(IndividualAgent<ReadingComments>),
    CreatingPost(IndividualAgent<CreatingPost>),
    CreatingComment(IndividualAgent<CreatingComment>),
}

impl IndividualWrapper {
    pub fn tick(self, engine: &RecommendationEngine) -> IndividualWrapper {
        match self {
            IndividualWrapper::Offline(val) => IndividualWrapper::Scrolling((val, engine).into()),
            _ => unimplemented!(),
        }
    }

    pub fn with_agent<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&dyn AgentAccessors) -> R,
    {
        match self {
            IndividualWrapper::Offline(agent) => f(agent),
            IndividualWrapper::Scrolling(agent) => f(agent),
            IndividualWrapper::ReadingPost(agent) => f(agent),
            IndividualWrapper::ReadingComments(agent) => f(agent),
            IndividualWrapper::CreatingPost(agent) => f(agent),
            IndividualWrapper::CreatingComment(agent) => f(agent),
        }
    }

    pub fn id(&self) -> usize {
        self.with_agent(|agent| agent.id())
    }

    pub fn interests(&self) -> InterestProfile {
        self.with_agent(|agent| agent.interests().clone())
    }

    pub fn progress(&self) -> Option<f32> {
        match self {
            IndividualWrapper::Offline(_) | IndividualWrapper::Scrolling(_) => Some(1.0),
            IndividualWrapper::ReadingPost(agent) => {
                Some(agent.state.ticks_spent as f32 / agent.state.ticks_required as f32)
            }
            IndividualWrapper::CreatingPost(agent) => {
                Some(agent.state.ticks_spent as f32 / agent.state.ticks_required as f32)
            }
            IndividualWrapper::ReadingComments(agent) => {
                Some(agent.state.ticks_spent as f32 / agent.state.ticks_required as f32)
            }
            IndividualWrapper::CreatingComment(agent) => {
                Some(agent.state.ticks_spent as f32 / agent.state.ticks_required as f32)
            }
        }
    }

    pub fn state_name(&self) -> &'static str {
        match self {
            IndividualWrapper::Offline(_) => "Offline",
            IndividualWrapper::Scrolling(_) => "Scrolling",
            IndividualWrapper::ReadingPost(_) => "Reading Post",
            IndividualWrapper::CreatingPost(_) => "Creating Post",
            IndividualWrapper::ReadingComments(_) => "Reading Comments",
            IndividualWrapper::CreatingComment(_) => "Creating Comment",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Individual {
    pub agent: IndividualWrapper,
}

impl Individual {
    pub fn new() -> Self {
        Self {
            agent: IndividualWrapper::Offline(IndividualAgent::<Offline>::new()),
        }
    }

    pub fn tick(self, engine: &RecommendationEngine) -> Self {
        Self {
            agent: self.agent.tick(engine),
        }
    }
}

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

impl RecommendationsUtils for IndividualCore {}

impl IndividualCore {
    fn new() -> Self {
        IndividualCore {
            next_post_likelihood: random(),
            attention_span: random(),
            read_speed: random(),
            viewed_content: Vec::new(),
            session_length_ticks: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IndividualAgent<S> {
    pub individual_core: IndividualCore,
    pub core: AgentCore,
    pub state: S,
}

impl IndividualAgent<Offline> {
    fn new() -> Self {
        Self {
            individual_core: IndividualCore::new(),
            core: AgentCore::new(),
            state: Offline {},
        }
    }
}

impl<S> AgentAccessors for IndividualAgent<S> {
    fn id(&self) -> usize {
        self.core.id
    }

    fn interests(&self) -> &InterestProfile {
        &self.core.interest_profile
    }
}

impl From<(IndividualAgent<Offline>, &RecommendationEngine)> for IndividualAgent<Scrolling> {
    fn from((agent, engine): (IndividualAgent<Offline>, &RecommendationEngine)) -> Self {
        IndividualAgent::<Scrolling>::new(agent.individual_core, agent.core, engine)
    }
}

impl IndividualAgent<Scrolling> {
    fn new(
        individual_core: IndividualCore,
        core: AgentCore,
        engine: &RecommendationEngine,
    ) -> Self {
        let viewed_content = &individual_core.viewed_content;
        let recommended_posts = engine.get_post_recommendations(
            &core.interest_profile,
            viewed_content,
            10,
            chrono::Utc::now().timestamp(),
        );
        IndividualAgent {
            individual_core,
            core,
            state: Scrolling {
                recommended_post_ids: recommended_posts,
            },
        }
    }
    fn select_post(&self, engine: &RecommendationEngine) -> Option<Post> {
        if self.state.recommended_post_ids.is_empty() {
            return None;
        };
        let recommended_post_ids = &self.state.recommended_post_ids;

        let agent_vector = &self.core.interest_profile.vector_representation;

        let scored_recommendations: Vec<_> = recommended_post_ids
            .iter()
            .map(|id| {
                let post = engine.get_content_by_id(*id).unwrap();
                let similarity = engine.calculate_vector_similarity(
                    &agent_vector,
                    &post.interest_profile.vector_representation,
                );
                (post, similarity)
            })
            .collect();

        let total_similarity: f32 = scored_recommendations
            .iter()
            .map(|(_, similarity)| similarity)
            .sum();

        let mut random_value = random::<f32>() * total_similarity;

        let _ = scored_recommendations.iter().map(|(post, similarity)| {
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
                        ticks_required: IndividualCore::calculate_required_ticks(
                            first_comment.length,
                            self.individual_core.read_speed,
                        ),
                        potential_interest_gain: IndividualCore::calculate_interest_gain(
                            &self.core.interest_profile,
                            &first_comment.interest_profile,
                            engine,
                        ),
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
}

impl From<(IndividualAgent<Scrolling>, &RecommendationEngine)> for IndividualAgent<Scrolling> {
    fn from(
        (agent, engine): (IndividualAgent<Scrolling>, &RecommendationEngine),
    ) -> IndividualAgent<Scrolling> {
        let recommended_posts = engine.get_post_recommendations(
            &agent.core.interest_profile,
            &agent.individual_core.viewed_content,
            10,
            chrono::Utc::now().timestamp(),
        );
        IndividualAgent {
            individual_core: agent.individual_core,
            core: agent.core,
            state: Scrolling {
                recommended_post_ids: recommended_posts,
            },
        }
    }
}

impl TryFrom<(IndividualAgent<Scrolling>, &RecommendationEngine)> for IndividualAgent<ReadingPost> {
    type Error = TransitionError;

    fn try_from(
        (agent, engine): (IndividualAgent<Scrolling>, &RecommendationEngine),
    ) -> Result<IndividualAgent<ReadingPost>, Self::Error> {
        let post = agent
            .select_post(engine)
            .ok_or(TransitionError::NoPostAvailable)?;

        let read_speed = agent.individual_core.read_speed;
        let interest_profile = agent.core.interest_profile.clone();

        Ok(IndividualAgent {
            individual_core: agent.individual_core,
            core: agent.core,
            state: ReadingPost::new(&post, read_speed, &interest_profile, engine),
        })
    }
}

impl TryFrom<(IndividualAgent<Scrolling>, &RecommendationEngine)>
    for IndividualAgent<ReadingComments>
{
    type Error = TransitionError;

    fn try_from(
        (agent, engine): (IndividualAgent<Scrolling>, &RecommendationEngine),
    ) -> Result<IndividualAgent<ReadingComments>, Self::Error> {
        let post = agent
            .select_post(engine)
            .ok_or(TransitionError::NoPostAvailable)?;

        let selected_comments = engine
            .get_comment_recommendations(post.id, Vec::new(), 10)
            .ok_or(TransitionError::NoCommentsAvailable)?;

        let read_speed = agent.individual_core.read_speed;
        let interest_profile = agent.core.interest_profile.clone();

        Ok(IndividualAgent {
            individual_core: agent.individual_core,
            core: agent.core,
            state: ReadingComments::new(
                &post,
                selected_comments,
                read_speed,
                &interest_profile,
                engine,
            ),
        })
    }
}

impl From<(IndividualAgent<Scrolling>, &SimulationConfig)> for IndividualAgent<CreatingPost> {
    fn from(
        (agent, config): (IndividualAgent<Scrolling>, &SimulationConfig),
    ) -> IndividualAgent<CreatingPost> {
        let write_speed = agent.core.create_speed;
        IndividualAgent {
            core: agent.core,
            individual_core: agent.individual_core,
            state: CreatingPost::new(write_speed, config),
        }
    }
}

impl
    TryFrom<(
        IndividualAgent<Scrolling>,
        &SimulationConfig,
        &RecommendationEngine,
    )> for IndividualAgent<CreatingComment>
{
    type Error = TransitionError;

    fn try_from(
        (agent, config, engine): (
            IndividualAgent<Scrolling>,
            &SimulationConfig,
            &RecommendationEngine,
        ),
    ) -> Result<IndividualAgent<CreatingComment>, Self::Error> {
        let post = agent
            .select_post(engine)
            .ok_or(TransitionError::NoPostAvailable)?;

        Ok(IndividualAgent::<CreatingComment>::new(
            agent.individual_core,
            agent.core,
            &post,
            config,
        ))
    }
}

impl IndividualAgent<ReadingPost> {}

// ReadingPost -> Scrolling
impl From<(IndividualAgent<ReadingPost>, &RecommendationEngine)> for IndividualAgent<Scrolling> {
    fn from((agent, engine): (IndividualAgent<ReadingPost>, &RecommendationEngine)) -> Self {
        IndividualAgent::<Scrolling>::new(agent.individual_core, agent.core, engine)
    }
}

// ReadingPost -> ReadingComments
impl TryFrom<(IndividualAgent<ReadingPost>, &RecommendationEngine)>
    for IndividualAgent<ReadingComments>
{
    type Error = TransitionError;

    fn try_from(
        (agent, engine): (IndividualAgent<ReadingPost>, &RecommendationEngine),
    ) -> Result<Self, Self::Error> {
        let post =
            engine
                .get_content_by_id(agent.state.post_id)
                .ok_or(TransitionError::PostNotFound {
                    id: agent.state.post_id,
                })?;

        let selected_comments = engine
            .get_comment_recommendations(post.id, Vec::new(), 10)
            .ok_or(TransitionError::NoCommentsAvailable)?;

        Ok(IndividualAgent::<ReadingComments>::new(
            agent.individual_core,
            agent.core,
            post,
            selected_comments,
            engine,
        ))
    }
}

// ReadingPost -> CreatingComment
impl
    TryFrom<(
        IndividualAgent<ReadingPost>,
        &SimulationConfig,
        &RecommendationEngine,
    )> for IndividualAgent<CreatingComment>
{
    type Error = TransitionError;

    fn try_from(
        (agent, config, engine): (
            IndividualAgent<ReadingPost>,
            &SimulationConfig,
            &RecommendationEngine,
        ),
    ) -> Result<IndividualAgent<CreatingComment>, Self::Error> {
        let post =
            engine
                .get_content_by_id(agent.state.post_id)
                .ok_or(TransitionError::PostNotFound {
                    id: agent.state.post_id,
                })?;

        Ok(IndividualAgent::<CreatingComment>::new(
            agent.individual_core,
            agent.core,
            &post,
            config,
        ))
    }
}

impl IndividualAgent<ReadingComments> {
    fn new(
        individual_core: IndividualCore,
        core: AgentCore,
        post: &Post,
        selected_comments: Vec<&Comment>,
        engine: &RecommendationEngine,
    ) -> Self {
        let read_speed = individual_core.read_speed;
        let interest_profile = core.interest_profile.clone();
        IndividualAgent {
            individual_core,
            core,
            state: ReadingComments::new(
                &post,
                selected_comments,
                read_speed,
                &interest_profile,
                engine,
            ),
        }
    }
}

impl IndividualAgent<CreatingPost> {
    // fn new(
    //     individual_core: IndividualCore,
    //     core: AgentCore,
    //     engine: &RecommendationEngine,
    // ) -> Self {
    //     Individual {}
    // }

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

impl From<(IndividualAgent<CreatingPost>, &RecommendationEngine)> for IndividualAgent<Scrolling> {
    fn from((agent, engine): (IndividualAgent<CreatingPost>, &RecommendationEngine)) -> Self {
        IndividualAgent::<Scrolling>::new(agent.individual_core, agent.core, engine)
    }
}

impl From<IndividualAgent<CreatingPost>> for IndividualAgent<Offline> {
    fn from(val: IndividualAgent<CreatingPost>) -> Self {
        IndividualAgent {
            core: val.core,
            individual_core: val.individual_core,
            state: Offline {},
        }
    }
}

impl IndividualAgent<CreatingComment> {
    fn new(
        individual_core: IndividualCore,
        core: AgentCore,
        post: &Post,
        config: &SimulationConfig,
    ) -> Self {
        let create_speed = core.create_speed;
        IndividualAgent {
            core,
            individual_core,
            state: CreatingComment::new(create_speed, config, post.id),
        }
    }

    pub fn generate_comment(&self, config: &SimulationConfig) -> Comment {
        let selected_tags = self
            .core
            .interest_profile
            .select_content_tags(config.min_content_tags, config.max_content_tags);

        let content_profile = self.core.interest_profile.filtered_clone(&selected_tags);

        Comment {
            id: rand::thread_rng().next_u32() as usize,
            commentor_id: self.core.id,
            timestamp: chrono::Utc::now().timestamp(),
            interest_profile: content_profile,
            length: (random::<f32>() * config.max_post_length as f32) as i32,
            engagement_score: 0.0,
        }
    }
}
