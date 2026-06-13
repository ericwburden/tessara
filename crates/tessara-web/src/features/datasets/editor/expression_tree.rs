//! Recursive expression-tree rendering for the dataset editor.

use super::super::types::*;
use super::helpers::{confirm_action, expression_button_class, operation_label};
use icons::{RefreshCcw, X};
use leptos::prelude::*;

pub(super) fn expression_tree_view(
    items: Vec<DatasetSourceDraft>,
    expression: DatasetExpressionDraft,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression_signal: RwSignal<DatasetExpressionDraft>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> AnyView {
    if items.is_empty() {
        return view! { <p class="muted">"Add an input to start the dataset expression."</p> }
            .into_any();
    }

    expression_tree_node(
        &items,
        &expression,
        Vec::new(),
        0,
        sources,
        expression_signal,
        fields,
        designer_selection,
        designer_sheet_open,
    )
}

fn expression_tree_node(
    items: &[DatasetSourceDraft],
    expression: &DatasetExpressionDraft,
    operation_path: Vec<bool>,
    depth: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression_signal: RwSignal<DatasetExpressionDraft>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> AnyView {
    let DatasetExpressionDraft::Operation {
        operation,
        left,
        right,
    } = expression
    else {
        let index = match expression {
            DatasetExpressionDraft::Source(index) => *index,
            DatasetExpressionDraft::Operation { .. } => unreachable!(),
        };
        let source_label = items
            .get(index)
            .map(|source| source.source_alias.clone())
            .unwrap_or_else(|| "missing_source".into());
        return expression_source_panel(
            index,
            source_label,
            sources,
            expression_signal,
            fields,
            designer_selection,
            designer_sheet_open,
        );
    };

    let layout_class = if depth.is_multiple_of(2) {
        "dataset-expression-group dataset-expression-group--row"
    } else {
        "dataset-expression-group dataset-expression-group--column"
    };
    let mut left_path = operation_path.clone();
    left_path.push(false);
    let mut right_path = operation_path.clone();
    right_path.push(true);
    let left = expression_tree_node(
        items,
        left,
        left_path,
        depth + 1,
        sources,
        expression_signal,
        fields,
        designer_selection,
        designer_sheet_open,
    );
    let right = expression_tree_node(
        items,
        right,
        right_path,
        depth + 1,
        sources,
        expression_signal,
        fields,
        designer_selection,
        designer_sheet_open,
    );
    let button_path = operation_path.clone();
    let reverse_path = operation_path.clone();
    let selected_path = operation_path.clone();
    let label = operation_label(operation);

    view! {
        <div class=layout_class>
            {left}
            <span class="dataset-expression-operation-stack">
                <button
                    class=move || expression_button_class(
                        designer_selection.get() == DatasetDesignerSelection::Operation(selected_path.clone()),
                        "dataset-expression-button dataset-expression-button--operation",
                    )
                    type="button"
                    on:click=move |_| {
                        designer_selection.set(DatasetDesignerSelection::Operation(button_path.clone()));
                        designer_sheet_open.set(true);
                    }
                >
                    {label}
                </button>
                <button
                    class="icon-button dataset-expression-reverse"
                    type="button"
                    aria-label="Reverse expression sides"
                    title="Reverse expression sides"
                    on:click=move |_| {
                        expression_signal.update(|draft| {
                            let _ = reverse_expression_at_path(draft, &reverse_path);
                        });
                    }
                >
                    <RefreshCcw class="icon-button__icon"/>
                </button>
            </span>
            {right}
        </div>
    }
    .into_any()
}

fn expression_source_panel(
    index: usize,
    source_label: String,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression: RwSignal<DatasetExpressionDraft>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> AnyView {
    let remove_label = source_label.clone();
    view! {
        <div class="dataset-expression-panel">
            <button
                class="icon-button icon-button--danger dataset-expression-remove"
                type="button"
                aria-label=format!("Remove input {}", remove_label)
                title="Remove input"
                on:click=move |_| {
                    if confirm_action("Remove this dataset input and its projected fields?") {
                        let removed_alias = sources.get().get(index).map(|source| source.source_alias.clone());
                        sources.update(|items| {
                            if index < items.len() {
                                items.remove(index);
                            }
                            if items.is_empty() {
                                items.push(DatasetSourceDraft::default());
                            }
                        });
                        if let Some(alias) = removed_alias {
                            fields.update(|items| items.retain(|field| field.source_alias != alias));
                        }
                        expression.update(|draft| {
                            *draft = remove_source_from_expression(draft, index)
                                .unwrap_or_else(DatasetExpressionDraft::default);
                        });
                        designer_selection.set(DatasetDesignerSelection::Operation(Vec::new()));
                        designer_sheet_open.set(false);
                    }
                }
            >
                <X class="icon-button__icon"/>
            </button>
            <button
                class=move || expression_button_class(
                    designer_selection.get() == DatasetDesignerSelection::Source(index),
                    "dataset-expression-button dataset-expression-button--source",
                )
                type="button"
                on:click=move |_| {
                    designer_selection.set(DatasetDesignerSelection::Source(index));
                    designer_sheet_open.set(true);
                }
            >
                {source_label.clone()}
            </button>
            <button
                class="button button--secondary button--compact dataset-expression-nest-button"
                type="button"
                on:click=move |_| {
                    let new_index = sources.get().len();
                    sources.update(|items| {
                        items.push(DatasetSourceDraft {
                            source_alias: format!("source_{}", new_index + 1),
                            ..DatasetSourceDraft::default()
                        });
                    });
                    expression.update(|draft| {
                        replace_source_with_expression(draft, index, new_index);
                    });
                    designer_selection.set(DatasetDesignerSelection::Source(new_index));
                    designer_sheet_open.set(true);
                }
            >
                "Convert To Expression"
            </button>
        </div>
    }.into_any()
}

fn replace_source_with_expression(
    expression: &mut DatasetExpressionDraft,
    source_index: usize,
    new_source_index: usize,
) -> bool {
    match expression {
        DatasetExpressionDraft::Source(index) if *index == source_index => {
            *expression = DatasetExpressionDraft::Operation {
                operation: "union".into(),
                left: Box::new(DatasetExpressionDraft::Source(source_index)),
                right: Box::new(DatasetExpressionDraft::Source(new_source_index)),
            };
            true
        }
        DatasetExpressionDraft::Source(_) => false,
        DatasetExpressionDraft::Operation { left, right, .. } => {
            replace_source_with_expression(left, source_index, new_source_index)
                || replace_source_with_expression(right, source_index, new_source_index)
        }
    }
}

fn reverse_expression_at_path(expression: &mut DatasetExpressionDraft, path: &[bool]) -> bool {
    match (expression, path.split_first()) {
        (DatasetExpressionDraft::Operation { left, right, .. }, None) => {
            std::mem::swap(left, right);
            true
        }
        (DatasetExpressionDraft::Operation { left, .. }, Some((false, rest))) => {
            reverse_expression_at_path(left, rest)
        }
        (DatasetExpressionDraft::Operation { right, .. }, Some((true, rest))) => {
            reverse_expression_at_path(right, rest)
        }
        _ => false,
    }
}

fn remove_source_from_expression(
    expression: &DatasetExpressionDraft,
    removed_index: usize,
) -> Option<DatasetExpressionDraft> {
    match expression {
        DatasetExpressionDraft::Source(index) if *index == removed_index => None,
        DatasetExpressionDraft::Source(index) if *index > removed_index => {
            Some(DatasetExpressionDraft::Source(index - 1))
        }
        DatasetExpressionDraft::Source(index) => Some(DatasetExpressionDraft::Source(*index)),
        DatasetExpressionDraft::Operation {
            operation,
            left,
            right,
        } => {
            let left = remove_source_from_expression(left, removed_index);
            let right = remove_source_from_expression(right, removed_index);
            match (left, right) {
                (Some(left), Some(right)) => Some(DatasetExpressionDraft::Operation {
                    operation: operation.clone(),
                    left: Box::new(left),
                    right: Box::new(right),
                }),
                (Some(remaining), None) | (None, Some(remaining)) => Some(remaining),
                (None, None) => None,
            }
        }
    }
}
