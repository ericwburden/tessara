//! Dataset detail surface and preview table.

use std::collections::BTreeMap;

use super::super::super::display::visibility_label;
use super::super::super::loaders::{load_account, load_dataset_detail, load_dataset_table};
use super::super::super::permissions::can_manage_datasets;
use super::super::super::types::*;
use super::summary::{MetricCard, tab_class};
use super::tables::{DatasetFieldsTable, DatasetSourcesTable, DatasetSqlPanel};
use crate::text::sentence_label;
use icons::{ChevronDown, ChevronRight, Database, FileText, X};
use leptos::portal::Portal;
use leptos::prelude::*;
use tessara_module_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, EmptyState,
    InteractiveDataTable, InteractiveTableColumn, InteractiveTableRow, PageHeader,
};

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
        <section class="route-panel datasets-page">
            {move || {
                if is_loading.get() {
                    view! { <EmptyState title="Loading dataset" message="Fetching dataset definition."/> }.into_any()
                } else if let Some(message) = load_error.get() {
                    view! { <EmptyState title="Dataset unavailable" message=message/> }.into_any()
                } else if let Some(loaded) = dataset.get() {
                    let edit_href = format!("/datasets/{}/edit", loaded.id);
                    let revisions_href = format!("/datasets/{}/revisions", loaded.id);
                    let tab_dataset = loaded.clone();
                    let visibility_nodes = loaded.visibility_nodes.clone();
                    view! {
                        <Breadcrumb>
                            <BreadcrumbItem>
                                <BreadcrumbLink href="/datasets">"Datasets"</BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator/>
                            <BreadcrumbItem>
                                <BreadcrumbPage>"Dataset Detail"</BreadcrumbPage>
                            </BreadcrumbItem>
                        </Breadcrumb>
                        <PageHeader title="Dataset Detail">
                            {move || if can_manage() && !edit {
                                view! {
                                    <div class="button-row">
                                        <a class="button button--secondary" href=revisions_href.clone()>"Revision History"</a>
                                        <a class="button button--secondary" href=edit_href.clone()>"Edit Dataset"</a>
                                    </div>
                                }.into_any()
                            } else if edit {
                                view! { <a class="button button--secondary" href=revisions_href.clone()>"Revision History"</a> }.into_any()
                            } else {
                                view! { <span></span> }.into_any()
                            }}
                        </PageHeader>
                        <h2>{loaded.name.clone()}</h2>
                        <section class="dataset-detail-summary">
                            <MetricCard label="Slug" value=loaded.slug.clone()/>
                            <MetricCard label="Grain" value=sentence_label(&loaded.grain)/>
                            <MetricCard label="Tags" value=tag_summary(&loaded.tags)/>
                            <MetricCard label="Provenance" value=provenance_summary(&loaded.provenance)/>
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
                                <button class=tab_class(active_tab, "tags") type="button" on:click=move |_| active_tab.set("tags".into())>"Tags"</button>
                                <button class=tab_class(active_tab, "provenance") type="button" on:click=move |_| active_tab.set("provenance".into())>"Provenance"</button>
                                <button class=tab_class(active_tab, "sql") type="button" on:click=move |_| active_tab.set("sql".into())>"SQL"</button>
                            </div>
                            {move || if active_tab.get() == "preview" {
                                view! { <DatasetPreviewTable dataset=tab_dataset.clone() table=table.get() error=table_error.get()/> }.into_any()
                            } else if active_tab.get() == "sources" {
                                view! { <DatasetSourcesTable sources=tab_dataset.sources.clone()/> }.into_any()
                            } else if active_tab.get() == "tags" {
                                view! { <DatasetTagsPanel tags=tab_dataset.tags.clone()/> }.into_any()
                            } else if active_tab.get() == "provenance" {
                                view! { <DatasetProvenancePanel lineage=tab_dataset.lineage.clone()/> }.into_any()
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
    }
}

#[component]
fn DatasetTagsPanel(tags: Vec<String>) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            {if tags.is_empty() {
                view! { <p class="muted">"No catalog tags are assigned."</p> }.into_any()
            } else {
                view! {
                    <div class="dataset-tag-list">
                        {tags.into_iter().map(|tag| view! {
                            <span class="dataset-tag-chip">{tag}</span>
                        }).collect_view()}
                    </div>
                }.into_any()
            }}
        </section>
    }
}

#[component]
fn DatasetProvenancePanel(lineage: DatasetLineageNode) -> impl IntoView {
    view! {
        <section class="route-panel__section">
            <div class="dataset-lineage-tree" role="tree">
                <LineageTreeNode node=lineage is_root=true/>
            </div>
        </section>
    }
}

#[component]
fn LineageTreeNode(node: DatasetLineageNode, is_root: bool) -> impl IntoView {
    let children = node.children;
    let has_children = !children.is_empty();
    let expanded = RwSignal::new(true);
    let source_type = node.source_type;
    let version_label = node.version_label;
    let id = node.id;
    let name = node.name;
    let href = match source_type.as_str() {
        "dataset" if !is_root => Some(format!("/datasets/{id}")),
        "form" => Some(format!("/forms/{id}")),
        _ => None,
    };
    let icon = if source_type == "dataset" {
        view! { <Database class="dataset-lineage-node__type-icon"/> }.into_any()
    } else {
        view! { <FileText class="dataset-lineage-node__type-icon"/> }.into_any()
    };

    view! {
        <div class=if is_root { "dataset-lineage-node dataset-lineage-node--root" } else { "dataset-lineage-node" } role="treeitem" aria-expanded=move || if has_children { Some(expanded.get().to_string()) } else { None }>
            <div class="dataset-lineage-node__row">
                {if has_children {
                    view! {
                        <button
                            class="dataset-lineage-node__toggle"
                            type="button"
                            aria-label=move || if expanded.get() { "Collapse lineage branch" } else { "Expand lineage branch" }
                            on:click=move |_| expanded.update(|value| *value = !*value)
                        >
                            {move || if expanded.get() {
                                view! { <ChevronDown class="dataset-lineage-node__toggle-icon"/> }.into_any()
                            } else {
                                view! { <ChevronRight class="dataset-lineage-node__toggle-icon"/> }.into_any()
                            }}
                        </button>
                    }.into_any()
                } else {
                    view! { <span class="dataset-lineage-node__toggle-spacer" aria-hidden="true"></span> }.into_any()
                }}
                <span class="dataset-lineage-node__icon" aria-hidden="true">{icon}</span>
                <span class="dataset-lineage-node__label">
                    {if let Some(href) = href {
                        view! { <a class="data-table__primary-link" href=href>{name.clone()}</a> }.into_any()
                    } else {
                        view! { <span>{name.clone()}</span> }.into_any()
                    }}
                </span>
                {version_label.map(|label| view! { <span class="dataset-lineage-node__version">{label}</span> })}
            </div>
            {move || if has_children && expanded.get() {
                view! {
                    <div class="dataset-lineage-node__children" role="group">
                        {children.clone().into_iter().map(|child| view! {
                            <LineageTreeNode node=child is_root=false/>
                        }).collect_view()}
                    </div>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }.into_any()
}

fn tag_summary(tags: &[String]) -> String {
    if tags.is_empty() {
        "No tags".into()
    } else {
        tags.join(", ")
    }
}

fn provenance_summary(provenance: &DatasetProvenanceSummary) -> String {
    let form_count = provenance.forms.len();
    let dataset_count = provenance.datasets.len();
    match (form_count, dataset_count) {
        (0, 0) => "No direct sources".into(),
        (forms, 0) => format!("{forms} form source{}", plural_suffix(forms)),
        (0, datasets) => format!("{datasets} dataset source{}", plural_suffix(datasets)),
        (forms, datasets) => format!(
            "{forms} form source{}, {datasets} dataset source{}",
            plural_suffix(forms),
            plural_suffix(datasets)
        ),
    }
}

fn plural_suffix(count: usize) -> &'static str {
    if count == 1 { "" } else { "s" }
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
        return view! { <EmptyState title="Preview unavailable" message=message/> }.into_any();
    }
    let Some(table) = table else {
        return view! { <EmptyState title="Loading preview" message="Fetching dataset preview rows."/> }.into_any();
    };
    if table.rows.is_empty() {
        return view! { <EmptyState title="No preview rows" message="This dataset has no submitted response rows available for preview."/> }.into_any();
    }
    let fields = detail_output_fields(&dataset);
    let columns = fields
        .iter()
        .map(|field| {
            InteractiveTableColumn::new(
                field.key.clone(),
                field.label.clone(),
                tessara_module_ui::InteractiveTableDataType::from_field_type(&field.field_type),
            )
        })
        .collect::<Vec<_>>();
    let rows = table
        .rows
        .into_iter()
        .map(|row| {
            let values = fields
                .iter()
                .map(|field| {
                    let value = row
                        .values
                        .get(&field.key)
                        .and_then(|value| value.clone())
                        .unwrap_or_default();
                    (
                        field.key.clone(),
                        display_preview_value(&field.field_type, &value),
                    )
                })
                .collect::<BTreeMap<_, _>>();
            InteractiveTableRow::new(row.submission_id, values)
        })
        .collect::<Vec<_>>();

    view! {
        <section class="route-panel__section">
            <InteractiveDataTable
                columns
                rows
                search_label="Search preview rows"
                search_placeholder="Search preview"
                item_label="preview rows"
                empty_message="No preview rows match the current table controls."
            />
        </section>
    }
    .into_any()
}

fn detail_output_fields(dataset: &DatasetDefinition) -> Vec<DatasetFieldDefinition> {
    if dataset.output_fields.is_empty() {
        dataset.fields.clone()
    } else {
        dataset.output_fields.clone()
    }
}

fn display_preview_value(field_type: &str, value: &str) -> String {
    if field_type == "number"
        && value.contains('.')
        && let Ok(number) = value.parse::<f64>()
        && number.is_finite()
    {
        return format!("{number:.2}");
    }
    value.to_string()
}
