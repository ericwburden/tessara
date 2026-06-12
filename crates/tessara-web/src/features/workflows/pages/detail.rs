//! Detail view components for the Workflows feature.
//!
//! Keep read-focused panels and detail-page presentation here; mutation workflows should live in editor or API modules.

use crate::features::shared::status_badge_class;
use crate::features::workflows::types::WorkflowDefinition;
use crate::features::workflows::{
    WorkflowDetailAssignmentsTable, WorkflowStepsTable, WorkflowVersionsTable,
    active_workflow_definition_version, load_workflow_detail, workflow_available_nodes_label,
    workflow_definition_status_label, workflow_definition_version_label, workflow_source_label,
};
use crate::types::route_params::WorkflowRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    EmptyState, InfoListTable, PageHeader, Timestamp, empty_view,
};
use crate::utils::text::nonempty_text;
use leptos::prelude::*;

#[component]
/// Renders the workflows detail page view.
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

#[component]
/// Renders the workflow detail content view.
fn WorkflowDetailContent(workflow: WorkflowDefinition) -> impl IntoView {
    let steps_expanded = RwSignal::new(false);
    let revisions_expanded = RwSignal::new(false);
    let assignments_expanded = RwSignal::new(false);
    let active_version = active_workflow_definition_version(&workflow).cloned();
    let active_status = active_version
        .as_ref()
        .map(|version| version.status.clone())
        .unwrap_or_else(|| "none".to_string());
    let active_version_label = workflow_definition_version_label(active_version.as_ref());
    let active_status_label = workflow_definition_status_label(active_version.as_ref());
    let active_step_count = active_version
        .as_ref()
        .map(|version| version.step_count.to_string())
        .unwrap_or_else(|| "-".to_string());
    let steps_toggle_count = active_step_count.clone();
    let published_at = active_version
        .as_ref()
        .and_then(|version| version.published_at.clone());
    let workflow_id = workflow.id.clone();
    let workflow_name = workflow.name.clone();
    let workflow_slug = workflow.slug.clone();
    let workflow_description = nonempty_text(Some(workflow.description.as_str()), "No description");
    let workflow_available_at = workflow_available_nodes_label(&workflow.available_nodes);
    let workflow_source = workflow_source_label(&workflow.source)
        .unwrap_or("Authored")
        .to_string();
    let revision_count = workflow.versions.len().to_string();
    let assignment_count = workflow.assignments.len().to_string();
    let revisions_toggle_count = revision_count.clone();
    let assignments_toggle_count = assignment_count.clone();
    let steps = active_version
        .as_ref()
        .map(|version| version.steps.clone())
        .unwrap_or_default();
    let versions = workflow.versions.clone();
    let assignments = workflow.assignments.clone();

    view! {
        <div class="organization-detail-content workflow-detail-content">
            <header class="organization-detail-content__header">
                <p>"Workflow Detail"</p>
                <h2>{workflow_name}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Details"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Slug"</th>
                            <td>{workflow_slug}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Description"</th>
                            <td>{workflow_description}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Available At"</th>
                            <td>{workflow_available_at}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Source"</th>
                            <td>{workflow_source}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Revisions"</th>
                            <td>{revision_count}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Assignments"</th>
                            <td>{assignment_count}</td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card">
                    <h3>"Active Revision"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Revision"</th>
                            <td>{active_version_label}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Status"</th>
                            <td><span class=status_badge_class(&active_status)>{active_status_label}</span></td>
                        </tr>
                        <tr>
                            <th scope="row">"Steps"</th>
                            <td>{active_step_count}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Published"</th>
                            <td>
                                {published_at
                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                            </td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Steps"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || steps_expanded.get().to_string()
                            on:click=move |_| steps_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if steps_expanded.get() {
                                    "Hide Steps".to_string()
                                } else {
                                    format!("Show {steps_toggle_count} Steps")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if steps_expanded.get() {
                            view! { <WorkflowStepsTable steps=steps.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Revisions"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || revisions_expanded.get().to_string()
                            on:click=move |_| revisions_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if revisions_expanded.get() {
                                    "Hide Revisions".to_string()
                                } else {
                                    format!("Show {revisions_toggle_count} Revisions")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if revisions_expanded.get() {
                            view! { <WorkflowVersionsTable workflow_id=workflow_id.clone() versions=versions.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card workflow-detail-assignments-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Assignments"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || assignments_expanded.get().to_string()
                            on:click=move |_| assignments_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if assignments_expanded.get() {
                                    "Hide Assignments".to_string()
                                } else {
                                    format!("Show {assignments_toggle_count} Assignments")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if assignments_expanded.get() {
                            view! { <WorkflowDetailAssignmentsTable assignments=assignments.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>
            </div>
        </div>
    }
}
