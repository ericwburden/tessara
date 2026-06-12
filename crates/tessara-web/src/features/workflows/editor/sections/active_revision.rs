//! Workflow editor active revision section.

use crate::features::shared::status_badge_class;
use leptos::prelude::*;

#[component]
pub(in crate::features::workflows) fn WorkflowActiveRevisionSection(
    edit_version_label: RwSignal<String>,
    edit_version_status: RwSignal<String>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Active Revision"</h3>
            <table class="info-list-table">
                <tbody>
                    <tr>
                        <th scope="row">"Revision"</th>
                        <td>{move || edit_version_label.get()}</td>
                    </tr>
                    <tr>
                        <th scope="row">"Status"</th>
                        <td>{move || {
                            let status = edit_version_status.get();
                            let key = status.to_lowercase().replace(' ', "-");
                            view! { <span class=status_badge_class(&key)>{status}</span> }
                        }}</td>
                    </tr>
                </tbody>
            </table>
        </section>
    }
}
