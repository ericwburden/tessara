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
    assert_eq!(seed["seed_version"], "uat-demo-v1");
    assert_eq!(seed["node_counts"]["partners"], 2);
    assert_eq!(seed["node_counts"]["programs"], 4);
    assert_eq!(seed["node_counts"]["activities"], 6);
    assert_eq!(seed["node_counts"]["sessions"], 8);
    assert_eq!(seed["form_count"], 4);
    assert_eq!(seed["draft_submission_count"], 4);
    assert_eq!(seed["submitted_submission_count"], 8);
    assert_eq!(seed["report_count"], 4);
    assert_eq!(seed["dashboard_count"], 1);
    assert!(
        seed["analytics_values"]
            .as_i64()
            .expect("seed should report analytics value count")
            >= 8
    );
    let app_summary = request_json(
        app.clone(),
        authorized_request("GET", "/api/app/summary", &token, None),
    )
    .await;
    assert_eq!(app_summary["published_form_versions"], 4);
    assert_eq!(app_summary["draft_submissions"], 4);
    assert_eq!(app_summary["submitted_submissions"], 8);
    assert_eq!(app_summary["datasets"], 0);
    assert_eq!(app_summary["reports"], 4);
    assert_eq!(app_summary["aggregations"], 0);
    assert_eq!(app_summary["dashboards"], 1);
    assert_eq!(app_summary["charts"], 4);
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
            .any(|node_type| {
                node_type["slug"] == "partner"
                    && node_type["node_count"].as_i64().unwrap_or_default() >= 2
            })
    );
    let partner_node_type_id = node_types
        .as_array()
        .expect("node type response should be an array")
        .iter()
        .find(|node_type| node_type["slug"] == "partner")
        .and_then(|node_type| node_type["id"].as_str())
        .expect("partner node type should be present");
    let node_type_definition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/node-types/{partner_node_type_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(node_type_definition["name"], "Partner");
    assert!(
        node_type_definition["metadata_fields"]
            .as_array()
            .expect("node type definition should include metadata fields")
            .iter()
            .any(|field| field["key"] == "region")
    );
    assert!(
        node_type_definition["child_relationships"]
            .as_array()
            .expect("node type definition should include child relationships")
            .iter()
            .any(|child| child["node_type_name"] == "Program")
    );
    assert!(
        node_type_definition["scoped_forms"]
            .as_array()
            .expect("node type definition should include scoped forms")
            .iter()
            .any(|form| form["name"] == "Demo Partner Profile"
                || form["form_name"] == "Demo Partner Profile")
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
            .any(|relationship| relationship["parent_name"] == "Partner"
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
                && field["node_type_name"] == "Partner"
                && field["required"] == true)
    );
    let nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=Demo", &token, None),
    )
    .await;
    assert!(
        nodes
            .as_array()
            .expect("nodes response should be an array")
            .iter()
            .any(|node| node["name"] == "Demo Partner North Star Services"
                && node["node_type_name"] == "Partner"
                && node["metadata"]["region"] == "north")
    );
    let node_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/nodes/{}",
                seed["organization_node_id"]
                    .as_str()
                    .expect("seed should include organization node id")
            ),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(node_detail["name"], "Demo Session April Orientation");
    assert!(
        node_detail["related_forms"]
            .as_array()
            .expect("node detail should include related forms")
            .iter()
            .any(|form| form["form_name"] == "Demo Session Log")
    );
    assert!(
        node_detail["related_responses"]
            .as_array()
            .expect("node detail should include related responses")
            .iter()
            .any(|submission| submission["submission_id"] == seed["submission_id"])
    );
    assert!(
        node_detail["related_dashboards"]
            .as_array()
            .expect("node detail should include related dashboards")
            .iter()
            .any(|dashboard| dashboard["dashboard_id"] == seed["dashboard_id"])
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
    let form_definition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/admin/forms/{}",
                seed["form_id"]
                    .as_str()
                    .expect("seed should include form id")
            ),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(form_definition["name"], "Demo Session Log");
    assert!(
        form_definition["versions"]
            .as_array()
            .expect("form definition should include versions")
            .iter()
            .any(|version| version["id"] == seed["form_version_id"])
    );
    assert!(
        form_definition["reports"]
            .as_array()
            .expect("form definition should include linked reports")
            .iter()
            .any(|report| report["name"] == "Participants Report")
    );
    let published_forms = request_json(
        app.clone(),
        authorized_request("GET", "/api/forms/published", &token, None),
    )
    .await;
    assert!(
        published_forms
            .as_array()
            .expect("published forms response should be an array")
            .iter()
            .any(|form_version| form_version["form_id"] == seed["form_id"]
                && form_version["form_name"] == "Demo Session Log"
                && form_version["form_version_id"] == seed["form_version_id"]
                && form_version["version_label"] == "1.0.0")
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
                && submission["value_count"] == 5)
    );
    let filtered_submissions = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/submissions?status=submitted&form_id={}&node_id={}&q=Session",
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
            .any(|event| event["event_type"]
                .as_str()
                .unwrap_or_default()
                .starts_with("seed_demo:"))
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
    assert!(
        report["rows"]
            .as_array()
            .expect("report should return row array")
            .iter()
            .any(|row| row["logical_key"] == "participants" && row["field_value"] == "42")
    );
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
            .any(|report| report["id"] == report_id && report["form_name"] == "Demo Session Log")
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
    assert_eq!(report_definition["form_name"], "Demo Session Log");

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
    assert!(
        dataset_table["rows"]
            .as_array()
            .expect("dataset table should return row array")
            .iter()
            .any(|row| row["submission_id"] == seed["submission_id"]
                && row["values"]["participant_count"] == "42")
    );
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
                    "form_id": seed["form_id"],
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
    assert!(
        compatibility_dataset_table["rows"]
            .as_array()
            .expect("compatibility dataset table should return row array")
            .iter()
            .any(|row| row["submission_id"] == seed["submission_id"]
                && row["values"]["participant_count"] == "42")
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
        dataset_report_definition["aggregations"]
            .as_array()
            .expect("dataset report definition should include linked aggregations")
            .is_empty()
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
    let dataset_definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/datasets/{dataset_id}"), &token, None),
    )
    .await;
    assert!(
        dataset_definition["reports"]
            .as_array()
            .expect("dataset definition should include linked reports")
            .iter()
            .any(|report| report["id"] == dataset_report_id
                && report["name"] == "Dataset Participants Report")
    );
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
        2.0
    );
    assert_eq!(
        dataset_aggregation_table["rows"][0]["metrics"]["participants_total"],
        60.0
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
    assert_eq!(aggregation_table["rows"][0]["metrics"]["responses"], 2.0);
    assert_eq!(
        aggregation_table["rows"][0]["metrics"]["participants_total"],
        60.0
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
                && chart["report_form_name"] == "Demo Session Log"
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
    let report_definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/reports/{report_id}"), &token, None),
    )
    .await;
    assert!(
        report_definition["aggregations"]
            .as_array()
            .expect("report definition should include linked aggregations")
            .iter()
            .any(|aggregation| aggregation["id"] == aggregation_id)
    );
    assert!(
        report_definition["charts"]
            .as_array()
            .expect("report definition should include linked charts")
            .iter()
            .any(|chart| chart["id"] == aggregation_chart_id)
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
        authorized_request(
            "GET",
            &format!("/api/dashboards/{dashboard_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert!(
        dashboard["components"]
            .as_array()
            .expect("dashboard response should include components")
            .iter()
            .any(|component| component["chart"]["report_id"] == report_id
                && component["chart"]["report_name"] == "Participants Report"
                && component["chart"]["report_form_name"] == "Demo Session Log")
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
        authorized_request("GET", "/api/dashboards", &token, None),
    )
    .await;
    assert!(
        dashboards
            .as_array()
            .expect("dashboards response should be an array")
            .iter()
            .any(|dashboard| dashboard["id"] == dashboard_id)
    );
    let chart_definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/charts/{chart_id}"), &token, None),
    )
    .await;
    assert_eq!(chart_definition["chart"]["id"], chart_id);
    assert!(
        chart_definition["chart"]["report_id"].is_string()
            || chart_definition["chart"]["aggregation_id"].is_string()
    );
    assert!(
        chart_definition["dashboards"]
            .as_array()
            .expect("chart definition should include dashboards")
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
async fn demo_seed_creates_full_uat_dataset_and_is_repeatable() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let first_seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;
    let second_seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;

    assert_eq!(first_seed["seed_version"], "uat-demo-v1");
    assert_eq!(first_seed["seed_version"], second_seed["seed_version"]);
    assert_eq!(first_seed["dashboard_id"], second_seed["dashboard_id"]);
    assert_eq!(first_seed["report_id"], second_seed["report_id"]);
    assert_eq!(first_seed["submission_id"], second_seed["submission_id"]);
    assert_eq!(first_seed["node_counts"]["partners"], 2);
    assert_eq!(first_seed["node_counts"]["programs"], 4);
    assert_eq!(first_seed["node_counts"]["activities"], 6);
    assert_eq!(first_seed["node_counts"]["sessions"], 8);
    assert_eq!(first_seed["form_count"], 4);
    assert_eq!(first_seed["draft_submission_count"], 4);
    assert_eq!(first_seed["submitted_submission_count"], 8);
    assert_eq!(first_seed["report_count"], 4);
    assert_eq!(first_seed["dashboard_count"], 1);

    let node_types = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-types", &token, None),
    )
    .await;
    for (slug, count) in [
        ("partner", 2),
        ("program", 4),
        ("activity", 6),
        ("session", 8),
    ] {
        assert!(
            node_types
                .as_array()
                .expect("node types should be an array")
                .iter()
                .any(|node_type| node_type["slug"] == slug && node_type["node_count"] == count)
        );
    }

    let metadata_fields = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-metadata-fields", &token, None),
    )
    .await;
    for (node_type_name, key, field_type) in [
        ("Partner", "legacy_id", "text"),
        ("Partner", "region", "single_choice"),
        ("Partner", "active_contract", "boolean"),
        ("Partner", "partner_since", "date"),
        ("Partner", "focus_areas", "multi_choice"),
        ("Program", "annual_target", "number"),
        ("Activity", "delivery_mode", "single_choice"),
        ("Session", "topics", "multi_choice"),
    ] {
        assert!(
            metadata_fields
                .as_array()
                .expect("metadata fields should be an array")
                .iter()
                .any(|field| field["node_type_name"] == node_type_name
                    && field["key"] == key
                    && field["field_type"] == field_type)
        );
    }

    let forms = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/forms", &token, None),
    )
    .await;
    for name in [
        "Demo Partner Profile",
        "Demo Program Snapshot",
        "Demo Activity Plan",
        "Demo Session Log",
    ] {
        assert!(
            forms
                .as_array()
                .expect("forms should be an array")
                .iter()
                .any(|form| form["name"] == name && form["versions"][0]["status"] == "published")
        );
    }

    let all_submissions = request_json(
        app.clone(),
        authorized_request("GET", "/api/submissions", &token, None),
    )
    .await;
    let demo_drafts = all_submissions
        .as_array()
        .expect("submissions should be an array")
        .iter()
        .filter(|submission| {
            submission["status"] == "draft"
                && submission["form_name"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("Demo ")
        })
        .count();
    let demo_submitted = all_submissions
        .as_array()
        .expect("submissions should be an array")
        .iter()
        .filter(|submission| {
            submission["status"] == "submitted"
                && submission["form_name"]
                    .as_str()
                    .unwrap_or_default()
                    .starts_with("Demo ")
        })
        .count();
    assert_eq!(demo_drafts, 4);
    assert_eq!(demo_submitted, 8);

    let reports = request_json(
        app.clone(),
        authorized_request("GET", "/api/reports", &token, None),
    )
    .await;
    for (name, form_name) in [
        ("Demo Partner Profile Report", "Demo Partner Profile"),
        ("Demo Program Snapshot Report", "Demo Program Snapshot"),
        ("Demo Activity Plan Report", "Demo Activity Plan"),
        ("Participants Report", "Demo Session Log"),
    ] {
        assert!(
            reports
                .as_array()
                .expect("reports should be an array")
                .iter()
                .any(|report| report["name"] == name && report["form_name"] == form_name)
        );
    }

    let dashboards = request_json(
        app,
        authorized_request("GET", "/api/dashboards", &token, None),
    )
    .await;
    assert!(
        dashboards
            .as_array()
            .expect("dashboards should be an array")
            .iter()
            .any(|dashboard| dashboard["name"] == "Demo Operations Dashboard")
    );
}

