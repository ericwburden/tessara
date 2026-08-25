use std::{collections::BTreeMap, sync::Arc};

use axum::{
    Router,
    body::{Body, to_bytes},
    http::{Request, StatusCode, header},
    response::Response,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::de::DeserializeOwned;
use serde_json::json;
use sha2::{Digest, Sha256};
use tessara_composition::{
    ActorEvidenceV1, OwnerBootstrapAuthorizationV1, OwnerBootstrapProviderActionV1,
    OwnerBootstrapRequestV1, OwnerBootstrapResponseV1,
};
use tessara_forms_contract::{
    FORM_VERSION_SCHEMA_CONTRACT_ID, FORM_VERSION_SCHEMA_VERSION, FormVersionField,
    FormVersionSchemaResponse, FormVersionSection, RESPONSE_FORM_VERSION_SCHEMA_ACTION,
    RESPONSE_FORM_VERSION_SCHEMA_PATH,
};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationAudienceV1, CapabilityScopeBindingV1,
    MODULE_SERVICE_IDENTITY_REGISTRY_SCHEMA_VERSION_V1, ModuleDefinitionId,
    ModuleServiceIdentityRegistrationV1, ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1,
    ModuleServiceRequestV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, ResourceOwner,
    SecurityCapabilityId, ServiceActionMethod,
};
use tessara_module_runtime::CoreVerifiers;
use tessara_response_module::{
    RESPONSE_BOOTSTRAP_SCHEMA_VERSION, RESPONSE_FORM_BINDING,
    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING, RESPONSE_WORKFLOW_CONTEXT_BINDING,
    ResponseBootstrapDefinitionV1, ResponseBootstrapLifecycleStateV1, ResponseBootstrapReadBackV1,
    ResponseBootstrapV1, ResponseCoreVerifiers, ResponseRuntime, ResponseServiceEndpoints,
    ResponseValidationFaultControl, router,
};
use tessara_responses_contract::{
    RESPONSE_EXPORT_BINDING_KEY, RESPONSE_EXPORT_CHECKPOINT_ACTION,
    RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_CONTRACT_ID, RESPONSE_EXPORT_MEDIA_TYPE,
    RESPONSE_EXPORT_SCHEMA_VERSION, ResponseExportAction, ResponseExportCheckpointRequest,
    ResponseExportCheckpointResponse, ResponseExportPartition, ResponseLifecycleState,
    ResponseReference,
};
use tessara_workflows_contract::{
    WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
    WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH, WORKFLOW_RESPONSE_CONTEXT_ACTION,
    WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID, WORKFLOW_RESPONSE_CONTEXT_PATH,
    WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION, WorkflowResponseStartContext,
};
use tower::ServiceExt;
use uuid::Uuid;

struct BootstrapFixture {
    app: Router,
    request: OwnerBootstrapRequestV1<ResponseBootstrapV1>,
    receipt_signer: Arc<PurposeBoundSigningKeyV1>,
    owner_signer: Arc<PurposeBoundSigningKeyV1>,
    dataset_service_signer: Arc<PurposeBoundSigningKeyV1>,
    installation_id: Uuid,
    module_instance_id: Uuid,
}

