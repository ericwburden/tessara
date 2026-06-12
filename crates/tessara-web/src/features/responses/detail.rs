//! Detail view components for the Responses feature.
//!
//! Keep read-focused panels and detail-page presentation here; mutation workflows should live in editor or API modules.

use super::components::{ResponseAuditTable, ResponseRuntimeCard, ResponseValuesTable};
use super::loaders::load_submission_detail;
use crate::features::responses::types::SubmissionDetail;
use crate::features::shared::status_badge_class;
use crate::types::route_params::{SubmissionRouteParams, require_route_params};
use crate::ui::empty_view;
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    InfoListTable, PageHeader, Timestamp,
};
use crate::utils::metadata::metadata_label;

use leptos::prelude::*;

#[component]
/// Renders the responses detail page content view.
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

#[component]
/// Renders the response detail content view.
fn ResponseDetailContent(detail: SubmissionDetail) -> impl IntoView {
    let values_expanded = RwSignal::new(false);
    let audit_expanded = RwSignal::new(false);
    let status_key = detail.status.trim().to_lowercase();
    let status_label = metadata_label(&detail.status);
    let edit_href = format!("/responses/{}/edit", detail.id);
    let node_href = format!("/organization/{}", detail.node_id);
    let form_href = format!("/forms/{}", detail.form_id);
    let submitted_at = detail.submitted_at.clone();
    let runtime = detail.runtime.clone();
    let values = detail.values.clone();
    let audit_events = detail.audit_events.clone();
    let values_count = values.len().to_string();
    let audit_count = audit_events.len().to_string();
    let is_draft = status_key == "draft";

    view! {
        <div class="organization-detail-content response-detail-content">
            <header class="organization-detail-content__header">
                <p>"Response Detail"</p>
                <h2>{detail.form_name.clone()}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Summary"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Form"</th>
                            <td><a href=form_href>{detail.form_name}</a></td>
                        </tr>
                        <tr>
                            <th scope="row">"Form Version"</th>
                            <td>{detail.version_label}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Node"</th>
                            <td><a href=node_href>{detail.node_name}</a></td>
                        </tr>
                        <tr>
                            <th scope="row">"Status"</th>
                            <td><span class=status_badge_class(&status_key)>{status_label}</span></td>
                        </tr>
                        <tr>
                            <th scope="row">"Created"</th>
                            <td><Timestamp value=detail.created_at/></td>
                        </tr>
                        <tr>
                            <th scope="row">"Submitted"</th>
                            <td>
                                {submitted_at
                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                            </td>
                        </tr>
                    </InfoListTable>
                    <div class="form-actions">
                        <a class="button button--secondary" href="/responses">"Back to Responses"</a>
                        {if is_draft {
                            view! { <a class="button" href=edit_href>"Edit Draft"</a> }.into_any()
                        } else {
                            empty_view()
                        }}
                    </div>
                </section>

                {runtime
                    .map(|runtime| view! { <ResponseRuntimeCard runtime/> }.into_any())
                    .unwrap_or_else(empty_view)}

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Response Values"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || values_expanded.get().to_string()
                            on:click=move |_| values_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if values_expanded.get() {
                                    "Hide Values".to_string()
                                } else {
                                    format!("Show {values_count} Values")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if values_expanded.get() {
                            view! { <ResponseValuesTable values=values.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>

                <section class="organization-detail-card organization-detail-card--wide form-detail-fields-card">
                    <header class="form-detail-disclosure-header">
                        <h3>"Audit Trail"</h3>
                        <button
                            class="link-button form-detail-disclosure-toggle"
                            type="button"
                            aria-expanded=move || audit_expanded.get().to_string()
                            on:click=move |_| audit_expanded.update(|expanded| *expanded = !*expanded)
                        >
                            {move || {
                                if audit_expanded.get() {
                                    "Hide Audit Trail".to_string()
                                } else {
                                    format!("Show {audit_count} Audit Events")
                                }
                            }}
                        </button>
                    </header>
                    {move || {
                        if audit_expanded.get() {
                            view! { <ResponseAuditTable events=audit_events.clone()/> }.into_any()
                        } else {
                            empty_view()
                        }
                    }}
                </section>
            </div>
        </div>
    }
}
