use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;
use tracing_subscriber::EnvFilter;

#[path = "support/datasets.rs"]
mod dataset_support;

use dataset_support::{
    aggregation_operation, calculated_fields_operation, detail_payload_for_restricted_tier,
    filter_operation, projection_operation,
};

static TEST_DATABASE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));
static TEST_TRACING: LazyLock<()> = LazyLock::new(|| {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("tessara_api=debug,sqlx=warn")),
        )
        .with_test_writer()
        .try_init();
});

#[tokio::test]
async fn demo_seed_uses_capability_scope_ownership_and_components() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    assert_eq!(seed["seed_version"], "uat-demo-v1");
    assert_eq!(seed["dataset_count"], 4);
    assert_eq!(seed["dataset_revision_count"], 4);
    assert_eq!(seed["component_count"], 4);
    assert_eq!(seed["dashboard_count"], 1);

    let summary = request_json(
        app.clone(),
        authorized_request("GET", "/api/summary", &admin_token, None),
    )
    .await;
    assert_eq!(summary["datasets"], 4);
    assert_eq!(summary["dataset_revisions"], 4);
    assert_eq!(summary["components"], 4);
    assert_eq!(summary["component_versions"], 4);
    assert_eq!(summary["dashboards"], 1);
    assert!(summary.get("reports").is_none());
    assert!(summary.get("charts").is_none());

    let components = request_json(
        app.clone(),
        authorized_request("GET", "/api/components", &admin_token, None),
    )
    .await;
    assert!(
        components
            .as_array()
            .expect("components should be an array")
            .iter()
            .any(|component| component["current_version_id"] == seed["component_version_id"])
    );

    let dashboard = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/dashboards/{}",
                seed["dashboard_id"].as_str().expect("dashboard id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        dashboard["components"]
            .as_array()
            .expect("dashboard components should be an array")
            .len(),
        4
    );
    assert!(
        dashboard["components"]
            .as_array()
            .expect("dashboard components should be an array")
            .iter()
            .any(|component| component["component_version_id"] == seed["component_version_id"])
    );

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let operator_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &operator_token, None),
    )
    .await;
    assert!(
        operator_me["capabilities"]
            .as_array()
            .expect("capabilities should be an array")
            .iter()
            .any(|capability| capability == "forms:read")
    );
    assert!(
        !operator_me["scope_nodes"]
            .as_array()
            .expect("operator should have scoped nodes")
            .is_empty()
    );

    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;
    let respondent_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &respondent_token, None),
    )
    .await;
    assert!(
        respondent_me["capabilities"]
            .as_array()
            .expect("capabilities should be an array")
            .iter()
            .any(|capability| capability == "submissions:read_own")
    );

    let respondent_submissions = request_json(
        app.clone(),
        authorized_request("GET", "/api/submissions", &respondent_token, None),
    )
    .await;
    assert!(
        respondent_submissions
            .as_array()
            .expect("respondent submissions should be an array")
            .iter()
            .all(|submission| submission["assigned_to_display_name"]
                == respondent_me["display_name"])
    );
}

#[tokio::test]
async fn seeded_capability_catalog_uses_components_and_dashboards() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", &admin_token, None),
    )
    .await;
    let keys = capabilities
        .as_array()
        .expect("capabilities should be an array")
        .iter()
        .map(|capability| capability["key"].as_str().expect("capability key"))
        .collect::<Vec<_>>();
    assert!(keys.contains(&"datasets:read"));
    assert!(keys.contains(&"components:read"));
    assert!(keys.contains(&"dashboards:read"));
    assert!(keys.contains(&"operations:view"));
}

