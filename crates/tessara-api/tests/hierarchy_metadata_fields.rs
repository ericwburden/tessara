#[allow(dead_code)]
mod support;

use std::collections::BTreeSet;

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use serde_json::{Value, json};
use sqlx::PgPool;
use tessara_api::router;
use uuid::Uuid;

use support::{
    TEST_DATABASE_LOCK, authorized_request, login_token, login_token_for, request_json,
    request_status_and_json, test_state,
};

const PASSWORD: &str = "hierarchy-metadata-test-password-123";

struct Actor {
    account_id: Uuid,
    token: String,
}

#[derive(Debug, PartialEq, Eq)]
struct HierarchyContractSnapshot {
    node_types: Vec<String>,
    metadata_fields: Vec<String>,
    nodes: Vec<String>,
    metadata_values: Vec<String>,
    relationships: Vec<String>,
}

#[tokio::test]
async fn readable_node_type_metadata_fields_are_narrow_authorized_and_non_mutating() {
    let _guard = TEST_DATABASE_LOCK.lock().await;
    let state = test_state().await;
    let pool = state.pool.clone();
    let app = router(state);
    let admin_token = login_token(app.clone()).await;

    let node_type_id = create_node_type(
        app.clone(),
        &admin_token,
        "Metadata Contract Node",
        "metadata-contract-node",
        "Metadata Contract Nodes",
    )
    .await;
    let scope_node_type_id = create_node_type(
        app.clone(),
        &admin_token,
        "Metadata Scope Root",
        "metadata-scope-root",
        "Metadata Scope Roots",
    )
    .await;
    let unrelated_node_type_id = create_node_type(
        app.clone(),
        &admin_token,
        "Unrelated Metadata Node",
        "unrelated-metadata-node",
        "Unrelated Metadata Nodes",
    )
    .await;
    let zeta_field_id = create_metadata_field(
        app.clone(),
        &admin_token,
        node_type_id,
        "zeta_note",
        "Zeta note",
        "text",
        false,
    )
    .await;
    let alpha_field_id = create_metadata_field(
        app.clone(),
        &admin_token,
        node_type_id,
        "alpha_active",
        "Alpha active",
        "boolean",
        true,
    )
    .await;
    let unrelated_field_id = create_metadata_field(
        app.clone(),
        &admin_token,
        unrelated_node_type_id,
        "middle_hidden",
        "Middle hidden",
        "text",
        false,
    )
    .await;

    let scope_node_id = response_id(
        request_json(
            app.clone(),
            authorized_request(
                "POST",
                "/api/admin/nodes",
                &admin_token,
                Some(json!({
                    "node_type_id": scope_node_type_id,
                    "parent_node_id": null,
                    "name": "Metadata Contract Scope",
                    "metadata": {}
                })),
            ),
        )
        .await,
        "created scope node",
    );

    let capabilities = request_json(
        app.clone(),
        authorized_request("GET", "/api/admin/capabilities", &admin_token, None),
    )
    .await;
    let reader = create_actor(
        app.clone(),
        &admin_token,
        &capabilities,
        "hierarchy-metadata-reader",
        &["hierarchy:read"],
        Some(scope_node_id),
    )
    .await;
    let manager = create_actor(
        app.clone(),
        &admin_token,
        &capabilities,
        "hierarchy-metadata-manager",
        &["hierarchy:manage"],
        Some(scope_node_id),
    )
    .await;
    let no_access = create_actor(
        app.clone(),
        &admin_token,
        &capabilities,
        "hierarchy-metadata-no-access",
        &[],
        None,
    )
    .await;
    assert_eq!(
        role_assignment_nodes(&pool, reader.account_id).await,
        vec![Some(scope_node_id)],
        "the hierarchy reader must exercise a grant scoped outside the requested metadata type"
    );
    assert_eq!(
        role_assignment_nodes(&pool, manager.account_id).await,
        vec![Some(scope_node_id)],
        "the hierarchy manager must exercise manage-implies-read outside the requested metadata type"
    );

    let endpoint = format!("/api/node-types/{node_type_id}/metadata-fields");
    let expected = json!([
        {
            "id": alpha_field_id,
            "node_type_id": node_type_id,
            "node_type_name": "Metadata Contract Node",
            "key": "alpha_active",
            "label": "Alpha active",
            "field_type": "boolean",
            "required": true
        },
        {
            "id": zeta_field_id,
            "node_type_id": node_type_id,
            "node_type_name": "Metadata Contract Node",
            "key": "zeta_note",
            "label": "Zeta note",
            "field_type": "text",
            "required": false
        }
    ]);
    let unrelated_field_id = unrelated_field_id.to_string();
    let contract_node_type_ids = [node_type_id, scope_node_type_id, unrelated_node_type_id];
    let hierarchy_before = hierarchy_contract_snapshot(&pool, &contract_node_type_ids).await;

    for (actor_name, token) in [
        ("hierarchy reader", &reader.token),
        ("hierarchy manager", &manager.token),
        ("administrator", &admin_token),
    ] {
        let body = request_json(
            app.clone(),
            authorized_request("GET", &endpoint, token, None),
        )
        .await;
        assert_eq!(
            body, expected,
            "{actor_name} should receive the exact ordered metadata schema"
        );
        assert_exact_metadata_field_shape(&body);
        assert!(
            body.as_array()
                .expect("metadata field response should be an array")
                .iter()
                .all(|field| field["id"].as_str() != Some(unrelated_field_id.as_str())),
            "{actor_name} must not receive a field belonging to another node type"
        );
    }

    let fieldless_body = request_json(
        app.clone(),
        authorized_request(
            "GET",
            &format!("/api/node-types/{scope_node_type_id}/metadata-fields"),
            &reader.token,
            None,
        ),
    )
    .await;
    assert_eq!(
        fieldless_body,
        json!([]),
        "an existing fieldless node type must return an exact empty collection"
    );

    let (forbidden_status, forbidden_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &endpoint, &no_access.token, None),
    )
    .await;
    assert_error(
        forbidden_status,
        &forbidden_body,
        StatusCode::FORBIDDEN,
        "forbidden",
        "The current account is missing required capability 'hierarchy:read'.",
    );

    let (anonymous_status, anonymous_body) = request_status_and_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(&endpoint)
            .body(Body::empty())
            .expect("anonymous request"),
    )
    .await;
    assert_error(
        anonymous_status,
        &anonymous_body,
        StatusCode::UNAUTHORIZED,
        "auth_unauthorized",
        "Authentication is required.",
    );

    let unknown_node_type_id = Uuid::new_v4();
    let unknown_endpoint = format!("/api/node-types/{unknown_node_type_id}/metadata-fields");
    let (unknown_forbidden_status, unknown_forbidden_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &unknown_endpoint, &no_access.token, None),
    )
    .await;
    assert_eq!(
        (unknown_forbidden_status, unknown_forbidden_body),
        (forbidden_status, forbidden_body.clone()),
        "an unauthorized actor must receive the exact same status and body for known and random node-type identifiers"
    );

    let (unknown_anonymous_status, unknown_anonymous_body) = request_status_and_json(
        app.clone(),
        Request::builder()
            .method("GET")
            .uri(&unknown_endpoint)
            .body(Body::empty())
            .expect("anonymous unknown-node-type request"),
    )
    .await;
    assert_eq!(
        (unknown_anonymous_status, unknown_anonymous_body),
        (anonymous_status, anonymous_body.clone()),
        "an anonymous actor must receive the exact same status and body for known and random node-type identifiers"
    );

    let (not_found_status, not_found_body) = request_status_and_json(
        app.clone(),
        authorized_request("GET", &unknown_endpoint, &reader.token, None),
    )
    .await;
    assert_error(
        not_found_status,
        &not_found_body,
        StatusCode::NOT_FOUND,
        "not_found",
        &format!("node type {unknown_node_type_id}"),
    );

    for (actor_name, token) in [
        ("scoped hierarchy reader", &reader.token),
        ("scoped hierarchy manager", &manager.token),
    ] {
        let (status, body) = request_status_and_json(
            app.clone(),
            authorized_request(
                "GET",
                &format!("/api/admin/node-types/{node_type_id}"),
                token,
                None,
            ),
        )
        .await;
        assert_error(
            status,
            &body,
            StatusCode::FORBIDDEN,
            "forbidden",
            "The current account is missing required capability 'admin:all'.",
        );
        assert!(
            body.get("scoped_forms").is_none(),
            "{actor_name} must not receive any admin node-type definition data"
        );
    }

    assert_eq!(
        hierarchy_contract_snapshot(&pool, &contract_node_type_ids).await,
        hierarchy_before,
        "read-only metadata requests must preserve every complete hierarchy fixture row"
    );
}

