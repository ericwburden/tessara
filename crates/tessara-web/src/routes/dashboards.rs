use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, SsrMode, path};

use crate::features::dashboards::{
    DashboardsDetailPage, DashboardsEditPage, DashboardsNewPage, DashboardsPage,
};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

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
