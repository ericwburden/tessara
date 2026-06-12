//! Workflow assignment detail sheet.

use crate::features::shared::status_badge_class;
use crate::features::workflows::assignments::WorkflowAssignmentSummary;
use crate::features::workflows::{
    workflow_assignment_revision_label, workflow_assignment_state, workflow_assignment_state_label,
    workflow_assignment_status_key, workflow_assignment_status_label,
};
use crate::ui::Timestamp;
use crate::utils::text::nonempty_text;
use icons::X;
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
/// Renders the workflow assignment detail sheet.
pub(in crate::features::workflows) fn WorkflowAssignmentDetailSheet(
    assignment: WorkflowAssignmentSummary,
    on_close: impl Fn(leptos::ev::MouseEvent) + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let workflow_href = format!("/workflows/{}", assignment.workflow_id);
    let node_href = format!("/organization/{}", assignment.node_id);
    let state_key = workflow_assignment_state(&assignment);
    let state_label = workflow_assignment_state_label(&assignment);
    let status_key = workflow_assignment_status_key(&assignment);
    let status_label = workflow_assignment_status_label(&assignment);

    view! {
        <Portal>
            <section class="sheet-overlay workflow-assignment-detail-overlay" aria-label="Workflow assignment detail">
                <button class="sheet-overlay__scrim" type="button" aria-label="Close assignment details" on:click=on_close></button>
                <aside class="sheet-panel blurred-surface workflow-assignment-detail-sheet" role="dialog" aria-modal="true" aria-label="Workflow assignment details">
                    <div class="sheet-panel__actions">
                        <button class="icon-button sheet-panel__close" type="button" aria-label="Close assignment details" title="Close assignment details" on:click=on_close>
                            <X class="icon-button__icon"/>
                        </button>
                    </div>
                    <header class="sheet-panel__header">
                        <p>"Assignment Detail"</p>
                        <h2>{assignment.workflow_name.clone()}</h2>
                    </header>
                    <section class="sheet-panel__section">
                        <h3>"Workflow"</h3>
                        <table class="info-list-table">
                            <tbody>
                                <tr>
                                    <th scope="row">"Workflow"</th>
                                    <td><a class="data-table__primary-link" href=workflow_href.clone()>{assignment.workflow_name.clone()}</a></td>
                                </tr>
                                <tr>
                                    <th scope="row">"Revision"</th>
                                    <td>{workflow_assignment_revision_label(assignment.workflow_version_label.as_deref())}</td>
                                </tr>
                                <tr>
                                    <th scope="row">"Step"</th>
                                    <td>{assignment.workflow_step_title.clone()}</td>
                                </tr>
                                <tr>
                                    <th scope="row">"Form"</th>
                                    <td>{assignment.form_name.clone()}</td>
                                </tr>
                                <tr>
                                    <th scope="row">"Form Version"</th>
                                    <td>{nonempty_text(assignment.form_version_label.as_deref(), "-")}</td>
                                </tr>
                            </tbody>
                        </table>
                    </section>
                    <section class="sheet-panel__section">
                        <h3>"Assignment"</h3>
                        <table class="info-list-table">
                            <tbody>
                                <tr>
                                    <th scope="row">"Node"</th>
                                    <td><a class="data-table__primary-link" href=node_href.clone()>{assignment.node_name.clone()}</a></td>
                                </tr>
                                <tr>
                                    <th scope="row">"Assignee"</th>
                                    <td>{assignment.account_display_name.clone()}</td>
                                </tr>
                                <tr>
                                    <th scope="row">"Email"</th>
                                    <td>{assignment.account_email.clone()}</td>
                                </tr>
                                <tr>
                                    <th scope="row">"Work State"</th>
                                    <td><span class=status_badge_class(state_key)>{state_label}</span></td>
                                </tr>
                                <tr>
                                    <th scope="row">"Status"</th>
                                    <td><span class=status_badge_class(status_key)>{status_label}</span></td>
                                </tr>
                                <tr>
                                    <th scope="row">"Assigned"</th>
                                    <td><Timestamp value=assignment.created_at.clone()/></td>
                                </tr>
                            </tbody>
                        </table>
                    </section>
                </aside>
            </section>
        </Portal>
    }
}
