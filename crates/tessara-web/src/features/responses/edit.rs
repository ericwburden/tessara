//! Response route components.

use super::api::{load_submission_edit_context, save_submission_values, submit_response_values};
use crate::features::forms::{RenderedField, RenderedForm};
use crate::features::responses::display::{rendered_form_field_layout_style, response_field_class};
use crate::features::responses::types::SubmissionDetail;
use crate::features::shared::status_badge_class;
use crate::types::route_params::{SubmissionRouteParams, require_route_params};
use crate::ui::empty_view;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    InfoListTable, PageHeader,
};
use crate::utils::metadata::metadata_label;
use std::collections::HashMap;

use leptos::prelude::*;

#[component]
/// Renders the responses edit page content view.
pub(super) fn ResponsesEditPageContent() -> impl IntoView {
    let params = require_route_params::<SubmissionRouteParams>();
    let submission_id = params.submission_id;
    let detail = RwSignal::new(None::<SubmissionDetail>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let text_values = RwSignal::new(HashMap::<String, String>::new());
    let boolean_values = RwSignal::new(HashMap::<String, bool>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let load_error = RwSignal::new(None::<String>);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_submission_edit_context(
            submission_id.clone(),
            detail,
            rendered_form,
            text_values,
            boolean_values,
            is_loading,
            load_error,
        );
    });

    view! {
        <AppShell active_route="responses" title="Edit Response">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/responses">"Responses"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Edit Response"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <section class="route-panel responses-page">
                    <PageHeader title="Edit Response"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading response form"</h3>
                                    <p>"Fetching response values and form fields."</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(message) = load_error.get() {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response unavailable"</h3>
                                    <p>{message}</p>
                                </section>
                            }
                            .into_any()
                        } else if let Some(detail) = detail.get() {
                            if detail.status != "draft" {
                                let detail_href = format!("/responses/{}", detail.id);
                                view! {
                                    <section class="organization-state" aria-live="polite">
                                        <h3>"Submitted response"</h3>
                                        <p>"This response has been submitted and is read-only."</p>
                                        <a class="button button--secondary" href=detail_href>"Back to Detail"</a>
                                    </section>
                                }
                                .into_any()
                            } else if let Some(rendered_form) = rendered_form.get() {
                                view! {
                                    <ResponseEditForm
                                        detail
                                        rendered_form
                                        text_values
                                        boolean_values
                                        is_saving
                                        message
                                    />
                                }
                                .into_any()
                            } else {
                                view! {
                                    <section class="organization-state is-error" role="alert">
                                        <h3>"Response form unavailable"</h3>
                                        <p>"The selected response form could not be loaded."</p>
                                    </section>
                                }
                                .into_any()
                            }
                        } else {
                            view! {
                                <section class="organization-state is-error" role="alert">
                                    <h3>"Response unavailable"</h3>
                                    <p>"The selected response could not be loaded."</p>
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
/// Renders the response edit form view.
fn ResponseEditForm(
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

#[component]
/// Renders the response field input view.
fn ResponseFieldInput(
    field: RenderedField,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
) -> impl IntoView {
    let field_key = field.key.clone();
    let field_key_for_input = field.key.clone();
    let field_key_for_bool = field.key.clone();
    let input_id = format!("response-field-{}", field.id);
    let required_label = if field.required { " *" } else { "" };
    let layout_style = rendered_form_field_layout_style(&field);
    let field_height = field.grid_height;
    let field_class = response_field_class(&field.field_type);

    view! {
        <div class=field_class style=layout_style>
            {if field.field_type == "static_text" {
                empty_view()
            } else {
                view! { <span>{format!("{}{}", field.label, required_label)}</span> }.into_any()
            }}
            {if field.field_type == "static_text" {
                view! {
                    <p class="response-form-field__static-text">{field.label.clone()}</p>
                }
                .into_any()
            } else if field.field_type == "boolean" {
                let input_id_for_label = input_id.clone();
                view! {
                    <label class="form-field--checkbox" for=input_id_for_label>
                        <input
                            id=input_id
                            type="checkbox"
                            prop:checked=move || {
                                boolean_values
                                    .get()
                                    .get(&field_key_for_bool)
                                    .copied()
                                    .unwrap_or(false)
                            }
                            on:change=move |event| {
                                let checked = event_target_checked(&event);
                                boolean_values.update(|values| {
                                    values.insert(field_key.clone(), checked);
                                });
                            }
                        />
                        <span>"Yes"</span>
                    </label>
                }
                .into_any()
            } else {
                let input_type = if field.field_type == "number" {
                    "number"
                } else if field.field_type == "date" {
                    "date"
                } else {
                    "text"
                };
                if input_type == "text" && field_height > 1 {
                    view! {
                        <textarea
                            id=input_id
                            required=field.required
                            prop:value=move || {
                                text_values
                                    .get()
                                    .get(&field_key_for_input)
                                    .cloned()
                                    .unwrap_or_default()
                            }
                            on:input=move |event| {
                                let value = event_target_value(&event);
                                text_values.update(|values| {
                                    values.insert(field.key.clone(), value);
                                });
                            }
                        ></textarea>
                    }
                    .into_any()
                } else {
                    view! {
                        <input
                            id=input_id
                            type=input_type
                            required=field.required
                            prop:value=move || {
                                text_values
                                    .get()
                                    .get(&field_key_for_input)
                                    .cloned()
                                    .unwrap_or_default()
                            }
                            on:input=move |event| {
                                let value = event_target_value(&event);
                                text_values.update(|values| {
                                    values.insert(field.key.clone(), value);
                                });
                            }
                        />
                    }
                    .into_any()
                }
            }}
        </div>
    }
}
