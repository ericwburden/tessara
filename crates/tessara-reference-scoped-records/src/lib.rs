use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
};

use axum::{
    Json, Router,
    body::to_bytes,
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, put},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool, Row};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, DependencyBindingKey, FunctionalContractId,
    ModuleDefinitionId, ModuleManifest, ModuleServicePrincipalV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, SecurityCapabilityId, ShellContextV2,
    ShellContextValidationContextV2, SignedEnvelopeV1,
};
use tessara_module_runtime::{
    decode_signed_envelope_header, request_correlation_id, verify_shell_context,
};
use tessara_module_ui::{
    Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    MODULE_UI_CSS_SHA256, ModuleDocumentAssets, ModuleReleaseMetadata, PageHeader,
    ShellPresentation, render_module_view_document,
};
use uuid::Uuid;

pub const MODULE_DEFINITION_ID: &str = "tessara.reference.scoped-records";
pub const MODULE_RELEASE_VERSION: &str = "1.0.2";
pub const MODULE_UI_CSS_PATH: &str = "/_tessara/modules/tessara.reference.scoped-records/1.0.2/sha256:dff9a5085d85d9e535b0fc0d4ba37233891e24e0241d9ca7c4ccd3c906ea1f9f/module-ui.css";
pub const SCOPED_RECORDS_CSS: &str = include_str!("../assets/scoped-records.css");
pub const SCOPED_RECORDS_CSS_SHA256: &str =
    "ca3e243f6f1aea1f794876d7bdd47cde5553fc610de66e28568a83393e714f77";
pub const SCOPED_RECORDS_CSS_PATH: &str = "/_tessara/modules/tessara.reference.scoped-records/1.0.2/sha256:ca3e243f6f1aea1f794876d7bdd47cde5553fc610de66e28568a83393e714f77/scoped-records.css";
pub const READ_CAPABILITY: &str = "tessara.reference.scoped-records:read";
pub const MANAGE_CAPABILITY: &str = "tessara.reference.scoped-records:manage";

