use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::State,
    http::{HeaderMap, Request, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tessara_composition::{
    OwnerBootstrapAuthorizationV1, OwnerBootstrapProviderActionV1, OwnerBootstrapRequestV1,
    OwnerBootstrapResponseV1,
};
use tessara_control_plane_contract::{
    CONTROL_PLANE_SCHEMA_VERSION, SCOPE_CATALOG_ACTION, SCOPE_CATALOG_BINDING_KEY,
    SCOPE_CATALOG_CONTRACT_ID, SCOPE_CATALOG_MEDIA_TYPE, SCOPE_CATALOG_PATH, ScopeCatalogNode,
    ScopeCatalogRequest, ScopeCatalogResponse,
};
use tessara_dataset_module::{
    DatasetCoreVerifiers, DatasetModuleState, DatasetServiceEndpoints,
    DatasetValidationFaultControl, REQUIRED_DATASET_PROVIDER_BINDINGS, router,
};
use tessara_datasets_contract::DatasetMajorLineReference;
use tessara_forms_contract::{
    FORM_VERSION_SCHEMA_ACTION, FORM_VERSION_SCHEMA_BINDING_KEY, FORM_VERSION_SCHEMA_CONTRACT_ID,
    FORM_VERSION_SCHEMA_MEDIA_TYPE, FORM_VERSION_SCHEMA_PATH, FORM_VERSION_SCHEMA_VERSION,
    FormVersionField, FormVersionSchemaRequest, FormVersionSchemaResponse, FormVersionSection,
};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationAudienceV1, CapabilityScopeBindingV1, ModuleDefinitionId,
    ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, SecurityCapabilityId, ServiceActionMethod, SignedEnvelopeV1,
};
use tessara_responses_contract::{
    RESPONSE_EXPORT_BINDING_KEY, RESPONSE_EXPORT_CHECKPOINT_ACTION,
    RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_CONTRACT_ID, RESPONSE_EXPORT_MEDIA_TYPE,
    RESPONSE_EXPORT_PAGE_ACTION, RESPONSE_EXPORT_PAGE_PATH, RESPONSE_EXPORT_SCHEMA_VERSION,
    RESPONSE_EXPORT_START_ACTION, RESPONSE_EXPORT_START_PATH, ResponseExportCheckpointRequest,
    ResponseExportCheckpointResponse, ResponseExportCursor, ResponseExportEntry,
    ResponseExportPageRequest, ResponseExportPageResponse, ResponseExportStartRequest,
    ResponseExportStartResponse, SubmittedResponseChange, SubmittedResponseRestrictionTier,
    SubmittedResponseUpsert, SubmittedResponseValue,
};
use tower::ServiceExt;
use uuid::Uuid;

const DATASET_BOOTSTRAP_SCHEMA_VERSION: &str = "tessara.io/dataset-bootstrap/v1";

#[derive(Clone)]
struct BootstrapProviders {
    installation_id: Uuid,
    module_instance_id: Uuid,
    scope_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    form_field_id: Uuid,
    provider_epoch: Uuid,
    owner_verifier: PurposeBoundVerifyingKeyV1,
    dataset_service_verifier: PurposeBoundVerifyingKeyV1,
    calls: Arc<Mutex<usize>>,
    incompatible_response_checkpoint: bool,
}

struct BootstrapFixture {
    app: Router,
    installation_id: Uuid,
    module_instance_id: Uuid,
    scope_id: Uuid,
    form_id: Uuid,
    form_version_id: Uuid,
    owner_signer: PurposeBoundSigningKeyV1,
    receipt_verifier: PurposeBoundVerifyingKeyV1,
    provider_calls: Arc<Mutex<usize>>,
    server: tokio::task::JoinHandle<()>,
}

