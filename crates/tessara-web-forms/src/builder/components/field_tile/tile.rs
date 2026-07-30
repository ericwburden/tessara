//! Form builder grid tile component.

use super::resize_handles::FormBuilderFieldResizeHandles;
use leptos::prelude::*;

use crate::builder::FormBuilderFieldDraft;
use crate::builder::{
    FormBuilderDragPreview, clear_form_builder_drag_intent, form_builder_field_default_label,
    form_builder_field_type_icon, schedule_form_builder_drag_preview,
};
use tessara_core::GridRect;
use tessara_module_ui::placement_editor::{PlacementTileShell, placement_grid_cell_id};

#[component]
pub(crate) fn FormBuilderGridTile(
    field_id: usize,
    section_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) -> impl IntoView {
    let field = Memo::new(move |_| {
        builder_fields
            .get()
            .into_iter()
            .find(|field| field.id == field_id)
    });
    let display_label = move || {
        field
            .get()
            .map(|field| {
                if field.label.trim().is_empty() {
                    form_builder_field_default_label(&field.field_type, field_id)
                } else {
                    field.label
                }
            })
            .unwrap_or_else(|| format!("Field {field_id}"))
    };
    let rect = Signal::derive(move || {
        field.get().map(|field| {
            GridRect::new(
                field.grid_row.max(1),
                field.grid_column.max(1),
                field.grid_width.max(1),
                field.grid_height.max(1),
            )
        })
    });
    let class = Signal::derive(move || {
        let width_class = field
            .get()
            .map(|field| {
                if field.grid_width <= 2 {
                    " form-builder-grid-tile--icon-only"
                } else if field.grid_width >= 4 {
                    " form-builder-grid-tile--mobile-label"
                } else {
                    ""
                }
            })
            .unwrap_or("");
        format!(
            "form-builder-grid-tile form-builder-grid-field--summary form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary{width_class}"
        )
    });
    let selected = Signal::derive(move || active_builder_field.get() == Some(field_id));
    let dragging = Signal::derive(move || dragged_builder_field.get() == Some(field_id));
    let on_select = Callback::new(move |_| {
        if suppress_builder_field_click.get_untracked() == Some(field_id) {
            suppress_builder_field_click.set(None);
        } else {
            dragged_builder_field.set(None);
            active_builder_field.set(Some(field_id));
        }
    });
    let on_drag_start = Callback::new(move |_event: leptos::ev::DragEvent| {
        #[cfg(feature = "hydrate")]
        {
            use wasm_bindgen::JsCast;
            if let Some(target) = _event
                .target()
                .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                && target
                    .closest(".placement-editor-resize-handle")
                    .ok()
                    .flatten()
                    .is_some()
            {
                _event.prevent_default();
                return;
            }
        }
        clear_form_builder_drag_intent(
            builder_drag_preview,
            pending_builder_drag_preview,
            builder_drag_preview_timeout,
        );
        dragged_builder_field.set(Some(field_id));
    });
    let on_drag_enter = Callback::new(move |event: leptos::ev::DragEvent| {
        if let Some(dragged_field_id) = dragged_builder_field.get_untracked() {
            event.prevent_default();
            let Some(field) = field.get_untracked() else {
                return;
            };
            let cell = tessara_module_ui::placement_editor::PlacementGridCell {
                row: field.grid_row.max(1),
                column: field.grid_column.max(1),
            };
            schedule_form_builder_drag_preview(
                builder_drag_preview,
                pending_builder_drag_preview,
                builder_drag_preview_timeout,
                FormBuilderDragPreview {
                    placement_id: dragged_field_id,
                    canvas_id: section_id,
                    row: cell.row,
                    column: cell.column,
                },
                placement_grid_cell_id(&format!("form-builder-section-{section_id}"), cell),
            );
        }
    });
    let on_drag_end = Callback::new(move |_| {
        clear_form_builder_drag_intent(
            builder_drag_preview,
            pending_builder_drag_preview,
            builder_drag_preview_timeout,
        );
        dragged_builder_field.set(None);
    });

    view! {
        <PlacementTileShell
            rect=rect
            class=class
            selected=selected
            dragging=dragging
            on_select=on_select
            on_drag_start=on_drag_start
            on_drag_enter=on_drag_enter
            on_drag_end=on_drag_end
        >
            <button
                class="form-builder-grid-field__summary"
                type="button"
                title=display_label
                aria-label=move || format!("Configure {}", display_label())
                on:click=move |event| {
                    event.stop_propagation();
                    if suppress_builder_field_click.get_untracked() == Some(field_id) {
                        suppress_builder_field_click.set(None);
                    } else {
                        dragged_builder_field.set(None);
                        active_builder_field.set(Some(field_id));
                    }
                }
            >
                <span class="form-builder-field-type-icon">
                    {move || {
                        field
                            .get()
                            .map(|field| form_builder_field_type_icon(&field.field_type))
                            .unwrap_or_else(|| form_builder_field_type_icon("text"))
                    }}
                </span>
                <div>
                    <h5>{display_label}</h5>
                </div>
            </button>
            <FormBuilderFieldResizeHandles field_id builder_fields suppress_builder_field_click/>
        </PlacementTileShell>
    }
}
