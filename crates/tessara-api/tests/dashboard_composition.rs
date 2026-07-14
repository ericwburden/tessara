#[allow(dead_code)]
mod support;

use std::{collections::BTreeSet, time::Duration};

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use support::{
    TEST_DATABASE_LOCK, authorized_request, login_token, login_token_for, request_json,
    request_status_and_json, test_app,
};
use uuid::Uuid;

#[tokio::test]
async fn composition_reconcile_preserves_ids_and_rejects_capacity_stale_and_invalid_layouts() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let dashboard_id = seed["dashboard_id"].as_str().expect("dashboard id");
    let composition_uri = format!("/api/admin/dashboards/{dashboard_id}/composition");
    let initial = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    let initial_placements = initial["dashboard"]["placements"]
        .as_array()
        .expect("initial placements");
    assert_eq!(initial_placements.len(), 9);
    let initial_ids = placement_ids(initial_placements);
    let component_version_id = initial["available_component_versions"][0]["component_version_id"]
        .as_str()
        .expect("placeable version");

    let mut add_commands = retain_commands(initial_placements);
    add_commands.push(json!({
        "operation": "bind",
        "client_key": "new-placement",
        "component_version_id": component_version_id,
        "geometry": {
            "grid_row": 21,
            "grid_column": 1,
            "grid_width": 6,
            "grid_height": 4
        },
        "title": "Additional placement"
    }));
    let added = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({ "commands": add_commands })),
        ),
    )
    .await;
    assert_eq!(added["dashboard"]["placement_count"], 10);
    assert_eq!(
        added["new_placement_ids"]
            .as_array()
            .expect("new id mappings")
            .len(),
        1
    );
    let added_placements = added["dashboard"]["placements"]
        .as_array()
        .expect("added placements");
    assert!(initial_ids.is_subset(&placement_ids(added_placements)));

    let (stale_status, stale_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({ "commands": retain_commands(initial_placements) })),
        ),
    )
    .await;
    assert_eq!(stale_status, StatusCode::CONFLICT);
    assert_eq!(stale_body["code"], "dashboard_composition_stale");

    let mut invalid_commands = retain_commands(added_placements);
    invalid_commands[0]["geometry"]["grid_row"] = json!(240);
    invalid_commands[0]["geometry"]["grid_height"] = json!(2);
    let (invalid_status, invalid_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({ "commands": invalid_commands })),
        ),
    )
    .await;
    assert_eq!(invalid_status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_body["code"], "dashboard_layout_invalid_geometry");

    let mut negative_commands = retain_commands(added_placements);
    negative_commands[0]["geometry"]["grid_row"] = json!(-1);
    let (negative_status, negative_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({ "commands": negative_commands })),
        ),
    )
    .await;
    assert_eq!(negative_status, StatusCode::BAD_REQUEST);
    assert_eq!(negative_body["code"], "dashboard_layout_invalid_geometry");

    let mut over_capacity_commands = retain_commands(added_placements);
    for index in 0..231 {
        over_capacity_commands.push(json!({
            "operation": "bind",
            "client_key": format!("capacity-{index}"),
            "component_version_id": component_version_id,
            "geometry": {
                "grid_row": 20,
                "grid_column": 1,
                "grid_width": 1,
                "grid_height": 1
            }
        }));
    }
    let (capacity_status, capacity_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({ "commands": over_capacity_commands })),
        ),
    )
    .await;
    assert_eq!(capacity_status, StatusCode::BAD_REQUEST);
    assert_eq!(capacity_body["code"], "dashboard_placement_limit_exceeded");

    let after_failures = request_json(
        app,
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert_eq!(after_failures["dashboard"]["placement_count"], 10);
    assert_eq!(
        placement_ids(
            after_failures["dashboard"]["placements"]
                .as_array()
                .expect("placements after failures")
        ),
        placement_ids(added_placements)
    );
}

#[tokio::test]
async fn table_placements_enforce_six_by_four_minimum_without_fixing_their_size() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let seeded_dashboard_id = seed["dashboard_id"].as_str().expect("seeded Dashboard id");
    let seeded_composition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/dashboards/{seeded_dashboard_id}/composition"),
            &admin_token,
            None,
        ),
    )
    .await;
    let table_option = seeded_composition["available_component_versions"]
        .as_array()
        .expect("placeable Component versions")
        .iter()
        .find(|option| option["component_slug"] == "demo-session-log-table")
        .expect("Session Log Table option");
    assert_eq!(table_option["component_type"], "table");
    assert_eq!(table_option["default_grid_width"], 6);
    assert_eq!(table_option["default_grid_height"], 4);
    let table_version_id = table_option["component_version_id"]
        .as_str()
        .expect("Table version id");
    let visibility_node_ids = visibility_node_ids(&seeded_composition);
    let (dashboard_id, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Table sizing contract",
        &visibility_node_ids,
    )
    .await;

    for (client_key, width, height) in [("too-narrow", 5, 4), ("too-short", 6, 3)] {
        let (status, body) = request_status_and_json(
            app.clone(),
            authorized_request(
                "PUT",
                &composition_uri,
                &admin_token,
                Some(json!({
                    "commands": [{
                        "operation": "bind",
                        "client_key": client_key,
                        "component_version_id": table_version_id,
                        "geometry": {
                            "grid_row": 1,
                            "grid_column": 1,
                            "grid_width": width,
                            "grid_height": height
                        }
                    }]
                })),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body["code"], "dashboard_layout_invalid_geometry");
    }

    let at_minimum = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "bind",
                    "client_key": "table",
                    "component_version_id": table_version_id,
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
    let placement_id = at_minimum["new_placement_ids"][0]["placement_id"]
        .as_str()
        .expect("Table placement id");

    let expanded = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "retain",
                    "placement_id": placement_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 12,
                        "grid_height": 240
                    }
                }]
            })),
        ),
    )
    .await;
    assert_eq!(expanded["dashboard"]["placements"][0]["grid_width"], 12);
    assert_eq!(expanded["dashboard"]["placements"][0]["grid_height"], 240);

    let persisted_expanded = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert_eq!(
        persisted_expanded["dashboard"]["placements"][0]["grid_height"],
        240
    );

    let pool = test_pool().await;
    sqlx::query("UPDATE dashboard_components SET config = $2 WHERE id = $1")
        .bind(placement_id.parse::<Uuid>().expect("Table placement UUID"))
        .bind(json!({
            "schema_version": 1,
            "grid_row": 1,
            "grid_column": 1,
            "grid_width": 5,
            "grid_height": 4
        }))
        .execute(&pool)
        .await
        .expect("install undersized stored Table fixture");
    let repair_state = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert_eq!(
        repair_state["dashboard"]["placements"][0]["config_state"],
        "needs_repair"
    );

    let repaired = request_json(
        app,
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "retain",
                    "placement_id": placement_id,
                    "repair": true,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 12,
                        "grid_height": 6
                    }
                }]
            })),
        ),
    )
    .await;
    assert_eq!(
        repaired["dashboard"]["placements"][0]["config_state"],
        "valid"
    );
    assert_eq!(
        repaired["dashboard"]["placements"][0]["placement_id"],
        placement_id
    );
    assert_eq!(repaired["dashboard"]["id"], dashboard_id);
}

