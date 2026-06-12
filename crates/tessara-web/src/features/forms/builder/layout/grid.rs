//! Form builder grid layout rules.

pub(crate) use super::collision::form_builder_field_has_collision;
use super::collision::{form_builder_fields_overlap, form_builder_linear_grid_index};
use crate::features::forms::builder::FormBuilderFieldDraft;
use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderSectionDraft,
};
use std::collections::HashSet;

pub(crate) fn blank_form_builder_field_at(
    id: usize,
    section_id: usize,
    grid_row: i32,
    grid_column: i32,
    grid_width: i32,
) -> FormBuilderFieldDraft {
    FormBuilderFieldDraft {
        id,
        remote_id: None,
        section_id,
        label: String::new(),
        key: String::new(),
        field_type: "text".into(),
        required: false,
        grid_row,
        grid_column,
        grid_width: grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
        grid_height: 1,
        key_was_edited: false,
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderGridCell {
    pub(crate) row: i32,
    pub(crate) column: i32,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormBuilderSectionLayout {
    pub(crate) fields: Vec<FormBuilderFieldDraft>,
    pub(crate) occupied_cells: HashSet<(i32, i32)>,
    pub(crate) column_count: i32,
    pub(crate) row_count: i32,
}

pub(crate) fn form_builder_section_fields(
    section_id: usize,
    fields: &[FormBuilderFieldDraft],
) -> Vec<FormBuilderFieldDraft> {
    fields
        .iter()
        .filter(|field| field.section_id == section_id)
        .cloned()
        .collect()
}

pub(crate) fn form_builder_occupancy_map(fields: &[FormBuilderFieldDraft]) -> HashSet<(i32, i32)> {
    let mut occupied = HashSet::new();

    for field in fields {
        let row_start = field.grid_row.max(1);
        let row_end = row_start + field.grid_height.max(1) - 1;
        let column_start = field.grid_column.max(1);
        let column_end = column_start + field.grid_width.max(1) - 1;

        for row in row_start..=row_end {
            for column in column_start..=column_end {
                occupied.insert((row, column));
            }
        }
    }

    occupied
}

pub(crate) fn form_builder_section_layout(
    section: &FormBuilderSectionDraft,
    fields: &[FormBuilderFieldDraft],
) -> FormBuilderSectionLayout {
    let section_fields = form_builder_section_fields(section.id, fields);
    let occupied_cells = form_builder_occupancy_map(&section_fields);
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let bottom_occupied_row = section_fields
        .iter()
        .map(|field| field.grid_row.max(1) + field.grid_height.max(1) - 1)
        .max()
        .unwrap_or(0);
    let row_count = (bottom_occupied_row + 1).max(2);

    FormBuilderSectionLayout {
        fields: section_fields,
        occupied_cells,
        column_count,
        row_count,
    }
}

pub(crate) fn form_builder_reflow_section_fields(
    fields: &[FormBuilderFieldDraft],
    preview: FormBuilderDragPreview,
) -> Vec<FormBuilderFieldDraft> {
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let mut section_fields = fields
        .iter()
        .filter(|field| field.section_id == preview.section_id)
        .cloned()
        .map(|mut field| {
            if field.id == preview.field_id {
                field.grid_row = preview.row.max(1);
                field.grid_column = preview.column.max(1);
                field.grid_width = field.grid_width.min(column_count).max(1);
                let max_column = (column_count - field.grid_width + 1).max(1);
                field.grid_column = field.grid_column.clamp(1, max_column);
            }
            field
        })
        .collect::<Vec<_>>();

    section_fields.sort_by(|left, right| {
        form_builder_linear_grid_index(left, column_count)
            .cmp(&form_builder_linear_grid_index(right, column_count))
            .then_with(|| {
                let left_dragged = left.id == preview.field_id;
                let right_dragged = right.id == preview.field_id;
                right_dragged.cmp(&left_dragged)
            })
            .then(left.id.cmp(&right.id))
    });

    let mut placed = Vec::new();

    for field in section_fields {
        let width = field.grid_width.clamp(1, column_count);
        let height = field.grid_height.clamp(1, 6);
        let start_index = form_builder_linear_grid_index(&field, column_count).max(0);

        for index in start_index..=(column_count * 240) {
            let row = index / column_count + 1;
            let column = index % column_count + 1;

            if column + width - 1 > column_count {
                continue;
            }

            let mut candidate = field.clone();
            candidate.grid_row = row;
            candidate.grid_column = column;
            candidate.grid_width = width;
            candidate.grid_height = height;

            if !placed
                .iter()
                .any(|placed_field| form_builder_fields_overlap(&candidate, placed_field))
            {
                placed.push(candidate);
                break;
            }
        }
    }

    fields
        .iter()
        .filter(|field| field.section_id != preview.section_id)
        .cloned()
        .chain(placed)
        .collect()
}
