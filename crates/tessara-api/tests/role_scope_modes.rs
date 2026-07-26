#[allow(dead_code)]
mod support;

use axum::http::StatusCode;
use serde_json::{Value, json};
use sqlx::PgPool;
use tessara_api::router;
use uuid::Uuid;

use support::{
    TEST_DATABASE_LOCK, authorized_request, login_token, login_token_for, request_json,
    request_status_and_json, test_state,
};

const TEST_PASSWORD: &str = "role-scope-test-password-123";

#[tokio::test]
async fn role_scope_mode_mutations_reject_mixed_and_scoped_global_changes_atomically() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;
    let capabilities = capability_catalog(app.clone(), &admin_token).await;
    let admin_all = capability_id(&capabilities, "admin:all");
    let forms_read = capability_id(&capabilities, "forms:read");
    let modules_read = capability_id(&capabilities, "modules:read");

    let (mixed_create_status, mixed_create_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "POST",
            "/api/admin/roles",
            &admin_token,
            Some(json!({
                "name": "Rejected mixed role",
                "capability_ids": [forms_read, modules_read]
            })),
        ),
    )
    .await;
    assert_eq!(mixed_create_status, StatusCode::BAD_REQUEST);
    assert_eq!(mixed_create_body["code"], "mixed_capability_scope_modes");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM roles WHERE name = $1")
            .bind("Rejected mixed role")
            .fetch_one(&pool)
            .await
            .expect("rejected role count"),
        0,
        "a rejected mixed bundle must not create a role"
    );

    let scoped_role_id = create_role(
        app.clone(),
        &admin_token,
        "Atomic scoped role",
        &[forms_read],
    )
    .await;
    let (mixed_update_status, mixed_update_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/roles/{scoped_role_id}"),
            &admin_token,
            Some(json!({ "capability_ids": [forms_read, modules_read] })),
        ),
    )
    .await;
    assert_eq!(mixed_update_status, StatusCode::BAD_REQUEST);
    assert_eq!(mixed_update_body["code"], "mixed_capability_scope_modes");
    assert_eq!(
        role_capability_keys(&pool, scoped_role_id).await,
        vec!["forms:read".to_string()],
        "a rejected mixed update must leave the existing bundle intact"
    );

    let scoped_account_id = create_user(
        app.clone(),
        &admin_token,
        "atomic-scoped@example.test",
        "Atomic Scoped Account",
        &[scoped_role_id],
    )
    .await;
    let scope_node_id = create_scope_node(&pool, "atomic").await;
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{scoped_account_id}/access"),
            &admin_token,
            Some(json!({
                "scope_node_ids": [scope_node_id],
                "delegate_account_ids": []
            })),
        ),
    )
    .await;
    assert_eq!(
        role_assignment_nodes(&pool, scoped_account_id, scoped_role_id).await,
        vec![Some(scope_node_id)]
    );

    let (global_update_status, global_update_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/roles/{scoped_role_id}"),
            &admin_token,
            Some(json!({ "capability_ids": [modules_read] })),
        ),
    )
    .await;
    assert_eq!(global_update_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        global_update_body["code"],
        "global_capability_requires_global_role_assignment"
    );
    assert_eq!(
        role_capability_keys(&pool, scoped_role_id).await,
        vec!["forms:read".to_string()],
        "the rejected global conversion must not replace role capabilities"
    );
    assert_eq!(
        role_assignment_nodes(&pool, scoped_account_id, scoped_role_id).await,
        vec![Some(scope_node_id)],
        "the rejected global conversion must not rewrite scoped assignments"
    );

    let (scoped_admin_status, scoped_admin_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/roles/{scoped_role_id}"),
            &admin_token,
            Some(json!({ "capability_ids": [admin_all, forms_read] })),
        ),
    )
    .await;
    assert_eq!(scoped_admin_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        scoped_admin_body["code"],
        "global_capability_requires_global_role_assignment"
    );
    assert_eq!(
        role_capability_keys(&pool, scoped_role_id).await,
        vec!["forms:read".to_string()],
        "the rejected admin sentinel conversion must not replace role capabilities"
    );
    assert_eq!(
        role_assignment_nodes(&pool, scoped_account_id, scoped_role_id).await,
        vec![Some(scope_node_id)],
        "the rejected admin sentinel conversion must not rewrite scoped assignments"
    );

    let admin_exception_role_id = create_role(
        app.clone(),
        &admin_token,
        "Admin sentinel mixed exception",
        &[admin_all, forms_read],
    )
    .await;
    assert_eq!(
        role_capability_keys(&pool, admin_exception_role_id).await,
        vec!["admin:all".to_string(), "forms:read".to_string()],
        "admin:all is the sole allowed mixed-scope role bundle"
    );
    let updated_admin_exception = request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/roles/{admin_exception_role_id}"),
            &admin_token,
            Some(json!({
                "capability_ids": [admin_all, forms_read, modules_read]
            })),
        ),
    )
    .await;
    assert_eq!(
        updated_admin_exception["id"],
        admin_exception_role_id.to_string()
    );
    assert_eq!(
        role_capability_keys(&pool, admin_exception_role_id).await,
        vec![
            "admin:all".to_string(),
            "forms:read".to_string(),
            "modules:read".to_string(),
        ],
        "an updated mixed role remains valid only because it retains admin:all"
    );
    let admin_exception_account_id = create_user(
        app.clone(),
        &admin_token,
        "admin-exception@example.test",
        "Admin Exception Account",
        &[admin_exception_role_id],
    )
    .await;
    assert_eq!(
        role_assignment_nodes(&pool, admin_exception_account_id, admin_exception_role_id).await,
        vec![None],
        "a mixed role containing admin:all remains installation-global"
    );
    let (admin_exception_scope_status, admin_exception_scope_body) = request_status_and_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{admin_exception_account_id}/access"),
            &admin_token,
            Some(json!({
                "scope_node_ids": [scope_node_id],
                "delegate_account_ids": []
            })),
        ),
    )
    .await;
    assert_eq!(admin_exception_scope_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        admin_exception_scope_body["code"],
        "global_capability_requires_global_role_assignment"
    );
    assert_eq!(
        role_assignment_nodes(&pool, admin_exception_account_id, admin_exception_role_id).await,
        vec![None],
        "a rejected scope change must leave the admin exception global"
    );

    let global_role_id = create_role(
        app.clone(),
        &admin_token,
        "Global-only role",
        &[modules_read],
    )
    .await;
    let global_account_id = create_user(
        app.clone(),
        &admin_token,
        "global-only@example.test",
        "Global Only Account",
        &[global_role_id],
    )
    .await;
    assert_eq!(
        role_assignment_nodes(&pool, global_account_id, global_role_id).await,
        vec![None],
        "installation-global roles must start with a NULL node assignment"
    );

    let (scoped_global_status, scoped_global_body) = request_status_and_json(
        app,
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{global_account_id}/access"),
            &admin_token,
            Some(json!({
                "scope_node_ids": [scope_node_id],
                "delegate_account_ids": []
            })),
        ),
    )
    .await;
    assert_eq!(scoped_global_status, StatusCode::BAD_REQUEST);
    assert_eq!(
        scoped_global_body["code"],
        "global_capability_requires_global_role_assignment"
    );
    assert_eq!(
        role_assignment_nodes(&pool, global_account_id, global_role_id).await,
        vec![None],
        "a rejected explicit scope must leave the global assignment unchanged"
    );
}

