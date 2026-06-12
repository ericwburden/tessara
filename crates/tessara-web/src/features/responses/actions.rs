//! Signal-aware response actions.
//!
//! Keep save, submit, start, and navigation orchestration here; endpoint transport belongs in `api`.

use crate::features::forms::RenderedForm;
#[cfg(feature = "hydrate")]
use crate::features::responses::api::{
    ResponseApiError, save_submission_values_api, start_assignment_response, submit_submission_api,
};
#[cfg(feature = "hydrate")]
use crate::features::responses::types::SaveSubmissionValuesPayload;
#[cfg(feature = "hydrate")]
use crate::features::responses::value_collection::collect_response_values;
#[cfg(feature = "hydrate")]
use crate::http::{navigate_to_href, redirect_to_login};
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
fn prepare_submission_values_payload(
    rendered_form: &RenderedForm,
    text_values: &HashMap<String, String>,
    boolean_values: &HashMap<String, bool>,
) -> Result<SaveSubmissionValuesPayload, ResponseApiError> {
    collect_response_values(rendered_form, text_values, boolean_values)
        .map(|values| SaveSubmissionValuesPayload { values })
        .map_err(ResponseApiError::message)
}

#[cfg(feature = "hydrate")]
async fn save_response_draft(
    submission_id: &str,
    rendered_form: &RenderedForm,
    text_values: &HashMap<String, String>,
    boolean_values: &HashMap<String, bool>,
) -> Result<(), ResponseApiError> {
    let payload = prepare_submission_values_payload(rendered_form, text_values, boolean_values)?;
    save_submission_values_api(submission_id, payload)
        .await
        .map(|_| ())
}

#[cfg(feature = "hydrate")]
fn handle_response_action_error(
    error: ResponseApiError,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    match error {
        ResponseApiError::Unauthorized => {
            redirect_to_login();
            is_saving.set(false);
        }
        ResponseApiError::Message(error) => {
            message.set(Some(error));
            is_saving.set(false);
        }
    }
}

pub(crate) fn start_assignment_response_and_navigate(
    workflow_assignment_id: String,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(Some("Starting assigned response...".into()));

            match start_assignment_response(&workflow_assignment_id).await {
                Ok(id) => {
                    navigate_to_href(&format!("/responses/{id}/edit"));
                }
                Err(error) => handle_response_action_error(error, is_saving, message),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflow_assignment_id, is_saving, message);
    }
}

pub(crate) fn save_submission_values(
    submission_id: String,
    rendered_form: RenderedForm,
    text_values: HashMap<String, String>,
    boolean_values: HashMap<String, bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            match save_response_draft(
                &submission_id,
                &rendered_form,
                &text_values,
                &boolean_values,
            )
            .await
            {
                Ok(_) => {
                    message.set(Some("Draft saved.".into()));
                    is_saving.set(false);
                }
                Err(error) => handle_response_action_error(error, is_saving, message),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            submission_id,
            rendered_form,
            text_values,
            boolean_values,
            is_saving,
            message,
        );
    }
}

pub(crate) fn submit_response_values(
    submission_id: String,
    rendered_form: RenderedForm,
    text_values: HashMap<String, String>,
    boolean_values: HashMap<String, bool>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            match save_response_draft(
                &submission_id,
                &rendered_form,
                &text_values,
                &boolean_values,
            )
            .await
            {
                Ok(_) => match submit_submission_api(&submission_id).await {
                    Ok(response) => navigate_to_href(&format!("/responses/{}", response.id)),
                    Err(error) => handle_response_action_error(error, is_saving, message),
                },
                Err(error) => handle_response_action_error(error, is_saving, message),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            submission_id,
            rendered_form,
            text_values,
            boolean_values,
            is_saving,
            message,
        );
    }
}
