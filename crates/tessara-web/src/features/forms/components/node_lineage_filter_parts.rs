//! Subcomponents for the forms node lineage filter.

use crate::features::forms::{FormNodeFilterOption, indented_node_label};
use leptos::prelude::*;

#[component]
pub(super) fn FormsNodeLineageSelected(
    selected_options: Vec<FormNodeFilterOption>,
    selected_node_id: RwSignal<Option<String>>,
    query: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="forms-node-filter__selected">
            {move || {
                if selected_options.is_empty() {
                    view! { <p class="forms-node-filter__empty">"No node selected"</p> }.into_any()
                } else {
                    view! {
                        <div class="forms-node-filter__chips">
                            {selected_options
                                .clone()
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
    }
}

#[component]
pub(super) fn FormsNodeLineageOptions(
    visible_options: Vec<FormNodeFilterOption>,
    selected_node_id: RwSignal<Option<String>>,
    query: RwSignal<String>,
    is_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="forms-node-filter__options" role="listbox">
            {move || {
                if visible_options.is_empty() {
                    view! { <p class="forms-node-filter__empty">"No matching nodes"</p> }.into_any()
                } else {
                    visible_options
                        .clone()
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
    }
}
