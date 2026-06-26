//! Editor-side field lists for each dataset authoring pipeline stage.

use crate::features::datasets::types::{
    DatasetAggregationDraft, DatasetCalculatedFieldDraft, DatasetFieldDraft,
};
use std::collections::BTreeSet;

pub(super) fn fields_after_aggregation(
    fields: Vec<DatasetFieldDraft>,
    aggregation: DatasetAggregationDraft,
) -> Vec<DatasetFieldDraft> {
    if !aggregation.enabled {
        return fields;
    }

    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    for key in &aggregation.group_fields {
        if let Some(field) = fields.iter().find(|field| field.key == *key)
            && seen.insert(field.key.clone())
        {
            output.push(field.clone());
        }
    }

    if aggregation.row_picker.is_some() {
        for field in fields {
            if seen.insert(field.key.clone()) {
                output.push(field);
            }
        }
        return output;
    }

    for metric in aggregation.metrics {
        let source_field = fields
            .iter()
            .find(|field| field.key == metric.source_field_key);
        output.push(DatasetFieldDraft {
            key: metric.key,
            label: metric.label,
            source_alias: "aggregation".into(),
            source_field_key: metric.source_field_key,
            field_type: aggregation_metric_field_type(&metric.function, source_field),
        });
    }

    output
}

pub(super) fn fields_after_calculations(
    mut fields: Vec<DatasetFieldDraft>,
    calculated_fields: Vec<DatasetCalculatedFieldDraft>,
) -> Vec<DatasetFieldDraft> {
    let calculated_outputs = calculated_fields
        .into_iter()
        .filter_map(|calculation| calculated_field_output(&calculation, &fields))
        .collect::<Vec<_>>();
    fields.extend(calculated_outputs);
    fields
}

fn aggregation_metric_field_type(
    function: &str,
    source_field: Option<&DatasetFieldDraft>,
) -> String {
    match function {
        "count_rows" | "count_values" | "sum" | "average" => "number".into(),
        "min" | "max" => source_field
            .map(|field| field.field_type.clone())
            .unwrap_or_else(|| "text".into()),
        _ => "text".into(),
    }
}

fn calculated_field_output(
    calculation: &DatasetCalculatedFieldDraft,
    fields: &[DatasetFieldDraft],
) -> Option<DatasetFieldDraft> {
    if calculation.key.trim().is_empty() || calculation.base_field_key.trim().is_empty() {
        return None;
    }
    let base_field = fields
        .iter()
        .find(|field| field.key == calculation.base_field_key)?;
    let mut field_type = base_field.field_type.clone();
    for function in &calculation.functions {
        field_type = calculation_function_output_type(&function.function, &field_type);
    }
    Some(DatasetFieldDraft {
        key: calculation.key.clone(),
        label: calculation.label.clone(),
        source_alias: "calculated".into(),
        source_field_key: calculation.base_field_key.clone(),
        field_type,
    })
}

fn calculation_function_output_type(function: &str, input_type: &str) -> String {
    match function {
        "trim" | "uppercase" | "lowercase" | "prefix" | "suffix" | "concat" | "map_value"
        | "format_date" | "to_text" => "text".into(),
        "add" | "subtract" | "multiply" | "divide" | "round" | "to_number" => "number".into(),
        "greater_than"
        | "greater_than_or_equal"
        | "less_than"
        | "less_than_or_equal"
        | "equal"
        | "not_equal"
        | "is_empty"
        | "is_not_empty"
        | "to_boolean" => "boolean".into(),
        "to_date" => "date".into(),
        "coalesce" | "constant" => input_type.into(),
        _ => input_type.into(),
    }
}
