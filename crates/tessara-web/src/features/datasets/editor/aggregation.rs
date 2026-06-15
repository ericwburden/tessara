//! Dataset editor aggregation controls.

use super::super::types::*;
use crate::ui::DataTable;
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetAggregationEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    aggregation: RwSignal<DatasetAggregationDraft>,
) -> impl IntoView {
    let group_picker_value = RwSignal::new(String::new());
    let projected_fields = move || fields.get();
    let group_fields = move || aggregation.get().group_fields;
    let aggregation_enabled = move || aggregation.get().enabled;
    let node_field_key = move || {
        projected_fields()
            .into_iter()
            .find(|field| field.source_field_key == "__node_id")
            .map(|field| field.key)
    };
    let node_grouped = move || {
        node_field_key().is_some_and(|node_key| aggregation.get().group_fields.contains(&node_key))
    };

    Effect::new(move |_| {
        let Some(node_key) = node_field_key() else {
            return;
        };
        let draft = aggregation.get();
        if draft.enabled
            && !draft.node_grouping_manually_removed
            && !draft.group_fields.contains(&node_key)
        {
            aggregation.update(|draft| {
                draft.group_fields.insert(0, node_key);
            });
        }
    });

    let selected_group_fields = move || {
        let selected = group_fields();
        projected_fields()
            .into_iter()
            .filter(|field| selected.contains(&field.key))
            .collect::<Vec<_>>()
    };
    let available_group_fields = move || {
        let selected = group_fields();
        projected_fields()
            .into_iter()
            .filter(|field| !selected.contains(&field.key))
            .collect::<Vec<_>>()
    };
    let aggregation_mode = move || {
        let draft = aggregation.get();
        if !draft.enabled {
            "none"
        } else if draft.row_picker.is_some() {
            "row"
        } else {
            "metrics"
        }
    };

    view! {
        <section class="route-panel__section dataset-editor-section dataset-aggregation-section">
            <div class="dataset-editor-section__header">
                <h3>"Aggregation"</h3>
            </div>
            <div class="dataset-aggregation-top-row">
                <span class="dataset-aggregation-top-row__label">"Aggregate by"</span>
                <div class="dataset-aggregation-mode" role="group" aria-label="Aggregate by">
                    <button
                        class=move || if aggregation_mode() == "none" { "dataset-aggregation-mode__option is-active" } else { "dataset-aggregation-mode__option" }
                        type="button"
                        on:click=move |_| {
                            aggregation.update(|draft| {
                                draft.enabled = false;
                                draft.group_fields.clear();
                                draft.metrics.clear();
                                draft.row_picker = None;
                                draft.node_grouping_manually_removed = false;
                            });
                        }
                    >"None"</button>
                    <button
                        class=move || if aggregation_mode() == "row" { "dataset-aggregation-mode__option is-active" } else { "dataset-aggregation-mode__option" }
                        type="button"
                        on:click=move |_| {
                            let sort_field_key = projected_fields()
                                .first()
                                .map(|field| field.key.clone())
                                .unwrap_or_default();
                            aggregation.update(|draft| {
                                draft.enabled = true;
                                draft.metrics.clear();
                                draft.row_picker = Some(DatasetRowPickerDraft {
                                    sort_field_key,
                                    direction: "lowest".into(),
                                });
                            });
                        }
                    >"Row"</button>
                    <button
                        class=move || if aggregation_mode() == "metrics" { "dataset-aggregation-mode__option is-active" } else { "dataset-aggregation-mode__option" }
                        type="button"
                        on:click=move |_| {
                            aggregation.update(|draft| {
                                draft.enabled = true;
                                draft.row_picker = None;
                            });
                        }
                    >"Field"</button>
                </div>
            </div>
            {move || if !aggregation_enabled() {
                view! {
                    <p class="muted">"Aggregation is off. Rows pass through with the selected fields unchanged."</p>
                }.into_any()
            } else if !node_grouped() {
                view! {
                    <p class="form-status is-warning">
                        "Attached Node ID is not grouped, so row-based visibility will not be applied to the materialized rows. Dataset visibility controls access to every aggregated row."
                    </p>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {move || if aggregation_enabled() {
                view! {
                    <div class="dataset-aggregation-layout">
                        <section class="dataset-aggregation-panel dataset-aggregation-panel--grouping">
                            <h4>"Grouping"</h4>
                            <label class="form-field">
                                <span>"Add Group Field"</span>
                                <input
                                    class="dataset-aggregation-field-picker"
                                    list="dataset-aggregation-group-fields"
                                    placeholder="Select field..."
                                    prop:value=move || group_picker_value.get()
                                    on:input=move |event| {
                                        let typed_value = event_target_value(&event);
                                        if let Some(value) =
                                            resolve_field_picker_value(&available_group_fields(), &typed_value)
                                        {
                                            aggregation.update(|draft| {
                                                if !draft.group_fields.contains(&value) {
                                                    draft.group_fields.push(value.clone());
                                                }
                                                if node_field_key().is_some_and(|node_key| node_key == value) {
                                                    draft.node_grouping_manually_removed = false;
                                                }
                                            });
                                            group_picker_value.set(String::new());
                                        } else {
                                            group_picker_value.set(typed_value);
                                        }
                                    }
                                    on:change=move |event| {
                                        let selected_key =
                                            resolve_field_picker_value(&available_group_fields(), &event_target_value(&event));
                                        if let Some(value) = selected_key {
                                            aggregation.update(|draft| {
                                                if !draft.group_fields.contains(&value) {
                                                    draft.group_fields.push(value.clone());
                                                }
                                                if node_field_key().is_some_and(|node_key| node_key == value) {
                                                    draft.node_grouping_manually_removed = false;
                                                }
                                            });
                                            group_picker_value.set(String::new());
                                        }
                                    }
                                />
                                <datalist id="dataset-aggregation-group-fields">
                                    {move || available_group_fields().into_iter().map(|field| {
                                        view! { <option value=field_option_label(&field)></option> }
                                    }).collect_view()}
                                </datalist>
                            </label>
                            <div class="dataset-aggregation-selected-list">
                                {move || {
                                    let selected = selected_group_fields();
                                    if selected.is_empty() {
                                        view! { <p class="muted">"No group fields selected."</p> }.into_any()
                                    } else {
                                        view! {
                                            <ul>
                                                {selected.into_iter().map(|field| {
                                                    let field_key = field.key.clone();
                                                    let field_key_for_remove = field_key.clone();
                                                    view! {
                                                        <li>
                                                            <span>
                                                                <strong>{field.label}</strong>
                                                                <code>{field_key}</code>
                                                            </span>
                                                            <button
                                                                class="button button--secondary button--compact"
                                                                type="button"
                                                                on:click=move |_| {
                                                                    aggregation.update(|draft| {
                                                                        draft.group_fields.retain(|key| key != &field_key_for_remove);
                                                                        if node_field_key().is_some_and(|node_key| node_key == field_key_for_remove) {
                                                                            draft.node_grouping_manually_removed = true;
                                                                        }
                                                                    });
                                                                }
                                                            >"Remove"</button>
                                                        </li>
                                                    }
                                                }).collect_view()}
                                            </ul>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </section>
                        {move || if aggregation_mode() == "row" {
                            view! {
                                <section class="dataset-aggregation-panel dataset-aggregation-panel--row">
                                    <h4>"Pick Whole Row"</h4>
                                    <div class="form-grid">
                                        <label class="form-field">
                                            <span>"Sort Field"</span>
                                            <select
                                                prop:value=move || aggregation.get().row_picker.map(|picker| picker.sort_field_key).unwrap_or_default()
                                                on:change=move |event| {
                                                    let value = event_target_value(&event);
                                                    aggregation.update(|draft| {
                                                        if let Some(row_picker) = &mut draft.row_picker {
                                                            row_picker.sort_field_key = value;
                                                        }
                                                    });
                                                }
                                            >
                                                {move || projected_fields().into_iter().map(|field| {
                                                    view! { <option value=field.key>{field_option_label(&field)}</option> }
                                                }).collect_view()}
                                            </select>
                                        </label>
                                        <label class="form-field">
                                            <span>"Direction"</span>
                                            <select
                                                prop:value=move || aggregation.get().row_picker.map(|picker| picker.direction).unwrap_or_else(|| "lowest".into())
                                                on:change=move |event| {
                                                    let value = event_target_value(&event);
                                                    aggregation.update(|draft| {
                                                        if let Some(row_picker) = &mut draft.row_picker {
                                                            row_picker.direction = value;
                                                        }
                                                    });
                                                }
                                            >
                                                <option value="lowest">"Lowest / earliest first"</option>
                                                <option value="highest">"Highest / latest first"</option>
                                            </select>
                                        </label>
                                    </div>
                                </section>
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {move || if aggregation_mode() == "metrics" {
                view! {
                    <div class="dataset-editor-section__header">
                        <h4>"Metrics"</h4>
                        <button
                            class="button button--secondary"
                            disabled=move || !aggregation_enabled()
                            type="button"
                            on:click=move |_| {
                                aggregation.update(|draft| {
                                    draft.row_picker = None;
                                    let next = draft.metrics.len() + 1;
                                    draft.metrics.push(DatasetAggregationMetricDraft {
                                        key: format!("metric_{next}"),
                                        label: format!("Metric {next}"),
                                        function: "count_rows".into(),
                                        source_field_key: String::new(),
                                    });
                                });
                            }
                        >"Add Metric"</button>
                    </div>
                    <div class="table-wrap dataset-aggregation-table">
                        <DataTable>
                            <thead>
                                <tr>
                                    <th>"Function"</th>
                                    <th>"Source Field"</th>
                                    <th>"Output Key"</th>
                                    <th>"Output Label"</th>
                                    <th>"Remove"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {move || aggregation.get().metrics.into_iter().enumerate().map(|(index, metric)| {
                                    view! {
                                        <tr>
                                            <td>
                                                <select
                                                    disabled=move || !aggregation_enabled()
                                                    prop:value=metric.function.clone()
                                                    on:change=move |event| {
                                                        let value = event_target_value(&event);
                                                        aggregation.update(|draft| {
                                                            if let Some(metric) = draft.metrics.get_mut(index) {
                                                                metric.function = value;
                                                                if metric.function == "count_rows" {
                                                                    metric.source_field_key.clear();
                                                                }
                                                            }
                                                        });
                                                    }
                                                >
                                                    <option value="count_rows">"Count rows"</option>
                                                    <option value="count_values">"Count values present"</option>
                                                    <option value="sum">"Sum"</option>
                                                    <option value="average">"Average"</option>
                                                    <option value="min">"Min"</option>
                                                    <option value="max">"Max"</option>
                                                </select>
                                            </td>
                                            <td>
                                                <select
                                                    disabled=move || !aggregation_enabled() || metric.function == "count_rows"
                                                    prop:value=metric.source_field_key.clone()
                                                    on:change=move |event| {
                                                        let value = event_target_value(&event);
                                                        aggregation.update(|draft| {
                                                            if let Some(metric) = draft.metrics.get_mut(index) {
                                                                metric.source_field_key = value;
                                                            }
                                                        });
                                                    }
                                                >
                                                    <option value="">"Select field"</option>
                                                    {move || projected_fields().into_iter().map(|field| {
                                                        view! { <option value=field.key>{field_option_label(&field)}</option> }
                                                    }).collect_view()}
                                                </select>
                                            </td>
                                            <td>
                                                <input
                                                    disabled=move || !aggregation_enabled()
                                                    prop:value=metric.key
                                                    on:input=move |event| {
                                                        let value = event_target_value(&event);
                                                        aggregation.update(|draft| {
                                                            if let Some(metric) = draft.metrics.get_mut(index) {
                                                                metric.key = value;
                                                            }
                                                        });
                                                    }
                                                />
                                            </td>
                                            <td>
                                                <input
                                                    disabled=move || !aggregation_enabled()
                                                    prop:value=metric.label
                                                    on:input=move |event| {
                                                        let value = event_target_value(&event);
                                                        aggregation.update(|draft| {
                                                            if let Some(metric) = draft.metrics.get_mut(index) {
                                                                metric.label = value;
                                                            }
                                                        });
                                                    }
                                                />
                                            </td>
                                            <td>
                                                <button
                                                    class="button button--secondary button--compact"
                                                    disabled=move || !aggregation_enabled()
                                                    type="button"
                                                    on:click=move |_| {
                                                        aggregation.update(|draft| {
                                                            if index < draft.metrics.len() {
                                                                draft.metrics.remove(index);
                                                            }
                                                        });
                                                    }
                                                >"Remove"</button>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </DataTable>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </section>
    }
}

fn field_option_label(field: &DatasetFieldDraft) -> String {
    format!("{} ({})", field.label, field.key)
}

fn resolve_field_picker_value(fields: &[DatasetFieldDraft], value: &str) -> Option<String> {
    fields
        .iter()
        .find(|field| field_option_label(field) == value || field.key == value)
        .map(|field| field.key.clone())
}