impl Drop for BootstrapFixture {
    fn drop(&mut self) {
        self.server.abort();
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn signed_dataset_bootstrap_materializes_replays_and_returns_typed_read_back(
    pool: sqlx::PgPool,
) {
    let fixture = bootstrap_fixture(pool.clone()).await;
    let input = bootstrap_input(&fixture);
    let request = fixture.request(input, "dataset-bootstrap-success", true, 1);

    let first = fixture.send(&request).await;
    assert_eq!(first.status(), StatusCode::OK);
    let first: OwnerBootstrapResponseV1 = response_json(first).await;
    assert!(first.has_exact_signed_receipt());
    fixture
        .receipt_verifier
        .verify(&first.signed_receipt)
        .expect("Dataset bootstrap receipt signature");
    assert_eq!(first.receipt.owner, "tessara.datasets");
    assert!(first.receipt.changed);
    assert_eq!(first.receipt.input_digest, request.input_digest);

    let major_line: DatasetMajorLineReference = serde_json::from_str(
        first
            .receipt
            .resource_ids
            .get("dataset.base")
            .expect("Dataset logical read-back"),
    )
    .expect("canonical Dataset v2 major-line reference");
    assert_eq!(
        major_line.reference().installation_id(),
        fixture.installation_id
    );
    assert_eq!(major_line.module_instance_id(), fixture.module_instance_id);
    assert_eq!(major_line.major(), 1);
    let typed_read_back: Value = serde_json::from_str(
        first
            .receipt
            .resource_ids
            .get("dataset.base.read_back")
            .expect("typed Dataset read-back"),
    )
    .expect("typed Dataset read-back JSON");
    assert_eq!(typed_read_back["schema_version"], 2);
    assert_eq!(typed_read_back["materialized_row_count"], 1);

    let first_resources = first.receipt.resource_ids.clone();
    let replay = fixture.send(&request).await;
    assert_eq!(replay.status(), StatusCode::OK);
    let replay: OwnerBootstrapResponseV1 = response_json(replay).await;
    assert!(!replay.receipt.changed);
    assert_eq!(replay.receipt.resource_ids, first_resources);
    assert!(replay.has_exact_signed_receipt());
    fixture
        .receipt_verifier
        .verify(&replay.signed_receipt)
        .expect("replay receipt signature");

    let changed_apply = fixture.request(
        bootstrap_input(&fixture),
        "dataset-bootstrap-success",
        true,
        2,
    );
    let conflict = fixture.send(&changed_apply).await;
    assert_eq!(conflict.status(), StatusCode::CONFLICT);

    let counts = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        "SELECT
           (SELECT COUNT(*) FROM datasets),
           (SELECT COUNT(*) FROM dataset_revisions),
           (SELECT COUNT(*) FROM dataset_materialization_receipts),
           (SELECT COUNT(*) FROM dataset_bootstrap_receipts)",
    )
    .fetch_one(&pool)
    .await
    .expect("Dataset bootstrap state counts");
    assert_eq!(counts, (1, 1, 1, 1));
}

#[sqlx::test(migrations = "./migrations")]
async fn missing_exact_provider_action_rolls_back_the_entire_dataset_bootstrap(pool: sqlx::PgPool) {
    let fixture = bootstrap_fixture(pool.clone()).await;
    let request = fixture.request(
        bootstrap_input(&fixture),
        "dataset-bootstrap-missing-page-authority",
        false,
        1,
    );

    let response = fixture.send(&request).await;
    assert_eq!(response.status(), StatusCode::FORBIDDEN);
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT
           (SELECT COUNT(*) FROM datasets),
           (SELECT COUNT(*) FROM dataset_revisions),
           (SELECT COUNT(*) FROM dataset_imported_responses),
           (SELECT COUNT(*) FROM dataset_materialization_receipts),
           (SELECT COUNT(*) FROM dataset_bootstrap_receipts)",
    )
    .fetch_one(&pool)
    .await
    .expect("failed Dataset bootstrap state counts");
    assert_eq!(counts, (0, 0, 0, 0, 0));
}

#[sqlx::test(migrations = "./migrations")]
async fn incompatible_response_checkpoint_is_rejected_before_owner_write(pool: sqlx::PgPool) {
    let fixture = bootstrap_fixture_with_options(
        pool.clone(),
        DatasetValidationFaultControl::disabled(),
        true,
    )
    .await;
    let request = fixture.request(
        bootstrap_input(&fixture),
        "dataset-bootstrap-response-incompatible",
        true,
        1,
    );

    let response = fixture.send(&request).await;
    assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
    let body: Value = response_json(response).await;
    assert_eq!(body["error"]["code"], "dataset.dependency_incompatible");
    let counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
        "SELECT
           (SELECT COUNT(*) FROM datasets),
           (SELECT COUNT(*) FROM dataset_revisions),
           (SELECT COUNT(*) FROM dataset_imported_responses),
           (SELECT COUNT(*) FROM dataset_materialization_receipts),
           (SELECT COUNT(*) FROM dataset_bootstrap_receipts)",
    )
    .fetch_one(&pool)
    .await
    .expect("incompatible Response pre-write state counts");
    assert_eq!(counts, (0, 0, 0, 0, 0));
}

