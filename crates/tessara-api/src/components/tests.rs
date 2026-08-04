//! Tests for the parent module.

use serde_json::json;
use std::collections::{BTreeMap, HashMap};
use tessara_data_ops::{DataField, FieldType};

use uuid::Uuid;

use super::{
    ComponentLifecycleAction, ComponentSummary, ComponentTableQuery, ComponentVersionForTable,
    CreateComponentRequest, CreateComponentVersionRequest, UpdateComponentRequest,
    component_config_validation_finding, component_filter_sql, component_pagination_sql,
    component_visual_source_limit_clause, effective_component_page_size, lifecycle_transition,
    parse_component_query_filters, parse_component_sort, require_component_version_draft,
    table_order_by_sql, table_search_fields, validate_component_config, visible_table_fields,
    visual_from_rows,
};

#[test]
fn lifecycle_state_machine_is_exhaustive_and_terminal() {
    assert_eq!(
        lifecycle_transition("active", ComponentLifecycleAction::Deactivate).unwrap(),
        "inactive"
    );
    assert_eq!(
        lifecycle_transition("inactive", ComponentLifecycleAction::Activate).unwrap(),
        "active"
    );
    assert_eq!(
        lifecycle_transition("active", ComponentLifecycleAction::Archive).unwrap(),
        "archived"
    );
    assert_eq!(
        lifecycle_transition("inactive", ComponentLifecycleAction::Archive).unwrap(),
        "archived"
    );
    assert_eq!(
        lifecycle_transition("archived", ComponentLifecycleAction::Tombstone).unwrap(),
        "tombstoned"
    );
    for action in [
        ComponentLifecycleAction::Activate,
        ComponentLifecycleAction::Deactivate,
        ComponentLifecycleAction::Archive,
        ComponentLifecycleAction::Tombstone,
    ] {
        assert!(lifecycle_transition("tombstoned", action).is_err());
    }
    assert!(lifecycle_transition("archived", ComponentLifecycleAction::Activate).is_err());
}

fn field(key: &str, field_type: FieldType) -> DataField {
    DataField {
        key: key.into(),
        label: key.into(),
        field_type,
        position: 0,
    }
}

fn visual_row(values: &[(&str, Option<&str>)]) -> BTreeMap<String, Option<String>> {
    values
        .iter()
        .map(|(key, value)| ((*key).to_string(), value.map(str::to_string)))
        .collect()
}

#[test]
fn reader_component_summary_omits_absent_draft_metadata() {
    let summary = ComponentSummary {
        id: Uuid::nil(),
        name: "Published Table".into(),
        slug: "published_table".into(),
        description: None,
        current_version_id: Some(Uuid::nil()),
        current_version_label: Some("1".into()),
        current_component_type: Some("table".into()),
        draft_version_id: None,
        draft_version_label: None,
    };

    let value = serde_json::to_value(summary).expect("summary should serialize");

    assert!(value.get("draft_version_id").is_none());
    assert!(value.get("draft_version_label").is_none());
}

#[test]
fn table_config_validates_presentation_fields() {
    let fields = vec![
        field("program", FieldType::Text),
        field("amount", FieldType::Number),
    ];
    let config = json!({
        "visible_columns": ["program", "amount"],
        "filters": [
            {
                "field_key": "program",
                "operator": "not_contains",
                "value": "archived"
            }
        ],
        "search_fields": ["program"],
        "default_sort": {
            "field_key": "amount",
            "direction": "desc"
        },
        "page_size": 25,
        "display_labels": {
            "amount": "Award Amount"
        }
    });

    validate_component_config("table", &config, &fields)
        .expect("valid table component config should pass");
}

#[test]
fn table_config_rejects_stale_analytical_keys() {
    let fields = vec![field("program", FieldType::Text)];
    let config = json!({
        "visible_columns": ["program"],
        "metrics": [
            {
                "function": "count",
                "field_key": "program"
            }
        ]
    });

    let error = validate_component_config("table", &config, &fields)
        .expect_err("component analytical keys should fail");
    assert!(error.to_string().contains("unknown field `metrics`"));
}

#[test]
fn table_config_validates_saved_filters() {
    let fields = vec![field("score", FieldType::Number)];
    let config = json!({
        "visible_columns": ["score"],
        "filters": [
            {
                "field_key": "score",
                "operator": "contains",
                "value": "10"
            }
        ]
    });

    let error = validate_component_config("table", &config, &fields)
        .expect_err("invalid saved filter should fail");
    assert!(
        error
            .to_string()
            .contains("filter operator 'contains' is not supported")
    );
}

