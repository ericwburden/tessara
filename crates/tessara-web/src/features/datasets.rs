use std::collections::BTreeMap;
use std::sync::Arc;

use leptos::prelude::*;
use serde::Deserialize;

#[cfg(feature = "hydrate")]
use crate::features::native_runtime::get_json;
use crate::features::native_shell::{BreadcrumbItem, MetadataStrip, NativePage, PageHeader, Panel};
use crate::infra::routing::{DatasetRouteParams, require_route_params};
#[cfg(feature = "hydrate")]
use wasm_bindgen_futures::spawn_local;

#[derive(Clone, Deserialize)]
struct DatasetSummary {
    id: String,
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    source_count: i64,
    field_count: i64,
}

#[derive(Clone, Deserialize)]
struct DatasetDefinition {
    id: String,
    name: String,
    slug: String,
    grain: String,
    composition_mode: String,
    sources: Vec<DatasetSourceDefinition>,
    fields: Vec<DatasetFieldDefinition>,
    reports: Vec<DatasetReportLink>,
}

#[derive(Clone, Deserialize)]
struct DatasetSourceDefinition {
    source_alias: String,
    form_id: Option<String>,
    form_name: Option<String>,
    form_version_major: Option<i32>,
    selection_rule: String,
    position: i32,
}

#[derive(Clone, Deserialize)]
struct DatasetFieldDefinition {
    key: String,
    label: String,
    source_alias: String,
    source_field_key: String,
    field_type: String,
    position: i32,
}

#[derive(Clone, Deserialize)]
struct DatasetReportLink {
    id: String,
    name: String,
}

#[derive(Clone, Deserialize)]
struct DatasetTable {
    rows: Vec<DatasetTableRow>,
}

#[derive(Clone, Deserialize)]
struct DatasetTableRow {
    submission_id: String,
    node_name: String,
    source_alias: String,
    values: BTreeMap<String, Option<String>>,
}