#[cfg(feature = "sprint-8b-validation-faults")]
#[sqlx::test(migrations = "./migrations")]
async fn derived_rebuild_validation_fault_rolls_back_and_cannot_be_bypassed(pool: sqlx::PgPool) {
    let fixture = bootstrap_fixture_with_fault(
        pool.clone(),
        DatasetValidationFaultControl::deterministic_derived_rebuild_for_test(Uuid::new_v4()),
    )
    .await;
    let request = fixture.request(
        bootstrap_input_with_derived(&fixture),
        "dataset-bootstrap-derived-fault",
        true,
        1,
    );

    for _ in 0..2 {
        let response = fixture.send(&request).await;
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body: Value = response_json(response).await;
        assert_eq!(body["error"]["code"], "dataset.dependency_unavailable");
        let counts = sqlx::query_as::<_, (i64, i64, i64, i64, i64)>(
            "SELECT
               (SELECT COUNT(*) FROM datasets),
               (SELECT COUNT(*) FROM dataset_revisions),
               (SELECT COUNT(*) FROM dataset_imported_responses),
               (SELECT COUNT(*) FROM dataset_materialization_receipts),
               (SELECT COUNT(*) FROM dataset_bootstrap_receipts)",
        )
        .fetch_one(&pool)
        .await
        .expect("derived fault rollback state counts");
        assert_eq!(counts, (0, 0, 0, 0, 0));
    }
}

#[sqlx::test(migrations = "./migrations")]
async fn bootstrap_rejects_unauthenticated_malformed_media_and_oversize_before_provider_or_write(
    pool: sqlx::PgPool,
) {
    let fixture = bootstrap_fixture(pool.clone()).await;
    let control_key = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let cases = [
        (None, Some("application/json"), b"{".to_vec()),
        (
            Some(control_key.as_str()),
            Some("text/plain"),
            b"{}".to_vec(),
        ),
        (
            Some(control_key.as_str()),
            Some("application/json"),
            b"{".to_vec(),
        ),
        (
            Some(control_key.as_str()),
            Some("application/json"),
            vec![b' '; 1024 * 1024 + 1],
        ),
    ];
    for (presented_key, media_type, body) in cases {
        let mut builder = Request::builder()
            .method("POST")
            .uri("/api/private/bootstrap");
        if let Some(presented_key) = presented_key {
            builder = builder.header("x-tessara-module-control-key", presented_key);
        }
        if let Some(media_type) = media_type {
            builder = builder.header(header::CONTENT_TYPE, media_type);
        }
        let response = fixture
            .app
            .clone()
            .oneshot(builder.body(Body::from(body)).unwrap())
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::FORBIDDEN | StatusCode::BAD_REQUEST
        ));
        let envelope: Value = response_json(response).await;
        assert!(envelope["error"]["code"].as_str().is_some());
    }
    assert_eq!(
        *fixture.provider_calls.lock().expect("provider call count"),
        0
    );
    let counts = sqlx::query_as::<_, (i64, i64)>(
        "SELECT (SELECT COUNT(*) FROM datasets),
                (SELECT COUNT(*) FROM dataset_bootstrap_receipts)",
    )
    .fetch_one(&pool)
    .await
    .expect("bootstrap rejection state counts");
    assert_eq!(counts, (0, 0));
}