#[test]
fn table_config_rejects_invalid_numeric_saved_filter_value() {
    let fields = vec![field("score", FieldType::Number)];
    let config = json!({
        "visible_columns": ["score"],
        "filters": [
            {
                "field_key": "score",
                "operator": "equals",
                "value": "not-a-number"
            }
        ]
    });

    let error = validate_component_config("table", &config, &fields)
        .expect_err("invalid numeric saved filter should fail");
    assert!(error.to_string().contains("invalid value 'not-a-number'"));
}

#[test]
fn table_config_rejects_invalid_date_saved_filter_range() {
    let fields = [field("submitted_on", FieldType::Date)];
    let config = json!({
        "visible_columns": ["submitted_on"],
        "filters": [
            {
                "field_key": "submitted_on",
                "operator": "between",
                "value": "2026-01-01..soon"
            }
        ]
    });

    let error = validate_component_config("table", &config, &fields)
        .expect_err("invalid date saved filter range should fail");
    assert!(error.to_string().contains("invalid value 'soon'"));
}

#[test]
fn component_filter_sql_rejects_invalid_runtime_filter_value() {
    let fields = [field("submitted_on", FieldType::Date)];
    let filters = vec![super::ComponentFilterConfig {
        field_key: "submitted_on".into(),
        operator: "gte".into(),
        value: Some("not-a-date".into()),
    }];
    let refs = fields.iter().collect::<Vec<_>>();

    let error = component_filter_sql(&filters, &refs)
        .expect_err("invalid runtime filter literal should fail");
    assert!(error.to_string().contains("invalid value 'not-a-date'"));
}

#[test]
fn table_config_rejects_field_mode_component_filters() {
    let fields = [field("program", FieldType::Text)];
    let config = json!({
        "visible_columns": ["program"],
        "filters": [
            {
                "field_key": "program",
                "operator": "equals",
                "value_field_key": "other_program"
            }
        ]
    });

    let error = validate_component_config("table", &config, &fields)
        .expect_err("field-mode component filter should fail");
    assert!(
        error
            .to_string()
            .contains("unknown field `value_field_key`")
    );
}

#[test]
fn table_config_rejects_missing_visible_column() {
    let fields = [field("program", FieldType::Text)];
    let config = json!({
        "visible_columns": ["program", "amount"]
    });

    let error = validate_component_config("table", &config, &fields)
        .expect_err("unknown visible column should fail");
    assert!(
        error
            .to_string()
            .contains("table visible column references field 'amount'")
    );
}

#[test]
fn old_component_table_kinds_are_rejected() {
    let fields = [field("program", FieldType::Text)];
    let config = json!({ "visible_columns": ["program"] });

    let detail_error = validate_component_config("detail_table", &config, &fields)
        .expect_err("old detail kind should fail");
    let aggregate_error = validate_component_config("aggregate_table", &config, &fields)
        .expect_err("old aggregate kind should fail");

    assert!(
        detail_error
            .to_string()
            .contains("unsupported component type")
    );
    assert!(
        aggregate_error
            .to_string()
            .contains("unsupported component type")
    );
}

#[test]
fn visual_component_config_accepts_supported_kinds() {
    let fields = vec![
        field("program", FieldType::Text),
        field("region", FieldType::Text),
        field("amount", FieldType::Number),
        field("period", FieldType::Text),
    ];
    let configs = [
        (
            "bar",
            json!({
                "mode": "comparison",
                "summary_field": "amount",
                "summary_type": "sum",
                "category_field": "program",
                "comparison_field": "region",
                "comparison_layout": "stacked",
                "orientation": "horizontal",
                "x_axis_label": "Award amount",
                "y_axis_label": "Program",
                "filters": [{
                    "field_key": "region",
                    "operator": "equals",
                    "value": "North"
                }],
                "sort_field": "summary_value",
                "sort_direction": "desc",
                "number_of_points": 5
            }),
        ),
        (
            "line",
            json!({
                "summary_field": "amount",
                "summary_type": "average",
                "x_field": "period"
            }),
        ),
        (
            "pie",
            json!({
                "summary_field": "amount",
                "summary_type": "sum",
                "category_field": "program",
                "max_slices": 10
            }),
        ),
        (
            "donut",
            json!({
                "summary_field": "amount",
                "summary_type": "sum",
                "category_field": "program"
            }),
        ),
        (
            "stat_card",
            json!({
                "summary_field": "amount",
                "summary_type": "median",
                "label": "Median award",
                "panel_style": "accent"
            }),
        ),
    ];

    for (kind, config) in configs {
        validate_component_config(kind, &config, &fields)
            .unwrap_or_else(|error| panic!("{kind} should validate: {error}"));
    }
}