#[tokio::test]
async fn global_and_scope_aware_roles_compose_without_widening_product_scope() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;
    let capabilities = capability_catalog(app.clone(), &admin_token).await;
    let forms_read = capability_id(&capabilities, "forms:read");
    let modules_read = capability_id(&capabilities, "modules:read");
    let scoped_role_id = create_role(
        app.clone(),
        &admin_token,
        "Composed product role",
        &[forms_read],
    )
    .await;
    let global_role_id = create_role(
        app.clone(),
        &admin_token,
        "Composed module role",
        &[modules_read],
    )
    .await;
    let account_id = create_user(
        app.clone(),
        &admin_token,
        "composed-scope@example.test",
        "Composed Scope Account",
        &[scoped_role_id],
    )
    .await;
    let scope_node_id = create_scope_node(&pool, "composition").await;
    let outside_scope_node_id = create_scope_node(&pool, "composition-outside").await;
    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{account_id}/access"),
            &admin_token,
            Some(json!({
                "scope_node_ids": [scope_node_id],
                "delegate_account_ids": []
            })),
        ),
    )
    .await;

    request_json(
        app.clone(),
        authorized_request(
            "PUT",
            &format!("/api/admin/users/{account_id}"),
            &admin_token,
            Some(json!({
                "email": "composed-scope@example.test",
                "display_name": "Composed Scope Account",
                "password": null,
                "is_active": true,
                "role_ids": [scoped_role_id, global_role_id]
            })),
        ),
    )
    .await;

    assert_eq!(
        role_assignment_nodes(&pool, account_id, scoped_role_id).await,
        vec![Some(scope_node_id)],
        "the product role must retain only its selected scope root"
    );
    assert_eq!(
        role_assignment_nodes(&pool, account_id, global_role_id).await,
        vec![None],
        "the installation-global role must persist only a NULL node row"
    );
    let product_has_global_assignment = sqlx::query_scalar::<_, bool>(
        r#"
        SELECT EXISTS(
            SELECT 1
            FROM role_assignments
            JOIN role_capabilities ON role_capabilities.role_id = role_assignments.role_id
            JOIN capabilities ON capabilities.id = role_capabilities.capability_id
            WHERE role_assignments.account_id = $1
              AND capabilities.key = 'forms:read'
              AND role_assignments.node_id IS NULL
        )
        "#,
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await
    .expect("product global-assignment check");
    assert!(
        !product_has_global_assignment,
        "adding a separate installation-global role must not widen product access"
    );

    let account_token =
        login_token_for(app.clone(), "composed-scope@example.test", TEST_PASSWORD).await;
    let session = request_json(
        app.clone(),
        authorized_request("GET", "/api/auth/session", &account_token, None),
    )
    .await;
    let account = &session["account"];
    assert_eq!(
        account["scope_nodes"][0]["node_id"],
        scope_node_id.to_string()
    );
    assert!(
        account["capabilities"]
            .as_array()
            .expect("session capabilities")
            .iter()
            .any(|capability| capability == "forms:read")
    );
    assert!(
        account["capabilities"]
            .as_array()
            .expect("session capabilities")
            .iter()
            .any(|capability| capability == "modules:read")
    );
    assert_eq!(
        account["global_capabilities"],
        json!(["modules:read"]),
        "the session wire must distinguish the global module grant from the scoped product grant"
    );
    request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/modules", &account_token, None),
    )
    .await;
    let inside_form_id = create_form(
        app.clone(),
        &admin_token,
        "Composed scope inside form",
        "composed-scope-inside-form",
        scope_node_id,
    )
    .await;
    let outside_form_id = create_form(
        app.clone(),
        &admin_token,
        "Composed scope outside form",
        "composed-scope-outside-form",
        outside_scope_node_id,
    )
    .await;
    let visible_forms = request_json(
        app,
        authorized_request("GET", "/api/forms", &account_token, None),
    )
    .await;
    let visible_form_ids = visible_forms
        .as_array()
        .expect("visible Forms response")
        .iter()
        .map(|form| {
            form["id"]
                .as_str()
                .expect("Form id")
                .parse::<Uuid>()
                .expect("Form UUID")
        })
        .collect::<Vec<_>>();
    assert!(
        visible_form_ids.contains(&inside_form_id),
        "the composed actor must see the Form attached to its scoped product role"
    );
    assert!(
        !visible_form_ids.contains(&outside_form_id),
        "the separate global module role must not disclose an out-of-scope Form"
    );
}

