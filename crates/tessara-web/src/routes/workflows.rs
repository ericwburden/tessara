//! Route definitions for the Workflows feature.
//!
//! Keep URL nesting, route parameters, and route-to-page wiring here; page composition and data loading belong in tessara-web-workflows.

use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use tessara_web_workflows::{
    WorkflowAssignmentsContent, WorkflowDetailContent, WorkflowEditContent, WorkflowNewContent,
    WorkflowsIndexContent,
};

use crate::routes::PRIMARY_SSR_MODE;
use crate::types::route_params::{WorkflowRouteParams, require_route_params};
use crate::ui::AppShell;

pub fn workflow_routes() -> impl MatchNestedRoutes + Clone {
    view! {
        <>
            <Route
                path=path!("/workflows")
                view=WorkflowsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/workflows/new")
                view=WorkflowsNewPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/workflows/assignments")
                view=WorkflowAssignmentsPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/workflows/:workflow_id")
                view=WorkflowsDetailPage
                ssr=PRIMARY_SSR_MODE
            />
            <Route
                path=path!("/workflows/:workflow_id/edit")
                view=WorkflowsEditPage
                ssr=PRIMARY_SSR_MODE
            />
        </>
    }
}

#[component]
fn WorkflowsPage() -> impl IntoView {
    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowsIndexContent/>
        </AppShell>
    }
}

#[component]
fn WorkflowsNewPage() -> impl IntoView {
    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowNewContent/>
        </AppShell>
    }
}

#[component]
fn WorkflowAssignmentsPage() -> impl IntoView {
    view! {
        <AppShell active_route="workflows" title="Workflow Assignments">
            <WorkflowAssignmentsContent/>
        </AppShell>
    }
}

#[component]
fn WorkflowsDetailPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowDetailContent workflow_id=params.workflow_id/>
        </AppShell>
    }
}

#[component]
fn WorkflowsEditPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <WorkflowEditContent workflow_id=params.workflow_id/>
        </AppShell>
    }
}
