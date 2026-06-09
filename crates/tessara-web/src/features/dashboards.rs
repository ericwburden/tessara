use leptos::prelude::*;

use crate::ui::components::{AppShell, EmptyState, InfoListTable, InfoRow, PageHeader, StatusBadge};

#[component]
fn NativePlaceholderRoute(
    active_route: &'static str,
    title: &'static str,
    route: &'static str,
    status: &'static str,
    next_step: &'static str,
) -> impl IntoView {
    view! {
        <AppShell active_route title>
            <section class="route-panel">
                <PageHeader title description="This route is served by the native Tessara interface. Functional depth will be added when the underlying product area is ready.">
                    <StatusBadge label=status/>
                </PageHeader>
                <InfoListTable>
                    <InfoRow label="Route" value=route/>
                    <InfoRow label="Rendering" value="Native Leptos SSR component"/>
                    <InfoRow label="Next step" value=next_step/>
                </InfoListTable>
                <EmptyState
                    title="Native route placeholder"
                    message="The route is present in the primary interface; detailed workflows will be filled in as their functionality is restored."
                />
            </section>
        </AppShell>
    }
}

#[component]
pub fn DashboardsPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Dashboards" route="/dashboards" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}

#[component]
pub fn DashboardsNewPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Create Dashboard" route="/dashboards/new" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}

#[component]
pub fn DashboardsDetailPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Dashboard Detail" route="/dashboards/:dashboard_id" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}

#[component]
pub fn DashboardsEditPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="dashboards" title="Edit Dashboard" route="/dashboards/:dashboard_id/edit" status="Registered" next_step="Dashboard functionality will be filled in later."/> }
}