#[tokio::test]
async fn total_counts_redaction_and_manage_only_editor_access_follow_the_contract() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let dashboard_id = seed["dashboard_id"].as_str().expect("dashboard id");

    let operator_token = login_token_for(
        app.clone(),
        "operator@tessara.local",
        "tessara-dev-operator",
    )
    .await;
    let scoped_dashboards = request_json(
        app.clone(),
        authorized_request("GET", "/api/dashboards", &operator_token, None),
    )
    .await;
    let scoped_dashboard = scoped_dashboards
        .as_array()
        .expect("scoped dashboard list")
        .iter()
        .find(|dashboard| dashboard["id"] == dashboard_id)
        .expect("scoped user should see demo Dashboard");
    assert_eq!(scoped_dashboard["placement_count"], 9);

    let reader_token = create_user_with_capabilities(
        app.clone(),
        &admin_token,
        "dashboard-reader-only@tessara.local",
        "Dashboard Reader Only",
        &["dashboards:read"],
    )
    .await;
    let reader_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/dashboards/{dashboard_id}"),
            &reader_token,
            None,
        ),
    )
    .await;
    assert_eq!(reader_detail["placement_count"], 9);
    assert_eq!(reader_detail["can_manage"], false);
    for placement in reader_detail["placements"]
        .as_array()
        .expect("redacted placements")
    {
        assert_eq!(placement["availability"], "unavailable");
        assert!(placement.get("component").is_none());
        assert!(placement.get("title").is_none());
        assert!(placement["grid_row"].as_u64().is_some());
    }

    let manager_token = create_user_with_capabilities(
        app.clone(),
        &admin_token,
        "dashboard-manager-only@tessara.local",
        "Dashboard Manager Only",
        &["dashboards:manage"],
    )
    .await;
    let (directory_status, directory_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", "/api/dashboards", &manager_token, None),
    )
    .await;
    assert_eq!(directory_status, StatusCode::FORBIDDEN);
    assert_eq!(directory_body["code"], "forbidden");

    let composition_uri = format!("/api/admin/dashboards/{dashboard_id}/composition");
    let composition = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &manager_token, None),
    )
    .await;
    assert_eq!(composition["dashboard"]["can_manage"], true);
    let visibility_options = request_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/admin/dashboards/visibility-nodes",
            &manager_token,
            None,
        ),
    )
    .await;
    assert!(
        !visibility_options
            .as_array()
            .expect("manager visibility options")
            .is_empty()
    );
    assert_eq!(
        composition["available_component_versions"]
            .as_array()
            .expect("manager-only picker")
            .len(),
        0
    );
    let redacted_editor_placements = composition["dashboard"]["placements"]
        .as_array()
        .expect("manager-only placements");
    assert!(
        redacted_editor_placements
            .iter()
            .all(|placement| placement["availability"] == "unavailable")
    );
    let saved = request_json(
        app,
        authorized_request(
            "PUT",
            &composition_uri,
            &manager_token,
            Some(json!({
                "commands": retain_commands(redacted_editor_placements)
            })),
        ),
    )
    .await;
    assert_eq!(saved["dashboard"]["placement_count"], 9);
}

