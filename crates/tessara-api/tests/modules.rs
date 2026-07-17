#[allow(dead_code)]
mod support;

#[cfg(not(debug_assertions))]
use std::time::{Duration, Instant};

use axum::{
    body::{Body, Bytes, to_bytes},
    http::{HeaderMap, Request, StatusCode, header},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tessara_api::router;
use tower::ServiceExt;
use uuid::Uuid;

use support::{
    TEST_DATABASE_LOCK, authorized_request, login_token, login_token_for, request_json,
    request_status_and_json, test_state,
};
#[cfg(feature = "ssr")]
use support::{cookie_authenticated_request, login_cookie_for};

const PASSWORD: &str = "tessara-test-password-123";
const FORMS_DEFINITION: &str = "tessara.forms";
const RESPONSES_DEFINITION: &str = "tessara.responses";
const MIGRATION_DEFINITION: &str = "tessara.migration";
const UNKNOWN_DEFINITION: &str = "tessara.unknown-definition";
const FORM_RESOURCE_TYPE: &str = "tessara.transition.form";
const RESPONSE_RESOURCE_TYPE: &str = "tessara.transition.response";
#[cfg(not(debug_assertions))]
const RESTRICTED_TIMING_SAMPLES_PER_IDENTIFIER: usize = 200;
#[cfg(not(debug_assertions))]
const RESTRICTED_TIMING_WARMUP_PAIRS: usize = 20;

#[derive(Debug)]
struct Actor {
    account_id: Uuid,
    token: String,
    #[cfg(feature = "ssr")]
    cookie: String,
}

#[tokio::test]
async fn module_http_apis_enforce_global_authority_and_preserve_exact_sources() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;

    let reader = create_actor(
        app.clone(),
        &admin_token,
        "module-api-reader",
        &["modules:read"],
    )
    .await;
    let manager = create_actor(
        app.clone(),
        &admin_token,
        "module-api-manager",
        &["modules:manage_navigation"],
    )
    .await;
    let scoped_reader = create_actor(
        app.clone(),
        &admin_token,
        "module-api-scoped-reader",
        &["modules:read"],
    )
    .await;
    let scoped_manager = create_actor(
        app.clone(),
        &admin_token,
        "module-api-scoped-manager",
        &["modules:manage_navigation"],
    )
    .await;
    let product_only = create_actor(
        app.clone(),
        &admin_token,
        "module-api-product-only",
        &["forms:read"],
    )
    .await;
    let no_access = create_actor(app.clone(), &admin_token, "module-api-no-access", &[]).await;

    let scope_node_id = create_test_scope_node(&pool).await;
    force_scoped_assignment(&pool, scoped_reader.account_id, scope_node_id).await;
    force_scoped_assignment(&pool, scoped_manager.account_id, scope_node_id).await;

    let (anonymous_status, anonymous_body) = request_status_and_json(
        app.clone(),
        Request::builder()
            .uri("/api/admin/modules")
            .body(Body::empty())
            .expect("valid anonymous inventory request"),
    )
    .await;
    assert_eq!(anonymous_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_body["code"], "auth_unauthorized");

    let (anonymous_shell_status, anonymous_shell_body) = request_status_and_json(
        app.clone(),
        Request::builder()
            .uri("/api/shell/navigation")
            .body(Body::empty())
            .expect("valid anonymous shell request"),
    )
    .await;
    assert_eq!(anonymous_shell_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_shell_body["code"], "auth_unauthorized");

    let reader_shell = request_json(
        app.clone(),
        authorized_request("GET", "/api/shell/navigation", &reader.token, None),
    )
    .await;
    assert_eq!(reader_shell["schema_version"], 2);
    assert_eq!(reader_shell["state"], "available");
    assert!(shell_item(&reader_shell, "module_management").is_some());
    assert!(shell_item(&reader_shell, "administration").is_none());
    assert_eq!(
        reader_shell["groups"]
            .as_array()
            .expect("shell groups")
            .iter()
            .map(|group| group["name"].as_str().expect("group name"))
            .collect::<Vec<_>>(),
        vec!["Main", "Admin"]
    );

    let manager_shell = request_json(
        app.clone(),
        authorized_request("GET", "/api/shell/navigation", &manager.token, None),
    )
    .await;
    assert!(shell_item(&manager_shell, "module_management").is_some());
    assert!(shell_item(&manager_shell, "administration").is_none());

    let admin_shell = request_json(
        app.clone(),
        authorized_request("GET", "/api/shell/navigation", &admin_token, None),
    )
    .await;
    assert!(shell_item(&admin_shell, "module_management").is_some());
    assert!(shell_item(&admin_shell, "administration").is_none());
    for key in ["user_management", "roles_access", "node_types"] {
        assert!(shell_item(&admin_shell, key).is_some(), "missing {key}");
    }

    for (name, actor) in [
        ("scoped read", &scoped_reader),
        ("scoped manage", &scoped_manager),
        ("product only", &product_only),
        ("no access", &no_access),
    ] {
        let shell = request_json(
            app.clone(),
            authorized_request("GET", "/api/shell/navigation", &actor.token, None),
        )
        .await;
        assert_eq!(shell["state"], "available", "{name}");
        assert!(
            shell_item(&shell, "module_management").is_none(),
            "{name} must not receive Module Management"
        );
    }

    for (name, actor) in [
        ("scoped read", &scoped_reader),
        ("scoped manage", &scoped_manager),
        ("product only", &product_only),
        ("no access", &no_access),
    ] {
        let (status, body) = request_status_and_json(
            app.clone(),
            authorized_request("GET", "/api/admin/modules", &actor.token, None),
        )
        .await;
        assert_eq!(status, StatusCode::FORBIDDEN, "{name} must fail closed");
        assert_eq!(
            body["code"], "modules_read_global_required",
            "{name} must receive the stable global-read denial"
        );
    }

    let reader_inventory = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &reader.token, None),
    )
    .await;
    let manager_inventory = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &manager.token, None),
    )
    .await;
    let admin_inventory = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &admin_token, None),
    )
    .await;
    assert_eq!(reader_inventory, manager_inventory);
    assert_eq!(reader_inventory, admin_inventory);
    assert_eq!(reader_inventory["schema_version"], 1);
    assert_eq!(
        reader_inventory["entries"].as_array().map(Vec::len),
        Some(7)
    );
    assert_eq!(
        reader_inventory["core_runtime"]["provenance"],
        "development_unresolved"
    );
    assert_eq!(
        reader_inventory["core_runtime"]["finding_code"],
        "core_release_provenance_unresolved"
    );

    let entries = reader_inventory["entries"]
        .as_array()
        .expect("module inventory entries should be an array");
    assert_eq!(
        entries
            .iter()
            .filter(|entry| entry["descriptor"]["availability"] == "active_in_process")
            .count(),
        6
    );
    let migration = inventory_entry(entries, MIGRATION_DEFINITION);
    assert_eq!(migration["kind"], "transitional_in_process");
    assert_eq!(migration["descriptor"]["availability"], "retired");
    assert_eq!(migration["provider_eligible"], false);
    assert_eq!(migration["supervisor_materializable"], false);
    assert_eq!(
        migration["findings"][0]["code"],
        "transition_destination_retired"
    );
    assert!(migration.get("release").is_none());
    assert!(migration.get("instance").is_none());

    let responses = inventory_entry(entries, RESPONSES_DEFINITION);
    assert_eq!(
        responses["findings"]
            .as_array()
            .expect("Responses findings")
            .iter()
            .map(|finding| finding["code"].as_str())
            .collect::<Vec<_>>(),
        vec![
            Some("transition_internal_only"),
            Some("transition_internal_only")
        ]
    );

    let reader_detail = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/admin/modules/{FORMS_DEFINITION}"),
            &reader.token,
            None,
        ),
    )
    .await;
    assert_eq!(reader_detail["schema_version"], 1);
    assert_eq!(reader_detail["entry"]["kind"], "transitional_in_process");
    assert_eq!(
        reader_detail["entry"]["descriptor"]["reserved_definition_id"],
        FORMS_DEFINITION
    );
    assert_eq!(
        reader_detail["entry"]["source_digest"],
        inventory_entry(entries, FORMS_DEFINITION)["source_digest"]
    );
    assert_eq!(
        reader_detail["entry"]["resource_owner"]["installation_id"],
        reader_inventory["installation"]["id"]
    );

    let unknown_path = format!("/api/admin/modules/{UNKNOWN_DEFINITION}");
    let (unknown_status, unknown_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &unknown_path, &reader.token, None),
    )
    .await;
    assert_eq!(unknown_status, StatusCode::NOT_FOUND);
    assert_eq!(unknown_body["code"], "module_definition_not_found");

    let known_path = format!("/api/admin/modules/{FORMS_DEFINITION}");
    let (scoped_known_status, scoped_known_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &known_path, &scoped_reader.token, None),
    )
    .await;
    let (scoped_unknown_status, scoped_unknown_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &unknown_path, &scoped_reader.token, None),
    )
    .await;
    assert_eq!(scoped_known_status, StatusCode::FORBIDDEN);
    assert_eq!(scoped_unknown_status, StatusCode::FORBIDDEN);
    assert_eq!(scoped_known_body, scoped_unknown_body);
    assert_eq!(scoped_known_body["code"], "modules_read_global_required");

    let (anonymous_known_status, anonymous_known_body) = request_status_and_json(
        app.clone(),
        Request::builder()
            .uri(&known_path)
            .body(Body::empty())
            .expect("valid anonymous known-detail request"),
    )
    .await;
    let (anonymous_unknown_status, anonymous_unknown_body) = request_status_and_json(
        app.clone(),
        Request::builder()
            .uri(&unknown_path)
            .body(Body::empty())
            .expect("valid anonymous unknown-detail request"),
    )
    .await;
    assert_eq!(anonymous_known_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_unknown_status, StatusCode::UNAUTHORIZED);
    assert_eq!(anonymous_known_body, anonymous_unknown_body);

    let descriptor_path = format!("/api/admin/modules/{FORMS_DEFINITION}/descriptor");
    let (descriptor_status, descriptor_headers, descriptor_body) = response_bytes(
        app.clone(),
        authorized_request("GET", &descriptor_path, &reader.token, None),
    )
    .await;
    assert_eq!(descriptor_status, StatusCode::OK);
    assert_eq!(
        descriptor_headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("application/json")
    );
    assert_eq!(
        descriptor_body.as_ref(),
        include_bytes!("../../tessara-module-contract/tests/fixtures/transition-forms-v1.json")
    );
    let descriptor_etag = descriptor_headers
        .get(header::ETAG)
        .expect("descriptor should expose its exact source digest")
        .clone();
    assert_eq!(
        descriptor_etag
            .to_str()
            .expect("ETag should be visible ASCII"),
        format!(
            "\"{}\"",
            include_str!(
                "../../tessara-module-contract/tests/fixtures/transition-forms-v1.json.sha256"
            )
            .trim()
        )
    );
    assert_eq!(
        reader_detail["entry"]["source_digest"],
        descriptor_etag
            .to_str()
            .expect("ETag should be visible ASCII")
            .trim_matches('"')
    );

    let conditional_request = Request::builder()
        .method("GET")
        .uri(&descriptor_path)
        .header(header::AUTHORIZATION, format!("Bearer {}", reader.token))
        .header(header::IF_NONE_MATCH, descriptor_etag.clone())
        .body(Body::empty())
        .expect("valid conditional descriptor request");
    let (conditional_status, conditional_headers, conditional_body) =
        response_bytes(app.clone(), conditional_request).await;
    assert_eq!(conditional_status, StatusCode::NOT_MODIFIED);
    assert_eq!(
        conditional_headers.get(header::ETAG),
        Some(&descriptor_etag)
    );
    assert!(conditional_body.is_empty());

    let descriptor_unknown_path = format!("/api/admin/modules/{UNKNOWN_DEFINITION}/descriptor");
    let (descriptor_unknown_status, descriptor_unknown_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &descriptor_unknown_path, &reader.token, None),
    )
    .await;
    assert_eq!(descriptor_unknown_status, StatusCode::NOT_FOUND);
    assert_eq!(
        descriptor_unknown_body["code"],
        "module_definition_not_found"
    );
    let (scoped_descriptor_known_status, scoped_descriptor_known_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &descriptor_path, &scoped_reader.token, None),
    )
    .await;
    let (scoped_descriptor_unknown_status, scoped_descriptor_unknown_body) =
        request_status_and_json(
            app.clone(),
            authorized_request("GET", &descriptor_unknown_path, &scoped_reader.token, None),
        )
        .await;
    assert_eq!(scoped_descriptor_known_status, StatusCode::FORBIDDEN);
    assert_eq!(scoped_descriptor_unknown_status, StatusCode::FORBIDDEN);
    assert_eq!(
        scoped_descriptor_known_body, scoped_descriptor_unknown_body,
        "descriptor lookup must not become a scoped-only definition oracle"
    );
    assert_eq!(
        scoped_descriptor_known_body["code"],
        "modules_read_global_required"
    );

    let reader_policy = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/navigation-policy", &reader.token, None),
    )
    .await;
    assert_eq!(reader_policy["can_manage_navigation"], false);
    assert_eq!(reader_policy["schema_version"], 2);
    assert_eq!(reader_policy["groups"].as_array().map(Vec::len), Some(2));
    assert_eq!(
        reader_policy["destinations"].as_array().map(Vec::len),
        Some(13)
    );
    assert!(
        reader_policy["groups"]
            .as_array()
            .expect("policy groups")
            .iter()
            .map(|group| group["id"].as_str().expect("group id"))
            .eq(["core.main", "core.admin"])
    );
    assert!(
        reader_policy["destinations"]
            .as_array()
            .expect("policy destinations")
            .iter()
            .any(|entry| {
                entry["id"] == "core.admin.modules"
                    && entry["group_id"] == "core.admin"
                    && entry["route"] == "/administration/modules"
                    && entry["can_hide"] == false
                    && entry["can_move_between_groups"] == false
            })
    );

    let (scoped_policy_status, scoped_policy_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "GET",
            "/api/admin/navigation-policy",
            &scoped_reader.token,
            None,
        ),
    )
    .await;
    assert_eq!(scoped_policy_status, StatusCode::FORBIDDEN);
    assert_eq!(scoped_policy_body["code"], "modules_read_global_required");

    let reader_policy_update = policy_update_request(&reader_policy);
    let (reader_put_status, reader_put_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            "/api/admin/navigation-policy",
            &reader.token,
            Some(reader_policy_update),
        ),
    )
    .await;
    assert_eq!(reader_put_status, StatusCode::FORBIDDEN);
    assert_eq!(
        reader_put_body["code"],
        "modules_manage_navigation_global_required"
    );

    let manager_policy = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/navigation-policy", &manager.token, None),
    )
    .await;
    assert_eq!(manager_policy["can_manage_navigation"], true);
    let manager_update = policy_update_request(&manager_policy);
    let manager_saved = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            "/api/admin/navigation-policy",
            &manager.token,
            Some(manager_update),
        ),
    )
    .await;
    assert_eq!(
        manager_saved, manager_policy,
        "a no-op PUT must be idempotent"
    );

    let admin_policy = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/navigation-policy", &admin_token, None),
    )
    .await;
    assert_eq!(admin_policy["can_manage_navigation"], true);

    let scoped_manager_update = policy_update_request(&manager_policy);
    let (scoped_put_status, scoped_put_body) = request_status_and_json(
        app,
        authorized_request(
            "PUT",
            "/api/admin/navigation-policy",
            &scoped_manager.token,
            Some(scoped_manager_update),
        ),
    )
    .await;
    assert_eq!(scoped_put_status, StatusCode::FORBIDDEN);
    assert_eq!(
        scoped_put_body["code"],
        "modules_manage_navigation_global_required"
    );
}

