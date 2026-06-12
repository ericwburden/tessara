//! Audit trail table for response details.

use crate::features::responses::types::SubmissionAuditEventSummary;
use crate::ui::{DataTable, Timestamp};
use crate::utils::metadata::metadata_label;
use crate::utils::text::nonempty_text;
use leptos::prelude::*;

/// Renders response audit events.
#[component]
pub(crate) fn ResponseAuditTable(events: Vec<SubmissionAuditEventSummary>) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Event"</th>
                    <th scope="col">"Account"</th>
                    <th scope="col">"When"</th>
                </tr>
            </thead>
            <tbody>
                {if events.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Audit Events to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    events
                        .into_iter()
                        .map(|event| {
                            view! {
                                <tr>
                                    <th scope="row">{metadata_label(&event.event_type)}</th>
                                    <td>{nonempty_text(event.account_email.as_deref(), "System")}</td>
                                    <td><Timestamp value=event.created_at/></td>
                                </tr>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </tbody>
        </DataTable>
    }
}
