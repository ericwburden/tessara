//! Shared searchable data-table layout component.

use leptos::prelude::*;

use crate::{DataTable, TableSearch};

#[component]
pub fn SearchableDataTable(
    search_label: &'static str,
    placeholder: &'static str,
    search: RwSignal<String>,
    children: Children,
) -> impl IntoView {
    view! {
        <div class="searchable-data-table">
            <TableSearch
                value=Signal::from(search)
                on_input=Callback::new(move |value| search.set(value))
                label=search_label
                placeholder=placeholder
            />
            <DataTable>{children()}</DataTable>
        </div>
    }
}
