use std::sync::Arc;

use axum::{
    body::{Body, to_bytes},
    http::{Request, StatusCode},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::Serialize;
use sha2::{Digest, Sha256};
use tessara_dataset_module::{
    DatasetCoreVerifiers, DatasetModuleState, DatasetServiceEndpoints, router,
};
use tessara_datasets_contract::{
    DATASET_BINDING_KEY, DATASET_CONTRACT_ID, DATASET_CONTRACT_SCHEMA_VERSION,
    DATASET_CORE_BINDING_KEY, DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
    DATASET_OPERATIONS_STATUS_ACTION, DATASET_OPERATIONS_STATUS_PATH, DATASET_RESOLVE_ACTION,
    DATASET_RESOLVE_PATH, DATASET_RESOURCE_OBSERVATION_CONTRACT_ID, DATASET_SUMMARY_ACTION,
    DATASET_SUMMARY_PATH, DatasetAction, DatasetDistinctValuesRequest,
    DatasetDistinctValuesResponse, DatasetExecutionRequest, DatasetExecutionResponse,
    DatasetMajorLineReference, DatasetOperationsStatusRequest, DatasetOperationsStatusResponse,
    DatasetProviderResultState, DatasetReference, DatasetResourceObservationRequest,
    DatasetResourceObservationResponse, DatasetRevisionReference, DatasetSummaryRequest,
    DatasetSummaryResponse,
};
use tessara_module_contract::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, AuthorizationAudienceV1, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, CapabilityScopeBindingV1, CoreServiceRequestV1, DependencyBindingKey,
    FunctionalContractId, ModuleDefinitionId, ModuleServiceIdentityRegistryV1,
    ModuleServicePrincipalV1, ModuleServiceRequestV1, ProtocolSignaturePurposeV1,
    PurposeBoundSigningKeyV1, SecurityCapabilityId,
};
use tower::ServiceExt;
use uuid::Uuid;

struct ProviderFixture {
    app: axum::Router,
    authorization_signer: PurposeBoundSigningKeyV1,
    core_service_request_signer: PurposeBoundSigningKeyV1,
    caller_signer: PurposeBoundSigningKeyV1,
    installation_id: Uuid,
    provider_instance_id: Uuid,
    caller_instance_id: Uuid,
    actor_id: Uuid,
    allowed_scope: Uuid,
    reference: DatasetMajorLineReference,
    dataset_reference: DatasetReference,
    revision_reference: DatasetRevisionReference,
    hidden_reference: DatasetReference,
}

#[sqlx::test(migrations = "./migrations")]
async fn signed_provider_execute_and_distinct_enforce_row_scope_and_tier(pool: sqlx::PgPool) {
    let fixture = provider_fixture(pool).await;
    let execute = DatasetExecutionRequest {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        action: DatasetAction::Execute,
        reference: fixture.reference.clone(),
        projection: vec!["label".into()],
        filters: Vec::new(),
        search: None,
        group_by: Vec::new(),
        group_missing_policies: Default::default(),
        aggregates: Vec::new(),
        order_by: Vec::new(),
        limit: 100,
        cursor: None,
    };
    let read_only = call(
        &fixture,
        "/api/private/datasets/execute",
        "datasets.execute",
        &execute,
        false,
    )
    .await;
    assert_eq!(read_only.status(), StatusCode::OK);
    let read_only: DatasetExecutionResponse = serde_json::from_slice(
        &to_bytes(read_only.into_body(), usize::MAX)
            .await
            .expect("read execute response"),
    )
    .expect("canonical execute response");
    assert_eq!(read_only.rows.len(), 1);
    assert_eq!(read_only.rows[0].row_id, "allowed-public");

    let restricted = call(
        &fixture,
        "/api/private/datasets/execute",
        "datasets.execute",
        &execute,
        true,
    )
    .await;
    assert_eq!(restricted.status(), StatusCode::OK);
    let restricted: DatasetExecutionResponse = serde_json::from_slice(
        &to_bytes(restricted.into_body(), usize::MAX)
            .await
            .expect("read restricted execute response"),
    )
    .expect("canonical restricted execute response");
    assert_eq!(
        restricted
            .rows
            .iter()
            .map(|row| row.row_id.as_str())
            .collect::<Vec<_>>(),
        ["allowed-public", "allowed-restricted"]
    );

    let distinct = DatasetDistinctValuesRequest {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        action: DatasetAction::DistinctValues,
        reference: fixture.reference.clone(),
        field_key: "label".into(),
        limit: 20,
    };
    let response = call(
        &fixture,
        "/api/private/datasets/distinct-values",
        "datasets.distinct_values",
        &distinct,
        false,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: DatasetDistinctValuesResponse = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read distinct response"),
    )
    .expect("canonical distinct response");
    assert_eq!(response.values, [serde_json::json!("Allowed")]);
}

