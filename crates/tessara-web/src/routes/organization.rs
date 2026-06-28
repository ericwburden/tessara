//! Route definitions for the Organization feature.
//!
//! Keep URL nesting, route parameters, shell wrapping, and route-to-content wiring here;
//! Organization page bodies and data loading belong in tessara-web-organization.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{NodeRouteParams, require_route_params};
use crate::ui::AppShell;
use tessara_web_organization::{
    OrganizationDetailContent, OrganizationIndexContent, OrganizationNodeCreateContent,
    OrganizationNodeEditContent,
};

pub fn organization_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/organization")
                view=OrganizationPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/organization/new")
                view=OrganizationNewPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/organization/:node_id")
                view=OrganizationDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/organization/:node_id/edit")
                view=OrganizationEditPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}

#[component]
fn OrganizationPage() -> impl IntoView {
    view! {
        <AppShell active_route="organization" title="Organization">
            <OrganizationIndexContent/>
        </AppShell>
    }
}

#[component]
fn OrganizationNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="organization" title="Organization">
            <OrganizationNodeCreateContent/>
        </AppShell>
    }
}

#[component]
fn OrganizationDetailPage() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;

    view! {
        <AppShell active_route="organization" title="Organization">
            <OrganizationDetailContent node_id/>
        </AppShell>
    }
}

#[component]
fn OrganizationEditPage() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;

    view! {
        <AppShell active_route="organization" title="Organization">
            <OrganizationNodeEditContent node_id/>
        </AppShell>
    }
}
