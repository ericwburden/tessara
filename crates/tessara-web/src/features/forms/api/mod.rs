//! Transport calls for the Forms feature.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders and save actions.

mod options;

#[cfg(feature = "hydrate")]
use crate::features::forms::{FormDefinition, FormSummary, RenderedForm};

#[cfg(feature = "hydrate")]
pub(super) use options::{fetch_form_create_options, fetch_form_edit_options};

#[cfg(feature = "hydrate")]
pub(super) enum FormsApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl FormsApiError {
    pub(super) fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_forms() -> Result<Vec<FormSummary>, FormsApiError> {
    match gloo_net::http::Request::get("/api/forms").send().await {
        Ok(response) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<Vec<FormSummary>>()
            .await
            .map_err(|error| FormsApiError::message(format!("Unable to parse forms: {error}"))),
        Ok(response) => Err(FormsApiError::message(format!(
            "Unable to load forms. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(FormsApiError::message(format!(
            "Unable to load forms: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_form_detail(form_id: &str) -> Result<FormDefinition, FormsApiError> {
    match gloo_net::http::Request::get(&format!("/api/forms/{form_id}"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        Ok(response) if response.ok() => response.json::<FormDefinition>().await.map_err(|error| {
            FormsApiError::message(format!("Unable to parse form detail: {error}"))
        }),
        Ok(response) => Err(FormsApiError::message(format!(
            "Unable to load form detail. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(FormsApiError::message(format!(
            "Unable to load form detail: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_rendered_form_version(
    form_version_id: &str,
) -> Result<RenderedForm, FormsApiError> {
    match gloo_net::http::Request::get(&format!("/api/form-versions/{form_version_id}/render"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(FormsApiError::Unauthorized),
        Ok(response) if response.ok() => response.json::<RenderedForm>().await.map_err(|error| {
            FormsApiError::message(format!("Unable to parse rendered form: {error}"))
        }),
        Ok(response) => Err(FormsApiError::message(format!(
            "Unable to load rendered form. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(FormsApiError::message(format!(
            "Unable to load rendered form: {error}"
        ))),
    }
}
