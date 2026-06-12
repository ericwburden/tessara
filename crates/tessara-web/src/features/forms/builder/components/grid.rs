//! Form builder grid component.
//!
//! Keep grid-cell rendering, drop targets, and field placement visualization here.

use leptos::prelude::*;

use super::grid_actions::add_form_builder_field_from_grid_click;
use super::grid_empty_cells::FormBuilderGridEmptyCells;
use crate::features::forms::builder::FormBuilderFieldDraft;
use crate::features::forms::builder::components::field_tile::FormBuilderGridTile;
use crate::features::forms::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderGridCell,
    FormBuilderSectionLayout, clear_form_builder_drag_intent, commit_form_builder_drag_preview,
    form_builder_grid_cell_from_drag_event, form_builder_grid_cell_from_pointer,
    schedule_form_builder_drag_preview, set_form_builder_drag_preview,
};

#[component]
pub(crate) fn FormBuilderGrid(
    section_id: usize,
    layout: Memo<FormBuilderSectionLayout>,
    default_column_width: Memo<i32>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) -> impl IntoView {
    let grid_rows = Memo::new(move |_| layout.get().row_count);
    let grid_cells = Memo::new(move |_| {
        let row_count = grid_rows.get();
        (1..=row_count)
            .flat_map(|row| {
                (1..=FORM_BUILDER_COLUMN_COUNT)
                    .map(move |column| FormBuilderGridCell { row, column })
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div
            data-section-id=section_id
            class=move || {
                if dragged_builder_field.get().is_some() {
                    "form-builder-layout-grid is-dragging"
                } else {
                    "form-builder-layout-grid"
                }
            }
            style=move || {
                let row_count = grid_rows.get();
                format!(
                    "--form-builder-rows: {}; --form-builder-max-height: {}px;",
                    row_count,
                    row_count * 80,
                )
            }
            on:dragenter=move |event| {
                let Some(field_id) = dragged_builder_field.get_untracked() else {
                    return;
                };
                let Some((row, column, target_id)) = form_builder_grid_cell_from_drag_event(&event) else {
                    return;
                };
                event.prevent_default();
                schedule_form_builder_drag_preview(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    FormBuilderDragPreview {
                        field_id,
                        section_id,
                        row,
                        column,
                    },
                    target_id,
                );
            }
            on:dragover=move |event| {
                let Some(field_id) = dragged_builder_field.get_untracked() else {
                    return;
                };
                event.prevent_default();
                let Some((row, column, target_id)) =
                    form_builder_grid_cell_from_pointer(&event, grid_rows.get_untracked())
                else {
                    return;
                };
                schedule_form_builder_drag_preview(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    FormBuilderDragPreview {
                        field_id,
                        section_id,
                        row,
                        column,
                    },
                    target_id,
                );
            }
            on:drop=move |event| {
                event.prevent_default();
                if let Some(field_id) = dragged_builder_field.get_untracked()
                    && let Some((row, column, _)) =
                        form_builder_grid_cell_from_pointer(&event, grid_rows.get_untracked())
                    {
                        set_form_builder_drag_preview(
                            builder_drag_preview,
                            FormBuilderDragPreview {
                                field_id,
                                section_id,
                                row,
                                column,
                            },
                        );
                    }
                commit_form_builder_drag_preview(
                    builder_fields,
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    dragged_builder_field,
                    suppress_builder_field_click,
                );
            }
            on:mouseleave=move |_| {
                if dragged_builder_field.get_untracked().is_some() {
                    clear_form_builder_drag_intent(
                        builder_drag_preview,
                        pending_builder_drag_preview,
                        builder_drag_preview_timeout,
                    );
                }
            }
            on:click=move |event| {
                add_form_builder_field_from_grid_click(
                    event,
                    section_id,
                    default_column_width,
                    builder_fields,
                    active_builder_field,
                    suppress_builder_field_click,
                    next_builder_field_id,
                );
            }
        >
            <FormBuilderGridEmptyCells section_id grid_cells/>
            <For
                each=move || layout.get().fields
                key=|field| field.id
                children=move |field| {
                    view! {
                        <FormBuilderGridTile
                            field_id=field.id
                            section_id=section_id
                            builder_fields=builder_fields
                            active_builder_field=active_builder_field
                            dragged_builder_field=dragged_builder_field
                            builder_drag_preview=builder_drag_preview
                            pending_builder_drag_preview=pending_builder_drag_preview
                            builder_drag_preview_timeout=builder_drag_preview_timeout
                            suppress_builder_field_click=suppress_builder_field_click
                        />
                    }
                }
            />
        </div>
    }
}
