//! Dataset editor aggregation controls.

use super::super::types::*;
use crate::ui::{DataTable, SegmentedToggle, SegmentedToggleOption};
use icons::{ChevronsUpDown, Search};
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetAggregationEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    aggregation: RwSignal<DatasetAggregationDraft>,
) -> impl IntoView {
    let group_picker_open = RwSignal::new(false);
    let group_picker_search = RwSignal::new(String::new());
    let group_search_input = NodeRef::<leptos::html::Input>::new();
    let sort_picker_open = RwSignal::new(false);
    let sort_picker_search = RwSignal::new(String::new());
    let sort_search_input = NodeRef::<leptos::html::Input>::new();
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
    let sort_fields = move || {
        aggregation
            .get()
            .row_picker
            .map(|picker| picker.sort_fields)
            .unwrap_or_default()
    };
    let selected_sort_fields = move || {
        let selected = sort_fields();
        projected_fields()
            .into_iter()
            .filter(|field| selected.iter().any(|sort| sort.field_key == field.key))
            .collect::<Vec<_>>()
    };
    let available_sort_fields = move || {
        let selected = sort_fields();
        projected_fields()
            .into_iter()
            .filter(|field| !selected.iter().any(|sort| sort.field_key == field.key))
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
    let aggregation_mode_signal = Signal::derive(move || aggregation_mode().to_string());
    let row_direction_signal = Signal::derive(move || {
        aggregation
            .get()
            .row_picker
            .map(|picker| picker.direction)
            .unwrap_or_else(|| "lowest".into())
    });

    Effect::new(move |_| {
        if group_picker_open.get() {
            if let Some(input) = group_search_input.get() {
                let _ = input.focus();
            }
        }
    });
    Effect::new(move |_| {
        if sort_picker_open.get() {
            if let Some(input) = sort_search_input.get() {
                let _ = input.focus();
            }
        }
    });

    view! {
        <section class="route-panel__section dataset-editor-section dataset-aggregation-section">
            <div class="dataset-editor-section__header">
                <h3>"Aggregation"</h3>
            </div>
            <div class="dataset-aggregation-top-row">
                <span class="dataset-aggregation-top-row__label">"Aggregate by"</span>
                <SegmentedToggle
                    active=aggregation_mode_signal
                    aria_label="Aggregate by"
                    options=vec![
                        SegmentedToggleOption { value: "none", label: "None" },
                        SegmentedToggleOption { value: "row", label: "Row" },
                        SegmentedToggleOption { value: "metrics", label: "Field" },
                    ]
                    on_select=Callback::new(move |mode: String| {
                        match mode.as_str() {
                            "none" => {
                                aggregation.update(|draft| {
                                    draft.enabled = false;
                                    draft.group_fields.clear();
                                    draft.metrics.clear();
                                    draft.row_picker = None;
                                });
                            }
                            "row" => {
                                aggregation.update(|draft| {
                                    draft.enabled = true;
                                    draft.metrics.clear();
                                    draft.row_picker = Some(DatasetRowPickerDraft {
                                        sort_fields: Vec::new(),
                                        direction: "lowest".into(),
                                    });
                                });
                            }
                            "metrics" => {
                                aggregation.update(|draft| {
                                    draft.enabled = true;
                                    draft.row_picker = None;
                                });
                            }
                            _ => {}
                        }
                    })
                />
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
                                                node_ref=group_search_input
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
                                        view! { <span></span> }.into_any()
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
                        {move || if aggregation_mode_signal.get() == "row" {
                            view! {
                                <section class="dataset-aggregation-panel dataset-aggregation-panel--row">
                                    <h4>"Pick Whole Row"</h4>
                                    <p class="muted">"Sort fields are applied in order."</p>
                                    <div class="form-field dataset-row-picker-direction">
                                        <span>"Direction"</span>
                                        <SegmentedToggle
                                            active=row_direction_signal
                                            aria_label="Sort direction"
                                            class="segmented-toggle--direction"
                                            options=vec![
                                                SegmentedToggleOption { value: "lowest", label: "Lowest / earliest first" },
                                                SegmentedToggleOption { value: "highest", label: "Highest / latest first" },
                                            ]
                                            on_select=Callback::new(move |direction: String| {
                                                aggregation.update(|draft| {
                                                    let row_picker = draft.row_picker.get_or_insert_with(|| DatasetRowPickerDraft {
                                                        sort_fields: Vec::new(),
                                                        direction: "lowest".into(),
                                                    });
                                                    row_picker.direction = direction;
                                                });
                                            })
                                        />
                                    </div>
                                    <div class="form-field">
                                        <span>"Add Sort Field"</span>
                                        <div class=move || if sort_picker_open.get() { "dataset-combobox is-open" } else { "dataset-combobox" }>
                                            <button
                                                class="dataset-combobox__trigger"
                                                type="button"
                                                aria-haspopup="listbox"
                                                aria-expanded=move || sort_picker_open.get().to_string()
                                                on:click=move |_| sort_picker_open.update(|open| *open = !*open)
                                            >
                                                <span class="truncate">"Select field..."</span>
                                                <ChevronsUpDown class="dataset-combobox__trigger-icon"/>
                                            </button>
                                            <button
                                                class="dataset-combobox__scrim"
                                                type="button"
                                                aria-label="Close sort field picker"
                                                on:click=move |_| sort_picker_open.set(false)
                                            ></button>
                                            <div class="dataset-combobox__content blurred-surface">
                                                <div class="dataset-combobox__search">
                                                    <Search class="dataset-combobox__search-icon"/>
                                                    <input
                                                        class="dataset-combobox__input"
                                                        type="search"
                                                        placeholder="Search fields..."
                                                        node_ref=sort_search_input
                                                        prop:value=move || sort_picker_search.get()
                                                        on:input=move |event| sort_picker_search.set(event_target_value(&event))
                                                    />
                                                </div>
                                                <div class="dataset-combobox__list" role="listbox">
                                                    {move || {
                                                        let query = sort_picker_search.get().trim().to_lowercase();
                                                        let fields = available_sort_fields()
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
                                                                                        let row_picker = draft.row_picker.get_or_insert_with(|| DatasetRowPickerDraft {
                                                                                            sort_fields: Vec::new(),
                                                                                            direction: "lowest".into(),
                                                                                        });
                                                                                        if !row_picker.sort_fields.iter().any(|sort| sort.field_key == field_key) {
                                                                                            row_picker.sort_fields.push(DatasetRowPickerSortDraft {
                                                                                                field_key: field_key.clone(),
                                                                                            });
                                                                                        }
                                                                                    });
                                                                                    sort_picker_search.set(String::new());
                                                                                    sort_picker_open.set(false);
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
                                            let selected = selected_sort_fields();
                                            if selected.is_empty() {
                                                view! { <span></span> }.into_any()
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
                                                                                if let Some(row_picker) = &mut draft.row_picker {
                                                                                    row_picker.sort_fields.retain(|sort| sort.field_key != field_key_for_remove);
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
                            }.into_any()
                        } else {
                            view! { <span></span> }.into_any()
                        }}
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {move || if aggregation_mode_signal.get() == "metrics" {
                view! {
                    <section class="dataset-aggregation-panel dataset-aggregation-panel--metrics">
                        <div class="dataset-editor-section__header dataset-editor-section__header--compact">
                            <h4>"Metrics"</h4>
                            <button
                                class="button button--secondary"
                                disabled=move || !aggregation_enabled()
                                type="button"
                                on:click=move |_| {
                                    aggregation.update(|draft| {
                                        draft.row_picker = None;
                                        let next = next_metric_id(&draft.metrics);
                                        draft.metrics.push(DatasetAggregationMetricDraft {
                                            id: next,
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
                                    <For
                                        each=move || aggregation.get().metrics
                                        key=|metric| metric.id
                                        children=move |metric| {
                                            let metric_id = metric.id;
                                            let initial_key = metric.key.clone();
                                            let initial_label = metric.label.clone();
                                            view! {
                                                <tr>
                                                    <td>
                                                        <select
                                                            class="form-control"
                                                            disabled=move || !aggregation_enabled()
                                                            prop:value=move || aggregation
                                                                .get()
                                                                .metrics
                                                                .into_iter()
                                                                .find(|metric| metric.id == metric_id)
                                                                .map(|metric| metric.function)
                                                                .unwrap_or_else(|| "count_rows".into())
                                                            on:change=move |event| {
                                                                let value = event_target_value(&event);
                                                                aggregation.update(|draft| {
                                                                    if let Some(metric) = draft.metrics.iter_mut().find(|metric| metric.id == metric_id) {
                                                                        metric.function = value;
                                                                        if metric.function == "count_rows"
                                                                            || !metric_source_field_is_allowed(
                                                                                &metric.function,
                                                                                &metric.source_field_key,
                                                                                &projected_fields(),
                                                                            )
                                                                        {
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
                                                            class="form-control"
                                                            disabled=move || {
                                                                !aggregation_enabled()
                                                                    || aggregation
                                                                        .get()
                                                                        .metrics
                                                                        .into_iter()
                                                                        .find(|metric| metric.id == metric_id)
                                                                        .map(|metric| metric.function == "count_rows")
                                                                        .unwrap_or(true)
                                                            }
                                                            prop:value=move || aggregation
                                                                .get()
                                                                .metrics
                                                                .into_iter()
                                                                .find(|metric| metric.id == metric_id)
                                                                .map(|metric| metric.source_field_key)
                                                                .unwrap_or_default()
                                                            on:change=move |event| {
                                                                let value = event_target_value(&event);
                                                                aggregation.update(|draft| {
                                                                    if let Some(metric) = draft.metrics.iter_mut().find(|metric| metric.id == metric_id) {
                                                                        metric.source_field_key = value;
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <option value="">"Select field"</option>
                                                            {move || {
                                                                let function = aggregation
                                                                    .get()
                                                                    .metrics
                                                                    .into_iter()
                                                                    .find(|metric| metric.id == metric_id)
                                                                    .map(|metric| metric.function)
                                                                    .unwrap_or_else(|| "count_rows".into());
                                                                eligible_metric_fields(&function, &projected_fields())
                                                                    .into_iter()
                                                                    .map(|field| {
                                                                        view! { <option value=field.key>{field_option_label(&field)}</option> }
                                                                    })
                                                                    .collect_view()
                                                            }}
                                                        </select>
                                                    </td>
                                                    <td>
                                                        <input
                                                            class="form-control"
                                                            disabled=move || !aggregation_enabled()
                                                            value=initial_key
                                                            on:change=move |event| {
                                                                let value = event_target_value(&event);
                                                                aggregation.update(|draft| {
                                                                    if let Some(metric) = draft.metrics.iter_mut().find(|metric| metric.id == metric_id) {
                                                                        metric.key = value;
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </td>
                                                    <td>
                                                        <input
                                                            class="form-control"
                                                            disabled=move || !aggregation_enabled()
                                                            value=initial_label
                                                            on:change=move |event| {
                                                                let value = event_target_value(&event);
                                                                aggregation.update(|draft| {
                                                                    if let Some(metric) = draft.metrics.iter_mut().find(|metric| metric.id == metric_id) {
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
                                                                    draft.metrics.retain(|metric| metric.id != metric_id);
                                                                });
                                                            }
                                                        >"Remove"</button>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </DataTable>
                        </div>
                    </section>
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

fn next_metric_id(metrics: &[DatasetAggregationMetricDraft]) -> u64 {
    metrics.iter().map(|metric| metric.id).max().unwrap_or(0) + 1
}

fn eligible_metric_fields(function: &str, fields: &[DatasetFieldDraft]) -> Vec<DatasetFieldDraft> {
    fields
        .iter()
        .filter(|field| metric_field_type_is_allowed(function, &field.field_type))
        .cloned()
        .collect()
}

fn metric_source_field_is_allowed(
    function: &str,
    source_field_key: &str,
    fields: &[DatasetFieldDraft],
) -> bool {
    if source_field_key.trim().is_empty() {
        return false;
    }
    fields.iter().any(|field| {
        field.key == source_field_key && metric_field_type_is_allowed(function, &field.field_type)
    })
}

fn metric_field_type_is_allowed(function: &str, field_type: &str) -> bool {
    match function {
        "count_rows" => false,
        "count_values" => true,
        "sum" | "average" => field_type == "number",
        "min" | "max" => matches!(
            field_type,
            "number" | "date" | "datetime" | "timestamp" | "single_choice" | "multi_choice"
        ),
        _ => false,
    }
}
