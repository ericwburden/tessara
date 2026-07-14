//! Transport calls for the Forms feature.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders and save actions.

mod options;

#[cfg(feature = "hydrate")]
use crate::{FormDefinition, FormSummary, RenderedForm};

#[cfg(feature = "hydrate")]
pub(super) use options::{fetch_form_create_options, fetch_form_edit_options};

#[cfg(feature = "hydrate")]
pub(super) enum FormsApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl FormsApiError {
    fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_forms() -> Result<Vec<FormSummary>, FormsApiError> {
    tessara_web_http::fetch_json("/api/forms", "Forms")
        .await
        .map_err(FormsApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_form_detail(form_id: &str) -> Result<FormDefinition, FormsApiError> {
    tessara_web_http::fetch_json(&format!("/api/forms/{form_id}"), "Form detail")
        .await
        .map_err(FormsApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_rendered_form_version(
    form_version_id: &str,
) -> Result<RenderedForm, FormsApiError> {
    tessara_web_http::fetch_json(
        &format!("/api/form-versions/{form_version_id}/render"),
        "Rendered form",
    )
    .await
    .map_err(FormsApiError::from_request)
}
