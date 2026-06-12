//! Signal-aware actions for form builder grid interactions.

use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderFieldDraft, blank_form_builder_field_at,
    form_builder_add_tile_from_click_event, form_builder_occupancy_map,
    form_builder_section_fields, max_form_builder_new_field_width_at,
};
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
pub(in crate::features::forms::builder::components) fn add_form_builder_field_from_grid_click(
    event: leptos::ev::MouseEvent,
    section_id: usize,
    default_column_width: Memo<i32>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) {
    let Some((row, column)) = form_builder_add_tile_from_click_event(&event) else {
        return;
    };
    event.prevent_default();
    if suppress_builder_field_click.get_untracked().is_some() {
        suppress_builder_field_click.set(None);
        return;
    }
    let fields = builder_fields.get_untracked();
    let occupied_cells = {
        let section_fields = form_builder_section_fields(section_id, &fields);
        form_builder_occupancy_map(&section_fields)
    };
    if occupied_cells.contains(&(row, column)) {
        return;
    }
    let field_id = next_builder_field_id.get_untracked();
    next_builder_field_id.set(field_id + 1);
    let default_width = default_column_width
        .get_untracked()
        .clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let available_width = max_form_builder_new_field_width_at(section_id, row, column, &fields);
    let new_field = blank_form_builder_field_at(
        field_id,
        section_id,
        row,
        column,
        default_width.min(available_width),
    );
    builder_fields.update(|fields| fields.push(new_field));
    active_builder_field.set(Some(field_id));
}
