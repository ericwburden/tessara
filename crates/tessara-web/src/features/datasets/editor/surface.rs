//! Dataset editor page surface.

use super::operations::catalog_after_operations;
use super::{
    DatasetEditorMessages, DatasetEditorState, DatasetIdentitySection, DatasetOperationSequence,
    DatasetRestrictionsEditor, DatasetSourcesEditor, DatasetSqlPreviewPanel,
    DatasetVisibilityEditor, install_dataset_editor_loaders, submit_dataset_editor,
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
    let final_fields = Signal::derive(move || {
        catalog_after_operations(
            state.initial_source.get(),
            state.forms.get(),
            state.rendered_forms.get(),
            state.operation_order.get(),
        )
    });

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
                        initial_source=state.initial_source
                        forms=state.forms
                        datasets=state.datasets
                        rendered_forms=state.rendered_forms
                        operation_order=state.operation_order
                    />
                    <DatasetOperationSequence
                        operation_order=state.operation_order
                        initial_source=state.initial_source
                        forms=state.forms
                        datasets=state.datasets
                        rendered_forms=state.rendered_forms
                        nodes=state.nodes
                        users=state.users
                    />
                    <DatasetRestrictionsEditor
                        fields=final_fields
                        restriction_internal_field_key=state.restriction_internal_field_key
                        restriction_restricted_field_key=state.restriction_restricted_field_key
                        restriction_confidential_field_key=state.restriction_confidential_field_key
                    />
                    <DatasetSqlPreviewPanel
                        dataset_id=dataset_id.clone()
                        name=state.name
                        slug=state.slug
                        visibility_node_ids=state.visibility_node_ids
                        initial_source=state.initial_source
                        operation_order=state.operation_order
                        restriction_internal_field_key=state.restriction_internal_field_key
                        restriction_restricted_field_key=state.restriction_restricted_field_key
                        restriction_confidential_field_key=state.restriction_confidential_field_key
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
                    <button class="button" type="submit" form="dataset-editor-form">
                        {if is_edit { "Save Dataset" } else { "Create Dataset" }}
                    </button>
                </div>
            </section>
        </AppShell>
    }
}
