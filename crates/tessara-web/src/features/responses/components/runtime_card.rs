//! Workflow runtime card for response details.

use crate::features::responses::types::SubmissionRuntimeDetail;
use crate::ui::InfoListTable;
use crate::utils::metadata::metadata_label;
use crate::utils::text::nonempty_text;
use leptos::prelude::*;

/// Renders workflow runtime progress for a response.
#[component]
pub(crate) fn ResponseRuntimeCard(runtime: SubmissionRuntimeDetail) -> impl IntoView {
    let current_position = runtime.current_step_position + 1;
    let next_step = nonempty_text(runtime.next_step_title.as_deref(), "Final step");
    let history = runtime.history.clone();

    view! {
        <section class="organization-detail-card">
            <h3>"Workflow Runtime"</h3>
            <InfoListTable>
                <tr>
                    <th scope="row">"Workflow"</th>
                    <td>{runtime.workflow_name}</td>
                </tr>
                <tr>
                    <th scope="row">"Current Step"</th>
                    <td>{format!("{} of {}: {}", current_position, runtime.step_count, runtime.current_step_title)}</td>
                </tr>
                <tr>
                    <th scope="row">"Next Step"</th>
                    <td>{next_step}</td>
                </tr>
            </InfoListTable>
            <div class="form-detail-attached-list">
                {if history.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No runtime steps to display"</p> }.into_any()
                } else {
                    history
                        .into_iter()
                        .map(|step| {
                            let status = step.status.clone();
                            view! {
                                <div class="forms-attached-sheet__item">
                                    <span>{format!("Step {}: {}", step.position + 1, step.title)}</span>
                                    <small>{format!("{} - {}", step.form_name, metadata_label(&status))}</small>
                                </div>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </section>
    }
}
