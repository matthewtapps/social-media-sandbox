use rand::random;

use crate::models::{
    CreatingComment, CreatingPost, Offline, ReadingComments, ReadingPost, Scrolling,
};

use super::IndividualAgent;
// Define enums for each possible transition decision
#[derive(Debug, Clone)]
pub enum OfflineTransition {
    ToScrolling,
}

#[derive(Debug, Clone)]
pub enum ScrollingTransition {
    Refresh,
    ToReadingPost,
    ToReadingComments,
    ToCreatingPost,
    ToCreatingComment,
    ToOffline,
}

#[derive(Debug, Clone)]
pub enum ReadingPostTransition {
    Continue,
    ToScrolling,
    ToReadingComments,
    ToCreatingComment,
    ToOffline,
}

#[derive(Debug, Clone)]
pub enum ReadingCommentsTransition {
    Continue,
    ToScrolling,
    ToReadingPost,
    ToCreatingComment,
    ToOffline,
}

#[derive(Debug, Clone)]
pub enum CreatingPostTransition {
    Continue,
    ToScrolling,
    ToOffline,
}

#[derive(Debug, Clone)]
pub enum CreatingCommentTransition {
    Continue,
    ToScrolling,
    ToOffline,
}

pub trait TransitionPolicy {
    fn decide_offline_transition(&self, agent: &IndividualAgent<Offline>) -> OfflineTransition;
    fn decide_scrolling_transition(
        &self,
        agent: &IndividualAgent<Scrolling>,
    ) -> ScrollingTransition;
    fn decide_reading_post_transition(
        &self,
        agent: &IndividualAgent<ReadingPost>,
    ) -> ReadingPostTransition;
    fn decide_reading_comments_transition(
        &self,
        agent: &IndividualAgent<ReadingComments>,
    ) -> ReadingCommentsTransition;
    fn decide_creating_post_transition(
        &self,
        agent: &IndividualAgent<CreatingPost>,
    ) -> CreatingPostTransition;
    fn decide_creating_comment_transition(
        &self,
        agent: &IndividualAgent<CreatingComment>,
    ) -> CreatingCommentTransition;
}

pub struct DefaultPolicy;

impl TransitionPolicy for DefaultPolicy {
    fn decide_offline_transition(&self, _agent: &IndividualAgent<Offline>) -> OfflineTransition {
        OfflineTransition::ToScrolling
    }

    fn decide_scrolling_transition(
        &self,
        agent: &IndividualAgent<Scrolling>,
    ) -> ScrollingTransition {
        // Decision logic based on individual properties and randomness
        let random_val = random::<f32>();

        // Use agent properties to influence decision
        let post_threshold = 0.6 * agent.individual_core.next_post_likelihood;

        if random_val < post_threshold {
            ScrollingTransition::ToReadingPost
        } else if random_val < 0.8 {
            ScrollingTransition::ToReadingComments
        } else if random_val < 0.9 {
            ScrollingTransition::ToCreatingPost
        } else if random_val < 0.95 {
            ScrollingTransition::ToCreatingComment
        } else {
            ScrollingTransition::ToOffline
        }
    }

    fn decide_reading_post_transition(
        &self,
        agent: &IndividualAgent<ReadingPost>,
    ) -> ReadingPostTransition {
        // Check if reading is complete
        if agent.state.ticks_spent >= agent.state.ticks_required {
            // Reading complete, decide next action
            let random_val = random::<f32>();

            if random_val < 0.4 {
                ReadingPostTransition::ToReadingComments
            } else if random_val < 0.6 {
                ReadingPostTransition::ToCreatingComment
            } else if random_val < 0.95 {
                ReadingPostTransition::ToScrolling
            } else {
                ReadingPostTransition::ToOffline
            }
        } else {
            // Not done reading yet
            ReadingPostTransition::Continue
        }
    }

    fn decide_reading_comments_transition(
        &self,
        agent: &IndividualAgent<ReadingComments>,
    ) -> ReadingCommentsTransition {
        // Check if reading current comment is complete
        if agent.state.ticks_spent >= agent.state.ticks_required {
            // If there are more comments, move to next one (represented as Continue)
            if agent.state.current_comment_index < agent.state.current_comment_ids.len() - 1 {
                ReadingCommentsTransition::Continue
            } else {
                // Done with all comments, decide next action
                let random_val = random::<f32>();

                if random_val < 0.2 {
                    ReadingCommentsTransition::ToCreatingComment
                } else if random_val < 0.9 {
                    ReadingCommentsTransition::ToScrolling
                } else {
                    ReadingCommentsTransition::ToOffline
                }
            }
        } else {
            // Still reading current comment
            ReadingCommentsTransition::Continue
        }
    }

    fn decide_creating_post_transition(
        &self,
        agent: &IndividualAgent<CreatingPost>,
    ) -> CreatingPostTransition {
        // Check if post creation is complete
        if agent.state.ticks_spent >= agent.state.ticks_required {
            // Post creation complete, decide next action
            let random_val = random::<f32>();

            if random_val < 0.8 {
                CreatingPostTransition::ToScrolling
            } else {
                CreatingPostTransition::ToOffline
            }
        } else {
            // Still creating post
            CreatingPostTransition::Continue
        }
    }

    fn decide_creating_comment_transition(
        &self,
        agent: &IndividualAgent<CreatingComment>,
    ) -> CreatingCommentTransition {
        // Check if comment creation is complete
        if agent.state.ticks_spent >= agent.state.ticks_required {
            // Comment creation complete, decide next action
            let random_val = random::<f32>();

            if random_val < 0.8 {
                CreatingCommentTransition::ToScrolling
            } else {
                CreatingCommentTransition::ToOffline
            }
        } else {
            // Still creating comment
            CreatingCommentTransition::Continue
        }
    }
}
