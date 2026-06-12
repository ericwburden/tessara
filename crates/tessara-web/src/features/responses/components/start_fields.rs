//! Response start assignment picker fields.

use crate::features::responses::types::AssignmentResponseStartOption;
use crate::features::workflows::workflow_revision_label_from_option;
use crate::ui::empty_view;
use crate::utils::text::nonempty_text;
use leptos::prelude::*;

#[component]
pub(crate) fn ResponseAssignmentStartFields(
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
