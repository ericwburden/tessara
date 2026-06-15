//! Dataset editor aggregation controls.

use super::super::types::*;
use crate::ui::DataTable;
use icons::{ChevronsUpDown, Search};
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetAggregationEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    aggregation: RwSignal<DatasetAggregationDraft>,
) -> impl IntoView {
    let group_picker_open = RwSignal::new(false);
    let group_picker_search = RwSignal::new(String::new());
    let projected_fields = move || fields.get();
    let group_fields = move || aggregation.get().group_fields;
    let aggregation_enabled = move || aggregation.get().enabled;

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
                            });
                        }
                    >"None"</button>
                    <button
                        class=move || if aggregation_mode() == "row" { "dataset-aggregation-mode__option is-active" } else { "dataset-aggregation-mode__option" }
                        type="button"
                        on:click=move |_| {
                            let sort_field = first_available_sort_field(&projected_fields(), &[]);
                            aggregation.update(|draft| {
                                draft.enabled = true;
                                draft.metrics.clear();
                                draft.row_picker = Some(DatasetRowPickerDraft {
                                    sort_fields: if sort_field.is_empty() {
                                        Vec::new()
                                    } else {
                                        vec![DatasetRowPickerSortDraft {
                                            field_key: sort_field,
                                            direction: "lowest".into(),
                                        }]
                                    },
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
            } else {
                view! { <span></span> }.into_any()
            }}
            {move || if aggregation_enabled() {
                view! {
                    <div class="dataset-aggregation-layout">
                        <section class="dataset-aggregation-panel dataset-aggregation-panel--grouping">
                            <h4>"Grouping"</h4>
                            <div class="form-field">
                                <span>"Add Group Field"</span>
                                <div class=move || if group_picker_open.get() { "dataset-combobox is-open" } else { "dataset-combobox" }>
                                    <button
                                        class="dataset-combobox__trigger"
                                        type="button"
                                        aria-haspopup="listbox"
                                        aria-expanded=move || group_picker_open.get().to_string()
                                        on:click=move |_| group_picker_open.update(|open| *open = !*open)
                                    >
                                        <span class="truncate">"Select field..."</span>
                                        <ChevronsUpDown class="dataset-combobox__trigger-icon"/>
                                    </button>
                                    <button
                                        class="dataset-combobox__scrim"
                                        type="button"
                                        aria-label="Close group field picker"
                                        on:click=move |_| group_picker_open.set(false)
                                    ></button>
                                    <div class="dataset-combobox__content blurred-surface">
                                        <div class="dataset-combobox__search">
                                            <Search class="dataset-combobox__search-icon"/>
                                            <input
                                                class="dataset-combobox__input"
                                                type="search"
                                                placeholder="Search fields..."
                                                prop:value=move || group_picker_search.get()
                                                on:input=move |event| group_picker_search.set(event_target_value(&event))
                                            />
                                        </div>
                                        <div class="dataset-combobox__list" role="listbox">
                                            {move || {
                                                let query = group_picker_search.get().trim().to_lowercase();
                                                let fields = available_group_fields()
                                                    .into_iter()
                                                    .filter(|field| {
                                                        if query.is_empty() {
                                                            true
                                                        } else {
                                                            field_option_label(field).to_lowercase().contains(&query)
                                                        }
                                                    })
                                                    .collect::<Vec<_>>();
                                                if fields.is_empty() {
                                                    view! {
                                                        <div class="dataset-combobox__empty">"No fields found."</div>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div class="dataset-combobox__group">
                                                            {fields.into_iter().map(|field| {
                                                                let field_key = field.key.clone();
                                                                let label = field_option_label(&field);
                                                                view! {
                                                                    <button
                                                                        class="dataset-combobox__item"
                                                                        type="button"
                                                                        role="option"
                                                                        on:click=move |_| {
                                                                            aggregation.update(|draft| {
                                                                                if !draft.group_fields.contains(&field_key) {
                                                                                    draft.group_fields.push(field_key.clone());
                                                                                }
                                                                            });
                                                                            group_picker_search.set(String::new());
                                                                            group_picker_open.set(false);
                                                                        }
                                                                    >
                                                                        {label}
                                                                    </button>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    }.into_any()
                                                }
                                            }}
                                        </div>
                                    </div>
                                </div>
                            </div>
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
                                    <div class="dataset-editor-section__header dataset-editor-section__header--compact">
                                        <span class="muted">"Sort fields are applied in order."</span>
                                        <button
                                            class="button button--secondary button--compact"
                                            type="button"
                                            on:click=move |_| {
                                                let current = aggregation
                                                    .get()
                                                    .row_picker
                                                    .map(|picker| picker.sort_fields)
                                                    .unwrap_or_default();
                                                let field_key = first_available_sort_field(&projected_fields(), &current);
                                                if field_key.is_empty() {
                                                    return;
                                                }
                                                aggregation.update(|draft| {
                                                    let row_picker = draft.row_picker.get_or_insert_with(|| DatasetRowPickerDraft {
                                                        sort_fields: Vec::new(),
                                                    });
                                                    row_picker.sort_fields.push(DatasetRowPickerSortDraft {
                                                        field_key,
                                                        direction: "lowest".into(),
                                                    });
                                                });
                                            }
                                        >"Add Sort Field"</button>
                                    </div>
                                    <div class="table-wrap dataset-row-picker-table">
                                        <DataTable>
                                            <thead>
                                                <tr>
                                                    <th>"Order"</th>
                                                    <th>"Sort Field"</th>
                                                    <th>"Direction"</th>
                                                    <th>"Remove"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {move || {
                                                    let sort_fields = aggregation
                                                        .get()
                                                        .row_picker
                                                        .map(|picker| picker.sort_fields)
                                                        .unwrap_or_default();
                                                    let sort_count = sort_fields.len();
                                                    sort_fields.into_iter().enumerate().map(|(index, sort)| {
                                                        let current_field = sort.field_key.clone();
                                                        view! {
                                                            <tr>
                                                                <td>{index + 1}</td>
                                                                <td>
                                                                    <select
                                                                        prop:value=sort.field_key
                                                                        on:change=move |event| {
                                                                            let value = event_target_value(&event);
                                                                            aggregation.update(|draft| {
                                                                                if let Some(row_picker) = &mut draft.row_picker {
                                                                                    if let Some(sort) = row_picker.sort_fields.get_mut(index) {
                                                                                        sort.field_key = value;
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        {move || {
                                                                            let selected = aggregation
                                                                                .get()
                                                                                .row_picker
                                                                                .map(|picker| picker.sort_fields.into_iter().map(|sort| sort.field_key).collect::<Vec<_>>())
                                                                                .unwrap_or_default();
                                                                            projected_fields()
                                                                                .into_iter()
                                                                                .filter(|field| field.key == current_field || !selected.contains(&field.key))
                                                                                .map(|field| {
                                                                                    view! { <option value=field.key>{field_option_label(&field)}</option> }
                                                                                })
                                                                                .collect_view()
                                                                        }}
                                                                    </select>
                                                                </td>
                                                                <td>
                                                                    <select
                                                                        prop:value=sort.direction
                                                                        on:change=move |event| {
                                                                            let value = event_target_value(&event);
                                                                            aggregation.update(|draft| {
                                                                                if let Some(row_picker) = &mut draft.row_picker {
                                                                                    if let Some(sort) = row_picker.sort_fields.get_mut(index) {
                                                                                        sort.direction = value;
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        <option value="lowest">"Lowest / earliest first"</option>
                                                                        <option value="highest">"Highest / latest first"</option>
                                                                    </select>
                                                                </td>
                                                                <td>
                                                                    <button
                                                                        class="button button--secondary button--compact"
                                                                        disabled=sort_count <= 1
                                                                        type="button"
                                                                        on:click=move |_| {
                                                                            aggregation.update(|draft| {
                                                                                if let Some(row_picker) = &mut draft.row_picker {
                                                                                    if row_picker.sort_fields.len() > 1 && index < row_picker.sort_fields.len() {
                                                                                        row_picker.sort_fields.remove(index);
                                                                                    }
                                                                                }
                                                                            });
                                                                        }
                                                                    >"Remove"</button>
                                                                </td>
                                                            </tr>
                                                        }
                                                    }).collect_view()
                                                }}
                                            </tbody>
                                        </DataTable>
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

fn first_available_sort_field(
    fields: &[DatasetFieldDraft],
    selected: &[DatasetRowPickerSortDraft],
) -> String {
    fields
        .iter()
        .find(|field| !selected.iter().any(|sort| sort.field_key == field.key))
        .or_else(|| fields.first())
        .map(|field| field.key.clone())
        .unwrap_or_default()
}
