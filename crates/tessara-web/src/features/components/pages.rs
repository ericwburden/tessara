//! Route-level page composition for the Components feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use leptos::prelude::*;

use crate::features::shared::NativePlaceholderRoute;

#[component]
/// Renders the components page view.
pub fn ComponentsPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="components" title="Components" route="/components" status="Registered" next_step="Component functionality will be filled in with dashboards and datasets."/> }
}

#[component]
/// Renders the components detail page view.
pub fn ComponentsDetailPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="components" title="Component Detail" route="/components/:component_ref" status="Registered" next_step="Component detail will be filled in with the component model."/> }
}