#[sqlx::test(migrations = "./migrations")]
async fn core_gateway_reverse_actions_are_path_body_signature_presenter_and_replay_bound(
    pool: sqlx::PgPool,
) {
    let fixture = provider_fixture(pool.clone()).await;
    let summary = DatasetSummaryRequest { schema_version: 1 };
    let wrong_media = prepare_core_call(
        &fixture,
        DATASET_SUMMARY_PATH,
        DATASET_SUMMARY_ACTION,
        &summary,
        CoreCallMutation::None,
    );
    assert_eq!(
        send_core_call_with_media(&fixture, &wrong_media, "application/vnd.tessara+json")
            .await
            .status(),
        StatusCode::FORBIDDEN
    );
    assert_eq!(
        sqlx::query_scalar::<_, i64>("SELECT count(*) FROM dataset_consumed_core_service_nonces")
            .fetch_one(&pool)
            .await
            .expect("count Dataset Core nonce consumption after wrong media"),
        0,
        "media validation must precede authorization and nonce consumption"
    );
    assert_eq!(
        send_core_call(&fixture, &wrong_media).await.status(),
        StatusCode::OK,
        "the same authenticated call remains usable after a pre-auth media rejection"
    );
    let valid = prepare_core_call(
        &fixture,
        DATASET_SUMMARY_PATH,
        DATASET_SUMMARY_ACTION,
        &summary,
        CoreCallMutation::None,
    );
    let response = send_core_call(&fixture, &valid).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: DatasetSummaryResponse = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read Dataset summary response"),
    )
    .expect("canonical Dataset summary response");
    response.validate().expect("valid Dataset summary response");
    assert_eq!(response.state, DatasetProviderResultState::Available);
    assert_eq!(response.dataset_count, Some(1));

    assert_eq!(
        send_core_call(&fixture, &valid).await.status(),
        StatusCode::FORBIDDEN,
        "the Core nonce and authorization JTI are one-use"
    );

    for mutation in [
        CoreCallMutation::WrongPresenter,
        CoreCallMutation::WrongPath,
        CoreCallMutation::WrongBody,
        CoreCallMutation::WrongSignature,
    ] {
        let call = prepare_core_call(
            &fixture,
            DATASET_SUMMARY_PATH,
            DATASET_SUMMARY_ACTION,
            &summary,
            mutation,
        );
        assert_eq!(
            send_core_call(&fixture, &call).await.status(),
            StatusCode::FORBIDDEN,
            "{mutation:?} must fail closed"
        );
    }

    let operations = prepare_core_call(
        &fixture,
        DATASET_OPERATIONS_STATUS_PATH,
        DATASET_OPERATIONS_STATUS_ACTION,
        &DatasetOperationsStatusRequest {
            schema_version: 1,
            requested_scope_node_ids: vec![fixture.allowed_scope],
        },
        CoreCallMutation::None,
    );
    let response = send_core_call(&fixture, &operations).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: DatasetOperationsStatusResponse = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read Dataset operations response"),
    )
    .expect("canonical Dataset operations response");
    response
        .validate()
        .expect("valid Dataset operations response");
    assert_eq!(response.state, DatasetProviderResultState::Available);
    assert_eq!(response.items.len(), 1);
}

