//! Node lineage filter for forms lists.

use crate::features::forms::{
    FormNodeFilterOption, indented_node_label, visible_form_node_filter_options,
};

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
                <div class="forms-node-filter__selected">
                    {move || {
                        let selected = selected_options();
                        if selected.is_empty() {
                            view! { <p class="forms-node-filter__empty">"No node selected"</p> }.into_any()
                        } else {
                            view! {
                                <div class="forms-node-filter__chips">
                                    {selected
                                        .into_iter()
                                        .map(|option| {
                                            let option_id = option.id.clone();
                                            let selected_node_id_for_chip = selected_node_id;
                                            let query_for_chip = query;
                                            view! {
                                                <button
                                                    class="forms-node-filter__chip"
                                                    type="button"
                                                    on:click=move |_| {
                                                        selected_node_id_for_chip.set(Some(option_id.clone()));
                                                        query_for_chip.set(String::new());
                                                    }
                                                >
                                                    <span>{option.name}</span>
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                            .into_any()
                        }
                    }}
                    {move || {
                        if selected_node_id.get().is_some() {
                            view! {
                                <button
                                    class="forms-node-filter__clear"
                                    type="button"
                                    on:click=move |_| {
                                        selected_node_id.set(None);
                                        query.set(String::new());
                                    }
                                >
                                    "Clear node filter"
                                </button>
                            }
                            .into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }
                    }}
                </div>
                <div class="forms-node-filter__options" role="listbox">
                    {move || {
                        let visible = visible_options();
                        if visible.is_empty() {
                            view! { <p class="forms-node-filter__empty">"No matching nodes"</p> }.into_any()
                        } else {
                            visible
                                .into_iter()
                                .map(|option| {
                                    let option_id = option.id.clone();
                                    let selected_node_id_for_option = selected_node_id;
                                    let query_for_option = query;
                                    let is_open_for_option = is_open;
                                    let is_selected = selected_node_id
                                        .get()
                                        .as_deref()
                                        .is_some_and(|selected| selected == option_id.as_str());
                                    view! {
                                        <button
                                            class=if is_selected { "forms-node-filter__option is-active" } else { "forms-node-filter__option" }
                                            type="button"
                                            role="option"
                                            aria-selected=is_selected.to_string()
                                            on:click=move |_| {
                                                selected_node_id_for_option.set(Some(option_id.clone()));
                                                query_for_option.set(String::new());
                                                is_open_for_option.set(false);
                                            }
                                        >
                                            <span>{indented_node_label(&option)}</span>
                                        </button>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}
