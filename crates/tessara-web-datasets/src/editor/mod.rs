//! Dataset editor helpers and feature-local editor logic.

mod aggregation;
mod calculations;
mod fields;
mod filters;
mod helpers;
mod identity;
mod lifecycle;
mod messages;
mod operations;
mod pipeline_fields;
mod source_field_actions;
mod source_options;
mod source_options_panel;
mod sources;
mod sql_preview;
mod state;
mod submit;
mod surface;
mod visibility;

pub(crate) use calculations::DatasetRestrictionsEditor;
pub(crate) use identity::DatasetIdentitySection;
pub(crate) use lifecycle::install_dataset_editor_loaders;
pub(crate) use messages::DatasetEditorMessages;
pub(crate) use operations::DatasetOperationSequence;
#[cfg(feature = "hydrate")]
pub(crate) use source_field_actions::canonical_field_key;
pub(crate) use source_options_panel::SourceOptionsFields;
pub(crate) use sources::DatasetSourcesEditor;
pub(crate) use sql_preview::DatasetSqlPreviewPanel;
pub(crate) use state::DatasetEditorState;
pub(crate) use submit::submit_dataset_editor;
pub(crate) use surface::DatasetEditorSurface;
pub(crate) use visibility::DatasetVisibilityEditor;