#[tokio::test]
async fn navigation_policy_http_rejections_are_atomic_and_exactly_audited() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;
    let manager = create_actor(
        app.clone(),
        &admin_token,
        "navigation-policy-negative-manager",
        &["modules:manage_navigation"],
    )
    .await;

    let original_policy = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/navigation-policy", &manager.token, None),
    )
    .await;
    let original_request = policy_update_request(&original_policy);
    let denials_before =
        control_plane_audit_count(&pool, "navigation_policy.update_denied", manager.account_id)
            .await;
    let successes_before =
        control_plane_audit_count(&pool, "navigation_policy.updated", manager.account_id).await;

    let mut missing_required_group = original_request.clone();
    missing_required_group["groups"]
        .as_array_mut()
        .expect("policy groups should be an array")
        .retain(|group| group["id"] != "core.admin");

    let mut duplicate_group_label = original_request.clone();
    navigation_group_mut(&mut duplicate_group_label, "core.admin")["label"] = json!("Main");

    let mut partial_collection = original_request.clone();
    partial_collection["destinations"]
        .as_array_mut()
        .expect("policy mutations should be an array")
        .pop();

    let mut duplicate_id = original_request.clone();
    let duplicate = duplicate_id["destinations"]
        .as_array()
        .expect("policy mutations should be an array")[0]
        .clone();
    duplicate_id["destinations"]
        .as_array_mut()
        .expect("policy mutations should be an array")
        .push(duplicate);

    let mut unknown_id = original_request.clone();
    unknown_id["destinations"]
        .as_array_mut()
        .expect("policy mutations should be an array")
        .push(json!({
            "id": "tessara.unknown.navigation",
            "group_id": "core.main",
            "visible": true,
            "order": 9
        }));

    let mut core_module_management = original_request.clone();
    navigation_mutation_mut(&mut core_module_management, "core.admin.modules")["visible"] =
        json!(false);

    let mut non_dense_order = original_request.clone();
    navigation_mutation_mut(&mut non_dense_order, "tessara.forms.navigation")["order"] = json!(99);

    let rejection_cases = [
        (
            "missing required group",
            missing_required_group,
            "navigation_policy_groups_invalid",
        ),
        (
            "duplicate group label",
            duplicate_group_label,
            "navigation_policy_groups_invalid",
        ),
        (
            "partial destination collection",
            partial_collection,
            "navigation_policy_destinations_invalid",
        ),
        (
            "duplicate destination ID",
            duplicate_id,
            "navigation_policy_destinations_invalid",
        ),
        (
            "unknown destination ID",
            unknown_id,
            "navigation_policy_destinations_invalid",
        ),
        (
            "protected Module Management visibility mutation",
            core_module_management,
            "navigation_policy_destination_protected",
        ),
        (
            "non-dense destination order",
            non_dense_order,
            "navigation_policy_destinations_invalid",
        ),
    ];

    for (index, (case_name, request, code)) in rejection_cases.into_iter().enumerate() {
        assert_navigation_policy_rejection(
            app.clone(),
            &pool,
            &manager,
            request,
            &original_policy,
            StatusCode::BAD_REQUEST,
            code,
            denials_before + index as i64 + 1,
            successes_before,
            case_name,
        )
        .await;
    }

    let mut changed_request = original_request.clone();
    navigation_mutation_mut(&mut changed_request, "tessara.forms.navigation")["order"] = json!(3);
    navigation_mutation_mut(&mut changed_request, "tessara.workflows.navigation")["order"] =
        json!(2);
    navigation_mutation_mut(&mut changed_request, "tessara.dashboards.navigation")["visible"] =
        json!(false);
    let changed_policy = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            "/api/admin/navigation-policy",
            &manager.token,
            Some(changed_request),
        ),
    )
    .await;
    assert_eq!(
        changed_policy["revision"].as_i64(),
        original_policy["revision"]
            .as_i64()
            .map(|revision| revision + 1)
    );
    assert_eq!(
        control_plane_audit_count(&pool, "navigation_policy.update_denied", manager.account_id,)
            .await,
        denials_before + 7
    );
    assert_eq!(
        control_plane_audit_count(&pool, "navigation_policy.updated", manager.account_id,).await,
        successes_before + 1
    );
    let (
        success_installation_id,
        success_event_type,
        success_actor_kind,
        success_actor_account_id,
        success_correlation_id,
        success_payload,
    ): (Uuid, String, String, Option<Uuid>, Uuid, Value) = sqlx::query_as(
        r#"
        SELECT installation_id, event_type, actor_kind, actor_account_id, correlation_id, payload
        FROM core_control_plane_audit_events
        WHERE event_type = 'navigation_policy.updated'
          AND actor_account_id = $1
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(manager.account_id)
    .fetch_one(&pool)
    .await
    .expect("successful navigation-policy audit event");
    let expected_installation_id = Uuid::parse_str(
        original_policy["installation_id"]
            .as_str()
            .expect("policy installation ID"),
    )
    .expect("canonical policy installation UUID");
    assert_eq!(success_installation_id, expected_installation_id);
    assert_eq!(success_event_type, "navigation_policy.updated");
    assert_eq!(success_actor_kind, "account");
    assert_eq!(success_actor_account_id, Some(manager.account_id));
    assert_ne!(success_correlation_id, Uuid::nil());
    assert_eq!(
        success_payload,
        json!({
            "schema_version": 2,
            "installation_id": expected_installation_id,
            "before_revision": original_policy["revision"],
            "after_revision": changed_policy["revision"],
            "groups": policy_audit_groups(&changed_policy),
            "placements": policy_audit_placements(&changed_policy),
            "success": true,
        })
    );

    assert_navigation_policy_rejection(
        app.clone(),
        &pool,
        &manager,
        original_request.clone(),
        &changed_policy,
        StatusCode::CONFLICT,
        "navigation_policy_revision_conflict",
        denials_before + 8,
        successes_before + 1,
        "stale revision",
    )
    .await;

    let mut restore_request = original_request;
    restore_request["expected_revision"] = changed_policy["revision"].clone();
    let restored_policy = request_json(
        app,
        authorized_request(
            "PUT",
            "/api/admin/navigation-policy",
            &manager.token,
            Some(restore_request),
        ),
    )
    .await;
    assert_eq!(
        restored_policy["revision"].as_i64(),
        original_policy["revision"]
            .as_i64()
            .map(|revision| revision + 2)
    );
    assert_eq!(restored_policy["groups"], original_policy["groups"]);
    assert_eq!(
        restored_policy["destinations"],
        original_policy["destinations"]
    );
    assert_eq!(
        control_plane_audit_count(&pool, "navigation_policy.update_denied", manager.account_id,)
            .await,
        denials_before + 8
    );
    assert_eq!(
        control_plane_audit_count(&pool, "navigation_policy.updated", manager.account_id,).await,
        successes_before + 2
    );
}

