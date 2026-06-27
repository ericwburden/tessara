//! Workflow editor form sections.

mod active_revision;
mod availability;
mod identity;
mod steps;

pub(crate) use active_revision::WorkflowActiveRevisionSection;
pub(crate) use availability::WorkflowAvailabilitySection;
pub(crate) use identity::WorkflowIdentityFields;
pub(crate) use steps::{WorkflowCreateStepsSection, WorkflowEditStepsSection};
