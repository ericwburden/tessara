//! Route-level page composition for the Datasets feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use std::collections::{BTreeMap, BTreeSet};

use leptos::portal::Portal;
use leptos::prelude::*;

use crate::types::route_params::{DatasetRouteParams, require_route_params};
use crate::ui::{AppShell, DataTable, EmptyState, PageHeader, StatusBadge};
use crate::utils::{
    pagination::{
        pagination_current_page, pagination_page_count, pagination_page_end, pagination_page_start,
    },
    text::{sentence_label, text_matches},
};
use icons::{Search, X};

use super::loaders::*;
use super::types::*;

#[component]
/// Renders the datasets page view.
pub fn DatasetsPage() -> impl IntoView {
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let account = RwSignal::new(None::<SessionAccount>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let search = RwSignal::new(String::new());
    let page_index = RwSignal::new(0usize);
    let page_size = RwSignal::new(10usize);

    Effect::new(move |_| {
        load_account(account);
        load_datasets(datasets, is_loading, load_error);
    });

    let filtered = Memo::new(move |_| {
        let query = search.get();
        datasets
            .get()
            .into_iter()
            .filter(|dataset| {
                text_matches(
                    &query,
                    &[
                        dataset.name.as_str(),
                        dataset.slug.as_str(),
                        dataset.grain.as_str(),
                        dataset.composition_mode.as_str(),
                    ],
                )
            })
            .collect::<Vec<_>>()
    });
    let can_manage = move || {
        account
            .get()
            .is_some_and(|account| can_manage_datasets(&account))
    };

    view! {
        <AppShell active_route="datasets" title="Datasets">
            <section class="route-panel datasets-page">
                <PageHeader title="Datasets">
                    {move || if can_manage() {
                        view! { <a class="button" href="/datasets/new">"Create Dataset"</a> }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! { <EmptyState title="Loading datasets" message="Fetching visible datasets."/> }.into_any()
                    } else if let Some(message) = load_error.get() {
                        view! { <EmptyState title="Datasets unavailable" message=Box::leak(message.into_boxed_str())/> }.into_any()
                    } else if datasets.get().is_empty() {
                        view! { <EmptyState title="No visible datasets" message="No datasets are visible for the current account."/> }.into_any()
                    } else {
                        view! {
                            <DatasetDirectoryTable
                                datasets=filtered.get()
                                search
                                page_index
                                page_size
                            />
                        }.into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the dataset directory table view.
fn DatasetDirectoryTable(
    datasets: Vec<DatasetSummary>,
    search: RwSignal<String>,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    let total_count = datasets.len();
    let page_count = pagination_page_count(total_count, page_size.get());
    let current_page = pagination_current_page(total_count, page_size.get(), page_index.get());
    let summary = table_summary(total_count, page_size.get(), page_index.get(), "datasets");
    let page_start = pagination_page_start(total_count, page_size.get(), page_index.get());
    let paged_datasets = datasets
        .iter()
        .skip(page_start)
        .take(page_size.get())
        .cloned()
        .collect::<Vec<_>>();

    view! {
        <section class="route-panel__section dataset-table-section">
            <label class="searchable-data-table__search searchable-data-table__control">
                <Search class="searchable-data-table__control-icon"/>
                <span class="sr-only">"Search datasets"</span>
                <input
                    type="search"
                    placeholder="Search datasets"
                    prop:value=move || search.get()
                    on:input=move |event| {
                        search.set(event_target_value(&event));
                        page_index.set(0);
                    }
                />
            </label>
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Dataset"</th>
                        <th scope="col">"Grain"</th>
                        <th scope="col">"Composition"</th>
                        <th scope="col">"Visibility"</th>
                        <th scope="col" class="data-table__cell--center">"Sources"</th>
                        <th scope="col" class="data-table__cell--center">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {paged_datasets
                        .into_iter()
                        .map(|dataset| view! { <DatasetSummaryRow dataset/> })
                        .collect_view()}
                </tbody>
            </DataTable>
            <DatasetMobileCards datasets=datasets.clone() page_index page_size/>
            <TablePagination
                summary=summary
                page_count=page_count
                current_page=current_page
                page_index
                page_size
            />
        </section>
    }
}

#[component]
/// Renders the dataset summary row view.
fn DatasetSummaryRow(dataset: DatasetSummary) -> impl IntoView {
    let href = format!("/datasets/{}", dataset.id);
    view! {
        <tr>
            <th scope="row">
                <a class="data-table__primary-link" href=href>{dataset.name}</a>
                <span class="data-table__secondary-text">{dataset.slug}</span>
            </th>
            <td>{sentence_label(&dataset.grain)}</td>
            <td>{sentence_label(&dataset.composition_mode)}</td>
            <td>{visibility_label(&dataset.visibility_nodes)}</td>
            <td class="data-table__cell--center">{dataset.source_count}</td>
            <td class="data-table__cell--center">{dataset.field_count}</td>
        </tr>
    }
}

#[component]
/// Renders the dataset mobile cards view.
fn DatasetMobileCards(
    datasets: Vec<DatasetSummary>,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    let total_count = datasets.len();
    let page_start = pagination_page_start(total_count, page_size.get(), page_index.get());
    let paged_datasets = datasets
        .into_iter()
        .skip(page_start)
        .take(page_size.get())
        .collect::<Vec<_>>();
    view! {
        <div class="related-work-mobile-cards">
            {paged_datasets
                .into_iter()
                .map(|dataset| {
                    let href = format!("/datasets/{}", dataset.id);
                    view! {
                        <article class="related-work-mobile-card">
                            <h4><a href=href>{dataset.name}</a></h4>
                            <dl>
                                <dt>"Grain"</dt><dd>{sentence_label(&dataset.grain)}</dd>
                                <dt>"Composition"</dt><dd>{sentence_label(&dataset.composition_mode)}</dd>
                                <dt>"Visibility"</dt><dd>{visibility_label(&dataset.visibility_nodes)}</dd>
                                <dt>"Sources"</dt><dd>{dataset.source_count}</dd>
                                <dt>"Fields"</dt><dd>{dataset.field_count}</dd>
                            </dl>
                        </article>
                    }
                })
                .collect_view()}
        </div>
    }
}

#[component]
/// Renders the datasets detail page view.
pub fn DatasetsDetailPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    view! { <DatasetDetailSurface dataset_id edit=false/> }
}

#[component]
/// Renders the datasets edit page view.
pub fn DatasetsEditPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    view! { <DatasetEditorSurface dataset_id=Some(dataset_id)/> }
}

#[component]
/// Renders the datasets new page view.
pub fn DatasetsNewPage() -> impl IntoView {
    view! { <DatasetEditorSurface dataset_id=None/> }
}

#[component]
/// Renders the datasets preview page view.
pub fn DatasetsPreviewPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let table = RwSignal::new(None::<DatasetTable>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let table_error = RwSignal::new(None::<String>);

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_dataset_detail(dataset_id.clone(), dataset, is_loading, load_error);
            load_dataset_table(dataset_id.clone(), table, table_error);
        }
    });

    view! {
        <main class="dataset-preview-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading preview" message="Fetching dataset preview rows."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Preview unavailable" message=Box::leak(message.into_boxed_str())/> }.into_any()
                } else if let Some(loaded) = dataset.get() {
                    view! {
                        <section class="dataset-preview-page__content">
                            <header class="dataset-preview-page__header">
                                <p>"Dataset Preview"</p>
                                <h1>{loaded.name.clone()}</h1>
                            </header>
                            <DatasetPreviewTable dataset=loaded table=table.get() error=table_error.get()/ >
                        </section>
                    }.into_any()
                } else {
                    view! { <EmptyState title="Preview unavailable" message="Dataset details could not be loaded."/> }.into_any()
                }
            }}
        </main>
    }
}

