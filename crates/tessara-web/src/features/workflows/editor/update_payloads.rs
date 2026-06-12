//! Payload preparation for workflow update actions.

use super::{workflow_step_payloads_from_drafts, workflow_step_signature};
use crate::features::workflows::{
    CreateWorkflowStepPayload, UpdateWorkflowPayload, WorkflowSaveIntent, WorkflowStepDraft,
};
use crate::utils::text::IntoNonemptyString;
use std::collections::HashSet;

pub(super) struct PreparedWorkflowUpdate {
    pub(super) payload: UpdateWorkflowPayload,
    pub(super) step_payload: Option<Vec<CreateWorkflowStepPayload>>,
}

pub(super) fn prepare_workflow_update(
    name: String,
    slug: String,
    available_node_ids: HashSet<String>,
    current_steps: Vec<WorkflowStepDraft>,
    original_steps: &[WorkflowStepDraft],
    description: String,
    version_is_draft: bool,
    intent: WorkflowSaveIntent,
) -> Result<PreparedWorkflowUpdate, String> {
    let workflow_name = name.trim().to_string();
    if workflow_name.is_empty() {
        return Err("Workflow name is required.".into());
    }

    let workflow_slug = slug.trim().to_string();
    if workflow_slug.is_empty() {
        return Err("Workflow slug is missing. Reload the workflow and try again.".into());
    }

    let mut selected_available_node_ids = available_node_ids.into_iter().collect::<Vec<_>>();
    selected_available_node_ids.sort();
    if selected_available_node_ids.is_empty() {
        return Err("Select at least one available node.".into());
    }

    let steps_changed =
        workflow_step_signature(&current_steps) != workflow_step_signature(original_steps);
    if intent == WorkflowSaveIntent::Publish && !version_is_draft && !steps_changed {
        return Err("Make a workflow step change before publishing a new revision.".into());
    }

    let step_payload = if steps_changed {
        if current_steps.is_empty() {
            return Err("Add at least one workflow step.".into());
        }
        if current_steps
            .iter()
            .any(|step| step.form_version_id.trim().is_empty())
        {
            return Err("Select a form version for each workflow step.".into());
        }

        Some(workflow_step_payloads_from_drafts(current_steps))
    } else {
        None
    };

    Ok(PreparedWorkflowUpdate {
        payload: UpdateWorkflowPayload {
            available_node_ids: selected_available_node_ids,
            name: workflow_name,
            slug: workflow_slug,
            description: description.trim().to_string().into_nonempty(),
        },
        step_payload,
    })
}
