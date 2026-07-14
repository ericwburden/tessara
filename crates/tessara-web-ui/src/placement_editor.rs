//! Reusable, domain-neutral placement-editor interactions.
//!
//! This module owns Leptos/DOM mechanics only. Bounds, collision, reflow, and
//! final resize validation remain in `tessara-core::grid_layout`.

use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    rc::Rc,
};
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

#[cfg(test)]
use tessara_core::{GridConstraints, resolve_move_request, resolve_resize_request};
pub use tessara_core::{
    GridMoveDirection, GridMoveRequest, GridRect, GridResizeAxis, GridResizeRequest,
    GridResizeStep, GridSize,
};

/// Tessara placement canvases always use twelve columns.
pub const PLACEMENT_GRID_COLUMN_COUNT: i32 = 12;
/// Minimum desktop row-track height.
pub const PLACEMENT_GRID_MIN_TRACK_PX: f64 = 48.0;
/// Maximum desktop row-track height.
pub const PLACEMENT_GRID_MAX_TRACK_PX: f64 = 80.0;
/// Delay before a hovered drag target becomes the visible preview.
pub const PLACEMENT_DRAG_PREVIEW_DELAY_MS: i32 = 1_000;

#[cfg(feature = "hydrate")]
thread_local! {
    static PLACEMENT_DRAG_TIMEOUT_CALLBACKS: RefCell<BTreeMap<i32, Closure<dyn FnMut()>>> =
        const { RefCell::new(BTreeMap::new()) };
}

/// A one-based cell in a placement canvas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementGridCell {
    pub row: i32,
    pub column: i32,
}

/// A resolved canvas target and the guide-cell element that represents it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PlacementGridTarget {
    pub cell: PlacementGridCell,
    pub target_id: String,
}

/// Actual rendered canvas bounds used by pointer calculations.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacementGridMetrics {
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub column_track: f64,
    pub row_track: f64,
    pub row_count: i32,
}

impl PlacementGridMetrics {
    /// Builds metrics from the browser's actual rendered rectangle.
    pub fn from_rendered_bounds(
        left: f64,
        top: f64,
        width: f64,
        height: f64,
        row_count: i32,
    ) -> Option<Self> {
        let row_count = row_count.max(1);
        if !left.is_finite()
            || !top.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
        {
            return None;
        }

        Some(Self {
            left,
            top,
            width,
            height,
            column_track: width / f64::from(PLACEMENT_GRID_COLUMN_COUNT),
            row_track: height / f64::from(row_count),
            row_count,
        })
    }

    /// Builds the ideal desktop metrics before a canvas is rendered.
    pub fn squareish(left: f64, top: f64, width: f64, row_count: i32) -> Option<Self> {
        if !width.is_finite() || width <= 0.0 {
            return None;
        }
        let row_count = row_count.max(1);
        let row_track = placement_grid_track_px(width);
        Self::from_rendered_bounds(
            left,
            top,
            width,
            row_track * f64::from(row_count),
            row_count,
        )
    }
}

/// Generic delayed drag preview state shared by feature adapters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlacementDragPreview<PlacementId, CanvasId> {
    pub placement_id: PlacementId,
    pub canvas_id: CanvasId,
    pub row: i32,
    pub column: i32,
}

/// Pointer resize session; feature adapters decide whether the request is valid.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacementResizeSession {
    pub start_rect: GridRect,
    pub axis: GridResizeAxis,
    pub start_x: f64,
    pub start_y: f64,
}

impl PlacementResizeSession {
    pub const fn new(
        start_rect: GridRect,
        axis: GridResizeAxis,
        start_x: f64,
        start_y: f64,
    ) -> Self {
        Self {
            start_rect,
            axis,
            start_x,
            start_y,
        }
    }

