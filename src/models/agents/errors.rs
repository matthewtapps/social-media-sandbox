use std::fmt;

#[derive(Debug)]
pub enum TransitionError {
    // When we try to read a post but none are available
    NoPostAvailable,

    // When we look up a post by ID but fail
    PostNotFound {
        id: usize,
    },

    // When we try to read comments but the post has none
    NoCommentsAvailable,

    // When we try to transition to a state but prerequisites aren't met
    InvalidTransition {
        from_state: &'static str,
        to_state: &'static str,
        reason: String,
    },

    // When recommendation engine operations fail
    RecommendationError {
        operation: String,
        error: String,
    },

    // When we exceed session length limits
    SessionExpired {
        current_ticks: i32,
        max_ticks: i32,
    },

    // For unexpected runtime errors
    InternalError(String),
}

// Implement standard error handling traits
impl std::error::Error for TransitionError {}

impl fmt::Display for TransitionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransitionError::NoPostAvailable => {
                write!(f, "No posts available for reading")
            }
            TransitionError::PostNotFound { id } => {
                write!(f, "Post not found with id {}", id)
            }
            TransitionError::NoCommentsAvailable => {
                write!(f, "No comments available on the selected post")
            }
            TransitionError::InvalidTransition {
                from_state,
                to_state,
                reason,
            } => {
                write!(
                    f,
                    "Invalid transition from {} to {}: {}",
                    from_state, to_state, reason
                )
            }
            TransitionError::RecommendationError { operation, error } => {
                write!(
                    f,
                    "Recommendation engine error during {}: {}",
                    operation, error
                )
            }
            TransitionError::SessionExpired {
                current_ticks,
                max_ticks,
            } => {
                write!(
                    f,
                    "Session expired after {} ticks (max: {})",
                    current_ticks, max_ticks
                )
            }
            TransitionError::InternalError(msg) => {
                write!(f, "Internal error: {}", msg)
            }
        }
    }
}