#[tokio::test]
async fn dashboard_manage_affordances_follow_each_dashboard_scope() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;

    let pool = test_pool().await;
    let leaf_nodes = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT nodes.id
        FROM nodes
        WHERE NOT EXISTS (
            SELECT 1 FROM nodes AS children WHERE children.parent_node_id = nodes.id
        )
        ORDER BY nodes.id
        LIMIT 2
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("leaf visibility nodes");
    assert_eq!(leaf_nodes.len(), 2, "demo data should provide two leaves");
    let managed_node = leaf_nodes[0].to_string();
    let read_only_node = leaf_nodes[1].to_string();
    let (managed_dashboard_id, managed_composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Scoped manager target",
        std::slice::from_ref(&managed_node),
    )
    .await;
    let (read_only_dashboard_id, read_only_composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Scoped reader target",
        std::slice::from_ref(&read_only_node),
    )
    .await;
    let token = create_user_with_global_read_and_scoped_manage(
        app.clone(),
        &admin_token,
        "dashboard-split-scope@tessara.local",
        &managed_node,
    )
    .await;

    let directory = request_json(
        app.clone(),
        authorized_request("GET", "/api/dashboards", &token, None),
    )
    .await;
    let rows = directory.as_array().expect("Dashboard directory");
    let managed = rows
        .iter()
        .find(|dashboard| dashboard["id"] == managed_dashboard_id)
        .expect("managed Dashboard should be readable");
    let read_only = rows
        .iter()
        .find(|dashboard| dashboard["id"] == read_only_dashboard_id)
        .expect("out-of-manage Dashboard should still be readable");
    assert_eq!(managed["can_manage"], true);
    assert_eq!(read_only["can_manage"], false);

    let managed_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/dashboards/{managed_dashboard_id}"),
            &token,
            None,
        ),
    )
    .await;
    let read_only_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/dashboards/{read_only_dashboard_id}"),
            &token,
            None,
        ),
    )
    .await;
    assert_eq!(managed_detail["can_manage"], true);
    assert_eq!(read_only_detail["can_manage"], false);

    let managed_editor = request_status_and_json(
        app.clone(),
        authorized_request("GET", &managed_composition_uri, &token, None),
    )
    .await;
    let read_only_editor = request_status_and_json(
        app,
        authorized_request("GET", &read_only_composition_uri, &token, None),
    )
    .await;
    assert_eq!(managed_editor.0, StatusCode::OK);
    assert_eq!(read_only_editor.0, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn malformed_v1_requires_explicit_repair_and_preserves_raw_config_on_ordinary_save() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let seeded_dashboard_id = seed["dashboard_id"].as_str().expect("dashboard id");
    let seeded_composition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/dashboards/{seeded_dashboard_id}/composition"),
            &admin_token,
            None,
        ),
    )
    .await;
    let visibility_node_ids = seeded_composition["dashboard"]["visibility_nodes"]
        .as_array()
        .expect("seeded visibility nodes")
        .iter()
        .map(|node| {
            node["node_id"]
                .as_str()
                .expect("seeded visibility node")
                .to_string()
        })
        .collect::<Vec<_>>();
    let component_version_id =
        seeded_composition["available_component_versions"][0]["component_version_id"]
            .as_str()
            .expect("placeable version");

    let created = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/dashboards",
            &admin_token,
            Some(json!({
                "name": "Repair semantics",
                "description": "Malformed V1 integration coverage",
                "visibility_node_ids": visibility_node_ids
            })),
        ),
    )
    .await;
    let dashboard_id = created["id"].as_str().expect("new dashboard id");
    let composition_uri = format!("/api/admin/dashboards/{dashboard_id}/composition");
    let added = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "bind",
                    "client_key": "repair-target",
                    "component_version_id": component_version_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 6,
                        "grid_height": 4
                    },
                    "title": "Original title"
                }]
            })),
        ),
    )
    .await;
    let placement_id = added["new_placement_ids"][0]["placement_id"]
        .as_str()
        .expect("new placement id");
    let placement_uuid = placement_id.parse::<uuid::Uuid>().expect("placement uuid");
    let raw_malformed = json!({
        "schema_version": 1,
        "title": "Malformed title",
        "grid_row": "not-an-integer",
        "grid_column": 1,
        "grid_width": 6,
        "grid_height": 2,
        "opaque_extension": {"must_survive": true}
    });
    let database_url = std::env::var("TEST_DATABASE_URL").expect("test database url");
    let pool = sqlx::PgPool::connect(&database_url)
        .await
        .expect("test database connection");
    sqlx::query("UPDATE dashboard_components SET config = $2 WHERE id = $1")
        .bind(placement_uuid)
        .bind(&raw_malformed)
        .execute(&pool)
        .await
        .expect("install malformed config");

    let manager_token = create_user_with_capabilities(
        app.clone(),
        &admin_token,
        "dashboard-repair-manager@tessara.local",
        "Dashboard Repair Manager",
        &["dashboards:manage"],
    )
    .await;
    let redacted = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &manager_token, None),
    )
    .await;
    let malformed = &redacted["dashboard"]["placements"][0];
    assert_eq!(malformed["config_state"], "needs_repair");
    assert_eq!(malformed["availability"], "unavailable");
    assert_eq!(malformed["grid_row"], 1);
    assert_eq!(malformed["grid_width"], 12);
    assert!(
        malformed["allowed_operations"]
            .as_array()
            .expect("allowed operations")
            .iter()
            .any(|operation| operation == "repair")
    );

    let retained = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &manager_token,
            Some(json!({
                "commands": [{
                    "operation": "retain",
                    "placement_id": placement_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 12,
                        "grid_height": 1
                    }
                }]
            })),
        ),
    )
    .await;
    assert_eq!(
        retained["dashboard"]["placements"][0]["config_state"],
        "needs_repair"
    );
    let after_retain: Value =
        sqlx::query_scalar("SELECT config FROM dashboard_components WHERE id = $1")
            .bind(placement_uuid)
            .fetch_one(&pool)
            .await
            .expect("retained raw config");
    assert_eq!(after_retain, raw_malformed);

    let (oracle_status, oracle_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &manager_token,
            Some(json!({
                "commands": [{
                    "operation": "bind",
                    "placement_id": placement_id,
                    "component_version_id": component_version_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 12,
                        "grid_height": 1
                    }
                }]
            })),
        ),
    )
    .await;
    assert_eq!(oracle_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        oracle_body["code"],
        "dashboard_component_version_unavailable"
    );

    let repaired = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &manager_token,
            Some(json!({
                "commands": [{
                    "operation": "retain",
                    "placement_id": placement_id,
                    "repair": true,
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
    assert_eq!(
        repaired["dashboard"]["placements"][0]["config_state"],
        "valid"
    );
    assert_eq!(
        repaired["dashboard"]["placements"][0]["availability"],
        "unavailable"
    );
    assert_eq!(
        repaired["dashboard"]["placements"][0]["placement_id"],
        placement_id
    );

    let canonical: Value =
        sqlx::query_scalar("SELECT config FROM dashboard_components WHERE id = $1")
            .bind(placement_uuid)
            .fetch_one(&pool)
            .await
            .expect("canonical repaired config");
    assert_eq!(canonical["schema_version"], 1);
    assert_eq!(canonical["title"], "Malformed title");
    assert_eq!(canonical["grid_width"], 6);
    assert_eq!(canonical["grid_height"], 4);
    assert!(canonical.get("opaque_extension").is_none());

    let admin_detail = request_json(
        app,
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert_eq!(
        admin_detail["dashboard"]["placements"][0]["title"],
        "Malformed title"
    );
}

#[tokio::test]
async fn concurrent_same_membership_saves_are_serialized_last_write_wins() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let (_, visibility_node_ids, component_version_id) =
        seed_composition_context(app.clone(), &admin_token).await;
    let (_, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Concurrent retained placement",
        &visibility_node_ids,
    )
    .await;
    let placement_id = add_single_placement(
        app.clone(),
        &admin_token,
        &composition_uri,
        &component_version_id,
    )
    .await;

    let save_at_row_one = authorized_request(
        "PUT",
        &composition_uri,
        &admin_token,
        Some(json!({
            "commands": [{
                "operation": "retain",
                "placement_id": placement_id,
                "geometry": {
                    "grid_row": 1,
                    "grid_column": 1,
                    "grid_width": 1,
                    "grid_height": 1
                }
            }]
        })),
    );
    let save_at_row_two = authorized_request(
        "PUT",
        &composition_uri,
        &admin_token,
        Some(json!({
            "commands": [{
                "operation": "retain",
                "placement_id": placement_id,
                "geometry": {
                    "grid_row": 2,
                    "grid_column": 1,
                    "grid_width": 1,
                    "grid_height": 1
                }
            }]
        })),
    );
    let (first, second) = tokio::join!(
        request_status_and_json(app.clone(), save_at_row_one),
        request_status_and_json(app.clone(), save_at_row_two)
    );
    assert_eq!(first.0, StatusCode::OK, "first save: {}", first.1);
    assert_eq!(second.0, StatusCode::OK, "second save: {}", second.1);
    assert_eq!(first.1["dashboard"]["placements"][0]["grid_row"], 1);
    assert_eq!(second.1["dashboard"]["placements"][0]["grid_row"], 2);

    let final_state = request_json(
        app,
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert!(matches!(
        final_state["dashboard"]["placements"][0]["grid_row"].as_i64(),
        Some(1 | 2)
    ));
}

#[tokio::test]
async fn concurrent_add_then_remove_detects_structural_staleness() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let (_, visibility_node_ids, component_version_id) =
        seed_composition_context(app.clone(), &admin_token).await;
    let (dashboard_id, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Concurrent membership",
        &visibility_node_ids,
    )
    .await;
    let placement_id = add_single_placement(
        app.clone(),
        &admin_token,
        &composition_uri,
        &component_version_id,
    )
    .await;

    let pool = test_pool().await;
    let mut blocker = pool.begin().await.expect("Dashboard lock transaction");
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM dashboards WHERE id = $1 FOR UPDATE")
        .bind(
            dashboard_id
                .parse::<Uuid>()
                .expect("concurrent Dashboard id"),
        )
        .fetch_one(&mut *blocker)
        .await
        .expect("lock concurrent Dashboard");

    let add_task = tokio::spawn(request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [
                    {
                        "operation": "retain",
                        "placement_id": placement_id,
                        "geometry": {
                            "grid_row": 1,
                            "grid_column": 1,
                            "grid_width": 1,
                            "grid_height": 1
                        }
                    },
                    {
                        "operation": "bind",
                        "client_key": "concurrent-add",
                        "component_version_id": component_version_id,
                        "geometry": {
                            "grid_row": 2,
                            "grid_column": 1,
                            "grid_width": 1,
                            "grid_height": 1
                        }
                    }
                ]
            })),
        ),
    ));
    tokio::time::sleep(Duration::from_millis(100)).await;
    let remove_task = tokio::spawn(request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "remove",
                    "placement_id": placement_id
                }]
            })),
        ),
    ));
    tokio::time::sleep(Duration::from_millis(100)).await;
    blocker.commit().await.expect("release Dashboard lock");

    let added = add_task.await.expect("add request task");
    let removed = remove_task.await.expect("remove request task");
    assert_eq!(added.0, StatusCode::OK, "add response: {}", added.1);
    assert_eq!(removed.0, StatusCode::CONFLICT, "remove: {}", removed.1);
    assert_eq!(removed.1["code"], "dashboard_composition_stale");
    let final_state = request_json(
        app,
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert_eq!(final_state["dashboard"]["placement_count"], 2);
}

