//! Signal-aware loaders for the Operations feature.

#[cfg(feature = "hydrate")]
use super::api;
use super::types::OperationsStatus;
use leptos::prelude::*;

/// Loads the operations status data.
pub(crate) fn load_operations_status(
    status: RwSignal<Option<OperationsStatus>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match api::fetch_operations_status().await {
                Ok(loaded_status) => status.set(Some(loaded_status)),
                Err(error) => {
                    status.set(None);
                    load_error.set(Some(error));
                }
            }

            is_loading.set(false);
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (status, load_error);
        is_loading.set(false);
    }
}
