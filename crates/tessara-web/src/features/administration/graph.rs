//! Graph traversal helpers for Administration node type relationships.

use crate::features::organization::NodeTypeCatalogEntry;
use std::collections::HashSet;

/// Returns all ancestor node type IDs for a node type.
pub(crate) fn node_type_ancestor_ids(
    node_type_id: &str,
    node_types: &[NodeTypeCatalogEntry],
) -> HashSet<String> {
    let mut ancestors = HashSet::new();
    let mut stack = node_types
        .iter()
        .find(|node_type| node_type.id == node_type_id)
        .map(|node_type| {
            node_type
                .parent_relationships
                .iter()
                .map(|peer| peer.node_type_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    while let Some(candidate_id) = stack.pop() {
        if ancestors.insert(candidate_id.clone())
            && let Some(candidate) = node_types
                .iter()
                .find(|node_type| node_type.id == candidate_id)
        {
            stack.extend(
                candidate
                    .parent_relationships
                    .iter()
                    .map(|peer| peer.node_type_id.clone()),
            );
        }
    }

    ancestors
}

/// Returns all descendant node type IDs for a node type.
pub(crate) fn node_type_descendant_ids(
    node_type_id: &str,
    node_types: &[NodeTypeCatalogEntry],
) -> HashSet<String> {
    let mut descendants = HashSet::new();
    let mut stack = node_types
        .iter()
        .find(|node_type| node_type.id == node_type_id)
        .map(|node_type| {
            node_type
                .child_relationships
                .iter()
                .map(|peer| peer.node_type_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    while let Some(candidate_id) = stack.pop() {
        if descendants.insert(candidate_id.clone())
            && let Some(candidate) = node_types
                .iter()
                .find(|node_type| node_type.id == candidate_id)
        {
            stack.extend(
                candidate
                    .child_relationships
                    .iter()
                    .map(|peer| peer.node_type_id.clone()),
            );
        }
    }

    descendants
}
