use std::sync::LazyLock;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::{Row, postgres::PgPoolOptions};
use tessara_api::{config::Config, db, router};
use tower::ServiceExt;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[path = "support/database_safety.rs"]
mod database_safety;
#[path = "support/datasets.rs"]
mod dataset_support;

use database_safety::{DISPOSABLE_DATABASE_NAME_TOKENS, is_disposable_database_name};
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
    let app = test_app().await;
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    assert_eq!(seed["seed_version"], "uat-demo-v2");
    assert_eq!(seed["dataset_count"], 4);
    assert_eq!(seed["dataset_revision_count"], 4);
    assert_eq!(seed["component_count"], 9);
    assert_eq!(seed["dashboard_count"], 1);
    assert_eq!(seed["submitted_submission_count"], 58);

    let summary = request_json(
        app.clone(),
        authorized_request("GET", "/api/summary", &admin_token, None),
    )
    .await;
    assert_eq!(summary["datasets"], 4);
    assert_eq!(summary["dataset_revisions"], 4);
    assert_eq!(summary["components"], 9);
    assert_eq!(summary["component_versions"], 9);
    assert_eq!(summary["dashboards"], 1);
    assert_eq!(summary["submitted_submissions"], 58);
    assert!(summary.get("reports").is_none());
    assert!(summary.get("charts").is_none());

    let components = request_json(
        app.clone(),
        authorized_request("GET", "/api/components", &admin_token, None),
    )
    .await;
    let seeded_component = components
        .as_array()
        .expect("components should be an array")
        .iter()
        .find(|component| component["current_version_id"] == seed["component_version_id"])
        .expect("seeded component should appear in the component directory");
    assert!(
        components
            .as_array()
            .expect("components should be an array")
            .iter()
            .any(|component| component["current_version_id"] == seed["component_version_id"])
    );
    let component_kinds = components
        .as_array()
        .expect("components should be an array")
        .iter()
        .filter_map(|component| component["current_component_type"].as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for kind in ["table", "bar", "line", "pie", "donut", "stat_card"] {
        assert!(
            component_kinds.contains(kind),
            "seeded demo components should include {kind}"
        );
    }
    let seeded_dataset_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/datasets/{}/table?page_size=100",
                seed["dataset_id"].as_str().expect("dataset id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        seeded_dataset_table["rows"]
            .as_array()
            .expect("seeded dataset table rows")
            .len(),
        52
    );
    let seeded_dataset_columns = seeded_dataset_table["rows"]
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(|row| row["values"].as_object())
        .expect("seeded dataset row values")
        .keys()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    for field in [
        "session__session_date",
        "session__participants",
        "session__completed_as_planned",
        "session__facilitator_notes",
        "session__topics_covered",
    ] {
        assert!(
            seeded_dataset_columns.contains(field),
            "session log dataset should keep field {field}"
        );
    }
    let completed_values = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/datasets/{}/distinct-values?version_major=1&field=session__completed_as_planned",
                seed["dataset_id"].as_str().expect("dataset id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(completed_values["version_major"], 1);
    assert_eq!(completed_values["field"], "session__completed_as_planned");
    assert_eq!(completed_values["values"], json!(["false", "true"]));
    let seeded_component_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/components/{}/table?page_size=100",
                seeded_component["slug"].as_str().expect("component slug")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        seeded_component_table["rows"]
            .as_array()
            .expect("seeded component table rows")
            .len(),
        52
    );

    let seeded_bar = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/components/demo-session-log-bar/bar",
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(seeded_bar["component_type"], "bar");
    assert!(
        seeded_bar["points"]
            .as_array()
            .expect("seeded bar points")
            .iter()
            .any(|point| {
                point["comparison"] == "Completed as planned"
                    && point["color"] == "var(--semantic-primary)"
            })
    );

    let seeded_line = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/components/demo-session-log-line/line",
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(seeded_line["component_type"], "line");
    assert!(
        !seeded_line["points"]
            .as_array()
            .expect("seeded line points")
            .is_empty()
    );

    for (slug, kind) in [
        ("demo-session-completion-pie", "pie"),
        ("demo-session-completion-donut", "donut"),
    ] {
        let seeded_slices = request_json(
            app.clone(),
            authorized_request(
                "GET",
                &format!("/api/components/{slug}/{kind}"),
                &admin_token,
                None,
            ),
        )
        .await;
        assert_eq!(seeded_slices["component_type"], kind);
        assert_eq!(seeded_slices["legend_title"], "Completion Status");
        assert!(
            seeded_slices["slices"]
                .as_array()
                .expect("seeded slices")
                .iter()
                .any(|slice| {
                    slice["category"] == "Did not complete as planned"
                        && slice["color"] == "var(--semantic-warning)"
                })
        );
    }

    let seeded_stat_card = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/components/demo-session-total-participants-stat-card/stat-card",
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(seeded_stat_card["component_type"], "stat_card");
    assert_eq!(seeded_stat_card["stat"]["label"], "Total participants");
    assert_eq!(
        seeded_stat_card["stat"]["supporting_text"],
        "Submitted Demo Session Log entries"
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
    assert_eq!(dashboard["placement_count"], 9);
    assert_eq!(
        dashboard["placements"]
            .as_array()
            .expect("dashboard placements should be an array")
            .len(),
        9
    );
    assert!(
        dashboard["placements"]
            .as_array()
            .expect("dashboard placements should be an array")
            .iter()
            .any(|placement| {
                placement["component"]["component_version_id"] == seed["component_version_id"]
            })
    );
    assert!(
        dashboard["placements"]
            .as_array()
            .expect("dashboard placements should be an array")
            .iter()
            .all(|placement| {
                placement["availability"] == "available"
                    && placement["grid_row"].as_u64().is_some_and(|row| row >= 1)
                    && placement["grid_column"]
                        .as_u64()
                        .is_some_and(|column| (1..=12).contains(&column))
                    && placement["grid_width"]
                        .as_u64()
                        .is_some_and(|width| (1..=12).contains(&width))
                    && placement["grid_height"]
                        .as_u64()
                        .is_some_and(|height| (1..=6).contains(&height))
            })
    );

    let composition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/admin/dashboards/{}/composition",
                seed["dashboard_id"].as_str().expect("dashboard id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    let composition_placements = composition["dashboard"]["placements"]
        .as_array()
        .expect("composition should include placements");
    assert_eq!(composition_placements.len(), 9);
    let seed_pool = PgPoolOptions::new()
        .connect(&std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL should be set"))
        .await
        .expect("seed config verification pool");
    let seeded_configs = sqlx::query_scalar::<_, Value>(
        "SELECT config FROM dashboard_components WHERE dashboard_id = $1 ORDER BY position, id",
    )
    .bind(
        seed["dashboard_id"]
            .as_str()
            .expect("dashboard id")
            .parse::<Uuid>()
            .expect("dashboard UUID"),
    )
    .fetch_all(&seed_pool)
    .await
    .expect("seeded Dashboard configs");
    assert_eq!(seeded_configs.len(), 9);
    assert!(seeded_configs.iter().all(|config| {
        config["schema_version"] == 1
            && config["grid_row"].as_i64().is_some()
            && config["grid_column"].as_i64().is_some()
            && config["grid_width"].as_i64().is_some()
            && config["grid_height"].as_i64().is_some()
    }));
    let session_table_config = seeded_configs
        .iter()
        .find(|config| config["title"] == "Session Log Table")
        .expect("seeded Session Log Table config");
    assert_eq!(session_table_config["grid_row"], 9);
    assert_eq!(session_table_config["grid_column"], 1);
    assert_eq!(session_table_config["grid_width"], 12);
    assert_eq!(session_table_config["grid_height"], 6);
    assert!(
        seeded_configs
            .iter()
            .filter(|config| matches!(
                config["title"].as_str(),
                Some("Participants by Completion" | "Participants Over Time")
            ))
            .all(|config| config["grid_row"] == 15)
    );
    assert!(
        composition["available_component_versions"]
            .as_array()
            .is_some_and(|options| options.len() >= 9)
    );
    let commands = composition_placements
        .iter()
        .map(|placement| {
            json!({
                "operation": "retain",
                "placement_id": placement["placement_id"],
                "geometry": {
                    "grid_row": placement["grid_row"],
                    "grid_column": placement["grid_column"],
                    "grid_width": placement["grid_width"],
                    "grid_height": placement["grid_height"]
                }
            })
        })
        .collect::<Vec<_>>();
    let reconciled = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!(
                "/api/admin/dashboards/{}/composition",
                seed["dashboard_id"].as_str().expect("dashboard id")
            ),
            &admin_token,
            Some(json!({ "commands": commands })),
        ),
    )
    .await;
    assert_eq!(reconciled["dashboard"]["placement_count"], 9);
    assert_eq!(
        reconciled["dashboard"]["placements"]
            .as_array()
            .expect("reconciled placements")
            .iter()
            .filter_map(|placement| placement["placement_id"].as_str())
            .collect::<std::collections::BTreeSet<_>>(),
        composition_placements
            .iter()
            .filter_map(|placement| placement["placement_id"].as_str())
            .collect::<std::collections::BTreeSet<_>>()
    );
    let seeded_revision = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/datasets/{}/revisions/{}",
                seed["dataset_id"].as_str().expect("dataset id"),
                seed["dataset_revision_id"]
                    .as_str()
                    .expect("dataset revision id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        seeded_revision["dependencies"]["component_version_count"],
        6
    );
    assert_eq!(seeded_revision["dependencies"]["dashboard_count"], 1);
    assert!(
        seeded_revision["dependency_impacts"]
            .as_array()
            .expect("dependency impacts")
            .iter()
            .any(|impact| {
                impact["kind"] == "component_version"
                    && impact["id"] == seed["component_version_id"]
            })
    );
    assert!(
        seeded_revision["dependency_impacts"]
            .as_array()
            .expect("dependency impacts")
            .iter()
            .any(|impact| impact["kind"] == "dashboard" && impact["id"] == seed["dashboard_id"])
    );

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let scoped_seeded_revision = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/datasets/{}/revisions/{}",
                seed["dataset_id"].as_str().expect("dataset id"),
                seed["dataset_revision_id"]
                    .as_str()
                    .expect("dataset revision id")
            ),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        scoped_seeded_revision["dependencies"]["component_version_count"],
        6
    );
    assert_eq!(scoped_seeded_revision["dependencies"]["dashboard_count"], 1);
    assert!(
        scoped_seeded_revision["dependency_impacts"]
            .as_array()
            .expect("scoped dependency impacts")
            .iter()
            .any(|impact| impact["kind"] == "component_version")
    );
    assert!(
        scoped_seeded_revision["dependency_impacts"]
            .as_array()
            .expect("scoped dependency impacts")
            .iter()
            .any(|impact| impact["kind"] == "dashboard")
    );
    let admin_revision_history = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/datasets/{}/revisions",
                seed["dataset_id"].as_str().expect("dataset id")
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    let admin_seeded_revision_summary = admin_revision_history
        .as_array()
        .expect("admin revision history")
        .iter()
        .find(|revision| revision["id"] == seed["dataset_revision_id"])
        .expect("admin seeded revision summary");
    assert_eq!(
        admin_seeded_revision_summary["dependencies"]["component_version_count"],
        6
    );
    assert_eq!(
        admin_seeded_revision_summary["dependencies"]["dashboard_count"],
        1
    );
    let scoped_revision_history = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!(
                "/api/datasets/{}/revisions",
                seed["dataset_id"].as_str().expect("dataset id")
            ),
            &operator_token,
            None,
        ),
    )
    .await;
    let scoped_seeded_revision_summary = scoped_revision_history
        .as_array()
        .expect("scoped revision history")
        .iter()
        .find(|revision| revision["id"] == seed["dataset_revision_id"])
        .expect("scoped seeded revision summary");
    assert_eq!(
        scoped_seeded_revision_summary["dependencies"]["component_version_count"],
        6
    );
    assert_eq!(
        scoped_seeded_revision_summary["dependencies"]["dashboard_count"],
        1
    );
    let label_update = request_json(
        app.clone(),
        authorized_request(
            "PATCH",
            &format!(
                "/api/admin/datasets/{}/revisions/{}/label",
                seed["dataset_id"].as_str().expect("dataset id"),
                seed["dataset_revision_id"]
                    .as_str()
                    .expect("dataset revision id")
            ),
            &admin_token,
            Some(json!({
                "version_label": "Initial Seed",
                "revision_notes": "Seeded baseline notes"
            })),
        ),
    )
    .await;
    assert_eq!(label_update["version_label"], "Initial Seed");
    assert_eq!(label_update["revision_notes"], "Seeded baseline notes");
    let partial_label_update = request_json(
        app.clone(),
        authorized_request(
            "PATCH",
            &format!(
                "/api/admin/datasets/{}/revisions/{}/label",
                seed["dataset_id"].as_str().expect("dataset id"),
                seed["dataset_revision_id"]
                    .as_str()
                    .expect("dataset revision id")
            ),
            &admin_token,
            Some(json!({ "version_label": "Retitled Seed" })),
        ),
    )
    .await;
    assert_eq!(partial_label_update["version_label"], "Retitled Seed");
    assert_eq!(
        partial_label_update["revision_notes"],
        "Seeded baseline notes"
    );
    let partial_notes_update = request_json(
        app.clone(),
        authorized_request(
            "PATCH",
            &format!(
                "/api/admin/datasets/{}/revisions/{}/label",
                seed["dataset_id"].as_str().expect("dataset id"),
                seed["dataset_revision_id"]
                    .as_str()
                    .expect("dataset revision id")
            ),
            &admin_token,
            Some(json!({ "revision_notes": "Updated notes only" })),
        ),
    )
    .await;
    assert_eq!(partial_notes_update["version_label"], "Retitled Seed");
    assert_eq!(partial_notes_update["revision_notes"], "Updated notes only");

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
async fn demo_seed_requires_an_empty_domain_database() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    assert_eq!(seed["seed_version"], "uat-demo-v2");

    let (status, body) = request_status_and_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "bad_request");
    assert!(
        body["message"]
            .as_str()
            .expect("error message should be a string")
            .contains("requires an empty database")
    );
}

