//! Workflow detail versions table.

use crate::features::shared::status_badge_class;
use crate::features::workflows::types::WorkflowVersionSummary;
use crate::features::workflows::workflow_revision_label_from_option;
use crate::ui::{DataTable, Timestamp};
use crate::utils::text::sentence_label;
use icons::Pencil;
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowVersionsTable(
    workflow_id: String,
    versions: Vec<WorkflowVersionSummary>,
) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Revision"</th>
                    <th scope="col">"Status"</th>
                    <th scope="col">"Published"</th>
                    <th class="data-table__cell--center" scope="col">"Steps"</th>
                    <th class="data-table__cell--center" scope="col">"Actions"</th>
                </tr>
            </thead>
            <tbody>
                {if versions.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="5">"No Revisions to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    versions
                        .into_iter()
                        .map(|version| {
                            let status = version.status.clone();
                            let published_at = version.published_at.clone();
                            let version_label = workflow_revision_label_from_option(version.workflow_revision_label);
                            let edit_href = format!("/workflows/{}/edit?version_id={}", workflow_id, version.id);
                            let edit_title = format!("Edit {} workflow revision", sentence_label(&status));
                            view! {
                                <tr>
                                    <th scope="row">{version_label}</th>
                                    <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                    <td>
                                        {published_at
                                            .map(|value| view! { <Timestamp value/> }.into_any())
                                            .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                    </td>
                                    <td class="data-table__cell--center">{version.step_count.to_string()}</td>
                                    <td class="data-table__cell--center">
                                        <a class="data-table__action" href=edit_href aria-label=edit_title.clone() title=edit_title>
                                            <Pencil class="icon-button__icon"/>
                                        </a>
                                    </td>
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
