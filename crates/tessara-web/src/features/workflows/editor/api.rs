//! Workflow editor mutation transport.

#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowPayload, CreateWorkflowRevisionPayload, UpdateWorkflowPayload,
    UpdateWorkflowRevisionStepsPayload,
};
#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, send_json_id_request};

#[cfg(feature = "hydrate")]
pub(super) async fn update_workflow(
    workflow_id: &str,
    payload: UpdateWorkflowPayload,
) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Update request could not be prepared.".to_string())?;
    let workflow_url = format!("/api/workflows/{workflow_id}");

    send_json_id_request(
        gloo_net::http::Request::put(&workflow_url),
        Some(body),
        "Update workflow",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn create_workflow(payload: CreateWorkflowPayload) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Create request could not be prepared.".to_string())?;

    send_json_id_request(
        gloo_net::http::Request::post("/api/workflows"),
        Some(body),
        "Create workflow",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn create_initial_workflow_revision(
    workflow_id: &str,
    payload: CreateWorkflowRevisionPayload,
) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Workflow step request could not be prepared.".to_string())?;
    let version_url = format!("/api/workflows/{workflow_id}/versions");

    send_json_id_request(
        gloo_net::http::Request::post(&version_url),
        Some(body),
        "Create workflow steps",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_workflow_revision_steps(
    version_id: &str,
    payload: UpdateWorkflowRevisionStepsPayload,
) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Workflow step update request could not be prepared.".to_string())?;
    let steps_url = format!("/api/workflow-versions/{version_id}/steps");

    send_json_id_request(
        gloo_net::http::Request::put(&steps_url),
        Some(body),
        "Update workflow steps",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn create_workflow_revision(
    workflow_id: &str,
    payload: CreateWorkflowRevisionPayload,
) -> Result<IdResponse, String> {
    let body = serde_json::to_string(&payload)
        .map_err(|_| "Workflow revision request could not be prepared.".to_string())?;
    let version_url = format!("/api/workflows/{workflow_id}/versions");

    send_json_id_request(
        gloo_net::http::Request::post(&version_url),
        Some(body),
        "Create workflow revision",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn publish_workflow_revision(version_id: &str) -> Result<IdResponse, String> {
    let publish_url = format!("/api/workflow-versions/{version_id}/publish");

    send_json_id_request(
        gloo_net::http::Request::post(&publish_url),
        None,
        "Publish workflow revision",
    )
    .await
}
