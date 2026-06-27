//! Route definitions for the Forms feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in tessara-web-forms.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{FormRouteParams, require_route_params};
use crate::ui::AppShell;
use tessara_web_forms::{FormDetailContent, FormEditContent, FormNewContent, FormsIndexContent};

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

#[component]
fn FormsPage() -> impl IntoView {
    view! {
        <AppShell active_route="forms" title="Forms">
            <FormsIndexContent/>
        </AppShell>
    }
}

#[component]
fn FormsNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="forms" title="Forms">
            <FormNewContent/>
        </AppShell>
    }
}

#[component]
fn FormsDetailPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;

    view! {
        <AppShell active_route="forms" title="Forms">
            <FormDetailContent form_id/>
        </AppShell>
    }
}

#[component]
fn FormsEditPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;

    view! {
        <AppShell active_route="forms" title="Forms">
            <FormEditContent form_id/>
        </AppShell>
    }
}
