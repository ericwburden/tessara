//! Workflow editor state helpers.

use super::options::workflow_form_version_options;
use crate::features::forms::FormSummary;
use crate::features::organization::NodeTypeCatalogEntry;
use crate::features::workflows::types::WorkflowStepDraft;
use leptos::prelude::*;
use std::collections::HashSet;

/// Removes workflow steps whose selected form version is no longer available.
pub(in crate::features::workflows) fn prune_unavailable_workflow_steps(
    forms: &[FormSummary],
    node_types: &[NodeTypeCatalogEntry],
    steps: RwSignal<Vec<WorkflowStepDraft>>,
) {
    let available_options = workflow_form_version_options(forms, node_types, "");
    steps.update(|steps| {
        steps.retain(|step| {
            step.form_version_id.is_empty()
                || available_options
                    .iter()
                    .any(|(id, _, _)| id == &step.form_version_id)
        });
    });
}

/// Appends a new empty workflow step and advances the next step id.
pub(in crate::features::workflows) fn add_workflow_step(
    next_step_id: RwSignal<usize>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
) {
    let id = next_step_id.get_untracked();
    next_step_id.set(id + 1);
    steps.update(|steps| {
        steps.push(WorkflowStepDraft {
            id,
            title: format!("Step {}", steps.len() + 1),
            form_version_id: String::new(),
        });
    });
}

/// Returns whether the workflow editor has enough valid state to submit.
pub(in crate::features::workflows) fn can_submit_workflow_editor(
    is_saving: RwSignal<bool>,
    name: RwSignal<String>,
    available_node_ids: RwSignal<HashSet<String>>,
    steps: RwSignal<Vec<WorkflowStepDraft>>,
) -> bool {
    if is_saving.get() || name.get().trim().is_empty() {
        return false;
    }
    if available_node_ids.get().is_empty() {
        return false;
    }
    let current_steps = steps.get();
    !current_steps.is_empty()
        && current_steps
            .iter()
            .all(|step| !step.form_version_id.trim().is_empty())
}
