//! Editing components and helpers for the Workflows feature.
//!
//! Keep form state, draft manipulation, and edit-page presentation here; transport payload submission belongs in API modules.

mod action_helpers;
mod api;
mod available_nodes_picker;
mod create;
mod create_actions;
mod edit;
mod edit_form;
#[cfg(feature = "hydrate")]
mod errors;
mod options;
#[cfg(feature = "hydrate")]
mod payloads;
mod sections;
mod seed;
mod state;
mod step_list;
mod steps;
mod update_actions;
#[cfg(feature = "hydrate")]
mod update_payloads;
#[cfg(feature = "hydrate")]
mod validation;

pub(crate) use available_nodes_picker::WorkflowAvailableNodesPicker;
pub use create::WorkflowNewContent;
pub(crate) use create_actions::submit_create_workflow;
pub use edit::WorkflowEditContent;
pub(crate) use edit_form::WorkflowEditForm;
#[cfg(feature = "hydrate")]
pub(crate) use options::existing_workflow_slugs;
pub(crate) use options::workflow_form_version_options;
pub(crate) use sections::{
    WorkflowActiveRevisionSection, WorkflowAvailabilitySection, WorkflowCreateStepsSection,
    WorkflowEditStepsSection, WorkflowIdentityFields,
};
pub(crate) use seed::seed_workflow_from_form_query;
pub(crate) use state::{
    add_workflow_step, can_submit_workflow_editor, prune_unavailable_workflow_steps,
    workflow_edit_initial_state,
};
pub(crate) use step_list::WorkflowStepList;
#[cfg(feature = "hydrate")]
pub(crate) use steps::workflow_step_payloads_from_drafts;
pub(crate) use steps::workflow_step_signature;
pub(crate) use update_actions::{SubmitUpdateWorkflowInput, submit_update_workflow};
