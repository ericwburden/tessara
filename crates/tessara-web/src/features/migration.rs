use leptos::prelude::*;
use leptos_router::{LazyRoute, lazy_route};

use crate::app::transitional::{TransitionalPage, extract_app_root, render_transitional_route};

#[component]
pub fn MigrationPage() -> impl IntoView {
    render_transitional_route(TransitionalPage {
        title: "Tessara Migration",
        description: "Tessara migration workbench.",
        body_html: extract_app_root(crate::migration_application_shell_html()),
        page_key: "migration",
        active_route: "migration",
        record_id: None,
    })
}

pub struct MigrationLazyRoute;

#[lazy_route]
impl LazyRoute for MigrationLazyRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;

        view! { <MigrationPage/> }.into_any()
    }
}
