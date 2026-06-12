//! Route-level node-type administration page entrypoint.

use super::surface::AdministrationNodeTypesSurface;
use leptos::prelude::*;

#[component]
pub fn AdministrationNodeTypesPage() -> impl IntoView {
    view! { <AdministrationNodeTypesSurface/> }
}
