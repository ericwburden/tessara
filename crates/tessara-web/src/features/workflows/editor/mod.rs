//! Editing components and helpers for the Workflows feature.
//!
//! Keep form state, draft manipulation, and edit-page presentation here; transport payload submission belongs in API modules.

mod available_nodes_picker;
mod options;
mod pages;
mod state;
mod step_list;
mod steps;

pub(in crate::features::workflows) use available_nodes_picker::WorkflowAvailableNodesPicker;
#[cfg(feature = "hydrate")]
pub(crate) use options::existing_workflow_slugs;
pub(crate) use options::workflow_form_version_options;
pub(crate) use pages::{WorkflowsEditPage, WorkflowsNewPage};
pub(in crate::features::workflows) use state::{
    add_workflow_step, can_submit_workflow_editor, prune_unavailable_workflow_steps,
};
pub(in crate::features::workflows) use step_list::WorkflowStepList;
#[cfg(feature = "hydrate")]
pub(in crate::features::workflows) use steps::workflow_step_payloads_from_drafts;
pub(crate) use steps::workflow_step_signature;
