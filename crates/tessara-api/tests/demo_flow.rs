use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{PgPool, postgres::PgPoolOptions};
use tessara_api::{config::Config, db, legacy_import, router};
use tower::ServiceExt;
use uuid::Uuid;

static TEST_DATABASE_LOCK: LazyLock<tokio::sync::Mutex<()>> =
    LazyLock::new(|| tokio::sync::Mutex::new(()));

#[tokio::test]
async fn demo_seed_report_and_dashboard_flow_works_against_database() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let (asset_status, asset_body) = request_status_and_text(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/assets/tessara-favicon-32.svg")
            .body(Body::empty())
            .expect("valid asset request"),
    )
    .await;
    assert_eq!(asset_status, StatusCode::OK);
    assert!(asset_body.contains("<svg"));

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;
    assert_eq!(seed["analytics_values"], 1);
    let app_summary = request_json(
        app.clone(),
        authorized_request("GET", "/api/app/summary", &token, None),
    )
    .await;
    assert_eq!(app_summary["published_form_versions"], 1);
    assert_eq!(app_summary["submitted_submissions"], 1);
    assert_eq!(app_summary["datasets"], 0);
    assert_eq!(app_summary["reports"], 1);
    assert_eq!(app_summary["aggregations"], 0);
    assert_eq!(app_summary["dashboards"], 1);
    assert_eq!(app_summary["charts"], 1);
    let legacy_validation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/legacy-fixtures/validate",
            &token,
            Some(json!({
                "fixture_json": include_str!("../../../fixtures/legacy-rehearsal.json")
            })),
        ),
    )
    .await;
    assert_eq!(legacy_validation["issue_count"], 0);
    let legacy_dry_run = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/legacy-fixtures/dry-run",
            &token,
            Some(json!({
                "fixture_json": include_str!("../../../fixtures/legacy-rehearsal.json")
            })),
        ),
    )
    .await;
    assert_eq!(legacy_dry_run["fixture_name"], "legacy-rehearsal");
    assert_eq!(legacy_dry_run["would_import"], true);
    assert_eq!(legacy_dry_run["validation"]["issue_count"], 0);
    let legacy_import = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/legacy-fixtures/import",
            &token,
            Some(json!({
                "fixture_json": include_str!("../../../fixtures/legacy-rehearsal.json")
            })),
        ),
    )
    .await;
    assert_eq!(legacy_import["fixture_name"], "legacy-rehearsal");
    assert!(
        legacy_import["analytics_values"]
            .as_i64()
            .expect("legacy import should report analytics value count")
            >= 1
    );
    let legacy_examples = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/legacy-fixtures/examples", &token, None),
    )
    .await;
    assert!(
        legacy_examples
            .as_array()
            .expect("legacy example response should be an array")
            .iter()
            .any(|example| example["name"] == "legacy-inactive-locked"
                && example["fixture_json"]
                    .as_str()
                    .expect("example should include fixture json")
                    .contains("Inactive Partner"))
    );

    let node_types = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-types", &token, None),
    )
    .await;
    assert!(
        node_types
            .as_array()
            .expect("node type response should be an array")
            .iter()
            .any(|node_type| node_type["slug"] == "organization"
                && node_type["node_count"].as_i64().unwrap_or_default() >= 1)
    );
    let relationships = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-type-relationships", &token, None),
    )
    .await;
    assert!(
        relationships
            .as_array()
            .expect("relationships response should be an array")
            .iter()
            .any(|relationship| relationship["parent_name"] == "Organization"
                && relationship["child_name"] == "Program")
    );
    let metadata_fields = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-metadata-fields", &token, None),
    )
    .await;
    assert!(
        metadata_fields
            .as_array()
            .expect("metadata response should be an array")
            .iter()
            .any(|field| field["key"] == "region"
                && field["node_type_name"] == "Organization"
                && field["required"] == true)
    );
    let nodes = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/nodes?q=Demo")
            .body(Body::empty())
            .expect("valid nodes request"),
    )
    .await;
    assert!(
        nodes
            .as_array()
            .expect("nodes response should be an array")
            .iter()
            .any(|node| node["name"] == "Demo Organization"
                && node["node_type_name"] == "Organization"
                && node["metadata"]["region"] == "North")
    );

    let forms = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/forms", &token, None),
    )
    .await;
    assert!(
        forms
            .as_array()
            .expect("forms response should be an array")
            .iter()
            .any(|form| form["id"] == seed["form_id"]
                && form["versions"][0]["id"] == seed["form_version_id"]
                && form["versions"][0]["status"] == "published")
    );
    let published_forms = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/forms/published")
            .body(Body::empty())
            .expect("valid published forms request"),
    )
    .await;
    assert!(
        published_forms
            .as_array()
            .expect("published forms response should be an array")
            .iter()
            .any(|form_version| form_version["form_id"] == seed["form_id"]
                && form_version["form_name"] == "Quarterly Check In"
                && form_version["form_version_id"] == seed["form_version_id"]
                && form_version["version_label"] == "v1")
    );

    let submissions = request_json(
        app.clone(),
        authorized_request("GET", "/api/submissions", &token, None),
    )
    .await;
    assert!(
        submissions
            .as_array()
            .expect("submissions response should be an array")
            .iter()
            .any(|submission| submission["id"] == seed["submission_id"]
                && submission["status"] == "submitted"
                && submission["value_count"] == 1)
    );
    let filtered_submissions = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/submissions?status=submitted&form_id={}&node_id={}&q=Quarterly",
                seed["form_id"]
                    .as_str()
                    .expect("seed should include form id"),
                seed["organization_node_id"]
                    .as_str()
                    .expect("seed should include organization node id")
            ),
            &token,
            None,
        ),
    )
    .await;
    assert!(
        filtered_submissions
            .as_array()
            .expect("filtered submissions response should be an array")
            .iter()
            .any(|submission| submission["id"] == seed["submission_id"]
                && submission["form_id"] == seed["form_id"]
                && submission["created_at"].as_str().is_some())
    );
    let invalid_submission_status = request_status_and_json(
        app.clone(),
        authorized_request("GET", "/api/submissions?status=archived", &token, None),
    )
    .await;
    assert_eq!(invalid_submission_status.0, StatusCode::BAD_REQUEST);
    let submission_id = seed["submission_id"]
        .as_str()
        .expect("seed response should contain submission id");
    let submission_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/submissions/{submission_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(submission_detail["status"], "submitted");
    assert_eq!(submission_detail["form_id"], seed["form_id"]);
    assert!(submission_detail["created_at"].as_str().is_some());
    assert!(
        submission_detail["values"]
            .as_array()
            .expect("submission detail should include values")
            .iter()
            .any(|value| value["key"] == "participants"
                && value["required"] == true
                && value["value"] == 42)
    );
    assert!(
        submission_detail["audit_events"]
            .as_array()
            .expect("submission detail should include audit events")
            .iter()
            .any(|event| event["event_type"] == "seed_demo")
    );
    let draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/submissions/drafts",
            &token,
            Some(json!({
                "form_version_id": seed["form_version_id"],
                "node_id": seed["organization_node_id"]
            })),
        ),
    )
    .await;
    let draft_id = draft["id"]
        .as_str()
        .expect("draft response should contain id");
    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/submissions/{draft_id}"),
            &token,
            None,
        ),
    )
    .await;
    let deleted_draft = request_status_and_json(
        app.clone(),
        authorized_request("GET", &format!("/api/submissions/{draft_id}"), &token, None),
    )
    .await;
    assert_eq!(deleted_draft.0, StatusCode::NOT_FOUND);
    let delete_submitted = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/submissions/{submission_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(delete_submitted.0, StatusCode::BAD_REQUEST);

    let report_id = seed["report_id"]
        .as_str()
        .expect("seed response should contain report id");
    let report = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{report_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(report["rows"][0]["field_value"], "42");
    let reports = request_json(
        app.clone(),
        authorized_request("GET", "/api/reports", &token, None),
    )
    .await;
    assert!(
        reports
            .as_array()
            .expect("reports response should be an array")
            .iter()
            .any(|report| report["id"] == report_id && report["form_name"] == "Quarterly Check In")
    );
    let report_definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/reports/{report_id}"), &token, None),
    )
    .await;
    assert!(
        report_definition["bindings"]
            .as_array()
            .expect("report definition should include bindings")
            .iter()
            .any(|binding| binding["logical_key"] == "participants"
                && binding["source_field_key"] == "participants")
    );
    assert_eq!(report_definition["form_name"], "Quarterly Check In");

    let dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Quarterly Check In Dataset",
                "slug": "quarterly-check-in-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "check_in",
                    "form_id": seed["form_id"],
                    "compatibility_group_id": null,
                    "selection_rule": "all"
                }],
                "fields": [{
                    "key": "participant_count",
                    "label": "Participant Count",
                    "source_alias": "check_in",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let dataset_id = dataset["id"]
        .as_str()
        .expect("dataset response should contain id");
    let dataset_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(dataset_table["dataset_id"], dataset_id);
    assert_eq!(
        dataset_table["rows"][0]["values"]["participant_count"],
        "42"
    );
    let compatibility_group_id = forms
        .as_array()
        .expect("forms response should be an array")
        .iter()
        .find(|form| form["id"] == seed["form_id"])
        .and_then(|form| form["versions"][0]["compatibility_group_id"].as_str())
        .expect("seeded form version should expose a compatibility group id");
    let compatibility_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Quarterly Compatibility Dataset",
                "slug": "quarterly-compatibility-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "check_in",
                    "form_id": null,
                    "compatibility_group_id": compatibility_group_id,
                    "selection_rule": "all"
                }],
                "fields": [{
                    "key": "participant_count",
                    "label": "Participant Count",
                    "source_alias": "check_in",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let compatibility_dataset_id = compatibility_dataset["id"]
        .as_str()
        .expect("compatibility dataset response should contain id");
    let compatibility_dataset_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{compatibility_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        compatibility_dataset_table["rows"][0]["values"]["participant_count"],
        "42"
    );

    let follow_up_form = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &token,
            Some(json!({
                "name": "Follow Up Report",
                "slug": "follow-up-report",
                "scope_node_type_id": null
            })),
        ),
    )
    .await;
    let follow_up_form_id = id_from(&follow_up_form);
    let follow_up_version_id =
        create_form_version(app.clone(), &token, follow_up_form_id, "v1").await;
    let follow_up_section_id =
        create_form_section(app.clone(), &token, follow_up_version_id, "Main").await;
    create_number_field(
        app.clone(),
        &token,
        follow_up_version_id,
        follow_up_section_id,
        "attendees",
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{follow_up_version_id}/publish"),
            &token,
            None,
        ),
    )
    .await;
    let follow_up_draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/submissions/drafts",
            &token,
            Some(json!({
                "form_version_id": follow_up_version_id,
                "node_id": seed["organization_node_id"]
            })),
        ),
    )
    .await;
    let follow_up_submission_id = id_from(&follow_up_draft);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/submissions/{follow_up_submission_id}/values"),
            &token,
            Some(json!({"values": {"attendees": 7}})),
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{follow_up_submission_id}/submit"),
            &token,
            None,
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request("POST", "/api/admin/analytics/refresh", &token, None),
    )
    .await;
    let multi_source_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Program Activity Dataset",
                "slug": "program-activity-dataset",
                "grain": "submission",
                "sources": [
                    {
                        "source_alias": "check_in",
                        "form_id": seed["form_id"],
                        "compatibility_group_id": null,
                        "selection_rule": "all"
                    },
                    {
                        "source_alias": "follow_up",
                        "form_id": follow_up_form_id,
                        "compatibility_group_id": null,
                        "selection_rule": "all"
                    }
                ],
                "fields": [
                    {
                        "key": "participant_count",
                        "label": "Participant Count",
                        "source_alias": "check_in",
                        "source_field_key": "participants",
                        "position": 0
                    },
                    {
                        "key": "attendee_count",
                        "label": "Attendee Count",
                        "source_alias": "follow_up",
                        "source_field_key": "attendees",
                        "position": 1
                    }
                ]
            })),
        ),
    )
    .await;
    let multi_source_dataset_id = multi_source_dataset["id"]
        .as_str()
        .expect("multi-source dataset response should contain id");
    let multi_source_dataset_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{multi_source_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert!(
        multi_source_dataset_table["rows"]
            .as_array()
            .expect("multi-source dataset table should include rows")
            .iter()
            .any(|row| {
                row["source_alias"] == "check_in" && row["values"]["participant_count"] == "42"
            })
    );
    assert!(
        multi_source_dataset_table["rows"]
            .as_array()
            .expect("multi-source dataset table should include rows")
            .iter()
            .any(|row| {
                row["source_alias"] == "follow_up" && row["values"]["attendee_count"] == "7"
            })
    );
    let multi_source_report = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Program Activity Report",
                "form_id": null,
                "dataset_id": multi_source_dataset_id,
                "fields": [
                    {
                        "logical_key": "participant_count",
                        "source_field_key": "participant_count",
                        "computed_expression": null,
                        "missing_policy": "null"
                    },
                    {
                        "logical_key": "attendee_count",
                        "source_field_key": "attendee_count",
                        "computed_expression": null,
                        "missing_policy": "null"
                    }
                ]
            })),
        ),
    )
    .await;
    let multi_source_report_id = multi_source_report["id"]
        .as_str()
        .expect("multi-source report response should contain id");
    let multi_source_report_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{multi_source_report_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert!(
        multi_source_report_table["rows"]
            .as_array()
            .expect("multi-source report table should include rows")
            .iter()
            .any(|row| {
                row["source_alias"] == "check_in"
                    && row["logical_key"] == "participant_count"
                    && row["field_value"] == "42"
            })
    );
    assert!(
        multi_source_report_table["rows"]
            .as_array()
            .expect("multi-source report table should include rows")
            .iter()
            .any(|row| {
                row["source_alias"] == "follow_up"
                    && row["logical_key"] == "attendee_count"
                    && row["field_value"] == "7"
            })
    );

    let dataset_report = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Dataset Participants Report",
                "form_id": null,
                "dataset_id": dataset_id,
                "fields": [
                    {
                        "logical_key": "participant_count",
                        "source_field_key": "participant_count",
                        "computed_expression": null,
                        "missing_policy": "null"
                    },
                    {
                        "logical_key": "response_label",
                        "source_field_key": null,
                        "computed_expression": "literal:Submitted",
                        "missing_policy": "null"
                    }
                ]
            })),
        ),
    )
    .await;
    let dataset_report_id = dataset_report["id"]
        .as_str()
        .expect("dataset report response should contain id");
    let dataset_report_definition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{dataset_report_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        dataset_report_definition["dataset_name"],
        "Quarterly Check In Dataset"
    );
    assert!(
        dataset_report_definition["bindings"]
            .as_array()
            .expect("dataset report definition should include bindings")
            .iter()
            .any(|binding| binding["logical_key"] == "response_label"
                && binding["computed_expression"] == "literal:Submitted")
    );
    let dataset_report_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{dataset_report_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert!(
        dataset_report_table["rows"]
            .as_array()
            .expect("dataset report table should include rows")
            .iter()
            .any(|row| row["logical_key"] == "participant_count" && row["field_value"] == "42")
    );
    assert!(
        dataset_report_table["rows"]
            .as_array()
            .expect("dataset report table should include rows")
            .iter()
            .any(|row| row["logical_key"] == "response_label" && row["field_value"] == "Submitted")
    );
    let dataset_aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/aggregations",
            &token,
            Some(json!({
                "name": "Dataset Participants Aggregation",
                "report_id": dataset_report_id,
                "group_by_logical_key": null,
                "metrics": [
                    {
                        "metric_key": "responses",
                        "source_logical_key": null,
                        "metric_kind": "count"
                    },
                    {
                        "metric_key": "participants_total",
                        "source_logical_key": "participant_count",
                        "metric_kind": "sum"
                    }
                ]
            })),
        ),
    )
    .await;
    let dataset_aggregation_id = dataset_aggregation["id"]
        .as_str()
        .expect("dataset aggregation response should contain id");
    let dataset_aggregation_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/aggregations/{dataset_aggregation_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(dataset_aggregation_table["rows"][0]["group_key"], "All");
    assert_eq!(
        dataset_aggregation_table["rows"][0]["metrics"]["responses"],
        1.0
    );
    assert_eq!(
        dataset_aggregation_table["rows"][0]["metrics"]["participants_total"],
        42.0
    );

    let aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/aggregations",
            &token,
            Some(json!({
                "name": "Participants Aggregation",
                "report_id": report_id,
                "group_by_logical_key": null,
                "metrics": [
                    {
                        "metric_key": "responses",
                        "source_logical_key": null,
                        "metric_kind": "count"
                    },
                    {
                        "metric_key": "participants_total",
                        "source_logical_key": "participants",
                        "metric_kind": "sum"
                    }
                ]
            })),
        ),
    )
    .await;
    let aggregation_id = aggregation["id"]
        .as_str()
        .expect("aggregation response should contain id");
    let aggregations = request_json(
        app.clone(),
        authorized_request("GET", "/api/aggregations", &token, None),
    )
    .await;
    assert!(
        aggregations
            .as_array()
            .expect("aggregation list should be an array")
            .iter()
            .any(|aggregation| aggregation["id"] == aggregation_id
                && aggregation["metric_count"] == 2)
    );
    let aggregation_definition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/aggregations/{aggregation_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(aggregation_definition["name"], "Participants Aggregation");
    assert!(
        aggregation_definition["metrics"]
            .as_array()
            .expect("aggregation definition should include metrics")
            .iter()
            .any(|metric| metric["metric_key"] == "participants_total"
                && metric["metric_kind"] == "sum")
    );
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/aggregations/{aggregation_id}"),
            &token,
            Some(json!({
                "name": "Updated Participants Aggregation",
                "report_id": report_id,
                "group_by_logical_key": null,
                "metrics": [
                    {
                        "metric_key": "responses",
                        "source_logical_key": null,
                        "metric_kind": "count"
                    },
                    {
                        "metric_key": "participants_total",
                        "source_logical_key": "participants",
                        "metric_kind": "sum"
                    }
                ]
            })),
        ),
    )
    .await;
    let aggregation_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/aggregations/{aggregation_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(aggregation_table["rows"][0]["group_key"], "All");
    assert_eq!(aggregation_table["rows"][0]["metrics"]["responses"], 1.0);
    assert_eq!(
        aggregation_table["rows"][0]["metrics"]["participants_total"],
        42.0
    );

    let aggregation_chart = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/charts",
            &token,
            Some(json!({
                "name": "Participants Totals Chart",
                "report_id": null,
                "aggregation_id": aggregation_id,
                "chart_type": "table"
            })),
        ),
    )
    .await;
    let aggregation_chart_id = aggregation_chart["id"]
        .as_str()
        .expect("aggregation chart response should contain id");
    let invalid_chart = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/charts",
            &token,
            Some(json!({
                "name": "Invalid Mixed Source Chart",
                "report_id": report_id,
                "aggregation_id": aggregation_id,
                "chart_type": "table"
            })),
        ),
    )
    .await;
    assert_eq!(invalid_chart.0, StatusCode::BAD_REQUEST);
    assert!(
        invalid_chart.1["error"]
            .as_str()
            .expect("invalid chart error should include message")
            .contains("either a report or an aggregation")
    );

    let chart_id = seed["chart_id"]
        .as_str()
        .expect("seed response should contain chart id");
    let charts = request_json(
        app.clone(),
        authorized_request("GET", "/api/charts", &token, None),
    )
    .await;
    assert!(
        charts
            .as_array()
            .expect("charts response should be an array")
            .iter()
            .any(|chart| chart["id"] == chart_id
                && chart["chart_type"] == "table"
                && chart["report_name"] == "Participants Report"
                && chart["report_form_name"] == "Quarterly Check In"
                && chart["report_id"] == report_id)
    );
    assert!(
        charts
            .as_array()
            .expect("charts response should be an array")
            .iter()
            .any(|chart| chart["id"] == aggregation_chart_id
                && chart["aggregation_id"] == aggregation_id
                && chart["aggregation_name"] == "Updated Participants Aggregation"
                && chart["aggregation_report_name"] == "Participants Report"
                && chart["aggregation_url"]
                    .as_str()
                    .expect("aggregation chart should include execution url")
                    .contains(aggregation_id))
    );

    let dashboard_id = seed["dashboard_id"]
        .as_str()
        .expect("seed response should contain dashboard id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/dashboards/{dashboard_id}/components"),
            &token,
            Some(json!({
                "chart_id": aggregation_chart_id,
                "position": 1,
                "config": {
                    "title": "Participant Totals"
                }
            })),
        ),
    )
    .await;
    let dashboard = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{dashboard_id}"))
            .body(Body::empty())
            .expect("valid dashboard request"),
    )
    .await;
    assert_eq!(dashboard["components"][0]["chart"]["report_id"], report_id);
    assert_eq!(
        dashboard["components"][0]["chart"]["report_name"],
        "Participants Report"
    );
    assert_eq!(
        dashboard["components"][0]["chart"]["report_form_name"],
        "Quarterly Check In"
    );
    assert!(
        dashboard["components"]
            .as_array()
            .expect("dashboard response should include components")
            .iter()
            .any(|component| component["chart"]["id"] == aggregation_chart_id
                && component["chart"]["aggregation_id"] == aggregation_id
                && component["chart"]["aggregation_name"] == "Updated Participants Aggregation")
    );
    let dashboards = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/dashboards")
            .body(Body::empty())
            .expect("valid dashboard list request"),
    )
    .await;
    assert!(
        dashboards
            .as_array()
            .expect("dashboards response should be an array")
            .iter()
            .any(|dashboard| dashboard["id"] == dashboard_id)
    );

    let disposable_aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/aggregations",
            &token,
            Some(json!({
                "name": "Disposable Aggregation",
                "report_id": dataset_report_id,
                "group_by_logical_key": null,
                "metrics": [
                    {
                        "metric_key": "responses",
                        "source_logical_key": null,
                        "metric_kind": "count"
                    }
                ]
            })),
        ),
    )
    .await;
    let disposable_aggregation_id = disposable_aggregation["id"]
        .as_str()
        .expect("disposable aggregation response should contain id");
    let deleted_aggregation = request_json(
        app,
        authorized_request(
            "DELETE",
            &format!("/api/admin/aggregations/{disposable_aggregation_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(deleted_aggregation["id"], disposable_aggregation_id);
}

#[tokio::test]
async fn form_builder_guards_cross_version_sections_and_supersedes_previous_publish() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let form = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &token,
            Some(json!({
                "name": "Monthly Service Report",
                "slug": "monthly-service-report",
                "scope_node_type_id": null
            })),
        ),
    )
    .await;
    let form_id = id_from(&form);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/forms/{form_id}"),
            &token,
            Some(json!({
                "name": "Monthly Services Report",
                "slug": "monthly-services-report",
                "scope_node_type_id": null
            })),
        ),
    )
    .await;

    let version_one_id = create_form_version(app.clone(), &token, form_id, "v1").await;
    let duplicate_version_label = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/forms/{form_id}/versions"),
            &token,
            Some(json!({
                "version_label": "v1",
                "compatibility_group_name": "Default compatibility"
            })),
        ),
    )
    .await;
    assert_eq!(duplicate_version_label.0, StatusCode::BAD_REQUEST);
    assert!(
        duplicate_version_label.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("already in use")
    );
    let section_one_id = create_form_section(app.clone(), &token, version_one_id, "Main").await;
    let field_one_id = create_number_field(
        app.clone(),
        &token,
        version_one_id,
        section_one_id,
        "participants",
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/form-sections/{section_one_id}"),
            &token,
            Some(json!({
                "title": "Updated Main",
                "position": 1
            })),
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/form-fields/{field_one_id}"),
            &token,
            Some(json!({
                "section_id": section_one_id,
                "key": "participant_count",
                "label": "Participant Count",
                "field_type": "number",
                "required": false,
                "position": 1
            })),
        ),
    )
    .await;

    let removable_section_id =
        create_form_section(app.clone(), &token, version_one_id, "Removable").await;
    let removable_field_id = create_number_field(
        app.clone(),
        &token,
        version_one_id,
        removable_section_id,
        "temporary_count",
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/form-fields/{removable_field_id}"),
            &token,
            None,
        ),
    )
    .await;
    let after_field_delete = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/form-versions/{version_one_id}/render"))
            .body(Body::empty())
            .expect("valid render request"),
    )
    .await;
    assert!(
        !after_field_delete["sections"]
            .as_array()
            .expect("sections should be an array")
            .iter()
            .flat_map(|section| {
                section["fields"]
                    .as_array()
                    .expect("section fields should be an array")
            })
            .any(|field| field["key"] == "temporary_count")
    );
    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/form-sections/{removable_section_id}"),
            &token,
            None,
        ),
    )
    .await;
    let after_section_delete = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/form-versions/{version_one_id}/render"))
            .body(Body::empty())
            .expect("valid render request"),
    )
    .await;
    assert!(
        !after_section_delete["sections"]
            .as_array()
            .expect("sections should be an array")
            .iter()
            .any(|section| section["id"] == removable_section_id.to_string())
    );
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{version_one_id}/publish"),
            &token,
            None,
        ),
    )
    .await;
    let published_field_delete = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/form-fields/{field_one_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(published_field_delete.0, StatusCode::BAD_REQUEST);
    assert!(
        published_field_delete.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("cannot be modified")
    );
    let published_section_delete = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/form-sections/{section_one_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(published_section_delete.0, StatusCode::BAD_REQUEST);
    assert!(
        published_section_delete.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("cannot be modified")
    );
    let published_field_update = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/form-fields/{field_one_id}"),
            &token,
            Some(json!({
                "section_id": section_one_id,
                "key": "participants",
                "label": "Participants",
                "field_type": "number",
                "required": true,
                "position": 0
            })),
        ),
    )
    .await;
    assert_eq!(published_field_update.0, StatusCode::BAD_REQUEST);
    assert!(
        published_field_update.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("cannot be modified")
    );

    let version_two_id = create_form_version(app.clone(), &token, form_id, "v2").await;
    let cross_version_field = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{version_two_id}/fields"),
            &token,
            Some(json!({
                "section_id": section_one_id,
                "key": "participants",
                "label": "Participants",
                "field_type": "number",
                "required": true,
                "position": 0
            })),
        ),
    )
    .await;
    assert_eq!(cross_version_field.0, StatusCode::BAD_REQUEST);
    assert!(
        cross_version_field.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("same form version")
    );

    let section_two_id = create_form_section(app.clone(), &token, version_two_id, "Main").await;
    create_number_field(
        app.clone(),
        &token,
        version_two_id,
        section_two_id,
        "participants",
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{version_two_id}/publish"),
            &token,
            None,
        ),
    )
    .await;

    let version_one = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/form-versions/{version_one_id}/render"))
            .body(Body::empty())
            .expect("valid render request"),
    )
    .await;
    let version_two = request_json(
        app,
        Request::builder()
            .method("GET")
            .uri(format!("/api/form-versions/{version_two_id}/render"))
            .body(Body::empty())
            .expect("valid render request"),
    )
    .await;
    assert_eq!(version_one["status"], "superseded");
    assert_eq!(version_one["form_name"], "Monthly Services Report");
    assert_eq!(version_one["sections"][0]["title"], "Updated Main");
    assert_eq!(
        version_one["sections"][0]["fields"][0]["key"],
        "participant_count"
    );
    assert_eq!(
        version_one["sections"][0]["fields"][0]["label"],
        "Participant Count"
    );
    assert_eq!(version_one["sections"][0]["fields"][0]["required"], false);
    assert_eq!(version_two["status"], "published");
    assert_eq!(version_two["form_name"], "Monthly Services Report");
}

