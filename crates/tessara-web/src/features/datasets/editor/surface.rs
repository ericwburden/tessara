//! Dataset editor page surface.

use super::{DatasetFieldsEditor, DatasetSourcesEditor, DatasetSqlPreviewPanel};
use crate::features::datasets::loaders::*;
use crate::features::datasets::types::*;
use crate::features::datasets::validation::node_matches_visibility_query;
use crate::ui::{AppShell, DataTable, PageHeader};
use crate::utils::text::sentence_label;
use icons::Search;
use leptos::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

#[component]
/// Renders the dataset editor surface view.
pub(crate) fn DatasetEditorSurface(dataset_id: Option<String>) -> impl IntoView {
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
