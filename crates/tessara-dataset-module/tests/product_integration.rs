use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde_json::Value;
use sqlx::Row;
use tessara_dataset_module::{
    DatasetCoreVerifiers, DatasetModuleState, DatasetServiceEndpoints, router,
};
use tessara_datasets_contract::{
    DATASET_IDEMPOTENCY_HEADER, DatasetProductRevisionDetailV1, DatasetProductRevisionStatusV1,
    DatasetProductRevisionSummaryV1, DatasetProductSummaryV1,
};
use tessara_module_contract::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, AuthorizationAudienceV1, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, BrowserLifecycleBootstrapV1, CapabilityScopeBindingV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ModuleServicePrincipalV1,
    NavigationContributionId, OriginalActorProjectionV1, ProtocolSignaturePurposeV1,
    PurposeBoundSigningKeyV1, SHELL_CONTEXT_SCHEMA_VERSION_V2, SecurityCapabilityId,
    ShellContextV2, ShellDocumentStateV1, ShellNavigationGroupProjectionV2,
    ShellNavigationItemProjectionV2, ShellThemeV1,
};
use tower::ServiceExt;
use uuid::Uuid;

fn dataset_test_app(pool: sqlx::PgPool) -> axum::Router {
    let core_signer = |purpose| {
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "dataset-configuration-test",
            purpose,
            [90; 32],
        )
        .expect("Core test signing key")
    };
    let service_identity_registry =
        tessara_module_contract::ModuleServiceIdentityRegistryV1::from_json(
            r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"test-component-v1","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
        )
        .expect("service identity registry");
    let service_signer = std::sync::Arc::new(
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.datasets",
            "dataset-configuration-test",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            [91; 32],
        )
        .expect("Dataset service signer"),
    );
    let receipt_signer = std::sync::Arc::new(
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.datasets",
            "dataset-configuration-test",
            ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
            [91; 32],
        )
        .expect("Dataset receipt signer"),
    );
    router(
        DatasetModuleState::new(
            pool,
            DatasetCoreVerifiers {
                authorization: core_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)
                    .verifier(),
                owner_bootstrap: core_signer(
                    ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
                )
                .verifier(),
                service_request: core_signer(ProtocolSignaturePurposeV1::ModuleServiceRequest)
                    .verifier(),
                bootstrap_validation: core_signer(
                    ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
                )
                .verifier(),
                shell: core_signer(ProtocolSignaturePurposeV1::ShellContext).verifier(),
            },
            service_identity_registry,
            service_signer,
            receipt_signer,
            DatasetServiceEndpoints::from_json(
                "http://127.0.0.1:1",
                r#"{
                    "tessara.datasets.response-export":"http://127.0.0.1:2",
                    "tessara.datasets.form-version-schema":"http://127.0.0.1:3",
                    "tessara.datasets.scope-catalog":"http://127.0.0.1:4",
                    "tessara.datasets.principal-display-catalog":"http://127.0.0.1:5"
                }"#,
            )
            .expect("Dataset service endpoints"),
            tessara_dataset_module::DatasetValidationFaultControl::disabled(),
        )
        .expect("Dataset state"),
    )
}

async fn get_json(app: &axum::Router, uri: &str, control_key: bool) -> (StatusCode, Value) {
    let mut request = Request::builder().uri(uri);
    if control_key {
        request = request.header(
            "x-tessara-module-control-key",
            "development-module-control-only",
        );
    }
    let response = app
        .clone()
        .oneshot(request.body(Body::empty()).expect("JSON request"))
        .await
        .expect("JSON response");
    let status = response.status();
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE),
        Some(&header::HeaderValue::from_static("application/json"))
    );
    assert!(response.headers().get(header::LOCATION).is_none());
    let value = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read JSON response"),
    )
    .expect("canonical JSON response");
    (status, value)
}

