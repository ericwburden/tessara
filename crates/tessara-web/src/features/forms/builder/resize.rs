//! Browser resize handling for form builder grid tiles.

#[cfg(feature = "hydrate")]
use crate::features::forms::builder::FORM_BUILDER_COLUMN_COUNT;
use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderResizeAxis};
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::layout::form_builder_field_has_collision;
#[cfg(feature = "hydrate")]
use super::sizing::set_form_builder_field_size;
#[cfg(feature = "hydrate")]
use std::{cell::Cell, cell::RefCell, rc::Rc};
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

#[cfg(feature = "hydrate")]
fn form_builder_grid_tile_style(field: &FormBuilderFieldDraft) -> String {
    let column = field.grid_column.max(1);
    let row = field.grid_row.max(1);
    let width = field.grid_width.max(1);
    let height = field.grid_height.max(1);

    format!("grid-column: {column} / span {width}; grid-row: {row} / span {height};")
}

/// Starts pointer-driven resizing for a form builder field tile.
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

/// No-op resize hook for server-side builds.
#[cfg(not(feature = "hydrate"))]
pub(crate) fn start_form_builder_field_resize(
    _event: leptos::ev::MouseEvent,
    _axis: FormBuilderResizeAxis,
    _field_id: usize,
    _builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    _suppress_builder_field_click: RwSignal<Option<usize>>,
) {
}
