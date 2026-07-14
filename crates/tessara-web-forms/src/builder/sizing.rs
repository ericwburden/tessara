//! Form builder field sizing rules.

use crate::builder::{
    FORM_BUILDER_COLUMN_COUNT, FORM_BUILDER_GRID_CONSTRAINTS, FormBuilderFieldDraft,
    form_builder_field_has_collision,
};
use tessara_core::{
    GridMoveRequest, GridPlacement, GridRect, GridResizeRequest, GridSize, resolve_move_request,
    resolve_resize_request, validate_resize,
};

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

pub(crate) fn form_builder_layout_candidate(
    field: &FormBuilderFieldDraft,
    control_index: usize,
    value: i32,
) -> FormBuilderFieldDraft {
    let mut candidate = field.clone();
    let current = GridRect::new(
        field.grid_row.max(1),
        field.grid_column.max(1),
        field.grid_width.max(1),
        field.grid_height.max(1),
    );

    match control_index {
        0 => {
            let request = GridMoveRequest::Direct {
                row: value,
                column: current.column,
            };
            candidate.grid_row =
                resolve_move_request(FORM_BUILDER_GRID_CONSTRAINTS, current, request)
                    .map(|rect| rect.row)
                    .unwrap_or(value);
        }
        1 => {
            let max_column = (FORM_BUILDER_COLUMN_COUNT - candidate.grid_width.max(1) + 1)
                .clamp(1, FORM_BUILDER_COLUMN_COUNT);
            let column = value.clamp(1, max_column);
            let request = GridMoveRequest::Direct {
                row: current.row,
                column,
            };
            candidate.grid_column =
                resolve_move_request(FORM_BUILDER_GRID_CONSTRAINTS, current, request)
                    .map(|rect| rect.column)
                    .unwrap_or(column);
        }
        2 => {
            let request = GridResizeRequest::Direct(GridSize::new(value, current.height));
            candidate.grid_width = resolve_resize_request(
                FORM_BUILDER_GRID_CONSTRAINTS,
                current,
                request,
                GridSize::ONE,
            )
            .map(|size| size.width)
            .unwrap_or(value);
        }
        _ => {
            let height = value.clamp(1, 6);
            let request = GridResizeRequest::Direct(GridSize::new(current.width, height));
            candidate.grid_height = resolve_resize_request(
                FORM_BUILDER_GRID_CONSTRAINTS,
                current,
                request,
                GridSize::ONE,
            )
            .map(|size| size.height)
            .unwrap_or(height);
        }
    }

    candidate
}

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
    let section_placements = fields
        .iter()
        .filter(|field| field.section_id == candidate.section_id)
        .map(|field| {
            GridPlacement::new(
                field.id,
                GridRect::new(
                    field.grid_row.max(1),
                    field.grid_column.max(1),
                    field.grid_width.max(1),
                    field.grid_height.max(1),
                ),
            )
        })
        .collect::<Vec<_>>();
    let Ok(resized) = validate_resize(
        FORM_BUILDER_GRID_CONSTRAINTS,
        &section_placements,
        &field_id,
        GridSize::new(
            width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
            height.clamp(1, 6),
        ),
        GridSize::ONE,
    ) else {
        return;
    };

    candidate.grid_width = resized.rect.width;
    candidate.grid_height = resized.rect.height;
    fields[position] = candidate;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::blank_form_builder_field_at;

    fn field(id: usize, column: i32, width: i32) -> FormBuilderFieldDraft {
        blank_form_builder_field_at(id, 1, 1, column, width)
    }

    #[test]
    fn form_resize_keeps_last_valid_size_when_growth_would_collide() {
        let mut fields = vec![field(1, 1, 4), field(2, 7, 6)];

        set_form_builder_field_size(&mut fields, 1, 6, 2);
        assert_eq!((fields[0].grid_width, fields[0].grid_height), (6, 2));

        set_form_builder_field_size(&mut fields, 1, 7, 2);
        assert_eq!((fields[0].grid_width, fields[0].grid_height), (6, 2));
    }

    #[test]
    fn form_direct_layout_candidates_preserve_clamping_contract() {
        let current = field(1, 10, 3);
        assert_eq!(
            form_builder_layout_candidate(&current, 1, 12).grid_column,
            10
        );
        assert_eq!(
            form_builder_layout_candidate(&current, 3, 99).grid_height,
            6
        );
    }
}
