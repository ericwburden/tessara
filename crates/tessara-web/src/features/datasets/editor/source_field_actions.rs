//! Field reference helpers for dataset editor sources.

use crate::features::datasets::types::{DatasetFieldDraft, DatasetOperationDraft};
use leptos::prelude::*;

pub(crate) fn canonical_field_key(source_alias: &str, source_field_key: &str) -> String {
    let field_key = source_field_key.trim_start_matches('_');
    if field_key.is_empty() {
        source_alias.into()
    } else {
        format!("{source_alias}__{field_key}")
    }
}

pub(crate) fn rename_source_alias_references(
    old_alias: &str,
    new_alias: &str,
    operation_order: RwSignal<Vec<DatasetOperationDraft>>,
) {
    if old_alias == new_alias {
        return;
    }

    operation_order.update(|operations| {
        rename_source_alias_references_in_operations(old_alias, new_alias, operations);
    });
}

fn rename_source_alias_references_in_operations(
    old_alias: &str,
    new_alias: &str,
    operations: &mut [DatasetOperationDraft],
) {
    for operation in operations {
        rename_fields(&mut operation.projection_fields, old_alias, new_alias);
        rename_aggregation_fields(operation, old_alias, new_alias);
        rename_calculation_fields(operation, old_alias, new_alias);
        rename_filter_fields(operation, old_alias, new_alias);
        rename_field_reference(&mut operation.left_field_key, old_alias, new_alias);
        rename_field_reference(&mut operation.right_field_key, old_alias, new_alias);
    }
}

fn rename_fields(fields: &mut [DatasetFieldDraft], old_alias: &str, new_alias: &str) {
    for field in fields {
        rename_projection_output_key(field, old_alias, new_alias);
        if field.source_alias == old_alias {
            field.source_alias = new_alias.to_string();
        }
    }
}

fn rename_aggregation_fields(
    operation: &mut DatasetOperationDraft,
    old_alias: &str,
    new_alias: &str,
) {
    for group_field in &mut operation.aggregation.group_fields {
        rename_field_reference(group_field, old_alias, new_alias);
    }
    for metric in &mut operation.aggregation.metrics {
        rename_field_reference(&mut metric.source_field_key, old_alias, new_alias);
    }
    if let Some(row_picker) = &mut operation.aggregation.row_picker {
        for sort_field in &mut row_picker.sort_fields {
            rename_field_reference(&mut sort_field.field_key, old_alias, new_alias);
        }
    }
}

fn rename_calculation_fields(
    operation: &mut DatasetOperationDraft,
    old_alias: &str,
    new_alias: &str,
) {
    for calculation in &mut operation.calculated_fields {
        rename_field_reference(&mut calculation.base_field_key, old_alias, new_alias);
        for function in &mut calculation.functions {
            rename_field_reference(&mut function.argument_field_key, old_alias, new_alias);
        }
    }
}

fn rename_filter_fields(operation: &mut DatasetOperationDraft, old_alias: &str, new_alias: &str) {
    for filter in &mut operation.row_filters {
        rename_field_reference(&mut filter.field_key, old_alias, new_alias);
        rename_field_reference(&mut filter.value_field_key, old_alias, new_alias);
    }
}

fn rename_projection_output_key(field: &mut DatasetFieldDraft, old_alias: &str, new_alias: &str) {
    let old_canonical = canonical_field_key(old_alias, &field.source_field_key);
    if field.key == old_canonical {
        field.key = canonical_field_key(new_alias, &field.source_field_key);
    }
}

fn rename_field_reference(field_key: &mut String, old_alias: &str, new_alias: &str) {
    let Some(source_field_key) = field_key.strip_prefix(&format!("{old_alias}__")) else {
        return;
    };
    *field_key = canonical_field_key(new_alias, source_field_key);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::datasets::types::{
        DatasetAggregationDraft, DatasetAggregationMetricDraft, DatasetOperationDraftKind,
    };

    #[test]
    fn alias_rename_updates_operation_references_without_global_fields() {
        let mut operations = vec![{
            let mut operation =
                DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection);
            operation.projection_fields = vec![
                DatasetFieldDraft {
                    key: "program__participant_target".into(),
                    label: "Participant Target".into(),
                    source_alias: "program".into(),
                    source_field_key: "participant_target".into(),
                    field_type: "number".into(),
                },
                DatasetFieldDraft {
                    key: "target".into(),
                    label: "Target".into(),
                    source_alias: "program".into(),
                    source_field_key: "participant_target".into(),
                    field_type: "number".into(),
                },
            ];
            operation.aggregation = DatasetAggregationDraft {
                enabled: true,
                group_fields: vec!["program__node_name".into()],
                metrics: vec![DatasetAggregationMetricDraft {
                    id: 1,
                    function: "sum".into(),
                    source_field_key: "program__participant_target".into(),
                    key: "target_sum".into(),
                    label: "Target Sum".into(),
                }],
                row_picker: None,
            };
            operation
        }];

        rename_source_alias_references_in_operations("program", "primary", &mut operations);
        let operation = operations.remove(0);

        assert_eq!(
            operation.projection_fields[0].key,
            "primary__participant_target"
        );
        assert_eq!(operation.projection_fields[0].source_alias, "primary");
        assert_eq!(operation.projection_fields[1].key, "target");
        assert_eq!(operation.projection_fields[1].source_alias, "primary");
        assert_eq!(
            operation.aggregation.group_fields,
            vec!["primary__node_name"]
        );
        assert_eq!(
            operation.aggregation.metrics[0].source_field_key,
            "primary__participant_target"
        );
    }

    #[test]
    fn joined_source_alias_rename_updates_projection_and_join_references() {
        let mut operations = vec![
            {
                let mut operation =
                    DatasetOperationDraft::new(1, DatasetOperationDraftKind::AddSource);
                operation.add_type = "left_join".into();
                operation.left_field_key = "program__node_id".into();
                operation.right_field_key = "source_2__node_id".into();
                operation
            },
            {
                let mut operation =
                    DatasetOperationDraft::new(2, DatasetOperationDraftKind::Projection);
                operation.projection_fields = vec![DatasetFieldDraft {
                    key: "source_2__submitted_at".into(),
                    label: "Submitted Date".into(),
                    source_alias: "source_2".into(),
                    source_field_key: "submitted_at".into(),
                    field_type: "date".into(),
                }];
                operation
            },
        ];

        rename_source_alias_references_in_operations("source_2", "session", &mut operations);

        assert_eq!(operations[0].left_field_key, "program__node_id");
        assert_eq!(operations[0].right_field_key, "session__node_id");
        assert_eq!(
            operations[1].projection_fields[0].key,
            "session__submitted_at"
        );
        assert_eq!(operations[1].projection_fields[0].source_alias, "session");
    }
}
