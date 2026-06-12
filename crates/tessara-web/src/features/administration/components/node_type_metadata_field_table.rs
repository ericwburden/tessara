//! Desktop table for node type metadata fields.

use super::node_type_metadata_field_actions::delete_node_type_metadata_field;
use crate::features::organization::NodeMetadataFieldSummary;
use crate::ui::{DataTable, DropdownMenu};
use crate::utils::metadata::metadata_label;
use icons::{Pencil, Trash2};
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
#[component]
pub(super) fn NodeTypeMetadataFieldTable(
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
    let table_fields = move || {
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
}
