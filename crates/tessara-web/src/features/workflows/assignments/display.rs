//! Display formatting helpers for the Workflows feature.
//!
//! Keep label, class, and summary formatting here when it depends on Workflows domain values but not on route state.

use crate::features::shared::FormAttachmentLink;
use crate::features::workflows::assignments::types::{
    WorkflowAssigneeOption, WorkflowAssignmentCandidate, WorkflowAssignmentSummary,
};
use crate::features::workflows::types::{WorkflowAvailableNodeSummary, WorkflowSummary};
use crate::features::workflows::workflow_revision_label_from_raw;

/// Handles the assignment count label behavior.
fn assignment_count_label(count: usize) -> String {
    if count == 1 {
        "1 Assignment".to_string()
    } else {
        format!("{count} Assignments")
    }
}

/// Handles the workflow assigned user links behavior.
pub(crate) fn workflow_assigned_user_links(workflow: &WorkflowSummary) -> Vec<FormAttachmentLink> {
    workflow
        .assigned_users
        .iter()
        .map(|user| FormAttachmentLink {
            href: format!("/administration/users/{}", user.id),
            label: user.display_name.clone(),
            title: format!(
                "{} - {}",
                user.email,
                assignment_count_label(user.assignment_count.max(0) as usize)
            ),
        })
        .collect()
}

/// Handles the workflow available node links behavior.
pub(crate) fn workflow_available_node_links(
    nodes: &[WorkflowAvailableNodeSummary],
) -> Vec<FormAttachmentLink> {
    nodes
        .iter()
        .map(|node| FormAttachmentLink {
            href: format!("/organization/{}", node.id),
            label: node.name.clone(),
            title: format!("{} - {}", node.node_type_name, node.path),
        })
        .collect()
}

/// Handles the workflow assignment state behavior.
pub(crate) fn workflow_assignment_state(assignment: &WorkflowAssignmentSummary) -> &'static str {
    if assignment.has_submitted {
        "submitted"
    } else if assignment.has_draft {
        "draft"
    } else {
        "pending"
    }
}

/// Handles the workflow assignment state label behavior.
pub(crate) fn workflow_assignment_state_label(
    assignment: &WorkflowAssignmentSummary,
) -> &'static str {
    match workflow_assignment_state(assignment) {
        "submitted" => "Submitted",
        "draft" => "Draft Exists",
        _ => "Pending",
    }
}

/// Handles the workflow assignment status key behavior.
pub(crate) fn workflow_assignment_status_key(
    assignment: &WorkflowAssignmentSummary,
) -> &'static str {
    if assignment.is_active {
        "active"
    } else {
        "inactive"
    }
}

/// Handles the workflow assignment status label behavior.
pub(crate) fn workflow_assignment_status_label(
    assignment: &WorkflowAssignmentSummary,
) -> &'static str {
    if assignment.is_active {
        "Active"
    } else {
        "Inactive"
    }
}

/// Handles the workflow assignment revision label behavior.
pub(crate) fn workflow_assignment_revision_label(label: Option<&str>) -> String {
    label
        .map(workflow_revision_label_from_raw)
        .unwrap_or_else(|| "-".to_string())
}

/// Handles the workflow assignment candidate key behavior.
pub(crate) fn workflow_assignment_candidate_key(candidate: &WorkflowAssignmentCandidate) -> String {
    format!("{}|{}", candidate.workflow_version_id, candidate.node_id)
}

/// Handles the workflow assignee label behavior.
pub(crate) fn workflow_assignee_label(assignee: &WorkflowAssigneeOption) -> String {
    if assignee.display_name.trim().is_empty() {
        assignee.email.clone()
    } else {
        format!("{} ({})", assignee.display_name, assignee.email)
    }
}

/// Handles the workflow assignment assignee label behavior.
pub(crate) fn workflow_assignment_assignee_label(assignment: &WorkflowAssignmentSummary) -> String {
    if assignment.account_display_name.trim().is_empty() {
        assignment.account_email.clone()
    } else {
        format!(
            "{} ({})",
            assignment.account_display_name, assignment.account_email
        )
    }
}
