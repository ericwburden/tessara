//! Source field and form-version option helpers for the dataset editor.

use super::super::types::*;
use std::collections::BTreeMap;

/// Finds the first published version for a form.
pub(crate) fn first_published_version(
    forms: &[DatasetFormOption],
    form_id: &str,
) -> Option<DatasetFormVersionOption> {
    forms
        .iter()
        .find(|form| form.id == form_id)
        .and_then(|form| {
            published_versions_for_form(forms, &form.id)
                .into_iter()
                .next()
        })
}

/// Returns published versions for a form.
pub(crate) fn published_versions_for_form(
    forms: &[DatasetFormOption],
    form_id: &str,
) -> Vec<DatasetFormVersionOption> {
    forms
        .iter()
        .find(|form| form.id == form_id)
        .map(|form| {
            form.versions
                .iter()
                .filter(|version| version.status == "published")
                .cloned()
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

/// Returns source field options for the selected source alias.
pub(crate) fn source_field_options(
    sources: &[DatasetSourceDraft],
    datasets: &[DatasetSummary],
    _forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_alias: &str,
) -> Vec<DatasetRenderedField> {
    let Some(source) = sources
        .iter()
        .find(|source| source.source_alias == source_alias)
    else {
        return Vec::new();
    };
    if !source_has_selected_reference(source) {
        return Vec::new();
    }
    if source.input_kind.eq_ignore_ascii_case("dataset") {
        return datasets
            .iter()
            .find(|dataset| {
                dataset.id == source.dataset_id
                    || dataset.current_revision_id.as_deref()
                        == Some(source.dataset_revision_id.as_str())
            })
            .map(|dataset| {
                let fields = dataset
                    .revisions
                    .iter()
                    .find(|revision| revision.id == source.dataset_revision_id)
                    .map(|revision| revision.output_fields.as_slice())
                    .unwrap_or(dataset.output_fields.as_slice());
                fields
                    .iter()
                    .map(|field| DatasetRenderedField {
                        key: field.key.clone(),
                        label: field.label.clone(),
                        field_type: field.field_type.clone(),
                        value_options: Vec::new(),
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
    }
    let mut options = system_source_field_options();
    options.extend(
        rendered_forms
            .get(&source.form_version_id)
            .map(|rendered| {
                rendered
                    .sections
                    .iter()
                    .flat_map(|section| section.fields.iter().cloned())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
    );
    options
}

fn source_has_selected_reference(source: &DatasetSourceDraft) -> bool {
    if source.input_kind.eq_ignore_ascii_case("dataset") {
        !source.dataset_revision_id.trim().is_empty()
    } else {
        !source.form_version_id.trim().is_empty()
    }
}

/// Built-in submission metadata fields available for every form source.
pub(crate) fn system_source_field_options() -> Vec<DatasetRenderedField> {
    let mut fields = [
        ("__submission_id", "Submission ID", "text"),
        ("__form_version_id", "Form Version ID", "text"),
        ("__node_id", "Attached Node ID", "text"),
        ("__node_name", "Attached Node Name", "text"),
        ("__submission_status", "Submission Status", "single_choice"),
        ("__submitted_at", "Submitted Date", "date"),
        ("__submission_created_at", "Created Date", "date"),
        ("__last_updated_at", "Updated Date", "date"),
        (
            "__last_updated_by_user_name",
            "Updated By User Name",
            "text",
        ),
    ]
    .into_iter()
    .map(|(key, label, field_type)| DatasetRenderedField {
        key: key.into(),
        label: label.into(),
        field_type: field_type.into(),
        value_options: Vec::new(),
    })
    .collect::<Vec<_>>();
    if let Some(status_field) = fields
        .iter_mut()
        .find(|field| field.key == "__submission_status")
    {
        status_field.value_options = vec!["draft".into(), "submitted".into()];
    }
    fields
}
