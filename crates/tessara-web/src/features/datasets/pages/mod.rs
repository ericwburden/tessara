//! Route-level page composition for the Datasets feature.
//!
//! Keep Leptos page components that correspond directly to routes here; reusable widgets, API calls, and DTOs should live in sibling modules.

use std::collections::{BTreeMap, BTreeSet};

use leptos::portal::Portal;
use leptos::prelude::*;

use crate::types::route_params::{DatasetRouteParams, require_route_params};
use crate::ui::{AppShell, DataTable, EmptyState, PageHeader};
use crate::utils::text::{sentence_label, text_matches};
use icons::{Search, X};

use super::components::{DatasetDetailSurface, DatasetDirectoryTable, DatasetPreviewTable};
use super::editor::{
    DatasetExpressionChain, DatasetSqlPreviewPanel, ExpressionPreview, add_fields_from_source,
    confirm_action, field_metadata, find_version, first_published_version, join_key_option_label,
    join_key_options_for_source_index, operation_label, published_versions_for_form,
    resolved_form_version_id, source_field_options_with_selected, source_seed_key, version_label,
};
use super::expressions::is_join_operation;
use super::loaders::*;
use super::permissions::can_manage_datasets;
use super::types::*;
use super::validation::node_matches_visibility_query;

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
