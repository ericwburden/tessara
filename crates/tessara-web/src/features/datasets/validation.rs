//! Pure validation and matching helpers for Datasets.

use super::types::NodeResponse;
use crate::utils::text::text_matches;

/// Returns whether an organization node matches a dataset visibility search query.
pub(crate) fn node_matches_visibility_query(node: &NodeResponse, query: &str) -> bool {
    query.trim().is_empty()
        || text_matches(query, &[&node.name])
        || text_matches(query, &[&node.node_type_name])
        || node
            .parent_node_name
            .as_ref()
            .is_some_and(|parent| text_matches(query, &[parent]))
}