#[tokio::test]
async fn role_based_access_respects_scope_and_respondent_context() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let respondent_token = login_token_for(
        app.clone(),
        "respondent@tessara.local",
        "tessara-dev-respondent",
    )
    .await;
    let delegator_token = login_token_for(
        app.clone(),
        "delegator@tessara.local",
        "tessara-dev-delegator",
    )
    .await;

    let operator_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &operator_token, None),
    )
    .await;
    assert_eq!(operator_me["ui_access_profile"], "operator");
    assert_eq!(
        operator_me["scope_nodes"]
            .as_array()
            .expect("operator scope should be present")
            .len(),
        2
    );

    let respondent_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &respondent_token, None),
    )
    .await;
    assert_eq!(respondent_me["ui_access_profile"], "response_user");
    assert_eq!(
        respondent_me["delegations"]
            .as_array()
            .expect("respondent delegation list should be present")
            .len(),
        0
    );

    let delegator_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &delegator_token, None),
    )
    .await;
    assert_eq!(delegator_me["ui_access_profile"], "response_user");
    assert_eq!(
        delegator_me["delegations"]
            .as_array()
            .expect("delegator delegation list should be present")
            .len(),
        1
    );

    let operator_nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=Demo", &operator_token, None),
    )
    .await;
    let operator_node_names = operator_nodes
        .as_array()
        .expect("operator nodes should be an array")
        .iter()
        .filter_map(|node| node["name"].as_str())
        .collect::<Vec<_>>();
    assert!(operator_node_names.contains(&"Demo Program Family Outreach"));
    assert!(operator_node_names.contains(&"Demo Activity Job Coaching"));
    assert!(!operator_node_names.contains(&"Demo Partner Community Bridge"));

    let all_nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=Community Bridge", &admin_token, None),
    )
    .await;
    let out_of_scope_node_id = all_nodes
        .as_array()
        .expect("admin nodes should be an array")
        .iter()
        .find(|node| node["name"] == "Demo Partner Community Bridge")
        .and_then(|node| node["id"].as_str())
        .expect("out of scope node id should be present");
    let forbidden_node = request_status_and_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/nodes/{out_of_scope_node_id}"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(forbidden_node.0, StatusCode::FORBIDDEN);

    let respondent_forms = request_status_and_json(
        app.clone(),
        authorized_request("GET", "/api/forms", &respondent_token, None),
    )
    .await;
    assert_eq!(respondent_forms.0, StatusCode::FORBIDDEN);

    let operator_create_form = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &operator_token,
            Some(json!({
                "name": "Blocked Form",
                "slug": "blocked-form",
                "scope_node_type_id": null
            })),
        ),
    )
    .await;
    assert_eq!(operator_create_form.0, StatusCode::FORBIDDEN);

    let respondent_options = request_json(
        app.clone(),
        authorized_request("GET", "/api/responses/options", &respondent_token, None),
    )
    .await;
    assert_eq!(respondent_options["mode"], "assignment");
    assert!(
        respondent_options["assignments"]
            .as_array()
            .expect("respondent options should include assignments")
            .iter()
            .any(|assignment| assignment["node_name"] == "Demo Program Health Navigation")
    );

    let delegate_account_id = delegator_me["delegations"]
        .as_array()
        .expect("delegator delegation list should be present")
        .first()
        .and_then(|account| account["account_id"].as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("delegator should expose delegated account id");
    let delegated_options = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/responses/options?delegate_account_id={delegate_account_id}"),
            &delegator_token,
            None,
        ),
    )
    .await;
    assert_eq!(delegated_options["mode"], "assignment");
    assert!(
        delegated_options["assignments"]
            .as_array()
            .expect("delegated options should include assignments")
            .iter()
            .any(|assignment| assignment["node_name"] == "Demo Activity After School Check-ins")
    );
}

