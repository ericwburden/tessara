//! Transport calls for the Responses feature.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders and actions.

#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, send_json_request};
#[cfg(feature = "hydrate")]
use crate::types::{
    AssignmentResponseStartOptions, RenderedForm, SaveSubmissionValuesPayload, SubmissionDetail,
    SubmissionSummary,
};

#[cfg(feature = "hydrate")]
pub(super) enum ResponseApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl ResponseApiError {
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

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_submissions() -> Result<Vec<SubmissionSummary>, ResponseApiError> {
    tessara_web_http::fetch_json("/api/submissions", "Responses")
        .await
        .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_submission_detail(
    submission_id: &str,
) -> Result<SubmissionDetail, ResponseApiError> {
    tessara_web_http::fetch_json(
        &format!("/api/submissions/{submission_id}"),
        "Response detail",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_rendered_form(
    form_version_id: &str,
) -> Result<RenderedForm, ResponseApiError> {
    tessara_web_http::fetch_json(
        &format!("/api/form-versions/{form_version_id}/render"),
        "Response form",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_response_start_options(
    delegate_account_id: Option<&str>,
) -> Result<AssignmentResponseStartOptions, ResponseApiError> {
    let path = delegate_account_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("/api/responses/options?delegate_account_id={value}"))
        .unwrap_or_else(|| "/api/responses/options".to_string());

    tessara_web_http::fetch_json(&path, "Assigned response start options")
        .await
        .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn start_assignment_response(
    workflow_assignment_id: &str,
) -> Result<String, ResponseApiError> {
    let response = send_json_request::<serde_json::Value>(
        gloo_net::http::Request::post(&format!(
            "/api/workflow-assignments/{workflow_assignment_id}/start"
        )),
        Some("{}".into()),
        "Start assigned response",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)?;

    response
        .get("id")
        .and_then(|value| value.as_str().map(str::to_owned))
        .or_else(|| {
            response
                .get("id")
                .and_then(|value| value.as_i64().map(|value| value.to_string()))
        })
        .ok_or_else(|| {
            ResponseApiError::message(
                "Assigned response was started, but the response id was missing.",
            )
        })
}

#[cfg(feature = "hydrate")]
pub(super) async fn save_submission_values_api(
    submission_id: &str,
    payload: SaveSubmissionValuesPayload,
) -> Result<IdResponse, ResponseApiError> {
    let body = serde_json::to_string(&payload).map_err(|error| {
        ResponseApiError::message(format!("Response values could not be prepared: {error}"))
    })?;

    send_json_request::<IdResponse>(
        gloo_net::http::Request::put(&format!("/api/submissions/{submission_id}/values")),
        Some(body),
        "Save response draft",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn submit_submission_api(
    submission_id: &str,
) -> Result<IdResponse, ResponseApiError> {
    send_json_request::<IdResponse>(
        gloo_net::http::Request::post(&format!("/api/submissions/{submission_id}/submit")),
        Some("{}".into()),
        "Submit response",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)
}