impl BootstrapFixture {
    fn request(
        &self,
        input: Value,
        idempotency_key: &str,
        include_page_action: bool,
        apply_sequence: u64,
    ) -> OwnerBootstrapRequestV1<Value> {
        let input_digest = tessara_composition::canonical_digest(&input)
            .expect("canonical Dataset bootstrap input");
        let now = Utc::now();
        let correlation_id = Uuid::new_v4();
        let authorization = self
            .owner_signer
            .sign(OwnerBootstrapAuthorizationV1 {
                schema_version:
                    tessara_composition::OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1,
                installation_id: self.installation_id,
                owner: AuthorizationAudienceV1::ModuleInstance {
                    module_instance_id: self.module_instance_id,
                    module_definition_id: ModuleDefinitionId::new("tessara.datasets")
                        .expect("Dataset definition ID"),
                },
                owner_definition_id: "tessara.datasets".into(),
                initiator: tessara_composition::ActorEvidenceV1 {
                    actor_id: Uuid::from_u128(801).to_string(),
                    actor_kind: "user".into(),
                    authority: "bootstrap-integration".into(),
                },
                original_actor_id: Uuid::from_u128(801),
                capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("datasets:manage")
                        .expect("Dataset manage capability"),
                    organization_root_id: self.scope_id,
                    authorized_organization_ids: Vec::new(),
                }],
                provider_actions: provider_actions(self.installation_id, include_page_action),
                authorization_revision: 7,
                organization_revision: 11,
                locked_input_digest: input_digest.clone(),
                input_digest: input_digest.clone(),
                desired_revision: 1,
                apply_sequence,
                target_plan_digest: digest('a'),
                idempotency_key: idempotency_key.into(),
                correlation_id,
                jti: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + Duration::seconds(60),
            })
            .expect("signed Dataset bootstrap authority");
        OwnerBootstrapRequestV1 {
            installation_id: self.installation_id,
            desired_revision: 1,
            apply_sequence,
            target_plan_digest: digest('a'),
            idempotency_key: idempotency_key.into(),
            locked_input_digest: input_digest.clone(),
            input_digest,
            authorization,
            dependency_validation: None,
            input,
        }
    }

    async fn send(&self, request: &OwnerBootstrapRequestV1<Value>) -> Response {
        let control_key = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
            .unwrap_or_else(|_| "development-module-control-only".into());
        self.app
            .clone()
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/api/private/bootstrap")
                    .header(header::CONTENT_TYPE, "application/json")
                    .header("x-tessara-module-control-key", control_key)
                    .body(Body::from(
                        serde_json::to_vec(request).expect("serialize Dataset bootstrap request"),
                    ))
                    .expect("Dataset bootstrap request"),
            )
            .await
            .expect("Dataset bootstrap response")
    }
}

async fn bootstrap_fixture(pool: sqlx::PgPool) -> BootstrapFixture {
    bootstrap_fixture_with_options(pool, DatasetValidationFaultControl::disabled(), false).await
}

#[cfg(feature = "sprint-8b-validation-faults")]
async fn bootstrap_fixture_with_fault(
    pool: sqlx::PgPool,
    validation_fault_control: DatasetValidationFaultControl,
) -> BootstrapFixture {
    bootstrap_fixture_with_options(pool, validation_fault_control, false).await
}