#[tokio::test]
async fn platform_http_apis_enforce_strict_wires_authority_order_and_non_disclosure() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;

    let global_forms = create_actor(
        app.clone(),
        &admin_token,
        "platform-global-forms",
        &["forms:read"],
    )
    .await;
    let scoped_forms = create_actor(
        app.clone(),
        &admin_token,
        "platform-scoped-forms",
        &["forms:read"],
    )
    .await;
    let no_access = create_actor(app.clone(), &admin_token, "platform-no-access", &[]).await;
    let scope_node_id = create_test_scope_node(&pool).await;
    force_scoped_assignment(&pool, scoped_forms.account_id, scope_node_id).await;

    let known_form_id: Uuid = sqlx::query_scalar(
        "INSERT INTO forms (name, slug) VALUES ('Platform reference fixture', 'platform-reference-fixture') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("known platform-reference Form should be created");
    let random_form_id = Uuid::new_v4();
    let foreign_installation_id = Uuid::new_v4();

    let inventory = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &admin_token, None),
    )
    .await;
    let installation_id = Uuid::parse_str(
        inventory["installation"]["id"]
            .as_str()
            .expect("inventory should expose the installation identity"),
    )
    .expect("installation identity should be a UUID");
    let current_owner = json!({
        "kind": "core_installation",
        "installation_id": installation_id,
    });

    for path in [
        "/api/platform/destinations/resolve",
        "/api/platform/resource-references",
        "/api/platform/resource-references/resolve",
    ] {
        let (status, body) = request_status_and_json(
            app.clone(),
            Request::builder()
                .method("POST")
                .uri(path)
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{"))
                .expect("valid anonymous malformed platform request"),
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{path}");
        assert_eq!(body["code"], "auth_unauthorized", "{path}");
    }

    let destination_request = json!({
        "schema_version": 1,
        "destination": {
            "owner": current_owner.clone(),
            "route": "forms.detail",
            "parameters": {
                "form_id": { "type": "uuid", "value": known_form_id }
            }
        }
    });
    let expected_destination = json!({
        "schema_version": 1,
        "status": "resolved",
        "path": format!("/forms/{known_form_id}"),
    });
    for (name, actor) in [
        ("global product authority", &global_forms),
        ("scoped product authority", &scoped_forms),
    ] {
        let (status, body) = request_status_and_json(
            app.clone(),
            authorized_request(
                "POST",
                "/api/platform/destinations/resolve",
                &actor.token,
                Some(destination_request.clone()),
            ),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{name}");
        assert_eq!(body, expected_destination, "{name}");
    }

    let (unauthorized_destination_status, unauthorized_destination) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/destinations/resolve",
            &no_access.token,
            Some(destination_request.clone()),
        ),
    )
    .await;
    assert_eq!(unauthorized_destination_status, StatusCode::OK);
    assert_eq!(unauthorized_destination["status"], "rejected");
    assert_eq!(
        unauthorized_destination["finding"]["code"],
        "semantic_destination_unauthorized"
    );
    assert!(unauthorized_destination.get("path").is_none());

    let mut wrong_owner_destination = destination_request.clone();
    wrong_owner_destination["destination"]["owner"] = json!({
        "kind": "core_installation",
        "installation_id": foreign_installation_id,
    });
    let (wrong_owner_status, wrong_owner_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/destinations/resolve",
            &global_forms.token,
            Some(wrong_owner_destination),
        ),
    )
    .await;
    assert_eq!(wrong_owner_status, StatusCode::OK);
    assert_eq!(
        wrong_owner_body["finding"]["code"],
        "semantic_destination_owner_mismatch"
    );
    assert!(wrong_owner_body.get("path").is_none());

    let mut unknown_destination = destination_request.clone();
    unknown_destination["destination"]["route"] = json!("forms.unknown");
    unknown_destination["destination"]["parameters"] = json!({});
    let (unknown_destination_status, unknown_destination_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/destinations/resolve",
            &global_forms.token,
            Some(unknown_destination),
        ),
    )
    .await;
    assert_eq!(unknown_destination_status, StatusCode::OK);
    assert_eq!(
        unknown_destination_body["finding"]["code"],
        "semantic_destination_unknown"
    );
    assert!(unknown_destination_body.get("path").is_none());

    let mut wrong_parameter_type = destination_request.clone();
    wrong_parameter_type["destination"]["parameters"]["form_id"] =
        json!({ "type": "string", "value": known_form_id.to_string() });
    let (parameter_status, parameter_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/destinations/resolve",
            &global_forms.token,
            Some(wrong_parameter_type),
        ),
    )
    .await;
    assert_eq!(parameter_status, StatusCode::OK);
    assert_eq!(
        parameter_body["finding"]["code"],
        "semantic_destination_parameter_type_mismatch"
    );
    assert!(parameter_body.get("path").is_none());

    let mut destination_with_url = destination_request.clone();
    destination_with_url["destination"]["url"] = json!("https://caller.invalid/forms");
    let (destination_wire_status, destination_wire_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/destinations/resolve",
            &global_forms.token,
            Some(destination_with_url),
        ),
    )
    .await;
    assert_eq!(destination_wire_status, StatusCode::BAD_REQUEST);
    assert_eq!(destination_wire_body["code"], "platform_request_invalid");

    let mut unsupported_destination_schema = destination_request.clone();
    unsupported_destination_schema["schema_version"] = json!(2);
    let (destination_schema_status, destination_schema_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/destinations/resolve",
            &global_forms.token,
            Some(unsupported_destination_schema),
        ),
    )
    .await;
    assert_eq!(destination_schema_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        destination_schema_body["code"],
        "platform_schema_version_unsupported"
    );

    let reference_request = json!({
        "schema_version": 1,
        "installation_id": installation_id,
        "owner": current_owner.clone(),
        "resource_type": FORM_RESOURCE_TYPE,
        "resource_id": known_form_id.to_string(),
    });
    let global_reference = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(reference_request.clone()),
        ),
    )
    .await;
    assert_eq!(global_reference["schema_version"], 1);
    assert_eq!(
        global_reference["reference"]["installation_id"],
        json!(installation_id)
    );
    assert_eq!(global_reference["reference"]["owner"], current_owner);
    assert_eq!(
        global_reference["reference"]["resource_type"],
        FORM_RESOURCE_TYPE
    );
    assert_eq!(
        global_reference["reference"]["resource_id"],
        known_form_id.to_string()
    );

    let scoped_reference = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &scoped_forms.token,
            Some(reference_request.clone()),
        ),
    )
    .await;
    assert_eq!(scoped_reference, global_reference);

    let mut invalid_identifier = reference_request.clone();
    invalid_identifier["resource_id"] = json!("not-a-canonical-uuid");
    let (unauthorized_reference_status, unauthorized_reference_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &no_access.token,
            Some(invalid_identifier.clone()),
        ),
    )
    .await;
    assert_eq!(unauthorized_reference_status, StatusCode::FORBIDDEN);
    assert_eq!(
        unauthorized_reference_body["code"], "resource_reference_capability_required",
        "product authority must be checked before identifier existence/shape"
    );
    let (invalid_identifier_status, invalid_identifier_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(invalid_identifier),
        ),
    )
    .await;
    assert_eq!(invalid_identifier_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        invalid_identifier_body["code"],
        "resource_reference_id_invalid"
    );

    let mut foreign_installation = reference_request.clone();
    foreign_installation["installation_id"] = json!(foreign_installation_id);
    foreign_installation["owner"] = json!({
        "kind": "core_installation",
        "installation_id": foreign_installation_id,
    });
    let (foreign_installation_status, foreign_installation_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(foreign_installation),
        ),
    )
    .await;
    assert_eq!(foreign_installation_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        foreign_installation_body["code"],
        "resource_reference_installation_mismatch"
    );

    let mut foreign_owner = reference_request.clone();
    foreign_owner["owner"] = json!({
        "kind": "core_installation",
        "installation_id": foreign_installation_id,
    });
    let (foreign_owner_status, foreign_owner_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(foreign_owner),
        ),
    )
    .await;
    assert_eq!(foreign_owner_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        foreign_owner_body["code"],
        "resource_reference_owner_mismatch"
    );

    let mut unknown_resource_type = reference_request.clone();
    unknown_resource_type["resource_type"] = json!("tessara.transition.unknown");
    let (unknown_type_status, unknown_type_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(unknown_resource_type),
        ),
    )
    .await;
    assert_eq!(unknown_type_status, StatusCode::BAD_REQUEST);
    assert_eq!(unknown_type_body["code"], "resource_reference_type_unknown");

    let mut reference_with_unknown_field = reference_request.clone();
    reference_with_unknown_field["deployment_url"] = json!("https://caller.invalid");
    let (reference_wire_status, reference_wire_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(reference_with_unknown_field),
        ),
    )
    .await;
    assert_eq!(reference_wire_status, StatusCode::BAD_REQUEST);
    assert_eq!(reference_wire_body["code"], "platform_request_invalid");

    let mut unsupported_reference_schema = reference_request;
    unsupported_reference_schema["schema_version"] = json!(2);
    let (reference_schema_status, reference_schema_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references",
            &global_forms.token,
            Some(unsupported_reference_schema),
        ),
    )
    .await;
    assert_eq!(reference_schema_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        reference_schema_body["code"],
        "platform_schema_version_unsupported"
    );

    let known_reference = global_reference["reference"].clone();
    let mut random_reference = known_reference.clone();
    random_reference["resource_id"] = json!(random_form_id.to_string());
    let known_resolution_request =
        json!({ "schema_version": 1, "reference": known_reference.clone() });
    let random_resolution_request =
        json!({ "schema_version": 1, "reference": random_reference.clone() });

    let global_known = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references/resolve",
            &global_forms.token,
            Some(known_resolution_request.clone()),
        ),
    )
    .await;
    assert_eq!(
        global_known,
        json!({
            "schema_version": 1,
            "access_state": "authorized",
            "owner_state": { "kind": "core_installation", "state": "live" },
            "resource_identity_state": "resolved",
            "resource_lifecycle_state": { "kind": "provider_defined", "state": "active" },
            "compatibility_state": "compatible",
            "availability_state": "available",
        })
    );
    let global_random = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references/resolve",
            &global_forms.token,
            Some(random_resolution_request.clone()),
        ),
    )
    .await;
    assert_eq!(global_random, authorized_unknown_resolution());

    for (name, actor, access_state) in [
        ("unauthorized", &no_access, "unauthorized"),
        ("scoped/not evaluated", &scoped_forms, "not_evaluated"),
    ] {
        let known = request_json(
            app.clone(),
            authorized_request(
                "POST",
                "/api/platform/resource-references/resolve",
                &actor.token,
                Some(known_resolution_request.clone()),
            ),
        )
        .await;
        let random = request_json(
            app.clone(),
            authorized_request(
                "POST",
                "/api/platform/resource-references/resolve",
                &actor.token,
                Some(random_resolution_request.clone()),
            ),
        )
        .await;
        let expected = restricted_resolution(access_state);
        assert_eq!(known, expected, "{name} known identifier");
        assert_eq!(random, expected, "{name} random identifier");
        assert_eq!(
            known, random,
            "{name} must not disclose identifier existence"
        );
    }

    let mut inconsistent_reference = known_reference.clone();
    inconsistent_reference["owner"]["installation_id"] = json!(foreign_installation_id);
    let (inconsistent_status, inconsistent_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references/resolve",
            &global_forms.token,
            Some(json!({ "schema_version": 1, "reference": inconsistent_reference })),
        ),
    )
    .await;
    assert_eq!(inconsistent_status, StatusCode::BAD_REQUEST);
    assert_eq!(inconsistent_body["code"], "platform_request_invalid");

    let mut resolution_with_unknown_field = known_resolution_request.clone();
    resolution_with_unknown_field["diagnostics"] = json!(true);
    let (resolution_wire_status, resolution_wire_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/platform/resource-references/resolve",
            &global_forms.token,
            Some(resolution_with_unknown_field),
        ),
    )
    .await;
    assert_eq!(resolution_wire_status, StatusCode::BAD_REQUEST);
    assert_eq!(resolution_wire_body["code"], "platform_request_invalid");

    let mut unsupported_resolution_schema = known_resolution_request;
    unsupported_resolution_schema["schema_version"] = json!(2);
    let (resolution_schema_status, resolution_schema_body) = request_status_and_json(
        app,
        authorized_request(
            "POST",
            "/api/platform/resource-references/resolve",
            &global_forms.token,
            Some(unsupported_resolution_schema),
        ),
    )
    .await;
    assert_eq!(resolution_schema_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        resolution_schema_body["code"],
        "platform_schema_version_unsupported"
    );
}

