//! Form update save orchestration.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
#[cfg(feature = "hydrate")]
use crate::features::forms::save::drafts::prepare_update_form_save;
#[cfg(feature = "hydrate")]
use crate::features::forms::save::payloads::{form_field_payload, form_section_payload};
use crate::features::forms::types::{FormSummary, RenderedForm};
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
use leptos::prelude::*;
#[cfg(feature = "hydrate")]
use std::collections::HashMap;

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
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

        let prepared_save = match prepare_update_form_save(
            &form_id,
            name.get().trim().to_string(),
            workflow_node_type_id.get().trim().to_string(),
            &sections.get_untracked(),
            &fields.get_untracked(),
            existing_forms.get_untracked().as_slice(),
            rendered_form.get_untracked(),
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
        let original_section_ids = prepared_save.original_section_ids;
        let original_field_ids = prepared_save.original_field_ids;
        let kept_section_ids = prepared_save.kept_section_ids;
        let kept_field_ids = prepared_save.kept_field_ids;
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
                let section_payload = form_section_payload(section);
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
                let field_payload =
                    form_field_payload(field, section_id.clone(), (index + 1) as i32);
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
