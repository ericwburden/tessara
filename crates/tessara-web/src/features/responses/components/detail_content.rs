//! Response detail presentation.

use super::{ResponseAuditTable, ResponseRuntimeCard, ResponseValuesTable};
use crate::features::responses::types::SubmissionDetail;
use crate::features::shared::status_badge_class;
use crate::ui::{InfoListTable, Timestamp, empty_view};
use crate::utils::metadata::metadata_label;
use leptos::prelude::*;

#[component]
pub(in crate::features::responses) fn ResponseDetailContent(
    detail: SubmissionDetail,
) -> impl IntoView {
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