#[test]
fn visual_component_config_rejects_invalid_contracts() {
    let fields = vec![
        field("program", FieldType::Text),
        field("amount", FieldType::Number),
    ];

    let non_numeric = validate_component_config(
        "bar",
        &json!({
            "mode": "summary",
            "summary_field": "program",
            "summary_type": "sum",
            "category_field": "program"
        }),
        &fields,
    )
    .expect_err("numeric summaries should require numeric fields");
    assert!(non_numeric.to_string().contains("requires numeric"));

    let missing_comparison = validate_component_config(
        "bar",
        &json!({
            "mode": "comparison",
            "summary_field": "amount",
            "summary_type": "sum",
            "category_field": "program"
        }),
        &fields,
    )
    .expect_err("comparison bar should require comparison field");
    assert!(
        missing_comparison
            .to_string()
            .contains("requires comparison_field")
    );

    let bad_layout = validate_component_config(
        "bar",
        &json!({
            "mode": "comparison",
            "summary_field": "amount",
            "summary_type": "sum",
            "category_field": "program",
            "comparison_field": "program",
            "comparison_layout": "clustered"
        }),
        &fields,
    )
    .expect_err("comparison bar should reject unsupported comparison layouts");
    assert!(bad_layout.to_string().contains("bar comparison layout"));

    let non_additive_stack = validate_component_config(
        "bar",
        &json!({
            "mode": "comparison",
            "summary_field": "amount",
            "summary_type": "average",
            "category_field": "program",
            "comparison_field": "program",
            "comparison_layout": "stacked"
        }),
        &fields,
    )
    .expect_err("stacking non-additive summaries should fail");
    assert!(
        non_additive_stack
            .to_string()
            .contains("requires row_count, count, or sum")
    );

    let stale_key = validate_component_config(
        "pie",
        &json!({
            "summary_field": "amount",
            "summary_type": "sum",
            "category_field": "program",
            "max_items": 5
        }),
        &fields,
    )
    .expect_err("removed visual config keys should fail");
    assert!(stale_key.to_string().contains("unknown field `max_items`"));
}

#[test]
fn component_config_findings_identify_the_invalid_role() {
    let type_mismatch = component_config_validation_finding(crate::error::ApiError::BadRequest(
        "summary type 'sum' requires numeric summary field 'completed'".into(),
    ));
    assert_eq!(type_mismatch.code, "COMPONENT_SUMMARY_FIELD_TYPE_MISMATCH");
    assert_eq!(
        type_mismatch.field_path.as_deref(),
        Some("config.summary_field")
    );

    let missing_category = component_config_validation_finding(crate::error::ApiError::BadRequest(
        "bar category field references field 'removed' outside the dataset major-line contract"
            .into(),
    ));
    assert_eq!(
        missing_category.code,
        "COMPONENT_CATEGORY_FIELD_NOT_IN_MAJOR_LINE"
    );
    assert_eq!(
        missing_category.field_path.as_deref(),
        Some("config.category_field")
    );

    let missing_table_column = component_config_validation_finding(
        crate::error::ApiError::BadRequest(
            "table visible column references field 'removed' outside the dataset major-line contract"
                .into(),
        ),
    );
    assert_eq!(
        missing_table_column.code,
        "COMPONENT_FIELD_NOT_IN_MAJOR_LINE"
    );
    assert_eq!(missing_table_column.field_path.as_deref(), Some("config"));
}

#[test]
fn visual_transform_groups_sorts_and_limits_points() {
    let version = ComponentVersionForTable {
        id: Uuid::nil(),
        component_id: Uuid::nil(),
        dataset_id: Uuid::nil(),
        dataset_version_major: 1,
        component_type: "bar".into(),
        config: json!({
            "mode": "summary",
            "summary_field": "amount",
            "summary_type": "sum",
            "category_field": "program",
            "sort_field": "summary_value",
            "sort_direction": "desc",
            "number_of_points": 2,
            "value_format": "integer"
        }),
    };
    let rows = vec![
        visual_row(&[("program", Some("Alpha")), ("amount", Some("10"))]),
        visual_row(&[("program", Some("Beta")), ("amount", Some("7"))]),
        visual_row(&[("program", Some("Alpha")), ("amount", Some("5"))]),
        visual_row(&[("program", Some("Gamma")), ("amount", Some("20"))]),
    ];
    let fields = [
        field("program", FieldType::Text),
        field("amount", FieldType::Number),
    ];

    let visual = visual_from_rows(
        version,
        super::VisualComponentConfig::parse(
            "bar",
            &json!({
                "mode": "summary",
                "summary_field": "amount",
                "summary_type": "sum",
                "category_field": "program",
                "sort_field": "summary_value",
                "sort_direction": "desc",
                "number_of_points": 2,
                "value_format": "integer"
            }),
        )
        .expect("visual config"),
        rows,
        &fields,
    )
    .expect("visual should transform");

    assert_eq!(visual.points.len(), 2);
    assert_eq!(visual.points[0].x, "Gamma");
    assert_eq!(visual.points[0].value, 20.0);
    assert_eq!(visual.points[1].x, "Alpha");
    assert_eq!(visual.points[1].display_value, "15");
}

