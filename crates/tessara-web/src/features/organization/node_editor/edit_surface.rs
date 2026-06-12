//! Organization node edit route surface.

use crate::types::route_params::{NodeRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    EmptyState, PageHeader,
};
use leptos::prelude::*;

use super::{OrganizationNodeEditForm, OrganizationNodeEditState, load_organization_edit_options};

#[component]
pub(super) fn OrganizationNodeEditSurface() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;
    let OrganizationNodeEditState {
        node_types,
        nodes,
        detail,
        selected_parent_node_id,
        name,
        metadata_fields,
        metadata_values,
        metadata_booleans,
        is_loading,
        is_saving,
        message,
    } = OrganizationNodeEditState::new();

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
                        view! {
                            <OrganizationNodeEditForm
                                node_id=node_id.clone()
                                node_types
                                nodes
                                detail
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
