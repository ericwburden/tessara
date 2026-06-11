//! Owns the features::forms::builder::components::field_tile module behavior.

use leptos::prelude::*;

use crate::features::forms::builder::FormBuilderFieldDraft;
use crate::features::forms::builder::{
    FormBuilderDragPreview, FormBuilderResizeAxis, clear_form_builder_drag_intent,
    form_builder_field_default_label, form_builder_field_type_icon,
    schedule_form_builder_drag_preview, start_form_builder_field_resize,
};

#[component]
/// Renders the form builder grid tile view.
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

    view! {
        <div
            class=move || {
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
                if dragged_builder_field.get() == Some(field_id) {
                    format!(
                        "form-builder-grid-tile form-builder-grid-field--summary is-dragging form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary{width_class}"
                    )
                } else {
                    format!(
                        "form-builder-grid-tile form-builder-grid-field--summary form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary{width_class}"
                    )
                }
            }
            draggable="true"
            style=move || {
                field
                    .get()
                    .map(|field| {
                        let width = field.grid_width.max(1);
                        let height = field.grid_height.max(1);
                        let row = field.grid_row.max(1);
                        let column = field.grid_column.max(1);
                        format!(
                            "grid-column: {column} / span {width}; grid-row: {row} / span {height};"
                        )
                    })
                    .unwrap_or_else(|| "display: none;".into())
            }
            on:dragstart=move |_event: leptos::ev::DragEvent| {
                #[cfg(feature = "hydrate")]
                {
                    use wasm_bindgen::JsCast;
                    if let Some(target) = _event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    {
                        if target.closest(".form-builder-resize-handle").ok().flatten().is_some() {
                            _event.prevent_default();
                            return;
                        }
                    }
                }
                clear_form_builder_drag_intent(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                );
                dragged_builder_field.set(Some(field_id));
            }
            on:dragenter=move |event| {
                if let Some(dragged_field_id) = dragged_builder_field.get_untracked() {
                    event.prevent_default();
                    let Some(field) = field.get_untracked() else {
                        return;
                    };
                    schedule_form_builder_drag_preview(
                        builder_drag_preview,
                        pending_builder_drag_preview,
                        builder_drag_preview_timeout,
                        FormBuilderDragPreview {
                            field_id: dragged_field_id,
                            section_id,
                            row: field.grid_row.max(1),
                            column: field.grid_column.max(1),
                        },
                        format!(
                            "form-builder-section-{section_id}-cell-r{}-c{}",
                            field.grid_row.max(1),
                            field.grid_column.max(1),
                        ),
                    );
                }
            }
            on:click=move |_| {
                if suppress_builder_field_click.get_untracked() == Some(field_id) {
                    suppress_builder_field_click.set(None);
                } else {
                    dragged_builder_field.set(None);
                    active_builder_field.set(Some(field_id));
                }
            }
            on:dragend=move |_| {
                clear_form_builder_drag_intent(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                );
                dragged_builder_field.set(None);
            }
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
            <span
                class="form-builder-resize-handle form-builder-resize-handle--width"
                title="Resize field width"
                aria-hidden="true"
                on:mousedown=move |event| {
                    start_form_builder_field_resize(
                        event,
                        FormBuilderResizeAxis::Width,
                        field_id,
                        builder_fields,
                        suppress_builder_field_click,
                    );
                }
            ></span>
            <span
                class="form-builder-resize-handle form-builder-resize-handle--height"
                title="Resize field height"
                aria-hidden="true"
                on:mousedown=move |event| {
                    start_form_builder_field_resize(
                        event,
                        FormBuilderResizeAxis::Height,
                        field_id,
                        builder_fields,
                        suppress_builder_field_click,
                    );
                }
            ></span>
        </div>
    }
}
