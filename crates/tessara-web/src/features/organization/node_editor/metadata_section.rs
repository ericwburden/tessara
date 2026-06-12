//! Shared metadata editor section for organization node forms.

use super::MetadataFieldInput;
use crate::features::organization::types::NodeMetadataFieldSummary;
use leptos::prelude::*;
use std::collections::HashMap;

#[component]
pub(in crate::features::organization::node_editor) fn OrganizationNodeMetadataSection(
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
) -> impl IntoView {
    view! {
        <section class="form-section">
            <h3>"Metadata"</h3>
            {move || {
                let fields = metadata_fields.get();
                if fields.is_empty() {
                    view! { <p class="muted">"No metadata fields are configured for this node type."</p> }.into_any()
                } else {
                    view! {
                        <div class="form-grid">
                            {fields.into_iter().map(|field| {
                                view! {
                                    <MetadataFieldInput
                                        field
                                        metadata_values
                                        metadata_booleans
                                    />
                                }
                            }).collect_view()}
                        </div>
                    }
                    .into_any()
                }
            }}
        </section>
    }
}
