//! Organization node create page implementation.

use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, PageHeader,
};
use leptos::prelude::*;
use std::collections::HashMap;

use super::{
    OrganizationNodeCreateForm, OrganizationNodeCreateState, load_node_type_metadata,
    load_organization_create_options,
};

/// Route page for creating an organization node and seeding its metadata editor.
#[component]
pub(crate) fn OrganizationNewPage() -> impl IntoView {
    let OrganizationNodeCreateState {
        node_types,
        nodes,
        selected_node_type_id,
        selected_parent_node_id,
        name,
        metadata_fields,
        metadata_values,
        metadata_booleans,
        is_loading,
        is_saving,
        message,
    } = OrganizationNodeCreateState::new();

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
                            <OrganizationNodeCreateForm
                                node_types
                                nodes
                                selected_node_type_id
                                selected_parent_node_id
                                name
                                metadata_fields
                                metadata_values
                                metadata_booleans
                                is_loading
                                is_saving
                                message
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