#[test]
fn visual_category_sort_uses_numeric_field_type() {
    let config = json!({
        "mode": "summary",
        "summary_field": "amount",
        "summary_type": "sum",
        "category_field": "score",
        "sort_field": "category",
        "sort_direction": "asc"
    });
    let fields = [
        field("score", FieldType::Number),
        field("amount", FieldType::Number),
    ];
    let rows = vec![
        visual_row(&[("score", Some("10")), ("amount", Some("1"))]),
        visual_row(&[("score", Some("2")), ("amount", Some("1"))]),
        visual_row(&[("score", Some("1")), ("amount", Some("1"))]),
    ];

    let visual = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "bar".into(),
            config: config.clone(),
        },
        super::VisualComponentConfig::parse("bar", &config).expect("visual config"),
        rows,
        &fields,
    )
    .expect("numeric categories should transform");

    assert_eq!(
        visual
            .points
            .iter()
            .map(|point| point.x.as_str())
            .collect::<Vec<_>>(),
        vec!["1", "2", "10"]
    );
}

#[test]
fn unique_count_preserves_exact_source_values() {
    let config = json!({
        "summary_field": "code",
        "summary_type": "unique_count"
    });
    let fields = [field("code", FieldType::Text)];
    let rows = vec![
        visual_row(&[("code", Some("Aa"))]),
        visual_row(&[("code", Some("BB"))]),
        visual_row(&[("code", Some("Aa"))]),
    ];

    let visual = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "stat_card".into(),
            config: config.clone(),
        },
        super::VisualComponentConfig::parse("stat_card", &config).expect("visual config"),
        rows,
        &fields,
    )
    .expect("unique count should transform");

    assert_eq!(visual.stat.and_then(|stat| stat.value), Some(2.0));
}

#[test]
fn negative_values_are_preserved_for_bars_and_rejected_for_slices() {
    let base_config = json!({
        "summary_field": "amount",
        "summary_type": "sum",
        "category_field": "program"
    });
    let fields = [
        field("program", FieldType::Text),
        field("amount", FieldType::Number),
    ];
    let rows = vec![visual_row(&[
        ("program", Some("Alpha")),
        ("amount", Some("-5")),
    ])];
    let bar_config = json!({
        "mode": "summary",
        "summary_field": "amount",
        "summary_type": "sum",
        "category_field": "program"
    });
    let bar = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "bar".into(),
            config: bar_config.clone(),
        },
        super::VisualComponentConfig::parse("bar", &bar_config).expect("bar config"),
        rows.clone(),
        &fields,
    )
    .expect("bar should preserve negative values");
    assert_eq!(bar.points[0].value, -5.0);

    let pie_result = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "pie".into(),
            config: base_config.clone(),
        },
        super::VisualComponentConfig::parse("pie", &base_config).expect("pie config"),
        rows,
        &fields,
    );
    let pie_error = match pie_result {
        Ok(_) => panic!("pie should reject negative values"),
        Err(error) => error,
    };
    assert!(pie_error.to_string().contains("do not support negative"));
}

#[test]
fn visual_row_count_does_not_require_a_value_field() {
    let fields = [field("program", FieldType::Text)];
    let config = json!({
        "mode": "summary",
        "summary_field": "",
        "summary_type": "row_count",
        "category_field": "program"
    });
    validate_component_config("bar", &config, &fields).expect("row count config");

    let rows = vec![
        visual_row(&[("program", Some("Alpha"))]),
        visual_row(&[("program", Some("Alpha"))]),
        visual_row(&[("program", Some("Beta"))]),
    ];
    let visual = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "bar".into(),
            config: config.clone(),
        },
        super::VisualComponentConfig::parse("bar", &config).expect("visual config"),
        rows,
        &fields,
    )
    .expect("row count should transform");

    assert_eq!(visual.points.len(), 2);
    assert_eq!(
        visual.points.iter().map(|point| point.value).sum::<f64>(),
        3.0
    );
}

