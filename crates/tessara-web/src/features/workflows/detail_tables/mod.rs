//! Workflow detail tables and assignment panels.

mod assignments;
mod steps;
mod versions;

pub(in crate::features::workflows) use assignments::WorkflowDetailAssignmentsTable;
pub(in crate::features::workflows) use steps::WorkflowStepsTable;
pub(in crate::features::workflows) use versions::WorkflowVersionsTable;
