//! Editor helper functions for Datasets feature screens.

use super::super::types::*;

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

pub(crate) fn expression_label(
    sources: &[DatasetSourceDraft],
    expression: &DatasetExpressionDraft,
) -> String {
    expression_label_inner(sources, expression)
        .unwrap_or_else(|| "Choose at least one input".into())
}

fn expression_label_inner(
    sources: &[DatasetSourceDraft],
    expression: &DatasetExpressionDraft,
) -> Option<String> {
    match expression {
        DatasetExpressionDraft::Source(index) => sources
            .get(*index)
            .map(|source| source.source_alias.trim().to_string())
            .filter(|alias| !alias.is_empty()),
        DatasetExpressionDraft::Operation {
            operation,
            left,
            right,
        } => {
            let left = expression_label_inner(sources, left)?;
            let right = expression_label_inner(sources, right)?;
            Some(format!("({left}) {} ({right})", operation_label(operation)))
        }
    }
}

pub(crate) fn expression_button_class(is_active: bool, base: &'static str) -> String {
    if is_active {
        format!("{base} is-active")
    } else {
        base.into()
    }
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
