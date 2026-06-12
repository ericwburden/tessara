//! List view components for the Workflows feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use crate::features::organization::OrganizationNode;
use crate::features::workflows::list::WorkflowsList;
use crate::features::workflows::loaders::{load_workflow_assignment_nodes, load_workflows};
use crate::features::workflows::types::WorkflowSummary;
use crate::features::workflows::workflow_assigned_users_label;
use crate::features::workflows::{
    workflow_available_nodes_label, workflow_description_label, workflow_status_label,
    workflow_version_label,
};
use crate::ui::{AppShell, Button, PageHeader};
use crate::utils::filtering::unique_filter_options;
use crate::utils::text::text_matches;
use leptos::prelude::*;

#[component]
pub fn WorkflowsPage() -> impl IntoView {
    let workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflows(workflows, is_loading, load_error);
        load_workflow_assignment_nodes(organization_nodes);
    });

    let filtered_workflows = move || {
        let query = search.get();
        let selected_status = status_filter.get();
        workflows
            .get()
            .into_iter()
            .filter(|workflow| {
                let version_label = workflow_version_label(workflow);
                let status_label = workflow_status_label(workflow);
                let assigned_to = workflow_assigned_users_label(workflow);
                let description = workflow_description_label(workflow);
                let available_at = workflow_available_nodes_label(&workflow.available_nodes);
                text_matches(
                    &query,
                    &[
                        workflow.name.as_str(),
                        workflow.slug.as_str(),
                        description.as_str(),
                        version_label.as_str(),
                        status_label.as_str(),
                        assigned_to.as_str(),
                        available_at.as_str(),
                    ],
                ) && (selected_status == "all" || selected_status == status_label)
            })
            .collect::<Vec<_>>()
    };

    let status_options = move || {
        unique_filter_options(
            workflows
                .get()
                .iter()
                .map(workflow_status_label)
                .collect::<Vec<_>>(),
        )
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <section class="route-panel workflows-page">
                <PageHeader title="Workflows">
                    <Button label="Create Workflow" href="/workflows/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading workflows"</h3>
                                <p>"Fetching workflow definitions."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Workflows unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <WorkflowsList
                                workflows=filtered_workflows()
                                search=search
                                status_filter=status_filter
                                status_options=status_options()
                                organization_nodes=organization_nodes.get()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}
