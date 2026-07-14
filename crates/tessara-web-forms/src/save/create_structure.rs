//! Create-flow section and field persistence helpers.

use crate::builder::{FormBuilderFieldDraft, FormBuilderSectionDraft};
use crate::http::IdResponse;
use crate::save::payloads::{form_field_payload, form_section_payload};
use std::collections::HashMap;

pub(super) enum FormStructureSaveError {
    Unauthorized,
    Message(String),
}

impl FormStructureSaveError {
    fn from_request(error: tessara_web_http::RequestError) -> Self {
        if error.is_authentication() {
            Self::Unauthorized
        } else {
            Self::Message(error.into_message())
        }
    }
}

pub(super) async fn create_form_sections_for_new_form(
    version_id: &str,
    sections: &[FormBuilderSectionDraft],
) -> Result<HashMap<usize, String>, FormStructureSaveError> {
    let mut section_ids = HashMap::new();
    for section in sections {
        let section_payload = form_section_payload(section);
        let created_section: IdResponse = tessara_web_http::send_json(
            gloo_net::http::Request::post(&format!(
                "/api/admin/form-versions/{version_id}/sections"
            )),
            &section_payload,
            &format!("Create {} section", section.title),
        )
        .await
        .map_err(FormStructureSaveError::from_request)?;
        section_ids.insert(section.id, created_section.id);
    }

    Ok(section_ids)
}

pub(super) async fn create_form_fields_for_new_form(
    version_id: &str,
    fields: &[FormBuilderFieldDraft],
    section_ids: &HashMap<usize, String>,
) -> Result<(), FormStructureSaveError> {
    for (index, field) in fields.iter().enumerate() {
        let Some(section_id) = section_ids.get(&field.section_id) else {
            return Err(FormStructureSaveError::Message(format!(
                "{} field could not be matched to a section.",
                field.label
            )));
        };
        let field_payload = form_field_payload(field, section_id.clone(), (index + 1) as i32);
        tessara_web_http::send_json_without_response(
            gloo_net::http::Request::post(&format!("/api/admin/form-versions/{version_id}/fields")),
            &field_payload,
            &format!("Create {} field", field.label),
        )
        .await
        .map_err(FormStructureSaveError::from_request)?;
    }

    Ok(())
}
