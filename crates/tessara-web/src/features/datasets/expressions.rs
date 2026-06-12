//! Expression AST helpers for Datasets.

use super::types::{
    DatasetExpressionDraft, DatasetExpressionPayload, DatasetJoinKeyPayload, DatasetSourceDraft,
};

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
            selection_rule,
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
                selection_rule: selection_rule.clone(),
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
                selection_rule: "latest".into(),
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
    expression: &DatasetExpressionDraft,
    operation: &str,
    join_left_key: &str,
    join_right_key: &str,
) -> Option<DatasetExpressionPayload> {
    expression_to_payload(
        expression,
        sources,
        operation,
        join_left_key.trim(),
        join_right_key.trim(),
    )
}

fn expression_to_payload(
    expression: &DatasetExpressionDraft,
    sources: &[DatasetSourceDraft],
    operation: &str,
    join_left_key: &str,
    join_right_key: &str,
) -> Option<DatasetExpressionPayload> {
    match expression {
        DatasetExpressionDraft::Source(index) => sources.get(*index).and_then(source_expression),
        DatasetExpressionDraft::Operation { left, right } => {
            let left =
                expression_to_payload(left, sources, operation, join_left_key, join_right_key)?;
            let right =
                expression_to_payload(right, sources, operation, join_left_key, join_right_key)?;
            Some(DatasetExpressionPayload::Operation {
                alias: "result".into(),
                operation: operation.into(),
                left: Box::new(left),
                right: Box::new(right),
                join_keys: if is_join_operation(operation)
                    && !join_left_key.is_empty()
                    && !join_right_key.is_empty()
                {
                    vec![DatasetJoinKeyPayload {
                        left_field: join_left_key.into(),
                        right_field: join_right_key.into(),
                    }]
                } else {
                    Vec::new()
                },
            })
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
            selection_rule: source.selection_rule.clone(),
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
            left: Box::new(DatasetExpressionDraft::Source(0)),
            right: Box::new(DatasetExpressionDraft::Operation {
                left: Box::new(DatasetExpressionDraft::Source(1)),
                right: Box::new(DatasetExpressionDraft::Source(2)),
            }),
        };

        let Some(DatasetExpressionPayload::Operation { left, right, .. }) =
            build_expression_ast(&sources, &expression, "union", "", "")
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
}
