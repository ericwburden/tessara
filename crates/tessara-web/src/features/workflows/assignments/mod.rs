//! Workflow assignment module boundary.
//!
//! Re-export assignment API helpers, display formatters, and DTOs from here while keeping the broader workflow feature independent of assignment internals.

mod api;
mod components;
mod create_form;
mod display;
mod loaders;
mod mutations;
mod state;
pub(crate) mod types;

pub(crate) use crate::features::workflows::pages::assignments::WorkflowAssignmentsPage;
pub(in crate::features::workflows) use components::WorkflowAssignmentsList;
pub(in crate::features::workflows) use create_form::WorkflowAssignmentCreateForm;
pub(crate) use display::{
    workflow_assigned_user_links, workflow_assignee_label, workflow_assignment_candidate_key,
    workflow_assignment_revision_label, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
    workflow_available_node_links,
};
pub(crate) use loaders::{
    load_pending_work, load_workflow_assignment_assignees, load_workflow_assignment_candidates,
    load_workflow_assignments,
};
pub(crate) use mutations::{submit_workflow_assignment_bulk, toggle_workflow_assignment};
pub(in crate::features::workflows) use state::{assignee_filter_options, filtered_assignments};
#[cfg(feature = "hydrate")]
pub(crate) use types::{BulkWorkflowAssignmentPayload, UpdateWorkflowAssignmentPayload};
pub(crate) use types::{
    PendingWorkflowWork, WorkflowAssigneeOption, WorkflowAssignmentCandidate,
    WorkflowAssignmentSummary,
};
