//! Dataset detail source, field, and SQL panels.

use super::super::super::types::{DatasetFieldDefinition, DatasetSourceDefinition};
use crate::text::sentence_label;
use leptos::prelude::*;
use tessara_module_ui::{DataTable, EmptyState};

#[component]
pub(super) fn DatasetSourcesTable(sources: Vec<DatasetSourceDefinition>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Sources"</h3>
            <DataTable>
                <thead><tr><th>"Alias"</th><th>"Source"</th><th>"Source Type"</th><th>"Version"</th></tr></thead>
                <tbody>
                    {sources.into_iter().map(source_row).collect_view()}
                </tbody>
            </DataTable>
        </section>
    }
}

fn source_row(source: DatasetSourceDefinition) -> impl IntoView {
    let source_type = if source.form_id.is_some() {
        "Form"
    } else if source.source_dataset_id.is_some() {
        "Dataset"
    } else {
        "Unavailable"
    };
    let source_label = source
        .form_name
        .clone()
        .or_else(|| source.source_dataset_name.clone())
        .unwrap_or_else(|| "Unavailable source".into());
    let source_href = source
        .form_id
        .as_ref()
        .map(|id| format!("/forms/{id}"))
        .or_else(|| {
            source
                .source_dataset_id
                .as_ref()
                .map(|id| format!("/datasets/{id}"))
        });
    let version_label = source_version_label(&source);
    view! {
        <tr>
            <th scope="row">{source.source_alias}</th>
            <td>
                {if let Some(href) = source_href {
                    view! { <a class="data-table__primary-link" href=href>{source_label}</a> }.into_any()
                } else {
                    view! { <span>{source_label}</span> }.into_any()
                }}
            </td>
            <td>{source_type}</td>
            <td>{version_label}</td>
        </tr>
    }
}

fn source_version_label(source: &DatasetSourceDefinition) -> String {
    if let Some(label) = source.form_version_label.as_deref() {
        label.to_string()
    } else if let Some(label) = source.dataset_revision_label.as_deref() {
        label.to_string()
    } else if let Some(major) = source.dataset_version_major {
        format!("v{major}")
    } else {
        "Unknown version".into()
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
