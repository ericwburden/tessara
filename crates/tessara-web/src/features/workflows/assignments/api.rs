//! Transport calls for workflow assignments.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders.

#[cfg(feature = "hydrate")]
use super::errors::{WorkflowAssignmentApiError, WorkflowAssignmentMutationError};
#[cfg(feature = "hydrate")]
use crate::features::workflows::assignments::types::{
    BulkWorkflowAssignmentPayload, PendingWorkflowWork, UpdateWorkflowAssignmentPayload,
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_pending_work()
-> Result<Vec<PendingWorkflowWork>, WorkflowAssignmentApiError> {
    match gloo_net::http::Request::get("/api/workflow-assignments/pending")
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(WorkflowAssignmentApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<Vec<PendingWorkflowWork>>()
                .await
                .map_err(|error| {
                    WorkflowAssignmentApiError::message(format!(
                        "Unable to parse assigned work: {error}"
                    ))
                })
        }
        Ok(response) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load assigned work. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load assigned work: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn create_workflow_assignments_bulk(
    payload: BulkWorkflowAssignmentPayload,
) -> Result<(), WorkflowAssignmentMutationError> {
    let body = serde_json::to_string(&payload).map_err(|_| {
        WorkflowAssignmentMutationError::message("Assignment request could not be prepared.")
    })?;

    let response = gloo_net::http::Request::post("/api/workflow-assignments/bulk")
        .header("Content-Type", "application/json")
        .body(body)
        .map_err(|_| {
            WorkflowAssignmentMutationError::message("Assignment request could not be prepared.")
        })?
        .send()
        .await;

    match response {
        Ok(response) if response.status() == 401 => {
            Err(WorkflowAssignmentMutationError::Unauthorized)
        }
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => Err(WorkflowAssignmentMutationError::message(format!(
            "Create assignments failed with status {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowAssignmentMutationError::message(format!(
            "Could not reach the assignments API: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_workflow_assignment(
    assignment_id: &str,
    payload: UpdateWorkflowAssignmentPayload,
) -> Result<(), WorkflowAssignmentMutationError> {
    let body = serde_json::to_string(&payload).map_err(|_| {
        WorkflowAssignmentMutationError::message("Update request could not be prepared.")
    })?;

    let response =
        gloo_net::http::Request::put(&format!("/api/workflow-assignments/{assignment_id}"))
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|_| {
                WorkflowAssignmentMutationError::message("Update request could not be prepared.")
            })?
            .send()
            .await;

    match response {
        Ok(response) if response.status() == 401 => {
            Err(WorkflowAssignmentMutationError::Unauthorized)
        }
        Ok(response) if response.ok() => Ok(()),
        Ok(response) => Err(WorkflowAssignmentMutationError::message(format!(
            "Update assignment failed with status {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowAssignmentMutationError::message(format!(
            "Could not reach the assignments API: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignments()
-> Result<Vec<WorkflowAssignmentSummary>, WorkflowAssignmentApiError> {
    match gloo_net::http::Request::get("/api/workflow-assignments")
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(WorkflowAssignmentApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<Vec<WorkflowAssignmentSummary>>()
            .await
            .map_err(|error| {
                WorkflowAssignmentApiError::message(format!(
                    "Unable to parse workflow assignments: {error}"
                ))
            }),
        Ok(response) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load workflow assignments. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load workflow assignments: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignment_candidates()
-> Result<Vec<WorkflowAssignmentCandidate>, WorkflowAssignmentApiError> {
    match gloo_net::http::Request::get("/api/workflow-assignment-candidates")
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(WorkflowAssignmentApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<Vec<WorkflowAssignmentCandidate>>()
            .await
            .map_err(|error| {
                WorkflowAssignmentApiError::message(format!(
                    "Unable to parse assignment candidates: {error}"
                ))
            }),
        Ok(response) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load assignment candidates. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load assignment candidates: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignment_assignees(
    workflow_version_id: &str,
    node_id: &str,
) -> Result<Vec<WorkflowAssigneeOption>, WorkflowAssignmentApiError> {
    let url = format!(
        "/api/workflow-assignment-candidates/assignees?workflow_version_id={workflow_version_id}&node_id={node_id}"
    );

    match gloo_net::http::Request::get(&url).send().await {
        Ok(response) if response.status() == 401 => Err(WorkflowAssignmentApiError::Unauthorized),
        Ok(response) if response.ok() => response
            .json::<Vec<WorkflowAssigneeOption>>()
            .await
            .map_err(|error| {
                WorkflowAssignmentApiError::message(format!(
                    "Unable to parse eligible assignees: {error}"
                ))
            }),
        Ok(response) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load eligible assignees. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowAssignmentApiError::message(format!(
            "Unable to load eligible assignees: {error}"
        ))),
    }
}
