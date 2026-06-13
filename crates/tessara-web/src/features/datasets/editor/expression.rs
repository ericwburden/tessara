//! Dataset editor expression and SQL preview components.

use super::super::types::*;
use super::expression_tree::expression_tree_view;
use super::helpers::expression_label;
use leptos::prelude::*;

#[component]
pub(crate) fn ExpressionPreview(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression: RwSignal<DatasetExpressionDraft>,
) -> impl IntoView {
    view! {
        <div class="dataset-expression-preview">
            <span>"Expression"</span>
            <code>{move || expression_label(&sources.get(), &expression.get())}</code>
        </div>
    }
}

#[component]
pub(crate) fn DatasetExpressionChain(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression: RwSignal<DatasetExpressionDraft>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <div class="dataset-expression-chain" aria-label="Dataset expression">
            <div class="dataset-expression-tree">
                {move || {
                    let items = sources.get();
                    let draft = expression.get();
                    expression_tree_view(
                        items,
                        draft,
                        sources,
                        expression,
                        fields,
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
                    expression.update(|draft| {
                        *draft = DatasetExpressionDraft::Operation {
                            operation: "union".into(),
                            left: Box::new(draft.clone()),
                            right: Box::new(DatasetExpressionDraft::Source(next - 1)),
                        };
                    });
                    designer_selection.set(DatasetDesignerSelection::Source(next - 1));
                    designer_sheet_open.set(true);
                }
            >
                "Add Input"
            </button>
        </div>
    }
}
