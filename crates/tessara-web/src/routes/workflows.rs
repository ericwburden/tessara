use leptos::prelude::*;
use leptos_router::components::Route;
use leptos_router::{MatchNestedRoutes, path};

use crate::features::workflows::{
    WorkflowAssignmentsPage, WorkflowsDetailPage, WorkflowsEditPage, WorkflowsNewPage,
    WorkflowsPage,
};

use crate::routes::PRIMARY_SSR_MODE;

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
