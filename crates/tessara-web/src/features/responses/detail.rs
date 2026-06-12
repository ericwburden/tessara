//! Detail view components for the Responses feature.
//!
//! Keep read-focused panels and detail-page presentation here; mutation workflows should live in editor or API modules.

use super::components::ResponseDetailContent;
use super::loaders::load_submission_detail;
use crate::features::responses::types::SubmissionDetail;
use crate::types::route_params::{SubmissionRouteParams, require_route_params};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};

use leptos::prelude::*;

#[component]
pub(super) fn ResponsesDetailPageContent() -> impl IntoView {
    let params = require_route_params::<SubmissionRouteParams>();
    let submission_id = params.submission_id;
    let detail = RwSignal::new(None::<SubmissionDetail>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_submission_detail(submission_id.clone(), detail, is_loading, load_error);
    });

    view! {
        <AppShell active_route="responses" title="Response Detail">
            <div class="app-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/responses">"Responses"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>"Response Detail"</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <section class="route-panel responses-page">
                    <PageHeader title="Response Detail"/>

                    {move || {
                        if is_loading.get() {
                            view! {
                                <section class="organization-state" aria-live="polite">
                                    <h3>"Loading response"</h3>
                                    <p>"Fetching response values and audit history."</p>
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
                            view! { <ResponseDetailContent detail/> }.into_any()
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
