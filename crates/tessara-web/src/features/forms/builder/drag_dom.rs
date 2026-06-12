//! DOM helpers for form builder drag interactions.

#[cfg(feature = "hydrate")]
use crate::features::forms::builder::FORM_BUILDER_COLUMN_COUNT;
use wasm_bindgen::JsCast;

pub(crate) fn form_builder_grid_cell_from_drag_event(
    event: &leptos::ev::DragEvent,
) -> Option<(i32, i32, String)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let cell = target.closest(".form-builder-grid-cell").ok().flatten()?;
    let row = cell.get_attribute("data-row")?.parse::<i32>().ok()?;
    let column = cell.get_attribute("data-column")?.parse::<i32>().ok()?;
    Some((row, column, cell.id()))
}

#[cfg(feature = "hydrate")]
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
pub(crate) fn form_builder_grid_cell_from_pointer(
    _event: &leptos::ev::DragEvent,
    _row_count: i32,
) -> Option<(i32, i32, String)> {
    None
}

#[cfg(feature = "hydrate")]
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
pub(crate) fn form_builder_add_tile_from_click_event(
    _event: &leptos::ev::MouseEvent,
) -> Option<(i32, i32)> {
    None
}

#[cfg(feature = "hydrate")]
pub(super) fn clear_form_builder_drag_target_dom() {
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
pub(super) fn set_form_builder_drag_target_dom(target_id: &str) {
    clear_form_builder_drag_target_dom();

    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(target_id))
    {
        let _ = element.class_list().add_1("is-drop-target");
    }
}
