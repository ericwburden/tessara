//! Expression AST helpers for Datasets.

use super::types::{
    DatasetExpressionDraft, DatasetExpressionPayload, DatasetFieldDraft, DatasetJoinKeyPayload,
    DatasetSourceDraft,
};
use std::collections::BTreeSet;

/// Returns whether a dataset composition operation uses join keys.
pub(crate) fn is_join_operation(value: &str) -> bool {
    matches!(value, "left_join" | "inner_join" | "outer_join")
}

#[allow(dead_code)]
/// Converts a dataset expression AST into editor draft state.
pub(crate) fn expression_to_editor_drafts(
    ast: &DatasetExpressionPayload,
) -> (
    Vec<DatasetSourceDraft>,
    DatasetExpressionDraft,
    String,
    Vec<DatasetJoinKeyPayload>,
) {
    let mut sources = Vec::new();
    let mut operation = String::new();
    let mut join_keys = Vec::new();
    let expression = collect_expression_drafts(ast, &mut sources, &mut operation, &mut join_keys)
        .unwrap_or_default();
    (sources, expression, operation, join_keys)
}

#[allow(dead_code)]
/// Collects source drafts and join metadata from a dataset expression AST.
fn collect_expression_drafts(
    ast: &DatasetExpressionPayload,
    sources: &mut Vec<DatasetSourceDraft>,
    operation: &mut String,
    join_keys: &mut Vec<DatasetJoinKeyPayload>,
) -> Option<DatasetExpressionDraft> {
    match ast {
        DatasetExpressionPayload::Form {
            alias,
            form_id,
            form_version_major,
        } => {
            let index = sources.len();
            sources.push(DatasetSourceDraft {
                input_kind: "form".into(),
                source_alias: alias.clone(),
                form_id: form_id.clone(),
                form_version_id: String::new(),
                form_version_major: *form_version_major,
                dataset_id: String::new(),
                dataset_revision_id: String::new(),
            });
            Some(DatasetExpressionDraft::Source(index))
        }
        DatasetExpressionPayload::Dataset {
            alias,
            dataset_id,
            dataset_revision_id,
        } => {
            let index = sources.len();
            sources.push(DatasetSourceDraft {
                input_kind: "dataset".into(),
                source_alias: alias.clone(),
                form_id: String::new(),
                form_version_id: String::new(),
                form_version_major: None,
                dataset_id: dataset_id.clone(),
                dataset_revision_id: dataset_revision_id.clone(),
            });
            Some(DatasetExpressionDraft::Source(index))
        }
        DatasetExpressionPayload::Operation {
            operation: node_operation,
            left,
            right,
            join_keys: node_join_keys,
            ..
        } => {
            if operation.is_empty() {
                *operation = node_operation.clone();
                *join_keys = node_join_keys.clone();
            }
            Some(DatasetExpressionDraft::Operation {
                operation: node_operation.clone(),
                left: Box::new(collect_expression_drafts(
                    left, sources, operation, join_keys,
                )?),
                right: Box::new(collect_expression_drafts(
                    right, sources, operation, join_keys,
                )?),
            })
        }
    }
}

#[allow(dead_code)]
/// Builds a dataset expression AST from editor source drafts.
pub(crate) fn build_expression_ast(
    sources: &[DatasetSourceDraft],
    _fields: &[DatasetFieldDraft],
    expression: &DatasetExpressionDraft,
    join_left_key: &str,
    join_right_key: &str,
) -> Option<DatasetExpressionPayload> {
    expression_to_payload(
        expression,
        sources,
        join_left_key.trim(),
        join_right_key.trim(),
    )
}