#[sqlx::test(migrations = "./migrations")]
async fn resource_observation_resolves_all_v2_types_and_collapses_hidden_with_random(
    pool: sqlx::PgPool,
) {
    let fixture = provider_fixture(pool).await;

    for reference in [
        fixture.dataset_reference.reference().clone(),
        fixture.revision_reference.reference().clone(),
        fixture.reference.reference().clone(),
    ] {
        let request = DatasetResourceObservationRequest {
            schema_version: 1,
            reference: reference.clone(),
        };
        let call = prepare_core_call_for_contract(
            &fixture,
            DATASET_RESOLVE_PATH,
            DATASET_RESOLVE_ACTION,
            DATASET_RESOURCE_OBSERVATION_CONTRACT_ID,
            &request,
            CoreCallMutation::None,
        );
        let response = send_core_call(&fixture, &call).await;
        assert_eq!(response.status(), StatusCode::OK);
        let response: DatasetResourceObservationResponse = serde_json::from_slice(
            &to_bytes(response.into_body(), usize::MAX)
                .await
                .expect("read Dataset observation response"),
        )
        .expect("canonical Dataset observation response");
        response
            .validate_for(&reference)
            .expect("valid Dataset observation response");
        assert!(response.observation.is_some());
    }

    let hidden = DatasetResourceObservationRequest {
        schema_version: 1,
        reference: fixture.hidden_reference.reference().clone(),
    };
    let random = DatasetResourceObservationRequest {
        schema_version: 1,
        reference: DatasetReference::from_parts(
            fixture.installation_id,
            fixture.provider_instance_id,
            Uuid::new_v4(),
        )
        .expect("random Dataset reference")
        .reference()
        .clone(),
    };
    let mut restricted_wires = Vec::new();
    for request in [&hidden, &random] {
        let call = prepare_core_call_for_contract(
            &fixture,
            DATASET_RESOLVE_PATH,
            DATASET_RESOLVE_ACTION,
            DATASET_RESOURCE_OBSERVATION_CONTRACT_ID,
            request,
            CoreCallMutation::None,
        );
        let response = send_core_call(&fixture, &call).await;
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read restricted Dataset observation response");
        let decoded: DatasetResourceObservationResponse =
            serde_json::from_slice(&bytes).expect("canonical restricted observation response");
        decoded
            .validate_for(&request.reference)
            .expect("valid restricted observation response");
        assert!(decoded.observation.is_none());
        restricted_wires.push(bytes);
    }
    assert_eq!(restricted_wires[0], restricted_wires[1]);

    let wrong_owner = DatasetResourceObservationRequest {
        schema_version: 1,
        reference: DatasetReference::from_parts(
            fixture.installation_id,
            Uuid::new_v4(),
            fixture.dataset_reference.dataset_id(),
        )
        .expect("wrong-owner Dataset reference")
        .reference()
        .clone(),
    };
    let call = prepare_core_call_for_contract(
        &fixture,
        DATASET_RESOLVE_PATH,
        DATASET_RESOLVE_ACTION,
        DATASET_RESOURCE_OBSERVATION_CONTRACT_ID,
        &wrong_owner,
        CoreCallMutation::None,
    );
    let response = send_core_call(&fixture, &call).await;
    assert_eq!(response.status(), StatusCode::OK);
    let response: DatasetResourceObservationResponse = serde_json::from_slice(
        &to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("read wrong-owner observation response"),
    )
    .expect("canonical wrong-owner observation response");
    response
        .validate_for(&wrong_owner.reference)
        .expect("valid nondisclosing wrong-owner response");
    assert!(response.observation.is_none());
}

