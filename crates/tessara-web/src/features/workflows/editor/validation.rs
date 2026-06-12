//! Pure validation helpers for workflow editor actions.

use crate::features::workflows::WorkflowStepDraft;
use std::collections::HashSet;

pub(super) fn validated_workflow_name(name: String) -> Result<String, String> {
    let workflow_name = name.trim().to_string();
    if workflow_name.is_empty() {
        return Err("Workflow name is required.".into());
    }

    Ok(workflow_name)
}

pub(super) fn validated_workflow_slug(slug: String) -> Result<String, String> {
    let workflow_slug = slug.trim().to_string();
    if workflow_slug.is_empty() {
        return Err("Workflow slug is missing. Reload the workflow and try again.".into());
    }

    Ok(workflow_slug)
}

pub(super) fn validated_available_node_ids(
    available_node_ids: HashSet<String>,
) -> Result<Vec<String>, String> {
    let mut selected_available_node_ids = available_node_ids.into_iter().collect::<Vec<_>>();
    selected_available_node_ids.sort();
    if selected_available_node_ids.is_empty() {
        return Err("Select at least one available node.".into());
    }

    Ok(selected_available_node_ids)
}

pub(super) fn validate_workflow_steps(steps: &[WorkflowStepDraft]) -> Result<(), String> {
    if steps.is_empty() {
        return Err("Add at least one workflow step.".into());
    }
    if steps
        .iter()
        .any(|step| step.form_version_id.trim().is_empty())
    {
        return Err("Select a form version for each workflow step.".into());
    }

    Ok(())
}
