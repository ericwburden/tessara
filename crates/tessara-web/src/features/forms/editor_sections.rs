//! Form editor page sections.

use crate::features::organization::NodeTypeCatalogEntry;
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
