//! Dataset detail source, field, and SQL panels.

use super::super::super::types::{DatasetFieldDefinition, DatasetSourceDefinition};
use crate::ui::{DataTable, EmptyState};
use crate::utils::text::sentence_label;
use leptos::prelude::*;

#[component]
pub(super) fn DatasetSourcesTable(sources: Vec<DatasetSourceDefinition>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Sources"</h3>
            <DataTable>
                <thead><tr><th>"Alias"</th><th>"Form"</th><th>"Major"</th></tr></thead>
                <tbody>
                    {sources.into_iter().map(|source| view! {
                        <tr>
                            <th scope="row">{source.source_alias}</th>
                            <td>{source.form_name.unwrap_or_else(|| "Unavailable form".into())}</td>
                            <td>{source.form_version_major.map(|value| value.to_string()).unwrap_or_else(|| "Current".into())}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
    }
}

#[component]
pub(super) fn DatasetFieldsTable(fields: Vec<DatasetFieldDefinition>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Fields"</h3>
            <DataTable>
                <thead><tr><th>"Field"</th><th>"Source"</th><th>"Source Field"</th><th>"Type"</th></tr></thead>
                <tbody>
                    {fields.into_iter().map(|field| view! {
                        <tr>
                            <th scope="row" class="data-table__stacked-label">
                                <span>{field.label}</span>
                                <span class="data-table__secondary-text">{field.key}</span>
                            </th>
                            <td>{field.source_alias}</td>
                            <td>{field.source_field_key}</td>
                            <td>{sentence_label(&field.field_type)}</td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
    }
}

#[component]
pub(super) fn DatasetSqlPanel(sql: Option<String>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Generated SQL"</h3>
            {if let Some(sql) = sql {
                view! { <pre class="dataset-sql-panel"><code>{sql}</code></pre> }.into_any()
            } else {
                view! { <EmptyState title="SQL unavailable" message="This dataset revision does not have generated SQL metadata."/> }.into_any()
            }}
        </section>
    }
}
