use leptos::prelude::*;
use leptos_router::{path, MatchNestedRoutes, SsrMode};
use leptos_router::components::Route;

use crate::features::workflows::{
    WorkflowAssignmentsPage, WorkflowsDetailPage, WorkflowsEditPage, WorkflowsNewPage, WorkflowsPage,
};

const PRIMARY_SSR_MODE: SsrMode = SsrMode::InOrder;

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
