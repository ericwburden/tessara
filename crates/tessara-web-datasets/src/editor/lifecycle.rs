//! Loading lifecycle for the dataset editor surface.

use super::DatasetEditorState;
use crate::bootstrap::dataset_route_bootstrap;
use crate::loaders::{
    DatasetEditLoadTargets, load_dataset_for_edit, load_dataset_revision_for_edit, load_datasets,
    load_forms, load_nodes, load_users, seed_dataset_for_edit, seed_dataset_revision_for_edit,
};
use leptos::prelude::*;

pub(crate) fn install_dataset_editor_loaders(
    dataset_id: Option<String>,
    revision_id: Option<String>,
    state: DatasetEditorState,
) {
    let seeded_from_bootstrap = seed_dataset_editor_from_bootstrap(state);
    Effect::new(move |_| {
        if seeded_from_bootstrap {
            return;
        }
        load_forms(state.forms, state.load_error);
        load_datasets(state.datasets, RwSignal::new(false), state.load_error);
        load_nodes(state.nodes, state.load_error);
        load_users(state.users, state.load_error);
        if let (Some(dataset_id), Some(revision_id)) = (dataset_id.clone(), revision_id.clone()) {
            load_dataset_revision_for_edit(dataset_id, revision_id, edit_targets(state));
        } else if let Some(dataset_id) = dataset_id.clone() {
            load_dataset_for_edit(dataset_id, edit_targets(state));
        } else {
            state.editor_ready.set(true);
        }
    });
}

fn seed_dataset_editor_from_bootstrap(state: DatasetEditorState) -> bool {
    let Some(editor) = dataset_route_bootstrap().and_then(|bootstrap| bootstrap.editor().cloned())
    else {
        return false;
    };
    let bootstrap_is_complete = editor.provider_error.is_none();
    state.datasets.set(editor.datasets);
    state.forms.set(editor.forms);
    state.nodes.set(editor.nodes);
    state.users.set(editor.principals);
    state.rendered_forms.set(editor.rendered_forms);
    state.load_error.set(editor.provider_error);
    let tags = editor
        .dataset
        .as_ref()
        .map(|dataset| dataset.tags.clone())
        .unwrap_or_default();
    if let Some(revision) = editor.revision {
        seed_dataset_revision_for_edit(revision, tags, &edit_targets(state));
    } else if let Some(dataset) = editor.dataset {
        seed_dataset_for_edit(dataset, &edit_targets(state));
    } else {
        state.editor_ready.set(true);
    }
    // A complete owner projection is authoritative for this document and
    // must not be fetched a second time during hydration. A degraded SSR
    // projection still triggers the canonical Dataset-owned loaders so a
    // provider that recovered between document render and hydration can heal
    // without discarding the editor route.
    bootstrap_is_complete
}

fn edit_targets(state: DatasetEditorState) -> DatasetEditLoadTargets {
    DatasetEditLoadTargets {
        name: state.name,
        slug: state.slug,
        tags: state.tags,
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
    }
}
