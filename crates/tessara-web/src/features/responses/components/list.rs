//! Response list table and mobile card components.

use crate::features::responses::display::{
    submission_assignee_label, submission_progress_label, submission_status_key,
    submission_status_label, submission_step_label, submission_workflow_label,
};
use crate::features::responses::types::SubmissionSummary;
use crate::features::shared::status_badge_class;
#[cfg(feature = "hydrate")]
use crate::http::navigate_to_href;
use crate::ui::{
    DataTable, DropdownMenu, TableFilterHeader, TablePaginationFooter, Timestamp, empty_view,
};
use crate::utils::pagination::pagination_page_start;

use icons::{PanelRight, Pencil, Search};
use leptos::prelude::*;

#[component]
/// Renders the responses list view.
pub(crate) fn ResponsesList(
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
                                <TableFilterHeader
                                    label="Assignee"
                                    all_label="All Assignees"
                                    filter=assignee_filter
                                    options=assignee_options
                                    always_searchable=true
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">
                                <TableFilterHeader
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
                                .skip(pagination_page_start(total_count, page_size.get(), page_index.get()))
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
                        .skip(pagination_page_start(total_count, page_size.get(), page_index.get()))
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