    /// Converts the current pointer location into the same direct resize
    /// request used by an inspector input.
    pub fn request_at(self, metrics: PlacementGridMetrics, x: f64, y: f64) -> GridResizeRequest {
        let width_delta = ((x - self.start_x) / metrics.column_track).round() as i32;
        let height_delta = ((y - self.start_y) / metrics.row_track).round() as i32;
        let size = match self.axis {
            GridResizeAxis::Width => GridSize::new(
                self.start_rect.width.saturating_add(width_delta),
                self.start_rect.height,
            ),
            GridResizeAxis::Height => GridSize::new(
                self.start_rect.width,
                self.start_rect.height.saturating_add(height_delta),
            ),
        };
        GridResizeRequest::Direct(size)
    }
}

/// Event delivered when one of the pointer resize handles is pressed.
#[derive(Clone)]
pub struct PlacementResizeStart {
    pub axis: GridResizeAxis,
    pub event: leptos::ev::MouseEvent,
}

/// Shared selected-placement signal used by a tile and an inspector.
pub struct PlacementSelection<Id> {
    selected: RwSignal<Option<Id>>,
}

impl<Id> Clone for PlacementSelection<Id> {
    fn clone(&self) -> Self {
        Self {
            selected: self.selected,
        }
    }
}

impl<Id: Clone + PartialEq + Send + Sync + 'static> PlacementSelection<Id> {
    pub fn new(initial: Option<Id>) -> Self {
        Self {
            selected: RwSignal::new(initial),
        }
    }

    pub const fn signal(&self) -> RwSignal<Option<Id>> {
        self.selected
    }

    pub fn select(&self, id: Id) {
        self.selected.set(Some(id));
    }

    pub fn clear(&self) {
        self.selected.set(None);
    }

    pub fn is_selected(&self, id: &Id) -> bool {
        self.selected
            .get()
            .as_ref()
            .is_some_and(|selected| selected == id)
    }
}

/// Returns the responsive square-ish desktop row track for a rendered width.
pub fn placement_grid_track_px(rendered_width: f64) -> f64 {
    (rendered_width / f64::from(PLACEMENT_GRID_COLUMN_COUNT))
        .clamp(PLACEMENT_GRID_MIN_TRACK_PX, PLACEMENT_GRID_MAX_TRACK_PX)
}

/// Maps a pointer to a one-based cell using actual rendered DOM metrics.
pub fn placement_grid_cell_from_pointer(
    metrics: PlacementGridMetrics,
    client_x: f64,
    client_y: f64,
) -> PlacementGridCell {
    let x = (client_x - metrics.left).clamp(0.0, metrics.width - f64::EPSILON);
    let y = (client_y - metrics.top).clamp(0.0, metrics.height - f64::EPSILON);
    PlacementGridCell {
        row: ((y / metrics.row_track).floor() as i32 + 1).clamp(1, metrics.row_count),
        column: ((x / metrics.column_track).floor() as i32 + 1)
            .clamp(1, PLACEMENT_GRID_COLUMN_COUNT),
    }
}

/// Converts a pointer target into the same move request used by direct inputs.
pub fn placement_move_request_from_pointer(
    metrics: PlacementGridMetrics,
    client_x: f64,
    client_y: f64,
) -> GridMoveRequest {
    let cell = placement_grid_cell_from_pointer(metrics, client_x, client_y);
    GridMoveRequest::Direct {
        row: cell.row,
        column: cell.column,
    }
}

/// Creates a direct row/column edit request.
pub const fn placement_move_request_from_direct(row: i32, column: i32) -> GridMoveRequest {
    GridMoveRequest::Direct { row, column }
}

/// Creates a keyboard movement request.
pub const fn placement_move_request_from_keyboard(direction: GridMoveDirection) -> GridMoveRequest {
    GridMoveRequest::Keyboard(direction)
}

/// Creates a direct width/height edit request.
pub const fn placement_resize_request_from_direct(width: i32, height: i32) -> GridResizeRequest {
    GridResizeRequest::Direct(GridSize::new(width, height))
}