#[tokio::test]
async fn dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let form_id = seed["form_id"].as_str().expect("seed form id");
    let form_version_id = seed["form_version_id"]
        .as_str()
        .expect("seed form version id");
    let visibility_node_id = seed["program_node_id"]
        .as_str()
        .expect("seed program node id");

    let rendered_form = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/form-versions/{form_version_id}/render"),
            &admin_token,
            None,
        ),
    )
    .await;
    let fields = rendered_form["sections"]
        .as_array()
        .expect("rendered sections")
        .iter()
        .flat_map(|section| {
            section["fields"]
                .as_array()
                .expect("rendered fields")
                .iter()
        })
        .collect::<Vec<_>>();
    let number_field = fields
        .iter()
        .copied()
        .find(|field| field["field_type"] == "number")
        .expect("demo form should include a numeric field");
    let date_field = fields
        .iter()
        .copied()
        .find(|field| {
            field["field_type"] == "date"
                || field["field_type"] == "datetime"
                || field["field_type"] == "timestamp"
        })
        .expect("demo form should include a date-like field");
    let text_field = fields
        .iter()
        .copied()
        .find(|field| field["key"] == "submission_status")
        .or_else(|| {
            fields
                .iter()
                .copied()
                .find(|field| field["field_type"] == "text")
        })
        .expect("demo form should include a text field");
    let boolean_field = fields
        .iter()
        .copied()
        .find(|field| field["field_type"] == "boolean")
        .expect("demo form should include a boolean field");
    let number_key = number_field["key"].as_str().expect("number field key");
    let number_label = number_field["label"].as_str().expect("number label");
    let date_key = date_field["key"].as_str().expect("date field key");
    let date_label = date_field["label"].as_str().expect("date label");
    let text_key = text_field["key"].as_str().expect("text field key");
    let text_label = text_field["label"].as_str().expect("text label");
    let boolean_key = boolean_field["key"].as_str().expect("boolean field key");
    let boolean_label = boolean_field["label"]
        .as_str()
        .expect("boolean field label");

    let payload = json!({
        "name": "Advanced Authoring UAT Dataset",
        "slug": "advanced-authoring-uat-dataset",
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
                "key": number_key,
                "label": number_label,
                "source_alias": "form_a",
                "source_field_key": number_key,
                "position": 0
            }, {
                "key": date_key,
                "label": date_label,
                "source_alias": "form_a",
                "source_field_key": date_key,
                "position": 1
            }, {
                "key": text_key,
                "label": text_label,
                "source_alias": "form_a",
                "source_field_key": text_key,
                "position": 2
            }, {
                "key": boolean_key,
                "label": boolean_label,
                "source_alias": "form_a",
                "source_field_key": boolean_key,
                "position": 3
            }]), 0),
            calculated_fields_operation(json!([{
                "key": "date_lte_self",
                "label": "Date Lte Self",
                "base_field_key": date_key,
                "functions": [{
                    "function": "less_than_or_equal",
                    "argument": null,
                    "argument_mode": "field",
                    "argument_field_key": date_key,
                    "position": 0
                }],
                "position": 0
            }, {
                "key": "status_mapped",
                "label": "Status Mapped",
                "base_field_key": text_key,
                "functions": [{
                    "function": "to_text",
                    "argument": null,
                    "argument_mode": "value",
                    "argument_field_key": null,
                    "position": 0
                }, {
                    "function": "map_value",
                    "argument": "draft=>booger, submitted=>snot",
                    "argument_mode": "value",
                    "argument_field_key": null,
                    "position": 1
                }],
                "position": 1
            }, {
                "key": "internal_flag",
                "label": "Internal Flag",
                "base_field_key": boolean_key,
                "functions": [{
                    "function": "constant",
                    "argument": "true",
                    "argument_mode": "value",
                    "argument_field_key": null,
                    "position": 0
                }],
                "position": 2
            }, {
                "key": "restricted_flag",
                "label": "Restricted Flag",
                "base_field_key": boolean_key,
                "functions": [{
                    "function": "constant",
                    "argument": "true",
                    "argument_mode": "value",
                    "argument_field_key": null,
                    "position": 0
                }],
                "position": 3
            }]), 1),
            filter_operation(json!([{
                "field_key": number_key,
                "operator": "greater_than_or_equal",
                "value_mode": "field",
                "value": null,
                "value_field_key": number_key,
                "position": 0
            }, {
                "field_key": date_key,
                "operator": "less_than_or_equal",
                "value_mode": "field",
                "value": null,
                "value_field_key": date_key,
                "position": 1
            }]), 2)
        ],
        "restriction_policy": {
            "internal_field_key": "internal_flag",
            "restricted_field_key": "restricted_flag"
        }
    });

    let preview = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(payload.clone()),
        ),
    )
    .await;
    let preview_sql = preview["generated_sql"]
        .as_str()
        .expect("preview should include generated SQL");
    assert!(preview_sql.contains("filtered_fields"));
    assert!(preview_sql.contains("__restriction_tier"));
    assert!(preview_sql.contains(&format!(
        "NULLIF(\"{date_key}\", '')::date <= NULLIF(\"{date_key}\", '')::date"
    )));
    assert!(preview_sql.contains("booger"));
    assert!(preview_sql.contains("snot"));
    assert!(
        preview_sql
            .find("\"restricted_flag\"")
            .expect("restricted flag tier branch")
            < preview_sql
                .find("\"internal_flag\"")
                .expect("internal flag tier branch"),
        "the more sensitive restricted tier should be evaluated before internal when multiple flags are true"
    );

    let created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(payload.clone()),
        ),
    )
    .await;
    let dataset_id = created["id"].as_str().expect("created dataset id");
    let detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        detail["output_fields"]
            .as_array()
            .expect("output fields")
            .iter()
            .any(|field| field["key"] == number_key || field["key"] == date_key),
        "source fields used by filters and calculations should remain analytical output fields"
    );
    assert!(
        detail["output_fields"]
            .as_array()
            .expect("output fields")
            .iter()
            .any(|field| field["key"] == "date_lte_self")
    );
    let calculated_detail_operation = detail_operation(&detail, "calculated_fields");
    let filter_detail_operation = detail_operation(&detail, "filter");
    assert_eq!(
        calculated_detail_operation["fields"][0]["functions"][0]["argument_mode"],
        "field"
    );
    assert_eq!(
        filter_detail_operation["filters"][0]["value_field_key"],
        number_key
    );

    let admin_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    let admin_rows = admin_table["rows"].as_array().expect("admin rows");
    assert!(!admin_rows.is_empty());
    assert!(
        admin_rows
            .iter()
            .any(|row| row["values"].get(number_key).is_some()),
        "numeric filter field should be exposed as an analytical output field"
    );
    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let operator_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert!(
        operator_table["rows"]
            .as_array()
            .expect("operator rows")
            .len()
            < admin_rows.len(),
        "restricted flag should win over internal when both boolean flags are true"
    );

    let mut invalid_function_payload = payload.clone();
    invalid_function_payload["slug"] = json!("advanced-authoring-invalid-function");
    invalid_function_payload["name"] = json!("Advanced Authoring Invalid Function");
    invalid_function_payload["operations"][1]["fields"][0]["functions"][0]["function"] =
        json!("unsupported_function");
    let invalid_function_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(invalid_function_payload),
        ),
    )
    .await;
    assert_eq!(invalid_function_status, StatusCode::BAD_REQUEST);

    let mut invalid_argument_payload = payload;
    invalid_argument_payload["slug"] = json!("advanced-authoring-invalid-argument");
    invalid_argument_payload["name"] = json!("Advanced Authoring Invalid Argument");
    invalid_argument_payload["operations"][1]["fields"][0]["functions"][0]["argument_field_key"] =
        json!(number_key);
    let invalid_argument_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(invalid_argument_payload),
        ),
    )
    .await;
    assert_eq!(invalid_argument_status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn admin_dataset_query_designer_materializes_generated_sql() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let form_id = seed["form_id"].as_str().expect("seed form id");
    let form_version_id = seed["form_version_id"]
        .as_str()
        .expect("seed form version id");
    let visibility_node_id = seed["program_node_id"]
        .as_str()
        .expect("seed program node id");
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let rendered_form = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/form-versions/{form_version_id}/render"),
            &admin_token,
            None,
        ),
    )
    .await;
    let field = &rendered_form["sections"]
        .as_array()
        .expect("rendered sections")[0]["fields"]
        .as_array()
        .expect("rendered fields")[0];
    let boolean_field = rendered_form["sections"]
        .as_array()
        .expect("rendered sections")
        .iter()
        .flat_map(|section| {
            section["fields"]
                .as_array()
                .expect("rendered fields")
                .iter()
        })
        .find(|field| field["field_type"] == "boolean")
        .expect("demo form should include a boolean field");
    let field_key = field["key"].as_str().expect("field key");
    let field_label = field["label"].as_str().expect("field label");
    let boolean_field_key = boolean_field["key"].as_str().expect("boolean field key");
    let boolean_field_label = boolean_field["label"]
        .as_str()
        .expect("boolean field label");
    let payload = json!({
        "name": "Query Designer Test Dataset",
        "slug": "query-designer-test-dataset",
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
                "key": boolean_field_key,
                "label": boolean_field_label,
                "source_alias": "form_a",
                "source_field_key": boolean_field_key,
                "position": 1
            }]), 0),
            calculated_fields_operation(json!([{
                "key": "field_upper",
                "label": "Upper Field",
                "base_field_key": field_key,
                "functions": [{
                    "function": "uppercase",
                    "argument": null,
                    "position": 0
                }],
                "position": 0
            }]), 1),
            filter_operation(json!([{
                "field_key": field_key,
                "operator": "is_not_empty",
                "value_mode": "value",
                "value": null,
                "value_field_key": null,
                "position": 0
            }]), 2)
        ],
        "restriction_policy": {
            "internal_field_key": boolean_field_key
        }
    });
    let legacy_payload = json!({
        "name": "Legacy Flat Source Dataset",
        "slug": "legacy-flat-source-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "form",
            "alias": "form_a",
            "form_id": form_id,
            "form_version_id": form_version_id
        },
        "sources": [{
            "source_alias": "form_a",
            "form_id": form_id,
            "form_version_id": form_version_id
        }],
        "fields": [{
            "key": field_key,
            "label": field_label,
            "source_alias": "form_a",
            "source_field_key": field_key,
            "position": 0
        }]
    });
    let legacy_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(legacy_payload),
        ),
    )
    .await;
    assert!(
        !legacy_status.is_success(),
        "legacy flat source payloads should be rejected"
    );

    let created = request_json(
        app.clone(),
        authorized_request("POST", "/api/admin/datasets", &admin_token, Some(payload)),
    )
    .await;
    let dataset_id = created["id"].as_str().expect("created dataset id");
    let detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(detail["initial_source"]["kind"], "form");
    let generated_sql = detail["generated_sql"]
        .as_str()
        .expect("dataset detail includes generated sql");
    assert!(generated_sql.contains("analytics.submission_fact"));
    assert!(generated_sql.contains("filtered_fields"));
    assert!(generated_sql.contains("calculated_fields"));
    assert!(generated_sql.contains("UPPER"));
    assert!(generated_sql.contains(&format!("NULLIF(\"{field_key}\", '') IS NOT NULL")));
    assert!(generated_sql.contains("submission_value_fact.field_id"));
    assert!(!generated_sql.contains("submission_value_fact.field_key"));
    assert!(!generated_sql.contains("field_dim.field_key"));
    assert!(
        !generated_sql
            .contains("JOIN form_versions ON form_versions.id = submission_fact.form_version_id")
    );
    assert!(generated_sql.contains(field_key));
    let filter_detail_operation = detail_operation(&detail, "filter");
    let calculated_detail_operation = detail_operation(&detail, "calculated_fields");
    assert_eq!(
        filter_detail_operation["filters"][0]["field_key"],
        field_key
    );
    assert_eq!(
        filter_detail_operation["filters"][0]["operator"],
        "is_not_empty"
    );
    assert_eq!(
        calculated_detail_operation["fields"][0]["key"],
        "field_upper"
    );
    assert!(
        detail["fields"]
            .as_array()
            .expect("included fields")
            .iter()
            .any(|field| field["key"] == field_key),
        "included field should persist in the dataset catalog"
    );
    assert!(
        detail["fields"]
            .as_array()
            .expect("included fields")
            .iter()
            .any(|field| field["key"] == boolean_field_key),
        "boolean restriction field should persist in the dataset catalog"
    );
    assert!(
        detail["output_fields"]
            .as_array()
            .expect("output fields")
            .iter()
            .any(|field| field["key"] == field_key),
        "projected analytical fields should appear in output fields"
    );
    assert!(
        detail["output_fields"]
            .as_array()
            .expect("output fields")
            .iter()
            .any(|field| field["key"] == "field_upper"),
        "calculated fields remain visible output fields"
    );
    assert_eq!(
        detail["restriction_policy"]["internal_field_key"],
        boolean_field_key
    );
    assert!(
        detail["materialized_table"]
            .as_str()
            .is_some_and(|table| table.starts_with("dataset_"))
    );
    assert!(
        detail["materialized_row_count"]
            .as_i64()
            .is_some_and(|count| count > 0)
    );

    let table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        table["rows"]
            .as_array()
            .expect("preview rows should be an array")
            .iter()
            .any(|row| row["values"].get(field_key).is_some()),
        "projected analytical fields should be returned in table values"
    );
    assert!(
        table["rows"]
            .as_array()
            .expect("preview rows should be an array")
            .iter()
            .any(|row| row["values"].get("field_upper").is_some())
    );
    let mut no_visible_payload = detail_payload_for_restricted_tier(
        form_id,
        form_version_id,
        visibility_node_id,
        field_key,
        field_label,
        boolean_field_key,
        boolean_field_label,
    );
    no_visible_payload["name"] = json!("No Visible Fields Dataset");
    no_visible_payload["slug"] = json!("no-visible-fields-dataset");
    no_visible_payload["operations"] = json!([{
        "kind": "projection",
        "fields": [],
        "position": 0
    }]);
    no_visible_payload["restriction_policy"] = json!(null);
    let no_visible_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(no_visible_payload),
        ),
    )
    .await;
    assert_eq!(no_visible_status, StatusCode::BAD_REQUEST);
    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let operator_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        operator_table["rows"]
            .as_array()
            .expect("operator preview rows")
            .len(),
        table["rows"].as_array().expect("admin preview rows").len(),
        "scoped readers with dataset visibility should see the full materialized dataset output"
    );
    let mut restricted_payload = detail_payload_for_restricted_tier(
        form_id,
        form_version_id,
        visibility_node_id,
        field_key,
        field_label,
        boolean_field_key,
        boolean_field_label,
    );
    restricted_payload["slug"] = json!("query-designer-restricted-dataset");
    restricted_payload["name"] = json!("Query Designer Restricted Dataset");
    let restricted_created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(restricted_payload),
        ),
    )
    .await;
    let restricted_dataset_id = restricted_created["id"]
        .as_str()
        .expect("restricted dataset id");
    let restricted_admin_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{restricted_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        !restricted_admin_table["rows"]
            .as_array()
            .expect("restricted admin rows")
            .is_empty()
    );
    let restricted_operator_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{restricted_dataset_id}/table"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        restricted_operator_table["rows"]
            .as_array()
            .expect("restricted operator rows")
            .len(),
        0,
        "datasets:read without tier capabilities should not see restricted rows"
    );
    let restricted_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{restricted_dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let restricted_revision_id = restricted_detail["current_revision_id"]
        .as_str()
        .expect("restricted dataset revision id");
    let derived_payload = json!({
        "name": "Derived Restricted Dataset",
        "slug": "derived-restricted-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "dataset",
            "alias": "restricted_source",
            "dataset_id": restricted_dataset_id,
            "dataset_revision_id": restricted_revision_id
        },
        "operations": [
            projection_operation(json!([{
                "key": "derived_value",
                "label": "Derived Value",
                "source_alias": "restricted_source",
                "source_field_key": field_key,
                "position": 0
            }, {
                "key": "derived_restricted_flag",
                "label": "Derived Restricted Flag",
                "source_alias": "restricted_source",
                "source_field_key": "restricted_flag",
                "position": 1
            }]), 0)
        ],
        "restriction_policy": {
            "restricted_field_key": "derived_restricted_flag"
        }
    });
    let derived_created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(derived_payload),
        ),
    )
    .await;
    let derived_dataset_id = derived_created["id"].as_str().expect("derived dataset id");
    let derived_admin_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{derived_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        !derived_admin_table["rows"]
            .as_array()
            .expect("derived admin rows")
            .is_empty()
    );
    let derived_operator_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{derived_dataset_id}/table"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        derived_operator_table["rows"]
            .as_array()
            .expect("derived operator rows")
            .len(),
        0,
        "derived datasets should remain restricted when they explicitly carry and apply a boolean restriction field"
    );
    let mixed_aggregation_payload = json!({
        "name": "Mixed Source Aggregated Dataset",
        "slug": "mixed-source-aggregated-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "form",
            "alias": "public_form",
            "form_id": form_id,
            "form_version_id": form_version_id
        },
        "operations": [
            {
                "kind": "add_source",
                "add_type": "union_all",
                "source": {
                    "kind": "dataset",
                    "alias": "restricted_source",
                    "dataset_id": restricted_dataset_id,
                    "dataset_revision_id": restricted_revision_id
                },
                "position": 0
            },
            projection_operation(json!([{
                "key": "public_value",
                "label": "Public Value",
                "source_alias": "public_form",
                "source_field_key": field_key,
                "position": 0
            }, {
                "key": "restricted_value",
                "label": "Restricted Value",
                "source_alias": "restricted_source",
                "source_field_key": field_key,
                "position": 1
            }]), 1),
            aggregation_operation(json!([]), json!([{
                "key": "mixed_count",
                "label": "Mixed Count",
                "function": "count_rows",
                "source_field_key": null,
                "position": 0
            }]), Value::Null, 2)
        ]
    });
    let mixed_preview = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(mixed_aggregation_payload.clone()),
        ),
    )
    .await;
    let mixed_sql = mixed_preview["generated_sql"]
        .as_str()
        .expect("mixed aggregation generated SQL");
    assert!(!mixed_sql.contains("__source_restriction_rank"));
    assert!(mixed_sql.contains("GREATEST("));
    let mixed_created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(mixed_aggregation_payload),
        ),
    )
    .await;
    let mixed_dataset_id = mixed_created["id"]
        .as_str()
        .expect("mixed aggregate dataset id");
    let mixed_admin_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{mixed_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        mixed_admin_table["rows"]
            .as_array()
            .expect("mixed admin rows")
            .len(),
        1
    );
    let mixed_operator_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{mixed_dataset_id}/table"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        mixed_operator_table["rows"]
            .as_array()
            .expect("mixed operator rows")
            .len(),
        0,
        "mixed-source aggregates inherit the most sensitive upstream restriction tier"
    );
    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;
    let respondent_table_status = request_status(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &respondent_token,
            None,
        ),
    )
    .await;
    assert_eq!(respondent_table_status, StatusCode::FORBIDDEN);

    let aggregation_payload = json!({
        "name": "Query Designer Aggregated Dataset",
        "slug": "query-designer-aggregated-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "form",
            "alias": "form_a",
            "form_id": form_id,
            "form_version_id": form_version_id
        },
        "operations": [
            projection_operation(json!([
                {
                    "key": "node_id",
                    "label": "Attached Node ID",
                    "source_alias": "form_a",
                    "source_field_key": "__node_id",
                    "position": 0
                },
                {
                    "key": field_key,
                    "label": field_label,
                    "source_alias": "form_a",
                    "source_field_key": field_key,
                    "position": 1
                }
            ]), 0),
            aggregation_operation(json!(["node_id"]), json!([{
                "key": "response_count",
                "label": "Response Count",
                "function": "count_rows",
                "source_field_key": null,
                "position": 0
            }]), json!({
                "sort_fields": [{
                    "field_key": field_key,
                    "position": 0
                }],
                "direction": "lowest"
            }), 1),
            calculated_fields_operation(json!([{
                "key": "response_count_plus_one",
                "label": "Response Count Plus One",
                "base_field_key": "response_count",
                "functions": [{
                    "function": "add",
                    "argument": "1",
                    "argument_mode": "value",
                    "argument_field_key": null,
                    "position": 0
                }],
                "position": 0
            }, {
                "key": "response_count_has_rows",
                "label": "Response Count Has Rows",
                "base_field_key": "response_count",
                "functions": [{
                    "function": "greater_than",
                    "argument": "0",
                    "argument_mode": "value",
                    "argument_field_key": null,
                    "position": 0
                }],
                "position": 1
            }]), 2),
            filter_operation(json!([{
                "field_key": "response_count_plus_one",
                "operator": "greater_than_or_equal",
                "value_mode": "value",
                "value": "1",
                "value_field_key": null,
                "position": 0
            }]), 3)
        ],
        "restriction_policy": {
            "restricted_field_key": "response_count_has_rows"
        }
    });
    let preview = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(aggregation_payload.clone()),
        ),
    )
    .await;
    let aggregation_sql = preview["generated_sql"]
        .as_str()
        .expect("aggregation preview sql");
    assert!(aggregation_sql.contains("GROUP BY \"node_id\""));
    let projection_index = aggregation_sql
        .find("projection_2 AS")
        .expect("projection stage");
    let aggregated_index = aggregation_sql
        .find("aggregation_3 AS")
        .expect("aggregation stage");
    let calculated_index = aggregation_sql
        .find("calculated_fields_4 AS")
        .expect("calculated_fields stage");
    let filtered_index = aggregation_sql
        .find("filtered_fields_5 AS")
        .expect("filtered_fields stage");
    let final_index = aggregation_sql
        .rfind("FROM \"filtered_fields_5\"")
        .expect("final select uses filtered_fields");
    assert!(projection_index < aggregated_index);
    assert!(aggregated_index < calculated_index);
    assert!(calculated_index < filtered_index);
    assert!(filtered_index < final_index);
    assert!(aggregation_sql.contains("NULLIF(\"response_count\", '')::numeric + 1"));
    assert!(
        aggregation_sql.contains(
            "NULLIF(\"response_count_plus_one\", '')::numeric >= NULLIF('1', '')::numeric"
        )
    );
    assert!(aggregation_sql.contains("LOWER(COALESCE(\"response_count_has_rows\", ''))"));
    let mut invalid_average_payload = aggregation_payload.clone();
    let created_aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(aggregation_payload),
        ),
    )
    .await;
    let aggregated_dataset_id = created_aggregation["id"]
        .as_str()
        .expect("created aggregated dataset id");
    let aggregated_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{aggregated_dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        detail_operation(&aggregated_detail, "aggregation")
            .get("scope_mode")
            .is_none(),
        "dataset aggregation should not expose implicit row-scope mode"
    );
    assert!(
        aggregated_detail["output_fields"]
            .as_array()
            .expect("output fields")
            .iter()
            .any(|field| field["key"] == "response_count")
    );
    let aggregated_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{aggregated_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        aggregated_table["rows"]
            .as_array()
            .expect("aggregated preview rows")
            .iter()
            .any(|row| row["values"].get("response_count_plus_one").is_some())
    );
    let aggregated_operator_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{aggregated_dataset_id}/table"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        aggregated_operator_table["rows"]
            .as_array()
            .expect("aggregated operator rows")
            .len(),
        0,
        "restriction policy should be evaluated after aggregate calculated fields"
    );

    invalid_average_payload["name"] = json!("Invalid Average Dataset");
    invalid_average_payload["slug"] = json!("invalid-average-dataset");
    invalid_average_payload["restriction_policy"] = json!(null);
    invalid_average_payload["operations"] = json!([
        invalid_average_payload["operations"][0].clone(),
        aggregation_operation(
            json!(["node_id"]),
            json!([{
                "key": "average_text",
                "label": "Average Text",
                "function": "average",
                "source_field_key": field_key,
                "position": 0
            }]),
            Value::Null,
            1
        )
    ]);
    let invalid_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(invalid_average_payload.clone()),
        ),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::BAD_REQUEST);

    let mut max_text_payload = invalid_average_payload.clone();
    max_text_payload["name"] = json!("Max Text Dataset");
    max_text_payload["slug"] = json!("max-text-dataset");
    max_text_payload["operations"][1]["metrics"] = json!([{
        "key": "max_text",
        "label": "Max Text",
        "function": "max",
        "source_field_key": field_key,
        "position": 0
    }]);
    let max_text_preview = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(max_text_payload),
        ),
    )
    .await;
    assert!(
        max_text_preview["generated_sql"]
            .as_str()
            .is_some_and(|sql| sql.contains("max_text"))
    );

    let hidden_join_key_payload = json!({
        "name": "Query Designer Joined Dataset",
        "slug": "query-designer-joined-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "form",
            "alias": "left_form",
            "form_id": form_id,
            "form_version_id": form_version_id
        },
        "operations": [
            {
                "kind": "add_source",
                "add_type": "inner_join",
                "source": {
                    "kind": "form",
                    "alias": "right_form",
                    "form_id": form_id,
                    "form_version_id": form_version_id
                },
                "join_keys": [{
                    "left_field": "left_form__node_id",
                    "right_field": "right_form__node_id"
                }],
                "position": 0
            },
            projection_operation(json!([
                {
                    "key": format!("left_form__{field_key}"),
                    "label": format!("Left {field_label}"),
                    "source_alias": "left_form",
                    "source_field_key": field_key,
                    "position": 0
                },
                {
                    "key": format!("right_form__{field_key}"),
                    "label": format!("Right {field_label}"),
                    "source_alias": "right_form",
                    "source_field_key": field_key,
                    "position": 1
                }
            ]), 0)
        ]
    });
    let hidden_join_key_preview = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets/sql-preview",
            &admin_token,
            Some(hidden_join_key_payload),
        ),
    )
    .await;
    let hidden_join_key_sql = hidden_join_key_preview["generated_sql"]
        .as_str()
        .expect("hidden join key preview sql");
    assert!(hidden_join_key_sql.contains("INNER JOIN"));
    assert!(hidden_join_key_sql.contains("l.\"left_form__node_id\" = r.\"right_form__node_id\""));
    assert!(hidden_join_key_sql.contains("submission_value_fact.field_id"));
    assert!(!hidden_join_key_sql.contains("submission_value_fact.field_key"));
    assert!(!hidden_join_key_sql.contains("field_dim.field_key"));
}