#[tokio::test]
async fn seeded_capability_catalog_uses_components_and_dashboards() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
async fn capability_catalog_and_role_detail_expose_scope_and_durable_provenance() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
    let admin_token =
        login_token_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", &admin_token, None),
    )
    .await;
    let capability_items = capabilities.as_array().expect("capability catalog");
    let forms_read = capability_items
        .iter()
        .find(|capability| capability["key"] == "forms:read")
        .expect("forms:read capability");
    assert_eq!(forms_read["scope_mode"], "scope_aware");
    let forms_provenance = forms_read["provenance"]
        .as_array()
        .expect("forms:read provenance");
    assert_eq!(forms_provenance.len(), 2);
    assert_eq!(forms_provenance[0]["source_kind"], "core");
    assert_eq!(forms_provenance[0]["source_key"], "core");
    assert_eq!(forms_provenance[0]["provider_state"], "core_authoritative");
    assert!(forms_provenance[0]["source_digest"].is_null());
    assert_eq!(
        forms_provenance[1]["source_kind"],
        "transition_contribution"
    );
    assert_eq!(forms_provenance[1]["definition_id"], "tessara.forms");
    assert_eq!(forms_provenance[1]["definition_display_name"], "Forms");
    assert_eq!(
        forms_provenance[1]["provider_state"],
        "transitional_in_process"
    );
    assert_eq!(
        forms_provenance[1]["source_digest"],
        "sha256:71bebdd07ff0028cc0da8bbd9707c393bade9951e5cedb265a4b8465d54b493e"
    );

    let modules_read = capability_items
        .iter()
        .find(|capability| capability["key"] == "modules:read")
        .expect("modules:read capability");
    assert_eq!(modules_read["scope_mode"], "installation_global");
    let module_provenance = modules_read["provenance"]
        .as_array()
        .expect("modules:read provenance");
    assert_eq!(module_provenance.len(), 1);
    assert_eq!(module_provenance[0]["source_kind"], "core");
    assert_eq!(module_provenance[0]["provider_state"], "core_authoritative");

    let roles = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/roles", &admin_token, None),
    )
    .await;
    let operator_role_id = roles
        .as_array()
        .expect("role catalog")
        .iter()
        .find(|role| role["name"] == "operator")
        .and_then(|role| role["id"].as_str())
        .expect("operator role id");
    let operator_role = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/roles/{operator_role_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let role_forms_read = operator_role["capabilities"]
        .as_array()
        .expect("role capabilities")
        .iter()
        .find(|capability| capability["key"] == "forms:read")
        .expect("role forms:read capability");
    assert_eq!(role_forms_read["scope_mode"], "scope_aware");
    assert_eq!(role_forms_read["provenance"], forms_read["provenance"]);
}