async fn bootstrap_fixture(
    sql_pool: &sqlx::PgPool,
    validation_fault_control: ResponseValidationFaultControl,
    resource_key: &str,
    idempotency_key: &str,
) -> BootstrapFixture {
    let installation_id = Uuid::new_v4();
    let module_instance_id =
        tessara_composition::module_instance_id(installation_id, "tessara.responses");
    sqlx::query(
        "INSERT INTO response_module_security_state
         (singleton,schema_version,installation_id,module_instance_id,
          authorization_revision,organization_revision,enabled,document_state)
         VALUES(true,1,$1,$2,7,11,true,'enabled')",
    )
    .bind(installation_id)
    .bind(module_instance_id)
    .execute(sql_pool)
    .await
    .expect("Response security state");

    let owner_signer = Arc::new(signer(
        "tessara.core",
        "response-bootstrap-core",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [41; 32],
    ));
    let receipt_signer = Arc::new(signer(
        "tessara.responses",
        "response-bootstrap-owner",
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        [42; 32],
    ));
    let service_signer = Arc::new(signer(
        "tessara.responses",
        "response-bootstrap-owner",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [42; 32],
    ));
    let dataset_service_signer = Arc::new(signer(
        "tessara.datasets",
        "response-bootstrap-dataset",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [43; 32],
    ));
    let authorization_signer = signer(
        "tessara.core",
        "response-bootstrap-core",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [41; 32],
    );
    let shell_signer = signer(
        "tessara.core",
        "response-bootstrap-core",
        ProtocolSignaturePurposeV1::ShellContext,
        [41; 32],
    );
    let core_service_signer = signer(
        "tessara.core",
        "response-bootstrap-core",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [41; 32],
    );
    let core_compatibility_signer = signer(
        "tessara.core",
        "response-bootstrap-core",
        ProtocolSignaturePurposeV1::ProviderCompatibilityResponse,
        [41; 32],
    );
    let registry = ModuleServiceIdentityRegistryV1 {
        schema_version: MODULE_SERVICE_IDENTITY_REGISTRY_SCHEMA_VERSION_V1,
        identities: BTreeMap::from([
            (
                ModuleDefinitionId::new("tessara.responses").expect("Response definition"),
                ModuleServiceIdentityRegistrationV1 {
                    key_id: "response-bootstrap-owner".into(),
                    public_key: URL_SAFE_NO_PAD
                        .encode(service_signer.verifier().public_key_bytes()),
                },
            ),
            (
                ModuleDefinitionId::new("tessara.datasets").expect("Dataset definition"),
                ModuleServiceIdentityRegistrationV1 {
                    key_id: "response-bootstrap-dataset".into(),
                    public_key: URL_SAFE_NO_PAD
                        .encode(dataset_service_signer.verifier().public_key_bytes()),
                },
            ),
        ]),
    };
    let unavailable = "http://127.0.0.1:1".to_string();
    let endpoints = ResponseServiceEndpoints::new(
        &unavailable,
        BTreeMap::from([
            (RESPONSE_FORM_BINDING.into(), unavailable.clone()),
            (
                RESPONSE_WORKFLOW_CONTEXT_BINDING.into(),
                unavailable.clone(),
            ),
            (
                RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING.into(),
                unavailable.clone(),
            ),
        ]),
    )
    .expect("Response service endpoints");
    let app = router(Arc::new(ResponseRuntime::new(
        sql_pool.clone(),
        ResponseCoreVerifiers {
            runtime: CoreVerifiers {
                authorization: authorization_signer.verifier(),
                shell: shell_signer.verifier(),
            },
            service_request: core_service_signer.verifier(),
            provider_compatibility: core_compatibility_signer.verifier(),
            owner_bootstrap: owner_signer.verifier(),
        },
        service_signer,
        registry,
        endpoints,
        receipt_signer.clone(),
        validation_fault_control,
    )));

    let actor_id = Uuid::new_v4();
    let node_id = Uuid::new_v4();
    let form_id = Uuid::new_v4();
    let form_version_id = Uuid::new_v4();
    let field_id = Uuid::new_v4();
    let section_id = Uuid::new_v4();
    let form = FormVersionSchemaResponse {
        schema_version: FORM_VERSION_SCHEMA_VERSION,
        form_id,
        form_version_id,
        form_name: "Response bootstrap form".into(),
        form_slug: "response-bootstrap-form".into(),
        version_label: Some("Published".into()),
        version_major: Some(1),
        source_scope_node_ids: vec![node_id],
        source_scope_revision: String::new(),
        source_scope_digest: String::new(),
        content_revision: String::new(),
        content_digest: String::new(),
        sections: vec![FormVersionSection {
            section_id,
            key: "main".into(),
            label: "Main".into(),
            description: String::new(),
            position: 0,
        }],
        fields: vec![FormVersionField {
            field_id,
            key: "answer".into(),
            label: "Answer".into(),
            field_type: "text".into(),
            required: true,
            options: Vec::new(),
            section_id: Some(section_id),
            position: 0,
            grid_row: 1,
            grid_column: 1,
            grid_width: 12,
            grid_height: 1,
        }],
    }
    .with_recomputed_digests()
    .expect("canonical Form snapshot");
    let now = Utc::now();
    let workflow = WorkflowResponseStartContext {
        schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
        workflow_assignment_id: Uuid::new_v4(),
        workflow_id: Uuid::new_v4(),
        workflow_name: "Response bootstrap workflow".into(),
        workflow_description: String::new(),
        workflow_version_id: Uuid::new_v4(),
        workflow_version_label: Some("Published".into()),
        workflow_step_id: Uuid::new_v4(),
        workflow_step_title: "Respond".into(),
        workflow_step_position: 0,
        workflow_step_count: 1,
        next_workflow_step_title: None,
        next_workflow_step_form_name: None,
        history: Vec::new(),
        workflow_instance_id: Uuid::new_v4(),
        workflow_step_instance_id: Uuid::new_v4(),
        form_id,
        form_version_id,
        node_id,
        node_name: "Primary".into(),
        assignee_account_id: actor_id,
        assignee_display_name: "Response Owner".into(),
        started_by_account_id: actor_id,
        delegation_basis: None,
        one_use_nonce: Uuid::new_v4(),
        issued_at: now.to_rfc3339(),
        expires_at: (now + Duration::minutes(5)).to_rfc3339(),
        context_digest: String::new(),
    }
    .with_recomputed_digest()
    .expect("canonical Workflow context");
    let input = ResponseBootstrapV1 {
        schema_version: RESPONSE_BOOTSTRAP_SCHEMA_VERSION.into(),
        responses: vec![ResponseBootstrapDefinitionV1 {
            resource_key: resource_key.into(),
            form,
            workflow,
            values: BTreeMap::from([("answer".into(), json!("Complete"))]),
            lifecycle_state: ResponseBootstrapLifecycleStateV1::Submitted,
        }],
    };
    let request = bootstrap_request(
        installation_id,
        module_instance_id,
        actor_id,
        input,
        idempotency_key,
        &owner_signer,
    );

    BootstrapFixture {
        app,
        request,
        receipt_signer,
        owner_signer,
        dataset_service_signer,
        installation_id,
        module_instance_id,
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn signed_response_bootstrap_materializes_submits_and_replays(sql_pool: sqlx::PgPool) {
    let fixture = bootstrap_fixture(
        &sql_pool,
        ResponseValidationFaultControl::disabled(),
        "response.submitted.owner",
        "response-bootstrap-integration",
    )
    .await;

    let first: OwnerBootstrapResponseV1 = send(&fixture.app, &fixture.request).await;
    assert!(first.receipt.changed);
    assert!(first.has_exact_signed_receipt());
    fixture
        .receipt_signer
        .verifier()
        .verify(&first.signed_receipt)
        .expect("Response receipt signature");
    let reference: ResponseReference =
        serde_json::from_str(first.receipt.resource_ids["response.submitted.owner"].as_str())
            .expect("Response logical reference");
    assert_eq!(
        reference.reference().installation_id(),
        fixture.installation_id
    );
    assert_eq!(
        reference.reference().owner(),
        &ResourceOwner::ModuleInstance {
            installation_id: fixture.installation_id,
            module_instance_id: fixture.module_instance_id,
        }
    );
    let read_back: ResponseBootstrapReadBackV1 = serde_json::from_str(
        first.receipt.resource_ids["response.submitted.owner.read_back"].as_str(),
    )
    .expect("Response typed read-back");
    assert_eq!(read_back.lifecycle_state, ResponseLifecycleState::Submitted);
    assert_eq!(read_back.revision, 2);
    assert_eq!(read_back.workflow_event_sequence, 2);

    let compatible_provider_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM response_provider_observations
         WHERE compatibility_state='compatible' AND last_observed_at IS NOT NULL
           AND last_compatible_at IS NOT NULL AND last_stable_finding IS NULL",
    )
    .fetch_one(&sql_pool)
    .await
    .expect("authenticated provider compatibility projection");
    assert_eq!(compatible_provider_count, 3);
    sqlx::query(
        "UPDATE response_provider_observations
         SET compatibility_state='unknown',last_observed_at=NULL,last_compatible_at=NULL,
             last_stable_finding=NULL",
    )
    .execute(&sql_pool)
    .await
    .expect("simulate lost derived provider projection");

    let replay: OwnerBootstrapResponseV1 = send(&fixture.app, &fixture.request).await;
    assert!(!replay.receipt.changed);
    assert_eq!(replay.receipt.resource_ids, first.receipt.resource_ids);
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT (SELECT COUNT(*) FROM responses),
                (SELECT COUNT(*) FROM response_values),
                (SELECT COUNT(*) FROM response_audit_events),
                (SELECT COUNT(*) FROM response_workflow_events),
                (SELECT COUNT(*) FROM response_bootstrap_receipts)",
    )
    .fetch_one(&sql_pool)
    .await
    .expect("Response bootstrap counts");
    assert_eq!(counts, (1, 1, 2, 2, 1));
    let replay_compatible_provider_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM response_provider_observations
         WHERE compatibility_state='compatible' AND last_observed_at IS NOT NULL
           AND last_compatible_at IS NOT NULL AND last_stable_finding IS NULL",
    )
    .fetch_one(&sql_pool)
    .await
    .expect("replayed provider compatibility projection");
    assert_eq!(replay_compatible_provider_count, 3);
}

