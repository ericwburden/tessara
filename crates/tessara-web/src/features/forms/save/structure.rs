//! Form section and field persistence helpers.

use crate::features::forms::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
use crate::features::forms::save::payloads::{form_field_payload, form_section_payload};
use crate::http::send_json_id_request;
use std::collections::HashMap;

/// Saves prepared sections and returns local draft ids mapped to remote section ids.
pub(super) async fn save_form_sections(
    version_id: &str,
    sections: &[FormBuilderSectionDraft],
    update_existing_draft: bool,
) -> Result<HashMap<usize, String>, String> {
    let mut section_ids = HashMap::new();
    for section in sections {
        let section_payload = form_section_payload(section);
        let section_body = serde_json::to_string(&section_payload)
            .map_err(|_| format!("{} section request could not be prepared.", section.title))?;

        if update_existing_draft && let Some(section_id) = section.remote_id.as_ref() {
            let saved_section = send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/admin/form-sections/{section_id}")),
                Some(section_body),
                "Update form section",
            )
            .await?;
            section_ids.insert(section.id, saved_section.id);
        } else {
            let saved_section = send_json_id_request(
                gloo_net::http::Request::post(&format!(
                    "/api/admin/form-versions/{version_id}/sections"
                )),
                Some(section_body),
                "Create form section",
            )
            .await?;
            section_ids.insert(section.id, saved_section.id);
        }
    }

    Ok(section_ids)
}

/// Saves prepared fields against previously saved section ids.
pub(super) async fn save_form_fields(
    version_id: &str,
    fields: &[FormBuilderFieldDraft],
    section_ids: &HashMap<usize, String>,
    update_existing_draft: bool,
) -> Result<(), String> {
    for (index, field) in fields.iter().enumerate() {
        let Some(section_id) = section_ids.get(&field.section_id) else {
            return Err(format!(
                "{} field could not be matched to a section.",
                field.label
            ));
        };
        let field_payload = form_field_payload(field, section_id.clone(), (index + 1) as i32);
        let field_body = serde_json::to_string(&field_payload)
            .map_err(|_| format!("{} field request could not be prepared.", field.label))?;

        if update_existing_draft && let Some(field_id) = field.remote_id.as_ref() {
            send_json_id_request(
                gloo_net::http::Request::put(&format!(
                    "/api/admin/form-versions/{version_id}/fields/{field_id}"
                )),
                Some(field_body),
                "Update form field",
            )
            .await?;
        } else {
            send_json_id_request(
                gloo_net::http::Request::post(&format!(
                    "/api/admin/form-versions/{version_id}/fields"
                )),
                Some(field_body),
                "Create form field",
            )
            .await?;
        }
    }

    Ok(())
}
