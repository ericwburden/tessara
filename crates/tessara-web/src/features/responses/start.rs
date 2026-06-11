//! Response start page components.
//!
//! Keep the workflow/form launch flow for new responses here; response editing and detail presentation belong in sibling modules.

use super::actions::start_workflow_assignment_response;
use super::loaders::load_response_start_options;
use crate::features::responses::display::{
    response_selected_assignment, response_start_can_submit,
};
use crate::features::responses::types::{
    AssignmentResponseStartOption, AssignmentResponseStartOptions,
};
use crate::features::workflows::workflow_revision_label_from_option;
use crate::ui::empty_view;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use crate::utils::text::nonempty_text;
#[cfg(feature = "hydrate")]
use crate::utils::url::current_search_param;

use leptos::prelude::*;

#[component]
/// Renders the responses new page content view.
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
/// Renders the response assignment start fields view.
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
