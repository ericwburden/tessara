//! Submit orchestration for the dataset editor surface.

use super::DatasetEditorState;
use crate::features::datasets::actions::save_dataset;
use leptos::prelude::Get;

pub(crate) fn submit_dataset_editor(dataset_id: Option<String>, state: DatasetEditorState) {
    save_dataset(
        dataset_id,
        state.name.get(),
        state.slug.get(),
        state.composition_mode.get(),
        state.visibility_node_ids.get().into_iter().collect(),
        state.sources.get(),
        state.expression.get(),
        state.fields.get(),
        state.join_left_key.get(),
        state.join_right_key.get(),
        state.save_error,
        state.save_message,
    );
}
