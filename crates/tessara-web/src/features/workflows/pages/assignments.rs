//! Assignments support for the Workflows feature.
//!
//! Keep functionality here when it is owned by Workflows and specifically supports the Assignments concern.

use crate::features::workflows::assignments::{
    WorkflowAssignmentsPageState, WorkflowAssignmentsSurface,
    install_workflow_assignments_page_effects,
};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};
use leptos::prelude::*;

#[component]
pub fn WorkflowAssignmentsPage() -> impl IntoView {
    let state = WorkflowAssignmentsPageState::new();
    install_workflow_assignments_page_effects(state);

    view! {
        <AppShell active_route="workflows" title="Workflow Assignments">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/workflows">"Workflows"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Assignments"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>

                <WorkflowAssignmentsSurface state/>
            </div>
        </AppShell>
    }
}
