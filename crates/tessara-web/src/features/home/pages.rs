//! Owns the features::home::pages module behavior.

use leptos::prelude::*;

use crate::features::workflows::api::{
    start_workflow_assignment_response, workflow_revision_label_from_raw,
};
use crate::features::workflows::assignments::{PendingWorkflowWork, load_pending_work};
use crate::ui::{AppShell, DataTable, PageHeader, Timestamp};
use crate::utils::text::nonempty_text;

#[component]
/// Renders the home page view.
pub fn HomePage() -> impl IntoView {
    let pending_work = RwSignal::new(Vec::<PendingWorkflowWork>::new());
    let pending_work_loading = RwSignal::new(true);
    let pending_work_error = RwSignal::new(None::<String>);
    let is_starting = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_pending_work(pending_work, pending_work_loading, pending_work_error);
    });

    view! {
        <AppShell active_route="home" title="Home">
            <section class="route-panel home-page">
                <section class="organization-detail-card organization-detail-card--wide">
                    <PageHeader title="Assigned to Me">
                        <a class="button button--secondary" href="/responses/new">"Start Response"</a>
                    </PageHeader>
                    {move || {
                        if pending_work_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading assigned work"</h3>
                                    <p>"Fetching workflow assignments ready for completion."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(error) = pending_work_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Assigned work unavailable"</h3>
                                    <p>{error}</p>
                                </section>
                            }
                            .into_any()
                        } else {
                            view! {
                                <HomePendingWork
                                    pending_work=pending_work.get()
                                    is_starting=is_starting
                                    message=message
                                />
                            }
                            .into_any()
                        }
                    }}
                    {move || message.get().map(|message| view! {
                        <p class="form-message" role="status">{message}</p>
                    })}
                </section>
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the home pending work view.
fn HomePendingWork(
    pending_work: Vec<PendingWorkflowWork>,
    is_starting: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let mut pending_work = pending_work;
    pending_work.sort_by(|left, right| right.assigned_at.cmp(&left.assigned_at));
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count = pending_work.len();
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
            "No assigned work to display".to_string()
        } else {
            format!(
                "Showing {}-{} of {} assigned work items",
                page_start() + 1,
                page_end(),
                total_count
            )
        }
    };

    view! {
        <div class="searchable-data-table home-pending-work-table">
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Workflow"</th>
                        <th scope="col">"Step"</th>
                        <th scope="col">"Form"</th>
                        <th scope="col">"Node"</th>
                        <th scope="col">"Assigned"</th>
                        <th class="data-table__cell--center" scope="col">"Actions"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || if pending_work.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="6">"No Assigned Work to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        pending_work
                            .iter()
                            .skip(page_start())
                            .take(page_size.get())
                            .cloned()
                            .map(|work| {
                                let workflow_href = format!("/workflows/{}", work.workflow_id);
                                let assignment_id = work.workflow_assignment_id.clone();
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <a class="data-table__primary-link" href=workflow_href>{work.workflow_name}</a>
                                            <small class="workflow-assignment-step-meta">
                                                {format!(
                                                    "Revision {}",
                                                    work.workflow_version_label
                                                        .as_deref()
                                                        .map(workflow_revision_label_from_raw)
                                                        .unwrap_or_else(|| "-".to_string())
                                                )}
                                            </small>
                                        </th>
                                        <td>
                                            <span>{work.workflow_step_title}</span>
                                            <small class="workflow-assignment-step-meta">
                                                {format!("Step {} of {}", work.workflow_step_position + 1, work.workflow_step_count)}
                                            </small>
                                        </td>
                                        <td>
                                            <span>{work.form_name}</span>
                                            <small class="workflow-assignment-step-meta">
                                                {format!(
                                                    "Form Version {}",
                                                    nonempty_text(work.form_version_label.as_deref(), "-")
                                                )}
                                            </small>
                                        </td>
                                        <td>{work.node_name}</td>
                                        <td><Timestamp value=work.assigned_at/></td>
                                        <td class="data-table__cell--center">
                                            <button
                                                class="button button--secondary"
                                                type="button"
                                                disabled=move || is_starting.get()
                                                on:click=move |_| {
                                                    start_workflow_assignment_response(
                                                        assignment_id.clone(),
                                                        is_starting,
                                                        message,
                                                    );
                                                }
                                            >
                                                {move || if is_starting.get() { "Starting..." } else { "Start" }}
                                            </button>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
            </DataTable>
            <div class="directory-table-pagination" aria-label="Assigned work table pagination">
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
    }
}
