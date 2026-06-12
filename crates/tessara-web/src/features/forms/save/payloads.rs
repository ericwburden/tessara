//! Shared payload builders for form save operations.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
use crate::features::forms::types::{CreateFormFieldPayload, CreateFormSectionPayload};

/// Builds the section payload used by create and update form saves.
pub(super) fn form_section_payload(section: &FormBuilderSectionDraft) -> CreateFormSectionPayload {
    CreateFormSectionPayload {
        title: section.title.clone(),
        position: section.position,
        description: section.description.clone(),
    }
}

/// Builds the field payload used by create and update form saves.
pub(super) fn form_field_payload(
    field: &FormBuilderFieldDraft,
    section_id: String,
    position: i32,
) -> CreateFormFieldPayload {
    CreateFormFieldPayload {
        section_id,
        key: field.key.clone(),
        label: field.label.clone(),
        field_type: field.field_type.clone(),
        required: field.required,
        position,
        grid_row: field.grid_row,
        grid_column: field.grid_column,
        grid_width: field.grid_width,
        grid_height: field.grid_height,
    }
}
