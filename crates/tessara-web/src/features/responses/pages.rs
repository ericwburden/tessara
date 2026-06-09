//! Response route components.
//!
//! The response implementation still shares DTOs and helpers with the broader
//! native module. Keeping the public route components here gives the next
//! cleanup pass a stable module boundary for moving the remaining response
//! internals without changing app routing.

use crate::features::shared::FilterHeader as SharedFilterHeader;
use crate::features::shared::*;
use crate::features::workflows::submission::*;
use crate::features::workflows::*;
use crate::types::route_params::{SubmissionRouteParams, require_route_params};
use crate::ui::components::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    DataTable, DropdownMenu, InfoListTable, InfoRow, PageHeader, SearchableDataTable, Timestamp,
};
use crate::ui::empty_view;
use crate::utils::text::{nonempty_text, text_matches};
use leptos::portal::Portal;
use std::collections::HashMap;

use icons::{PanelRight, Pencil, Search};
use leptos::prelude::*;

#[component]
pub fn ResponsesPage() -> impl IntoView {
    view! { <ResponsesPageContent/> }
}

#[component]
pub fn ResponsesNewPage() -> impl IntoView {
    view! { <ResponsesNewPageContent/> }
}

#[component]
pub fn ResponsesDetailPage() -> impl IntoView {
    view! { <ResponsesDetailPageContent/> }
}

#[component]
pub fn ResponsesEditPage() -> impl IntoView {
    view! { <ResponsesEditPageContent/> }
}

