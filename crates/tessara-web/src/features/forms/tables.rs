//! Table components for the Forms feature.
//!
//! Keep reusable Forms table rendering here when several pages share row layout or table affordances.

mod dataset_sources;
mod workflows;

pub(crate) use dataset_sources::FormRelatedDatasetSourcesTable;
pub(crate) use workflows::FormRelatedWorkflowsTable;
