//! Dataset editor operation option panel.

use super::super::expressions::is_join_operation;
use super::super::types::*;
use super::helpers::{join_key_option_label, operation_label};
use super::source_field_actions::canonical_field_key;
use super::source_options::source_field_options;
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[component]
pub(crate) fn OperationOptionsPanel(
    path: Vec<bool>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    expression: RwSignal<DatasetExpressionDraft>,
    composition_mode: RwSignal<String>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
) -> impl IntoView {
    let header_path = path.clone();
    let select_path = path.clone();
    let join_path = path.clone();
    view! {
        <div class="dataset-options-sheet__content">
            <header class="dataset-options-sheet__header">
                <span>"Operation"</span>
                <h4>{move || operation_label(&operation_at_path(&expression.get(), &header_path).unwrap_or_default())}</h4>
            </header>
            <label class="form-field">
                <span>"Operation"</span>
                <select prop:value=move || operation_at_path(&expression.get(), &select_path).unwrap_or_default() on:change=move |event| {
                    let value = event_target_value(&event);
                    expression.update(|draft| {
                        let _ = set_operation_at_path(draft, &path, &value);
                    });
                    if path.is_empty() {
                        composition_mode.set(value);
                    }
                }>
                    <option value="union">"Union"</option>
                    <option value="union_all">"Union All"</option>
                    <option value="left_join">"Left Join"</option>
                    <option value="inner_join">"Inner Join"</option>
                    <option value="outer_join">"Outer Join"</option>
                </select>
            </label>
            {move || if operation_at_path(&expression.get(), &join_path).is_some_and(|operation| is_join_operation(&operation)) {
                let expression_value = expression.get();
                let left_options = join_key_options_for_expression_side(
                    &expression_value,
                    &sources.get(),
                    &forms.get(),
                    &rendered_forms.get(),
                    &join_path,
                    false,
                    &join_left_key.get(),
                );
                let right_options = join_key_options_for_expression_side(
                    &expression_value,
                    &sources.get(),
                    &forms.get(),
                    &rendered_forms.get(),
                    &join_path,
                    true,
                    &join_right_key.get(),
                );
                view! {
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Left Join Key"</span>
                            <select prop:value=move || join_left_key.get() on:change=move |event| join_left_key.set(event_target_value(&event))>
                                <option value="">"Select field"</option>
                                {left_options.into_iter().map(|option| {
                                    view! { <option value=option.key.clone()>{join_key_option_label(&option)}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Right Join Key"</span>
                            <select prop:value=move || join_right_key.get() on:change=move |event| join_right_key.set(event_target_value(&event))>
                                <option value="">"Select field"</option>
                                {right_options.into_iter().map(|option| {
                                    view! { <option value=option.key.clone()>{join_key_option_label(&option)}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

fn operation_at_path(expression: &DatasetExpressionDraft, path: &[bool]) -> Option<String> {
    match (expression, path.split_first()) {
        (DatasetExpressionDraft::Operation { operation, .. }, None) => Some(operation.clone()),
        (DatasetExpressionDraft::Operation { left, .. }, Some((false, rest))) => {
            operation_at_path(left, rest)
        }
        (DatasetExpressionDraft::Operation { right, .. }, Some((true, rest))) => {
            operation_at_path(right, rest)
        }
        _ => None,
    }
}

fn expression_at_path<'a>(
    expression: &'a DatasetExpressionDraft,
    path: &[bool],
) -> Option<&'a DatasetExpressionDraft> {
    match (expression, path.split_first()) {
        (_, None) => Some(expression),
        (DatasetExpressionDraft::Operation { left, .. }, Some((false, rest))) => {
            expression_at_path(left, rest)
        }
        (DatasetExpressionDraft::Operation { right, .. }, Some((true, rest))) => {
            expression_at_path(right, rest)
        }
        _ => None,
    }
}

fn join_key_options_for_expression_side(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    operation_path: &[bool],
    right_side: bool,
    selected_key: &str,
) -> Vec<DatasetRenderedField> {
    let Some(DatasetExpressionDraft::Operation { left, right, .. }) =
        expression_at_path(expression, operation_path)
    else {
        return Vec::new();
    };
    let side = if right_side { right } else { left };
    let mut aliases = BTreeSet::new();
    collect_source_aliases(side, sources, &mut aliases);
    let mut options_by_key = BTreeMap::new();
    for alias in aliases {
        for option in source_field_options(sources, forms, rendered_forms, &alias) {
            let key = canonical_field_key(&alias, &option.key);
            options_by_key
                .entry(key.clone())
                .or_insert(DatasetRenderedField {
                    key,
                    label: option.label,
                    field_type: option.field_type,
                });
        }
    }
    let mut options = options_by_key.into_values().collect::<Vec<_>>();

    if !selected_key.is_empty() && !options.iter().any(|option| option.key == selected_key) {
        options.push(DatasetRenderedField {
            key: selected_key.to_string(),
            label: "Unknown field".into(),
            field_type: String::new(),
        });
    }

    options
}

fn collect_source_aliases(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
    aliases: &mut BTreeSet<String>,
) {
    match expression {
        DatasetExpressionDraft::Source(index) => {
            if let Some(source) = sources.get(*index) {
                aliases.insert(source.source_alias.clone());
            }
        }
        DatasetExpressionDraft::Operation { left, right, .. } => {
            collect_source_aliases(left, sources, aliases);
            collect_source_aliases(right, sources, aliases);
        }
    }
}

fn set_operation_at_path(
    expression: &mut DatasetExpressionDraft,
    path: &[bool],
    operation: &str,
) -> bool {
    match (expression, path.split_first()) {
        (
            DatasetExpressionDraft::Operation {
                operation: target, ..
            },
            None,
        ) => {
            *target = operation.into();
            true
        }
        (DatasetExpressionDraft::Operation { left, .. }, Some((false, rest))) => {
            set_operation_at_path(left, rest, operation)
        }
        (DatasetExpressionDraft::Operation { right, .. }, Some((true, rest))) => {
            set_operation_at_path(right, rest, operation)
        }
        _ => false,
    }
}