#[tokio::test]
async fn user_management_supports_create_edit_and_account_status() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let roles = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/roles", &admin_token, None),
    )
    .await;
    let operator_role_id = roles
        .as_array()
        .expect("roles should be an array")
        .iter()
        .find(|role| role["name"] == "operator")
        .and_then(|role| role["id"].as_str())
        .expect("operator role should be present");

    let created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/users",
            &admin_token,
            Some(json!({
                "email": "sprint1-operator@tessara.local",
                "display_name": "Sprint 1 Operator",
                "password": "tessara-dev-sprint1",
                "is_active": true,
                "role_ids": [operator_role_id]
            })),
        ),
    )
    .await;
    let account_id = created["id"]
        .as_str()
        .and_then(|value| Uuid::parse_str(value).ok())
        .expect("created user should expose id");

    let users = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/users", &admin_token, None),
    )
    .await;
    assert!(
        users
            .as_array()
            .expect("users should be an array")
            .iter()
            .any(|user| user["id"] == account_id.to_string() && user["is_active"] == true)
    );

    let detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/users/{account_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(detail["display_name"], "Sprint 1 Operator");
    assert_eq!(detail["is_active"], true);
    assert!(
        detail["roles"]
            .as_array()
            .expect("user detail should include roles")
            .iter()
            .any(|role| role["name"] == "operator")
    );

    let operator_token = login_token_for(
        app.clone(),
        "sprint1-operator@tessara.local",
        "tessara-dev-sprint1",
    )
    .await;
    let operator_me = request_json(
        app.clone(),
        authorized_request("GET", "/api/me", &operator_token, None),
    )
    .await;
    assert_eq!(operator_me["display_name"], "Sprint 1 Operator");

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{account_id}"),
            &admin_token,
            Some(json!({
                "email": "sprint1-operator@tessara.local",
                "display_name": "Sprint 1 Operator Updated",
                "password": null,
                "is_active": false,
                "role_ids": [operator_role_id]
            })),
        ),
    )
    .await;

    let inactive_me = request_status_and_json(
        app.clone(),
        authorized_request("GET", "/api/me", &operator_token, None),
    )
    .await;
    assert_eq!(inactive_me.0, StatusCode::UNAUTHORIZED);

    let inactive_login = request_status_and_json(
        app.clone(),
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "email": "sprint1-operator@tessara.local",
                    "password": "tessara-dev-sprint1"
                })
                .to_string(),
            ))
            .expect("valid inactive login request"),
    )
    .await;
    assert_eq!(inactive_login.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn role_management_updates_capabilities_and_scoped_access() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let roles = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/roles", &admin_token, None),
    )
    .await;
    let operator_role_id = roles
        .as_array()
        .expect("roles should be an array")
        .iter()
        .find(|role| role["name"] == "operator")
        .and_then(|role| role["id"].as_str())
        .expect("operator role should be present");

    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", &admin_token, None),
    )
    .await;
    let selected_capability_ids = capabilities
        .as_array()
        .expect("capabilities should be an array")
        .iter()
        .filter(|capability| {
            matches!(
                capability["key"].as_str(),
                Some("hierarchy:read" | "submissions:write")
            )
        })
        .filter_map(|capability| capability["id"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(selected_capability_ids.len(), 2);

    let observer_capability_ids = capabilities
        .as_array()
        .expect("capabilities should be an array")
        .iter()
        .filter(|capability| matches!(capability["key"].as_str(), Some("reports:read")))
        .filter_map(|capability| capability["id"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(observer_capability_ids.len(), 1);

    let created_role = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            &admin_token,
            Some(json!({
                "name": "observer",
                "capability_ids": observer_capability_ids
            })),
        ),
    )
    .await;
    let observer_role_id = created_role["id"]
        .as_str()
        .expect("created role should return an id");

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/roles/{operator_role_id}"),
            &admin_token,
            Some(json!({ "capability_ids": selected_capability_ids })),
        ),
    )
    .await;

    let operator_role_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/roles/{operator_role_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let operator_capability_keys = operator_role_detail["capabilities"]
        .as_array()
        .expect("role detail should include capabilities")
        .iter()
        .filter_map(|capability| capability["key"].as_str())
        .collect::<Vec<_>>();
    assert_eq!(operator_capability_keys.len(), 2);
    assert!(operator_capability_keys.contains(&"hierarchy:read"));
    assert!(operator_capability_keys.contains(&"submissions:write"));
    assert!(
        operator_role_detail["assigned_accounts"]
            .as_array()
            .expect("role detail should include assigned accounts")
            .iter()
            .any(|account| account["email"] == "operator@tessara.local")
    );

    let users = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/users", &admin_token, None),
    )
    .await;
    let operator_account_id = users
        .as_array()
        .expect("users should be an array")
        .iter()
        .find(|user| user["email"] == "operator@tessara.local")
        .and_then(|user| user["id"].as_str())
        .expect("seeded operator account should be present");
    let operator_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/users/{operator_account_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let updated_role_ids = operator_detail["roles"]
        .as_array()
        .expect("operator detail should include roles")
        .iter()
        .filter_map(|role| role["id"].as_str())
        .chain(std::iter::once(observer_role_id))
        .collect::<Vec<_>>();

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{operator_account_id}"),
            &admin_token,
            Some(json!({
                "email": operator_detail["email"],
                "display_name": operator_detail["display_name"],
                "password": null,
                "is_active": operator_detail["is_active"],
                "role_ids": updated_role_ids
            })),
        ),
    )
    .await;

    let observer_role_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/roles/{observer_role_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        observer_role_detail["assigned_accounts"]
            .as_array()
            .expect("created role should expose assigned accounts")
            .iter()
            .any(|account| account["email"] == "operator@tessara.local")
    );

    let north_star_nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=North%20Star", &admin_token, None),
    )
    .await;
    let north_star_partner_id = north_star_nodes
        .as_array()
        .expect("north star nodes should be an array")
        .iter()
        .find(|node| {
            node["name"] == "Demo Partner North Star Services"
                && node["node_type_name"] == "Partner"
        })
        .and_then(|node| node["id"].as_str())
        .expect("north star partner should be present");

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{operator_account_id}/access"),
            &admin_token,
            Some(json!({
                "scope_node_ids": [north_star_partner_id],
                "delegate_account_ids": []
            })),
        ),
    )
    .await;

    let updated_operator = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/users/{operator_account_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(updated_operator["ui_access_profile"], "operator");
    assert!(
        updated_operator["capabilities"]
            .as_array()
            .expect("user detail should include capabilities")
            .iter()
            .any(|capability| capability == "hierarchy:read")
    );
    assert_eq!(
        updated_operator["scope_nodes"]
            .as_array()
            .expect("user detail should include scope nodes")
            .len(),
        1
    );
    assert_eq!(
        updated_operator["scope_nodes"][0]["node_name"],
        "Demo Partner North Star Services"
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
    assert_eq!(operator_me["ui_access_profile"], "operator");
    assert_eq!(
        operator_me["scope_nodes"]
            .as_array()
            .expect("operator scope should be present")
            .len(),
        1
    );

    let operator_demo_nodes = request_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=Demo", &operator_token, None),
    )
    .await;
    let operator_demo_names = operator_demo_nodes
        .as_array()
        .expect("operator node list should be an array")
        .iter()
        .filter_map(|node| node["name"].as_str())
        .collect::<Vec<_>>();
    assert!(
        operator_demo_nodes
            .as_array()
            .expect("operator node list should be an array")
            .len()
            > 1
    );
    assert!(operator_demo_names.contains(&"Demo Partner North Star Services"));
    assert!(!operator_demo_names.contains(&"Demo Partner Community Bridge"));

    let operator_bridge_nodes = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/nodes?q=Community%20Bridge",
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        operator_bridge_nodes
            .as_array()
            .expect("operator bridge nodes should be an array")
            .len(),
        0
    );

    let no_capabilities: Vec<&str> = Vec::new();
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/roles/{operator_role_id}"),
            &admin_token,
            Some(json!({ "capability_ids": no_capabilities })),
        ),
    )
    .await;

    let forbidden_nodes = request_status_and_json(
        app.clone(),
        authorized_request("GET", "/api/nodes?q=North%20Star", &operator_token, None),
    )
    .await;
    assert_eq!(forbidden_nodes.0, StatusCode::FORBIDDEN);
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
    let version_two_id = create_form_version(app.clone(), &token, form_id, "v2").await;
    assert_ne!(version_one_id, version_two_id);
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
    assert_eq!(version_one["status"], "published");
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
        authorized_request("GET", "/api/nodes?q=Updated", &token, None),
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
async fn node_metadata_fields_can_be_deleted() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

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
            "DELETE",
            &format!("/api/admin/node-metadata-fields/{metadata_field_id}"),
            &token,
            None,
        ),
    )
    .await;

    let metadata_fields = request_json(
        app,
        authorized_request("GET", "/api/admin/node-metadata-fields", &token, None),
    )
    .await;
    assert!(
        metadata_fields
            .as_array()
            .expect("metadata field list should be an array")
            .iter()
            .all(|field| field["id"] != metadata_field_id.to_string())
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
            Some(json!({})),
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
async fn readable_node_type_catalog_exposes_labels_and_relationships() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;

    let admin_node_types = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/node-types", &admin_token, None),
    )
    .await;
    let partner_summary = admin_node_types
        .as_array()
        .expect("admin node type list should be an array")
        .iter()
        .find(|entry| entry["slug"] == "partner")
        .expect("partner summary should exist");
    assert_eq!(partner_summary["singular_label"], "Partner");
    assert_eq!(partner_summary["plural_label"], "Partners");
    assert_eq!(partner_summary["is_root_type"], true);

    let readable_catalog = request_json(
        app.clone(),
        authorized_request("GET", "/api/node-types", &operator_token, None),
    )
    .await;
    let program_entry = readable_catalog
        .as_array()
        .expect("readable node type catalog should be an array")
        .iter()
        .find(|entry| entry["slug"] == "program")
        .expect("program entry should exist");
    assert_eq!(program_entry["singular_label"], "Program");
    assert_eq!(program_entry["plural_label"], "Programs");
    assert_eq!(program_entry["is_root_type"], false);
    assert!(
        program_entry["parent_relationships"]
            .as_array()
            .expect("program entry should include parent relationships")
            .iter()
            .any(|parent| parent["node_type_slug"] == "partner"
                && parent["singular_label"] == "Partner"
                && parent["plural_label"] == "Partners")
    );
    assert!(
        program_entry["child_relationships"]
            .as_array()
            .expect("program entry should include child relationships")
            .iter()
            .any(|child| child["node_type_slug"] == "activity"
                && child["singular_label"] == "Activity")
    );
}