#[tokio::test]
async fn hierarchy_builder_rejects_required_metadata_after_nodes_exist() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let unauthorized = request_status_and_json(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/admin/node-types")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Organization",
                    "slug": "organization"
                })
                .to_string(),
            ))
            .expect("valid unauthorized request"),
    )
    .await;
    assert_eq!(unauthorized.0, StatusCode::UNAUTHORIZED);

    let node_type = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-types",
            &token,
            Some(json!({
                "name": "Organization",
                "slug": "organization"
            })),
        ),
    )
    .await;
    let node_type_id = id_from(&node_type);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-types/{node_type_id}"),
            &token,
            Some(json!({
                "name": "Community Organization",
                "slug": "community-organization"
            })),
        ),
    )
    .await;

    let metadata_field = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-metadata-fields",
            &token,
            Some(json!({
                "node_type_id": node_type_id,
                "key": "region",
                "label": "Region",
                "field_type": "text",
                "required": false
            })),
        ),
    )
    .await;
    let metadata_field_id = id_from(&metadata_field);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-metadata-fields/{metadata_field_id}"),
            &token,
            Some(json!({
                "key": "region_code",
                "label": "Region Code",
                "field_type": "text",
                "required": false
            })),
        ),
    )
    .await;

    let node = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/nodes",
            &token,
            Some(json!({
                "node_type_id": node_type_id,
                "parent_node_id": null,
                "name": "Pilot Organization",
                "metadata": {"region_code": "North"}
            })),
        ),
    )
    .await;
    let node_id = id_from(&node);

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/nodes/{node_id}"),
            &token,
            Some(json!({
                "parent_node_id": null,
                "name": "Updated Pilot Organization",
                "metadata": {"region_code": "South"}
            })),
        ),
    )
    .await;
    let nodes = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri("/api/nodes?q=Updated")
            .body(Body::empty())
            .expect("valid nodes request"),
    )
    .await;
    assert!(
        nodes
            .as_array()
            .expect("nodes response should be an array")
            .iter()
            .any(|node| node["id"] == node_id.to_string()
                && node["name"] == "Updated Pilot Organization"
                && node["metadata"]["region_code"] == "South")
    );

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-metadata-fields/{metadata_field_id}"),
            &token,
            Some(json!({
                "key": "region_code",
                "label": "Region",
                "field_type": "text",
                "required": false
            })),
        ),
    )
    .await;
    let required_metadata_update = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-metadata-fields/{metadata_field_id}"),
            &token,
            Some(json!({
                "key": "region_code",
                "label": "Region",
                "field_type": "text",
                "required": true
            })),
        ),
    )
    .await;
    assert_eq!(required_metadata_update.0, StatusCode::BAD_REQUEST);
    assert!(
        required_metadata_update.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("cannot be made required")
    );
    let metadata_key_update = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-metadata-fields/{metadata_field_id}"),
            &token,
            Some(json!({
                "key": "region",
                "label": "Region",
                "field_type": "text",
                "required": false
            })),
        ),
    )
    .await;
    assert_eq!(metadata_key_update.0, StatusCode::BAD_REQUEST);
    assert!(
        metadata_key_update.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("keys cannot be changed")
    );
    let metadata_type_update = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-metadata-fields/{metadata_field_id}"),
            &token,
            Some(json!({
                "key": "region_code",
                "label": "Region",
                "field_type": "number",
                "required": false
            })),
        ),
    )
    .await;
    assert_eq!(metadata_type_update.0, StatusCode::BAD_REQUEST);
    assert!(
        metadata_type_update.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("types cannot be changed")
    );

    let required_metadata = request_status_and_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/node-metadata-fields",
            &token,
            Some(json!({
                "node_type_id": node_type_id,
                "key": "district",
                "label": "District",
                "field_type": "text",
                "required": true
            })),
        ),
    )
    .await;
    assert_eq!(required_metadata.0, StatusCode::BAD_REQUEST);
    assert!(
        required_metadata.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("cannot be added after nodes")
    );
}

