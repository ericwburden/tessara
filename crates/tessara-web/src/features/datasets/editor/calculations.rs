//! Dataset calculated-field and restriction-tier controls.

use super::helpers::confirm_action;
use crate::features::datasets::types::{
    DatasetCalculatedFieldDraft, DatasetCalculationFunctionDraft, DatasetFieldDraft,
};
use crate::ui::{Combobox, ComboboxOption};
use icons::{ChevronsUpDown, Diamond, DiamondPlus, Pencil, Trash2, WandSparkles};
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetCalculationsEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
    #[prop(optional)] embedded: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(embedded);
    let section_class = if embedded {
        "route-panel__section dataset-editor-section dataset-calculations-section dataset-editor-section--embedded"
    } else {
        "route-panel__section dataset-editor-section dataset-calculations-section"
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
                            <h3>"Calculated Fields"</h3>
                            <ChevronsUpDown class="dataset-sql-header__icon"/>
                        </button>
                    </div>
                }.into_any()
            }}
            {move || if is_open.get() {
                view! {
                    <div class="dataset-calculation-list">
                        {move || {
                            let calculations = calculated_fields.get();
                            if calculations.is_empty() {
                                view! { <p class="muted">"No calculated fields configured."</p> }.into_any()
                            } else {
                                view! {
                                    {calculations.into_iter().map(|calculation| {
                                        calculation_editor(
                                            fields,
                                            calculated_fields,
                                            on_calculated_fields_change,
                                            calculation,
                                        )
                                    }).collect_view()}
                                }.into_any()
                            }
                        }}
                        <button
                            class="button button--secondary dataset-list-add-button"
                            type="button"
                            on:click=move |_| {
                                let base_field_key = fields
                                    .get()
                                    .first()
                                    .map(|field| field.key.clone())
                                    .unwrap_or_default();
                                mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                                    let id = items.iter().map(|field| field.id).max().unwrap_or(0) + 1;
                                    items.push(DatasetCalculatedFieldDraft {
                                        id,
                                        key: format!("calculated_{id}"),
                                        label: format!("Calculated {id}"),
                                        base_field_key: base_field_key.clone(),
                                        functions: Vec::new(),
                                    });
                                });
                            }
                        >
                            "Add Calculated Field"
                        </button>
                    </div>
                }.into_any()
            } else {
                view! { <span class="dataset-editor-section__collapsed-spacer"></span> }.into_any()
            }}
        </section>
    }
}

#[component]
pub(crate) fn DatasetRestrictionsEditor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    restriction_internal_field_key: RwSignal<String>,
    restriction_restricted_field_key: RwSignal<String>,
    restriction_confidential_field_key: RwSignal<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    view! {
        <section class="route-panel__section dataset-editor-section dataset-restrictions-section">
            <div class="dataset-editor-section__header">
                <button
                    class="dataset-editor-section__header dataset-sql-header dataset-editor-section__collapse"
                    type="button"
                    aria-expanded=move || is_open.get().to_string()
                    on:click=move |_| is_open.update(|open| *open = !*open)
                >
                    <h3>"View Restrictions"</h3>
                    <ChevronsUpDown class="dataset-sql-header__icon"/>
                </button>
            </div>
            {move || if is_open.get() {
                view! {
                    <div class="dataset-restriction-policy">
                        <p class="muted">
                            "Rows are public by default. A true value in one of these boolean fields sets the row to that tier; if more than one selected flag is true, the lowest tier wins."
                        </p>
                        {restriction_flag_row("Internal", fields, restriction_internal_field_key)}
                        {restriction_flag_row("Restricted", fields, restriction_restricted_field_key)}
                        {restriction_flag_row("Confidential", fields, restriction_confidential_field_key)}
                    </div>
                }.into_any()
            } else {
                view! { <span class="dataset-editor-section__collapsed-spacer"></span> }.into_any()
            }}
        </section>
    }
}

