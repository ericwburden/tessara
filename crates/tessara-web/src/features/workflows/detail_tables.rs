//! Workflow detail tables and assignment panels.

use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::types::{WorkflowStepSummary, WorkflowVersionSummary};
use crate::features::workflows::{
    toggle_workflow_assignment, workflow_assignment_revision_label, workflow_assignment_state,
    workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label, workflow_revision_label_from_option,
};
use crate::ui::{DataTable, DropdownMenu, Timestamp};
use crate::utils::text::{nonempty_text, sentence_label};
use icons::{PanelRight, Pencil, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the workflow steps table view.
pub(in crate::features::workflows) fn WorkflowStepsTable(
    steps: Vec<WorkflowStepSummary>,
) -> impl IntoView {
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
pub(in crate::features::workflows) fn WorkflowVersionsTable(
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
pub(in crate::features::workflows) fn WorkflowDetailAssignmentsTable(
    assignments: Vec<WorkflowAssignmentSummary>,
) -> impl IntoView {
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
