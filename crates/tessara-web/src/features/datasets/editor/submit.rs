//! Submit orchestration for the dataset editor surface.

use super::DatasetEditorState;
use crate::features::datasets::actions::save_dataset;
use leptos::prelude::Get;

pub(crate) fn submit_dataset_editor(dataset_id: Option<String>, state: DatasetEditorState) {
    save_dataset(
        dataset_id,
        state.name.get(),
        state.slug.get(),
        state.visibility_node_ids.get().into_iter().collect(),
        state.initial_source.get(),
        state.operation_order.get(),
        state.restriction_internal_field_key.get(),
        state.restriction_restricted_field_key.get(),
        state.restriction_confidential_field_key.get(),
        state.save_error,
        state.save_message,
    );
}
