//! Form builder field sizing rules.

use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, form_builder_field_has_collision,
};

/// Handles the max form builder new field width at behavior.
pub(crate) fn max_form_builder_new_field_width_at(
    section_id: usize,
    row: i32,
    column: i32,
    fields: &[FormBuilderFieldDraft],
) -> i32 {
    let row = row.max(1);
    let column = column.clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let mut width = 0;

    for candidate_column in column..=FORM_BUILDER_COLUMN_COUNT {
        let candidate = FormBuilderFieldDraft {
            id: usize::MAX,
            remote_id: None,
            section_id,
            label: String::new(),
            key: String::new(),
            field_type: "text".into(),
            required: false,
            grid_row: row,
            grid_column: column,
            grid_width: candidate_column - column + 1,
            grid_height: 1,
            key_was_edited: false,
        };

        if form_builder_field_has_collision(&candidate, fields) {
            break;
        }

        width += 1;
    }

    width.max(1)
}

/// Handles the max form builder field width behavior.
pub(crate) fn max_form_builder_field_width(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> i32 {
    let row = field.grid_row.max(1);
    let column = field.grid_column.max(1);
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let mut width = 0;

    for candidate_column in column..=column_count {
        let mut candidate = field.clone();
        candidate.grid_row = row;
        candidate.grid_column = column;
        candidate.grid_width = candidate_column - column + 1;

        let blocked = form_builder_field_has_collision(&candidate, fields);

        if blocked {
            break;
        }

        width += 1;
    }

    width.max(1)
}

/// Handles the max form builder field height behavior.
pub(crate) fn max_form_builder_field_height(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> i32 {
    let mut height = 0;

    for candidate_height in 1..=6 {
        let mut candidate = field.clone();
        candidate.grid_height = candidate_height;

        if form_builder_field_has_collision(&candidate, fields) {
            break;
        }

        height += 1;
    }

    height.max(1)
}

/// Handles the form builder layout candidate behavior.
pub(crate) fn form_builder_layout_candidate(
    field: &FormBuilderFieldDraft,
    control_index: usize,
    value: i32,
) -> FormBuilderFieldDraft {
    let mut candidate = field.clone();

    match control_index {
        0 => candidate.grid_row = value,
        1 => {
            let max_column = (FORM_BUILDER_COLUMN_COUNT - candidate.grid_width.max(1) + 1)
                .clamp(1, FORM_BUILDER_COLUMN_COUNT);
            candidate.grid_column = value.clamp(1, max_column);
        }
        2 => candidate.grid_width = value,
        _ => candidate.grid_height = value.clamp(1, 6),
    }

    candidate
}

/// Handles the valid form builder layout values behavior.
pub(crate) fn valid_form_builder_layout_values(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
    control_index: usize,
    max_value: i32,
) -> Vec<i32> {
    let current_value = match control_index {
        0 => field.grid_row,
        1 => field.grid_column,
        2 => field.grid_width,
        _ => field.grid_height,
    }
    .max(1);

    let mut values = (1..=max_value.max(1))
        .filter(|value| {
            let candidate = form_builder_layout_candidate(field, control_index, *value);
            let candidate_column_end =
                candidate.grid_column.max(1) + candidate.grid_width.max(1) - 1;

            candidate_column_end <= FORM_BUILDER_COLUMN_COUNT
                && !form_builder_field_has_collision(&candidate, fields)
        })
        .collect::<Vec<_>>();

    let current_candidate = form_builder_layout_candidate(field, control_index, current_value);
    let current_column_end =
        current_candidate.grid_column.max(1) + current_candidate.grid_width.max(1) - 1;
    let current_is_valid = current_column_end <= FORM_BUILDER_COLUMN_COUNT
        && !form_builder_field_has_collision(&current_candidate, fields);

    if current_is_valid && !values.contains(&current_value) {
        values.push(current_value);
        values.sort_unstable();
    }

    values
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the set form builder field size behavior.
pub(crate) fn set_form_builder_field_size(
    fields: &mut [FormBuilderFieldDraft],
    field_id: usize,
    width: i32,
    height: i32,
) {
    let Some(position) = fields.iter().position(|field| field.id == field_id) else {
        return;
    };

    let mut candidate = fields[position].clone();
    candidate.grid_width = width.clamp(1, FORM_BUILDER_COLUMN_COUNT);
    candidate.grid_height = height.clamp(1, 6);

    let column_end = candidate.grid_column.max(1) + candidate.grid_width.max(1) - 1;
    if column_end > FORM_BUILDER_COLUMN_COUNT {
        return;
    }

    if form_builder_field_has_collision(&candidate, fields) {
        return;
    }

    fields[position] = candidate;
}
