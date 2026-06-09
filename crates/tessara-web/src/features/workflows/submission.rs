use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use crate::api::client::{redirect_to_login, send_json_request};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateFormPayload {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateFormPayload {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) scope_node_type_id: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateWorkflowPayload {
    pub(crate) available_node_ids: Vec<String>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateWorkflowPayload {
    pub(crate) available_node_ids: Vec<String>,
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateWorkflowRevisionPayload {
    pub(crate) steps: Vec<CreateWorkflowStepPayload>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateWorkflowRevisionStepsPayload {
    pub(crate) steps: Vec<CreateWorkflowStepPayload>,
}

#[derive(Clone, Debug, Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateWorkflowStepPayload {
    pub(crate) title: String,
    pub(crate) form_version_id: String,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct WorkflowStepDraft {
    pub(crate) id: usize,
    pub(crate) title: String,
    pub(crate) form_version_id: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) enum WorkflowSaveIntent {
    Draft,
    Publish,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssignmentSummary {
    pub(crate) id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_version_id: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) workflow_step_id: String,
    pub(crate) workflow_step_title: String,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) account_id: String,
    pub(crate) account_display_name: String,
    pub(crate) account_email: String,
    pub(crate) is_active: bool,
    pub(crate) has_draft: bool,
    pub(crate) has_submitted: bool,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct PendingWorkflowWork {
    pub(crate) workflow_assignment_id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_description: String,
    pub(crate) workflow_version_id: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) workflow_step_title: String,
    pub(crate) workflow_step_position: i32,
    pub(crate) workflow_step_count: i64,
    pub(crate) next_workflow_step_title: Option<String>,
    pub(crate) next_workflow_step_form_name: Option<String>,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) account_id: String,
    pub(crate) account_display_name: String,
    pub(crate) assigned_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionSummary {
    pub(crate) id: String,
    pub(crate) form_id: String,
    pub(crate) form_version_id: String,
    pub(crate) form_name: String,
    pub(crate) workflow_name: Option<String>,
    pub(crate) workflow_description: Option<String>,
    pub(crate) workflow_step_position: Option<i32>,
    pub(crate) workflow_step_count: Option<i64>,
    pub(crate) workflow_steps_completed: Option<i64>,
    pub(crate) current_workflow_step_title: Option<String>,
    pub(crate) next_workflow_step_title: Option<String>,
    pub(crate) next_workflow_step_form_name: Option<String>,
    pub(crate) assigned_to_display_name: Option<String>,
    pub(crate) version_label: String,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) status: String,
    pub(crate) value_count: i64,
    pub(crate) created_at: String,
    pub(crate) last_modified_at: String,
    pub(crate) submitted_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionDetail {
    pub(crate) id: String,
    pub(crate) form_id: String,
    pub(crate) form_version_id: String,
    pub(crate) form_name: String,
    pub(crate) version_label: String,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) status: String,
    pub(crate) created_at: String,
    pub(crate) submitted_at: Option<String>,
    #[serde(default)]
    pub(crate) values: Vec<SubmissionValueDetail>,
    #[serde(default)]
    pub(crate) audit_events: Vec<SubmissionAuditEventSummary>,
    pub(crate) runtime: Option<SubmissionRuntimeDetail>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionRuntimeDetail {
    pub(crate) workflow_name: String,
    pub(crate) current_step_title: String,
    pub(crate) current_step_position: i32,
    pub(crate) step_count: i64,
    pub(crate) next_step_title: Option<String>,
    #[serde(default)]
    pub(crate) history: Vec<SubmissionRuntimeStepHistory>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionRuntimeStepHistory {
    pub(crate) title: String,
    pub(crate) form_name: String,
    pub(crate) status: String,
    pub(crate) position: i32,
    pub(crate) completed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionValueDetail {
    pub(crate) field_id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) value: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct SubmissionAuditEventSummary {
    pub(crate) event_type: String,
    pub(crate) account_email: Option<String>,
    pub(crate) created_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AssignmentResponseStartOption {
    pub(crate) workflow_assignment_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) workflow_step_title: String,
    pub(crate) workflow_step_position: i32,
    pub(crate) workflow_step_count: i64,
    pub(crate) form_id: String,
    pub(crate) form_name: String,
    pub(crate) form_version_id: String,
    pub(crate) form_version_label: Option<String>,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) account_id: String,
    pub(crate) account_display_name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct AssignmentResponseStartOptions {
    #[serde(default)]
    pub(crate) assignments: Vec<AssignmentResponseStartOption>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SaveSubmissionValuesPayload {
    pub(crate) values: HashMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssignmentCandidate {
    pub(crate) workflow_version_id: String,
    pub(crate) workflow_id: String,
    pub(crate) workflow_name: String,
    pub(crate) workflow_version_label: Option<String>,
    pub(crate) node_id: String,
    pub(crate) node_name: String,
    pub(crate) node_path: String,
    pub(crate) label: String,
    pub(crate) step_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub(crate) struct WorkflowAssigneeOption {
    pub(crate) account_id: String,
    pub(crate) email: String,
    pub(crate) display_name: String,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct BulkWorkflowAssignmentPayload {
    pub(crate) workflow_version_id: String,
    pub(crate) node_id: String,
    pub(crate) account_ids: Vec<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct UpdateWorkflowAssignmentPayload {
    pub(crate) node_id: String,
    pub(crate) account_id: String,
    pub(crate) is_active: bool,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateFormSectionPayload {
    pub(crate) title: String,
    pub(crate) position: i32,
    pub(crate) description: String,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct CreateFormFieldPayload {
    pub(crate) section_id: String,
    pub(crate) key: String,
    pub(crate) label: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) position: i32,
    pub(crate) grid_row: i32,
    pub(crate) grid_column: i32,
    pub(crate) grid_width: i32,
    pub(crate) grid_height: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderSectionDraft {
    pub(crate) id: usize,
    pub(crate) remote_id: Option<String>,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) default_column_width: i32,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderFieldDraft {
    pub(crate) id: usize,
    pub(crate) remote_id: Option<String>,
    pub(crate) section_id: usize,
    pub(crate) label: String,
    pub(crate) key: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) grid_row: i32,
    pub(crate) grid_column: i32,
    pub(crate) grid_width: i32,
    pub(crate) grid_height: i32,
    pub(crate) key_was_edited: bool,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn load_pending_work(
    pending_work: RwSignal<Vec<PendingWorkflowWork>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/workflow-assignments/pending")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    pending_work.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<PendingWorkflowWork>>().await {
                        Ok(loaded_work) => {
                            pending_work.set(loaded_work);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            pending_work.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse assigned work: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load assigned work. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(format!("Unable to load assigned work: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (pending_work, is_loading, load_error);
    }
}

pub(crate) fn start_workflow_assignment_response(
    workflow_assignment_id: String,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(Some("Starting assigned response...".into()));

            match send_json_request::<serde_json::Value>(
                gloo_net::http::Request::post(&format!(
                    "/api/workflow-assignments/{workflow_assignment_id}/start"
                )),
                Some("{}".into()),
                "Start assigned response",
            )
            .await
            {
                Ok(response) => {
                    let id = response
                        .get("id")
                        .and_then(|value| value.as_str().map(str::to_owned))
                        .or_else(|| {
                            response
                                .get("id")
                                .and_then(|value| value.as_i64().map(|value| value.to_string()))
                        });
                    if let Some(id) = id {
                        if let Some(window) = web_sys::window() {
                            let _ = window.location().set_href(&format!("/responses/{id}/edit"));
                        }
                    }
                }
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflow_assignment_id, is_saving, message);
    }
}

pub(crate) fn workflow_revision_label_from_raw(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }

    if let Ok(revision) = trimmed.parse::<u64>() {
        return revision.to_string();
    }

    trimmed
        .split('.')
        .next()
        .and_then(|part| part.trim().parse::<u64>().ok())
        .map(|revision| revision.to_string())
        .unwrap_or_else(|| trimmed.to_string())
}
