//! Route-level page composition for the Organization feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use super::detail::{OrganizationDetailFullContent, OrganizationDetailSheet};
pub(crate) use super::node_editor::{OrganizationEditPage, OrganizationNewPage};
use super::tree::{load_organization_detail, load_organization_tree, organization_tree_view};
use super::types::{NodeTypeCatalogEntry, OrganizationNodeDetail, OrganizationTreeNode};
use crate::types::route_params::{NodeRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, EmptyState, PageHeader,
};
use leptos::prelude::*;
use std::collections::HashSet;

#[component]
/// Renders the organization page view.
pub fn OrganizationPage() -> impl IntoView {
    let tree = RwSignal::new(Vec::<OrganizationTreeNode>::new());
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let expanded_nodes = RwSignal::new(HashSet::<String>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let detail = RwSignal::new(None::<OrganizationNodeDetail>);
    let detail_is_loading = RwSignal::new(false);
    let detail_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_organization_tree(tree, node_types, expanded_nodes, is_loading, load_error);
    });

    view! {
        <AppShell active_route="organization" title="Organization">
            <section class="route-panel organization-page">
                <PageHeader title="Organization Explorer">
                    <Button label="Create Node" href="/organization/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading hierarchy"</h3>
                                <p>"Fetching visible organization nodes."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Organization unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if tree.get().is_empty() {
                        view! {
                            <EmptyState
                                title="No visible organization nodes"
                                message="Create a node or update account scope to populate this explorer."
                            />
                        }
                        .into_any()
                    } else {
                        view! {
                            {organization_tree_view(
                                tree.get(),
                                node_types.get(),
                                expanded_nodes,
                                detail,
                                detail_is_loading,
                                detail_error,
                                0,
                                Vec::new(),
                            )}
                        }
                        .into_any()
                    }
                }}
                <OrganizationDetailSheet
                    detail
                    is_loading=detail_is_loading
                    error=detail_error
                />
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the organization detail page view.
pub fn OrganizationDetailPage() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;
    let detail = RwSignal::new(None::<OrganizationNodeDetail>);
    let is_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_organization_detail(node_id.clone(), detail, is_loading, error);
    });

    view! {
        <AppShell active_route="organization" title="Organization">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/organization">"Organization"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Detail"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>

            <section class="route-panel organization-page organization-detail-page">
                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading detail"</h3>
                                <p>"Fetching organization node details."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Organization detail unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(node_detail) = detail.get() {
                        let edit_href = format!("/organization/{}/edit", node_detail.id);
                        view! {
                            <PageHeader title="Organization Detail">
                                <a class="button" href=edit_href>"Edit Node"</a>
                            </PageHeader>
                            <OrganizationDetailFullContent detail=node_detail/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Organization detail unavailable"
                                message="The selected node could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
