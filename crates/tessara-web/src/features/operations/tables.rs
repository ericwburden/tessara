//! Table and summary component boundary for the Operations feature.

mod dataset_readiness;
mod summary;
mod workflow_assignments;

pub(crate) use dataset_readiness::DatasetReadinessTable;
pub(crate) use summary::OperationsSummaryPanel;
pub(crate) use workflow_assignments::WorkflowAssignmentsTable;