#[tokio::test]
async fn non_root_node_types_require_a_parent_node() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let parent_type_id = create_node_type(app.clone(), &token, "Partner", "partner");
    let parent_type_id = parent_type_id.await;
    let child_type_id = create_node_type(app.clone(), &token, "Program", "program");
    let child_type_id = child_type_id.await;
    create_node_type_relationship(app.clone(), &token, parent_type_id, child_type_id).await;

    let missing_parent = request_status_and_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/nodes",
            &token,
            Some(json!({
                "node_type_id": child_type_id,
                "parent_node_id": null,
                "name": "Detached Program",
                "metadata": {}
            })),
        ),
    )
    .await;
    assert_eq!(missing_parent.0, StatusCode::BAD_REQUEST);
    assert!(
        missing_parent.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("parent is required")
    );
}

#[tokio::test]
async fn operator_cannot_access_admin_node_type_management_routes() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;

    let forbidden = request_status_and_json(
        app,
        authorized_request("GET", "/api/admin/node-types", &operator_token, None),
    )
    .await;
    assert_eq!(forbidden.0, StatusCode::FORBIDDEN);
    assert_eq!(forbidden.1["error"], "forbidden: admin:all");
}

#[tokio::test]
async fn node_type_updates_reject_cycles_in_parent_child_selections() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;

    let parent_type_id = create_node_type(app.clone(), &admin_token, "Partner", "partner").await;
    let child_type_id = create_node_type(app.clone(), &admin_token, "Program", "program").await;

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/node-types/{parent_type_id}"),
            &admin_token,
            Some(json!({
                "name": "Partner",
                "slug": "partner",
                "plural_label": "Partners",
                "child_node_type_ids": [child_type_id]
            })),
        ),
    )
    .await;

    let cyclic_update = request_status_and_json(
        app,
        authorized_request(
            "PUT",
            &format!("/api/admin/node-types/{child_type_id}"),
            &admin_token,
            Some(json!({
                "name": "Program",
                "slug": "program",
                "plural_label": "Programs",
                "child_node_type_ids": [parent_type_id]
            })),
        ),
    )
    .await;

    assert_eq!(cyclic_update.0, StatusCode::BAD_REQUEST);
    assert!(
        cyclic_update.1["error"]
            .as_str()
            .expect("error body should include message")
            .contains("both a parent and child")
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
            .body(Body::from(json!({}).to_string()))
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
                    "composition_mode": "union",
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
                && dataset["composition_mode"] == "union"
                && dataset["source_count"] == 1
                && dataset["field_count"] == 1)
    );

    let definition = request_json(
        app.clone(),
        authorized_request("GET", &format!("/api/datasets/{dataset_id}"), &token, None),
    )
    .await;
    assert_eq!(definition["name"], "Participant Dataset");
    assert_eq!(definition["composition_mode"], "union");
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
                "composition_mode": "union",
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
    assert_eq!(updated_definition["composition_mode"], "union");
    assert_eq!(updated_definition["sources"][0]["source_alias"], "services");
    assert_eq!(updated_definition["fields"][0]["key"], "participants_total");

    let join_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Joined Participant Dataset",
                "slug": "joined-participant-dataset",
                "grain": "submission",
                "composition_mode": "join",
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
    let join_dataset_id = id_from(&join_dataset);
    let join_definition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{join_dataset_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(join_definition["composition_mode"], "join");
    let join_dataset_run = request_status_and_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{join_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(join_dataset_run.0, StatusCode::BAD_REQUEST);
    assert!(
        join_dataset_run.1["error"]
            .as_str()
            .expect("join dataset run should include an error message")
            .contains("at least two sources")
    );
    let join_report = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Joined Participant Report",
                "form_id": null,
                "dataset_id": join_dataset_id,
                "fields": [{
                    "logical_key": "participant_count",
                    "source_field_key": "participant_count",
                    "computed_expression": null,
                    "missing_policy": "null"
                }]
            })),
        ),
    )
    .await;
    assert_eq!(join_report.0, StatusCode::BAD_REQUEST);
    assert!(
        join_report.1["error"]
            .as_str()
            .expect("join report create should include an error message")
            .contains("executable dataset sources")
    );

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
                "composition_mode": "union",
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
                "composition_mode": "union",
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
                "composition_mode": "union",
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
            Some(json!({"values": {
                "session_date": "2026-07-01",
                "participants": 99,
                "completed_as_planned": true
            }})),
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
    assert!(
        latest_rows["rows"]
            .as_array()
            .expect("latest dataset should return row array")
            .iter()
            .any(
                |row| row["submission_id"] == follow_up_submission_id.to_string()
                    && row["values"]["participant_count"] == "99"
            )
    );

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
    assert!(
        earliest_rows["rows"]
            .as_array()
            .expect("earliest dataset should return row array")
            .iter()
            .any(|row| row["submission_id"] == seed["submission_id"]
                && row["values"]["participant_count"] == "42")
    );
}