async fn test_app() -> Option<axum::Router> {
    LazyLock::force(&TEST_TRACING);
    let database_url = match std::env::var("TEST_DATABASE_URL") {
        Ok(url) => url,
        Err(_) => {
            eprintln!("skipping database integration test because TEST_DATABASE_URL is not set");
            return None;
        }
    };
    let reset_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    sqlx::query("DROP SCHEMA public CASCADE")
        .execute(&reset_pool)
        .await
        .expect("drop test database schema");
    sqlx::query("DROP SCHEMA IF EXISTS analytics CASCADE")
        .execute(&reset_pool)
        .await
        .expect("drop analytics schema");
    sqlx::query("CREATE SCHEMA public")
        .execute(&reset_pool)
        .await
        .expect("create test database schema");
    reset_pool.close().await;
    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".into(),
        dev_admin_email: "admin@tessara.local".into(),
        dev_admin_password: "tessara-dev-admin".into(),
        auth_cookie_name: "tessara_session".into(),
        auth_cookie_secure: false,
        auth_session_ttl_hours: 12,
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("prepare database");
    Some(router(db::AppState { pool, config }))
}

async fn login_token_for(app: axum::Router, email: &str, password: &str) -> String {
    let response = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({ "email": email, "password": password }).to_string(),
            ))
            .expect("valid login request"),
    )
    .await;
    response["token"]
        .as_str()
        .expect("login response should include token")
        .to_string()
}

