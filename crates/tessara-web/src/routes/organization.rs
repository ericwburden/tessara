//! Route definitions for the Organization feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::organization.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::organization::{
    OrganizationDetailPage, OrganizationEditPage, OrganizationNewPage, OrganizationPage,
};

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the organization routes behavior.
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
