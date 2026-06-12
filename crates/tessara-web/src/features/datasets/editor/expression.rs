//! Dataset editor expression and SQL preview components.

use super::super::actions::preview_dataset_sql;
use super::super::types::*;
use super::helpers::{confirm_action, expression_button_class, expression_label, operation_label};
use crate::ui::EmptyState;
use icons::X;
use leptos::prelude::*;
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
#[component]
/// Renders the dataset sql preview panel view.
pub(crate) fn DatasetSqlPreviewPanel(
    dataset_id: Option<String>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    composition_mode: RwSignal<String>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
    sql_preview: RwSignal<Option<String>>,
    sql_preview_error: RwSignal<Option<String>>,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Generated SQL"</h3>
                <div class="dataset-editor-section__actions">
                    <button class="button button--secondary button--compact" type="button" on:click=move |_| expanded.update(|value| *value = !*value)>
                        {move || if expanded.get() { "Hide SQL" } else { "Show SQL" }}
                    </button>
                    <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                        expanded.set(true);
                        preview_dataset_sql(
                            dataset_id.clone(),
                            name.get(),
                            slug.get(),
                            composition_mode.get(),
                            visibility_node_ids.get().into_iter().collect(),
                            sources.get(),
                            fields.get(),
                            join_left_key.get(),
                            join_right_key.get(),
                            sql_preview,
                            sql_preview_error,
                        );
                    }>"Preview SQL"</button>
                </div>
            </div>
            <Show when=move || expanded.get()>
                {move || sql_preview_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                {move || if let Some(sql) = sql_preview.get() {
                    view! { <pre class="dataset-sql-panel"><code>{sql}</code></pre> }.into_any()
                } else {
                    view! { <EmptyState title="SQL preview unavailable" message="Preview SQL to compile the current dataset definition without saving."/> }.into_any()
                }}
            </Show>
        </section>
    }
}

#[component]
/// Renders the expression preview view.
pub(crate) fn ExpressionPreview(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    composition_mode: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="dataset-expression-preview">
            <span>"Expression"</span>
            <code>{move || expression_label(&sources.get(), &composition_mode.get())}</code>
        </div>
    }
}

#[component]
/// Renders the dataset expression chain view.
pub(crate) fn DatasetExpressionChain(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    composition_mode: RwSignal<String>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="dataset-expression-chain" aria-label="Dataset expression">
            <div class="dataset-expression-tree">
                {move || {
                    let items = sources.get();
                    expression_tree_view(
                        items,
                        sources,
                        fields,
                        composition_mode,
                        designer_selection,
                        designer_sheet_open,
                    )
                }}
            </div>
            <button
                class="button button--secondary button--compact dataset-expression-chain-add"
                type="button"
                on:click=move |_| {
                    let next = sources.get().len() + 1;
                    sources.update(|items| items.push(DatasetSourceDraft {
                        source_alias: format!("source_{next}"),
                        ..DatasetSourceDraft::default()
                    }));
                    designer_selection.set(DatasetDesignerSelection::Source(next - 1));
                    designer_sheet_open.set(true);
                }
            >
                "Add Input"
            </button>
        </div>
    }
}

/// Handles the expression tree view behavior.
fn expression_tree_view(
    items: Vec<DatasetSourceDraft>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    composition_mode: RwSignal<String>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> AnyView {
    if items.is_empty() {
        return view! { <p class="muted">"Add an input to start the dataset expression."</p> }
            .into_any();
    }

    expression_tree_range(
        &items,
        0,
        items.len(),
        0,
        sources,
        fields,
        composition_mode,
        designer_selection,
        designer_sheet_open,
    )
}
/// Handles the expression tree range behavior.
fn expression_tree_range(
    items: &[DatasetSourceDraft],
    start: usize,
    end: usize,
    depth: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    composition_mode: RwSignal<String>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> AnyView {
    if end.saturating_sub(start) <= 1 {
        return expression_source_panel(
            start,
            items[start].source_alias.clone(),
            sources,
            fields,
            designer_selection,
            designer_sheet_open,
        );
    }

    let split = end - 1;
    let layout_class = if depth.is_multiple_of(2) {
        "dataset-expression-group dataset-expression-group--row"
    } else {
        "dataset-expression-group dataset-expression-group--column"
    };
    let left = expression_tree_range(
        items,
        start,
        split,
        depth + 1,
        sources,
        fields,
        composition_mode,
        designer_selection,
        designer_sheet_open,
    );
    let right = expression_tree_range(
        items,
        split,
        end,
        depth + 1,
        sources,
        fields,
        composition_mode,
        designer_selection,
        designer_sheet_open,
    );

    view! {
        <div class=layout_class>
            {left}
            <button
                class=move || expression_button_class(
                    designer_selection.get() == DatasetDesignerSelection::Operation,
                    "dataset-expression-button dataset-expression-button--operation",
                )
                type="button"
                on:click=move |_| {
                    designer_selection.set(DatasetDesignerSelection::Operation);
                    designer_sheet_open.set(true);
                }
            >
                {operation_label(&composition_mode.get())}
            </button>
            {right}
        </div>
    }
    .into_any()
}

/// Handles the expression source panel behavior.
fn expression_source_panel(
    index: usize,
    source_label: String,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
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
                        designer_selection.set(DatasetDesignerSelection::Operation);
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
                    sources.update(|items| {
                        let next = items.len() + 1;
                        let insert_at = (index + 1).min(items.len());
                        items.insert(insert_at, DatasetSourceDraft {
                            source_alias: format!("source_{next}"),
                            ..DatasetSourceDraft::default()
                        });
                    });
                    designer_selection.set(DatasetDesignerSelection::Source(index + 1));
                    designer_sheet_open.set(true);
                }
            >
                "Convert To Expression"
            </button>
        </div>
    }.into_any()
}
