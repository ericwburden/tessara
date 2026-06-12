//! Response edit loading orchestration.

use crate::features::forms::RenderedForm;
use crate::features::responses::types::SubmissionDetail;
#[cfg(feature = "hydrate")]
use crate::features::responses::value_collection::submission_value_maps;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::api::{ResponseApiError, fetch_rendered_form, fetch_submission_detail};

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
