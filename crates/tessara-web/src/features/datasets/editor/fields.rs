//! Dataset editor projected field components.

use super::super::types::*;
use super::helpers::{confirm_action, field_metadata};
use crate::ui::DataTable;
use crate::utils::text::sentence_label;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub(crate) fn DatasetFieldsEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Fields"</h3>
                <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                    let next = fields.get().len() + 1;
                    fields.update(|items| items.push(DatasetFieldDraft {
                        key: format!("field_{next}"),
                        label: format!("Field {next}"),
                        source_alias: sources.get().first().map(|source| source.source_alias.clone()).unwrap_or_default(),
                        source_field_key: String::new(),
                    }));
                    designer_selection.set(DatasetDesignerSelection::Field(next - 1));
                    designer_sheet_open.set(true);
                }>"Add Field"</button>
            </div>
            <div class="table-wrap dataset-fields-table">
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Source"</th>
                            <th scope="col">"Field"</th>
                            <th scope="col">"Form Field Label"</th>
                            <th scope="col">"Source Field"</th>
                            <th scope="col">"Data Type"</th>
                            <th scope="col">"Remove"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || fields.get().into_iter().enumerate().map(|(index, field)| {
                            let metadata = field_metadata(&field, &sources.get(), &forms.get(), &rendered_forms.get());
                            view! {
                                <tr class=move || if designer_selection.get() == DatasetDesignerSelection::Field(index) { "is-selected" } else { "" }>
                                    <td>{field.source_alias.clone()}</td>
                                    <th scope="row">
                                        <button
                                            class="link-button"
                                            type="button"
                                            on:click=move |_| {
                                                designer_selection.set(DatasetDesignerSelection::Field(index));
                                                designer_sheet_open.set(true);
                                            }
                                        >
                                            {field.label.clone()}
                                        </button>
                                        <span class="data-table__secondary-text">{field.key.clone()}</span>
                                    </th>
                                    <td>{metadata.label}</td>
                                    <td>{field.source_field_key.clone()}</td>
                                    <td>{sentence_label(&metadata.field_type)}</td>
                                    <td>
                                        <button
                                            class="button button--secondary button--compact"
                                            type="button"
                                            on:click=move |_| {
                                                if confirm_action("Remove this projected field?") {
                                                    fields.update(|items| {
                                                        if index < items.len() {
                                                            items.remove(index);
                                                        }
                                                    });
                                                    designer_selection.set(DatasetDesignerSelection::Operation);
                                                }
                                            }
                                        >
                                            "Remove"
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </DataTable>
            </div>
        </section>
    }
}
