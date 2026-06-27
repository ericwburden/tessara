use serde_json::{Value, json};

pub fn detail_payload_for_restricted_tier(
    form_id: &str,
    form_version_id: &str,
    visibility_node_id: &str,
    field_key: &str,
    field_label: &str,
    restricted_flag_key: &str,
    restricted_flag_label: &str,
) -> Value {
    json!({
        "name": "Query Designer Restricted Dataset",
        "slug": "query-designer-restricted-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "form",
            "alias": "form_a",
            "form_id": form_id,
            "form_version_id": form_version_id
        },
        "operations": [
            projection_operation(json!([{
                "key": field_key,
                "label": field_label,
                "source_alias": "form_a",
                "source_field_key": field_key,
                "position": 0
            }, {
                "key": restricted_flag_key,
                "label": restricted_flag_label,
                "source_alias": "form_a",
                "source_field_key": restricted_flag_key,
                "position": 1
            }]), 0),
            calculated_fields_operation(json!([{
                "key": "restricted_flag",
                "label": "Restricted Flag",
                "base_field_key": restricted_flag_key,
                "functions": [{
                    "function": "constant",
                    "argument": "true",
                    "position": 0
                }],
                "position": 0
            }]), 1)
        ],
        "restriction_policy": {
            "restricted_field_key": "restricted_flag"
        }
    })
}

pub fn projection_operation(fields: Value, position: i32) -> Value {
    let mut fields = fields
        .as_array()
        .cloned()
        .expect("projection operation fields should be an array");
    for (index, field) in fields.iter_mut().enumerate() {
        let field_object = field
            .as_object_mut()
            .expect("projection field should be an object");
        let input_field_key = field_object.get("key").cloned().unwrap_or(Value::Null);
        if let (Some(source_alias), Some(source_field_key)) = (
            field_object.get("source_alias").and_then(Value::as_str),
            field_object.get("source_field_key").and_then(Value::as_str),
        ) {
            field_object.insert(
                "input_field_key".into(),
                json!(canonical_source_field_key(source_alias, source_field_key)),
            );
        }
        field_object
            .entry("input_field_key")
            .or_insert(input_field_key);
        field_object
            .entry("position")
            .or_insert_with(|| json!(index as i32));
        field_object.remove("source_alias");
        field_object.remove("source_field_key");
    }
    json!({
        "kind": "projection",
        "fields": fields,
        "position": position
    })
}

pub fn canonical_source_field_key(source_alias: &str, source_field_key: &str) -> String {
    format!(
        "{source_alias}__{}",
        source_field_key.trim_start_matches('_')
    )
}

pub fn aggregation_operation(
    group_fields: Value,
    metrics: Value,
    row_picker: Value,
    position: i32,
) -> Value {
    json!({
        "kind": "aggregation",
        "group_fields": group_fields,
        "metrics": metrics,
        "row_picker": row_picker,
        "position": position
    })
}

pub fn calculated_fields_operation(fields: Value, position: i32) -> Value {
    json!({
        "kind": "calculated_fields",
        "fields": fields,
        "position": position
    })
}

pub fn filter_operation(filters: Value, position: i32) -> Value {
    json!({
        "kind": "filter",
        "filters": filters,
        "position": position
    })
}
