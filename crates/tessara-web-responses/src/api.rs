//! Transport calls for the Responses feature.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders and actions.

#[cfg(feature = "hydrate")]
use crate::http::send_json_request;
#[cfg(feature = "hydrate")]
use crate::types::{
    ResponseDetail, ResponseRevisionRequest, ResponseStartOptions, ResponseSummary,
    SaveResponseValuesRequest,
};
#[cfg(feature = "hydrate")]
use tessara_responses_contract::{ResponseMutationResult, StartResponseRequest};
#[cfg(feature = "hydrate")]
use uuid::Uuid;

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
pub(super) async fn fetch_responses() -> Result<Vec<ResponseSummary>, ResponseApiError> {
    tessara_web_http::fetch_json("/api/responses", "Responses")
        .await
        .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_response_detail(
    response_id: &str,
) -> Result<ResponseDetail, ResponseApiError> {
    tessara_web_http::fetch_json(&format!("/api/responses/{response_id}"), "Response detail")
        .await
        .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_response_start_options(
    delegate_account_id: Option<&str>,
) -> Result<ResponseStartOptions, ResponseApiError> {
    let path = delegate_account_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("/api/responses/start-options?delegate_account_id={value}"))
        .unwrap_or_else(|| "/api/responses/start-options".to_string());

    tessara_web_http::fetch_json(&path, "Assigned response start options")
        .await
        .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn start_assignment_response(
    workflow_assignment_id: Uuid,
) -> Result<Uuid, ResponseApiError> {
    let body = serde_json::to_string(&StartResponseRequest {
        workflow_assignment_id,
    })
    .map_err(|error| ResponseApiError::message(error.to_string()))?;
    let response = send_json_request::<ResponseMutationResult>(
        gloo_net::http::Request::post("/api/responses"),
        Some(body),
        "Start assigned response",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)?;

    Ok(response.id)
}

#[cfg(feature = "hydrate")]
pub(super) async fn save_response_values_api(
    response_id: Uuid,
    payload: SaveResponseValuesRequest,
) -> Result<ResponseMutationResult, ResponseApiError> {
    let body = serde_json::to_string(&payload).map_err(|error| {
        ResponseApiError::message(format!("Response values could not be prepared: {error}"))
    })?;

    send_json_request::<ResponseMutationResult>(
        gloo_net::http::Request::put(&format!("/api/responses/{response_id}/values")),
        Some(body),
        "Save response draft",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)
}

#[cfg(feature = "hydrate")]
pub(super) async fn submit_response_api(
    response_id: Uuid,
    expected_revision: u64,
) -> Result<ResponseMutationResult, ResponseApiError> {
    let body = serde_json::to_string(&ResponseRevisionRequest { expected_revision })
        .map_err(|error| ResponseApiError::message(error.to_string()))?;
    send_json_request::<ResponseMutationResult>(
        gloo_net::http::Request::post(&format!("/api/responses/{response_id}/submit")),
        Some(body),
        "Submit response",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)
}