#[component]
/// Renders the dataset detail surface view.
fn DatasetDetailSurface(dataset_id: String, edit: bool) -> impl IntoView {
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
fn DatasetPreviewTable(
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

#[component]
/// Renders the dataset editor surface view.
fn DatasetEditorSurface(dataset_id: Option<String>) -> impl IntoView {
    let is_edit = dataset_id.is_some();
    let title = if is_edit {
        "Edit Dataset"
    } else {
        "Create Dataset"
    };
    let name = RwSignal::new(String::new());
    let slug = RwSignal::new(String::new());
    let composition_mode = RwSignal::new("union".to_string());
    let visibility_node_ids = RwSignal::new(BTreeSet::<String>::new());
    let sources = RwSignal::new(vec![DatasetSourceDraft::default()]);
    let fields = RwSignal::new(Vec::<DatasetFieldDraft>::new());
    let join_left_key = RwSignal::new(String::new());
    let join_right_key = RwSignal::new(String::new());
    let forms = RwSignal::new(Vec::<DatasetFormOption>::new());
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let nodes = RwSignal::new(Vec::<NodeResponse>::new());
    let rendered_forms = RwSignal::new(BTreeMap::<String, DatasetRenderedForm>::new());
    let load_error = RwSignal::new(None::<String>);
    let save_error = RwSignal::new(None::<String>);
    let save_message = RwSignal::new(None::<String>);
    let sql_preview = RwSignal::new(None::<String>);
    let sql_preview_error = RwSignal::new(None::<String>);
    let sql_preview_expanded = RwSignal::new(false);
    let visibility_search = RwSignal::new(String::new());
    let designer_selection = RwSignal::new(DatasetDesignerSelection::Operation);
    let designer_sheet_open = RwSignal::new(false);
    let auto_seeded_sources = RwSignal::new(BTreeSet::<String>::new());

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_forms(forms, load_error);
            load_datasets(datasets, RwSignal::new(false), load_error);
            load_nodes(nodes, load_error);
            if let Some(dataset_id) = dataset_id.clone() {
                load_dataset_for_edit(
                    dataset_id.clone(),
                    name,
                    slug,
                    composition_mode,
                    visibility_node_ids,
                    sources,
                    fields,
                    join_left_key,
                    join_right_key,
                    sql_preview,
                    load_error,
                );
            }
        }
    });

    let save_dataset_id = dataset_id.clone();
    let preview_dataset_id = dataset_id.clone();

    view! {
        <AppShell active_route="datasets" title=title>
            <section class="route-panel datasets-page">
                <PageHeader title>
                    <a class="button button--secondary" href="/datasets">"Back to Datasets"</a>
                </PageHeader>
                {move || load_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                {move || save_error.get().map(|message| view! { <p class="form-status is-error">{message}</p> })}
                {move || save_message.get().map(|message| view! { <p class="form-status is-success">{message}</p> })}
                <form class="dataset-editor" on:submit=move |event| {
                    event.prevent_default();
                    save_dataset(
                        save_dataset_id.clone(),
                        name.get(),
                        slug.get(),
                        composition_mode.get(),
                        visibility_node_ids.get().into_iter().collect(),
                        sources.get(),
                        fields.get(),
                        join_left_key.get(),
                        join_right_key.get(),
                        save_error,
                        save_message,
                    );
                }>
                    <section class="route-panel__section dataset-editor-section">
                        <h3>"Dataset Definition"</h3>
                        <div class="form-grid">
                            <label class="form-field">
                                <span>"Name"</span>
                                <input required prop:value=move || name.get() on:input=move |event| name.set(event_target_value(&event))/>
                            </label>
                            <label class="form-field">
                                <span>"Slug"</span>
                                <input required prop:value=move || slug.get() on:input=move |event| slug.set(event_target_value(&event))/>
                            </label>
                        </div>
                    </section>
                    <DatasetSourcesEditor
                        sources
                        forms
                        datasets
                        rendered_forms
                        composition_mode
                        fields
                        join_left_key
                        join_right_key
                        designer_selection
                        designer_sheet_open
                        auto_seeded_sources
                    />
                    <DatasetFieldsEditor fields sources forms rendered_forms designer_selection designer_sheet_open/>
                    <DatasetSqlPreviewPanel
                        dataset_id=dataset_id.clone()
                        name
                        slug
                        composition_mode
                        visibility_node_ids
                        sources
                        fields
                        join_left_key
                        join_right_key
                        sql_preview
                        sql_preview_error
                        expanded=sql_preview_expanded
                    />
                    <section class="route-panel__section dataset-editor-section">
                        <div class="dataset-editor-section__header">
                            <h3>"Visibility"</h3>
                            <label class="searchable-data-table__search">
                                <Search class="searchable-data-table__search-icon"/>
                                <span class="sr-only">"Search visibility nodes"</span>
                                <input
                                    type="search"
                                    placeholder="Search nodes"
                                    prop:value=move || visibility_search.get()
                                    on:input=move |event| visibility_search.set(event_target_value(&event))
                                />
                            </label>
                        </div>
                        <div class="table-wrap dataset-visibility-table">
                            <DataTable>
                                <thead>
                                    <tr>
                                        <th scope="col">"Visible"</th>
                                        <th scope="col">"Node"</th>
                                        <th scope="col">"Type"</th>
                                        <th scope="col">"Parent"</th>
                                    </tr>
                                </thead>
                                <tbody>
                                    {move || {
                                        let query = visibility_search.get();
                                        let mut visible_nodes = nodes.get();
                                        visible_nodes.sort_by(|left, right| {
                                            left.node_type_name
                                                .cmp(&right.node_type_name)
                                                .then_with(|| left.parent_node_name.cmp(&right.parent_node_name))
                                                .then_with(|| left.name.cmp(&right.name))
                                        });
                                        visible_nodes
                                            .into_iter()
                                            .filter(|node| node_matches_visibility_query(node, &query))
                                            .map(|node| {
                                                let node_id = node.id.clone();
                                                let checked = visibility_node_ids.get().contains(&node_id);
                                                view! {
                                                    <tr>
                                                        <td>
                                                            <input
                                                                type="checkbox"
                                                                checked=checked
                                                                aria-label=format!("Toggle visibility for {}", node.name)
                                                                on:change=move |event| {
                                                                    let is_checked = event_target_checked(&event);
                                                                    visibility_node_ids.update(|ids| {
                                                                        if is_checked { ids.insert(node_id.clone()); } else { ids.remove(&node_id); }
                                                                    });
                                                                }
                                                            />
                                                        </td>
                                                        <th scope="row">{node.name}</th>
                                                        <td>{sentence_label(&node.node_type_name)}</td>
                                                        <td>{node.parent_node_name.unwrap_or_else(|| "Top-level".into())}</td>
                                                    </tr>
                                                }
                                            })
                                            .collect_view()
                                    }}
                                </tbody>
                            </DataTable>
                        </div>
                    </section>
                    <div class="form-actions">
                        <button class="button" type="submit">{if is_edit { "Save Dataset" } else { "Create Dataset" }}</button>
                    </div>
                </form>
                {move || preview_dataset_id.clone().map(|id| view! {
                    <section class="route-panel__section dataset-editor-preview-link">
                        <a class="button button--secondary" href=format!("/datasets/{id}/preview") target="_blank" rel="noopener">"Open Preview"</a>
                    </section>
                })}
            </section>
        </AppShell>
    }
}

