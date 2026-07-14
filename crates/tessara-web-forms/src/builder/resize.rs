//! Browser resize handling for form builder grid tiles.

use crate::builder::{FormBuilderFieldDraft, FormBuilderResizeAxis};
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::layout::{FORM_BUILDER_GRID_CONSTRAINTS, form_builder_field_has_collision};
#[cfg(feature = "hydrate")]
use super::sizing::set_form_builder_field_size;
#[cfg(feature = "hydrate")]
use std::{cell::Cell, cell::RefCell, rc::Rc};
#[cfg(feature = "hydrate")]
use tessara_core::{GridRect, GridSize, resolve_resize_request};
#[cfg(feature = "hydrate")]
use tessara_web_ui::placement_editor::{
    PlacementResizeSession, placement_grid_metrics_from_element, placement_tile_style,
};
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

#[cfg(feature = "hydrate")]
type MouseEventCallback = Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>>>;

#[cfg(feature = "hydrate")]
struct ActiveFormBuilderResize {
    window: web_sys::Window,
    tile: web_sys::Element,
    active: Rc<Cell<bool>>,
    move_callback: MouseEventCallback,
    up_callback: MouseEventCallback,
}

#[cfg(feature = "hydrate")]
thread_local! {
    static ACTIVE_FORM_BUILDER_RESIZE: RefCell<Option<ActiveFormBuilderResize>> =
        const { RefCell::new(None) };
}

/// Binds global resize listeners to the current form-builder owner.
#[cfg(feature = "hydrate")]
pub(crate) fn install_form_builder_resize_cleanup() {
    on_cleanup(clear_active_form_builder_resize);
}

#[cfg(not(feature = "hydrate"))]
pub(crate) fn install_form_builder_resize_cleanup() {}

#[cfg(feature = "hydrate")]
fn clear_active_form_builder_resize() {
    ACTIVE_FORM_BUILDER_RESIZE.with(|active_resize| {
        let Some(active_resize) = active_resize.borrow_mut().take() else {
            return;
        };

        active_resize.active.set(false);
        let _ = active_resize.tile.class_list().remove_1("is-resizing");
        if let Some(callback) = active_resize.move_callback.borrow().as_ref() {
            let _ = active_resize.window.remove_event_listener_with_callback(
                "mousemove",
                callback.as_ref().unchecked_ref(),
            );
        }
        if let Some(callback) = active_resize.up_callback.borrow().as_ref() {
            let _ = active_resize
                .window
                .remove_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
        }
        active_resize.move_callback.borrow_mut().take();
        active_resize.up_callback.borrow_mut().take();
    });
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
    let Some(tile) = target.closest(".placement-editor-tile").ok().flatten() else {
        return;
    };
    let Some(grid) = target
        .closest("[data-placement-grid-canvas]")
        .ok()
        .flatten()
    else {
        return;
    };
    let Some(start_field) = builder_fields
        .get_untracked()
        .into_iter()
        .find(|field| field.id == field_id)
    else {
        return;
    };

    let Some(metrics) = placement_grid_metrics_from_element(&grid) else {
        return;
    };
    let start_rect = GridRect::new(
        start_field.grid_row.max(1),
        start_field.grid_column.max(1),
        start_field.grid_width.max(1),
        start_field.grid_height.max(1),
    );

    clear_active_form_builder_resize();
    suppress_builder_field_click.set(Some(field_id));
    let _ = tile.class_list().add_1("is-resizing");

    let active = Rc::new(Cell::new(true));
    let last_valid_width = Rc::new(Cell::new(start_field.grid_width.max(1)));
    let last_valid_height = Rc::new(Cell::new(start_field.grid_height.max(1)));
    let start_x = event.client_x();
    let start_y = event.client_y();
    let resize_session =
        PlacementResizeSession::new(start_rect, axis, f64::from(start_x), f64::from(start_y));

    let move_callback: MouseEventCallback = Rc::new(RefCell::new(None));
    let up_callback: MouseEventCallback = Rc::new(RefCell::new(None));

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

        let request = resize_session.request_at(
            metrics,
            f64::from(event.client_x()),
            f64::from(event.client_y()),
        );
        let Ok(size) = resolve_resize_request(
            FORM_BUILDER_GRID_CONSTRAINTS,
            start_rect,
            request,
            GridSize::ONE,
        ) else {
            return;
        };
        let mut candidate = start_field_for_move.clone();
        candidate.grid_width = size.width;
        candidate.grid_height = size.height;

        let fields = builder_fields_for_move.get_untracked();
        if form_builder_field_has_collision(&candidate, &fields) {
            return;
        }

        last_width_for_move.set(candidate.grid_width.max(1));
        last_height_for_move.set(candidate.grid_height.max(1));
        let _ = tile_for_move.set_attribute(
            "style",
            &placement_tile_style(GridRect::new(
                candidate.grid_row,
                candidate.grid_column,
                candidate.grid_width,
                candidate.grid_height,
            )),
        );
    }) as Box<dyn FnMut(_)>));

    let active_for_up = active.clone();
    let last_width_for_up = last_valid_width.clone();
    let last_height_for_up = last_valid_height.clone();
    *up_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_up.replace(false) {
            return;
        }
        event.prevent_default();
        builder_fields.update(|fields| {
            set_form_builder_field_size(
                fields,
                field_id,
                last_width_for_up.get(),
                last_height_for_up.get(),
            );
        });

        clear_active_form_builder_resize();
    }) as Box<dyn FnMut(_)>));

    if let Some(callback) = move_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref());
    }
    if let Some(callback) = up_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
    }
    ACTIVE_FORM_BUILDER_RESIZE.with(|active_resize| {
        *active_resize.borrow_mut() = Some(ActiveFormBuilderResize {
            window,
            tile,
            active,
            move_callback,
            up_callback,
        });
    });
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
