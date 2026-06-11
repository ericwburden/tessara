//! Form create/update save orchestration for Organization-owned form flows.

#[cfg(feature = "hydrate")]
use super::super::helpers::IntoNonemptyString;
use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::builder::{
    prepared_form_builder_fields, prepared_form_builder_sections,
};
#[cfg(feature = "hydrate")]
use crate::features::forms::types::{
    CreateFormFieldPayload, CreateFormPayload, CreateFormSectionPayload, UpdateFormPayload,
};
use crate::features::forms::{FormSummary, RenderedForm};
#[cfg(feature = "hydrate")]
use crate::features::organization::types::IdResponse;
#[cfg(feature = "hydrate")]
use crate::features::shared::{existing_form_slugs_for_update, unique_slug_from_label};
#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_id_request};
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use std::collections::{HashMap, HashSet};

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

        let form_name = name.get().trim().to_string();
        if form_name.is_empty() {
            message.set(Some("Form name is required.".into()));
            return;
        }

        let form_slug = unique_slug_from_label(
            &form_name,
            &crate::features::shared::existing_form_slugs(
                existing_forms.get_untracked().as_slice(),
            ),
        );
        if form_slug.is_empty() {
            message.set(Some("Form name must contain letters or numbers.".into()));
            return;
        }

        let current_fields = fields.get_untracked();
        let prepared_sections = match prepared_form_builder_sections(&sections.get_untracked()) {
            Ok(sections) => sections,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let prepared_fields = match prepared_form_builder_fields(&current_fields) {
            Ok(fields) => fields,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        if prepared_fields.is_empty() {
            message.set(Some("Add at least one field to the form builder.".into()));
            return;
        }

        let payload = CreateFormPayload {
            name: form_name,
            slug: form_slug,
            scope_node_type_id: workflow_node_type_id
                .get()
                .trim()
                .to_string()
                .into_nonempty(),
        };

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
                                    let section_payload = CreateFormSectionPayload {
                                        title: section.title.clone(),
                                        position: section.position,
                                        description: section.description.clone(),
                                    };
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
                                    let field_payload = CreateFormFieldPayload {
                                        section_id: section_id.clone(),
                                        key: field.key.clone(),
                                        label: field.label.clone(),
                                        field_type: field.field_type.clone(),
                                        required: field.required,
                                        position: (index + 1) as i32,
                                        grid_row: field.grid_row,
                                        grid_column: field.grid_column,
                                        grid_width: field.grid_width,
                                        grid_height: field.grid_height,
                                    };
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

/// Submits the submit update form request.
pub(crate) fn submit_update_form(
    form_id: String,
    name: RwSignal<String>,
    workflow_node_type_id: RwSignal<String>,
    sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    publish_after_save: bool,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let form_name = name.get().trim().to_string();
        if form_name.is_empty() {
            message.set(Some("Form name is required.".into()));
            return;
        }

        let form_slug = unique_slug_from_label(
            &form_name,
            &existing_form_slugs_for_update(existing_forms.get_untracked().as_slice(), &form_id),
        );
        if form_slug.is_empty() {
            message.set(Some("Form name must contain letters or numbers.".into()));
            return;
        }

        let current_sections = sections.get_untracked();
        let current_fields = fields.get_untracked();
        let prepared_sections = match prepared_form_builder_sections(&current_sections) {
            Ok(sections) => sections,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let prepared_fields = match prepared_form_builder_fields(&current_fields) {
            Ok(fields) => fields,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        if prepared_fields.is_empty() {
            message.set(Some("Add at least one field to the form builder.".into()));
            return;
        }

        let payload = UpdateFormPayload {
            name: form_name,
            slug: form_slug,
            scope_node_type_id: workflow_node_type_id
                .get()
                .trim()
                .to_string()
                .into_nonempty(),
        };
        let current_rendered_form = rendered_form.get_untracked();
        let original_section_ids = current_rendered_form
            .as_ref()
            .map(|rendered| {
                rendered
                    .sections
                    .iter()
                    .map(|section| section.id.clone())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        let original_field_ids = current_rendered_form
            .as_ref()
            .map(|rendered| {
                rendered
                    .sections
                    .iter()
                    .flat_map(|section| section.fields.iter().map(|field| field.id.clone()))
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        let kept_section_ids = prepared_sections
            .iter()
            .filter_map(|section| section.remote_id.clone())
            .collect::<HashSet<_>>();
        let kept_field_ids = prepared_fields
            .iter()
            .filter_map(|field| field.remote_id.clone())
            .collect::<HashSet<_>>();
        let update_existing_draft = edit_version_status.get_untracked().as_deref() == Some("draft");
        let existing_version_id = edit_version_id.get_untracked();

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Update request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            if let Err(error) = send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/admin/forms/{form_id}")),
                Some(body),
                "Update form",
            )
            .await
            {
                message.set(Some(error));
                is_saving.set(false);
                return;
            }

            let version_id = if update_existing_draft {
                match existing_version_id {
                    Some(version_id) => version_id,
                    None => {
                        message.set(Some("No editable draft version was available.".into()));
                        is_saving.set(false);
                        return;
                    }
                }
            } else {
                match send_json_id_request(
                    gloo_net::http::Request::post(&format!("/api/admin/forms/{form_id}/versions")),
                    Some("{}".into()),
                    "Create draft version",
                )
                .await
                {
                    Ok(created) => created.id,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            };

            if update_existing_draft {
                for field_id in original_field_ids.difference(&kept_field_ids) {
                    if let Err(error) = send_json_id_request(
                        gloo_net::http::Request::delete(&format!(
                            "/api/admin/form-fields/{field_id}"
                        )),
                        None,
                        "Delete form field",
                    )
                    .await
                    {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }

                for section_id in original_section_ids.difference(&kept_section_ids) {
                    if let Err(error) = send_json_id_request(
                        gloo_net::http::Request::delete(&format!(
                            "/api/admin/form-sections/{section_id}"
                        )),
                        None,
                        "Delete form section",
                    )
                    .await
                    {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            }

            let mut section_ids = HashMap::new();
            for section in &prepared_sections {
                let section_payload = CreateFormSectionPayload {
                    title: section.title.clone(),
                    position: section.position,
                    description: section.description.clone(),
                };
                let section_body = match serde_json::to_string(&section_payload) {
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

                let request = if update_existing_draft {
                    section
                        .remote_id
                        .as_ref()
                        .map(|section_id| {
                            (
                                gloo_net::http::Request::put(&format!(
                                    "/api/admin/form-sections/{section_id}"
                                )),
                                "Update form section",
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                gloo_net::http::Request::post(&format!(
                                    "/api/admin/form-versions/{version_id}/sections"
                                )),
                                "Create form section",
                            )
                        })
                } else {
                    (
                        gloo_net::http::Request::post(&format!(
                            "/api/admin/form-versions/{version_id}/sections"
                        )),
                        "Create form section",
                    )
                };

                match send_json_id_request(request.0, Some(section_body), request.1).await {
                    Ok(saved_section) => {
                        section_ids.insert(section.id, saved_section.id);
                    }
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            }

            for (index, field) in prepared_fields.iter().enumerate() {
                let Some(section_id) = section_ids.get(&field.section_id) else {
                    message.set(Some(format!(
                        "{} field could not be matched to a section.",
                        field.label
                    )));
                    is_saving.set(false);
                    return;
                };
                let field_payload = CreateFormFieldPayload {
                    section_id: section_id.clone(),
                    key: field.key.clone(),
                    label: field.label.clone(),
                    field_type: field.field_type.clone(),
                    required: field.required,
                    position: (index + 1) as i32,
                    grid_row: field.grid_row,
                    grid_column: field.grid_column,
                    grid_width: field.grid_width,
                    grid_height: field.grid_height,
                };
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

                let request = if update_existing_draft {
                    field
                        .remote_id
                        .as_ref()
                        .map(|field_id| {
                            (
                                gloo_net::http::Request::put(&format!(
                                    "/api/admin/form-fields/{field_id}"
                                )),
                                "Update form field",
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                gloo_net::http::Request::post(&format!(
                                    "/api/admin/form-versions/{version_id}/fields"
                                )),
                                "Create form field",
                            )
                        })
                } else {
                    (
                        gloo_net::http::Request::post(&format!(
                            "/api/admin/form-versions/{version_id}/fields"
                        )),
                        "Create form field",
                    )
                };

                if let Err(error) =
                    send_json_id_request(request.0, Some(field_body), request.1).await
                {
                    message.set(Some(error));
                    is_saving.set(false);
                    return;
                }
            }

            if publish_after_save {
                if let Err(error) = send_json_id_request(
                    gloo_net::http::Request::post(&format!(
                        "/api/admin/form-versions/{version_id}/publish"
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
                let _ = window.location().set_href(&format!("/forms/{form_id}"));
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            form_id,
            name,
            workflow_node_type_id,
            sections,
            fields,
            existing_forms,
            edit_version_id,
            edit_version_status,
            rendered_form,
            is_saving,
            message,
            publish_after_save,
        );
    }
}
