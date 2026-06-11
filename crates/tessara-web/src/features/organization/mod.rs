//! Public boundary for the Organization feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Organization-specific implementation details in child modules.

pub(crate) mod api;
mod detail;
mod node_editor;
mod node_metadata;
mod node_options;
pub(crate) mod pages;
mod related_work;
mod related_work_controls;
mod related_work_tables;
mod tree;
pub(crate) mod types;
pub(crate) use api::IntoNonemptyString;
#[cfg(feature = "hydrate")]
pub(crate) use api::current_search_param;
#[cfg(feature = "hydrate")]
pub(crate) use api::editable_form_definition_version;
pub(crate) use api::{
    active_form_definition_version, active_form_version, form_version_label,
    form_version_sort_label, workflow_assigned_users_label,
};
pub(crate) use api::{
    load_workflow_assignment_nodes, load_workflows, submit_create_form, submit_create_workflow,
    submit_update_form, submit_update_workflow, submit_workflow_assignment_bulk,
    toggle_workflow_assignment, workflow_step_form_version_id_by_id, workflow_step_signature,
    workflow_step_title_by_id,
};
pub(crate) use pages::{
    OrganizationDetailPage, OrganizationEditPage, OrganizationNewPage, OrganizationPage,
};
pub(crate) use related_work::RelatedWorkPaginationFooter;
pub(crate) use types::{
    AdminRoleSummary, NodeMetadataFieldSummary, NodeTypeCatalogEntry, NodeTypeDefinition,
    NodeTypeFormLink, NodeTypeUpsertRequest, OrganizationNode,
};
#[cfg(feature = "hydrate")]
pub(crate) use types::{
    CreateNodeMetadataFieldRequest, IdResponse, UpdateNodeMetadataFieldRequest,
};
