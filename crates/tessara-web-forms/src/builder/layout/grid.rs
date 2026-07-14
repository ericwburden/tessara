//! Form builder grid layout rules.

use super::collision::field_rect;
pub(crate) use super::collision::form_builder_field_has_collision;
use crate::builder::FormBuilderFieldDraft;
use crate::builder::{FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderSectionDraft};
use std::collections::{HashMap, HashSet};
use tessara_core::{GridConstraints, GridPlacement, reflow_movement};

pub(crate) const FORM_BUILDER_GRID_CONSTRAINTS: GridConstraints =
    GridConstraints::new(FORM_BUILDER_COLUMN_COUNT, i32::MAX, usize::MAX, 6);

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
    fields
        .iter()
        .filter_map(|field| field_rect(field).occupied_cells().ok())
        .flatten()
        .collect()
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
        .map(|field| {
            field
                .grid_row
                .max(1)
                .saturating_add(field.grid_height.max(1).saturating_sub(1))
        })
        .max()
        .unwrap_or(0);
    let row_count = bottom_occupied_row.saturating_add(1).max(2);

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
    let section_fields = fields
        .iter()
        .filter(|field| field.section_id == preview.canvas_id)
        .cloned()
        .collect::<Vec<_>>();
    let placements = section_fields
        .iter()
        .map(|field| GridPlacement::new(field.id, field_rect(field)))
        .collect::<Vec<_>>();
    let Ok(placed) = reflow_movement(
        FORM_BUILDER_GRID_CONSTRAINTS,
        &placements,
        &preview.placement_id,
        preview.row,
        preview.column,
    ) else {
        return fields.to_vec();
    };
    let fields_by_id = section_fields
        .into_iter()
        .map(|field| (field.id, field))
        .collect::<HashMap<_, _>>();
    let placed = placed.into_iter().filter_map(|placement| {
        let mut field = fields_by_id.get(&placement.id)?.clone();
        field.grid_row = placement.rect.row;
        field.grid_column = placement.rect.column;
        field.grid_width = placement.rect.width;
        field.grid_height = placement.rect.height;
        Some(field)
    });

    fields
        .iter()
        .filter(|field| field.section_id != preview.canvas_id)
        .cloned()
        .chain(placed)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(
        id: usize,
        section_id: usize,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
    ) -> FormBuilderFieldDraft {
        let mut field = blank_form_builder_field_at(id, section_id, row, column, width);
        field.grid_height = height;
        field
    }

    #[test]
    fn section_layout_preserves_form_grid_rows_and_occupancy() {
        let section = FormBuilderSectionDraft {
            id: 1,
            remote_id: None,
            title: "Main".into(),
            description: String::new(),
            default_column_width: 6,
            position: 1,
        };
        let fields = vec![field(1, 1, 2, 3, 2, 2)];
        let layout = form_builder_section_layout(&section, &fields);

        assert_eq!(layout.column_count, 12);
        assert_eq!(layout.row_count, 4);
        assert_eq!(
            layout.occupied_cells,
            HashSet::from([(2, 3), (2, 4), (3, 3), (3, 4)])
        );
    }

    #[test]
    fn drag_reflow_keeps_dragged_field_at_target_and_row_major_order() {
        let fields = vec![
            field(1, 1, 1, 1, 6, 1),
            field(2, 1, 1, 7, 6, 1),
            field(3, 2, 1, 1, 12, 1),
        ];
        let result = form_builder_reflow_section_fields(
            &fields,
            FormBuilderDragPreview {
                placement_id: 2,
                canvas_id: 1,
                row: 1,
                column: 1,
            },
        );

        assert_eq!(
            result.iter().map(|field| field.id).collect::<Vec<_>>(),
            vec![3, 2, 1]
        );
        let moved = result.iter().find(|field| field.id == 2).expect("moved");
        let reflowed = result.iter().find(|field| field.id == 1).expect("reflowed");
        assert_eq!((moved.grid_row, moved.grid_column), (1, 1));
        assert_eq!((reflowed.grid_row, reflowed.grid_column), (1, 7));
    }

    #[test]
    fn form_reflow_is_not_limited_by_dashboard_row_capacity() {
        let fields = vec![field(1, 1, 241, 1, 12, 1)];
        let result = form_builder_reflow_section_fields(
            &fields,
            FormBuilderDragPreview {
                placement_id: 1,
                canvas_id: 1,
                row: 242,
                column: 1,
            },
        );

        assert_eq!(result[0].grid_row, 242);
    }

    #[test]
    fn form_reflow_repairs_preexisting_overlaps() {
        let fields = vec![field(1, 1, 1, 1, 6, 1), field(2, 1, 1, 1, 6, 1)];
        let result = form_builder_reflow_section_fields(
            &fields,
            FormBuilderDragPreview {
                placement_id: 2,
                canvas_id: 1,
                row: 1,
                column: 1,
            },
        );

        let moved = result.iter().find(|field| field.id == 2).expect("moved");
        let repaired = result.iter().find(|field| field.id == 1).expect("repaired");
        assert_eq!((moved.grid_row, moved.grid_column), (1, 1));
        assert_eq!((repaired.grid_row, repaired.grid_column), (1, 7));
    }
}
