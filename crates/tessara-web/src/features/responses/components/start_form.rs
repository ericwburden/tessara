//! Response start assignment form.

use super::ResponseAssignmentStartFields;
use crate::features::responses::actions::start_workflow_assignment_response;
use crate::features::responses::display::{
    response_selected_assignment, response_start_can_submit,
};
use crate::features::responses::types::AssignmentResponseStartOption;
use leptos::prelude::*;

#[component]
pub(crate) fn ResponseAssignmentStartForm(
    assignments: Vec<AssignmentResponseStartOption>,
    options: RwSignal<Option<crate::features::responses::types::AssignmentResponseStartOptions>>,
    is_loading: RwSignal<bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    selected_assignment_index: RwSignal<String>,
) -> impl IntoView {
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
            <ResponseAssignmentStartFields assignments selected_assignment_index/>

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
}
