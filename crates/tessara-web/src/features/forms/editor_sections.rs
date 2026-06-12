//! Form editor page sections.

use crate::features::organization::NodeTypeCatalogEntry;
use crate::features::shared::status_badge_class;
use crate::ui::{InfoListTable, InfoRow};
use crate::utils::text::sentence_label;
use leptos::prelude::*;

#[component]
/// Renders the shared form identity and scope editor fields.
pub(in crate::features::forms) fn FormIdentityFields(
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
) -> impl IntoView {
    view! {
        <div class="form-grid">
            <label class="form-field form-field--wide" for="form-name">
                <span>"Form Name"</span>
                <input
                    id="form-name"
                    type="text"
                    autocomplete="off"
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                    required
                />
            </label>

            <label class="form-field" for="form-scope-node-type">
                <span>"Scope"</span>
                <select
                    id="form-scope-node-type"
                    prop:value=move || workflow_node_type_id.get()
                    on:change=move |event| workflow_node_type_id.set(event_target_value(&event))
                >
                    <option value="">"No scope"</option>
                    {move || {
                        let mut options = node_types.get();
                        options.sort_by(|left, right| {
                            left.singular_label
                                .cmp(&right.singular_label)
                                .then(left.name.cmp(&right.name))
                        });
                        options
                            .into_iter()
                            .map(|node_type| {
                                view! {
                                    <option value=node_type.id>{node_type.singular_label}</option>
                                }
                            })
                            .collect_view()
                    }}
                </select>
            </label>
        </div>
    }
}

#[component]
/// Renders the create-form initial version summary.
pub(in crate::features::forms) fn FormInitialVersionSummary(
    builder_field_count: Memo<usize>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Initial Version"</h3>
            <InfoListTable>
                <InfoRow label="Status" value="Draft"/>
                <tr>
                    <th scope="row">"Fields"</th>
                    <td>
                        {move || builder_field_count.get().to_string()}
                    </td>
                </tr>
            </InfoListTable>
        </section>
    }
}

#[component]
/// Renders the edit-form editable version summary.
pub(in crate::features::forms) fn FormEditableVersionSummary(
    edit_version_status: RwSignal<Option<String>>,
    builder_field_count: Memo<usize>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Editable Version"</h3>
            <InfoListTable>
                <tr>
                    <th scope="row">"Status"</th>
                    <td>
                        {move || {
                            edit_version_status
                                .get()
                                .map(|status| {
                                    view! {
                                        <span class=status_badge_class(&status)>
                                            {sentence_label(&status)}
                                        </span>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(|| view! { <span>"Draft"</span> }.into_any())
                        }}
                    </td>
                </tr>
                <tr>
                    <th scope="row">"Fields"</th>
                    <td>
                        {move || builder_field_count.get().to_string()}
                    </td>
                </tr>
            </InfoListTable>
        </section>
    }
}
