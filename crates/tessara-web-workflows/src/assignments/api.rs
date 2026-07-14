//! Transport calls for workflow assignments.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders.

#[cfg(feature = "hydrate")]
use super::errors::{WorkflowAssignmentApiError, WorkflowAssignmentMutationError};
#[cfg(feature = "hydrate")]
use crate::assignments::types::{
    BulkWorkflowAssignmentPayload, UpdateWorkflowAssignmentPayload, WorkflowAssigneeOption,
    WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};

#[cfg(feature = "hydrate")]
pub(super) async fn create_workflow_assignments_bulk(
    payload: BulkWorkflowAssignmentPayload,
) -> Result<(), WorkflowAssignmentMutationError> {
    tessara_web_http::send_json_without_response(
        gloo_net::http::Request::post("/api/workflow-assignments/bulk"),
        &payload,
        "Create workflow assignments",
    )
    .await
    .map_err(WorkflowAssignmentMutationError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_workflow_assignment(
    assignment_id: &str,
    payload: UpdateWorkflowAssignmentPayload,
) -> Result<(), WorkflowAssignmentMutationError> {
    tessara_web_http::send_json_without_response(
        gloo_net::http::Request::put(&format!("/api/workflow-assignments/{assignment_id}")),
        &payload,
        "Update workflow assignment",
    )
    .await
    .map_err(WorkflowAssignmentMutationError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignments()
-> Result<Vec<WorkflowAssignmentSummary>, WorkflowAssignmentApiError> {
    tessara_web_http::fetch_json("/api/workflow-assignments", "Workflow assignments")
        .await
        .map_err(WorkflowAssignmentApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignment_candidates()
-> Result<Vec<WorkflowAssignmentCandidate>, WorkflowAssignmentApiError> {
    tessara_web_http::fetch_json(
        "/api/workflow-assignment-candidates",
        "Workflow assignment candidates",
    )
    .await
    .map_err(WorkflowAssignmentApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignment_assignees(
    workflow_version_id: &str,
    node_id: &str,
) -> Result<Vec<WorkflowAssigneeOption>, WorkflowAssignmentApiError> {
    let url = format!(
        "/api/workflow-assignment-candidates/assignees?workflow_version_id={workflow_version_id}&node_id={node_id}"
    );

    tessara_web_http::fetch_json(&url, "Workflow assignment assignees")
        .await
        .map_err(WorkflowAssignmentApiError::from_request)
}
