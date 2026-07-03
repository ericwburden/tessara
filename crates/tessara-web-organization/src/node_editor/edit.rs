//! Organization node edit route page.

use leptos::prelude::*;

use super::OrganizationNodeEditSurface;

#[component]
pub fn OrganizationNodeEditContent(node_id: String) -> impl IntoView {
    view! { <OrganizationNodeEditSurface node_id/> }
}
