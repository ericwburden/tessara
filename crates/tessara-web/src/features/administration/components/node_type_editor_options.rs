//! Relationship option helpers for the node-type editor.

use super::super::graph::{node_type_ancestor_ids, node_type_descendant_ids};
use crate::features::organization::NodeTypeCatalogEntry;
use std::collections::HashSet;

pub(super) fn eligible_parent_node_types(
    all_node_types: &[NodeTypeCatalogEntry],
    current_id: Option<&String>,
    descendant_ids: &HashSet<String>,
    selected_child_ids: &HashSet<String>,
) -> Vec<NodeTypeCatalogEntry> {
    let mut disqualified_parent_ids = descendant_ids.clone();
    for child_id in selected_child_ids {
        disqualified_parent_ids.insert(child_id.clone());
        disqualified_parent_ids.extend(node_type_descendant_ids(child_id, all_node_types));
    }

    all_node_types
        .iter()
        .filter(|node_type| current_id != Some(&node_type.id))
        .filter(|node_type| !disqualified_parent_ids.contains(&node_type.id))
        .cloned()
        .collect()
}

pub(super) fn eligible_child_node_types(
    all_node_types: &[NodeTypeCatalogEntry],
    current_id: Option<&String>,
    ancestor_ids: &HashSet<String>,
    selected_parent_ids: &HashSet<String>,
) -> Vec<NodeTypeCatalogEntry> {
    let mut disqualified_child_ids = ancestor_ids.clone();
    for parent_id in selected_parent_ids {
        disqualified_child_ids.insert(parent_id.clone());
        disqualified_child_ids.extend(node_type_ancestor_ids(parent_id, all_node_types));
    }

    all_node_types
        .iter()
        .filter(|node_type| current_id != Some(&node_type.id))
        .filter(|node_type| !disqualified_child_ids.contains(&node_type.id))
        .cloned()
        .collect()
}
