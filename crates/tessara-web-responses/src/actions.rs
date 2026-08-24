//! Signal-aware response actions.
//!
//! Keep save, submit, start, and navigation orchestration here; endpoint transport belongs in `api`.

#[cfg(feature = "hydrate")]
use crate::api::{
    ResponseApiError, save_response_values_api, start_assignment_response, submit_response_api,
};
#[cfg(feature = "hydrate")]
use crate::http::{navigate_to_href, redirect_to_login};
use crate::types::ResponseFormSnapshot;
#[cfg(feature = "hydrate")]
use crate::types::SaveResponseValuesRequest;
#[cfg(feature = "hydrate")]
use crate::value_collection::collect_response_values;
use leptos::prelude::*;
use std::collections::HashMap;
#[cfg(feature = "hydrate")]
use tessara_responses_contract::ResponseMutationResult;
use uuid::Uuid;

#[cfg(feature = "hydrate")]
fn prepare_response_values_payload(
    rendered_form: &ResponseFormSnapshot,
    expected_revision: u64,
    text_values: &HashMap<String, String>,
    boolean_values: &HashMap<String, bool>,
) -> Result<SaveResponseValuesRequest, ResponseApiError> {
    collect_response_values(rendered_form, text_values, boolean_values)
        .map(|values| SaveResponseValuesRequest {
            expected_revision,
            values,
        })
        .map_err(ResponseApiError::message)
}

#[cfg(feature = "hydrate")]
async fn save_response_draft(
    response_id: Uuid,
    rendered_form: &ResponseFormSnapshot,
    expected_revision: u64,
    text_values: &HashMap<String, String>,
    boolean_values: &HashMap<String, bool>,
) -> Result<ResponseMutationResult, ResponseApiError> {
    let payload = prepare_response_values_payload(
        rendered_form,
        expected_revision,
        text_values,
        boolean_values,
    )?;
    save_response_values_api(response_id, payload).await
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
    workflow_assignment_id: Uuid,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(Some("Starting assigned response...".into()));

            match start_assignment_response(workflow_assignment_id).await {
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

pub(crate) fn save_response_values(
    response_id: Uuid,
    rendered_form: ResponseFormSnapshot,
    revision: RwSignal<u64>,
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
                response_id,
                &rendered_form,
                revision.get_untracked(),
                &text_values,
                &boolean_values,
            )
            .await
            {
                Ok(saved) => {
                    revision.set(saved.revision);
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
            response_id,
            rendered_form,
            revision,
            text_values,
            boolean_values,
            is_saving,
            message,
        );
    }
}

pub(crate) fn submit_response_values(
    response_id: Uuid,
    rendered_form: ResponseFormSnapshot,
    revision: RwSignal<u64>,
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
                response_id,
                &rendered_form,
                revision.get_untracked(),
                &text_values,
                &boolean_values,
            )
            .await
            {
                Ok(saved) => match submit_response_api(response_id, saved.revision).await {
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
            response_id,
            rendered_form,
            revision,
            text_values,
            boolean_values,
            is_saving,
            message,
        );
    }
}
