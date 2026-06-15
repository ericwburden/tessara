//! Dataset detail surface and preview table.

use super::super::super::display::visibility_label;
use super::super::super::loaders::{load_account, load_dataset_detail, load_dataset_table};
use super::super::super::permissions::can_manage_datasets;
use super::super::super::types::*;
use super::summary::{MetricCard, tab_class};
use super::tables::{DatasetFieldsTable, DatasetSourcesTable, DatasetSqlPanel};
use crate::ui::{AppShell, DataTable, EmptyState, PageHeader};
use crate::utils::text::sentence_label;
use icons::X;
use leptos::portal::Portal;
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetDetailSurface(dataset_id: String, edit: bool) -> impl IntoView {
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let table = RwSignal::new(None::<DatasetTable>);
    let account = RwSignal::new(None::<SessionAccount>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let table_error = RwSignal::new(None::<String>);
    let active_tab = RwSignal::new("preview".to_string());
    let visibility_sheet_open = RwSignal::new(false);

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
                        let tab_dataset = loaded.clone();
                        let visibility_nodes = loaded.visibility_nodes.clone();
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
                                <button class="metric-card metric-card--button" type="button" aria-label="Show dataset visibility nodes" on:click=move |_| visibility_sheet_open.set(true)>
                                    <span>"Visibility"</span>
                                    <strong>{visibility_label(&loaded.visibility_nodes)}</strong>
                                </button>
                            </section>
                            <div class="tabs" data-active=move || active_tab.get()>
                                <div class="tabs-list" role="tablist">
                                    <button class=tab_class(active_tab, "preview") type="button" on:click=move |_| active_tab.set("preview".into())>"Preview"</button>
                                    <button class=tab_class(active_tab, "sources") type="button" on:click=move |_| active_tab.set("sources".into())>"Sources"</button>
                                    <button class=tab_class(active_tab, "fields") type="button" on:click=move |_| active_tab.set("fields".into())>"Fields"</button>
                                    <button class=tab_class(active_tab, "sql") type="button" on:click=move |_| active_tab.set("sql".into())>"SQL"</button>
                                </div>
                                {move || if active_tab.get() == "preview" {
                                    view! { <DatasetPreviewTable dataset=tab_dataset.clone() table=table.get() error=table_error.get()/> }.into_any()
                                } else if active_tab.get() == "sources" {
                                    view! { <DatasetSourcesTable sources=tab_dataset.sources.clone()/> }.into_any()
                                } else if active_tab.get() == "sql" {
                                    view! { <DatasetSqlPanel sql=tab_dataset.generated_sql.clone()/> }.into_any()
                                } else {
                                    view! { <DatasetFieldsTable fields=detail_output_fields(&tab_dataset) /> }.into_any()
                                }}
                            </div>
                            <DatasetVisibilitySheet nodes=visibility_nodes open=visibility_sheet_open/>
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
fn DatasetVisibilitySheet(
    nodes: Vec<DatasetVisibilityNode>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let close = move |_| open.set(false);
    let nodes = RwSignal::new(nodes);

    view! {
        <Portal>
            <Show when=move || open.get()>
                <section class="sheet-overlay dataset-visibility-overlay" aria-label="Dataset visibility nodes overlay">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close dataset visibility nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface dataset-visibility-sheet" role="dialog" aria-modal="true" aria-label="Dataset visibility nodes">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close dataset visibility nodes" title="Close dataset visibility nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        <header class="sheet-panel__header">
                            <p>"Dataset Visibility"</p>
                            <h2>{move || visibility_label(&nodes.get())}</h2>
                        </header>
                        <section class="sheet-panel__section">
                            <h3>"Visible Nodes"</h3>
                            {move || if nodes.get().is_empty() {
                                view! { <p class="muted">"No visibility nodes are selected for this dataset."</p> }.into_any()
                            } else {
                                view! {
                                    <div class="dataset-visibility-sheet__list">
                                        {move || nodes.get().into_iter().map(|node| {
                                            view! {
                                                <article class="dataset-visibility-sheet__node">
                                                    <strong>{node.node_name}</strong>
                                                    <span>{format!("{} · {}", sentence_label(&node.node_type_name), node.node_path)}</span>
                                                </article>
                                            }
                                        }).collect_view()}
                                    </div>
                                }.into_any()
                            }}
                        </section>
                    </aside>
                </section>
            </Show>
        </Portal>
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
    let fields = detail_output_fields(&dataset);
    view! {
        <section class="route-panel__section">
            <h3>"Preview"</h3>
            <DataTable>
                <thead>
                    <tr>
                        {fields.iter().map(|field| view! { <th>{field.label.clone()}</th> }).collect_view()}
                    </tr>
                </thead>
                <tbody>
                    {table.rows.into_iter().map(|row| {
                        let values = row.values.clone();
                        view! {
                            <tr>
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

fn detail_output_fields(dataset: &DatasetDefinition) -> Vec<DatasetFieldDefinition> {
    if dataset.output_fields.is_empty() {
        dataset.fields.clone()
    } else {
        dataset.output_fields.clone()
    }
}
