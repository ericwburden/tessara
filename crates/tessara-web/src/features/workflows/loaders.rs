//! Signal-aware loaders for workflow pages.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
use crate::features::workflows::types::{WorkflowDefinition, WorkflowSummary};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::api::{
    WorkflowApiError, fetch_workflow_assignment_nodes, fetch_workflow_detail, fetch_workflows,
};
#[cfg(feature = "hydrate")]
use super::options::ordered_workflow_editor_options;

/// Loads workflow summaries.
pub(crate) fn load_workflows(
    workflows: RwSignal<Vec<WorkflowSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_workflows().await {
                Ok(loaded_workflows) => {
                    workflows.set(loaded_workflows);
                    is_loading.set(false);
                }
                Err(WorkflowApiError::Unauthorized) => {
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(WorkflowApiError::Message(error)) => {
                    workflows.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflows, is_loading, load_error);
    }
}

/// Loads organization nodes used by workflow assignment panels.
pub(crate) fn load_workflow_assignment_nodes(nodes: RwSignal<Vec<OrganizationNode>>) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match fetch_workflow_assignment_nodes().await {
                Ok(loaded_nodes) => nodes.set(loaded_nodes),
                Err(WorkflowApiError::Unauthorized) => {
                    nodes.set(Vec::new());
                    redirect_to_login();
                }
                Err(WorkflowApiError::Message(_)) => nodes.set(Vec::new()),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = nodes;
    }
}

/// Loads workflow detail for detail and editor pages.
pub(crate) fn load_workflow_detail(
    workflow_id: String,
    detail: RwSignal<Option<WorkflowDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_workflow_detail(&workflow_id).await {
                Ok(workflow) => {
                    detail.set(Some(workflow));
                    is_loading.set(false);
                }
                Err(WorkflowApiError::Unauthorized) => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(WorkflowApiError::Message(error)) => {
                    detail.set(None);
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflow_id, detail, is_loading, load_error);
    }
}

/// Loads selectable node types, nodes, forms, and workflows for workflow editors.
pub(crate) fn load_workflow_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    organization_nodes: RwSignal<Vec<OrganizationNode>>,
    forms: RwSignal<Vec<FormSummary>>,
    workflows: RwSignal<Vec<WorkflowSummary>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

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
                (Ok(response), _, _, _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _, _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response), _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, _, Ok(response)) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
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
                        (
                            Ok(loaded_node_types),
                            Ok(loaded_nodes),
                            Ok(loaded_forms),
                            Ok(loaded_workflows),
                        ) => {
                            let options = ordered_workflow_editor_options(
                                loaded_node_types,
                                loaded_nodes,
                                loaded_forms,
                                loaded_workflows,
                            );

                            node_types.set(options.node_types);
                            organization_nodes.set(options.organization_nodes);
                            forms.set(options.forms);
                            workflows.set(options.workflows);
                            is_loading.set(false);
                        }
                        _ => {
                            node_types.set(Vec::new());
                            organization_nodes.set(Vec::new());
                            forms.set(Vec::new());
                            workflows.set(Vec::new());
                            message.set(Some("Workflow options could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (
                    Ok(node_types_response),
                    Ok(nodes_response),
                    Ok(forms_response),
                    Ok(workflows_response),
                ) => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    message.set(Some(format!(
                        "Workflow options failed with status {} / {} / {} / {}.",
                        node_types_response.status(),
                        nodes_response.status(),
                        forms_response.status(),
                        workflows_response.status()
                    )));
                    is_loading.set(false);
                }
                _ => {
                    node_types.set(Vec::new());
                    organization_nodes.set(Vec::new());
                    forms.set(Vec::new());
                    workflows.set(Vec::new());
                    message.set(Some("Could not reach the workflow option APIs.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_types,
            organization_nodes,
            forms,
            workflows,
            is_loading,
            message,
        );
    }
}
