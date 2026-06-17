//! Dataset editor filter controls.

use leptos::prelude::*;

#[component]
pub(crate) fn DatasetFiltersEditor() -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section dataset-filters-section">
            <div class="dataset-editor-section__header">
                <h3>"Filters"</h3>
            </div>
            <p class="muted">"No filters configured."</p>
        </section>
    }
}
