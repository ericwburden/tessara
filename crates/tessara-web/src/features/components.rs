use leptos::prelude::*;

use crate::features::shared::NativePlaceholderRoute;

#[component]
pub fn ComponentsPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="components" title="Components" route="/components" status="Registered" next_step="Component functionality will be filled in with dashboards and datasets."/> }
}

#[component]
pub fn ComponentsDetailPage() -> impl IntoView {
    view! { <NativePlaceholderRoute active_route="components" title="Component Detail" route="/components/:component_ref" status="Registered" next_step="Component detail will be filled in with the component model."/> }
}
