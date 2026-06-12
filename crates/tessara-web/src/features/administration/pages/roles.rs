//! Route-level role-management administration page.

mod state;
mod surface;

use self::surface::AdministrationRolesSurface;
use leptos::prelude::*;

#[component]
pub fn AdministrationRolesPage() -> impl IntoView {
    view! { <AdministrationRolesSurface/> }
}
