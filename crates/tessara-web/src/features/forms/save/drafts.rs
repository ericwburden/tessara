//! Prepared form save drafts built from editor state.

use crate::features::forms::builder::{
    FormBuilderFieldDraft, FormBuilderSectionDraft, prepared_form_builder_fields,
    prepared_form_builder_sections,
};
use crate::features::forms::filtering::{existing_form_slugs, existing_form_slugs_for_update};
use crate::features::forms::types::{
    CreateFormPayload, FormSummary, RenderedForm, UpdateFormPayload,
};
use crate::features::shared::unique_slug_from_label;
use crate::utils::text::IntoNonemptyString;
use std::collections::HashSet;

/// Validated form create state ready for the async save sequence.
pub(super) struct PreparedCreateFormSave {
    pub(super) payload: CreateFormPayload,
    pub(super) sections: Vec<FormBuilderSectionDraft>,
    pub(super) fields: Vec<FormBuilderFieldDraft>,
}

/// Validated form update state ready for the async save sequence.
pub(super) struct PreparedUpdateFormSave {
    pub(super) payload: UpdateFormPayload,
    pub(super) sections: Vec<FormBuilderSectionDraft>,
    pub(super) fields: Vec<FormBuilderFieldDraft>,
    pub(super) original_section_ids: HashSet<String>,
    pub(super) original_field_ids: HashSet<String>,
    pub(super) kept_section_ids: HashSet<String>,
    pub(super) kept_field_ids: HashSet<String>,
}

/// Validates create form editor state and builds the create payload.
pub(super) fn prepare_create_form_save(
    form_name: String,
    workflow_node_type_id: String,
    sections: &[FormBuilderSectionDraft],
    fields: &[FormBuilderFieldDraft],
    existing_forms: &[FormSummary],
) -> Result<PreparedCreateFormSave, String> {
    if form_name.is_empty() {
        return Err("Form name is required.".into());
    }

    let form_slug = unique_slug_from_label(&form_name, &existing_form_slugs(existing_forms));
    if form_slug.is_empty() {
        return Err("Form name must contain letters or numbers.".into());
    }

    let prepared_sections = prepared_form_builder_sections(sections)?;
    let prepared_fields = prepared_form_builder_fields(fields)?;
    if prepared_fields.is_empty() {
        return Err("Add at least one field to the form builder.".into());
    }

    Ok(PreparedCreateFormSave {
        payload: CreateFormPayload {
            name: form_name,
            slug: form_slug,
            scope_node_type_id: workflow_node_type_id.into_nonempty(),
        },
        sections: prepared_sections,
        fields: prepared_fields,
    })
}

/// Validates update form editor state and builds the update payload plus draft diff sets.
pub(super) fn prepare_update_form_save(
    form_id: &str,
    form_name: String,
    workflow_node_type_id: String,
    sections: &[FormBuilderSectionDraft],
    fields: &[FormBuilderFieldDraft],
    existing_forms: &[FormSummary],
    rendered_form: Option<RenderedForm>,
) -> Result<PreparedUpdateFormSave, String> {
    if form_name.is_empty() {
        return Err("Form name is required.".into());
    }

    let form_slug = unique_slug_from_label(
        &form_name,
        &existing_form_slugs_for_update(existing_forms, form_id),
    );
    if form_slug.is_empty() {
        return Err("Form name must contain letters or numbers.".into());
    }

    let prepared_sections = prepared_form_builder_sections(sections)?;
    let prepared_fields = prepared_form_builder_fields(fields)?;
    if prepared_fields.is_empty() {
        return Err("Add at least one field to the form builder.".into());
    }

    let original_section_ids = rendered_form
        .as_ref()
        .map(|rendered| {
            rendered
                .sections
                .iter()
                .map(|section| section.id.clone())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default();
    let original_field_ids = rendered_form
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

    Ok(PreparedUpdateFormSave {
        payload: UpdateFormPayload {
            name: form_name,
            slug: form_slug,
            scope_node_type_id: workflow_node_type_id.into_nonempty(),
        },
        sections: prepared_sections,
        fields: prepared_fields,
        original_section_ids,
        original_field_ids,
        kept_section_ids,
        kept_field_ids,
    })
}