async fn provider_fixture(pool: sqlx::PgPool) -> ProviderFixture {
    let installation_id = Uuid::new_v4();
    let provider_instance_id = Uuid::new_v4();
    let caller_instance_id = Uuid::new_v4();
    let actor_id = Uuid::new_v4();
    let allowed_scope = Uuid::new_v4();
    let hidden_scope = Uuid::new_v4();
    let dataset_id = Uuid::new_v4();
    let revision_id = Uuid::new_v4();
    let hidden_dataset_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dataset_security_state
         (singleton,installation_id,module_instance_id,authorization_revision,
          organization_revision,enabled,document_state)
         VALUES(true,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(provider_instance_id)
    .execute(&pool)
    .await
    .expect("install Dataset security state");
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,'Provider rows','provider-rows','submission')")
        .bind(dataset_id)
        .execute(&pool)
        .await
        .expect("insert Dataset");
    sqlx::query(
        "INSERT INTO datasets(id,name,slug,grain)
         VALUES($1,'Hidden provider rows','hidden-provider-rows','submission')",
    )
    .bind(hidden_dataset_id)
    .execute(&pool)
    .await
    .expect("insert hidden Dataset");
    for (scope, name) in [(allowed_scope, "allowed"), (hidden_scope, "hidden")] {
        sqlx::query(
            "INSERT INTO dataset_scope_nodes
             (dataset_id,node_id,node_name,node_type_name,node_path,
              requested_set_revision,requested_set_digest)
             VALUES($1,$2,$3,'organization',$4,'scope:1',$5)",
        )
        .bind(dataset_id)
        .bind(scope)
        .bind(name)
        .bind(format!("/{name}"))
        .bind(format!("sha256:{}", "a".repeat(64)))
        .execute(&pool)
        .await
        .expect("insert Dataset scope");
    }
    sqlx::query(
        "INSERT INTO dataset_scope_nodes
         (dataset_id,node_id,node_name,node_type_name,node_path,
          requested_set_revision,requested_set_digest)
         VALUES($1,$2,'hidden-only','organization','/hidden-only','scope:1',$3)",
    )
    .bind(hidden_dataset_id)
    .bind(hidden_scope)
    .bind(format!("sha256:{}", "b".repeat(64)))
    .execute(&pool)
    .await
    .expect("insert hidden Dataset scope");
    let output_fields = serde_json::json!([{
        "key": "label",
        "label": "Label",
        "source_alias": "responses",
        "source_field_key": "label",
        "field_type": "text",
        "position": 0
    }]);
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,
          version_patch,semantic_bump,started_new_major_line,status,output_fields,
          materialized_schema,materialized_table,materialized_row_count,materialized_at)
         VALUES($1,$2,1,'1.0.0',1,0,0,'initial',true,'published',$3,
                'dataset_materialized','provider_rows',4,now())",
    )
    .bind(revision_id)
    .bind(dataset_id)
    .bind(output_fields)
    .execute(&pool)
    .await
    .expect("insert Dataset revision");
    for (scope, name) in [(allowed_scope, "allowed"), (hidden_scope, "hidden")] {
        sqlx::query(
            "INSERT INTO dataset_revision_scope_nodes
             (revision_id,node_id,node_name,node_type_name,node_path,
              requested_set_revision,requested_set_digest)
             VALUES($1,$2,$3,'organization',$4,'scope:1',$5)",
        )
        .bind(revision_id)
        .bind(scope)
        .bind(name)
        .bind(format!("/{name}"))
        .bind(format!("sha256:{}", "a".repeat(64)))
        .execute(&pool)
        .await
        .expect("insert revision scope");
    }
    sqlx::query(
        "CREATE TABLE dataset_materialized.provider_rows (
            __row_id text PRIMARY KEY,
            __restriction_tier text NOT NULL,
            __scope_node_ids uuid[] NOT NULL CHECK (cardinality(__scope_node_ids)>0),
            label text
         )",
    )
    .execute(&pool)
    .await
    .expect("create provider materialization");
    for (row_id, tier, scope, label) in [
        ("allowed-public", "public", allowed_scope, "Allowed"),
        (
            "allowed-restricted",
            "restricted",
            allowed_scope,
            "Restricted",
        ),
        (
            "allowed-confidential",
            "confidential",
            allowed_scope,
            "Confidential",
        ),
        ("hidden-public", "public", hidden_scope, "Hidden"),
    ] {
        sqlx::query(
            "INSERT INTO dataset_materialized.provider_rows
             (__row_id,__restriction_tier,__scope_node_ids,label) VALUES($1,$2,$3,$4)",
        )
        .bind(row_id)
        .bind(tier)
        .bind(vec![scope])
        .bind(label)
        .execute(&pool)
        .await
        .expect("insert materialized row");
    }
    sqlx::query(
        "INSERT INTO dataset_major_materializations
         (dataset_id,version_major,materialized_schema,materialized_table,
          materialized_row_count,materialized_at,rebuild_status)
         VALUES($1,1,'dataset_materialized','provider_rows',4,now(),'ready')",
    )
    .bind(dataset_id)
    .execute(&pool)
    .await
    .expect("insert major materialization");

    let authorization_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "provider-test-core",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [41; 32],
    )
    .expect("authorization signer");
    let core_service_request_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "provider-test-core",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [41; 32],
    )
    .expect("Core service-request signer");
    let bootstrap_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "provider-test-core",
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        [41; 32],
    )
    .expect("bootstrap signer");
    let owner_bootstrap_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "provider-test-core",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [41; 32],
    )
    .expect("owner bootstrap signer");
    let shell_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.core",
        "provider-test-core",
        ProtocolSignaturePurposeV1::ShellContext,
        [41; 32],
    )
    .expect("shell signer");
    let caller_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.components",
        "provider-test-component",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [42; 32],
    )
    .expect("caller signer");
    let registry = ModuleServiceIdentityRegistryV1::from_json(
        &serde_json::json!({
            "schema_version": 1,
            "identities": {
                "tessara.components": {
                    "key_id": "provider-test-component",
                    "public_key": URL_SAFE_NO_PAD.encode(caller_signer.verifier().public_key_bytes())
                }
            }
        })
        .to_string(),
    )
    .expect("service identity registry");
    let dataset_signer = Arc::new(
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.datasets",
            "provider-test-dataset",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            [43; 32],
        )
        .expect("Dataset service signer"),
    );
    let receipt_signer = Arc::new(
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.datasets",
            "provider-test-dataset",
            ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
            [43; 32],
        )
        .expect("Dataset receipt signer"),
    );
    let app = router(
        DatasetModuleState::new(
            pool,
            DatasetCoreVerifiers {
                authorization: authorization_signer.verifier(),
                owner_bootstrap: owner_bootstrap_signer.verifier(),
                service_request: core_service_request_signer.verifier(),
                bootstrap_validation: bootstrap_signer.verifier(),
                shell: shell_signer.verifier(),
            },
            registry,
            dataset_signer,
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
            .expect("service endpoints"),
            tessara_dataset_module::DatasetValidationFaultControl::disabled(),
        )
        .expect("Dataset state"),
    );
    ProviderFixture {
        app,
        authorization_signer,
        core_service_request_signer,
        caller_signer,
        installation_id,
        provider_instance_id,
        caller_instance_id,
        actor_id,
        allowed_scope,
        reference: DatasetMajorLineReference::from_parts(
            installation_id,
            provider_instance_id,
            dataset_id,
            1,
        )
        .expect("Dataset reference"),
        dataset_reference: DatasetReference::from_parts(
            installation_id,
            provider_instance_id,
            dataset_id,
        )
        .expect("Dataset resource reference"),
        revision_reference: DatasetRevisionReference::from_parts(
            installation_id,
            provider_instance_id,
            revision_id,
        )
        .expect("Dataset revision reference"),
        hidden_reference: DatasetReference::from_parts(
            installation_id,
            provider_instance_id,
            hidden_dataset_id,
        )
        .expect("hidden Dataset resource reference"),
    }
}