#[tokio::test]
async fn dataset_advanced_authoring_compiles_typed_fields_and_restriction_precedence() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
            .find("WHEN LOWER(COALESCE(\"restricted_flag\", '')) IN")
            .expect("restricted flag tier branch")
            < preview_sql
                .find("WHEN LOWER(COALESCE(\"internal_flag\", '')) IN")
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
async fn dataset_revision_draft_publish_preserves_current_until_publish() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
    let first_field = fields.first().expect("demo form should include fields");
    let second_field = fields
        .iter()
        .copied()
        .find(|field| field["key"] != first_field["key"])
        .expect("demo form should include a second field");
    let first_key = first_field["key"].as_str().expect("first field key");
    let first_label = first_field["label"].as_str().expect("first field label");
    let second_key = second_field["key"].as_str().expect("second field key");
    let second_label = second_field["label"].as_str().expect("second field label");

    let initial_payload = dataset_revision_payload(
        "Revision Lifecycle Dataset",
        "revision-lifecycle-dataset",
        form_id,
        form_version_id,
        visibility_node_id,
        json!([{
            "key": first_key,
            "label": first_label,
            "source_alias": "form_a",
            "source_field_key": first_key,
            "position": 0
        }]),
    );
    let created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(initial_payload.clone()),
        ),
    )
    .await;
    let dataset_id = created["id"].as_str().expect("created dataset id");
    let legacy_update_status = request_status(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/datasets/{dataset_id}"),
            &admin_token,
            Some(initial_payload.clone()),
        ),
    )
    .await;
    assert!(
        matches!(
            legacy_update_status,
            StatusCode::METHOD_NOT_ALLOWED | StatusCode::NOT_FOUND
        ),
        "legacy dataset update route should not accept published-state mutations"
    );
    let published_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let initial_revision_id = published_detail["current_revision_id"]
        .as_str()
        .expect("initial current revision id")
        .to_string();
    assert_dataset_fields(&published_detail, &[first_key], &[second_key]);
    let published_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_table_values(&published_table, &[first_key], &[second_key]);
    let initial_major_line_row_count = published_table["rows"]
        .as_array()
        .expect("initial dataset rows")
        .len();
    let component_slug = "revision-lifecycle-component-table";
    let created_component = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Revision Lifecycle Component Table",
                "slug": component_slug,
                "description": "Component bound to Dataset major version 1 for revision lifecycle coverage.",
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key]
                    }
                }
            })),
        ),
    )
    .await;
    let component_id = created_component["id"]
        .as_str()
        .expect("revision lifecycle component id")
        .to_string();
    let component_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{component_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let initial_component_version_id = component_detail["versions"][0]["id"]
        .as_str()
        .expect("initial component version id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!(
                "/api/admin/components/{component_id}/versions/{initial_component_version_id}/publish"
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    let atomic_slug = "atomic-save-command-component";
    let atomic_created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/save",
            &admin_token,
            Some(json!({
                "action": "save_draft",
                "component": {
                    "name": "Atomic Save Command Component",
                    "slug": atomic_slug,
                    "description": "Created through the edit-screen command endpoint."
                },
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key]
                    }
                }
            })),
        ),
    )
    .await;
    let atomic_component_id = atomic_created["id"].as_str().expect("atomic component id");
    let atomic_draft_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{atomic_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let atomic_draft_id = atomic_draft_detail["versions"][0]["id"]
        .as_str()
        .expect("atomic draft id");
    assert_eq!(atomic_draft_detail["versions"][0]["status"], "draft");

    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/save",
            &admin_token,
            Some(json!({
                "component_id": atomic_component_id,
                "draft_version_id": atomic_draft_id,
                "action": "create_new_version",
                "component": {
                    "name": "Atomic Save Command Component",
                    "slug": atomic_slug,
                    "description": "Published through the edit-screen command endpoint."
                },
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "version_note": "Initial atomic publish.",
                    "config": {
                        "visible_columns": [first_key]
                    }
                }
            })),
        ),
    )
    .await;
    let atomic_published_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{atomic_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let atomic_published_id = atomic_published_detail["versions"][0]["id"]
        .as_str()
        .expect("atomic published id")
        .to_string();
    assert_eq!(
        atomic_published_detail["versions"][0]["status"],
        "published"
    );
    assert_eq!(
        atomic_published_detail["versions"][0]["version_note"],
        "Initial atomic publish."
    );

    let pinning_dashboard = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/dashboards",
            &admin_token,
            Some(json!({
                "name": "Current-published pinning contract",
                "description": "Stable placement id and mutable current-published payload coverage.",
                "visibility_node_ids": [visibility_node_id]
            })),
        ),
    )
    .await;
    let pinning_dashboard_id = pinning_dashboard["id"]
        .as_str()
        .expect("pinning Dashboard id");
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/dashboards/{pinning_dashboard_id}/composition"),
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "bind",
                    "client_key": "current-published-pin",
                    "component_version_id": atomic_published_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 6,
                        "grid_height": 4
                    }
                }]
            })),
        ),
    )
    .await;
    let table_before_update = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{atomic_slug}/versions/{atomic_published_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_ne!(table_before_update["pagination"]["page_size"], 1);

    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/save",
            &admin_token,
            Some(json!({
                "component_id": atomic_component_id,
                "published_version_id": atomic_published_id,
                "action": "update_existing_version",
                "component": {
                    "name": "Atomic Save Command Component Updated",
                    "slug": atomic_slug,
                    "description": "Updated in place through the edit-screen command endpoint."
                },
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "version_note": "Updated current version in place.",
                    "config": {
                        "visible_columns": [first_key],
                        "page_size": 1
                    }
                }
            })),
        ),
    )
    .await;
    let atomic_updated_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{atomic_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        atomic_updated_detail["name"],
        "Atomic Save Command Component Updated"
    );
    assert_eq!(
        atomic_updated_detail["versions"][0]["id"],
        atomic_published_id
    );
    assert_eq!(atomic_updated_detail["versions"][0]["status"], "published");
    assert_eq!(
        atomic_updated_detail["versions"][0]["version_note"],
        "Updated current version in place."
    );
    assert!(
        atomic_updated_detail["versions"]
            .as_array()
            .expect("atomic versions")
            .iter()
            .all(|version| version["status"] != "draft")
    );
    let table_after_update = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{atomic_slug}/versions/{atomic_published_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        table_after_update["component_version_id"],
        atomic_published_id
    );
    assert_eq!(table_after_update["pagination"]["page_size"], 1);
    let pinning_detail_after_update = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/dashboards/{pinning_dashboard_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        pinning_detail_after_update["placements"][0]["component"]["component_version_id"],
        atomic_published_id
    );

    request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/save",
            &admin_token,
            Some(json!({
                "component_id": atomic_component_id,
                "action": "create_new_version",
                "component": {
                    "name": "Atomic Save Command Component Updated",
                    "slug": atomic_slug,
                    "description": "A newer version must not repin existing Dashboard placements."
                },
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "version_note": "Separate newer version for Dashboard pinning coverage.",
                    "config": {
                        "visible_columns": [first_key],
                        "page_size": 2
                    }
                }
            })),
        ),
    )
    .await;
    let after_new_version = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{atomic_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let newer_version_id = after_new_version["versions"]
        .as_array()
        .expect("atomic version history")
        .iter()
        .find(|version| version["status"] == "published")
        .and_then(|version| version["id"].as_str())
        .expect("new current published version");
    assert_ne!(newer_version_id, atomic_published_id);
    assert!(
        after_new_version["versions"]
            .as_array()
            .expect("atomic version history")
            .iter()
            .any(|version| {
                version["id"] == atomic_published_id && version["status"] == "superseded"
            })
    );
    let pinning_detail_after_new_version = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/dashboards/{pinning_dashboard_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        pinning_detail_after_new_version["placements"][0]["component"]["component_version_id"],
        atomic_published_id
    );

    let (superseded_update_status, superseded_update_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/save",
            &admin_token,
            Some(json!({
                "component_id": atomic_component_id,
                "published_version_id": atomic_published_id,
                "action": "update_existing_version",
                "component": {
                    "name": "Atomic Save Command Component Updated",
                    "slug": atomic_slug,
                    "description": "Superseded versions are immutable."
                },
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "version_note": "This write must be rejected.",
                    "config": { "visible_columns": [first_key] }
                }
            })),
        ),
    )
    .await;
    assert_eq!(superseded_update_status, StatusCode::BAD_REQUEST);
    assert_eq!(superseded_update_body["code"], "bad_request");
    let (legacy_shell_status, legacy_shell_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Legacy Revision Shell Component",
                "slug": "legacy-revision-shell-component",
                "description": "Should be rejected because component shells do not carry Dataset revision bindings.",
                "dataset_revision_id": initial_revision_id
            })),
        ),
    )
    .await;
    assert_eq!(legacy_shell_status, StatusCode::BAD_REQUEST);
    assert!(
        legacy_shell_body["error"]
            .as_str()
            .expect("legacy shell error")
            .contains("dataset_revision_id")
    );
    assert_eq!(legacy_shell_body["code"], "bad_request");
    assert!(legacy_shell_body["message"].is_string());
    let (legacy_shell_update_status, legacy_shell_update_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PATCH",
            &format!("/api/admin/components/{component_id}"),
            &admin_token,
            Some(json!({
                "name": "Revision Lifecycle Component Table",
                "slug": component_slug,
                "description": "Should be rejected through Tessara's API error envelope.",
                "dataset_revision_id": initial_revision_id
            })),
        ),
    )
    .await;
    assert_eq!(legacy_shell_update_status, StatusCode::BAD_REQUEST);
    assert_eq!(legacy_shell_update_body["code"], "bad_request");
    assert!(legacy_shell_update_body["message"].is_string());
    assert!(
        legacy_shell_update_body["error"]
            .as_str()
            .expect("legacy shell update error")
            .contains("dataset_revision_id")
    );
    let (legacy_component_status, legacy_component_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Legacy Revision Bound Component",
                "slug": "legacy-revision-bound-component",
                "description": "Should be rejected because component versions bind Dataset major lines.",
                "version": {
                    "dataset_revision_id": initial_revision_id,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key]
                    },
                }
            })),
        ),
    )
    .await;
    assert_eq!(legacy_component_status, StatusCode::BAD_REQUEST);
    assert!(
        legacy_component_body["error"]
            .as_str()
            .expect("legacy component error")
            .contains("dataset_revision_id")
    );
    let (legacy_version_status, legacy_version_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/components/{component_id}/versions"),
            &admin_token,
            Some(json!({
                "dataset_revision_id": initial_revision_id,
                "component_type": "table",
                "config": {
                    "visible_columns": [first_key]
                },
            })),
        ),
    )
    .await;
    assert_eq!(legacy_version_status, StatusCode::BAD_REQUEST);
    assert!(
        legacy_version_body["error"]
            .as_str()
            .expect("legacy version error")
            .contains("dataset_revision_id")
    );
    let (legacy_kind_detail_status, legacy_kind_detail_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/validate",
            &admin_token,
            Some(json!({
                "dataset_id": dataset_id,
                "dataset_version_major": 1,
                "component_type": "detail_table",
                "config": {
                    "visible_columns": [first_key]
                }
            })),
        ),
    )
    .await;
    assert_eq!(legacy_kind_detail_status, StatusCode::OK);
    assert_eq!(legacy_kind_detail_body["valid"], false);
    assert_eq!(
        legacy_kind_detail_body["findings"][0]["code"],
        "COMPONENT_UNSUPPORTED_KIND"
    );
    let (legacy_kind_aggregate_status, legacy_kind_aggregate_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components/validate",
            &admin_token,
            Some(json!({
                "dataset_id": dataset_id,
                "dataset_version_major": 1,
                "component_type": "aggregate_table",
                "config": {
                    "visible_columns": [first_key]
                }
            })),
        ),
    )
    .await;
    assert_eq!(legacy_kind_aggregate_status, StatusCode::OK);
    assert_eq!(legacy_kind_aggregate_body["valid"], false);
    assert_eq!(
        legacy_kind_aggregate_body["findings"][0]["code"],
        "COMPONENT_UNSUPPORTED_KIND"
    );
    let component_table_before_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{component_slug}/table?page_size=200"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(component_table_before_publish["dataset_version_major"], 1);
    assert_eq!(
        component_table_before_publish["rows"]
            .as_array()
            .expect("component rows before publish")
            .len(),
        initial_major_line_row_count
    );

    let visual_slug = "revision-lifecycle-visual-component";
    let visual_component = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Revision Lifecycle Visual Component",
                "slug": visual_slug,
                "description": "Visual component endpoint coverage over a Dataset major line.",
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "bar",
                    "config": {
                        "mode": "summary",
                        "summary_field": first_key,
                        "summary_type": "count",
                        "category_field": first_key,
                        "sort_field": "summary_value",
                        "sort_direction": "desc",
                        "number_of_points": 20,
                        "value_format": "integer"
                    }
                }
            })),
        ),
    )
    .await;
    let visual_component_id = visual_component["id"]
        .as_str()
        .expect("visual component id");
    let visual_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{visual_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let bar_version_id = visual_detail["versions"][0]["id"]
        .as_str()
        .expect("bar visual version id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!(
                "/api/admin/components/{visual_component_id}/versions/{bar_version_id}/publish"
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    let visual_bar = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/bar"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(visual_bar["component_type"], "bar");
    assert_eq!(visual_bar["component_version_id"], bar_version_id);
    assert_eq!(visual_bar["materialization_state"], "ready");
    assert!(
        !visual_bar["points"]
            .as_array()
            .expect("bar points")
            .is_empty()
    );
    let (wrong_kind_status, wrong_kind_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/line"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(wrong_kind_status, StatusCode::BAD_REQUEST);
    assert!(
        wrong_kind_body["error"]
            .as_str()
            .expect("wrong kind visual error")
            .contains("expected component type 'line'")
    );
    let table_for_visual_status = request_status(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(table_for_visual_status, StatusCode::BAD_REQUEST);
    let stat_card_alias_status = request_status(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/stat_card"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(stat_card_alias_status, StatusCode::NOT_FOUND);

    let line_draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/components/{visual_component_id}/versions"),
            &admin_token,
            Some(json!({
                "dataset_id": dataset_id,
                "dataset_version_major": 1,
                "component_type": "line",
                "version_note": "Switch visual component to a line chart.",
                "config": {
                    "summary_field": first_key,
                    "summary_type": "count",
                    "x_field": first_key,
                    "number_of_points": 20
                }
            })),
        ),
    )
    .await;
    let line_version_id = line_draft["id"].as_str().expect("line draft id");
    request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!(
                "/api/admin/components/{visual_component_id}/versions/{line_version_id}/publish"
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    let current_line = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/line"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(current_line["component_type"], "line");
    assert_eq!(current_line["component_version_id"], line_version_id);
    let historical_bar = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/versions/{bar_version_id}/bar"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(historical_bar["component_type"], "bar");
    assert_eq!(historical_bar["component_version_id"], bar_version_id);
    let versioned_wrong_kind_status = request_status(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{visual_slug}/versions/{line_version_id}/bar"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(versioned_wrong_kind_status, StatusCode::BAD_REQUEST);

    for (kind, path, config, output_key) in [
        (
            "pie",
            "pie",
            json!({
                "summary_field": first_key,
                "summary_type": "count",
                "category_field": first_key,
                "max_slices": 20
            }),
            "slices",
        ),
        (
            "donut",
            "donut",
            json!({
                "summary_field": first_key,
                "summary_type": "count",
                "category_field": first_key,
                "max_slices": 20
            }),
            "slices",
        ),
        (
            "stat_card",
            "stat-card",
            json!({
                "summary_field": first_key,
                "summary_type": "count",
                "label": "Submission count",
                "value_format": "integer",
                "panel_style": "accent"
            }),
            "stat",
        ),
    ] {
        let slug = format!("revision-lifecycle-{kind}-component");
        let component = request_json(
            app.clone(),
            authorized_request(
                "POST",
                "/api/admin/components",
                &admin_token,
                Some(json!({
                    "name": format!("Revision Lifecycle {kind} Component"),
                    "slug": slug,
                    "description": "Visual component endpoint coverage.",
                    "version": {
                        "dataset_id": dataset_id,
                        "dataset_version_major": 1,
                        "component_type": kind,
                        "config": config
                    }
                })),
            ),
        )
        .await;
        let component_id = component["id"].as_str().expect("visual component id");
        let detail = request_json(
            app.clone(),
            authorized_request(
                "GET",
                &format!("/api/admin/components/{slug}"),
                &admin_token,
                None,
            ),
        )
        .await;
        let version_id = detail["versions"][0]["id"]
            .as_str()
            .expect("visual version id");
        request_json(
            app.clone(),
            authorized_request(
                "POST",
                &format!("/api/admin/components/{component_id}/versions/{version_id}/publish"),
                &admin_token,
                None,
            ),
        )
        .await;
        let visual = request_json(
            app.clone(),
            authorized_request(
                "GET",
                &format!("/api/components/{slug}/{path}"),
                &admin_token,
                None,
            ),
        )
        .await;
        assert_eq!(visual["component_type"], kind);
        assert_eq!(visual["component_version_id"], version_id);
        if output_key == "stat" {
            assert!(visual[output_key].is_object());
        } else {
            assert!(
                !visual[output_key]
                    .as_array()
                    .unwrap_or_else(|| panic!("{kind} should return {output_key}"))
                    .is_empty()
            );
        }
    }

    let revision_before_component_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{initial_revision_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let published_history_dependency_count =
        revision_before_component_draft["dependencies"]["component_version_count"].clone();
    let concurrent_draft_payload = json!({
        "dataset_id": dataset_id,
        "dataset_version_major": 1,
        "component_type": "table",
        "config": {
            "visible_columns": [first_key],
            "page_size": 25
        },
    });
    let (first_concurrent_draft, second_concurrent_draft) = tokio::join!(
        request_json(
            app.clone(),
            authorized_request(
                "POST",
                &format!("/api/admin/components/{component_id}/versions"),
                &admin_token,
                Some(concurrent_draft_payload.clone()),
            ),
        ),
        request_json(
            app.clone(),
            authorized_request(
                "POST",
                &format!("/api/admin/components/{component_id}/versions"),
                &admin_token,
                Some(concurrent_draft_payload),
            ),
        )
    );
    assert_eq!(first_concurrent_draft["id"], second_concurrent_draft["id"]);
    let component_after_concurrent_drafts = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{component_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let versions_after_concurrent_drafts = component_after_concurrent_drafts["versions"]
        .as_array()
        .expect("component versions after concurrent drafts");
    assert_eq!(
        versions_after_concurrent_drafts
            .iter()
            .filter(|version| version["status"] == "draft")
            .count(),
        1,
        "concurrent draft writes should converge on the one working draft"
    );
    assert_eq!(
        versions_after_concurrent_drafts
            .iter()
            .filter(|version| version["status"] == "published")
            .count(),
        1,
        "published component version should remain current while a draft is edited"
    );
    let revision_with_component_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{initial_revision_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        revision_with_component_draft["dependencies"]["component_version_count"],
        published_history_dependency_count,
        "working component drafts should not inflate published-history dependency counts"
    );
    let no_op_draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/draft-revision"),
            &admin_token,
            Some(initial_payload.clone()),
        ),
    )
    .await;
    let no_op_draft_revision_id = no_op_draft["revision_id"]
        .as_str()
        .expect("no-op draft revision id");
    let no_op_publish_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            &format!(
                "/api/admin/datasets/{dataset_id}/revisions/{no_op_draft_revision_id}/publish"
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(no_op_publish_status, StatusCode::BAD_REQUEST);
    let dependent_payload = json!({
        "name": "Revision Lifecycle Dependent Dataset",
        "slug": "revision-lifecycle-dependent-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "dataset",
            "alias": "upstream",
            "dataset_id": dataset_id,
            "dataset_revision_id": initial_revision_id
        },
        "operations": [
            projection_operation(json!([{
                "key": "dependent_value",
                "label": "Dependent Value",
                "source_alias": "upstream",
                "source_field_key": first_key,
                "position": 0
            }]), 0)
        ],
        "restriction_policy": null
    });
    let dependent_created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(dependent_payload),
        ),
    )
    .await;
    let dependent_dataset_id = dependent_created["id"]
        .as_str()
        .expect("dependent dataset id")
        .to_string();
    let major_dependent_payload = json!({
        "name": "Revision Lifecycle Major Dependent Dataset",
        "slug": "revision-lifecycle-major-dependent-dataset",
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "dataset_major",
            "alias": "upstream",
            "dataset_id": dataset_id,
            "version_major": 1
        },
        "operations": [
            projection_operation(json!([{
                "key": "major_dependent_value",
                "label": "Major Dependent Value",
                "source_alias": "upstream",
                "source_field_key": first_key,
                "position": 0
            }]), 0)
        ],
        "restriction_policy": null
    });
    let major_dependent_created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/datasets",
            &admin_token,
            Some(major_dependent_payload.clone()),
        ),
    )
    .await;
    let major_dependent_dataset_id = major_dependent_created["id"]
        .as_str()
        .expect("major dependent dataset id")
        .to_string();
    let major_dependent_before_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{major_dependent_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    let major_dependent_before_count = major_dependent_before_publish["rows"]
        .as_array()
        .expect("major dependent rows")
        .len();

    let draft_payload = dataset_revision_payload(
        "Revision Lifecycle Dataset Draft",
        "revision-lifecycle-dataset",
        form_id,
        form_version_id,
        visibility_node_id,
        json!([{
            "key": first_key,
            "label": first_label,
            "source_alias": "form_a",
            "source_field_key": first_key,
            "position": 0
        }, {
            "key": second_key,
            "label": second_label,
            "source_alias": "form_a",
            "source_field_key": second_key,
            "position": 1
        }]),
    );
    let draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/draft-revision"),
            &admin_token,
            Some(draft_payload),
        ),
    )
    .await;
    assert_eq!(draft["status"], "draft");
    let draft_revision_id = draft["revision_id"]
        .as_str()
        .expect("draft revision id")
        .to_string();
    assert_ne!(draft_revision_id, initial_revision_id);

    let detail_after_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        detail_after_draft["current_revision_id"],
        initial_revision_id
    );
    assert_eq!(detail_after_draft["name"], "Revision Lifecycle Dataset");
    assert_dataset_fields(&detail_after_draft, &[first_key], &[second_key]);
    let table_after_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_table_values(&table_after_draft, &[first_key], &[second_key]);

    let draft_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{draft_revision_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(draft_detail["status"], "draft");
    assert_eq!(
        draft_detail["metadata"]["name"],
        "Revision Lifecycle Dataset Draft"
    );
    assert_eq!(draft_detail["materialized_table"], Value::Null);
    assert_revision_fields(&draft_detail, &[first_key, second_key]);
    assert_eq!(draft_detail["dependencies"]["dataset_count"], 2);
    assert!(
        draft_detail["dependency_impacts"]
            .as_array()
            .expect("dependency impacts")
            .iter()
            .any(|impact| { impact["kind"] == "dataset" && impact["id"] == dependent_dataset_id })
    );
    assert!(
        draft_detail["dependency_impacts"]
            .as_array()
            .expect("dependency impacts")
            .iter()
            .any(|impact| {
                impact["kind"] == "dataset"
                    && impact["binding_mode"] == "major_line"
                    && impact["pinned_version_major"] == 1
            })
    );
    assert!(
        draft_detail["compatibility_findings"]
            .as_array()
            .expect("compatibility findings")
            .iter()
            .any(|finding| {
                finding["code"] == "added_output_field" && finding["field_key"] == second_key
            })
    );
    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{initial_revision_id}"),
            &operator_token,
            None,
        ),
    )
    .await;
    let operator_draft_status = request_status(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{draft_revision_id}"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(operator_draft_status, StatusCode::FORBIDDEN);
    let operator_publish_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/revisions/{draft_revision_id}/publish"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_eq!(operator_publish_status, StatusCode::FORBIDDEN);

    let scoped_manager_token = create_scoped_dataset_manager_token(
        app.clone(),
        &admin_token,
        "scoped-dataset-manager@tessara.local",
        "Scoped Dataset Manager",
        "tessara-dev-scoped-manager",
        visibility_node_id,
    )
    .await;
    let scoped_manager_draft_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{draft_revision_id}"),
            &scoped_manager_token,
            None,
        ),
    )
    .await;
    assert_eq!(scoped_manager_draft_detail["status"], "draft");

    let out_of_scope_manager_token = create_scoped_dataset_manager_token(
        app.clone(),
        &admin_token,
        "out-of-scope-dataset-manager@tessara.local",
        "Out Of Scope Dataset Manager",
        "tessara-dev-out-of-scope-manager",
        seed["activity_node_id"]
            .as_str()
            .expect("seed activity node id"),
    )
    .await;
    let out_of_scope_draft_status = request_status(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{draft_revision_id}"),
            &out_of_scope_manager_token,
            None,
        ),
    )
    .await;
    assert_eq!(out_of_scope_draft_status, StatusCode::FORBIDDEN);
    let out_of_scope_publish_status = request_status(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/revisions/{draft_revision_id}/publish"),
            &out_of_scope_manager_token,
            None,
        ),
    )
    .await;
    assert_eq!(out_of_scope_publish_status, StatusCode::FORBIDDEN);

    let history_after_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_revision_statuses(
        &history_after_draft,
        &[
            (&initial_revision_id, "published", true),
            (&draft_revision_id, "draft", false),
        ],
    );
    let operator_history_after_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions"),
            &operator_token,
            None,
        ),
    )
    .await;
    assert_revision_statuses(
        &operator_history_after_draft,
        &[(&initial_revision_id, "published", true)],
    );
    assert!(
        operator_history_after_draft
            .as_array()
            .expect("operator revision history")
            .iter()
            .all(|revision| revision["id"] != draft_revision_id)
    );
    let scoped_manager_history_after_draft = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions"),
            &scoped_manager_token,
            None,
        ),
    )
    .await;
    assert_revision_statuses(
        &scoped_manager_history_after_draft,
        &[
            (&initial_revision_id, "published", true),
            (&draft_revision_id, "draft", false),
        ],
    );

    let published = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/revisions/{draft_revision_id}/publish"),
            &scoped_manager_token,
            None,
        ),
    )
    .await;
    assert_eq!(published["status"], "published");
    assert_eq!(published["revision_id"], draft_revision_id);
    assert_eq!(published["superseded_revision_id"], initial_revision_id);

    let detail_after_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        detail_after_publish["current_revision_id"],
        draft_revision_id
    );
    assert_eq!(
        detail_after_publish["name"],
        "Revision Lifecycle Dataset Draft"
    );
    assert_dataset_fields(&detail_after_publish, &[first_key, second_key], &[]);
    let table_after_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_table_values(&table_after_publish, &[first_key, second_key], &[]);
    let exact_dependent_after_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dependent_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        exact_dependent_after_publish["rows"]
            .as_array()
            .expect("exact dependent rows")
            .len(),
        major_dependent_before_count,
        "exact revision consumers should remain pinned to the original materialized revision"
    );
    let major_dependent_after_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{major_dependent_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        major_dependent_after_publish["rows"]
            .as_array()
            .expect("major dependent rows after publish")
            .len(),
        major_dependent_before_count * 2,
        "major-line consumers should be rematerialized from every revision in the selected major"
    );
    let component_table_after_major_one_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{component_slug}/table?page_size=200"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        component_table_after_major_one_publish["dataset_version_major"],
        1
    );
    assert_eq!(
        component_table_after_major_one_publish["rows"]
            .as_array()
            .expect("component rows after major one publish")
            .len(),
        initial_major_line_row_count * 2,
        "component bound to Dataset v1 should include compatible v1 minor/patch rows"
    );
    assert_major_line_null_fills_added_field(dataset_id, 1, second_key).await;

    let history_after_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_revision_statuses(
        &history_after_publish,
        &[
            (&initial_revision_id, "superseded", false),
            (&draft_revision_id, "published", true),
        ],
    );
    let superseded_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{dataset_id}/revisions/{initial_revision_id}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        superseded_detail["dependency_impacts"]
            .as_array()
            .expect("superseded dependency impacts")
            .iter()
            .any(|impact| {
                impact["binding_mode"] == "exact_revision" && impact["id"] == dependent_dataset_id
            })
    );
    assert!(
        superseded_detail["dependency_impacts"]
            .as_array()
            .expect("superseded dependency impacts")
            .iter()
            .any(|impact| {
                impact["binding_mode"] == "major_line"
                    && impact["pinned_version_major"] == 1
                    && impact["pinned_revision_id"] == Value::Null
                    && impact["id"] == major_dependent_dataset_id
            })
    );

    let mut major_two_payload = dataset_revision_payload(
        "Revision Lifecycle Dataset V2",
        "revision-lifecycle-dataset",
        form_id,
        form_version_id,
        visibility_node_id,
        json!([{
            "key": first_key,
            "label": first_label,
            "source_alias": "form_a",
            "source_field_key": first_key,
            "position": 0
        }, {
            "key": second_key,
            "label": second_label,
            "source_alias": "form_a",
            "source_field_key": second_key,
            "position": 1
        }]),
    );
    major_two_payload["force_new_major_version"] = json!(true);
    let major_two_draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/draft-revision"),
            &admin_token,
            Some(major_two_payload),
        ),
    )
    .await;
    let major_two_revision_id = major_two_draft["revision_id"]
        .as_str()
        .expect("major two draft revision id");
    let major_two_publish = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{dataset_id}/revisions/{major_two_revision_id}/publish"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(major_two_publish["semantic_version"], "v2.0.0");
    let major_dependent_after_new_major = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/datasets/{major_dependent_dataset_id}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        major_dependent_after_new_major["rows"]
            .as_array()
            .expect("major dependent rows after new major")
            .len(),
        major_dependent_before_count * 2,
        "Version 1 consumers should remain on the Version 1 major-line table after Version 2 publishes"
    );
    let component_table_after_major_two_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{component_slug}/table?page_size=200"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(
        component_table_after_major_two_publish["dataset_version_major"],
        1
    );
    assert_eq!(
        component_table_after_major_two_publish["rows"]
            .as_array()
            .expect("component rows after major two publish")
            .len(),
        initial_major_line_row_count * 2,
        "component bound to Dataset v1 should not include Dataset v2 rows"
    );
    let mut major_dependent_draft_payload = major_dependent_payload;
    major_dependent_draft_payload["name"] =
        json!("Revision Lifecycle Major Dependent Dataset Draft");
    let major_dependent_draft = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!("/api/admin/datasets/{major_dependent_dataset_id}/draft-revision"),
            &admin_token,
            Some(major_dependent_draft_payload),
        ),
    )
    .await;
    let major_dependent_draft_revision_id = major_dependent_draft["revision_id"]
        .as_str()
        .expect("major dependent draft revision id");
    let major_dependent_publish = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!(
                "/api/admin/datasets/{major_dependent_dataset_id}/revisions/{major_dependent_draft_revision_id}/publish"
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(major_dependent_publish["status"], "published");

    let concurrent_publish_slug = "revision-lifecycle-concurrent-publish-component";
    let concurrent_publish_component = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Revision Lifecycle Concurrent Publish Component",
                "slug": concurrent_publish_slug,
                "description": "Component publish concurrency invariant coverage.",
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key]
                    },
                }
            })),
        ),
    )
    .await;
    let concurrent_publish_component_id = concurrent_publish_component["id"]
        .as_str()
        .expect("concurrent publish component id");
    let concurrent_publish_component_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{concurrent_publish_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let concurrent_publish_version_id = concurrent_publish_component["current_version"]["id"]
        .as_str()
        .or_else(|| {
            concurrent_publish_component_detail["versions"]
                .as_array()
                .and_then(|versions| versions.first())
                .and_then(|version| version["id"].as_str())
        })
        .expect("concurrent publish component version id");
    let (first_publish_attempt, second_publish_attempt) = tokio::join!(
        request_status_and_json(
            app.clone(),
            authorized_request(
                "POST",
                &format!(
                    "/api/admin/components/{concurrent_publish_component_id}/versions/{concurrent_publish_version_id}/publish"
                ),
                &admin_token,
                None,
            ),
        ),
        request_status_and_json(
            app.clone(),
            authorized_request(
                "POST",
                &format!(
                    "/api/admin/components/{concurrent_publish_component_id}/versions/{concurrent_publish_version_id}/publish"
                ),
                &admin_token,
                None,
            ),
        )
    );
    assert!(
        first_publish_attempt.0.is_success() || second_publish_attempt.0.is_success(),
        "at least one concurrent publish request should publish the draft"
    );
    let component_after_concurrent_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{concurrent_publish_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let versions_after_concurrent_publish = component_after_concurrent_publish["versions"]
        .as_array()
        .expect("component versions after concurrent publish");
    assert_eq!(
        versions_after_concurrent_publish
            .iter()
            .filter(|version| version["status"] == "published")
            .count(),
        1,
        "concurrent publish attempts should leave exactly one published component version"
    );
    assert_eq!(
        versions_after_concurrent_publish
            .iter()
            .filter(|version| version["status"] == "draft")
            .count(),
        0,
        "concurrent publish attempts should consume the working draft"
    );

    let publish_update_race_slug = "revision-lifecycle-publish-update-race-component";
    let publish_update_race_component = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Revision Lifecycle Publish Update Race Component",
                "slug": publish_update_race_slug,
                "description": "Component publish/update race invariant coverage.",
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key]
                    },
                }
            })),
        ),
    )
    .await;
    let publish_update_race_component_id = publish_update_race_component["id"]
        .as_str()
        .expect("publish/update race component id");
    let publish_update_race_component_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{publish_update_race_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let publish_update_race_version_id = publish_update_race_component["current_version"]["id"]
        .as_str()
        .or_else(|| {
            publish_update_race_component_detail["versions"]
                .as_array()
                .and_then(|versions| versions.first())
                .and_then(|version| version["id"].as_str())
        })
        .expect("publish/update race component version id");
    let (publish_race_attempt, update_race_attempt) = tokio::join!(
        request_status_and_json(
            app.clone(),
            authorized_request(
                "POST",
                &format!(
                    "/api/admin/components/{publish_update_race_component_id}/versions/{publish_update_race_version_id}/publish"
                ),
                &admin_token,
                None,
            ),
        ),
        request_status_and_json(
            app.clone(),
            authorized_request(
                "PATCH",
                &format!(
                    "/api/admin/components/{publish_update_race_component_id}/versions/{publish_update_race_version_id}"
                ),
                &admin_token,
                Some(json!({
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key],
                        "page_size": 25
                    },
                })),
            ),
        )
    );
    assert!(
        publish_race_attempt.0.is_success(),
        "publish should eventually consume the draft in a publish/update race: {:?}",
        publish_race_attempt.1
    );
    assert!(
        update_race_attempt.0.is_success() || update_race_attempt.0 == StatusCode::BAD_REQUEST,
        "update should either win before publish or fail deterministically after publish: {:?}",
        update_race_attempt.1
    );
    let component_after_publish_update_race = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{publish_update_race_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let versions_after_publish_update_race = component_after_publish_update_race["versions"]
        .as_array()
        .expect("component versions after publish/update race");
    assert_eq!(
        versions_after_publish_update_race
            .iter()
            .filter(|version| version["status"] == "published")
            .count(),
        1,
        "publish/update race should leave exactly one published component version"
    );
    assert_eq!(
        versions_after_publish_update_race
            .iter()
            .filter(|version| version["status"] == "draft")
            .count(),
        0,
        "publish/update race should not leave a mutable draft behind"
    );

    set_major_line_materialization_status(dataset_id, 1, "failed").await;
    let failed_component_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{component_slug}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(failed_component_table["materialization_state"], "failed");
    assert!(
        failed_component_table["rows"]
            .as_array()
            .expect("failed component rows")
            .is_empty(),
        "failed major-line materialization should render as a stable failed state"
    );

    let pending_component_slug = "revision-lifecycle-pending-component-table";
    let pending_component = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/components",
            &admin_token,
            Some(json!({
                "name": "Revision Lifecycle Pending Component Table",
                "slug": pending_component_slug,
                "description": "Component publish coverage when Dataset major-line materialization is missing.",
                "version": {
                    "dataset_id": dataset_id,
                    "dataset_version_major": 1,
                    "component_type": "table",
                    "config": {
                        "visible_columns": [first_key]
                    },
                }
            })),
        ),
    )
    .await;
    let pending_component_id = pending_component["id"]
        .as_str()
        .expect("pending materialization component id");
    let pending_component_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{pending_component_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    let pending_component_version_id = pending_component["current_version"]["id"]
        .as_str()
        .or_else(|| {
            pending_component_detail["versions"]
                .as_array()
                .and_then(|versions| versions.first())
                .and_then(|version| version["id"].as_str())
        })
        .expect("pending materialization component version id");

    remove_major_line_materialization(dataset_id, 1).await;
    let pending_publish = request_json(
        app.clone(),
        authorized_request(
            "POST",
            &format!(
                "/api/admin/components/{pending_component_id}/versions/{pending_component_version_id}/publish"
            ),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(pending_publish["id"], pending_component_version_id);
    let pending_component_after_publish = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/components/{pending_component_slug}"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert!(
        pending_component_after_publish["versions"]
            .as_array()
            .expect("pending materialization component versions")
            .iter()
            .any(|version| version["id"] == pending_component_version_id
                && version["status"] == "published"),
        "pending materialization publish should mark the draft version published"
    );
    let pending_component_table = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/components/{pending_component_slug}/table"),
            &admin_token,
            None,
        ),
    )
    .await;
    assert_eq!(pending_component_table["materialization_state"], "pending");
    assert!(
        pending_component_table["rows"]
            .as_array()
            .expect("pending component rows")
            .is_empty(),
        "missing major-line materialization should render as pending empty rows, not a publish-time validation failure"
    );
}

