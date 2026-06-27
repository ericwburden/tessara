//! Workflow detail tables and assignment panels.

mod assignments;
mod steps;
mod versions;

pub(crate) use assignments::WorkflowDetailAssignmentsTable;
pub(crate) use steps::WorkflowStepsTable;
pub(crate) use versions::WorkflowVersionsTable;
