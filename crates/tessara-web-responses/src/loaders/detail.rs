//! Response detail loading orchestration.

#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use crate::types::ResponseDetail;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::super::api::{ResponseApiError, fetch_response_detail};

pub(crate) fn load_response_detail(
    response_id: String,
    detail: RwSignal<Option<ResponseDetail>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_response_detail(&response_id).await {
                Ok(loaded_detail) => {
                    detail.set(Some(loaded_detail));
                    is_loading.set(false);
                }
                Err(ResponseApiError::Unauthorized) => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(ResponseApiError::Message(error)) => {
                    detail.set(None);
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (response_id, detail, is_loading, load_error);
    }
}