fn calculation_editor(
    fields: Signal<Vec<DatasetFieldDraft>>,
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
    calculation: DatasetCalculatedFieldDraft,
) -> impl IntoView {
    let calculation_id = calculation.id;
    let is_open = RwSignal::new(true);
    let key_id = calculation_id;
    let label_id = calculation_id;
    let base_id = calculation_id;
    let add_function_id = calculation_id;
    let remove_id = calculation_id;
    let summary_key = calculation.key.clone();
    let summary_label = calculation.label.clone();
    view! {
        <article class="dataset-calculation-row">
            <div class="dataset-calculation-row__header">
                <button
                    class="dataset-calculation-row__collapse"
                    type="button"
                    aria-expanded=move || is_open.get().to_string()
                    on:click=move |_| is_open.update(|open| *open = !*open)
                >
                    <ChevronsUpDown class="dataset-calculation-row__collapse-icon"/>
                    <span>{summary_label}</span>
                    <code>{summary_key}</code>
                </button>
            </div>
            <div class=move || if is_open.get() {
                "dataset-calculation-row__body"
            } else {
                "dataset-calculation-row__body is-collapsed"
            }>
            <div class="dataset-calculation-row__grid">
                <label class="form-field">
                    <span>"Output Key"</span>
                    <input prop:value=calculation.key on:change=move |event| {
                        let value = event_target_value(&event);
                        mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                            if let Some(field) = items.iter_mut().find(|field| field.id == key_id) {
                                field.key = value;
                            }
                        });
                    }/>
                </label>
                <label class="form-field">
                    <span>"Label"</span>
                    <input prop:value=calculation.label on:change=move |event| {
                        let value = event_target_value(&event);
                        mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                            if let Some(field) = items.iter_mut().find(|field| field.id == label_id) {
                                field.label = value;
                            }
                        });
                    }/>
                </label>
                <label class="form-field">
                    <span>"Base Field"</span>
                    <select prop:value=move || calculation_current_base_key(calculated_fields.get(), calculation_id) on:change=move |event| {
                        let value = event_target_value(&event);
                        mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                            if let Some(field) = items.iter_mut().find(|field| field.id == base_id) {
                                field.base_field_key = value;
                                normalize_calculation_function_pipeline(field, &fields.get());
                            }
                        });
                    }>
                        <option value="">"Select field"</option>
                        {move || {
                            let selected_key = calculation_current_base_key(calculated_fields.get(), calculation_id);
                            calculation_base_field_options(fields.get(), &selected_key).into_iter().map(|field| {
                                view! { <option value=field.key.clone()>{field_label(&field)}</option> }
                            }).collect_view()
                        }}
                    </select>
                </label>
                <div class="dataset-calculation-row__actions">
                    <button
                        class="icon-button icon-button--compact-control dataset-calculation-action"
                        type="button"
                        aria-label="Add function"
                        title="Add function"
                        on:click=move |_| add_calculation_function_after(
                            add_function_id,
                            None,
                            fields,
                            calculated_fields,
                            on_calculated_fields_change,
                        )
                    >
                        <DiamondPlus class="icon-button__icon"/>
                    </button>
                    <button
                        class="icon-button icon-button--compact-control dataset-calculation-action"
                        type="button"
                        aria-label="Remove calculated field"
                        title="Remove calculated field"
                        on:click=move |_| {
                            if confirm_action("Remove this calculated field?") {
                                mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                                    items.retain(|field| field.id != remove_id);
                                });
                            }
                        }
                    >
                        <Trash2 class="icon-button__icon"/>
                    </button>
                </div>
            </div>
            <div class="dataset-calculation-functions">
                {calculation.functions.into_iter().enumerate().map(|(index, function)| {
                    function_editor(
                        calculation_id,
                        function,
                        index == 0,
                        fields,
                        calculated_fields,
                        on_calculated_fields_change,
                    )
                }).collect_view()}
            </div>
            <code class="dataset-calculation-preview">
                {move || calculation_preview(calculated_fields.get(), calculation_id)}
            </code>
            </div>
        </article>
    }
}