#[tokio::test]
async fn hierarchy_and_form_builders_return_diagnostics_for_invalid_references() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let blank_node_type = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-types",
            &token,
            Some(json!({
                "name": "   ",
                "slug": "blank-node-type"
            })),
        ),
    )
    .await;
    assert_eq!(blank_node_type.0, StatusCode::BAD_REQUEST);
    assert!(
        blank_node_type.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("node type name is required")
    );

    let parent_type = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-types",
            &token,
            Some(json!({
                "name": "Parent Organization",
                "slug": "parent-organization"
            })),
        ),
    )
    .await;
    let parent_type_id = id_from(&parent_type);
    let child_type = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-types",
            &token,
            Some(json!({
                "name": "Child Program",
                "slug": "child-program"
            })),
        ),
    )
    .await;
    let child_type_id = id_from(&child_type);
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-type-relationships",
            &token,
            Some(json!({
                "parent_node_type_id": parent_type_id,
                "child_node_type_id": child_type_id
            })),
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/node-type-relationships/{parent_type_id}/{child_type_id}"),
            &token,
            None,
        ),
    )
    .await;
    let removed_relationship = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-type-relationships", &token, None),
    )
    .await;
    assert!(
        removed_relationship
            .as_array()
            .expect("relationship list should be an array")
            .iter()
            .all(
                |relationship| relationship["parent_node_type_id"] != parent_type_id.to_string()
                    || relationship["child_node_type_id"] != child_type_id.to_string()
            )
    );
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-type-relationships",
            &token,
            Some(json!({
                "parent_node_type_id": parent_type_id,
                "child_node_type_id": child_type_id
            })),
        ),
    )
    .await;
    let parent_node = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/nodes",
            &token,
            Some(json!({
                "node_type_id": parent_type_id,
                "parent_node_id": null,
                "name": "Parent Node",
                "metadata": {}
            })),
        ),
    )
    .await;
    let parent_node_id = id_from(&parent_node);
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/nodes",
            &token,
            Some(json!({
                "node_type_id": child_type_id,
                "parent_node_id": parent_node_id,
                "name": "Child Node",
                "metadata": {}
            })),
        ),
    )
    .await;
    let used_relationship_delete = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/node-type-relationships/{parent_type_id}/{child_type_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(used_relationship_delete.0, StatusCode::BAD_REQUEST);
    assert!(
        used_relationship_delete.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("existing nodes use it")
    );
    let missing_relationship_delete = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!(
                "/api/admin/node-type-relationships/{}/{}",
                Uuid::new_v4(),
                Uuid::new_v4()
            ),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(missing_relationship_delete.0, StatusCode::NOT_FOUND);

    let missing_scoped_node_type = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &token,
            Some(json!({
                "name": "Invalid Scoped Form",
                "slug": "invalid-scoped-form",
                "scope_node_type_id": Uuid::new_v4()
            })),
        ),
    )
    .await;
    assert_eq!(missing_scoped_node_type.0, StatusCode::NOT_FOUND);

    let missing_form = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/forms/{}/versions", Uuid::new_v4()),
            &token,
            Some(json!({
                "version_label": "v1",
                "compatibility_group_name": "Default compatibility"
            })),
        ),
    )
    .await;
    assert_eq!(missing_form.0, StatusCode::NOT_FOUND);

    let missing_metadata_node_type = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-metadata-fields",
            &token,
            Some(json!({
                "node_type_id": Uuid::new_v4(),
                "key": "region",
                "label": "Region",
                "field_type": "text",
                "required": false
            })),
        ),
    )
    .await;
    assert_eq!(missing_metadata_node_type.0, StatusCode::NOT_FOUND);

    let missing_node_type = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/nodes",
            &token,
            Some(json!({
                "node_type_id": Uuid::new_v4(),
                "parent_node_id": null,
                "name": "Unknown Node",
                "metadata": {}
            })),
        ),
    )
    .await;
    assert_eq!(missing_node_type.0, StatusCode::NOT_FOUND);

    let organization_type_id = create_node_type(app.clone(), &token, "Organization", "org").await;
    let program_type_id = create_node_type(app.clone(), &token, "Program", "program").await;
    let activity_type_id = create_node_type(app.clone(), &token, "Activity", "activity").await;

    let duplicate_node_type_slug = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-types",
            &token,
            Some(json!({
                "name": "Duplicate Organization",
                "slug": "org"
            })),
        ),
    )
    .await;
    assert_eq!(duplicate_node_type_slug.0, StatusCode::BAD_REQUEST);
    assert!(
        duplicate_node_type_slug.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("already in use")
    );

    let missing_relationship_node_type = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-type-relationships",
            &token,
            Some(json!({
                "parent_node_type_id": organization_type_id,
                "child_node_type_id": Uuid::new_v4()
            })),
        ),
    )
    .await;
    assert_eq!(missing_relationship_node_type.0, StatusCode::NOT_FOUND);

    let self_relationship = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/node-type-relationships",
            &token,
            Some(json!({
                "parent_node_type_id": organization_type_id,
                "child_node_type_id": organization_type_id
            })),
        ),
    )
    .await;
    assert_eq!(self_relationship.0, StatusCode::BAD_REQUEST);
    assert!(
        self_relationship.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("same type")
    );

    create_node_type_relationship(app.clone(), &token, organization_type_id, program_type_id).await;
    create_node_type_relationship(app.clone(), &token, program_type_id, activity_type_id).await;

    let cyclic_relationship = request_status_and_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/node-type-relationships",
            &token,
            Some(json!({
                "parent_node_type_id": activity_type_id,
                "child_node_type_id": organization_type_id
            })),
        ),
    )
    .await;
    assert_eq!(cyclic_relationship.0, StatusCode::BAD_REQUEST);
    assert!(
        cyclic_relationship.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("cycle")
    );
}