#[tokio::test]
async fn metadata_scope_and_composition_share_one_lock_and_scope_contraction_rolls_back() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let (_, visibility_node_ids, component_version_id) =
        seed_composition_context(app.clone(), &admin_token).await;
    let pool = test_pool().await;
    let component_uuid = component_version_id
        .parse::<Uuid>()
        .expect("Component version UUID");
    let dataset_nodes = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT dataset_scope_nodes.node_id
        FROM component_versions
        JOIN dataset_scope_nodes
          ON dataset_scope_nodes.dataset_id = component_versions.dataset_id
        WHERE component_versions.id = $1
        ORDER BY dataset_scope_nodes.node_id
        "#,
    )
    .bind(component_uuid)
    .fetch_all(&pool)
    .await
    .expect("candidate Dataset scope");
    let excluded_node = dataset_nodes
        .first()
        .expect("candidate Dataset scope node")
        .to_string();
    let contracted_scope = visibility_node_ids
        .iter()
        .filter(|node_id| node_id.as_str() != excluded_node.as_str())
        .cloned()
        .collect::<Vec<_>>();
    assert!(!contracted_scope.is_empty());

    let (dashboard_id, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Concurrent scope",
        &visibility_node_ids,
    )
    .await;
    let composition_request = authorized_request(
        "PUT",
        &composition_uri,
        &admin_token,
        Some(json!({
            "commands": [{
                "operation": "bind",
                "client_key": "scope-race",
                "component_version_id": component_version_id,
                "geometry": {
                    "grid_row": 1,
                    "grid_column": 1,
                    "grid_width": 1,
                    "grid_height": 1
                }
            }]
        })),
    );
    let metadata_request = authorized_request(
        "PUT",
        &format!("/api/admin/dashboards/{dashboard_id}"),
        &admin_token,
        Some(json!({
            "name": "Concurrent scope changed",
            "description": "Only one transaction may win",
            "visibility_node_ids": contracted_scope.clone()
        })),
    );
    let (composition_result, metadata_result) = tokio::join!(
        request_status_and_json(app.clone(), composition_request),
        request_status_and_json(app.clone(), metadata_request)
    );
    assert!(
        (composition_result.0 == StatusCode::OK && metadata_result.0 == StatusCode::CONFLICT)
            || (composition_result.0 == StatusCode::CONFLICT
                && metadata_result.0 == StatusCode::OK),
        "composition={} {}, metadata={} {}",
        composition_result.0,
        composition_result.1,
        metadata_result.0,
        metadata_result.1
    );
    let concurrent_final = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    if composition_result.0 == StatusCode::OK {
        assert_eq!(concurrent_final["dashboard"]["placement_count"], 1);
        assert_eq!(concurrent_final["dashboard"]["name"], "Concurrent scope");
    } else {
        assert_eq!(concurrent_final["dashboard"]["placement_count"], 0);
        assert_eq!(
            concurrent_final["dashboard"]["name"],
            "Concurrent scope changed"
        );
    }

    let (rollback_dashboard_id, rollback_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Scope rollback",
        &visibility_node_ids,
    )
    .await;
    let rollback_placement_id = add_single_placement(
        app.clone(),
        &admin_token,
        &rollback_uri,
        &component_version_id,
    )
    .await;
    let (scope_status, scope_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/dashboards/{rollback_dashboard_id}"),
            &admin_token,
            Some(json!({
                "name": "Must roll back",
                "description": "Incompatible contraction",
                "visibility_node_ids": contracted_scope.clone()
            })),
        ),
    )
    .await;
    assert_eq!(scope_status, StatusCode::CONFLICT);
    assert_eq!(scope_body["code"], "dashboard_scope_incompatible");
    let rollback_final = request_json(
        app,
        authorized_request("GET", &rollback_uri, &admin_token, None),
    )
    .await;
    assert_eq!(rollback_final["dashboard"]["name"], "Scope rollback");
    assert_eq!(rollback_final["dashboard"]["placement_count"], 1);
    assert_eq!(
        rollback_final["dashboard"]["placements"][0]["placement_id"],
        rollback_placement_id
    );
    assert_eq!(
        visibility_node_id_set(&rollback_final),
        visibility_node_ids.iter().cloned().collect()
    );
}

