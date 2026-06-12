//! Loading lifecycle for the dataset editor surface.

use super::DatasetEditorState;
use crate::features::datasets::loaders::{
    load_dataset_for_edit, load_datasets, load_forms, load_nodes,
};
use leptos::prelude::*;

pub(crate) fn install_dataset_editor_loaders(
    dataset_id: Option<String>,
    state: DatasetEditorState,
) {
    Effect::new(move |_| {
        load_forms(state.forms, state.load_error);
        load_datasets(state.datasets, RwSignal::new(false), state.load_error);
        load_nodes(state.nodes, state.load_error);
        if let Some(dataset_id) = dataset_id.clone() {
            load_dataset_for_edit(
                dataset_id,
                state.name,
                state.slug,
                state.composition_mode,
                state.visibility_node_ids,
                state.sources,
                state.expression,
                state.fields,
                state.join_left_key,
                state.join_right_key,
                state.sql_preview,
                state.load_error,
            );
        }
    });
}
