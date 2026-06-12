//! Transport calls for workflow pages.
//!
//! Keep endpoint requests and response parsing here; signal orchestration belongs in loaders and actions.

#[cfg(feature = "hydrate")]
use crate::features::organization::OrganizationNode;
#[cfg(feature = "hydrate")]
use crate::features::workflows::types::{WorkflowDefinition, WorkflowSummary};

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl WorkflowApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflows() -> Result<Vec<WorkflowSummary>, WorkflowApiError> {
    match gloo_net::http::Request::get("/api/workflows").send().await {
        Ok(response) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<Vec<WorkflowSummary>>()
                .await
                .map_err(|error| {
                    WorkflowApiError::message(format!("Unable to parse workflows: {error}"))
                })
        }
        Ok(response) => Err(WorkflowApiError::message(format!(
            "Unable to load workflows. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowApiError::message(format!(
            "Unable to load workflows: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignment_nodes()
-> Result<Vec<OrganizationNode>, WorkflowApiError> {
    match gloo_net::http::Request::get("/api/nodes").send().await {
        Ok(response) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<Vec<OrganizationNode>>()
                .await
                .map_err(|error| {
                    WorkflowApiError::message(format!(
                        "Unable to parse workflow assignment nodes: {error}"
                    ))
                })
        }
        Ok(response) => Err(WorkflowApiError::message(format!(
            "Unable to load workflow assignment nodes. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowApiError::message(format!(
            "Unable to load workflow assignment nodes: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_detail(
    workflow_id: &str,
) -> Result<WorkflowDefinition, WorkflowApiError> {
    match gloo_net::http::Request::get(&format!("/api/workflows/{workflow_id}"))
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<WorkflowDefinition>()
                .await
                .map_err(|error| {
                    WorkflowApiError::message(format!("Unable to parse workflow detail: {error}"))
                })
        }
        Ok(response) => Err(WorkflowApiError::message(format!(
            "Unable to load workflow detail. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(WorkflowApiError::message(format!(
            "Unable to load workflow detail: {error}"
        ))),
    }
}
