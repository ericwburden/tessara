//! Signal-aware workflow loaders used through the Organization API boundary.

use crate::features::organization::types::OrganizationNode;
use crate::features::workflows::types::WorkflowSummary;
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

/// Loads the load workflow assignment nodes data.
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