fn expression_to_payload(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
    join_left_key: &str,
    join_right_key: &str,
) -> Option<DatasetExpressionPayload> {
    match expression {
        DatasetExpressionDraft::Source(index) => sources.get(*index).and_then(source_expression),
        DatasetExpressionDraft::Operation {
            operation,
            left,
            right,
        } => {
            if operation.trim().is_empty() {
                return None;
            }
            let left_key = normalize_join_key_for_side(left, sources, join_left_key);
            let right_key = normalize_join_key_for_side(right, sources, join_right_key);
            let left = expression_to_payload(left, sources, join_left_key, join_right_key)?;
            let right = expression_to_payload(right, sources, join_left_key, join_right_key)?;
            Some(DatasetExpressionPayload::Operation {
                alias: "result".into(),
                operation: operation.into(),
                left: Box::new(left),
                right: Box::new(right),
                join_keys: if is_join_operation(operation)
                    && left_key.as_ref().is_some_and(|key| !key.is_empty())
                    && right_key.as_ref().is_some_and(|key| !key.is_empty())
                {
                    vec![DatasetJoinKeyPayload {
                        left_field: left_key.unwrap_or_default(),
                        right_field: right_key.unwrap_or_default(),
                    }]
                } else {
                    Vec::new()
                },
            })
        }
    }
}

fn normalize_join_key_for_side(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
    requested_key: &str,
) -> Option<String> {
    let requested_key = requested_key.trim();
    if requested_key.is_empty() {
        return None;
    }

    let source_aliases = source_aliases_for_expression(expression, sources);
    if source_aliases
        .iter()
        .any(|alias| requested_key.starts_with(&format!("{alias}__")))
    {
        return Some(requested_key.to_string());
    }

    if source_aliases.len() == 1 {
        let alias = source_aliases.iter().next()?;
        return Some(canonical_source_field_key(alias, requested_key));
    }

    Some(requested_key.to_string())
}

fn source_aliases_for_expression(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
) -> BTreeSet<String> {
    let mut aliases = BTreeSet::new();
    collect_source_aliases(expression, sources, &mut aliases);
    aliases
}

fn collect_source_aliases(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
    aliases: &mut BTreeSet<String>,
) {
    match expression {
        DatasetExpressionDraft::Source(index) => {
            if let Some(source) = sources.get(*index) {
                aliases.insert(source.source_alias.clone());
            }
        }
        DatasetExpressionDraft::Operation { left, right, .. } => {
            collect_source_aliases(left, sources, aliases);
            collect_source_aliases(right, sources, aliases);
        }
    }
}

fn canonical_source_field_key(source_alias: &str, source_field_key: &str) -> String {
    let field_key = source_field_key.trim_start_matches('_');
    if field_key.is_empty() {
        source_alias.into()
    } else {
        format!("{source_alias}__{field_key}")
    }
}

#[allow(dead_code)]
pub(crate) fn root_expression_operation(expression: &DatasetExpressionDraft) -> Option<String> {
    match expression {
        DatasetExpressionDraft::Operation { operation, .. } if !operation.trim().is_empty() => {
            Some(operation.clone())
        }
        _ => None,
    }
}

#[allow(dead_code)]
pub(crate) fn expression_uses_join(expression: &DatasetExpressionDraft) -> bool {
    match expression {
        DatasetExpressionDraft::Source(_) => false,
        DatasetExpressionDraft::Operation {
            operation,
            left,
            right,
        } => {
            is_join_operation(operation)
                || expression_uses_join(left)
                || expression_uses_join(right)
        }
    }
}