#[tokio::test]
async fn admin_dataset_query_designer_materializes_generated_sql() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let app = test_app().await;
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
                "operator": "is_not_null",
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
    assert!(generated_sql.contains(&format!("\"{field_key}\" IS NOT NULL")));
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
        "is_not_null"
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
                "input_field_key": format!("union_1__{field_key}"),
                "position": 0
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
            ]), 1)
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

async fn test_app() -> axum::Router {
    LazyLock::force(&TEST_TRACING);
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL is required; database integration tests must never skip");
    assert!(
        !database_url.trim().is_empty(),
        "TEST_DATABASE_URL is required and must not be empty"
    );
    let reset_pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    let database_name: String = sqlx::query_scalar("SELECT current_database()")
        .fetch_one(&reset_pool)
        .await
        .expect("current database should be readable");
    assert!(
        is_disposable_database_name(&database_name),
        "TEST_DATABASE_URL must point at a database with a token-bounded disposable name marker ({}); got '{database_name}'",
        DISPOSABLE_DATABASE_NAME_TOKENS.join(", ")
    );
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
    router(db::AppState { pool, config })
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

async fn create_scoped_dataset_manager_token(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    display_name: &str,
    password: &str,
    scope_node_id: &str,
) -> String {
    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", admin_token, None),
    )
    .await;
    let datasets_manage_capability_id = capabilities
        .as_array()
        .expect("capability list")
        .iter()
        .find(|capability| capability["key"] == "datasets:manage")
        .and_then(|capability| capability["id"].as_str())
        .expect("datasets:manage capability id");
    let forms_read_capability_id = capabilities
        .as_array()
        .expect("capability list")
        .iter()
        .find(|capability| capability["key"] == "forms:read")
        .and_then(|capability| capability["id"].as_str())
        .expect("forms:read capability id");

    let role = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            admin_token,
            Some(json!({
                "name": format!("{display_name} Role"),
                "capability_ids": [datasets_manage_capability_id, forms_read_capability_id]
            })),
        ),
    )
    .await;
    let role_id = role["id"].as_str().expect("created scoped role id");

    let user = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/users",
            admin_token,
            Some(json!({
                "email": email,
                "display_name": display_name,
                "password": password,
                "is_active": true,
                "role_ids": [role_id]
            })),
        ),
    )
    .await;
    let account_id = user["id"].as_str().expect("created scoped user id");

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{account_id}/access"),
            admin_token,
            Some(json!({
                "scope_node_ids": [scope_node_id],
                "delegate_account_ids": []
            })),
        ),
    )
    .await;

    login_token_for(app, email, password).await
}

