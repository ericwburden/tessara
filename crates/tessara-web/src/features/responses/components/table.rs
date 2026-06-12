//! Desktop response table for response summaries.

use crate::features::responses::display::{
    submission_assignee_label, submission_progress_label, submission_status_key,
    submission_status_label, submission_step_label, submission_workflow_label,
};
use crate::features::responses::types::SubmissionSummary;
use crate::features::shared::status_badge_class;
#[cfg(feature = "hydrate")]
use crate::http::navigate_to_href;
use crate::ui::{DataTable, DropdownMenu, TableFilterHeader, Timestamp, empty_view};
use crate::utils::pagination::pagination_page_start;
use icons::{PanelRight, Pencil};
use leptos::prelude::*;

#[component]
pub(in crate::features::responses) fn ResponseDesktopTable(
    submissions: Vec<SubmissionSummary>,
    total_count: usize,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
    assignee_filter: RwSignal<String>,
    status_filter: RwSignal<String>,
    assignee_options: Vec<String>,
    status_options: Vec<String>,
) -> impl IntoView {
    view! {
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
                {move || if submissions.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="7">"No Responses to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    submissions
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
    }
}