async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let (status, body) = request_status_and_json(app, request).await;
    assert!(
        status.is_success(),
        "expected success status for {method} {uri}, got {status}: {body}"
    );
    body
}

async fn request_status_and_json(app: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app.oneshot(request).await.expect("request should succeed");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 1_000_000)
        .await
        .expect("read response body");
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).unwrap_or_else(|_| {
            panic!(
                "response should be json, status {status}, body {}",
                String::from_utf8_lossy(&bytes)
            )
        })
    };
    (status, body)
}

fn authorized_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    builder = builder.header(header::AUTHORIZATION, format!("Bearer {token}"));
    if body.is_some() {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
    }
    builder
        .body(match body {
            Some(body) => Body::from(body.to_string()),
            None => Body::empty(),
        })
        .expect("valid authorized request")
}

fn detail_operation<'a>(detail: &'a Value, kind: &str) -> &'a Value {
    detail["operations"]
        .as_array()
        .and_then(|operations| {
            operations
                .iter()
                .find(|operation| operation["kind"] == kind)
        })
        .unwrap_or_else(|| panic!("detail response should include {kind} operation"))
}

async fn request_status(app: axum::Router, request: Request<Body>) -> StatusCode {
    app.oneshot(request)
        .await
        .expect("request should succeed")
        .status()
}
