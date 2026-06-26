//! Payload preparation helpers for dataset mutations.

use super::editor::canonical_field_key;
use super::expressions::source_payload;
use super::types::*;

#[cfg(feature = "hydrate")]
pub(super) struct DatasetPayloadDrafts {
    pub(super) name: String,
    pub(super) slug: String,
    pub(super) visibility_node_ids: Vec<String>,
    pub(super) initial_source: DatasetSourceDraft,
    pub(super) operation_order: Vec<DatasetOperationDraft>,
    pub(super) restriction_internal_field_key: String,
    pub(super) restriction_restricted_field_key: String,
    pub(super) restriction_confidential_field_key: String,
}

#[cfg(feature = "hydrate")]
pub(super) fn dataset_payload_from_drafts(
    drafts: DatasetPayloadDrafts,
) -> Result<DatasetPayload, String> {
    let DatasetPayloadDrafts {
        name,
        slug,
        visibility_node_ids,
        initial_source,
        operation_order,
        restriction_internal_field_key,
        restriction_restricted_field_key,
        restriction_confidential_field_key,
    } = drafts;

    let Some(initial_source) = source_payload(&initial_source) else {
        return Err("Choose at least one complete dataset input before saving.".into());
    };
    let mut operations = Vec::new();
    let mut operation_position = 0;
    for operation in operation_order {
        let next_operation = match operation.kind {
            DatasetOperationDraftKind::AddSource => {
                source_operation_from_draft(&operation, operation_position)
            }
            DatasetOperationDraftKind::Projection => {
                let projection_fields =
                    projection_field_payloads_from_drafts(operation.projection_fields.clone());
                (!projection_fields.is_empty()).then_some(DatasetOperationPayload::Projection {
                    fields: projection_fields,
                    position: operation_position,
                })
            }
            DatasetOperationDraftKind::Aggregation => {
                aggregation_operation_from_draft(operation.aggregation.clone(), operation_position)
            }
            DatasetOperationDraftKind::CalculatedFields => {
                let fields = calculated_field_payloads_from_drafts(operation.calculated_fields);
                (!fields.is_empty()).then_some(DatasetOperationPayload::CalculatedFields {
                    fields,
                    position: operation_position,
                })
            }
            DatasetOperationDraftKind::Filter => {
                let filters = row_filter_payloads_from_drafts(operation.row_filters);
                (!filters.is_empty()).then_some(DatasetOperationPayload::Filter {
                    filters,
                    position: operation_position,
                })
            }
        };
        if let Some(next_operation) = next_operation {
            operations.push(next_operation);
            operation_position += 1;
        }
    }

    Ok(DatasetPayload {
        name,
        slug,
        grain: "submission".into(),
        visibility_node_ids,
        initial_source,
        operations,
        restriction_policy: restriction_policy_payload_from_draft(
            restriction_internal_field_key,
            restriction_restricted_field_key,
            restriction_confidential_field_key,
        ),
    })
}

