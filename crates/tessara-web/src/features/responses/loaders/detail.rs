//! Response detail loading orchestration.

use crate::features::responses::types::SubmissionDetail;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::super::api::{ResponseApiError, fetch_submission_detail};

pub(crate) fn load_submission_detail(
    submission_id: String,
    detail: RwSignal<Option<SubmissionDetail>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_submission_detail(&submission_id).await {
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
        let _ = (submission_id, detail, is_loading, load_error);
    }
}
