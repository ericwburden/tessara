//! Collision helpers for form builder grid layout.

use crate::builder::FormBuilderFieldDraft;
use tessara_core::GridRect;

pub(super) fn form_builder_fields_overlap(
    left: &FormBuilderFieldDraft,
    right: &FormBuilderFieldDraft,
) -> bool {
    if left.section_id != right.section_id || left.id == right.id {
        return false;
    }

    field_rect(left).overlaps(field_rect(right))
}

pub(crate) fn form_builder_field_has_collision(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> bool {
    fields
        .iter()
        .any(|candidate| candidate.id != field.id && form_builder_fields_overlap(field, candidate))
}

pub(super) fn field_rect(field: &FormBuilderFieldDraft) -> GridRect {
    GridRect::new(
        field.grid_row.max(1),
        field.grid_column.max(1),
        field.grid_width.max(1),
        field.grid_height.max(1),
    )
}
