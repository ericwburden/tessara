//! Editing components and helpers for the Workflows feature.
//!
//! Keep form state, draft manipulation, and edit-page presentation here; transport payload submission belongs in API modules.

mod available_nodes_picker;
mod create;
mod create_actions;
mod edit;
mod options;
mod sections;
mod state;
mod step_list;
mod steps;
mod update_actions;

pub(in crate::features::workflows) use available_nodes_picker::WorkflowAvailableNodesPicker;
pub(crate) use create::WorkflowsNewPage;
pub(crate) use create_actions::submit_create_workflow;
pub(crate) use edit::WorkflowsEditPage;
#[cfg(feature = "hydrate")]
pub(crate) use options::existing_workflow_slugs;
pub(crate) use options::workflow_form_version_options;
pub(in crate::features::workflows) use sections::{
    WorkflowActiveRevisionSection, WorkflowAvailabilitySection, WorkflowCreateStepsSection,
    WorkflowEditStepsSection, WorkflowIdentityFields,
};
pub(in crate::features::workflows) use state::{
    add_workflow_step, can_submit_workflow_editor, prune_unavailable_workflow_steps,
    workflow_edit_initial_state,
};
pub(in crate::features::workflows) use step_list::WorkflowStepList;
#[cfg(feature = "hydrate")]
pub(in crate::features::workflows) use steps::workflow_step_payloads_from_drafts;
pub(crate) use steps::workflow_step_signature;
pub(crate) use update_actions::submit_update_workflow;