#[component]
/// Renders the dataset sources editor view.
fn DatasetSourcesEditor(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    composition_mode: RwSignal<String>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
    auto_seeded_sources: RwSignal<BTreeSet<String>>,
) -> impl IntoView {
    Effect::new(move |_| {
        let form_options = forms.get();
        sources.update(|items| {
            for source in items {
                if source.input_kind != "form"
                    || source.form_id.is_empty()
                    || !source.form_version_id.is_empty()
                {
                    continue;
                }
                let version = source
                    .form_version_major
                    .and_then(|major| {
                        published_versions_for_form(&form_options, &source.form_id)
                            .into_iter()
                            .find(|version| version.version_major == Some(major))
                    })
                    .or_else(|| first_published_version(&form_options, &source.form_id));
                if let Some(version) = version {
                    source.form_version_id = version.id;
                    source.form_version_major = version.version_major;
                }
            }
        });
    });

    Effect::new(move |_| {
        let form_options = forms.get();
        for (index, source) in sources.get().into_iter().enumerate() {
            if source.input_kind == "form"
                && let Some(version_id) = resolved_form_version_id(&source, &form_options)
            {
                load_rendered_form(version_id.clone(), rendered_forms);
                if rendered_forms.get().contains_key(&version_id) {
                    let seed_key = source_seed_key(index, &version_id);
                    if !auto_seeded_sources.get().contains(&seed_key) {
                        add_fields_from_source(index, sources, forms, rendered_forms, fields);
                        auto_seeded_sources.update(|keys| {
                            keys.insert(seed_key);
                        });
                    }
                }
            }
        }
    });

    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Operation Designer"</h3>
                <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                    let next = sources.get().len() + 1;
                    sources.update(|items| items.push(DatasetSourceDraft { source_alias: format!("source_{next}"), ..DatasetSourceDraft::default() }));
                    designer_selection.set(DatasetDesignerSelection::Source(next - 1));
                    designer_sheet_open.set(true);
                }>"Add Input"</button>
            </div>
            <div class="dataset-expression-workspace">
                <div class="dataset-expression-canvas">
                    <ExpressionPreview sources=sources composition_mode/>
                    <DatasetExpressionChain
                        sources
                        fields
                        composition_mode
                        designer_selection
                        designer_sheet_open
                    />
                </div>
                <DatasetDesignerOptionsSheet
                    selection=designer_selection
                    is_open=designer_sheet_open
                    sources
                    forms
                    datasets
                    rendered_forms
                    composition_mode
                    fields
                    join_left_key
                    join_right_key
                />
            </div>
        </section>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
