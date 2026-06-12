//! Empty-cell layer for the form builder grid.

use crate::features::forms::builder::FormBuilderGridCell;
use leptos::prelude::*;

#[component]
pub(in crate::features::forms::builder::components) fn FormBuilderGridEmptyCells(
    section_id: usize,
    grid_cells: Memo<Vec<FormBuilderGridCell>>,
) -> impl IntoView {
    view! {
        <div class="form-builder-grid-cells">
            <For
                each=move || grid_cells.get()
                key=|cell| (cell.row, cell.column)
                children=move |cell| {
                    let cell_label =
                        format!("Add field at row {}, column {}", cell.row, cell.column);
                    view! {
                        <div
                            id=format!("form-builder-section-{section_id}-cell-r{}-c{}", cell.row, cell.column)
                            class="form-builder-grid-cell form-builder-grid-cell--empty"
                            data-row=cell.row
                            data-column=cell.column
                            data-empty=true
                            aria-label=cell_label
                            style=format!("grid-column: {}; grid-row: {};", cell.column, cell.row)
                        ></div>
                    }
                }
            />
        </div>
    }
}