async fn bootstrap_fixture_with_options(
    pool: sqlx::PgPool,
    validation_fault_control: DatasetValidationFaultControl,
    incompatible_response_checkpoint: bool,
) -> BootstrapFixture {
    let installation_id = Uuid::new_v4();
    let module_instance_id =
        tessara_composition::module_instance_id(installation_id, "tessara.datasets");
    let scope_id = Uuid::new_v4();
    let form_id = Uuid::new_v4();
    let form_version_id = Uuid::new_v4();
    let form_field_id = Uuid::new_v4();
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
    .expect("install Dataset bootstrap security state");

    let owner_signer = signing_key(
        "tessara.core",
        "bootstrap-integration-core",
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        [41; 32],
    );
    let dataset_service_signer = Arc::new(signing_key(
        "tessara.datasets",
        "bootstrap-integration-dataset",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [42; 32],
    ));
    let receipt_signer = Arc::new(signing_key(
        "tessara.datasets",
        "bootstrap-integration-dataset",
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        [42; 32],
    ));
    let providers = BootstrapProviders {
        installation_id,
        module_instance_id,
        scope_id,
        form_id,
        form_version_id,
        form_field_id,
        provider_epoch: Uuid::new_v4(),
        owner_verifier: owner_signer.verifier(),
        dataset_service_verifier: dataset_service_signer.verifier(),
        calls: Arc::new(Mutex::new(0)),
        incompatible_response_checkpoint,
    };
    let provider_calls = providers.calls.clone();
    let provider_router = Router::new()
        .route(SCOPE_CATALOG_PATH, post(mock_scope_catalog))
        .route(FORM_VERSION_SCHEMA_PATH, post(mock_form_schema))
        .route(RESPONSE_EXPORT_CHECKPOINT_PATH, post(mock_checkpoint))
        .route(RESPONSE_EXPORT_START_PATH, post(mock_start))
        .route(RESPONSE_EXPORT_PAGE_PATH, post(mock_page))
        .with_state(providers);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind Dataset bootstrap providers");
    let address = listener.local_addr().expect("Dataset provider address");
    let server = tokio::spawn(async move {
        axum::serve(listener, provider_router)
            .await
            .expect("serve Dataset bootstrap providers");
    });
    let provider_origin = format!("http://{address}");
    let provider_urls = REQUIRED_DATASET_PROVIDER_BINDINGS
        .into_iter()
        .map(|binding| (binding.to_owned(), provider_origin.clone()))
        .collect::<BTreeMap<_, _>>();
    let authorization_signer = signing_key(
        "tessara.core",
        "bootstrap-integration-core",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        [43; 32],
    );
    let core_service_signer = signing_key(
        "tessara.core",
        "bootstrap-integration-core",
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        [44; 32],
    );
    let bootstrap_validation_signer = signing_key(
        "tessara.core",
        "bootstrap-integration-core",
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        [45; 32],
    );
    let shell_signer = signing_key(
        "tessara.core",
        "bootstrap-integration-core",
        ProtocolSignaturePurposeV1::ShellContext,
        [46; 32],
    );
    let identity_registry = ModuleServiceIdentityRegistryV1::from_json(
        r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"bootstrap-integration-component","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
    )
    .expect("bootstrap integration identity registry");
    let state = DatasetModuleState::new(
        pool,
        DatasetCoreVerifiers {
            authorization: authorization_signer.verifier(),
            owner_bootstrap: owner_signer.verifier(),
            service_request: core_service_signer.verifier(),
            bootstrap_validation: bootstrap_validation_signer.verifier(),
            shell: shell_signer.verifier(),
        },
        identity_registry,
        dataset_service_signer,
        receipt_signer.clone(),
        DatasetServiceEndpoints::new("http://127.0.0.1:9", provider_urls)
            .expect("strict bootstrap provider endpoints"),
        validation_fault_control,
    )
    .expect("Dataset bootstrap state");
    BootstrapFixture {
        app: router(state),
        installation_id,
        module_instance_id,
        scope_id,
        form_id,
        form_version_id,
        owner_signer,
        receipt_verifier: receipt_signer.verifier(),
        provider_calls,
        server,
    }
}

fn bootstrap_input(fixture: &BootstrapFixture) -> Value {
    json!({
        "schema_version": DATASET_BOOTSTRAP_SCHEMA_VERSION,
        "datasets": [{
            "resource_key": "dataset.base",
            "definition": {
                "name": "Bootstrap Base",
                "slug": "bootstrap-base",
                "grain": "submission",
                "version_label": "Initial",
                "force_new_major_version": false,
                "visibility_node_ids": [fixture.scope_id],
                "initial_source": {
                    "kind": "form",
                    "alias": "responses",
                    "form_id": fixture.form_id,
                    "form_version_id": fixture.form_version_id
                },
                "operations": [{
                    "kind": "projection",
                    "fields": [{
                        "key": "score",
                        "label": "Score",
                        "input_field_key": "responses__score",
                        "position": 0
                    }],
                    "position": 0
                }]
            },
            "reference_bindings": []
        }],
        "expected_rejections": []
    })
}

#[cfg(feature = "sprint-8b-validation-faults")]
fn bootstrap_input_with_derived(fixture: &BootstrapFixture) -> Value {
    let mut input = bootstrap_input(fixture);
    input["datasets"]
        .as_array_mut()
        .expect("Dataset bootstrap definitions")
        .push(json!({
            "resource_key": "dataset.derived",
            "definition": {
                "name": "Bootstrap Derived",
                "slug": "bootstrap-derived",
                "grain": "submission",
                "version_label": "Initial",
                "force_new_major_version": false,
                "visibility_node_ids": [fixture.scope_id],
                "initial_source": null,
                "operations": [{
                    "kind": "projection",
                    "fields": [{
                        "key": "score",
                        "label": "Score",
                        "input_field_key": "base__score",
                        "position": 0
                    }],
                    "position": 0
                }]
            },
            "reference_bindings": [{
                "target_pointer": "/initial_source",
                "source_resource_key": "dataset.base",
                "source_alias": "base",
                "selector": {"kind": "major_line", "version_major": 1}
            }]
        }));
    input
}