#[tokio::test]
async fn join_mode_datasets_merge_selected_source_rows_by_node() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;

    let follow_up_form = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/forms",
            &token,
            Some(json!({
                "name": "Join Follow Up",
                "slug": "join-follow-up",
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

    let joined_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Joined Activity Dataset",
                "slug": "joined-activity-dataset",
                "grain": "submission",
                "composition_mode": "join",
                "sources": [
                    {
                        "source_alias": "check_in",
                        "form_id": seed["form_id"],
                        "compatibility_group_id": null,
                        "selection_rule": "latest"
                    },
                    {
                        "source_alias": "follow_up",
                        "form_id": follow_up_form_id,
                        "compatibility_group_id": null,
                        "selection_rule": "latest"
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
    let joined_dataset_id = id_from(&joined_dataset);
    let joined_rows = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{joined_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        joined_rows["rows"]
            .as_array()
            .expect("join dataset should return row array")
            .len(),
        2
    );
    let primary_joined_row = joined_rows["rows"]
        .as_array()
        .expect("join dataset should return row array")
        .iter()
        .find(|row| row["values"]["attendee_count"] == "7")
        .expect("joined dataset should include the follow-up row");
    assert_eq!(primary_joined_row["source_alias"], "join");
    assert_eq!(primary_joined_row["values"]["participant_count"], "42");
    assert_eq!(primary_joined_row["values"]["attendee_count"], "7");
    let joined_submission_id = primary_joined_row["submission_id"]
        .as_str()
        .expect("join dataset row should expose a composed submission id");
    assert!(joined_submission_id.contains("check_in:"));
    assert!(joined_submission_id.contains("follow_up:"));

    let joined_report = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/reports",
            &token,
            Some(json!({
                "name": "Joined Activity Report",
                "form_id": null,
                "dataset_id": joined_dataset_id,
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
                    },
                    {
                        "logical_key": "status",
                        "source_field_key": null,
                        "computed_expression": "literal:Submitted",
                        "missing_policy": "null"
                    }
                ]
            })),
        ),
    )
    .await;
    let joined_report_id = id_from(&joined_report);
    let joined_report_rows = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/reports/{joined_report_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        joined_report_rows["rows"]
            .as_array()
            .expect("joined report should return row array")
            .len(),
        6
    );
    assert!(
        joined_report_rows["rows"]
            .as_array()
            .expect("joined report should return row array")
            .iter()
            .any(|row| row["logical_key"] == "participant_count"
                && row["field_value"] == "42"
                && row["source_alias"] == "join")
    );
    assert!(
        joined_report_rows["rows"]
            .as_array()
            .expect("joined report should return row array")
            .iter()
            .any(|row| row["logical_key"] == "attendee_count"
                && row["field_value"] == "7"
                && row["source_alias"] == "join")
    );
    assert!(
        joined_report_rows["rows"]
            .as_array()
            .expect("joined report should return row array")
            .iter()
            .any(|row| row["logical_key"] == "status"
                && row["field_value"] == "Submitted"
                && row["source_alias"] == "join")
    );

    let joined_aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/aggregations",
            &token,
            Some(json!({
                "name": "Joined Activity Aggregation",
                "report_id": joined_report_id,
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
                    },
                    {
                        "metric_key": "attendees_total",
                        "source_logical_key": "attendee_count",
                        "metric_kind": "sum"
                    }
                ]
            })),
        ),
    )
    .await;
    let joined_aggregation_id = id_from(&joined_aggregation);
    let joined_aggregation_rows = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/aggregations/{joined_aggregation_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        joined_aggregation_rows["rows"][0]["metrics"]["responses"],
        2.0
    );
    assert_eq!(
        joined_aggregation_rows["rows"][0]["metrics"]["participants_total"],
        60.0
    );
    assert_eq!(
        joined_aggregation_rows["rows"][0]["metrics"]["attendees_total"],
        7.0
    );

    let joined_chart = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/charts",
            &token,
            Some(json!({
                "name": "Joined Activity Chart",
                "report_id": null,
                "aggregation_id": joined_aggregation_id,
                "chart_type": "table"
            })),
        ),
    )
    .await;
    let joined_chart_id = id_from(&joined_chart);
    let joined_dashboard = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/dashboards",
            &token,
            Some(json!({
                "name": "Joined Activity Dashboard"
            })),
        ),
    )
    .await;
    let joined_dashboard_id = id_from(&joined_dashboard);
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/dashboards/{joined_dashboard_id}/components"),
            &token,
            Some(json!({
                "chart_id": joined_chart_id,
                "position": 0,
                "config": { "title": "Joined Activity Summary" }
            })),
        ),
    )
    .await;
    let joined_dashboard_view = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/dashboards/{joined_dashboard_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(
        joined_dashboard_view["components"][0]["chart"]["chart_type"],
        "table"
    );
    assert_eq!(
        joined_dashboard_view["components"][0]["chart"]["aggregation_id"],
        joined_aggregation_id.to_string()
    );
    assert_eq!(
        joined_dashboard_view["components"][0]["chart"]["aggregation_url"],
        format!("/api/aggregations/{joined_aggregation_id}/table")
    );

    let invalid_join_dataset = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &token,
            Some(json!({
                "name": "Invalid Joined Activity Dataset",
                "slug": "invalid-joined-activity-dataset",
                "grain": "submission",
                "composition_mode": "join",
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
                        "selection_rule": "latest"
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
    let invalid_join_dataset_id = id_from(&invalid_join_dataset);
    let invalid_join_run = request_status_and_json(
        app,
        authorized_request(
            "GET",
            &format!("/api/datasets/{invalid_join_dataset_id}/table"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(invalid_join_run.0, StatusCode::BAD_REQUEST);
    assert!(
        invalid_join_run.1["error"]
            .as_str()
            .expect("invalid join dataset run should include an error message")
            .contains("latest or earliest")
    );
}

#[tokio::test]
async fn aggregations_support_avg_min_and_max_metrics() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let token = login_token(app.clone()).await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;

    let second_draft = request_json(
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
    let second_submission_id = id_from(&second_draft);
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/submissions/{second_submission_id}/values"),
            &token,
            Some(json!({"values": {
                "session_date": "2026-07-15",
                "participants": 10,
                "completed_as_planned": true
            }})),
        ),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/submissions/{second_submission_id}/submit"),
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

    let report_id = seed["report_id"]
        .as_str()
        .expect("seed should include a report id");
    let aggregation = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/aggregations",
            &token,
            Some(json!({
                "name": "Participants Statistics",
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
                    },
                    {
                        "metric_key": "participants_average",
                        "source_logical_key": "participants",
                        "metric_kind": "avg"
                    },
                    {
                        "metric_key": "participants_minimum",
                        "source_logical_key": "participants",
                        "metric_kind": "min"
                    },
                    {
                        "metric_key": "participants_maximum",
                        "source_logical_key": "participants",
                        "metric_kind": "max"
                    }
                ]
            })),
        ),
    )
    .await;
    let aggregation_id = id_from(&aggregation);
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
    assert_eq!(aggregation_table["rows"][0]["metrics"]["responses"], 3.0);
    assert_eq!(
        aggregation_table["rows"][0]["metrics"]["participants_total"],
        70.0
    );
    let average = aggregation_table["rows"][0]["metrics"]["participants_average"]
        .as_f64()
        .expect("participants average should be numeric");
    assert!((average - (70.0 / 3.0)).abs() < 0.0001);
    assert_eq!(
        aggregation_table["rows"][0]["metrics"]["participants_minimum"],
        10.0
    );
    assert_eq!(
        aggregation_table["rows"][0]["metrics"]["participants_maximum"],
        42.0
    );

    let invalid_avg = request_status_and_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/aggregations",
            &token,
            Some(json!({
                "name": "Broken Average",
                "report_id": report_id,
                "group_by_logical_key": null,
                "metrics": [{
                    "metric_key": "participants_average",
                    "source_logical_key": null,
                    "metric_kind": "avg"
                }]
            })),
        ),
    )
    .await;
    assert_eq!(invalid_avg.0, StatusCode::BAD_REQUEST);
    assert!(
        invalid_avg.1["error"]
            .as_str()
            .expect("invalid avg aggregation should include an error message")
            .contains("avg metrics require a source logical key")
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
        authorized_request(
            "GET",
            &format!("/api/dashboards/{dashboard_id}"),
            &token,
            None,
        ),
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
        authorized_request(
            "GET",
            &format!("/api/dashboards/{dashboard_id}"),
            &token,
            None,
        ),
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
        authorized_request(
            "GET",
            &format!("/api/dashboards/{dashboard_id}"),
            &token,
            None,
        ),
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
        authorized_request(
            "GET",
            &format!("/api/dashboards/{dashboard_id}"),
            &token,
            None,
        ),
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
        authorized_request(
            "GET",
            &format!("/api/dashboards/{}", summary.dashboard_id),
            &token,
            None,
        ),
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
    login_token_for(app, "admin@tessara.local", "tessara-dev-admin").await
}

async fn login_token_for(app: axum::Router, email: &str, password: &str) -> String {
    let login = request_json(
        app,
        Request::builder()
            .method("POST")
            .uri("/api/auth/login")
            .header(header::CONTENT_TYPE, "application/json")
            .body(Body::from(
                json!({
                    "email": email,
                    "password": password
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
    _version_label: &str,
) -> Uuid {
    let version = request_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/admin/forms/{form_id}/versions"),
            token,
            Some(json!({})),
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
    let sanitized_uri = uri.trim().replace(' ', "%20");
    let mut builder = Request::builder()
        .method(method)
        .uri(sanitized_uri)
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
