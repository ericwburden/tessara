//! Workflows-local display helpers and small view DTOs.

use crate::types::OrganizationNode;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormAttachmentLink {
    pub(crate) href: String,
    pub(crate) label: String,
    pub(crate) title: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowAssignedUsersSheetData {
    pub(crate) workflow_name: String,
    pub(crate) workflow_href: String,
    pub(crate) users: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct WorkflowAvailableNodesSheetData {
    pub(crate) workflow_name: String,
    pub(crate) workflow_href: String,
    pub(crate) nodes: Vec<FormAttachmentLink>,
}

pub(crate) fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" | "submitted" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}

pub(crate) fn node_display_path(node: &OrganizationNode) -> String {
    node.parent_node_name
        .as_deref()
        .map(|parent| format!("{parent} / {}", node.name))
        .unwrap_or_else(|| node.name.clone())
}

pub(crate) fn node_count_label(count: usize) -> String {
    if count == 1 {
        "1 Node".to_string()
    } else {
        format!("{count} Nodes")
    }
}

pub(crate) fn user_count_label(count: usize) -> String {
    if count == 1 {
        "1 User".to_string()
    } else {
        format!("{count} Users")
    }
}