fn provider_actions(
    installation_id: Uuid,
    include_page_action: bool,
) -> Vec<OwnerBootstrapProviderActionV1> {
    let audience = AuthorizationAudienceV1::CoreInstallation { installation_id };
    let mut actions = vec![
        provider_action(
            SCOPE_CATALOG_BINDING_KEY,
            SCOPE_CATALOG_CONTRACT_ID,
            SCOPE_CATALOG_ACTION,
            SCOPE_CATALOG_PATH,
            &audience,
        ),
        provider_action(
            FORM_VERSION_SCHEMA_BINDING_KEY,
            FORM_VERSION_SCHEMA_CONTRACT_ID,
            FORM_VERSION_SCHEMA_ACTION,
            FORM_VERSION_SCHEMA_PATH,
            &audience,
        ),
        provider_action(
            RESPONSE_EXPORT_BINDING_KEY,
            RESPONSE_EXPORT_CONTRACT_ID,
            RESPONSE_EXPORT_CHECKPOINT_ACTION,
            RESPONSE_EXPORT_CHECKPOINT_PATH,
            &audience,
        ),
        provider_action(
            RESPONSE_EXPORT_BINDING_KEY,
            RESPONSE_EXPORT_CONTRACT_ID,
            RESPONSE_EXPORT_START_ACTION,
            RESPONSE_EXPORT_START_PATH,
            &audience,
        ),
    ];
    if include_page_action {
        actions.push(provider_action(
            RESPONSE_EXPORT_BINDING_KEY,
            RESPONSE_EXPORT_CONTRACT_ID,
            RESPONSE_EXPORT_PAGE_ACTION,
            RESPONSE_EXPORT_PAGE_PATH,
            &audience,
        ));
    }
    actions
}

fn provider_action(
    binding: &str,
    contract: &str,
    action: &str,
    path: &str,
    audience: &AuthorizationAudienceV1,
) -> OwnerBootstrapProviderActionV1 {
    OwnerBootstrapProviderActionV1 {
        dependency_binding: binding.into(),
        functional_contract: contract.into(),
        action: action.into(),
        method: ServiceActionMethod::Post,
        path: path.into(),
        audience: audience.clone(),
    }
}

async fn mock_scope_catalog(
    State(state): State<BootstrapProviders>,
    headers: HeaderMap,
    Json(request): Json<ScopeCatalogRequest>,
) -> Response {
    *state.calls.lock().expect("provider call count") += 1;
    if verify_provider_call(
        &state,
        &headers,
        SCOPE_CATALOG_BINDING_KEY,
        SCOPE_CATALOG_CONTRACT_ID,
        SCOPE_CATALOG_ACTION,
        SCOPE_CATALOG_PATH,
        &request,
    )
    .is_err()
        || request.schema_version != CONTROL_PLANE_SCHEMA_VERSION
        || request.node_ids != vec![state.scope_id]
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    exact_json_response(
        SCOPE_CATALOG_MEDIA_TYPE,
        &ScopeCatalogResponse {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            nodes: vec![ScopeCatalogNode {
                node_id: state.scope_id,
                parent_node_id: None,
                node_type_key: "organization".into(),
                node_type_name: "Organization".into(),
                display_label: "Bootstrap scope".into(),
                node_path: "/bootstrap".into(),
            }],
            requested_set_revision: "scope:1".into(),
            requested_set_digest: format!("sha256:{}", "1".repeat(64)),
        },
    )
}

async fn mock_form_schema(
    State(state): State<BootstrapProviders>,
    headers: HeaderMap,
    Json(request): Json<FormVersionSchemaRequest>,
) -> Response {
    *state.calls.lock().expect("provider call count") += 1;
    if verify_provider_call(
        &state,
        &headers,
        FORM_VERSION_SCHEMA_BINDING_KEY,
        FORM_VERSION_SCHEMA_CONTRACT_ID,
        FORM_VERSION_SCHEMA_ACTION,
        FORM_VERSION_SCHEMA_PATH,
        &request,
    )
    .is_err()
        || request.form_version_id != state.form_version_id
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let section_id = Uuid::from_u128(802);
    let response = FormVersionSchemaResponse {
        schema_version: FORM_VERSION_SCHEMA_VERSION,
        form_id: state.form_id,
        form_version_id: state.form_version_id,
        form_name: "Bootstrap Form".into(),
        form_slug: "bootstrap-form".into(),
        version_label: Some("Published".into()),
        version_major: Some(1),
        source_scope_node_ids: vec![state.scope_id],
        source_scope_revision: String::new(),
        source_scope_digest: String::new(),
        content_revision: String::new(),
        content_digest: String::new(),
        sections: vec![FormVersionSection {
            section_id,
            key: "metrics".into(),
            label: "Metrics".into(),
            description: String::new(),
            position: 0,
        }],
        fields: vec![FormVersionField {
            field_id: state.form_field_id,
            key: "score".into(),
            label: "Score".into(),
            field_type: "number".into(),
            required: true,
            options: Vec::new(),
            section_id: Some(section_id),
            position: 0,
            grid_row: 1,
            grid_column: 1,
            grid_width: 6,
            grid_height: 1,
        }],
    }
    .with_recomputed_digests()
    .expect("canonical bootstrap Form schema");
    exact_json_response(FORM_VERSION_SCHEMA_MEDIA_TYPE, &response)
}

