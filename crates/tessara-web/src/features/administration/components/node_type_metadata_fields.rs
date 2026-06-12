//! Node type metadata field list and editor components.

use super::node_type_metadata_field_actions::delete_node_type_metadata_field;
use super::node_type_metadata_field_sheet::NodeTypeMetadataFieldSheet;
use crate::features::organization::NodeMetadataFieldSummary;
use crate::features::shared::status_badge_class;
use crate::ui::{DataTable, DropdownMenu};
use crate::utils::metadata::metadata_label;
use icons::{Pencil, Plus, Search, Trash2};
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
    let table_searchable_fields = fields.clone();
    let card_searchable_fields = fields;
    let table_fields = move || {
        let query = search.get().trim().to_lowercase();
        table_searchable_fields
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
    let card_fields = move || {
        let query = search.get().trim().to_lowercase();
        card_searchable_fields
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
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th scope="col">"Field"</th>
                                        <th scope="col">"Key"</th>
                                        <th scope="col">"Type"</th>
                                        <th scope="col">"Required"</th>
                                        <th class="data-table__cell--center" scope="col">"Actions"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let visible_fields = table_fields();
                                        if visible_fields.is_empty() {
                                            view! {
                                                <tr>
                                                    <td class="data-table__empty" colspan="5">"No Metadata Fields to Display"</td>
                                                </tr>
                                            }
                                            .into_any()
                                        } else {
                                            visible_fields
                                                .into_iter()
                                                .map(|field| {
                                                    let edit_field = field.clone();
                                                    let delete_field = field.clone();
                                                    let row_label = field.label.clone();
                                                    view! {
                                                        <tr>
                                                            <th scope="row">{field.label}</th>
                                                            <td>{field.key}</td>
                                                            <td>{metadata_label(&field.field_type)}</td>
                                                            <td>{if field.required { "Yes" } else { "No" }}</td>
                                                            <td class="data-table__cell--center">
                                                                <DropdownMenu label=format!("Open actions for {row_label}")>
                                                                    <button
                                                                        class="dropdown-menu__item"
                                                                        type="button"
                                                                        role="menuitem"
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
                                                                        <Pencil class="dropdown-menu__item-icon"/>
                                                                        <span>"Edit Field"</span>
                                                                    </button>
                                                                    <button
                                                                        class="dropdown-menu__item"
                                                                        type="button"
                                                                        role="menuitem"
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
                                                                        <Trash2 class="dropdown-menu__item-icon"/>
                                                                        <span>"Delete Field"</span>
                                                                    </button>
                                                                </DropdownMenu>
                                                            </td>
                                                        </tr>
                                                    }
                                                })
                                                .collect_view()
                                                .into_any()
                                        }
                                    }}
                                </tbody>
                            </DataTable>
                        }
                        .into_any()
                    }}
                </div>
                <div class="forms-list-mobile-cards node-type-metadata-mobile-cards">
                    {if !has_fields {
                        view! { <p class="forms-list-mobile-empty">"No Metadata Fields to Display"</p> }.into_any()
                    } else {
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
