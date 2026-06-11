//! Display helpers for Datasets feature screens.

use super::types::DatasetVisibilityNode;
use crate::utils::pagination::{pagination_page_end, pagination_page_start};

/// Returns the visible label for a dataset visibility scope.
pub(crate) fn visibility_label(nodes: &[DatasetVisibilityNode]) -> String {
    match nodes.len() {
        0 => "No nodes".into(),
        1 => nodes[0].node_path.clone(),
        count => format!("{count} nodes"),
    }
}

/// Returns the row summary shown beside dataset pagination controls.
pub(crate) fn table_summary(
    total_count: usize,
    page_size: usize,
    page_index: usize,
    label: &str,
) -> String {
    if total_count == 0 {
        format!("No {label} to display")
    } else {
        format!(
            "Showing {}-{} of {} {label}",
            pagination_page_start(total_count, page_size, page_index) + 1,
            pagination_page_end(total_count, page_size, page_index),
            total_count
        )
    }
}
