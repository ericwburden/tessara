//! Transport calls for workflow assignments.
//!
//! Keep endpoint requests and response parsing here; Leptos signal orchestration belongs in loaders.

#[cfg(feature = "hydrate")]
use crate::features::workflows::assignments::types::{
    PendingWorkflowWork, WorkflowAssigneeOption, WorkflowAssignmentCandidate,
    WorkflowAssignmentSummary,
};

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowAssignmentApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowAssignmentApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

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
