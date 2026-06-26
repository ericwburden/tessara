//! Dataset editor aggregation controls.

use super::super::types::*;
use crate::ui::{
    Combobox, ComboboxOption, DataTable, DraggablePanelList, DraggablePanelListAnchor,
    DraggablePanelListDraggable, DraggablePanelListDropZone, DraggablePanelListItem,
    DraggablePanelListMove, SegmentedToggle, SegmentedToggleOption, empty_view,
};
use icons::{ChevronsUpDown, Trash2};
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetAggregationEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    aggregation: Signal<DatasetAggregationDraft>,
    on_aggregation_change: Callback<DatasetAggregationDraft>,
    #[prop(optional)] embedded: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(embedded);
    let section_class = if embedded {
        "route-panel__section dataset-editor-section dataset-aggregation-section dataset-editor-section--embedded"
    } else {
        "route-panel__section dataset-editor-section dataset-aggregation-section"
    };
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
        selected
            .into_iter()
            .filter_map(|sort| {
                projected_fields()
                    .into_iter()
                    .find(|field| field.key == sort.field_key)
            })
            .collect::<Vec<_>>()
    };
    let selected_sort_items = Signal::derive(move || {
        sort_fields()
            .into_iter()
            .map(|sort| DraggablePanelListItem { id: sort.field_key })
            .collect::<Vec<_>>()
    });
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
    let available_group_options =
        Signal::derive(move || field_combobox_options(available_group_fields()));
    let available_sort_options =
        Signal::derive(move || field_combobox_options(available_sort_fields()));

    view! {
        <section class=section_class>
            {if embedded {
                view! { <span></span> }.into_any()
            } else {
                view! {
                    <div class="dataset-editor-section__header">
                        <button
                            class="dataset-editor-section__header dataset-sql-header dataset-editor-section__collapse"
                            type="button"
                            aria-expanded=move || is_open.get().to_string()
                            on:click=move |_| is_open.update(|open| *open = !*open)
                        >
                            <h3>"Aggregation"</h3>
                            <ChevronsUpDown class="dataset-sql-header__icon"/>
                        </button>
                    </div>
                }.into_any()
            }}
            <div class=move || if is_open.get() { "dataset-aggregation-content" } else { "dataset-aggregation-content is-collapsed" }>
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
                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                    draft.enabled = false;
                                    draft.group_fields.clear();
                                    draft.metrics.clear();
                                    draft.row_picker = None;
                                });
                            }
                            "row" => {
                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                    draft.enabled = true;
                                    draft.metrics.clear();
                                    draft.row_picker = Some(DatasetRowPickerDraft {
                                        sort_fields: Vec::new(),
                                        direction: "lowest".into(),
                                    });
                                });
                            }
                            "metrics" => {
                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
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
                                <Combobox
                                    options=available_group_options
                                    placeholder="Select field..."
                                    search_placeholder="Search fields..."
                                    empty_label="No fields found."
                                    aria_label="Add group field"
                                    on_select=Callback::new(move |field_key: String| {
                                        mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                            if !draft.group_fields.contains(&field_key) {
                                                draft.group_fields.push(field_key);
                                            }
                                        });
                                    })
                                />
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
                                                                class="icon-button icon-button--compact-control"
                                                                type="button"
                                                                aria-label="Remove group field"
                                                                title="Remove group field"
                                                                on:click=move |_| {
                                                                    mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                                                        draft.group_fields.retain(|key| key != &field_key_for_remove);
                                                                    });
                                                                }
                                                            >
                                                                <Trash2 class="icon-button__icon"/>
                                                            </button>
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
                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
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
                                        <Combobox
                                            options=available_sort_options
                                            placeholder="Select field..."
                                            search_placeholder="Search fields..."
                                            empty_label="No fields found."
                                            aria_label="Add sort field"
                                            on_select=Callback::new(move |field_key: String| {
                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                                    let row_picker = draft.row_picker.get_or_insert_with(|| DatasetRowPickerDraft {
                                                        sort_fields: Vec::new(),
                                                        direction: "lowest".into(),
                                                    });
                                                    if !row_picker.sort_fields.iter().any(|sort| sort.field_key == field_key) {
                                                        row_picker.sort_fields.push(DatasetRowPickerSortDraft {
                                                            field_key,
                                                        });
                                                    }
                                                });
                                            })
                                        />
                                    </div>
                                    <div class="dataset-aggregation-selected-list">
                                        {move || {
                                            let selected = selected_sort_fields();
                                            if selected.is_empty() {
                                                view! { <span></span> }.into_any()
                                            } else {
                                                view! {
                                                    <DraggablePanelList
                                                        list_id="aggregation-row-picker-sort-fields"
                                                        items=selected_sort_items
                                                        container_class="dataset-aggregation-selected-list__draggable"
                                                        list_class="dataset-aggregation-selected-list__items"
                                                        draggable_class="dataset-aggregation-selected-list__item"
                                                        drop_zone_class="dataset-aggregation-selected-list__drop-zone"
                                                        drag_handle_title="Drag sort field to reorder"
                                                        data_transfer_type="application/x-tessara-aggregation-sort-field"
                                                        render_drop_zone=Callback::new(move |_drop_zone: DraggablePanelListDropZone| {
                                                            empty_view()
                                                        })
                                                        render_draggable=Callback::new(move |draggable: DraggablePanelListDraggable| {
                                                            let Some(field) = selected
                                                                .iter()
                                                                .find(|field| field.key == draggable.id)
                                                                .cloned()
                                                            else {
                                                                return empty_view();
                                                            };
                                                            let field_key = field.key.clone();
                                                            let field_key_for_remove = field_key.clone();
                                                            view! {
                                                                <div class="dataset-aggregation-selected-list__row">
                                                                    <span>
                                                                        <strong>{field.label}</strong>
                                                                        <code>{field_key}</code>
                                                                    </span>
                                                                    <button
                                                                        class="icon-button icon-button--compact-control"
                                                                        type="button"
                                                                        aria-label="Remove sort field"
                                                                        title="Remove sort field"
                                                                        on:click=move |_| {
                                                                            mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                                                                if let Some(row_picker) = &mut draft.row_picker {
                                                                                    row_picker.sort_fields.retain(|sort| sort.field_key != field_key_for_remove);
                                                                                }
                                                                            });
                                                                        }
                                                                    >
                                                                        <Trash2 class="icon-button__icon"/>
                                                                    </button>
                                                                </div>
                                                            }.into_any()
                                                        })
                                                        on_move=Callback::new(move |move_event: DraggablePanelListMove| {
                                                            mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                                                if let Some(row_picker) = &mut draft.row_picker {
                                                                    let target_index = aggregation_sort_insert_index_for_anchor(
                                                                        &row_picker.sort_fields,
                                                                        &move_event.anchor,
                                                                    );
                                                                    move_aggregation_sort_field_to_index(
                                                                        &mut row_picker.sort_fields,
                                                                        &move_event.dragged_id,
                                                                        target_index,
                                                                    );
                                                                }
                                                            });
                                                        })
                                                    />
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
                                    mutate_aggregation(aggregation, on_aggregation_change, |draft| {
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
                                        <th>"Actions"</th>
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
                                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
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
                                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
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
                                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
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
                                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                                                    if let Some(metric) = draft.metrics.iter_mut().find(|metric| metric.id == metric_id) {
                                                                        metric.label = value;
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </td>
                                                    <td>
                                                        <button
                                                            class="icon-button icon-button--compact-control"
                                                            disabled=move || !aggregation_enabled()
                                                            type="button"
                                                            aria-label="Remove metric"
                                                            title="Remove metric"
                                                            on:click=move |_| {
                                                                mutate_aggregation(aggregation, on_aggregation_change, |draft| {
                                                                    draft.metrics.retain(|metric| metric.id != metric_id);
                                                                });
                                                            }
                                                        >
                                                            <Trash2 class="icon-button__icon"/>
                                                        </button>
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
            </div>
        </section>
    }
}

fn field_option_label(field: &DatasetFieldDraft) -> String {
    format!("{} ({})", field.label, field.key)
}

fn mutate_aggregation(
    aggregation: Signal<DatasetAggregationDraft>,
    on_aggregation_change: Callback<DatasetAggregationDraft>,
    update: impl FnOnce(&mut DatasetAggregationDraft),
) {
    let mut draft = aggregation.get();
    update(&mut draft);
    on_aggregation_change.run(draft);
}

fn aggregation_sort_insert_index_for_anchor(
    sort_fields: &[DatasetRowPickerSortDraft],
    anchor: &DraggablePanelListAnchor,
) -> usize {
    match anchor {
        DraggablePanelListAnchor::Start => 0,
        DraggablePanelListAnchor::After(field_key) => sort_fields
            .iter()
            .position(|sort| &sort.field_key == field_key)
            .map(|index| index + 1)
            .unwrap_or(sort_fields.len()),
    }
}

fn move_aggregation_sort_field_to_index(
    sort_fields: &mut Vec<DatasetRowPickerSortDraft>,
    dragged_key: &str,
    target_index: usize,
) {
    let Some(dragged_index) = sort_fields
        .iter()
        .position(|sort| sort.field_key == dragged_key)
    else {
        return;
    };
    let dragged_field = sort_fields.remove(dragged_index);
    let target_index = if dragged_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    }
    .min(sort_fields.len());
    sort_fields.insert(target_index, dragged_field);
}

fn field_combobox_options(fields: Vec<DatasetFieldDraft>) -> Vec<ComboboxOption> {
    let mut options = fields
        .into_iter()
        .map(|field| ComboboxOption {
            label: field_option_label(&field),
            value: field.key,
        })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| left.value.cmp(&right.value));
    options
}

fn next_metric_id(metrics: &[DatasetAggregationMetricDraft]) -> u64 {
    metrics.iter().map(|metric| metric.id).max().unwrap_or(0) + 1
}

fn eligible_metric_fields(function: &str, fields: &[DatasetFieldDraft]) -> Vec<DatasetFieldDraft> {
    let mut eligible = fields
        .iter()
        .filter(|field| metric_field_type_is_allowed(function, &field.field_type))
        .cloned()
        .collect::<Vec<_>>();
    eligible.sort_by(|left, right| left.key.cmp(&right.key));
    eligible
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
