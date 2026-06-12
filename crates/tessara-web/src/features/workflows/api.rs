//! Transport calls for workflow pages.
//!
//! Keep endpoint requests and response parsing here; signal orchestration belongs in loaders and actions.

#[cfg(feature = "hydrate")]
use crate::features::forms::FormSummary;
#[cfg(feature = "hydrate")]
use crate::features::organization::NodeTypeCatalogEntry;
#[cfg(feature = "hydrate")]
use crate::features::organization::OrganizationNode;
#[cfg(feature = "hydrate")]
use crate::features::workflows::types::{WorkflowDefinition, WorkflowSummary};
#[cfg(feature = "hydrate")]
use crate::features::workflows::{
    CreateWorkflowPayload, CreateWorkflowRevisionPayload, UpdateWorkflowPayload,
    UpdateWorkflowRevisionStepsPayload,
};
#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, send_json_id_request};

#[cfg(feature = "hydrate")]
pub(super) enum WorkflowApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
pub(super) struct WorkflowEditorOptionResponses {
    pub(super) node_types: Vec<NodeTypeCatalogEntry>,
    pub(super) organization_nodes: Vec<OrganizationNode>,
    pub(super) forms: Vec<FormSummary>,
    pub(super) workflows: Vec<WorkflowSummary>,
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

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_editor_options()
-> Result<WorkflowEditorOptionResponses, WorkflowApiError> {
    let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
    let nodes_response = gloo_net::http::Request::get("/api/nodes").send().await;
    let forms_response = gloo_net::http::Request::get("/api/forms").send().await;
    let workflows_response = gloo_net::http::Request::get("/api/workflows").send().await;

    match (
        node_types_response,
        nodes_response,
        forms_response,
        workflows_response,
    ) {
        (Ok(response), _, _, _) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        (_, Ok(response), _, _) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        (_, _, Ok(response), _) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        (_, _, _, Ok(response)) if response.status() == 401 => Err(WorkflowApiError::Unauthorized),
        (
            Ok(node_types_response),
            Ok(nodes_response),
            Ok(forms_response),
            Ok(workflows_response),
        ) if node_types_response.ok()
            && nodes_response.ok()
            && forms_response.ok()
            && workflows_response.ok() =>
        {
            let loaded_node_types = node_types_response
                .json::<Vec<NodeTypeCatalogEntry>>()
                .await;
            let loaded_nodes = nodes_response.json::<Vec<OrganizationNode>>().await;
            let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;
            let loaded_workflows = workflows_response.json::<Vec<WorkflowSummary>>().await;

            match (
                loaded_node_types,
                loaded_nodes,
                loaded_forms,
                loaded_workflows,
            ) {
                (Ok(node_types), Ok(organization_nodes), Ok(forms), Ok(workflows)) => {
                    Ok(WorkflowEditorOptionResponses {
                        node_types,
                        organization_nodes,
                        forms,
                        workflows,
                    })
                }
                _ => Err(WorkflowApiError::message(
                    "Workflow options could not be read.",
                )),
            }
        }
        (
            Ok(node_types_response),
            Ok(nodes_response),
            Ok(forms_response),
            Ok(workflows_response),
        ) => Err(WorkflowApiError::message(format!(
            "Workflow options failed with status {} / {} / {} / {}.",
            node_types_response.status(),
            nodes_response.status(),
            forms_response.status(),
            workflows_response.status()
        ))),
        _ => Err(WorkflowApiError::message(
            "Could not reach the workflow option APIs.",
        )),
    }
}

#[cfg(feature = "hydrate")]
pub(in crate::features::workflows) async fn update_workflow(
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
pub(in crate::features::workflows) async fn create_workflow(
    payload: CreateWorkflowPayload,
) -> Result<IdResponse, String> {
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
pub(in crate::features::workflows) async fn create_initial_workflow_revision(
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
pub(in crate::features::workflows) async fn update_workflow_revision_steps(
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
pub(in crate::features::workflows) async fn create_workflow_revision(
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
pub(in crate::features::workflows) async fn publish_workflow_revision(
    version_id: &str,
) -> Result<IdResponse, String> {
    let publish_url = format!("/api/workflow-versions/{version_id}/publish");

    send_json_id_request(
        gloo_net::http::Request::post(&publish_url),
        None,
        "Publish workflow revision",
    )
    .await
}
