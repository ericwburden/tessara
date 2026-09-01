//! Mobile card list for response summaries.

use crate::display::{
    response_assignee_label, response_progress_label, response_status_key, response_status_label,
    response_step_label, response_workflow_label,
};
use crate::pagination::pagination_page_start;
use crate::status::status_badge_class;
use crate::types::ResponseSummary;
use leptos::prelude::*;
use tessara_module_ui::{Timestamp, empty_view};

#[component]
pub(crate) fn ResponseMobileCards(
    submissions: Vec<ResponseSummary>,
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
                    .map(|response| {
                        let detail_href = format!("/responses/{}", response.id);
                        let edit_href = format!("/responses/{}/edit", response.id);
                        let node_href = format!("/organization/{}", response.node_id);
                        let status_key = response_status_key(&response);
                        let status_label = response_status_label(&response);
                        let workflow_label = response_workflow_label(&response);
                        let step_label = response_step_label(&response);
                        let progress_label = response_progress_label(&response);
                        let assignee = response_assignee_label(&response);
                        let is_draft = status_key == "draft";
                        view! {
                            <article class="forms-list-mobile-card response-mobile-card">
                                <div class="forms-list-mobile-card__header">
                                    <div class="forms-list-mobile-card__title-row">
                                        <h3><a href=detail_href.clone()>{response.form_name}</a></h3>
                                    </div>
                                </div>
                                <dl>
                                    <div>
                                        <dt>"Status"</dt>
                                        <dd><span class=status_badge_class(&status_key)>{status_label}</span></dd>
                                    </div>
                                    <div>
                                        <dt>"Form Version"</dt>
                                        <dd>{response.version_label}</dd>
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
                                        <dd><a href=node_href>{response.node_name}</a></dd>
                                    </div>
                                    <div>
                                        <dt>"Assignee"</dt>
                                        <dd>{assignee}</dd>
                                    </div>
                                    <div>
                                        <dt>"Last Updated"</dt>
                                        <dd><Timestamp value=response.last_modified_at/></dd>
                                    </div>
                                    {if let Some(submitted_at) = response.submitted_at {
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