#[tokio::test]
async fn admin_mutation_routes_require_authentication() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let form_id = Uuid::new_v4();
    let form_version_id = Uuid::new_v4();
    let dashboard_id = Uuid::new_v4();
    let node_type_id = Uuid::new_v4();
    let parent_node_type_id = Uuid::new_v4();
    let chart_id = Uuid::new_v4();

    let requests = vec![
        Request::builder()
            .method("POST")
            .uri("/api/admin/node-types")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({"name": "Organization", "slug": "organization"}).to_string(),
            ))
            .expect("valid node type request"),
        Request::builder()
            .method("PUT")
            .uri(format!("/api/admin/node-types/{node_type_id}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({"name": "Organization", "slug": "organization"}).to_string(),
            ))
            .expect("valid node type update request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/node-type-relationships")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "parent_node_type_id": parent_node_type_id,
                    "child_node_type_id": node_type_id
                })
                .to_string(),
            ))
            .expect("valid relationship request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/node-metadata-fields")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "node_type_id": node_type_id,
                    "key": "region",
                    "label": "Region",
                    "field_type": "text",
                    "required": false
                })
                .to_string(),
            ))
            .expect("valid metadata request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/nodes")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "node_type_id": node_type_id,
                    "parent_node_id": null,
                    "name": "Demo Organization",
                    "metadata": {}
                })
                .to_string(),
            ))
            .expect("valid node request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/forms")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Monthly Report",
                    "slug": "monthly-report",
                    "scope_node_type_id": null
                })
                .to_string(),
            ))
            .expect("valid form request"),
        Request::builder()
            .method("PUT")
            .uri(format!("/api/admin/forms/{form_id}"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Monthly Report",
                    "slug": "monthly-report",
                    "scope_node_type_id": null
                })
                .to_string(),
            ))
            .expect("valid form update request"),
        Request::builder()
            .method("POST")
            .uri(format!("/api/admin/forms/{form_id}/versions"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "version_label": "v1",
                    "compatibility_group_name": "Default compatibility"
                })
                .to_string(),
            ))
            .expect("valid form version request"),
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/form-versions/{form_version_id}/sections"
            ))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({"title": "Main", "position": 0}).to_string(),
            ))
            .expect("valid section request"),
        Request::builder()
            .method("POST")
            .uri(format!("/api/admin/form-versions/{form_version_id}/fields"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "section_id": Uuid::new_v4(),
                    "key": "participants",
                    "label": "Participants",
                    "field_type": "number",
                    "required": true,
                    "position": 0
                })
                .to_string(),
            ))
            .expect("valid field request"),
        Request::builder()
            .method("POST")
            .uri(format!(
                "/api/admin/form-versions/{form_version_id}/publish"
            ))
            .body(Body::empty())
            .expect("valid publish request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/analytics/refresh")
            .body(Body::empty())
            .expect("valid analytics request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/datasets")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Participant Dataset",
                    "slug": "participant-dataset",
                    "grain": "submission",
                    "sources": [{
                        "source_alias": "service",
                        "form_id": form_id,
                        "compatibility_group_id": null,
                        "selection_rule": "all"
                    }],
                    "fields": [{
                        "key": "participants",
                        "label": "Participants",
                        "source_alias": "service",
                        "source_field_key": "participants",
                        "position": 0
                    }]
                })
                .to_string(),
            ))
            .expect("valid dataset request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/reports")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Participants Report",
                    "form_id": null,
                    "fields": [{
                        "logical_key": "participants",
                        "source_field_key": "participants",
                        "missing_policy": "null"
                    }]
                })
                .to_string(),
            ))
            .expect("valid report request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/aggregations")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Participants Aggregation",
                    "report_id": Uuid::new_v4(),
                    "group_by_logical_key": null,
                    "metrics": [{
                        "metric_key": "responses",
                        "source_logical_key": null,
                        "metric_kind": "count"
                    }]
                })
                .to_string(),
            ))
            .expect("valid aggregation request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/charts")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "name": "Participants Chart",
                    "report_id": null,
                    "chart_type": "table"
                })
                .to_string(),
            ))
            .expect("valid chart request"),
        Request::builder()
            .method("POST")
            .uri("/api/admin/dashboards")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(json!({"name": "Dashboard"}).to_string()))
            .expect("valid dashboard request"),
        Request::builder()
            .method("POST")
            .uri(format!("/api/admin/dashboards/{dashboard_id}/components"))
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "chart_id": chart_id,
                    "position": 0,
                    "config": {}
                })
                .to_string(),
            ))
            .expect("valid dashboard component request"),
        Request::builder()
            .method("POST")
            .uri("/api/demo/seed")
            .body(Body::empty())
            .expect("valid demo seed request"),
    ];

    for request in requests {
        let uri = request.uri().to_string();
        let (status, _) = request_status_and_json(app.clone(), request).await;
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{uri} should require auth"
        );
    }
}

