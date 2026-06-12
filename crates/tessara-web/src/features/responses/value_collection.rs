//! Response value collection helpers.
//!
//! Keep form-field value extraction and submission value maps here so response start and edit flows share one conversion path.

use crate::features::forms::RenderedForm;
use crate::features::responses::types::SubmissionDetail;
use serde_json::Value;
use std::collections::HashMap;

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn response_input_value(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) => value.clone(),
        Some(Value::Number(value)) => value.to_string(),
        Some(Value::Array(values)) => values
            .iter()
            .filter_map(|value| value.as_str())
            .collect::<Vec<_>>()
            .join(", "),
        Some(Value::Bool(value)) => value.to_string(),
        Some(value) if !value.is_null() => value.to_string(),
        _ => String::new(),
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) fn submission_value_maps(
    detail: &SubmissionDetail,
) -> (HashMap<String, String>, HashMap<String, bool>) {
    let mut text_values = HashMap::new();
    let mut boolean_values = HashMap::new();

    for value in &detail.values {
        if value.field_type == "boolean" {
            boolean_values.insert(
                value.key.clone(),
                value
                    .value
                    .as_ref()
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
            );
        } else {
            text_values.insert(
                value.key.clone(),
                response_input_value(value.value.as_ref()),
            );
        }
    }

    (text_values, boolean_values)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Collects the collect response values values.
pub(crate) fn collect_response_values(
    rendered_form: &RenderedForm,
    text_values: &HashMap<String, String>,
    boolean_values: &HashMap<String, bool>,
) -> Result<HashMap<String, Value>, String> {
    let mut values = HashMap::new();

    for section in &rendered_form.sections {
        for field in &section.fields {
            if field.field_type == "boolean" {
                values.insert(
                    field.key.clone(),
                    Value::Bool(*boolean_values.get(&field.key).unwrap_or(&false)),
                );
                continue;
            }

            let raw = text_values
                .get(&field.key)
                .map(String::as_str)
                .unwrap_or_default()
                .trim();
            if raw.is_empty() {
                if field.required {
                    return Err(format!("Required fields missing: {}", field.label));
                }
                continue;
            }

            let value = match field.field_type.as_str() {
                "number" => {
                    let parsed = raw
                        .parse::<f64>()
                        .map_err(|_| format!("{} must be a number.", field.label))?;
                    Value::Number(
                        serde_json::Number::from_f64(parsed)
                            .ok_or_else(|| format!("{} must be a finite number.", field.label))?,
                    )
                }
                "multi_choice" => Value::Array(
                    raw.split(',')
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .map(|value| Value::String(value.to_string()))
                        .collect(),
                ),
                _ => Value::String(raw.to_string()),
            };

            values.insert(field.key.clone(), value);
        }
    }

    Ok(values)
}
