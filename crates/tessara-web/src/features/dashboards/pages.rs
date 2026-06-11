//! Route-level page composition for the Dashboards feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use leptos::prelude::*;

use crate::features::shared::NativePlaceholderRoute;

#[component]
/// Renders the dashboards page view.
pub fn DashboardsPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Dashboards" route="/dashboards" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}

#[component]
/// Renders the dashboards new page view.
pub fn DashboardsNewPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Create Dashboard" route="/dashboards/new" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}

#[component]
/// Renders the dashboards detail page view.
pub fn DashboardsDetailPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Dashboard Detail" route="/dashboards/:dashboard_id" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}

#[component]
/// Renders the dashboards edit page view.
pub fn DashboardsEditPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Edit Dashboard" route="/dashboards/:dashboard_id/edit" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}
