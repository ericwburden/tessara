//! Workflow detail route page.
//!
//! Keep route parameter handling and load-state switching here; read-focused detail presentation lives in the Workflows detail module.

use crate::features::workflows::types::WorkflowDefinition;
use crate::features::workflows::{WorkflowDetailContent, load_workflow_detail};
use crate::types::route_params::WorkflowRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    EmptyState, PageHeader, empty_view,
};
use leptos::prelude::*;

#[component]
pub fn WorkflowsDetailPage() -> impl IntoView {
    let params = require_route_params::<WorkflowRouteParams>();
    let workflow_id = params.workflow_id;
    let detail = RwSignal::new(None::<WorkflowDefinition>);
    let is_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflow_detail(workflow_id.clone(), detail, is_loading, error);
    });

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail.get().map(|workflow| {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>{workflow.name}</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                    })
                }}
                {move || {
                    if detail.get().is_none() {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>"Detail"</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                        .into_any()
                    } else {
                        empty_view()
                    }
                }}
            </Breadcrumb>

            <section class="route-panel workflows-page workflow-detail-page">
                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading workflow"</h3>
                                <p>"Fetching workflow details."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Workflow detail unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(workflow) = detail.get() {
                        let assignments_href =
                            format!("/workflows/assignments?workflow_id={}", workflow.id);
                        view! {
                            <PageHeader title="Workflow Detail">
                                <a class="button button--secondary" href=assignments_href>"Manage Assignments"</a>
                            </PageHeader>
                            <WorkflowDetailContent workflow/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Workflow detail unavailable"
                                message="The selected workflow could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