#[tokio::test]
async fn dataset_definitions_validate_sources_and_fields() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let form = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &token,
            Some(json!({
                "name": "Quarterly Service Dataset Form",
                "slug": "quarterly-service-dataset-form",
                "scope_node_type_id": null
            })),
        ),
    )
    .await;
    let form_id = id_from(&form);
    let form_version_id = create_form_version(app.clone(), &token, form_id, "v1").await;
    let section_id = create_form_section(app.clone(), &token, form_version_id, "Main").await;
    create_number_field(
        app.clone(),
        &token,
        form_version_id,
        section_id,
        "participants",
    )
    .await;

    let dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Participant Dataset",
                "slug": "participant-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "service",
                    "form_id": form_id,
                    "compatibility_group_id": null,
                    "selection_rule": "all"
                }],
                "fields": [{
                    "key": "participant_count",
                    "label": "Participant Count",
                    "source_alias": "service",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let dataset_id = id_from(&dataset);

    let datasets = request_json(
        app.clone(),
        authorized_request("GET", "/api/datasets", &token, None),
    )
    .await;
    assert!(
        datasets
            .as_array()
            .expect("dataset list should be an array")
            .iter()
            .any(|dataset| dataset["id"] == dataset_id.to_string()
                && dataset["slug"] == "participant-dataset"
                && dataset["grain"] == "submission"
                && dataset["source_count"] == 1
                && dataset["field_count"] == 1)
    );

    let definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/datasets/{dataset_id}"), &token, None),
    )
    .await;
    assert_eq!(definition["name"], "Participant Dataset");
    assert_eq!(definition["sources"][0]["source_alias"], "service");
    assert_eq!(definition["sources"][0]["form_id"], form_id.to_string());
    assert_eq!(
        definition["sources"][0]["form_name"],
        "Quarterly Service Dataset Form"
    );
    assert_eq!(definition["fields"][0]["key"], "participant_count");
    assert_eq!(definition["fields"][0]["source_field_key"], "participants");
    assert_eq!(definition["fields"][0]["field_type"], "number");

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/datasets/{dataset_id}"),
            &token,
            Some(json!({
                "name": "Updated Participant Dataset",
                "slug": "updated-participant-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "services",
                    "form_id": form_id,
                    "compatibility_group_id": null,
                    "selection_rule": "all"
                }],
                "fields": [{
                    "key": "participants_total",
                    "label": "Participants Total",
                    "source_alias": "services",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let updated_definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/datasets/{dataset_id}"), &token, None),
    )
    .await;
    assert_eq!(updated_definition["name"], "Updated Participant Dataset");
    assert_eq!(updated_definition["slug"], "updated-participant-dataset");
    assert_eq!(updated_definition["sources"][0]["source_alias"], "services");
    assert_eq!(updated_definition["fields"][0]["key"], "participants_total");

    let duplicate_field = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Duplicate Dataset",
                "slug": "duplicate-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "service",
                    "form_id": form_id,
                    "compatibility_group_id": null,
                    "selection_rule": "all"
                }],
                "fields": [
                    {
                        "key": "participant_count",
                        "label": "Participant Count",
                        "source_alias": "service",
                        "source_field_key": "participants",
                        "position": 0
                    },
                    {
                        "key": "participant_count",
                        "label": "Duplicate Participant Count",
                        "source_alias": "service",
                        "source_field_key": "participants",
                        "position": 1
                    }
                ]
            })),
        ),
    )
    .await;
    assert_eq!(duplicate_field.0, StatusCode::BAD_REQUEST);
    assert!(
        duplicate_field.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("duplicated")
    );

    let missing_source_field = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Missing Field Dataset",
                "slug": "missing-field-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "service",
                    "form_id": form_id,
                    "compatibility_group_id": null,
                    "selection_rule": "all"
                }],
                "fields": [{
                    "key": "missing_field",
                    "label": "Missing Field",
                    "source_alias": "service",
                    "source_field_key": "missing_field",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    assert_eq!(missing_source_field.0, StatusCode::BAD_REQUEST);
    assert!(
        missing_source_field.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("not available on source")
    );

    let disposable_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Disposable Dataset",
                "slug": "disposable-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "service",
                    "form_id": form_id,
                    "compatibility_group_id": null,
                    "selection_rule": "all"
                }],
                "fields": [{
                    "key": "participant_count",
                    "label": "Participant Count",
                    "source_alias": "service",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let disposable_dataset_id = id_from(&disposable_dataset);
    let deleted_dataset = request_json(
        app,
        authorized_request(
            "DELETE",
            &format!("/api/admin/datasets/{disposable_dataset_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(deleted_dataset["id"], disposable_dataset_id.to_string());
}

#[tokio::test]
async fn dataset_selection_rules_pick_latest_and_earliest_submissions() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;

    let follow_up_draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/submissions/drafts",
            &token,
            Some(json!({
                "form_version_id": seed["form_version_id"],
                "node_id": seed["organization_node_id"]
            })),
        ),
    )
    .await;
    let follow_up_submission_id = id_from(&follow_up_draft);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/submissions/{follow_up_submission_id}/values"),
            &token,
            Some(json!({"values": {"participants": 99}})),
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{follow_up_submission_id}/submit"),
            &token,
            None,
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request("POST", "/api/admin/analytics/refresh", &token, None),
    )
    .await;

    let latest_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Latest Participants Dataset",
                "slug": "latest-participants-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "service",
                    "form_id": seed["form_id"],
                    "compatibility_group_id": null,
                    "selection_rule": "latest"
                }],
                "fields": [{
                    "key": "participant_count",
                    "label": "Participant Count",
                    "source_alias": "service",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let latest_dataset_id = id_from(&latest_dataset);
    let latest_rows = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{latest_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(latest_rows["rows"][0]["values"]["participant_count"], "99");

    let earliest_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Earliest Participants Dataset",
                "slug": "earliest-participants-dataset",
                "grain": "submission",
                "sources": [{
                    "source_alias": "service",
                    "form_id": seed["form_id"],
                    "compatibility_group_id": null,
                    "selection_rule": "earliest"
                }],
                "fields": [{
                    "key": "participant_count",
                    "label": "Participant Count",
                    "source_alias": "service",
                    "source_field_key": "participants",
                    "position": 0
                }]
            })),
        ),
    )
    .await;
    let earliest_dataset_id = id_from(&earliest_dataset);
    let earliest_rows = request_json(
        app,
        authorized_request(
            "GET",
            &format!("/api/datasets/{earliest_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        earliest_rows["rows"][0]["values"]["participant_count"],
        "42"
    );
}

#[tokio::test]
async fn reporting_and_dashboard_builders_return_diagnostics_for_invalid_references() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let form = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &token,
            Some(json!({
                "name": "Monthly Service Report",
                "slug": "monthly-service-report",
                "scope_node_type_id": null
            })),
        ),
    )
    .await;
    let form_id = id_from(&form);
    let form_version_id = create_form_version(app.clone(), &token, form_id, "v1").await;
    let section_id = create_form_section(app.clone(), &token, form_version_id, "Main").await;
    create_number_field(
        app.clone(),
        &token,
        form_version_id,
        section_id,
        "participants",
    )
    .await;

    let missing_field = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Invalid Report",
                "form_id": form_id,
                "fields": [{
                    "logical_key": "missing",
                    "source_field_key": "missing",
                    "missing_policy": "null"
                }]
            })),
        ),
    )
    .await;
    assert_eq!(missing_field.0, StatusCode::BAD_REQUEST);
    assert!(
        missing_field.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("not available on form")
    );

    let duplicate_logical_key = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Duplicate Logical Field Report",
                "form_id": form_id,
                "fields": [
                    {
                        "logical_key": "participants",
                        "source_field_key": "participants",
                        "missing_policy": "null"
                    },
                    {
                        "logical_key": "participants",
                        "source_field_key": "participants",
                        "missing_policy": "bucket_unknown"
                    }
                ]
            })),
        ),
    )
    .await;
    assert_eq!(duplicate_logical_key.0, StatusCode::BAD_REQUEST);
    assert!(
        duplicate_logical_key.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("duplicated")
    );

    let missing_report = request_status_and_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{}/table", Uuid::new_v4()),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(missing_report.0, StatusCode::NOT_FOUND);

    let missing_chart_report = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/charts",
            &token,
            Some(json!({
                "name": "Missing Report Chart",
                "report_id": Uuid::new_v4(),
                "chart_type": "table"
            })),
        ),
    )
    .await;
    assert_eq!(missing_chart_report.0, StatusCode::NOT_FOUND);

    let dashboard = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/dashboards",
            &token,
            Some(json!({"name": "Diagnostics Dashboard"})),
        ),
    )
    .await;
    let dashboard_id = id_from(&dashboard);
    let missing_component_chart = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/dashboards/{dashboard_id}/components"),
            &token,
            Some(json!({
                "chart_id": Uuid::new_v4(),
                "position": 0,
                "config": {}
            })),
        ),
    )
    .await;
    assert_eq!(missing_component_chart.0, StatusCode::NOT_FOUND);

    let valid_report = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Participants Report",
                "form_id": form_id,
                "fields": [{
                    "logical_key": "participants",
                    "source_field_key": "participants",
                    "missing_policy": "null"
                }]
            })),
        ),
    )
    .await;
    let valid_report_id = id_from(&valid_report);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/reports/{valid_report_id}"),
            &token,
            Some(json!({
                "name": "Updated Participants Report",
                "form_id": form_id,
                "fields": [{
                    "logical_key": "participant_count",
                    "source_field_key": "participants",
                    "missing_policy": "bucket_unknown"
                }]
            })),
        ),
    )
    .await;
    let updated_report = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{valid_report_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(updated_report["name"], "Updated Participants Report");
    assert_eq!(
        updated_report["bindings"][0]["logical_key"],
        "participant_count"
    );
    let missing_update_report = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/reports/{}", Uuid::new_v4()),
            &token,
            Some(json!({
                "name": "Missing Report",
                "form_id": form_id,
                "fields": [{
                    "logical_key": "participants",
                    "source_field_key": "participants",
                    "missing_policy": "null"
                }]
            })),
        ),
    )
    .await;
    assert_eq!(missing_update_report.0, StatusCode::NOT_FOUND);
    let valid_chart = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/charts",
            &token,
            Some(json!({
                "name": "Participants Chart",
                "report_id": valid_report_id,
                "chart_type": "table"
            })),
        ),
    )
    .await;
    let valid_chart_id = id_from(&valid_chart);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/charts/{valid_chart_id}"),
            &token,
            Some(json!({
                "name": "Updated Participants Chart",
                "report_id": valid_report_id,
                "chart_type": "table"
            })),
        ),
    )
    .await;
    let updated_charts = request_json(
        app.clone(),
        authorized_request("GET", "/api/charts", &token, None),
    )
    .await;
    assert!(
        updated_charts
            .as_array()
            .expect("chart list should be an array")
            .iter()
            .any(|chart| chart["id"] == valid_chart_id.to_string()
                && chart["name"] == "Updated Participants Chart")
    );
    let missing_update_chart = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/charts/{}", Uuid::new_v4()),
            &token,
            Some(json!({
                "name": "Missing Chart",
                "report_id": valid_report_id,
                "chart_type": "table"
            })),
        ),
    )
    .await;
    assert_eq!(missing_update_chart.0, StatusCode::NOT_FOUND);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/dashboards/{dashboard_id}"),
            &token,
            Some(json!({"name": "Updated Diagnostics Dashboard"})),
        ),
    )
    .await;
    let updated_dashboard = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{dashboard_id}"))
            .body(Body::empty())
            .expect("valid dashboard request"),
    )
    .await;
    assert_eq!(updated_dashboard["name"], "Updated Diagnostics Dashboard");
    let missing_update_dashboard = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/dashboards/{}", Uuid::new_v4()),
            &token,
            Some(json!({"name": "Missing Dashboard"})),
        ),
    )
    .await;
    assert_eq!(missing_update_dashboard.0, StatusCode::NOT_FOUND);
    let component = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/dashboards/{dashboard_id}/components"),
            &token,
            Some(json!({
                "chart_id": valid_chart_id,
                "position": 2,
                "config": {"title": "Initial chart"}
            })),
        ),
    )
    .await;
    let component_id = id_from(&component);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/dashboard-components/{component_id}"),
            &token,
            Some(json!({
                "chart_id": valid_chart_id,
                "position": 1,
                "config": {"title": "Updated chart"}
            })),
        ),
    )
    .await;
    let dashboard_with_component = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{dashboard_id}"))
            .body(Body::empty())
            .expect("valid dashboard request"),
    )
    .await;
    assert!(
        dashboard_with_component["components"]
            .as_array()
            .expect("dashboard should include components")
            .iter()
            .any(|component| component["id"] == component_id.to_string()
                && component["position"] == 1
                && component["config"]["title"] == "Updated chart")
    );
    let missing_update_component = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/dashboard-components/{}", Uuid::new_v4()),
            &token,
            Some(json!({
                "chart_id": valid_chart_id,
                "position": 0,
                "config": {}
            })),
        ),
    )
    .await;
    assert_eq!(missing_update_component.0, StatusCode::NOT_FOUND);
    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/dashboard-components/{component_id}"),
            &token,
            None,
        ),
    )
    .await;
    let dashboard_after_delete = request_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{dashboard_id}"))
            .body(Body::empty())
            .expect("valid dashboard request"),
    )
    .await;
    assert!(
        dashboard_after_delete["components"]
            .as_array()
            .expect("dashboard should include components")
            .iter()
            .all(|component| component["id"] != component_id.to_string())
    );
    let missing_component_dashboard = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/dashboards/{}/components", Uuid::new_v4()),
            &token,
            Some(json!({
                "chart_id": valid_chart_id,
                "position": 0,
                "config": {}
            })),
        ),
    )
    .await;
    assert_eq!(missing_component_dashboard.0, StatusCode::NOT_FOUND);

    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/charts/{valid_chart_id}"),
            &token,
            None,
        ),
    )
    .await;
    let charts_after_delete = request_json(
        app.clone(),
        authorized_request("GET", "/api/charts", &token, None),
    )
    .await;
    assert!(
        charts_after_delete
            .as_array()
            .expect("chart list should be an array")
            .iter()
            .all(|chart| chart["id"] != valid_chart_id.to_string())
    );
    let missing_delete_chart = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/charts/{}", Uuid::new_v4()),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(missing_delete_chart.0, StatusCode::NOT_FOUND);

    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/dashboards/{dashboard_id}"),
            &token,
            None,
        ),
    )
    .await;
    let deleted_dashboard = request_status_and_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{dashboard_id}"))
            .body(Body::empty())
            .expect("valid dashboard request"),
    )
    .await;
    assert_eq!(deleted_dashboard.0, StatusCode::NOT_FOUND);
    let missing_delete_dashboard = request_status_and_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/dashboards/{}", Uuid::new_v4()),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(missing_delete_dashboard.0, StatusCode::NOT_FOUND);

    request_json(
        app.clone(),
        authorized_request(
            "DELETE",
            &format!("/api/admin/reports/{valid_report_id}"),
            &token,
            None,
        ),
    )
    .await;
    let deleted_report = request_status_and_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{valid_report_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(deleted_report.0, StatusCode::NOT_FOUND);
    let missing_delete_report = request_status_and_json(
        app,
        authorized_request(
            "DELETE",
            &format!("/api/admin/reports/{}", Uuid::new_v4()),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(missing_delete_report.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn legacy_fixture_import_rehearsal_projects_report_and_dashboard() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let summary = legacy_import::import_legacy_fixture_str(
        &state.pool,
        include_str!("../../../fixtures/legacy-rehearsal.json"),
    )
    .await
    .expect("legacy fixture should import");
    assert_eq!(summary.analytics_values, 5);

    let app = router(state);
    let token = login_token(app.clone()).await;
    let report = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{}/table", summary.report_id),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(report["rows"][0]["field_value"], "42");

    let dashboard = request_json(
        app,
        Request::builder()
            .method("GET")
            .uri(format!("/api/dashboards/{}", summary.dashboard_id))
            .body(Body::empty())
            .expect("valid imported dashboard request"),
    )
    .await;
    assert_eq!(
        dashboard["components"][0]["chart"]["report_id"],
        summary.report_id.to_string()
    );
}

#[tokio::test]
async fn legacy_fixture_import_rehearsal_is_repeatable() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let fixture = include_str!("../../../fixtures/legacy-rehearsal.json");

    let first = legacy_import::import_legacy_fixture_str(&state.pool, fixture)
        .await
        .expect("first legacy fixture import should succeed");
    let second = legacy_import::import_legacy_fixture_str(&state.pool, fixture)
        .await
        .expect("second legacy fixture import should succeed");

    assert_eq!(first.partner_node_id, second.partner_node_id);
    assert_eq!(first.program_node_id, second.program_node_id);
    assert_eq!(first.activity_node_id, second.activity_node_id);
    assert_eq!(first.session_node_id, second.session_node_id);
    assert_eq!(first.form_id, second.form_id);
    assert_eq!(first.form_version_id, second.form_version_id);
    assert_eq!(first.submission_id, second.submission_id);
    assert_eq!(first.report_id, second.report_id);
    assert_eq!(first.dashboard_id, second.dashboard_id);

    let audit_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM submission_audit_events WHERE event_type LIKE 'legacy_import:legacy-rehearsal:%'",
    )
    .fetch_one(&state.pool)
    .await
    .expect("audit count should query");
    assert_eq!(audit_count, 1);

    let dashboard_component_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dashboard_components WHERE dashboard_id = $1")
            .bind(first.dashboard_id)
            .fetch_one(&state.pool)
            .await
            .expect("dashboard component count should query");
    assert_eq!(dashboard_component_count, 1);
}

#[tokio::test]
async fn legacy_inactive_locked_fixture_preserves_status_metadata() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(state) = test_state().await else {
        return;
    };
    let fixture = include_str!("../../../fixtures/legacy-inactive-locked.json");

    let report = legacy_import::validate_legacy_fixture_str(fixture)
        .expect("inactive fixture should deserialize");
    assert!(report.is_clean());

    let summary = legacy_import::import_legacy_fixture_str(&state.pool, fixture)
        .await
        .expect("inactive fixture should import");
    assert_eq!(summary.fixture_name, "legacy-inactive-locked");

    assert_eq!(
        node_metadata_value(&state.pool, summary.partner_node_id, "is_active").await,
        json!(false)
    );
    assert_eq!(
        node_metadata_value(&state.pool, summary.partner_node_id, "locked").await,
        json!(true)
    );
    assert_eq!(
        node_metadata_value(&state.pool, summary.session_node_id, "session_date").await,
        json!("2026-03-15")
    );

    let inactive_choice_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM choice_list_items
        WHERE value = 'legacy-inactive:cancelled'
          AND label = 'inactive: Cancelled'
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .expect("inactive choice marker should query");
    assert_eq!(inactive_choice_count, 1);
}

