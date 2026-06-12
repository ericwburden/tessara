//! Dataset editor page surface.

use super::{
    DatasetEditorMessages, DatasetFieldsEditor, DatasetSourcesEditor, DatasetSqlPreviewPanel,
    DatasetVisibilityEditor,
};
use crate::features::datasets::loaders::*;
use crate::features::datasets::types::*;
use crate::ui::{AppShell, PageHeader};
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
                <DatasetEditorMessages load_error save_error save_message/>
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
                    <DatasetVisibilityEditor nodes visibility_node_ids visibility_search/>
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
