//! Node lineage filter for forms lists.

use super::node_lineage_filter_parts::{FormsNodeLineageOptions, FormsNodeLineageSelected};
use crate::features::forms::{FormNodeFilterOption, visible_form_node_filter_options};

use icons::{ChevronDown, ListFilter, Search};
use leptos::prelude::*;

#[component]
pub(crate) fn FormsNodeLineageFilter(
    options: Vec<FormNodeFilterOption>,
    selected_node_id: RwSignal<Option<String>>,
    query: RwSignal<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let options_for_visible = options.clone();
    let options_for_label = options.clone();
    let options_for_selected = options.clone();
    let trigger_label = move || {
        let selected = selected_node_id.get();
        selected
            .as_deref()
            .and_then(|id| {
                options_for_label
                    .iter()
                    .find(|option| option.id == id)
                    .map(|option| option.name.clone())
            })
            .unwrap_or_else(|| "Filter by node".to_string())
    };
    let trigger_class = move || {
        if selected_node_id.get().is_none() {
            "forms-node-filter__trigger"
        } else {
            "forms-node-filter__trigger is-filtered"
        }
    };
    let visible_options = move || {
        visible_form_node_filter_options(
            &options_for_visible,
            selected_node_id.get().as_deref(),
            &query.get(),
        )
    };
    let selected_options = move || {
        selected_node_id
            .get()
            .as_deref()
            .and_then(|selected| {
                options_for_selected
                    .iter()
                    .find(|option| option.id == selected)
                    .cloned()
            })
            .into_iter()
            .collect::<Vec<_>>()
    };

    view! {
        <div class=move || if is_open.get() { "forms-node-filter is-open" } else { "forms-node-filter" }>
            <button
                class=trigger_class
                type="button"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get().to_string()
                aria-label="Filter forms by organization node"
                title="Filter forms by organization node"
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
                <span>{trigger_label}</span>
                <ChevronDown/>
            </button>
            <button
                class="forms-node-filter__scrim"
                type="button"
                aria-label="Close node filter"
                on:click=move |_| is_open.set(false)
            ></button>
            <div
                class="forms-node-filter__menu blurred-surface floating-layer"
                data-mobile-behavior="dialog"
                role="dialog"
                aria-label="Filter by organization node"
            >
                <label class="forms-node-filter__search">
                    <Search/>
                    <span class="sr-only">"Search organization nodes"</span>
                    <input
                        type="search"
                        placeholder="Search organization nodes"
                        prop:value=move || query.get()
                        on:input=move |event| query.set(event_target_value(&event))
                    />
                </label>
                {move || view! {
                    <FormsNodeLineageSelected
                        selected_options=selected_options()
                        selected_node_id
                        query
                    />
                }}
                {move || view! {
                    <FormsNodeLineageOptions
                        visible_options=visible_options()
                        selected_node_id
                        query
                        is_open
                    />
                }}
            </div>
        </div>
    }
}
