//! Owns the features::shared::ui module behavior.

use crate::features::organization::OrganizationNode;

/// Handles the node display path behavior.
pub(crate) fn node_display_path(node: &OrganizationNode) -> String {
    node.parent_node_name
        .as_deref()
        .map(|parent| format!("{parent} / {}", node.name))
        .unwrap_or_else(|| node.name.clone())
}

/// Handles the node count label behavior.
pub(crate) fn node_count_label(count: usize) -> String {
    if count == 1 {
        "1 Node".to_string()
    } else {
        format!("{count} Nodes")
    }
}

/// Handles the user count label behavior.
pub(crate) fn user_count_label(count: usize) -> String {
    if count == 1 {
        "1 User".to_string()
    } else {
        format!("{count} Users")
    }
}
