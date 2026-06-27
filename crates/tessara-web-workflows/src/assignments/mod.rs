//! Workflow assignment module boundary.
//!
//! Re-export assignment API helpers, display formatters, and DTOs from here while keeping the broader workflow feature independent of assignment internals.

mod api;
mod assignee_picker;
mod candidate_pair_picker;
mod components;
mod create_form;
mod detail_sheet;
mod display;
mod errors;
mod filtering;
mod lifecycle;
mod loaders;
mod mobile_cards;
mod mutations;
mod page_state;
mod surface;
mod table_row;
pub(crate) mod types;

pub use crate::pages::assignments::WorkflowAssignmentsContent;
pub(crate) use assignee_picker::WorkflowAssignmentAssigneePicker;
pub(crate) use candidate_pair_picker::WorkflowAssignmentCandidatePairPicker;
pub(crate) use components::WorkflowAssignmentsList;
pub(crate) use create_form::WorkflowAssignmentCreateForm;
pub(crate) use detail_sheet::WorkflowAssignmentDetailSheet;
pub(crate) use display::{
    workflow_assigned_user_links, workflow_assignee_label, workflow_assignment_candidate_key,
    workflow_assignment_revision_label, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
    workflow_available_node_links,
};
pub(crate) use filtering::{assignee_filter_options, filtered_assignments};
pub(crate) use lifecycle::install_workflow_assignments_page_effects;
pub(crate) use loaders::{
    load_workflow_assignment_assignees, load_workflow_assignment_candidates,
    load_workflow_assignments,
};
pub(crate) use mobile_cards::WorkflowAssignmentMobileCards;
pub(crate) use mutations::toggle_workflow_assignment;
pub(crate) use page_state::WorkflowAssignmentsPageState;
pub(crate) use surface::WorkflowAssignmentsSurface;
pub(crate) use table_row::WorkflowAssignmentTableRow;
#[cfg(feature = "hydrate")]
pub(crate) use types::{BulkWorkflowAssignmentPayload, UpdateWorkflowAssignmentPayload};
pub(crate) use types::{WorkflowAssignmentCandidate, WorkflowAssignmentSummary};