#[tokio::test]
async fn candidate_errors_do_not_reveal_missing_draft_or_out_of_scope_versions() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let (seeded_dashboard_id, visibility_node_ids, _) =
        seed_composition_context(app.clone(), &admin_token).await;
    let seeded_composition = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/dashboards/{seeded_dashboard_id}/composition"),
            &admin_token,
            None,
        ),
    )
    .await;
    let component_version_id = seeded_composition["available_component_versions"]
        .as_array()
        .expect("placeable Component versions")
        .iter()
        .find(|option| option["component_slug"] == "demo-partner-profile-table")
        .and_then(|option| option["component_version_id"].as_str())
        .expect("placeable Partner Profile Table")
        .to_string();
    let (_, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Candidate non-leakage",
        &visibility_node_ids,
    )
    .await;
    let pool = test_pool().await;
    let published_version_id = component_version_id
        .parse::<Uuid>()
        .expect("published Component version id");
    let component_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO components (name, slug) VALUES ($1, $2) RETURNING id",
    )
    .bind("Candidate Draft Fixture")
    .bind(format!("candidate-draft-{}", Uuid::new_v4()))
    .fetch_one(&pool)
    .await
    .expect("draft fixture Component");
    let draft_version_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO component_versions (
            component_id,
            dataset_id,
            dataset_version_major,
            component_type,
            version_number,
            version_label,
            status,
            config
        )
        SELECT $1,
               dataset_id,
               dataset_version_major,
               component_type,
               1,
               'Draft candidate',
               'draft'::component_version_status,
               '{}'::jsonb
        FROM component_versions
        WHERE id = $2
        RETURNING id
        "#,
    )
    .bind(component_id)
    .bind(published_version_id)
    .fetch_one(&pool)
    .await
    .expect("draft Component version fixture");

    let mut hidden_errors = Vec::new();
    for (client_key, candidate_id) in [("missing", Uuid::new_v4()), ("draft", draft_version_id)] {
        let result = request_status_and_json(
            app.clone(),
            authorized_request(
                "PUT",
                &composition_uri,
                &admin_token,
                Some(json!({
                    "commands": [{
                        "operation": "bind",
                        "client_key": client_key,
                        "component_version_id": candidate_id,
                        "geometry": {
                            "grid_row": 1,
                            "grid_column": 1,
                            "grid_width": 1,
                            "grid_height": 1
                        }
                    }]
                })),
            ),
        )
        .await;
        assert_eq!(result.0, StatusCode::BAD_REQUEST);
        assert_eq!(result.1["code"], "dashboard_component_version_unavailable");
        hidden_errors.push(result.1["message"].clone());
    }

    let dataset_nodes = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT dataset_scope_nodes.node_id
        FROM component_versions
        JOIN dataset_scope_nodes
          ON dataset_scope_nodes.dataset_id = component_versions.dataset_id
        WHERE component_versions.id = $1
        ORDER BY dataset_scope_nodes.node_id
        "#,
    )
    .bind(published_version_id)
    .fetch_all(&pool)
    .await
    .expect("candidate Dataset scope");
    let visibility_uuids = visibility_node_ids
        .iter()
        .map(|node_id| node_id.parse::<Uuid>().expect("visibility node UUID"))
        .collect::<Vec<_>>();
    let outside_scope_node = sqlx::query_scalar::<_, Uuid>(
        r#"
        WITH RECURSIVE subtrees AS (
            SELECT nodes.id AS root_id, nodes.id
            FROM nodes
            WHERE nodes.id = ANY($1)
            UNION ALL
            SELECT subtrees.root_id, children.id
            FROM subtrees
            JOIN nodes AS children ON children.parent_node_id = subtrees.id
        )
        SELECT root_id
        FROM subtrees
        GROUP BY root_id
        HAVING NOT BOOL_OR(id = ANY($2))
        ORDER BY root_id
        LIMIT 1
        "#,
    )
    .bind(&visibility_uuids)
    .bind(&dataset_nodes)
    .fetch_one(&pool)
    .await
    .expect("scope with no candidate Dataset overlap");
    let outside_scope = vec![outside_scope_node.to_string()];
    let (_, scoped_composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Out of scope candidate",
        &outside_scope,
    )
    .await;
    let scoped_token = create_scoped_user_with_capabilities(
        app.clone(),
        &admin_token,
        "dashboard-candidate-scope@tessara.local",
        "Dashboard Candidate Scope",
        &["dashboards:manage", "components:read"],
        &outside_scope,
    )
    .await;
    let out_of_scope = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &scoped_composition_uri,
            &scoped_token,
            Some(json!({
                "commands": [{
                    "operation": "bind",
                    "client_key": "out-of-scope",
                    "component_version_id": published_version_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 1,
                        "grid_height": 1
                    }
                }]
            })),
        ),
    )
    .await;
    assert_eq!(out_of_scope.0, StatusCode::BAD_REQUEST);
    assert_eq!(
        out_of_scope.1["code"],
        "dashboard_component_version_unavailable"
    );
    hidden_errors.push(out_of_scope.1["message"].clone());
    assert!(hidden_errors.windows(2).all(|pair| pair[0] == pair[1]));
    let unchanged = request_json(
        app,
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    assert_eq!(unchanged["dashboard"]["placement_count"], 0);
}