#[test]
fn visual_do_not_summarize_rejects_duplicate_groups() {
    let config = json!({
        "mode": "summary",
        "summary_field": "amount",
        "summary_type": "none",
        "category_field": "program"
    });
    let rows = vec![
        visual_row(&[("program", Some("Alpha")), ("amount", Some("10"))]),
        visual_row(&[("program", Some("Alpha")), ("amount", Some("12"))]),
    ];
    let fields = [
        field("program", FieldType::Text),
        field("amount", FieldType::Number),
    ];
    let result = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "bar".into(),
            config: config.clone(),
        },
        super::VisualComponentConfig::parse("bar", &config).expect("visual config"),
        rows,
        &fields,
    );
    let error = match result {
        Ok(_) => panic!("duplicate groups must not be silently repaired"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("requires exactly one row"));
    assert!(error.to_string().contains("category 'Alpha'"));
}

#[test]
fn bar_comparison_display_uses_comparison_labels_and_colors() {
    let version = ComponentVersionForTable {
        id: Uuid::nil(),
        component_id: Uuid::nil(),
        dataset_id: Uuid::nil(),
        dataset_version_major: 1,
        component_type: "bar".into(),
        config: json!({
            "mode": "comparison",
            "summary_field": "amount",
            "summary_type": "sum",
            "category_field": "period",
            "comparison_field": "completed",
            "category_labels": {
                "true": "Completed",
                "false": "Not completed"
            },
            "category_colors": {
                "true": "var(--semantic-primary)",
                "false": "var(--semantic-warning)"
            }
        }),
    };
    let rows = vec![
        visual_row(&[
            ("period", Some("2026-05-01")),
            ("completed", Some("true")),
            ("amount", Some("10")),
        ]),
        visual_row(&[
            ("period", Some("2026-05-01")),
            ("completed", Some("false")),
            ("amount", Some("3")),
        ]),
    ];
    let fields = [
        field("period", FieldType::Date),
        field("completed", FieldType::Boolean),
        field("amount", FieldType::Number),
    ];

    let visual = visual_from_rows(
        version,
        super::VisualComponentConfig::parse(
            "bar",
            &json!({
                "mode": "comparison",
                "summary_field": "amount",
                "summary_type": "sum",
                "category_field": "period",
                "comparison_field": "completed",
                "category_labels": {
                    "true": "Completed",
                    "false": "Not completed"
                },
                "category_colors": {
                    "true": "var(--semantic-primary)",
                    "false": "var(--semantic-warning)"
                }
            }),
        )
        .expect("visual config"),
        rows,
        &fields,
    )
    .expect("visual should transform");

    assert_eq!(visual.points.len(), 2);
    assert_eq!(visual.points[0].x, "2026-05-01");
    assert_eq!(
        visual.points[0].comparison.as_deref(),
        Some("Not completed")
    );
    assert_eq!(
        visual.points[0].color.as_deref(),
        Some("var(--semantic-warning)")
    );
    assert_eq!(visual.points[1].comparison.as_deref(), Some("Completed"));
    assert_eq!(
        visual.points[1].color.as_deref(),
        Some("var(--semantic-primary)")
    );
}

#[test]
fn bar_role_missing_policies_are_applied_independently() {
    let config = json!({
        "mode": "comparison",
        "summary_field": "amount",
        "summary_type": "sum",
        "value_missing_policy": "zero",
        "category_field": "program",
        "category_missing_policy": "omit",
        "comparison_field": "region",
        "comparison_missing_policy": "explicit_missing"
    });
    let fields = [
        field("program", FieldType::Text),
        field("region", FieldType::Text),
        field("amount", FieldType::Number),
    ];
    validate_component_config("bar", &config, &fields).expect("role policies should validate");
    let rows = vec![
        visual_row(&[
            ("program", Some("Alpha")),
            ("region", None),
            ("amount", Some("5")),
        ]),
        visual_row(&[
            ("program", None),
            ("region", Some("East")),
            ("amount", Some("9")),
        ]),
        visual_row(&[
            ("program", Some("Beta")),
            ("region", Some("East")),
            ("amount", None),
        ]),
    ];
    let visual = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "bar".into(),
            config: config.clone(),
        },
        super::VisualComponentConfig::parse("bar", &config).expect("visual config"),
        rows,
        &fields,
    )
    .expect("role-specific policies should transform");

    assert_eq!(visual.points.len(), 2);
    assert!(visual.points.iter().any(|point| {
        point.x == "Alpha" && point.comparison.as_deref() == Some("(Missing)") && point.value == 5.0
    }));
    assert!(visual.points.iter().any(|point| {
        point.x == "Beta" && point.comparison.as_deref() == Some("East") && point.value == 0.0
    }));
}