#[tokio::test]
async fn seeded_admin_is_a_global_admin_all_sentinel_with_effective_product_and_module_access() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;

    let rows = sqlx::query_as::<_, (String, String, Option<Uuid>)>(
        r#"
        SELECT capabilities.key, capabilities.scope_mode, role_assignments.node_id
        FROM roles
        JOIN role_capabilities ON role_capabilities.role_id = roles.id
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        JOIN role_assignments ON role_assignments.role_id = roles.id
        JOIN accounts ON accounts.id = role_assignments.account_id
        WHERE roles.name = 'admin'
          AND accounts.email = 'admin@tessara.local'
        ORDER BY capabilities.key, role_assignments.node_id
        "#,
    )
    .fetch_all(&pool)
    .await
    .expect("seeded admin role rows");
    assert_eq!(
        rows,
        vec![("admin:all".into(), "installation_global".into(), None)],
        "the built-in admin role must contain only one global sentinel grant"
    );

    let session = request_json(
        app.clone(),
        authorized_request("GET", "/api/auth/session", &admin_token, None),
    )
    .await;
    assert_eq!(session["account"]["capabilities"], json!(["admin:all"]));
    assert_eq!(
        session["account"]["global_capabilities"],
        json!([
            "admin:all",
            "hierarchy:manage",
            "hierarchy:read",
            "modules:manage_navigation",
            "modules:read"
        ])
    );

    request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/roles", &admin_token, None),
    )
    .await;
    request_json(
        app.clone(),
        authorized_request("GET", "/api/forms", &admin_token, None),
    )
    .await;
    request_json(
        app,
        authorized_request("GET", "/api/admin/modules", &admin_token, None),
    )
    .await;
}

