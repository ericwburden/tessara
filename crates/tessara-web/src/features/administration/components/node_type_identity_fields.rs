//! Identity fields for node-type administration.

use crate::features::organization::NodeTypeDefinition;
use leptos::prelude::*;

#[component]
pub(super) fn NodeTypeIdentityFields(
    selected_detail: Option<NodeTypeDefinition>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    plural_label: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="form-grid administration-node-type-fields">
            <label class="form-field">
                <span>"Name"</span>
                <input
                    type="text"
                    placeholder="Program"
                    prop:value=move || name.get()
                    on:input=move |event| name.set(event_target_value(&event))
                />
            </label>
            <label class="form-field">
                <span>"Slug"</span>
                <input
                    type="text"
                    placeholder="program"
                    prop:value=move || slug.get()
                    on:input=move |event| slug.set(event_target_value(&event))
                />
            </label>
            <label class="form-field">
                <span>"Plural Label"</span>
                <input
                    type="text"
                    placeholder="Programs"
                    prop:value=move || plural_label.get()
                    on:input=move |event| plural_label.set(event_target_value(&event))
                />
            </label>
            {if let Some(detail) = selected_detail.as_ref() {
                let node_count = detail.node_count.to_string();
                let metadata_count = detail.metadata_fields.len().to_string();
                let scoped_form_count = detail.scoped_forms.len().to_string();
                view! {
                    <div class="node-type-count-badges" aria-label="Node type counts">
                        <span class="node-type-count-badge">
                            <strong>{node_count}</strong>
                            <span>"Nodes"</span>
                        </span>
                        <span class="node-type-count-badge">
                            <strong>{metadata_count}</strong>
                            <span>"Metadata Fields"</span>
                        </span>
                        <span class="node-type-count-badge">
                            <strong>{scoped_form_count}</strong>
                            <span>"Scoped Forms"</span>
                        </span>
                    </div>
                }
                .into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}
