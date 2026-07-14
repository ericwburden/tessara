//! Component editor controls and kind-specific configuration surfaces.

use super::*;
use std::collections::BTreeMap;

#[component]
pub(super) fn TableDefaultsControls(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    page_size: RwSignal<String>,
) -> impl IntoView {
    view! {
        <ComponentEditorFieldset title="Table Defaults" class="component-editor__table-defaults">
            <label class="form-field">
                <span>"Sort Field"</span>
                <select aria-label="Order categories by" prop:value=move || sort_field.get() on:change=move |event| sort_field.set(event_target_value(&event))>
                    <option value="">"Default row order"</option>
                    {move || fields.get().into_iter().map(|field| {
                        let label = format!("{} ({})", field.label, field.field_type);
                        view! { <option value=field.key>{label}</option> }
                    }).collect_view()}
                </select>
            </label>
            <label class="form-field">
                <span>"Sort Direction"</span>
                <select prop:value=move || sort_direction.get() on:change=move |event| sort_direction.set(event_target_value(&event))>
                    <option value="asc">"Ascending"</option>
                    <option value="desc">"Descending"</option>
                </select>
            </label>
            <label class="form-field">
                <span>"Page Size"</span>
                <input
                    type="number"
                    min="1"
                    max="200"
                    prop:value=move || page_size.get()
                    on:input=move |event| page_size.set(event_target_value(&event))
                />
            </label>
        </ComponentEditorFieldset>
    }
}

#[component]
pub(super) fn ComponentKindControls(
    component_type: RwSignal<String>,
    has_kind_specific_changes: Signal<bool>,
    on_kind_change: Callback<String>,
) -> impl IntoView {
    const KINDS: [(&str, &str); 6] = [
        ("table", "Table"),
        ("bar", "Bar"),
        ("line", "Line"),
        ("pie", "Pie"),
        ("donut", "Donut"),
        ("stat_card", "Stat Card"),
    ];
    let pending_kind = RwSignal::new(None::<String>);
    let request_kind_change = move |next_kind: String| {
        if next_kind == component_type.get_untracked() {
            return;
        }
        if has_kind_specific_changes.get() {
            pending_kind.set(Some(next_kind));
        } else {
            on_kind_change.run(next_kind);
        }
    };
    view! {
        <ComponentEditorFieldset title="Component Kind" class="component-editor__kind-panel">
            <div class="component-editor__kind-grid" role="radiogroup" aria-label="Component kind">
                {KINDS.into_iter().map(|(value, label)| {
                    view! {
                        <button
                            type="button"
                            role="radio"
                            class:component-editor__kind-button=true
                            class:is-selected=move || component_type.get() == value
                            aria-checked=move || (component_type.get() == value).to_string()
                            on:click=move |_| request_kind_change(value.to_string())
                        >
                            {label}
                        </button>
                    }
                }).collect_view()}
            </div>
            <label class="form-field component-editor__kind-select">
                <span>"Kind"</span>
                <select prop:value=move || component_type.get() on:change=move |event| request_kind_change(event_target_value(&event))>
                    {KINDS.into_iter().map(|(value, label)| view! { <option value=value>{label}</option> }).collect_view()}
                </select>
            </label>
            <p class="component-kind-description">
                {move || component_kind_description(&component_type.get())}
            </p>
            {move || pending_kind.get().map(|next_kind| {
                let next_label = component_type_label(&next_kind);
                let confirm_kind = next_kind.clone();
                view! {
                    <div class="component-editor__kind-confirmation" role="alertdialog" aria-label="Confirm component kind change">
                        <p>{format!("Change to {next_label}? Kind-specific settings in this draft will be cleared. Dataset binding and filters will be kept.")}</p>
                        <div>
                            <button class="button button--quiet" type="button" on:click=move |_| pending_kind.set(None)>"Cancel"</button>
                            <button class="button" type="button" on:click=move |_| {
                                pending_kind.set(None);
                                on_kind_change.run(confirm_kind.clone());
                            }>{format!("Change to {next_label}")}</button>
                        </div>
                    </div>
                }
            })}
        </ComponentEditorFieldset>
    }
}

