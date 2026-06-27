//! Public boundary for the Workflows feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Workflows-specific implementation details in child modules.

mod api;
pub(crate) mod assignments;
mod detail;
mod detail_tables;
mod display;
mod editor;
mod filtering;
mod http;
mod list;
mod list_panels;
mod loaders;
#[cfg(feature = "hydrate")]
mod options;
mod pages;
mod pagination;
mod payloads;
mod shared;
mod slug;
mod text;
pub(crate) mod types;
mod url;

pub(crate) use assignments::{
    workflow_assigned_user_links, workflow_assignee_label, workflow_assignment_revision_label,
    workflow_assignment_state, workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label, workflow_available_node_links,
};
pub(crate) use detail::WorkflowDetailBody;
pub(crate) use detail_tables::{
    WorkflowDetailAssignmentsTable, WorkflowStepsTable, WorkflowVersionsTable,
};
pub(crate) use display::{
    WorkflowSourceMarker, active_workflow_definition_version, workflow_assigned_users_label,
    workflow_available_nodes_label, workflow_definition_status_label,
    workflow_definition_version_label, workflow_description_label,
    workflow_revision_label_from_option, workflow_revision_label_from_raw, workflow_source_label,
    workflow_status_key, workflow_status_label, workflow_version_label,
};
pub(crate) use list_panels::{
    WorkflowAssignedUsersList, WorkflowAssignedUsersSheet, WorkflowAvailableNodesList,
    WorkflowAvailableNodesSheet,
};
pub(crate) use payloads::CreateWorkflowStepPayload;
#[cfg(feature = "hydrate")]
pub(crate) use payloads::{
    CreateWorkflowPayload, CreateWorkflowRevisionPayload, UpdateWorkflowPayload,
    UpdateWorkflowRevisionStepsPayload,
};
pub(crate) use types::{WorkflowSaveIntent, WorkflowStepDraft};

pub use assignments::WorkflowAssignmentsContent;
pub use editor::{WorkflowEditContent, WorkflowNewContent};
pub use pages::detail::WorkflowDetailContent;
pub use pages::list::WorkflowsIndexContent;