async fn test_app() -> Option<axum::Router> {
    Some(router(test_state().await?))
}

async fn test_state() -> Option<db::AppState> {
    let Some(database_url) = std::env::var("TEST_DATABASE_URL").ok() else {
        eprintln!("skipping database integration test; TEST_DATABASE_URL is not set");
        return None;
    };

    reset_database(&database_url).await;
    let config = Config {
        database_url,
        bind_addr: "127.0.0.1:0".to_string(),
        dev_admin_email: "admin@tessara.local".to_string(),
        dev_admin_password: "tessara-dev-admin".to_string(),
    };
    let pool = db::connect_and_prepare(&config)
        .await
        .expect("database should migrate and seed");
    let state = db::AppState { pool, config };

    Some(state)
}

async fn login_token(app: axum::Router) -> String {
    let login = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "email": "admin@tessara.local",
                    "password": "tessara-dev-admin"
                })
                .to_string(),
            ))
            .expect("valid login request"),
    )
    .await;

    login["token"]
        .as_str()
        .expect("login response should contain token")
        .to_string()
}

async fn create_form_version(
    app: axum::Router,
    token: &str,
    form_id: Uuid,
    version_label: &str,
) -> Uuid {
    let version = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/admin/forms/{form_id}/versions"),
            token,
            Some(json!({
                "version_label": version_label,
                "compatibility_group_name": "Default compatibility"
            })),
        ),
    )
    .await;

    id_from(&version)
}