#[sqlx::test(migrations = "./migrations")]
async fn rejected_configuration_save_preserves_the_complete_stored_value(pool: sqlx::PgPool) {
    let app = dataset_test_app(pool.clone());
    for invalid in [
        serde_json::json!({
            "schema_version": 1,
            "display_label": "Partial",
            "provider_request_timeout_seconds": 7,
            "response_export_page_size": 500
        }),
        serde_json::json!({
            "schema_version": 1,
            "display_label": "Clamped",
            "provider_request_timeout_seconds": 31,
            "provider_retry_limit": 4,
            "response_export_page_size": 1001
        }),
        serde_json::json!({
            "schema_version": 1,
            "display_label": "Coerced",
            "provider_request_timeout_seconds": "5",
            "provider_retry_limit": 1,
            "response_export_page_size": 250
        }),
        serde_json::json!({
            "schema_version": 1,
            "display_label": "Unknown",
            "provider_request_timeout_seconds": 5,
            "provider_retry_limit": 1,
            "response_export_page_size": 250,
            "unknown": true
        }),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/configuration")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header(
                        "x-tessara-module-control-key",
                        "development-module-control-only",
                    )
                    .body(Body::from(serde_json::to_vec(&invalid).unwrap()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));
        let envelope: Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read configuration rejection"),
        )
        .expect("module-owned configuration rejection");
        assert_eq!(envelope["schema_version"], 1);
        assert!(envelope["error"]["code"].as_str().is_some());
        assert_ne!(envelope["correlation_id"], Value::Null);
    }

    for (media_type, body) in [
        ("text/plain", b"{}".as_slice()),
        ("application/json", b"{".as_slice()),
    ] {
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/configuration")
                    .header(header::CONTENT_TYPE, media_type)
                    .header(
                        "x-tessara-module-control-key",
                        "development-module-control-only",
                    )
                    .body(Body::from(body.to_vec()))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let envelope: Value = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read malformed configuration rejection"),
        )
        .expect("module-owned malformed configuration rejection");
        assert_eq!(envelope["error"]["code"], "dataset.malformed_request");
    }

    let unauthenticated = app
        .oneshot(
            Request::builder()
                .method("PUT")
                .uri("/api/configuration")
                .header(header::CONTENT_TYPE, "application/json")
                .body(Body::from("{"))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(unauthenticated.status(), StatusCode::FORBIDDEN);

    let stored = sqlx::query_as::<_, (i32, String, i32, i32, i32)>(
        "SELECT schema_version,display_label,provider_request_timeout_seconds,\
         provider_retry_limit,response_export_page_size \
         FROM dataset_configuration WHERE singleton=true",
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(stored, (1, "Datasets".into(), 5, 1, 250));
}

#[sqlx::test(migrations = "./migrations")]
async fn readiness_and_diagnostics_share_last_good_and_freshness_state(pool: sqlx::PgPool) {
    let app = dataset_test_app(pool.clone());
    let (status, missing_security) = get_json(&app, "/health/ready", false).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(missing_security["status"], "not_ready");
    assert_eq!(missing_security["database"], "ready");
    assert_eq!(missing_security["security_state"], "missing");
    assert_eq!(missing_security["configuration"], "valid");
    assert_eq!(missing_security["required_bindings"], "compatible");
    assert_eq!(missing_security["product_state"], "empty");
    assert_eq!(missing_security["freshness"], "not_applicable");
    assert_eq!(
        missing_security["failures"],
        serde_json::json!(["dataset.security_state.missing"])
    );

    let installation_id = Uuid::new_v4();
    let module_instance_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dataset_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES(true,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(module_instance_id)
    .execute(&pool)
    .await
    .expect("install Dataset security projection");

    let (status, controlled_bootstrap) = get_json(&app, "/health/ready", false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(controlled_bootstrap["status"], "ready");
    assert_eq!(controlled_bootstrap["product_state"], "empty");
    assert_eq!(controlled_bootstrap["freshness"], "not_applicable");
    assert_eq!(controlled_bootstrap["failures"], serde_json::json!([]));

    let dataset_id = Uuid::new_v4();
    let revision_id = Uuid::new_v4();
    let source_binding_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO datasets(id,name,slug,grain) VALUES($1,'Status','status','submission')",
    )
    .bind(dataset_id)
    .execute(&pool)
    .await
    .expect("insert status Dataset");
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,
          version_patch,semantic_bump,started_new_major_line,status,published_at,initial_source)
         VALUES($1,$2,1,'1.0.0',1,0,0,'initial',true,'published',now(),$3)",
    )
    .bind(revision_id)
    .bind(dataset_id)
    .bind(serde_json::json!({
        "kind": "form",
        "alias": "responses",
        "form_id": Uuid::new_v4().to_string(),
        "form_version_id": Uuid::new_v4().to_string()
    }))
    .execute(&pool)
    .await
    .expect("insert published status revision");
    sqlx::query(
        "INSERT INTO dataset_major_materializations(dataset_id,version_major,rebuild_status)
         VALUES($1,1,'pending')",
    )
    .bind(dataset_id)
    .execute(&pool)
    .await
    .expect("insert pending status materialization");
    sqlx::query(
        "INSERT INTO dataset_source_bindings
         (id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest)
         VALUES($1,$2,'responses','response_export',$3,$4)",
    )
    .bind(source_binding_id)
    .bind(dataset_id)
    .bind(serde_json::json!({"raw_reference":"must-not-appear-in-diagnostics"}))
    .bind(format!("sha256:{}", "a".repeat(64)))
    .execute(&pool)
    .await
    .expect("insert status source binding");
    sqlx::query(
        "INSERT INTO dataset_sync_partitions
         (source_binding_id,generation,last_checked_at,freshness_state,sanitized_failure_code)
         VALUES($1,0,now(),'failed','dataset.dependency_unavailable')",
    )
    .bind(source_binding_id)
    .execute(&pool)
    .await
    .expect("insert failed freshness observation");

    let (status, no_last_good) = get_json(&app, "/health/ready", false).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(no_last_good["status"], "not_ready");
    assert_eq!(no_last_good["product_state"], "last_good_missing");
    assert_eq!(no_last_good["freshness"], "failed");
    assert_eq!(
        no_last_good["failures"],
        serde_json::json!([
            "dataset.dependency_unavailable",
            "dataset.product.last_good_missing"
        ])
    );

    sqlx::query("CREATE TABLE dataset_materialized.status_fixture(id uuid PRIMARY KEY)")
        .execute(&pool)
        .await
        .expect("create status materialization table");
    sqlx::query(
        "UPDATE dataset_major_materializations
         SET materialized_schema='dataset_materialized',materialized_table='status_fixture',
             materialized_row_count=0,materialized_at=now(),rebuild_status='ready',updated_at=now()
         WHERE dataset_id=$1 AND version_major=1",
    )
    .bind(dataset_id)
    .execute(&pool)
    .await
    .expect("publish valid last-good status materialization");

    let (status, degraded) = get_json(&app, "/health/ready", false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(degraded["status"], "degraded");
    assert_eq!(degraded["product_state"], "last_good_available");
    assert_eq!(degraded["freshness"], "failed");
    assert_eq!(
        degraded["failures"],
        serde_json::json!(["dataset.dependency_unavailable"])
    );

    let (status, diagnostics) = get_json(&app, "/api/diagnostics", true).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(diagnostics["database"]["status"], "ready");
    assert_eq!(diagnostics["authorization"]["projection"], "installed");
    assert_eq!(diagnostics["product"]["state"], "last_good_available");
    assert_eq!(diagnostics["product"]["freshness"], "failed");
    assert!(
        diagnostics["product"]["last_materialization_at"]
            .as_str()
            .is_some()
    );
    let dependencies = diagnostics["dependencies"]
        .as_array()
        .expect("sanitized binding diagnostics");
    assert_eq!(dependencies.len(), 4);
    assert!(
        dependencies
            .iter()
            .all(|dependency| dependency["status"] == "compatible")
    );
    assert_eq!(
        dependencies[0]["binding_key"],
        "tessara.datasets.response-export"
    );
    assert_eq!(dependencies[0]["freshness"], "failed");
    assert_eq!(
        dependencies[0]["failure_code"],
        "dataset.dependency_unavailable"
    );
    assert!(dependencies[0]["last_observation_at"].as_str().is_some());
    for dependency in &dependencies[1..] {
        assert_eq!(dependency["freshness"], "not_observed");
        assert_eq!(dependency["last_observation_at"], Value::Null);
    }
    let encoded_diagnostics = serde_json::to_string(&diagnostics).expect("encode diagnostics");
    for forbidden in [
        "must-not-appear-in-diagnostics",
        "127.0.0.1",
        "source_identity",
        "committed_cursor",
        "materialized_row_count",
        &installation_id.to_string(),
        &module_instance_id.to_string(),
    ] {
        assert!(
            !encoded_diagnostics.contains(forbidden),
            "diagnostics leaked {forbidden}"
        );
    }

    sqlx::query(
        "UPDATE dataset_sync_partitions
         SET freshness_state='current',sanitized_failure_code=NULL,
             last_checked_at=now(),last_succeeded_at=now(),updated_at=now()
         WHERE source_binding_id=$1",
    )
    .bind(source_binding_id)
    .execute(&pool)
    .await
    .expect("record recovered freshness");
    let (status, recovered) = get_json(&app, "/health/ready", false).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(recovered["status"], "ready");
    assert_eq!(recovered["freshness"], "current");
    assert_eq!(recovered["failures"], serde_json::json!([]));
}

#[sqlx::test(migrations = "./migrations")]
async fn dataset_directory_is_module_owned_and_scope_filtered(pool: sqlx::PgPool) {
    let installation_id = Uuid::new_v4();
    let module_instance_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let allowed_scope = Uuid::new_v4();
    let hidden_scope = Uuid::new_v4();
    let visible_dataset = Uuid::new_v4();
    let hidden_dataset = Uuid::new_v4();

    sqlx::query(
        "INSERT INTO dataset_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES(true,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(module_instance_id)
    .execute(&pool)
    .await
    .expect("install Dataset security state");
    for (dataset_id, slug, scope) in [
        (visible_dataset, "visible", allowed_scope),
        (hidden_dataset, "hidden", hidden_scope),
    ] {
        sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,$2,$3,'submission')")
            .bind(dataset_id)
            .bind(slug)
            .bind(slug)
            .execute(&pool)
            .await
            .expect("insert Dataset");
        sqlx::query(
            "INSERT INTO dataset_scope_nodes
             (dataset_id,node_id,node_name,node_type_name,node_path,
              requested_set_revision,requested_set_digest)
             VALUES($1,$2,$3,'organization',$4,$5,$5)",
        )
        .bind(dataset_id)
        .bind(scope)
        .bind(slug)
        .bind(format!("/{slug}"))
        .bind(format!("sha256:{}", "a".repeat(64)))
        .execute(&pool)
        .await
        .expect("insert Dataset scope");
    }
    let visible_revision = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,
          version_patch,semantic_bump,started_new_major_line,status,initial_source)
         VALUES($1,$2,1,'1.0.0',1,0,0,'major',true,'published',$3)",
    )
    .bind(visible_revision)
    .bind(visible_dataset)
    .bind(serde_json::json!({
        "kind": "form",
        "alias": "responses",
        "form_id": Uuid::new_v4().to_string(),
        "form_version_id": Uuid::new_v4().to_string()
    }))
    .execute(&pool)
    .await
    .expect("insert published Dataset revision");
    let superseded_revision = Uuid::new_v4();
    let draft_revision = Uuid::new_v4();
    for (revision_id, number, label, status) in [
        (superseded_revision, 2, "1.1.0", "superseded"),
        (draft_revision, 3, "Draft 3", "draft"),
    ] {
        sqlx::query(
            "INSERT INTO dataset_revisions
             (id,dataset_id,version_number,version_label,status,initial_source,output_fields)
             VALUES($1,$2,$3,$4,$5,$6,$7)",
        )
        .bind(revision_id)
        .bind(visible_dataset)
        .bind(number)
        .bind(label)
        .bind(status)
        .bind(serde_json::json!({
            "kind": "form",
            "alias": "responses",
            "form_id": Uuid::new_v4().to_string(),
            "form_version_id": Uuid::new_v4().to_string()
        }))
        .bind(serde_json::json!([{
            "key": "program",
            "label": "Program",
            "source_alias": "responses",
            "source_field_key": "program",
            "field_type": "text",
            "position": 0
        }]))
        .execute(&pool)
        .await
        .expect("insert historical Dataset revision");
    }
    let downstream_dataset = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO datasets(id,name,slug,grain) VALUES($1,'Downstream','downstream','submission')",
    )
    .bind(downstream_dataset)
    .execute(&pool)
    .await
    .expect("insert downstream Dataset");
    sqlx::query(
        "INSERT INTO dataset_scope_nodes
         (dataset_id,node_id,node_name,node_type_name,node_path,
          requested_set_revision,requested_set_digest)
         VALUES($1,$2,'allowed','organization','/allowed',$3,$3)",
    )
    .bind(downstream_dataset)
    .bind(allowed_scope)
    .bind(format!("sha256:{}", "b".repeat(64)))
    .execute(&pool)
    .await
    .expect("insert downstream Dataset scope");
    let downstream_source_binding = Uuid::new_v4();
    let downstream_source_reference = serde_json::json!({
        "kind": "dataset",
        "alias": "upstream",
        "dataset_id": visible_dataset.to_string(),
        "dataset_revision_id": visible_revision.to_string()
    });
    sqlx::query(
        "INSERT INTO dataset_source_bindings
         (id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest)
         VALUES($1,$2,'upstream','dataset_major_line',$3,$4)",
    )
    .bind(downstream_source_binding)
    .bind(downstream_dataset)
    .bind(&downstream_source_reference)
    .bind(format!("sha256:{}", "c".repeat(64)))
    .execute(&pool)
    .await
    .expect("insert downstream Dataset source binding");
    sqlx::query(
        "INSERT INTO dataset_sync_partitions(source_binding_id,freshness_state)
         VALUES($1,'current')",
    )
    .bind(downstream_source_binding)
    .execute(&pool)
    .await
    .expect("insert downstream Dataset source partition");
    sqlx::query(
        "INSERT INTO dataset_sources
         (id,dataset_id,source_binding_id,source_alias,source_kind,source_reference,
          source_name,source_scope_node_ids,source_scope_revision,source_scope_digest,
          source_content_revision,source_content_digest,position)
         VALUES($1,$2,$3,'upstream','dataset_revision',$4,'visible',$5,$6,$6,$7,$7,0)",
    )
    .bind(Uuid::new_v4())
    .bind(downstream_dataset)
    .bind(downstream_source_binding)
    .bind(downstream_source_reference)
    .bind(vec![allowed_scope])
    .bind(format!("sha256:{}", "d".repeat(64)))
    .bind(format!("sha256:{}", "e".repeat(64)))
    .execute(&pool)
    .await
    .expect("insert downstream Dataset source");

    let authorization_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "dataset-product-test",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [31; 32],
    )
    .expect("authorization signing key");
    let core_service_request_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "dataset-product-test",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [31; 32],
    )
    .expect("Core service-request signing key");
    let shell_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "dataset-product-test",
        ProtocolSignaturePurposeV1::ShellContext,
        [32; 32],
    )
    .expect("shell signing key");
    let bootstrap_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "dataset-product-test",
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        [31; 32],
    )
    .expect("bootstrap signing key");
    let owner_bootstrap_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "dataset-product-test",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [31; 32],
    )
    .expect("owner bootstrap signing key");
    let service_identity_registry =
        tessara_module_contract::ModuleServiceIdentityRegistryV1::from_json(
            r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"test-component-v1","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
        )
        .expect("service identity registry");
    let service_signer = std::sync::Arc::new(
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.datasets",
            "dataset-product-test",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            [33; 32],
        )
        .expect("service signing key"),
    );
    let receipt_signer = std::sync::Arc::new(
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.datasets",
            "dataset-product-test",
            ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
            [33; 32],
        )
        .expect("bootstrap receipt signing key"),
    );
    let app = router(
        DatasetModuleState::new(
            pool.clone(),
            DatasetCoreVerifiers {
                authorization: authorization_signer.verifier(),
                owner_bootstrap: owner_bootstrap_signer.verifier(),
                service_request: core_service_request_signer.verifier(),
                bootstrap_validation: bootstrap_signer.verifier(),
                shell: shell_signer.verifier(),
            },
            service_identity_registry,
            service_signer,
            receipt_signer,
            DatasetServiceEndpoints::from_json(
                "http://127.0.0.1:1",
                r#"{
                    "tessara.datasets.response-export":"http://127.0.0.1:2",
                    "tessara.datasets.form-version-schema":"http://127.0.0.1:3",
                    "tessara.datasets.scope-catalog":"http://127.0.0.1:4",
                    "tessara.datasets.principal-display-catalog":"http://127.0.0.1:5"
                }"#,
            )
            .expect("Dataset service endpoints"),
            tessara_dataset_module::DatasetValidationFaultControl::disabled(),
        )
        .expect("Dataset state"),
    );
    let readiness = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/health/ready")
                .body(Body::empty())
                .expect("Dataset readiness request"),
        )
        .await
        .expect("Dataset readiness response");
    assert_eq!(readiness.status(), StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(
        serde_json::from_slice::<Value>(
            &to_bytes(readiness.into_body(), usize::MAX)
                .await
                .expect("read Dataset readiness response"),
        )
        .expect("canonical Dataset readiness response"),
        serde_json::json!({
            "schema_version": 1,
            "module_definition_id": "tessara.datasets",
            "module_release_version": tessara_dataset_module::MODULE_RELEASE_VERSION,
            "status": "not_ready",
            "database": "ready",
            "security_state": "installed",
            "configuration": "valid",
            "required_bindings": "compatible",
            "product_state": "last_good_missing",
            "freshness": "current",
            "failures": ["dataset.product.last_good_missing"]
        })
    );
    let correlation_id = Uuid::new_v4();
    let now = Utc::now();
    let authorization = authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id,
            original_actor_id: actor_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("Dataset definition id"),
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                .expect("Dataset Core binding"),
            functional_contract: FunctionalContractId::new("tessara.datasets.dataset-major-line")
                .expect("Dataset resource contract"),
            action: "datasets.list".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:read")
                    .expect("Dataset read capability"),
                organization_root_id: allowed_scope,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .expect("sign Dataset grant");
    let encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&authorization).expect("serialize Dataset authorization grant"));
    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri("/api/datasets")
                .header("x-tessara-authorization", encoded)
                .header("x-tessara-correlation-id", correlation_id.to_string())
                .body(Body::empty())
                .expect("Dataset directory request"),
        )
        .await
        .expect("Dataset directory response");
    assert_eq!(response.status(), StatusCode::OK);
    let body = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read Dataset directory body");
    let datasets: Vec<DatasetProductSummaryV1> =
        serde_json::from_slice(&body).expect("canonical Dataset directory response");
    assert_eq!(datasets.len(), 2);
    let visible_summary = datasets
        .iter()
        .find(|dataset| dataset.id == visible_dataset.to_string())
        .expect("visible Dataset summary");
    assert_eq!(
        visible_summary.visibility_nodes[0].node_id,
        allowed_scope.to_string()
    );

    let detail_correlation_id = Uuid::new_v4();
    let detail_now = Utc::now();
    let detail_authorization = authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id,
            original_actor_id: actor_id,
            correlation_id: detail_correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("Dataset definition id"),
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                .expect("Dataset Core binding"),
            functional_contract: FunctionalContractId::new("tessara.datasets.dataset-major-line")
                .expect("Dataset resource contract"),
            action: "datasets.get".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:read")
                    .expect("Dataset read capability"),
                organization_root_id: allowed_scope,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: detail_now,
            expires_at: detail_now + Duration::seconds(60),
        })
        .expect("sign Dataset detail grant");
    let detail_encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&detail_authorization).expect("serialize Dataset detail grant"));
    let detail = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/datasets/{visible_dataset}"))
                .header("x-tessara-authorization", &detail_encoded)
                .header(
                    "x-tessara-correlation-id",
                    detail_correlation_id.to_string(),
                )
                .body(Body::empty())
                .expect("Dataset detail request"),
        )
        .await
        .expect("Dataset detail response");
    assert_eq!(detail.status(), StatusCode::OK);
    let definition: tessara_datasets_contract::DatasetProductDefinitionV1 = serde_json::from_slice(
        &to_bytes(detail.into_body(), usize::MAX)
            .await
            .expect("read Dataset detail"),
    )
    .expect("canonical Dataset detail response");
    assert_eq!(definition.id, visible_dataset.to_string());
    assert_eq!(
        definition.current_revision_id,
        Some(visible_revision.to_string())
    );

    let shell = shell_signer
        .sign(ShellContextV2 {
            schema_version: SHELL_CONTEXT_SCHEMA_VERSION_V2,
            installation_id,
            module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                .expect("Dataset definition id"),
            module_instance_id,
            original_actor: OriginalActorProjectionV1 {
                actor_id,
                display_name: "Dataset Reader".into(),
                email: None,
            },
            theme: ShellThemeV1::System,
            navigation: vec![ShellNavigationGroupProjectionV2 {
                id: "core.main".into(),
                label: "Main".into(),
                items: vec![ShellNavigationItemProjectionV2 {
                    contribution_id: NavigationContributionId::new("tessara.datasets.navigation")
                        .expect("Dataset navigation contribution"),
                    key: "datasets".into(),
                    label: "Datasets".into(),
                    href: "/datasets".into(),
                }],
            }],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            correlation_id: detail_correlation_id,
            document_state: ShellDocumentStateV1::Active,
            issued_at: detail_now,
            expires_at: detail_now + Duration::seconds(60),
        })
        .expect("sign Dataset shell context");
    let shell_encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&shell).expect("serialize Dataset shell context"));
    let before_document_gets = owner_write_fingerprint(&pool).await;
    let direct_document = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/datasets/{visible_dataset}"))
                .header("x-tessara-authorization", &detail_encoded)
                .header("x-tessara-shell-context", &shell_encoded)
                .header(
                    "x-tessara-correlation-id",
                    detail_correlation_id.to_string(),
                )
                .body(Body::empty())
                .expect("direct Dataset document request"),
        )
        .await
        .expect("direct Dataset document response");
    assert_eq!(direct_document.status(), StatusCode::OK);
    assert_eq!(
        direct_document.headers()[header::CACHE_CONTROL],
        "private, no-store"
    );
    let direct_html = String::from_utf8(
        to_bytes(direct_document.into_body(), usize::MAX)
            .await
            .expect("read direct Dataset document")
            .to_vec(),
    )
    .expect("Dataset document UTF-8");
    assert!(direct_html.contains(">visible</h2>"));
    assert!(!direct_html.contains("Loading dataset"));
    let after_direct_document = owner_write_fingerprint(&pool).await;
    assert_eq!(after_direct_document, before_document_gets);

    let lifecycle_document = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/datasets/{visible_dataset}"))
                .header(
                    header::ACCEPT,
                    "application/vnd.tessara.module-view+json; version=1",
                )
                .header("x-tessara-authorization", &detail_encoded)
                .header("x-tessara-shell-context", &shell_encoded)
                .header(
                    "x-tessara-correlation-id",
                    detail_correlation_id.to_string(),
                )
                .body(Body::empty())
                .expect("lifecycle Dataset document request"),
        )
        .await
        .expect("lifecycle Dataset document response");
    assert_eq!(lifecycle_document.status(), StatusCode::OK);
    assert_eq!(
        lifecycle_document.headers()[header::CONTENT_TYPE],
        "application/vnd.tessara.module-view+json; version=1"
    );
    assert_eq!(
        lifecycle_document.headers()[header::CACHE_CONTROL],
        "private, no-store"
    );
    let lifecycle: BrowserLifecycleBootstrapV1 = serde_json::from_slice(
        &to_bytes(lifecycle_document.into_body(), usize::MAX)
            .await
            .expect("read Dataset lifecycle document"),
    )
    .expect("canonical Dataset lifecycle bootstrap");
    assert_eq!(lifecycle.definition_id.as_str(), "tessara.datasets");
    assert_eq!(lifecycle.destination.as_str(), "datasets.detail");
    assert_eq!(lifecycle.path, format!("/datasets/{visible_dataset}"));
    assert_eq!(lifecycle.lifecycle_abi.to_string(), "1.0.0");
    assert!(lifecycle.entry_asset.url.ends_with("/dataset.js"));
    assert_eq!(lifecycle.stylesheet_assets.len(), 1);
    assert!(
        lifecycle.stylesheet_assets[0]
            .url
            .ends_with("/dataset-lifecycle.css")
    );
    assert_eq!(lifecycle.payload["route"], "detail");
    assert_eq!(lifecycle.payload["dataset"]["name"], "visible");
    assert_ne!(lifecycle.payload["table_error"], Value::Null);
    assert_eq!(owner_write_fingerprint(&pool).await, before_document_gets);

    let document_credentials = |action: &str, contract: &str, include_manage: bool| {
        let correlation_id = Uuid::new_v4();
        let now = Utc::now();
        let mut bindings = vec![CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new("datasets:read")
                .expect("Dataset read capability"),
            organization_root_id: allowed_scope,
            authorized_organization_ids: Vec::new(),
        }];
        if include_manage {
            bindings.push(CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:manage")
                    .expect("Dataset manage capability"),
                organization_root_id: allowed_scope,
                authorized_organization_ids: Vec::new(),
            });
        }
        let authorization = authorization_signer
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id,
                original_actor_id: actor_id,
                correlation_id,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id,
                    module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                        .expect("Dataset definition id"),
                },
                dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                    .expect("Dataset Core binding"),
                functional_contract: FunctionalContractId::new(contract)
                    .expect("Dataset document contract"),
                action: action.into(),
                operation: AuthorizationGrantOperationV1::Read,
                capability_scope_bindings: bindings,
                resource_assertion: None,
                delegation_basis: Vec::new(),
                authorization_revision: 7,
                organization_revision: 11,
                jti: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + Duration::seconds(60),
            })
            .expect("sign Dataset document grant");
        let shell = shell_signer
            .sign(ShellContextV2 {
                schema_version: SHELL_CONTEXT_SCHEMA_VERSION_V2,
                installation_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("Dataset definition id"),
                module_instance_id,
                original_actor: OriginalActorProjectionV1 {
                    actor_id,
                    display_name: "Dataset Reader".into(),
                    email: None,
                },
                theme: ShellThemeV1::System,
                navigation: Vec::new(),
                return_destination: "/".into(),
                locale: "en-US".into(),
                time_zone: "UTC".into(),
                correlation_id,
                document_state: ShellDocumentStateV1::Active,
                issued_at: now,
                expires_at: now + Duration::seconds(60),
            })
            .expect("sign Dataset document shell context");
        (
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&authorization).expect("serialize Dataset document grant"),
            ),
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&shell).expect("serialize Dataset document shell context"),
            ),
            correlation_id,
        )
    };
    let document_cases = [
        (
            "/datasets".to_string(),
            "datasets.list",
            "tessara.datasets.dataset-major-line",
            false,
            "datasets.directory",
            "directory",
        ),
        (
            "/datasets/new".to_string(),
            "datasets.create",
            "tessara.datasets.authoring",
            true,
            "datasets.create",
            "create",
        ),
        (
            format!("/datasets/{visible_dataset}"),
            "datasets.get",
            "tessara.datasets.dataset-major-line",
            false,
            "datasets.detail",
            "detail",
        ),
        (
            format!("/datasets/{visible_dataset}/preview"),
            "datasets.preview_table",
            "tessara.datasets.dataset-major-line",
            false,
            "datasets.preview",
            "preview",
        ),
        (
            format!("/datasets/{visible_dataset}/edit"),
            "datasets.get_revision",
            "tessara.datasets.authoring",
            true,
            "datasets.edit",
            "edit",
        ),
        (
            format!("/datasets/{visible_dataset}/revisions"),
            "datasets.list_revisions",
            "tessara.datasets.dataset-major-line",
            true,
            "datasets.revisions",
            "revisions",
        ),
        (
            format!("/datasets/{visible_dataset}/revisions/{draft_revision}"),
            "datasets.get_revision",
            "tessara.datasets.dataset-major-line",
            true,
            "datasets.revision_detail",
            "revision_detail",
        ),
        (
            format!("/datasets/{visible_dataset}/revisions/{draft_revision}/edit"),
            "datasets.get_revision",
            "tessara.datasets.authoring",
            true,
            "datasets.revision_edit",
            "revision_edit",
        ),
    ];
    for (path, action, contract, include_manage, expected_destination, expected_route) in
        document_cases
    {
        let (authorization, shell, correlation_id) =
            document_credentials(action, contract, include_manage);
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri(&path)
                    .header(
                        header::ACCEPT,
                        "application/vnd.tessara.module-view+json; version=1",
                    )
                    .header("x-tessara-authorization", authorization)
                    .header("x-tessara-shell-context", shell)
                    .header("x-tessara-correlation-id", correlation_id.to_string())
                    .body(Body::empty())
                    .expect("Dataset lifecycle destination request"),
            )
            .await
            .expect("Dataset lifecycle destination response");
        assert_eq!(response.status(), StatusCode::OK, "{path}");
        let projection: BrowserLifecycleBootstrapV1 = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read Dataset lifecycle destination response"),
        )
        .expect("canonical Dataset lifecycle destination response");
        assert_eq!(
            projection.destination.as_str(),
            expected_destination,
            "{path}"
        );
        assert_eq!(projection.payload["route"], expected_route, "{path}");
    }
    assert_eq!(owner_write_fingerprint(&pool).await, before_document_gets);

    let hidden = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/datasets/{hidden_dataset}"))
                .header("x-tessara-authorization", &detail_encoded)
                .header(
                    "x-tessara-correlation-id",
                    detail_correlation_id.to_string(),
                )
                .body(Body::empty())
                .expect("hidden Dataset detail request"),
        )
        .await
        .expect("hidden Dataset detail response");
    assert_eq!(hidden.status(), StatusCode::NOT_FOUND);
    let hidden_body = to_bytes(hidden.into_body(), usize::MAX)
        .await
        .expect("read hidden Dataset response");
    let random = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/datasets/{}", Uuid::new_v4()))
                .header("x-tessara-authorization", &detail_encoded)
                .header(
                    "x-tessara-correlation-id",
                    detail_correlation_id.to_string(),
                )
                .body(Body::empty())
                .expect("random Dataset detail request"),
        )
        .await
        .expect("random Dataset detail response");
    assert_eq!(random.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        hidden_body,
        to_bytes(random.into_body(), usize::MAX)
            .await
            .expect("read random Dataset response")
    );

    let revision_grant = |action: &str, include_manage: bool| {
        let correlation_id = Uuid::new_v4();
        let now = Utc::now();
        let mut bindings = vec![CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new("datasets:read")
                .expect("Dataset read capability"),
            organization_root_id: allowed_scope,
            authorized_organization_ids: Vec::new(),
        }];
        if include_manage {
            bindings.push(CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:manage")
                    .expect("Dataset manage capability"),
                organization_root_id: allowed_scope,
                authorized_organization_ids: Vec::new(),
            });
        }
        let grant = authorization_signer
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id,
                original_actor_id: actor_id,
                correlation_id,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id,
                    module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                        .expect("Dataset definition id"),
                },
                dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                    .expect("Dataset Core binding"),
                functional_contract: FunctionalContractId::new(
                    "tessara.datasets.dataset-major-line",
                )
                .expect("Dataset resource contract"),
                action: action.into(),
                operation: AuthorizationGrantOperationV1::Read,
                capability_scope_bindings: bindings,
                resource_assertion: None,
                delegation_basis: Vec::new(),
                authorization_revision: 7,
                organization_revision: 11,
                jti: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + Duration::seconds(60),
            })
            .expect("sign Dataset revision grant");
        (
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&grant).expect("serialize Dataset revision authorization grant"),
            ),
            correlation_id,
        )
    };
    let (reader_revisions_grant, reader_revisions_correlation) =
        revision_grant("datasets.list_revisions", false);
    let reader_revisions_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/datasets/{visible_dataset}/revisions"))
                .header("x-tessara-authorization", reader_revisions_grant)
                .header(
                    "x-tessara-correlation-id",
                    reader_revisions_correlation.to_string(),
                )
                .body(Body::empty())
                .expect("reader Dataset revisions request"),
        )
        .await
        .expect("reader Dataset revisions response");
    assert_eq!(reader_revisions_response.status(), StatusCode::OK);
    let reader_revisions: Vec<DatasetProductRevisionSummaryV1> = serde_json::from_slice(
        &to_bytes(reader_revisions_response.into_body(), usize::MAX)
            .await
            .expect("read reader Dataset revisions"),
    )
    .expect("canonical reader Dataset revision summaries");
    assert_eq!(reader_revisions.len(), 2);
    assert!(
        reader_revisions
            .iter()
            .all(|revision| revision.status != DatasetProductRevisionStatusV1::Draft)
    );

    let (manager_revisions_grant, manager_revisions_correlation) =
        revision_grant("datasets.list_revisions", true);
    let manager_revisions_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/datasets/{visible_dataset}/revisions"))
                .header("x-tessara-authorization", manager_revisions_grant)
                .header(
                    "x-tessara-correlation-id",
                    manager_revisions_correlation.to_string(),
                )
                .body(Body::empty())
                .expect("manager Dataset revisions request"),
        )
        .await
        .expect("manager Dataset revisions response");
    let manager_revisions: Vec<DatasetProductRevisionSummaryV1> = serde_json::from_slice(
        &to_bytes(manager_revisions_response.into_body(), usize::MAX)
            .await
            .expect("read manager Dataset revisions"),
    )
    .expect("canonical manager Dataset revision summaries");
    assert_eq!(manager_revisions.len(), 3);
    assert_eq!(manager_revisions[0].id, draft_revision.to_string());

    let (manager_revision_grant, manager_revision_correlation) =
        revision_grant("datasets.get_revision", true);
    let manager_revision_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/datasets/{visible_dataset}/revisions/{draft_revision}"
                ))
                .header("x-tessara-authorization", manager_revision_grant)
                .header(
                    "x-tessara-correlation-id",
                    manager_revision_correlation.to_string(),
                )
                .body(Body::empty())
                .expect("manager Dataset revision request"),
        )
        .await
        .expect("manager Dataset revision response");
    assert_eq!(manager_revision_response.status(), StatusCode::OK);
    let manager_revision: DatasetProductRevisionDetailV1 = serde_json::from_slice(
        &to_bytes(manager_revision_response.into_body(), usize::MAX)
            .await
            .expect("read manager Dataset revision"),
    )
    .expect("canonical manager Dataset revision detail");
    assert_eq!(manager_revision.id, draft_revision.to_string());
    assert_eq!(manager_revision.dependencies.dataset_count, 1);
    assert_eq!(manager_revision.dependency_impacts.len(), 1);

    let (reader_revision_grant, reader_revision_correlation) =
        revision_grant("datasets.get_revision", false);
    let reader_draft_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/datasets/{visible_dataset}/revisions/{draft_revision}"
                ))
                .header("x-tessara-authorization", &reader_revision_grant)
                .header(
                    "x-tessara-correlation-id",
                    reader_revision_correlation.to_string(),
                )
                .body(Body::empty())
                .expect("reader draft Dataset revision request"),
        )
        .await
        .expect("reader draft Dataset revision response");
    assert_eq!(reader_draft_response.status(), StatusCode::NOT_FOUND);
    let reader_draft_body = to_bytes(reader_draft_response.into_body(), usize::MAX)
        .await
        .expect("read hidden draft Dataset revision response");
    let random_revision_response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/datasets/{visible_dataset}/revisions/{}",
                    Uuid::new_v4()
                ))
                .header("x-tessara-authorization", reader_revision_grant)
                .header(
                    "x-tessara-correlation-id",
                    reader_revision_correlation.to_string(),
                )
                .body(Body::empty())
                .expect("random Dataset revision request"),
        )
        .await
        .expect("random Dataset revision response");
    assert_eq!(random_revision_response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        reader_draft_body,
        to_bytes(random_revision_response.into_body(), usize::MAX)
            .await
            .expect("read random Dataset revision response")
    );

    let tag_correlation_id = Uuid::new_v4();
    let tag_now = Utc::now();
    let tag_authorization = authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id,
            original_actor_id: actor_id,
            correlation_id: tag_correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("Dataset definition id"),
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                .expect("Dataset Core binding"),
            functional_contract: FunctionalContractId::new("tessara.datasets.authoring")
                .expect("Dataset authoring contract"),
            action: "datasets.update_tags".into(),
            operation: AuthorizationGrantOperationV1::Mutation,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:manage")
                    .expect("Dataset manage capability"),
                organization_root_id: allowed_scope,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: tag_now,
            expires_at: tag_now + Duration::seconds(30),
        })
        .expect("sign Dataset tag grant");
    let tag_grant = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&tag_authorization).expect("serialize Dataset tag grant"));
    let tag_body = serde_json::to_vec(&serde_json::json!({
        "tags": [" Outcomes ", "outcomes", "2026"]
    }))
    .expect("serialize Dataset tags");
    let retired_header_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/admin/datasets/{visible_dataset}/tags"))
                .header("content-type", "application/json")
                .header("idempotency-key", "retired-dataset-header")
                .header("x-tessara-authorization", &tag_grant)
                .header("x-tessara-correlation-id", tag_correlation_id.to_string())
                .body(Body::from(tag_body.clone()))
                .expect("retired Dataset idempotency header request"),
        )
        .await
        .expect("retired Dataset idempotency header response");
    assert_eq!(retired_header_response.status(), StatusCode::BAD_REQUEST);
    let retired_header_error: tessara_datasets_contract::DatasetErrorEnvelope =
        serde_json::from_slice(
            &to_bytes(retired_header_response.into_body(), usize::MAX)
                .await
                .expect("read retired Dataset idempotency header response"),
        )
        .expect("canonical retired Dataset idempotency header error");
    assert_eq!(retired_header_error.error.code, "dataset.malformed_request");
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dataset_idempotency_receipts")
            .fetch_one(&pool)
            .await
            .expect("count receipts after retired Dataset idempotency header"),
        0
    );
    let tag_request = |body: Vec<u8>| {
        Request::builder()
            .method("PATCH")
            .uri(format!("/api/admin/datasets/{visible_dataset}/tags"))
            .header("content-type", "application/json")
            .header(DATASET_IDEMPOTENCY_HEADER, "dataset-tags-replay")
            .header("x-tessara-authorization", &tag_grant)
            .header("x-tessara-correlation-id", tag_correlation_id.to_string())
            .body(Body::from(body))
            .expect("Dataset tag request")
    };
    let first_tag_response = app
        .clone()
        .oneshot(tag_request(tag_body.clone()))
        .await
        .expect("first Dataset tag response");
    assert_eq!(first_tag_response.status(), StatusCode::OK);
    let first_tag_body = to_bytes(first_tag_response.into_body(), usize::MAX)
        .await
        .expect("read first Dataset tag response");
    let tag_result: tessara_datasets_contract::DatasetMutationIdResponseV1 =
        serde_json::from_slice(&first_tag_body).expect("canonical Dataset tag response");
    assert_eq!(tag_result.id, visible_dataset.to_string());
    let replay_tag_response = app
        .clone()
        .oneshot(tag_request(tag_body))
        .await
        .expect("replayed Dataset tag response");
    assert_eq!(replay_tag_response.status(), StatusCode::OK);
    assert_eq!(
        first_tag_body,
        to_bytes(replay_tag_response.into_body(), usize::MAX)
            .await
            .expect("read replayed Dataset tag response")
    );
    let stored_tags = sqlx::query_scalar::<_, String>(
        "SELECT tag FROM dataset_tags WHERE dataset_id=$1 ORDER BY tag",
    )
    .bind(visible_dataset)
    .fetch_all(&pool)
    .await
    .expect("read stored Dataset tags");
    assert_eq!(stored_tags, vec!["2026", "Outcomes"]);
    let mismatch_response = app
        .clone()
        .oneshot(tag_request(
            serde_json::to_vec(&serde_json::json!({"tags": ["different"]}))
                .expect("serialize mismatched Dataset tags"),
        ))
        .await
        .expect("mismatched Dataset tag response");
    assert_eq!(mismatch_response.status(), StatusCode::CONFLICT);
    let mismatch: tessara_datasets_contract::DatasetErrorEnvelope = serde_json::from_slice(
        &to_bytes(mismatch_response.into_body(), usize::MAX)
            .await
            .expect("read mismatched Dataset tag response"),
    )
    .expect("canonical Dataset idempotency error");
    assert_eq!(mismatch.error.code, "dataset.idempotency_mismatch");
    assert_eq!(mismatch.correlation_id, tag_correlation_id);
    let wrong_media_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method("PATCH")
                .uri(format!("/api/admin/datasets/{visible_dataset}/tags"))
                .header("content-type", "text/plain")
                .header(DATASET_IDEMPOTENCY_HEADER, "dataset-tags-wrong-media")
                .header("x-tessara-authorization", &tag_grant)
                .header("x-tessara-correlation-id", tag_correlation_id.to_string())
                .body(Body::from(r#"{"tags":["must-not-write"]}"#))
                .expect("wrong-media Dataset tag request"),
        )
        .await
        .expect("wrong-media Dataset tag response");
    assert_eq!(wrong_media_response.status(), StatusCode::BAD_REQUEST);
    let wrong_media: tessara_datasets_contract::DatasetErrorEnvelope = serde_json::from_slice(
        &to_bytes(wrong_media_response.into_body(), usize::MAX)
            .await
            .expect("read wrong-media Dataset tag response"),
    )
    .expect("canonical Dataset malformed request error");
    assert_eq!(wrong_media.error.code, "dataset.malformed_request");
    assert_eq!(
        sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM dataset_idempotency_receipts
             WHERE actor_id=$1 AND action='datasets.update_tags'",
        )
        .bind(actor_id)
        .fetch_one(&pool)
        .await
        .expect("count Dataset tag receipts"),
        1
    );

    let mutation_grant = |action: &str| {
        let correlation_id = Uuid::new_v4();
        let now = Utc::now();
        let authorization = authorization_signer
            .sign(AuthorizationGrantV3 {
                schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
                installation_id,
                original_actor_id: actor_id,
                correlation_id,
                presenting_service: ModuleServicePrincipalV1::CoreGateway,
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id,
                    module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                        .expect("Dataset definition id"),
                },
                dependency_binding: DependencyBindingKey::new("tessara.core.datasets")
                    .expect("Dataset Core binding"),
                functional_contract: FunctionalContractId::new("tessara.datasets.authoring")
                    .expect("Dataset authoring contract"),
                action: action.into(),
                operation: AuthorizationGrantOperationV1::Mutation,
                capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("datasets:manage")
                        .expect("Dataset manage capability"),
                    organization_root_id: allowed_scope,
                    authorized_organization_ids: Vec::new(),
                }],
                resource_assertion: None,
                delegation_basis: Vec::new(),
                authorization_revision: 7,
                organization_revision: 11,
                jti: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + Duration::seconds(30),
            })
            .expect("sign Dataset mutation grant");
        (
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&authorization)
                    .expect("serialize Dataset mutation authorization grant"),
            ),
            correlation_id,
        )
    };
    let (label_grant, label_correlation_id) = mutation_grant("datasets.update_revision_label");
    let label_body = serde_json::to_vec(&serde_json::json!({
        "version_label": " Published outcome ",
        "revision_notes": " Reviewed by owner "
    }))
    .expect("serialize Dataset revision label");
    let label_request = |key: &str, body: Vec<u8>| {
        Request::builder()
            .method("PATCH")
            .uri(format!(
                "/api/admin/datasets/{visible_dataset}/revisions/{visible_revision}/label"
            ))
            .header("content-type", "application/json")
            .header(DATASET_IDEMPOTENCY_HEADER, key)
            .header("x-tessara-authorization", &label_grant)
            .header("x-tessara-correlation-id", label_correlation_id.to_string())
            .body(Body::from(body))
            .expect("Dataset revision label request")
    };
    let cross_action_response = app
        .clone()
        .oneshot(label_request("dataset-tags-replay", label_body.clone()))
        .await
        .expect("cross-action replay response");
    assert_eq!(cross_action_response.status(), StatusCode::CONFLICT);
    let cross_action_error: tessara_datasets_contract::DatasetErrorEnvelope =
        serde_json::from_slice(
            &to_bytes(cross_action_response.into_body(), usize::MAX)
                .await
                .expect("read cross-action replay response"),
        )
        .expect("canonical cross-action replay error");
    assert_eq!(
        cross_action_error.error.code,
        "dataset.idempotency_mismatch"
    );
    let first_label_response = app
        .clone()
        .oneshot(label_request("dataset-label-replay", label_body.clone()))
        .await
        .expect("first Dataset revision label response");
    assert_eq!(first_label_response.status(), StatusCode::OK);
    let first_label_body = to_bytes(first_label_response.into_body(), usize::MAX)
        .await
        .expect("read first Dataset revision label response");
    let label_result: tessara_datasets_contract::DatasetRevisionLabelResponseV1 =
        serde_json::from_slice(&first_label_body)
            .expect("canonical Dataset revision label response");
    assert_eq!(label_result.version_label, "Published outcome");
    assert_eq!(label_result.revision_notes, "Reviewed by owner");
    let replay_label_response = app
        .clone()
        .oneshot(label_request("dataset-label-replay", label_body))
        .await
        .expect("replayed Dataset revision label response");
    assert_eq!(
        first_label_body,
        to_bytes(replay_label_response.into_body(), usize::MAX)
            .await
            .expect("read replayed Dataset revision label response")
    );

    let (options_grant, options_correlation_id) =
        mutation_grant("datasets.update_revision_options");
    let options_body = serde_json::to_vec(&serde_json::json!({
        "force_new_major_version": true
    }))
    .expect("serialize Dataset revision options");
    let options_request = |revision_id: Uuid, key: &str, body: Vec<u8>| {
        Request::builder()
            .method("PATCH")
            .uri(format!(
                "/api/admin/datasets/{visible_dataset}/revisions/{revision_id}/options"
            ))
            .header("content-type", "application/json")
            .header(DATASET_IDEMPOTENCY_HEADER, key)
            .header("x-tessara-authorization", &options_grant)
            .header(
                "x-tessara-correlation-id",
                options_correlation_id.to_string(),
            )
            .body(Body::from(body))
            .expect("Dataset revision options request")
    };
    let published_options_response = app
        .clone()
        .oneshot(options_request(
            visible_revision,
            "dataset-published-options",
            options_body.clone(),
        ))
        .await
        .expect("published Dataset revision options response");
    assert_eq!(
        published_options_response.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let first_options_response = app
        .clone()
        .oneshot(options_request(
            draft_revision,
            "dataset-options-replay",
            options_body.clone(),
        ))
        .await
        .expect("first Dataset revision options response");
    assert_eq!(first_options_response.status(), StatusCode::OK);
    let first_options_body = to_bytes(first_options_response.into_body(), usize::MAX)
        .await
        .expect("read first Dataset revision options response");
    let options_detail: DatasetProductRevisionDetailV1 =
        serde_json::from_slice(&first_options_body)
            .expect("canonical Dataset revision options response");
    assert!(options_detail.force_new_major_version);
    assert_eq!(options_detail.version_major, Some(2));
    assert_eq!(options_detail.version_minor, Some(0));
    assert_eq!(options_detail.version_patch, Some(0));
    assert_eq!(
        options_detail.semantic_bump,
        Some(tessara_datasets_contract::DatasetProductSemanticBumpV1::Major)
    );
    assert_eq!(options_detail.started_new_major_line, Some(true));
    let replay_options_response = app
        .clone()
        .oneshot(options_request(
            draft_revision,
            "dataset-options-replay",
            options_body,
        ))
        .await
        .expect("replayed Dataset revision options response");
    assert_eq!(
        first_options_body,
        to_bytes(replay_options_response.into_body(), usize::MAX)
            .await
            .expect("read replayed Dataset revision options response")
    );
    let stored_options = sqlx::query(
        "SELECT force_new_major_version,version_major,version_minor,version_patch,semantic_bump
         FROM dataset_revisions WHERE id=$1",
    )
    .bind(draft_revision)
    .fetch_one(&pool)
    .await
    .expect("read stored Dataset revision options");
    assert!(stored_options.get::<bool, _>("force_new_major_version"));
    assert_eq!(
        stored_options.get::<Option<i32>, _>("version_major"),
        Some(2)
    );
    assert_eq!(
        stored_options.get::<Option<i32>, _>("version_minor"),
        Some(0)
    );
    assert_eq!(
        stored_options.get::<Option<i32>, _>("version_patch"),
        Some(0)
    );
    assert_eq!(
        stored_options.get::<Option<String>, _>("semantic_bump"),
        Some("major".into())
    );

    let (delete_grant, delete_correlation_id) = mutation_grant("datasets.delete_revision");
    let delete_request = |revision_id: Uuid, key: &str| {
        Request::builder()
            .method("DELETE")
            .uri(format!(
                "/api/admin/datasets/{visible_dataset}/revisions/{revision_id}"
            ))
            .header(DATASET_IDEMPOTENCY_HEADER, key)
            .header("x-tessara-authorization", &delete_grant)
            .header(
                "x-tessara-correlation-id",
                delete_correlation_id.to_string(),
            )
            .body(Body::empty())
            .expect("Dataset revision delete request")
    };
    let published_delete_response = app
        .clone()
        .oneshot(delete_request(visible_revision, "dataset-published-delete"))
        .await
        .expect("published Dataset revision delete response");
    assert_eq!(
        published_delete_response.status(),
        StatusCode::UNPROCESSABLE_ENTITY
    );
    let published_delete_error: tessara_datasets_contract::DatasetErrorEnvelope =
        serde_json::from_slice(
            &to_bytes(published_delete_response.into_body(), usize::MAX)
                .await
                .expect("read published Dataset revision delete response"),
        )
        .expect("canonical Dataset validation error");
    assert_eq!(
        published_delete_error.error.code,
        "dataset.validation_failed"
    );
    let first_delete_response = app
        .clone()
        .oneshot(delete_request(draft_revision, "dataset-draft-delete"))
        .await
        .expect("first draft Dataset revision delete response");
    assert_eq!(first_delete_response.status(), StatusCode::OK);
    let first_delete_body = to_bytes(first_delete_response.into_body(), usize::MAX)
        .await
        .expect("read first draft Dataset revision delete response");
    let replay_delete_response = app
        .clone()
        .oneshot(delete_request(draft_revision, "dataset-draft-delete"))
        .await
        .expect("replayed draft Dataset revision delete response");
    assert_eq!(replay_delete_response.status(), StatusCode::OK);
    assert_eq!(
        first_delete_body,
        to_bytes(replay_delete_response.into_body(), usize::MAX)
            .await
            .expect("read replayed draft Dataset revision delete response")
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dataset_revisions WHERE id=$1",)
            .bind(draft_revision)
            .fetch_one(&pool)
            .await
            .expect("count deleted draft Dataset revision"),
        0
    );
    let (dataset_delete_grant, dataset_delete_correlation_id) = mutation_grant("datasets.delete");
    let dataset_delete_request = || {
        Request::builder()
            .method("DELETE")
            .uri(format!("/api/admin/datasets/{downstream_dataset}"))
            .header(DATASET_IDEMPOTENCY_HEADER, "dataset-delete-replay")
            .header("x-tessara-authorization", &dataset_delete_grant)
            .header(
                "x-tessara-correlation-id",
                dataset_delete_correlation_id.to_string(),
            )
            .body(Body::empty())
            .expect("Dataset delete request")
    };
    let first_dataset_delete_response = app
        .clone()
        .oneshot(dataset_delete_request())
        .await
        .expect("first Dataset delete response");
    assert_eq!(first_dataset_delete_response.status(), StatusCode::OK);
    let first_dataset_delete_body = to_bytes(first_dataset_delete_response.into_body(), usize::MAX)
        .await
        .expect("read first Dataset delete response");
    let replay_dataset_delete_response = app
        .clone()
        .oneshot(dataset_delete_request())
        .await
        .expect("replayed Dataset delete response");
    assert_eq!(replay_dataset_delete_response.status(), StatusCode::OK);
    assert_eq!(
        first_dataset_delete_body,
        to_bytes(replay_dataset_delete_response.into_body(), usize::MAX)
            .await
            .expect("read replayed Dataset delete response")
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM datasets WHERE id=$1")
            .bind(downstream_dataset)
            .fetch_one(&pool)
            .await
            .expect("count deleted Dataset"),
        0
    );

    let unsigned = app
        .oneshot(
            Request::builder()
                .uri("/api/datasets")
                .body(Body::empty())
                .expect("unsigned Dataset request"),
        )
        .await
        .expect("unsigned Dataset response");
    assert_eq!(unsigned.status(), StatusCode::FORBIDDEN);
    let unsigned_body: Value = serde_json::from_slice(
        &to_bytes(unsigned.into_body(), usize::MAX)
            .await
            .expect("read unsigned response"),
    )
    .expect("Dataset error envelope");
    assert_eq!(
        unsigned_body["error"]["code"],
        "dataset.not_found_or_forbidden"
    );
    assert_ne!(unsigned_body["correlation_id"], Uuid::nil().to_string());
}

