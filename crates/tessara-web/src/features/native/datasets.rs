use std::collections::{BTreeMap, BTreeSet};

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

use super::{
    pagination_current_page, pagination_page_count, pagination_page_end, pagination_page_start,
    text_matches,
};
#[cfg(feature = "hydrate")]
use crate::infra::http::{redirect_to_login, send_json_request};
use crate::infra::routing::{DatasetRouteParams, require_route_params};
use crate::ui::components::{AppShell, DataTable, EmptyState, PageHeader, StatusBadge};
use icons::Search;

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct SessionAccount {
    capabilities: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetSummary {
    id: String,
    current_revision_id: Option<String>,
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    visibility_nodes: Vec<DatasetVisibilityNode>,
    source_count: i64,
    field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetDefinition {
    id: String,
    current_revision_id: Option<String>,
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    visibility_nodes: Vec<DatasetVisibilityNode>,
    sources: Vec<DatasetSourceDefinition>,
    fields: Vec<DatasetFieldDefinition>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetVisibilityNode {
    node_id: String,
    node_name: String,
    node_type_name: String,
    parent_node_id: Option<String>,
    node_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetSourceDefinition {
    source_alias: String,
    form_id: Option<String>,
    form_name: Option<String>,
    form_version_major: Option<i32>,
    selection_rule: String,
    position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetFieldDefinition {
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    field_type: String,
    position: i32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetTable {
    rows: Vec<DatasetTableRow>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct DatasetTableRow {
    submission_id: String,
    node_name: String,
    source_alias: String,
    values: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormSummary {
    id: String,
    name: String,
    versions: Vec<FormVersionSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormVersionSummary {
    id: String,
    version_label: Option<String>,
    status: String,
    version_major: Option<i32>,
    field_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RenderedForm {
    form_version_id: String,
    form_id: String,
    form_name: String,
    sections: Vec<RenderedSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RenderedSection {
    fields: Vec<RenderedField>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RenderedField {
    key: String,
    label: String,
    field_type: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeResponse {
    id: String,
    node_type_name: String,
    parent_node_name: Option<String>,
    name: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
struct DatasetPayload {
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    visibility_node_ids: Vec<String>,
    sources: Vec<DatasetSourcePayload>,
    fields: Vec<DatasetFieldPayload>,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
struct DatasetSourcePayload {
    source_alias: String,
    form_id: Option<String>,
    form_version_major: Option<i32>,
    selection_rule: String,
}

#[allow(dead_code)]
#[derive(Clone, Debug, Serialize)]
struct DatasetFieldPayload {
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    position: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct DatasetSourceDraft {
    source_alias: String,
    form_id: String,
    form_version_id: String,
    form_version_major: Option<i32>,
    selection_rule: String,
}

#[derive(Clone, Debug, PartialEq)]
struct DatasetFieldDraft {
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
}

impl Default for DatasetSourceDraft {
    fn default() -> Self {
        Self {
            source_alias: "source_1".into(),
            form_id: String::new(),
            form_version_id: String::new(),
            form_version_major: None,
            selection_rule: "latest".into(),
        }
    }
}

#[component]
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
pub fn DatasetsDetailPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    view! { <DatasetDetailSurface dataset_id edit=false/> }
}

#[component]
pub fn DatasetsEditPage() -> impl IntoView {
    let params = require_route_params::<DatasetRouteParams>();
    let dataset_id = params.dataset_id;
    view! { <DatasetEditorSurface dataset_id=Some(dataset_id)/> }
}

#[component]
pub fn DatasetsNewPage() -> impl IntoView {
    view! { <DatasetEditorSurface dataset_id=None/> }
}

#[component]
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
                                </div>
                                {move || if active_tab.get() == "preview" {
                                    view! { <DatasetPreviewTable dataset=loaded.clone() table=table.get() error=table_error.get()/> }.into_any()
                                } else if active_tab.get() == "sources" {
                                    view! { <DatasetSourcesTable sources=loaded.sources.clone()/> }.into_any()
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
fn MetricCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="metric-card">
            <span>{label}</span>
            <strong>{value}</strong>
        </div>
    }
}

#[component]
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
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let nodes = RwSignal::new(Vec::<NodeResponse>::new());
    let rendered_forms = RwSignal::new(BTreeMap::<String, RenderedForm>::new());
    let table = RwSignal::new(None::<DatasetTable>);
    let load_error = RwSignal::new(None::<String>);
    let table_error = RwSignal::new(None::<String>);
    let save_error = RwSignal::new(None::<String>);
    let save_message = RwSignal::new(None::<String>);

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_forms(forms, load_error);
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
                    load_error,
                );
                load_dataset_table(dataset_id, table, table_error);
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
                            <label class="form-field">
                                <span>"Composition"</span>
                                <select prop:value=move || composition_mode.get() on:change=move |event| composition_mode.set(event_target_value(&event))>
                                    <option value="union">"Union"</option>
                                    <option value="join">"Join"</option>
                                </select>
                            </label>
                        </div>
                    </section>
                    <section class="route-panel__section dataset-editor-section">
                        <h3>"Visibility"</h3>
                        <div class="dataset-checkbox-grid">
                            {move || nodes.get().into_iter().map(|node| {
                                let node_id = node.id.clone();
                                let checked = visibility_node_ids.get().contains(&node_id);
                                view! {
                                    <label class="dataset-checkbox">
                                        <input
                                            type="checkbox"
                                            checked=checked
                                            on:change=move |event| {
                                                let is_checked = event_target_checked(&event);
                                                visibility_node_ids.update(|ids| {
                                                    if is_checked { ids.insert(node_id.clone()); } else { ids.remove(&node_id); }
                                                });
                                            }
                                        />
                                        <span>{node_label(&node)}</span>
                                    </label>
                                }
                            }).collect_view()}
                        </div>
                    </section>
                    <DatasetSourcesEditor sources forms rendered_forms composition_mode fields/>
                    <DatasetFieldsEditor fields sources rendered_forms/>
                    <div class="form-actions">
                        <button class="button" type="submit">{if is_edit { "Save Dataset" } else { "Create Dataset" }}</button>
                    </div>
                </form>
                {move || {
                    preview_dataset_id.clone().map(|id| {
                        let preview_dataset = DatasetDefinition {
                            id,
                            current_revision_id: None,
                            name: name.get(),
                            slug: slug.get(),
                            grain: "submission".into(),
                            composition_mode: composition_mode.get(),
                            visibility_nodes: Vec::new(),
                            sources: Vec::new(),
                            fields: fields
                                .get()
                                .into_iter()
                                .enumerate()
                                .map(|(index, field)| DatasetFieldDefinition {
                                    key: field.key,
                                    label: field.label,
                                    source_alias: field.source_alias,
                                    source_field_key: field.source_field_key,
                                    field_type: String::new(),
                                    position: index as i32,
                                })
                                .collect(),
                        };
                        view! {
                            <DatasetPreviewTable
                                dataset=preview_dataset
                                table=table.get()
                                error=table_error.get()
                            />
                        }
                    })
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn DatasetSourcesEditor(
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<FormSummary>>,
    rendered_forms: RwSignal<BTreeMap<String, RenderedForm>>,
    composition_mode: RwSignal<String>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <div class="dataset-editor-section__header">
                <h3>"Sources"</h3>
                <button class="button button--secondary button--compact" type="button" on:click=move |_| {
                    let next = sources.get().len() + 1;
                    sources.update(|items| items.push(DatasetSourceDraft { source_alias: format!("source_{next}"), ..DatasetSourceDraft::default() }));
                }>"Add Source"</button>
            </div>
            {move || sources.get().into_iter().enumerate().map(|(index, source)| {
                view! {
                    <div class="dataset-editor-row dataset-editor-row--source">
                        <label class="form-field">
                            <span>"Alias"</span>
                            <input prop:value=source.source_alias.clone() on:input=move |event| {
                                let value = event_target_value(&event);
                                sources.update(|items| if let Some(item) = items.get_mut(index) { item.source_alias = value; });
                            }/>
                        </label>
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
                        <button class="button button--compact button--secondary" type="button" on:click=move |_| add_fields_from_source(index, sources, rendered_forms, fields)>"Add Fields"</button>
                    </div>
                }
            }).collect_view()}
        </section>
    }
}

#[component]
fn DatasetFieldsEditor(
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    rendered_forms: RwSignal<BTreeMap<String, RenderedForm>>,
) -> impl IntoView {
    view! {
        <section class="route-panel__section dataset-editor-section">
            <h3>"Fields"</h3>
            {move || fields.get().into_iter().enumerate().map(|(index, field)| {
                view! {
                    <div class="dataset-editor-row dataset-editor-row--fields">
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
                                {source_field_options(&sources.get(), &rendered_forms.get(), &field.source_alias).into_iter().map(|option| {
                                    view! { <option value=option.key>{option.label}</option> }
                                }).collect_view()}
                            </select>
                        </label>
                    </div>
                }
            }).collect_view()}
        </section>
    }
}

#[component]
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

fn can_manage_datasets(account: &SessionAccount) -> bool {
    account
        .capabilities
        .iter()
        .any(|capability| capability == "admin:all")
}

fn sentence_label(value: &str) -> String {
    value
        .split('_')
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn visibility_label(nodes: &[DatasetVisibilityNode]) -> String {
    match nodes.len() {
        0 => "No nodes".into(),
        1 => nodes[0].node_path.clone(),
        count => format!("{count} nodes"),
    }
}

fn node_label(node: &NodeResponse) -> String {
    node.parent_node_name
        .as_ref()
        .map(|parent| format!("{parent} / {}", node.name))
        .unwrap_or_else(|| node.name.clone())
}

fn version_label(version: &FormVersionSummary) -> String {
    version
        .version_label
        .clone()
        .unwrap_or_else(|| format!("Major {}", version.version_major.unwrap_or(1)))
}

fn first_published_version(forms: &[FormSummary], form_id: &str) -> Option<FormVersionSummary> {
    forms
        .iter()
        .find(|form| form.id == form_id)
        .and_then(|form| {
            published_versions_for_form(forms, &form.id)
                .into_iter()
                .next()
        })
}

fn published_versions_for_form(forms: &[FormSummary], form_id: &str) -> Vec<FormVersionSummary> {
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

fn find_version(forms: &[FormSummary], version_id: &str) -> Option<FormVersionSummary> {
    forms
        .iter()
        .flat_map(|form| form.versions.iter())
        .find(|version| version.id == version_id)
        .cloned()
}

fn source_field_options(
    sources: &[DatasetSourceDraft],
    rendered_forms: &BTreeMap<String, RenderedForm>,
    source_alias: &str,
) -> Vec<RenderedField> {
    let Some(source) = sources
        .iter()
        .find(|source| source.source_alias == source_alias)
    else {
        return Vec::new();
    };
    rendered_forms
        .get(&source.form_version_id)
        .map(|rendered| {
            rendered
                .sections
                .iter()
                .flat_map(|section| section.fields.iter().cloned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn add_fields_from_source(
    index: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    rendered_forms: RwSignal<BTreeMap<String, RenderedForm>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
) {
    let source = sources.get().get(index).cloned();
    if let Some(source) = source {
        let options =
            source_field_options(&sources.get(), &rendered_forms.get(), &source.source_alias);
        fields.update(|items| {
            for option in options {
                let key = format!("{}_{}", source.source_alias, option.key);
                if items.iter().any(|item| item.key == key) {
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

fn tab_class(active_tab: RwSignal<String>, value: &'static str) -> impl Fn() -> &'static str {
    move || {
        if active_tab.get() == value {
            "tabs-trigger is-active"
        } else {
            "tabs-trigger"
        }
    }
}

#[cfg(feature = "hydrate")]
fn load_account(account: RwSignal<Option<SessionAccount>>) {
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get("/api/me").send().await {
            Ok(response) if response.status() == 401 => redirect_to_login(),
            Ok(response) if response.ok() => {
                if let Ok(payload) = response.json::<SessionAccount>().await {
                    account.set(Some(payload));
                }
            }
            _ => {}
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_account(_: RwSignal<Option<SessionAccount>>) {}

#[cfg(feature = "hydrate")]
fn load_datasets(
    datasets: RwSignal<Vec<DatasetSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        match gloo_net::http::Request::get("/api/datasets").send().await {
            Ok(response) if response.status() == 401 => redirect_to_login(),
            Ok(response) if response.ok() => match response.json::<Vec<DatasetSummary>>().await {
                Ok(payload) => datasets.set(payload),
                Err(_) => load_error.set(Some("Dataset list could not be read.".into())),
            },
            Ok(response) => load_error.set(Some(format!(
                "Dataset list failed with status {}.",
                response.status()
            ))),
            Err(_) => load_error.set(Some("Could not reach the dataset API.".into())),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_datasets(_: RwSignal<Vec<DatasetSummary>>, _: RwSignal<bool>, _: RwSignal<Option<String>>) {
}

#[cfg(feature = "hydrate")]
fn load_dataset_detail(
    dataset_id: String,
    dataset: RwSignal<Option<DatasetDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        match gloo_net::http::Request::get(&format!("/api/datasets/{dataset_id}"))
            .send()
            .await
        {
            Ok(response) if response.status() == 401 => redirect_to_login(),
            Ok(response) if response.ok() => match response.json::<DatasetDefinition>().await {
                Ok(payload) => dataset.set(Some(payload)),
                Err(_) => load_error.set(Some("Dataset detail could not be read.".into())),
            },
            Ok(response) => load_error.set(Some(format!(
                "Dataset detail failed with status {}.",
                response.status()
            ))),
            Err(_) => load_error.set(Some("Could not reach the dataset API.".into())),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_dataset_detail(
    _: String,
    _: RwSignal<Option<DatasetDefinition>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
fn load_dataset_table(
    dataset_id: String,
    table: RwSignal<Option<DatasetTable>>,
    table_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get(&format!("/api/datasets/{dataset_id}/table"))
            .send()
            .await
        {
            Ok(response) if response.status() == 401 => redirect_to_login(),
            Ok(response) if response.ok() => match response.json::<DatasetTable>().await {
                Ok(payload) => table.set(Some(payload)),
                Err(_) => table_error.set(Some("Dataset preview could not be read.".into())),
            },
            Ok(response) => table_error.set(Some(format!(
                "Dataset preview failed with status {}.",
                response.status()
            ))),
            Err(_) => table_error.set(Some("Could not reach the dataset preview API.".into())),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_dataset_table(_: String, _: RwSignal<Option<DatasetTable>>, _: RwSignal<Option<String>>) {}

#[cfg(feature = "hydrate")]
fn load_forms(forms: RwSignal<Vec<FormSummary>>, load_error: RwSignal<Option<String>>) {
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get("/api/forms").send().await {
            Ok(response) if response.status() == 401 => redirect_to_login(),
            Ok(response) if response.ok() => match response.json::<Vec<FormSummary>>().await {
                Ok(payload) => forms.set(payload),
                Err(_) => load_error.set(Some("Form options could not be read.".into())),
            },
            Ok(response) => load_error.set(Some(format!(
                "Form options failed with status {}.",
                response.status()
            ))),
            Err(_) => load_error.set(Some("Could not reach the forms API.".into())),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_forms(_: RwSignal<Vec<FormSummary>>, _: RwSignal<Option<String>>) {}

#[cfg(feature = "hydrate")]
fn load_nodes(nodes: RwSignal<Vec<NodeResponse>>, load_error: RwSignal<Option<String>>) {
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get("/api/nodes").send().await {
            Ok(response) if response.status() == 401 => redirect_to_login(),
            Ok(response) if response.ok() => match response.json::<Vec<NodeResponse>>().await {
                Ok(payload) => nodes.set(payload),
                Err(_) => load_error.set(Some("Visibility node options could not be read.".into())),
            },
            Ok(response) => load_error.set(Some(format!(
                "Visibility nodes failed with status {}.",
                response.status()
            ))),
            Err(_) => load_error.set(Some("Could not reach the nodes API.".into())),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_nodes(_: RwSignal<Vec<NodeResponse>>, _: RwSignal<Option<String>>) {}

#[cfg(feature = "hydrate")]
fn load_rendered_form(
    form_version_id: String,
    rendered_forms: RwSignal<BTreeMap<String, RenderedForm>>,
) {
    if form_version_id.is_empty()
        || rendered_forms
            .get_untracked()
            .contains_key(&form_version_id)
    {
        return;
    }
    leptos::task::spawn_local(async move {
        if let Ok(response) =
            gloo_net::http::Request::get(&format!("/api/form-versions/{form_version_id}/render"))
                .send()
                .await
        {
            if response.ok() {
                if let Ok(payload) = response.json::<RenderedForm>().await {
                    rendered_forms.update(|forms| {
                        forms.insert(form_version_id, payload);
                    });
                }
            }
        }
    });
}

#[cfg(not(feature = "hydrate"))]
fn load_rendered_form(_: String, _: RwSignal<BTreeMap<String, RenderedForm>>) {}

#[cfg(feature = "hydrate")]
fn load_dataset_for_edit(
    dataset_id: String,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    composition_mode: RwSignal<String>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match gloo_net::http::Request::get(&format!("/api/datasets/{dataset_id}"))
            .send()
            .await
        {
            Ok(response) if response.ok() => match response.json::<DatasetDefinition>().await {
                Ok(payload) => {
                    name.set(payload.name);
                    slug.set(payload.slug);
                    composition_mode.set(payload.composition_mode);
                    visibility_node_ids.set(
                        payload
                            .visibility_nodes
                            .into_iter()
                            .map(|node| node.node_id)
                            .collect(),
                    );
                    sources.set(
                        payload
                            .sources
                            .into_iter()
                            .map(|source| DatasetSourceDraft {
                                source_alias: source.source_alias,
                                form_id: source.form_id.unwrap_or_default(),
                                form_version_id: String::new(),
                                form_version_major: source.form_version_major,
                                selection_rule: source.selection_rule,
                            })
                            .collect(),
                    );
                    fields.set(
                        payload
                            .fields
                            .into_iter()
                            .map(|field| DatasetFieldDraft {
                                key: field.key,
                                label: field.label,
                                source_alias: field.source_alias,
                                source_field_key: field.source_field_key,
                            })
                            .collect(),
                    );
                }
                Err(_) => load_error.set(Some("Dataset edit data could not be read.".into())),
            },
            Ok(response) => load_error.set(Some(format!(
                "Dataset edit data failed with status {}.",
                response.status()
            ))),
            Err(_) => load_error.set(Some("Could not reach the dataset API.".into())),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
fn load_dataset_for_edit(
    _: String,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<BTreeSet<String>>,
    _: RwSignal<Vec<DatasetSourceDraft>>,
    _: RwSignal<Vec<DatasetFieldDraft>>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
#[allow(clippy::too_many_arguments)]
fn save_dataset(
    dataset_id: Option<String>,
    name: String,
    slug: String,
    composition_mode: String,
    visibility_node_ids: Vec<String>,
    mut sources: Vec<DatasetSourceDraft>,
    fields: Vec<DatasetFieldDraft>,
    save_error: RwSignal<Option<String>>,
    save_message: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        save_error.set(None);
        save_message.set(None);
        if composition_mode == "join" {
            for source in &mut sources {
                if source.selection_rule == "all" {
                    source.selection_rule = "latest".into();
                }
            }
        }
        let payload = DatasetPayload {
            name,
            slug,
            grain: "submission".into(),
            composition_mode,
            visibility_node_ids,
            sources: sources
                .into_iter()
                .filter(|source| {
                    !source.source_alias.trim().is_empty() && !source.form_id.is_empty()
                })
                .map(|source| DatasetSourcePayload {
                    source_alias: source.source_alias,
                    form_id: Some(source.form_id),
                    form_version_major: source.form_version_major,
                    selection_rule: source.selection_rule,
                })
                .collect(),
            fields: fields
                .into_iter()
                .enumerate()
                .filter(|(_, field)| {
                    !field.key.trim().is_empty()
                        && !field.label.trim().is_empty()
                        && !field.source_alias.trim().is_empty()
                        && !field.source_field_key.trim().is_empty()
                })
                .map(|(index, field)| DatasetFieldPayload {
                    key: field.key,
                    label: field.label,
                    source_alias: field.source_alias,
                    source_field_key: field.source_field_key,
                    position: index as i32,
                })
                .collect(),
        };
        let Ok(body) = serde_json::to_string(&payload) else {
            save_error.set(Some("Dataset payload could not be prepared.".into()));
            return;
        };
        let result: Result<serde_json::Value, String> = if let Some(dataset_id) = dataset_id {
            send_json_request(
                gloo_net::http::Request::put(&format!("/api/admin/datasets/{dataset_id}")),
                Some(body),
                "dataset update",
            )
            .await
        } else {
            send_json_request(
                gloo_net::http::Request::post("/api/admin/datasets"),
                Some(body),
                "dataset creation",
            )
            .await
        };
        match result {
            Ok(value) => {
                let id = value
                    .get("id")
                    .and_then(|value| value.as_str())
                    .unwrap_or_default()
                    .to_string();
                save_message.set(Some("Dataset saved.".into()));
                if !id.is_empty() {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href(&format!("/datasets/{id}"));
                    }
                }
            }
            Err(message) => save_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
fn save_dataset(
    _: Option<String>,
    _: String,
    _: String,
    _: String,
    _: Vec<String>,
    _: Vec<DatasetSourceDraft>,
    _: Vec<DatasetFieldDraft>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}
