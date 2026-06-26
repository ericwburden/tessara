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
    initial_source: RwSignal<DatasetSourceDraft>,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    restriction_internal_field_key: RwSignal<String>,
    restriction_restricted_field_key: RwSignal<String>,
    restriction_confidential_field_key: RwSignal<String>,
    sql_preview: RwSignal<Option<String>>,
    sql_preview_error: RwSignal<Option<String>>,
    expanded: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <button
                class="dataset-editor-section__header dataset-sql-header"
                type="button"
                aria-expanded=move || expanded.get().to_string()
                on:click=move |_| {
                    let next_expanded = !expanded.get_untracked();
                    expanded.set(next_expanded);
                    if next_expanded {
                        preview_dataset_sql(
                            dataset_id.clone(),
                            name.get_untracked(),
                            slug.get_untracked(),
                            visibility_node_ids.get_untracked().into_iter().collect(),
                            initial_source.get_untracked(),
                            operation_order.get_untracked(),
                            restriction_internal_field_key.get_untracked(),
                            restriction_restricted_field_key.get_untracked(),
                            restriction_confidential_field_key.get_untracked(),
                            sql_preview,
                            sql_preview_error,
                        );
                    }
                }
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
