//! Workflow assignment UI components.

use crate::features::shared::FilterHeader as SharedFilterHeader;
use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::{
    toggle_workflow_assignment, workflow_assignment_revision_label, workflow_assignment_state,
    workflow_assignment_state_label, workflow_assignment_status_key,
    workflow_assignment_status_label,
};
use crate::ui::{DataTable, DropdownMenu, Timestamp};
use crate::utils::text::nonempty_text;
use icons::{PanelRight, Search, X};
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the workflow assignments list view.
pub(in crate::features::workflows) fn WorkflowAssignmentsList(
    assignments: Vec<WorkflowAssignmentSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    state_filter: RwSignal<String>,
    assignee_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    assignments_signal: RwSignal<Vec<WorkflowAssignmentSummary>>,
    assignments_loading: RwSignal<bool>,
    assignments_error: RwSignal<Option<String>>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let mut table_assignments = assignments.clone();
    table_assignments.sort_by(|left, right| {
        left.workflow_name
            .to_lowercase()
            .cmp(&right.workflow_name.to_lowercase())
            .then(
                left.account_display_name
                    .to_lowercase()
                    .cmp(&right.account_display_name.to_lowercase()),
            )
            .then(left.id.cmp(&right.id))
    });
    let card_assignments = table_assignments.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_assignments.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            total_count.div_ceil(page_size.get()).max(1)
        }
    };
    let current_page = move || page_index.get().min(page_count() - 1);
    let page_start = move || {
        if total_count == 0 {
            0
        } else {
            current_page() * page_size.get()
        }
    };
    let page_end = move || (page_start() + page_size.get()).min(total_count);
    let page_summary = move || {
        if total_count == 0 {
            "No workflow assignments to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} workflow assignments",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };
    let selected_detail = RwSignal::new(None::<WorkflowAssignmentSummary>);
    let close_detail = move |_| selected_detail.set(None);

    view! {
        <div class="forms-list forms-list-responsive-table workflow-assignments-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search assignments"</span>
                        <input
                            type="search"
                            placeholder="Search assignments"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow"</th>
                            <th scope="col">
                                <SharedFilterHeader
                                    label="Assignee"
                                    all_label="All Assignees"
                                    filter=assignee_filter
                                    options=assignee_options
                                    always_searchable=true
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <SharedFilterHeader
                                    label="Work State"
                                    all_label="All States"
                                    filter=state_filter
                                    options=vec!["pending".into(), "draft".into(), "submitted".into()]
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <SharedFilterHeader
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=vec!["active".into(), "inactive".into()]
                                />
                            </th>
                            <th scope="col">"Assigned"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_assignments.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="6">"No Workflow Assignments to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_assignments
                                .iter()
                                .skip(page_start())
                                .take(page_size.get())
                                .cloned()
                                .map(|assignment| {
                                    let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                                    let state_label = workflow_assignment_state_label(&assignment);
                                    let state_key = workflow_assignment_state(&assignment);
                                    let status_key = workflow_assignment_status_key(&assignment);
                                    let status_label = workflow_assignment_status_label(&assignment);
                                    let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                                    let assignment_for_toggle = assignment.clone();
                                    let assignment_for_detail = assignment.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=workflow_href>{assignment.workflow_name.clone()}</a>
                                            </th>
                                            <td>
                                                <span>{assignment.account_display_name}</span>
                                                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                            </td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(state_key)>{state_label}</span>
                                            </td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(status_key)>{status_label}</span>
                                            </td>
                                            <td><Timestamp value=assignment.created_at/></td>
                                            <td class="data-table__cell--center">
                                                <DropdownMenu label=format!("Open actions for {}", assignment.workflow_name)>
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
                        }}
                    </tbody>
                </DataTable>
                <div class="directory-table-pagination" aria-label="Workflow assignments table pagination">
                    <p>{move || page_summary()}</p>
                    <div class="directory-table-pagination__actions">
                        <label class="directory-table-pagination__page-size searchable-data-table__filter searchable-data-table__control">
                            <span>"Rows"</span>
                            <select
                                prop:value=move || page_size.get().to_string()
                                on:change=move |event| {
                                    if let Ok(size) = event_target_value(&event).parse::<usize>() {
                                        page_size.set(size);
                                        page_index.set(0);
                                    }
                                }
                            >
                                <option value="10">"10"</option>
                                <option value="25">"25"</option>
                                <option value="50">"50"</option>
                            </select>
                        </label>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || current_page() == 0
                            on:click=move |_| {
                                page_index.update(|page| *page = page.saturating_sub(1));
                            }
                        >
                            "Previous"
                        </button>
                        <span>{move || format!("Page {} of {}", current_page() + 1, page_count())}</span>
                        <button
                            class="button button--compact button--secondary"
                            type="button"
                            disabled=move || { current_page() + 1 >= page_count() }
                            on:click=move |_| {
                                let last_page = page_count().saturating_sub(1);
                                page_index.update(|page| *page = (*page + 1).min(last_page));
                            }
                        >
                            "Next"
                        </button>
                    </div>
                </div>
            </div>
            <div class="forms-list-mobile-cards workflow-assignment-mobile-cards">
                {move || if card_assignments.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflow Assignments to Display"</p> }.into_any()
                } else {
                    card_assignments
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|assignment| {
                            let workflow_href = format!("/workflows/{}", assignment.workflow_id);
                            let node_href = format!("/organization/{}", assignment.node_id);
                            let state_label = workflow_assignment_state_label(&assignment);
                            let state_key = workflow_assignment_state(&assignment);
                            let status_key = workflow_assignment_status_key(&assignment);
                            let status_label = workflow_assignment_status_label(&assignment);
                            let action_label = if assignment.is_active { "Deactivate" } else { "Activate" };
                            let assignment_for_toggle = assignment.clone();
                            let assignment_for_detail = assignment.clone();
                            view! {
                                <article class="forms-list-mobile-card workflow-assignment-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div class="forms-list-mobile-card__title-row">
                                            <h3><a href=workflow_href>{assignment.workflow_name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Assignee"</dt>
                                            <dd>
                                                <span>{assignment.account_display_name}</span>
                                                <small class="workflow-assignment-step-meta">{assignment.account_email}</small>
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Form"</dt>
                                            <dd>{assignment.form_name}</dd>
                                        </div>
                                        <div>
                                            <dt>"Node"</dt>
                                            <dd><a href=node_href>{assignment.node_name}</a></dd>
                                        </div>
                                        <div>
                                            <dt>"Step"</dt>
                                            <dd>{assignment.workflow_step_title}</dd>
                                        </div>
                                        <div>
                                            <dt>"Work State"</dt>
                                            <dd><span class=status_badge_class(state_key)>{state_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Assigned"</dt>
                                            <dd><Timestamp value=assignment.created_at/></dd>
                                        </div>
                                    </dl>
                                    <div class="workflow-assignment-mobile-card__actions">
                                        <button
                                            class="button button--compact"
                                            type="button"
                                            on:click=move |_| selected_detail.set(Some(assignment_for_detail.clone()))
                                        >
                                            "View Details"
                                        </button>
                                        <button
                                            class="button button--compact"
                                            type="button"
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
                                            {action_label}
                                        </button>
                                    </div>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
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
