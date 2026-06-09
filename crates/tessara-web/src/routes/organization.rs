use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, SsrMode, path};

use crate::features::organization::{
    OrganizationDetailPage, OrganizationEditPage, OrganizationNewPage, OrganizationPage,
};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

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