async fn owner_write_fingerprint(pool: &sqlx::PgPool) -> Value {
    sqlx::query_scalar::<_, Value>(
        r#"
        SELECT jsonb_build_object(
          'security_state', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_security_state t),
          'configuration', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_configuration t),
          'datasets', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM datasets t),
          'scope_nodes', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_scope_nodes t),
          'tags', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_tags t),
          'revisions', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_revisions t),
          'revision_scopes', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_revision_scope_nodes t),
          'revision_sources', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_revision_sources t),
          'major_materializations', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_major_materializations t),
          'sources', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_sources t),
          'fields', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_fields t),
          'source_bindings', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_source_bindings t),
          'consumed_service_nonces', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_consumed_service_nonces t),
          'consumed_core_service_nonces', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_consumed_core_service_nonces t),
          'partitions', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_sync_partitions t),
          'attempts', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_sync_attempts t),
          'staged_changes', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_sync_staged_changes t),
          'imported_responses', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_imported_responses t),
          'imported_values', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_imported_response_values t),
          'materialization_receipts', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_materialization_receipts t),
          'idempotency_receipts', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_idempotency_receipts t),
          'bootstrap_receipts', (SELECT COALESCE(jsonb_agg(to_jsonb(t) ORDER BY to_jsonb(t)::text), '[]'::jsonb) FROM dataset_bootstrap_receipts t)
        )
        "#,
    )
    .fetch_one(pool)
    .await
    .expect("Dataset owner write fingerprint")
}
