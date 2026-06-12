//! Filtering helpers for the operations dataset readiness table.

use crate::features::operations::types::DatasetStatus;
use crate::utils::text::text_matches;

pub(super) fn filtered_dataset_readiness(
    datasets: &[DatasetStatus],
    query: &str,
    selected_status: &str,
) -> Vec<DatasetStatus> {
    datasets
        .iter()
        .filter(|dataset| {
            let matches_status = selected_status == "all" || dataset.readiness == selected_status;
            matches_status
                && text_matches(
                    query,
                    &[
                        dataset.dataset_name.as_str(),
                        dataset.readiness.as_str(),
                        dataset.revision_status.as_str(),
                    ],
                )
        })
        .cloned()
        .collect()
}
