//! Detail and table loaders for the Datasets feature.

#[cfg(feature = "hydrate")]
use super::super::api;
use super::super::types::{DatasetDefinition, DatasetTable};
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_dataset_detail(
    dataset_id: String,
    dataset: RwSignal<Option<DatasetDefinition>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        match api::fetch_dataset_detail(&dataset_id).await {
            Ok(Some(payload)) => dataset.set(Some(payload)),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_dataset_detail(
    _: String,
    _: RwSignal<Option<DatasetDefinition>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_dataset_table(
    dataset_id: String,
    table: RwSignal<Option<DatasetTable>>,
    table_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_dataset_table(&dataset_id).await {
            Ok(Some(payload)) => table.set(Some(payload)),
            Ok(None) => {}
            Err(message) => table_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_dataset_table(
    _: String,
    _: RwSignal<Option<DatasetTable>>,
    _: RwSignal<Option<String>>,
) {
}
