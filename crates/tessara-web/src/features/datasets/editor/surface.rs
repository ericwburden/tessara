//! Dataset editor page surface.

use super::{
    DatasetEditorMessages, DatasetEditorState, DatasetFieldsEditor, DatasetIdentitySection,
    DatasetSourcesEditor, DatasetSqlPreviewPanel, DatasetVisibilityEditor,
};
use crate::features::datasets::actions::save_dataset;
use crate::features::datasets::loaders::*;
use crate::ui::{AppShell, PageHeader};
use leptos::prelude::*;

#[component]
/// Renders the dataset editor surface view.
pub(crate) fn DatasetEditorSurface(dataset_id: Option<String>) -> impl IntoView {
    let is_edit = dataset_id.is_some();
    let title = if is_edit {
        "Edit Dataset"
    } else {
        "Create Dataset"
    };
    let state = DatasetEditorState::new();

    Effect::new({
        let dataset_id = dataset_id.clone();
        move |_| {
            load_forms(state.forms, state.load_error);
            load_datasets(state.datasets, RwSignal::new(false), state.load_error);
            load_nodes(state.nodes, state.load_error);
            if let Some(dataset_id) = dataset_id.clone() {
                load_dataset_for_edit(
                    dataset_id.clone(),
                    state.name,
                    state.slug,
                    state.composition_mode,
                    state.visibility_node_ids,
                    state.sources,
                    state.fields,
                    state.join_left_key,
                    state.join_right_key,
                    state.sql_preview,
                    state.load_error,
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
                <DatasetEditorMessages
                    load_error=state.load_error
                    save_error=state.save_error
                    save_message=state.save_message
                />
                <form class="dataset-editor" on:submit=move |event| {
                    event.prevent_default();
                    save_dataset(
                        save_dataset_id.clone(),
                        state.name.get(),
                        state.slug.get(),
                        state.composition_mode.get(),
                        state.visibility_node_ids.get().into_iter().collect(),
                        state.sources.get(),
                        state.fields.get(),
                        state.join_left_key.get(),
                        state.join_right_key.get(),
                        state.save_error,
                        state.save_message,
                    );
                }>
                    <DatasetIdentitySection name=state.name slug=state.slug/>
                    <DatasetSourcesEditor
                        sources=state.sources
                        forms=state.forms
                        datasets=state.datasets
                        rendered_forms=state.rendered_forms
                        composition_mode=state.composition_mode
                        fields=state.fields
                        join_left_key=state.join_left_key
                        join_right_key=state.join_right_key
                        designer_selection=state.designer_selection
                        designer_sheet_open=state.designer_sheet_open
                        auto_seeded_sources=state.auto_seeded_sources
                    />
                    <DatasetFieldsEditor
                        fields=state.fields
                        sources=state.sources
                        forms=state.forms
                        rendered_forms=state.rendered_forms
                        designer_selection=state.designer_selection
                        designer_sheet_open=state.designer_sheet_open
                    />
                    <DatasetSqlPreviewPanel
                        dataset_id=dataset_id.clone()
                        name=state.name
                        slug=state.slug
                        composition_mode=state.composition_mode
                        visibility_node_ids=state.visibility_node_ids
                        sources=state.sources
                        fields=state.fields
                        join_left_key=state.join_left_key
                        join_right_key=state.join_right_key
                        sql_preview=state.sql_preview
                        sql_preview_error=state.sql_preview_error
                        expanded=state.sql_preview_expanded
                    />
                    <DatasetVisibilityEditor
                        nodes=state.nodes
                        visibility_node_ids=state.visibility_node_ids
                        visibility_search=state.visibility_search
                    />
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
