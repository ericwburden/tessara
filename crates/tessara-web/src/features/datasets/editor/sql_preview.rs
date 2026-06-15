//! Dataset editor SQL preview panel.

use super::super::actions::preview_dataset_sql;
use super::super::types::*;
use crate::ui::EmptyState;
use icons::ChevronsUpDown;
use leptos::prelude::*;
use std::collections::BTreeSet;

#[allow(clippy::too_many_arguments)]
#[component]
pub(crate) fn DatasetSqlPreviewPanel(
    dataset_id: Option<String>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression: RwSignal<DatasetExpressionDraft>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
    sql_preview: RwSignal<Option<String>>,
    sql_preview_error: RwSignal<Option<String>>,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    Effect::new(move |_| {
        if expanded.get() {
            preview_dataset_sql(
                dataset_id.clone(),
                name.get(),
                slug.get(),
                visibility_node_ids.get().into_iter().collect(),
                sources.get(),
                expression.get(),
                fields.get(),
                join_left_key.get(),
                join_right_key.get(),
                sql_preview,
                sql_preview_error,
            );
        }
    });

    view! {
        <section class="route-panel__section dataset-editor-section">
            <button
                class="dataset-editor-section__header dataset-sql-header"
                type="button"
                aria-expanded=move || expanded.get().to_string()
                on:click=move |_| expanded.update(|value| *value = !*value)
            >
                <h3>"Generated SQL"</h3>
                <ChevronsUpDown class="dataset-sql-header__icon"/>
            </button>
            <Show when=move || expanded.get()>
                {move || sql_preview_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                {move || if let Some(sql) = sql_preview.get() {
                    view! { <pre class="dataset-sql-panel"><code>{sql}</code></pre> }.into_any()
                } else {
                    view! { <EmptyState title="SQL preview unavailable" message="Open this panel to compile the current dataset definition without saving."/> }.into_any()
                }}
            </Show>
        </section>
    }
}