#[sqlx::test(migrations = "./migrations")]
async fn owner_bootstrap_authorizes_dataset_export_checkpoint(sql_pool: sqlx::PgPool) {
    let fixture = bootstrap_fixture(
        &sql_pool,
        ResponseValidationFaultControl::disabled(),
        "response.export.owner",
        "response-export-owner-bootstrap",
    )
    .await;
    let dataset_definition =
        ModuleDefinitionId::new("tessara.datasets").expect("Dataset definition");
    let dataset_instance_id = tessara_composition::module_instance_id(
        fixture.installation_id,
        dataset_definition.as_str(),
    );
    let presenting_service = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: dataset_instance_id,
        module_definition_id: dataset_definition.clone(),
    };
    let scope_id = Uuid::new_v4();
    let partition = ResponseExportPartition {
        source_binding_id: Uuid::new_v4(),
        form_version_ids: vec![Uuid::new_v4()],
        authorized_scope_node_ids: vec![scope_id],
        authorization_digest: String::new(),
    }
    .with_recomputed_authorization_digest(&presenting_service, fixture.installation_id)
    .expect("authorized Response export partition");
    let body = serde_json::to_vec(&ResponseExportCheckpointRequest {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        action: ResponseExportAction::Checkpoint,
        partition,
        committed_cursor: None,
    })
    .expect("Response export checkpoint request");
    let now = Utc::now();
    let correlation_id = Uuid::new_v4();
    let authorization = fixture
        .owner_signer
        .sign(OwnerBootstrapAuthorizationV1 {
            schema_version: tessara_composition::OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1,
            installation_id: fixture.installation_id,
            owner: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: dataset_instance_id,
                module_definition_id: dataset_definition.clone(),
            },
            owner_definition_id: dataset_definition.to_string(),
            initiator: ActorEvidenceV1 {
                actor_id: Uuid::new_v4().to_string(),
                actor_kind: "user".into(),
                authority: "bootstrap-integration".into(),
            },
            original_actor_id: Uuid::new_v4(),
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("submissions:manage")
                    .expect("submission management capability"),
                organization_root_id: scope_id,
                authorized_organization_ids: Vec::new(),
            }],
            provider_actions: vec![OwnerBootstrapProviderActionV1 {
                dependency_binding: RESPONSE_EXPORT_BINDING_KEY.into(),
                functional_contract: RESPONSE_EXPORT_CONTRACT_ID.into(),
                action: RESPONSE_EXPORT_CHECKPOINT_ACTION.into(),
                method: ServiceActionMethod::Post,
                path: RESPONSE_EXPORT_CHECKPOINT_PATH.into(),
                audience: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: fixture.module_instance_id,
                    module_definition_id: ModuleDefinitionId::new("tessara.responses")
                        .expect("Response definition"),
                },
            }],
            authorization_revision: 7,
            organization_revision: 11,
            locked_input_digest: digest('b'),
            input_digest: digest('c'),
            desired_revision: 1,
            apply_sequence: 1,
            target_plan_digest: digest('d'),
            idempotency_key: "response-export-owner-bootstrap".into(),
            correlation_id,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .expect("signed Dataset owner bootstrap authorization");
    let encoded_authorization = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&authorization).expect("serialize owner bootstrap authorization"),
    );
    let service_request = fixture
        .dataset_service_signer
        .sign(ModuleServiceRequestV1 {
            schema_version: 1,
            installation_id: fixture.installation_id,
            module_instance_id: dataset_instance_id,
            module_definition_id: dataset_definition,
            method: "POST".into(),
            path: RESPONSE_EXPORT_CHECKPOINT_PATH.into(),
            canonical_body_digest: sha256_hex(&body),
            inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            nonce: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .expect("signed Dataset service request");
    let encoded_service_request = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&service_request).expect("serialize Dataset service request"));

    let response = send_export_checkpoint(
        &fixture.app,
        &body,
        &encoded_authorization,
        &encoded_service_request,
        correlation_id,
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get(header::CONTENT_TYPE).unwrap(),
        RESPONSE_EXPORT_MEDIA_TYPE
    );
    let checkpoint: ResponseExportCheckpointResponse = response_json(response).await;
    assert_eq!(checkpoint.schema_version, RESPONSE_EXPORT_SCHEMA_VERSION);
    assert!(checkpoint.committed_cursor_valid);
    assert!(checkpoint.changed);

    let replay = send_export_checkpoint(
        &fixture.app,
        &body,
        &encoded_authorization,
        &encoded_service_request,
        correlation_id,
    )
    .await;
    assert_eq!(replay.status(), StatusCode::NOT_FOUND);
}

