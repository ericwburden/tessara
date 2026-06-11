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
#[cfg(feature = "hydrate")]
pub(crate) use filtering::unique_slug_from_label;
pub(crate) use filtering::{slug_from_label, unique_filter_options};
