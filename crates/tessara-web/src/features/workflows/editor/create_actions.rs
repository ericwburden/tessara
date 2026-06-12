//! Workflow creation save orchestration.

#[cfg(feature = "hydrate")]
use super::existing_workflow_slugs;
#[cfg(feature = "hydrate")]
use crate::features::shared::unique_slug_from_label;
#[cfg(feature = "hydrate")]
use crate::features::workflows::CreateWorkflowStepPayload;
use crate::features::workflows::WorkflowStepDraft;
use crate::features::workflows::types::WorkflowSummary;
#[cfg(feature = "hydrate")]
use crate::features::workflows::{CreateWorkflowPayload, CreateWorkflowRevisionPayload};
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
#[cfg(feature = "hydrate")]
use crate::utils::text::IntoNonemptyString;
use leptos::prelude::*;
use std::collections::HashSet;

pub(crate) fn submit_create_workflow(
    name: RwSignal<String>,
    available_node_ids: RwSignal<HashSet<String>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
    description: RwSignal<String>,
    existing_workflows: RwSignal<Vec<WorkflowSummary>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let workflow_name = name.get().trim().to_string();
        if workflow_name.is_empty() {
            message.set(Some("Workflow name is required.".into()));
            return;
        }
        let mut selected_available_node_ids =
            available_node_ids.get().into_iter().collect::<Vec<_>>();
        selected_available_node_ids.sort();
        if selected_available_node_ids.is_empty() {
            message.set(Some("Select at least one available node.".into()));
            return;
        }

        let current_steps = steps.get();
        if current_steps.is_empty() {
            message.set(Some("Add at least one workflow step.".into()));
            return;
        }
        if current_steps
            .iter()
            .any(|step| step.form_version_id.trim().is_empty())
        {
            message.set(Some("Select a form version for each workflow step.".into()));
            return;
        }

        let workflow_steps = current_steps
            .into_iter()
            .enumerate()
            .map(|(index, step)| CreateWorkflowStepPayload {
                title: step
                    .title
                    .trim()
                    .to_string()
                    .into_nonempty()
                    .unwrap_or_else(|| format!("Step {}", index + 1)),
                form_version_id: step.form_version_id,
            })
            .collect::<Vec<_>>();

        let workflow_slug = unique_slug_from_label(
            &workflow_name,
            &existing_workflow_slugs(existing_workflows.get_untracked().as_slice()),
        );
        if workflow_slug.is_empty() {
            message.set(Some(
                "Workflow name must contain letters or numbers.".into(),
            ));
            return;
        }

        let payload = CreateWorkflowPayload {
            available_node_ids: selected_available_node_ids,
            name: workflow_name,
            slug: workflow_slug,
            description: description.get().trim().to_string().into_nonempty(),
        };
        let version_payload = CreateWorkflowRevisionPayload {
            steps: workflow_steps,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let workflow_body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Create request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };
            let version_body = match serde_json::to_string(&version_payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Workflow step request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_id_request(
                gloo_net::http::Request::post("/api/workflows"),
                Some(workflow_body),
                "Create workflow",
            )
            .await
            {
                Ok(created) => {
                    let version_url = format!("/api/workflows/{}/versions", created.id);
                    match send_json_id_request(
                        gloo_net::http::Request::post(&version_url),
                        Some(version_body),
                        "Create workflow steps",
                    )
                    .await
                    {
                        Ok(_) => {
                            if let Some(window) = web_sys::window() {
                                let _ = window
                                    .location()
                                    .set_href(&format!("/workflows/{}", created.id));
                            }
                        }
                        Err(error) => {
                            if error != "Authentication is required." {
                                message.set(Some(error));
                            }
                            is_saving.set(false);
                        }
                    }
                }
                Err(error) => {
                    if error != "Authentication is required." {
                        message.set(Some(error));
                    }
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            name,
            available_node_ids,
            steps,
            description,
            existing_workflows,
            is_saving,
            message,
        );
    }
}
