//! Form builder grid component.
//!
//! Keep grid-cell rendering, drop targets, and field placement visualization here.

use leptos::prelude::*;

use super::grid_actions::add_form_builder_field_from_grid_click;
use super::grid_empty_cells::FormBuilderGridEmptyCells;
use crate::builder::FormBuilderFieldDraft;
use crate::builder::components::field_tile::FormBuilderGridTile;
use crate::builder::{
    FORM_BUILDER_COLUMN_COUNT, FormBuilderDragPreview, FormBuilderSectionLayout,
    clear_form_builder_drag_intent, commit_form_builder_drag_preview,
    schedule_form_builder_drag_preview, set_form_builder_drag_preview,
};
use tessara_web_ui::placement_editor::{
    PlacementGridCanvas, PlacementGridCell, PlacementGridTarget,
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
    let grid_cells = Signal::derive(move || {
        let row_count = grid_rows.get();
        (1..=row_count)
            .flat_map(|row| {
                (1..=FORM_BUILDER_COLUMN_COUNT).map(move |column| PlacementGridCell { row, column })
            })
            .collect::<Vec<_>>()
    });
    let row_count = Signal::derive(move || grid_rows.get());
    let dragging = Signal::derive(move || dragged_builder_field.get().is_some());
    let on_drag_target = Callback::new(move |target: PlacementGridTarget| {
        let Some(field_id) = dragged_builder_field.get_untracked() else {
            return;
        };
        schedule_form_builder_drag_preview(
            builder_drag_preview,
            pending_builder_drag_preview,
            builder_drag_preview_timeout,
            FormBuilderDragPreview {
                placement_id: field_id,
                canvas_id: section_id,
                row: target.cell.row,
                column: target.cell.column,
            },
            target.target_id,
        );
    });
    let on_drop_target = Callback::new(move |target: PlacementGridTarget| {
        if let Some(field_id) = dragged_builder_field.get_untracked() {
            set_form_builder_drag_preview(
                builder_drag_preview,
                FormBuilderDragPreview {
                    placement_id: field_id,
                    canvas_id: section_id,
                    row: target.cell.row,
                    column: target.cell.column,
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
    });
    let on_cancel_drag = Callback::new(move |_| {
        clear_form_builder_drag_intent(
            builder_drag_preview,
            pending_builder_drag_preview,
            builder_drag_preview_timeout,
        );
    });
    let on_click = Callback::new(move |event| {
        add_form_builder_field_from_grid_click(
            event,
            section_id,
            default_column_width,
            builder_fields,
            active_builder_field,
            suppress_builder_field_click,
            next_builder_field_id,
        );
    });

    view! {
        <PlacementGridCanvas
            canvas_id=format!("form-builder-section-{section_id}")
            row_count=row_count
            dragging=dragging
            on_drag_target=on_drag_target
            on_drop_target=on_drop_target
            on_cancel_drag=on_cancel_drag
            on_click=on_click
            class="form-builder-layout-grid"
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
        </PlacementGridCanvas>
    }
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;
    use crate::builder::{
        FormBuilderSectionDraft, blank_form_builder_field_at, form_builder_section_layout,
    };

    #[test]
    fn form_grid_renders_shared_primitives_without_changing_form_affordances() {
        let html = Owner::new().with(|| {
            let section = FormBuilderSectionDraft {
                id: 1,
                remote_id: None,
                title: "Main".into(),
                description: String::new(),
                default_column_width: 6,
                position: 1,
            };
            let mut field = blank_form_builder_field_at(1, 1, 1, 1, 6);
            field.label = "Customer".into();
            let builder_fields = RwSignal::new(vec![field]);
            let layout =
                Memo::new(move |_| form_builder_section_layout(&section, &builder_fields.get()));
            let default_column_width = Memo::new(|_| 6);

            view! {
                <FormBuilderGrid
                    section_id=1
                    layout
                    default_column_width
                    builder_fields
                    active_builder_field=RwSignal::new(None::<usize>)
                    dragged_builder_field=RwSignal::new(None::<usize>)
                    builder_drag_preview=RwSignal::new(None::<FormBuilderDragPreview>)
                    pending_builder_drag_preview=RwSignal::new(None::<FormBuilderDragPreview>)
                    builder_drag_preview_timeout=RwSignal::new(None::<i32>)
                    suppress_builder_field_click=RwSignal::new(None::<usize>)
                    next_builder_field_id=RwSignal::new(2usize)
                />
            }
            .to_html()
        });

        assert!(html.contains("placement-editor-grid form-builder-layout-grid"));
        assert!(html.contains("aria-label=\"Add field at row 1, column 1\""));
        assert!(html.contains("aria-label=\"Configure Customer\""));
        assert!(html.contains("title=\"Resize field width\""));
        assert!(html.contains("title=\"Resize field height\""));
    }
}
