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

/// Finds a form version by id across the editor's loaded forms.
pub(crate) fn find_version(
    forms: &[DatasetFormOption],
    version_id: &str,
) -> Option<DatasetFormVersionOption> {
    forms
        .iter()
        .flat_map(|form| form.versions.iter())
        .find(|version| version.id == version_id)
        .cloned()
}

/// Returns source field options for the selected source alias.
pub(crate) fn source_field_options(
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_alias: &str,
) -> Vec<DatasetRenderedField> {
    let Some(source) = sources
        .iter()
        .find(|source| source.source_alias == source_alias)
    else {
        return Vec::new();
    };
    let mut options = system_source_field_options();
    let form_version_id = resolved_form_version_id(source, forms);
    options.extend(
        form_version_id
            .as_deref()
            .and_then(|version_id| rendered_forms.get(version_id))
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

pub(crate) fn source_display_name(
    source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
    datasets: &[DatasetSummary],
) -> String {
    if source.input_kind == "dataset" {
        return datasets
            .iter()
            .find(|dataset| dataset.id == source.dataset_id)
            .map(|dataset| dataset.name.clone())
            .unwrap_or_else(|| "Dataset input".into());
    }

    forms
        .iter()
        .find(|form| form.id == source.form_id)
        .map(|form| form.name.clone())
        .unwrap_or_else(|| "Form input".into())
}

/// Resolves the source's form version id from explicit id, major version, or first published version.
pub(crate) fn resolved_form_version_id(
    source: &DatasetSourceDraft,
    forms: &[DatasetFormOption],
) -> Option<String> {
    if !source.form_version_id.is_empty() {
        return Some(source.form_version_id.clone());
    }
    source
        .form_version_major
        .and_then(|major| {
            published_versions_for_form(forms, &source.form_id)
                .into_iter()
                .find(|version| version.version_major == Some(major))
        })
        .or_else(|| first_published_version(forms, &source.form_id))
        .map(|version| version.id)
}

/// Built-in submission metadata fields available for every form source.
pub(crate) fn system_source_field_options() -> Vec<DatasetRenderedField> {
    [
        ("__submission_id", "Submission ID", "text"),
        ("__form_version_id", "Form Version ID", "text"),
        ("__node_id", "Attached Node ID", "text"),
        ("__node_name", "Attached Node Name", "text"),
        ("__submission_status", "Submission Status", "text"),
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
    })
    .collect()
}
