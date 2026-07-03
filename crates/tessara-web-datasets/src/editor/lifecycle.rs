//! Loading lifecycle for the dataset editor surface.

use super::DatasetEditorState;
use crate::loaders::{
    DatasetEditLoadTargets, load_dataset_for_edit, load_dataset_revision_for_edit, load_datasets,
    load_forms, load_nodes, load_users,
};
use leptos::prelude::*;

pub(crate) fn install_dataset_editor_loaders(
    dataset_id: Option<String>,
    revision_id: Option<String>,
    state: DatasetEditorState,
) {
    Effect::new(move |_| {
        load_forms(state.forms, state.load_error);
        load_datasets(state.datasets, RwSignal::new(false), state.load_error);
        load_nodes(state.nodes, state.load_error);
        load_users(state.users, state.load_error);
        if let (Some(dataset_id), Some(revision_id)) = (dataset_id.clone(), revision_id.clone()) {
            load_dataset_revision_for_edit(
                dataset_id,
                revision_id,
                DatasetEditLoadTargets {
                    name: state.name,
                    slug: state.slug,
                    visibility_node_ids: state.visibility_node_ids,
                    initial_source: state.initial_source,
                    operation_order: state.operation_order,
                    force_new_major_version: state.force_new_major_version,
                    rendered_forms: state.rendered_forms,
                    restriction_internal_field_key: state.restriction_internal_field_key,
                    restriction_restricted_field_key: state.restriction_restricted_field_key,
                    restriction_confidential_field_key: state.restriction_confidential_field_key,
                    sql_preview: state.sql_preview,
                    load_error: state.load_error,
                    editor_ready: state.editor_ready,
                },
            );
        } else if let Some(dataset_id) = dataset_id.clone() {
            load_dataset_for_edit(
                dataset_id,
                DatasetEditLoadTargets {
                    name: state.name,
                    slug: state.slug,
                    visibility_node_ids: state.visibility_node_ids,
                    initial_source: state.initial_source,
                    operation_order: state.operation_order,
                    force_new_major_version: state.force_new_major_version,
                    rendered_forms: state.rendered_forms,
                    restriction_internal_field_key: state.restriction_internal_field_key,
                    restriction_restricted_field_key: state.restriction_restricted_field_key,
                    restriction_confidential_field_key: state.restriction_confidential_field_key,
                    sql_preview: state.sql_preview,
                    load_error: state.load_error,
                    editor_ready: state.editor_ready,
                },
            );
        } else {
            state.editor_ready.set(true);
        }
    });
}