#[tokio::test]
async fn future_schema_can_only_be_retained_unchanged_or_removed_and_preserves_raw_json() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let (_, visibility_node_ids, component_version_id) =
        seed_composition_context(app.clone(), &admin_token).await;
    let (_, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Future schema",
        &visibility_node_ids,
    )
    .await;
    let placement_id = add_single_placement(
        app.clone(),
        &admin_token,
        &composition_uri,
        &component_version_id,
    )
    .await;
    let placement_uuid = placement_id.parse::<Uuid>().expect("placement UUID");
    let pool = test_pool().await;
    let future_raw = json!({
        "schema_version": 9,
        "opaque": {
            "future_geometry": [300, -4, "unknown"],
            "must_survive": true
        }
    });
    sqlx::query("UPDATE dashboard_components SET config = $2 WHERE id = $1")
        .bind(placement_uuid)
        .bind(&future_raw)
        .execute(&pool)
        .await
        .expect("install future-schema config");

    let future = request_json(
        app.clone(),
        authorized_request("GET", &composition_uri, &admin_token, None),
    )
    .await;
    let placement = &future["dashboard"]["placements"][0];
    assert_eq!(placement["config_state"], "future_schema");
    assert_eq!(placement["availability"], "unavailable");
    assert_eq!(placement["grid_row"], 1);
    assert_eq!(placement["grid_width"], 12);

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "retain",
                    "placement_id": placement_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 12,
                        "grid_height": 1
                    }
                }]
            })),
        ),
    )
    .await;
    let retained_raw: Value =
        sqlx::query_scalar("SELECT config FROM dashboard_components WHERE id = $1")
            .bind(placement_uuid)
            .fetch_one(&pool)
            .await
            .expect("retained future config");
    assert_eq!(retained_raw, future_raw);

    let (move_status, move_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "retain",
                    "placement_id": placement_id,
                    "geometry": {
                        "grid_row": 2,
                        "grid_column": 1,
                        "grid_width": 12,
                        "grid_height": 1
                    }
                }]
            })),
        ),
    )
    .await;
    assert_eq!(move_status, StatusCode::BAD_REQUEST);
    assert_eq!(move_body["code"], "dashboard_layout_invalid_geometry");
    let after_failed_move: Value =
        sqlx::query_scalar("SELECT config FROM dashboard_components WHERE id = $1")
            .bind(placement_uuid)
            .fetch_one(&pool)
            .await
            .expect("future config after failed move");
    assert_eq!(after_failed_move, future_raw);

    let removed = request_json(
        app,
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({
                "commands": [{
                    "operation": "remove",
                    "placement_id": placement_id
                }]
            })),
        ),
    )
    .await;
    assert_eq!(removed["dashboard"]["placement_count"], 0);
    let still_exists: bool =
        sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM dashboard_components WHERE id = $1)")
            .bind(placement_uuid)
            .fetch_one(&pool)
            .await
            .expect("future placement existence");
    assert!(!still_exists);
}

#[tokio::test]
async fn exactly_240_placements_succeed_and_database_trigger_rejects_241st() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let (_, visibility_node_ids, component_version_id) =
        seed_composition_context(app.clone(), &admin_token).await;
    let (dashboard_id, composition_uri) = create_dashboard_fixture(
        app.clone(),
        &admin_token,
        "Exact placement capacity",
        &visibility_node_ids,
    )
    .await;
    let commands = (0..240)
        .map(|index| {
            json!({
                "operation": "bind",
                "client_key": format!("capacity-{index}"),
                "component_version_id": component_version_id,
                "geometry": {
                    "grid_row": index + 1,
                    "grid_column": 1,
                    "grid_width": 1,
                    "grid_height": 1
                }
            })
        })
        .collect::<Vec<_>>();
    let at_capacity = request_json(
        app,
        authorized_request(
            "PUT",
            &composition_uri,
            &admin_token,
            Some(json!({ "commands": commands })),
        ),
    )
    .await;
    assert_eq!(at_capacity["dashboard"]["placement_count"], 240);
    assert_eq!(
        at_capacity["new_placement_ids"]
            .as_array()
            .expect("capacity id mappings")
            .len(),
        240
    );

    let pool = test_pool().await;
    let trigger_error = sqlx::query(
        r#"
        INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
        VALUES ($1, $2, 240, $3)
        "#,
    )
    .bind(dashboard_id.parse::<Uuid>().expect("capacity Dashboard id"))
    .bind(
        component_version_id
            .parse::<Uuid>()
            .expect("capacity Component version id"),
    )
    .bind(json!({
        "schema_version": 1,
        "grid_row": 240,
        "grid_column": 2,
        "grid_width": 1,
        "grid_height": 1
    }))
    .execute(&pool)
    .await
    .expect_err("database trigger must reject placement 241");
    let database_error = trigger_error
        .as_database_error()
        .expect("capacity database error");
    assert_eq!(
        database_error.constraint(),
        Some("dashboard_components_capacity_chk")
    );
    let stored_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dashboard_components WHERE dashboard_id = $1")
            .bind(dashboard_id.parse::<Uuid>().expect("capacity Dashboard id"))
            .fetch_one(&pool)
            .await
            .expect("stored capacity count");
    assert_eq!(stored_count, 240);
}

#[tokio::test]
async fn placement_capacity_migration_preflight_aborts_without_mutation() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app,
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let dashboard_id = seed["dashboard_id"]
        .as_str()
        .expect("seeded Dashboard id")
        .parse::<Uuid>()
        .expect("seeded Dashboard UUID");
    let component_version_id = seed["component_version_id"]
        .as_str()
        .expect("seeded Component version id")
        .parse::<Uuid>()
        .expect("seeded Component version UUID");
    let pool = test_pool().await;
    let mut tx = pool.begin().await.expect("preflight fixture transaction");
    sqlx::query("DROP TRIGGER dashboard_components_capacity_trigger ON dashboard_components")
        .execute(&mut *tx)
        .await
        .expect("temporarily remove capacity trigger");
    sqlx::query(
        r#"
        INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
        SELECT $1, $2, 1000 + generated.position, '{}'::jsonb
        FROM generate_series(1, 232) AS generated(position)
        "#,
    )
    .bind(dashboard_id)
    .bind(component_version_id)
    .execute(&mut *tx)
    .await
    .expect("install over-cap preflight fixture");
    let before_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dashboard_components WHERE dashboard_id = $1")
            .bind(dashboard_id)
            .fetch_one(&mut *tx)
            .await
            .expect("over-cap fixture count");
    assert_eq!(before_count, 241);

    let migration_error = sqlx::raw_sql(include_str!(
        "../migrations/002_dashboard_placement_capacity.sql"
    ))
    .execute(&mut *tx)
    .await
    .expect_err("migration preflight must abort");
    let message = migration_error.to_string();
    assert!(message.contains("dashboard placement capacity preflight failed"));
    assert!(message.contains(&dashboard_id.to_string()));
    assert!(message.contains("241 placements"));
    tx.rollback()
        .await
        .expect("rollback preflight fixture and trigger drop");

    let restored_count: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM dashboard_components WHERE dashboard_id = $1")
            .bind(dashboard_id)
            .fetch_one(&pool)
            .await
            .expect("post-preflight Dashboard count");
    assert_eq!(restored_count, 9);
    let trigger_restored: bool = sqlx::query_scalar(
        "SELECT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'dashboard_components_capacity_trigger' AND NOT tgisinternal)",
    )
    .fetch_one(&pool)
    .await
    .expect("capacity trigger restoration");
    assert!(trigger_restored);
}

