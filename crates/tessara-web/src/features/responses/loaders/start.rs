//! Response start option loading orchestration.

use crate::features::responses::types::AssignmentResponseStartOptions;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::super::api::{ResponseApiError, fetch_response_start_options};

pub(crate) fn load_response_start_options(
    options: RwSignal<Option<AssignmentResponseStartOptions>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    delegate_account_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            match fetch_response_start_options(delegate_account_id.as_deref()).await {
                Ok(loaded_options) => {
                    options.set(Some(loaded_options));
                    is_loading.set(false);
                }
                Err(ResponseApiError::Unauthorized) => {
                    options.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(ResponseApiError::Message(error)) => {
                    options.set(None);
                    message.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (options, is_loading, message, delegate_account_id);
    }
}