#[cfg(feature = "sprint-8c-validation-faults")]
#[sqlx::test(migrations = "./migrations")]
async fn mid_apply_validation_fault_rolls_back_complete_bootstrap_transaction(
    sql_pool: sqlx::PgPool,
) {
    let fixture = bootstrap_fixture(
        &sql_pool,
        ResponseValidationFaultControl::deterministic_mid_apply_for_test(Uuid::new_v4()),
        "response.submitted.fault",
        "response-bootstrap-mid-apply-fault",
    )
    .await;

    let failed = send_response(&fixture.app, &fixture.request).await;
    assert_eq!(failed.status(), StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        bootstrap_state_counts(&sql_pool).await,
        (0, 0, 0, 0, 0, 0, 0, 0),
        "the injected first attempt must roll back every Response-owned write"
    );

    let recovered: OwnerBootstrapResponseV1 = send(&fixture.app, &fixture.request).await;
    assert!(recovered.receipt.changed);
    assert_eq!(
        bootstrap_state_counts(&sql_pool).await,
        (1, 1, 2, 2, 1, 2, 1, 1),
        "the disarmed successor attempt must commit the complete bootstrap graph"
    );
}

#[cfg(feature = "sprint-8c-validation-faults")]
async fn bootstrap_state_counts(pool: &sqlx::PgPool) -> (i64, i64, i64, i64, i64, i64, i64, i64) {
    sqlx::query_as(
        "SELECT
           (SELECT COUNT(*) FROM responses),
           (SELECT COUNT(*) FROM response_values),
           (SELECT COUNT(*) FROM response_audit_events),
           (SELECT COUNT(*) FROM response_workflow_events),
           (SELECT COUNT(*) FROM response_start_claims),
           (SELECT COUNT(*) FROM response_idempotency_receipts),
           (SELECT COUNT(*) FROM response_export_changes),
           (SELECT COUNT(*) FROM response_bootstrap_receipts)",
    )
    .fetch_one(pool)
    .await
    .expect("Response bootstrap state counts")
}

