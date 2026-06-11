//! Editor helper functions for Datasets feature screens.

use super::super::types::*;
use leptos::prelude::*;
use std::collections::BTreeMap;
/// Handles the operation label behavior.
pub(crate) fn operation_label(value: &str) -> &'static str {
    match value {
        "union" => "UNION",
        "union_all" => "UNION ALL",
        "left_join" => "LEFT JOIN",
        "inner_join" => "INNER JOIN",
        "outer_join" => "OUTER JOIN",
        _ => "OPERATION",
    }
}

/// Handles the expression label behavior.
pub(crate) fn expression_label(sources: &[DatasetSourceDraft], operation: &str) -> String {
    let aliases = sources
        .iter()
        .filter(|source| !source.source_alias.trim().is_empty())
        .map(|source| source.source_alias.clone())
        .collect::<Vec<_>>();
    if aliases.is_empty() {
        return "Choose at least one input".into();
    }
    aliases
        .into_iter()
        .reduce(|left, right| format!("({left}) {} ({right})", operation_label(operation)))
        .unwrap_or_else(|| "Choose at least one input".into())
}

/// Handles the expression button class behavior.
pub(crate) fn expression_button_class(is_active: bool, base: &'static str) -> String {
    if is_active {
        format!("{base} is-active")
    } else {
        base.into()
    }
}

/// Handles the field metadata behavior.
pub(crate) fn field_metadata(
    field: &DatasetFieldDraft,
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
) -> DatasetRenderedField {
    source_field_options(sources, forms, rendered_forms, &field.source_alias)
        .into_iter()
        .find(|option| option.key == field.source_field_key)
        .unwrap_or_else(|| DatasetRenderedField {
            key: field.source_field_key.clone(),
            label: "Unknown field".into(),
            field_type: String::new(),
        })
}

/// Handles the confirm action behavior.
pub(crate) fn confirm_action(message: &str) -> bool {
    #[cfg(feature = "hydrate")]
    {
        return web_sys::window()
            .and_then(|window| window.confirm_with_message(message).ok())
            .unwrap_or(false);
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = message;
        true
    }
}

/// Handles the version label behavior.
pub(crate) fn version_label(version: &DatasetFormVersionOption) -> String {
    version
        .version_label
        .clone()
        .unwrap_or_else(|| format!("Major {}", version.version_major.unwrap_or(1)))
}

/// Handles the first published version behavior.
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

/// Handles the published versions for form behavior.
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

/// Handles the find version behavior.
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

/// Handles the source field options behavior.
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

/// Handles the source field options with selected behavior.
pub(crate) fn source_field_options_with_selected(
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_alias: &str,
    selected_key: &str,
) -> Vec<DatasetRenderedField> {
    let mut options = source_field_options(sources, forms, rendered_forms, source_alias);

    if !selected_key.is_empty() && !options.iter().any(|option| option.key == selected_key) {
        options.push(DatasetRenderedField {
            key: selected_key.to_string(),
            label: "Unknown field".into(),
            field_type: String::new(),
        });
    }

    options
}

/// Handles the join key options for source index behavior.
pub(crate) fn join_key_options_for_source_index(
    sources: &[DatasetSourceDraft],
    forms: &[DatasetFormOption],
    rendered_forms: &BTreeMap<String, DatasetRenderedForm>,
    source_index: usize,
    selected_key: &str,
) -> Vec<DatasetRenderedField> {
    let mut options = sources
        .get(source_index)
        .map(|source| source_field_options(sources, forms, rendered_forms, &source.source_alias))
        .unwrap_or_default();

    if !selected_key.is_empty() && !options.iter().any(|option| option.key == selected_key) {
        options.push(DatasetRenderedField {
            key: selected_key.to_string(),
            label: "Unknown field".into(),
            field_type: String::new(),
        });
    }

    options
}

/// Handles the resolved form version id behavior.
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

/// Handles the system source field options behavior.
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

/// Handles the join key option label behavior.
pub(crate) fn join_key_option_label(field: &DatasetRenderedField) -> String {
    format!("{} ({})", truncate_field_label(&field.label), field.key)
}

/// Handles the truncate field label behavior.
pub(crate) fn truncate_field_label(label: &str) -> String {
    const MAX_CHARS: usize = 32;
    let mut chars = label.chars();
    let truncated = chars.by_ref().take(MAX_CHARS).collect::<String>();
    if chars.next().is_some() {
        format!("{truncated}...")
    } else {
        truncated
    }
}

/// Handles the add fields from source behavior.
pub(crate) fn add_fields_from_source(
    index: usize,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    forms: RwSignal<Vec<DatasetFormOption>>,
    rendered_forms: RwSignal<BTreeMap<String, DatasetRenderedForm>>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
) {
    let source = sources.get().get(index).cloned();
    if let Some(source) = source {
        let options = source_field_options(
            &sources.get(),
            &forms.get(),
            &rendered_forms.get(),
            &source.source_alias,
        );
        fields.update(|items| {
            for option in options {
                let key = format!("{}_{}", source.source_alias, option.key);
                if items.iter().any(|item| {
                    item.key == key
                        || (item.source_alias == source.source_alias
                            && item.source_field_key == option.key)
                }) {
                    continue;
                }
                items.push(DatasetFieldDraft {
                    key,
                    label: option.label,
                    source_alias: source.source_alias.clone(),
                    source_field_key: option.key,
                });
            }
        });
    }
}

/// Handles the source seed key behavior.
pub(crate) fn source_seed_key(index: usize, form_version_id: &str) -> String {
    format!("{index}:{form_version_id}")
}