async fn create_node_type(
    app: axum::Router,
    admin_token: &str,
    name: &str,
    slug: &str,
    plural_label: &str,
) -> Uuid {
    response_id(
        request_json(
            app,
            authorized_request(
                "POST",
                "/api/admin/node-types",
                admin_token,
                Some(json!({
                    "name": name,
                    "slug": slug,
                    "plural_label": plural_label
                })),
            ),
        )
        .await,
        "created node type",
    )
}

async fn create_metadata_field(
    app: axum::Router,
    admin_token: &str,
    node_type_id: Uuid,
    key: &str,
    label: &str,
    field_type: &str,
    required: bool,
) -> Uuid {
    response_id(
        request_json(
            app,
            authorized_request(
                "POST",
                "/api/admin/node-metadata-fields",
                admin_token,
                Some(json!({
                    "node_type_id": node_type_id,
                    "key": key,
                    "label": label,
                    "field_type": field_type,
                    "required": required
                })),
            ),
        )
        .await,
        "created metadata field",
    )
}

async fn create_actor(
    app: axum::Router,
    admin_token: &str,
    capabilities: &Value,
    identity: &str,
    capability_keys: &[&str],
    scope_node_id: Option<Uuid>,
) -> Actor {
    let capability_ids = capability_keys
        .iter()
        .map(|key| capability_id(capabilities, key))
        .collect::<Vec<_>>();
    let role_id = response_id(
        request_json(
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
        .await,
        "created role",
    );
    let email = format!("{identity}@tessara.local");
    let account_id = response_id(
        request_json(
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
                    "role_ids": [role_id]
                })),
            ),
        )
        .await,
        "created account",
    );

    if let Some(scope_node_id) = scope_node_id {
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
    }

    let token = login_token_for(app, &email, PASSWORD).await;
    Actor { account_id, token }
}

