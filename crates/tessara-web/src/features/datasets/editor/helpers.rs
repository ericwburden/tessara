//! Editor helper functions for Datasets feature screens.

use super::super::types::*;
use std::collections::BTreeMap;

use super::source_options::source_field_options;

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

pub(crate) fn expression_button_class(is_active: bool, base: &'static str) -> String {
    if is_active {
        format!("{base} is-active")
    } else {
        base.into()
    }
}

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

pub(crate) fn version_label(version: &DatasetFormVersionOption) -> String {
    version
        .version_label
        .clone()
        .unwrap_or_else(|| format!("Major {}", version.version_major.unwrap_or(1)))
}

pub(crate) fn join_key_option_label(field: &DatasetRenderedField) -> String {
    format!("{} ({})", truncate_field_label(&field.label), field.key)
}

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

pub(crate) fn source_seed_key(index: usize, form_version_id: &str) -> String {
    format!("{index}:{form_version_id}")
}