#[cfg(feature = "hydrate")]
pub(super) fn focus_component_kind_editor() {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once(Box::new(move || {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        if let Ok(Some(element)) = document.query_selector("[data-component-kind-editor]")
            && let Some(element) = element.dyn_ref::<web_sys::HtmlElement>()
        {
            let _ = element.focus();
        }
    }) as Box<dyn FnOnce()>);
    let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    callback.forget();
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn focus_component_kind_editor() {}

#[cfg(feature = "hydrate")]
pub(super) fn focus_component_preview_drawer() {
    focus_component_editor_element("component-editor-preview-drawer");
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn focus_component_preview_drawer() {}

#[cfg(feature = "hydrate")]
pub(super) fn focus_component_preview_button() {
    focus_component_editor_element("component-editor-preview-fab");
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn focus_component_preview_button() {}

#[cfg(feature = "hydrate")]
pub(super) fn focus_component_preview_close_button() {
    focus_component_editor_element("component-editor-preview-close");
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn focus_component_preview_close_button() {}

#[cfg(feature = "hydrate")]
pub(super) fn focus_component_dataset_picker_trigger() {
    focus_component_editor_element("component-dataset-picker-trigger");
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn focus_component_dataset_picker_trigger() {}

#[cfg(feature = "hydrate")]
fn focus_component_editor_element(id: &'static str) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let callback = Closure::once(Box::new(move || {
        let Some(document) = web_sys::window().and_then(|window| window.document()) else {
            return;
        };
        if let Some(element) = document.get_element_by_id(id)
            && let Some(element) = element.dyn_ref::<web_sys::HtmlElement>()
        {
            let _ = element.focus();
        }
    }) as Box<dyn FnOnce()>);
    let _ = window.request_animation_frame(callback.as_ref().unchecked_ref());
    callback.forget();
}

#[cfg(feature = "hydrate")]
pub(super) fn cancel_component_edit() {
    let Some(window) = web_sys::window() else {
        return;
    };
    if window
        .history()
        .ok()
        .and_then(|history| history.length().ok().map(|length| (history, length)))
        .is_some_and(|(history, length)| length > 1 && history.back().is_ok())
    {
        return;
    }
    let _ = window.location().set_href("/components");
}

#[cfg(not(feature = "hydrate"))]
pub(super) fn cancel_component_edit() {}

#[component]
#[allow(clippy::too_many_arguments)]
pub(super) fn BarConfigEditor(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    category_field: RwSignal<String>,
    category_labels: RwSignal<String>,
    category_colors: RwSignal<String>,
    legend_title: RwSignal<String>,
    comparison_field: RwSignal<String>,
    orientation: RwSignal<String>,
    comparison_layout: RwSignal<String>,
    x_axis_label: RwSignal<String>,
    y_axis_label: RwSignal<String>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    limit: RwSignal<String>,
    value_format: RwSignal<String>,
    category_missing_policy: RwSignal<String>,
    comparison_missing_policy: RwSignal<String>,
    value_missing_policy: RwSignal<String>,
) -> impl IntoView {
    let split_bars = RwSignal::new(!comparison_field.get_untracked().trim().is_empty());
    let last_comparison_field = RwSignal::new(comparison_field.get_untracked());

    Effect::new(move |_| {
        let comparison = comparison_field.get();
        if !comparison.trim().is_empty() {
            split_bars.set(true);
            last_comparison_field.set(comparison);
        }
    });
    Effect::new(move |_| {
        let calculation = summary_type.get();
        if comparison_layout.get_untracked() == "stacked"
            && !matches!(calculation.as_str(), "row_count" | "count" | "sum")
        {
            comparison_layout.set("grouped".into());
        }
    });

    view! {
        <ComponentEditorFieldset title="Fields & Calculation" class="component-editor__bar-fields">
            <header class="component-editor__section-heading">
                <h2>"Build the bars"</h2>
                <p>"Choose what creates each bar and how its value is calculated."</p>
            </header>
            <div class="component-editor__role-grid">
                <section class="component-editor__role-card">
                    <header class="component-editor__role-card-header">
                        <strong>"Category"</strong>
                        <HeaderHelpButton label="Category" help="Creates one group for every distinct Category Field value."/>
                    </header>
                    <p>"What should each row or column of bars represent?"</p>
                    <label class="form-field">
                        <span>"Category field"</span>
                        <select prop:value=move || category_field.get() on:change=move |event| {
                            let next = event_target_value(&event);
                            if next != category_field.get_untracked() && comparison_field.get_untracked().trim().is_empty() {
                                category_labels.set(String::new());
                                category_colors.set(String::new());
                            }
                            category_field.set(next);
                        }>
                            <option value="">"Select field"</option>
                            {move || fields.get().into_iter().map(|field| field_option_selected(field, category_field)).collect_view()}
                        </select>
                    </label>
                    <label class="form-field">
                        <span>"Missing categories"</span>
                        <select prop:value=move || category_missing_policy.get() on:change=move |event| category_missing_policy.set(event_target_value(&event))>
                            <option value="omit">"Omit rows"</option>
                            <option value="explicit_missing">"Show as Missing"</option>
                        </select>
                    </label>
                </section>

                <section class="component-editor__role-card">
                    <header class="component-editor__role-card-header">
                        <strong>"Series"</strong>
                        <label class="component-editor__switch-row">
                            <span>"Split bars"</span>
                            <input
                                type="checkbox"
                                prop:checked=move || split_bars.get()
                                on:change=move |event| {
                                    let enabled = event_target_checked(&event);
                                    split_bars.set(enabled);
                                    if enabled {
                                        let previous = last_comparison_field.get_untracked();
                                        if !previous.trim().is_empty() {
                                            comparison_field.set(previous);
                                        }
                                    } else {
                                        let current = comparison_field.get_untracked();
                                        if !current.trim().is_empty() {
                                            last_comparison_field.set(current);
                                        }
                                        comparison_field.set(String::new());
                                        category_labels.set(String::new());
                                        category_colors.set(String::new());
                                        legend_title.set(String::new());
                                    }
                                }
                            />
                        </label>
                    </header>
                    <p>"Optionally compare a second dimension within each category."</p>
                    {move || split_bars.get().then(|| view! {
                        <label class="form-field">
                            <span>"Series field"</span>
                            <select prop:value=move || comparison_field.get() on:change=move |event| {
                                let next = event_target_value(&event);
                                if next != comparison_field.get_untracked() {
                                    category_labels.set(String::new());
                                    category_colors.set(String::new());
                                    legend_title.set(field_label_for_key(&fields.get_untracked(), &next).unwrap_or_default());
                                }
                                comparison_field.set(next);
                            }>
                                <option value="">"Select field"</option>
                                {move || fields.get().into_iter().map(|field| field_option_selected(field, comparison_field)).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Missing series"</span>
                            <select prop:value=move || comparison_missing_policy.get() on:change=move |event| comparison_missing_policy.set(event_target_value(&event))>
                                <option value="omit">"Omit rows"</option>
                                <option value="explicit_missing">"Show as Missing"</option>
                            </select>
                        </label>
                    })}
                </section>

                <section class="component-editor__role-card component-editor__role-card--measure">
                    <header class="component-editor__role-card-header">
                        <strong>"Measure"</strong>
                        <HeaderHelpButton label="Measure" help="Calculates one value for every Category and Series group using the selected Calculation and Value Field."/>
                    </header>
                    <p>"What number should determine the length of each bar?"</p>
                    <div class="component-editor__measure-grid">
                        <label class="form-field">
                            <span>"Calculation"</span>
                            <select prop:value=move || summary_type.get() on:change=move |event| summary_type.set(event_target_value(&event))>
                                <option value="row_count">"Count rows"</option>
                                <option value="count">"Count non-empty values"</option>
                                <option value="unique_count">"Count unique values"</option>
                                <option value="sum">"Sum"</option>
                                <option value="average">"Average"</option>
                                <option value="median">"Median"</option>
                                <option value="none">"Do not summarize"</option>
                            </select>
                        </label>
                        {move || (summary_type.get() != "row_count").then(|| view! {
                            <label class="form-field">
                                <span>"Value field"</span>
                                <select prop:value=move || summary_field.get() on:change=move |event| summary_field.set(event_target_value(&event))>
                                    <option value="">"Select field"</option>
                                    {move || fields.get().into_iter().map(|field| field_option_selected(field, summary_field)).collect_view()}
                                </select>
                            </label>
                            <label class="form-field">
                                <span>"Missing values"</span>
                                <select prop:value=move || value_missing_policy.get() on:change=move |event| value_missing_policy.set(event_target_value(&event))>
                                    <option value="omit">"Omit missing values"</option>
                                    <option value="zero">"Treat missing as zero"</option>
                                    <option value="explicit_missing">"Include as a distinct value"</option>
                                </select>
                            </label>
                        })}
                        {move || (summary_type.get() == "none").then(|| view! {
                            <p class="form-message form-message--warning component-editor__calculation-warning">
                                "Every category and series group must resolve to exactly one row. Preview and execution will report an error when duplicates exist."
                            </p>
                        })}
                        <div class="component-editor__calculation-summary">
                            <CircleHelp class="component-field-help__glyph"/>
                            <span>{move || bar_calculation_summary(
                                &fields.get(),
                                &summary_field.get(),
                                &summary_type.get(),
                                &category_field.get(),
                                &comparison_field.get(),
                            )}</span>
                        </div>
                    </div>
                </section>
            </div>
        </ComponentEditorFieldset>

        <ComponentEditorFieldset title="Order & Display" class="component-editor__bar-display">
            <label class="form-field">
                <FieldHelpLabel label="Order categories by" help="Category label: sorts alphabetically or chronologically by the Category Field.\nTotal value: sorts by the summarized value across all series."/>
                <select prop:value=move || sort_field.get() on:change=move |event| sort_field.set(event_target_value(&event))>
                    <option value="category">"Category label"</option>
                    <option value="summary_value">"Total value"</option>
                </select>
            </label>
            <label class="form-field">
                <span>"Direction"</span>
                <select prop:value=move || sort_direction.get() on:change=move |event| sort_direction.set(event_target_value(&event))>
                    <option value="desc">"Descending"</option>
                    <option value="asc">"Ascending"</option>
                </select>
            </label>
            <label class="form-field">
                <FieldHelpLabel label="Category Limit" help="Applied to categories after sorting, not to individual series bars."/>
                <input aria-label="Category Limit" type="number" min="1" max="100" prop:value=move || limit.get() on:input=move |event| limit.set(event_target_value(&event))/>
            </label>
            <label class="form-field">
                <span>"Orientation"</span>
                <select prop:value=move || orientation.get() on:change=move |event| {
                    let next = event_target_value(&event);
                    if next != orientation.get_untracked() {
                        let previous_x = x_axis_label.get_untracked();
                        x_axis_label.set(y_axis_label.get_untracked());
                        y_axis_label.set(previous_x);
                    }
                    orientation.set(next);
                }>
                    <option value="horizontal">"Horizontal"</option>
                    <option value="vertical">"Vertical"</option>
                </select>
            </label>
            {move || (!comparison_field.get().trim().is_empty()).then(|| view! {
                <label class="form-field">
                    <DynamicFieldHelpLabel
                        label="Comparison Layout"
                        help=Signal::derive(move || if matches!(summary_type.get().as_str(), "row_count" | "count" | "sum") {
                            "Stacked is available for row count, count, and sum calculations.".to_string()
                        } else {
                            "Stacked is unavailable for this non-additive calculation.".to_string()
                        })
                    />
                    <select aria-label="Comparison Layout" prop:value=move || comparison_layout.get() on:change=move |event| comparison_layout.set(event_target_value(&event))>
                        <option value="grouped">"Grouped"</option>
                        <option value="stacked" disabled=move || !matches!(summary_type.get().as_str(), "row_count" | "count" | "sum")>"Stacked"</option>
                    </select>
                </label>
            })}
            <label class="form-field">
                <span>"Value format"</span>
                <select prop:value=move || value_format.get() on:change=move |event| value_format.set(event_target_value(&event))>
                    <option value="plain">"Plain"</option>
                    <option value="integer">"Integer"</option>
                    <option value="decimal">"Decimal"</option>
                    <option value="percent">"Percent"</option>
                </select>
            </label>
            <label class="form-field">
                <span>"Category axis title"</span>
                <input prop:value=move || if orientation.get() == "horizontal" { y_axis_label.get() } else { x_axis_label.get() } on:input=move |event| {
                    if orientation.get_untracked() == "horizontal" {
                        y_axis_label.set(event_target_value(&event));
                    } else {
                        x_axis_label.set(event_target_value(&event));
                    }
                }/>
            </label>
            <label class="form-field">
                <span>"Value axis title"</span>
                <input prop:value=move || if orientation.get() == "horizontal" { x_axis_label.get() } else { y_axis_label.get() } on:input=move |event| {
                    if orientation.get_untracked() == "horizontal" {
                        x_axis_label.set(event_target_value(&event));
                    } else {
                        y_axis_label.set(event_target_value(&event));
                    }
                }/>
            </label>
        </ComponentEditorFieldset>
    }
}

fn bar_calculation_summary(
    fields: &[DatasetFieldDefinition],
    summary_field: &str,
    summary_type: &str,
    category_field: &str,
    comparison_field: &str,
) -> String {
    let value =
        field_label_for_key(fields, summary_field).unwrap_or_else(|| "the selected field".into());
    let category =
        field_label_for_key(fields, category_field).unwrap_or_else(|| "each category".into());
    let series = field_label_for_key(fields, comparison_field)
        .map(|label| format!(" and {label}"))
        .unwrap_or_default();
    let calculation = match summary_type {
        "row_count" => return format!("Counts rows for every {category}{series} group."),
        "unique_count" => "Counts unique values of",
        "sum" => "Adds",
        "average" => "Averages",
        "median" => "Finds the median of",
        "none" => "Uses the single value of",
        _ => "Counts non-empty values of",
    };
    format!("{calculation} {value} for every {category}{series} group.")
}

#[component]
fn VisualMeasureEditor(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    value_missing_policy: RwSignal<String>,
) -> impl IntoView {
    view! {
        <label class="form-field component-editor__calculation-field">
            <FieldHelpLabel label="Calculation" help="Count rows: counts every participating row.\nCount non-empty values: counts rows with a Value field value.\nCount unique values: counts distinct Value field values.\nSum: adds numeric values.\nAverage: averages numeric values.\nMedian: returns the middle numeric value.\nDo not summarize: requires exactly one row per group."/>
            <select aria-label="Calculation" prop:value=move || summary_type.get() on:change=move |event| summary_type.set(event_target_value(&event))>
                <option value="row_count">"Count rows"</option>
                <option value="count">"Count non-empty values"</option>
                <option value="unique_count">"Count unique values"</option>
                <option value="sum">"Sum"</option>
                <option value="average">"Average"</option>
                <option value="median">"Median"</option>
                <option value="none">"Do not summarize"</option>
            </select>
        </label>
        {move || (summary_type.get() != "row_count").then(|| view! {
                <label class="form-field component-editor__value-field">
                    <FieldHelpLabel label="Value field" help="Choose the Dataset output field whose values are counted or summarized."/>
                    <select aria-label="Value field" prop:value=move || summary_field.get() on:change=move |event| summary_field.set(event_target_value(&event))>
                        <option value="">"Select field"</option>
                        {move || fields.get().into_iter().map(|field| field_option_selected(field, summary_field)).collect_view()}
                    </select>
                </label>
                <label class="form-field component-editor__missing-measure-field">
                    <FieldHelpLabel label="Missing measure values" help="Omit: skips missing values.\nZero: treats missing numeric values as 0 where supported.\nExplicit Missing: includes missing values as a distinct value for compatible calculations."/>
                    <select aria-label="Missing measure values" prop:value=move || value_missing_policy.get() on:change=move |event| value_missing_policy.set(event_target_value(&event))>
                        <option value="omit">"Omit"</option>
                        <option value="zero">"Zero"</option>
                        <option value="explicit_missing">"Explicit Missing"</option>
                    </select>
                </label>
        })}
        {move || (summary_type.get() == "none").then(|| view! {
            <p class="form-message form-message--warning component-editor__calculation-warning">
                "Every group must resolve to exactly one row. Preview and execution report an error when duplicates exist."
            </p>
        })}
    }
}

#[component]
fn VisualOrderEditor(
    kind: &'static str,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    limit: RwSignal<String>,
    value_format: RwSignal<String>,
) -> impl IntoView {
    Effect::new(move |_| {
        let allowed = visual_sort_field_options(kind, "");
        let current = sort_field.get();
        if !current.is_empty() && !allowed.iter().any(|(value, _)| *value == current) {
            sort_field.set(String::new());
        }
    });
    view! {
        <label class="form-field component-editor__sort-field">
            <DynamicFieldHelpLabel label="Sort Field" help=Signal::derive(move || visual_sort_field_help(kind, ""))/>
            <select aria-label="Sort Field" prop:value=move || sort_field.get() on:change=move |event| sort_field.set(event_target_value(&event))>
                {visual_sort_field_options(kind, "").into_iter().map(|(value, label)| {
                    let selected_value = value.to_string();
                    view! { <option value=value prop:selected=move || sort_field.get() == selected_value>{label}</option> }
                }).collect_view()}
            </select>
        </label>
        <label class="form-field component-editor__sort-direction-field">
            <FieldHelpLabel label="Sort Direction" help="Ascending: smallest or A-Z first.\nDescending: largest or Z-A first."/>
            <select aria-label="Sort Direction" prop:value=move || sort_direction.get() on:change=move |event| sort_direction.set(event_target_value(&event))>
                <option value="asc">"Ascending"</option>
                <option value="desc">"Descending"</option>
            </select>
        </label>
        <label class="form-field component-editor__limit-field">
            <FieldHelpLabel label="Limit" help="Caps grouped chart output after sorting. Range is 1 to 100."/>
            <input aria-label="Limit" type="number" min="1" max="100" prop:value=move || limit.get() on:input=move |event| limit.set(event_target_value(&event))/>
        </label>
        <label class="form-field component-editor__format-field">
            <FieldHelpLabel label="Format" help="Plain: compact default number.\nInteger: whole number with no decimals.\nDecimal: two decimal places.\nPercent: multiplies by 100 and appends a percent sign."/>
            <select aria-label="Format" prop:value=move || value_format.get() on:change=move |event| value_format.set(event_target_value(&event))>
                <option value="plain">"Plain"</option>
                <option value="integer">"Integer"</option>
                <option value="decimal">"Decimal"</option>
                <option value="percent">"Percent"</option>
            </select>
        </label>
    }
}

#[component]
#[allow(clippy::too_many_arguments)]
pub(super) fn LineConfigEditor(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    x_field: RwSignal<String>,
    smoothing: RwSignal<bool>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    limit: RwSignal<String>,
    value_format: RwSignal<String>,
    x_missing_policy: RwSignal<String>,
    value_missing_policy: RwSignal<String>,
) -> impl IntoView {
    view! { <ComponentEditorFieldset title="Line Config" class="component-editor__visual-defaults component-editor__line-config">
        <VisualMeasureEditor fields summary_field summary_type value_missing_policy/>
        <label class="form-field component-editor__category-field"><FieldHelpLabel label="Category Field" help="Groups line points along the horizontal axis, such as a date or ordered label."/><select aria-label="Category Field" prop:value=move || x_field.get() on:change=move |event| x_field.set(event_target_value(&event))><option value="">"Select field"</option>{move || fields.get().into_iter().map(|field| field_option_selected(field, x_field)).collect_view()}</select></label>
        <label class="form-field component-editor__missing-category-field"><FieldHelpLabel label="Missing Categories" help="Omit rows: excludes rows without a category. Show as Missing: retains them in a visible missing group."/><select aria-label="Missing Categories" prop:value=move || x_missing_policy.get() on:change=move |event| x_missing_policy.set(event_target_value(&event))><option value="omit">"Omit rows"</option><option value="explicit_missing">"Show as Missing"</option></select></label>
        <VisualOrderEditor kind="line" sort_field sort_direction limit value_format/>
        <div class="form-field component-editor__smoothing-field">
            <FieldHelpLabel label="Smoothing" help="On: draws a smooth curve through the points. Off: connects points with straight line segments."/>
            <div class="segmented-toggle segmented-toggle--binary" role="group" aria-label="Smoothing">
                <button type="button" class:segmented-toggle__option=true class:is-active=move || smoothing.get() on:click=move |_| smoothing.set(true) aria-pressed=move || smoothing.get()>"On"</button>
                <button type="button" class:segmented-toggle__option=true class:is-active=move || !smoothing.get() on:click=move |_| smoothing.set(false) aria-pressed=move || !smoothing.get()>"Off"</button>
            </div>
        </div>
    </ComponentEditorFieldset> }
}

#[component]
#[allow(clippy::too_many_arguments)]
pub(super) fn PieDonutConfigEditor(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    category_field: RwSignal<String>,
    category_labels: RwSignal<String>,
    category_colors: RwSignal<String>,
    legend_title: RwSignal<String>,
    sort_field: RwSignal<String>,
    sort_direction: RwSignal<String>,
    limit: RwSignal<String>,
    value_format: RwSignal<String>,
    category_missing_policy: RwSignal<String>,
    value_missing_policy: RwSignal<String>,
) -> impl IntoView {
    view! { <ComponentEditorFieldset title="Pie / Donut Config" class="component-editor__visual-defaults component-editor__pie-donut-config">
        <VisualMeasureEditor fields summary_field summary_type value_missing_policy/>
        <label class="form-field component-editor__category-field"><FieldHelpLabel label="Category Field" help="Groups results into labeled slices."/><select aria-label="Category Field" prop:value=move || category_field.get() on:change=move |event| { let next=event_target_value(&event); if next != category_field.get_untracked() { category_labels.set(String::new()); category_colors.set(String::new()); legend_title.set(field_label_for_key(&fields.get_untracked(), &next).unwrap_or_default()); } category_field.set(next); }><option value="">"Select field"</option>{move || fields.get().into_iter().map(|field| field_option_selected(field, category_field)).collect_view()}</select></label>
        <label class="form-field component-editor__missing-category-field"><FieldHelpLabel label="Missing categories" help="Omit rows: excludes rows without a category. Show as Missing: retains them in a visible missing group."/><select aria-label="Missing categories" prop:value=move || category_missing_policy.get() on:change=move |event| category_missing_policy.set(event_target_value(&event))><option value="omit">"Omit rows"</option><option value="explicit_missing">"Show as Missing"</option></select></label>
        <VisualOrderEditor kind="pie" sort_field sort_direction limit value_format/>
    </ComponentEditorFieldset> }
}

#[component]
#[allow(clippy::too_many_arguments)]
pub(super) fn StatCardConfigEditor(
    fields: Signal<Vec<DatasetFieldDefinition>>,
    summary_field: RwSignal<String>,
    summary_type: RwSignal<String>,
    value_format: RwSignal<String>,
    value_missing_policy: RwSignal<String>,
    stat_label: RwSignal<String>,
    stat_supporting_text: RwSignal<String>,
    stat_panel_style: RwSignal<String>,
) -> impl IntoView {
    view! { <ComponentEditorFieldset title="Stat Card Config" class="component-editor__visual-defaults component-editor__stat-card-config">
        <VisualMeasureEditor fields summary_field summary_type value_missing_policy/>
        <label class="form-field component-editor__format-field"><FieldHelpLabel label="Format" help="Plain: compact default number.\nInteger: whole number with no decimals.\nDecimal: two decimal places.\nPercent: multiplies by 100 and appends a percent sign."/><select aria-label="Format" prop:value=move || value_format.get() on:change=move |event| value_format.set(event_target_value(&event))><option value="plain">"Plain"</option><option value="integer">"Integer"</option><option value="decimal">"Decimal"</option><option value="percent">"Percent"</option></select></label>
        <label class="form-field component-editor__stat-label-field"><FieldHelpLabel label="Label" help="Overrides the Stat Card display label."/><input aria-label="Label" prop:value=move || stat_label.get() on:input=move |event| stat_label.set(event_target_value(&event))/></label>
        <label class="form-field component-editor__stat-style-field"><FieldHelpLabel label="Panel Style" help="Default: standard emphasis. Muted: quieter supporting value. Accent: stronger highlighted value."/><select aria-label="Panel Style" prop:value=move || stat_panel_style.get() on:change=move |event| stat_panel_style.set(event_target_value(&event))><option value="default">"Default"</option><option value="muted">"Muted"</option><option value="accent">"Accent"</option></select></label>
        <label class="form-field form-field--wide component-editor__stat-supporting-field"><FieldHelpLabel label="Supporting Text" help="Adds short context beneath the Stat Card value."/><input aria-label="Supporting Text" prop:value=move || stat_supporting_text.get() on:input=move |event| stat_supporting_text.set(event_target_value(&event))/></label>
    </ComponentEditorFieldset> }
}

#[component]
pub(super) fn ComponentEditorFieldset(
    title: &'static str,
    class: &'static str,
    children: Children,
) -> impl IntoView {
    let class_name = format!("route-panel__section component-editor-panel {class}");
    view! {
        <fieldset class=class_name>
            <legend>{title}</legend>
            {children()}
        </fieldset>
    }
}

#[component]
fn FieldHelpLabel(label: &'static str, #[prop(into)] help: String) -> impl IntoView {
    view! {
        <span class="component-field-label">
            <span>{label}</span>
            <FieldHelpButton label help/>
        </span>
    }
}

#[component]
fn FieldHelpButton(label: &'static str, #[prop(into)] help: String) -> impl IntoView {
    let help_title = format!("Show help for {label}");
    view! {
        <details class="component-field-help">
            <summary title=help_title.clone() aria-label=help_title>
                <CircleHelp class="component-field-help__glyph"/>
            </summary>
            <span class="component-field-help__content" role="tooltip">{help}</span>
        </details>
    }
}

#[component]
fn HeaderHelpButton(label: &'static str, #[prop(into)] help: String) -> impl IntoView {
    let help_title = format!("Show help for {label}");
    view! {
        <details class="component-field-help">
            <summary title=help_title.clone() aria-label=help_title>
                <CircleHelp class="component-field-help__glyph"/>
            </summary>
            <span class="component-field-help__content" role="tooltip">{help}</span>
        </details>
    }
}

#[component]
fn DynamicFieldHelpLabel(label: &'static str, help: Signal<String>) -> impl IntoView {
    let help_title = format!("Show help for {label}");
    view! {
        <span class="component-field-label">
            <span>{label}</span>
            <details class="component-field-help">
                <summary title=help_title.clone() aria-label=help_title>
                    <CircleHelp class="component-field-help__glyph"/>
                </summary>
                <span class="component-field-help__content" role="tooltip">{move || help.get()}</span>
            </details>
        </span>
    }
}

#[component]
pub(super) fn CategoryDisplayControls(
    dataset_id: RwSignal<String>,
    dataset_major: RwSignal<String>,
    component_type: RwSignal<String>,
    fields: Signal<Vec<DatasetFieldDefinition>>,
    category_field: RwSignal<String>,
    comparison_field: RwSignal<String>,
    category_labels: RwSignal<String>,
    category_colors: RwSignal<String>,
    legend_title: RwSignal<String>,
) -> impl IntoView {
    let category_values = RwSignal::new(Vec::<String>::new());
    let category_values_error = RwSignal::new(None::<String>);
    let active_display_field = RwSignal::new(String::new());
    let active_dataset_major = RwSignal::new(String::new());

    Effect::new(move |_| {
        let kind = component_type.get();
        let selected_dataset_id = dataset_id.get();
        let selected_dataset_major = dataset_major.get();
        let selected_category_field = category_field.get();
        let selected_comparison_field = comparison_field.get();
        let selected_display_field = if kind == "bar" {
            selected_comparison_field
        } else {
            selected_category_field
        };
        active_dataset_major.set(selected_dataset_major.clone());
        active_display_field.set(selected_display_field.clone());
        if !matches!(kind.as_str(), "bar" | "pie" | "donut")
            || selected_dataset_id.trim().is_empty()
            || selected_dataset_major.trim().parse::<i32>().is_err()
            || selected_display_field.trim().is_empty()
        {
            category_values.set(Vec::new());
            category_values_error.set(None);
            return;
        }
        load_category_values(
            selected_dataset_id,
            selected_dataset_major,
            selected_display_field,
            CategoryValueLoadState {
                active_dataset_id: dataset_id,
                active_dataset_major,
                active_category_field: active_display_field,
                values: category_values,
                error: category_values_error,
            },
        );
    });

    view! {
        <ComponentEditorFieldset title="Labels & Colors" class="component-editor__category-display">
            <label class="form-field">
                <FieldHelpLabel label="Legend Title" help="Optional text displayed above the chart legend."/>
                <input
                    aria-label="Legend Title"
                    prop:value=move || legend_title.get()
                    on:input=move |event| legend_title.set(event_target_value(&event))
                />
            </label>
            <CategoryLabelsControl
                component_type
                fields
                display_field=active_display_field
                category_values
                category_labels
                category_colors
                category_values_error
            />
        </ComponentEditorFieldset>
    }
}

#[component]
fn CategoryLabelsControl(
    component_type: RwSignal<String>,
    fields: Signal<Vec<DatasetFieldDefinition>>,
    display_field: RwSignal<String>,
    category_values: RwSignal<Vec<String>>,
    category_labels: RwSignal<String>,
    category_colors: RwSignal<String>,
    category_values_error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <div class="form-field form-field--wide component-category-labels">
            {move || if component_type.get() == "bar" {
                view! { <FieldHelpLabel label="Series Labels" help="Set optional display labels and colors for each Series Field value."/> }.into_any()
            } else {
                view! { <FieldHelpLabel label="Category Labels" help="Set optional display labels and colors for each Category Field value."/> }.into_any()
            }}
            {move || {
                if let Some(message) = category_values_error.get() {
                    view! { <p class="muted component-category-labels__message">{message}</p> }.into_any()
                } else {
                    let values = category_values.get();
                    if values.is_empty() {
                        let message = if component_type.get() == "bar" && display_field.get().trim().is_empty() {
                            "Select a Comparison Field to customize comparison labels and colors."
                        } else {
                            "Select a Category Field to load its values."
                        };
                        view! { <p class="muted component-category-labels__message">{message}</p> }.into_any()
                    } else {
                        let selected_field_label = field_label_for_key(&fields.get(), &display_field.get())
                            .unwrap_or_else(|| "Category".into());
                        let table_label = if component_type.get() == "bar" {
                            "Series Labels"
                        } else {
                            "Category Labels"
                        };
                        view! {
                            <table class="component-category-labels__table" aria-label=table_label>
                                <thead>
                                    <tr>
                                        <th scope="col">{format!("Original {selected_field_label}")}</th>
                                        <th scope="col">"Display Label"</th>
                                        <th scope="col">"Color"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {values.into_iter().map(|raw_value| {
                                        let raw_for_display = raw_value.clone();
                                        let raw_for_label_input = raw_value.clone();
                                        let raw_for_label_value = raw_value.clone();
                                        let raw_for_label_update = raw_value.clone();
                                        let raw_for_color_input = raw_value.clone();
                                        let raw_for_color_options = raw_value.clone();
                                        let raw_for_color_preview = raw_value.clone();
                                        let raw_for_color_value = raw_value.clone();
                                        let raw_for_color_update = raw_value.clone();
                                        let color_menu_open = RwSignal::new(false);
                                        view! {
                                            <tr>
                                                <th scope="row">{raw_for_display}</th>
                                                <td>
                                                    <input
                                                        aria-label=format!("Display label for {raw_for_label_input}")
                                                        placeholder=raw_for_label_input.clone()
                                                        prop:value=move || category_label_display(&category_labels.get(), &raw_for_label_value)
                                                        on:input=move |event| {
                                                            category_labels.set(update_category_label(
                                                                &category_labels.get_untracked(),
                                                                &raw_for_label_update,
                                                                event_target_value(&event),
                                                            ));
                                                        }
                                                    />
                                                </td>
                                                <td>
                                                    <div class:component-category-labels__color-picker=true class:is-open=move || color_menu_open.get()>
                                                        <button
                                                            type="button"
                                                            class="component-category-labels__color-button"
                                                            aria-label=format!("Color for {raw_for_color_input}")
                                                            on:click=move |_| color_menu_open.update(|open| *open = !*open)
                                                        >
                                                            <span
                                                                class="component-category-labels__color-preview"
                                                                style=move || format!(
                                                                    "--category-swatch-color: {}",
                                                                    category_swatch_style(&category_color_value(&category_colors.get(), &raw_for_color_preview)),
                                                                )
                                                            ></span>
                                                        </button>
                                                        <div class="component-category-labels__color-menu" role="radiogroup" aria-label=format!("Color options for {raw_for_color_options}")>
                                                            {category_color_options().into_iter().map(|option| {
                                                                let checked_value = option.value.to_string();
                                                                let update_value = option.value.to_string();
                                                                let swatch_label = option.label;
                                                                let raw_for_option_value = raw_for_color_value.clone();
                                                                let raw_for_option_update = raw_for_color_update.clone();
                                                                view! {
                                                                    <label
                                                                        class="component-category-labels__swatch"
                                                                        title=swatch_label
                                                                        style=format!("--category-swatch-color: {}", category_swatch_style(option.value))
                                                                    >
                                                                        <input
                                                                            type="radio"
                                                                            name=format!("category-color-{raw_for_option_value}")
                                                                            value=option.value
                                                                            prop:checked=move || category_color_value(&category_colors.get(), &raw_for_option_value) == checked_value
                                                                            on:change=move |_| {
                                                                                category_colors.set(update_category_color(
                                                                                    &category_colors.get_untracked(),
                                                                                    &raw_for_option_update,
                                                                                    update_value.clone(),
                                                                                ));
                                                                                color_menu_open.set(false);
                                                                            }
                                                                        />
                                                                        <span>{swatch_label}</span>
                                                                    </label>
                                                                }
                                                            }).collect_view()}
                                                        </div>
                                                    </div>
                                                </td>
                                            </tr>
                                        }
                                    }).collect_view()}
                                </tbody>
                            </table>
                        }.into_any()
                    }
                }
            }}
        </div>
    }
}

fn field_option_selected(
    field: DatasetFieldDefinition,
    selected: RwSignal<String>,
) -> impl IntoView {
    let key = field.key;
    let selected_key = key.clone();
    let label = format!("{} ({})", field.label, field.field_type);
    view! {
        <option value=key prop:selected=move || selected.get() == selected_key>{label}</option>
    }
}

pub(super) fn field_label_for_key(fields: &[DatasetFieldDefinition], key: &str) -> Option<String> {
    fields
        .iter()
        .find(|field| field.key == key)
        .map(|field| field.label.trim().to_string())
        .filter(|label| !label.is_empty())
}

fn component_kind_description(kind: &str) -> &'static str {
    match kind {
        "table" => {
            "A table lists individual rows from the Dataset. Use it when people need to scan, sort, or inspect record-level values."
        }
        "bar" => {
            "A bar chart compares calculated values across categories. Each bar is grouped by the Category field and its length is calculated from the Value field."
        }
        "line" => {
            "A line chart shows summarized values over an ordered X Field, such as dates. It is best for trends over time."
        }
        "pie" => {
            "A pie chart shows how a whole total is broken down into parts. Each slice is grouped by the Category field, and its size comes from the selected Calculation and Value field."
        }
        "donut" => {
            "A donut chart is a pie chart with a hole in the center. It shows how a whole total is broken down into parts. Each part is a colored arc, grouped by the Category field. The arc shows its percentage of the total from the selected Calculation and Value field."
        }
        "stat_card" => {
            "A Stat Card shows one calculated value, such as a total, average, or median. Use it for headline metrics."
        }
        _ => "",
    }
}

fn visual_sort_field_options(
    kind: &str,
    comparison_field: &str,
) -> Vec<(&'static str, &'static str)> {
    let mut options = vec![("", "Default")];
    match kind {
        "bar" => {
            options.push(("category", "Category"));
            if !comparison_field.trim().is_empty() {
                options.push(("comparison", "Comparison"));
            }
            options.push(("summary_value", "Summary Value"));
        }
        "line" => {
            options.push(("x", "Category"));
            options.push(("summary_value", "Summary Value"));
        }
        "pie" | "donut" => {
            options.push(("category", "Category"));
            options.push(("summary_value", "Summary Value"));
        }
        _ => {}
    }
    options
}

fn visual_sort_field_help(kind: &str, comparison_field: &str) -> String {
    visual_sort_field_options(kind, comparison_field)
        .into_iter()
        .map(|(value, label)| match value {
            "" => format!(
                "{label}: uses the order produced by the current grouping and summarization."
            ),
            "category" => format!("{label}: sorts by the displayed category label."),
            "x" => format!("{label}: sorts by the displayed horizontal category value."),
            "comparison" => format!("{label}: sorts by the displayed comparison group label."),
            "summary_value" => format!("{label}: sorts by the summarized numeric value."),
            _ => format!("{label}: sorts by this field."),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn category_label_display(labels: &str, raw: &str) -> String {
    category_labels_map(labels).remove(raw).unwrap_or_default()
}

fn update_category_label(labels: &str, raw: &str, display: String) -> String {
    let mut labels = category_labels_map(labels);
    let display = display.trim();
    if display.is_empty() {
        labels.remove(raw);
    } else {
        labels.insert(raw.to_string(), display.to_string());
    }
    category_labels_from_map(&labels)
}

pub(super) fn category_labels_map(value: &str) -> BTreeMap<String, String> {
    value
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                return None;
            }
            let (raw, display) = trimmed
                .split_once('=')
                .or_else(|| trimmed.split_once(':'))?;
            let raw = raw.trim();
            let display = display.trim();
            if raw.is_empty() || display.is_empty() {
                return None;
            }
            Some((raw.to_string(), display.to_string()))
        })
        .collect()
}

fn category_labels_from_map(labels: &BTreeMap<String, String>) -> String {
    labels
        .iter()
        .map(|(raw, display)| format!("{raw} = {display}"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[derive(Clone, Copy)]
struct CategoryColorOption {
    value: &'static str,
    label: &'static str,
}

fn category_color_options() -> Vec<CategoryColorOption> {
    vec![
        CategoryColorOption {
            value: "",
            label: "Default",
        },
        CategoryColorOption {
            value: "var(--semantic-primary)",
            label: "Primary",
        },
        CategoryColorOption {
            value: "var(--semantic-success)",
            label: "Success",
        },
        CategoryColorOption {
            value: "var(--semantic-info)",
            label: "Info",
        },
        CategoryColorOption {
            value: "var(--semantic-warning)",
            label: "Warning",
        },
        CategoryColorOption {
            value: "var(--semantic-danger)",
            label: "Danger",
        },
        CategoryColorOption {
            value: "var(--color-cyan)",
            label: "Cyan",
        },
        CategoryColorOption {
            value: "var(--semantic-secondary)",
            label: "Secondary",
        },
    ]
}

fn category_swatch_style(value: &str) -> &'static str {
    match value {
        "" => "transparent",
        "var(--semantic-primary)" => "var(--semantic-primary)",
        "var(--semantic-success)" => "var(--semantic-success)",
        "var(--semantic-info)" => "var(--semantic-info)",
        "var(--semantic-warning)" => "var(--semantic-warning)",
        "var(--semantic-danger)" => "var(--semantic-danger)",
        "var(--color-cyan)" => "var(--color-cyan)",
        "var(--semantic-secondary)" => "var(--semantic-secondary)",
        _ => "var(--semantic-primary)",
    }
}

fn category_color_value(colors: &str, raw: &str) -> String {
    category_colors_map(colors).remove(raw).unwrap_or_default()
}

fn update_category_color(colors: &str, raw: &str, color: String) -> String {
    let mut colors = category_colors_map(colors);
    let color = color.trim();
    if color.is_empty() {
        colors.remove(raw);
    } else {
        colors.insert(raw.to_string(), color.to_string());
    }
    category_colors_from_map(&colors)
}

pub(super) fn category_colors_map(value: &str) -> BTreeMap<String, String> {
    category_labels_map(value)
}

fn category_colors_from_map(colors: &BTreeMap<String, String>) -> String {
    category_labels_from_map(colors)
}
