use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::forms::{FormsDetailPage, FormsEditPage, FormsNewPage, FormsPage};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

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