async fn create_node_type(app: axum::Router, token: &str, name: &str, slug: &str) -> Uuid {
    let node_type = request_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/node-types",
            token,
            Some(json!({
                "name": name,
                "slug": slug
            })),
        ),
    )
    .await;

    id_from(&node_type)
}

async fn create_node_type_relationship(
    app: axum::Router,
    token: &str,
    parent_node_type_id: Uuid,
    child_node_type_id: Uuid,
) {
    request_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/node-type-relationships",
            token,
            Some(json!({
                "parent_node_type_id": parent_node_type_id,
                "child_node_type_id": child_node_type_id
            })),
        ),
    )
    .await;
}

async fn create_form_section(
    app: axum::Router,
    token: &str,
    form_version_id: Uuid,
    title: &str,
) -> Uuid {
    let section = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{form_version_id}/sections"),
            token,
            Some(json!({
                "title": title,
                "position": 0
            })),
        ),
    )
    .await;

    id_from(&section)
}

async fn create_number_field(
    app: axum::Router,
    token: &str,
    form_version_id: Uuid,
    section_id: Uuid,
    key: &str,
) -> Uuid {
    let field = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/admin/form-versions/{form_version_id}/fields"),
            token,
            Some(json!({
                "section_id": section_id,
                "key": key,
                "label": "Participants",
                "field_type": "number",
                "required": true,
                "position": 0
            })),
        ),
    )
    .await;

    id_from(&field)
}

