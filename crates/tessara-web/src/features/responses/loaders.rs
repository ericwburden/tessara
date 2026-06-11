//! Signal-aware loaders for the Responses feature.
//!
//! Keep page loading state and response-specific fallback behavior here; endpoint transport belongs in `api`.

use crate::features::forms::RenderedForm;
use crate::features::responses::types::{
    AssignmentResponseStartOptions, SubmissionDetail, SubmissionSummary,
};
#[cfg(feature = "hydrate")]
use crate::features::responses::value_collection::submission_value_maps;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::api::{
    ResponseApiError, fetch_rendered_form, fetch_response_start_options, fetch_submission_detail,
    fetch_submissions,
};

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

pub(crate) fn load_submission_edit_context(
    submission_id: String,
    detail: RwSignal<Option<SubmissionDetail>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    text_values: RwSignal<HashMap<String, String>>,
    boolean_values: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            let loaded_detail = match fetch_submission_detail(&submission_id).await {
                Ok(detail) => detail,
                Err(ResponseApiError::Unauthorized) => {
                    is_loading.set(false);
                    redirect_to_login();
                    return;
                }
                Err(ResponseApiError::Message(error)) => {
                    load_error.set(Some(error));
                    is_loading.set(false);
                    return;
                }
            };

            if loaded_detail.status != "draft" {
                let (loaded_text_values, loaded_boolean_values) =
                    submission_value_maps(&loaded_detail);
                text_values.set(loaded_text_values);
                boolean_values.set(loaded_boolean_values);
                detail.set(Some(loaded_detail));
                rendered_form.set(None);
                is_loading.set(false);
                return;
            }

            let loaded_rendered = match fetch_rendered_form(&loaded_detail.form_version_id).await {
                Ok(rendered) => rendered,
                Err(ResponseApiError::Unauthorized) => {
                    is_loading.set(false);
                    redirect_to_login();
                    return;
                }
                Err(ResponseApiError::Message(error)) => {
                    load_error.set(Some(error));
                    is_loading.set(false);
                    return;
                }
            };

            let (loaded_text_values, loaded_boolean_values) = submission_value_maps(&loaded_detail);
            text_values.set(loaded_text_values);
            boolean_values.set(loaded_boolean_values);
            detail.set(Some(loaded_detail));
            rendered_form.set(Some(loaded_rendered));
            is_loading.set(false);
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            submission_id,
            detail,
            rendered_form,
            text_values,
            boolean_values,
            is_loading,
            load_error,
        );
    }
}

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
