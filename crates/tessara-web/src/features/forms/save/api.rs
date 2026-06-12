//! Transport calls for form save operations.

use crate::features::forms::types::UpdateFormPayload;
use crate::http::{IdResponse, send_json_id_request};

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