async fn request_json(app: axum::Router, request: Request<Body>) -> Value {
    let (status, body) = request_status_and_json(app, request).await;
    assert_eq!(status, StatusCode::OK, "unexpected response: {body}");

    body
}

async fn node_metadata_value(pool: &PgPool, node_id: Uuid, key: &str) -> Value {
    sqlx::query_scalar(
        r#"
        SELECT node_metadata_values.value
        FROM node_metadata_values
        JOIN node_metadata_field_definitions
            ON node_metadata_field_definitions.id = node_metadata_values.field_definition_id
        WHERE node_metadata_values.node_id = $1
          AND node_metadata_field_definitions.key = $2
        "#,
    )
    .bind(node_id)
    .bind(key)
    .fetch_one(pool)
    .await
    .expect("metadata value should query")
}

async fn request_status_and_json(app: axum::Router, request: Request<Body>) -> (StatusCode, Value) {
    let response = app
        .oneshot(request)
        .await
        .expect("router should produce response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    (
        status,
        serde_json::from_slice(&body).unwrap_or_else(|_| {
            panic!(
                "response should be JSON: {}",
                String::from_utf8_lossy(&body)
            )
        }),
    )
}

async fn request_status_and_text(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, String) {
    let response = app
        .oneshot(request)
        .await
        .expect("router should produce response");
    let status = response.status();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");

    (
        status,
        String::from_utf8(body.to_vec()).expect("response should be UTF-8"),
    )
}

fn authorized_request(method: &str, uri: &str, token: &str, body: Option<Value>) -> Request<Body> {
    let mut builder = Request::builder()
        .method(method)
        .uri(uri)
        .header(header::AUTHORIZATION, format!("Bearer {token}"));

    let body = if let Some(body) = body {
        builder = builder.header(header::CONTENT_TYPE, "application/json");
        Body::from(body.to_string())
    } else {
        Body::empty()
    };

    builder.body(body).expect("valid authorized request")
}

fn id_from(value: &Value) -> Uuid {
    value["id"]
        .as_str()
        .expect("response should contain id")
        .parse()
        .expect("response id should be a UUID")
}

async fn reset_database(database_url: &str) {
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(database_url)
        .await
        .expect("test database should be reachable");
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&pool)
        .await
        .expect("current database should be readable");
    assert!(
        database_name.contains("test"),
        "TEST_DATABASE_URL must point at a disposable database; got '{database_name}'"
    );

    drop_all_public_tables(&pool).await;
    sqlx::query("DROP SCHEMA IF EXISTS analytics CASCADE")
        .execute(&pool)
        .await
        .expect("analytics schema should be droppable");
    sqlx::query("DROP TABLE IF EXISTS _sqlx_migrations")
        .execute(&pool)
        .await
        .expect("migration table should be droppable");
    for type_name in [
        "field_type",
        "form_version_status",
        "submission_status",
        "missing_data_policy",
    ] {
        sqlx::query(&format!("DROP TYPE IF EXISTS {type_name} CASCADE"))
            .execute(&pool)
            .await
            .expect("enum type should be droppable");
    }
}

async fn drop_all_public_tables(pool: &PgPool) {
    let tables = sqlx::query_scalar::<_, String>(
        r#"
        SELECT tablename
        FROM pg_tables
        WHERE schemaname = 'public'
        "#,
    )
    .fetch_all(pool)
    .await
    .expect("public tables should be listable");

    for table in tables {
        sqlx::query(&format!("DROP TABLE IF EXISTS public.{table} CASCADE"))
            .execute(pool)
            .await
            .expect("public table should be droppable");
    }
}