async fn assert_major_line_null_fills_added_field(
    dataset_id: &str,
    version_major: i32,
    field_key: &str,
) {
    let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL should be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    let materialization = sqlx::query(
        r#"
        SELECT materialized_schema, materialized_table
        FROM dataset_major_materializations
        WHERE dataset_id = $1
          AND version_major = $2
          AND rebuild_status = 'ready'
        "#,
    )
    .bind(Uuid::parse_str(dataset_id).expect("dataset id uuid"))
    .bind(version_major)
    .fetch_one(&pool)
    .await
    .expect("major-line materialization");
    let schema: String = materialization
        .try_get("materialized_schema")
        .expect("materialized schema");
    let table: String = materialization
        .try_get("materialized_table")
        .expect("materialized table");
    let row = sqlx::query(&format!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE {field} IS NULL)::bigint AS null_count,
            COUNT(*) FILTER (WHERE {field} IS NOT NULL)::bigint AS populated_count
        FROM {schema}.{table}
        "#,
        schema = quote_test_identifier(&schema),
        table = quote_test_identifier(&table),
        field = quote_test_identifier(field_key),
    ))
    .fetch_one(&pool)
    .await
    .expect("major-line null-fill counts");
    let null_count: i64 = row.try_get("null_count").expect("null count");
    let populated_count: i64 = row.try_get("populated_count").expect("populated count");
    pool.close().await;
    assert!(
        null_count > 0,
        "older rows in the major-line materialization should NULL-fill fields added later"
    );
    assert!(
        populated_count > 0,
        "newer rows in the major-line materialization should populate fields added in that revision"
    );
}

