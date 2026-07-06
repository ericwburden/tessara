//! Route definitions for the Components feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in tessara-web-components.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{ComponentRouteParams, require_route_params};
use crate::ui::AppShell;
use tessara_web_components::{
    ComponentEditorContent, ComponentPublishContent, ComponentVersionsContent,
    ComponentViewerContent, ComponentsIndexContent,
};

pub fn component_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/components")
                view=ComponentsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/new")
                view=ComponentsCreatePage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/:component_ref/edit")
                view=ComponentsEditPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/:component_ref/publish")
                view=ComponentsPublishPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/:component_ref/view")
                view=ComponentsViewerPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/:component_ref/versions")
                view=ComponentsVersionsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/components/:component_ref")
                view=ComponentsViewerPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}

#[component]
fn ComponentsPage() -> impl IntoView {
    view! {
        <AppShell active_route="components" title="Components">
            <ComponentsIndexContent/>
        </AppShell>
    }
}

#[component]
fn ComponentsCreatePage() -> impl IntoView {
    view! {
        <AppShell active_route="components" title="Create Component">
            <ComponentEditorContent component_ref=None/>
        </AppShell>
    }
}

#[component]
fn ComponentsEditPage() -> impl IntoView {
    let params = require_route_params::<ComponentRouteParams>();
    view! {
        <AppShell active_route="components" title="Edit Component">
            <ComponentEditorContent component_ref=Some(params.component_ref)/>
        </AppShell>
    }
}

#[component]
fn ComponentsPublishPage() -> impl IntoView {
    let params = require_route_params::<ComponentRouteParams>();
    view! {
        <AppShell active_route="components" title="Publish Component">
            <ComponentPublishContent component_ref=params.component_ref/>
        </AppShell>
    }
}

#[component]
fn ComponentsVersionsPage() -> impl IntoView {
    let params = require_route_params::<ComponentRouteParams>();
    view! {
        <AppShell active_route="components" title="Component Versions">
            <ComponentVersionsContent component_ref=params.component_ref/>
        </AppShell>
    }
}

#[component]
fn ComponentsViewerPage() -> impl IntoView {
    let params = require_route_params::<ComponentRouteParams>();
    view! {
        <AppShell active_route="components" title="Component">
            <ComponentViewerContent component_ref=params.component_ref/>
        </AppShell>
    }
}
