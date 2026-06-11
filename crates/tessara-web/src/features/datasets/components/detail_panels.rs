//! Dataset detail and preview components.

use super::super::display::visibility_label;
use super::super::loaders::{load_account, load_dataset_detail, load_dataset_table};
use super::super::permissions::can_manage_datasets;
use super::super::types::*;
use crate::ui::{AppShell, DataTable, EmptyState, PageHeader, StatusBadge};
use crate::utils::text::sentence_label;
use leptos::prelude::*;
#[component]
/// Renders the dataset detail surface view.
pub(crate) fn DatasetDetailSurface(dataset_id: String, edit: bool) -> impl IntoView {
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let table = RwSignal::new(None::<DatasetTable>);
    let account = RwSignal::new(None::<SessionAccount>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let table_error = RwSignal::new(None::<String>);
    let active_tab = RwSignal::new("preview".to_string());

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_account(account);
            load_dataset_detail(dataset_id.clone(), dataset, is_loading, load_error);
            load_dataset_table(dataset_id.clone(), table, table_error);
        }
    });

    let can_manage = move || {
        account
            .get()
            .is_some_and(|account| can_manage_datasets(&account))
    };

    view! {
        <AppShell active_route="datasets" title="Dataset Detail">
            <section class="route-panel datasets-page">
                {move || {
                    if is_loading.get() {
                        view! { <EmptyState title="Loading dataset" message="Fetching dataset definition."/> }.into_any()
                    } else if let Some(message) = load_error.get() {
                        view! { <EmptyState title="Dataset unavailable" message=Box::leak(message.into_boxed_str())/> }.into_any()
                    } else if let Some(loaded) = dataset.get() {
                        let edit_href = format!("/datasets/{}/edit", loaded.id);
                        view! {
                            <PageHeader title=Box::leak(loaded.name.clone().into_boxed_str())>
                                {if can_manage() && !edit {
                                    view! { <a class="button button--secondary" href=edit_href>"Edit Dataset"</a> }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </PageHeader>
                            <section class="dataset-detail-summary">
                                <MetricCard label="Slug" value=loaded.slug.clone()/>
                                <MetricCard label="Grain" value=sentence_label(&loaded.grain)/>
                                <MetricCard label="Composition" value=sentence_label(&loaded.composition_mode)/>
                                <MetricCard label="Visibility" value=visibility_label(&loaded.visibility_nodes)/>
                            </section>
                            <div class="tabs" data-active=move || active_tab.get()>
                                <div class="tabs-list" role="tablist">
                                    <button class=tab_class(active_tab, "preview") type="button" on:click=move |_| active_tab.set("preview".into())>"Preview"</button>
                                    <button class=tab_class(active_tab, "sources") type="button" on:click=move |_| active_tab.set("sources".into())>"Sources"</button>
                                    <button class=tab_class(active_tab, "fields") type="button" on:click=move |_| active_tab.set("fields".into())>"Fields"</button>
                                    <button class=tab_class(active_tab, "sql") type="button" on:click=move |_| active_tab.set("sql".into())>"SQL"</button>
                                </div>
                                {move || if active_tab.get() == "preview" {
                                    view! { <DatasetPreviewTable dataset=loaded.clone() table=table.get() error=table_error.get()/> }.into_any()
                                } else if active_tab.get() == "sources" {
                                    view! { <DatasetSourcesTable sources=loaded.sources.clone()/> }.into_any()
                                } else if active_tab.get() == "sql" {
                                    view! { <DatasetSqlPanel sql=loaded.generated_sql.clone()/> }.into_any()
                                } else {
                                    view! { <DatasetFieldsTable fields=loaded.fields.clone()/> }.into_any()
                                }}
                            </div>
                        }.into_any()
                    } else {
                        view! { <EmptyState title="Dataset unavailable" message="Dataset data could not be loaded."/> }.into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the metric card view.
fn MetricCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="metric-card">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    }
}

#[component]
/// Renders the dataset sources table view.
fn DatasetSourcesTable(sources: Vec<DatasetSourceDefinition>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Sources"</h3>
            <DataTable>
                <thead><tr><th>"Alias"</th><th>"Form"</th><th>"Major"</th><th>"Selection"</th></tr></thead>
                <tbody>
                    {sources.into_iter().map(|source| view! {
                        <tr>
                            <th scope="row">{source.source_alias}</th>
                            <td>{source.form_name.unwrap_or_else(|| "Unavailable form".into())}</td>
                            <td>{source.form_version_major.map(|value| value.to_string()).unwrap_or_else(|| "Current".into())}</td>
                            <td><StatusBadge label=sentence_label(&source.selection_rule)/></td>
                        </tr>
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
    }
}

#[component]
/// Renders the dataset fields table view.
fn DatasetFieldsTable(fields: Vec<DatasetFieldDefinition>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <h3>"Fields"</h3>
            <DataTable>
                <thead><tr><th>"Field"</th><th>"Source"</th><th>"Source Field"</th><th>"Type"</th></tr></thead>
                <tbody>
                    {fields.into_iter().map(|field| view! {
                        <tr>
                            <th scope="row">{field.label}<span class="data-table__secondary-text">{field.key}</span></th>
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
/// Renders the dataset sql panel view.
fn DatasetSqlPanel(sql: Option<String>) -> impl IntoView {
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

#[component]
/// Renders the dataset preview table view.
pub(crate) fn DatasetPreviewTable(
    dataset: DatasetDefinition,
    table: Option<DatasetTable>,
    error: Option<String>,
) -> impl IntoView {
    if let Some(message) = error {
        return view! { <EmptyState title="Preview unavailable" message=Box::leak(message.into_boxed_str())/> }.into_any();
    }
    let Some(table) = table else {
        return view! { <EmptyState title="Loading preview" message="Fetching dataset preview rows."/> }.into_any();
    };
    if table.rows.is_empty() {
        return view! { <EmptyState title="No preview rows" message="This dataset has no submitted response rows available for preview."/> }.into_any();
    }
    let fields = dataset.fields;
    view! {
        <section class="route-panel__section">
            <h3>"Preview"</h3>
            <DataTable>
                <thead>
                    <tr>
                        <th>"Node"</th>
                        <th>"Source"</th>
                        {fields.iter().map(|field| view! { <th>{field.label.clone()}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {table.rows.into_iter().map(|row| {
                        let values = row.values.clone();
                        view! {
                            <tr>
                                <th scope="row">{row.node_name}<span class="data-table__secondary-text">{row.submission_id}</span></th>
                                <td>{row.source_alias}</td>
                                {fields.iter().map(|field| {
                                    let value = values.get(&field.key).and_then(|value| value.clone()).unwrap_or_default();
                                    view! { <td>{value}</td> }
                                }).collect_view()}
                            </tr>
                        }
                    }).collect_view()}
                </tbody>
            </DataTable>
        </section>
    }.into_any()
}

/// Handles the tab class behavior.
fn tab_class(active_tab: RwSignal<String>, value: &'static str) -> impl Fn() -> &'static str {
    move || {
        if active_tab.get() == value {
            "tabs-trigger is-active"
        } else {
            "tabs-trigger"
        }
    }
}
