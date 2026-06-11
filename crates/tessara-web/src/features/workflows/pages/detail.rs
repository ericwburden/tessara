//! Owns the features::workflows::pages::detail module behavior.

use crate::features::organization::toggle_workflow_assignment;
use crate::features::shared::status_badge_class;
use crate::features::workflows::api::load_workflow_detail;
use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::types::{
    WorkflowDefinition, WorkflowStepSummary, WorkflowVersionSummary,
};
use crate::features::workflows::{
    active_workflow_definition_version, workflow_assignment_revision_label,
    workflow_assignment_state, workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label, workflow_available_nodes_label,
    workflow_definition_status_label, workflow_definition_version_label,
    workflow_revision_label_from_option, workflow_source_label,
};
use crate::types::route_params::WorkflowRouteParams;
use crate::types::route_params::require_route_params;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DataTable, DropdownMenu, EmptyState, InfoListTable, PageHeader, Timestamp, empty_view,
};
use crate::utils::text::{nonempty_text, sentence_label};
use icons::{PanelRight, Pencil, X};
use leptos::portal::Portal;
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

#[component]
/// Renders the workflow steps table view.
fn WorkflowStepsTable(steps: Vec<WorkflowStepSummary>) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Step"</th>
                    <th scope="col">"Form"</th>
                    <th scope="col">"Form Version"</th>
                </tr>
            </thead>
            <tbody>
                {if steps.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Workflow Steps to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    steps
                        .into_iter()
                        .map(|step| {
                            let form_href = format!("/forms/{}", step.form_id);
                            let step_title = nonempty_text(Some(&step.title), "Untitled step");
                            view! {
                                <tr>
                                    <th scope="row">{step_title}</th>
                                    <td><a class="data-table__primary-link" href=form_href>{step.form_name}</a></td>
                                    <td>{nonempty_text(step.form_version_label.as_deref(), "-")}</td>
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </DataTable>
    }
}

#[component]
/// Renders the workflow versions table view.
fn WorkflowVersionsTable(
    workflow_id: String,
    versions: Vec<WorkflowVersionSummary>,
) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Revision"</th>
                    <th scope="col">"Status"</th>
                    <th scope="col">"Published"</th>
                    <th class="data-table__cell--center" scope="col">"Steps"</th>
                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                </tr>
            </thead>
            <tbody>
                {if versions.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="5">"No Revisions to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    versions
                        .into_iter()
                        .map(|version| {
                            let status = version.status.clone();
                            let published_at = version.published_at.clone();
                            let version_label = workflow_revision_label_from_option(version.workflow_revision_label);
                            let edit_href = format!("/workflows/{}/edit?version_id={}", workflow_id, version.id);
                            let edit_title = format!("Edit {} workflow revision", sentence_label(&status));
                            view! {
                                <tr>
                                    <th scope="row">{version_label}</th>
                                    <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                    <td>
                                        {published_at
                                            .map(|value| view! { <Timestamp value/> }.into_any())
                                            .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                    </td>
                                    <td class="data-table__cell--center">{version.step_count.to_string()}</td>
                                    <td class="data-table__cell--center">
                                        <a class="data-table__action" href=edit_href aria-label=edit_title.clone() title=edit_title>
                                            <Pencil class="icon-button__icon"/>
                                        </a>
                                    </td>
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </DataTable>
    }
}

#[component]
/// Renders the workflow detail assignments table view.
fn WorkflowDetailAssignmentsTable(assignments: Vec<WorkflowAssignmentSummary>) -> impl IntoView {
    let assignments_signal = RwSignal::new(assignments);
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let assignments_loading = RwSignal::new(false);
    let assignments_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Assignee"</th>
                    <th class="data-table__cell--center" scope="col">"Work State"</th>
                    <th class="data-table__cell--center" scope="col">"Status"</th>
                    <th scope="col">"Assigned"</th>
                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let assignments = assignments_signal.get();
                    if assignments.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Assignments to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        assignments
                            .into_iter()
                            .map(|assignment| {
                                let state_key = workflow_assignment_state(&assignment);
                                let state_label = workflow_assignment_state_label(&assignment);
                                let status_key = workflow_assignment_status_key(&assignment);
                                let status_label = workflow_assignment_status_label(&assignment);
                                let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                                let assignment_for_detail = assignment.clone();
                                let assignment_for_toggle = assignment.clone();
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <span>{assignment.account_display_name.clone()}</span>
                                            <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                        </th>
                                        <td class="data-table__cell--center">
                                            <span class=status_badge_class(state_key)>{state_label}</span>
                                        </td>
                                        <td class="data-table__cell--center">
                                            <span class=status_badge_class(status_key)>{status_label}</span>
                                        </td>
                                        <td><Timestamp value=assignment.created_at/></td>
                                        <td class="data-table__cell--center">
                                            <DropdownMenu label=format!("Open actions for {}", assignment.account_display_name)>
                                                <button
                                                    class="dropdown-menu__item"
                                                    type="button"
                                                    role="menuitem"
                                                    on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                                >
                                                    <PanelRight class="dropdown-menu__item-icon"/>
                                                    <span>"View Details"</span>
                                                </button>
                                                <button
                                                    class="dropdown-menu__item"
                                                    type="button"
                                                    role="menuitem"
                                                    on:click=move |_| {
                                                        toggle_workflow_assignment(
                                                            assignment_for_toggle.clone(),
                                                            assignments_signal,
                                                            assignments_loading,
                                                            assignments_error,
                                                            message,
                                                        );
                                                    }
                                                >
                                                    <X class="dropdown-menu__item-icon"/>
                                                    <span>{action_label}</span>
                                                </button>
                                            </DropdownMenu>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </tbody>
        </DataTable>
        {move || selected_detail.get().map(|assignment| {
            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
            let node_href = format!("/organization/{}", assignment.node_id);
            let state_key = workflow_assignment_state(&assignment);
            let state_label = workflow_assignment_state_label(&assignment);
            let status_key = workflow_assignment_status_key(&assignment);
            let status_label = workflow_assignment_status_label(&assignment);

            view! {
                <Portal>
                    <section class="sheet-overlay workflow-assignment-detail-overlay" aria-label="Workflow assignment detail">
                        <button class="sheet-overlay__scrim" type="button" aria-label="Close assignment details" on:click=close_detail></button>
                        <aside class="sheet-panel blurred-surface workflow-assignment-detail-sheet" role="dialog" aria-modal="true" aria-label="Workflow assignment details">
                            <div class="sheet-panel__actions">
                                <button class="icon-button sheet-panel__close" type="button" aria-label="Close assignment details" title="Close assignment details" on:click=close_detail>
                                    <X class="icon-button__icon"/>
                                </button>
                            </div>
                            <header class="sheet-panel__header">
                                <p>"Assignment Detail"</p>
                                <h2>{assignment.workflow_name.clone()}</h2>
                            </header>
                            <section class="sheet-panel__section">
                                <h3>"Workflow"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Workflow"</th>
                                            <td><a class="data-table__primary-link" href=workflow_href.clone()>{assignment.workflow_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Revision"</th>
                                            <td>{workflow_assignment_revision_label(assignment.workflow_version_label.as_deref())}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Step"</th>
                                            <td>{assignment.workflow_step_title.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form"</th>
                                            <td>{assignment.form_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Form Version"</th>
                                            <td>{nonempty_text(assignment.form_version_label.as_deref(), "-")}</td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                            <section class="sheet-panel__section">
                                <h3>"Assignment"</h3>
                                <table class="info-list-table">
                                    <tbody>
                                        <tr>
                                            <th scope="row">"Node"</th>
                                            <td><a class="data-table__primary-link" href=node_href.clone()>{assignment.node_name.clone()}</a></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assignee"</th>
                                            <td>{assignment.account_display_name.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Email"</th>
                                            <td>{assignment.account_email.clone()}</td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Work State"</th>
                                            <td><span class=status_badge_class(state_key)>{state_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Status"</th>
                                            <td><span class=status_badge_class(status_key)>{status_label}</span></td>
                                        </tr>
                                        <tr>
                                            <th scope="row">"Assigned"</th>
                                            <td><Timestamp value=assignment.created_at.clone()/></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </section>
                        </aside>
                    </section>
                </Portal>
            }
        })}
    }
}
