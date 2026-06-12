//! Transport calls for form save operations.

use crate::features::forms::types::{CreateFormPayload, UpdateFormPayload};
use crate::http::{IdResponse, send_json_id_request};

pub(super) enum FormSaveApiError {
    Unauthorized,
    Message(String),
}

impl FormSaveApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

pub(super) async fn create_form(
    payload: CreateFormPayload,
) -> Result<IdResponse, FormSaveApiError> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| FormSaveApiError::message("Create request could not be prepared."))?;

    let response = gloo_net::http::Request::post("/api/admin/forms")
        .header("Content-Type", "application/json")
        .body(body)
        .expect("json request body should be valid")
        .send()
        .await;

    match response {
        Ok(response) if response.status() == 401 => Err(FormSaveApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<IdResponse>()
            .await
            .map_err(|_| FormSaveApiError::message("Create response could not be read.")),
        Ok(response) => Err(FormSaveApiError::message(format!(
            "Create failed with status {}.",
            response.status()
        ))),
        Err(_) => Err(FormSaveApiError::message(
            "Could not reach the create form API.",
        )),
    }
}

pub(super) async fn create_initial_form_version(
    form_id: &str,
) -> Result<IdResponse, FormSaveApiError> {
    let response = gloo_net::http::Request::post(&format!("/api/admin/forms/{form_id}/versions"))
        .header("Content-Type", "application/json")
        .body("{}")
        .expect("json request body should be valid")
        .send()
        .await;

    match response {
        Ok(response) if response.status() == 401 => Err(FormSaveApiError::Unauthorized),
        Ok(response) if response.ok() => response.json::<IdResponse>().await.map_err(|_| {
            FormSaveApiError::message(
                "Form was created, but draft version response could not be read.",
            )
        }),
        Ok(response) => Err(FormSaveApiError::message(format!(
            "Form was created, but draft version setup failed with status {}.",
            response.status()
        ))),
        Err(_) => Err(FormSaveApiError::message(
            "Form was created, but the draft version API could not be reached.",
        )),
    }
}

pub(super) async fn update_form(
    form_id: &str,
    payload: UpdateFormPayload,
) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Update request could not be prepared.".to_string())?;

    send_json_id_request(
        gloo_net::http::Request::put(&format!("/api/admin/forms/{form_id}")),
        Some(body),
        "Update form",
    )
    .await
}

pub(super) async fn create_draft_form_version(form_id: &str) -> Result<IdResponse, String> {
    send_json_id_request(
        gloo_net::http::Request::post(&format!("/api/admin/forms/{form_id}/versions")),
        Some("{}".into()),
        "Create draft version",
    )
    .await
}

pub(super) async fn delete_form_field(field_id: &str) -> Result<IdResponse, String> {
    send_json_id_request(
        gloo_net::http::Request::delete(&format!("/api/admin/form-fields/{field_id}")),
        None,
        "Delete form field",
    )
    .await
}

pub(super) async fn delete_form_section(section_id: &str) -> Result<IdResponse, String> {
    send_json_id_request(
        gloo_net::http::Request::delete(&format!("/api/admin/form-sections/{section_id}")),
        None,
        "Delete form section",
    )
    .await
}

pub(super) async fn publish_form_version(version_id: &str) -> Result<IdResponse, String> {
    send_json_id_request(
        gloo_net::http::Request::post(&format!("/api/admin/form-versions/{version_id}/publish")),
        None,
        "Publish form version",
    )
    .await
}