#[tokio::test]
async fn response_reference_resolution_preserves_ownership_delegation_scope_and_non_disclosure() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;
    let seed = tessara_api::demo::seed_demo(&pool)
        .await
        .expect("deterministic Response reference fixtures should seed");

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
    let no_access = create_actor(
        app.clone(),
        &admin_token,
        "response-reference-no-access",
        &[],
    )
    .await;
    let scoped_manager = create_actor(
        app.clone(),
        &admin_token,
        "response-reference-scoped-manager",
        &["submissions:manage"],
    )
    .await;
    force_scoped_assignment(&pool, scoped_manager.account_id, seed.session_node_id).await;

    let delegate_submission_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT submissions.id
        FROM submissions
        JOIN workflow_assignments
          ON workflow_assignments.id = submissions.workflow_assignment_id
        JOIN accounts ON accounts.id = workflow_assignments.account_id
        WHERE accounts.email = 'delegate@tessara.local'
          AND submissions.node_id <> $1
        ORDER BY submissions.id
        LIMIT 1
        "#,
    )
    .bind(seed.session_node_id)
    .fetch_one(&pool)
    .await
    .expect("the seed should contain an out-of-scope delegate Response");
    let respondent_submission_id = seed.submission_id;
    let random_submission_id = Uuid::new_v4();

    let inventory = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &admin_token, None),
    )
    .await;
    let installation_id = inventory["installation"]["id"]
        .as_str()
        .expect("inventory should expose installation id");
    let respondent_reference = response_reference(installation_id, respondent_submission_id);
    let delegate_reference = response_reference(installation_id, delegate_submission_id);
    let random_reference = response_reference(installation_id, random_submission_id);

    let admin_known =
        resolve_reference(app.clone(), &admin_token, respondent_reference.clone()).await;
    assert_eq!(admin_known["access_state"], "authorized");
    assert_eq!(admin_known["resource_identity_state"], "resolved");
    assert_eq!(
        admin_known["resource_lifecycle_state"],
        json!({ "kind": "provider_defined", "state": "submitted" })
    );
    let admin_random = resolve_reference(app.clone(), &admin_token, random_reference.clone()).await;
    assert_eq!(admin_random, authorized_unknown_resolution());

    let respondent_own =
        resolve_reference(app.clone(), &respondent_token, respondent_reference.clone()).await;
    assert_eq!(respondent_own, admin_known);
    let respondent_unrelated =
        resolve_reference(app.clone(), &respondent_token, delegate_reference.clone()).await;
    let respondent_random =
        resolve_reference(app.clone(), &respondent_token, random_reference.clone()).await;
    assert_eq!(respondent_unrelated, restricted_resolution("not_evaluated"));
    assert_eq!(respondent_unrelated, respondent_random);

    let delegator_delegated =
        resolve_reference(app.clone(), &delegator_token, delegate_reference.clone()).await;
    assert_eq!(delegator_delegated["access_state"], "authorized");
    assert_eq!(delegator_delegated["resource_identity_state"], "resolved");
    let delegator_unrelated =
        resolve_reference(app.clone(), &delegator_token, respondent_reference.clone()).await;
    let delegator_random =
        resolve_reference(app.clone(), &delegator_token, random_reference.clone()).await;
    assert_eq!(delegator_unrelated, restricted_resolution("not_evaluated"));
    assert_eq!(delegator_unrelated, delegator_random);

    let scoped_known = resolve_reference(
        app.clone(),
        &scoped_manager.token,
        respondent_reference.clone(),
    )
    .await;
    assert_eq!(scoped_known["access_state"], "authorized");
    assert_eq!(scoped_known["resource_identity_state"], "resolved");
    let scoped_outside = resolve_reference(
        app.clone(),
        &scoped_manager.token,
        delegate_reference.clone(),
    )
    .await;
    let scoped_random =
        resolve_reference(app.clone(), &scoped_manager.token, random_reference.clone()).await;
    assert_eq!(scoped_outside, restricted_resolution("not_evaluated"));
    assert_eq!(scoped_outside, scoped_random);

    let no_access_known =
        resolve_reference(app.clone(), &no_access.token, respondent_reference).await;
    let no_access_random = resolve_reference(app, &no_access.token, random_reference).await;
    assert_eq!(no_access_known, restricted_resolution("unauthorized"));
    assert_eq!(no_access_known, no_access_random);
}

