//! Error types shared by workflow assignment transport and orchestration.

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowAssignmentApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowAssignmentApiError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowAssignmentMutationError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowAssignmentMutationError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
