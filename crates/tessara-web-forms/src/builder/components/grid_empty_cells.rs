//! Form adapter for the shared placement guide layer.

use leptos::prelude::*;
use tessara_module_ui::placement_editor::{PlacementGridCell, PlacementGridGuides};

#[component]
pub(crate) fn FormBuilderGridEmptyCells(
    section_id: usize,
    grid_cells: Signal<Vec<PlacementGridCell>>,
) -> impl IntoView {
    view! {
        <PlacementGridGuides
            canvas_id=format!("form-builder-section-{section_id}")
            cells=grid_cells
            cell_label=Callback::new(|cell: PlacementGridCell| {
                format!("Add field at row {}, column {}", cell.row, cell.column)
            })
            class="form-builder-grid-cells"
            cell_class="form-builder-grid-cell form-builder-grid-cell--empty"
            empty=true
        />
    }
}
