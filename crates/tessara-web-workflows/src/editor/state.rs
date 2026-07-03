//! Workflow editor state helpers.

use super::options::workflow_form_version_options;
use crate::display::{active_workflow_definition_version, workflow_revision_label_from_raw};
use crate::text::sentence_label;
use crate::types::NodeTypeCatalogEntry;
use crate::types::{WorkflowDefinition, WorkflowFormSummary, WorkflowStepDraft};
use leptos::prelude::*;
use std::collections::HashSet;

/// Initial editor values derived from a loaded workflow definition.
pub(crate) struct WorkflowEditInitialState {
    pub(crate) name: String,
    pub(crate) slug: String,
    pub(crate) description: String,
    pub(crate) available_node_ids: HashSet<String>,
    pub(crate) edit_version_id: Option<String>,
    pub(crate) edit_version_label: String,
    pub(crate) edit_version_status: String,
    pub(crate) version_is_draft: bool,
    pub(crate) steps: Vec<WorkflowStepDraft>,
    pub(crate) next_step_id: usize,
}

/// Builds the edit page's initial signal values from workflow detail and an optional requested revision.
pub(crate) fn workflow_edit_initial_state(
    workflow: &WorkflowDefinition,
    requested_version_id: Option<String>,
) -> WorkflowEditInitialState {
    let edit_version = requested_version_id
        .as_ref()
        .and_then(|version_id| {
            workflow
                .versions
                .iter()
                .find(|version| version.id == *version_id)
                .cloned()
        })
        .or_else(|| active_workflow_definition_version(workflow).cloned());

    let mut step_summaries = edit_version
        .as_ref()
        .map(|version| version.steps.clone())
        .unwrap_or_default();
    step_summaries.sort_by_key(|step| step.position);
    let steps = step_summaries
        .into_iter()
        .enumerate()
        .map(|(index, step)| WorkflowStepDraft {
            id: index + 1,
            title: step.title,
            form_version_id: step.form_version_id,
        })
        .collect::<Vec<_>>();

    WorkflowEditInitialState {
        name: workflow.name.clone(),
        slug: workflow.slug.clone(),
        description: workflow.description.clone(),
        available_node_ids: workflow
            .available_nodes
            .iter()
            .map(|node| node.id.clone())
            .collect(),
        edit_version_id: edit_version.as_ref().map(|version| version.id.clone()),
        edit_version_label: edit_version
            .as_ref()
            .and_then(|version| version.workflow_revision_label.clone())
            .as_deref()
            .map(workflow_revision_label_from_raw)
            .unwrap_or_else(|| "-".to_string()),
        edit_version_status: edit_version
            .as_ref()
            .map(|version| sentence_label(&version.status))
            .unwrap_or_else(|| "No revisions".to_string()),
        version_is_draft: edit_version
            .as_ref()
            .map(|version| version.status.eq_ignore_ascii_case("draft"))
            .unwrap_or(false),
        next_step_id: steps.len() + 1,
        steps,
    }
}

/// Removes workflow steps whose selected form version is no longer available.
pub(crate) fn prune_unavailable_workflow_steps(
    forms: &[WorkflowFormSummary],
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
pub(crate) fn add_workflow_step(
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
pub(crate) fn can_submit_workflow_editor(
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
