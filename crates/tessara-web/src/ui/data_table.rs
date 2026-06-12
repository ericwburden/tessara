//! Shared data-table layout components.
//!
//! This module owns reusable table wrappers and structural markup; row content, filters, and domain actions belong with feature pages.

use icons::Search;
use leptos::prelude::*;

#[component]
pub fn DataTable(children: Children) -> impl IntoView {
    view! {
        <div class="table-wrap">
            <table class="data-table">
                {children()}
            </table>
        </div>
    }
}

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