#[tokio::test]
async fn placement_capacity_migration_rejects_overlapping_valid_v1_without_mutation() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app,
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let dashboard_id = seed["dashboard_id"]
        .as_str()
        .expect("seeded Dashboard id")
        .parse::<Uuid>()
        .expect("seeded Dashboard UUID");
    let pool = test_pool().await;
    let mut tx = pool.begin().await.expect("overlap fixture transaction");
    let placement_ids = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT id
        FROM dashboard_components
        WHERE dashboard_id = $1
        ORDER BY position, id
        LIMIT 2
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(&mut *tx)
    .await
    .expect("overlap fixture placement ids");
    assert_eq!(placement_ids.len(), 2);
    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET config = '{
            "schema_version": 1,
            "grid_row": 1,
            "grid_column": 1,
            "grid_width": 12,
            "grid_height": 6
        }'::jsonb
        WHERE id = ANY($1)
        "#,
    )
    .bind(&placement_ids)
    .execute(&mut *tx)
    .await
    .expect("install overlapping valid V1 fixture");

    let migration_error = sqlx::raw_sql(include_str!(
        "../migrations/002_dashboard_placement_capacity.sql"
    ))
    .execute(&mut *tx)
    .await
    .expect_err("overlapping valid V1 geometry must abort migration");
    let message = migration_error.to_string();
    assert!(message.contains("overlapping valid V1 geometry"));
    assert!(message.contains(&dashboard_id.to_string()));
    tx.rollback()
        .await
        .expect("rollback overlapping V1 fixture");

    let canonical_v1_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM dashboard_components
        WHERE dashboard_id = $1
          AND config ->> 'schema_version' = '1'
        "#,
    )
    .bind(dashboard_id)
    .fetch_one(&pool)
    .await
    .expect("post-overlap-preflight canonical config count");
    assert_eq!(canonical_v1_count, 9);
}

#[tokio::test]
async fn placement_capacity_migration_rejects_fallback_exhaustion_without_mutation() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let Some(app) = test_app().await else { return };
    let admin_token = login_token(app.clone()).await;
    let seed = request_json(
        app,
        authorized_request("POST", "/api/demo/seed", &admin_token, None),
    )
    .await;
    let dashboard_id = seed["dashboard_id"]
        .as_str()
        .expect("seeded Dashboard id")
        .parse::<Uuid>()
        .expect("seeded Dashboard UUID");
    let pool = test_pool().await;
    let mut tx = pool
        .begin()
        .await
        .expect("fallback-exhaustion fixture transaction");
    let spanning_placement_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT dashboard_components.id
        FROM dashboard_components
        JOIN component_versions
          ON component_versions.id = dashboard_components.component_version_id
        WHERE dashboard_components.dashboard_id = $1
          AND component_versions.component_type <> 'table'::component_type
        ORDER BY dashboard_components.position, dashboard_components.id
        LIMIT 1
        "#,
    )
    .bind(dashboard_id)
    .fetch_one(&mut *tx)
    .await
    .expect("non-Table placement for fallback-exhaustion fixture");
    sqlx::query("UPDATE dashboard_components SET config = '{}'::jsonb WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(&mut *tx)
        .await
        .expect("install legacy fallback fixtures");
    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET config = '{
            "schema_version": 1,
            "grid_row": 1,
            "grid_column": 1,
            "grid_width": 1,
            "grid_height": 240
        }'::jsonb
        WHERE id = $1
        "#,
    )
    .bind(spanning_placement_id)
    .execute(&mut *tx)
    .await
    .expect("install all-row valid V1 fixture");

    let migration_error = sqlx::raw_sql(include_str!(
        "../migrations/002_dashboard_placement_capacity.sql"
    ))
    .execute(&mut *tx)
    .await
    .expect_err("fallback-row exhaustion must abort migration");
    let message = migration_error.to_string();
    assert!(message.contains("dashboard placement display-layout preflight failed"));
    assert!(message.contains(&dashboard_id.to_string()));
    assert!(message.contains("240 occupied rows + 8 fallback rows"));
    tx.rollback()
        .await
        .expect("rollback fallback-exhaustion fixture");

    let canonical_v1_count: i64 = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM dashboard_components
        WHERE dashboard_id = $1
          AND config ->> 'schema_version' = '1'
        "#,
    )
    .bind(dashboard_id)
    .fetch_one(&pool)
    .await
    .expect("post-fallback-preflight canonical config count");
    assert_eq!(canonical_v1_count, 9);
}

fn retain_commands(placements: &[Value]) -> Vec<Value> {
    placements
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
        .collect()
}

fn placement_ids(placements: &[Value]) -> BTreeSet<&str> {
    placements
        .iter()
        .filter_map(|placement| placement["placement_id"].as_str())
        .collect()
}

async fn create_user_with_capabilities(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    display_name: &str,
    capability_keys: &[&str],
) -> String {
    create_user_with_capabilities_and_scope(
        app,
        admin_token,
        email,
        display_name,
        capability_keys,
        None,
    )
    .await
}

async fn create_scoped_user_with_capabilities(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    display_name: &str,
    capability_keys: &[&str],
    scope_node_ids: &[String],
) -> String {
    create_user_with_capabilities_and_scope(
        app,
        admin_token,
        email,
        display_name,
        capability_keys,
        Some(scope_node_ids),
    )
    .await
}