#[cfg(feature = "hydrate")]
fn projection_field_payloads_from_drafts(
    fields: Vec<DatasetFieldDraft>,
) -> Vec<DatasetProjectionFieldPayload> {
    fields
        .into_iter()
        .enumerate()
        .filter(|(_, field)| {
            !field.key.trim().is_empty()
                && !field.label.trim().is_empty()
                && !field.source_alias.trim().is_empty()
                && !field.source_field_key.trim().is_empty()
        })
        .map(|(index, field)| {
            let input_field_key = projection_input_key(&field);
            DatasetProjectionFieldPayload {
                key: field.key.clone(),
                label: field.label,
                input_field_key: Some(input_field_key),
                position: index as i32,
            }
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn projection_input_key(field: &DatasetFieldDraft) -> String {
    if field.source_alias.trim().is_empty() || field.source_field_key.trim().is_empty() {
        return field.key.clone();
    }
    let canonical_key = canonical_field_key(&field.source_alias, &field.source_field_key);
    if canonical_key.starts_with("aggregation__")
        || canonical_key.starts_with("calculated__")
        || canonical_key.starts_with("projection__")
    {
        field.key.clone()
    } else {
        canonical_key
    }
}

#[cfg(feature = "hydrate")]
fn source_operation_from_draft(
    operation: &DatasetOperationDraft,
    position: i32,
) -> Option<DatasetOperationPayload> {
    let source = source_for_operation(operation)?;
    let add_type = source_add_type(operation);
    let left_field = operation.left_field_key.trim().to_string();
    let right_field = operation.right_field_key.trim().to_string();
    let join_keys = if source_add_type_is_join(&add_type)
        && !left_field.is_empty()
        && !right_field.is_empty()
    {
        vec![DatasetJoinKeyPayload {
            left_field,
            right_field,
        }]
    } else {
        Vec::new()
    };

    Some(DatasetOperationPayload::AddSource {
        source,
        add_type,
        join_keys,
        position,
    })
}

#[cfg(feature = "hydrate")]
fn source_add_type(operation: &DatasetOperationDraft) -> String {
    match operation.add_type.trim() {
        "union" | "union_all" | "left_join" | "inner_join" | "outer_join" => {
            operation.add_type.clone()
        }
        _ => "union".into(),
    }
}

#[cfg(feature = "hydrate")]
fn source_add_type_is_join(add_type: &str) -> bool {
    matches!(add_type, "left_join" | "inner_join" | "outer_join")
}

#[cfg(feature = "hydrate")]
fn source_for_operation(operation: &DatasetOperationDraft) -> Option<DatasetSourcePayload> {
    operation.source.as_ref().and_then(source_payload)
}

#[cfg(feature = "hydrate")]
fn aggregation_operation_from_draft(
    aggregation: DatasetAggregationDraft,
    position: i32,
) -> Option<DatasetOperationPayload> {
    if !aggregation.enabled {
        return None;
    }

    let metrics = aggregation
        .metrics
        .into_iter()
        .enumerate()
        .filter(|(_, metric)| {
            !metric.key.trim().is_empty()
                && !metric.label.trim().is_empty()
                && !metric.function.trim().is_empty()
        })
        .map(|(index, metric)| DatasetAggregationMetricPayload {
            key: metric.key,
            label: metric.label,
            function: metric.function,
            source_field_key: if metric.source_field_key.trim().is_empty() {
                None
            } else {
                Some(metric.source_field_key)
            },
            position: index as i32,
        })
        .collect::<Vec<_>>();
    let row_picker = aggregation.row_picker.and_then(|row_picker| {
        let sort_fields = row_picker
            .sort_fields
            .into_iter()
            .enumerate()
            .filter(|(_, sort)| !sort.field_key.trim().is_empty())
            .map(|(index, sort)| DatasetRowPickerSortPayload {
                field_key: sort.field_key,
                position: index as i32,
            })
            .collect::<Vec<_>>();
        if sort_fields.is_empty() {
            None
        } else {
            Some(DatasetRowPickerPayload {
                sort_fields,
                direction: row_picker.direction,
            })
        }
    });
    let group_fields = aggregation
        .group_fields
        .into_iter()
        .filter(|field| !field.trim().is_empty())
        .collect::<Vec<_>>();
    if group_fields.is_empty() && metrics.is_empty() && row_picker.is_none() {
        None
    } else {
        Some(DatasetOperationPayload::Aggregation {
            group_fields,
            metrics,
            row_picker,
            position,
        })
    }
}

#[cfg(feature = "hydrate")]
fn row_filter_payloads_from_drafts(
    row_filters: Vec<DatasetRowFilterDraft>,
) -> Vec<DatasetRowFilterPayload> {
    row_filters
        .into_iter()
        .enumerate()
        .filter(|(_, filter)| {
            !filter.field_key.trim().is_empty() && !filter.operator.trim().is_empty()
        })
        .map(|(index, filter)| DatasetRowFilterPayload {
            field_key: filter.field_key,
            operator: filter.operator.clone(),
            value_mode: filter.value_mode.clone(),
            value: if matches!(filter.operator.as_str(), "is_empty" | "is_not_empty")
                || filter.value_mode == "field"
            {
                None
            } else {
                Some(filter.value)
            },
            value_field_key: if filter.value_mode == "field"
                && !filter.value_field_key.trim().is_empty()
            {
                Some(filter.value_field_key)
            } else {
                None
            },
            position: index as i32,
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn calculated_field_payloads_from_drafts(
    calculated_fields: Vec<DatasetCalculatedFieldDraft>,
) -> Vec<DatasetCalculatedFieldPayload> {
    calculated_fields
        .into_iter()
        .enumerate()
        .filter(|(_, field)| {
            !field.key.trim().is_empty()
                && !field.label.trim().is_empty()
                && !field.base_field_key.trim().is_empty()
        })
        .map(|(index, field)| DatasetCalculatedFieldPayload {
            key: field.key,
            label: field.label,
            base_field_key: field.base_field_key,
            functions: field
                .functions
                .into_iter()
                .enumerate()
                .filter(|(_, function)| !function.function.trim().is_empty())
                .map(|(function_index, function)| {
                    let argument_mode = if function.argument_mode == "field" {
                        "field".to_string()
                    } else {
                        "value".to_string()
                    };
                    DatasetCalculationFunctionPayload {
                        function: function.function.clone(),
                        argument: if calculation_function_uses_argument(&function.function)
                            && argument_mode == "value"
                            && !function.argument.trim().is_empty()
                        {
                            Some(function.argument)
                        } else {
                            None
                        },
                        argument_mode,
                        argument_field_key: if calculation_function_uses_argument(
                            &function.function,
                        ) && function.argument_mode == "field"
                            && !function.argument_field_key.trim().is_empty()
                        {
                            Some(function.argument_field_key)
                        } else {
                            None
                        },
                        position: function_index as i32,
                    }
                })
                .collect(),
            position: index as i32,
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn restriction_policy_payload_from_draft(
    restriction_internal_field_key: String,
    restriction_restricted_field_key: String,
    restriction_confidential_field_key: String,
) -> Option<DatasetRestrictionPolicyPayload> {
    let internal_field_key = trimmed_optional(restriction_internal_field_key);
    let restricted_field_key = trimmed_optional(restriction_restricted_field_key);
    let confidential_field_key = trimmed_optional(restriction_confidential_field_key);
    if internal_field_key.is_none()
        && restricted_field_key.is_none()
        && confidential_field_key.is_none()
    {
        None
    } else {
        Some(DatasetRestrictionPolicyPayload {
            internal_field_key,
            restricted_field_key,
            confidential_field_key,
        })
    }
}

#[cfg(feature = "hydrate")]
fn trimmed_optional(value: String) -> Option<String> {
    let value = value.trim().to_string();
    (!value.is_empty()).then_some(value)
}

#[cfg(feature = "hydrate")]
fn calculation_function_uses_argument(function: &str) -> bool {
    !matches!(
        function,
        "trim"
            | "uppercase"
            | "lowercase"
            | "to_text"
            | "to_number"
            | "to_boolean"
            | "to_date"
            | "is_empty"
            | "is_not_empty"
    )
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "hydrate")]
    use super::*;

    #[cfg(feature = "hydrate")]
    fn form_source(alias: &str) -> DatasetSourceDraft {
        DatasetSourceDraft {
            source_alias: alias.into(),
            form_id: format!("{alias}-form"),
            form_version_id: format!("{alias}-version"),
            ..DatasetSourceDraft::default()
        }
    }

    #[cfg(feature = "hydrate")]
    fn source_operation_draft(
        id: u64,
        kind: DatasetOperationDraftKind,
        alias: &str,
    ) -> DatasetOperationDraft {
        let mut operation = DatasetOperationDraft::new(id, kind);
        operation.source = Some(form_source(alias));
        operation
    }

    #[cfg(feature = "hydrate")]
    fn drafts_with_operations(operation_order: Vec<DatasetOperationDraft>) -> DatasetPayloadDrafts {
        DatasetPayloadDrafts {
            name: "Ordered Dataset".into(),
            slug: "ordered-dataset".into(),
            visibility_node_ids: Vec::new(),
            initial_source: form_source("program"),
            restriction_internal_field_key: String::new(),
            restriction_restricted_field_key: String::new(),
            restriction_confidential_field_key: String::new(),
            operation_order,
        }
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn dataset_payload_uses_operation_local_transform_state() {
        let mut calculated =
            DatasetOperationDraft::new(1, DatasetOperationDraftKind::CalculatedFields);
        calculated.calculated_fields = vec![DatasetCalculatedFieldDraft {
            id: 1,
            key: "calculated_1".into(),
            label: "Calculated 1".into(),
            base_field_key: "program__participant_target".into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id: 1,
                function: "greater_than".into(),
                argument: "10".into(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }];
        let mut projection = DatasetOperationDraft::new(2, DatasetOperationDraftKind::Projection);
        projection.projection_fields = vec![DatasetFieldDraft {
            key: "program__participant_target".into(),
            label: "Participant Target".into(),
            source_alias: "program".into(),
            source_field_key: "__participant_target".into(),
            field_type: "number".into(),
        }];
        let mut filter = DatasetOperationDraft::new(3, DatasetOperationDraftKind::Filter);
        filter.row_filters = vec![DatasetRowFilterDraft {
            id: 1,
            field_key: "calculated_1".into(),
            operator: "equals".into(),
            value: "true".into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }];

        let payload = dataset_payload_from_drafts(drafts_with_operations(vec![
            calculated, projection, filter,
        ]))
        .expect("payload");

        assert_eq!(payload.operations.len(), 3);
        assert!(matches!(
            payload.operations[0],
            DatasetOperationPayload::CalculatedFields { position: 0, .. }
        ));
        assert!(matches!(
            payload.operations[1],
            DatasetOperationPayload::Projection { position: 1, .. }
        ));
        assert!(matches!(
            payload.operations[2],
            DatasetOperationPayload::Filter { position: 2, .. }
        ));
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn projection_payload_uses_canonical_input_key_when_output_is_renamed() {
        let mut projection = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection);
        projection.projection_fields = vec![DatasetFieldDraft {
            key: "participant_target".into(),
            label: "Participant Target".into(),
            source_alias: "program".into(),
            source_field_key: "__participant_target".into(),
            field_type: "number".into(),
        }];

        let payload =
            dataset_payload_from_drafts(drafts_with_operations(vec![projection])).expect("payload");

        let DatasetOperationPayload::Projection { fields, .. } = &payload.operations[0] else {
            panic!("expected projection operation");
        };
        assert_eq!(fields[0].key, "participant_target");
        assert_eq!(
            fields[0].input_field_key.as_deref(),
            Some("program__participant_target")
        );
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn projection_payload_uses_current_key_for_derived_fields() {
        let mut projection = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Projection);
        projection.projection_fields = vec![
            DatasetFieldDraft {
                key: "target_count".into(),
                label: "Target Count".into(),
                source_alias: "aggregation".into(),
                source_field_key: "program__participant_target".into(),
                field_type: "number".into(),
            },
            DatasetFieldDraft {
                key: "status_label".into(),
                label: "Status Label".into(),
                source_alias: "calculated".into(),
                source_field_key: "program__submission_status".into(),
                field_type: "text".into(),
            },
        ];

        let payload =
            dataset_payload_from_drafts(drafts_with_operations(vec![projection])).expect("payload");

        let DatasetOperationPayload::Projection { fields, .. } = &payload.operations[0] else {
            panic!("expected projection operation");
        };
        assert_eq!(fields[0].input_field_key.as_deref(), Some("target_count"));
        assert_eq!(fields[1].input_field_key.as_deref(), Some("status_label"));
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn empty_operation_order_serializes_without_implicit_fixed_panels() {
        let payload =
            dataset_payload_from_drafts(drafts_with_operations(Vec::new())).expect("payload");

        assert!(payload.operations.is_empty());
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn dataset_payload_uses_explicit_source_operation_order() {
        let mut join = source_operation_draft(1, DatasetOperationDraftKind::AddSource, "program2");
        join.add_type = "inner_join".into();
        join.left_field_key = "program__submission_id".into();
        join.right_field_key = "program2__submission_id".into();

        let mut drafts = drafts_with_operations(vec![join, {
            let mut union =
                source_operation_draft(2, DatasetOperationDraftKind::AddSource, "program3");
            union.add_type = "union_all".into();
            union
        }]);
        drafts.name = "Source Ordered Dataset".into();
        drafts.slug = "source-ordered-dataset".into();
        drafts.initial_source = form_source("program");

        let payload = dataset_payload_from_drafts(drafts).expect("payload");

        assert_eq!(payload.operations.len(), 2);
        assert!(matches!(
            payload.operations[0],
            DatasetOperationPayload::AddSource {
                ref add_type,
                position: 0,
                ..
            } if add_type == "inner_join"
        ));
        assert!(matches!(
            payload.operations[1],
            DatasetOperationPayload::AddSource {
                ref add_type,
                position: 1,
                ..
            } if add_type == "union_all"
        ));
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn multiple_aggregation_operations_keep_separate_configuration() {
        let mut by_status = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Aggregation);
        by_status.aggregation = DatasetAggregationDraft {
            enabled: true,
            group_fields: vec!["program__submission_status".into()],
            metrics: vec![DatasetAggregationMetricDraft {
                id: 1,
                key: "count_rows".into(),
                label: "Count Rows".into(),
                function: "count_rows".into(),
                source_field_key: String::new(),
            }],
            row_picker: None,
        };
        let mut by_target = DatasetOperationDraft::new(2, DatasetOperationDraftKind::Aggregation);
        by_target.aggregation = DatasetAggregationDraft {
            enabled: true,
            group_fields: vec!["program__participant_target".into()],
            metrics: vec![DatasetAggregationMetricDraft {
                id: 1,
                key: "max_target".into(),
                label: "Max Target".into(),
                function: "max".into(),
                source_field_key: "program__participant_target".into(),
            }],
            row_picker: None,
        };

        let payload =
            dataset_payload_from_drafts(drafts_with_operations(vec![by_status, by_target]))
                .expect("payload");

        assert_eq!(payload.operations.len(), 2);
        let DatasetOperationPayload::Aggregation {
            group_fields,
            metrics,
            position,
            ..
        } = &payload.operations[0]
        else {
            panic!("expected first aggregation");
        };
        assert_eq!(*position, 0);
        assert_eq!(
            group_fields,
            &vec!["program__submission_status".to_string()]
        );
        assert_eq!(metrics[0].key, "count_rows");

        let DatasetOperationPayload::Aggregation {
            group_fields,
            metrics,
            position,
            ..
        } = &payload.operations[1]
        else {
            panic!("expected second aggregation");
        };
        assert_eq!(*position, 1);
        assert_eq!(
            group_fields,
            &vec!["program__participant_target".to_string()]
        );
        assert_eq!(metrics[0].key, "max_target");
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn multiple_filters_keep_separate_configuration_and_reorder_with_panel() {
        let mut status_filter = DatasetOperationDraft::new(1, DatasetOperationDraftKind::Filter);
        status_filter.row_filters = vec![DatasetRowFilterDraft {
            id: 1,
            field_key: "program__submission_status".into(),
            operator: "equals".into(),
            value: "submitted".into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }];
        let mut target_filter = DatasetOperationDraft::new(2, DatasetOperationDraftKind::Filter);
        target_filter.row_filters = vec![DatasetRowFilterDraft {
            id: 1,
            field_key: "program__participant_target".into(),
            operator: "greater_than_or_equal".into(),
            value: "25".into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }];

        let payload =
            dataset_payload_from_drafts(drafts_with_operations(vec![target_filter, status_filter]))
                .expect("payload");

        let DatasetOperationPayload::Filter {
            filters, position, ..
        } = &payload.operations[0]
        else {
            panic!("expected first filter");
        };
        assert_eq!(*position, 0);
        assert_eq!(filters[0].field_key, "program__participant_target");
        assert_eq!(filters[0].value.as_deref(), Some("25"));

        let DatasetOperationPayload::Filter {
            filters, position, ..
        } = &payload.operations[1]
        else {
            panic!("expected second filter");
        };
        assert_eq!(*position, 1);
        assert_eq!(filters[0].field_key, "program__submission_status");
        assert_eq!(filters[0].value.as_deref(), Some("submitted"));
    }

    #[cfg(feature = "hydrate")]
    #[test]
    fn multiple_calculated_field_operations_keep_separate_function_chains() {
        let mut text_calc =
            DatasetOperationDraft::new(1, DatasetOperationDraftKind::CalculatedFields);
        text_calc.calculated_fields = vec![DatasetCalculatedFieldDraft {
            id: 1,
            key: "status_label".into(),
            label: "Status Label".into(),
            base_field_key: "program__submission_status".into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id: 1,
                function: "uppercase".into(),
                argument: String::new(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }];
        let mut numeric_calc =
            DatasetOperationDraft::new(2, DatasetOperationDraftKind::CalculatedFields);
        numeric_calc.calculated_fields = vec![DatasetCalculatedFieldDraft {
            id: 1,
            key: "target_plus_one".into(),
            label: "Target Plus One".into(),
            base_field_key: "program__participant_target".into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id: 1,
                function: "add".into(),
                argument: "1".into(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }];

        let payload =
            dataset_payload_from_drafts(drafts_with_operations(vec![text_calc, numeric_calc]))
                .expect("payload");

        let DatasetOperationPayload::CalculatedFields { fields, .. } = &payload.operations[0]
        else {
            panic!("expected first calculated fields operation");
        };
        assert_eq!(fields[0].key, "status_label");
        assert_eq!(fields[0].functions[0].function, "uppercase");

        let DatasetOperationPayload::CalculatedFields { fields, .. } = &payload.operations[1]
        else {
            panic!("expected second calculated fields operation");
        };
        assert_eq!(fields[0].key, "target_plus_one");
        assert_eq!(fields[0].functions[0].function, "add");
        assert_eq!(fields[0].functions[0].argument.as_deref(), Some("1"));
    }

    #[test]
    fn repeated_operation_configs_move_with_reordered_panels() {
        let mut status_aggregation =
            DatasetOperationDraft::new(1, DatasetOperationDraftKind::Aggregation);
        status_aggregation.aggregation = DatasetAggregationDraft {
            enabled: true,
            group_fields: vec!["program__submission_status".into()],
            metrics: vec![DatasetAggregationMetricDraft {
                id: 1,
                key: "status_count".into(),
                label: "Status Count".into(),
                function: "count_rows".into(),
                source_field_key: String::new(),
            }],
            row_picker: None,
        };
        let mut target_aggregation =
            DatasetOperationDraft::new(2, DatasetOperationDraftKind::Aggregation);
        target_aggregation.aggregation = DatasetAggregationDraft {
            enabled: true,
            group_fields: vec!["program__participant_target".into()],
            metrics: vec![DatasetAggregationMetricDraft {
                id: 1,
                key: "target_max".into(),
                label: "Target Max".into(),
                function: "max".into(),
                source_field_key: "program__participant_target".into(),
            }],
            row_picker: None,
        };
        let mut status_filter = DatasetOperationDraft::new(3, DatasetOperationDraftKind::Filter);
        status_filter.row_filters = vec![DatasetRowFilterDraft {
            id: 1,
            field_key: "program__submission_status".into(),
            operator: "equals".into(),
            value: "submitted".into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }];
        let mut target_filter = DatasetOperationDraft::new(4, DatasetOperationDraftKind::Filter);
        target_filter.row_filters = vec![DatasetRowFilterDraft {
            id: 1,
            field_key: "program__participant_target".into(),
            operator: "greater_than_or_equal".into(),
            value: "25".into(),
            value_mode: "value".into(),
            value_field_key: String::new(),
        }];
        let mut status_calculation =
            DatasetOperationDraft::new(5, DatasetOperationDraftKind::CalculatedFields);
        status_calculation.calculated_fields = vec![DatasetCalculatedFieldDraft {
            id: 1,
            key: "status_upper".into(),
            label: "Status Upper".into(),
            base_field_key: "program__submission_status".into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id: 1,
                function: "uppercase".into(),
                argument: String::new(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }];
        let mut target_calculation =
            DatasetOperationDraft::new(6, DatasetOperationDraftKind::CalculatedFields);
        target_calculation.calculated_fields = vec![DatasetCalculatedFieldDraft {
            id: 1,
            key: "target_plus_one".into(),
            label: "Target Plus One".into(),
            base_field_key: "program__participant_target".into(),
            functions: vec![DatasetCalculationFunctionDraft {
                id: 1,
                function: "add".into(),
                argument: "1".into(),
                argument_mode: "value".into(),
                argument_field_key: String::new(),
            }],
        }];

        let payload = dataset_payload_from_drafts(drafts_with_operations(vec![
            target_filter,
            target_aggregation,
            target_calculation,
            status_filter,
            status_aggregation,
            status_calculation,
        ]))
        .expect("payload");

        assert_eq!(payload.operations.len(), 6);
        let DatasetOperationPayload::Filter {
            filters, position, ..
        } = &payload.operations[0]
        else {
            panic!("expected reordered target filter");
        };
        assert_eq!(*position, 0);
        assert_eq!(filters[0].field_key, "program__participant_target");
        assert_eq!(filters[0].value.as_deref(), Some("25"));

        let DatasetOperationPayload::Aggregation {
            group_fields,
            metrics,
            position,
            ..
        } = &payload.operations[1]
        else {
            panic!("expected reordered target aggregation");
        };
        assert_eq!(*position, 1);
        assert_eq!(
            group_fields,
            &vec!["program__participant_target".to_string()]
        );
        assert_eq!(metrics[0].key, "target_max");

        let DatasetOperationPayload::CalculatedFields {
            fields, position, ..
        } = &payload.operations[2]
        else {
            panic!("expected reordered target calculation");
        };
        assert_eq!(*position, 2);
        assert_eq!(fields[0].key, "target_plus_one");
        assert_eq!(fields[0].functions[0].function, "add");

        let DatasetOperationPayload::Filter {
            filters, position, ..
        } = &payload.operations[3]
        else {
            panic!("expected reordered status filter");
        };
        assert_eq!(*position, 3);
        assert_eq!(filters[0].field_key, "program__submission_status");
        assert_eq!(filters[0].value.as_deref(), Some("submitted"));

        let DatasetOperationPayload::Aggregation {
            group_fields,
            metrics,
            position,
            ..
        } = &payload.operations[4]
        else {
            panic!("expected reordered status aggregation");
        };
        assert_eq!(*position, 4);
        assert_eq!(
            group_fields,
            &vec!["program__submission_status".to_string()]
        );
        assert_eq!(metrics[0].key, "status_count");

        let DatasetOperationPayload::CalculatedFields {
            fields, position, ..
        } = &payload.operations[5]
        else {
            panic!("expected reordered status calculation");
        };
        assert_eq!(*position, 5);
        assert_eq!(fields[0].key, "status_upper");
        assert_eq!(fields[0].functions[0].function, "uppercase");
    }
}
