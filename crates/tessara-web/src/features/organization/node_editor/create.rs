//! Organization node create page implementation.

use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode,
};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, PageHeader,
};
use leptos::prelude::*;
use std::collections::HashMap;

use super::{
    MetadataFieldInput, available_node_types_for_parent, load_node_type_metadata,
    load_organization_create_options, parent_node_options, submit_create_node,
};

/// Renders the organization new page view.
#[component]
pub(crate) fn OrganizationNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let selected_node_type_id = RwSignal::new(String::new());
    let selected_parent_node_id = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let metadata_fields = RwSignal::new(Vec::<NodeMetadataFieldSummary>::new());
    let metadata_values = RwSignal::new(HashMap::<String, String>::new());
    let metadata_booleans = RwSignal::new(HashMap::<String, bool>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_organization_create_options(
            node_types,
            nodes,
            selected_node_type_id,
            selected_parent_node_id,
            is_loading,
            message,
        );
    });

    Effect::new(move |_| {
        let node_type_id = selected_node_type_id.get();
        if node_type_id.is_empty() {
            metadata_fields.set(Vec::new());
            metadata_values.set(HashMap::new());
            metadata_booleans.set(HashMap::new());
            return;
        }

        load_node_type_metadata(
            node_type_id,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            message,
        );
    });

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
        <AppShell active_route="organization" title="Organization">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/organization">"Organization"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Create Node"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel organization-page">
                <PageHeader title="Create Organization Node">
                    <Button label="Back to Organization" href="/organization"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading create options"</h3>
                                <p>"Fetching organization node types and visible parent records."</p>
                            </section>
                        }
                        .into_any()
                    } else {
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

                                <section class="form-section">
                                    <h3>"Metadata"</h3>
                                    {move || {
                                        let fields = metadata_fields.get();
                                        if fields.is_empty() {
                                            view! { <p class="muted">"No metadata fields are configured for this node type."</p> }.into_any()
                                        } else {
                                            view! {
                                                <div class="form-grid">
                                                    {fields.into_iter().map(|field| {
                                                        view! {
                                                            <MetadataFieldInput
                                                                field
                                                                metadata_values
                                                                metadata_booleans
                                                            />
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }
                                            .into_any()
                                        }
                                    }}
                                </section>

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
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