async fn mock_checkpoint(
    State(state): State<BootstrapProviders>,
    headers: HeaderMap,
    Json(request): Json<ResponseExportCheckpointRequest>,
) -> Response {
    *state.calls.lock().expect("provider call count") += 1;
    if verify_provider_call(
        &state,
        &headers,
        RESPONSE_EXPORT_BINDING_KEY,
        RESPONSE_EXPORT_CONTRACT_ID,
        RESPONSE_EXPORT_CHECKPOINT_ACTION,
        RESPONSE_EXPORT_CHECKPOINT_PATH,
        &request,
    )
    .is_err()
        || request
            .partition
            .validate_authorization_digest(
                &dataset_principal(state.module_instance_id),
                state.installation_id,
            )
            .is_err()
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    if state.incompatible_response_checkpoint {
        return exact_json_response(
            RESPONSE_EXPORT_MEDIA_TYPE,
            &json!({"schema_version": 65_535, "fault": "incompatible_provider_contract"}),
        );
    }
    exact_json_response(
        RESPONSE_EXPORT_MEDIA_TYPE,
        &ResponseExportCheckpointResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: state.provider_epoch,
            authenticated_head: ResponseExportCursor::parse("bootstrap:0001")
                .expect("bootstrap cursor"),
            committed_cursor_valid: true,
            changed: true,
        },
    )
}

async fn mock_start(
    State(state): State<BootstrapProviders>,
    headers: HeaderMap,
    Json(request): Json<ResponseExportStartRequest>,
) -> Response {
    *state.calls.lock().expect("provider call count") += 1;
    if verify_provider_call(
        &state,
        &headers,
        RESPONSE_EXPORT_BINDING_KEY,
        RESPONSE_EXPORT_CONTRACT_ID,
        RESPONSE_EXPORT_START_ACTION,
        RESPONSE_EXPORT_START_PATH,
        &request,
    )
    .is_err()
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    exact_json_response(
        RESPONSE_EXPORT_MEDIA_TYPE,
        &ResponseExportStartResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: request.provider_epoch,
            start_after_cursor: request.committed_cursor,
            snapshot_upper_bound: request.authenticated_head,
            full_snapshot_rebase: request.full_snapshot_rebase,
        },
    )
}

async fn mock_page(
    State(state): State<BootstrapProviders>,
    headers: HeaderMap,
    Json(request): Json<ResponseExportPageRequest>,
) -> Response {
    *state.calls.lock().expect("provider call count") += 1;
    if verify_provider_call(
        &state,
        &headers,
        RESPONSE_EXPORT_BINDING_KEY,
        RESPONSE_EXPORT_CONTRACT_ID,
        RESPONSE_EXPORT_PAGE_ACTION,
        RESPONSE_EXPORT_PAGE_PATH,
        &request,
    )
    .is_err()
    {
        return StatusCode::FORBIDDEN.into_response();
    }
    let change = SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
        response_id: Uuid::from_u128(803),
        form_id: state.form_id,
        form_version_id: state.form_version_id,
        node_id: state.scope_id,
        node_name: "Bootstrap scope".into(),
        submitted_at: "2026-08-13T12:01:00Z".into(),
        created_at: "2026-08-13T12:00:00Z".into(),
        last_modified_at: "2026-08-13T12:02:00Z".into(),
        last_modified_by_user_name: Some("Bootstrap owner".into()),
        status: "submitted".into(),
        restriction_tier: SubmittedResponseRestrictionTier::Public,
        scope_node_ids: vec![state.scope_id],
        values: BTreeMap::from([(
            "score".into(),
            SubmittedResponseValue {
                field_id: state.form_field_id,
                value: json!(42),
                value_text: Some("42".into()),
            },
        )]),
        content_digest: String::new(),
    }))
    .with_recomputed_content_digest()
    .expect("canonical bootstrap Response change");
    let entries = vec![ResponseExportEntry {
        cursor: ResponseExportCursor::parse("bootstrap:0001").expect("bootstrap cursor"),
        change,
    }];
    exact_json_response(
        RESPONSE_EXPORT_MEDIA_TYPE,
        &ResponseExportPageResponse {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            provider_epoch: request.provider_epoch,
            snapshot_upper_bound: request.snapshot_upper_bound,
            page_digest: ResponseExportPageResponse::canonical_page_digest(&entries)
                .expect("bootstrap page digest"),
            entries,
            next_after_cursor: None,
            complete: true,
        },
    )
}

