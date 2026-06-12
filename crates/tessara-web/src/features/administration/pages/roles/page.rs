//! Route-level role-management administration page entrypoint.

use super::surface::AdministrationRolesSurface;
use leptos::prelude::*;

#[component]
pub fn AdministrationRolesPage() -> impl IntoView {
    view! { <AdministrationRolesSurface/> }
}
