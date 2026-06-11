//! List view components for the Responses feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use super::api::load_submissions;
use crate::features::responses::display::{
    submission_assignee_label, submission_progress_label, submission_status_key,
    submission_status_label, submission_step_label, submission_workflow_label,
};
use crate::features::responses::types::SubmissionSummary;
#[cfg(feature = "hydrate")]
use crate::features::shared::navigate_to_href;
use crate::features::shared::{
    FilterHeader as SharedFilterHeader, status_badge_class, unique_filter_options,
};
use crate::ui::empty_view;
use crate::ui::{AppShell, DataTable, DropdownMenu, PageHeader, TablePaginationFooter, Timestamp};
use crate::utils::text::text_matches;

use icons::{PanelRight, Pencil, Search};
use leptos::prelude::*;

#[allow(non_snake_case)]
/// Renders the responses page content view.
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
/// Renders the responses list view.
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
    let total_count_memo = Memo::new(move |_| total_count);
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
                <TablePaginationFooter
                    aria_label="Responses table pagination"
                    item_label="responses"
                    total_count=total_count_memo
                    page_size=page_size
                    page_index=page_index
                />
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