/// Creates a keyboard sizing request.
pub const fn placement_resize_request_from_keyboard(
    axis: GridResizeAxis,
    step: GridResizeStep,
) -> GridResizeRequest {
    GridResizeRequest::Keyboard { axis, step }
}

/// Returns the deterministic guide-cell id for a canvas.
pub fn placement_grid_cell_id(canvas_id: &str, cell: PlacementGridCell) -> String {
    format!("{canvas_id}-cell-r{}-c{}", cell.row, cell.column)
}

/// CSS variables for a shared placement canvas.
pub fn placement_grid_canvas_style(row_count: i32) -> String {
    format!("--placement-grid-rows: {};", row_count.max(1))
}

/// CSS placement for a shared tile shell.
pub fn placement_tile_style(rect: GridRect) -> String {
    format!(
        "grid-column: {} / span {}; grid-row: {} / span {};",
        rect.column.max(1),
        rect.width.max(1),
        rect.row.max(1),
        rect.height.max(1),
    )
}

/// A 12-column canvas that owns DOM pointer resolution and responsive tracks.
#[component]
pub fn PlacementGridCanvas(
    canvas_id: String,
    row_count: Signal<i32>,
    dragging: Signal<bool>,
    on_drag_target: Callback<PlacementGridTarget>,
    on_drop_target: Callback<PlacementGridTarget>,
    on_cancel_drag: Callback<()>,
    on_click: Callback<leptos::ev::MouseEvent>,
    children: Children,
    #[prop(default = "")] class: &'static str,
    #[prop(default = false)] stack_on_narrow: bool,
) -> impl IntoView {
    let class_name = format!("placement-editor-grid {class}");
    let canvas_id_for_attribute = canvas_id.clone();

    view! {
        <div
            class=class_name
            class:is-dragging=move || dragging.get()
            data-placement-grid-canvas=canvas_id_for_attribute
            data-placement-stack-on-narrow=stack_on_narrow.then_some("true")
            data-placement-grid-rows=move || row_count.get().max(1)
            style=move || placement_grid_canvas_style(row_count.get())
            on:dragenter=move |event| {
                if !dragging.get_untracked() {
                    return;
                }
                if let Some(target) = placement_grid_target_from_cell_event(&event) {
                    event.prevent_default();
                    on_drag_target.run(target);
                }
            }
            on:dragover=move |event| {
                if !dragging.get_untracked() {
                    return;
                }
                event.prevent_default();
                if let Some(target) = placement_grid_target_from_pointer_event(&event) {
                    on_drag_target.run(target);
                }
            }
            on:drop=move |event| {
                if !dragging.get_untracked() {
                    return;
                }
                event.prevent_default();
                if let Some(target) = placement_grid_target_from_pointer_event(&event) {
                    on_drop_target.run(target);
                }
            }
            on:mouseleave=move |_| {
                if dragging.get_untracked() {
                    on_cancel_drag.run(());
                }
            }
            on:click=move |event| on_click.run(event)
        >
            {children()}
        </div>
    }
}

/// Renders the reusable guide/drop-target layer for a placement canvas.
#[component]
pub fn PlacementGridGuides(
    canvas_id: String,
    cells: Signal<Vec<PlacementGridCell>>,
    cell_label: Callback<PlacementGridCell, String>,
    #[prop(default = "")] class: &'static str,
    #[prop(default = "")] cell_class: &'static str,
    #[prop(default = false)] empty: bool,
    /// Guide cells are visual pointer targets, not keyboard controls. Keep
    /// them out of the accessibility tree unless a caller supplies equivalent
    /// interactive semantics and opts in explicitly.
    #[prop(default = true)]
    aria_hidden: bool,
) -> impl IntoView {
    let guides_class = format!("placement-editor-grid__guides {class}");
    view! {
        <div class=guides_class>{move || {
            cells
                .get()
                .into_iter()
                .map(|cell| {
                    let id = placement_grid_cell_id(&canvas_id, cell);
                    let label = cell_label.run(cell);
                    let class_name = format!("placement-editor-grid__cell {cell_class}");
                    view! {
                        <div
                            id=id
                            class=class_name
                            data-placement-grid-cell="true"
                            data-row=cell.row
                            data-column=cell.column
                            data-empty=empty.then_some("true")
                            aria-label=label
                            aria-hidden=aria_hidden.then_some("true")
                            style=format!("grid-column: {}; grid-row: {};", cell.column, cell.row)
                        ></div>
                    }
                })
                .collect_view()
        }}</div>
    }
}

