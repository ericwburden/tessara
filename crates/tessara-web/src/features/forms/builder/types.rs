//! Data contracts for the Forms feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Forms.

pub(crate) const FORM_BUILDER_COLUMN_COUNT: i32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct FormBuilderDragPreview {
    pub(crate) field_id: usize,
    pub(crate) section_id: usize,
    pub(crate) row: i32,
    pub(crate) column: i32,
}

#[derive(Clone, Copy)]
pub(crate) enum FormBuilderResizeAxis {
    Width,
    Height,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderSectionDraft {
    pub(crate) id: usize,
    pub(crate) remote_id: Option<String>,
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) default_column_width: i32,
    pub(crate) position: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderFieldDraft {
    pub(crate) id: usize,
    pub(crate) remote_id: Option<String>,
    pub(crate) section_id: usize,
    pub(crate) label: String,
    pub(crate) key: String,
    pub(crate) field_type: String,
    pub(crate) required: bool,
    pub(crate) grid_row: i32,
    pub(crate) grid_column: i32,
    pub(crate) grid_width: i32,
    pub(crate) grid_height: i32,
    pub(crate) key_was_edited: bool,
}

/// Handles the blank form builder section behavior.
pub(crate) fn blank_form_builder_section(id: usize) -> FormBuilderSectionDraft {
    FormBuilderSectionDraft {
        id,
        remote_id: None,
        title: if id == 1 {
            "Main".into()
        } else {
            format!("Section {id}")
        },
        description: String::new(),
        default_column_width: 6,
        position: id as i32,
    }
}
