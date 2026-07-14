//! Error type for organization tree transport.

#[cfg(feature = "hydrate")]
pub(super) enum OrganizationTreeApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl OrganizationTreeApiError {
    pub(super) fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}