#[test]
fn bar_comparison_limit_retains_all_series_for_each_category() {
    let config = json!({
        "mode": "comparison",
        "summary_field": "amount",
        "summary_type": "sum",
        "category_field": "program",
        "comparison_field": "region",
        "sort_field": "summary_value",
        "sort_direction": "desc",
        "number_of_points": 2
    });
    let rows = vec![
        visual_row(&[
            ("program", Some("Alpha")),
            ("region", Some("East")),
            ("amount", Some("5")),
        ]),
        visual_row(&[
            ("program", Some("Alpha")),
            ("region", Some("West")),
            ("amount", Some("5")),
        ]),
        visual_row(&[
            ("program", Some("Beta")),
            ("region", Some("East")),
            ("amount", Some("7")),
        ]),
        visual_row(&[
            ("program", Some("Beta")),
            ("region", Some("West")),
            ("amount", Some("7")),
        ]),
        visual_row(&[
            ("program", Some("Gamma")),
            ("region", Some("East")),
            ("amount", Some("9")),
        ]),
        visual_row(&[
            ("program", Some("Gamma")),
            ("region", Some("West")),
            ("amount", Some("9")),
        ]),
    ];
    let fields = [
        field("program", FieldType::Text),
        field("region", FieldType::Text),
        field("amount", FieldType::Number),
    ];
    let visual = visual_from_rows(
        ComponentVersionForTable {
            id: Uuid::nil(),
            component_id: Uuid::nil(),
            dataset_id: Uuid::nil(),
            dataset_version_major: 1,
            component_type: "bar".into(),
            config: config.clone(),
        },
        super::VisualComponentConfig::parse("bar", &config).expect("visual config"),
        rows,
        &fields,
    )
    .expect("comparison should transform");

    assert_eq!(visual.points.len(), 4);
    assert!(
        visual
            .points
            .iter()
            .all(|point| matches!(point.x.as_str(), "Beta" | "Gamma"))
    );
    assert_eq!(
        visual
            .points
            .iter()
            .filter(|point| point.x == "Beta")
            .count(),
        2
    );
    assert_eq!(
        visual
            .points
            .iter()
            .filter(|point| point.x == "Gamma")
            .count(),
        2
    );
}

#[test]
fn stat_card_empty_numeric_summary_returns_empty_value() {
    let version = ComponentVersionForTable {
        id: Uuid::nil(),
        component_id: Uuid::nil(),
        dataset_id: Uuid::nil(),
        dataset_version_major: 1,
        component_type: "stat_card".into(),
        config: json!({
            "summary_field": "amount",
            "summary_type": "average",
            "label": "Average award"
        }),
    };
    let rows = vec![
        visual_row(&[("amount", None)]),
        visual_row(&[("amount", Some(""))]),
    ];
    let fields = [field("amount", FieldType::Number)];

    let visual = visual_from_rows(
        version,
        super::VisualComponentConfig::parse(
            "stat_card",
            &json!({
                "summary_field": "amount",
                "summary_type": "average",
                "label": "Average award"
            }),
        )
        .expect("visual config"),
        rows,
        &fields,
    )
    .expect("visual should transform");

    let stat = visual.stat.expect("stat card view model");
    assert_eq!(stat.label, "Average award");
    assert_eq!(stat.value, None);
    assert_eq!(stat.display_value, None);
}

#[test]
fn component_filter_sql_supports_negative_operator() {
    let fields = [field("program", FieldType::Text)];
    let filters = vec![super::ComponentFilterConfig {
        field_key: "program".into(),
        operator: "not_contains".into(),
        value: Some("archived".into()),
    }];
    let refs = fields.iter().collect::<Vec<_>>();

    let sql = component_filter_sql(&filters, &refs).expect("filter should compile");
    assert_eq!(
        sql,
        vec!["POSITION(LOWER('archived') IN LOWER(COALESCE(\"program\", ''))) = 0"]
    );
}

#[test]
fn component_filter_sql_validates_operator_field_compatibility() {
    let fields = [field("score", FieldType::Number)];
    let filters = vec![super::ComponentFilterConfig {
        field_key: "score".into(),
        operator: "contains".into(),
        value: Some("10".into()),
    }];
    let refs = fields.iter().collect::<Vec<_>>();

    let error = component_filter_sql(&filters, &refs)
        .expect_err("text operator on numeric field should fail");
    assert!(
        error
            .to_string()
            .contains("filter operator 'contains' is not supported")
    );
}