/// Domain-neutral symbolic tile wrapper with synchronized selection callbacks.
#[component]
pub fn PlacementTileShell(
    rect: Signal<Option<GridRect>>,
    class: Signal<String>,
    selected: Signal<bool>,
    dragging: Signal<bool>,
    #[prop(default = true)] draggable: bool,
    on_select: Callback<leptos::ev::MouseEvent>,
    on_drag_start: Callback<leptos::ev::DragEvent>,
    on_drag_enter: Callback<leptos::ev::DragEvent>,
    on_drag_end: Callback<leptos::ev::DragEvent>,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=move || format!("placement-editor-tile {}", class.get())
            class:is-selected=move || selected.get()
            class:is-dragging=move || dragging.get()
            data-placement-selected=move || selected.get().then_some("true")
            draggable=draggable.to_string()
            style=move || {
                rect.get()
                    .map(placement_tile_style)
                    .unwrap_or_else(|| "display: none;".into())
            }
            on:click=move |event| on_select.run(event)
            on:dragstart=move |event| on_drag_start.run(event)
            on:dragenter=move |event| on_drag_enter.run(event)
            on:dragend=move |event| on_drag_end.run(event)
        >
            {children()}
        </div>
    }
}

/// Shared pointer resize handles. Keyboard/direct size controls use the
/// request helpers above and remain feature-owned inspector UI.
#[component]
pub fn PlacementResizeHandles(
    on_resize_start: Callback<PlacementResizeStart>,
    #[prop(default = "Resize width")] width_title: &'static str,
    #[prop(default = "Resize height")] height_title: &'static str,
    #[prop(default = "")] handle_class: &'static str,
    #[prop(default = "")] width_class: &'static str,
    #[prop(default = "")] height_class: &'static str,
) -> impl IntoView {
    let width_classes = format!(
        "placement-editor-resize-handle placement-editor-resize-handle--width {handle_class} {width_class}"
    );
    let height_classes = format!(
        "placement-editor-resize-handle placement-editor-resize-handle--height {handle_class} {height_class}"
    );
    view! {
        <span
            class=width_classes
            title=width_title
            aria-hidden="true"
            on:mousedown=move |event| {
                on_resize_start.run(PlacementResizeStart {
                    axis: GridResizeAxis::Width,
                    event,
                });
            }
        ></span>
        <span
            class=height_classes
            title=height_title
            aria-hidden="true"
            on:mousedown=move |event| {
                on_resize_start.run(PlacementResizeStart {
                    axis: GridResizeAxis::Height,
                    event,
                });
            }
        ></span>
    }
}

/// Immediately replaces the active drag preview when the target changed.
pub fn set_placement_drag_preview<PlacementId, CanvasId>(
    preview: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    next: PlacementDragPreview<PlacementId, CanvasId>,
) where
    PlacementId: Clone + PartialEq + Send + Sync + 'static,
    CanvasId: Clone + PartialEq + Send + Sync + 'static,
{
    if preview.get_untracked() != Some(next.clone()) {
        preview.set(Some(next));
    }
}

