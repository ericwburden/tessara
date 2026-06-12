//! Edit form for organization nodes.

use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};
use crate::ui::Button;
use leptos::prelude::*;
use std::collections::HashMap;

use super::{OrganizationNodeMetadataSection, parent_node_options_for_edit, submit_update_node};

#[component]
pub(super) fn OrganizationNodeEditForm(
    node_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let node = detail
        .get_untracked()
        .expect("edit form is rendered only after detail exists");
    let option_node_id = node_id.clone();
    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <form
            class="native-form organization-node-form"
            on:submit=move |event| {
                event.prevent_default();
                submit_update_node(
                    node_id.clone(),
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
                <label class="form-field" for="organization-node-type">
                    <span>"Node Type"</span>
                    <input
                        id="organization-node-type"
                        type="text"
                        value=node.node_type_singular_label
                        readonly
                    />
                </label>

                <label class="form-field" for="organization-parent-node">
                    <span>"Parent Node"</span>
                    <select
                        id="organization-parent-node"
                        prop:value=move || selected_parent_node_id.get()
                        on:change=move |event| {
                            selected_parent_node_id.set(event_target_value(&event))
                        }
                    >
                        <Show when=move || {
                            detail
                                .get()
                                .and_then(|detail| {
                                    node_types
                                        .get()
                                        .into_iter()
                                        .find(|node_type| node_type.id == detail.node_type_id)
                                })
                                .map(|node_type| node_type.is_root_type)
                                .unwrap_or(false)
                        }>
                            <option value="">"Top-level record"</option>
                        </Show>
                        {move || {
                            detail
                                .get()
                                .map(|detail| {
                                    parent_node_options_for_edit(
                                        &nodes.get(),
                                        &node_types.get(),
                                        &option_node_id,
                                        &detail.node_type_id,
                                    )
                                })
                                .unwrap_or_default()
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
                    {move || if is_saving.get() { "Saving..." } else { "Save Changes" }}
                </button>
            </div>
        </form>
    }
}
