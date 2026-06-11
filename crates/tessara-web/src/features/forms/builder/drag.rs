//! Owns the features::forms::builder::drag module behavior.

#[cfg(feature = "hydrate")]
use crate::features::forms::builder::FORM_BUILDER_COLUMN_COUNT;
use crate::features::forms::builder::FormBuilderFieldDraft;
use crate::features::forms::builder::{FormBuilderDragPreview, form_builder_reflow_section_fields};
use leptos::prelude::*;

use wasm_bindgen::JsCast;

#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;
/// Handles the set form builder drag preview behavior.
pub(crate) fn set_form_builder_drag_preview(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    next_preview: FormBuilderDragPreview,
) {
    if builder_drag_preview.get_untracked() != Some(next_preview) {
        builder_drag_preview.set(Some(next_preview));
    }
}

/// Handles the form builder grid cell from drag event behavior.
pub(crate) fn form_builder_grid_cell_from_drag_event(
    event: &leptos::ev::DragEvent,
) -> Option<(i32, i32, String)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let cell = target.closest(".form-builder-grid-cell").ok().flatten()?;
    let row = cell.get_attribute("data-row")?.parse::<i32>().ok()?;
    let column = cell.get_attribute("data-column")?.parse::<i32>().ok()?;
    Some((row, column, cell.id()))
}

#[cfg(not(feature = "hydrate"))]
#[cfg(feature = "hydrate")]
/// Handles the form builder grid cell from drag event behavior.
pub(crate) fn form_builder_grid_cell_from_drag_event(
    _event: &leptos::ev::DragEvent,
) -> Option<(i32, i32, String)> {
    None
}

#[cfg(feature = "hydrate")]
/// Handles the form builder grid cell from pointer behavior.
pub(crate) fn form_builder_grid_cell_from_pointer(
    event: &leptos::ev::DragEvent,
    row_count: i32,
) -> Option<(i32, i32, String)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let grid = target.closest(".form-builder-layout-grid").ok().flatten()?;
    let grid_id = grid.get_attribute("data-section-id")?;
    let bounds_fn = js_sys::Reflect::get(&grid, &"getBoundingClientRect".into())
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let bounds = bounds_fn.call0(&grid).ok()?;
    let left = js_sys::Reflect::get(&bounds, &"left".into())
        .ok()?
        .as_f64()?;
    let top = js_sys::Reflect::get(&bounds, &"top".into())
        .ok()?
        .as_f64()?;
    let width = js_sys::Reflect::get(&bounds, &"width".into())
        .ok()?
        .as_f64()?;
    let height = js_sys::Reflect::get(&bounds, &"height".into())
        .ok()?
        .as_f64()?;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let row_count = row_count.max(1);
    let x = (f64::from(event.client_x()) - left).clamp(0.0, width - 1.0);
    let y = (f64::from(event.client_y()) - top).clamp(0.0, height - 1.0);
    let column_width = width / f64::from(FORM_BUILDER_COLUMN_COUNT);
    let row_height = height / f64::from(row_count);
    let column = ((x / column_width).floor() as i32 + 1).clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let row = ((y / row_height).floor() as i32 + 1).clamp(1, row_count);

    Some((
        row,
        column,
        format!("form-builder-section-{grid_id}-cell-r{row}-c{column}"),
    ))
}

#[cfg(not(feature = "hydrate"))]
/// Handles the form builder grid cell from pointer behavior.
pub(crate) fn form_builder_grid_cell_from_pointer(
    _event: &leptos::ev::DragEvent,
    _row_count: i32,
) -> Option<(i32, i32, String)> {
    None
}

#[cfg(feature = "hydrate")]
/// Handles the form builder add tile from click event behavior.
pub(crate) fn form_builder_add_tile_from_click_event(
    event: &leptos::ev::MouseEvent,
) -> Option<(i32, i32)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let add_cell = target
        .closest(".form-builder-grid-cell[data-empty]")
        .ok()
        .flatten()?;
    let row = add_cell.get_attribute("data-row")?.parse::<i32>().ok()?;
    let column = add_cell.get_attribute("data-column")?.parse::<i32>().ok()?;
    Some((row, column))
}

#[cfg(not(feature = "hydrate"))]
/// Handles the form builder add tile from click event behavior.
pub(crate) fn form_builder_add_tile_from_click_event(
    _event: &leptos::ev::MouseEvent,
) -> Option<(i32, i32)> {
    None
}

#[cfg(feature = "hydrate")]
/// Handles the clear form builder drag target dom behavior.
pub(crate) fn clear_form_builder_drag_target_dom() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(targets) = document.query_selector_all(".form-builder-grid-cell.is-drop-target") else {
        return;
    };

    for index in 0..targets.length() {
        if let Some(target) = targets.item(index) {
            if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                let _ = element.class_list().remove_1("is-drop-target");
            }
        }
    }
}

#[cfg(feature = "hydrate")]
/// Handles the set form builder drag target dom behavior.
pub(crate) fn set_form_builder_drag_target_dom(target_id: &str) {
    clear_form_builder_drag_target_dom();

    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(target_id))
    {
        let _ = element.class_list().add_1("is-drop-target");
    }
}

/// Handles the clear form builder drag intent behavior.
pub(crate) fn clear_form_builder_drag_intent(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
) {
    pending_builder_drag_preview.set(None);
    builder_drag_preview.set(None);

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (
            web_sys::window(),
            builder_drag_preview_timeout.get_untracked(),
        ) {
            window.clear_timeout_with_handle(timeout_handle);
        }
        clear_form_builder_drag_target_dom();
    }

    builder_drag_preview_timeout.set(None);
}

/// Handles the schedule form builder drag preview behavior.
pub(crate) fn schedule_form_builder_drag_preview(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    _builder_drag_preview_timeout: RwSignal<Option<i32>>,
    next_preview: FormBuilderDragPreview,
    target_id: String,
) {
    if builder_drag_preview.get_untracked() == Some(next_preview) {
        return;
    }

    pending_builder_drag_preview.set(Some(next_preview));

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (
            web_sys::window(),
            _builder_drag_preview_timeout.get_untracked(),
        ) {
            window.clear_timeout_with_handle(timeout_handle);
        }

        let pending_preview = pending_builder_drag_preview;
        let preview_signal = builder_drag_preview;
        let timeout_signal = _builder_drag_preview_timeout;
        let callback = Closure::wrap(Box::new(move || {
            if pending_preview.get_untracked() == Some(next_preview) {
                set_form_builder_drag_preview(preview_signal, next_preview);
                set_form_builder_drag_target_dom(&target_id);
            }
            timeout_signal.set(None);
        }) as Box<dyn FnMut()>);

        if let Some(window) = web_sys::window() {
            if let Ok(timeout_handle) = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    1_000,
                )
            {
                _builder_drag_preview_timeout.set(Some(timeout_handle));
                callback.forget();
                return;
            }
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        set_form_builder_drag_preview(builder_drag_preview, next_preview);
        let _ = target_id;
    }
}

/// Handles the commit form builder drag preview behavior.
pub(crate) fn commit_form_builder_drag_preview(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) {
    let preview = builder_drag_preview.get_untracked();

    if let Some(preview) = preview {
        builder_fields.update(|fields| {
            *fields = form_builder_reflow_section_fields(fields, preview);
        });
        suppress_builder_field_click.set(Some(preview.field_id));
    }

    clear_form_builder_drag_intent(
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
    );
    dragged_builder_field.set(None);
}