fn bootstrap_request(
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
    input: ResponseBootstrapV1,
    idempotency_key: &str,
    signer: &PurposeBoundSigningKeyV1,
) -> OwnerBootstrapRequestV1<ResponseBootstrapV1> {
    let input_digest = tessara_composition::canonical_digest(&input).expect("input digest");
    let now = Utc::now();
    let idempotency_key = idempotency_key.to_string();
    let authorization = signer
        .sign(OwnerBootstrapAuthorizationV1 {
            schema_version: tessara_composition::OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1,
            installation_id,
            owner: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id,
                module_definition_id: ModuleDefinitionId::new("tessara.responses")
                    .expect("Response definition"),
            },
            owner_definition_id: "tessara.responses".into(),
            initiator: ActorEvidenceV1 {
                actor_id: actor_id.to_string(),
                actor_kind: "user".into(),
                authority: "bootstrap-integration".into(),
            },
            original_actor_id: actor_id,
            capability_scope_bindings: Vec::new(),
            provider_actions: bootstrap_provider_actions(installation_id),
            authorization_revision: 7,
            organization_revision: 11,
            locked_input_digest: input_digest.clone(),
            input_digest: input_digest.clone(),
            desired_revision: 1,
            apply_sequence: 1,
            target_plan_digest: digest('a'),
            idempotency_key: idempotency_key.clone(),
            correlation_id: Uuid::new_v4(),
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::minutes(1),
        })
        .expect("signed owner bootstrap authorization");
    OwnerBootstrapRequestV1 {
        installation_id,
        desired_revision: 1,
        apply_sequence: 1,
        target_plan_digest: digest('a'),
        idempotency_key,
        locked_input_digest: input_digest.clone(),
        input_digest,
        authorization,
        dependency_validation: None,
        input,
    }
}