/// Full validation invokes this exact test under `cargo test --release` with
/// `--nocapture`. The test is absent from debug suites because an unoptimized
/// timing result is not valid evidence.
#[cfg(not(debug_assertions))]
#[tokio::test]
async fn resource_reference_restricted_known_random_latency_profile() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;

    let scoped_forms = create_actor(
        app.clone(),
        &admin_token,
        "platform-timing-scoped-forms",
        &["forms:read"],
    )
    .await;
    let no_access = create_actor(app.clone(), &admin_token, "platform-timing-no-access", &[]).await;
    let scope_node_id = create_test_scope_node(&pool).await;
    force_scoped_assignment(&pool, scoped_forms.account_id, scope_node_id).await;

    let known_form_id: Uuid = sqlx::query_scalar(
        "INSERT INTO forms (name, slug) VALUES ('Platform timing fixture', 'platform-timing-fixture') RETURNING id",
    )
    .fetch_one(&pool)
    .await
    .expect("known timing-profile Form should be created");
    let inventory = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &admin_token, None),
    )
    .await;
    let installation_id = inventory["installation"]["id"]
        .as_str()
        .expect("inventory should expose the installation identity");
    let known_reference = json!({
        "installation_id": installation_id,
        "owner": {
            "kind": "core_installation",
            "installation_id": installation_id,
        },
        "resource_type": FORM_RESOURCE_TYPE,
        "resource_id": known_form_id.to_string(),
    });
    let mut random_reference = known_reference.clone();
    random_reference["resource_id"] = json!(Uuid::new_v4().to_string());

    for (access_state, actor, expected_state) in [
        ("unauthorized", &no_access, "unauthorized"),
        ("not_evaluated", &scoped_forms, "not_evaluated"),
    ] {
        let expected = restricted_resolution(expected_state);
        let _ = sample_restricted_resolution_latencies(
            app.clone(),
            &actor.token,
            &known_reference,
            &random_reference,
            &expected,
            RESTRICTED_TIMING_WARMUP_PAIRS,
        )
        .await;
        let (known_samples, random_samples) = sample_restricted_resolution_latencies(
            app.clone(),
            &actor.token,
            &known_reference,
            &random_reference,
            &expected,
            RESTRICTED_TIMING_SAMPLES_PER_IDENTIFIER,
        )
        .await;
        assert_restricted_timing_profile(access_state, &known_samples, &random_samples);
    }
}

