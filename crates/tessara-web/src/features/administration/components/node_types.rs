//! Node type Administration components.

use super::super::graph::{node_type_ancestor_ids, node_type_descendant_ids};
use crate::features::organization::{NodeTypeCatalogEntry, NodeTypeDefinition};
use crate::features::shared::status_badge_class;
use leptos::prelude::*;
use std::collections::HashSet;

use super::node_type_editor_options::{eligible_child_node_types, eligible_parent_node_types};
use super::node_type_identity_fields::NodeTypeIdentityFields;
use super::{NodeTypeDetailCollections, NodeTypeRelationshipPicker};

#[component]
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
            eligible_parent_node_types(
                &all_node_types,
                current_id.as_ref(),
                &descendant_ids,
                &child_node_type_ids.get(),
            )
        }
    };
    let child_picker_node_types = {
        let all_node_types = all_node_types.clone();
        let current_id = current_id.clone();
        let ancestor_ids = ancestor_ids.clone();
        move || {
            eligible_child_node_types(
                &all_node_types,
                current_id.as_ref(),
                &ancestor_ids,
                &parent_node_type_ids.get(),
            )
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
                        <NodeTypeIdentityFields
                            selected_detail=selected_detail.clone()
                            name
                            slug
                            plural_label
                        />
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
