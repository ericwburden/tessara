use crate::features::form_builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderResizeAxis,
};
use crate::features::workflows::submission::FormBuilderFieldDraft;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use std::{cell::Cell, cell::RefCell, rc::Rc};
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

fn form_builder_fields_overlap(
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

fn form_builder_linear_grid_index(field: &FormBuilderFieldDraft, column_count: i32) -> i32 {
    let column_count = column_count.max(1);
    (field.grid_row.max(1) - 1) * column_count + field.grid_column.max(1) - 1
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

#[cfg(feature = "hydrate")]
fn form_builder_grid_tile_style(field: &FormBuilderFieldDraft) -> String {
    let column = field.grid_column.max(1);
    let row = field.grid_row.max(1);
    let width = field.grid_width.max(1);
    let height = field.grid_height.max(1);

    format!("grid-column: {column} / span {width}; grid-row: {row} / span {height};")
}

#[cfg(feature = "hydrate")]
pub(crate) fn start_form_builder_field_resize(
    event: leptos::ev::MouseEvent,
    axis: FormBuilderResizeAxis,
    field_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) {
    event.prevent_default();
    event.stop_propagation();

    let Some(window) = web_sys::window() else {
        return;
    };
    if window
        .match_media("(max-width: 767px)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches())
    {
        return;
    }

    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Some(tile) = target.closest(".form-builder-grid-tile").ok().flatten() else {
        return;
    };
    let Some(grid) = target.closest(".form-builder-layout-grid").ok().flatten() else {
        return;
    };
    let Some(start_field) = builder_fields
        .get_untracked()
        .into_iter()
        .find(|field| field.id == field_id)
    else {
        return;
    };

    let Some(grid_element) = grid.dyn_ref::<web_sys::HtmlElement>() else {
        return;
    };
    let cell_width = f64::from(grid_element.client_width()) / f64::from(FORM_BUILDER_COLUMN_COUNT);
    let row_height = 80.0;
    if cell_width <= 0.0 {
        return;
    }

    suppress_builder_field_click.set(Some(field_id));
    let _ = tile.class_list().add_1("is-resizing");

    let active = Rc::new(Cell::new(true));
    let last_valid_width = Rc::new(Cell::new(start_field.grid_width.max(1)));
    let last_valid_height = Rc::new(Cell::new(start_field.grid_height.max(1)));
    let start_x = event.client_x();
    let start_y = event.client_y();

    let move_callback: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>>> =
        Rc::new(RefCell::new(None));
    let up_callback: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>>> =
        Rc::new(RefCell::new(None));

    let active_for_move = active.clone();
    let tile_for_move = tile.clone();
    let last_width_for_move = last_valid_width.clone();
    let last_height_for_move = last_valid_height.clone();
    let builder_fields_for_move = builder_fields;
    let start_field_for_move = start_field.clone();
    *move_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_move.get() {
            return;
        }
        event.prevent_default();

        let mut candidate = start_field_for_move.clone();
        match axis {
            FormBuilderResizeAxis::Width => {
                let width_delta =
                    (f64::from(event.client_x() - start_x) / cell_width).round() as i32;
                candidate.grid_width = (start_field_for_move.grid_width + width_delta)
                    .clamp(1, FORM_BUILDER_COLUMN_COUNT);
            }
            FormBuilderResizeAxis::Height => {
                let height_delta =
                    (f64::from(event.client_y() - start_y) / row_height).round() as i32;
                candidate.grid_height =
                    (start_field_for_move.grid_height + height_delta).clamp(1, 6);
            }
        }

        let column_end = candidate.grid_column.max(1) + candidate.grid_width.max(1) - 1;
        if column_end > FORM_BUILDER_COLUMN_COUNT {
            return;
        }

        let fields = builder_fields_for_move.get_untracked();
        if form_builder_field_has_collision(&candidate, &fields) {
            return;
        }

        last_width_for_move.set(candidate.grid_width.max(1));
        last_height_for_move.set(candidate.grid_height.max(1));
        let _ = tile_for_move.set_attribute("style", &form_builder_grid_tile_style(&candidate));
    }) as Box<dyn FnMut(_)>));

    let active_for_up = active.clone();
    let tile_for_up = tile.clone();
    let last_width_for_up = last_valid_width.clone();
    let last_height_for_up = last_valid_height.clone();
    let move_callback_for_up = move_callback.clone();
    let up_callback_for_up = up_callback.clone();
    *up_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_up.replace(false) {
            return;
        }
        event.prevent_default();
        let _ = tile_for_up.class_list().remove_1("is-resizing");
        builder_fields.update(|fields| {
            set_form_builder_field_size(
                fields,
                field_id,
                last_width_for_up.get(),
                last_height_for_up.get(),
            );
        });

        if let Some(window) = web_sys::window() {
            if let Some(callback) = move_callback_for_up.borrow().as_ref() {
                let _ = window.remove_event_listener_with_callback(
                    "mousemove",
                    callback.as_ref().unchecked_ref(),
                );
            }
            if let Some(callback) = up_callback_for_up.borrow().as_ref() {
                let _ = window.remove_event_listener_with_callback(
                    "mouseup",
                    callback.as_ref().unchecked_ref(),
                );
            }
        }
        move_callback_for_up.borrow_mut().take();
        up_callback_for_up.borrow_mut().take();
    }) as Box<dyn FnMut(_)>));

    if let Some(callback) = move_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref());
    }
    if let Some(callback) = up_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
    }
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn start_form_builder_field_resize(
    _event: leptos::ev::MouseEvent,
    _axis: FormBuilderResizeAxis,
    _field_id: usize,
    _builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    _suppress_builder_field_click: RwSignal<Option<usize>>,
) {
}
