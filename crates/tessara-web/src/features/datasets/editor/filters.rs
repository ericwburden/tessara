//! Dataset editor filter controls.

use super::source_options::source_field_options;
use crate::features::datasets::types::{DatasetFieldDraft, DatasetRowFilterDraft};
use crate::features::datasets::types::{
    DatasetFormOption, DatasetRenderedForm, DatasetSourceDraft, DatasetUserOption, NodeResponse,
};
use icons::{ChevronsUpDown, Pencil, Trash2, WandSparkles};
use leptos::prelude::*;
use std::collections::BTreeMap;

#[component]
pub(crate) fn DatasetFiltersEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    initial_source: RwSignal<DatasetSourceDraft>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    nodes: RwSignal<Vec<NodeResponse>>,
    users: RwSignal<Vec<DatasetUserOption>>,
    row_filters: Signal<Vec<DatasetRowFilterDraft>>,
    on_row_filters_change: Callback<Vec<DatasetRowFilterDraft>>,
    #[prop(optional)] embedded: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(embedded);
    let section_class = if embedded {
        "route-panel__section dataset-editor-section dataset-filters-section dataset-editor-section--embedded"
    } else {
        "route-panel__section dataset-editor-section dataset-filters-section"
    };
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
                            <h3>"Filters"</h3>
                            <ChevronsUpDown class="dataset-sql-header__icon"/>
                        </button>
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
                                    .as_ref()
                                    .map(|field| filter_value_options(
                                        field,
                                        &initial_source.get(),
                                        &forms.get(),
                                        &rendered_forms.get(),
                                        &nodes.get(),
                                        &users.get(),
                                    ))
                                    .unwrap_or_default();
                                let operator_options =
                                    filter_operator_options(
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
                                                            .as_ref()
                                                            .map(|field| !filter_value_options(
                                                                field,
                                                                &initial_source.get(),
                                                                &forms.get(),
                                                                &rendered_forms.get(),
                                                                &nodes.get(),
                                                                &users.get(),
                                                            ).is_empty())
                                                            .unwrap_or(false);
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

#[derive(Clone, Copy)]
struct FilterOperatorOption {
    value: &'static str,
    label: &'static str,
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
            {
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
                }
            }
            {move || {
                if value_mode.get() == "field" {
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

fn filter_value_options(
    field: &DatasetFieldDraft,
    initial_source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    nodes: &[NodeResponse],
    users: &[DatasetUserOption],
) -> Vec<String> {
    if field.field_type == "boolean" {
        return vec!["true".into(), "false".into()];
    }
    if field.source_field_key == "__node_name" {
        return sorted_unique_options(nodes.iter().map(|node| node.name.clone()).collect());
    }
    if field.source_field_key == "__node_id" {
        return sorted_unique_options(nodes.iter().map(|node| node.id.clone()).collect());
    }
    if field.source_field_key == "__last_updated_by_user_name" {
        return sorted_unique_options(users.iter().map(|user| user.display_name.clone()).collect());
    }
    let sources = [initial_source.clone()];
    let mut options =
        source_field_options(&sources, &[], forms, rendered_forms, &field.source_alias)
            .into_iter()
            .find(|option| option.key == field.source_field_key)
            .map(|option| option.value_options)
            .unwrap_or_default();
    options.sort();
    options.dedup();
    options
}

fn sorted_unique_options(mut options: Vec<String>) -> Vec<String> {
    options.retain(|option| !option.trim().is_empty());
    options.sort();
    options.dedup();
    options
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