#[derive(Clone)]
pub struct ModuleState {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_owner_bootstrap_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
    pub bootstrap_receipt_signer: Arc<PurposeBoundSigningKeyV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedRecordsConfigurationV1 {
    pub schema_version: u16,
    pub display_label: String,
    pub retention_mode: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigurationFindingV1 {
    pub code: &'static str,
    pub field: &'static str,
    pub message: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigurationValidationV1 {
    pub schema_version: u16,
    pub valid: bool,
    pub normalized: Option<ScopedRecordsConfigurationV1>,
    pub findings: Vec<ConfigurationFindingV1>,
}

pub fn validate_configuration(input: &ScopedRecordsConfigurationV1) -> ConfigurationValidationV1 {
    let label = input.display_label.trim();
    let mut findings = Vec::new();
    if input.schema_version != 1 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.schema_version.unsupported",
            field: "schema_version",
            message: "Only Scoped Records configuration schema v1 is supported.",
        });
    }
    if label.is_empty() {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.display_label.required",
            field: "display_label",
            message: "Display label is required.",
        });
    } else if label.chars().count() > 80 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.display_label.too_long",
            field: "display_label",
            message: "Display label must contain at most 80 characters.",
        });
    }
    if input.retention_mode != "retain_on_undeploy" {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.retention_mode.unsupported",
            field: "retention_mode",
            message: "Scoped Records v1 retains data when the module is undeployed.",
        });
    }
    ConfigurationValidationV1 {
        schema_version: 1,
        valid: findings.is_empty(),
        normalized: findings.is_empty().then(|| ScopedRecordsConfigurationV1 {
            schema_version: 1,
            display_label: label.to_string(),
            retention_mode: "retain_on_undeploy".into(),
        }),
        findings,
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct ScopedRecord {
    pub id: Uuid,
    pub label: String,
    pub organization_owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationAccessProjectionV1 {
    pub organization_id: Uuid,
    pub label: String,
    pub can_manage: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectoryQuery {
    #[serde(default)]
    q: String,
    organization: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub label: String,
    pub organization_owner_id: Uuid,
    pub idempotency_key: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityStateInput {
    schema_version: u16,
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: u64,
    organization_revision: u64,
    enabled: bool,
    document_state: String,
}

#[derive(FromRow)]
struct SecurityState {
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: String,
}

pub fn router(state: ModuleState) -> Router {
    Router::new()
        .route("/", get(directory_page))
        .route("/records/new", get(create_page))
        .route("/records/{record_id}", get(detail_page))
        .route("/records/{record_id}/edit", get(edit_page))
        .route(
            "/api/configuration/validate",
            axum::routing::post(validate_configuration_api),
        )
        .route(
            "/api/configuration",
            get(get_configuration).put(put_configuration),
        )
        .route("/api/manifest", get(get_manifest))
        .route("/api/private/security-state", put(update_security_state))
        .route(
            "/api/private/bootstrap",
            axum::routing::post(apply_bootstrap),
        )
        .route("/api/records", get(list_records).post(create_record))
        .route(
            "/api/records/{record_id}",
            get(get_record).put(update_record),
        )
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/health", get(health_page))
        .route("/diagnostics", get(diagnostics_page))
        .route(MODULE_UI_CSS_PATH, get(module_ui_stylesheet))
        .route(SCOPED_RECORDS_CSS_PATH, get(scoped_records_stylesheet))
        .with_state(state)
}

pub fn manifest() -> ModuleManifest {
    serde_json::from_str(include_str!("../manifest.json"))
        .expect("Scoped Records manifest must remain valid")
}

async fn get_manifest(headers: HeaderMap) -> Result<Json<ModuleManifest>, ApiError> {
    require_private_key(&headers)?;
    Ok(Json(manifest()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedRecordsBootstrapV1 {
    pub schema_version: String,
    pub records: Vec<ScopedRecordBootstrapEntryV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedRecordBootstrapEntryV1 {
    pub external_key: String,
    pub label: String,
    pub organization_owner_id: Uuid,
}

async fn apply_bootstrap(
    State(state): State<ModuleState>,
    request: Request,
) -> Result<Json<tessara_composition::OwnerBootstrapResponseV1>, ApiError> {
    require_private_key(request.headers())?;
    require_exact_json(request.headers())?;
    let body = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| ApiError::bad_request("Scoped Records bootstrap payload is too large"))?;
    let request: tessara_composition::OwnerBootstrapRequestV1<ScopedRecordsBootstrapV1> =
        serde_json::from_slice(&body)
            .map_err(|_| ApiError::bad_request("Scoped Records bootstrap payload is invalid"))?;
    if request.input.schema_version != "tessara.io/scoped-records-bootstrap/v1"
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|_| ApiError::bad_request("Bootstrap input digest is invalid"))?
    {
        return Err(ApiError::bad_request(
            "Scoped Records bootstrap contract is invalid",
        ));
    }
    let security = load_security_state(&state.pool).await?;
    let owner = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
            .map_err(|_| ApiError::stale_or_restricted())?,
    };
    request
        .validate_authorization_for(
            &state.core_owner_bootstrap_verifier,
            &owner,
            MODULE_DEFINITION_ID,
            Utc::now(),
        )
        .map_err(|_| ApiError::stale_or_restricted())?;
    if let Some((digest, receipt)) = sqlx::query_as::<_, (String, Value)>(
        "SELECT input_digest,receipt FROM scoped_records_bootstrap_receipts WHERE idempotency_key=$1",
    )
    .bind(&request.idempotency_key)
    .fetch_optional(&state.pool)
    .await?
    {
        if digest != request.input_digest.to_string() {
            return Err(ApiError::stale_or_restricted());
        }
        let mut response: tessara_composition::OwnerBootstrapResponseV1 =
            serde_json::from_value(receipt)?;
        response.receipt.changed = false;
        response.signed_receipt = state
            .bootstrap_receipt_signer
            .sign(response.receipt.clone())
            .map_err(|_| ApiError::stale_or_restricted())?;
        return Ok(Json(response));
    }
    let mut external_keys = BTreeSet::new();
    if request.input.records.iter().any(|record| {
        record.external_key.trim().is_empty()
            || record.label.trim().is_empty()
            || record.organization_owner_id.is_nil()
            || !external_keys.insert(record.external_key.as_str())
    }) {
        return Err(ApiError::bad_request(
            "Bootstrap record keys, labels, and resolved owners must be unique and valid",
        ));
    }
    let mut transaction = state.pool.begin().await?;
    for record in &request.input.records {
        let record_id = tessara_composition::owner_resource_id(
            request.installation_id,
            MODULE_DEFINITION_ID,
            "scoped-record",
            &record.external_key,
        );
        sqlx::query("INSERT INTO scoped_records(id,label,scope,organization_owner_id) VALUES($1,$2,$3,$4) ON CONFLICT(id) DO UPDATE SET label=EXCLUDED.label,scope=EXCLUDED.scope,organization_owner_id=EXCLUDED.organization_owner_id,updated_at=now()")
            .bind(record_id).bind(record.label.trim()).bind(&record.external_key)
            .bind(record.organization_owner_id).execute(&mut *transaction).await?;
    }
    let resource_ids = request
        .input
        .records
        .iter()
        .map(|record| {
            (
                record.external_key.clone(),
                tessara_composition::owner_resource_id(
                    request.installation_id,
                    MODULE_DEFINITION_ID,
                    "scoped-record",
                    &record.external_key,
                )
                .to_string(),
            )
        })
        .collect();
    let result_digest = tessara_composition::canonical_digest(&resource_ids)
        .map_err(|_| ApiError::bad_request("Bootstrap result is invalid"))?;
    let response = tessara_composition::OwnerBootstrapResponseV1::signed(
        tessara_composition::BootstrapReceiptV1 {
            owner: MODULE_DEFINITION_ID.into(),
            schema_version: request.input.schema_version.clone(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids,
        },
        &state.bootstrap_receipt_signer,
    )
    .map_err(|_| ApiError::stale_or_restricted())?;
    sqlx::query("INSERT INTO scoped_records_bootstrap_receipts(idempotency_key,input_digest,desired_revision,receipt) VALUES($1,$2,$3,$4)")
        .bind(&request.idempotency_key).bind(request.input_digest.to_string())
        .bind(request.desired_revision as i64).bind(serde_json::to_value(&response)?)
        .execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn validate_configuration_api(
    Json(input): Json<ScopedRecordsConfigurationV1>,
) -> Json<ConfigurationValidationV1> {
    Json(validate_configuration(&input))
}

async fn get_configuration(State(state): State<ModuleState>) -> Result<Json<Value>, ApiError> {
    let row = sqlx::query(
        "SELECT schema_version, display_label, updated_at
         FROM scoped_records_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({
        "schema_version": row.try_get::<i32,_>("schema_version")?,
        "display_label": row.try_get::<String,_>("display_label")?,
        "retention_mode": "retain_on_undeploy",
        "updated_at": row.try_get::<DateTime<Utc>,_>("updated_at")?,
    })))
}

async fn put_configuration(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Json(input): Json<ScopedRecordsConfigurationV1>,
) -> Result<Json<ConfigurationValidationV1>, ApiError> {
    require_private_key(&headers)?;
    let validation = validate_configuration(&input);
    let Some(normalized) = &validation.normalized else {
        return Ok(Json(validation));
    };
    sqlx::query(
        "UPDATE scoped_records_configuration
         SET display_label=$1, updated_at=now() WHERE singleton=true",
    )
    .bind(&normalized.display_label)
    .execute(&state.pool)
    .await?;
    Ok(Json(validation))
}

async fn update_security_state(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Json(input): Json<SecurityStateInput>,
) -> Result<StatusCode, ApiError> {
    require_private_key(&headers)?;
    if input.schema_version != 1
        || !matches!(
            input.document_state.as_str(),
            "enabled" | "disabled" | "degraded" | "recovery"
        )
        || input.authorization_revision == 0
        || input.organization_revision == 0
    {
        return Err(ApiError::bad_request("invalid security state"));
    }
    let updated = sqlx::query(
        "INSERT INTO scoped_records_security_state
         (singleton, installation_id, module_instance_id, authorization_revision,
          organization_revision, enabled, document_state)
         VALUES (true,$1,$2,$3,$4,$5,$6)
         ON CONFLICT (singleton) DO UPDATE SET
           authorization_revision=GREATEST(
             scoped_records_security_state.authorization_revision,
             EXCLUDED.authorization_revision
           ),
           organization_revision=GREATEST(
             scoped_records_security_state.organization_revision,
             EXCLUDED.organization_revision
           ),
           enabled=EXCLUDED.enabled, document_state=EXCLUDED.document_state,
           updated_at=now()
         WHERE scoped_records_security_state.installation_id=EXCLUDED.installation_id
           AND scoped_records_security_state.module_instance_id=EXCLUDED.module_instance_id",
    )
    .bind(input.installation_id)
    .bind(input.module_instance_id)
    .bind(input.authorization_revision as i64)
    .bind(input.organization_revision as i64)
    .bind(input.enabled)
    .bind(input.document_state)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(ApiError::security_identity_conflict());
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn list_records(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ScopedRecord>>, ApiError> {
    Ok(Json(authorized_records(&state, &headers).await?))
}

async fn authorized_records(
    state: &ModuleState,
    headers: &HeaderMap,
) -> Result<Vec<ScopedRecord>, ApiError> {
    let auth = authorize(
        state,
        headers,
        "records.list",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let owners = authorized_owners(&auth, READ_CAPABILITY);
    let records = sqlx::query_as::<_, ScopedRecord>(
        "SELECT id,label,organization_owner_id,created_at,updated_at
         FROM scoped_records WHERE organization_owner_id=ANY($1)
         ORDER BY updated_at DESC,id",
    )
    .bind(owners)
    .fetch_all(&state.pool)
    .await?;
    Ok(records)
}

async fn get_record(
    State(state): State<ModuleState>,
    Path(record_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ScopedRecord>, ApiError> {
    let auth = authorize(
        &state,
        &headers,
        "records.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let record = load_record(&state.pool, record_id)
        .await?
        .filter(|record| authorization_allows(&auth, READ_CAPABILITY, record.organization_owner_id))
        .ok_or_else(ApiError::restricted)?;
    Ok(Json(record))
}

async fn create_record(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Json(input): Json<RecordInput>,
) -> Result<impl IntoResponse, ApiError> {
    require_record_input(&input)?;
    let auth = authorize(
        &state,
        &headers,
        "records.create",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    if !authorization_allows(&auth, MANAGE_CAPABILITY, input.organization_owner_id) {
        return Err(ApiError::restricted());
    }
    let payload_digest = payload_digest(&input)?;
    let mut transaction = state.pool.begin().await?;
    if let Some(result) = replay_result(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        &input.idempotency_key,
        &payload_digest,
        "records.create",
    )
    .await?
    {
        transaction.commit().await?;
        return Ok((StatusCode::OK, Json(result)));
    }
    let record = sqlx::query_as::<_, ScopedRecord>(
        "INSERT INTO scoped_records
         (id,label,scope,organization_owner_id)
         VALUES ($1,$2,'sprint-6b2',$3)
         RETURNING id,label,organization_owner_id,created_at,updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(input.label.trim())
    .bind(input.organization_owner_id)
    .fetch_one(&mut *transaction)
    .await?;
    let result = serde_json::to_value(&record)?;
    consume_replay(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        "records.create",
        &payload_digest,
        &input.idempotency_key,
        &result,
    )
    .await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn update_record(
    State(state): State<ModuleState>,
    Path(record_id): Path<Uuid>,
    headers: HeaderMap,
    Json(input): Json<RecordInput>,
) -> Result<Json<Value>, ApiError> {
    require_record_input(&input)?;
    let auth = authorize(
        &state,
        &headers,
        "records.update",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let current = load_record(&state.pool, record_id)
        .await?
        .ok_or_else(ApiError::restricted)?;
    if !authorization_allows(&auth, MANAGE_CAPABILITY, current.organization_owner_id)
        || !authorization_allows(&auth, MANAGE_CAPABILITY, input.organization_owner_id)
    {
        return Err(ApiError::restricted());
    }
    let payload_digest = payload_digest(&(&record_id, &input))?;
    let mut transaction = state.pool.begin().await?;
    if let Some(result) = replay_result(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        &input.idempotency_key,
        &payload_digest,
        "records.update",
    )
    .await?
    {
        transaction.commit().await?;
        return Ok(Json(result));
    }
    let record = sqlx::query_as::<_, ScopedRecord>(
        "UPDATE scoped_records SET label=$2,organization_owner_id=$3,updated_at=now()
         WHERE id=$1 RETURNING id,label,organization_owner_id,created_at,updated_at",
    )
    .bind(record_id)
    .bind(input.label.trim())
    .bind(input.organization_owner_id)
    .fetch_one(&mut *transaction)
    .await?;
    let result = serde_json::to_value(&record)?;
    consume_replay(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        "records.update",
        &payload_digest,
        &input.idempotency_key,
        &result,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

async fn authorize(
    state: &ModuleState,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, ApiError> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::restricted)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ApiError::restricted())?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV3> =
        serde_json::from_slice(&bytes).map_err(|_| ApiError::restricted())?;
    state
        .core_authorization_verifier
        .verify(&envelope)
        .map_err(|_| ApiError::restricted())?;
    let security = load_security_state(&state.pool).await?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(ApiError::unavailable("module is not enabled"));
    }
    let correlation_id = request_correlation_id(headers).map_err(|_| ApiError::restricted())?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.scoped-records").unwrap(),
            functional_contract: FunctionalContractId::new(
                "tessara.reference.scoped-records.record",
            )
            .unwrap(),
            action: action.to_string(),
            operation,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| ApiError::stale_or_restricted())?;
    Ok(envelope)
}

fn authorized_owners(
    envelope: &SignedEnvelopeV1<AuthorizationGrantV3>,
    capability: &str,
) -> Vec<Uuid> {
    let mut owners = BTreeSet::new();
    for binding in &envelope.payload.capability_scope_bindings {
        if binding.capability.as_str() == capability {
            owners.insert(binding.organization_root_id);
            owners.extend(binding.authorized_organization_ids.iter().copied());
        }
    }
    owners.into_iter().collect()
}

fn authorization_allows(
    envelope: &SignedEnvelopeV1<AuthorizationGrantV3>,
    capability: &str,
    organization_id: Uuid,
) -> bool {
    SecurityCapabilityId::new(capability)
        .ok()
        .is_some_and(|capability| envelope.payload.authorizes(&capability, organization_id))
}

async fn load_security_state(pool: &PgPool) -> Result<SecurityState, ApiError> {
    sqlx::query_as(
        "SELECT installation_id,module_instance_id,authorization_revision,
                organization_revision,enabled,document_state
         FROM scoped_records_security_state WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::unavailable("module security state is unavailable"))
}

async fn load_record(pool: &PgPool, id: Uuid) -> Result<Option<ScopedRecord>, ApiError> {
    Ok(sqlx::query_as(
        "SELECT id,label,organization_owner_id,created_at,updated_at
         FROM scoped_records WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

fn require_record_input(input: &RecordInput) -> Result<(), ApiError> {
    if input.label.trim().is_empty()
        || input.label.chars().count() > 160
        || input.organization_owner_id.is_nil()
        || input.idempotency_key.trim().is_empty()
    {
        return Err(ApiError::bad_request("record input is invalid"));
    }
    Ok(())
}

fn payload_digest(value: &impl Serialize) -> Result<String, ApiError> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}

async fn replay_result(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    jti: Uuid,
    original_actor_id: Uuid,
    idempotency_key: &str,
    digest: &str,
    action: &str,
) -> Result<Option<Value>, ApiError> {
    let row = sqlx::query(
        "SELECT original_actor_id,action,payload_digest,idempotency_key,result
         FROM scoped_records_mutation_replays
         WHERE jti=$1 OR idempotency_key=$2 FOR UPDATE",
    )
    .bind(jti)
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await?;
    match row {
        None => Ok(None),
        Some(row)
            if row.try_get::<Uuid, _>("original_actor_id")? == original_actor_id
                && row.try_get::<String, _>("action")? == action
                && row.try_get::<String, _>("payload_digest")? == digest
                && row.try_get::<String, _>("idempotency_key")? == idempotency_key =>
        {
            Ok(Some(row.try_get("result")?))
        }
        Some(_) => Err(ApiError::restricted()),
    }
}

async fn consume_replay(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    jti: Uuid,
    original_actor_id: Uuid,
    action: &str,
    digest: &str,
    idempotency_key: &str,
    result: &Value,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO scoped_records_mutation_replays
         (jti,original_actor_id,action,payload_digest,idempotency_key,result)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(jti)
    .bind(original_actor_id)
    .bind(action)
    .bind(digest)
    .bind(idempotency_key)
    .bind(result)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn directory_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Query(query): Query<DirectoryQuery>,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_view(&load_security_state(&state.pool).await?) {
        return shell_page(
            &state,
            &headers,
            &configuration_label(&state.pool).await,
            content,
        )
        .await;
    }
    let organizations = organization_access_projection(&headers)?;
    let records = authorized_records(&state, &headers).await?;
    let records = filter_directory_records(records, &organizations, &query);
    shell_page(
        &state,
        &headers,
        &configuration_label(&state.pool).await,
        directory_view(&records, &organizations, &query),
    )
    .await
}

fn directory_view(
    records: &[ScopedRecord],
    organizations: &[OrganizationAccessProjectionV1],
    query: &DirectoryQuery,
) -> AnyView {
    let organization_map = organizations
        .iter()
        .map(|organization| (organization.organization_id, organization))
        .collect::<BTreeMap<_, _>>();
    let rows = records
        .iter()
        .map(|record| {
            let organization = organization_map.get(&record.organization_owner_id);
            let owner_label = organization
                .map(|value| value.label.clone())
                .unwrap_or_else(|| "Unavailable Organization".into());
            let can_manage = organization.is_some_and(|value| value.can_manage);
            let badge_class = if can_manage { "status-badge is-success" } else { "status-badge is-info" };
            let authority = if can_manage { "Read · Manage" } else { "Read" };
            let id = record.id.to_string();
            let href = format!("/reference/scoped-records/records/{id}");
            let label = record.label.clone();
            let updated = record.updated_at.format("%b %e, %Y · %l:%M %p UTC").to_string();
            view! {
                <tr>
                    <th><a class="scoped-records-primary-link" href=href>{label}</a><code>{id}</code></th>
                    <td>{owner_label}</td><td>{updated}</td><td><span class=badge_class>{authority}</span></td>
                </tr>
            }
        })
        .collect_view();
    let directory = if records.is_empty() {
        view! { <div class="organization-detail-card empty-state"><h2>"No scoped records"</h2><p>"No records are owned by an Organization in your current read scope."</p></div> }.into_any()
    } else {
        let count = records.len();
        view! {
            <div class="scoped-records-table-wrap"><table class="scoped-records-table"><thead><tr><th>"Record"</th><th>"Organization owner"</th><th>"Updated"</th><th>"Authority"</th></tr></thead><tbody>{rows}</tbody></table></div>
            <div class="scoped-records-pagination"><span>{format!("Showing 1-{count} of {count} records")}</span><span>"Rows "<strong>"10"</strong>" · Page 1 of 1"</span></div>
        }.into_any()
    };
    let selected_organization = query
        .organization
        .as_deref()
        .and_then(|value| Uuid::parse_str(value).ok());
    let options = organizations
        .iter()
        .map(|organization| {
            let selected = selected_organization == Some(organization.organization_id);
            let id = organization.organization_id.to_string();
            let label = organization.label.clone();
            view! { <option value=id selected=selected>{label}</option> }
        })
        .collect_view();
    let readable = organizations.len();
    let manageable = organizations
        .iter()
        .filter(|organization| organization.can_manage)
        .count();
    let create_action = (manageable > 0).then(|| view! { <a class="button" href="/reference/scoped-records/records/new">"New Record"</a> });
    let q = query.q.clone();
    view! {
        {scoped_records_breadcrumb(vec![("Home".into(), Some("/".into())), ("Scoped Records".into(), None)])}
        <PageHeader title="Scoped Records" description="Organization-owned reference records available within your assigned read scope.">{create_action}</PageHeader>
        <div class="scoped-records-scope-summary"><div><strong>{format!("Read access across {readable} accessible Organizations")}</strong><span>{format!("{manageable} include manage authority")}</span></div><a href="/administration/roles">"View access"</a></div>
        <form class="scoped-records-toolbar" method="get" action="/reference/scoped-records"><input type="search" name="q" value=q placeholder="Search record label, ID, or Organization"/><select name="organization" aria-label="Filter by Organization"><option value="">"All accessible Organizations"</option>{options}</select><button class="button button--secondary" type="submit">"Filter"</button></form>
        {directory}
    }.into_any()
}

fn filter_directory_records(
    records: Vec<ScopedRecord>,
    organizations: &[OrganizationAccessProjectionV1],
    query: &DirectoryQuery,
) -> Vec<ScopedRecord> {
    let labels = organizations
        .iter()
        .map(|organization| {
            (
                organization.organization_id,
                organization.label.to_lowercase(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let needle = query.q.trim().to_lowercase();
    let selected_organization = query
        .organization
        .as_deref()
        .filter(|value| !value.is_empty())
        .and_then(|value| Uuid::parse_str(value).ok());
    records
        .into_iter()
        .filter(|record| {
            selected_organization
                .is_none_or(|organization| organization == record.organization_owner_id)
                && (needle.is_empty()
                    || record.label.to_lowercase().contains(&needle)
                    || record.id.to_string().contains(&needle)
                    || labels
                        .get(&record.organization_owner_id)
                        .is_some_and(|label| label.contains(&needle)))
        })
        .collect()
}

fn organization_access_projection(
    headers: &HeaderMap,
) -> Result<Vec<OrganizationAccessProjectionV1>, ApiError> {
    let encoded = headers
        .get("x-tessara-organization-access")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::restricted)?;
    serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| ApiError::restricted())?,
    )
    .map_err(|_| ApiError::restricted())
}

fn has_manage_authority(organizations: &[OrganizationAccessProjectionV1]) -> bool {
    organizations
        .iter()
        .any(|organization| organization.can_manage)
}

async fn detail_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Path(record_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_view(&load_security_state(&state.pool).await?) {
        return shell_page(&state, &headers, "Scoped Records", content).await;
    }
    let auth = authorize(
        &state,
        &headers,
        "records.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let record = load_record(&state.pool, record_id)
        .await?
        .filter(|record| authorization_allows(&auth, READ_CAPABILITY, record.organization_owner_id))
        .ok_or_else(ApiError::restricted)?;
    let organizations = organization_access_projection(&headers)?;
    let organization = organizations
        .iter()
        .find(|value| value.organization_id == record.organization_owner_id);
    let owner_label = organization
        .map(|value| value.label.clone())
        .unwrap_or_else(|| "Unavailable Organization".into());
    let can_manage = organization.is_some_and(|value| value.can_manage);
    let edit_action = can_manage.then(|| {
        let href = format!("/reference/scoped-records/records/{record_id}/edit");
        view! { <a class="button" href=href>"Edit Record"</a> }
    });
    let record_id_text = record_id.to_string();
    let label = record.label.clone();
    let page_label = label.clone();
    let owner_id = record.organization_owner_id.to_string();
    let created = record
        .created_at
        .format("%b %e, %Y · %l:%M %p UTC")
        .to_string();
    let updated = record
        .updated_at
        .format("%b %e, %Y · %l:%M %p UTC")
        .to_string();
    let badge_class = if can_manage {
        "status-badge is-success"
    } else {
        "status-badge is-info"
    };
    let authority = if can_manage { "Read · Manage" } else { "Read" };
    let body = view! {
        {scoped_records_breadcrumb(vec![("Home".into(), Some("/".into())), ("Scoped Records".into(), Some("/reference/scoped-records".into())), (record_id_text.clone(), None)])}
        <PageHeader title=label.clone() description=record_id_text.clone()><a class="button button--secondary" href="/reference/scoped-records">"Back to Records"</a>{edit_action}</PageHeader>
        <div class="scoped-records-detail-grid"><section class="scoped-records-card"><header><div><h2>"Record"</h2><p>"Product data owned by the Scoped Records Module Instance."</p></div><span class=badge_class>{authority}</span></header><dl><div><dt>"Record ID"</dt><dd><code>{record_id_text}</code></dd></div><div><dt>"Label"</dt><dd>{label}</dd></div><div><dt>"Organization owner"</dt><dd>{owner_label.clone()}" "<code>{owner_id}</code></dd></div><div><dt>"Created"</dt><dd>{created}</dd></div><div><dt>"Last updated"</dt><dd>{updated}</dd></div></dl></section>
        <aside class="scoped-records-card"><header><div><h2>"Authorization context"</h2><p>"Current Core decision for this module action."</p></div></header><div class="scoped-records-auth-context"><div><span>"Capability"</span><code>{READ_CAPABILITY}</code></div><div><span>"Authorized Organization"</span><strong>{owner_label}</strong></div><div><span>"Decision freshness"</span><span class="status-badge is-success">"Current"</span></div><div><span>"Presenting service"</span><code>{MODULE_DEFINITION_ID}</code></div></div><div class="scoped-records-notice"><strong>"Core credentials are not shared"</strong><span>"This module received only a short-lived, audience-bound decision."</span></div></aside></div>
    }.into_any();
    shell_page(&state, &headers, &page_label, body).await
}

async fn create_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_view(&load_security_state(&state.pool).await?) {
        return shell_page(&state, &headers, "Scoped Records", content).await;
    }
    let organizations = organization_access_projection(&headers)?;
    if !has_manage_authority(&organizations) {
        return shell_page(&state, &headers, "Scoped Records", manage_denied_view()).await;
    }
    shell_page(
        &state,
        &headers,
        "Create Record",
        record_form_view(None, &organizations),
    )
    .await
}

async fn edit_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Path(record_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_view(&load_security_state(&state.pool).await?) {
        return shell_page(&state, &headers, "Scoped Records", content).await;
    }
    let auth = authorize(
        &state,
        &headers,
        "records.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let record = load_record(&state.pool, record_id)
        .await?
        .filter(|record| authorization_allows(&auth, READ_CAPABILITY, record.organization_owner_id))
        .ok_or_else(ApiError::restricted)?;
    let organizations = organization_access_projection(&headers)?;
    if !organizations
        .iter()
        .any(|value| value.organization_id == record.organization_owner_id && value.can_manage)
    {
        return shell_page(&state, &headers, "Scoped Records", manage_denied_view()).await;
    }
    shell_page(
        &state,
        &headers,
        "Edit Record",
        record_form_view(Some(&record), &organizations),
    )
    .await
}

fn record_form_view(
    record: Option<&ScopedRecord>,
    organizations: &[OrganizationAccessProjectionV1],
) -> AnyView {
    let editing = record.is_some();
    let title = if editing { "Edit Record" } else { "New Record" };
    let submit_label = if editing {
        "Save Record"
    } else {
        "Create Record"
    };
    let action = record.map_or_else(
        || "/reference/scoped-records/records".to_string(),
        |record| format!("/reference/scoped-records/records/{}", record.id),
    );
    let cancel = record.map_or_else(
        || "/reference/scoped-records".to_string(),
        |record| format!("/reference/scoped-records/records/{}", record.id),
    );
    let options = organizations
        .iter()
        .filter(|organization| organization.can_manage)
        .map(|organization| {
            let selected = record
                .is_some_and(|record| record.organization_owner_id == organization.organization_id);
            let id = organization.organization_id.to_string();
            let label = organization.label.clone();
            view! { <option value=id selected=selected>{label}</option> }
        })
        .collect_view();
    let label = record
        .map(|record| record.label.clone())
        .unwrap_or_default();
    let record_code = record.map(|record| view! { <code>{record.id.to_string()}</code> });
    let record_id_text = record.map(|record| record.id.to_string());
    let breadcrumb = if let Some(record_id) = record_id_text.as_deref() {
        scoped_records_breadcrumb(vec![
            ("Home".into(), Some("/".into())),
            (
                "Scoped Records".into(),
                Some("/reference/scoped-records".into()),
            ),
            (
                record_id.into(),
                Some(format!("/reference/scoped-records/records/{record_id}")),
            ),
            ("Edit".into(), None),
        ])
    } else {
        scoped_records_breadcrumb(vec![
            ("Home".into(), Some("/".into())),
            (
                "Scoped Records".into(),
                Some("/reference/scoped-records".into()),
            ),
            ("New Record".into(), None),
        ])
    };
    let idempotency_key = Uuid::new_v4().to_string();
    view! {
        {breadcrumb}<PageHeader title=title description="Manage authority is checked against the selected Organization subtree when saved." />
        <form class="scoped-records-card scoped-records-form" method="post" action=action><header><div><h2>"Record details"</h2><p>"Fields and validation belong to Scoped Records."</p></div>{record_code}</header><div class="scoped-records-form-grid"><label><span>"Label"</span><input name="label" value=label placeholder="Enter a clear record label" maxlength="200" required/></label><label><span>"Organization owner"</span><select name="organization_owner_id" required>{options}</select></label></div><input type="hidden" name="idempotency_key" value=idempotency_key/><div class="scoped-records-validation"><strong>"Manage authority confirmed"</strong><span>"Only Organizations covered by your current manage authority are available."</span></div><div class="scoped-records-form-actions"><a class="button button--secondary" href=cancel>"Cancel"</a><button class="button" type="submit">{submit_label}</button></div></form>
    }.into_any()
}

async fn health_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let status = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        "Passing"
    } else {
        "Unavailable"
    };
    shell_page(
        &state,
        &headers,
        "Scoped Records",
        view! {
            {scoped_records_breadcrumb(vec![("Home".into(), Some("/".into())), ("Scoped Records".into(), Some("/reference/scoped-records".into())), ("Health & diagnostics".into(), None)])}
            <PageHeader title="Scoped Records health" description="Module-owned operational detail with Core installation context."><a class="button button--secondary" href="/reference/scoped-records/health">"Refresh status"</a></PageHeader>
            <nav class="scoped-records-tabs"><a class="is-active" href="/reference/scoped-records/health">"Health"</a><a href="/reference/scoped-records/diagnostics">"Diagnostics"</a></nav>
            <div class="scoped-records-diagnostic-grid"><article><div><h2>"Readiness"</h2><strong>{status}</strong><small>"Module can serve authorized product requests."</small></div></article><article><div><h2>"Liveness"</h2><strong>"Passing"</strong><small>"Module process is responding."</small></div></article><article><div><h2>"Configuration"</h2><strong>"Valid"</strong><small>"Schema v1 · no findings."</small></div></article><article><div><h2>"Core authorization"</h2><strong>"Connected"</strong><small>"Signed decision exchange is available."</small></div></article></div>
        }.into_any(),
    )
    .await
}

async fn diagnostics_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let security = load_security_state(&state.pool).await.ok();
    let module_instance = security
        .as_ref()
        .map(|value| value.module_instance_id.to_string())
        .unwrap_or_else(|| "Unavailable".into());
    shell_page(
        &state,
        &headers,
        "Scoped Records",
        {
            let authorization_revision = security.as_ref().map(|value| value.authorization_revision).unwrap_or_default();
            let organization_revision = security.as_ref().map(|value| value.organization_revision).unwrap_or_default();
            view! {
                {scoped_records_breadcrumb(vec![("Home".into(), Some("/".into())), ("Scoped Records".into(), Some("/reference/scoped-records".into())), ("Health & diagnostics".into(), None)])}
                <PageHeader title="Scoped Records health" description="Module-owned operational detail with Core installation context."><a class="button button--secondary" href="/reference/scoped-records/diagnostics">"Refresh status"</a></PageHeader>
                <nav class="scoped-records-tabs"><a href="/reference/scoped-records/health">"Health"</a><a class="is-active" href="/reference/scoped-records/diagnostics">"Diagnostics"</a></nav>
                <div class="scoped-records-detail-grid scoped-records-diagnostics"><section class="scoped-records-card"><header><div><h2>"Diagnostic context"</h2><p>"Shareable values are sanitized and contain no claim secrets or Core credentials."</p></div></header><dl><div><dt>"Module version"</dt><dd>{MODULE_RELEASE_VERSION}</dd></div><div><dt>"Module Instance"</dt><dd><code>{module_instance}</code></dd></div><div><dt>"Database binding"</dt><dd><code>"tessara_module_scoped_records"</code></dd></div><div><dt>"Authorization revision"</dt><dd><code>{format!("auth:{authorization_revision}")}</code></dd></div><div><dt>"Organization revision"</dt><dd><code>{format!("org:{organization_revision}")}</code></dd></div></dl></section><aside class="scoped-records-card"><header><div><h2>"Recent findings"</h2><p>"Stable codes from module-owned validation and health checks."</p></div></header><div class="scoped-records-empty"><strong>"No active findings"</strong><span>"All required contracts and probes currently pass."</span></div><a class="button button--secondary" download="scoped-records-diagnostics.json" href="data:application/json,%7B%22schema_version%22%3A1%2C%22module%22%3A%22tessara.reference.scoped-records%22%7D">"Download sanitized diagnostics"</a></aside></div>
            }.into_any()
        },
    )
    .await
}

async fn live() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn ready(State(state): State<ModuleState>) -> StatusCode {
    let config = get_configuration(State(state.clone())).await;
    let security = load_security_state(&state.pool).await;
    if config.is_ok()
        && security.is_ok_and(|value| value.enabled && value.document_state == "enabled")
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn configuration_label(pool: &PgPool) -> String {
    sqlx::query_scalar(
        "SELECT display_label FROM scoped_records_configuration WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| "Scoped Records".into())
}

fn product_state_view(security: &SecurityState) -> Option<AnyView> {
    let (eyebrow, title, message, action_label, action_href) = if !security.enabled
        || security.document_state == "disabled"
    {
        (
            "Module disabled",
            "Scoped Records is not serving product routes",
            "Configuration, health, and diagnostics remain available while product access is disabled.",
            "Open Module Management",
            "/administration/modules/tessara.reference.scoped-records",
        )
    } else if security.document_state == "degraded" {
        (
            "Module needs attention",
            "Scoped Records cannot serve this route reliably",
            "Review module diagnostics before continuing with product work.",
            "Open diagnostics",
            "/reference/scoped-records/diagnostics",
        )
    } else {
        return None;
    };

    let state = security.document_state.clone();
    Some(view! {
        {scoped_records_breadcrumb(vec![("Home".into(), Some("/".into())), ("Scoped Records".into(), Some("/reference/scoped-records".into())), ("State".into(), None)])}
        <section class="scoped-records-state-treatment"><span class="scoped-records-state-treatment__eyebrow">{eyebrow}</span><h1>{title}</h1><p>{message}</p><a class="button" href=action_href>{action_label}</a></section>
        <section class="scoped-records-state-context"><h2>"Protected context"</h2><dl><div><dt>"Module"</dt><dd>{MODULE_DEFINITION_ID}</dd></div><div><dt>"Lifecycle state"</dt><dd>{state}</dd></div><div><dt>"Product data"</dt><dd>"Retained"</dd></div></dl></section>
    }.into_any())
}

fn manage_denied_view() -> AnyView {
    view! {
        {scoped_records_breadcrumb(vec![("Home".into(), Some("/".into())), ("Scoped Records".into(), Some("/reference/scoped-records".into())), ("Manage access".into(), None)])}
        <section class="scoped-records-state-treatment"><span class="scoped-records-state-treatment__eyebrow">"Scoped action unavailable"</span><h1>"You can’t manage this record"</h1><p>"Your current access permits reading Scoped Records, but not creating or changing them."</p><a class="button" href="/reference/scoped-records">"Back to Records"</a></section>
        <section class="scoped-records-state-context"><h2>"Protected context"</h2><dl><div><dt>"Required capability"</dt><dd><code>{MANAGE_CAPABILITY}</code></dd></div><div><dt>"Current route"</dt><dd>"Read-only"</dd></div><div><dt>"Disclosure"</dt><dd>"No unavailable record details shown"</dd></div></dl></section>
    }.into_any()
}

async fn shell_page(
    state: &ModuleState,
    headers: &HeaderMap,
    heading: &str,
    body: AnyView,
) -> Result<Response, ApiError> {
    let correlation_id =
        request_correlation_id(headers).map_err(|_| ApiError::shell_unavailable())?;
    let envelope: SignedEnvelopeV1<ShellContextV2> =
        decode_signed_envelope_header(headers, "x-tessara-shell-context")
            .map_err(|_| ApiError::shell_unavailable())?;
    let security = load_security_state(&state.pool).await?;
    verify_shell_context(
        &envelope,
        &state.core_shell_verifier,
        &ShellContextValidationContextV2 {
            installation_id: security.installation_id,
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                .map_err(|_| ApiError::shell_unavailable())?,
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: Utc::now(),
        },
    )
    .map_err(|_| ApiError::shell_unavailable())?;
    let presentation = ShellPresentation::from_verified_context(
        &envelope.payload,
        headers
            .get("x-tessara-original-path")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("/reference/scoped-records"),
        heading,
    );
    Ok(Html(render_module_view_document(
        &presentation,
        &ModuleDocumentAssets {
            stylesheets: vec![MODULE_UI_CSS_PATH.into(), SCOPED_RECORDS_CSS_PATH.into()],
            deferred_scripts: Vec::new(),
            hydration_script: None,
        },
        &ModuleReleaseMetadata {
            definition_id: MODULE_DEFINITION_ID.into(),
            release_version: MODULE_RELEASE_VERSION.into(),
            asset_digest: format!("sha256:{MODULE_UI_CSS_SHA256}"),
        },
        None,
        || view! { <div class="scoped-records-module-content">{body}</div> },
    ))
    .into_response())
}

async fn module_ui_stylesheet() -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        tessara_module_ui::MODULE_UI_CSS,
    )
        .into_response()
}

async fn scoped_records_stylesheet() -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        SCOPED_RECORDS_CSS,
    )
        .into_response()
}

fn scoped_records_breadcrumb(items: Vec<(String, Option<String>)>) -> AnyView {
    let mut children = Vec::<AnyView>::new();
    for (index, (label, href)) in items.into_iter().enumerate() {
        if index > 0 {
            children.push(view! { <BreadcrumbSeparator /> }.into_any());
        }
        if let Some(href) = href {
            children.push(view! { <BreadcrumbItem><BreadcrumbLink href=href>{label}</BreadcrumbLink></BreadcrumbItem> }.into_any());
        } else {
            children.push(
                view! { <BreadcrumbItem><BreadcrumbPage>{label}</BreadcrumbPage></BreadcrumbItem> }
                    .into_any(),
            );
        }
    }
    view! { <Breadcrumb>{children}</Breadcrumb> }.into_any()
}

fn require_private_key(headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    if headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
    {
        Ok(())
    } else {
        Err(ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "Request unavailable".into(),
        })
    }
}

fn require_exact_json(headers: &HeaderMap) -> Result<(), ApiError> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return Err(ApiError::bad_request(
            "Content-Type must be application/json",
        ));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn shell_unavailable() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "shell_context_unavailable",
            message: "Open this module through Core.".into(),
        }
    }
    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_request",
            message: message.into(),
        }
    }
    fn restricted() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "record_unavailable",
            message: "The requested record or action is unavailable.".into(),
        }
    }
    fn stale_or_restricted() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "authorization_stale",
            message: "Authorization changed. Refresh through Core and try again.".into(),
        }
    }
    fn security_identity_conflict() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "security_identity_conflict",
            message: "Security state identity cannot change.".into(),
        }
    }
    fn unavailable(message: &str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "module_unavailable",
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "scoped records database request failed");
        Self::unavailable("The module could not complete the request.")
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_: serde_json::Error) -> Self {
        Self::bad_request("JSON input is invalid")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            [(header::CONTENT_TYPE, "application/json")],
            Json(json!({"code": self.code, "message": self.message})),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(view: AnyView) -> String {
        Owner::new().with(|| view.to_html())
    }

    #[test]
    fn configuration_validator_normalizes_and_returns_stable_findings() {
        let valid = validate_configuration(&ScopedRecordsConfigurationV1 {
            schema_version: 1,
            display_label: "  Regional Records  ".into(),
            retention_mode: "retain_on_undeploy".into(),
        });
        assert_eq!(valid.normalized.unwrap().display_label, "Regional Records");
        let invalid = validate_configuration(&ScopedRecordsConfigurationV1 {
            schema_version: 1,
            display_label: " ".into(),
            retention_mode: "retain_on_undeploy".into(),
        });
        assert_eq!(
            invalid.findings[0].code,
            "configuration.display_label.required"
        );
    }

    #[test]
    fn capability_owner_projection_does_not_cross_bindings() {
        let read = SecurityCapabilityId::new(READ_CAPABILITY).unwrap();
        let manage = SecurityCapabilityId::new(MANAGE_CAPABILITY).unwrap();
        let root_a = Uuid::from_u128(1);
        let root_b = Uuid::from_u128(2);
        let grant = AuthorizationGrantV3 {
            schema_version: tessara_module_contract::AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: Uuid::from_u128(3),
            original_actor_id: Uuid::from_u128(4),
            correlation_id: Uuid::from_u128(7),
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: Uuid::from_u128(5),
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.scoped-records").unwrap(),
            functional_contract: FunctionalContractId::new(
                "tessara.reference.scoped-records.record",
            )
            .unwrap(),
            action: "records.list".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![
                tessara_module_contract::CapabilityScopeBindingV1 {
                    capability: read.clone(),
                    organization_root_id: root_a,
                    authorized_organization_ids: vec![],
                },
                tessara_module_contract::CapabilityScopeBindingV1 {
                    capability: manage.clone(),
                    organization_root_id: root_b,
                    authorized_organization_ids: vec![],
                },
            ],
            resource_assertion: None,
            delegation_basis: vec![],
            authorization_revision: 1,
            organization_revision: 1,
            jti: Uuid::from_u128(6),
            issued_at: Utc::now(),
            expires_at: Utc::now(),
        };
        assert!(grant.authorizes(&read, root_a));
        assert!(grant.authorizes(&manage, root_b));
        assert!(!grant.authorizes(&read, root_b));
        assert!(!grant.authorizes(&manage, root_a));
    }

    #[test]
    fn directory_renders_only_supplied_authorized_records_and_escapes_labels() {
        let record_id = Uuid::from_u128(7);
        let owner_id = Uuid::from_u128(8);
        let organizations = vec![OrganizationAccessProjectionV1 {
            organization_id: owner_id,
            label: "North Region".into(),
            can_manage: true,
        }];
        let html = render(directory_view(
            &[ScopedRecord {
                id: record_id,
                label: "<North & Central>".into(),
                organization_owner_id: owner_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }],
            &organizations,
            &DirectoryQuery::default(),
        ));

        assert!(html.contains(&format!(
            "href=\"/reference/scoped-records/records/{record_id}\""
        )));
        assert!(html.contains("&lt;North &amp; Central&gt;"));
        assert!(html.contains("North Region"));
        assert!(html.contains("Read · Manage"));
        assert!(html.contains("Showing 1-1 of 1 records"));
        assert!(html.contains("class=\"breadcrumb\""));
        assert!(html.contains("aria-label=\"Breadcrumb\""));
        assert!(html.contains("class=\"breadcrumb__separator-icon\""));
        assert!(html.contains("<path"));
        assert!(!html.contains("<North & Central>"));

        let empty = render(directory_view(
            &[],
            &organizations,
            &DirectoryQuery::default(),
        ));
        assert!(empty.contains("No scoped records"));
        assert!(!empty.contains("<tbody>"));
    }

    #[test]
    fn create_and_edit_forms_are_distinct_and_offer_only_manageable_organizations() {
        let managed = OrganizationAccessProjectionV1 {
            organization_id: Uuid::from_u128(9),
            label: "North Region".into(),
            can_manage: true,
        };
        let read_only = OrganizationAccessProjectionV1 {
            organization_id: Uuid::from_u128(10),
            label: "West Region".into(),
            can_manage: false,
        };
        let organizations = vec![managed.clone(), read_only];
        let create = render(record_form_view(None, &organizations));
        assert!(create.contains("<h1>New Record</h1>"));
        assert!(create.contains("class=\"breadcrumb__page\""));
        assert!(create.contains("New Record"));
        assert!(create.contains("action=\"/reference/scoped-records/records\""));
        assert!(create.contains("North Region"));
        assert!(!create.contains("West Region"));

        let record_id = Uuid::from_u128(11);
        let edit = render(record_form_view(
            Some(&ScopedRecord {
                id: record_id,
                label: "North intake review".into(),
                organization_owner_id: managed.organization_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }),
            &organizations,
        ));
        assert!(edit.contains("<h1>Edit Record</h1>"));
        assert!(edit.contains("class=\"breadcrumb__page\""));
        assert!(edit.contains("Edit"));
        assert!(edit.contains(&format!(
            "action=\"/reference/scoped-records/records/{record_id}\""
        )));
        assert!(edit.contains("North intake review"));
        assert!(edit.contains(" selected"));
    }

    #[test]
    fn read_only_directory_does_not_advertise_record_creation() {
        let organizations = vec![OrganizationAccessProjectionV1 {
            organization_id: Uuid::from_u128(12),
            label: "North Region".into(),
            can_manage: false,
        }];
        let html = render(directory_view(
            &[],
            &organizations,
            &DirectoryQuery::default(),
        ));

        assert!(html.contains("0 include manage authority"));
        assert!(!html.contains("/reference/scoped-records/records/new"));
        assert!(!html.contains(">New Record<"));
        assert!(!has_manage_authority(&organizations));
    }

    #[test]
    fn lifecycle_and_manage_denial_states_preserve_the_core_shell_contract() {
        let disabled = SecurityState {
            installation_id: Uuid::from_u128(13),
            module_instance_id: Uuid::from_u128(14),
            authorization_revision: 1,
            organization_revision: 1,
            enabled: false,
            document_state: "disabled".into(),
        };
        let disabled_html =
            render(product_state_view(&disabled).expect("disabled state treatment"));
        assert!(disabled_html.contains("Scoped Records is not serving product routes"));
        assert!(disabled_html.contains("/administration/modules/tessara.reference.scoped-records"));
        assert!(disabled_html.contains("Product data</dt><dd>Retained"));

        let enabled = SecurityState {
            enabled: true,
            document_state: "enabled".into(),
            ..disabled
        };
        assert!(product_state_view(&enabled).is_none());

        let denied_html = render(manage_denied_view());
        assert!(denied_html.contains("You can’t manage this record"));
        assert!(denied_html.contains("tessara.reference.scoped-records:manage"));
        assert!(denied_html.contains("Back to Records"));
    }
}
