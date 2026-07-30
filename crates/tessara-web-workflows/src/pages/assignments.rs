//! Assignments support for the Workflows feature.
//!
//! Keep functionality here when it is owned by Workflows and specifically supports the Assignments concern.

use crate::assignments::{
    WorkflowAssignmentsPageState, WorkflowAssignmentsSurface,
    install_workflow_assignments_page_effects,
};
use leptos::prelude::*;
use tessara_module_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
};

#[component]
pub fn WorkflowAssignmentsContent() -> impl IntoView {
    let state = WorkflowAssignmentsPageState::new();
    install_workflow_assignments_page_effects(state);

    view! {
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
    }
}
