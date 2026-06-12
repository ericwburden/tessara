//! Create form for organization nodes.

use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode,
};
use crate::ui::Button;
use leptos::prelude::*;
use std::collections::HashMap;

use super::{
    OrganizationNodeMetadataSection, available_node_types_for_parent, parent_node_options,
    submit_create_node,
};

#[component]
pub(super) fn OrganizationNodeCreateForm(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    selected_node_type_id: RwSignal<String>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let parent_options = move || parent_node_options(&nodes.get());
    let node_type_options = move || {
        available_node_types_for_parent(
            &selected_parent_node_id.get(),
            &node_types.get(),
            &nodes.get(),
        )
    };
    let can_submit =
        move || !is_loading.get() && !is_saving.get() && !selected_node_type_id.get().is_empty();

    view! {
        <form
            class="native-form organization-node-form"
            on:submit=move |event| {
                event.prevent_default();
                submit_create_node(
                    selected_node_type_id,
                    selected_parent_node_id,
                    name,
                    metadata_fields,
                    metadata_values,
                    metadata_booleans,
                    is_saving,
                    message,
                );
            }
        >
            <div class="form-grid">
                <label class="form-field" for="organization-parent-node">
                    <span>"Parent Node"</span>
                    <select
                        id="organization-parent-node"
                        prop:value=move || selected_parent_node_id.get()
                        on:change=move |event| {
                            let parent_id = event_target_value(&event);
                            let available_types = available_node_types_for_parent(
                                &parent_id,
                                &node_types.get(),
                                &nodes.get(),
                            );
                            let current_type_id = selected_node_type_id.get();

                            selected_parent_node_id.set(parent_id);

                            if !available_types
                                .iter()
                                .any(|node_type| node_type.id == current_type_id)
                            {
                                selected_node_type_id.set(
                                    available_types
                                        .first()
                                        .map(|node_type| node_type.id.clone())
                                        .unwrap_or_default(),
                                );
                            }
                        }
                    >
                        <option value="">"Top-level record"</option>
                        {move || {
                            parent_options()
                                .into_iter()
                                .map(|option| {
                                    view! {
                                        <option value=option.id>{option.label}</option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                </label>

                <label class="form-field" for="organization-node-type">
                    <span>"Node Type"</span>
                    <select
                        id="organization-node-type"
                        prop:value=move || selected_node_type_id.get()
                        on:change=move |event| {
                            selected_node_type_id.set(event_target_value(&event))
                        }
                    >
                        <option value="">"Select node type"</option>
                        {move || {
                            node_type_options()
                                .into_iter()
                                .map(|node_type| {
                                    view! {
                                        <option value=node_type.id>
                                            {node_type.singular_label}
                                        </option>
                                    }
                                })
                                .collect_view()
                        }}
                    </select>
                </label>

                <label class="form-field form-field--wide" for="organization-name">
                    <span>"Name"</span>
                    <input
                        id="organization-name"
                        type="text"
                        autocomplete="off"
                        prop:value=move || name.get()
                        on:input=move |event| name.set(event_target_value(&event))
                        required
                    />
                </label>
            </div>

            <OrganizationNodeMetadataSection metadata_fields metadata_values metadata_booleans/>

            {move || message.get().map(|message| view! {
                <p class="form-message" role="status">{message}</p>
            })}

            <div class="form-actions">
                <Button label="Cancel" href="/organization"/>
                <button class="button" type="submit" disabled=move || !can_submit()>
                    {move || if is_saving.get() { "Saving..." } else { "Create Node" }}
                </button>
            </div>
        </form>
    }
}