async fn set_major_line_materialization_status(
    dataset_id: &str,
    version_major: i32,
    rebuild_status: &str,
) {
    let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL should be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    let result = sqlx::query(
        r#"
        UPDATE dataset_major_materializations
        SET rebuild_status = $3
        WHERE dataset_id = $1
          AND version_major = $2
        "#,
    )
    .bind(Uuid::parse_str(dataset_id).expect("dataset id uuid"))
    .bind(version_major)
    .bind(rebuild_status)
    .execute(&pool)
    .await
    .expect("update major-line materialization status");
    pool.close().await;
    assert_eq!(
        result.rows_affected(),
        1,
        "expected one major-line materialization status update"
    );
}

async fn remove_major_line_materialization(dataset_id: &str, version_major: i32) {
    let database_url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL should be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("connect test database");
    sqlx::query(
        r#"
        DELETE FROM dataset_major_materializations
        WHERE dataset_id = $1
          AND version_major = $2
        "#,
    )
    .bind(Uuid::parse_str(dataset_id).expect("dataset id uuid"))
    .bind(version_major)
    .execute(&pool)
    .await
    .expect("remove major-line materialization");
    pool.close().await;
}

fn quote_test_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
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

fn dataset_revision_payload(
    name: &str,
    slug: &str,
    form_id: &str,
    form_version_id: &str,
    visibility_node_id: &str,
    projection_fields: Value,
) -> Value {
    json!({
        "name": name,
        "slug": slug,
        "grain": "submission",
        "visibility_node_ids": [visibility_node_id],
        "initial_source": {
            "kind": "form",
            "alias": "form_a",
            "form_id": form_id,
            "form_version_id": form_version_id
        },
        "operations": [
            projection_operation(projection_fields, 0)
        ],
        "restriction_policy": null
    })
}