#[test]
fn component_table_query_parses_runtime_filters_and_cursor() {
    let mut extra = HashMap::new();
    extra.insert("filter[program][operator]".into(), "not_contains".into());
    extra.insert("filter[program][value]".into(), "archived".into());
    let query = ComponentTableQuery {
        q: Some(" demo ".into()),
        page_size: Some(500),
        cursor: Some("offset:25".into()),
        sort: Some("program:desc".into()),
        visible_columns: Some("program, row_count".into()),
        extra,
    }
    .into_runtime_query()
    .expect("query should parse");

    assert_eq!(query.search.as_deref(), Some("demo"));
    assert_eq!(query.page_size, Some(200));
    assert_eq!(query.offset, 25);
    assert_eq!(query.visible_columns, vec!["program", "row_count"]);
    assert_eq!(query.filters[0].field_key, "program");
    assert_eq!(query.filters[0].operator, "not_contains");
    assert_eq!(query.filters[0].value.as_deref(), Some("archived"));
    assert_eq!(query.sort.expect("sort").direction, "desc");
}

#[test]
fn component_table_sort_and_page_sql_are_server_driven() {
    let fields = [
        field("program", FieldType::Text),
        field("score", FieldType::Number),
    ];
    let refs = fields.iter().collect::<Vec<_>>();
    let sort = parse_component_sort("score:desc").expect("sort should parse");
    let order_by = table_order_by_sql(Some(&sort), &refs, "__row_id").expect("sort should compile");

    assert!(order_by.contains("\"score\""));
    assert!(order_by.contains("DESC"));
    assert_eq!(component_pagination_sql(25, 50), " LIMIT 51 OFFSET 25");
    assert_eq!(effective_component_page_size(None, Some(500)), 200);
    assert_eq!(effective_component_page_size(Some(25), Some(500)), 25);
}

#[test]
fn component_preview_source_limit_is_optional_and_bounded() {
    assert_eq!(
        component_visual_source_limit_clause(Some(100)),
        " LIMIT 100"
    );
    assert_eq!(component_visual_source_limit_clause(Some(0)), " LIMIT 1");
    assert_eq!(component_visual_source_limit_clause(None), "");
}

#[test]
fn visible_table_fields_preserves_requested_order_and_rejects_unknown_columns() {
    let fields = [
        field("program", FieldType::Text),
        field("score", FieldType::Number),
    ];
    let refs = fields.iter().collect::<Vec<_>>();
    let selected = visible_table_fields(&refs, &["score".into(), "program".into()])
        .expect("known visible columns should pass");
    assert_eq!(
        selected
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>(),
        vec!["score", "program"]
    );

    let error = visible_table_fields(&refs, &["missing".into()])
        .expect_err("unknown visible column should fail");
    assert!(
        error
            .to_string()
            .contains("visible column 'missing' is outside")
    );
}

#[test]
fn table_search_defaults_to_component_projection_contract() {
    let config = super::TableComponentConfig {
        visible_columns: vec![super::ComponentFieldRef::Key("score".into())],
        filters: Vec::new(),
        search_fields: Vec::new(),
        default_sort: None,
        page_size: None,
        display_labels: BTreeMap::new(),
    };
    let fields = [
        field("program", FieldType::Text),
        field("score", FieldType::Number),
    ];

    let refs = fields.iter().collect::<Vec<_>>();
    let selected = visible_table_fields(&refs, &["score".into()])
        .expect("visible column projection should pass");

    let search_fields = table_search_fields(&config, &selected).expect("search fields");

    assert_eq!(
        selected
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>(),
        vec!["score"]
    );
    assert_eq!(
        search_fields
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>(),
        vec!["score"]
    );
}

#[test]
fn runtime_visible_columns_can_only_narrow_component_projection() {
    let fields = [
        field("program", FieldType::Text),
        field("score", FieldType::Number),
        field("hidden", FieldType::Text),
    ];
    let refs = fields.iter().collect::<Vec<_>>();
    let component_contract = visible_table_fields(&refs, &["program".into(), "score".into()])
        .expect("configured projection should pass");

    let selected = visible_table_fields(&component_contract, &["score".into()])
        .expect("query projection can narrow component projection");
    assert_eq!(
        selected
            .iter()
            .map(|field| field.key.as_str())
            .collect::<Vec<_>>(),
        vec!["score"]
    );

    let error = visible_table_fields(&component_contract, &["hidden".into()])
        .expect_err("query projection cannot expand component projection");
    assert!(
        error
            .to_string()
            .contains("visible column 'hidden' is outside")
    );
}

#[test]
fn component_query_filters_require_operators() {
    let mut extra = HashMap::new();
    extra.insert("filter[program][value]".into(), "demo".into());

    let error = match parse_component_query_filters(&extra) {
        Ok(_) => panic!("missing operator should fail"),
        Err(error) => error.to_string(),
    };
    assert!(error.contains("missing an operator"));
}

#[test]
fn component_publish_guard_rejects_immutable_versions() {
    let version_id = Uuid::nil();

    assert!(require_component_version_draft(version_id, "draft").is_ok());
    assert!(require_component_version_draft(version_id, "published").is_err());
    assert!(require_component_version_draft(version_id, "superseded").is_err());
}

