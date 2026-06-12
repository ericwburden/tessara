//! Mobile cards for node type metadata fields.

use super::node_type_metadata_field_actions::delete_node_type_metadata_field;
use crate::features::organization::NodeMetadataFieldSummary;
use crate::features::shared::status_badge_class;
use crate::utils::metadata::metadata_label;
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn NodeTypeMetadataMobileCards(
    fields: Vec<NodeMetadataFieldSummary>,
    search: RwSignal<String>,
    editing_field_id: RwSignal<Option<String>>,
    field_label: RwSignal<String>,
    field_key: RwSignal<String>,
    field_type: RwSignal<String>,
    field_required: RwSignal<bool>,
    field_message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    clear_field_editor: impl Fn() + 'static + Copy + Send + Sync,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) -> impl IntoView {
    let card_fields = move || {
        let query = search.get().trim().to_lowercase();
        fields
            .iter()
            .filter(|field| {
                query.is_empty()
                    || field.label.to_lowercase().contains(&query)
                    || field.key.to_lowercase().contains(&query)
                    || field.field_type.to_lowercase().contains(&query)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        {move || {
            let visible_fields = card_fields();
            if visible_fields.is_empty() {
                view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
            } else {
                visible_fields
                    .into_iter()
                    .map(|field| {
                        let edit_field = field.clone();
                        let delete_field = field.clone();
                        view! {
                            <article class="forms-list-mobile-card node-type-metadata-mobile-card">
                                <div class="forms-list-mobile-card__header">
                                    <div>
                                        <h3>{field.label}</h3>
                                        <span>{field.key}</span>
                                    </div>
                                    <span class=status_badge_class(if field.required { "active" } else { "inactive" })>
                                        {if field.required { "Required" } else { "Optional" }}
                                    </span>
                                </div>
                                <dl>
                                    <div>
                                        <dt>"Type"</dt>
                                        <dd>{metadata_label(&field.field_type)}</dd>
                                    </div>
                                </dl>
                                <div class="workflow-assignment-mobile-card__actions">
                                    <button
                                        class="button button--compact button--secondary"
                                        type="button"
                                        on:click=move |_| {
                                            editing_field_id.set(Some(edit_field.id.clone()));
                                            field_label.set(edit_field.label.clone());
                                            field_key.set(edit_field.key.clone());
                                            field_type.set(edit_field.field_type.clone());
                                            field_required.set(edit_field.required);
                                            field_message.set(None);
                                            sheet_open.set(true);
                                        }
                                    >
                                        "Edit Field"
                                    </button>
                                    <button
                                        class="button button--compact button--secondary"
                                        type="button"
                                        on:click=move |_| {
                                            delete_node_type_metadata_field(
                                                delete_field.id.clone(),
                                                field_message,
                                                sheet_open,
                                                clear_field_editor,
                                                on_metadata_changed,
                                            );
                                        }
                                    >
                                        "Delete Field"
                                    </button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()
                    .into_any()
            }
        }}
    }
}
