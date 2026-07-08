//! Dataset editor page surface.

use super::operations::catalog_after_operations;
use super::{
    DatasetEditorMessages, DatasetEditorState, DatasetIdentitySection, DatasetOperationSequence,
    DatasetRestrictionsEditor, DatasetSourcesEditor, DatasetSqlPreviewPanel,
    DatasetVisibilityEditor, install_dataset_editor_loaders, submit_dataset_editor,
};
use leptos::prelude::*;
use tessara_web_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator, PageHeader,
};

#[component]
pub(crate) fn DatasetEditorSurface(
    dataset_id: Option<String>,
    revision_id: Option<String>,
) -> impl IntoView {
    let is_edit = dataset_id.is_some();
    let is_revision_edit = revision_id.is_some();
    let title = if is_revision_edit {
        "Edit Revision"
    } else if is_edit {
        "Edit Dataset"
    } else {
        "Create Dataset"
    };
    let state = DatasetEditorState::new(!is_edit);
    install_dataset_editor_loaders(dataset_id.clone(), revision_id.clone(), state);
    let save_dataset_id = dataset_id.clone();
    let detail_href = dataset_id.as_ref().map(|id| format!("/datasets/{id}"));
    let history_href = dataset_id
        .as_ref()
        .map(|id| format!("/datasets/{id}/revisions"));
    let revision_href = dataset_id
        .as_ref()
        .zip(revision_id.as_ref())
        .map(|(dataset_id, revision_id)| format!("/datasets/{dataset_id}/revisions/{revision_id}"));
    let final_fields = Signal::derive(move || {
        catalog_after_operations(
            state.initial_source.get(),
            state.datasets.get(),
            state.forms.get(),
            state.rendered_forms.get(),
            state.operation_order.get(),
        )
    });
    let available_tags = Signal::derive(move || {
        dataset_tag_options(
            state.datasets.get(),
            state.tags.get(),
            state.known_tags.get(),
        )
    });

    view! {
        <section class="route-panel datasets-page">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/datasets">"Datasets"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {detail_href.map(|href| {
                    view! {
                        <BreadcrumbItem>
                            <BreadcrumbLink href=href>"Dataset Detail"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator/>
                    }
                })}
                {history_href.filter(|_| is_revision_edit).map(|href| {
                    view! {
                        <BreadcrumbItem>
                            <BreadcrumbLink href=href>"Revision History"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator/>
                    }
                })}
                {revision_href.map(|href| {
                    view! {
                        <BreadcrumbItem>
                            <BreadcrumbLink href=href>"Dataset Revision"</BreadcrumbLink>
                        </BreadcrumbItem>
                        <BreadcrumbSeparator/>
                    }
                })}
                <BreadcrumbItem>
                    <BreadcrumbPage>{title}</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <PageHeader title/>
            <DatasetEditorMessages
                load_error=state.load_error
                save_error=state.save_error
                save_message=state.save_message
                editor_ready=state.editor_ready
            />
            <form id="dataset-editor-form" class="dataset-editor" on:submit=move |event| {
                event.prevent_default();
                if state.editor_ready.get() {
                    submit_dataset_editor(save_dataset_id.clone(), state);
                }
            }>
                <fieldset class="dataset-editor__fieldset" disabled=move || !state.editor_ready.get()>
                    <DatasetIdentitySection
                        dataset_id=dataset_id.clone()
                        name=state.name
                        slug=state.slug
                        tags=state.tags
                        known_tags=state.known_tags
                        tag_input=state.tag_input
                        available_tags=available_tags
                        save_error=state.save_error
                        save_message=state.save_message
                    />
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
                </fieldset>
            </form>
            <div class="form-actions">
                <button class="button" type="submit" form="dataset-editor-form" disabled=move || !state.editor_ready.get()>
                    {move || {
                        if !state.editor_ready.get() {
                            "Loading..."
                        } else if is_revision_edit {
                            "Save Revision"
                        } else if is_edit {
                            "Save Dataset"
                        } else {
                            "Create Dataset"
                        }
                    }}
                </button>
            </div>
        </section>
    }
}

fn dataset_tag_options(
    datasets: Vec<super::super::types::DatasetSummary>,
    selected_tags: Vec<String>,
    known_tags: Vec<String>,
) -> Vec<String> {
    let mut tags = datasets
        .into_iter()
        .flat_map(|dataset| dataset.tags)
        .chain(selected_tags)
        .chain(known_tags)
        .map(|tag| tag.trim().to_string())
        .filter(|tag| !tag.is_empty())
        .collect::<Vec<_>>();
    tags.sort_by_key(|tag| tag.to_ascii_lowercase());
    tags.dedup_by(|left, right| left.eq_ignore_ascii_case(right));
    tags
}

#[cfg(test)]
mod tests {
    use super::dataset_tag_options;

    #[test]
    fn dataset_tag_options_include_catalog_and_selected_tags_once() {
        assert_eq!(
            dataset_tag_options(
                Vec::new(),
                vec!["Demo".into(), "demo".into()],
                vec!["New".into()]
            ),
            vec!["Demo".to_string(), "New".to_string()]
        );
    }
}