#[test]
fn new_component_versions_require_notes() {
    assert!(super::require_new_version_note("changed displayed fields").is_ok());
    let error =
        super::require_new_version_note("   ").expect_err("blank new-version note should fail");
    assert!(error.to_string().contains("require a version note"));
}

#[test]
fn component_table_without_materialization_uses_pending_state() {
    let version = ComponentVersionForTable {
        id: Uuid::new_v4(),
        component_id: Uuid::new_v4(),
        dataset_id: Uuid::new_v4(),
        dataset_version_major: 1,
        component_type: "table".into(),
        config: json!({ "visible_columns": ["program"] }),
    };

    let table = super::empty_component_table(version, "pending", Vec::new());

    assert_eq!(table.materialization_state, "pending");
    assert!(table.rows.is_empty());
    assert_eq!(table.pagination.page_size, 0);
    assert!(!table.pagination.has_more);
}

#[test]
fn component_table_materialization_failure_is_render_state() {
    let version = ComponentVersionForTable {
        id: Uuid::new_v4(),
        component_id: Uuid::new_v4(),
        dataset_id: Uuid::new_v4(),
        dataset_version_major: 1,
        component_type: "table".into(),
        config: json!({ "visible_columns": ["program"] }),
    };

    let table = super::empty_component_table(version, "failed", Vec::new());

    assert_eq!(table.materialization_state, "failed");
    assert!(table.rows.is_empty());
    assert!(!table.pagination.has_more);
}

#[test]
fn create_component_request_accepts_first_version_payload() {
    let dataset_id = Uuid::nil();
    let payload: CreateComponentRequest = serde_json::from_value(json!({
        "name": "Program table",
        "slug": "program-table",
        "description": "A first table component",
        "version": {
            "dataset_id": dataset_id,
            "dataset_version_major": 1,
            "component_type": "table",
            "config": {
                "visible_columns": ["program"]
            }
        }
    }))
    .expect("atomic create payload should deserialize");

    assert_eq!(payload.name, "Program table");
    let version = payload.version.expect("version should be present");
    assert_eq!(version.dataset_id, Some(dataset_id));
    assert_eq!(version.dataset_version_major, Some(1));
    assert_eq!(version.component_type, "table");
}

#[test]
fn component_shell_payloads_reject_unknown_fields() {
    let create_error = match serde_json::from_value::<CreateComponentRequest>(json!({
        "name": "Program table",
        "slug": "program-table",
        "description": "A first table component",
        "dataset_revision_id": Uuid::nil()
    })) {
        Ok(_) => panic!("create component shell should reject legacy revision fields"),
        Err(error) => error,
    };
    assert!(create_error.to_string().contains("dataset_revision_id"));

    let update_error = match serde_json::from_value::<UpdateComponentRequest>(json!({
        "name": "Program table",
        "slug": "program-table",
        "description": "Updated table component",
        "dataset_revision_id": Uuid::nil()
    })) {
        Ok(_) => panic!("update component shell should reject legacy revision fields"),
        Err(error) => error,
    };
    assert!(update_error.to_string().contains("dataset_revision_id"));
}

#[test]
fn atomic_component_version_payload_rejects_legacy_revision_binding() {
    let error = match serde_json::from_value::<CreateComponentRequest>(json!({
        "name": "Program table",
        "slug": "program-table",
        "description": "A first table component",
        "version": {
            "dataset_id": Uuid::nil(),
            "dataset_version_major": 1,
            "dataset_revision_id": Uuid::nil(),
            "component_type": "table",
            "config": {
                "visible_columns": ["program"]
            }
        }
    })) {
        Ok(_) => panic!("atomic create version should reject legacy revision fields"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("dataset_revision_id"));
}

#[test]
fn component_version_payload_rejects_legacy_revision_binding() {
    let error = match serde_json::from_value::<CreateComponentVersionRequest>(json!({
        "dataset_revision_id": Uuid::nil(),
        "component_type": "table",
        "config": {
            "visible_columns": ["program"]
        }
    })) {
        Ok(_) => panic!("legacy revision-bound payload should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("dataset_revision_id"));
}

#[test]
fn component_version_payload_rejects_inline_publish_flag() {
    let error = match serde_json::from_value::<CreateComponentVersionRequest>(json!({
        "dataset_id": Uuid::nil(),
        "dataset_version_major": 1,
        "component_type": "table",
        "config": {
            "visible_columns": ["program"]
        },
        "publish": true
    })) {
        Ok(_) => panic!("inline publish flag should be rejected"),
        Err(error) => error,
    };

    assert!(error.to_string().contains("publish"));
}