fn function_editor(
    calculation_id: u64,
    function: DatasetCalculationFunctionDraft,
    show_labels: bool,
    fields: Signal<Vec<DatasetFieldDraft>>,
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
) -> impl IntoView {
    let function_id = function.id;
    let select_id = function_id;
    let arg_id = function_id;
    let remove_id = function_id;
    view! {
        <div class="dataset-calculation-function">
            <span class="dataset-calculation-function__pipeline" aria-hidden="true">
                <Diamond class="dataset-calculation-function__pipeline-icon"/>
            </span>
            <label class="form-field">
                {if show_labels {
                    view! { <span>"Function"</span> }.into_any()
                } else {
                    view! { <span class="dataset-calculation-function__spacer" aria-hidden="true"></span> }.into_any()
                }}
                <Combobox
                    options=Signal::derive(move || {
                        let field_type = calculation_input_type_for_function(
                            calculated_fields.get(),
                            fields.get(),
                            calculation_id,
                            function_id,
                        );
                        calculation_function_options(field_type.as_deref())
                            .into_iter()
                            .map(|option| ComboboxOption {
                                value: option.value.into(),
                                label: option.label.into(),
                            })
                            .collect::<Vec<_>>()
                    })
                    selected_label=Signal::derive(move || {
                        calculation_current_function(calculated_fields.get(), calculation_id, function_id)
                            .map(|function| calculation_function_label(&function))
                            .unwrap_or_default()
                    })
                    placeholder="Select function..."
                    search_placeholder="Search functions..."
                    empty_label="No functions found."
                    aria_label="Function"
                    on_select=Callback::new(move |function_id_value: String| {
                        mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                            if let Some(field) = items.iter_mut().find(|field| field.id == calculation_id) {
                                if let Some(function) = field.functions.iter_mut().find(|function| function.id == select_id) {
                                    function.function = function_id_value;
                                    if !calculation_function_uses_argument(&function.function) {
                                        function.argument.clear();
                                        function.argument_mode = "value".into();
                                        function.argument_field_key.clear();
                                    }
                                }
                                normalize_calculation_function_pipeline(field, &fields.get());
                            }
                        });
                    })
                />
            </label>
            <label class="form-field">
                {if show_labels {
                    view! { <span>"Argument"</span> }.into_any()
                } else {
                    view! { <span class="dataset-calculation-function__spacer" aria-hidden="true"></span> }.into_any()
                }}
                {calculation_argument_control(
                    calculation_id,
                    function_id,
                    arg_id,
                    fields,
                    calculated_fields,
                    on_calculated_fields_change,
                )}
                <small class="dataset-calculation-function__hint">
                    {move || {
                        calculation_current_function(calculated_fields.get(), calculation_id, function_id)
                            .as_deref()
                            .map(calculation_function_argument_hint)
                            .unwrap_or("Select a function.")
                    }}
                </small>
            </label>
            <div class="dataset-calculation-function__actions">
                <button
                    class="icon-button icon-button--compact-control dataset-calculation-action"
                    type="button"
                    aria-label="Add function after this step"
                    title="Add function after this step"
                    on:click=move |_| add_calculation_function_after(
                        calculation_id,
                        Some(function_id),
                        fields,
                        calculated_fields,
                        on_calculated_fields_change,
                    )
                >
                    <DiamondPlus class="icon-button__icon"/>
                </button>
                <button
                    class="icon-button icon-button--compact-control dataset-calculation-action"
                    type="button"
                    aria-label="Remove function"
                    title="Remove function"
                    on:click=move |_| {
                        if confirm_action("Remove this function from the calculation pipeline?") {
                            mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                                if let Some(field) = items.iter_mut().find(|field| field.id == calculation_id) {
                                    field.functions.retain(|function| function.id != remove_id);
                                }
                            });
                        }
                    }
                >
                    <Trash2 class="icon-button__icon"/>
                </button>
            </div>
        </div>
    }
}

fn add_calculation_function_after(
    calculation_id: u64,
    after_function_id: Option<u64>,
    fields: Signal<Vec<DatasetFieldDraft>>,
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
) {
    mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
        if let Some(field) = items.iter_mut().find(|field| field.id == calculation_id) {
            let id = field
                .functions
                .iter()
                .map(|function| function.id)
                .max()
                .unwrap_or(0)
                + 1;
            let insert_at = after_function_id
                .and_then(|after_id| {
                    field
                        .functions
                        .iter()
                        .position(|function| function.id == after_id)
                        .map(|index| index + 1)
                })
                .unwrap_or(field.functions.len());
            let input_type = calculation_output_type_at_index(field, &fields.get(), insert_at);
            field.functions.insert(
                insert_at,
                DatasetCalculationFunctionDraft {
                    id,
                    function: default_calculation_function_for_input_type(input_type.as_deref())
                        .into(),
                    argument: String::new(),
                    argument_mode: "value".into(),
                    argument_field_key: String::new(),
                },
            );
        }
    });
}

fn restriction_boolean_field_options(fields: Vec<DatasetFieldDraft>) -> impl IntoView {
    restriction_boolean_field_choices(fields)
        .into_iter()
        .map(|(value, label)| view! { <option value=value>{label}</option> })
        .collect_view()
}

fn restriction_boolean_field_choices(fields: Vec<DatasetFieldDraft>) -> Vec<(String, String)> {
    fields
        .into_iter()
        .filter(|field| field.field_type == "boolean")
        .map(|field| (field.key.clone(), field_label(&field)))
        .collect()
}

fn restriction_flag_row(
    tier: &'static str,
    fields: Signal<Vec<DatasetFieldDraft>>,
    selected_field_key: RwSignal<String>,
) -> impl IntoView {
    let toggle_signal = selected_field_key;
    view! {
        <div class="dataset-restriction-row">
            <span class="dataset-restriction-row__label">{tier}</span>
            <label class="toggle-control">
                <input
                    aria-label=format!("{tier} flag enabled")
                    type="checkbox"
                    prop:checked=move || !toggle_signal.get().is_empty()
                    on:change=move |event| {
                        if event_target_checked(&event) {
                            let first_key = restriction_boolean_field_choices(fields.get())
                                .first()
                                .map(|(key, _)| key.clone())
                                .unwrap_or_default();
                            selected_field_key.set(first_key);
                        } else {
                            selected_field_key.set(String::new());
                        }
                    }
                />
                <span></span>
            </label>
            <select
                aria-label=format!("{tier} flag field")
                disabled=move || selected_field_key.get().is_empty()
                prop:value=move || selected_field_key.get()
                on:change=move |event| selected_field_key.set(event_target_value(&event))
            >
                <option value="">{format!("No {} flag", tier.to_lowercase())}</option>
                {move || restriction_boolean_field_options(fields.get())}
            </select>
        </div>
    }
}

