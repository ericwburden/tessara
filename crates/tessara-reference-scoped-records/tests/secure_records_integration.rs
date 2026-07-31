use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::postgres::PgPoolOptions;
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV1, CapabilityScopeBindingV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, NavigationContributionId,
    NavigationProjectionV1, OriginalActorProjectionV1, ProtocolSignaturePurposeV1,
    PurposeBoundSigningKeyV1, SecurityCapabilityId, ShellContextV1, ShellDocumentStateV1,
    ShellThemeV1,
};
use tessara_reference_scoped_records::{
    MANAGE_CAPABILITY, ModuleState, OrganizationAccessProjectionV1, READ_CAPABILITY, router,
};
use tower::ServiceExt;
use uuid::Uuid;

#[tokio::test]
async fn mutations_consume_replay_and_reads_filter_by_bound_organization() {
    let database_url = std::env::var("TEST_REFERENCE_MODULE_DATABASE_URL")
        .expect("TEST_REFERENCE_MODULE_DATABASE_URL is required for module integration tests");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("module test database is reachable");
    sqlx::migrate!().run(&pool).await.expect("migrations apply");

    let installation_id = Uuid::new_v4();
    let module_instance_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO scoped_records_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES (true,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(module_instance_id)
    .execute(&pool)
    .await
    .unwrap();
    let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "core-test-v1",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [81; 32],
    )
    .unwrap();
    let shell_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "core-test-v1",
        ProtocolSignaturePurposeV1::ShellContext,
        [81; 32],
    )
    .unwrap();
    let app = router(ModuleState {
        pool: pool.clone(),
        core_authorization_verifier: signer.verifier(),
        core_shell_verifier: shell_signer.verifier(),
    });
    let actor_id = Uuid::new_v4();
    let grant_context = GrantContext {
        installation_id,
        module_instance_id,
        actor_id,
    };
    let correlation_id = Uuid::new_v4();
    let now = Utc::now();
    let shell = shell_signer
        .sign(ShellContextV1 {
            schema_version: 1,
            installation_id,
            module_definition_id: ModuleDefinitionId::new(
                tessara_reference_scoped_records::MODULE_DEFINITION_ID,
            )
            .unwrap(),
            module_instance_id,
            original_actor: OriginalActorProjectionV1 {
                actor_id,
                display_name: "Scoped Reader".into(),
                email: None,
            },
            theme: ShellThemeV1::Dark,
            navigation: vec![NavigationProjectionV1 {
                contribution_id: NavigationContributionId::new("tessara.core.home").unwrap(),
                label: "Home".into(),
                href: "/".into(),
            }],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            correlation_id,
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .unwrap();
    let shell_header = URL_SAFE_NO_PAD.encode(serde_json::to_vec(&shell).unwrap());
    let shell_owner = Uuid::new_v4();
    let shell_grant = signed_grant(
        &signer,
        &grant_context,
        Uuid::new_v4(),
        "records.list",
        AuthorizationGrantOperationV1::Read,
        READ_CAPABILITY,
        shell_owner,
    );
    let shell_organizations = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&vec![OrganizationAccessProjectionV1 {
            organization_id: shell_owner,
            label: "Shell Test Organization".into(),
            can_manage: false,
        }])
        .unwrap(),
    );
    let shell_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/")
                .header("x-tessara-shell-context", shell_header)
                .header("x-tessara-correlation-id", correlation_id.to_string())
                .header("x-tessara-authorization", shell_grant)
                .header("x-tessara-organization-access", shell_organizations)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(shell_response.status(), StatusCode::OK);
    let shell_body = String::from_utf8(
        to_bytes(shell_response.into_body(), 1024 * 1024)
            .await
            .unwrap()
            .to_vec(),
    )
    .unwrap();
    assert!(shell_body.contains("data-shell-state=\"active\""));
    assert!(shell_body.contains("Scoped Reader"));
    let direct_page = app
        .clone()
        .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(
        direct_page.status(),
        StatusCode::NOT_FOUND,
        "direct module access must not disclose whether a product route exists"
    );
    let configuration_body = json!({
        "schema_version": 1,
        "display_label": "  Regional Records  ",
        "retention_mode": "retain_on_undeploy"
    });
    let public_configuration = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/configuration")
                .header("content-type", "application/json")
                .body(Body::from(configuration_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(public_configuration.status(), StatusCode::UNAUTHORIZED);
    let applied_configuration = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/configuration")
                .header("content-type", "application/json")
                .header(
                    "x-tessara-module-control-key",
                    "development-module-control-only",
                )
                .body(Body::from(configuration_body.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(applied_configuration.status(), StatusCode::OK);
    let stored_label: String = sqlx::query_scalar(
        "SELECT display_label FROM scoped_records_configuration WHERE singleton=true",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored_label, "Regional Records");
    let allowed_owner = Uuid::new_v4();
    let hidden_owner = Uuid::new_v4();
    let jti = Uuid::new_v4();
    let create_grant = signed_grant(
        &signer,
        &grant_context,
        jti,
        "records.create",
        AuthorizationGrantOperationV1::Mutation,
        MANAGE_CAPABILITY,
        allowed_owner,
    );
    let create_body = json!({
        "label": "Allowed record",
        "organization_owner_id": allowed_owner,
        "idempotency_key": "create-allowed-1"
    });
    let first = app
        .clone()
        .oneshot(module_request(
            "POST",
            "/api/records",
            &create_grant,
            create_body.clone(),
        ))
        .await
        .unwrap();
    assert_eq!(first.status(), StatusCode::CREATED);
    let repeated = app
        .clone()
        .oneshot(module_request(
            "POST",
            "/api/records",
            &create_grant,
            create_body,
        ))
        .await
        .unwrap();
    assert_eq!(repeated.status(), StatusCode::OK);
    let changed = app
        .clone()
        .oneshot(module_request(
            "POST",
            "/api/records",
            &create_grant,
            json!({
                "label": "Changed payload",
                "organization_owner_id": allowed_owner,
                "idempotency_key": "create-allowed-1"
            }),
        ))
        .await
        .unwrap();
    assert_eq!(changed.status(), StatusCode::NOT_FOUND);

    sqlx::query(
        "INSERT INTO scoped_records
         (id,label,scope,organization_owner_id)
         VALUES ($1,'Hidden record','test',$2)",
    )
    .bind(Uuid::new_v4())
    .bind(hidden_owner)
    .execute(&pool)
    .await
    .unwrap();
    let read_grant = signed_grant(
        &signer,
        &grant_context,
        Uuid::new_v4(),
        "records.list",
        AuthorizationGrantOperationV1::Read,
        READ_CAPABILITY,
        allowed_owner,
    );
    let response = app
        .oneshot(module_request(
            "GET",
            "/api/records",
            &read_grant,
            json!(null),
        ))
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    let body: Value =
        serde_json::from_slice(&to_bytes(response.into_body(), 1024 * 1024).await.unwrap())
            .unwrap();
    let records = body.as_array().unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(
        records[0]["organization_owner_id"],
        allowed_owner.to_string()
    );
}

struct GrantContext {
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
}

fn signed_grant(
    signer: &PurposeBoundSigningKeyV1,
    context: &GrantContext,
    jti: Uuid,
    action: &str,
    operation: AuthorizationGrantOperationV1,
    capability: &str,
    owner: Uuid,
) -> String {
    let now = Utc::now();
    let envelope = signer
        .sign(AuthorizationGrantV1 {
            schema_version: 1,
            installation_id: context.installation_id,
            original_actor_id: context.actor_id,
            presenting_service: ModuleDefinitionId::new("tessara.core").unwrap(),
            audience_module_instance_id: context.module_instance_id,
            dependency_binding: DependencyBindingKey::new("tessara.core.scoped-records").unwrap(),
            functional_contract: FunctionalContractId::new(
                "tessara.reference.scoped-records.record",
            )
            .unwrap(),
            action: action.into(),
            operation,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new(capability).unwrap(),
                organization_root_id: owner,
                authorized_organization_ids: vec![],
            }],
            resource_assertion: None,
            delegation_basis: vec![],
            authorization_revision: 7,
            organization_revision: 11,
            jti,
            issued_at: now,
            expires_at: now
                + Duration::seconds(match operation {
                    AuthorizationGrantOperationV1::Read => 60,
                    AuthorizationGrantOperationV1::Mutation => 30,
                }),
        })
        .unwrap();
    URL_SAFE_NO_PAD.encode(serde_json::to_vec(&envelope).unwrap())
}

fn module_request(method: &str, path: &str, authorization: &str, body: Value) -> Request<Body> {
    Request::builder()
        .method(method)
        .uri(path)
        .header("content-type", "application/json")
        .header("x-tessara-authorization", authorization)
        .body(Body::from(if method == "GET" {
            Vec::new()
        } else {
            serde_json::to_vec(&body).unwrap()
        }))
        .unwrap()
}
