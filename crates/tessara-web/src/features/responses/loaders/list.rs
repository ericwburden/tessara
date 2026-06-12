//! Response list loading orchestration.

use crate::features::responses::types::SubmissionSummary;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;

#[cfg(feature = "hydrate")]
use super::super::api::{ResponseApiError, fetch_submissions};

pub(crate) fn load_submissions(
    submissions: RwSignal<Vec<SubmissionSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match fetch_submissions().await {
                Ok(loaded_submissions) => {
                    submissions.set(loaded_submissions);
                    is_loading.set(false);
                }
                Err(ResponseApiError::Unauthorized) => {
                    submissions.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Err(ResponseApiError::Message(error)) => {
                    submissions.set(Vec::new());
                    load_error.set(Some(error));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (submissions, is_loading, load_error);
    }
}
