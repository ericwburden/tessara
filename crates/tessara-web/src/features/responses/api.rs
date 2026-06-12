//! Transport calls for the Responses feature.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders and actions.

#[cfg(feature = "hydrate")]
use crate::features::forms::RenderedForm;
#[cfg(feature = "hydrate")]
use crate::features::responses::types::{
    AssignmentResponseStartOptions, SaveSubmissionValuesPayload, SubmissionDetail,
    SubmissionSummary,
};
#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, send_json_request};

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

    pub(super) fn from_transport_error(error: String) -> Self {
        if error == "Authentication is required." {
            Self::Unauthorized
        } else {
            Self::Message(error)
        }
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_submissions() -> Result<Vec<SubmissionSummary>, ResponseApiError> {
    match gloo_net::http::Request::get("/api/submissions")
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(ResponseApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<Vec<SubmissionSummary>>()
                .await
                .map_err(|error| {
                    ResponseApiError::message(format!("Unable to parse responses: {error}"))
                })
        }
        Ok(response) => Err(ResponseApiError::message(format!(
            "Unable to load responses. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(ResponseApiError::message(format!(
            "Unable to load responses: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_submission_detail(
    submission_id: &str,
) -> Result<SubmissionDetail, ResponseApiError> {
    match gloo_net::http::Request::get(&format!("/api/submissions/{submission_id}"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(ResponseApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response.json::<SubmissionDetail>().await.map_err(|error| {
                ResponseApiError::message(format!("Unable to parse response: {error}"))
            })
        }
        Ok(response) => Err(ResponseApiError::message(format!(
            "Unable to load response. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(ResponseApiError::message(format!(
            "Unable to load response: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_rendered_form(
    form_version_id: &str,
) -> Result<RenderedForm, ResponseApiError> {
    match gloo_net::http::Request::get(&format!("/api/form-versions/{form_version_id}/render"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(ResponseApiError::Unauthorized),
        Ok(response) if response.ok() => response.json::<RenderedForm>().await.map_err(|error| {
            ResponseApiError::message(format!("Unable to parse response form: {error}"))
        }),
        Ok(response) => Err(ResponseApiError::message(format!(
            "Unable to load response form. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(ResponseApiError::message(format!(
            "Unable to load response form: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_response_start_options(
    delegate_account_id: Option<&str>,
) -> Result<AssignmentResponseStartOptions, ResponseApiError> {
    let path = delegate_account_id
        .filter(|value| !value.trim().is_empty())
        .map(|value| format!("/api/responses/options?delegate_account_id={value}"))
        .unwrap_or_else(|| "/api/responses/options".to_string());

    match gloo_net::http::Request::get(&path).send().await {
        Ok(response) if response.status() == 401 => Err(ResponseApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<AssignmentResponseStartOptions>()
            .await
            .map_err(|error| {
                ResponseApiError::message(format!(
                    "Unable to parse assigned response start options: {error}"
                ))
            }),
        Ok(response) => Err(ResponseApiError::message(format!(
            "Unable to load assigned response start options. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(ResponseApiError::message(format!(
            "Unable to load assigned response start options: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn start_assignment_response(
    workflow_assignment_id: &str,
) -> Result<Option<String>, ResponseApiError> {
    let response = send_json_request::<serde_json::Value>(
        gloo_net::http::Request::post(&format!(
            "/api/workflow-assignments/{workflow_assignment_id}/start"
        )),
        Some("{}".into()),
        "Start assigned response",
    )
    .await
    .map_err(ResponseApiError::from_transport_error)?;

    Ok(response
        .get("id")
        .and_then(|value| value.as_str().map(str::to_owned))
        .or_else(|| {
            response
                .get("id")
                .and_then(|value| value.as_i64().map(|value| value.to_string()))
        }))
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
