//! Expression AST helpers for Datasets.

use super::types::{DatasetExpressionPayload, DatasetJoinKeyPayload, DatasetSourceDraft};

/// Returns whether a dataset composition operation uses join keys.
pub(crate) fn is_join_operation(value: &str) -> bool {
    matches!(value, "left_join" | "inner_join" | "outer_join")
}

#[allow(dead_code)]
/// Converts a dataset expression AST into editor draft state.
pub(crate) fn expression_to_editor_drafts(
    ast: &DatasetExpressionPayload,
) -> (Vec<DatasetSourceDraft>, String, Vec<DatasetJoinKeyPayload>) {
    let mut sources = Vec::new();
    let mut operation = String::new();
    let mut join_keys = Vec::new();
    collect_expression_drafts(ast, &mut sources, &mut operation, &mut join_keys);
    (sources, operation, join_keys)
}

#[allow(dead_code)]
/// Collects source drafts and join metadata from a dataset expression AST.
fn collect_expression_drafts(
    ast: &DatasetExpressionPayload,
    sources: &mut Vec<DatasetSourceDraft>,
    operation: &mut String,
    join_keys: &mut Vec<DatasetJoinKeyPayload>,
) {
    match ast {
        DatasetExpressionPayload::Form {
            alias,
            form_id,
            form_version_major,
            selection_rule,
        } => sources.push(DatasetSourceDraft {
            input_kind: "form".into(),
            source_alias: alias.clone(),
            form_id: form_id.clone(),
            form_version_id: String::new(),
            form_version_major: *form_version_major,
            dataset_id: String::new(),
            dataset_revision_id: String::new(),
            selection_rule: selection_rule.clone(),
        }),
        DatasetExpressionPayload::Dataset {
            alias,
            dataset_id,
            dataset_revision_id,
        } => sources.push(DatasetSourceDraft {
            input_kind: "dataset".into(),
            source_alias: alias.clone(),
            form_id: String::new(),
            form_version_id: String::new(),
            form_version_major: None,
            dataset_id: dataset_id.clone(),
            dataset_revision_id: dataset_revision_id.clone(),
            selection_rule: "latest".into(),
        }),
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
            collect_expression_drafts(left, sources, operation, join_keys);
            collect_expression_drafts(right, sources, operation, join_keys);
        }
    }
}

#[allow(dead_code)]
/// Builds a dataset expression AST from editor source drafts.
pub(crate) fn build_expression_ast(
    sources: &[DatasetSourceDraft],
    operation: &str,
    join_left_key: &str,
    join_right_key: &str,
) -> Option<DatasetExpressionPayload> {
    let mut inputs = sources
        .iter()
        .filter_map(source_expression)
        .collect::<Vec<_>>()
        .into_iter();
    let first = inputs.next()?;
    Some(
        inputs.fold(first, |left, right| DatasetExpressionPayload::Operation {
            alias: "result".into(),
            operation: operation.into(),
            left: Box::new(left),
            right: Box::new(right),
            join_keys: if is_join_operation(operation)
                && !join_left_key.trim().is_empty()
                && !join_right_key.trim().is_empty()
            {
                vec![DatasetJoinKeyPayload {
                    left_field: join_left_key.trim().into(),
                    right_field: join_right_key.trim().into(),
                }]
            } else {
                Vec::new()
            },
        }),
    )
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
            selection_rule: source.selection_rule.clone(),
        })
    }
}
