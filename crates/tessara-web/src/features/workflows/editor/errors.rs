//! Error types for workflow editor mutations.

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowEditorMutationError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowEditorMutationError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }

    pub(super) fn from_transport_error(error: String) -> Self {
        if error == "Authentication is required." {
            Self::Unauthorized
        } else {
            Self::Message(error)
        }
    }
}