fn verify_provider_call<T: Serialize>(
    state: &BootstrapProviders,
    headers: &HeaderMap,
    binding: &str,
    contract: &str,
    action: &str,
    path: &str,
    request: &T,
) -> Result<(), ()> {
    if headers.contains_key("x-tessara-authorization") {
        return Err(());
    }
    let encoded_authorization = headers
        .get("x-tessara-owner-bootstrap-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(())?;
    let authorization: SignedEnvelopeV1<OwnerBootstrapAuthorizationV1> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded_authorization)
            .map_err(|_| ())?,
    )
    .map_err(|_| ())?;
    state
        .owner_verifier
        .verify(&authorization)
        .map_err(|_| ())?;
    let expected_owner = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: state.module_instance_id,
        module_definition_id: ModuleDefinitionId::new("tessara.datasets").map_err(|_| ())?,
    };
    if authorization.payload.installation_id != state.installation_id
        || authorization.payload.owner != expected_owner
        || !authorization
            .payload
            .provider_actions
            .iter()
            .any(|provider_action| {
                provider_action.dependency_binding == binding
                    && provider_action.functional_contract == contract
                    && provider_action.action == action
                    && provider_action.method == ServiceActionMethod::Post
                    && provider_action.path == path
                    && provider_action.audience
                        == AuthorizationAudienceV1::CoreInstallation {
                            installation_id: state.installation_id,
                        }
            })
    {
        return Err(());
    }
    let encoded_service_request = headers
        .get("x-tessara-module-service-request")
        .and_then(|value| value.to_str().ok())
        .ok_or(())?;
    let service_request: SignedEnvelopeV1<ModuleServiceRequestV1> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded_service_request)
            .map_err(|_| ())?,
    )
    .map_err(|_| ())?;
    state
        .dataset_service_verifier
        .verify(&service_request)
        .map_err(|_| ())?;
    let body = serde_json::to_vec(request).map_err(|_| ())?;
    service_request
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id: state.installation_id,
            module_instance_id: state.module_instance_id,
            module_definition_id: ModuleDefinitionId::new("tessara.datasets").map_err(|_| ())?,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(&body),
            inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
            correlation_id: authorization.payload.correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| ())
}

fn dataset_principal(module_instance_id: Uuid) -> ModuleServicePrincipalV1 {
    ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id,
        module_definition_id: ModuleDefinitionId::new("tessara.datasets")
            .expect("Dataset definition ID"),
    }
}

fn exact_json_response<T: Serialize>(media_type: &'static str, value: &T) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, media_type)
        .body(Body::from(
            serde_json::to_vec(value).expect("serialize bootstrap provider response"),
        ))
        .expect("bootstrap provider response")
}

fn signing_key(
    issuer: &str,
    key_id: &str,
    purpose: ProtocolSignaturePurposeV1,
    secret: [u8; 32],
) -> PurposeBoundSigningKeyV1 {
    PurposeBoundSigningKeyV1::from_secret_bytes(issuer, key_id, purpose, secret)
        .expect("bootstrap integration signing key")
}

fn digest(value: char) -> ArtifactDigest {
    ArtifactDigest::new(format!("sha256:{}", value.to_string().repeat(64)))
        .expect("test artifact digest")
}

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn response_json<T: DeserializeOwned>(response: Response) -> T {
    let bytes = to_bytes(response.into_body(), usize::MAX)
        .await
        .expect("read Dataset bootstrap response");
    serde_json::from_slice(&bytes).unwrap_or_else(|error| {
        panic!(
            "Dataset bootstrap response is not canonical JSON ({error}): {}",
            String::from_utf8_lossy(&bytes)
        )
    })
}
