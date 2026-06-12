//! Filtering helpers for workflow assignment pickers and tables.

use super::display::{
    workflow_assignment_assignee_label, workflow_assignment_revision_label,
    workflow_assignment_state, workflow_assignment_status_key,
};
use super::types::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use crate::features::shared::unique_filter_options;
use crate::utils::text::text_matches;
use std::collections::HashSet;

/// Filters candidate workflow versions for the assignment picker.
pub(in crate::features::workflows) fn filtered_workflow_candidates(
    candidates: Vec<WorkflowAssignmentCandidate>,
    query: &str,
    selected_node_id: &str,
) -> Vec<WorkflowAssignmentCandidate> {
    let mut seen = HashSet::new();
    let mut workflows = candidates
        .into_iter()
        .filter(|candidate| {
            (selected_node_id.is_empty() || candidate.node_id == selected_node_id)
                && seen.insert(candidate.workflow_version_id.clone())
                && text_matches(
                    query,
                    &[
                        candidate.workflow_name.as_str(),
                        candidate
                            .workflow_version_label
                            .as_deref()
                            .unwrap_or_default(),
                    ],
                )
        })
        .collect::<Vec<_>>();
    workflows.sort_by(|left, right| {
        left.workflow_name
            .cmp(&right.workflow_name)
            .then(left.workflow_version_id.cmp(&right.workflow_version_id))
    });
    workflows
}

/// Filters candidate nodes for the assignment picker.
pub(in crate::features::workflows) fn filtered_node_candidates(
    candidates: Vec<WorkflowAssignmentCandidate>,
    query: &str,
    selected_workflow_version_id: &str,
) -> Vec<WorkflowAssignmentCandidate> {
    let mut seen = HashSet::new();
    let mut nodes = candidates
        .into_iter()
        .filter(|candidate| {
            (selected_workflow_version_id.is_empty()
                || candidate.workflow_version_id == selected_workflow_version_id)
                && seen.insert(candidate.node_id.clone())
                && text_matches(
                    query,
                    &[candidate.node_name.as_str(), candidate.node_path.as_str()],
                )
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|left, right| left.node_path.cmp(&right.node_path));
    nodes
}

/// Returns whether a workflow-version and node selection is assignable.
pub(in crate::features::workflows) fn workflow_assignment_pair_is_valid(
    candidates: &[WorkflowAssignmentCandidate],
    workflow_version_id: &str,
    node_id: &str,
) -> bool {
    !workflow_version_id.is_empty()
        && !node_id.is_empty()
        && candidates.iter().any(|candidate| {
            candidate.workflow_version_id == workflow_version_id && candidate.node_id == node_id
        })
}

/// Builds the selected workflow summary for the assignment picker.
pub(in crate::features::workflows) fn selected_workflow_summary(
    candidates: Vec<WorkflowAssignmentCandidate>,
    selected_id: &str,
) -> Option<(String, String)> {
    candidates
        .into_iter()
        .find(|candidate| candidate.workflow_version_id == selected_id)
        .map(|candidate| {
            let revision =
                workflow_assignment_revision_label(candidate.workflow_version_label.as_deref());
            (candidate.workflow_name, format!("Revision {revision}"))
        })
}

/// Builds the selected node summary for the assignment picker.
pub(in crate::features::workflows) fn selected_node_summary(
    candidates: Vec<WorkflowAssignmentCandidate>,
    selected_id: &str,
) -> Option<(String, String)> {
    candidates
        .into_iter()
        .find(|candidate| candidate.node_id == selected_id)
        .map(|candidate| {
            let node_path = if candidate.node_path.trim().is_empty() {
                candidate.node_name.clone()
            } else {
                candidate.node_path.clone()
            };
            (candidate.node_name, node_path)
        })
}

/// Filters available assignees by search text.
pub(in crate::features::workflows) fn filtered_assignees(
    assignees: Vec<WorkflowAssigneeOption>,
    query: &str,
) -> Vec<WorkflowAssigneeOption> {
    assignees
        .into_iter()
        .filter(|assignee| {
            text_matches(
                query,
                &[assignee.display_name.as_str(), assignee.email.as_str()],
            )
        })
        .collect::<Vec<_>>()
}

/// Filters workflow assignment rows by search and active filters.
pub(in crate::features::workflows) fn filtered_assignments(
    assignments: Vec<WorkflowAssignmentSummary>,
    query: &str,
    status: &str,
    state: &str,
    assignee: &str,
) -> Vec<WorkflowAssignmentSummary> {
    assignments
        .into_iter()
        .filter(|assignment| {
            let matches_status =
                status == "all" || workflow_assignment_status_key(assignment) == status;
            let matches_state = state == "all" || workflow_assignment_state(assignment) == state;
            let matches_assignee =
                assignee == "all" || workflow_assignment_assignee_label(assignment) == assignee;
            matches_status
                && matches_state
                && matches_assignee
                && text_matches(
                    query,
                    &[
                        assignment.workflow_name.as_str(),
                        assignment.workflow_step_title.as_str(),
                        assignment.form_name.as_str(),
                        assignment.node_name.as_str(),
                        assignment.account_display_name.as_str(),
                        assignment.account_email.as_str(),
                        assignment.id.as_str(),
                    ],
                )
        })
        .collect::<Vec<_>>()
}

/// Builds assignment assignee filter options.
pub(in crate::features::workflows) fn assignee_filter_options(
    assignments: &[WorkflowAssignmentSummary],
) -> Vec<String> {
    unique_filter_options(
        assignments
            .iter()
            .map(workflow_assignment_assignee_label)
            .collect::<Vec<_>>(),
    )
}
