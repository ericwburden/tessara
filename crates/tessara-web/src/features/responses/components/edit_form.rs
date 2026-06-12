//! Response edit form component.

use super::ResponseFieldInput;
use crate::features::forms::RenderedForm;
use crate::features::responses::actions::{save_submission_values, submit_response_values};
use crate::features::responses::types::SubmissionDetail;
use crate::features::shared::status_badge_class;
use crate::ui::{InfoListTable, empty_view};
use crate::utils::metadata::metadata_label;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub(in crate::features::responses) fn ResponseEditForm(
    detail: SubmissionDetail,
    rendered_form: RenderedForm,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) -> impl IntoView {
    let detail_href = format!("/responses/{}", detail.id);
    let save_submission_id = detail.id.clone();
    let submit_submission_id = detail.id.clone();
    let rendered_for_save = rendered_form.clone();
    let rendered_for_submit = rendered_form.clone();

    view! {
        <form class="native-form response-edit-form" on:submit=move |event| event.prevent_default()>
            <section class="organization-detail-card">
                <h3>{detail.form_name}</h3>
                <InfoListTable>
                    <tr>
                        <th scope="row">"Form Version"</th>
                        <td>{detail.version_label}</td>
                    </tr>
                    <tr>
                        <th scope="row">"Node"</th>
                        <td>{detail.node_name}</td>
                    </tr>
                    <tr>
                        <th scope="row">"Status"</th>
                        <td><span class=status_badge_class(&detail.status)>{metadata_label(&detail.status)}</span></td>
                    </tr>
                </InfoListTable>
            </section>

            {rendered_form
                .sections
                .into_iter()
                .map(|section| {
                    view! {
                        <section class="organization-detail-card organization-detail-card--wide response-form-section">
                            <h3>{section.title}</h3>
                            {if !section.description.trim().is_empty() {
                                view! { <p>{section.description}</p> }.into_any()
                            } else {
                                empty_view()
                            }}
                            <div class="form-grid response-form-grid">
                                {section
                                    .fields
                                    .into_iter()
                                    .map(|field| {
                                        view! {
                                            <ResponseFieldInput
                                                field
                                                text_values
                                                boolean_values
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </div>
                        </section>
                    }
                })
                .collect_view()}

            {move || {
                message
                    .get()
                    .map(|message| {
                        let class = if message.to_lowercase().contains("saved") {
                            "form-message"
                        } else {
                            "form-message is-error"
                        };
                        view! { <p class=class role="status">{message}</p> }
                    })
            }}

            <div class="form-actions">
                <a class="button button--secondary" href=detail_href>"Back to Detail"</a>
                <button
                    class="button button--secondary"
                    type="button"
                    disabled=move || is_saving.get()
                    on:click=move |_| {
                        save_submission_values(
                            save_submission_id.clone(),
                            rendered_for_save.clone(),
                            text_values.get(),
                            boolean_values.get(),
                            is_saving,
                            message,
                        );
                    }
                >
                    {move || if is_saving.get() { "Saving..." } else { "Save Draft" }}
                </button>
                <button
                    class="button"
                    type="button"
                    disabled=move || is_saving.get()
                    on:click=move |_| {
                        submit_response_values(
                            submit_submission_id.clone(),
                            rendered_for_submit.clone(),
                            text_values.get(),
                            boolean_values.get(),
                            is_saving,
                            message,
                        );
                    }
                >
                    {move || if is_saving.get() { "Submitting..." } else { "Submit Response" }}
                </button>
            </div>
        </form>
    }
}
