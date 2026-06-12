//! List and account loaders for the Datasets feature.

#[cfg(feature = "hydrate")]
use super::super::api;
use super::super::types::{DatasetSummary, SessionAccount};
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_account(account: RwSignal<Option<SessionAccount>>) {
    leptos::task::spawn_local(async move {
        if let Ok(Some(payload)) = api::fetch_account().await {
            account.set(Some(payload));
        }
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_account(_: RwSignal<Option<SessionAccount>>) {}

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_datasets(
    datasets: RwSignal<Vec<DatasetSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        is_loading.set(true);
        match api::fetch_datasets().await {
            Ok(Some(payload)) => datasets.set(payload),
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
        is_loading.set(false);
    });
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_datasets(
    _: RwSignal<Vec<DatasetSummary>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}