#[allow(dead_code)]
/// Converts a source draft into a dataset expression leaf.
fn source_expression(source: &DatasetSourceDraft) -> Option<DatasetExpressionPayload> {
    if source.source_alias.trim().is_empty() {
        return None;
    }
    if source.input_kind == "dataset" {
        if source.dataset_id.is_empty() || source.dataset_revision_id.is_empty() {
            return None;
        }
        Some(DatasetExpressionPayload::Dataset {
            alias: source.source_alias.clone(),
            dataset_id: source.dataset_id.clone(),
            dataset_revision_id: source.dataset_revision_id.clone(),
        })
    } else {
        if source.form_id.is_empty() {
            return None;
        }
        Some(DatasetExpressionPayload::Form {
            alias: source.source_alias.clone(),
            form_id: source.form_id.clone(),
            form_version_major: source.form_version_major,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn form_source(alias: &str, form_id: &str) -> DatasetSourceDraft {
        DatasetSourceDraft {
            source_alias: alias.into(),
            form_id: form_id.into(),
            form_version_major: Some(1),
            ..DatasetSourceDraft::default()
        }
    }

    #[test]
    fn build_expression_ast_preserves_nested_source_expression() {
        let sources = vec![
            form_source("program", "program-form"),
            form_source("source_2", "source-2-form"),
            form_source("source_3", "source-3-form"),
        ];
        let expression = DatasetExpressionDraft::Operation {
            operation: "union".into(),
            left: Box::new(DatasetExpressionDraft::Source(0)),
            right: Box::new(DatasetExpressionDraft::Operation {
                operation: "union".into(),
                left: Box::new(DatasetExpressionDraft::Source(1)),
                right: Box::new(DatasetExpressionDraft::Source(2)),
            }),
        };

        let Some(DatasetExpressionPayload::Operation { left, right, .. }) =
            build_expression_ast(&sources, &[], &expression, "", "")
        else {
            panic!("expected root operation");
        };

        assert!(matches!(
            left.as_ref(),
            DatasetExpressionPayload::Form { alias, .. } if alias == "program"
        ));
        let DatasetExpressionPayload::Operation {
            left: nested_left,
            right: nested_right,
            ..
        } = right.as_ref()
        else {
            panic!("expected nested right operation");
        };
        assert!(matches!(
            nested_left.as_ref(),
            DatasetExpressionPayload::Form { alias, .. } if alias == "source_2"
        ));
        assert!(matches!(
            nested_right.as_ref(),
            DatasetExpressionPayload::Form { alias, .. } if alias == "source_3"
        ));
    }

    #[test]
    fn build_expression_ast_preserves_independent_operation_types() {
        let sources = vec![
            form_source("program", "program-form"),
            form_source("source_2", "source-2-form"),
            form_source("source_3", "source-3-form"),
        ];
        let expression = DatasetExpressionDraft::Operation {
            operation: "union".into(),
            left: Box::new(DatasetExpressionDraft::Source(0)),
            right: Box::new(DatasetExpressionDraft::Operation {
                operation: "union_all".into(),
                left: Box::new(DatasetExpressionDraft::Source(1)),
                right: Box::new(DatasetExpressionDraft::Source(2)),
            }),
        };

        let Some(DatasetExpressionPayload::Operation {
            operation, right, ..
        }) = build_expression_ast(&sources, &[], &expression, "", "")
        else {
            panic!("expected root operation");
        };

        assert_eq!(operation, "union");
        let DatasetExpressionPayload::Operation {
            operation: nested_operation,
            ..
        } = right.as_ref()
        else {
            panic!("expected nested right operation");
        };
        assert_eq!(nested_operation, "union_all");
    }

    #[test]
    fn build_expression_ast_normalizes_raw_join_keys_to_source_qualified_fields() {
        let sources = vec![
            form_source("program1", "program-1-form"),
            form_source("program2", "program-2-form"),
        ];
        let expression = DatasetExpressionDraft::Operation {
            operation: "inner_join".into(),
            left: Box::new(DatasetExpressionDraft::Source(0)),
            right: Box::new(DatasetExpressionDraft::Source(1)),
        };

        let Some(DatasetExpressionPayload::Operation { join_keys, .. }) =
            build_expression_ast(&sources, &[], &expression, "__node_id", "__node_id")
        else {
            panic!("expected root operation");
        };

        assert_eq!(
            join_keys,
            vec![DatasetJoinKeyPayload {
                left_field: "program1__node_id".into(),
                right_field: "program2__node_id".into(),
            }]
        );
    }
}
