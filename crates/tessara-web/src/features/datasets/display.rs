//! Display helpers for Datasets feature screens.

use super::types::DatasetVisibilityNode;

/// Returns the visible label for a dataset visibility scope.
pub(crate) fn visibility_label(nodes: &[DatasetVisibilityNode]) -> String {
    match nodes.len() {
        0 => "No nodes".into(),
        1 => nodes[0].node_path.clone(),
        count => format!("{count} nodes"),
    }
}