#[allow(non_snake_case)]
pub(super) fn ResponsesPageContent() -> impl IntoView {
    let submissions = RwSignal::new(Vec::<SubmissionSummary>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let assignee_filter = RwSignal::new("all".to_string());
    let status_filter = RwSignal::new("all".to_string());

    Effect::new(move |_| {
        load_submissions(submissions, is_loading, load_error);
    });

    let filtered_submissions = move || {
        let query = search.get();
        let assignee = assignee_filter.get();
        let status = status_filter.get();
        submissions
            .get()
            .into_iter()
            .filter(|submission| {
                let status_key = submission_status_key(submission);
                let assignee_label = submission_assignee_label(submission);
                let matches_assignee = assignee == "all" || assignee_label == assignee;
                (status == "all" || status_key == status)
                    && matches_assignee
                    && text_matches(
                        &query,
                        &[
                            submission.form_name.as_str(),
                            submission.workflow_name.as_deref().unwrap_or_default(),
                            submission
                                .current_workflow_step_title
                                .as_deref()
                                .unwrap_or_default(),
                            submission
                                .next_workflow_step_title
                                .as_deref()
                                .unwrap_or_default(),
                            submission
                                .next_workflow_step_form_name
                                .as_deref()
                                .unwrap_or_default(),
                            submission.node_name.as_str(),
                            submission
                                .assigned_to_display_name
                                .as_deref()
                                .unwrap_or_default(),
                            submission.status.as_str(),
                        ],
                    )
            })
            .collect::<Vec<_>>()
    };
    let status_options = move || {
        unique_filter_options(
            submissions
                .get()
                .iter()
                .map(submission_status_key)
                .collect::<Vec<_>>(),
        )
    };
    let assignee_options = move || {
        unique_filter_options(
            submissions
                .get()
                .iter()
                .map(submission_assignee_label)
                .collect::<Vec<_>>(),
        )
    };

    view! {
        <AppShell active_route="responses" title="Responses">
            <section class="route-panel responses-page">
                <PageHeader title="Responses"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading responses"</h3>
                                <p>"Fetching visible response records."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Responses unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <ResponsesList
                                submissions=filtered_submissions()
                                search
                                assignee_filter
                                status_filter
                                assignee_options=assignee_options()
                                status_options=status_options()
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
fn ResponsesList(
    submissions: Vec<SubmissionSummary>,
    search: RwSignal<String>,
    assignee_filter: RwSignal<String>,
    status_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    status_options: Vec<String>,
) -> impl IntoView {
    let mut table_submissions = submissions.clone();
    table_submissions.sort_by(|left, right| {
        right
            .last_modified_at
            .cmp(&left.last_modified_at)
            .then(
                left.form_name
                    .to_lowercase()
                    .cmp(&right.form_name.to_lowercase()),
            )
            .then(left.id.cmp(&right.id))
    });
    let card_submissions = table_submissions.clone();
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = table_submissions.len();
    let page_count = move || {
        if total_count == 0 {
            1
        } else {
            ((total_count + page_size.get() - 1) / page_size.get()).max(1)
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
            "No responses to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} responses",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };

    view! {
        <div class="forms-list forms-list-responsive-table responses-list">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search responses"</span>
                        <input
                            type="search"
                            placeholder="Search responses"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Response"</th>
                            <th scope="col">"Workflow"</th>
                            <th scope="col">"Node"</th>
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
                                    label="Status"
                                    all_label="All Statuses"
                                    filter=status_filter
                                    options=status_options
                                />
                            </th>
                            <th scope="col">"Last Updated"</th>
                            <th class="data-table__cell--center" scope="col">"Actions"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || if table_submissions.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="7">"No Responses to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_submissions
                                .iter()
                                .skip(page_start())
                                .take(page_size.get())
                                .cloned()
                                .map(|submission| {
                                    let detail_href = format!("/responses/{}", submission.id);
                                    let edit_href = format!("/responses/{}/edit", submission.id);
                                    let form_href = format!("/forms/{}", submission.form_id);
                                    let node_href = format!("/organization/{}", submission.node_id);
                                    let form_name = submission.form_name.clone();
                                    let form_version_label =
                                        format!("Form Version {}", submission.version_label);
                                    let status_key = submission_status_key(&submission);
                                    let status_label = submission_status_label(&submission);
                                    let workflow_label = submission_workflow_label(&submission);
                                    let step_label = submission_step_label(&submission);
                                    let progress_label = submission_progress_label(&submission);
                                    let assignee = submission_assignee_label(&submission);
                                    let is_draft = status_key == "draft";
                                    let detail_href_for_click = detail_href.clone();
                                    let edit_href_for_click = edit_href.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=detail_href.clone()>{form_name.clone()}</a>
                                                <small class="workflow-assignment-step-meta">
                                                    <a href=form_href>{form_version_label}</a>
                                                </small>
                                            </th>
                                            <td>
                                                <span>{workflow_label}</span>
                                                <small class="workflow-assignment-step-meta">{step_label}</small>
                                                <small class="workflow-assignment-step-meta">{progress_label}</small>
                                            </td>
                                            <td><a href=node_href>{submission.node_name}</a></td>
                                            <td>{assignee}</td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(&status_key)>{status_label}</span>
                                            </td>
                                            <td><Timestamp value=submission.last_modified_at/></td>
                                            <td class="data-table__cell--center">
                                                <DropdownMenu label=format!("Open actions for {form_name}")>
                                                    <button
                                                        class="dropdown-menu__item"
                                                        type="button"
                                                        role="menuitem"
                                                        on:click=move |_| {
                                                            #[cfg(feature = "hydrate")]
                                                            navigate_to_href(&detail_href_for_click);
                                                            #[cfg(not(feature = "hydrate"))]
                                                            let _ = &detail_href_for_click;
                                                        }
                                                    >
                                                        <PanelRight class="dropdown-menu__item-icon"/>
                                                        <span>"View Details"</span>
                                                    </button>
                                                    {if is_draft {
                                                        view! {
                                                            <button
                                                                class="dropdown-menu__item"
                                                                type="button"
                                                                role="menuitem"
                                                                on:click=move |_| {
                                                                    #[cfg(feature = "hydrate")]
                                                                    navigate_to_href(&edit_href_for_click);
                                                                    #[cfg(not(feature = "hydrate"))]
                                                                    let _ = &edit_href_for_click;
                                                                }
                                                            >
                                                                <Pencil class="dropdown-menu__item-icon"/>
                                                                <span>"Edit Draft"</span>
                                                            </button>
                                                        }
                                                        .into_any()
                                                    } else {
                                                        empty_view()
                                                    }}
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
                <div class="directory-table-pagination" aria-label="Responses table pagination">
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
            <div class="forms-list-mobile-cards responses-mobile-cards">
                {move || if card_submissions.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Responses to Display"</p> }.into_any()
                } else {
                    card_submissions
                        .iter()
                        .skip(page_start())
                        .take(page_size.get())
                        .cloned()
                        .map(|submission| {
                            let detail_href = format!("/responses/{}", submission.id);
                            let edit_href = format!("/responses/{}/edit", submission.id);
                            let node_href = format!("/organization/{}", submission.node_id);
                            let status_key = submission_status_key(&submission);
                            let status_label = submission_status_label(&submission);
                            let workflow_label = submission_workflow_label(&submission);
                            let step_label = submission_step_label(&submission);
                            let progress_label = submission_progress_label(&submission);
                            let assignee = submission_assignee_label(&submission);
                            let is_draft = status_key == "draft";
                            view! {
                                <article class="forms-list-mobile-card response-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div class="forms-list-mobile-card__title-row">
                                            <h3><a href=detail_href.clone()>{submission.form_name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Form Version"</dt>
                                            <dd>{submission.version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Workflow"</dt>
                                            <dd>{workflow_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Step"</dt>
                                            <dd>{step_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Progress"</dt>
                                            <dd>{progress_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Node"</dt>
                                            <dd><a href=node_href>{submission.node_name}</a></dd>
                                        </div>
                                        <div>
                                            <dt>"Assignee"</dt>
                                            <dd>{assignee}</dd>
                                        </div>
                                        <div>
                                            <dt>"Last Updated"</dt>
                                            <dd><Timestamp value=submission.last_modified_at/></dd>
                                        </div>
                                        {if let Some(submitted_at) = submission.submitted_at {
                                            view! {
                                                <div>
                                                    <dt>"Submitted"</dt>
                                                    <dd><Timestamp value=submitted_at/></dd>
                                                </div>
                                            }
                                            .into_any()
                                        } else {
                                            empty_view()
                                        }}
                                    </dl>
                                    <div class="response-mobile-card__actions">
                                        <a class="button button--compact button--quiet" href=detail_href>"View Details"</a>
                                        {if is_draft {
                                            view! { <a class="button button--compact button--quiet" href=edit_href>"Edit Draft"</a> }.into_any()
                                        } else {
                                            empty_view()
                                        }}
                                    </div>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}

#[component]
pub(super) fn ResponsesNewPageContent() -> impl IntoView {
    let options = RwSignal::new(None::<AssignmentResponseStartOptions>);
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let selected_assignment_index = RwSignal::new(String::new());

    #[cfg(feature = "hydrate")]
    let requested_workflow_assignment_id = current_search_param("workflowAssignmentId")
        .or_else(|| current_search_param("workflow_assignment_id"));
    #[cfg(not(feature = "hydrate"))]
    let requested_workflow_assignment_id = None::<String>;

    let requested_workflow_assignment_id_for_effect = requested_workflow_assignment_id.clone();
    let requested_workflow_assignment_id_for_view = requested_workflow_assignment_id.clone();
    #[cfg(feature = "hydrate")]
    let delegate_account_id_for_effect = current_search_param("delegateAccountId")
        .or_else(|| current_search_param("delegate_account_id"));
    #[cfg(not(feature = "hydrate"))]
    let delegate_account_id_for_effect = None::<String>;

    Effect::new(move |_| {
        if let Some(workflow_assignment_id) = requested_workflow_assignment_id_for_effect.clone() {
            is_loading.set(false);
            start_workflow_assignment_response(workflow_assignment_id, is_saving, message);
        } else {
            load_response_start_options(
                options,
                is_loading,
                message,
                delegate_account_id_for_effect.clone(),
            );
        }
    });

    view! {
        <AppShell active_route="responses" title="Start Response">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/responses">"Responses"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Start Response"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <section class="route-panel responses-page">
                    <PageHeader title="Start Response"/>

                    {move || {
                        if requested_workflow_assignment_id_for_view.is_some() && is_saving.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Starting assigned response"</h3>
                                    <p>"Creating a draft from the selected workflow assignment."</p>
                                </section>
                            }
                            .into_any()
                        } else if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading start options"</h3>
                                    <p>"Fetching available response contexts."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(error) = message.get().filter(|_| options.get().is_none()) {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response start unavailable"</h3>
                                    <p>{error}</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(loaded_options) = options.get() {
                            view! {
                                <form
                                    class="native-form response-start-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        if !response_start_can_submit(
                                            options,
                                            is_loading,
                                            is_saving,
                                            selected_assignment_index,
                                        ) {
                                            message.set(Some("Select assigned workflow work before starting a draft.".into()));
                                            return;
                                        }

                                        if let Some(assignment) = response_selected_assignment(options, selected_assignment_index) {
                                            start_workflow_assignment_response(
                                                assignment.workflow_assignment_id,
                                                is_saving,
                                                message,
                                            );
                                        }
                                    }
                                >
                                    <ResponseAssignmentStartFields
                                        assignments=loaded_options.assignments
                                        selected_assignment_index
                                    />

                                    {move || {
                                        message
                                            .get()
                                            .map(|message| {
                                                let class = if message.to_lowercase().contains("failed")
                                                    || message.to_lowercase().contains("unable")
                                                    || message.to_lowercase().contains("select")
                                                {
                                                    "form-message is-error"
                                                } else {
                                                    "form-message"
                                                };
                                                view! { <p class=class role="status">{message}</p> }
                                            })
                                    }}

                                    <div class="form-actions">
                                        <a class="button button--secondary" href="/responses">"Cancel"</a>
                                        <button
                                            class="button"
                                            type="submit"
                                            disabled=move || {
                                                !response_start_can_submit(
                                                    options,
                                                    is_loading,
                                                    is_saving,
                                                    selected_assignment_index,
                                                )
                                            }
                                        >
                                            {move || if is_saving.get() { "Starting..." } else { "Start Draft" }}
                                        </button>
                                    </div>
                                </form>
                            }
                            .into_any()
                        } else {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response start unavailable"</h3>
                                    <p>"Response start options could not be loaded."</p>
                                </section>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}

#[component]
fn ResponseAssignmentStartFields(
    assignments: Vec<AssignmentResponseStartOption>,
    selected_assignment_index: RwSignal<String>,
) -> impl IntoView {
    let has_assignments = !assignments.is_empty();
    let assignments_for_summary = assignments.clone();
    let selected_summary = move || {
        let index = selected_assignment_index.get().parse::<usize>().ok()?;
        assignments_for_summary.get(index).cloned()
    };

    view! {
        <div class="form-grid">
            <label class="form-field wide-field">
                <span>"Assigned Work"</span>
                <select
                    prop:value=move || selected_assignment_index.get()
                    disabled=!has_assignments
                    on:change=move |event| selected_assignment_index.set(event_target_value(&event))
                >
                    <option value="">"Select assigned response"</option>
                    {assignments
                        .into_iter()
                        .enumerate()
                        .map(|(index, assignment)| {
                            let workflow_revision = workflow_revision_label_from_option(
                                assignment.workflow_version_label.clone(),
                            );
                            let assignee = nonempty_text(
                                Some(assignment.account_display_name.as_str()),
                                "Assigned response",
                            );
                            view! {
                                <option value=index.to_string()>
                                    {format!(
                                        "{} - {} (Revision {}) at {} - {}",
                                        assignment.workflow_name,
                                        assignment.workflow_step_title,
                                        workflow_revision,
                                        assignment.node_name,
                                        assignee,
                                    )}
                                </option>
                            }
                        })
                        .collect_view()}
                </select>
            </label>
        </div>
        {move || {
            if !has_assignments {
                view! {
                    <section class="organization-state" aria-live="polite">
                        <h3>"No assigned responses"</h3>
                        <p>"There is no pending workflow work available for this response context."</p>
                    </section>
                }
                .into_any()
            } else if let Some(assignment) = selected_summary() {
                let workflow_revision =
                    workflow_revision_label_from_option(assignment.workflow_version_label);
                let form_version =
                    nonempty_text(assignment.form_version_label.as_deref(), "-");
                view! {
                    <section class="organization-state response-start-summary" aria-live="polite">
                        <h3>{assignment.workflow_name}</h3>
                        <p>{format!(
                            "Revision {} - Step {} of {}: {}",
                            workflow_revision,
                            assignment.workflow_step_position + 1,
                            assignment.workflow_step_count,
                            assignment.workflow_step_title,
                        )}</p>
                        <p>{format!(
                            "{} - Form Version {} at {}",
                            assignment.form_name,
                            form_version,
                            assignment.node_name,
                        )}</p>
                        <p>{nonempty_text(Some(assignment.account_display_name.as_str()), "Assigned response")}</p>
                    </section>
                }
                .into_any()
            } else {
                empty_view()
            }
        }}
    }
}

#[component]
pub(super) fn ResponsesDetailPageContent() -> impl IntoView {
    let params = require_route_params::<SubmissionRouteParams>();
    let submission_id = params.submission_id;
    let detail = RwSignal::new(None::<SubmissionDetail>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_submission_detail(submission_id.clone(), detail, is_loading, load_error);
    });

    view! {
        <AppShell active_route="responses" title="Response Detail">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/responses">"Responses"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Response Detail"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <section class="route-panel responses-page">
                    <PageHeader title="Response Detail"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading response"</h3>
                                    <p>"Fetching response values and audit history."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(message) = load_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response unavailable"</h3>
                                    <p>{message}</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(detail) = detail.get() {
                            view! { <ResponseDetailContent detail/> }.into_any()
                        } else {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response unavailable"</h3>
                                    <p>"The selected response could not be loaded."</p>
                                </section>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}

#[component]
pub(super) fn ResponsesEditPageContent() -> impl IntoView {
    let params = require_route_params::<SubmissionRouteParams>();
    let submission_id = params.submission_id;
    let detail = RwSignal::new(None::<SubmissionDetail>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let text_values = RwSignal::new(HashMap::<String, String>::new());
    let boolean_values = RwSignal::new(HashMap::<String, bool>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_submission_edit_context(
            submission_id.clone(),
            detail,
            rendered_form,
            text_values,
            boolean_values,
            is_loading,
            load_error,
        );
    });

    view! {
        <AppShell active_route="responses" title="Edit Response">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/responses">"Responses"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit Response"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <section class="route-panel responses-page">
                    <PageHeader title="Edit Response"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading response form"</h3>
                                    <p>"Fetching response values and form fields."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(message) = load_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response unavailable"</h3>
                                    <p>{message}</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(detail) = detail.get() {
                            if detail.status != "draft" {
                                let detail_href = format!("/responses/{}", detail.id);
                                view! {
                                    <section class="organization-state" aria-live="polite">
                                        <h3>"Submitted response"</h3>
                                        <p>"This response has been submitted and is read-only."</p>
                                        <a class="button button--secondary" href=detail_href>"Back to Detail"</a>
                                    </section>
                                }
                                .into_any()
                            } else if let Some(rendered_form) = rendered_form.get() {
                                view! {
                                    <ResponseEditForm
                                        detail
                                        rendered_form
                                        text_values
                                        boolean_values
                                        is_saving
                                        message
                                    />
                                }
                                .into_any()
                            } else {
                                view! {
                                    <section class="organization-state is-error" role="alert">
                                        <h3>"Response form unavailable"</h3>
                                        <p>"The selected response form could not be loaded."</p>
                                    </section>
                                }
                                .into_any()
                            }
                        } else {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response unavailable"</h3>
                                    <p>"The selected response could not be loaded."</p>
                                </section>
                            }
                            .into_any()
                        }
                    }}
                </section>
            </div>
        </AppShell>
    }
}

#[component]
fn ResponseDetailContent(detail: SubmissionDetail) -> impl IntoView {
    let values_expanded = RwSignal::new(false);
    let audit_expanded = RwSignal::new(false);
    let status_key = detail.status.trim().to_lowercase();
    let status_label = metadata_label(&detail.status);
    let edit_href = format!("/responses/{}/edit", detail.id);
    let node_href = format!("/organization/{}", detail.node_id);
    let form_href = format!("/forms/{}", detail.form_id);
    let submitted_at = detail.submitted_at.clone();
    let runtime = detail.runtime.clone();
    let values = detail.values.clone();
    let audit_events = detail.audit_events.clone();
    let values_count = values.len().to_string();
    let audit_count = audit_events.len().to_string();
    let is_draft = status_key == "draft";

    view! {
        <div class="organization-detail-content response-detail-content">
            <header class="organization-detail-content__header">
                <p>"Response Detail"</p>
                <h2>{detail.form_name.clone()}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Summary"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Form"</th>
                            <td><a href=form_href>{detail.form_name}</a></td>
                        </tr>
                        <tr>
                            <th scope="row">"Form Version"</th>
                            <td>{detail.version_label}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Node"</th>
                            <td><a href=node_href>{detail.node_name}</a></td>
                        </tr>
                        <tr>
                            <th scope="row">"Status"</th>
                            <td><span class=status_badge_class(&status_key)>{status_label}</span></td>
                        </tr>
                        <tr>
                            <th scope="row">"Created"</th>
                            <td><Timestamp value=detail.created_at/></td>
                        </tr>
                        <tr>
                            <th scope="row">"Submitted"</th>
                            <td>
                                {submitted_at
                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                            </td>
                        </tr>
                    </InfoListTable>
                    <div class="form-actions">
                        <a class="button button--secondary" href="/responses">"Back to Responses"</a>
                        {if is_draft {
                            view! { <a class="button" href=edit_href>"Edit Draft"</a> }.into_any()
                        } else {
                            empty_view()
                        }}
                    </div>
                </section>

                {runtime
                    .map(|runtime| view! { <ResponseRuntimeCard runtime/> }.into_any())
                    .unwrap_or_else(|| empty_view())}

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Response Values"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || values_expanded.get().to_string()
                            on:click=move |_| values_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if values_expanded.get() {
                                    "Hide Values".to_string()
                                } else {
                                    format!("Show {values_count} Values")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if values_expanded.get() {
                            view! { <ResponseValuesTable values=values.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Audit Trail"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || audit_expanded.get().to_string()
                            on:click=move |_| audit_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if audit_expanded.get() {
                                    "Hide Audit Trail".to_string()
                                } else {
                                    format!("Show {audit_count} Audit Events")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if audit_expanded.get() {
                            view! { <ResponseAuditTable events=audit_events.clone()/> }.into_any()
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
fn ResponseRuntimeCard(runtime: SubmissionRuntimeDetail) -> impl IntoView {
    let current_position = runtime.current_step_position + 1;
    let next_step = nonempty_text(runtime.next_step_title.as_deref(), "Final step");
    let history = runtime.history.clone();

    view! {
        <section class="organization-detail-card">
            <h3>"Workflow Runtime"</h3>
            <InfoListTable>
                <tr>
                    <th scope="row">"Workflow"</th>
                    <td>{runtime.workflow_name}</td>
                </tr>
                <tr>
                    <th scope="row">"Current Step"</th>
                    <td>{format!("{} of {}: {}", current_position, runtime.step_count, runtime.current_step_title)}</td>
                </tr>
                <tr>
                    <th scope="row">"Next Step"</th>
                    <td>{next_step}</td>
                </tr>
            </InfoListTable>
            <div class="form-detail-attached-list">
                {if history.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No runtime steps to display"</p> }.into_any()
                } else {
                    history
                        .into_iter()
                        .map(|step| {
                            let status = step.status.clone();
                            view! {
                                <div class="forms-attached-sheet__item">
                                    <span>{format!("Step {}: {}", step.position + 1, step.title)}</span>
                                    <small>{format!("{} - {}", step.form_name, metadata_label(&status))}</small>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </section>
    }
}

#[component]
fn ResponseValuesTable(values: Vec<SubmissionValueDetail>) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Field"</th>
                    <th scope="col">"Type"</th>
                    <th scope="col">"Value"</th>
                </tr>
            </thead>
            <tbody>
                {if values.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Response Values to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    values
                        .into_iter()
                        .map(|value| {
                            let rendered_value = response_value_label(value.value.as_ref());
                            view! {
                                <tr>
                                    <th scope="row">{value.label}</th>
                                    <td>{metadata_label(&value.field_type)}</td>
                                    <td>{rendered_value}</td>
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
fn ResponseAuditTable(events: Vec<SubmissionAuditEventSummary>) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Event"</th>
                    <th scope="col">"Account"</th>
                    <th scope="col">"When"</th>
                </tr>
            </thead>
            <tbody>
                {if events.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Audit Events to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    events
                        .into_iter()
                        .map(|event| {
                            view! {
                                <tr>
                                    <th scope="row">{metadata_label(&event.event_type)}</th>
                                    <td>{nonempty_text(event.account_email.as_deref(), "System")}</td>
                                    <td><Timestamp value=event.created_at/></td>
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
fn ResponseEditForm(
    detail: SubmissionDetail,
    rendered_form: RenderedForm,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let detail_href = format!("/responses/{}", detail.id);
    let save_submission_id = detail.id.clone();
    let submit_submission_id = detail.id.clone();
    let rendered_for_save = rendered_form.clone();
    let rendered_for_submit = rendered_form.clone();

    view! {
        <form class="native-form response-edit-form" on:submit=move |event| event.prevent_default()>
            <section class="organization-detail-card">
                <h3>{detail.form_name}</h3>
                <InfoListTable>
                    <tr>
                        <th scope="row">"Form Version"</th>
                        <td>{detail.version_label}</td>
                    </tr>
                    <tr>
                        <th scope="row">"Node"</th>
                        <td>{detail.node_name}</td>
                    </tr>
                    <tr>
                        <th scope="row">"Status"</th>
                        <td><span class=status_badge_class(&detail.status)>{metadata_label(&detail.status)}</span></td>
                    </tr>
                </InfoListTable>
            </section>

            {rendered_form
                .sections
                .into_iter()
                .map(|section| {
                    view! {
                        <section class="organization-detail-card organization-detail-card--wide response-form-section">
                            <h3>{section.title}</h3>
                            {if !section.description.trim().is_empty() {
                                view! { <p>{section.description}</p> }.into_any()
                            } else {
                                empty_view()
                            }}
                            <div class="form-grid response-form-grid">
                                {section
                                    .fields
                                    .into_iter()
                                    .map(|field| {
                                        view! {
                                            <ResponseFieldInput
                                                field
                                                text_values
                                                boolean_values
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </section>
                    }
                })
                .collect_view()}

            {move || {
                message
                    .get()
                    .map(|message| {
                        let class = if message.to_lowercase().contains("saved") {
                            "form-message"
                        } else {
                            "form-message is-error"
                        };
                        view! { <p class=class role="status">{message}</p> }
                    })
            }}

            <div class="form-actions">
                <a class="button button--secondary" href=detail_href>"Back to Detail"</a>
                <button
                    class="button button--secondary"
                    type="button"
                    disabled=move || is_saving.get()
                    on:click=move |_| {
                        save_submission_values(
                            save_submission_id.clone(),
                            rendered_for_save.clone(),
                            text_values.get(),
                            boolean_values.get(),
                            is_saving,
                            message,
                        );
                    }
                >
                    {move || if is_saving.get() { "Saving..." } else { "Save Draft" }}
                </button>
                <button
                    class="button"
                    type="button"
                    disabled=move || is_saving.get()
                    on:click=move |_| {
                        submit_response_values(
                            submit_submission_id.clone(),
                            rendered_for_submit.clone(),
                            text_values.get(),
                            boolean_values.get(),
                            is_saving,
                            message,
                        );
                    }
                >
                    {move || if is_saving.get() { "Submitting..." } else { "Submit Response" }}
                </button>
            </div>
        </form>
    }
}

#[component]
fn ResponseFieldInput(
    field: RenderedField,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
) -> impl IntoView {
    let field_key = field.key.clone();
    let field_key_for_input = field.key.clone();
    let field_key_for_bool = field.key.clone();
    let input_id = format!("response-field-{}", field.id);
    let required_label = if field.required { " *" } else { "" };
    let layout_style = rendered_form_field_layout_style(&field);
    let field_height = field.grid_height;
    let field_class = response_field_class(&field.field_type);

    view! {
        <div class=field_class style=layout_style>
            {if field.field_type == "static_text" {
                empty_view()
            } else {
                view! { <span>{format!("{}{}", field.label, required_label)}</span> }.into_any()
            }}
            {if field.field_type == "static_text" {
                view! {
                    <p class="response-form-field__static-text">{field.label.clone()}</p>
                }
                .into_any()
            } else if field.field_type == "boolean" {
                let input_id_for_label = input_id.clone();
                view! {
                    <label class="form-field--checkbox" for=input_id_for_label>
                        <input
                            id=input_id
                            type="checkbox"
                            prop:checked=move || {
                                boolean_values
                                    .get()
                                    .get(&field_key_for_bool)
                                    .copied()
                                    .unwrap_or(false)
                            }
                            on:change=move |event| {
                                let checked = event_target_checked(&event);
                                boolean_values.update(|values| {
                                    values.insert(field_key.clone(), checked);
                                });
                            }
                        />
                        <span>"Yes"</span>
                    </label>
                }
                .into_any()
            } else {
                let input_type = if field.field_type == "number" {
                    "number"
                } else if field.field_type == "date" {
                    "date"
                } else {
                    "text"
                };
                if input_type == "text" && field_height > 1 {
                    view! {
                        <textarea
                            id=input_id
                            required=field.required
                            prop:value=move || {
                                text_values
                                    .get()
                                    .get(&field_key_for_input)
                                    .cloned()
                                    .unwrap_or_default()
                            }
                            on:input=move |event| {
                                let value = event_target_value(&event);
                                text_values.update(|values| {
                                    values.insert(field.key.clone(), value);
                                });
                            }
                        ></textarea>
                    }
                    .into_any()
                } else {
                    view! {
                        <input
                            id=input_id
                            type=input_type
                            required=field.required
                            prop:value=move || {
                                text_values
                                    .get()
                                    .get(&field_key_for_input)
                                    .cloned()
                                    .unwrap_or_default()
                            }
                            on:input=move |event| {
                                let value = event_target_value(&event);
                                text_values.update(|values| {
                                    values.insert(field.key.clone(), value);
                                });
                            }
                        />
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
