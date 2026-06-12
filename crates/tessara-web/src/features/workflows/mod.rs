//! Public boundary for the Workflows feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Workflows-specific implementation details in child modules.

mod api;
pub(crate) mod assignments;
mod detail;
mod detail_tables;
mod display;
mod editor;
mod list;
mod list_panels;
mod loaders;
#[cfg(feature = "hydrate")]
mod options;
mod pages;
mod payloads;
pub(crate) mod types;

pub(crate) use assignments::{
    WorkflowAssignmentsPage, submit_workflow_assignment_bulk, toggle_workflow_assignment,
    workflow_assigned_user_links, workflow_assignee_label, workflow_assignment_candidate_key,
    workflow_assignment_revision_label, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
    workflow_available_node_links,
};
pub(in crate::features::workflows) use detail::WorkflowDetailContent;
pub(crate) use detail::WorkflowsDetailPage;
pub(in crate::features::workflows) use detail_tables::{
    WorkflowDetailAssignmentsTable, WorkflowStepsTable, WorkflowVersionsTable,
};
pub(crate) use display::{
    WorkflowSourceMarker, active_workflow_definition_version, workflow_assigned_users_label,
    workflow_available_nodes_label, workflow_definition_status_label,
    workflow_definition_version_label, workflow_description_label,
    workflow_revision_label_from_option, workflow_revision_label_from_raw, workflow_source_label,
    workflow_status_key, workflow_status_label, workflow_version_label,
};
pub(crate) use editor::workflow_step_signature;
pub(crate) use editor::{
    WorkflowsEditPage, WorkflowsNewPage, submit_create_workflow, submit_update_workflow,
    workflow_form_version_options,
};
pub(crate) use list::WorkflowsPage;
pub(in crate::features::workflows) use list_panels::{
    WorkflowAssignedUsersList, WorkflowAssignedUsersSheet, WorkflowAvailableNodesList,
    WorkflowAvailableNodesSheet,
};
pub(crate) use loaders::{
    load_workflow_assignment_nodes, load_workflow_create_options, load_workflow_detail,
    load_workflows,
};
pub(crate) use payloads::CreateWorkflowStepPayload;
#[cfg(feature = "hydrate")]
pub(crate) use payloads::{
    CreateWorkflowPayload, CreateWorkflowRevisionPayload, UpdateWorkflowPayload,
    UpdateWorkflowRevisionStepsPayload,
};
pub(crate) use types::{WorkflowSaveIntent, WorkflowStepDraft};
