//! Mobile card list for response summaries.

use crate::features::responses::display::{
    submission_assignee_label, submission_progress_label, submission_status_key,
    submission_status_label, submission_step_label, submission_workflow_label,
};
use crate::features::responses::types::SubmissionSummary;
use crate::features::shared::status_badge_class;
use crate::ui::{Timestamp, empty_view};
use crate::utils::pagination::pagination_page_start;
use leptos::prelude::*;

#[component]
pub(crate) fn ResponseMobileCards(
    submissions: Vec<SubmissionSummary>,
    total_count: usize,
    page_size: RwSignal<usize>,
    page_index: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="forms-list-mobile-cards responses-mobile-cards">
            {move || if submissions.is_empty() {
                view! { <p class="forms-list-mobile-empty">"No Responses to Display"</p> }.into_any()
            } else {
                submissions
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
    }
}
