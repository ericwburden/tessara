//! Collision helpers for form builder grid layout.

use crate::features::forms::builder::FormBuilderFieldDraft;

pub(super) fn form_builder_fields_overlap(
    left: &FormBuilderFieldDraft,
    right: &FormBuilderFieldDraft,
) -> bool {
    if left.section_id != right.section_id || left.id == right.id {
        return false;
    }

    let left_row_start = left.grid_row.max(1);
    let left_row_end = left_row_start + left.grid_height.max(1) - 1;
    let left_column_start = left.grid_column.max(1);
    let left_column_end = left_column_start + left.grid_width.max(1) - 1;

    let right_row_start = right.grid_row.max(1);
    let right_row_end = right_row_start + right.grid_height.max(1) - 1;
    let right_column_start = right.grid_column.max(1);
    let right_column_end = right_column_start + right.grid_width.max(1) - 1;

    left_row_start <= right_row_end
        && left_row_end >= right_row_start
        && left_column_start <= right_column_end
        && left_column_end >= right_column_start
}

pub(crate) fn form_builder_field_has_collision(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> bool {
    fields
        .iter()
        .any(|candidate| candidate.id != field.id && form_builder_fields_overlap(field, candidate))
}

pub(super) fn form_builder_linear_grid_index(
    field: &FormBuilderFieldDraft,
    column_count: i32,
) -> i32 {
    let column_count = column_count.max(1);
    (field.grid_row.max(1) - 1) * column_count + field.grid_column.max(1) - 1
}
