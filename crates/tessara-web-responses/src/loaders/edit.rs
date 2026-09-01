//! Response edit loading orchestration.

#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use crate::types::{ResponseDetail, ResponseFormSnapshot};
#[cfg(feature = "hydrate")]
use crate::value_collection::response_value_maps;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::api::{ResponseApiError, fetch_response_detail};

pub(crate) fn load_response_edit_context(
    response_id: String,
    detail: RwSignal<Option<ResponseDetail>>,
    rendered_form: RwSignal<Option<ResponseFormSnapshot>>,
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

            let loaded_detail = match fetch_response_detail(&response_id).await {
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
                    response_value_maps(&loaded_detail);
                text_values.set(loaded_text_values);
                boolean_values.set(loaded_boolean_values);
                detail.set(Some(loaded_detail));
                rendered_form.set(None);
                is_loading.set(false);
                return;
            }

            let (loaded_text_values, loaded_boolean_values) = response_value_maps(&loaded_detail);
            let loaded_rendered = loaded_detail.form.clone();
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
            response_id,
            detail,
            rendered_form,
            text_values,
            boolean_values,
            is_loading,
            load_error,
        );
    }
}
