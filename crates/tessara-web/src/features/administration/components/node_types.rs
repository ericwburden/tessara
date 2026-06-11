//! Node type Administration components.

use super::super::graph::{node_type_ancestor_ids, node_type_descendant_ids};
use crate::features::organization::{NodeTypeCatalogEntry, NodeTypeDefinition};
use crate::features::shared::status_badge_class;
use icons::{ChevronDown, ListFilter, Search};
use leptos::prelude::*;
use std::collections::HashSet;

use super::{NodeTypeDetailCollections, NodeTypeRelationshipPicker};

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

#[component]
/// Renders the administration node type editor view.
pub(crate) fn AdministrationNodeTypeEditor(
    all_node_types: Vec<NodeTypeCatalogEntry>,
    selected_detail: Option<NodeTypeDefinition>,
    is_creating: bool,
    detail_loading: bool,
    is_saving: bool,
    message: Option<String>,
    selected_node_type_id: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    plural_label: RwSignal<String>,
    parent_node_type_ids: RwSignal<HashSet<String>>,
    child_node_type_ids: RwSignal<HashSet<String>>,
    on_cancel: impl Fn(leptos::ev::MouseEvent) + 'static + Copy,
    on_submit: impl Fn(leptos::ev::SubmitEvent) + 'static + Copy,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let title = if is_creating {
        "Create Node Type".to_string()
    } else {
        selected_detail
            .as_ref()
            .map(|detail| detail.name.clone())
            .unwrap_or_else(|| "Select a Node Type".to_string())
    };
    let current_id = selected_node_type_id.get();
    let ancestor_ids = current_id
        .as_deref()
        .map(|id| node_type_ancestor_ids(id, &all_node_types))
        .unwrap_or_default();
    let descendant_ids = current_id
        .as_deref()
        .map(|id| node_type_descendant_ids(id, &all_node_types))
        .unwrap_or_default();
    let node_type_kind_label = move || {
        if parent_node_type_ids.get().is_empty() {
            "Root Type"
        } else {
            "Child Type"
        }
    };
    let node_type_kind_status = move || {
        if parent_node_type_ids.get().is_empty() {
            status_badge_class("active")
        } else {
            status_badge_class("inactive")
        }
    };
    let parent_picker_node_types = {
        let all_node_types = all_node_types.clone();
        let current_id = current_id.clone();
        let descendant_ids = descendant_ids.clone();
        move || {
            let selected_child_ids = child_node_type_ids.get();
            let mut disqualified_parent_ids = descendant_ids.clone();
            for child_id in &selected_child_ids {
                disqualified_parent_ids.insert(child_id.clone());
                disqualified_parent_ids.extend(node_type_descendant_ids(child_id, &all_node_types));
            }
            all_node_types
                .iter()
                .filter(|node_type| current_id.as_ref() != Some(&node_type.id))
                .filter(|node_type| !disqualified_parent_ids.contains(&node_type.id))
                .cloned()
                .collect::<Vec<_>>()
        }
    };
    let child_picker_node_types = {
        let all_node_types = all_node_types.clone();
        let current_id = current_id.clone();
        let ancestor_ids = ancestor_ids.clone();
        move || {
            let selected_parent_ids = parent_node_type_ids.get();
            let mut disqualified_child_ids = ancestor_ids.clone();
            for parent_id in &selected_parent_ids {
                disqualified_child_ids.insert(parent_id.clone());
                disqualified_child_ids.extend(node_type_ancestor_ids(parent_id, &all_node_types));
            }
            all_node_types
                .iter()
                .filter(|node_type| current_id.as_ref() != Some(&node_type.id))
                .filter(|node_type| !disqualified_child_ids.contains(&node_type.id))
                .cloned()
                .collect::<Vec<_>>()
        }
    };

    view! {
        <form class="native-form administration-node-type-editor" on:submit=on_submit>
            <section class="organization-detail-card organization-detail-card--wide">
                <div class="organization-detail-card__header">
                    <h2>{title}</h2>
                    {if selected_detail.is_some() || is_creating {
                        view! {
                            <span class=node_type_kind_status>{move || node_type_kind_label()}</span>
                        }
                        .into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </div>

                {if detail_loading {
                    view! {
                        <section class="organization-state" aria-live="polite">
                            <h3>"Loading node type"</h3>
                            <p>"Fetching node type details."</p>
                        </section>
                    }
                    .into_any()
                } else {
                    view! {
                        <div class="form-grid administration-node-type-fields">
                            <label class="form-field">
                                <span>"Name"</span>
                                <input
                                    type="text"
                                    placeholder="Program"
                                    prop:value=move || name.get()
                                    on:input=move |event| name.set(event_target_value(&event))
                                />
                            </label>
                            <label class="form-field">
                                <span>"Slug"</span>
                                <input
                                    type="text"
                                    placeholder="program"
                                    prop:value=move || slug.get()
                                    on:input=move |event| slug.set(event_target_value(&event))
                                />
                            </label>
                            <label class="form-field">
                                <span>"Plural Label"</span>
                                <input
                                    type="text"
                                    placeholder="Programs"
                                    prop:value=move || plural_label.get()
                                    on:input=move |event| plural_label.set(event_target_value(&event))
                                />
                            </label>
                            {if let Some(detail) = selected_detail.as_ref() {
                                let node_count = detail.node_count.to_string();
                                let metadata_count = detail.metadata_fields.len().to_string();
                                let scoped_form_count = detail.scoped_forms.len().to_string();
                                view! {
                                    <div class="node-type-count-badges" aria-label="Node type counts">
                                        <span class="node-type-count-badge">
                                            <strong>{node_count}</strong>
                                            <span>"Nodes"</span>
                                        </span>
                                        <span class="node-type-count-badge">
                                            <strong>{metadata_count}</strong>
                                            <span>"Metadata Fields"</span>
                                        </span>
                                        <span class="node-type-count-badge">
                                            <strong>{scoped_form_count}</strong>
                                            <span>"Scoped Forms"</span>
                                        </span>
                                    </div>
                                }
                                .into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </div>
                        {if selected_detail.is_none() {
                            view! { <p class="muted">"Configure labels and hierarchy relationships, then save this node type."</p> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}

                        <div class="administration-node-type-relationships">
                            {move || {
                                view! {
                                    <NodeTypeRelationshipPicker
                                        title="Allowed Parent Types"
                                        empty_message="No eligible parent types are available."
                                        node_types=parent_picker_node_types()
                                        selected_ids=parent_node_type_ids
                                        opposite_selected_ids=child_node_type_ids
                                    />
                                }
                            }}
                            {move || {
                                view! {
                                    <NodeTypeRelationshipPicker
                                        title="Allowed Child Types"
                                        empty_message="No eligible child types are available."
                                        node_types=child_picker_node_types()
                                        selected_ids=child_node_type_ids
                                        opposite_selected_ids=parent_node_type_ids
                                    />
                                }
                            }}
                        </div>

                        <NodeTypeDetailCollections detail=selected_detail.clone() on_metadata_changed/>

                        {if let Some(message) = message {
                            view! { <p class="form-message">{message}</p> }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}

                        <div class="form-actions">
                            {if is_creating {
                                view! {
                                    <button class="button button--secondary" type="button" on:click=on_cancel>"Cancel"</button>
                                }
                                .into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                            <button class="button" type="submit" disabled=is_saving>
                                {if is_saving { "Saving Node Type" } else { "Save Node Type" }}
                            </button>
                        </div>
                    }
                    .into_any()
                }}
            </section>
        </form>
    }
}
