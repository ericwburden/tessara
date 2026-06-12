//! Workflow editor form sections.

mod active_revision;
mod availability;
mod identity;
mod steps;

pub(in crate::features::workflows) use active_revision::WorkflowActiveRevisionSection;
pub(in crate::features::workflows) use availability::WorkflowAvailabilitySection;
pub(in crate::features::workflows) use identity::WorkflowIdentityFields;
pub(in crate::features::workflows) use steps::{
    WorkflowCreateStepsSection, WorkflowEditStepsSection,
};
