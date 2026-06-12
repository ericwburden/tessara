//! Node type selector component.

use crate::features::organization::NodeTypeCatalogEntry;
use icons::{ChevronDown, ListFilter, Search};
use leptos::prelude::*;

#[component]
/// Renders the administration node types list view.
pub(crate) fn AdministrationNodeTypesList(
    node_types: Vec<NodeTypeCatalogEntry>,
    search: RwSignal<String>,
    selected_node_type_id: RwSignal<Option<String>>,
    is_creating: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let options_for_label = node_types.clone();
    let options_for_visible = node_types;
    let selected_label = move || {
        if is_creating {
            "New node type".to_string()
        } else {
            selected_node_type_id
                .get()
                .as_deref()
                .and_then(|selected| {
                    options_for_label
                        .iter()
                        .find(|node_type| node_type.id == selected)
                })
                .map(|node_type| {
                    format!(
                        "{} ({}, {} nodes)",
                        node_type.name, node_type.slug, node_type.node_count
                    )
                })
                .unwrap_or_else(|| "Select node type".to_string())
        }
    };
    let visible_node_types = move || {
        let query = search.get().trim().to_lowercase();
        options_for_visible
            .iter()
            .filter(|node_type| {
                query.is_empty()
                    || node_type.name.to_lowercase().contains(&query)
                    || node_type.slug.to_lowercase().contains(&query)
                    || node_type.singular_label.to_lowercase().contains(&query)
                    || node_type.plural_label.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    view! {
        <section class="administration-node-types-selector">
            <h2>"Node Type Catalog"</h2>
            <div class=move || if is_open.get() { "forms-node-filter node-type-selector is-open" } else { "forms-node-filter node-type-selector" }>
                <button
                    class=move || if selected_node_type_id.get().is_some() && !is_creating { "forms-node-filter__trigger is-filtered" } else { "forms-node-filter__trigger" }
                    type="button"
                    role="combobox"
                    aria-haspopup="listbox"
                    aria-expanded=move || is_open.get().to_string()
                    aria-label="Select node type"
                    title="Select node type"
                    on:click=move |_| is_open.update(|open| *open = !*open)
                >
                    <ListFilter/>
                    <span>{selected_label}</span>
                    <ChevronDown/>
                </button>
                <button
                    class="forms-node-filter__scrim"
                    type="button"
                    aria-label="Close node type selector"
                    on:click=move |_| is_open.set(false)
                ></button>
                <div
                    class="forms-node-filter__menu blurred-surface floating-layer"
                    data-mobile-behavior="dialog"
                    role="dialog"
                    aria-label="Select node type"
                >
                    <label class="forms-node-filter__search">
                        <Search/>
                        <span class="sr-only">"Search node types"</span>
                        <input
                            type="search"
                            placeholder="Search node types"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                    <div class="forms-node-filter__options" role="listbox">
                        {move || {
                            let visible = visible_node_types();
                            if visible.is_empty() {
                                view! {
                                    <p class="forms-node-filter__empty">"No matching node types to display"</p>
                                }
                                .into_any()
                            } else {
                                visible
                                    .into_iter()
                                    .map(|node_type| {
                                        let node_type_id = node_type.id.clone();
                                        let selected_node_type_id_for_option = selected_node_type_id;
                                        let search_for_option = search;
                                        let is_selected = selected_node_type_id
                                            .get()
                                            .map(|selected| selected == node_type_id)
                                            .unwrap_or(false);
                                        view! {
                                            <button
                                                class=if is_selected { "forms-node-filter__option is-active node-type-selector__option" } else { "forms-node-filter__option node-type-selector__option" }
                                                type="button"
                                                role="option"
                                                aria-selected=is_selected.to_string()
                                                on:click=move |_| {
                                                    selected_node_type_id_for_option.set(Some(node_type_id.clone()));
                                                    search_for_option.set(String::new());
                                                    is_open.set(false);
                                                }
                                            >
                                                <span>
                                                    <strong>{node_type.name}</strong>
                                                    <small>{node_type.slug}</small>
                                                </span>
                                                <span class="node-type-list__meta">{node_type.node_count} " nodes"</span>
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
        </section>
    }
}