fn capability_id(capabilities: &Value, key: &str) -> Uuid {
    capabilities
        .as_array()
        .expect("capability catalog should be an array")
        .iter()
        .find(|capability| capability["key"] == key)
        .and_then(|capability| capability["id"].as_str())
        .unwrap_or_else(|| panic!("missing capability {key}"))
        .parse()
        .expect("capability id should be a UUID")
}

fn response_id(body: Value, context: &str) -> Uuid {
    body["id"]
        .as_str()
        .unwrap_or_else(|| panic!("{context} should expose an id"))
        .parse()
        .unwrap_or_else(|_| panic!("{context} id should be a UUID"))
}

fn assert_exact_metadata_field_shape(body: &Value) {
    let expected_keys = [
        "field_type",
        "id",
        "key",
        "label",
        "node_type_id",
        "node_type_name",
        "required",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();

    for field in body
        .as_array()
        .expect("metadata field response should be an array")
    {
        let actual_keys = field
            .as_object()
            .expect("metadata field should be an object")
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert_eq!(
            actual_keys, expected_keys,
            "metadata field response must not expose admin-only definition data"
        );
    }
}

fn assert_error(
    actual_status: StatusCode,
    actual_body: &Value,
    expected_status: StatusCode,
    code: &str,
    message: &str,
) {
    assert_eq!(
        actual_status, expected_status,
        "unexpected body: {actual_body}"
    );
    assert_eq!(
        actual_body,
        &json!({
            "code": code,
            "message": message,
            "error": message
        })
    );
}

async fn hierarchy_contract_snapshot(
    pool: &PgPool,
    node_type_ids: &[Uuid],
) -> HierarchyContractSnapshot {
    let node_types = sqlx::query_scalar::<_, String>(
        r#"
        SELECT to_jsonb(node_types)::text
        FROM node_types
        WHERE id = ANY($1)
        ORDER BY id
        "#,
    )
    .bind(node_type_ids)
    .fetch_all(pool)
    .await
    .expect("complete node type rows should load");
    let metadata_fields = sqlx::query_scalar::<_, String>(
        r#"
        SELECT to_jsonb(node_metadata_field_definitions)::text
        FROM node_metadata_field_definitions
        WHERE node_type_id = ANY($1)
        ORDER BY node_type_id, key, id
        "#,
    )
    .bind(node_type_ids)
    .fetch_all(pool)
    .await
    .expect("complete metadata field rows should load");
    let nodes = sqlx::query_scalar::<_, String>(
        r#"
        SELECT to_jsonb(nodes)::text
        FROM nodes
        WHERE node_type_id = ANY($1)
        ORDER BY id
        "#,
    )
    .bind(node_type_ids)
    .fetch_all(pool)
    .await
    .expect("complete node rows should load");
    let metadata_values = sqlx::query_scalar::<_, String>(
        r#"
        SELECT to_jsonb(node_metadata_values)::text
        FROM node_metadata_values
        JOIN nodes ON nodes.id = node_metadata_values.node_id
        WHERE nodes.node_type_id = ANY($1)
        ORDER BY node_metadata_values.node_id, node_metadata_values.field_definition_id
        "#,
    )
    .bind(node_type_ids)
    .fetch_all(pool)
    .await
    .expect("complete node metadata value rows should load");
    let relationships = sqlx::query_scalar::<_, String>(
        r#"
        SELECT to_jsonb(node_type_relationships)::text
        FROM node_type_relationships
        WHERE parent_node_type_id = ANY($1) OR child_node_type_id = ANY($1)
        ORDER BY parent_node_type_id, child_node_type_id
        "#,
    )
    .bind(node_type_ids)
    .fetch_all(pool)
    .await
    .expect("complete node type relationship rows should load");

    HierarchyContractSnapshot {
        node_types,
        metadata_fields,
        nodes,
        metadata_values,
        relationships,
    }
}

async fn role_assignment_nodes(pool: &PgPool, account_id: Uuid) -> Vec<Option<Uuid>> {
    sqlx::query_scalar(
        r#"
        SELECT node_id
        FROM role_assignments
        WHERE account_id = $1
        ORDER BY node_id NULLS FIRST
        "#,
    )
    .bind(account_id)
    .fetch_all(pool)
    .await
    .expect("role assignment snapshot should load")
}
