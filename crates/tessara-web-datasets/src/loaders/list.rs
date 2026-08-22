//! List and account loaders for the Datasets feature.

#[cfg(feature = "hydrate")]
use super::super::api;
use super::super::bootstrap::dataset_route_bootstrap;
use super::super::types::{DatasetSummary, SessionAccount};
use leptos::prelude::*;

pub(crate) fn load_account(account: RwSignal<Option<SessionAccount>>) {
    let can_manage = dataset_route_bootstrap().is_some_and(|bootstrap| bootstrap.can_manage());
    account.set(Some(SessionAccount {
        capabilities: if can_manage {
            vec!["admin:all".into()]
        } else {
            Vec::new()
        },
    }));
}

#[cfg(feature = "hydrate")]
pub(crate) fn load_datasets(
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
pub(crate) fn load_datasets(
    _: RwSignal<Vec<DatasetSummary>>,
    _: RwSignal<bool>,
    _: RwSignal<Option<String>>,
) {
}
