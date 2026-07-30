//! Data contracts for the Forms feature.
//!
//! Keep API response shapes, request payloads, and feature-local value objects here when they are owned by Forms.

pub(crate) const FORM_BUILDER_COLUMN_COUNT: i32 =
    tessara_module_ui::placement_editor::PLACEMENT_GRID_COLUMN_COUNT;

pub(crate) type FormBuilderDragPreview =
    tessara_module_ui::placement_editor::PlacementDragPreview<usize, usize>;
pub(crate) type FormBuilderResizeAxis = tessara_core::GridResizeAxis;

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