async fn create_user_with_global_read_and_scoped_manage(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    manage_node_id: &str,
) -> String {
    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", admin_token, None),
    )
    .await;
    let capability_id = |key: &str| {
        capabilities
            .as_array()
            .expect("capability list")
            .iter()
            .find(|capability| capability["key"] == key)
            .and_then(|capability| capability["id"].as_str())
            .unwrap_or_else(|| panic!("missing capability {key}"))
            .to_string()
    };
    let read_role = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            admin_token,
            Some(json!({
                "name": "Dashboard split-scope reader",
                "capability_ids": [capability_id("dashboards:read")]
            })),
        ),
    )
    .await;
    let manage_role = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            admin_token,
            Some(json!({
                "name": "Dashboard split-scope manager",
                "capability_ids": [capability_id("dashboards:manage")]
            })),
        ),
    )
    .await;
    let user = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/users",
            admin_token,
            Some(json!({
                "email": email,
                "display_name": "Dashboard Split Scope",
                "password": "tessara-test-password-123",
                "is_active": true,
                "role_ids": [read_role["id"], manage_role["id"]]
            })),
        ),
    )
    .await;

    let pool = test_pool().await;
    sqlx::query("UPDATE role_assignments SET node_id = $3 WHERE account_id = $1 AND role_id = $2")
        .bind(
            user["id"]
                .as_str()
                .expect("split-scope account id")
                .parse::<Uuid>()
                .expect("account UUID"),
        )
        .bind(
            manage_role["id"]
                .as_str()
                .expect("manage role id")
                .parse::<Uuid>()
                .expect("manage role UUID"),
        )
        .bind(
            manage_node_id
                .parse::<Uuid>()
                .expect("manage scope node UUID"),
        )
        .execute(&pool)
        .await
        .expect("scope only the Dashboard manage role");

    login_token_for(app, email, "tessara-test-password-123").await
}

async fn create_user_with_capabilities_and_scope(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    display_name: &str,
    capability_keys: &[&str],
    scope_node_ids: Option<&[String]>,
) -> String {
    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", admin_token, None),
    )
    .await;
    let capability_ids = capability_keys
        .iter()
        .map(|key| {
            capabilities
                .as_array()
                .expect("capability list")
                .iter()
                .find(|capability| capability["key"] == *key)
                .and_then(|capability| capability["id"].as_str())
                .unwrap_or_else(|| panic!("missing capability {key}"))
        })
        .collect::<Vec<_>>();
    let role = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            admin_token,
            Some(json!({
                "name": format!("{display_name} Role"),
                "capability_ids": capability_ids
            })),
        ),
    )
    .await;
    let user = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/users",
            admin_token,
            Some(json!({
                "email": email,
                "display_name": display_name,
                "password": "tessara-test-password-123",
                "is_active": true,
                "role_ids": [role["id"]]
            })),
        ),
    )
    .await;
    if let Some(scope_node_ids) = scope_node_ids {
        let account_id = user["id"].as_str().expect("created scoped account id");
        request_json(
            app.clone(),
            authorized_request(
                "PUT",
                &format!("/api/admin/users/{account_id}/access"),
                admin_token,
                Some(json!({
                    "scope_node_ids": scope_node_ids,
                    "delegate_account_ids": []
                })),
            ),
        )
        .await;
    }
    login_token_for(app, email, "tessara-test-password-123").await
}

async fn test_pool() -> PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL").expect("test database url");
    PgPool::connect(&database_url)
        .await
        .expect("test database connection")
}

async fn seed_composition_context(
    app: axum::Router,
    admin_token: &str,
) -> (String, Vec<String>, String) {
    let seed = request_json(
        app.clone(),
        authorized_request("POST", "/api/demo/seed", admin_token, None),
    )
    .await;
    let seeded_dashboard_id = seed["dashboard_id"].as_str().expect("seeded Dashboard id");
    let composition = request_json(
        app,
        authorized_request(
            "GET",
            &format!("/api/admin/dashboards/{seeded_dashboard_id}/composition"),
            admin_token,
            None,
        ),
    )
    .await;
    let component_version_id = composition["available_component_versions"]
        .as_array()
        .expect("placeable Component versions")
        .iter()
        .find(|option| option["component_type"] == "stat_card")
        .and_then(|option| option["component_version_id"].as_str())
        .expect("placeable Stat Card Component version")
        .to_string();
    (
        seeded_dashboard_id.to_string(),
        visibility_node_ids(&composition),
        component_version_id,
    )
}

fn visibility_node_ids(composition: &Value) -> Vec<String> {
    composition["dashboard"]["visibility_nodes"]
        .as_array()
        .expect("Dashboard visibility nodes")
        .iter()
        .map(|node| {
            node["node_id"]
                .as_str()
                .expect("Dashboard visibility node id")
                .to_string()
        })
        .collect()
}

fn visibility_node_id_set(composition: &Value) -> BTreeSet<String> {
    visibility_node_ids(composition).into_iter().collect()
}

async fn create_dashboard_fixture(
    app: axum::Router,
    admin_token: &str,
    name: &str,
    visibility_node_ids: &[String],
) -> (String, String) {
    let created = request_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/dashboards",
            admin_token,
            Some(json!({
                "name": name,
                "description": format!("{name} integration fixture"),
                "visibility_node_ids": visibility_node_ids
            })),
        ),
    )
    .await;
    let dashboard_id = created["id"]
        .as_str()
        .expect("created Dashboard id")
        .to_string();
    let composition_uri = format!("/api/admin/dashboards/{dashboard_id}/composition");
    (dashboard_id, composition_uri)
}

async fn add_single_placement(
    app: axum::Router,
    admin_token: &str,
    composition_uri: &str,
    component_version_id: &str,
) -> String {
    let added = request_json(
        app,
        authorized_request(
            "PUT",
            composition_uri,
            admin_token,
            Some(json!({
                "commands": [{
                    "operation": "bind",
                    "client_key": "initial-placement",
                    "component_version_id": component_version_id,
                    "geometry": {
                        "grid_row": 1,
                        "grid_column": 1,
                        "grid_width": 1,
                        "grid_height": 1
                    }
                }]
            })),
        ),
    )
    .await;
    added["new_placement_ids"][0]["placement_id"]
        .as_str()
        .expect("new placement id")
        .to_string()
}
