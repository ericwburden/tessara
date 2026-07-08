//! Shared data-operation authoring controls.

use icons::{
    ArrowDown, ArrowUp, ChevronsUpDown, Pencil, Search, Square, SquareCheckBig, Trash2,
    WandSparkles,
};
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};
use tessara_web_ui::{
    Combobox, ComboboxOption, DataTable, DraggablePanelList, DraggablePanelListAnchor,
    DraggablePanelListDraggable, DraggablePanelListDropZone, DraggablePanelListItem,
    DraggablePanelListMove, SegmentedToggle, SegmentedToggleOption, empty_view,
};

#[derive(Clone, Debug, PartialEq)]
pub struct DatasetFieldDraft {
    pub key: String,
    pub label: String,
    pub source_alias: String,
    pub source_field_key: String,
    pub field_type: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DatasetAggregationDraft {
    pub enabled: bool,
    pub group_fields: Vec<String>,
    pub metrics: Vec<DatasetAggregationMetricDraft>,
    pub row_picker: Option<DatasetRowPickerDraft>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DatasetAggregationMetricDraft {
    pub id: u64,
    pub key: String,
    pub label: String,
    pub function: String,
    pub source_field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DatasetRowPickerDraft {
    pub sort_fields: Vec<DatasetRowPickerSortDraft>,
    pub direction: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DatasetRowPickerSortDraft {
    pub field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DatasetRowFilterDraft {
    pub id: u64,
    pub field_key: String,
    pub operator: String,
    pub value: String,
    pub value_mode: String,
    pub value_field_key: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ProjectionFieldGroup {
    label: String,
    fields: Vec<DatasetFieldDraft>,
}

#[derive(Clone, Copy)]
struct FilterOperatorOption {
    value: &'static str,
    label: &'static str,
}

#[component]
pub fn DataOpsFiltersEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    row_filters: Signal<Vec<DatasetRowFilterDraft>>,
    on_row_filters_change: Callback<Vec<DatasetRowFilterDraft>>,
    #[prop(optional)] value_options_provider: Option<Callback<DatasetFieldDraft, Vec<String>>>,
    #[prop(optional)] allow_field_comparison: bool,
    #[prop(optional)] embedded: bool,
    #[prop(optional)] title: Option<&'static str>,
    #[prop(optional)] collapsible: bool,
    #[prop(default = true)] initially_open: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(embedded || !collapsible || initially_open);
    let section_class = if embedded {
        "route-panel__section dataset-editor-section dataset-filters-section dataset-editor-section--embedded"
    } else {
        "route-panel__section dataset-editor-section dataset-filters-section"
    };
    let title = title.unwrap_or("Filters");
    view! {
        <section class=section_class>
            {if embedded {
                if title == "Filters" {
                    view! { <span></span> }.into_any()
                } else {
                    view! {
                        <div class="dataset-editor-section__header">
                            <h3>{title}</h3>
                        </div>
                    }.into_any()
                }
            } else if collapsible {
                view! {
                    <div class="dataset-editor-section__header">
                        <button
                            class="dataset-editor-section__header dataset-sql-header dataset-editor-section__collapse"
                            type="button"
                            aria-expanded=move || is_open.get().to_string()
                            on:click=move |_| is_open.update(|open| *open = !*open)
                        >
                            <h3>{title}</h3>
                            {move || {
                                if is_open.get() {
                                    view! { <ChevronsUpDown class="dataset-editor-section__collapse-icon" /> }
                                        .into_any()
                                } else {
                                    view! { <ChevronsUpDown class="dataset-editor-section__collapse-icon is-collapsed" /> }
                                        .into_any()
                                }
                            }}
                        </button>
                    </div>
                }.into_any()
            } else {
                view! {
                    <div class="dataset-editor-section__header">
                        <h3>{title}</h3>
                    </div>
                }.into_any()
            }}
            {move || if is_open.get() {
                view! {
                    {move || {
                        let filters = row_filters.get();
                        if filters.is_empty() {
                            view! { <p class="muted">"No filters configured."</p> }.into_any()
                        } else {
                            view! {
                                <div class="dataset-filter-list">
                                    {filters.into_iter().map(|filter| {
                                        let filter_id = filter.id;
                                        let field_value_id = filter_id;
                                        let operator_value_id = filter_id;
                                        let input_value_id = filter_id;
                                        let remove_id = filter_id;
                                        let selected_field = fields
                                            .get()
                                            .into_iter()
                                            .find(|field| field.key == filter.field_key);
                                        let value_options = selected_field
                                            .clone()
                                            .and_then(|field| {
                                                value_options_provider.map(|callback| callback.run(field))
                                            })
                                            .unwrap_or_else(|| default_filter_value_options(selected_field.as_ref()));
                                        let operator_options = filter_operator_options(
                                            selected_field.as_ref().map(|field| field.field_type.as_str()),
                                            !value_options.is_empty(),
                                        );
                                        let selected_operator = if operator_options
                                            .iter()
                                            .any(|option| option.value == filter.operator)
                                        {
                                            filter.operator.clone()
                                        } else {
                                            operator_options
                                                .first()
                                                .map(|option| option.value.to_string())
                                                .unwrap_or_default()
                                        };
                                        let operator_for_input = selected_operator.clone();
                                        view! {
                                            <div class="dataset-filter-row">
                                                <label class="form-field">
                                                    <span>"Field"</span>
                                                    <select prop:value=filter.field_key on:change=move |event| {
                                                        let value = event_target_value(&event);
                                                        mutate_filters(row_filters, on_row_filters_change, |filters| {
                                                            if let Some(filter) = filters.iter_mut().find(|filter| filter.id == field_value_id) {
                                                                filter.field_key = value;
                                                                let selected_field = fields
                                                                    .get()
                                                                    .into_iter()
                                                                    .find(|field| field.key == filter.field_key);
                                                                let has_value_options = selected_field
                                                                    .clone()
                                                                    .and_then(|field| {
                                                                        value_options_provider.map(|callback| callback.run(field))
                                                                    })
                                                                    .map(|options| !options.is_empty())
                                                                    .unwrap_or_else(|| !default_filter_value_options(selected_field.as_ref()).is_empty());
                                                                let options = filter_operator_options(
                                                                    selected_field.as_ref().map(|field| field.field_type.as_str()),
                                                                    has_value_options,
                                                                );
                                                                if !options.iter().any(|option| option.value == filter.operator) {
                                                                    filter.operator = options
                                                                        .first()
                                                                        .map(|option| option.value.to_string())
                                                                        .unwrap_or_default();
                                                                }
                                                                if !filter_operator_uses_value(&filter.operator) {
                                                                    filter.value.clear();
                                                                    filter.value_field_key.clear();
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        <option value="">"Select field"</option>
                                                        {filter_field_options(fields.get(), &filter.field_key).into_iter().map(|field| {
                                                            view! { <option value=field.key.clone()>{field_filter_label(&field)}</option> }
                                                        }).collect_view()}
                                                    </select>
                                                </label>
                                                <label class="form-field">
                                                    <span>"Operator"</span>
                                                    <select prop:value=selected_operator on:change=move |event| {
                                                        let value = event_target_value(&event);
                                                        mutate_filters(row_filters, on_row_filters_change, |filters| {
                                                            if let Some(filter) = filters.iter_mut().find(|filter| filter.id == operator_value_id) {
                                                                filter.operator = value;
                                                                if !filter_operator_uses_value(&filter.operator) {
                                                                    filter.value.clear();
                                                                    filter.value_field_key.clear();
                                                                }
                                                            }
                                                        });
                                                    }>
                                                        {operator_options.into_iter().map(|option| {
                                                            view! { <option value=option.value>{option.label}</option> }
                                                        }).collect_view()}
                                                    </select>
                                                </label>
                                                <label class="form-field">
                                                    <span>"Value"</span>
                                                    {filter_value_control(FilterValueControlParams {
                                                        filter_id: input_value_id,
                                                        value: filter.value,
                                                        field: selected_field,
                                                        fields: fields.get(),
                                                        operator: operator_for_input,
                                                        value_options,
                                                        row_filters,
                                                        on_row_filters_change,
                                                        allow_field_comparison,
                                                    })}
                                                </label>
                                                <button
                                                    class="icon-button icon-button--compact-control"
                                                    type="button"
                                                    aria-label="Remove filter"
                                                    title="Remove filter"
                                                    on:click=move |_| mutate_filters(row_filters, on_row_filters_change, |filters| filters.retain(|filter| filter.id != remove_id))
                                                >
                                                    <Trash2 class="icon-button__icon"/>
                                                </button>
                                            </div>
                                        }
                                    }).collect_view()}
                                </div>
                            }.into_any()
                        }
                    }}
                    <button
                        class="button button--secondary dataset-list-add-button"
                        type="button"
                        on:click=move |_| {
                            let field_key = fields
                                .get()
                                .first()
                                .map(|field| field.key.clone())
                                .unwrap_or_default();
                            mutate_filters(row_filters, on_row_filters_change, |filters| {
                                let id = filters.iter().map(|filter| filter.id).max().unwrap_or(0) + 1;
                                filters.push(DatasetRowFilterDraft {
                                    id,
                                    field_key: field_key.clone(),
                                    operator: "equals".into(),
                                    value: String::new(),
                                    value_mode: "value".into(),
                                    value_field_key: String::new(),
                                });
                            });
                        }
                    >
                        "Add Filter"
                    </button>
                }.into_any()
            } else {
                view! { <span class="dataset-editor-section__collapsed-spacer"></span> }.into_any()
            }}
        </section>
    }
}

#[component]
pub fn DataOpsProjectionEditor(
    available_fields: Signal<Vec<DatasetFieldDraft>>,
    fields: Signal<Vec<DatasetFieldDraft>>,
    active_source_tab: Signal<Option<String>>,
    on_active_source_tab_change: Callback<Option<String>>,
    on_fields_change: Callback<Vec<DatasetFieldDraft>>,
    #[prop(optional)] title: Option<&'static str>,
    #[prop(optional)] collapsible: bool,
    #[prop(default = true)] initially_open: bool,
) -> impl IntoView {
    let title = title.unwrap_or("Projection");
    let is_open = RwSignal::new(!collapsible || initially_open);
    let search = RwSignal::new(String::new());
    let selected_field_keys = Memo::new(move |_| {
        fields.with(|items| {
            items
                .iter()
                .map(|field| field.key.clone())
                .collect::<BTreeSet<_>>()
        })
    });
    let previous_available_fields = RwSignal::new(Vec::<DatasetFieldDraft>::new());

    Effect::new(move |_| {
        let selected_fields = fields.get();
        let available_fields = available_fields.get();
        let reconciled_fields = reconcile_projection_fields(
            selected_fields.clone(),
            available_fields.clone(),
            &previous_available_fields.get_untracked(),
        );
        previous_available_fields.set(available_fields);
        if reconciled_fields != selected_fields {
            on_fields_change.run(reconciled_fields);
        }
    });

    let on_toggle_available_field = Callback::new(move |field_for_toggle: DatasetFieldDraft| {
        let mut items = fields.get();
        if let Some(index) = items
            .iter()
            .position(|item| item.key == field_for_toggle.key)
        {
            items.remove(index);
        } else {
            items.push(field_for_toggle);
        }
        on_fields_change.run(items);
    });
    let selected_field_items = Signal::derive(move || {
        fields
            .get()
            .into_iter()
            .map(|field| DraggablePanelListItem { id: field.key })
            .collect::<Vec<_>>()
    });

    view! {
        <section class="route-panel__section dataset-editor-section dataset-fields-section dataset-editor-section--embedded">
            {if collapsible {
                view! {
                    <div class="dataset-editor-section__header">
                        <button
                            class="dataset-editor-section__header dataset-sql-header dataset-editor-section__collapse"
                            type="button"
                            aria-expanded=move || is_open.get().to_string()
                            on:click=move |_| is_open.update(|open| *open = !*open)
                        >
                            <h3>{title}</h3>
                            {move || {
                                if is_open.get() {
                                    view! { <ChevronsUpDown class="dataset-editor-section__collapse-icon" /> }
                                        .into_any()
                                } else {
                                    view! { <ChevronsUpDown class="dataset-editor-section__collapse-icon is-collapsed" /> }
                                        .into_any()
                                }
                            }}
                        </button>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {move || if is_open.get() {
                view! {
            <div class="dataset-projection-builder">
                <div class="dataset-projection-builder__toolbar">
                    <div class="dataset-projection-builder__search">
                        <Search class="dataset-projection-builder__search-icon"/>
                        <input
                            class="dataset-projection-builder__search-input"
                            type="search"
                            placeholder="Search available fields..."
                            aria-label="Search available fields"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </div>
                    <div class="dataset-projection-builder__toolbar-actions">
                        <button
                            class="button button--secondary"
                            type="button"
                            disabled=move || available_fields.get().is_empty()
                            on:click=move |_| {
                                on_fields_change.run(include_all_projection_fields(
                                    fields.get(),
                                    available_fields.get(),
                                ));
                            }
                        >
                            "Include All"
                        </button>
                        <button
                            class="button button--secondary"
                            type="button"
                            disabled=move || fields.get().is_empty()
                            on:click=move |_| {
                                on_fields_change.run(Vec::new());
                            }
                        >
                            "Clear All"
                        </button>
                    </div>
                </div>

                <ProjectionAvailableFields
                    available_fields=available_fields
                    selected_field_keys=selected_field_keys
                    search=search
                    active_source_tab=active_source_tab
                    on_active_source_tab_change=on_active_source_tab_change
                    on_toggle_field=on_toggle_available_field
                />

                <div class="dataset-projection-selected">
                    <div class="dataset-projection-selected__header">
                        <h5>"Selected Fields"</h5>
                        <small>{move || format!("{} fields", fields.get().len())}</small>
                    </div>
                    {move || {
                        let selected_fields = fields.get();
                        if selected_fields.is_empty() {
                            view! {
                                <p class="muted dataset-projection-builder__empty">
                                    "No fields selected."
                                </p>
                            }
                                .into_any()
                        } else {
                            view! {
                                <DraggablePanelList
                                    list_id="projection-selected-fields"
                                    items=selected_field_items
                                    container_class="dataset-projection-selected__list"
                                    list_class="dataset-projection-selected__list-items"
                                    draggable_class="dataset-projection-selected__item"
                                    drop_zone_class="dataset-projection-selected__drop-zone"
                                    drag_handle_title="Drag field to reorder"
                                    data_transfer_type="application/x-tessara-projection-field"
                                    render_drop_zone=Callback::new(move |_drop_zone: DraggablePanelListDropZone| {
                                        empty_view()
                                    })
                                    render_draggable=Callback::new(move |draggable: DraggablePanelListDraggable| {
                                        let Some(field) = fields
                                            .get()
                                            .into_iter()
                                            .find(|field| field.key == draggable.id)
                                        else {
                                            return empty_view();
                                        };
                                        let field_key = field.key.clone();
                                        let field_key_for_label = field_key.clone();
                                        let field_key_for_remove = field_key.clone();
                                        let field_key_for_up = field_key.clone();
                                        let field_key_for_down = field_key.clone();
                                        view! {
                                            <div class="dataset-projection-selected__row">
                                                <div class="dataset-projection-selected__fields">
                                                    <label class="form-field dataset-projection-selected__label-field">
                                                        <span>"Display Label"</span>
                                                        <input
                                                            aria-label=format!("Display label for {}", field.label)
                                                            class="dataset-field-picker__label-input"
                                                            prop:value=field.label.clone()
                                                            on:change=move |event| {
                                                                let value = event_target_value(&event);
                                                                on_fields_change.run(update_projection_field_label(
                                                                    fields.get(),
                                                                    &field_key_for_label,
                                                                    value,
                                                                ));
                                                            }
                                                        />
                                                    </label>
                                                    <div class="dataset-projection-selected__meta">
                                                        <span>
                                                            <small>"Field Name"</small>
                                                            <code>{field.key.clone()}</code>
                                                        </span>
                                                        <span>
                                                            <small>"Data Type"</small>
                                                            <strong>{projection_field_type_label(&field)}</strong>
                                                        </span>
                                                    </div>
                                                </div>
                                                <div class="dataset-projection-selected__actions">
                                                    <button
                                                        class="icon-button icon-button--compact-control"
                                                        type="button"
                                                        title="Move field up"
                                                        aria-label=format!("Move {} up", field.label)
                                                        on:click=move |_| {
                                                            on_fields_change.run(move_projection_field_by_delta(
                                                                fields.get(),
                                                                &field_key_for_up,
                                                                -1,
                                                            ));
                                                        }
                                                    >
                                                        <ArrowUp class="icon-button__icon"/>
                                                    </button>
                                                    <button
                                                        class="icon-button icon-button--compact-control"
                                                        type="button"
                                                        title="Move field down"
                                                        aria-label=format!("Move {} down", field.label)
                                                        on:click=move |_| {
                                                            on_fields_change.run(move_projection_field_by_delta(
                                                                fields.get(),
                                                                &field_key_for_down,
                                                                1,
                                                            ));
                                                        }
                                                    >
                                                        <ArrowDown class="icon-button__icon"/>
                                                    </button>
                                                    <button
                                                        class="icon-button icon-button--compact-control"
                                                        type="button"
                                                        title="Remove field"
                                                        aria-label=format!("Remove {}", field.label)
                                                        on:click=move |_| {
                                                            on_fields_change.run(remove_projection_field(
                                                                fields.get(),
                                                                &field_key_for_remove,
                                                            ));
                                                        }
                                                    >
                                                        <Trash2 class="icon-button__icon"/>
                                                    </button>
                                                </div>
                                            </div>
                                        }
                                        .into_any()
                                    })
                                    on_move=Callback::new(move |move_event: DraggablePanelListMove| {
                                        let items = fields.get();
                                        let target_index = projection_insert_index_for_anchor(
                                            &items,
                                            &move_event.anchor,
                                        );
                                        on_fields_change.run(move_projection_field_to_index(
                                            items,
                                            &move_event.dragged_id,
                                            target_index,
                                        ));
                                    })
                                />
                            }
                                .into_any()
                        }
                    }}
                </div>
            </div>
                }.into_any()
            } else {
                view! { <span class="dataset-editor-section__collapsed-spacer"></span> }.into_any()
            }}
        </section>
    }
}

#[component]
pub fn DataOpsAggregationEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    aggregation: Signal<DatasetAggregationDraft>,
    on_aggregation_change: Callback<DatasetAggregationDraft>,
    #[prop(optional)] embedded: bool,
    #[prop(optional)] metrics_only: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(embedded);
    let section_class = if embedded {
        "route-panel__section dataset-editor-section dataset-aggregation-section dataset-editor-section--embedded"
    } else {
        "route-panel__section dataset-editor-section dataset-aggregation-section"
    };
    let projected_fields = move || fields.get();
    let group_fields = move || aggregation.get().group_fields;
    let aggregation_enabled = move || metrics_only || aggregation.get().enabled;

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
        if metrics_only {
            return "metrics";
        }
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
            {if metrics_only {
                view! { <span></span> }.into_any()
            } else {
                view! { <div class="dataset-aggregation-top-row">
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
                </div> }.into_any()
            }}
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
                                                            <option value="count_distinct">"Count distinct"</option>
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

fn filter_operator_options(
    field_type: Option<&str>,
    has_value_options: bool,
) -> Vec<FilterOperatorOption> {
    let mut options = vec![
        FilterOperatorOption {
            value: "equals",
            label: "Equals",
        },
        FilterOperatorOption {
            value: "not_equals",
            label: "Does not equal",
        },
    ];
    if !has_value_options && matches!(field_type, None | Some("text") | Some("static_text")) {
        options.push(FilterOperatorOption {
            value: "contains",
            label: "Contains",
        });
    }
    if matches!(
        field_type,
        Some("number") | Some("date") | Some("datetime") | Some("timestamp")
    ) {
        options.extend([
            FilterOperatorOption {
                value: "greater_than",
                label: "Greater than",
            },
            FilterOperatorOption {
                value: "greater_than_or_equal",
                label: "Greater than or equal",
            },
            FilterOperatorOption {
                value: "less_than",
                label: "Less than",
            },
            FilterOperatorOption {
                value: "less_than_or_equal",
                label: "Less than or equal",
            },
        ]);
    }
    options.extend([
        FilterOperatorOption {
            value: "is_empty",
            label: "Is empty",
        },
        FilterOperatorOption {
            value: "is_not_empty",
            label: "Is not empty",
        },
    ]);
    options
}

fn filter_operator_uses_value(operator: &str) -> bool {
    matches!(
        operator,
        "equals"
            | "not_equals"
            | "contains"
            | "greater_than"
            | "greater_than_or_equal"
            | "less_than"
            | "less_than_or_equal"
    )
}

fn mutate_filters(
    row_filters: Signal<Vec<DatasetRowFilterDraft>>,
    on_row_filters_change: Callback<Vec<DatasetRowFilterDraft>>,
    update: impl FnOnce(&mut Vec<DatasetRowFilterDraft>),
) {
    let mut filters = row_filters.get();
    update(&mut filters);
    on_row_filters_change.run(filters);
}

struct FilterValueControlParams {
    filter_id: u64,
    value: String,
    field: Option<DatasetFieldDraft>,
    fields: Vec<DatasetFieldDraft>,
    operator: String,
    value_options: Vec<String>,
    row_filters: Signal<Vec<DatasetRowFilterDraft>>,
    on_row_filters_change: Callback<Vec<DatasetRowFilterDraft>>,
    allow_field_comparison: bool,
}

fn filter_value_control(params: FilterValueControlParams) -> AnyView {
    let FilterValueControlParams {
        filter_id,
        value,
        field,
        fields,
        operator,
        value_options,
        row_filters,
        on_row_filters_change,
        allow_field_comparison,
    } = params;
    if !filter_operator_uses_value(&operator) {
        return view! { <input disabled=true prop:value="" /> }.into_any();
    }

    let value_mode = Signal::derive(move || {
        row_filters
            .get()
            .into_iter()
            .find(|filter| filter.id == filter_id)
            .map(|filter| filter.value_mode)
            .unwrap_or_else(|| "value".into())
    });
    let selected_value_field = Signal::derive(move || {
        row_filters
            .get()
            .into_iter()
            .find(|filter| filter.id == filter_id)
            .map(|filter| filter.value_field_key)
            .unwrap_or_default()
    });
    let field_key = field
        .as_ref()
        .map(|field| field.key.clone())
        .unwrap_or_default();
    let field_type = field
        .as_ref()
        .map(|field| field.field_type.clone())
        .unwrap_or_default();
    let compatible_fields = fields
        .into_iter()
        .filter(|candidate| candidate.key != field_key)
        .filter(|candidate| candidate.field_type == field_type)
        .collect::<Vec<_>>();
    let literal_value = value.clone();
    let literal_field = field.clone();
    let literal_operator = operator.clone();
    let literal_value_options = value_options.clone();

    view! {
        <div class=move || if value_mode.get() == "field" {
            "dataset-filter-value-control is-field-mode"
        } else {
            "dataset-filter-value-control is-value-mode"
        }>
            {if allow_field_comparison {
                let compatible_for_select = compatible_fields.clone();
                view! {
                    <button
                        class=move || if value_mode.get() == "field" {
                            "icon-button icon-button--compact-control dataset-filter-value-mode-toggle is-field-mode"
                        } else {
                            "icon-button icon-button--compact-control dataset-filter-value-mode-toggle is-value-mode"
                        }
                        type="button"
                        aria-label=move || if value_mode.get() == "field" {
                            "Compare against a field"
                        } else {
                            "Compare against a value"
                        }
                        title=move || if value_mode.get() == "field" {
                            "Field"
                        } else {
                            "Value"
                        }
                        on:click=move |_| {
                            mutate_filters(row_filters, on_row_filters_change, |filters| {
                                if let Some(filter) = filters.iter_mut().find(|filter| filter.id == filter_id) {
                                    if filter.value_mode == "field" {
                                        filter.value_mode = "value".into();
                                        filter.value_field_key.clear();
                                    } else {
                                        filter.value_mode = "field".into();
                                        filter.value.clear();
                                        if filter.value_field_key.is_empty() {
                                            filter.value_field_key = compatible_for_select
                                                .first()
                                                .map(|field| field.key.clone())
                                                .unwrap_or_default();
                                        }
                                    }
                                }
                            });
                        }
                    >
                        {move || if value_mode.get() == "field" {
                            view! { <WandSparkles class="icon-button__icon"/> }.into_any()
                        } else {
                            view! { <Pencil class="icon-button__icon"/> }.into_any()
                        }}
                    </button>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
            {move || {
                if allow_field_comparison && value_mode.get() == "field" {
                    view! {
                        <select disabled=compatible_fields.is_empty() prop:value=move || selected_value_field.get() on:change=move |event| {
                            let value = event_target_value(&event);
                            mutate_filters(row_filters, on_row_filters_change, |filters| {
                                if let Some(filter) = filters.iter_mut().find(|filter| filter.id == filter_id) {
                                    filter.value_field_key = value;
                                }
                            });
                        }>
                            {if compatible_fields.is_empty() {
                                view! { <option value="">"No compatible fields"</option> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                            {compatible_fields.clone().into_iter().map(|field| {
                                view! { <option value=field.key.clone()>{field_filter_label(&field)}</option> }
                            }).collect_view()}
                        </select>
                    }.into_any()
                } else {
                    literal_filter_value_control(
                        filter_id,
                        literal_value.clone(),
                        literal_field.clone(),
                        literal_operator.clone(),
                        literal_value_options.clone(),
                        row_filters,
                        on_row_filters_change,
                    )
                }
            }}
        </div>
    }
    .into_any()
}

fn literal_filter_value_control(
    filter_id: u64,
    value: String,
    field: Option<DatasetFieldDraft>,
    operator: String,
    value_options: Vec<String>,
    row_filters: Signal<Vec<DatasetRowFilterDraft>>,
    on_row_filters_change: Callback<Vec<DatasetRowFilterDraft>>,
) -> AnyView {
    if !value_options.is_empty() && matches!(operator.as_str(), "equals" | "not_equals") {
        return view! {
            <select prop:value=value on:change=move |event| {
                let value = event_target_value(&event);
                mutate_filters(row_filters, on_row_filters_change, |filters| {
                    if let Some(filter) = filters.iter_mut().find(|filter| filter.id == filter_id) {
                        filter.value = value;
                    }
                });
            }>
                <option value="">"Select value"</option>
                {value_options.into_iter().map(|option| {
                    let value = option.clone();
                    view! { <option value=value>{option}</option> }
                }).collect_view()}
            </select>
        }
        .into_any();
    }

    let input_type = match field.as_ref().map(|field| field.field_type.as_str()) {
        Some("number") => "number",
        Some("date") => "date",
        Some("datetime") | Some("timestamp") => "datetime-local",
        _ => "text",
    };
    view! {
        <input
            type=input_type
            prop:value=value
            on:change=move |event| {
                let value = event_target_value(&event);
                mutate_filters(row_filters, on_row_filters_change, |filters| {
                    if let Some(filter) = filters.iter_mut().find(|filter| filter.id == filter_id) {
                        filter.value = value;
                    }
                });
            }
        />
    }
    .into_any()
}

fn default_filter_value_options(field: Option<&DatasetFieldDraft>) -> Vec<String> {
    match field.map(|field| field.field_type.as_str()) {
        Some("boolean") => vec!["true".into(), "false".into()],
        _ => Vec::new(),
    }
}

fn field_filter_label(field: &DatasetFieldDraft) -> String {
    if field.label.trim().is_empty() {
        field.key.clone()
    } else {
        format!("{} ({})", field.label, field.key)
    }
}

fn filter_field_options(
    fields: Vec<DatasetFieldDraft>,
    selected_key: &str,
) -> Vec<DatasetFieldDraft> {
    let mut fields = fields;
    fields.sort_by(|left, right| left.key.cmp(&right.key));
    if !selected_key.is_empty() && !fields.iter().any(|field| field.key == selected_key) {
        fields.insert(
            0,
            DatasetFieldDraft {
                key: selected_key.into(),
                label: format!("Missing field ({selected_key})"),
                source_alias: String::new(),
                source_field_key: selected_key.into(),
                field_type: "text".into(),
            },
        );
    }
    fields
}

#[component]
fn ProjectionAvailableFields(
    available_fields: Signal<Vec<DatasetFieldDraft>>,
    selected_field_keys: Memo<BTreeSet<String>>,
    search: RwSignal<String>,
    active_source_tab: Signal<Option<String>>,
    on_active_source_tab_change: Callback<Option<String>>,
    on_toggle_field: Callback<DatasetFieldDraft>,
) -> impl IntoView {
    let option_groups = Memo::new(move |_| {
        projection_option_groups(
            sorted_projection_fields(available_fields.get()),
            &search.get(),
        )
    });

    view! {
        <div class="dataset-projection-builder__available" role="listbox" aria-label="Available fields">
            {move || {
                let groups = option_groups.get();
                if groups.is_empty() {
                    return view! {
                        <p class="muted dataset-projection-builder__empty">
                            "No available fields match the current search."
                        </p>
                    }
                        .into_any();
                }

                let active_label = active_source_tab
                    .get()
                    .filter(|label| groups.iter().any(|group| &group.label == label))
                    .unwrap_or_else(|| {
                        groups
                            .first()
                            .map(|group| group.label.clone())
                            .unwrap_or_default()
                    });
                let active_fields = groups
                    .iter()
                    .find(|group| group.label == active_label)
                    .map(|group| group.fields.clone())
                    .unwrap_or_default();

                view! {
                    <div class="dataset-projection-builder__source-tabs" role="tablist" aria-label="Available field sources">
                        {groups
                            .into_iter()
                            .map(|group| {
                                let tab_label = group.label.clone();
                                let is_active = tab_label == active_label;
                                view! {
                                    <button
                                        class="dataset-projection-builder__source-tab"
                                        class:is-active=is_active
                                        type="button"
                                        role="tab"
                                        aria-selected=is_active
                                        on:click=move |_| {
                                            on_active_source_tab_change.run(Some(tab_label.clone()));
                                        }
                                    >
                                        {group.label}
                                    </button>
                                }
                            })
                            .collect_view()}
                    </div>
                    <div class="dataset-projection-builder__options">
                        <For
                            each=move || active_fields.clone()
                            key=|field| field.key.clone()
                            children=move |field| {
                                view! {
                                    <ProjectionAvailableFieldOption
                                        field=field
                                        selected_field_keys=selected_field_keys
                                        on_toggle_field=on_toggle_field
                                    />
                                }
                            }
                        />
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}

#[component]
fn ProjectionAvailableFieldOption(
    field: DatasetFieldDraft,
    selected_field_keys: Memo<BTreeSet<String>>,
    on_toggle_field: Callback<DatasetFieldDraft>,
) -> impl IntoView {
    let label = field.label.clone();
    let key = field.key.clone();
    let type_label = projection_field_type_label(&field);
    let selected_key = key.clone();
    let label_for_aria = label.clone();
    let is_selected =
        Memo::new(move |_| selected_field_keys.with(|keys| keys.contains(&selected_key)));
    let field_for_toggle = field.clone();

    view! {
        <button
            class="dataset-projection-builder__option"
            class:is-selected=move || is_selected.get()
            type="button"
            role="option"
            aria-label=move || {
                if is_selected.get() {
                    format!("Remove {label_for_aria}")
                } else {
                    format!("Add {label_for_aria}")
                }
            }
            aria-selected=move || is_selected.get()
            on:mousedown=move |event| event.prevent_default()
            on:click=move |_| {
                on_toggle_field.run(field_for_toggle.clone());
            }
        >
            <span class="dataset-projection-builder__option-main">
                <span class="dataset-projection-builder__option-row">
                    <strong>{label}</strong>
                    <small>{type_label}</small>
                </span>
                <span class="dataset-projection-builder__option-meta-row">
                    <code>{key}</code>
                    <span class="dataset-projection-builder__option-check" aria-hidden="true">
                        {move || {
                            if is_selected.get() {
                                view! { <SquareCheckBig class="icon-button__icon"/> }.into_any()
                            } else {
                                view! { <Square class="icon-button__icon"/> }.into_any()
                            }
                        }}
                    </span>
                </span>
            </span>
        </button>
    }
}

fn projection_option_groups(
    available_fields: Vec<DatasetFieldDraft>,
    query: &str,
) -> Vec<ProjectionFieldGroup> {
    let query = query.trim().to_lowercase();
    let mut groups = BTreeMap::<String, Vec<DatasetFieldDraft>>::new();
    for field in available_fields {
        let searchable =
            format!("{} {} {}", field.label, field.key, field.source_alias).to_lowercase();
        if !query.is_empty() && !searchable.contains(&query) {
            continue;
        }
        groups
            .entry(field.source_alias.clone())
            .or_default()
            .push(field);
    }
    let mut groups = groups
        .into_iter()
        .map(|(label, fields)| ProjectionFieldGroup { label, fields })
        .collect::<Vec<_>>();
    groups.sort_by(|left, right| {
        projection_source_group_sort_key(&left.label)
            .cmp(&projection_source_group_sort_key(&right.label))
    });
    groups
}

fn projection_source_group_sort_key(label: &str) -> (u8, &str) {
    match label {
        "calculated" | "aggregation" => (1, label),
        _ => (0, label),
    }
}

fn include_all_projection_fields(
    mut selected_fields: Vec<DatasetFieldDraft>,
    available_fields: Vec<DatasetFieldDraft>,
) -> Vec<DatasetFieldDraft> {
    let mut selected_keys = selected_fields
        .iter()
        .map(|field| field.key.clone())
        .collect::<BTreeSet<_>>();
    for field in sorted_projection_fields(available_fields) {
        if selected_keys.insert(field.key.clone()) {
            selected_fields.push(field);
        }
    }
    selected_fields
}

fn reconcile_projection_fields(
    selected_fields: Vec<DatasetFieldDraft>,
    available_fields: Vec<DatasetFieldDraft>,
    previous_available_fields: &[DatasetFieldDraft],
) -> Vec<DatasetFieldDraft> {
    if available_fields.is_empty() && previous_available_fields.is_empty() {
        return selected_fields;
    }

    let mut available_by_key = BTreeMap::<String, DatasetFieldDraft>::new();
    let mut available_by_input = BTreeMap::<String, Vec<DatasetFieldDraft>>::new();
    for field in available_fields {
        available_by_input
            .entry(projection_source_key(&field))
            .or_default()
            .push(field.clone());
        available_by_key.insert(field.key.clone(), field);
    }
    let previous_by_key = previous_available_fields
        .iter()
        .map(|field| (field.key.clone(), field.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut previous_by_input = BTreeMap::<String, Vec<DatasetFieldDraft>>::new();
    for field in previous_available_fields {
        previous_by_input
            .entry(projection_source_key(field))
            .or_default()
            .push(field.clone());
    }

    let mut selected_keys = BTreeSet::<String>::new();
    selected_fields
        .into_iter()
        .filter_map(|selected_field| {
            let mut available_field =
                projection_catalog_match(&selected_field, &available_by_key, &available_by_input)?;

            if !selected_keys.insert(available_field.key.clone()) {
                return None;
            }

            if projection_catalog_match(&selected_field, &previous_by_key, &previous_by_input)
                .is_none_or(|previous_field| selected_field.label != previous_field.label)
            {
                available_field.label = selected_field.label;
            }
            Some(available_field)
        })
        .collect()
}

fn projection_catalog_match(
    field: &DatasetFieldDraft,
    fields_by_key: &BTreeMap<String, DatasetFieldDraft>,
    fields_by_input: &BTreeMap<String, Vec<DatasetFieldDraft>>,
) -> Option<DatasetFieldDraft> {
    fields_by_key.get(&field.key).cloned().or_else(|| {
        let input_key = projection_source_key(field);
        fields_by_input
            .get(&input_key)
            .and_then(|matches| (matches.len() == 1).then(|| matches[0].clone()))
    })
}

fn remove_projection_field(
    mut fields: Vec<DatasetFieldDraft>,
    field_key: &str,
) -> Vec<DatasetFieldDraft> {
    fields.retain(|field| field.key != field_key);
    fields
}

fn update_projection_field_label(
    mut fields: Vec<DatasetFieldDraft>,
    field_key: &str,
    label: String,
) -> Vec<DatasetFieldDraft> {
    if let Some(field) = fields.iter_mut().find(|field| field.key == field_key) {
        field.label = label;
    }
    fields
}

fn move_projection_field_by_delta(
    mut fields: Vec<DatasetFieldDraft>,
    field_key: &str,
    delta: isize,
) -> Vec<DatasetFieldDraft> {
    let Some(index) = fields.iter().position(|field| field.key == field_key) else {
        return fields;
    };
    let next_index =
        (index as isize + delta).clamp(0, fields.len().saturating_sub(1) as isize) as usize;
    if index != next_index {
        fields.swap(index, next_index);
    }
    fields
}

fn move_projection_field_to_index(
    mut fields: Vec<DatasetFieldDraft>,
    dragged_key: &str,
    target_index: usize,
) -> Vec<DatasetFieldDraft> {
    let Some(dragged_index) = fields.iter().position(|field| field.key == dragged_key) else {
        return fields;
    };
    let dragged_field = fields.remove(dragged_index);
    let target_index = if dragged_index < target_index {
        target_index.saturating_sub(1)
    } else {
        target_index
    }
    .min(fields.len());
    fields.insert(target_index, dragged_field);
    fields
}

fn projection_insert_index_for_anchor(
    fields: &[DatasetFieldDraft],
    anchor: &DraggablePanelListAnchor,
) -> usize {
    match anchor {
        DraggablePanelListAnchor::Start => 0,
        DraggablePanelListAnchor::After(field_key) => fields
            .iter()
            .position(|field| &field.key == field_key)
            .map(|index| index + 1)
            .unwrap_or(fields.len()),
    }
}

fn sorted_projection_fields(mut fields: Vec<DatasetFieldDraft>) -> Vec<DatasetFieldDraft> {
    fields.sort_by(|left, right| {
        projection_field_group(left)
            .cmp(&projection_field_group(right))
            .then_with(|| {
                projection_source_group_sort_key(&left.source_alias)
                    .cmp(&projection_source_group_sort_key(&right.source_alias))
            })
            .then_with(|| left.key.cmp(&right.key))
    });
    fields
}

fn projection_field_group(field: &DatasetFieldDraft) -> u8 {
    if projection_source_key(field).starts_with("__") {
        0
    } else {
        1
    }
}

fn projection_field_type_label(field: &DatasetFieldDraft) -> String {
    match projection_source_key(field).as_str() {
        "__submission_id" | "__form_version_id" | "__node_id" => "Key".into(),
        "__submission_status" => "Status".into(),
        "__node_name" | "__last_updated_by_user_name" => "Lookup".into(),
        _ => sentence_label(&field.field_type),
    }
}

fn projection_source_key(field: &DatasetFieldDraft) -> String {
    if field.source_field_key.starts_with("__") {
        return field.source_field_key.clone();
    }
    let source_prefix = format!("{}__", field.source_alias);
    let suffix = field
        .key
        .strip_prefix(&source_prefix)
        .unwrap_or(&field.source_field_key);
    match suffix.trim_start_matches('_') {
        "submission_id"
        | "form_version_id"
        | "node_id"
        | "node_name"
        | "submission_status"
        | "submitted_at"
        | "submission_created_at"
        | "last_updated_at"
        | "last_updated_by_user_name" => format!("__{}", suffix.trim_start_matches('_')),
        _ => field.source_field_key.clone(),
    }
}

fn sentence_label(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
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
        "count_values" | "count_distinct" => true,
        "sum" | "average" => field_type == "number",
        "min" | "max" => matches!(
            field_type,
            "text"
                | "static_text"
                | "number"
                | "date"
                | "datetime"
                | "timestamp"
                | "single_choice"
                | "multi_choice"
        ),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::metric_field_type_is_allowed;

    #[test]
    fn min_max_allow_text_and_static_text_fields() {
        for function in ["min", "max"] {
            assert!(metric_field_type_is_allowed(function, "text"));
            assert!(metric_field_type_is_allowed(function, "static_text"));
        }
    }
}
