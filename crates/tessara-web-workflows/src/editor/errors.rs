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

    pub(super) fn from_transport_error(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}