#[cfg(feature = "ssr")]
#[tokio::test]
async fn native_module_management_routes_render_authorized_restricted_and_not_found_states() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;
    let admin_cookie =
        login_cookie_for(app.clone(), "admin@tessara.local", "tessara-dev-admin").await;

    let reader = create_actor(
        app.clone(),
        &admin_token,
        "module-native-reader",
        &["modules:read"],
    )
    .await;
    let manager = create_actor(
        app.clone(),
        &admin_token,
        "module-native-manager",
        &["modules:manage_navigation"],
    )
    .await;
    let scoped_reader = create_actor(
        app.clone(),
        &admin_token,
        "module-native-scoped-reader",
        &["modules:read"],
    )
    .await;
    let product_only = create_actor(
        app.clone(),
        &admin_token,
        "module-native-product-only",
        &["forms:read"],
    )
    .await;
    let no_access = create_actor(app.clone(), &admin_token, "module-native-no-access", &[]).await;
    let scope_node_id = create_test_scope_node(&pool).await;
    force_scoped_assignment(&pool, scoped_reader.account_id, scope_node_id).await;

    for path in [
        "/administration/modules",
        "/administration/modules/tessara.forms",
        "/administration/modules/tessara.unknown-definition",
    ] {
        let (status, headers, _) = response_bytes(
            app.clone(),
            Request::builder()
                .uri(path)
                .body(Body::empty())
                .expect("valid anonymous native request"),
        )
        .await;
        assert_eq!(status, StatusCode::SEE_OTHER, "{path}");
        assert_eq!(
            headers
                .get(header::LOCATION)
                .and_then(|value| value.to_str().ok()),
            Some("/login"),
            "{path}"
        );
    }

    let (reader_status, reader_headers, reader_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", "/administration/modules", &reader.cookie, None),
    )
    .await;
    assert_eq!(reader_status, StatusCode::OK);
    assert_private_native_headers(&reader_headers);
    assert!(reader_html.contains("<title>Tessara Module Management</title>"));
    assert!(reader_html.contains("7 definitions"));
    assert!(reader_html.contains("Transitional — not independently deployable"));
    assert!(reader_html.contains("No Module Release"));
    assert!(reader_html.contains("No Module Instance"));
    assert!(reader_html.contains("Retired"));
    assert!(reader_html.contains("Read-only"));
    assert!(!reader_html.contains("Save navigation"));
    assert!(!reader_html.contains("Move earlier"));
    assert!(!reader_html.contains("/bridge/"));
    let reader_bootstrap = module_bootstrap(&reader_html);
    assert_eq!(reader_bootstrap["route"], "directory");
    assert_eq!(reader_bootstrap["access"]["can_read"], true);
    assert_eq!(reader_bootstrap["access"]["can_manage_navigation"], false);
    assert_eq!(
        reader_bootstrap["inventory"]["entries"]
            .as_array()
            .map(Vec::len),
        Some(7)
    );
    assert_eq!(
        reader_bootstrap["navigation_policy"]["policy"]["can_manage_navigation"],
        false
    );

    let (_, manager_headers, manager_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", "/administration/modules", &manager.cookie, None),
    )
    .await;
    assert_private_native_headers(&manager_headers);
    assert!(manager_html.contains("Save navigation"));
    assert!(manager_html.contains("Move earlier"));
    assert!(!manager_html.contains("Read-only"));
    assert!(!manager_html.contains("/bridge/"));
    let manager_bootstrap = module_bootstrap(&manager_html);
    assert_eq!(manager_bootstrap["route"], "directory");
    assert_eq!(manager_bootstrap["access"]["can_read"], true);
    assert_eq!(manager_bootstrap["access"]["can_manage_navigation"], true);
    assert_eq!(
        manager_bootstrap["navigation_policy"]["policy"]["can_manage_navigation"],
        true
    );

    let (_, _, admin_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", "/administration/modules", &admin_cookie, None),
    )
    .await;
    let admin_bootstrap = module_bootstrap(&admin_html);
    assert_eq!(admin_bootstrap["route"], "directory");
    assert_eq!(admin_bootstrap["access"]["can_read"], true);
    assert_eq!(admin_bootstrap["access"]["can_manage_navigation"], true);

    let forms_path = format!("/administration/modules/{FORMS_DEFINITION}");
    let (forms_status, forms_headers, forms_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", &forms_path, &reader.cookie, None),
    )
    .await;
    assert_eq!(forms_status, StatusCode::OK);
    assert_private_native_headers(&forms_headers);
    for exact_text in [
        "Transitional — not independently deployable",
        "No Module Release",
        "No Module Instance",
        "Feature Declarations",
        "Contracts",
        "Capabilities",
        "Dependencies",
        "Compatibility",
        "Configuration",
        "Readiness",
        "Health",
        "Findings",
        "Resources",
        "Destinations",
        "Navigation",
        "Open exact source descriptor",
    ] {
        assert!(
            forms_html.contains(exact_text),
            "Forms SSR should contain {exact_text:?}"
        );
    }
    assert_eq!(
        forms_html
            .matches("Not applicable — no Module Release/Instance")
            .count(),
        4,
        "each release/instance-dependent dimension should remain explicit"
    );
    for dimension in [
        "dependency",
        "compatibility",
        "configuration",
        "readiness",
        "health",
    ] {
        assert!(
            forms_html.contains(&format!("data-module-dimension=\"{dimension}\"")),
            "Forms SSR should keep the {dimension} dimension separate"
        );
    }
    assert!(!forms_html.contains(">Healthy<"));
    assert!(!forms_html.contains(">Unhealthy<"));
    assert!(!forms_html.contains("Create Module Release"));
    assert!(!forms_html.contains("Create Module Instance"));
    assert!(!forms_html.contains("/bridge/"));
    let forms_bootstrap = module_bootstrap(&forms_html);
    assert_eq!(forms_bootstrap["route"], "detail");
    assert_eq!(
        forms_bootstrap["detail"]["entry"]["descriptor"]["reserved_definition_id"],
        FORMS_DEFINITION
    );

    let migration_path = format!("/administration/modules/{MIGRATION_DEFINITION}");
    let (migration_status, _, migration_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", &migration_path, &reader.cookie, None),
    )
    .await;
    assert_eq!(migration_status, StatusCode::OK);
    assert!(migration_html.contains("Contribution retired"));
    assert!(migration_html.contains(
        "The roadmap identity is retired and no current in-process product surface exists."
    ));
    assert!(migration_html.contains("No Module Release"));
    assert!(migration_html.contains("No Module Instance"));
    assert!(!migration_html.contains("/bridge/"));
    let migration_bootstrap = module_bootstrap(&migration_html);
    assert_eq!(
        migration_bootstrap["detail"]["entry"]["descriptor"]["availability"],
        "retired"
    );

    let unknown_path = format!("/administration/modules/{UNKNOWN_DEFINITION}");
    let (unknown_status, unknown_headers, unknown_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", &unknown_path, &reader.cookie, None),
    )
    .await;
    assert_eq!(unknown_status, StatusCode::OK);
    assert_private_native_headers(&unknown_headers);
    assert!(unknown_html.contains("Module definition not found"));
    assert!(
        unknown_html.contains("No transition contribution exists for this definition identifier.")
    );
    assert!(!unknown_html.contains("/bridge/"));
    assert_eq!(
        module_bootstrap(&unknown_html),
        json!({
            "route": "not_found",
            "definition_id": UNKNOWN_DEFINITION
        })
    );

    let (scoped_known_status, scoped_known_headers, scoped_known_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", &forms_path, &scoped_reader.cookie, None),
    )
    .await;
    let (scoped_unknown_status, scoped_unknown_headers, scoped_unknown_html) = native_document(
        app.clone(),
        cookie_authenticated_request("GET", &unknown_path, &scoped_reader.cookie, None),
    )
    .await;
    assert_eq!(scoped_known_status, StatusCode::OK);
    assert_eq!(scoped_unknown_status, StatusCode::OK);
    assert_private_native_headers(&scoped_known_headers);
    assert_private_native_headers(&scoped_unknown_headers);
    assert_eq!(
        scoped_known_html, scoped_unknown_html,
        "restricted Module detail documents must be byte-identical for known and random definition identifiers"
    );
    let expected_restricted_detail = json!({
        "route": "restricted",
        "surface": "detail"
    });
    assert_eq!(
        module_bootstrap(&scoped_known_html),
        expected_restricted_detail
    );
    assert_eq!(
        module_bootstrap(&scoped_unknown_html),
        expected_restricted_detail
    );
    for html in [&scoped_known_html, &scoped_unknown_html] {
        assert!(html.contains("Module Management restricted"));
        assert!(html.contains("installation-global Module Management read access"));
        assert!(!html.contains("source_digest"));
        assert!(!html.contains("No Module Release"));
        assert!(!html.contains("/bridge/"));
    }

    for (name, actor) in [("product only", &product_only), ("no access", &no_access)] {
        let (status, _, html) = native_document(
            app.clone(),
            cookie_authenticated_request("GET", "/administration/modules", &actor.cookie, None),
        )
        .await;
        assert_eq!(status, StatusCode::OK, "{name}");
        assert_eq!(
            module_bootstrap(&html),
            json!({ "route": "restricted", "surface": "directory" }),
            "{name}"
        );
        assert!(html.contains("Module Management restricted"), "{name}");
    }
}

