//! Dataset editor helpers and feature-local editor logic.

mod expression;
mod field_options;
mod fields;
mod helpers;
mod identity;
mod lifecycle;
mod messages;
mod operation_options;
mod options;
mod source_options;
mod source_options_panel;
mod sources;
mod sql_preview;
mod state;
mod submit;
mod surface;
mod visibility;

pub(crate) use expression::{DatasetExpressionChain, ExpressionPreview};
pub(crate) use field_options::FieldOptionsPanel;
pub(crate) use fields::DatasetFieldsEditor;
pub(crate) use identity::DatasetIdentitySection;
pub(crate) use lifecycle::install_dataset_editor_loaders;
pub(crate) use messages::DatasetEditorMessages;
pub(crate) use operation_options::OperationOptionsPanel;
pub(crate) use options::DatasetDesignerOptionsSheet;
pub(crate) use source_options_panel::SourceOptionsPanel;
pub(crate) use sources::DatasetSourcesEditor;
pub(crate) use sql_preview::DatasetSqlPreviewPanel;
pub(crate) use state::DatasetEditorState;
pub(crate) use submit::submit_dataset_editor;
pub(crate) use surface::DatasetEditorSurface;
pub(crate) use visibility::DatasetVisibilityEditor;
