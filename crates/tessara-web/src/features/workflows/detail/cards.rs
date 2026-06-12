//! Card sections for workflow detail content.

use crate::features::shared::status_badge_class;
use crate::ui::{InfoListTable, Timestamp};
use leptos::prelude::*;

#[component]
pub(super) fn WorkflowDetailsCard(
    slug: String,
    description: String,
    available_at: String,
    source: String,
    revision_count: String,
    assignment_count: String,
) -> impl IntoView {
    view! {
        <section class="organization-detail-card">
            <h3>"Details"</h3>
            <InfoListTable>
                <tr>
                    <th scope="row">"Slug"</th>
                    <td>{slug}</td>
                </tr>
                <tr>
                    <th scope="row">"Description"</th>
                    <td>{description}</td>
                </tr>
                <tr>
                    <th scope="row">"Available At"</th>
                    <td>{available_at}</td>
                </tr>
                <tr>
                    <th scope="row">"Source"</th>
                    <td>{source}</td>
                </tr>
                <tr>
                    <th scope="row">"Revisions"</th>
                    <td>{revision_count}</td>
                </tr>
                <tr>
                    <th scope="row">"Assignments"</th>
                    <td>{assignment_count}</td>
                </tr>
            </InfoListTable>
        </section>
    }
}

#[component]
pub(super) fn WorkflowActiveRevisionCard(
    active_status: String,
    active_status_label: String,
    active_step_count: String,
    active_version_label: String,
    published_at: Option<String>,
) -> impl IntoView {
    view! {
        <section class="organization-detail-card">
            <h3>"Active Revision"</h3>
            <InfoListTable>
                <tr>
                    <th scope="row">"Revision"</th>
                    <td>{active_version_label}</td>
                </tr>
                <tr>
                    <th scope="row">"Status"</th>
                    <td><span class=status_badge_class(&active_status)>{active_status_label}</span></td>
                </tr>
                <tr>
                    <th scope="row">"Steps"</th>
                    <td>{active_step_count}</td>
                </tr>
                <tr>
                    <th scope="row">"Published"</th>
                    <td>
                        {published_at
                            .map(|value| view! { <Timestamp value/> }.into_any())
                            .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                    </td>
                </tr>
            </InfoListTable>
        </section>
    }
}