/// Clears pending and active drag intent plus its browser timer/DOM marker.
pub fn clear_placement_drag_intent<PlacementId, CanvasId>(
    preview: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    pending: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    timeout: RwSignal<Option<i32>>,
) where
    PlacementId: Clone + Send + Sync + 'static,
    CanvasId: Clone + Send + Sync + 'static,
{
    pending.set(None);
    preview.set(None);

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (web_sys::window(), timeout.get_untracked()) {
            window.clear_timeout_with_handle(timeout_handle);
            drop_placement_drag_timeout_callback(timeout_handle);
        }
        clear_placement_drag_target_dom();
    }

    timeout.set(None);
}

/// Binds pending drag timers and DOM target markers to the current Leptos
/// owner. Feature editors should install this once at their composition root.
pub fn install_placement_drag_cleanup<PlacementId, CanvasId>(
    preview: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    pending: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    timeout: RwSignal<Option<i32>>,
) where
    PlacementId: Clone + Send + Sync + 'static,
    CanvasId: Clone + Send + Sync + 'static,
{
    on_cleanup(move || clear_placement_drag_intent(preview, pending, timeout));
}

/// Schedules the shared one-second delayed drag preview.
pub fn schedule_placement_drag_preview<PlacementId, CanvasId>(
    preview: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    pending: RwSignal<Option<PlacementDragPreview<PlacementId, CanvasId>>>,
    _timeout: RwSignal<Option<i32>>,
    next: PlacementDragPreview<PlacementId, CanvasId>,
    target_id: String,
) where
    PlacementId: Clone + PartialEq + Send + Sync + 'static,
    CanvasId: Clone + PartialEq + Send + Sync + 'static,
{
    if preview.get_untracked() == Some(next.clone()) {
        return;
    }
    if pending.get_untracked() == Some(next.clone()) {
        return;
    }
    pending.set(Some(next.clone()));

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (web_sys::window(), _timeout.get_untracked())
        {
            window.clear_timeout_with_handle(timeout_handle);
            drop_placement_drag_timeout_callback(timeout_handle);
        }

        let next_for_callback = next.clone();
        let timeout_handle_for_callback = Rc::new(Cell::new(None::<i32>));
        let captured_timeout_handle = timeout_handle_for_callback.clone();
        let callback = Closure::wrap(Box::new(move || {
            if pending.get_untracked() == Some(next_for_callback.clone()) {
                set_placement_drag_preview(preview, next_for_callback.clone());
                set_placement_drag_target_dom(&target_id);
            }
            _timeout.set(None);
            if let Some(handle) = captured_timeout_handle.get() {
                drop_placement_drag_timeout_callback(handle);
            }
        }) as Box<dyn FnMut()>);

        if let Some(window) = web_sys::window()
            && let Ok(handle) = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                PLACEMENT_DRAG_PREVIEW_DELAY_MS,
            )
        {
            timeout_handle_for_callback.set(Some(handle));
            _timeout.set(Some(handle));
            PLACEMENT_DRAG_TIMEOUT_CALLBACKS.with(|callbacks| {
                callbacks.borrow_mut().insert(handle, callback);
            });
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        set_placement_drag_preview(preview, next);
        let _ = target_id;
    }
}

#[cfg(feature = "hydrate")]
fn drop_placement_drag_timeout_callback(handle: i32) {
    PLACEMENT_DRAG_TIMEOUT_CALLBACKS.with(|callbacks| {
        callbacks.borrow_mut().remove(&handle);
    });
}

/// Resolves the exact guide cell under a drag event target.
pub fn placement_grid_target_from_cell_event(
    event: &leptos::ev::DragEvent,
) -> Option<PlacementGridTarget> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let cell = target
        .closest("[data-placement-grid-cell]")
        .ok()
        .flatten()?;
    let row = cell.get_attribute("data-row")?.parse::<i32>().ok()?;
    let column = cell.get_attribute("data-column")?.parse::<i32>().ok()?;
    Some(PlacementGridTarget {
        cell: PlacementGridCell { row, column },
        target_id: cell.id(),
    })
}

