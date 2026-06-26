//! Edit-hydration loader for the Datasets feature.

#[cfg(feature = "hydrate")]
use super::super::api;
#[cfg(feature = "hydrate")]
use super::super::expressions::source_payload_to_draft;
#[cfg(feature = "hydrate")]
use super::super::types::DatasetFieldDraft;
#[cfg(feature = "hydrate")]
use super::super::types::{
    DatasetAggregationDraft, DatasetAggregationMetricDraft, DatasetAggregationMetricPayload,
    DatasetCalculatedFieldDraft, DatasetCalculatedFieldPayload, DatasetCalculationFunctionDraft,
    DatasetFieldDefinition, DatasetOperationDraftKind, DatasetOperationPayload,
    DatasetProjectionFieldPayload, DatasetRowFilterDraft, DatasetRowFilterPayload,
    DatasetRowPickerDraft, DatasetRowPickerPayload, DatasetRowPickerSortDraft,
};
use super::super::types::{DatasetOperationDraft, DatasetSourceDraft};
#[cfg(feature = "hydrate")]
use super::load_rendered_form;
use leptos::prelude::*;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(in crate::features::datasets) struct DatasetEditLoadTargets {
    pub(in crate::features::datasets) name: RwSignal<String>,
    pub(in crate::features::datasets) slug: RwSignal<String>,
    pub(in crate::features::datasets) visibility_node_ids: RwSignal<BTreeSet<String>>,
    pub(in crate::features::datasets) initial_source: RwSignal<DatasetSourceDraft>,
    pub(in crate::features::datasets) operation_order: RwSignal<Vec<DatasetOperationDraft>>,
    pub(in crate::features::datasets) rendered_forms:
        RwSignal<BTreeMap<String, super::super::types::DatasetRenderedForm>>,
    pub(in crate::features::datasets) restriction_internal_field_key: RwSignal<String>,
    pub(in crate::features::datasets) restriction_restricted_field_key: RwSignal<String>,
    pub(in crate::features::datasets) restriction_confidential_field_key: RwSignal<String>,
    pub(in crate::features::datasets) sql_preview: RwSignal<Option<String>>,
    pub(in crate::features::datasets) load_error: RwSignal<Option<String>>,
}

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_dataset_for_edit(
    dataset_id: String,
    targets: DatasetEditLoadTargets,
) {
    leptos::task::spawn_local(async move {
        let DatasetEditLoadTargets {
            name,
            slug,
            visibility_node_ids,
            initial_source,
            operation_order,
            rendered_forms,
            restriction_internal_field_key,
            restriction_restricted_field_key,
            restriction_confidential_field_key,
            sql_preview,
            load_error,
        } = targets;

        match api::fetch_dataset_detail(&dataset_id).await {
            Ok(Some(payload)) => {
                name.set(payload.name);
                slug.set(payload.slug);
                sql_preview.set(None);
                visibility_node_ids.set(
                    payload
                        .visibility_nodes
                        .into_iter()
                        .map(|node| node.node_id)
                        .collect(),
                );
                let Some(initial_source_payload) = payload.initial_source.as_ref() else {
                    load_error.set(Some(
                            "This dataset was not created with the operation pipeline and cannot be edited here."
                                .into(),
                        ));
                    return;
                };
                let initial_source_draft = source_payload_to_draft(initial_source_payload);
                preload_source_form(&initial_source_draft, rendered_forms);
                initial_source.set(initial_source_draft);
                let source_field_lookup = source_field_lookup(&payload.fields);
                let mut operation_order_drafts = Vec::new();
                for operation in payload.operations {
                    match operation {
                        DatasetOperationPayload::AddSource {
                            source,
                            add_type,
                            join_keys,
                            ..
                        } => {
                            let mut draft = DatasetOperationDraft::new(
                                operation_order_drafts.len() as u64 + 1,
                                DatasetOperationDraftKind::AddSource,
                            );
                            draft.source = Some(source_payload_to_draft(&source));
                            if let Some(source) = draft.source.as_ref() {
                                preload_source_form(source, rendered_forms);
                            }
                            draft.add_type = add_type;
                            if let Some(join_key) = join_keys.first() {
                                draft.left_field_key = join_key.left_field.clone();
                                draft.right_field_key = join_key.right_field.clone();
                            }
                            operation_order_drafts.push(draft);
                        }
                        DatasetOperationPayload::Projection {
                            fields: operation_fields,
                            ..
                        } => {
                            let mut draft = DatasetOperationDraft::new(
                                operation_order_drafts.len() as u64 + 1,
                                DatasetOperationDraftKind::Projection,
                            );
                            draft.projection_fields = projection_field_drafts_from_payload(
                                operation_fields,
                                &source_field_lookup,
                            );
                            operation_order_drafts.push(draft);
                        }
                        DatasetOperationPayload::Aggregation {
                            group_fields,
                            metrics,
                            row_picker,
                            ..
                        } => {
                            let mut draft = DatasetOperationDraft::new(
                                operation_order_drafts.len() as u64 + 1,
                                DatasetOperationDraftKind::Aggregation,
                            );
                            draft.aggregation =
                                aggregation_draft_from_payload(group_fields, metrics, row_picker);
                            operation_order_drafts.push(draft);
                        }
                        DatasetOperationPayload::CalculatedFields {
                            fields: operation_fields,
                            ..
                        } => {
                            let mut draft = DatasetOperationDraft::new(
                                operation_order_drafts.len() as u64 + 1,
                                DatasetOperationDraftKind::CalculatedFields,
                            );
                            draft.calculated_fields =
                                calculated_field_drafts_from_payload(operation_fields);
                            operation_order_drafts.push(draft);
                        }
                        DatasetOperationPayload::Filter {
                            filters: operation_filters,
                            ..
                        } => {
                            let mut draft = DatasetOperationDraft::new(
                                operation_order_drafts.len() as u64 + 1,
                                DatasetOperationDraftKind::Filter,
                            );
                            draft.row_filters = row_filter_drafts_from_payload(operation_filters);
                            operation_order_drafts.push(draft);
                        }
                    }
                }
                operation_order.set(operation_order_drafts);
                let restriction_policy = payload.restriction_policy;
                restriction_internal_field_key.set(
                    restriction_policy
                        .as_ref()
                        .and_then(|policy| policy.internal_field_key.clone())
                        .unwrap_or_default(),
                );
                restriction_restricted_field_key.set(
                    restriction_policy
                        .as_ref()
                        .and_then(|policy| policy.restricted_field_key.clone())
                        .unwrap_or_default(),
                );
                restriction_confidential_field_key.set(
                    restriction_policy
                        .and_then(|policy| policy.confidential_field_key)
                        .unwrap_or_default(),
                );
            }
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(feature = "hydrate")]
fn projection_field_drafts_from_payload(
    fields: Vec<DatasetProjectionFieldPayload>,
    source_field_lookup: &BTreeMap<String, DatasetFieldDefinition>,
) -> Vec<DatasetFieldDraft> {
    fields
        .into_iter()
        .map(|field| {
            let input_field_key = field.input_field_key.unwrap_or_else(|| field.key.clone());
            let field_definition = source_field_lookup
                .get(&field.key)
                .or_else(|| source_field_lookup.get(&input_field_key));
            let (fallback_alias, fallback_source_key) = split_canonical_field_key(&input_field_key);
            let source_alias = field_definition
                .map(|field| field.source_alias.clone())
                .unwrap_or(fallback_alias);
            let source_field_key = field_definition
                .map(|field| field.source_field_key.clone())
                .unwrap_or(fallback_source_key);
            let field_type = field_definition
                .map(|field| field.field_type.clone())
                .unwrap_or_else(|| "text".into());
            DatasetFieldDraft {
                key: field.key,
                label: field.label,
                source_alias,
                source_field_key,
                field_type,
            }
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn preload_source_form(
    source: &DatasetSourceDraft,
    rendered_forms: RwSignal<BTreeMap<String, super::super::types::DatasetRenderedForm>>,
) {
    if source.input_kind.eq_ignore_ascii_case("form")
        && !source.form_version_id.trim().is_empty()
        && !rendered_forms.with(|forms| forms.contains_key(&source.form_version_id))
    {
        load_rendered_form(source.form_version_id.clone(), rendered_forms);
    }
}

#[cfg(feature = "hydrate")]
fn aggregation_draft_from_payload(
    group_fields: Vec<String>,
    metrics: Vec<DatasetAggregationMetricPayload>,
    row_picker: Option<DatasetRowPickerPayload>,
) -> DatasetAggregationDraft {
    DatasetAggregationDraft {
        enabled: true,
        group_fields,
        metrics: metrics
            .into_iter()
            .enumerate()
            .map(|(index, metric)| DatasetAggregationMetricDraft {
                id: index as u64 + 1,
                key: metric.key,
                label: metric.label,
                function: metric.function,
                source_field_key: metric.source_field_key.unwrap_or_default(),
            })
            .collect(),
        row_picker: row_picker.map(|row_picker| DatasetRowPickerDraft {
            sort_fields: row_picker
                .sort_fields
                .into_iter()
                .map(|sort| DatasetRowPickerSortDraft {
                    field_key: sort.field_key,
                })
                .collect(),
            direction: row_picker.direction,
        }),
    }
}

#[cfg(feature = "hydrate")]
fn row_filter_drafts_from_payload(
    filters: Vec<DatasetRowFilterPayload>,
) -> Vec<DatasetRowFilterDraft> {
    filters
        .into_iter()
        .enumerate()
        .map(|(index, filter)| DatasetRowFilterDraft {
            id: index as u64 + 1,
            field_key: filter.field_key,
            operator: filter.operator,
            value: filter.value.unwrap_or_default(),
            value_mode: filter.value_mode,
            value_field_key: filter.value_field_key.unwrap_or_default(),
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn calculated_field_drafts_from_payload(
    fields: Vec<DatasetCalculatedFieldPayload>,
) -> Vec<DatasetCalculatedFieldDraft> {
    fields
        .into_iter()
        .enumerate()
        .map(|(index, field)| DatasetCalculatedFieldDraft {
            id: index as u64 + 1,
            key: field.key,
            label: field.label,
            base_field_key: field.base_field_key,
            functions: field
                .functions
                .into_iter()
                .enumerate()
                .map(
                    |(function_index, function)| DatasetCalculationFunctionDraft {
                        id: function_index as u64 + 1,
                        function: function.function,
                        argument: function.argument.unwrap_or_default(),
                        argument_mode: function.argument_mode,
                        argument_field_key: function.argument_field_key.unwrap_or_default(),
                    },
                )
                .collect(),
        })
        .collect()
}

#[cfg(feature = "hydrate")]
fn split_canonical_field_key(field_key: &str) -> (String, String) {
    field_key
        .split_once("__")
        .map(|(source_alias, source_field_key)| (source_alias.into(), source_field_key.into()))
        .unwrap_or_else(|| (String::new(), field_key.into()))
}

#[cfg(feature = "hydrate")]
fn source_field_lookup(
    fields: &[DatasetFieldDefinition],
) -> BTreeMap<String, DatasetFieldDefinition> {
    let mut lookup = BTreeMap::new();
    for field in fields {
        lookup.insert(field.key.clone(), field.clone());
    }
    lookup
}

#[cfg(not(feature = "hydrate"))]
pub(in crate::features::datasets) fn load_dataset_for_edit(_: String, _: DatasetEditLoadTargets) {}
