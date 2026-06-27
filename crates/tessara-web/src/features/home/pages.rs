//! Route-level page composition for the Home feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use leptos::prelude::*;
use serde::Deserialize;

#[cfg(feature = "hydrate")]
use crate::http::{navigate_to_href, redirect_to_login, send_json_request};
use crate::ui::{AppShell, DataTable, PageHeader, TablePaginationFooter, Timestamp};
use crate::utils::pagination::pagination_page_start;
use crate::utils::text::nonempty_text;

#[component]
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

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct PendingWorkflowWork {
    workflow_assignment_id: String,
    workflow_id: String,
    workflow_name: String,
    workflow_description: String,
    workflow_version_id: String,
    workflow_version_label: Option<String>,
    workflow_step_title: String,
    workflow_step_position: i32,
    workflow_step_count: i64,
    next_workflow_step_title: Option<String>,
    next_workflow_step_form_name: Option<String>,
    form_id: String,
    form_name: String,
    form_version_id: String,
    form_version_label: Option<String>,
    node_id: String,
    node_name: String,
    account_id: String,
    account_display_name: String,
    assigned_at: String,
}

#[cfg(feature = "hydrate")]
enum PendingWorkflowWorkApiError {
    Unauthorized,
    Message(String),
}

#[cfg(feature = "hydrate")]
impl PendingWorkflowWorkApiError {
    fn message(message: impl Into<String>) -> Self {
        Self::Message(message.into())
    }
}

fn load_pending_work(
    pending_work: RwSignal<Vec<PendingWorkflowWork>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_pending_work().await {
                Ok(loaded_work) => {
                    pending_work.set(loaded_work);
                    is_loading.set(false);
                }
                Err(PendingWorkflowWorkApiError::Unauthorized) => {
                    pending_work.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(PendingWorkflowWorkApiError::Message(error)) => {
                    pending_work.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (pending_work, is_loading, load_error);
    }
}

#[cfg(feature = "hydrate")]
async fn fetch_pending_work() -> Result<Vec<PendingWorkflowWork>, PendingWorkflowWorkApiError> {
    match gloo_net::http::Request::get("/api/workflow-assignments/pending")
        .send()
        .await
    {
        Ok(response) if response.status() == 401 => Err(PendingWorkflowWorkApiError::Unauthorized),
        Ok(response) if response.ok() => {
            response
                .json::<Vec<PendingWorkflowWork>>()
                .await
                .map_err(|error| {
                    PendingWorkflowWorkApiError::message(format!(
                        "Unable to parse assigned work: {error}"
                    ))
                })
        }
        Ok(response) => Err(PendingWorkflowWorkApiError::message(format!(
            "Unable to load assigned work. Server returned {}.",
            response.status()
        ))),
        Err(error) => Err(PendingWorkflowWorkApiError::message(format!(
            "Unable to load assigned work: {error}"
        ))),
    }
}

#[cfg(feature = "hydrate")]
impl PendingWorkflowWorkApiError {
    fn from_transport_error(error: String) -> Self {
        if error == "Authentication is required." {
            Self::Unauthorized
        } else {
            Self::Message(error)
        }
    }
}

#[cfg(feature = "hydrate")]
async fn start_pending_work_response(
    workflow_assignment_id: &str,
) -> Result<String, PendingWorkflowWorkApiError> {
    let response = send_json_request::<serde_json::Value>(
        gloo_net::http::Request::post(&format!(
            "/api/workflow-assignments/{workflow_assignment_id}/start"
        )),
        Some("{}".into()),
        "Start assigned response",
    )
    .await
    .map_err(PendingWorkflowWorkApiError::from_transport_error)?;

    response
        .get("id")
        .and_then(|value| value.as_str().map(str::to_owned))
        .or_else(|| {
            response
                .get("id")
                .and_then(|value| value.as_i64().map(|value| value.to_string()))
        })
        .ok_or_else(|| {
            PendingWorkflowWorkApiError::message(
                "Assigned response was started, but the response id was missing.",
            )
        })
}

fn start_pending_work_response_and_navigate(
    workflow_assignment_id: String,
    is_starting: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_starting.set(true);
            message.set(Some("Starting assigned response...".into()));

            match start_pending_work_response(&workflow_assignment_id).await {
                Ok(id) => {
                    navigate_to_href(&format!("/responses/{id}/edit"));
                }
                Err(PendingWorkflowWorkApiError::Unauthorized) => {
                    is_starting.set(false);
                    redirect_to_login();
                }
                Err(PendingWorkflowWorkApiError::Message(error)) => {
                    message.set(Some(error));
                    is_starting.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflow_assignment_id, is_starting, message);
    }
}

fn workflow_revision_label_from_raw(label: &str) -> String {
    let trimmed = label.trim();
    if trimmed.is_empty() {
        return "-".to_string();
    }

    if let Ok(revision) = trimmed.parse::<u64>() {
        return revision.to_string();
    }

    trimmed
        .split('.')
        .next()
        .and_then(|part| part.trim().parse::<u64>().ok())
        .map(|revision| revision.to_string())
        .unwrap_or_else(|| trimmed.to_string())
}

#[component]
fn HomePendingWork(
    pending_work: Vec<PendingWorkflowWork>,
    is_starting: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let mut pending_work = pending_work;
    pending_work.sort_by(|left, right| right.assigned_at.cmp(&left.assigned_at));
    let page_size = RwSignal::new(10usize);
    let page_index = RwSignal::new(0usize);
    let total_count_value = pending_work.len();
    let total_count = Memo::new(move |_| total_count_value);

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
                            .skip(pagination_page_start(total_count.get(), page_size.get(), page_index.get()))
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
                                                    start_pending_work_response_and_navigate(
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
            <TablePaginationFooter
                aria_label="Assigned work table pagination"
                item_label="assigned work items"
                empty_item_label="assigned work"
                total_count=total_count
                page_size=page_size
                page_index=page_index
            />
        </div>
    }
}
