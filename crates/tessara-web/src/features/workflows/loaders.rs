//! Signal-aware loaders for workflow pages.

use crate::features::forms::FormSummary;
use crate::features::organization::{NodeTypeCatalogEntry, OrganizationNode};
#[cfg(feature = "hydrate")]
use crate::features::shared::node_display_path;
use crate::features::workflows::types::{WorkflowDefinition, WorkflowSummary};
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

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

            match gloo_net::http::Request::get("/api/workflows").send().await {
                Ok(response) if response.status() == 401 => {
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowSummary>>().await {
                        Ok(loaded_workflows) => {
                            workflows.set(loaded_workflows);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            workflows.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse workflows: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    workflows.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load workflows. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    workflows.set(Vec::new());
                    load_error.set(Some(format!("Unable to load workflows: {error}")));
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
            match gloo_net::http::Request::get("/api/nodes").send().await {
                Ok(response) if response.status() == 401 => {
                    nodes.set(Vec::new());
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    if let Ok(loaded_nodes) = response.json::<Vec<OrganizationNode>>().await {
                        nodes.set(loaded_nodes);
                    }
                }
                _ => nodes.set(Vec::new()),
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

            match gloo_net::http::Request::get(&format!("/api/workflows/{workflow_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<WorkflowDefinition>().await {
                        Ok(workflow) => {
                            detail.set(Some(workflow));
                            is_loading.set(false);
                        }
                        Err(error) => {
                            detail.set(None);
                            load_error
                                .set(Some(format!("Unable to parse workflow detail: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load workflow detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load workflow detail: {error}")));
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
                            Ok(mut loaded_node_types),
                            Ok(mut loaded_nodes),
                            Ok(mut loaded_forms),
                            Ok(mut loaded_workflows),
                        ) => {
                            loaded_node_types.sort_by(|left, right| {
                                left.singular_label
                                    .cmp(&right.singular_label)
                                    .then(left.name.cmp(&right.name))
                            });
                            loaded_forms.sort_by(|left, right| {
                                left.name.cmp(&right.name).then(left.slug.cmp(&right.slug))
                            });
                            loaded_nodes.sort_by(|left, right| {
                                node_display_path(left)
                                    .cmp(&node_display_path(right))
                                    .then(left.name.cmp(&right.name))
                            });
                            loaded_workflows.sort_by(|left, right| {
                                left.name.cmp(&right.name).then(left.slug.cmp(&right.slug))
                            });

                            node_types.set(loaded_node_types);
                            organization_nodes.set(loaded_nodes);
                            forms.set(loaded_forms);
                            workflows.set(loaded_workflows);
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
