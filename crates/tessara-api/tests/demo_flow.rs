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

    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &token, None),
    )
    .await;
    assert_eq!(seed["analytics_values"], 1);

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
            .any(|report| report["id"] == report_id)
    );

    let dashboard_id = seed["dashboard_id"]
        .as_str()
        .expect("seed response should contain dashboard id");
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
    let dashboards = request_json(
        app,
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

    let version_one_id = create_form_version(app.clone(), &token, form_id, "v1").await;
    let section_one_id = create_form_section(app.clone(), &token, version_one_id, "Main").await;
    create_number_field(
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
            "POST",
            &format!("/api/admin/form-versions/{version_one_id}/publish"),
            &token,
            None,
        ),
    )
    .await;

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
    assert_eq!(version_two["status"], "published");
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
            "POST",
            "/api/admin/nodes",
            &token,
            Some(json!({
                "node_type_id": node_type_id,
                "parent_node_id": null,
                "name": "Pilot Organization",
                "metadata": {}
            })),
        ),
    )
    .await;

    let required_metadata = request_status_and_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/node-metadata-fields",
            &token,
            Some(json!({
                "node_type_id": node_type_id,
                "key": "region",
                "label": "Region",
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
    let valid_chart = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/charts",
            &token,
            Some(json!({
                "name": "Participants Chart",
                "report_id": id_from(&valid_report),
                "chart_type": "table"
            })),
        ),
    )
    .await;
    let missing_component_dashboard = request_status_and_json(
        app,
        authorized_request(
            "POST",
            &format!("/api/admin/dashboards/{}/components", Uuid::new_v4()),
            &token,
            Some(json!({
                "chart_id": id_from(&valid_chart),
                "position": 0,
                "config": {}
            })),
        ),
    )
    .await;
    assert_eq!(missing_component_dashboard.0, StatusCode::NOT_FOUND);
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
