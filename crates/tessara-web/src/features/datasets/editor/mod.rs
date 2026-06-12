//! Dataset editor helpers and feature-local editor logic.

mod expression;
mod field_options;
mod fields;
mod helpers;
mod identity;
mod messages;
mod options;
mod source_options;
mod sources;
mod sql_preview;
mod state;
mod surface;
mod visibility;

pub(crate) use expression::{DatasetExpressionChain, ExpressionPreview};
pub(crate) use field_options::FieldOptionsPanel;
pub(crate) use fields::DatasetFieldsEditor;
pub(crate) use identity::DatasetIdentitySection;
pub(crate) use messages::DatasetEditorMessages;
pub(crate) use options::DatasetDesignerOptionsSheet;
pub(crate) use sources::DatasetSourcesEditor;
pub(crate) use sql_preview::DatasetSqlPreviewPanel;
pub(crate) use state::DatasetEditorState;
pub(crate) use surface::DatasetEditorSurface;
pub(crate) use visibility::DatasetVisibilityEditor;