/// Resolves a drag pointer through actual rendered canvas dimensions.
pub fn placement_grid_target_from_pointer_event(
    event: &leptos::ev::DragEvent,
) -> Option<PlacementGridTarget> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let grid = target
        .closest("[data-placement-grid-canvas]")
        .ok()
        .flatten()?;
    let canvas_id = grid.get_attribute("data-placement-grid-canvas")?;
    let metrics = placement_grid_metrics_from_element(&grid)?;
    let cell = placement_grid_cell_from_pointer(
        metrics,
        f64::from(event.client_x()),
        f64::from(event.client_y()),
    );
    Some(PlacementGridTarget {
        target_id: placement_grid_cell_id(&canvas_id, cell),
        cell,
    })
}

/// Resolves an empty guide-cell click for feature-specific add behavior.
pub fn placement_grid_cell_from_click_event(
    event: &leptos::ev::MouseEvent,
) -> Option<PlacementGridCell> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let cell = target
        .closest("[data-placement-grid-cell][data-empty]")
        .ok()
        .flatten()?;
    Some(PlacementGridCell {
        row: cell.get_attribute("data-row")?.parse::<i32>().ok()?,
        column: cell.get_attribute("data-column")?.parse::<i32>().ok()?,
    })
}

/// Reads actual browser bounds and row count from a rendered canvas.
pub fn placement_grid_metrics_from_element(
    grid: &web_sys::Element,
) -> Option<PlacementGridMetrics> {
    let row_count = grid
        .get_attribute("data-placement-grid-rows")?
        .parse::<i32>()
        .ok()?
        .max(1);
    // The outer canvas may have borders and padding. The guide layer occupies
    // the actual CSS grid track box, so pointer math must use its rectangle.
    let metric_element = grid
        .query_selector(":scope > .placement-editor-grid__guides")
        .ok()
        .flatten()
        .unwrap_or_else(|| grid.clone());
    let bounds_fn = js_sys::Reflect::get(&metric_element, &"getBoundingClientRect".into())
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let bounds = bounds_fn.call0(&metric_element).ok()?;
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
    PlacementGridMetrics::from_rendered_bounds(left, top, width, height, row_count)
}

#[cfg(feature = "hydrate")]
fn clear_placement_drag_target_dom() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(targets) = document.query_selector_all(".placement-editor-grid__cell.is-drop-target")
    else {
        return;
    };
    for index in 0..targets.length() {
        if let Some(target) = targets.item(index)
            && let Ok(element) = target.dyn_into::<web_sys::Element>()
        {
            let _ = element.class_list().remove_1("is-drop-target");
        }
    }
}

