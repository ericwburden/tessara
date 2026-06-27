//! Root-local searchable data-table layout component.
//!
//! The generic `DataTable` wrapper lives in `tessara-web-ui`; this root module keeps the search affordance used by non-extracted features.

use icons::Search;
use leptos::prelude::*;
use tessara_web_ui::DataTable;

#[component]
pub fn SearchableDataTable(
    search_label: &'static str,
    placeholder: &'static str,
    search: RwSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="searchable-data-table">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">{search_label}</span>
                <input
                    type="search"
                    placeholder=placeholder
                    prop:value=move || search.get()
                    on:input=move |event| search.set(event_target_value(&event))
                />
            </label>
            <DataTable>{children()}</DataTable>
        </div>
    }
}
