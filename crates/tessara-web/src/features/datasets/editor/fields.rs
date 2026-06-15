//! Dataset editor projected field picker.

use super::super::types::*;
use super::source_field_actions::canonical_field_key;
use super::source_options::{source_display_name, source_field_options};
use crate::ui::DataTable;
use crate::utils::text::sentence_label;
use icons::ChevronDown;
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub(crate) fn DatasetFieldsEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Fields"</h3>
            </div>
            <div class="dataset-field-picker">
                {move || {
                    sources.get().into_iter().map(|source| {
                        let source_alias = source.source_alias.clone();
                        let source_options = source_field_options(
                            &sources.get(),
                            &forms.get(),
                            &rendered_forms.get(),
                            &source_alias,
                        );
                        let all_count = source_options.len();
                        let all_included = all_count > 0
                            && source_options.iter().all(|option| {
                                fields.get().iter().any(|field| {
                                    field.source_alias == source_alias && field.source_field_key == option.key
                                })
                            });
                        let included_count = fields.get().iter().filter(|field| field.source_alias == source_alias).count();
                        let source_heading = source_display_name(&source, &forms.get(), &datasets.get());
                        let source_alias_for_all = source_alias.clone();
                        let source_options_for_all = source_options.clone();
                        view! {
                            <details class="dataset-field-picker__source">
                                <summary class="dataset-field-picker__summary">
                                    <span class="dataset-field-picker__summary-title">{format!("{} ({})", source_alias.clone(), source_heading)}</span>
                                    <span class="dataset-field-picker__summary-meta">
                                        <small>{format!("{included_count} of {all_count} fields included")}</small>
                                        <ChevronDown class="dataset-field-picker__summary-icon"/>
                                    </span>
                                </summary>
                                <div class="table-wrap dataset-fields-table dataset-field-picker__table">
                                    <DataTable>
                                        <thead>
                                            <tr>
                                                <th scope="col">
                                                    <label class="dataset-field-picker__include-all">
                                                        <input
                                                            aria-label=format!("Include all fields from {}", source_alias)
                                                            type="checkbox"
                                                            prop:checked=all_included
                                                            on:change=move |event| {
                                                                let is_checked = event_target_checked(&event);
                                                                fields.update(|items| {
                                                                    if is_checked {
                                                                        for option in &source_options_for_all {
                                                                            if !items.iter().any(|field| {
                                                                                field.source_alias == source_alias_for_all
                                                                                    && field.source_field_key == option.key
                                                                            }) {
                                                                                items.push(DatasetFieldDraft {
                                                                                    key: canonical_field_key(&source_alias_for_all, &option.key),
                                                                                    label: option.label.clone(),
                                                                                    source_alias: source_alias_for_all.clone(),
                                                                                    source_field_key: option.key.clone(),
                                                                                    field_type: option.field_type.clone(),
                                                                                });
                                                                            }
                                                                        }
                                                                    } else {
                                                                        items.retain(|field| field.source_alias != source_alias_for_all);
                                                                    }
                                                                });
                                                            }
                                                        />
                                                        <span>"Include?"</span>
                                                    </label>
                                                </th>
                                                <th scope="col">"Display Label"</th>
                                                <th scope="col">"Form Label"</th>
                                                <th scope="col">"Field Name"</th>
                                                <th scope="col">"Data Type"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {source_options.into_iter().map(|option| {
                                                let option_key = option.key.clone();
                                                let field_key = canonical_field_key(&source_alias, &option_key);
                                                let projected = fields.get().into_iter().find(|field| {
                                                    field.source_alias == source_alias && field.source_field_key == option_key
                                                });
                                                let included = projected.is_some();
                                                let display_label = projected
                                                    .as_ref()
                                                    .map(|field| field.label.clone())
                                                    .unwrap_or_else(|| option.label.clone());
                                                let source_alias_for_include = source_alias.clone();
                                                let source_alias_for_label = source_alias.clone();
                                                let option_key_for_include = option_key.clone();
                                                let option_key_for_label = option_key.clone();
                                                let option_label_for_include = option.label.clone();
                                                let option_field_type_for_include = option.field_type.clone();
                                                let field_key_for_include = field_key.clone();
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <input
                                                                aria-label=format!("Include {}", option.label)
                                                                type="checkbox"
                                                                prop:checked=included
                                                                on:change=move |event| {
                                                                    let is_checked = event_target_checked(&event);
                                                                    fields.update(|items| {
                                                                        let existing_index = items.iter().position(|field| {
                                                                            field.source_alias == source_alias_for_include
                                                                                && field.source_field_key == option_key_for_include
                                                                        });
                                                                        match (is_checked, existing_index) {
                                                                            (true, None) => items.push(DatasetFieldDraft {
                                                                                key: field_key_for_include.clone(),
                                                                                label: option_label_for_include.clone(),
                                                                                source_alias: source_alias_for_include.clone(),
                                                                                source_field_key: option_key_for_include.clone(),
                                                                                field_type: option_field_type_for_include.clone(),
                                                                            }),
                                                                            (false, Some(index)) => {
                                                                                items.remove(index);
                                                                            }
                                                                            _ => {}
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                        </td>
                                                        <td>
                                                            <input
                                                                aria-label=format!("Display label for {}", option.label)
                                                                class="dataset-field-picker__label-input"
                                                                disabled=!included
                                                                prop:value=display_label
                                                                on:input=move |event| {
                                                                    let value = event_target_value(&event);
                                                                    fields.update(|items| {
                                                                        if let Some(field) = items.iter_mut().find(|field| {
                                                                            field.source_alias == source_alias_for_label
                                                                                && field.source_field_key == option_key_for_label
                                                                        }) {
                                                                            field.label = value;
                                                                        }
                                                                    });
                                                                }
                                                            />
                                                        </td>
                                                        <td>{option.label}</td>
                                                        <td>
                                                            <code>{field_key}</code>
                                                        </td>
                                                        <td>{sentence_label(&option.field_type)}</td>
                                                    </tr>
                                                }
                                            }).collect_view()}
                                        </tbody>
                                    </DataTable>
                                </div>
                            </details>
                        }
                    }).collect_view()
                }}
            </div>
        </section>
    }
}
