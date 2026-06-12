//! Route-level node-type administration page.

mod actions;
mod state;
mod surface;

use self::surface::AdministrationNodeTypesSurface;
use leptos::prelude::*;

#[component]
pub fn AdministrationNodeTypesPage() -> impl IntoView {
    view! { <AdministrationNodeTypesSurface/> }
}