fn field_label(field: &DatasetFieldDraft) -> String {
    if field.label.trim().is_empty() {
        field.key.clone()
    } else {
        format!("{} ({})", field.label, field.key)
    }
}

fn sorted_fields_by_key(mut fields: Vec<DatasetFieldDraft>) -> Vec<DatasetFieldDraft> {
    fields.sort_by(|left, right| left.key.cmp(&right.key));
    fields
}

fn calculation_current_base_key(
    calculations: Vec<DatasetCalculatedFieldDraft>,
    calculation_id: u64,
) -> String {
    calculations
        .into_iter()
        .find(|field| field.id == calculation_id)
        .map(|field| field.base_field_key)
        .unwrap_or_default()
}

fn calculation_base_field_options(
    fields: Vec<DatasetFieldDraft>,
    selected_key: &str,
) -> Vec<DatasetFieldDraft> {
    let mut fields = sorted_fields_by_key(fields);
    if !selected_key.is_empty() && !fields.iter().any(|field| field.key == selected_key) {
        fields.insert(0, missing_field_option(selected_key));
    }
    fields
}

fn missing_field_option(key: &str) -> DatasetFieldDraft {
    DatasetFieldDraft {
        key: key.into(),
        label: format!("Missing field ({key})"),
        source_alias: String::new(),
        source_field_key: key.into(),
        field_type: "text".into(),
    }
}

fn calculation_argument_control(
    calculation_id: u64,
    function_id: u64,
    arg_id: u64,
    fields: Signal<Vec<DatasetFieldDraft>>,
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
) -> impl IntoView {
    view! {
        {move || {
            let function = calculation_current_function(calculated_fields.get(), calculation_id, function_id)
                .unwrap_or_default();
            let input_type = calculation_input_type_for_function(
                calculated_fields.get(),
                fields.get(),
                calculation_id,
                function_id,
            );
            let argument_kind = calculation_function_argument_kind(&function, input_type.as_deref());
            let current_argument = calculation_current_argument(calculated_fields.get(), calculation_id, arg_id);
            if argument_kind == CalculationArgumentKind::None {
                view! { <input disabled=true prop:value="" /> }.into_any()
            } else if calculation_function_accepts_field_argument(&function) {
                let mode_signal = Signal::derive(move || {
                    calculation_current_argument_mode(calculated_fields.get(), calculation_id, arg_id)
                });
                let selected_field_signal = Signal::derive(move || {
                    calculation_current_argument_field(calculated_fields.get(), calculation_id, arg_id)
                });
                let compatible_fields = compatible_argument_fields(fields.get(), &argument_kind, input_type.as_deref());
                let compatible_fields_for_toggle = compatible_fields.clone();
                view! {
                    <div class=move || if mode_signal.get() == "field" {
                        "dataset-filter-value-control dataset-calculation-argument-control is-field-mode"
                    } else {
                        "dataset-filter-value-control dataset-calculation-argument-control is-value-mode"
                    }>
                        <button
                            class=move || if mode_signal.get() == "field" {
                                "icon-button icon-button--compact-control dataset-filter-value-mode-toggle is-field-mode"
                            } else {
                                "icon-button icon-button--compact-control dataset-filter-value-mode-toggle is-value-mode"
                            }
                            type="button"
                            aria-label=move || if mode_signal.get() == "field" {
                                "Use a field argument"
                            } else {
                                "Use a value argument"
                            }
                            title=move || if mode_signal.get() == "field" { "Field" } else { "Value" }
                            on:click=move |_| {
                                mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
                                    if let Some(field) = items.iter_mut().find(|field| field.id == calculation_id)
                                        && let Some(function) = field.functions.iter_mut().find(|function| function.id == arg_id)
                                    {
                                        if function.argument_mode == "field" {
                                            function.argument_mode = "value".into();
                                            function.argument_field_key.clear();
                                        } else {
                                            function.argument_mode = "field".into();
                                            function.argument.clear();
                                            if function.argument_field_key.is_empty() {
                                                function.argument_field_key = compatible_fields_for_toggle
                                                    .first()
                                                    .map(|field| field.key.clone())
                                                    .unwrap_or_default();
                                            }
                                        }
                                    }
                                });
                            }
                        >
                            {move || if mode_signal.get() == "field" {
                                view! { <WandSparkles class="icon-button__icon"/> }.into_any()
                            } else {
                                view! { <Pencil class="icon-button__icon"/> }.into_any()
                            }}
                        </button>
                        {move || if mode_signal.get() == "field" {
                            view! {
                                <select disabled=compatible_fields.is_empty() prop:value=move || selected_field_signal.get() on:change=move |event| {
                                    update_calculation_argument_field(
                                        calculated_fields,
                                        on_calculated_fields_change,
                                        calculation_id,
                                        arg_id,
                                        event_target_value(&event),
                                    );
                                }>
                                    {if compatible_fields.is_empty() {
                                        view! { <option value="">"No compatible fields"</option> }.into_any()
                                    } else {
                                        view! { <span></span> }.into_any()
                                    }}
                                    {compatible_fields.clone().into_iter().map(|field| {
                                        view! { <option value=field.key.clone()>{field_label(&field)}</option> }
                                    }).collect_view()}
                                </select>
                            }.into_any()
                        } else {
                            literal_calculation_argument_control(
                                argument_kind,
                                calculation_current_argument(calculated_fields.get(), calculation_id, arg_id),
                                calculated_fields,
                                on_calculated_fields_change,
                                calculation_id,
                                arg_id,
                            )
                        }}
                    </div>
                }.into_any()
            } else {
                literal_calculation_argument_control(
                    argument_kind,
                    current_argument,
                    calculated_fields,
                    on_calculated_fields_change,
                    calculation_id,
                    arg_id,
                )
            }
        }}
    }
}