#[cfg(feature = "hydrate")]
fn set_placement_drag_target_dom(target_id: &str) {
    clear_placement_drag_target_dom();
    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(target_id))
    {
        let _ = element.class_list().add_1("is-drop-target");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const GRID: GridConstraints = GridConstraints::new(12, 240, 240, 6);

    #[test]
    fn responsive_track_is_squareish_and_clamped() {
        assert_eq!(placement_grid_track_px(480.0), 48.0);
        assert_eq!(placement_grid_track_px(720.0), 60.0);
        assert_eq!(placement_grid_track_px(1_200.0), 80.0);
    }

    #[test]
    fn pointer_uses_actual_dom_tracks_and_clamps_edges() {
        let metrics = PlacementGridMetrics::from_rendered_bounds(100.0, 200.0, 720.0, 180.0, 3)
            .expect("metrics");
        assert_eq!(metrics.column_track, 60.0);
        assert_eq!(metrics.row_track, 60.0);
        assert_eq!(
            placement_grid_cell_from_pointer(metrics, 221.0, 321.0),
            PlacementGridCell { row: 3, column: 3 }
        );
        assert_eq!(
            placement_grid_cell_from_pointer(metrics, 10_000.0, -10_000.0),
            PlacementGridCell { row: 1, column: 12 }
        );
    }

    #[test]
    fn pointer_keyboard_and_direct_move_requests_agree() {
        let current = GridRect::new(2, 2, 3, 2);
        let metrics = PlacementGridMetrics::squareish(0.0, 0.0, 720.0, 4).expect("metrics");
        let pointer = placement_move_request_from_pointer(metrics, 121.0, 61.0);
        let direct = placement_move_request_from_direct(2, 3);
        let keyboard = placement_move_request_from_keyboard(GridMoveDirection::Right);

        assert_eq!(pointer, direct);
        assert_eq!(
            resolve_move_request(GRID, current, pointer).expect("pointer"),
            resolve_move_request(GRID, current, keyboard).expect("keyboard"),
        );
    }

    #[test]
    fn pointer_keyboard_and_direct_resize_requests_agree() {
        let current = GridRect::new(2, 2, 3, 2);
        let metrics = PlacementGridMetrics::squareish(0.0, 0.0, 720.0, 4).expect("metrics");
        let pointer = PlacementResizeSession::new(current, GridResizeAxis::Width, 100.0, 100.0)
            .request_at(metrics, 160.0, 100.0);
        let direct = placement_resize_request_from_direct(4, 2);
        let keyboard =
            placement_resize_request_from_keyboard(GridResizeAxis::Width, GridResizeStep::Increase);

        assert_eq!(pointer, direct);
        assert_eq!(
            resolve_resize_request(GRID, current, pointer, GridSize::ONE).expect("pointer"),
            resolve_resize_request(GRID, current, keyboard, GridSize::ONE).expect("keyboard"),
        );
    }

    #[cfg(feature = "ssr")]
    #[test]
    fn shared_canvas_tile_and_handles_render_stable_accessible_markup() {
        let html = Owner::new().with(|| {
            let row_count = Signal::derive(|| 2);
            let dragging = Signal::derive(|| false);
            let rect = Signal::derive(|| Some(GridRect::new(1, 1, 6, 2)));
            let class = Signal::derive(|| "fixture-tile".to_string());
            let selected = Signal::derive(|| true);
            let cells = Signal::derive(|| vec![PlacementGridCell { row: 1, column: 1 }]);
            view! {
                <PlacementGridCanvas
                    canvas_id="fixture".to_string()
                    row_count
                    dragging
                    on_drag_target=Callback::new(|_| {})
                    on_drop_target=Callback::new(|_| {})
                    on_cancel_drag=Callback::new(|_| {})
                    on_click=Callback::new(|_| {})
                >
                    <PlacementGridGuides
                        canvas_id="fixture".to_string()
                        cells
                        cell_label=Callback::new(|cell: PlacementGridCell| {
                            format!("Row {}, column {}", cell.row, cell.column)
                        })
                    />
                    <PlacementTileShell
                        rect
                        class
                        selected
                        dragging
                        draggable=false
                        on_select=Callback::new(|_| {})
                        on_drag_start=Callback::new(|_| {})
                        on_drag_enter=Callback::new(|_| {})
                        on_drag_end=Callback::new(|_| {})
                    >
                        <span>"Tile"</span>
                        <PlacementResizeHandles on_resize_start=Callback::new(|_| {})/>
                    </PlacementTileShell>
                </PlacementGridCanvas>
            }
            .to_html()
        });

        assert!(html.contains("placement-editor-grid"));
        assert!(html.contains("data-placement-grid-rows=\"2\""));
        assert!(html.contains("grid-column: 1 / span 6; grid-row: 1 / span 2;"));
        assert!(html.contains("aria-label=\"Row 1, column 1\""));
        assert!(html.contains("aria-hidden=\"true\""));
        assert!(!html.contains("placement-editor-grid__guides\"> <div"));
        assert!(html.contains("draggable=\"false\""));
        assert!(html.contains("title=\"Resize width\""));
    }
}
