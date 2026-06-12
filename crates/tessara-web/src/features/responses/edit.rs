//! Edit support for the Responses feature.
//!
//! Keep functionality here when it is owned by Responses and specifically supports the Edit concern.

use super::components::ResponseEditForm;
use super::loaders::load_submission_edit_context;
use crate::features::forms::RenderedForm;
use crate::features::responses::types::SubmissionDetail;
use crate::types::route_params::{SubmissionRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
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
