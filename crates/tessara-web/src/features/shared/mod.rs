//! Public boundary for the Shared feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Shared-specific implementation details in child modules.

mod display;
mod filter_header;
mod helpers;
mod placeholder;
mod types;
mod ui;
pub(crate) use display::status_badge_class;
pub(crate) use filter_header::FilterHeader;
pub(crate) use placeholder::NativePlaceholderRoute;
pub(crate) use types::{
    FormAttachmentLink, FormsAttachedNodesSheetData, WorkflowAssignedUsersSheetData,
    WorkflowAvailableNodesSheetData,
};
pub(crate) use ui::{node_count_label, node_display_path, user_count_label};

mod filtering;
pub(crate) use filtering::{
    FormNodeFilterOption, form_matches_node_filter, form_node_filter_options, indented_node_label,
    slug_from_label, unique_filter_options, visible_form_node_filter_options,
    workflow_form_version_options, workflow_step_form_label,
};
#[cfg(feature = "hydrate")]
pub(crate) use filtering::{
    existing_form_slugs, existing_form_slugs_for_update, existing_workflow_slugs,
    unique_slug_from_label,
};
