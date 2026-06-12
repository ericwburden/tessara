//! Form create save orchestration.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::drafts::prepare_create_form_save;
#[cfg(feature = "hydrate")]
use crate::features::forms::save::structure::{
    FormStructureSaveError, create_form_fields_for_new_form, create_form_sections_for_new_form,
};
use crate::features::forms::types::FormSummary;
#[cfg(feature = "hydrate")]
use crate::http::{IdResponse, redirect_to_login, send_json_id_request};
use leptos::prelude::*;

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
/// Submits the submit create form request.
pub(crate) fn submit_create_form(
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    publish_after_save: bool,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let prepared_save = match prepare_create_form_save(
            name.get().trim().to_string(),
            workflow_node_type_id.get().trim().to_string(),
            &sections.get_untracked(),
            &fields.get_untracked(),
            existing_forms.get_untracked().as_slice(),
        ) {
            Ok(prepared_save) => prepared_save,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let payload = prepared_save.payload;
        let prepared_sections = prepared_save.sections;
        let prepared_fields = prepared_save.fields;

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Create request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::post("/api/admin/forms")
                .header("Content-Type", "application/json")
                .body(body)
                .expect("json request body should be valid")
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<IdResponse>().await {
                    Ok(created) => {
                        let version_response = gloo_net::http::Request::post(&format!(
                            "/api/admin/forms/{}/versions",
                            created.id
                        ))
                        .header("Content-Type", "application/json")
                        .body("{}")
                        .expect("json request body should be valid")
                        .send()
                        .await;

                        match version_response {
                            Ok(response) if response.status() == 401 => {
                                is_saving.set(false);
                                redirect_to_login();
                            }
                            Ok(response) if response.ok() => {
                                let created_version = match response.json::<IdResponse>().await {
                                    Ok(created_version) => created_version,
                                    Err(_) => {
                                        message.set(Some(
                                            "Form was created, but draft version response could not be read."
                                                .into(),
                                        ));
                                        is_saving.set(false);
                                        return;
                                    }
                                };

                                let section_ids = match create_form_sections_for_new_form(
                                    &created_version.id,
                                    &prepared_sections,
                                )
                                .await
                                {
                                    Ok(section_ids) => section_ids,
                                    Err(FormStructureSaveError::Unauthorized) => {
                                        is_saving.set(false);
                                        redirect_to_login();
                                        return;
                                    }
                                    Err(FormStructureSaveError::Message(error)) => {
                                        message.set(Some(error));
                                        is_saving.set(false);
                                        return;
                                    }
                                };

                                if let Err(error) = create_form_fields_for_new_form(
                                    &created_version.id,
                                    &prepared_fields,
                                    &section_ids,
                                )
                                .await
                                {
                                    match error {
                                        FormStructureSaveError::Unauthorized => {
                                            is_saving.set(false);
                                            redirect_to_login();
                                        }
                                        FormStructureSaveError::Message(error) => {
                                            message.set(Some(error));
                                            is_saving.set(false);
                                        }
                                    }
                                    return;
                                }

                                if publish_after_save {
                                    if let Err(error) = send_json_id_request(
                                        gloo_net::http::Request::post(&format!(
                                            "/api/admin/form-versions/{}/publish",
                                            created_version.id
                                        )),
                                        None,
                                        "Publish form version",
                                    )
                                    .await
                                    {
                                        message.set(Some(error));
                                        is_saving.set(false);
                                        return;
                                    }
                                }

                                if let Some(window) = web_sys::window() {
                                    let _ = window
                                        .location()
                                        .set_href(&format!("/forms/{}", created.id));
                                }
                            }
                            Ok(response) => {
                                message.set(Some(format!(
                                    "Form was created, but draft version setup failed with status {}.",
                                    response.status()
                                )));
                                is_saving.set(false);
                            }
                            Err(_) => {
                                message.set(Some(
                                    "Form was created, but the draft version API could not be reached."
                                        .into(),
                                ));
                                is_saving.set(false);
                            }
                        }
                    }
                    Err(_) => {
                        message.set(Some("Create response could not be read.".into()));
                        is_saving.set(false);
                    }
                },
                Ok(response) => {
                    message.set(Some(format!(
                        "Create failed with status {}.",
                        response.status()
                    )));
                    is_saving.set(false);
                }
                Err(_) => {
                    message.set(Some("Could not reach the create form API.".into()));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            name,
            workflow_node_type_id,
            fields,
            existing_forms,
            is_saving,
            message,
            publish_after_save,
        );
    }
}
