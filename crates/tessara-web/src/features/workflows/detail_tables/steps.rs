//! Workflow detail steps table.

use crate::features::workflows::types::WorkflowStepSummary;
use crate::ui::DataTable;
use crate::utils::text::nonempty_text;
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowStepsTable(
    steps: Vec<WorkflowStepSummary>,
) -> impl IntoView {
    view! {
        <DataTable>
            <thead>
                <tr>
                    <th scope="col">"Step"</th>
                    <th scope="col">"Form"</th>
                    <th scope="col">"Form Version"</th>
                </tr>
            </thead>
            <tbody>
                {if steps.is_empty() {
                    view! {
                        <tr>
                            <td class="data-table__empty" colspan="3">"No Workflow Steps to Display"</td>
                        </tr>
                    }
                    .into_any()
                } else {
                    steps
                        .into_iter()
                        .map(|step| {
                            let form_href = format!("/forms/{}", step.form_id);
                            let step_title = nonempty_text(Some(&step.title), "Untitled step");
                            view! {
                                <tr>
                                    <th scope="row">{step_title}</th>
                                    <td><a class="data-table__primary-link" href=form_href>{step.form_name}</a></td>
                                    <td>{nonempty_text(step.form_version_label.as_deref(), "-")}</td>
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
