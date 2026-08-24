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
use tessara_composition::{
    ActorEvidenceV1, OwnerBootstrapAuthorizationV1, OwnerBootstrapRequestV1,
    OwnerBootstrapResponseV1,
};
use tessara_forms_contract::{
    FORM_VERSION_SCHEMA_VERSION, FormVersionField, FormVersionSchemaResponse, FormVersionSection,
};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationAudienceV1, MODULE_SERVICE_IDENTITY_REGISTRY_SCHEMA_VERSION_V1,
    ModuleDefinitionId, ModuleServiceIdentityRegistrationV1, ModuleServiceIdentityRegistryV1,
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, ResourceOwner,
};
use tessara_module_runtime::CoreVerifiers;
use tessara_response_module::{
    RESPONSE_BOOTSTRAP_SCHEMA_VERSION, RESPONSE_FORM_BINDING,
    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING, RESPONSE_WORKFLOW_CONTEXT_BINDING,
    ResponseBootstrapDefinitionV1, ResponseBootstrapLifecycleStateV1, ResponseBootstrapReadBackV1,
    ResponseBootstrapV1, ResponseRuntime, ResponseServiceEndpoints, router,
};
use tessara_responses_contract::{ResponseLifecycleState, ResponseReference};
use tessara_workflows_contract::{
    WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION, WorkflowResponseStartContext,
};
use tower::ServiceExt;
use uuid::Uuid;

#[sqlx::test(migrations = "./migrations")]
async fn signed_response_bootstrap_materializes_submits_and_replays(sql_pool: sqlx::PgPool) {
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
    .execute(&sql_pool)
    .await
    .expect("Response security state");

    let owner_signer = signer(
        "tessara.core",
        "response-bootstrap-core",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [41; 32],
    );
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
    let registry = ModuleServiceIdentityRegistryV1 {
        schema_version: MODULE_SERVICE_IDENTITY_REGISTRY_SCHEMA_VERSION_V1,
        identities: BTreeMap::from([(
            ModuleDefinitionId::new("tessara.responses").expect("Response definition"),
            ModuleServiceIdentityRegistrationV1 {
                key_id: "response-bootstrap-owner".into(),
                public_key: URL_SAFE_NO_PAD.encode(service_signer.verifier().public_key_bytes()),
            },
        )]),
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
        CoreVerifiers {
            authorization: authorization_signer.verifier(),
            shell: shell_signer.verifier(),
        },
        service_signer,
        core_service_signer.verifier(),
        registry,
        endpoints,
        owner_signer.verifier(),
        receipt_signer.clone(),
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
            resource_key: "response.submitted.owner".into(),
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
        &owner_signer,
    );

    let first: OwnerBootstrapResponseV1 = send(&app, &request).await;
    assert!(first.receipt.changed);
    assert!(first.has_exact_signed_receipt());
    receipt_signer
        .verifier()
        .verify(&first.signed_receipt)
        .expect("Response receipt signature");
    let reference: ResponseReference =
        serde_json::from_str(first.receipt.resource_ids["response.submitted.owner"].as_str())
            .expect("Response logical reference");
    assert_eq!(reference.reference().installation_id(), installation_id);
    assert_eq!(
        reference.reference().owner(),
        &ResourceOwner::ModuleInstance {
            installation_id,
            module_instance_id,
        }
    );
    let read_back: ResponseBootstrapReadBackV1 = serde_json::from_str(
        first.receipt.resource_ids["response.submitted.owner.read_back"].as_str(),
    )
    .expect("Response typed read-back");
    assert_eq!(read_back.lifecycle_state, ResponseLifecycleState::Submitted);
    assert_eq!(read_back.revision, 2);
    assert_eq!(read_back.workflow_event_sequence, 2);

    let replay: OwnerBootstrapResponseV1 = send(&app, &request).await;
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
}

fn bootstrap_request(
    installation_id: Uuid,
    module_instance_id: Uuid,
    actor_id: Uuid,
    input: ResponseBootstrapV1,
    signer: &PurposeBoundSigningKeyV1,
) -> OwnerBootstrapRequestV1<ResponseBootstrapV1> {
    let input_digest = tessara_composition::canonical_digest(&input).expect("input digest");
    let now = Utc::now();
    let idempotency_key = "response-bootstrap-integration".to_string();
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
            provider_actions: Vec::new(),
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

async fn send(
    app: &Router,
    request: &OwnerBootstrapRequestV1<ResponseBootstrapV1>,
) -> OwnerBootstrapResponseV1 {
    let response = app
        .clone()
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
        .expect("bootstrap response");
    assert_eq!(response.status(), StatusCode::OK);
    response_json(response).await
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
