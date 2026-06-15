//! Edit-hydration loader for the Datasets feature.

#[cfg(feature = "hydrate")]
use super::super::api;
#[cfg(feature = "hydrate")]
use super::super::expressions::expression_to_editor_drafts;
use super::super::types::{
    DatasetAggregationDraft, DatasetExpressionDraft, DatasetFieldDraft, DatasetSourceDraft,
};
#[cfg(feature = "hydrate")]
use super::super::types::{DatasetAggregationMetricDraft, DatasetRowPickerDraft};
use leptos::prelude::*;
use std::collections::BTreeSet;

#[cfg(feature = "hydrate")]
pub(in crate::features::datasets) fn load_dataset_for_edit(
    dataset_id: String,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    composition_mode: RwSignal<String>,
    visibility_node_ids: RwSignal<BTreeSet<String>>,
    sources: RwSignal<Vec<DatasetSourceDraft>>,
    expression: RwSignal<DatasetExpressionDraft>,
    fields: RwSignal<Vec<DatasetFieldDraft>>,
    aggregation: RwSignal<DatasetAggregationDraft>,
    join_left_key: RwSignal<String>,
    join_right_key: RwSignal<String>,
    sql_preview: RwSignal<Option<String>>,
    load_error: RwSignal<Option<String>>,
) {
    leptos::task::spawn_local(async move {
        match api::fetch_dataset_detail(&dataset_id).await {
            Ok(Some(payload)) => {
                name.set(payload.name);
                slug.set(payload.slug);
                composition_mode.set(payload.composition_mode);
                sql_preview.set(None);
                visibility_node_ids.set(
                    payload
                        .visibility_nodes
                        .into_iter()
                        .map(|node| node.node_id)
                        .collect(),
                );
                let Some(ast) = payload.definition_ast.as_ref() else {
                    load_error.set(Some(
                            "This dataset was not created with the query designer and cannot be edited here."
                                .into(),
                        ));
                    return;
                };
                let (source_drafts, expression_draft, root_operation, join_keys) =
                    expression_to_editor_drafts(ast);
                if !root_operation.is_empty() {
                    composition_mode.set(root_operation);
                }
                if let Some(join_key) = join_keys.first() {
                    join_left_key.set(join_key.left_field.clone());
                    join_right_key.set(join_key.right_field.clone());
                }
                sources.set(if source_drafts.is_empty() {
                    vec![DatasetSourceDraft::default()]
                } else {
                    source_drafts
                });
                expression.set(expression_draft);
                let field_drafts = payload
                    .fields
                    .into_iter()
                    .map(|field| DatasetFieldDraft {
                        key: field.key,
                        label: field.label,
                        source_alias: field.source_alias,
                        source_field_key: field.source_field_key,
                    })
                    .collect::<Vec<_>>();
                fields.set(field_drafts.clone());
                aggregation.set(
                    payload
                        .aggregation
                        .map(|aggregation| {
                            let has_node_grouping = aggregation.group_fields.iter().any(|key| {
                                field_drafts.iter().any(|field| {
                                    field.key == *key && field.source_field_key == "__node_id"
                                })
                            });
                            DatasetAggregationDraft {
                                enabled: true,
                                group_fields: aggregation.group_fields,
                                metrics: aggregation
                                    .metrics
                                    .into_iter()
                                    .map(|metric| DatasetAggregationMetricDraft {
                                        key: metric.key,
                                        label: metric.label,
                                        function: metric.function,
                                        source_field_key: metric
                                            .source_field_key
                                            .unwrap_or_default(),
                                    })
                                    .collect(),
                                row_picker: aggregation.row_picker.map(|row_picker| {
                                    DatasetRowPickerDraft {
                                        sort_field_key: row_picker.sort_field_key,
                                        direction: row_picker.direction,
                                    }
                                }),
                                node_grouping_manually_removed: !has_node_grouping,
                            }
                        })
                        .unwrap_or_default(),
                );
            }
            Ok(None) => {}
            Err(message) => load_error.set(Some(message)),
        }
    });
}

#[cfg(not(feature = "hydrate"))]
#[allow(clippy::too_many_arguments)]
pub(in crate::features::datasets) fn load_dataset_for_edit(
    _: String,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<BTreeSet<String>>,
    _: RwSignal<Vec<DatasetSourceDraft>>,
    _: RwSignal<DatasetExpressionDraft>,
    _: RwSignal<Vec<DatasetFieldDraft>>,
    _: RwSignal<DatasetAggregationDraft>,
    _: RwSignal<String>,
    _: RwSignal<String>,
    _: RwSignal<Option<String>>,
    _: RwSignal<Option<String>>,
) {
}