fn literal_calculation_argument_control(
    argument_kind: CalculationArgumentKind,
    argument: String,
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
    calculation_id: u64,
    arg_id: u64,
) -> AnyView {
    match argument_kind {
        CalculationArgumentKind::None => view! { <input disabled=true prop:value="" /> }.into_any(),
        CalculationArgumentKind::Boolean => view! {
            <select prop:value=argument.clone() on:change=move |event| {
                update_calculation_argument(calculated_fields, on_calculated_fields_change, calculation_id, arg_id, event_target_value(&event));
            }>
                <option value="">"Select value"</option>
                <option value="true">"True"</option>
                <option value="false">"False"</option>
            </select>
        }.into_any(),
        CalculationArgumentKind::Integer => view! {
            <input type="number" step="1" prop:value=argument.clone() on:change=move |event| {
                update_calculation_argument(calculated_fields, on_calculated_fields_change, calculation_id, arg_id, event_target_value(&event));
            }/>
        }.into_any(),
        CalculationArgumentKind::Number => view! {
            <input type="number" prop:value=argument.clone() on:change=move |event| {
                update_calculation_argument(calculated_fields, on_calculated_fields_change, calculation_id, arg_id, event_target_value(&event));
            }/>
        }.into_any(),
        CalculationArgumentKind::Date => view! {
            <input type="date" prop:value=argument.clone() on:change=move |event| {
                update_calculation_argument(calculated_fields, on_calculated_fields_change, calculation_id, arg_id, event_target_value(&event));
            }/>
        }.into_any(),
        CalculationArgumentKind::Text => view! {
            <input prop:value=argument.clone() on:change=move |event| {
                update_calculation_argument(calculated_fields, on_calculated_fields_change, calculation_id, arg_id, event_target_value(&event));
            }/>
        }.into_any(),
    }
}

fn update_calculation_argument(
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
    calculation_id: u64,
    arg_id: u64,
    value: String,
) {
    mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
        if let Some(field) = items.iter_mut().find(|field| field.id == calculation_id)
            && let Some(function) = field
                .functions
                .iter_mut()
                .find(|function| function.id == arg_id)
        {
            function.argument = value;
        }
    });
}

fn update_calculation_argument_field(
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
    calculation_id: u64,
    arg_id: u64,
    value: String,
) {
    mutate_calculations(calculated_fields, on_calculated_fields_change, |items| {
        if let Some(field) = items.iter_mut().find(|field| field.id == calculation_id)
            && let Some(function) = field
                .functions
                .iter_mut()
                .find(|function| function.id == arg_id)
        {
            function.argument_field_key = value;
        }
    });
}

fn mutate_calculations(
    calculated_fields: Signal<Vec<DatasetCalculatedFieldDraft>>,
    on_calculated_fields_change: Callback<Vec<DatasetCalculatedFieldDraft>>,
    update: impl FnOnce(&mut Vec<DatasetCalculatedFieldDraft>),
) {
    let mut items = calculated_fields.get();
    update(&mut items);
    on_calculated_fields_change.run(items);
}

fn calculation_current_argument_mode(
    calculations: Vec<DatasetCalculatedFieldDraft>,
    calculation_id: u64,
    function_id: u64,
) -> String {
    calculations
        .into_iter()
        .find(|field| field.id == calculation_id)
        .and_then(|field| {
            field
                .functions
                .into_iter()
                .find(|function| function.id == function_id)
        })
        .map(|function| {
            if function.argument_mode == "field" {
                "field".into()
            } else {
                "value".into()
            }
        })
        .unwrap_or_else(|| "value".into())
}

fn calculation_current_argument(
    calculations: Vec<DatasetCalculatedFieldDraft>,
    calculation_id: u64,
    function_id: u64,
) -> String {
    calculations
        .into_iter()
        .find(|field| field.id == calculation_id)
        .and_then(|field| {
            field
                .functions
                .into_iter()
                .find(|function| function.id == function_id)
        })
        .map(|function| function.argument)
        .unwrap_or_default()
}

