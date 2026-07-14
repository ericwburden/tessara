//! Transport calls for workflow pages.
//!
//! Keep endpoint requests and response parsing here; signal orchestration belongs in loaders and actions.

#[cfg(feature = "hydrate")]
use crate::types::NodeTypeCatalogEntry;
#[cfg(feature = "hydrate")]
use crate::types::OrganizationNode;
#[cfg(feature = "hydrate")]
use crate::types::{WorkflowDefinition, WorkflowFormSummary, WorkflowSummary};
#[cfg(feature = "hydrate")]
pub(super) enum WorkflowApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
pub(super) struct WorkflowEditorOptionResponses {
    pub(super) node_types: Vec<NodeTypeCatalogEntry>,
    pub(super) organization_nodes: Vec<OrganizationNode>,
    pub(super) forms: Vec<WorkflowFormSummary>,
    pub(super) workflows: Vec<WorkflowSummary>,
}

#[cfg(feature = "hydrate")]
impl WorkflowApiError {
    fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflows() -> Result<Vec<WorkflowSummary>, WorkflowApiError> {
    tessara_web_http::fetch_json("/api/workflows", "Workflows")
        .await
        .map_err(WorkflowApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_assignment_nodes()
-> Result<Vec<OrganizationNode>, WorkflowApiError> {
    tessara_web_http::fetch_json("/api/nodes", "Workflow assignment nodes")
        .await
        .map_err(WorkflowApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_detail(
    workflow_id: &str,
) -> Result<WorkflowDefinition, WorkflowApiError> {
    tessara_web_http::fetch_json(&format!("/api/workflows/{workflow_id}"), "Workflow detail")
        .await
        .map_err(WorkflowApiError::from_request)
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_workflow_editor_options()
-> Result<WorkflowEditorOptionResponses, WorkflowApiError> {
    let node_types = tessara_web_http::fetch_json("/api/node-types", "Workflow node types")
        .await
        .map_err(WorkflowApiError::from_request)?;
    let organization_nodes = tessara_web_http::fetch_json("/api/nodes", "Workflow nodes")
        .await
        .map_err(WorkflowApiError::from_request)?;
    let forms = tessara_web_http::fetch_json("/api/forms", "Workflow form options")
        .await
        .map_err(WorkflowApiError::from_request)?;
    let workflows = tessara_web_http::fetch_json("/api/workflows", "Workflow options")
        .await
        .map_err(WorkflowApiError::from_request)?;

    Ok(WorkflowEditorOptionResponses {
        node_types,
        organization_nodes,
        forms,
        workflows,
    })
}
