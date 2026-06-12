//! Public boundary for the Shared feature.
//!
//! Re-export only the pages, types, and helpers other modules need; keep Shared-specific implementation details in child modules.

mod display;
mod placeholder;
mod types;
mod ui;
pub(crate) use display::status_badge_class;
pub(crate) use placeholder::NativePlaceholderRoute;
pub(crate) use types::{
    FormAttachmentLink, FormsAttachedNodesSheetData, WorkflowAssignedUsersSheetData,
    WorkflowAvailableNodesSheetData,
};
pub(crate) use ui::{node_count_label, node_display_path, user_count_label};
