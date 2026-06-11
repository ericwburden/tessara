//! Client-side API orchestration for the Responses feature.
//!
//! Keep endpoint calls, request assembly, and response handling for Responses screens here; pure DTOs and display formatting belong in sibling modules.

use crate::features::forms::RenderedForm;
#[cfg(feature = "hydrate")]
use crate::features::organization::IdResponse;
#[cfg(feature = "hydrate")]
use crate::features::responses::types::SaveSubmissionValuesPayload;
use crate::features::responses::types::{
    AssignmentResponseStartOptions, SubmissionDetail, SubmissionSummary,
};
#[cfg(feature = "hydrate")]
use crate::features::responses::value_collection::{
    collect_response_values, submission_value_maps,
};
#[cfg(feature = "hydrate")]
use crate::features::shared::navigate_to_href;
#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_request};
use leptos::prelude::*;
use std::collections::HashMap;

/// Loads the load submissions data.
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

            match gloo_net::http::Request::get("/api/submissions")
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    submissions.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<SubmissionSummary>>().await {
                        Ok(loaded_submissions) => {
                            submissions.set(loaded_submissions);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            submissions.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse responses: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    submissions.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load responses. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    submissions.set(Vec::new());
                    load_error.set(Some(format!("Unable to load responses: {error}")));
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

/// Loads the load submission detail data.
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

            match gloo_net::http::Request::get(&format!("/api/submissions/{submission_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<SubmissionDetail>().await {
                    Ok(loaded_detail) => {
                        detail.set(Some(loaded_detail));
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        load_error.set(Some(format!("Unable to parse response: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load response. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load response: {error}")));
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

/// Loads the load submission edit context data.
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

            let loaded_detail =
                match gloo_net::http::Request::get(&format!("/api/submissions/{submission_id}"))
                    .send()
                    .await
                {
                    Ok(response) if response.status() == 401 => {
                        is_loading.set(false);
                        redirect_to_login();
                        return;
                    }
                    Ok(response) if response.ok() => {
                        match response.json::<SubmissionDetail>().await {
                            Ok(detail) => detail,
                            Err(error) => {
                                load_error.set(Some(format!("Unable to parse response: {error}")));
                                is_loading.set(false);
                                return;
                            }
                        }
                    }
                    Ok(response) => {
                        load_error.set(Some(format!(
                            "Unable to load response. Server returned {}.",
                            response.status()
                        )));
                        is_loading.set(false);
                        return;
                    }
                    Err(error) => {
                        load_error.set(Some(format!("Unable to load response: {error}")));
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

            let loaded_rendered = match gloo_net::http::Request::get(&format!(
                "/api/form-versions/{}/render",
                loaded_detail.form_version_id
            ))
            .send()
            .await
            {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                    return;
                }
                Ok(response) if response.ok() => match response.json::<RenderedForm>().await {
                    Ok(rendered) => rendered,
                    Err(error) => {
                        load_error.set(Some(format!("Unable to parse response form: {error}")));
                        is_loading.set(false);
                        return;
                    }
                },
                Ok(response) => {
                    load_error.set(Some(format!(
                        "Unable to load response form. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                    return;
                }
                Err(error) => {
                    load_error.set(Some(format!("Unable to load response form: {error}")));
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

/// Loads the load response start options data.
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

            let path = delegate_account_id
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .map(|value| format!("/api/responses/options?delegate_account_id={value}"))
                .unwrap_or_else(|| "/api/responses/options".to_string());

            match gloo_net::http::Request::get(&path).send().await {
                Ok(response) if response.status() == 401 => {
                    options.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<AssignmentResponseStartOptions>().await {
                        Ok(loaded_options) => {
                            options.set(Some(loaded_options));
                            is_loading.set(false);
                        }
                        Err(error) => {
                            options.set(None);
                            message.set(Some(format!(
                                "Unable to parse assigned response start options: {error}"
                            )));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    options.set(None);
                    message.set(Some(format!(
                        "Unable to load assigned response start options. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    options.set(None);
                    message.set(Some(format!(
                        "Unable to load assigned response start options: {error}"
                    )));
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

/// Handles the save submission values behavior.
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

            let values =
                match collect_response_values(&rendered_form, &text_values, &boolean_values) {
                    Ok(values) => values,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                };

            let body = match serde_json::to_string(&SaveSubmissionValuesPayload { values }) {
                Ok(body) => body,
                Err(error) => {
                    message.set(Some(format!(
                        "Response values could not be prepared: {error}"
                    )));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_request::<IdResponse>(
                gloo_net::http::Request::put(&format!("/api/submissions/{submission_id}/values")),
                Some(body),
                "Save response draft",
            )
            .await
            {
                Ok(_) => {
                    message.set(Some("Draft saved.".into()));
                    is_saving.set(false);
                }
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
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

/// Submits the submit response values request.
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

            let values =
                match collect_response_values(&rendered_form, &text_values, &boolean_values) {
                    Ok(values) => values,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                };

            let body = match serde_json::to_string(&SaveSubmissionValuesPayload { values }) {
                Ok(body) => body,
                Err(error) => {
                    message.set(Some(format!(
                        "Response values could not be prepared: {error}"
                    )));
                    is_saving.set(false);
                    return;
                }
            };

            match send_json_request::<IdResponse>(
                gloo_net::http::Request::put(&format!("/api/submissions/{submission_id}/values")),
                Some(body),
                "Save response draft",
            )
            .await
            {
                Ok(_) => match send_json_request::<IdResponse>(
                    gloo_net::http::Request::post(&format!(
                        "/api/submissions/{submission_id}/submit"
                    )),
                    Some("{}".into()),
                    "Submit response",
                )
                .await
                {
                    Ok(response) => navigate_to_href(&format!("/responses/{}", response.id)),
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                    }
                },
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
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
