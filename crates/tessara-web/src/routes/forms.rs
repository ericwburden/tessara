//! Owns the routes::forms module behavior.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::forms::{FormsDetailPage, FormsEditPage, FormsNewPage, FormsPage};

use crate::routes::PRIMARY_SSR_MODE;

/// Handles the form routes behavior.
pub fn form_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route path=path!("/forms") view=FormsPage ssr=PRIMARY_SSR_MODE/>
            <Route path=path!("/forms/new") view=FormsNewPage ssr=PRIMARY_SSR_MODE/>
            <Route
                path=path!("/forms/:form_id")
                view=FormsDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/forms/:form_id/edit")
                view=FormsEditPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}