/// Renders the dataset sql preview panel view.
fn DatasetSqlPreviewPanel(
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
fn ExpressionPreview(
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
fn DatasetExpressionChain(
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

#[allow(clippy::too_many_arguments)]
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

#[allow(clippy::too_many_arguments)]
#[component]
/// Renders the dataset designer options sheet view.
fn DatasetDesignerOptionsSheet(
    selection: RwSignal<DatasetDesignerSelection>,
    is_open: RwSignal<bool>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    composition_mode: RwSignal<String>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || is_open.get()>
                <section class="sheet-overlay dataset-options-overlay" aria-label="Dataset designer options overlay">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close dataset designer options" on:click=move |_| is_open.set(false)></button>
                    <aside class="sheet-panel blurred-surface dataset-options-sheet" role="dialog" aria-modal="true" aria-label="Dataset designer options">
                        <div class="sheet-panel__actions">
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close dataset designer options" title="Close dataset designer options" on:click=move |_| is_open.set(false)>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || match selection.get() {
                            DatasetDesignerSelection::Operation => view! {
                                <OperationOptionsPanel
                                    sources
                                    forms
                                    rendered_forms
                                    composition_mode
                                    join_left_key
                                    join_right_key
                                />
                            }.into_any(),
                            DatasetDesignerSelection::Source(index) => view! {
                                <SourceOptionsPanel
                                    index
                                    sources
                                    forms
                                    datasets
                                    rendered_forms
                                    fields
                                    composition_mode
                                />
                            }.into_any(),
                            DatasetDesignerSelection::Field(index) => view! {
                                <FieldOptionsPanel
                                    index
                                    fields
                                    sources
                                    forms
                                    rendered_forms
                                />
                            }.into_any(),
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
/// Renders the operation options panel view.
fn OperationOptionsPanel(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    composition_mode: RwSignal<String>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="dataset-options-sheet__content">
            <header class="dataset-options-sheet__header">
                <span>"Operation"</span>
                <h4>{move || operation_label(&composition_mode.get())}</h4>
            </header>
            <label class="form-field">
                <span>"Operation"</span>
                <select prop:value=move || composition_mode.get() on:change=move |event| composition_mode.set(event_target_value(&event))>
                    <option value="union">"Union"</option>
                    <option value="union_all">"Union All"</option>
                    <option value="left_join">"Left Join"</option>
                    <option value="inner_join">"Inner Join"</option>
                    <option value="outer_join">"Outer Join"</option>
                </select>
            </label>
            {move || if is_join_operation(&composition_mode.get()) {
                let left_options = join_key_options_for_source_index(
                    &sources.get(),
                    &forms.get(),
                    &rendered_forms.get(),
                    0,
                    &join_left_key.get(),
                );
                let right_options = join_key_options_for_source_index(
                    &sources.get(),
                    &forms.get(),
                    &rendered_forms.get(),
                    1,
                    &join_right_key.get(),
                );
                view! {
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Left Join Key"</span>
                            <select prop:value=move || join_left_key.get() on:change=move |event| join_left_key.set(event_target_value(&event))>
                                <option value="">"Select field"</option>
                                {left_options.into_iter().map(|option| {
                                    view! { <option value=option.key.clone()>{join_key_option_label(&option)}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Right Join Key"</span>
                            <select prop:value=move || join_right_key.get() on:change=move |event| join_right_key.set(event_target_value(&event))>
                                <option value="">"Select field"</option>
                                {right_options.into_iter().map(|option| {
                                    view! { <option value=option.key.clone()>{join_key_option_label(&option)}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

#[allow(clippy::too_many_arguments)]
#[component]
/// Renders the source options panel view.
fn SourceOptionsPanel(
    index: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    datasets: RwSignal<Vec<DatasetSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    composition_mode: RwSignal<String>,
) -> impl IntoView {
    view! {
        {move || sources.get().get(index).cloned().map(|source| {
            view! {
                <div class="dataset-options-sheet__content">
                    <header class="dataset-options-sheet__header">
                        <span>"Source"</span>
                        <h4>{source.source_alias.clone()}</h4>
                    </header>
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Alias"</span>
                            <input prop:value=source.source_alias.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                sources.update(|items| if let Some(item) = items.get_mut(index) { item.source_alias = value; });
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Input Type"</span>
                            <select prop:value=source.input_kind.clone() on:change=move |event| {
                                let value = event_target_value(&event);
                                sources.update(|items| {
                                    if let Some(item) = items.get_mut(index) {
                                        item.input_kind = value.clone();
                                    }
                                });
                            }>
                                <option value="form">"Form"</option>
                                <option value="dataset">"Dataset"</option>
                            </select>
                        </label>
                        {if source.input_kind == "dataset" {
                            view! {
                                <label class="form-field">
                                    <span>"Dataset"</span>
                                    <select prop:value=source.dataset_id.clone() on:change=move |event| {
                                        let dataset_id = event_target_value(&event);
                                        let revision_id = datasets
                                            .get()
                                            .into_iter()
                                            .find(|dataset| dataset.id == dataset_id)
                                            .and_then(|dataset| dataset.current_revision_id)
                                            .unwrap_or_default();
                                        sources.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.dataset_id = dataset_id.clone();
                                                item.dataset_revision_id = revision_id.clone();
                                            }
                                        });
                                    }>
                                        <option value="">"Select dataset"</option>
                                        {datasets.get().into_iter().filter(|dataset| dataset.current_revision_id.is_some()).map(|dataset| {
                                            view! { <option value=dataset.id>{dataset.name}</option> }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Revision"</span>
                                    <input readonly prop:value=source.dataset_revision_id.clone()/>
                                </label>
                            }.into_any()
                        } else {
                            view! {
                                <label class="form-field">
                                    <span>"Form"</span>
                                    <select prop:value=source.form_id.clone() on:change=move |event| {
                                        let form_id = event_target_value(&event);
                                        sources.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.form_id = form_id.clone();
                                                if let Some(version) = first_published_version(&forms.get(), &form_id) {
                                                    item.form_version_id = version.id.clone();
                                                    item.form_version_major = version.version_major;
                                                    load_rendered_form(version.id.clone(), rendered_forms);
                                                }
                                            }
                                        });
                                    }>
                                        <option value="">"Select form"</option>
                                        {forms.get().into_iter().map(|form| view! { <option value=form.id>{form.name}</option> }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Version"</span>
                                    <select prop:value=source.form_version_id.clone() on:change=move |event| {
                                        let version_id = event_target_value(&event);
                                        sources.update(|items| {
                                            if let Some(item) = items.get_mut(index) {
                                                item.form_version_id = version_id.clone();
                                                item.form_version_major = find_version(&forms.get(), &version_id).and_then(|version| version.version_major);
                                                load_rendered_form(version_id.clone(), rendered_forms);
                                            }
                                        });
                                    }>
                                        {published_versions_for_form(&forms.get(), &source.form_id).into_iter().map(|version| {
                                            view! { <option value=version.id>{version_label(&version)}</option> }
                                        }).collect_view()}
                                    </select>
                                </label>
                                <label class="form-field">
                                    <span>"Selection"</span>
                                    <select prop:value=source.selection_rule.clone() on:change=move |event| {
                                        let value = event_target_value(&event);
                                        sources.update(|items| if let Some(item) = items.get_mut(index) { item.selection_rule = value; });
                                    }>
                                        <option value="latest">"Latest"</option>
                                        <option value="earliest">"Earliest"</option>
                                        {move || if composition_mode.get() == "union" {
                                            view! { <option value="all">"All"</option> }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </select>
                                </label>
                                <button class="button button--secondary" type="button" on:click=move |_| add_fields_from_source(index, sources, forms, rendered_forms, fields)>"Add Fields From Source"</button>
                            }.into_any()
                        }}
                    </div>
                </div>
            }.into_any()
        }).unwrap_or_else(|| view! {
            <div class="dataset-options-sheet__content">
                <header class="dataset-options-sheet__header">
                    <span>"Source"</span>
                    <h4>"No Source Selected"</h4>
                </header>
            </div>
        }.into_any())}
    }
}

#[component]
/// Renders the field options panel view.
fn FieldOptionsPanel(
    index: usize,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
) -> impl IntoView {
    view! {
        {move || fields.get().get(index).cloned().map(|field| {
            view! {
                <div class="dataset-options-sheet__content">
                    <header class="dataset-options-sheet__header">
                        <span>"Projected Field"</span>
                        <h4>{field.label.clone()}</h4>
                    </header>
                    <div class="dataset-options-sheet__stack">
                        <label class="form-field">
                            <span>"Key"</span>
                            <input prop:value=field.key.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.key = value; });
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Label"</span>
                            <input prop:value=field.label.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.label = value; });
                            }/>
                        </label>
                        <label class="form-field">
                            <span>"Source"</span>
                            <select prop:value=field.source_alias.clone() on:change=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.source_alias = value; });
                            }>
                                {sources.get().into_iter().map(|source| view! { <option value=source.source_alias.clone()>{source.source_alias.clone()}</option> }).collect_view()}
                            </select>
                        </label>
                        <label class="form-field">
                            <span>"Source Field"</span>
                            <select prop:value=field.source_field_key.clone() on:change=move |event| {
                                let value = event_target_value(&event);
                                fields.update(|items| if let Some(item) = items.get_mut(index) { item.source_field_key = value; });
                            }>
                                {source_field_options_with_selected(
                                    &sources.get(),
                                    &forms.get(),
                                    &rendered_forms.get(),
                                    &field.source_alias,
                                    &field.source_field_key,
                                ).into_iter().map(|option| {
                                    view! { <option value=option.key.clone()>{join_key_option_label(&option)}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </div>
                </div>
            }.into_any()
        }).unwrap_or_else(|| view! {
            <div class="dataset-options-sheet__content">
                <header class="dataset-options-sheet__header">
                    <span>"Projected Field"</span>
                    <h4>"No Field Selected"</h4>
                </header>
            </div>
        }.into_any())}
    }
}

#[component]
/// Renders the dataset fields editor view.
fn DatasetFieldsEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    designer_selection: RwSignal<DatasetDesignerSelection>,
    designer_sheet_open: RwSignal<bool>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Fields"</h3>
                <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                    let next = fields.get().len() + 1;
                    fields.update(|items| items.push(DatasetFieldDraft {
                        key: format!("field_{next}"),
                        label: format!("Field {next}"),
                        source_alias: sources.get().first().map(|source| source.source_alias.clone()).unwrap_or_default(),
                        source_field_key: String::new(),
                    }));
                    designer_selection.set(DatasetDesignerSelection::Field(next - 1));
                    designer_sheet_open.set(true);
                }>"Add Field"</button>
            </div>
            <div class="table-wrap dataset-fields-table">
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Source"</th>
                            <th scope="col">"Field"</th>
                            <th scope="col">"Form Field Label"</th>
                            <th scope="col">"Source Field"</th>
                            <th scope="col">"Data Type"</th>
                            <th scope="col">"Remove"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {move || fields.get().into_iter().enumerate().map(|(index, field)| {
                            let metadata = field_metadata(&field, &sources.get(), &forms.get(), &rendered_forms.get());
                            view! {
                                <tr class=move || if designer_selection.get() == DatasetDesignerSelection::Field(index) { "is-selected" } else { "" }>
                                    <td>{field.source_alias.clone()}</td>
                                    <th scope="row">
                                        <button
                                            class="link-button"
                                            type="button"
                                            on:click=move |_| {
                                                designer_selection.set(DatasetDesignerSelection::Field(index));
                                                designer_sheet_open.set(true);
                                            }
                                        >
                                            {field.label.clone()}
                                        </button>
                                        <span class="data-table__secondary-text">{field.key.clone()}</span>
                                    </th>
                                    <td>{metadata.label}</td>
                                    <td>{field.source_field_key.clone()}</td>
                                    <td>{sentence_label(&metadata.field_type)}</td>
                                    <td>
                                        <button
                                            class="button button--secondary button--compact"
                                            type="button"
                                            on:click=move |_| {
                                                if confirm_action("Remove this projected field?") {
                                                    fields.update(|items| {
                                                        if index < items.len() {
                                                            items.remove(index);
                                                        }
                                                    });
                                                    designer_selection.set(DatasetDesignerSelection::Operation);
                                                }
                                            }
                                        >
                                            "Remove"
                                        </button>
                                    </td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </DataTable>
            </div>
        </section>
    }
}

#[component]
/// Renders the table pagination view.
fn TablePagination(
    summary: String,
    page_count: usize,
    current_page: usize,
    page_index: RwSignal<usize>,
    page_size: RwSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="directory-table-pagination">
            <span>{summary}</span>
            <div class="directory-table-pagination__actions">
                <label>"Rows"
                    <select prop:value=move || page_size.get().to_string() on:change=move |event| {
                        if let Ok(value) = event_target_value(&event).parse::<usize>() {
                            page_size.set(value);
                            page_index.set(0);
                        }
                    }>
                        <option value="10">"10"</option>
                        <option value="25">"25"</option>
                        <option value="50">"50"</option>
                    </select>
                </label>
                <button class="button button--compact" type="button" disabled=move || page_index.get() == 0 on:click=move |_| page_index.update(|value| *value = value.saturating_sub(1))>"Previous"</button>
                <strong>{format!("Page {current_page} of {page_count}")}</strong>
                <button class="button button--compact" type="button" disabled=move || page_index.get() + 1 >= page_count on:click=move |_| page_index.update(|value| *value += 1)>"Next"</button>
            </div>
        </div>
    }
}

/// Handles the can manage datasets behavior.
fn can_manage_datasets(account: &SessionAccount) -> bool {
    account
        .capabilities
        .iter()
        .any(|capability| capability == "admin:all")
}

/// Handles the operation label behavior.
fn operation_label(value: &str) -> &'static str {
    match value {
        "union" => "UNION",
        "union_all" => "UNION ALL",
        "left_join" => "LEFT JOIN",
        "inner_join" => "INNER JOIN",
        "outer_join" => "OUTER JOIN",
        _ => "OPERATION",
    }
}

/// Returns whether the is join operation condition is met.
pub(super) fn is_join_operation(value: &str) -> bool {
    matches!(value, "left_join" | "inner_join" | "outer_join")
}

/// Handles the expression label behavior.
fn expression_label(sources: &[DatasetSourceDraft], operation: &str) -> String {
    let aliases = sources
        .iter()
        .filter(|source| !source.source_alias.trim().is_empty())
        .map(|source| source.source_alias.clone())
        .collect::<Vec<_>>();
    if aliases.is_empty() {
        return "Choose at least one input".into();
    }
    aliases
        .into_iter()
        .reduce(|left, right| format!("({left}) {} ({right})", operation_label(operation)))
        .unwrap_or_else(|| "Choose at least one input".into())
}

/// Handles the expression button class behavior.
fn expression_button_class(is_active: bool, base: &'static str) -> String {
    if is_active {
        format!("{base} is-active")
    } else {
        base.into()
    }
}

#[allow(dead_code)]
/// Handles the expression to editor drafts behavior.
pub(super) fn expression_to_editor_drafts(
    ast: &DatasetExpressionPayload,
) -> (Vec<DatasetSourceDraft>, String, Vec<DatasetJoinKeyPayload>) {
    let mut sources = Vec::new();
    let mut operation = String::new();
    let mut join_keys = Vec::new();
    collect_expression_drafts(ast, &mut sources, &mut operation, &mut join_keys);
    (sources, operation, join_keys)
}

#[allow(dead_code)]
/// Collects the collect expression drafts values.
fn collect_expression_drafts(
    ast: &DatasetExpressionPayload,
    sources: &mut Vec<DatasetSourceDraft>,
    operation: &mut String,
    join_keys: &mut Vec<DatasetJoinKeyPayload>,
) {
    match ast {
        DatasetExpressionPayload::Form {
            alias,
            form_id,
            form_version_major,
            selection_rule,
        } => sources.push(DatasetSourceDraft {
            input_kind: "form".into(),
            source_alias: alias.clone(),
            form_id: form_id.clone(),
            form_version_id: String::new(),
            form_version_major: *form_version_major,
            dataset_id: String::new(),
            dataset_revision_id: String::new(),
            selection_rule: selection_rule.clone(),
        }),
        DatasetExpressionPayload::Dataset {
            alias,
            dataset_id,
            dataset_revision_id,
        } => sources.push(DatasetSourceDraft {
            input_kind: "dataset".into(),
            source_alias: alias.clone(),
            form_id: String::new(),
            form_version_id: String::new(),
            form_version_major: None,
            dataset_id: dataset_id.clone(),
            dataset_revision_id: dataset_revision_id.clone(),
            selection_rule: "latest".into(),
        }),
        DatasetExpressionPayload::Operation {
            operation: node_operation,
            left,
            right,
            join_keys: node_join_keys,
            ..
        } => {
            if operation.is_empty() {
                *operation = node_operation.clone();
                *join_keys = node_join_keys.clone();
            }
            collect_expression_drafts(left, sources, operation, join_keys);
            collect_expression_drafts(right, sources, operation, join_keys);
        }
    }
}

#[allow(dead_code)]
/// Builds the build expression ast value.
pub(super) fn build_expression_ast(
    sources: &[DatasetSourceDraft],
    operation: &str,
    join_left_key: &str,
    join_right_key: &str,
) -> Option<DatasetExpressionPayload> {
    let mut inputs = sources
        .iter()
        .filter_map(source_expression)
        .collect::<Vec<_>>()
        .into_iter();
    let first = inputs.next()?;
    Some(
        inputs.fold(first, |left, right| DatasetExpressionPayload::Operation {
            alias: "result".into(),
            operation: operation.into(),
            left: Box::new(left),
            right: Box::new(right),
            join_keys: if is_join_operation(operation)
                && !join_left_key.trim().is_empty()
                && !join_right_key.trim().is_empty()
            {
                vec![DatasetJoinKeyPayload {
                    left_field: join_left_key.trim().into(),
                    right_field: join_right_key.trim().into(),
                }]
            } else {
                Vec::new()
            },
        }),
    )
}

#[allow(dead_code)]
/// Handles the source expression behavior.
fn source_expression(source: &DatasetSourceDraft) -> Option<DatasetExpressionPayload> {
    if source.source_alias.trim().is_empty() {
        return None;
    }
    if source.input_kind == "dataset" {
        if source.dataset_id.is_empty() || source.dataset_revision_id.is_empty() {
            return None;
        }
        Some(DatasetExpressionPayload::Dataset {
            alias: source.source_alias.clone(),
            dataset_id: source.dataset_id.clone(),
            dataset_revision_id: source.dataset_revision_id.clone(),
        })
    } else {
        if source.form_id.is_empty() {
            return None;
        }
        Some(DatasetExpressionPayload::Form {
            alias: source.source_alias.clone(),
            form_id: source.form_id.clone(),
            form_version_major: source.form_version_major,
            selection_rule: source.selection_rule.clone(),
        })
    }
}

/// Handles the visibility label behavior.
fn visibility_label(nodes: &[DatasetVisibilityNode]) -> String {
    match nodes.len() {
        0 => "No nodes".into(),
        1 => nodes[0].node_path.clone(),
        count => format!("{count} nodes"),
    }
}

/// Handles the node matches visibility query behavior.
fn node_matches_visibility_query(node: &NodeResponse, query: &str) -> bool {
    query.trim().is_empty()
        || text_matches(query, &[&node.name])
        || text_matches(query, &[&node.node_type_name])
        || node
            .parent_node_name
            .as_ref()
            .is_some_and(|parent| text_matches(query, &[parent]))
}

/// Handles the field metadata behavior.
fn field_metadata(
    field: &DatasetFieldDraft,
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
) -> DatasetRenderedField {
    source_field_options(sources, forms, rendered_forms, &field.source_alias)
        .into_iter()
        .find(|option| option.key == field.source_field_key)
        .unwrap_or_else(|| DatasetRenderedField {
            key: field.source_field_key.clone(),
            label: "Unknown field".into(),
            field_type: String::new(),
        })
}

/// Handles the confirm action behavior.
fn confirm_action(message: &str) -> bool {
    #[cfg(feature = "hydrate")]
    {
        return web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(false);
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = message;
        true
    }
}

/// Handles the version label behavior.
fn version_label(version: &DatasetFormVersionOption) -> String {
    version
        .version_label
        .clone()
        .unwrap_or_else(|| format!("Major {}", version.version_major.unwrap_or(1)))
}

/// Handles the first published version behavior.
fn first_published_version(
    forms: &[DatasetFormOption],
    form_id: &str,
) -> Option<DatasetFormVersionOption> {
    forms
        .iter()
        .find(|form| form.id == form_id)
        .and_then(|form| {
            published_versions_for_form(forms, &form.id)
                .into_iter()
                .next()
        })
}

/// Handles the published versions for form behavior.
fn published_versions_for_form(
    forms: &[DatasetFormOption],
    form_id: &str,
) -> Vec<DatasetFormVersionOption> {
    forms
        .iter()
        .find(|form| form.id == form_id)
        .map(|form| {
            form.versions
                .iter()
                .filter(|version| version.status == "published")
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Handles the find version behavior.
fn find_version(forms: &[DatasetFormOption], version_id: &str) -> Option<DatasetFormVersionOption> {
    forms
        .iter()
        .flat_map(|form| form.versions.iter())
        .find(|version| version.id == version_id)
        .cloned()
}

/// Handles the source field options behavior.
fn source_field_options(
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_alias: &str,
) -> Vec<DatasetRenderedField> {
    let Some(source) = sources
        .iter()
        .find(|source| source.source_alias == source_alias)
    else {
        return Vec::new();
    };
    let mut options = system_source_field_options();
    let form_version_id = resolved_form_version_id(source, forms);
    options.extend(
        form_version_id
            .as_deref()
            .and_then(|version_id| rendered_forms.get(version_id))
            .map(|rendered| {
                rendered
                    .sections
                    .iter()
                    .flat_map(|section| section.fields.iter().cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    );
    options
}

/// Handles the source field options with selected behavior.
fn source_field_options_with_selected(
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_alias: &str,
    selected_key: &str,
) -> Vec<DatasetRenderedField> {
    let mut options = source_field_options(sources, forms, rendered_forms, source_alias);

    if !selected_key.is_empty() && !options.iter().any(|option| option.key == selected_key) {
        options.push(DatasetRenderedField {
            key: selected_key.to_string(),
            label: "Unknown field".into(),
            field_type: String::new(),
        });
    }

    options
}

/// Handles the join key options for source index behavior.
fn join_key_options_for_source_index(
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_index: usize,
    selected_key: &str,
) -> Vec<DatasetRenderedField> {
    let mut options = sources
        .get(source_index)
        .map(|source| source_field_options(sources, forms, rendered_forms, &source.source_alias))
        .unwrap_or_default();

    if !selected_key.is_empty() && !options.iter().any(|option| option.key == selected_key) {
        options.push(DatasetRenderedField {
            key: selected_key.to_string(),
            label: "Unknown field".into(),
            field_type: String::new(),
        });
    }

    options
}

/// Handles the resolved form version id behavior.
fn resolved_form_version_id(
    source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
) -> Option<String> {
    if !source.form_version_id.is_empty() {
        return Some(source.form_version_id.clone());
    }
    source
        .form_version_major
        .and_then(|major| {
            published_versions_for_form(forms, &source.form_id)
                .into_iter()
                .find(|version| version.version_major == Some(major))
        })
        .or_else(|| first_published_version(forms, &source.form_id))
        .map(|version| version.id)
}

/// Handles the system source field options behavior.
fn system_source_field_options() -> Vec<DatasetRenderedField> {
    [
        ("__submission_id", "Submission ID", "text"),
        ("__form_version_id", "Form Version ID", "text"),
        ("__node_id", "Attached Node ID", "text"),
        ("__node_name", "Attached Node Name", "text"),
        ("__submission_status", "Submission Status", "text"),
        ("__submitted_at", "Submitted Date", "date"),
        ("__submission_created_at", "Created Date", "date"),
        ("__last_updated_at", "Updated Date", "date"),
        (
            "__last_updated_by_user_name",
            "Updated By User Name",
            "text",
        ),
    ]
    .into_iter()
    .map(|(key, label, field_type)| DatasetRenderedField {
        key: key.into(),
        label: label.into(),
        field_type: field_type.into(),
    })
    .collect()
}

/// Handles the join key option label behavior.
fn join_key_option_label(field: &DatasetRenderedField) -> String {
    format!("{} ({})", truncate_field_label(&field.label), field.key)
}

/// Handles the truncate field label behavior.
fn truncate_field_label(label: &str) -> String {
    const MAX_CHARS: usize = 32;
    let mut chars = label.chars();
    let truncated = chars.by_ref().take(MAX_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

/// Handles the add fields from source behavior.
fn add_fields_from_source(
    index: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
) {
    let source = sources.get().get(index).cloned();
    if let Some(source) = source {
        let options = source_field_options(
            &sources.get(),
            &forms.get(),
            &rendered_forms.get(),
            &source.source_alias,
        );
        fields.update(|items| {
            for option in options {
                let key = format!("{}_{}", source.source_alias, option.key);
                if items.iter().any(|item| {
                    item.key == key
                        || (item.source_alias == source.source_alias
                            && item.source_field_key == option.key)
                }) {
                    continue;
                }
                items.push(DatasetFieldDraft {
                    key,
                    label: option.label,
                    source_alias: source.source_alias.clone(),
                    source_field_key: option.key,
                });
            }
        });
    }
}

/// Handles the source seed key behavior.
fn source_seed_key(index: usize, form_version_id: &str) -> String {
    format!("{index}:{form_version_id}")
}

/// Handles the table summary behavior.
fn table_summary(total_count: usize, page_size: usize, page_index: usize, label: &str) -> String {
    if total_count == 0 {
        format!("No {label} to display")
    } else {
        format!(
            "Showing {}-{} of {} {label}",
            pagination_page_start(total_count, page_size, page_index) + 1,
            pagination_page_end(total_count, page_size, page_index),
            total_count
        )
    }
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
