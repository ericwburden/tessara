//! Error types shared by workflow assignment transport and orchestration.

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowAssignmentApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowAssignmentApiError {
    pub(super) fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowAssignmentMutationError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowAssignmentMutationError {
    pub(super) fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}
