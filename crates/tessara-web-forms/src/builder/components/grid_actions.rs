//! Signal-aware actions for form builder grid interactions.

use crate::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, blank_form_builder_field_at,
    form_builder_occupancy_map, form_builder_section_fields, max_form_builder_new_field_width_at,
};
use leptos::prelude::*;
use tessara_web_ui::placement_editor::placement_grid_cell_from_click_event;

#[allow(clippy::too_many_arguments)]
pub(crate) fn add_form_builder_field_from_grid_click(
    event: leptos::ev::MouseEvent,
    section_id: usize,
    default_column_width: Memo<i32>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) {
    let Some(cell) = placement_grid_cell_from_click_event(&event) else {
        return;
    };
    let row = cell.row;
    let column = cell.column;
    event.prevent_default();
    if suppress_builder_field_click.get_untracked().is_some() {
        suppress_builder_field_click.set(None);
        return;
    }
    let fields = builder_fields.get_untracked();
    let field_id = next_builder_field_id.get_untracked();
    let Some(new_field) = form_builder_field_for_grid_add(
        section_id,
        row,
        column,
        default_column_width.get_untracked(),
        field_id,
        &fields,
    ) else {
        return;
    };
    next_builder_field_id.set(field_id + 1);
    builder_fields.update(|fields| fields.push(new_field));
    active_builder_field.set(Some(field_id));
}

fn form_builder_field_for_grid_add(
    section_id: usize,
    row: i32,
    column: i32,
    default_column_width: i32,
    field_id: usize,
    fields: &[FormBuilderFieldDraft],
) -> Option<FormBuilderFieldDraft> {
    let occupied_cells = {
        let section_fields = form_builder_section_fields(section_id, fields);
        form_builder_occupancy_map(&section_fields)
    };
    if occupied_cells.contains(&(row, column)) {
        return None;
    }
    let default_width = default_column_width.clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let available_width = max_form_builder_new_field_width_at(section_id, row, column, fields);
    Some(blank_form_builder_field_at(
        field_id,
        section_id,
        row,
        column,
        default_width.min(available_width),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn field(id: usize, row: i32, column: i32, width: i32) -> FormBuilderFieldDraft {
        blank_form_builder_field_at(id, 1, row, column, width)
    }

    #[test]
    fn grid_add_preserves_default_configuration_and_available_width() {
        let fields = vec![field(1, 1, 7, 6)];
        let added = form_builder_field_for_grid_add(1, 1, 1, 8, 2, &fields)
            .expect("empty cell accepts a field");

        assert_eq!(added.id, 2);
        assert_eq!(added.section_id, 1);
        assert_eq!((added.grid_row, added.grid_column), (1, 1));
        assert_eq!((added.grid_width, added.grid_height), (6, 1));
        assert_eq!(added.field_type, "text");
        assert!(added.label.is_empty());
        assert!(added.key.is_empty());
    }

    #[test]
    fn grid_add_still_rejects_an_occupied_cell() {
        let fields = vec![field(1, 1, 1, 6)];
        assert!(form_builder_field_for_grid_add(1, 1, 3, 6, 2, &fields).is_none());
    }
}