fn calculation_current_argument_field(
    calculations: Vec<DatasetCalculatedFieldDraft>,
    calculation_id: u64,
    function_id: u64,
) -> String {
    calculations
        .into_iter()
        .find(|field| field.id == calculation_id)
        .and_then(|field| {
            field
                .functions
                .into_iter()
                .find(|function| function.id == function_id)
        })
        .map(|function| function.argument_field_key)
        .unwrap_or_default()
}

fn compatible_argument_fields(
    fields: Vec<DatasetFieldDraft>,
    argument_kind: &CalculationArgumentKind,
    input_type: Option<&str>,
) -> Vec<DatasetFieldDraft> {
    let mut fields = fields
        .into_iter()
        .filter(|field| match argument_kind {
            CalculationArgumentKind::Number | CalculationArgumentKind::Integer => {
                field.field_type == "number"
            }
            CalculationArgumentKind::Boolean => field.field_type == "boolean",
            CalculationArgumentKind::Date => {
                matches!(field.field_type.as_str(), "date" | "datetime" | "timestamp")
            }
            CalculationArgumentKind::Text => {
                matches!(field.field_type.as_str(), "text" | "static_text")
            }
            CalculationArgumentKind::None => false,
        })
        .filter(|field| {
            input_type
                .filter(|_| {
                    matches!(
                        argument_kind,
                        CalculationArgumentKind::Boolean | CalculationArgumentKind::Date
                    )
                })
                .map(|input_type| argument_field_type_matches_input(field, input_type))
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    fields.sort_by(|left, right| left.key.cmp(&right.key));
    fields
}

fn argument_field_type_matches_input(field: &DatasetFieldDraft, input_type: &str) -> bool {
    match input_type {
        "date" | "datetime" | "timestamp" => {
            matches!(field.field_type.as_str(), "date" | "datetime" | "timestamp")
        }
        "boolean" => field.field_type == "boolean",
        _ => field.field_type == input_type,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CalculationArgumentKind {
    None,
    Text,
    Number,
    Integer,
    Boolean,
    Date,
}

fn calculation_function_argument_kind(
    function: &str,
    input_type: Option<&str>,
) -> CalculationArgumentKind {
    match function {
        "trim" | "uppercase" | "lowercase" | "to_text" | "to_number" | "to_boolean" | "to_date"
        | "is_empty" | "is_not_empty" => CalculationArgumentKind::None,
        "equal" | "not_equal" => match input_type {
            Some("number") => CalculationArgumentKind::Number,
            Some("boolean") => CalculationArgumentKind::Boolean,
            Some("date") | Some("datetime") | Some("timestamp") => CalculationArgumentKind::Date,
            _ => CalculationArgumentKind::Text,
        },
        "add" | "subtract" | "multiply" | "divide" => CalculationArgumentKind::Number,
        "greater_than" | "greater_than_or_equal" | "less_than" | "less_than_or_equal" => {
            match input_type {
                Some("date") | Some("datetime") | Some("timestamp") => {
                    CalculationArgumentKind::Date
                }
                _ => CalculationArgumentKind::Number,
            }
        }
        "round" => CalculationArgumentKind::Integer,
        "constant" | "coalesce" => match input_type {
            Some("number") => CalculationArgumentKind::Number,
            Some("boolean") => CalculationArgumentKind::Boolean,
            Some("date") | Some("datetime") | Some("timestamp") => CalculationArgumentKind::Date,
            _ => CalculationArgumentKind::Text,
        },
        "format_date" | "prefix" | "suffix" | "concat" | "map_value" => {
            CalculationArgumentKind::Text
        }
        _ => CalculationArgumentKind::Text,
    }
}

fn calculation_function_uses_argument(function: &str) -> bool {
    !matches!(
        function,
        "trim"
            | "uppercase"
            | "lowercase"
            | "to_text"
            | "to_number"
            | "to_boolean"
            | "to_date"
            | "is_empty"
            | "is_not_empty"
    )
}

fn calculation_function_accepts_field_argument(function: &str) -> bool {
    matches!(
        function,
        "prefix"
            | "suffix"
            | "concat"
            | "coalesce"
            | "constant"
            | "add"
            | "subtract"
            | "multiply"
            | "divide"
            | "greater_than"
            | "greater_than_or_equal"
            | "less_than"
            | "less_than_or_equal"
            | "equal"
            | "not_equal"
    )
}

fn calculation_function_label(function: &str) -> String {
    calculation_function_options(None)
        .into_iter()
        .chain(calculation_function_options(Some("number")))
        .chain(calculation_function_options(Some("date")))
        .chain(calculation_function_options(Some("boolean")))
        .find(|option| option.value == function)
        .map(|option| option.label)
        .unwrap_or(function)
        .to_string()
}

fn calculation_function_argument_hint(function: &str) -> &'static str {
    match function {
        "trim" | "uppercase" | "lowercase" => "No argument needed.",
        "to_text" => "Casts the current value to text. No argument needed.",
        "to_number" => "Casts the current value to a number. No argument needed.",
        "to_boolean" => "Casts true, t, 1, yes, or y to true. No argument needed.",
        "to_date" => "Casts the current value to a date. No argument needed.",
        "equal" => "Value to compare against; returns true when the current value matches.",
        "not_equal" => "Value to compare against; returns true when the current value differs.",
        "is_empty" => "Returns true when the current value is empty. No argument needed.",
        "is_not_empty" => "Returns true when the current value is not empty. No argument needed.",
        "prefix" => "Text to add before the current value.",
        "suffix" => "Text to add after the current value.",
        "concat" => "Text to append to the current value.",
        "coalesce" => "Fallback value when the current value is empty.",
        "constant" => "Replacement value to use for every row.",
        "map_value" => {
            "Use from=>to pairs separated by commas, such as draft=>internal, submitted=>public."
        }
        "add" => "Number to add to the current value.",
        "subtract" => "Number to subtract from the current value.",
        "multiply" => "Number to multiply the current value by.",
        "divide" => "Number to divide the current value by.",
        "round" => "Number of decimal places, such as 0 or 2.",
        "format_date" => "Date format pattern, such as %Y-%m-%d or %b %d, %Y.",
        "greater_than" => {
            "Value to compare against; returns true when the current value is greater."
        }
        "greater_than_or_equal" => {
            "Value to compare against; returns true when the current value is greater or equal."
        }
        "less_than" => "Value to compare against; returns true when the current value is less.",
        "less_than_or_equal" => {
            "Value to compare against; returns true when the current value is less or equal."
        }
        _ => "Enter the function argument.",
    }
}

#[derive(Clone, Copy)]
struct CalculationFunctionOption {
    value: &'static str,
    label: &'static str,
}

fn calculation_function_options(field_type: Option<&str>) -> Vec<CalculationFunctionOption> {
    let mut options = vec![
        CalculationFunctionOption {
            value: "constant",
            label: "Constant",
        },
        CalculationFunctionOption {
            value: "coalesce",
            label: "Default",
        },
        CalculationFunctionOption {
            value: "to_text",
            label: "Cast to Text",
        },
        CalculationFunctionOption {
            value: "equal",
            label: "Equal",
        },
        CalculationFunctionOption {
            value: "not_equal",
            label: "Not Equal",
        },
        CalculationFunctionOption {
            value: "is_empty",
            label: "Empty",
        },
        CalculationFunctionOption {
            value: "is_not_empty",
            label: "Not Empty",
        },
    ];
    match field_type {
        Some("number") => options.extend([
            CalculationFunctionOption {
                value: "add",
                label: "Add",
            },
            CalculationFunctionOption {
                value: "subtract",
                label: "Subtract",
            },
            CalculationFunctionOption {
                value: "multiply",
                label: "Multiply",
            },
            CalculationFunctionOption {
                value: "divide",
                label: "Divide",
            },
            CalculationFunctionOption {
                value: "round",
                label: "Round",
            },
            CalculationFunctionOption {
                value: "greater_than",
                label: "Greater Than",
            },
            CalculationFunctionOption {
                value: "greater_than_or_equal",
                label: "Greater Than or Equal",
            },
            CalculationFunctionOption {
                value: "less_than",
                label: "Less Than",
            },
            CalculationFunctionOption {
                value: "less_than_or_equal",
                label: "Less Than or Equal",
            },
        ]),
        Some("date") | Some("datetime") | Some("timestamp") => options.extend([
            CalculationFunctionOption {
                value: "greater_than",
                label: "Greater Than",
            },
            CalculationFunctionOption {
                value: "greater_than_or_equal",
                label: "Greater Than or Equal",
            },
            CalculationFunctionOption {
                value: "less_than",
                label: "Less Than",
            },
            CalculationFunctionOption {
                value: "less_than_or_equal",
                label: "Less Than or Equal",
            },
            CalculationFunctionOption {
                value: "format_date",
                label: "Format Date",
            },
            CalculationFunctionOption {
                value: "to_date",
                label: "Cast to Date",
            },
        ]),
        Some("boolean") => options.extend([CalculationFunctionOption {
            value: "to_boolean",
            label: "Cast to Boolean",
        }]),
        _ => options.extend([
            CalculationFunctionOption {
                value: "trim",
                label: "Trim",
            },
            CalculationFunctionOption {
                value: "uppercase",
                label: "Uppercase",
            },
            CalculationFunctionOption {
                value: "lowercase",
                label: "Lowercase",
            },
            CalculationFunctionOption {
                value: "prefix",
                label: "Prefix",
            },
            CalculationFunctionOption {
                value: "suffix",
                label: "Suffix",
            },
            CalculationFunctionOption {
                value: "concat",
                label: "Concat",
            },
            CalculationFunctionOption {
                value: "map_value",
                label: "Map Value",
            },
            CalculationFunctionOption {
                value: "to_number",
                label: "Cast to Number",
            },
            CalculationFunctionOption {
                value: "to_boolean",
                label: "Cast to Boolean",
            },
            CalculationFunctionOption {
                value: "to_date",
                label: "Cast to Date",
            },
        ]),
    }
    options
}

fn default_calculation_function_for_input_type(field_type: Option<&str>) -> &'static str {
    match field_type {
        Some("number") => "add",
        Some("date") | Some("datetime") | Some("timestamp") => "format_date",
        Some("boolean") => "to_text",
        _ => "trim",
    }
}

fn calculation_current_function(
    calculations: Vec<DatasetCalculatedFieldDraft>,
    calculation_id: u64,
    function_id: u64,
) -> Option<String> {
    calculations
        .into_iter()
        .find(|field| field.id == calculation_id)?
        .functions
        .into_iter()
        .find(|function| function.id == function_id)
        .map(|function| function.function)
}

fn calculation_input_type_for_function(
    calculations: Vec<DatasetCalculatedFieldDraft>,
    fields: Vec<DatasetFieldDraft>,
    calculation_id: u64,
    function_id: u64,
) -> Option<String> {
    let calculation = calculations
        .into_iter()
        .find(|field| field.id == calculation_id)?;
    let mut field_type = fields
        .into_iter()
        .find(|field| field.key == calculation.base_field_key)
        .map(|field| field.field_type)?;
    for function in calculation.functions {
        if function.id == function_id {
            return Some(field_type);
        }
        field_type = calculation_function_output_type(&function.function, &field_type);
    }
    Some(field_type)
}

fn calculation_output_type_at_index(
    calculation: &DatasetCalculatedFieldDraft,
    fields: &[DatasetFieldDraft],
    function_index: usize,
) -> Option<String> {
    let mut field_type = base_field_type_for_calculation(calculation, fields)?;
    for function in calculation.functions.iter().take(function_index) {
        field_type = calculation_function_output_type(&function.function, &field_type);
    }
    Some(field_type)
}

fn base_field_type_for_calculation(
    calculation: &DatasetCalculatedFieldDraft,
    fields: &[DatasetFieldDraft],
) -> Option<String> {
    fields
        .iter()
        .find(|field| field.key == calculation.base_field_key)
        .map(|field| field.field_type.clone())
}

fn normalize_calculation_function_pipeline(
    calculation: &mut DatasetCalculatedFieldDraft,
    fields: &[DatasetFieldDraft],
) {
    let Some(mut field_type) = base_field_type_for_calculation(calculation, fields) else {
        return;
    };
    for function in &mut calculation.functions {
        let allowed = calculation_function_options(Some(&field_type))
            .into_iter()
            .any(|option| option.value == function.function);
        if !allowed {
            function.function =
                default_calculation_function_for_input_type(Some(&field_type)).into();
            function.argument.clear();
            function.argument_mode = "value".into();
            function.argument_field_key.clear();
        } else if !calculation_function_uses_argument(&function.function) {
            function.argument.clear();
            function.argument_mode = "value".into();
            function.argument_field_key.clear();
        } else if !calculation_function_accepts_field_argument(&function.function) {
            function.argument_mode = "value".into();
            function.argument_field_key.clear();
        }
        field_type = calculation_function_output_type(&function.function, &field_type);
    }
}

fn calculation_function_output_type(function: &str, input_type: &str) -> String {
    match function {
        "trim" | "uppercase" | "lowercase" | "prefix" | "suffix" | "concat" | "map_value"
        | "format_date" | "to_text" => "text".into(),
        "add" | "subtract" | "multiply" | "divide" | "round" | "to_number" => "number".into(),
        "greater_than"
        | "greater_than_or_equal"
        | "less_than"
        | "less_than_or_equal"
        | "equal"
        | "not_equal"
        | "is_empty"
        | "is_not_empty"
        | "to_boolean" => "boolean".into(),
        "to_date" => "date".into(),
        "coalesce" | "constant" => input_type.into(),
        _ => input_type.into(),
    }
}

fn calculation_preview(fields: Vec<DatasetCalculatedFieldDraft>, calculation_id: u64) -> String {
    let Some(field) = fields.into_iter().find(|field| field.id == calculation_id) else {
        return String::new();
    };
    let mut parts = vec![field.base_field_key];
    parts.extend(field.functions.into_iter().map(|function| {
        if calculation_function_uses_argument(&function.function) {
            let argument = if function.argument_mode == "field" {
                function.argument_field_key
            } else {
                function.argument
            };
            format!("{}({})", function.function, argument)
        } else {
            function.function
        }
    }));
    format!("{} = {}", field.key, parts.join(" | "))
}