fn bootstrap_provider_actions(installation_id: Uuid) -> Vec<OwnerBootstrapProviderActionV1> {
    [
        (
            RESPONSE_FORM_BINDING,
            FORM_VERSION_SCHEMA_CONTRACT_ID,
            RESPONSE_FORM_VERSION_SCHEMA_ACTION,
            RESPONSE_FORM_VERSION_SCHEMA_PATH,
        ),
        (
            RESPONSE_WORKFLOW_CONTEXT_BINDING,
            WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
            WORKFLOW_RESPONSE_CONTEXT_ACTION,
            WORKFLOW_RESPONSE_CONTEXT_PATH,
        ),
        (
            RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
            WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
            WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION,
            WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
        ),
    ]
    .into_iter()
    .map(
        |(binding, contract, action, path)| OwnerBootstrapProviderActionV1 {
            dependency_binding: binding.into(),
            functional_contract: contract.into(),
            action: action.into(),
            method: ServiceActionMethod::Post,
            path: path.into(),
            audience: AuthorizationAudienceV1::CoreInstallation { installation_id },
        },
    )
    .collect()
}

async fn send(
    app: &Router,
    request: &OwnerBootstrapRequestV1<ResponseBootstrapV1>,
) -> OwnerBootstrapResponseV1 {
    let response = send_response(app, request).await;
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
}

async fn send_response(
    app: &Router,
    request: &OwnerBootstrapRequestV1<ResponseBootstrapV1>,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/private/bootstrap")
                .header(header::CONTENT_TYPE, "application/json")
                .header(
                    "x-tessara-module-control-key",
                    std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
                        .unwrap_or_else(|_| "development-module-control-only".into()),
                )
                .body(Body::from(
                    serde_json::to_vec(request).expect("bootstrap request JSON"),
                ))
                .expect("bootstrap request"),
        )
        .await
        .expect("bootstrap response")
}

async fn send_export_checkpoint(
    app: &Router,
    body: &[u8],
    encoded_authorization: &str,
    encoded_service_request: &str,
    correlation_id: Uuid,
) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(RESPONSE_EXPORT_CHECKPOINT_PATH)
                .header(header::CONTENT_TYPE, RESPONSE_EXPORT_MEDIA_TYPE)
                .header(
                    "x-tessara-owner-bootstrap-authorization",
                    encoded_authorization,
                )
                .header("x-tessara-module-service-request", encoded_service_request)
                .header("x-tessara-correlation-id", correlation_id.to_string())
                .body(Body::from(body.to_vec()))
                .expect("Response export checkpoint request"),
        )
        .await
        .expect("Response export checkpoint response")
}

fn signer(
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    secret: [u8; 32],
) -> PurposeBoundSigningKeyV1 {
    PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, secret)
        .expect("test signing key")
}

fn digest(value: char) -> ArtifactDigest {
    ArtifactDigest::new(format!("sha256:{}", value.to_string().repeat(64))).expect("test digest")
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

async fn response_json<T: DeserializeOwned>(response: Response) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("response bytes");
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "Response bootstrap returned invalid JSON ({error}): {}",
            String::from_utf8_lossy(&bytes)
        )
    })
}
