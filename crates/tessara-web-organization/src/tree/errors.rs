//! Error type for organization tree transport.

#[cfg(feature = "hydrate")]
pub(super) enum OrganizationTreeApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl OrganizationTreeApiError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}
