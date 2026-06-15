//! Dataset editor aggregation controls.

use super::super::types::*;
use crate::ui::DataTable;
use crate::utils::text::sentence_label;
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetAggregationEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    aggregation: RwSignal<DatasetAggregationDraft>,
) -> impl IntoView {
    let projected_fields = move || fields.get();
    let group_fields = move || aggregation.get().group_fields;
    let node_grouped = move || {
        projected_fields().into_iter().any(|field| {
            group_fields().contains(&field.key) && field.source_field_key == "__node_id"
        })
    };

    view! {
        <section class="route-panel__section dataset-editor-section dataset-aggregation-section">
            <div class="dataset-editor-section__header">
                <h3>"Aggregation"</h3>
            </div>
            {move || if !aggregation_is_active(&aggregation.get()) {
                view! { <p class="muted">"No grouping or aggregation configured."</p> }.into_any()
            } else if !node_grouped() {
                view! {
                    <p class="form-status is-warning">
                        "This aggregation excludes Attached Node ID, so row-based visibility will not be applied to the materialized rows. Dataset visibility controls access to every aggregated row."
                    </p>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            <div class="dataset-aggregation-layout">
                <section class="dataset-aggregation-panel">
                    <h4>"Grouping"</h4>
                    <DataTable>
                        <thead>
                            <tr>
                                <th>"Group?"</th>
                                <th>"Field"</th>
                                <th>"Field Name"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || projected_fields().into_iter().map(|field| {
                                let field_key = field.key.clone();
                                let field_key_for_change = field_key.clone();
                                let is_grouped = aggregation.get().group_fields.contains(&field_key);
                                view! {
                                    <tr>
                                        <td>
                                            <input
                                                aria-label=format!("Group by {}", field.label)
                                                type="checkbox"
                                                prop:checked=is_grouped
                                                on:change=move |event| {
                                                    let checked = event_target_checked(&event);
                                                    aggregation.update(|draft| {
                                                        if checked {
                                                            if !draft.group_fields.contains(&field_key_for_change) {
                                                                draft.group_fields.push(field_key_for_change.clone());
                                                            }
                                                        } else {
                                                            draft.group_fields.retain(|key| key != &field_key_for_change);
                                                        }
                                                    });
                                                }
                                            />
                                        </td>
                                        <td>{field.label}</td>
                                        <td><code>{field_key}</code></td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </DataTable>
                </section>
                <section class="dataset-aggregation-panel">
                    <h4>"Pick Whole Row"</h4>
                    <div class="form-grid">
                        <label class="form-field">
                            <span>"Sort Field"</span>
                            <select
                                prop:value=move || aggregation.get().row_picker.map(|picker| picker.sort_field_key).unwrap_or_default()
                                on:change=move |event| {
                                    let value = event_target_value(&event);
                                    aggregation.update(|draft| {
                                        if value.is_empty() {
                                            draft.row_picker = None;
                                        } else {
                                            let direction = draft.row_picker.as_ref().map(|picker| picker.direction.clone()).unwrap_or_else(|| "lowest".into());
                                            draft.row_picker = Some(DatasetRowPickerDraft { sort_field_key: value, direction });
                                        }
                                    });
                                }
                            >
                                <option value="">"Do not pick a row"</option>
                                {move || projected_fields().into_iter().map(|field| {
                                    view! { <option value=field.key>{field.label}</option> }
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
            </div>
            <div class="dataset-editor-section__header">
                <h4>"Metrics"</h4>
                <button class="button button--secondary" type="button" on:click=move |_| {
                    aggregation.update(|draft| {
                        let next = draft.metrics.len() + 1;
                        draft.metrics.push(DatasetAggregationMetricDraft {
                            key: format!("metric_{next}"),
                            label: format!("Metric {next}"),
                            function: "count_rows".into(),
                            source_field_key: String::new(),
                        });
                    });
                }>"Add Metric"</button>
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
                                            <option value="count_values">"Count values"</option>
                                            <option value="sum">"Sum"</option>
                                            <option value="average">"Average"</option>
                                            <option value="min">"Min"</option>
                                            <option value="max">"Max"</option>
                                        </select>
                                    </td>
                                    <td>
                                        <select
                                            disabled=metric.function == "count_rows"
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
                                                view! { <option value=field.key>{format!("{} ({})", field.label, sentence_label(&field.source_field_key))}</option> }
                                            }).collect_view()}
                                        </select>
                                    </td>
                                    <td>
                                        <input
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
                                        <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                                            aggregation.update(|draft| {
                                                if index < draft.metrics.len() {
                                                    draft.metrics.remove(index);
                                                }
                                            });
                                        }>"Remove"</button>
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

fn aggregation_is_active(aggregation: &DatasetAggregationDraft) -> bool {
    !aggregation.group_fields.is_empty()
        || !aggregation.metrics.is_empty()
        || aggregation.row_picker.is_some()
}
