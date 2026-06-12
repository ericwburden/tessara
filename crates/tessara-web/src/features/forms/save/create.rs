//! Form create save orchestration.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::drafts::prepare_create_form_save;
#[cfg(feature = "hydrate")]
use crate::features::forms::save::payloads::{form_field_payload, form_section_payload};
use crate::features::forms::types::FormSummary;
#[cfg(feature = "hydrate")]
use crate::features::organization::types::IdResponse;
#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_id_request};
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use std::collections::HashMap;

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

                                let mut section_ids = HashMap::new();
                                for section in &prepared_sections {
                                    let section_payload = form_section_payload(section);
                                    let section_body = match serde_json::to_string(&section_payload)
                                    {
                                        Ok(body) => body,
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "{} section request could not be prepared.",
                                                section.title
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    };
                                    let section_response = gloo_net::http::Request::post(&format!(
                                        "/api/admin/form-versions/{}/sections",
                                        created_version.id
                                    ))
                                    .header("Content-Type", "application/json")
                                    .body(section_body)
                                    .expect("json request body should be valid")
                                    .send()
                                    .await;

                                    let created_section = match section_response {
                                        Ok(response) if response.status() == 401 => {
                                            is_saving.set(false);
                                            redirect_to_login();
                                            return;
                                        }
                                        Ok(response) if response.ok() => {
                                            match response.json::<IdResponse>().await {
                                                Ok(created_section) => created_section,
                                                Err(_) => {
                                                    message.set(Some(format!(
                                                        "{} section response could not be read.",
                                                        section.title
                                                    )));
                                                    is_saving.set(false);
                                                    return;
                                                }
                                            }
                                        }
                                        Ok(response) => {
                                            message.set(Some(format!(
                                                "Form was created, but {} section setup failed with status {}.",
                                                section.title,
                                                response.status()
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "Form was created, but the {} section API could not be reached.",
                                                section.title
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    };
                                    section_ids.insert(section.id, created_section.id);
                                }

                                for (index, field) in prepared_fields.iter().enumerate() {
                                    let Some(section_id) = section_ids.get(&field.section_id)
                                    else {
                                        message.set(Some(format!(
                                            "{} field could not be matched to a section.",
                                            field.label
                                        )));
                                        is_saving.set(false);
                                        return;
                                    };
                                    let field_payload = form_field_payload(
                                        field,
                                        section_id.clone(),
                                        (index + 1) as i32,
                                    );
                                    let field_body = match serde_json::to_string(&field_payload) {
                                        Ok(body) => body,
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "{} field request could not be prepared.",
                                                field.label
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    };
                                    let field_response = gloo_net::http::Request::post(&format!(
                                        "/api/admin/form-versions/{}/fields",
                                        created_version.id
                                    ))
                                    .header("Content-Type", "application/json")
                                    .body(field_body)
                                    .expect("json request body should be valid")
                                    .send()
                                    .await;

                                    match field_response {
                                        Ok(response) if response.status() == 401 => {
                                            is_saving.set(false);
                                            redirect_to_login();
                                            return;
                                        }
                                        Ok(response) if response.ok() => {}
                                        Ok(response) => {
                                            message.set(Some(format!(
                                                "{} field setup failed with status {}.",
                                                field.label,
                                                response.status()
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "{} field API could not be reached.",
                                                field.label
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    }
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
