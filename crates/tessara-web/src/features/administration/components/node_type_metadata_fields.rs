//! Node type metadata field list and editor components.

use super::node_type_metadata_field_sheet::NodeTypeMetadataFieldSheet;
use super::node_type_metadata_field_table::NodeTypeMetadataFieldTable;
use super::node_type_metadata_mobile_cards::NodeTypeMetadataMobileCards;
use crate::features::organization::NodeMetadataFieldSummary;
use icons::{Plus, Search};
use leptos::prelude::*;

/// Renders the node type metadata field list and editor sheet.
#[component]
pub(super) fn NodeTypeMetadataList(
    node_type_id: String,
    fields: Vec<NodeMetadataFieldSummary>,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    #[cfg(not(feature = "hydrate"))]
    let _ = (&node_type_id, &on_metadata_changed);
    let node_type_id_value = RwSignal::new(node_type_id);
    let search = RwSignal::new(String::new());
    let editing_field_id = RwSignal::new(None::<String>);
    let field_label = RwSignal::new(String::new());
    let field_key = RwSignal::new(String::new());
    let field_type = RwSignal::new("text".to_string());
    let field_required = RwSignal::new(false);
    let is_saving_field = RwSignal::new(false);
    let field_message = RwSignal::new(None::<String>);
    let sheet_open = RwSignal::new(false);
    let has_fields = !fields.is_empty();
    let table_fields = fields.clone();
    let mobile_fields = fields;
    let clear_field_editor = move || {
        editing_field_id.set(None);
        field_label.set(String::new());
        field_key.set(String::new());
        field_type.set("text".to_string());
        field_required.set(false);
        field_message.set(None);
    };
    let open_new_field_sheet = move |_| {
        clear_field_editor();
        sheet_open.set(true);
    };

    view! {
        <section class="organization-detail-card node-type-detail-list node-type-detail-list--wide">
            <div class="node-type-detail-list__header">
                <h3>"Metadata Fields"</h3>
            </div>
            <div class="forms-list forms-list-responsive-table node-type-metadata-list">
                <div class="searchable-data-table">
                    <div class="searchable-data-table__toolbar forms-list__toolbar">
                        <label class="searchable-data-table__search searchable-data-table__control">
                            <Search class="searchable-data-table__control-icon"/>
                            <span class="sr-only">"Search metadata fields"</span>
                            <input
                                type="search"
                                placeholder="Search metadata"
                                prop:value=move || search.get()
                                on:input=move |event| search.set(event_target_value(&event))
                            />
                        </label>
                        <button
                            class="button button--compact button--secondary node-type-add-field-button"
                            type="button"
                            on:click=open_new_field_sheet
                        >
                            <Plus class="button__icon"/>
                            "Add Field"
                        </button>
                    </div>
                    <Show when=move || field_message.get().is_some() && !sheet_open.get()>
                        <p class="form-message">{move || field_message.get().unwrap_or_default()}</p>
                    </Show>
                    {if !has_fields {
                        view! { <p class="muted">"No metadata fields configured."</p> }.into_any()
                    } else {
                        view! {
                            <NodeTypeMetadataFieldTable
                                fields=table_fields.clone()
                                search
                                editing_field_id
                                field_label
                                field_key
                                field_type
                                field_required
                                field_message
                                sheet_open
                                clear_field_editor
                                on_metadata_changed
                            />
                        }
                        .into_any()
                    }}
                </div>
                <div class="forms-list-mobile-cards node-type-metadata-mobile-cards">
                    {if !has_fields {
                        view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
                    } else {
                        view! {
                            <NodeTypeMetadataMobileCards
                                fields=mobile_fields.clone()
                                search
                                editing_field_id
                                field_label
                                field_key
                                field_type
                                field_required
                                field_message
                                sheet_open
                                clear_field_editor
                                on_metadata_changed
                            />
                        }
                        .into_any()
                    }}
                </div>
            </div>
            <NodeTypeMetadataFieldSheet
                node_type_id=node_type_id_value
                editing_field_id
                field_label
                field_key
                field_type
                field_required
                is_saving_field
                field_message
                sheet_open
                clear_field_editor
                on_metadata_changed
            />
        </section>
    }
}
