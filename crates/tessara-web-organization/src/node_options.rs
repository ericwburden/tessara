//! Parent node and node-type option helpers for Organization node editing.

use super::tree::build_organization_tree;
use super::types::{
    NodeTypeCatalogEntry, OrganizationNode, OrganizationTreeNode, ParentNodeOption,
};
use std::collections::HashSet;

/// Builds parent node options from the organization tree.
pub(crate) fn parent_node_options(nodes: &[OrganizationNode]) -> Vec<ParentNodeOption> {
    let branches = build_organization_tree(nodes.to_vec());
    let mut options = Vec::new();
    append_parent_node_options(&branches, 0, &mut options);
    options
}

pub(crate) fn parent_node_options_for_edit(
    nodes: &[OrganizationNode],
    node_types: &[NodeTypeCatalogEntry],
    edited_node_id: &str,
    edited_node_type_id: &str,
) -> Vec<ParentNodeOption> {
    let excluded_ids = descendant_node_ids(nodes, edited_node_id);
    parent_node_options(nodes)
        .into_iter()
        .filter(|option| !excluded_ids.contains(&option.id))
        .filter(|option| {
            nodes
                .iter()
                .find(|node| node.id == option.id)
                .and_then(|node| {
                    node_types
                        .iter()
                        .find(|node_type| node_type.id == node.node_type_id)
                })
                .map(|node_type| {
                    node_type
                        .child_relationships
                        .iter()
                        .any(|relationship| relationship.node_type_id == edited_node_type_id)
                })
                .unwrap_or(false)
        })
        .collect()
}

pub(crate) fn descendant_node_ids(nodes: &[OrganizationNode], root_id: &str) -> HashSet<String> {
    let mut descendants = HashSet::from([root_id.to_string()]);
    let mut changed = true;

    while changed {
        changed = false;
        for node in nodes {
            if descendants.contains(&node.id) {
                continue;
            }

            if node
                .parent_node_id
                .as_ref()
                .map(|parent_id| descendants.contains(parent_id))
                .unwrap_or(false)
            {
                descendants.insert(node.id.clone());
                changed = true;
            }
        }
    }

    descendants
}

pub(crate) fn append_parent_node_options(
    branches: &[OrganizationTreeNode],
    depth: usize,
    options: &mut Vec<ParentNodeOption>,
) {
    for branch in branches {
        let prefix = if depth == 0 {
            String::new()
        } else {
            format!("{} ", "--".repeat(depth))
        };

        options.push(ParentNodeOption {
            id: branch.node.id.clone(),
            label: format!(
                "{}{} ({})",
                prefix, branch.node.name, branch.node.node_type_singular_label
            ),
        });
        append_parent_node_options(&branch.children, depth + 1, options);
    }
}

pub(crate) fn available_node_types_for_parent(
    parent_node_id: &str,
    node_types: &[NodeTypeCatalogEntry],
    nodes: &[OrganizationNode],
) -> Vec<NodeTypeCatalogEntry> {
    if parent_node_id.is_empty() {
        return node_types
            .iter()
            .filter(|node_type| node_type.is_root_type)
            .cloned()
            .collect();
    }

    let Some(parent_node) = nodes.iter().find(|node| node.id == parent_node_id) else {
        return Vec::new();
    };
    let Some(parent_type) = node_types
        .iter()
        .find(|node_type| node_type.id == parent_node.node_type_id)
    else {
        return Vec::new();
    };

    parent_type
        .child_relationships
        .iter()
        .filter_map(|relationship| {
            node_types
                .iter()
                .find(|node_type| node_type.id == relationship.node_type_id)
                .cloned()
        })
        .collect()
}