async fn create_actor(
    app: axum::Router,
    admin_token: &str,
    identity: &str,
    capability_keys: &[&str],
) -> Actor {
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
                .expect("capability list should be an array")
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
                "name": format!("{identity} role"),
                "capability_ids": capability_ids
            })),
        ),
    )
    .await;
    assert!(role["id"].as_str().is_some());

    let email = format!("{identity}@tessara.local");
    let user = request_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/users",
            admin_token,
            Some(json!({
                "email": email,
                "display_name": identity,
                "password": PASSWORD,
                "is_active": true,
                "role_ids": [role["id"]]
            })),
        ),
    )
    .await;
    let account_id = Uuid::parse_str(
        user["id"]
            .as_str()
            .expect("created user should expose its account id"),
    )
    .expect("created account id should be a UUID");
    let token = login_token_for(app.clone(), &email, PASSWORD).await;
    #[cfg(feature = "ssr")]
    let cookie = login_cookie_for(app, &email, PASSWORD).await;

    Actor {
        account_id,
        token,
        #[cfg(feature = "ssr")]
        cookie,
    }
}

async fn create_test_scope_node(pool: &PgPool) -> Uuid {
    let node_type_id: Uuid = sqlx::query_scalar(
        "INSERT INTO node_types (name, slug) VALUES ('Module Test Scope', 'module-test-scope') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("test scope node type should be created");
    sqlx::query_scalar(
        "INSERT INTO nodes (node_type_id, name) VALUES ($1, 'Module Test Scope Root') RETURNING id",
    )
    .bind(node_type_id)
    .fetch_one(pool)
    .await
    .expect("test scope node should be created")
}

async fn force_scoped_assignment(pool: &PgPool, account_id: Uuid, node_id: Uuid) {
    let result = sqlx::query(
        "UPDATE role_assignments SET node_id = $2 WHERE account_id = $1 AND node_id IS NULL",
    )
    .bind(account_id)
    .bind(node_id)
    .execute(pool)
    .await
    .expect("the scoped-only negative fixture should be injected");
    assert_eq!(
        result.rows_affected(),
        1,
        "the scoped-only fixture should replace exactly one global role assignment"
    );
}

fn inventory_entry<'a>(entries: &'a [Value], definition_id: &str) -> &'a Value {
    entries
        .iter()
        .find(|entry| entry["descriptor"]["reserved_definition_id"] == definition_id)
        .unwrap_or_else(|| panic!("missing module inventory entry {definition_id}"))
}

fn shell_item<'a>(shell: &'a Value, key: &str) -> Option<&'a Value> {
    shell["groups"]
        .as_array()?
        .iter()
        .filter_map(|group| group["items"].as_array())
        .flatten()
        .find(|item| item["key"] == key)
}

fn policy_update_request(policy: &Value) -> Value {
    json!({
        "schema_version": 2,
        "expected_revision": policy["revision"],
        "groups": policy["groups"]
            .as_array()
            .expect("policy groups should be an array")
            .iter()
            .map(|entry| json!({
                "id": entry["id"],
                "label": entry["label"],
                "order": entry["order"]
            }))
            .collect::<Vec<_>>(),
        "destinations": policy["destinations"]
            .as_array()
            .expect("policy destinations should be an array")
            .iter()
            .map(|entry| json!({
                "id": entry["id"],
                "group_id": entry["group_id"],
                "visible": entry["visible"],
                "order": entry["order"]
            }))
            .collect::<Vec<_>>()
    })
}

fn navigation_mutation_mut<'a>(request: &'a mut Value, destination_id: &str) -> &'a mut Value {
    request["destinations"]
        .as_array_mut()
        .expect("policy mutations should be an array")
        .iter_mut()
        .find(|entry| entry["id"] == destination_id)
        .unwrap_or_else(|| panic!("missing navigation mutation {destination_id}"))
}

fn navigation_group_mut<'a>(request: &'a mut Value, group_id: &str) -> &'a mut Value {
    request["groups"]
        .as_array_mut()
        .expect("policy groups should be an array")
        .iter_mut()
        .find(|entry| entry["id"] == group_id)
        .unwrap_or_else(|| panic!("missing navigation group {group_id}"))
}

fn policy_audit_groups(policy: &Value) -> Vec<Value> {
    policy["groups"]
        .as_array()
        .expect("policy groups should be an array")
        .iter()
        .map(|group| {
            json!({
                "id": group["id"],
                "label": group["label"],
                "order": group["order"],
            })
        })
        .collect()
}

fn policy_audit_placements(policy: &Value) -> Vec<Value> {
    policy["destinations"]
        .as_array()
        .expect("policy destinations should be an array")
        .iter()
        .map(|destination| {
            json!({
                "id": destination["id"],
                "group_id": destination["group_id"],
                "visible": destination["visible"],
                "order": destination["order"],
            })
        })
        .collect()
}

