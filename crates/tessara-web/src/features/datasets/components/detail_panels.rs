//! Dataset detail and preview components.

mod summary;
mod tables;

use super::super::display::visibility_label;
use super::super::loaders::{load_account, load_dataset_detail, load_dataset_table};
use super::super::permissions::can_manage_datasets;
use super::super::types::*;
use crate::ui::{AppShell, DataTable, EmptyState, PageHeader};
use crate::utils::text::sentence_label;
use leptos::prelude::*;
use summary::{MetricCard, tab_class};
use tables::{DatasetFieldsTable, DatasetSourcesTable, DatasetSqlPanel};

#[component]
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