fn assert_dataset_fields(detail: &Value, expected: &[&str], absent: &[&str]) {
    let fields = detail["output_fields"]
        .as_array()
        .expect("dataset detail should include output fields");
    for key in expected {
        assert!(
            fields.iter().any(|field| field["key"] == *key),
            "dataset detail should include output field {key}"
        );
    }
    for key in absent {
        assert!(
            fields.iter().all(|field| field["key"] != *key),
            "dataset detail should not include output field {key}"
        );
    }
}

fn assert_revision_fields(detail: &Value, expected: &[&str]) {
    let fields = detail["output_fields"]
        .as_array()
        .expect("revision detail should include output fields");
    for key in expected {
        assert!(
            fields.iter().any(|field| field["key"] == *key),
            "revision detail should include output field {key}"
        );
    }
}

fn assert_table_values(table: &Value, expected: &[&str], absent: &[&str]) {
    let rows = table["rows"]
        .as_array()
        .expect("dataset table should include rows");
    assert!(
        !rows.is_empty(),
        "dataset table should include preview rows"
    );
    for row in rows {
        let values = row["values"]
            .as_object()
            .expect("dataset table row should include values");
        for key in expected {
            assert!(
                values.contains_key(*key),
                "dataset table row should include value {key}"
            );
        }
        for key in absent {
            assert!(
                !values.contains_key(*key),
                "dataset table row should not include value {key}"
            );
        }
    }
}

fn assert_revision_statuses(history: &Value, expected: &[(&str, &str, bool)]) {
    let revisions = history
        .as_array()
        .expect("revision history should be an array");
    for (revision_id, status, is_current) in expected {
        let revision = revisions
            .iter()
            .find(|revision| revision["id"] == *revision_id)
            .unwrap_or_else(|| panic!("revision history should include {revision_id}"));
        assert_eq!(revision["status"], *status);
        assert_eq!(revision["is_current"], *is_current);
    }
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