#[component]
pub fn DatasetsPage() -> impl IntoView {
    let datasets = RwSignal::new(Vec::<DatasetSummary>::new());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        spawn_local(async move {
            loading.set(true);
            match get_json::<Vec<DatasetSummary>>("/api/datasets").await {
                Ok(items) => {
                    datasets.set(items);
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Datasets"
            description="Tessara datasets list screen."
            page_key="dataset-list"
            active_route="datasets"
            workspace_label="Internal Area"
            required_capability="datasets:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::current("Datasets"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Readable Dataset Directory"
                description="Browse readable dataset definitions and inspect source and field structure from the shared shell."
                actions=Arc::new(|| {
                    view! {
                        <a class="button-link button is-light" href="/app/administration">"Open Administration"</a>
                    }
                    .into_any()
                })
            />
            <MetadataStrip items=vec![
                ("Mode", "Directory".into()),
                ("Surface", "Internal analytics".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel
                title="Dataset Directory"
                description="Dataset definitions remain internal-facing, but they should be readable without dropping into the legacy builder for every inspection task."
            >
                <div id="dataset-list" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading dataset definitions..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }

                            let items = datasets.get();
                            if items.is_empty() {
                                return view! { <p class="muted">"No dataset definitions are readable yet."</p> }.into_any();
                            }

                            view! {
                                {items
                                    .into_iter()
                                    .map(|dataset| {
                                        let detail_href = format!("/app/datasets/{}", dataset.id);
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{dataset.name}</h4>
                                                <p class="muted">{format!("/{}", dataset.slug)}</p>
                                                <p>{format!("{} grain · {}", dataset.grain, dataset.composition_mode)}</p>
                                                <p class="muted">{format!("{} sources · {} fields", dataset.source_count, dataset.field_count)}</p>
                                                <div class="actions">
                                                    <a class="button-link" href=detail_href>"View"</a>
                                                </div>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any()
                        }}
                    </Show>
                </div>
            </Panel>
        </NativePage>
    }
}

#[component]
pub fn DatasetDetailPage() -> impl IntoView {
    let DatasetRouteParams { dataset_id } = require_route_params();
    let dataset = RwSignal::new(None::<DatasetDefinition>);
    let preview = RwSignal::new(None::<DatasetTable>);
    let loading = RwSignal::new(true);
    let preview_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let preview_error = RwSignal::new(None::<String>);
    let record_id = dataset_id.clone();
    let _record_id_for_preview = record_id.clone();

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let dataset_id = dataset_id.clone();
        spawn_local(async move {
            loading.set(true);
            match get_json::<DatasetDefinition>(&format!("/api/datasets/{dataset_id}")).await {
                Ok(item) => {
                    dataset.set(Some(item));
                    error.set(None);
                }
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    #[cfg(feature = "hydrate")]
    Effect::new(move |_| {
        let dataset_id = _record_id_for_preview.clone();
        spawn_local(async move {
            preview_loading.set(true);
            match get_json::<DatasetTable>(&format!("/api/datasets/{dataset_id}/table")).await {
                Ok(table) => {
                    preview.set(Some(table));
                    preview_error.set(None);
                }
                Err(message) => preview_error.set(Some(message)),
            }
            preview_loading.set(false);
        });
    });

    view! {
        <NativePage
            title="Dataset Detail"
            description="Inspect a Tessara dataset definition."
            page_key="dataset-detail"
            active_route="datasets"
            workspace_label="Internal Area"
            record_id=record_id.clone()
            required_capability="datasets:read"
            breadcrumbs=vec![
                BreadcrumbItem::link("Home", "/app"),
                BreadcrumbItem::link("Datasets", "/app/datasets"),
                BreadcrumbItem::current("Dataset Detail"),
            ]
        >
            <PageHeader
                eyebrow="Internal Area"
                title="Dataset Detail"
                description="Review the selected dataset definition, its source structure, and a preview of readable rows when available."
                actions=Arc::new(|| {
                    view! {
                        <a class="button-link button is-light" href="/app/datasets">"Back to List"</a>
                        <a class="button-link button is-light" href="/app/administration">"Open Administration"</a>
                    }
                    .into_any()
                })
            />
            <MetadataStrip items=vec![
                ("Mode", "Detail".into()),
                ("Surface", "Dataset definition".into()),
                ("State", "Native SSR shell".into()),
            ]/>
            <Panel
                title="Dataset Definition"
                description="Identity, source coverage, and related reports appear here."
            >
                <div id="dataset-detail" class="record-list">
                    <Show
                        when=move || !loading.get()
                        fallback=|| view! { <p class="muted">"Loading dataset detail..."</p> }
                    >
                        {move || {
                            if let Some(message) = error.get() {
                                return view! { <p class="muted">{message}</p> }.into_any();
                            }

                            match dataset.get() {
                                Some(dataset) => {
                                    let report_items = if dataset.reports.is_empty() {
                                        view! { <li class="muted">"No reports currently reference this dataset."</li> }.into_any()
                                    } else {
                                        view! {
                                            {dataset.reports
                                                .iter()
                                                .map(|report| {
                                                    view! {
                                                        <li>
                                                            <a href=format!("/app/reports/{}", report.id)>{report.name.clone()}</a>
                                                        </li>
                                                    }
                                                })
                                                .collect_view()}
                                        }.into_any()
                                    };

                                    view! {
                                        <article class="record-card">
                                            <h4>{dataset.name.clone()}</h4>
                                            <p class="muted">{format!("/{}", dataset.slug)}</p>
                                            <div class="detail-grid">
                                                <p><strong>"Grain:"</strong> {format!(" {}", dataset.grain)}</p>
                                                <p><strong>"Composition:"</strong> {format!(" {}", dataset.composition_mode)}</p>
                                                <p><strong>"Sources:"</strong> {format!(" {}", dataset.sources.len())}</p>
                                                <p><strong>"Fields:"</strong> {format!(" {}", dataset.fields.len())}</p>
                                            </div>
                                            <section class="detail-section">
                                                <h4>"Related Reports"</h4>
                                                <ul class="app-list">{report_items}</ul>
                                            </section>
                                        </article>
                                    }.into_any()
                                }
                                None => view! { <p class="muted">"Dataset detail is unavailable."</p> }.into_any(),
                            }
                        }}
                    </Show>
                </div>
            </Panel>
            <Panel
                title="Source Definitions"
                description="Dataset sources stay visible so downstream record shape is inspectable."
            >
                <div id="dataset-source-summary" class="record-list">
                    {move || {
                        if loading.get() {
                            return view! { <p class="muted">"Loading dataset sources..."</p> }.into_any();
                        }
                        if error.get().is_some() {
                            return view! { <p class="muted">"Source detail is unavailable while the dataset fails to load."</p> }.into_any();
                        }

                        match dataset.get() {
                            Some(dataset) if dataset.sources.is_empty() => {
                                view! { <p class="muted">"No dataset sources are configured."</p> }.into_any()
                            }
                            Some(dataset) => view! {
                                {dataset.sources
                                    .into_iter()
                                    .map(|source| {
                                        let form_label = source
                                            .form_name
                                            .clone()
                                            .unwrap_or_else(|| "Compatibility group source".into());
                                        let form_version = source
                                            .form_version_major
                                            .map(|version| format!("Version {version}"))
                                            .unwrap_or_else(|| "Latest compatible version".into());
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{source.source_alias}</h4>
                                                <p>{form_label}</p>
                                                <p class="muted">{form_version}</p>
                                                <p class="muted">{format!("Selection: {} · Order {}", source.selection_rule, source.position)}</p>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any(),
                            None => view! { <p class="muted">"Source detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </div>
            </Panel>
            <Panel
                title="Field Definitions"
                description="Semantic fields remain visible so dataset shape is inspectable without opening the builder."
            >
                <div id="dataset-field-summary" class="record-list">
                    {move || {
                        if loading.get() {
                            return view! { <p class="muted">"Loading dataset fields..."</p> }.into_any();
                        }
                        if error.get().is_some() {
                            return view! { <p class="muted">"Field detail is unavailable while the dataset fails to load."</p> }.into_any();
                        }

                        match dataset.get() {
                            Some(dataset) if dataset.fields.is_empty() => {
                                view! { <p class="muted">"No dataset fields are configured."</p> }.into_any()
                            }
                            Some(dataset) => view! {
                                {dataset.fields
                                    .into_iter()
                                    .map(|field| {
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{field.label}</h4>
                                                <p>{field.key}</p>
                                                <p class="muted">{format!("{} · {} · {}", field.source_alias, field.source_field_key, field.field_type)}</p>
                                                <p class="muted">{format!("Display order {}", field.position)}</p>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any(),
                            None => view! { <p class="muted">"Field detail is unavailable."</p> }.into_any(),
                        }
                    }}
                </div>
            </Panel>
            <Panel
                title="Readable Preview"
                description="A small dataset preview stays attached to the detail route when rows are executable."
            >
                <div id="dataset-preview" class="record-list">
                    {move || {
                        if preview_loading.get() {
                            return view! { <p class="muted">"Loading dataset preview..."</p> }.into_any();
                        }
                        if let Some(message) = preview_error.get() {
                            return view! { <p class="muted">{message}</p> }.into_any();
                        }

                        match preview.get() {
                            Some(table) if table.rows.is_empty() => {
                                view! { <p class="muted">"No readable dataset rows are available yet."</p> }.into_any()
                            }
                            Some(table) => view! {
                                {table.rows
                                    .into_iter()
                                    .take(3)
                                    .map(|row| {
                                        let values = row
                                            .values
                                            .into_iter()
                                            .take(4)
                                            .map(|(key, value)| {
                                                let display = value.unwrap_or_else(|| "—".into());
                                                view! { <li>{format!("{key}: {display}")}</li> }
                                            })
                                            .collect_view();
                                        view! {
                                            <article class="record-card compact-record-card">
                                                <h4>{row.node_name}</h4>
                                                <p>{format!("{} · {}", row.source_alias, row.submission_id)}</p>
                                                <ul class="app-list">{values}</ul>
                                            </article>
                                        }
                                    })
                                    .collect_view()}
                            }.into_any(),
                            None => view! { <p class="muted">"Dataset preview is unavailable."</p> }.into_any(),
                        }
                    }}
                </div>
            </Panel>
        </NativePage>
    }
}
