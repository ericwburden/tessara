//! Dataset editor page surface.

use super::{
    DatasetAggregationEditor, DatasetEditorMessages, DatasetEditorState, DatasetFieldsEditor,
    DatasetIdentitySection, DatasetSourcesEditor, DatasetSqlPreviewPanel, DatasetVisibilityEditor,
    install_dataset_editor_loaders, submit_dataset_editor,
};
use crate::ui::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    PageHeader,
};
use leptos::prelude::*;

#[component]
pub(crate) fn DatasetEditorSurface(dataset_id: Option<String>) -> impl IntoView {
    let is_edit = dataset_id.is_some();
    let title = if is_edit {
        "Edit Dataset"
    } else {
        "Create Dataset"
    };
    let state = DatasetEditorState::new();
    install_dataset_editor_loaders(dataset_id.clone(), state);
    let save_dataset_id = dataset_id.clone();
    let preview_dataset_id = dataset_id.clone();

    view! {
        <AppShell active_route="datasets" title=title>
            <section class="route-panel datasets-page">
                <Breadcrumb>
                    <BreadcrumbItem>
                        <BreadcrumbLink href="/datasets">"Datasets"</BreadcrumbLink>
                    </BreadcrumbItem>
                    <BreadcrumbSeparator/>
                    <BreadcrumbItem>
                        <BreadcrumbPage>{title}</BreadcrumbPage>
                    </BreadcrumbItem>
                </Breadcrumb>
                <PageHeader title/>
                <DatasetEditorMessages
                    load_error=state.load_error
                    save_error=state.save_error
                    save_message=state.save_message
                />
                <form id="dataset-editor-form" class="dataset-editor" on:submit=move |event| {
                    event.prevent_default();
                    submit_dataset_editor(save_dataset_id.clone(), state);
                }>
                    <DatasetIdentitySection name=state.name slug=state.slug/>
                    <DatasetSourcesEditor
                        sources=state.sources
                        expression=state.expression
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
                        datasets=state.datasets
                        rendered_forms=state.rendered_forms
                    />
                    <DatasetAggregationEditor
                        fields=state.fields
                        aggregation=state.aggregation
                    />
                    <DatasetSqlPreviewPanel
                        dataset_id=dataset_id.clone()
                        name=state.name
                        slug=state.slug
                        visibility_node_ids=state.visibility_node_ids
                        sources=state.sources
                        expression=state.expression
                        fields=state.fields
                        aggregation=state.aggregation
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
                        expanded_node_ids=state.visibility_expanded_node_ids
                    />
                </form>
                <div class="form-actions">
                    {move || preview_dataset_id.clone().map(|id| {
                        let preview_url = format!("/datasets/{id}");
                        view! {
                            <button
                                class="button button--secondary"
                                type="button"
                                onclick=format!("window.location.href='/datasets/{id}';")
                                on:click=move |_| {
                                    if let Some(window) = web_sys::window() {
                                        let _ = window.location().set_href(&preview_url);
                                    }
                                }
                            >
                                "Open Preview"
                            </button>
                        }
                    })}
                    <button class="button" type="submit" form="dataset-editor-form">
                        {if is_edit { "Save Dataset" } else { "Create Dataset" }}
                    </button>
                </div>
            </section>
        </AppShell>
    }
}
