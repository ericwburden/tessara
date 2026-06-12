//! List view components for the Responses feature.
//!
//! Keep collection tables, list filters, and list-page presentation here; detail/editor flows should stay in their dedicated modules.

use super::components::ResponsesList;
use super::loaders::load_submissions;
use crate::features::responses::display::{submission_assignee_label, submission_status_key};
use crate::features::responses::types::SubmissionSummary;
use crate::features::shared::unique_filter_options;
use crate::ui::{AppShell, PageHeader};
use crate::utils::text::text_matches;
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