#[derive(Clone, Copy, Debug)]
enum CoreCallMutation {
    None,
    WrongPresenter,
    WrongPath,
    WrongBody,
    WrongSignature,
}

struct PreparedCoreCall {
    path: &'static str,
    body: Vec<u8>,
    encoded_authorization: String,
    encoded_service_request: String,
    correlation_id: Uuid,
}

fn prepare_core_call<T: Serialize>(
    fixture: &ProviderFixture,
    path: &'static str,
    action: &'static str,
    payload: &T,
    mutation: CoreCallMutation,
) -> PreparedCoreCall {
    prepare_core_call_for_contract(
        fixture,
        path,
        action,
        DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
        payload,
        mutation,
    )
}

fn prepare_core_call_for_contract<T: Serialize>(
    fixture: &ProviderFixture,
    path: &'static str,
    action: &'static str,
    functional_contract: &'static str,
    payload: &T,
    mutation: CoreCallMutation,
) -> PreparedCoreCall {
    let signed_body = serde_json::to_vec(payload).expect("serialize Core provider request");
    let now = Utc::now();
    let correlation_id = Uuid::new_v4();
    let presenting_service = if matches!(mutation, CoreCallMutation::WrongPresenter) {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: fixture.caller_instance_id,
            module_definition_id: ModuleDefinitionId::new("tessara.components")
                .expect("caller definition"),
        }
    } else {
        ModuleServicePrincipalV1::CoreGateway
    };
    let authorization = fixture
        .authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: fixture.installation_id,
            original_actor_id: fixture.actor_id,
            correlation_id,
            presenting_service,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: fixture.provider_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("provider definition"),
            },
            dependency_binding: DependencyBindingKey::new(DATASET_CORE_BINDING_KEY)
                .expect("Core Dataset binding"),
            functional_contract: FunctionalContractId::new(functional_contract)
                .expect("Dataset provider contract"),
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:read")
                    .expect("Dataset read capability"),
                organization_root_id: fixture.allowed_scope,
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
        .expect("sign Core provider authorization");
    let encoded_authorization = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&authorization).expect("serialize Core provider authorization"));
    let service_request = CoreServiceRequestV1 {
        schema_version: 1,
        installation_id: fixture.installation_id,
        method: "POST".into(),
        path: if matches!(mutation, CoreCallMutation::WrongPath) {
            DATASET_OPERATIONS_STATUS_PATH.into()
        } else {
            path.into()
        },
        canonical_body_digest: sha256_hex(&signed_body),
        inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
        correlation_id: correlation_id.to_string(),
        nonce: Uuid::new_v4(),
        issued_at: now,
        expires_at: now + Duration::seconds(30),
    };
    let signed_service_request = if matches!(mutation, CoreCallMutation::WrongSignature) {
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "provider-test-core",
            ProtocolSignaturePurposeV1::ModuleServiceRequest,
            [99; 32],
        )
        .expect("wrong Core service-request signer")
        .sign(service_request)
    } else {
        fixture.core_service_request_signer.sign(service_request)
    }
    .expect("sign Core service request");
    let encoded_service_request = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&signed_service_request).expect("serialize Core service request"),
    );
    let body = if matches!(mutation, CoreCallMutation::WrongBody) {
        br#"{"schema_version":0}"#.to_vec()
    } else {
        signed_body
    };
    PreparedCoreCall {
        path,
        body,
        encoded_authorization,
        encoded_service_request,
        correlation_id,
    }
}

