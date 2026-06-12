//! Organization node edit page implementation.

use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
};
use crate::types::route_params::{NodeRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, EmptyState, PageHeader,
};
use leptos::prelude::*;
use std::collections::HashMap;

use super::{
    OrganizationNodeMetadataSection, load_organization_edit_options, parent_node_options_for_edit,
    submit_update_node,
};

/// Route page for editing an organization node and its metadata values.
#[component]
pub(crate) fn OrganizationEditPage() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let detail = RwSignal::new(None::<OrganizationNodeDetail>);
    let selected_parent_node_id = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let metadata_fields = RwSignal::new(Vec::<NodeMetadataFieldSummary>::new());
    let metadata_values = RwSignal::new(HashMap::<String, String>::new());
    let metadata_booleans = RwSignal::new(HashMap::<String, bool>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    let load_node_id = node_id.clone();
    Effect::new(move |_| {
        load_organization_edit_options(
            load_node_id.clone(),
            node_types,
            nodes,
            detail,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_loading,
            message,
        );
    });

    let option_node_id = node_id.clone();
    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="organization" title="Organization">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/organization">"Organization"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail.get().map(|node| {
                        let href = format!("/organization/{}", node.id);
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbLink href=href>{node.name}</BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator/>
                        }
                    })
                }}
                <BreadcrumbItem>
                    <BreadcrumbPage>"Edit Node"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel organization-page organization-edit-page">
                <PageHeader title="Edit Organization Node"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading edit options"</h3>
                                <p>"Fetching organization node details."</p>
                            </section>
                        }
                        .into_any()
                    } else if detail.get().is_none() {
                        view! {
                            <EmptyState
                                title="Organization node unavailable"
                                message="The selected node could not be loaded for editing."
                            />
                        }
                        .into_any()
                    } else {
                        let node = detail.get().expect("detail is checked above");
                        let option_node_id_for_options = option_node_id.clone();
                        let submit_node_id = node_id.clone();
                        view! {
                            <form
                                class="native-form organization-node-form"
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit_update_node(
                                        submit_node_id.clone(),
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
                                            on:change=move |event| selected_parent_node_id.set(event_target_value(&event))
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
                                                            &option_node_id_for_options,
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

                                <OrganizationNodeMetadataSection
                                    metadata_fields
                                    metadata_values
                                    metadata_booleans
                                />

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
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