async fn capability_catalog(app: axum::Router, admin_token: &str) -> Value {
    request_json(
        app,
        authorized_request("GET", "/api/admin/capabilities", admin_token, None),
    )
    .await
}

fn capability_id(capabilities: &Value, key: &str) -> Uuid {
    capabilities
        .as_array()
        .expect("capability catalog")
        .iter()
        .find(|capability| capability["key"] == key)
        .and_then(|capability| capability["id"].as_str())
        .unwrap_or_else(|| panic!("missing capability {key}"))
        .parse()
        .expect("capability id")
}

async fn create_role(
    app: axum::Router,
    admin_token: &str,
    name: &str,
    capability_ids: &[Uuid],
) -> Uuid {
    request_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/roles",
            admin_token,
            Some(json!({
                "name": name,
                "capability_ids": capability_ids
            })),
        ),
    )
    .await["id"]
        .as_str()
        .expect("created role id")
        .parse()
        .expect("role UUID")
}

async fn create_user(
    app: axum::Router,
    admin_token: &str,
    email: &str,
    display_name: &str,
    role_ids: &[Uuid],
) -> Uuid {
    request_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/users",
            admin_token,
            Some(json!({
                "email": email,
                "display_name": display_name,
                "password": TEST_PASSWORD,
                "is_active": true,
                "role_ids": role_ids
            })),
        ),
    )
    .await["id"]
        .as_str()
        .expect("created account id")
        .parse()
        .expect("account UUID")
}

async fn create_form(
    app: axum::Router,
    admin_token: &str,
    name: &str,
    slug: &str,
    visibility_node_id: Uuid,
) -> Uuid {
    request_json(
        app,
        authorized_request(
            "POST",
            "/api/admin/forms",
            admin_token,
            Some(json!({
                "name": name,
                "slug": slug,
                "scope_node_type_id": null,
                "visibility_node_ids": [visibility_node_id]
            })),
        ),
    )
    .await["id"]
        .as_str()
        .expect("created Form id")
        .parse()
        .expect("Form UUID")
}

async fn role_capability_keys(pool: &PgPool, role_id: Uuid) -> Vec<String> {
    sqlx::query_scalar(
        r#"
        SELECT capabilities.key
        FROM role_capabilities
        JOIN capabilities ON capabilities.id = role_capabilities.capability_id
        WHERE role_capabilities.role_id = $1
        ORDER BY capabilities.key
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await
    .expect("role capability keys")
}

async fn role_assignment_nodes(
    pool: &PgPool,
    account_id: Uuid,
    role_id: Uuid,
) -> Vec<Option<Uuid>> {
    sqlx::query_scalar::<_, Option<Uuid>>(
        r#"
        SELECT node_id
        FROM role_assignments
        WHERE account_id = $1
          AND role_id = $2
        ORDER BY node_id NULLS FIRST
        "#,
    )
    .bind(account_id)
    .bind(role_id)
    .fetch_all(pool)
    .await
    .expect("role assignment nodes")
}

async fn create_scope_node(pool: &PgPool, suffix: &str) -> Uuid {
    let node_type_id: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO node_types (name, slug)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(format!("Role Scope Test {suffix}"))
    .bind(format!("role-scope-test-{suffix}"))
    .fetch_one(pool)
    .await
    .expect("test scope node type");
    sqlx::query_scalar(
        r#"
        INSERT INTO nodes (node_type_id, name)
        VALUES ($1, $2)
        RETURNING id
        "#,
    )
    .bind(node_type_id)
    .bind(format!("Role Scope Root {suffix}"))
    .fetch_one(pool)
    .await
    .expect("test scope node")
}