async fn send_core_call(
    fixture: &ProviderFixture,
    call: &PreparedCoreCall,
) -> axum::response::Response {
    send_core_call_with_media(fixture, call, "application/json").await
}

async fn send_core_call_with_media(
    fixture: &ProviderFixture,
    call: &PreparedCoreCall,
    media_type: &str,
) -> axum::response::Response {
    fixture
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(call.path)
                .header("content-type", media_type)
                .header("x-tessara-authorization", &call.encoded_authorization)
                .header(
                    "x-tessara-core-service-request",
                    &call.encoded_service_request,
                )
                .header("x-tessara-correlation-id", call.correlation_id.to_string())
                .body(Body::from(call.body.clone()))
                .expect("Core provider request"),
        )
        .await
        .expect("Core provider response")
}

async fn call<T: Serialize>(
    fixture: &ProviderFixture,
    path: &'static str,
    action: &'static str,
    payload: &T,
    restricted: bool,
) -> axum::response::Response {
    let body = serde_json::to_vec(payload).expect("serialize provider request");
    let now = Utc::now();
    let correlation_id = Uuid::new_v4();
    let mut bindings = vec![CapabilityScopeBindingV1 {
        capability: SecurityCapabilityId::new("datasets:read").expect("read capability"),
        organization_root_id: fixture.allowed_scope,
        authorized_organization_ids: Vec::new(),
    }];
    if restricted {
        bindings.push(CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new("datasets:read_restricted")
                .expect("restricted capability"),
            organization_root_id: fixture.allowed_scope,
            authorized_organization_ids: Vec::new(),
        });
    }
    let authorization = fixture
        .authorization_signer
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: fixture.installation_id,
            original_actor_id: fixture.actor_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: fixture.caller_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.components")
                    .expect("caller definition"),
            },
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: fixture.provider_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                    .expect("provider definition"),
            },
            dependency_binding: DependencyBindingKey::new(DATASET_BINDING_KEY)
                .expect("Dataset binding"),
            functional_contract: FunctionalContractId::new(DATASET_CONTRACT_ID)
                .expect("Dataset contract"),
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: bindings,
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 11,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .expect("sign provider authorization");
    let encoded_authorization = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&authorization).expect("serialize provider authorization"));
    let service_request = fixture
        .caller_signer
        .sign(ModuleServiceRequestV1 {
            schema_version: 1,
            installation_id: fixture.installation_id,
            module_instance_id: fixture.caller_instance_id,
            module_definition_id: ModuleDefinitionId::new("tessara.components")
                .expect("caller definition"),
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(&body),
            inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            nonce: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .expect("sign service request");
    let encoded_service = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&service_request).expect("serialize service request"));
    fixture
        .app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(path)
                .header("content-type", "application/json")
                .header("x-tessara-authorization", encoded_authorization)
                .header("x-tessara-module-service-request", encoded_service)
                .header("x-tessara-correlation-id", correlation_id.to_string())
                .body(Body::from(body))
                .expect("provider request"),
        )
        .await
        .expect("provider response")
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
