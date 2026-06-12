//! Route definitions for the Dashboards feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in features::dashboards.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::dashboards::{
    DashboardsDetailPage, DashboardsEditPage, DashboardsNewPage, DashboardsPage,
};

use crate::routes::PRIMARY_SSR_MODE;

pub fn dashboard_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/dashboards")
                view=DashboardsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/new")
                view=DashboardsNewPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/:dashboard_id")
                view=DashboardsDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/dashboards/:dashboard_id/edit")
                view=DashboardsEditPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