#[allow(clippy::too_many_arguments)]
async fn assert_navigation_policy_rejection(
    app: axum::Router,
    pool: &PgPool,
    manager: &Actor,
    request: Value,
    expected_policy: &Value,
    expected_status: StatusCode,
    expected_code: &str,
    expected_denial_count: i64,
    expected_success_count: i64,
    case_name: &str,
) {
    let presented_revision = request["expected_revision"].clone();
    let (status, body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            "/api/admin/navigation-policy",
            &manager.token,
            Some(request),
        ),
    )
    .await;
    assert_eq!(status, expected_status, "{case_name}");
    assert_eq!(body["code"], expected_code, "{case_name}");

    let policy_after = request_json(
        app,
        authorized_request("GET", "/api/admin/navigation-policy", &manager.token, None),
    )
    .await;
    assert_eq!(
        policy_after, *expected_policy,
        "{case_name} must preserve the complete policy and revision"
    );
    assert_eq!(
        control_plane_audit_count(pool, "navigation_policy.update_denied", manager.account_id,)
            .await,
        expected_denial_count,
        "{case_name} must add exactly one denial audit event"
    );
    assert_eq!(
        control_plane_audit_count(pool, "navigation_policy.updated", manager.account_id).await,
        expected_success_count,
        "{case_name} must not add a success audit event"
    );
    let denial_payload: Value = sqlx::query_scalar(
        r#"
        SELECT payload
        FROM core_control_plane_audit_events
        WHERE event_type = 'navigation_policy.update_denied'
          AND actor_account_id = $1
          AND payload ->> 'denial_code' = $2
        ORDER BY created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(manager.account_id)
    .bind(expected_code)
    .fetch_one(pool)
    .await
    .unwrap_or_else(|error| panic!("{case_name} denial audit should exist: {error}"));
    assert_eq!(
        denial_payload,
        json!({
            "schema_version": 1,
            "action": "navigation_policy.update",
            "presented_revision": presented_revision,
            "denial_code": expected_code,
            "success": false,
        }),
        "{case_name} denial audit payload"
    );
}

async fn control_plane_audit_count(pool: &PgPool, event_type: &str, actor_account_id: Uuid) -> i64 {
    sqlx::query_scalar(
        r#"
        SELECT COUNT(*)
        FROM core_control_plane_audit_events
        WHERE event_type = $1 AND actor_account_id = $2
        "#,
    )
    .bind(event_type)
    .bind(actor_account_id)
    .fetch_one(pool)
    .await
    .expect("control-plane audit count")
}

fn restricted_resolution(access_state: &str) -> Value {
    json!({
        "schema_version": 1,
        "access_state": access_state,
        "owner_state": { "kind": "undisclosed" },
        "resource_identity_state": "undisclosed",
        "resource_lifecycle_state": { "kind": "undisclosed" },
        "compatibility_state": "undisclosed",
        "availability_state": "undisclosed",
    })
}

fn authorized_unknown_resolution() -> Value {
    json!({
        "schema_version": 1,
        "access_state": "authorized",
        "owner_state": { "kind": "core_installation", "state": "live" },
        "resource_identity_state": "unknown_resource",
        "resource_lifecycle_state": { "kind": "not_evaluated" },
        "compatibility_state": "compatible",
        "availability_state": "available",
    })
}

fn response_reference(installation_id: &str, submission_id: Uuid) -> Value {
    json!({
        "installation_id": installation_id,
        "owner": {
            "kind": "core_installation",
            "installation_id": installation_id,
        },
        "resource_type": RESPONSE_RESOURCE_TYPE,
        "resource_id": submission_id.to_string(),
    })
}

async fn resolve_reference(app: axum::Router, token: &str, reference: Value) -> Value {
    request_json(
        app,
        authorized_request(
            "POST",
            "/api/platform/resource-references/resolve",
            token,
            Some(json!({ "schema_version": 1, "reference": reference })),
        ),
    )
    .await
}

#[cfg(not(debug_assertions))]
async fn sample_restricted_resolution_latencies(
    app: axum::Router,
    token: &str,
    known_reference: &Value,
    random_reference: &Value,
    expected: &Value,
    pair_count: usize,
) -> (Vec<Duration>, Vec<Duration>) {
    let mut known_samples = Vec::with_capacity(pair_count);
    let mut random_samples = Vec::with_capacity(pair_count);

    for pair_index in 0..pair_count {
        let order = if pair_index % 2 == 0 {
            [(known_reference, true), (random_reference, false)]
        } else {
            [(random_reference, false), (known_reference, true)]
        };
        for (reference, is_known) in order {
            let request = authorized_request(
                "POST",
                "/api/platform/resource-references/resolve",
                token,
                Some(json!({ "schema_version": 1, "reference": reference })),
            );
            let started = Instant::now();
            let (status, _, bytes) = response_bytes(app.clone(), request).await;
            let elapsed = started.elapsed();
            assert_eq!(status, StatusCode::OK);
            let body: Value = serde_json::from_slice(&bytes)
                .expect("timed resource resolution should return JSON");
            assert_eq!(&body, expected);
            if is_known {
                known_samples.push(elapsed);
            } else {
                random_samples.push(elapsed);
            }
        }
    }

    (known_samples, random_samples)
}

#[cfg(not(debug_assertions))]
fn assert_restricted_timing_profile(
    access_state: &str,
    known_samples: &[Duration],
    random_samples: &[Duration],
) {
    assert_eq!(
        known_samples.len(),
        RESTRICTED_TIMING_SAMPLES_PER_IDENTIFIER
    );
    assert_eq!(
        random_samples.len(),
        RESTRICTED_TIMING_SAMPLES_PER_IDENTIFIER
    );

    for (percentile_name, percentile) in [("median", 50), ("p95", 95)] {
        let known_ms = percentile_ms(known_samples, percentile);
        let random_ms = percentile_ms(random_samples, percentile);
        let delta_ms = (known_ms - random_ms).abs();
        let allowed_delta_ms = 2.0_f64.max(known_ms.min(random_ms) * 0.20);
        println!(
            "restricted resource timing access_state={access_state} percentile={percentile_name} known_ms={known_ms:.3} random_ms={random_ms:.3} delta_ms={delta_ms:.3} allowed_delta_ms={allowed_delta_ms:.3} samples_per_identifier={}",
            known_samples.len()
        );
        assert!(
            delta_ms <= allowed_delta_ms,
            "{access_state} {percentile_name} known/random latency delta was {delta_ms:.3} ms, above the fixed {allowed_delta_ms:.3} ms tolerance (larger of 2 ms or 20%)"
        );
    }
}

#[cfg(not(debug_assertions))]
fn percentile_ms(samples: &[Duration], percentile: usize) -> f64 {
    assert!(!samples.is_empty());
    assert!((1..=100).contains(&percentile));
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = (ordered.len() * percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)].as_secs_f64() * 1_000.0
}

async fn response_bytes(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, HeaderMap, Bytes) {
    let response = app
        .oneshot(request)
        .await
        .expect("router should produce a response");
    let status = response.status();
    let headers = response.headers().clone();
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response body should be readable");
    (status, headers, body)
}

#[cfg(feature = "ssr")]
async fn native_document(
    app: axum::Router,
    request: Request<Body>,
) -> (StatusCode, HeaderMap, String) {
    let (status, headers, body) = response_bytes(app, request).await;
    let html = String::from_utf8(body.to_vec()).expect("native document should be UTF-8");
    (status, headers, html)
}

#[cfg(feature = "ssr")]
fn assert_private_native_headers(headers: &HeaderMap) {
    assert_eq!(
        headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
        Some("text/html; charset=utf-8")
    );
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|value| value.to_str().ok()),
        Some("private, no-store")
    );
    assert_eq!(
        headers
            .get(header::VARY)
            .and_then(|value| value.to_str().ok()),
        Some("Cookie, Authorization")
    );
}

#[cfg(feature = "ssr")]
fn module_bootstrap(html: &str) -> Value {
    const OPEN: &str =
        r#"<script id="tessara-module-management-bootstrap" type="application/json">"#;
    let json_start = html.find(OPEN).unwrap_or_else(|| {
        panic!("native document should contain the Module Management bootstrap")
    }) + OPEN.len();
    let json_end = html[json_start..]
        .find("</script>")
        .map(|offset| json_start + offset)
        .expect("Module Management bootstrap script should close");
    serde_json::from_str(&html[json_start..json_end])
        .expect("Module Management bootstrap should be valid JSON")
}
