//! Core-owned composition planning, approval, and read-back projection.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_composition::{
    AUTHORIZATION_API_V1, ActorEvidenceV1, ApplicationBlueprintV1, ApplicationLockfileV1,
    ApplyAuthorizationV1, ApplyOperationKindV1, ApprovedEffectV1,
    BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_SCHEMA_VERSION_V1,
    BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1,
    BootstrapDependencyValidationAuthorizationIssueRequestV1,
    BootstrapDependencyValidationAuthorizationIssueResponseV1,
    BootstrapDependencyValidationAuthorizationV1, BootstrapDependencyValidationInvocationV1,
    BootstrapDependencyValidationRequestV1, CompositionError, CompositionOperationV1,
    InstallationReceiptV1, MaterializationActionV1,
    OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1, OwnerBootstrapAuthorizationV1,
    OwnerBootstrapProviderActionV1, PLAN_API_V1, ReleaseCatalogV1, canonical_digest,
    required_effects, resolve_against, resolve_bootstrap_dependency_validation,
};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationAudienceV1, ModuleManifest, ProtocolSignaturePurposeV1,
    PurposeBoundSigningKeyV1,
};
use uuid::Uuid;

use crate::{
    auth::{self, AuthenticatedRequest},
    db::AppState,
    error::{ApiError, ApiResult},
};

const COMPOSITION_HTTP_SCHEMA_VERSION_V1: u16 = 1;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/composition", get(summary))
        .route("/api/admin/composition/blueprints", post(create_blueprint))
        .route(
            "/api/admin/composition/blueprints/{revision}/resolve",
            post(resolve_blueprint),
        )
        .route(
            "/api/admin/composition/blueprints/{revision}/approve",
            post(approve_blueprint),
        )
        .route(
            "/api/admin/composition/blueprints/{revision}/apply",
            post(apply_blueprint),
        )
        .route(
            "/api/admin/composition/operations/{operation_id}",
            get(operation),
        )
        .route(
            "/api/internal/composition/operations",
            post(project_operation),
        )
        .route("/api/internal/composition/receipts", post(project_receipt))
        .route(
            "/api/internal/composition/bootstrap-capabilities",
            post(enroll_bootstrap_capabilities),
        )
        .route(
            "/api/internal/composition/bootstrap/core",
            post(apply_core_bootstrap),
        )
        .route(
            "/api/internal/composition/bootstrap/dependency-authorization",
            post(issue_bootstrap_dependency_authorization),
        )
        .route(
            "/api/admin/composition/drift/{finding_id}/adopt",
            post(adopt_drift),
        )
        .route(
            "/api/admin/composition/drift/{finding_id}/reconcile",
            post(reconcile_drift),
        )
        .route(
            "/api/admin/composition/modules/{definition_id}/emergency-disable",
            post(emergency_disable),
        )
}

#[derive(Debug, Serialize)]
struct SummaryResponseV1 {
    schema_version: u16,
    installation_id: Uuid,
    latest_blueprint: Option<Value>,
    latest_lockfile: Option<Value>,
    latest_approval: Option<ApprovalProjectionV1>,
    active_operation: Option<Value>,
    latest_receipt: Option<Value>,
    drift_findings: Vec<DriftProjectionV1>,
    emergency_overrides: Vec<Value>,
}

#[derive(Debug, Serialize)]
struct ApprovalProjectionV1 {
    blueprint_revision: i64,
    lockfile_digest: String,
    plan_digest: String,
    approved_effects: Value,
    reason: Option<String>,
    approved_by: Uuid,
    approved_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
struct DriftProjectionV1 {
    finding_id: Uuid,
    code: String,
    path: String,
    desired: Option<Value>,
    observed: Option<Value>,
    disposition: String,
    recorded_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ResolveRequestV1 {
    catalog: ReleaseCatalogV1,
}

#[derive(Debug, Serialize)]
struct ResolveResponseV1 {
    lockfile_digest: String,
    plan_digest: String,
    lockfile: ApplicationLockfileV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ApproveRequestV1 {
    approved_effects: BTreeSet<ApprovedEffectV1>,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmergencyDisableRequestV1 {
    reason: String,
    #[serde(default = "default_emergency_minutes")]
    expires_in_minutes: u32,
}

fn default_emergency_minutes() -> u32 {
    60
}

#[derive(Debug, Serialize)]
struct ApprovalResponseV1 {
    blueprint_revision: u64,
    lockfile_digest: String,
    plan_digest: String,
    approved_effects: BTreeSet<ApprovedEffectV1>,
    approved_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OperationProjectionRequestV1 {
    blueprint_revision: u64,
    operation: CompositionOperationV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReceiptProjectionRequestV1 {
    lockfile: ApplicationLockfileV1,
    receipt: InstallationReceiptV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BootstrapCapabilityEnrollmentRequestV1 {
    lockfile: ApplicationLockfileV1,
    manifests: BTreeMap<String, ModuleManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapV1 {
    schema_version: String,
    root_node_type_external_key: String,
    root_node_type_name: String,
    root_node_external_key: String,
    root_node_name: String,
    #[serde(default)]
    additional_nodes: Vec<CoreBootstrapNodeV1>,
    #[serde(default)]
    forms: Vec<CoreBootstrapFormV1>,
    #[serde(default)]
    responses: Vec<CoreBootstrapResponseV1>,
    #[serde(default)]
    actors: Vec<CoreBootstrapActorV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapNodeV1 {
    external_key: String,
    name: String,
    parent_node_key: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapFormV1 {
    resource_key: String,
    source_alias: String,
    name: String,
    slug: String,
    version_label: String,
    published_at: DateTime<Utc>,
    scope_node_keys: Vec<String>,
    fields: Vec<CoreBootstrapFormFieldV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapFormFieldV1 {
    key: String,
    label: String,
    field_type: String,
    #[serde(default)]
    required: bool,
    position: i32,
    grid_row: i32,
    grid_column: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapResponseV1 {
    resource_key: String,
    form_resource_key: String,
    node_key: String,
    created_at: DateTime<Utc>,
    submitted_at: DateTime<Utc>,
    values: BTreeMap<String, Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapActorV1 {
    resource_key: String,
    email: String,
    display_name: String,
    password: String,
    capabilities: Vec<String>,
    scope_node_keys: Vec<String>,
}

async fn summary(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
) -> ApiResult<Json<SummaryResponseV1>> {
    require_global(&auth, "composition:read")?;
    let installation_id = installation_id(&state).await?;
    let latest_blueprint = sqlx::query_scalar(
        "SELECT document FROM composition_blueprints WHERE installation_id=$1 ORDER BY revision DESC LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?;
    let latest_lockfile: Option<Value> = sqlx::query_scalar(
        "SELECT document FROM composition_lockfiles WHERE installation_id=$1 ORDER BY blueprint_revision DESC LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?;
    let latest_approval = sqlx::query(
        "SELECT blueprint_revision,lockfile_digest,plan_digest,approved_effects,reason,approved_by,approved_at FROM composition_approvals WHERE installation_id=$1 ORDER BY blueprint_revision DESC LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?
    .map(|row| ApprovalProjectionV1 {
        blueprint_revision: row.get("blueprint_revision"),
        lockfile_digest: row.get("lockfile_digest"),
        plan_digest: row.get("plan_digest"),
        approved_effects: row.get("approved_effects"),
        reason: row.get("reason"),
        approved_by: row.get("approved_by"),
        approved_at: row.get("approved_at"),
    });
    let active_operation = sqlx::query_scalar(
        "SELECT operation FROM composition_operation_projections WHERE installation_id=$1 AND state NOT IN ('succeeded','failed','rolled_back') ORDER BY updated_at DESC LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?;
    let mut latest_receipt: Option<Value> = sqlx::query_scalar(
        "SELECT receipt FROM composition_receipt_projections WHERE installation_id=$1 ORDER BY revision DESC LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?;
    if latest_receipt.is_none()
        && let Ok(supervisor_url) = std::env::var("TESSARA_SUPERVISOR_URL")
        && let Ok(response) = reqwest::Client::new()
            .get(format!(
                "{}/v1/receipts/current",
                supervisor_url.trim_end_matches('/')
            ))
            .send()
            .await
        && response.status().is_success()
    {
        latest_receipt = response.json::<Value>().await.ok();
    }
    if let Some(document) = &latest_lockfile {
        detect_composition_drift(
            &state,
            installation_id,
            document,
            latest_blueprint.as_ref(),
            latest_receipt.as_ref(),
        )
        .await?;
    }
    let drift_findings = sqlx::query(
        "SELECT finding_id,code,path,desired,observed,disposition,recorded_at FROM composition_drift_findings WHERE installation_id=$1 AND disposition='open' ORDER BY recorded_at,finding_id",
    )
    .bind(installation_id)
    .fetch_all(&state.pool)
    .await?
    .into_iter()
    .map(|row| DriftProjectionV1 {
        finding_id: row.get("finding_id"),
        code: row.get("code"),
        path: row.get("path"),
        desired: row.get("desired"),
        observed: row.get("observed"),
        disposition: row.get("disposition"),
        recorded_at: row.get("recorded_at"),
    })
    .collect();
    let emergency_overrides = if let Ok(supervisor_url) = std::env::var("TESSARA_SUPERVISOR_URL") {
        match reqwest::Client::new()
            .get(format!(
                "{}/v1/emergency-overrides",
                supervisor_url.trim_end_matches('/')
            ))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => {
                response.json::<Vec<Value>>().await.unwrap_or_default()
            }
            _ => Vec::new(),
        }
    } else {
        Vec::new()
    };
    Ok(Json(SummaryResponseV1 {
        schema_version: COMPOSITION_HTTP_SCHEMA_VERSION_V1,
        installation_id,
        latest_blueprint,
        latest_lockfile,
        latest_approval,
        active_operation,
        latest_receipt,
        drift_findings,
        emergency_overrides,
    }))
}

async fn create_blueprint(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Json(blueprint): Json<ApplicationBlueprintV1>,
) -> ApiResult<(StatusCode, Json<ApplicationBlueprintV1>)> {
    require_global(&auth, "composition:plan")?;
    let installation_id = installation_id(&state).await?;
    if blueprint.installation_id != installation_id {
        return Err(ApiError::BadRequest(
            "Blueprint installation_id does not match this installation".into(),
        ));
    }
    let next_revision: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(revision),0)+1 FROM composition_blueprints WHERE installation_id=$1",
    )
    .bind(installation_id)
    .fetch_one(&state.pool)
    .await?;
    if blueprint.revision != next_revision as u64 {
        return Err(ApiError::BadRequest(format!(
            "Blueprint revision must be the next revision ({next_revision})"
        )));
    }
    let digest = canonical_digest(&blueprint)
        .map_err(|error| ApiError::Internal(error.into()))?
        .to_string();
    sqlx::query("INSERT INTO composition_blueprints(installation_id,revision,digest,document,state,created_by) VALUES($1,$2,$3,$4,'draft',$5)")
        .bind(installation_id)
        .bind(next_revision)
        .bind(digest)
        .bind(serde_json::to_value(&blueprint).map_err(|error| ApiError::Internal(error.into()))?)
        .bind(auth.account_id)
        .execute(&state.pool)
        .await?;
    Ok((StatusCode::CREATED, Json(blueprint)))
}

async fn current_applied_lockfile(
    state: &AppState,
    installation_id: Uuid,
) -> ApiResult<Option<ApplicationLockfileV1>> {
    let projected = sqlx::query(
        "SELECT lockfile,receipt
         FROM composition_receipt_projections
         WHERE installation_id=$1
         ORDER BY revision DESC
         LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(projected) = projected else {
        return Ok(None);
    };
    let mut lockfile: ApplicationLockfileV1 =
        serde_json::from_value(projected.try_get("lockfile")?)
            .map_err(|error| ApiError::Internal(error.into()))?;
    let receipt: InstallationReceiptV1 = serde_json::from_value(projected.try_get("receipt")?)
        .map_err(|error| ApiError::Internal(error.into()))?;
    if lockfile.installation_id != installation_id || receipt.installation_id != installation_id {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "current composition projection belongs to another installation"
        )));
    }
    // Emergency enablement is an observed override rather than a mutation of
    // the desired Module selection. Feed the observed state into delta
    // planning so the next ordinary revision explicitly reconciles it instead
    // of silently leaving the owner disabled.
    for module in &mut lockfile.modules {
        if let Some(enabled) = receipt.observed_enablement.get(&module.definition_id) {
            module.enabled = *enabled;
        }
    }
    Ok(Some(lockfile))
}

async fn resolve_blueprint(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(revision): Path<u64>,
    Json(request): Json<ResolveRequestV1>,
) -> ApiResult<Json<ResolveResponseV1>> {
    require_global(&auth, "composition:plan")?;
    let installation_id = installation_id(&state).await?;
    let document: Value = sqlx::query_scalar(
        "SELECT document FROM composition_blueprints WHERE installation_id=$1 AND revision=$2",
    )
    .bind(installation_id)
    .bind(revision as i64)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("Blueprint revision was not found".into()))?;
    let blueprint: ApplicationBlueprintV1 =
        serde_json::from_value(document).map_err(|error| ApiError::Internal(error.into()))?;
    let current = current_applied_lockfile(&state, installation_id).await?;
    let lockfile =
        resolve_against(&blueprint, &request.catalog, current.as_ref()).map_err(findings_error)?;
    let lockfile_digest = canonical_digest(&lockfile)
        .map_err(|error| ApiError::Internal(error.into()))?
        .to_string();
    let plan_digest = lockfile.materialization_plan_digest.to_string();
    let catalog_digest = lockfile.catalog_digest.to_string();
    let lockfile_value =
        serde_json::to_value(&lockfile).map_err(|error| ApiError::Internal(error.into()))?;
    let mut transaction = state.pool.begin().await?;
    sqlx::query("INSERT INTO composition_lockfiles(installation_id,blueprint_revision,lockfile_digest,plan_digest,catalog_digest,document,resolved_by) VALUES($1,$2,$3,$4,$5,$6,$7) ON CONFLICT(installation_id,blueprint_revision) DO UPDATE SET lockfile_digest=EXCLUDED.lockfile_digest,plan_digest=EXCLUDED.plan_digest,catalog_digest=EXCLUDED.catalog_digest,document=EXCLUDED.document,resolved_by=EXCLUDED.resolved_by,resolved_at=now()")
        .bind(installation_id).bind(revision as i64).bind(&lockfile_digest).bind(&plan_digest)
        .bind(catalog_digest).bind(lockfile_value).bind(auth.account_id).execute(&mut *transaction).await?;
    sqlx::query("UPDATE composition_blueprints SET state='resolved' WHERE installation_id=$1 AND revision=$2")
        .bind(installation_id).bind(revision as i64).execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(ResolveResponseV1 {
        lockfile_digest,
        plan_digest,
        lockfile,
    }))
}

async fn approve_blueprint(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(revision): Path<u64>,
    Json(request): Json<ApproveRequestV1>,
) -> ApiResult<Json<ApprovalResponseV1>> {
    require_global(&auth, "composition:approve")?;
    if request
        .approved_effects
        .contains(&ApprovedEffectV1::DestroyData)
    {
        return Err(ApiError::BadRequest(
            "Destructive data removal is outside the v1 composition contract".into(),
        ));
    }
    let installation_id = installation_id(&state).await?;
    let row = sqlx::query("SELECT lockfile_digest,plan_digest,document FROM composition_lockfiles WHERE installation_id=$1 AND blueprint_revision=$2")
        .bind(installation_id).bind(revision as i64).fetch_optional(&state.pool).await?
        .ok_or_else(|| ApiError::NotFound("Resolved Blueprint revision was not found".into()))?;
    let lockfile_digest: String = row.get("lockfile_digest");
    let plan_digest: String = row.get("plan_digest");
    let lockfile: ApplicationLockfileV1 = serde_json::from_value(row.get("document"))
        .map_err(|error| ApiError::Internal(error.into()))?;
    if request.approved_effects != required_effects(&lockfile.materialization_plan) {
        return Err(ApiError::BadRequest(
            "Approved effects must exactly match the current materialization plan".into(),
        ));
    }
    let approved_at = Utc::now();
    let effects = serde_json::to_value(&request.approved_effects)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let mut transaction = state.pool.begin().await?;
    sqlx::query("INSERT INTO composition_approvals(installation_id,blueprint_revision,lockfile_digest,plan_digest,approved_effects,reason,approved_by,approved_at) VALUES($1,$2,$3,$4,$5,$6,$7,$8) ON CONFLICT(installation_id,blueprint_revision) DO UPDATE SET lockfile_digest=EXCLUDED.lockfile_digest,plan_digest=EXCLUDED.plan_digest,approved_effects=EXCLUDED.approved_effects,reason=EXCLUDED.reason,approved_by=EXCLUDED.approved_by,approved_at=EXCLUDED.approved_at")
        .bind(installation_id).bind(revision as i64).bind(&lockfile_digest).bind(&plan_digest)
        .bind(effects).bind(&request.reason).bind(auth.account_id).bind(approved_at)
        .execute(&mut *transaction).await?;
    sqlx::query("UPDATE composition_blueprints SET state='approved' WHERE installation_id=$1 AND revision=$2")
        .bind(installation_id).bind(revision as i64).execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(ApprovalResponseV1 {
        blueprint_revision: revision,
        lockfile_digest,
        plan_digest,
        approved_effects: request.approved_effects,
        approved_at,
    }))
}

async fn apply_blueprint(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(revision): Path<i64>,
) -> ApiResult<Json<Value>> {
    require_global(&auth, "composition:approve")?;
    let installation_id = installation_id(&state).await?;
    let row = sqlx::query(
        "SELECT lockfile.document, approval.plan_digest, approval.approved_effects, approval.approved_by
         FROM composition_lockfiles lockfile
         JOIN composition_approvals approval
           ON approval.installation_id=lockfile.installation_id
          AND approval.blueprint_revision=lockfile.blueprint_revision
         WHERE lockfile.installation_id=$1 AND lockfile.blueprint_revision=$2",
    )
    .bind(installation_id)
    .bind(revision)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::BadRequest("Resolve and explicitly approve this Blueprint first".into()))?;
    let lockfile: ApplicationLockfileV1 = serde_json::from_value(row.try_get("document")?)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let approved_plan_digest: String = row.try_get("plan_digest")?;
    if approved_plan_digest != lockfile.materialization_plan_digest.to_string() {
        return Err(ApiError::BadRequest(
            "The approval does not bind the current materialization plan".into(),
        ));
    }
    let approved_effects: BTreeSet<ApprovedEffectV1> =
        serde_json::from_value(row.try_get("approved_effects")?)
            .map_err(|error| ApiError::Internal(error.into()))?;
    if approved_effects != required_effects(&lockfile.materialization_plan) {
        return Err(ApiError::BadRequest(
            "The approval effects do not exactly match the current plan".into(),
        ));
    }
    let supervisor_url = std::env::var("TESSARA_SUPERVISOR_URL")
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Supervisor URL is not configured")))?;
    let client = reqwest::Client::new();
    let current_response = client
        .get(format!(
            "{}/v1/receipts/current",
            supervisor_url.trim_end_matches('/')
        ))
        .send()
        .await
        .map_err(|_| ApiError::NotFound("Supervisor is unavailable".into()))?;
    let current_receipt = if current_response.status() == reqwest::StatusCode::NOT_FOUND {
        None
    } else {
        Some(
            current_response
                .error_for_status()
                .map_err(|_| ApiError::NotFound("Supervisor read-back is unavailable".into()))?
                .json::<InstallationReceiptV1>()
                .await
                .map_err(|error| ApiError::Internal(error.into()))?,
        )
    };
    let now = Utc::now();
    let actor_id = auth.account_id;
    let approved_by: Uuid = row.try_get("approved_by")?;
    let authorization = ApplyAuthorizationV1 {
        api_version: AUTHORIZATION_API_V1.into(),
        operation: ApplyOperationKindV1::Materialize,
        installation_id,
        base_receipt_digest: current_receipt
            .as_ref()
            .map(canonical_digest)
            .transpose()
            .map_err(|error| ApiError::Internal(error.into()))?,
        target_plan_digest: lockfile.materialization_plan_digest.clone(),
        desired_revision: lockfile.blueprint_revision,
        apply_sequence: current_receipt
            .as_ref()
            .map_or(1, |receipt| receipt.revision + 1),
        nonce: Uuid::new_v4(),
        idempotency_key: format!(
            "core-ui-r{}-a{}-{}",
            lockfile.blueprint_revision,
            current_receipt
                .as_ref()
                .map_or(1, |receipt| receipt.revision + 1),
            Uuid::new_v4()
        ),
        initiator: ActorEvidenceV1 {
            actor_id: actor_id.to_string(),
            actor_kind: "account".into(),
            authority: "composition:approve".into(),
        },
        approver: ActorEvidenceV1 {
            actor_id: approved_by.to_string(),
            actor_kind: "account".into(),
            authority: "composition:approve".into(),
        },
        issued_at: now,
        expires_at: now + Duration::minutes(10),
        approved_effects,
        reason: Some("Approved through Application Composition".into()),
    };
    let signer = apply_authorization_signer()?;
    let signed = signer
        .sign(authorization)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let response = client
        .post(format!("{}/v1/apply", supervisor_url.trim_end_matches('/')))
        .json(&serde_json::json!({"lockfile": lockfile, "authorization": signed}))
        .send()
        .await
        .map_err(|_| ApiError::NotFound("Supervisor is unavailable".into()))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    if !status.is_success() {
        return Err(ApiError::BadRequest(
            body.get("message")
                .and_then(Value::as_str)
                .unwrap_or("Supervisor rejected the apply request")
                .to_string(),
        ));
    }
    Ok(Json(body))
}

fn decode_secret_hex(name: &str) -> ApiResult<[u8; 32]> {
    let value = std::env::var(name)
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("{name} is not configured")))?;
    if value.len() != 64 {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "{name} must contain 32 hexadecimal bytes"
        )));
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)
            .map_err(|_| ApiError::Internal(anyhow::anyhow!("{name} is not hexadecimal")))?;
    }
    Ok(bytes)
}

fn apply_authorization_signer() -> ApiResult<PurposeBoundSigningKeyV1> {
    PurposeBoundSigningKeyV1::from_secret_bytes(
        std::env::var("TESSARA_COMPOSITION_APPLY_SIGNING_ISSUER")
            .unwrap_or_else(|_| "tessara.local.sprint-6f".into()),
        std::env::var("TESSARA_COMPOSITION_APPLY_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "apply-dev-v1".into()),
        ProtocolSignaturePurposeV1::ApplyAuthorization,
        decode_secret_hex("TESSARA_COMPOSITION_APPLY_SIGNING_SECRET_HEX")?,
    )
    .map_err(|error| ApiError::Internal(error.into()))
}

async fn operation(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(operation_id): Path<Uuid>,
) -> ApiResult<Json<Value>> {
    require_global(&auth, "composition:read")?;
    let operation = sqlx::query_scalar(
        "SELECT operation FROM composition_operation_projections WHERE operation_id=$1",
    )
    .bind(operation_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::NotFound("Composition operation was not found".into()))?;
    Ok(Json(operation))
}

async fn project_operation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<OperationProjectionRequestV1>,
) -> ApiResult<StatusCode> {
    require_projection_token(&headers)?;
    let value = serde_json::to_value(&request.operation)
        .map_err(|error| ApiError::Internal(error.into()))?;
    sqlx::query("INSERT INTO composition_operation_projections(operation_id,installation_id,blueprint_revision,state,operation) VALUES($1,$2,$3,$4,$5) ON CONFLICT(operation_id) DO UPDATE SET state=EXCLUDED.state,operation=EXCLUDED.operation,updated_at=now()")
        .bind(request.operation.operation_id).bind(request.operation.installation_id)
        .bind(request.blueprint_revision as i64).bind(request.operation.state.as_str()).bind(value)
        .execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn project_receipt(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ReceiptProjectionRequestV1>,
) -> ApiResult<StatusCode> {
    require_projection_token(&headers)?;
    let projected_digest =
        canonical_digest(&request.lockfile).map_err(|error| ApiError::Internal(error.into()))?;
    if projected_digest != request.receipt.lockfile_digest {
        return Err(ApiError::BadRequest(
            "Projected receipt does not bind the supplied lockfile".into(),
        ));
    }
    let lockfile_value: Value = sqlx::query_scalar("SELECT document FROM composition_lockfiles WHERE installation_id=$1 AND blueprint_revision=$2")
        .bind(request.receipt.installation_id)
        .bind(request.lockfile.blueprint_revision as i64)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::BadRequest("Projected receipt does not match a resolved Core lockfile".into()))?;
    let resolved_lockfile: ApplicationLockfileV1 =
        serde_json::from_value(lockfile_value).map_err(|error| ApiError::Internal(error.into()))?;
    if request.lockfile != resolved_lockfile
        && !is_constrained_emergency_lockfile(
            &resolved_lockfile,
            &request.lockfile,
            &request.receipt,
        )
        .map_err(|error| ApiError::Internal(error.into()))?
    {
        return Err(ApiError::BadRequest(
            "Projected receipt lockfile is not the resolved plan or a constrained emergency disable"
                .into(),
        ));
    }
    let previous_lockfile: Option<ApplicationLockfileV1> = sqlx::query_scalar::<_, Value>(
        "SELECT lockfile FROM composition_receipt_projections
         WHERE installation_id=$1 ORDER BY revision DESC LIMIT 1",
    )
    .bind(request.receipt.installation_id)
    .fetch_optional(&state.pool)
    .await?
    .map(serde_json::from_value)
    .transpose()
    .map_err(|error| ApiError::Internal(error.into()))?;
    let lockfile = request.lockfile;
    let endpoints = module_control_endpoints()?;
    let client = reqwest::Client::new();
    let control_key = module_control_key()?;
    let mut manifests = BTreeMap::new();
    for module in &lockfile.modules {
        let endpoint = endpoints.get(&module.definition_id).ok_or_else(|| {
            ApiError::BadRequest(format!(
                "Composition module '{}' has no configured control endpoint",
                module.definition_id
            ))
        })?;
        let manifest = client
            .get(format!("{}/api/manifest", endpoint.trim_end_matches('/')))
            .header("x-tessara-module-control-key", &control_key)
            .send()
            .await
            .map_err(|error| ApiError::Internal(error.into()))?
            .error_for_status()
            .map_err(|error| ApiError::Internal(error.into()))?
            .json::<ModuleManifest>()
            .await
            .map_err(|error| ApiError::Internal(error.into()))?;
        let digest =
            canonical_digest(&manifest).map_err(|error| ApiError::Internal(error.into()))?;
        if manifest.definition_id.as_str() != module.definition_id
            || manifest.release_version != module.version
            || digest != module.manifest_digest
        {
            return Err(ApiError::BadRequest(format!(
                "Live manifest does not match resolved release '{}'",
                module.definition_id
            )));
        }
        manifests.insert(module.definition_id.clone(), manifest);
    }
    let receipt_digest = canonical_digest(&request.receipt)
        .map_err(|error| ApiError::Internal(error.into()))?
        .to_string();
    let receipt_value =
        serde_json::to_value(&request.receipt).map_err(|error| ApiError::Internal(error.into()))?;
    let lockfile_value =
        serde_json::to_value(&lockfile).map_err(|error| ApiError::Internal(error.into()))?;
    crate::modules::project_composition_modules(
        &state.pool,
        &lockfile,
        &request.receipt,
        &manifests,
        previous_lockfile.as_ref(),
        crate::modules::CompositionProjectionDocuments {
            digest: &receipt_digest,
            lockfile: &lockfile_value,
            receipt: &receipt_value,
        },
    )
    .await
    .map_err(ApiError::Internal)?;
    Ok(StatusCode::NO_CONTENT)
}

fn is_constrained_emergency_lockfile(
    resolved: &ApplicationLockfileV1,
    projected: &ApplicationLockfileV1,
    receipt: &InstallationReceiptV1,
) -> Result<bool, serde_json::Error> {
    let [
        MaterializationActionV1::SetEnablement {
            definition_id,
            enabled: false,
        },
        MaterializationActionV1::VerifyReadBack,
    ] = projected.materialization_plan.actions.as_slice()
    else {
        return Ok(false);
    };
    if !resolved
        .modules
        .iter()
        .any(|module| module.definition_id == *definition_id && module.enabled)
        || projected.materialization_plan.api_version != PLAN_API_V1
        || projected.materialization_plan.installation_id != resolved.installation_id
        || projected.materialization_plan.desired_revision != resolved.blueprint_revision
        || receipt.plan_digest != projected.materialization_plan_digest
        || receipt.desired_enablement.get(definition_id) != Some(&true)
        || receipt.observed_enablement.get(definition_id) != Some(&false)
    {
        return Ok(false);
    }
    let expected_plan_digest = canonical_digest(&projected.materialization_plan)?;
    if expected_plan_digest != projected.materialization_plan_digest {
        return Ok(false);
    }
    let mut expected = resolved.clone();
    expected.materialization_plan = projected.materialization_plan.clone();
    expected.materialization_plan_digest = expected_plan_digest;
    Ok(&expected == projected)
}

async fn issue_bootstrap_dependency_authorization(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BootstrapDependencyValidationAuthorizationIssueRequestV1>,
) -> ApiResult<Json<BootstrapDependencyValidationAuthorizationIssueResponseV1>> {
    require_projection_token(&headers)?;
    if apply_authorization_signer()?
        .verifier()
        .verify(&request.apply_authorization)
        .is_err()
    {
        return Err(reject_bootstrap_authorization("apply_signature"));
    }

    let lockfile_value: Value = sqlx::query_scalar(
        "SELECT document FROM composition_lockfiles
         WHERE installation_id=$1 AND blueprint_revision=$2",
    )
    .bind(request.installation_id)
    .bind(request.desired_revision as i64)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        ApiError::BadRequest("Bootstrap authorization lockfile is unavailable".into())
    })?;
    let lockfile: ApplicationLockfileV1 =
        serde_json::from_value(lockfile_value).map_err(|error| ApiError::Internal(error.into()))?;
    let current_receipt_digest = sqlx::query_scalar::<_, String>(
        "SELECT digest FROM composition_receipt_projections
         WHERE installation_id=$1 ORDER BY revision DESC LIMIT 1",
    )
    .bind(request.installation_id)
    .fetch_optional(&state.pool)
    .await?
    .map(ArtifactDigest::new)
    .transpose()
    .map_err(|error| ApiError::Internal(error.into()))?;
    let now = Utc::now();
    if let Err(finding) = request.apply_authorization.payload.validate_for(
        &lockfile.materialization_plan,
        &lockfile.materialization_plan_digest,
        current_receipt_digest.as_ref(),
        now,
    ) {
        tracing::warn!(
            operation = "bootstrap_dependency_authorization",
            result = "rejected",
            reason = "apply_contract",
            finding_code = %finding.code,
            "bootstrap dependency authorization rejected"
        );
        return Err(ApiError::Forbidden("bootstrap:authorize".into()));
    }
    if request.apply_authorization.payload.operation != ApplyOperationKindV1::Materialize
        || !request
            .apply_authorization
            .payload
            .approved_effects
            .contains(&ApprovedEffectV1::Bootstrap)
        || request.installation_id != lockfile.installation_id
        || request.desired_revision != lockfile.blueprint_revision
        || request.apply_sequence != request.apply_authorization.payload.apply_sequence
        || request.desired_revision != request.apply_authorization.payload.desired_revision
        || !lockfile.materialization_plan.actions.iter().any(|action| {
            matches!(action, MaterializationActionV1::Bootstrap { owner, input_digest }
                if owner == &request.owner_definition_id
                    && input_digest == &request.locked_input_digest)
        })
    {
        return Err(reject_bootstrap_authorization("plan_binding"));
    }

    let module = lockfile
        .modules
        .iter()
        .find(|module| module.definition_id == request.owner_definition_id && module.enabled);
    let module_instance_id = module.map(|_| {
        tessara_composition::module_instance_id(
            request.installation_id,
            &request.owner_definition_id,
        )
    });
    let manifest = if module.is_some() {
        let endpoint = module_control_endpoints()?
            .remove(&request.owner_definition_id)
            .ok_or_else(|| reject_bootstrap_authorization("owner_endpoint"))?;
        Some(
            reqwest::Client::new()
                .get(format!("{}/api/manifest", endpoint.trim_end_matches('/')))
                .header("x-tessara-module-control-key", module_control_key()?)
                .send()
                .await
                .map_err(|_| reject_bootstrap_authorization("manifest_request"))?
                .error_for_status()
                .map_err(|_| reject_bootstrap_authorization("manifest_status"))?
                .json::<ModuleManifest>()
                .await
                .map_err(|_| reject_bootstrap_authorization("manifest_decode"))?,
        )
    } else if request.owner_definition_id == "core" {
        None
    } else {
        return Err(reject_bootstrap_authorization("owner_not_enabled"));
    };
    let locked_bootstrap = match module {
        Some(module) => module.bootstrap.as_ref(),
        None => lockfile.core.bootstrap.as_ref(),
    };
    let (locked_input, receipt_bindings) = locked_bootstrap
        .and_then(|bootstrap| match bootstrap {
            tessara_composition::BootstrapInputV1::Inline {
                value,
                receipt_bindings,
                ..
            } => Some((value, receipt_bindings.as_slice())),
            tessara_composition::BootstrapInputV1::LocalCas { .. } => None,
        })
        .ok_or_else(|| reject_bootstrap_authorization("bootstrap_input"))?;
    if request.idempotency_key.trim().is_empty()
        || canonical_digest(locked_input).map_err(|error| ApiError::Internal(error.into()))?
            != request.locked_input_digest
        || canonical_digest(&request.input).map_err(|error| ApiError::Internal(error.into()))?
            != request.input_digest
        || !resolved_bootstrap_input_matches_lock(locked_input, receipt_bindings, &request.input)
    {
        return Err(reject_bootstrap_authorization("input_binding"));
    }
    if let Some(manifest) = manifest.as_ref() {
        let registry = crate::module_service_requests::configured_registry()
            .map_err(ApiError::Internal)?
            .ok_or_else(|| reject_bootstrap_authorization("service_registry"))?;
        if registry.identity(&manifest.definition_id).is_none() {
            return Err(reject_bootstrap_authorization("service_identity"));
        }
    }
    let validation = match (module, manifest.as_ref()) {
        (Some(module), Some(manifest)) => resolve_bootstrap_dependency_validation(
            &lockfile,
            module,
            manifest,
            Some(&request.input),
        )
        .map_err(|_| reject_bootstrap_authorization("dependency_validation"))?,
        _ => None,
    };
    if let Some(validation) = validation.as_ref() {
        require_bootstrap_validation_provider_target(&lockfile, &validation.target).await?;
    }
    let expires_at = std::cmp::min(
        now + Duration::seconds(30),
        request.apply_authorization.payload.expires_at,
    );
    if expires_at <= now {
        return Err(reject_bootstrap_authorization("authorization_expired"));
    }
    let original_actor_id =
        Uuid::parse_str(&request.apply_authorization.payload.initiator.actor_id)
            .map_err(|_| reject_bootstrap_authorization("initiator_identity"))?;
    let provider_actions = match (module, manifest.as_ref()) {
        (Some(module), Some(manifest)) => manifest
            .consumed_service_actions
            .iter()
            .filter_map(|action| {
                let binding = module
                    .dependency_bindings
                    .get(action.dependency_binding.as_str())?;
                if binding.provider != "core"
                    || binding.contract_id != action.functional_contract.as_str()
                {
                    return None;
                }
                let declaration = crate::core_service_providers::resolve_service_action(
                    action.functional_contract.as_str(),
                    &action.authorization_action,
                )?;
                Some((
                    declaration.required_capability,
                    OwnerBootstrapProviderActionV1 {
                        dependency_binding: action.dependency_binding.to_string(),
                        functional_contract: action.functional_contract.to_string(),
                        action: action.authorization_action.clone(),
                        method: declaration.method,
                        path: declaration.path.into(),
                        audience: AuthorizationAudienceV1::CoreInstallation {
                            installation_id: request.installation_id,
                        },
                    },
                ))
            })
            .collect::<Vec<_>>(),
        _ => Vec::new(),
    };
    let required_capabilities = provider_actions
        .iter()
        .map(|(capability, _)| *capability)
        .collect::<BTreeSet<_>>();
    let mut capability_scope_bindings = Vec::new();
    for required_capability in required_capabilities {
        capability_scope_bindings.extend(
            crate::core_security::capability_bindings(
                &state.pool,
                original_actor_id,
                required_capability,
            )
            .await?,
        );
    }
    let (authorization_revision, organization_revision) = sqlx::query_as::<_, (i64, i64)>(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    if authorization_revision <= 0 || organization_revision <= 0 {
        return Err(reject_bootstrap_authorization("security_revision"));
    }
    let owner = match (module_instance_id, manifest.as_ref()) {
        (Some(module_instance_id), Some(manifest)) => AuthorizationAudienceV1::ModuleInstance {
            module_instance_id,
            module_definition_id: manifest.definition_id.clone(),
        },
        _ => AuthorizationAudienceV1::CoreInstallation {
            installation_id: request.installation_id,
        },
    };
    let owner_authorization = crate::core_security::protocol_signer(
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
    )?
    .sign(OwnerBootstrapAuthorizationV1 {
        schema_version: OWNER_BOOTSTRAP_AUTHORIZATION_SCHEMA_VERSION_V1,
        installation_id: request.installation_id,
        owner,
        owner_definition_id: request.owner_definition_id.clone(),
        initiator: request.apply_authorization.payload.initiator.clone(),
        original_actor_id,
        capability_scope_bindings,
        provider_actions: provider_actions
            .into_iter()
            .map(|(_, action)| action)
            .collect(),
        authorization_revision: authorization_revision as u64,
        organization_revision: organization_revision as u64,
        locked_input_digest: request.locked_input_digest.clone(),
        input_digest: request.input_digest.clone(),
        desired_revision: request.desired_revision,
        apply_sequence: request.apply_sequence,
        target_plan_digest: request
            .apply_authorization
            .payload
            .target_plan_digest
            .clone(),
        idempotency_key: request.idempotency_key,
        correlation_id: Uuid::new_v4(),
        jti: Uuid::new_v4(),
        issued_at: now,
        expires_at,
    })
    .map_err(|error| ApiError::Internal(error.into()))?;
    let validation = if let Some(validation) = validation {
        let validation_request = BootstrapDependencyValidationRequestV1 {
            schema_version: BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1,
            input_digest: request.input_digest.clone(),
            desired_revision: request.desired_revision,
            apply_sequence: request.apply_sequence,
            target_plan_digest: request
                .apply_authorization
                .payload
                .target_plan_digest
                .clone(),
            payload: validation.payload,
        };
        let request_digest = canonical_digest(&validation_request)
            .map_err(|error| ApiError::Internal(error.into()))?;
        let authorization = crate::core_security::protocol_signer(
            ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        )?
        .sign(BootstrapDependencyValidationAuthorizationV1 {
            schema_version: BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_SCHEMA_VERSION_V1,
            installation_id: request.installation_id,
            module_instance_id: module_instance_id
                .ok_or_else(|| reject_bootstrap_authorization("module_instance"))?,
            module_definition_id: request.owner_definition_id,
            input_digest: request.input_digest,
            desired_revision: request.desired_revision,
            apply_sequence: request.apply_sequence,
            target_plan_digest: validation_request.target_plan_digest.clone(),
            dependency_binding: validation.target.dependency_binding.clone(),
            functional_contract: validation.target.functional_contract.clone(),
            functional_contract_version: validation.target.functional_contract_version.clone(),
            action: validation.target.action.clone(),
            method: validation.target.method,
            path: validation.target.path.clone(),
            audience: validation.target.audience.clone(),
            request_digest,
            correlation_id: owner_authorization.payload.correlation_id,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at,
        })
        .map_err(|error| ApiError::Internal(error.into()))?;
        Some(BootstrapDependencyValidationInvocationV1 {
            target: validation.target,
            request: validation_request,
            authorization,
        })
    } else {
        None
    };
    Ok(Json(
        BootstrapDependencyValidationAuthorizationIssueResponseV1 {
            authorization: owner_authorization,
            validation,
        },
    ))
}

fn reject_bootstrap_authorization(reason: &'static str) -> ApiError {
    tracing::warn!(
        operation = "bootstrap_dependency_authorization",
        result = "rejected",
        reason,
        "bootstrap dependency authorization rejected"
    );
    ApiError::Forbidden("bootstrap:authorize".into())
}

async fn require_bootstrap_validation_provider_target(
    lockfile: &ApplicationLockfileV1,
    target: &tessara_composition::BootstrapDependencyValidationTargetV1,
) -> ApiResult<()> {
    let invalid = || ApiError::Forbidden("bootstrap:authorize".into());
    match &target.audience {
        AuthorizationAudienceV1::CoreInstallation { installation_id } => {
            if *installation_id != lockfile.installation_id {
                return Err(invalid());
            }
            let action = crate::core_service_providers::resolve_service_action(
                &target.functional_contract,
                &target.action,
            )
            .ok_or_else(invalid)?;
            let contract_version =
                crate::core_service_providers::contract_version(&target.functional_contract)
                    .ok_or_else(invalid)?;
            if action.path != target.path
                || action.method != target.method
                || action.functional_contract != target.functional_contract
                || contract_version != target.functional_contract_version
            {
                return Err(invalid());
            }
        }
        AuthorizationAudienceV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => {
            let provider = lockfile
                .modules
                .iter()
                .find(|module| {
                    module.enabled && module.definition_id == module_definition_id.as_str()
                })
                .ok_or_else(invalid)?;
            if *module_instance_id
                != tessara_composition::module_instance_id(
                    lockfile.installation_id,
                    &provider.definition_id,
                )
            {
                return Err(invalid());
            }
            let endpoint = module_control_endpoints()?
                .remove(&provider.definition_id)
                .ok_or_else(invalid)?;
            let manifest = reqwest::Client::new()
                .get(format!("{}/api/manifest", endpoint.trim_end_matches('/')))
                .header("x-tessara-module-control-key", module_control_key()?)
                .send()
                .await
                .map_err(|_| invalid())?
                .error_for_status()
                .map_err(|_| invalid())?
                .json::<ModuleManifest>()
                .await
                .map_err(|_| invalid())?;
            if manifest.definition_id != *module_definition_id
                || manifest.release_version != provider.version
                || canonical_digest(&manifest).map_err(|error| ApiError::Internal(error.into()))?
                    != provider.manifest_digest
                || !manifest.provided_contracts.iter().any(|contract| {
                    contract.id.as_str() == target.functional_contract
                        && contract.version == target.functional_contract_version
                })
                || !manifest.provided_service_actions.iter().any(|action| {
                    action.functional_contract.as_str() == target.functional_contract
                        && action.authorization_action == target.action
                        && action.method == target.method
                        && action.path == target.path
                })
            {
                return Err(invalid());
            }
        }
    }
    Ok(())
}

fn resolved_bootstrap_input_matches_lock(
    locked_input: &Value,
    receipt_bindings: &[tessara_composition::BootstrapReceiptBindingV1],
    resolved_input: &Value,
) -> bool {
    if receipt_bindings.is_empty() {
        return locked_input == resolved_input;
    }
    let mut normalized = resolved_input.clone();
    let mut targets = BTreeSet::new();
    for binding in receipt_bindings {
        if !binding.target_pointer.starts_with('/')
            || binding.source_owner.trim().is_empty()
            || binding.resource_key.trim().is_empty()
            || !targets.insert(binding.target_pointer.as_str())
            || locked_input.pointer(&binding.target_pointer) != Some(&Value::Null)
        {
            return false;
        }
        let Some(target) = normalized.pointer_mut(&binding.target_pointer) else {
            return false;
        };
        if target.is_null() {
            return false;
        }
        *target = Value::Null;
    }
    &normalized == locked_input
}

async fn enroll_bootstrap_capabilities(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BootstrapCapabilityEnrollmentRequestV1>,
) -> ApiResult<StatusCode> {
    require_projection_token(&headers)?;
    let stored_lockfile: Value = sqlx::query_scalar(
        "SELECT document FROM composition_lockfiles
         WHERE installation_id=$1 AND blueprint_revision=$2",
    )
    .bind(request.lockfile.installation_id)
    .bind(request.lockfile.blueprint_revision as i64)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::BadRequest("Bootstrap capability lockfile is unavailable".into()))?;
    let stored_lockfile: ApplicationLockfileV1 = serde_json::from_value(stored_lockfile)
        .map_err(|error| ApiError::Internal(error.into()))?;
    if stored_lockfile != request.lockfile {
        return Err(ApiError::BadRequest(
            "Bootstrap capability lockfile is not the source-exact projected lock".into(),
        ));
    }
    validate_bootstrap_capability_manifests(&request.lockfile.modules, &request.manifests)?;

    let mut transaction = state.pool.begin().await?;
    insert_bootstrap_capabilities(&mut transaction, &request.manifests).await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

fn validate_bootstrap_capability_manifests(
    modules: &[tessara_composition::ResolvedModuleReleaseV1],
    manifests: &BTreeMap<String, ModuleManifest>,
) -> ApiResult<()> {
    let expected_definitions = modules
        .iter()
        .filter(|module| module.enabled)
        .map(|module| module.definition_id.as_str())
        .collect::<BTreeSet<_>>();
    if manifests.len() != expected_definitions.len()
        || manifests
            .keys()
            .any(|definition| !expected_definitions.contains(definition.as_str()))
    {
        return Err(ApiError::BadRequest(
            "Bootstrap capability manifests do not exactly cover enabled lockfile modules".into(),
        ));
    }
    for module in modules.iter().filter(|module| module.enabled) {
        let manifest = manifests.get(&module.definition_id).ok_or_else(|| {
            ApiError::BadRequest("Bootstrap capability manifest is unavailable".into())
        })?;
        if manifest.definition_id.as_str() != module.definition_id
            || manifest.release_version != module.version
            || canonical_digest(manifest).map_err(|error| ApiError::Internal(error.into()))?
                != module.manifest_digest
        {
            return Err(ApiError::BadRequest(format!(
                "Bootstrap capability manifest for '{}' is not source-exact",
                module.definition_id
            )));
        }
    }
    Ok(())
}

async fn insert_bootstrap_capabilities(
    transaction: &mut Transaction<'_, Postgres>,
    manifests: &BTreeMap<String, ModuleManifest>,
) -> ApiResult<()> {
    for manifest in manifests.values() {
        for capability in &manifest.security_capabilities {
            sqlx::query(
                "INSERT INTO capabilities(key,description,scope_mode)
                 VALUES($1,$2,'scope_aware')
                 ON CONFLICT(key) DO UPDATE SET description=EXCLUDED.description,
                     scope_mode=EXCLUDED.scope_mode",
            )
            .bind(capability.id.as_str())
            .bind(&capability.description)
            .execute(&mut **transaction)
            .await?;
        }
    }
    Ok(())
}

async fn apply_core_bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<tessara_composition::OwnerBootstrapRequestV1<CoreBootstrapV1>>,
) -> ApiResult<Json<tessara_composition::OwnerBootstrapResponseV1>> {
    require_projection_token(&headers)?;
    let core_owner = AuthorizationAudienceV1::CoreInstallation {
        installation_id: request.installation_id,
    };
    if request.input.schema_version != "tessara.io/core-bootstrap/v1"
        || request.apply_sequence == 0
        || request.dependency_validation.is_some()
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|error| ApiError::Internal(error.into()))?
        || request
            .validate_authorization_for(
                &crate::core_security::protocol_signer(
                    ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
                )?
                .verifier(),
                &core_owner,
                "core",
                Utc::now(),
            )
            .is_err()
    {
        return Err(ApiError::BadRequest(
            "Core bootstrap contract or digest is invalid".into(),
        ));
    }
    if let Some((locked_digest, digest, desired_revision, apply_sequence, receipt)) =
        sqlx::query_as::<_, (String, String, i64, i64, Value)>(
            "SELECT locked_input_digest,input_digest,desired_revision,apply_sequence,receipt
             FROM core_bootstrap_receipts WHERE idempotency_key=$1",
        )
        .bind(&request.idempotency_key)
        .fetch_optional(&state.pool)
        .await?
    {
        if locked_digest != request.locked_input_digest.to_string()
            || digest != request.input_digest.to_string()
            || desired_revision != request.desired_revision as i64
            || apply_sequence != request.apply_sequence as i64
        {
            return Err(ApiError::BadRequest(
                "Core bootstrap idempotency key was reused with different locked input or apply identity"
                    .into(),
            ));
        }
        let mut response: tessara_composition::OwnerBootstrapResponseV1 =
            serde_json::from_value(receipt).map_err(|error| ApiError::Internal(error.into()))?;
        response.receipt.changed = false;
        response.signed_receipt = crate::core_security::protocol_signer(
            ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        )?
        .sign(response.receipt.clone())
        .map_err(|error| ApiError::Internal(error.into()))?;
        return Ok(Json(response));
    }
    let installation_id = installation_id(&state).await?;
    if installation_id != request.installation_id {
        return Err(ApiError::BadRequest(
            "Core bootstrap input is invalid".into(),
        ));
    }
    validate_core_bootstrap_input(&request.input)?;
    let mut transaction = state.pool.begin().await?;
    let resource_ids = materialize_core_bootstrap(
        &mut transaction,
        request.installation_id,
        request.authorization.payload.original_actor_id,
        &request.input,
    )
    .await?;
    let result_digest =
        canonical_digest(&resource_ids).map_err(|error| ApiError::Internal(error.into()))?;
    let response = tessara_composition::OwnerBootstrapResponseV1::signed(
        tessara_composition::BootstrapReceiptV1 {
            owner: "core".into(),
            schema_version: request.input.schema_version.clone(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids,
        },
        &crate::core_security::protocol_signer(ProtocolSignaturePurposeV1::OwnerBootstrapReceipt)?,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    sqlx::query("INSERT INTO core_bootstrap_receipts(idempotency_key,locked_input_digest,input_digest,desired_revision,apply_sequence,authority_jti,receipt) VALUES($1,$2,$3,$4,$5,$6,$7)")
        .bind(&request.idempotency_key).bind(request.locked_input_digest.to_string())
        .bind(request.input_digest.to_string()).bind(request.desired_revision as i64)
        .bind(request.apply_sequence as i64).bind(request.authorization.payload.jti)
        .bind(serde_json::to_value(&response).map_err(|error| ApiError::Internal(error.into()))?)
        .execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(response))
}

#[derive(Clone)]
struct BootstrappedForm {
    form_version_id: Uuid,
    workflow_version_id: Uuid,
    workflow_step_id: Uuid,
    field_ids: BTreeMap<String, Uuid>,
}

fn validate_core_bootstrap_input(input: &CoreBootstrapV1) -> ApiResult<()> {
    if input.schema_version != "tessara.io/core-bootstrap/v1"
        || !is_core_bootstrap_key(&input.root_node_type_external_key)
        || !is_core_bootstrap_key(&input.root_node_external_key)
        || input.root_node_type_name.trim().is_empty()
        || input.root_node_name.trim().is_empty()
    {
        return Err(ApiError::BadRequest(
            "Core bootstrap root identity is invalid".into(),
        ));
    }

    let mut node_keys = BTreeSet::from([input.root_node_external_key.as_str()]);
    for node in &input.additional_nodes {
        if !is_core_bootstrap_key(&node.external_key)
            || node.name.trim().is_empty()
            || !node_keys.insert(&node.external_key)
            || node
                .parent_node_key
                .as_deref()
                .is_some_and(|parent| !node_keys.contains(parent))
        {
            return Err(ApiError::BadRequest(
                "Core bootstrap nodes must have unique logical keys and reference only an earlier parent"
                    .into(),
            ));
        }
    }

    let mut form_keys = BTreeSet::new();
    let mut form_slugs = BTreeSet::new();
    for form in &input.forms {
        let Ok(version) = semver::Version::parse(&form.version_label) else {
            return Err(ApiError::BadRequest(
                "Core bootstrap FormVersion label is invalid".into(),
            ));
        };
        let mut field_keys = BTreeSet::new();
        let mut scope_node_keys = BTreeSet::new();
        if !is_core_bootstrap_key(&form.resource_key)
            || !is_core_bootstrap_key(&form.source_alias)
            || form.name.trim().is_empty()
            || !is_core_bootstrap_slug(&form.slug)
            || !form_keys.insert(&form.resource_key)
            || !form_slugs.insert(&form.slug)
            || version.major == 0
            || !version.pre.is_empty()
            || !version.build.is_empty()
            || form.scope_node_keys.is_empty()
            || form.scope_node_keys.iter().any(|key| {
                !node_keys.contains(key.as_str()) || !scope_node_keys.insert(key.as_str())
            })
            || form.fields.is_empty()
            || form.fields.iter().any(|field| {
                !is_core_bootstrap_key(&field.key)
                    || field.label.trim().is_empty()
                    || !matches!(
                        field.field_type.as_str(),
                        "text" | "number" | "boolean" | "date" | "single_choice" | "multi_choice"
                    )
                    || !field_keys.insert(&field.key)
                    || field.position < 0
                    || field.grid_row <= 0
                    || field.grid_column <= 0
            })
        {
            return Err(ApiError::BadRequest(
                "Core bootstrap Form source contract is invalid".into(),
            ));
        }
    }

    let mut response_keys = BTreeSet::new();
    for response in &input.responses {
        let Some(form) = input
            .forms
            .iter()
            .find(|form| form.resource_key == response.form_resource_key)
        else {
            return Err(ApiError::BadRequest(
                "Core bootstrap Response references an unknown Form".into(),
            ));
        };
        let field_keys = form
            .fields
            .iter()
            .map(|field| field.key.as_str())
            .collect::<BTreeSet<_>>();
        let required_field_keys = form
            .fields
            .iter()
            .filter(|field| field.required)
            .map(|field| field.key.as_str())
            .collect::<BTreeSet<_>>();
        if !is_core_bootstrap_key(&response.resource_key)
            || !response_keys.insert(&response.resource_key)
            || !node_keys.contains(response.node_key.as_str())
            || response.submitted_at < response.created_at
            || response.values.is_empty()
            || response
                .values
                .keys()
                .any(|key| !field_keys.contains(key.as_str()))
            || required_field_keys.iter().any(|key| {
                response
                    .values
                    .get(*key)
                    .is_none_or(serde_json::Value::is_null)
            })
        {
            return Err(ApiError::BadRequest(
                "Core bootstrap submitted Response contract is invalid".into(),
            ));
        }
        for (key, value) in &response.values {
            if value.is_null() {
                continue;
            }
            let field = form
                .fields
                .iter()
                .find(|field| field.key == *key)
                .expect("unknown Response fields were rejected above");
            let field_type = crate::hierarchy::parse_field_type(&field.field_type)?;
            crate::hierarchy::validate_field_value(field_type, value).map_err(|_| {
                ApiError::BadRequest(
                    "Core bootstrap submitted Response value type is invalid".into(),
                )
            })?;
        }
    }

    let mut actor_keys = BTreeSet::from(["actor.admin"]);
    let mut actor_emails = BTreeSet::new();
    for actor in &input.actors {
        let mut capabilities = BTreeSet::new();
        let mut scope_node_keys = BTreeSet::new();
        if !is_core_bootstrap_key(&actor.resource_key)
            || !actor.resource_key.starts_with("actor.")
            || !actor_keys.insert(&actor.resource_key)
            || actor.email.trim().is_empty()
            || !actor_emails.insert(actor.email.to_ascii_lowercase())
            || actor.display_name.trim().is_empty()
            || actor.password.len() < 12
            || actor.capabilities.is_empty()
            || actor.capabilities.iter().any(|capability| {
                capability.trim().is_empty() || !capabilities.insert(capability.as_str())
            })
            || actor.scope_node_keys.is_empty()
            || actor.scope_node_keys.iter().any(|key| {
                !node_keys.contains(key.as_str()) || !scope_node_keys.insert(key.as_str())
            })
        {
            return Err(ApiError::BadRequest(
                "Core bootstrap actor/RBAC contract is invalid".into(),
            ));
        }
    }

    let mut receipt_keys = node_keys
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>();
    if !receipt_keys.insert("actor.admin".into())
        || input
            .actors
            .iter()
            .any(|actor| !receipt_keys.insert(actor.resource_key.clone()))
        || input
            .responses
            .iter()
            .any(|response| !receipt_keys.insert(response.resource_key.clone()))
    {
        return Err(ApiError::BadRequest(
            "Core bootstrap receipt logical keys overlap".into(),
        ));
    }
    for form in &input.forms {
        for suffix in ["form_id", "form_version_id", "dataset_source", "schema"] {
            if !receipt_keys.insert(format!("{}.{suffix}", form.resource_key)) {
                return Err(ApiError::BadRequest(
                    "Core bootstrap Form receipt logical keys overlap".into(),
                ));
            }
        }
    }
    Ok(())
}

async fn materialize_core_bootstrap(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    apply_actor_id: Uuid,
    input: &CoreBootstrapV1,
) -> ApiResult<BTreeMap<String, String>> {
    let node_type_id = core_bootstrap_resource_id(
        installation_id,
        "node_type",
        &input.root_node_type_external_key,
    );
    sqlx::query(
        "INSERT INTO node_types(id,name,slug,plural_label,description)
         VALUES($1,$2,$3,$4,$5)
         ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,
             plural_label=EXCLUDED.plural_label,description=EXCLUDED.description",
    )
    .bind(node_type_id)
    .bind(input.root_node_type_name.trim())
    .bind(&input.root_node_type_external_key)
    .bind(format!("{}s", input.root_node_type_name.trim()))
    .bind("Application composition bootstrap scope")
    .execute(&mut **transaction)
    .await?;

    let mut node_ids = BTreeMap::new();
    let root_node_id =
        core_bootstrap_resource_id(installation_id, "node", &input.root_node_external_key);
    sqlx::query(
        "INSERT INTO nodes(id,node_type_id,parent_node_id,name)
         VALUES($1,$2,NULL,$3)
         ON CONFLICT(id) DO UPDATE SET node_type_id=EXCLUDED.node_type_id,
             parent_node_id=NULL,name=EXCLUDED.name",
    )
    .bind(root_node_id)
    .bind(node_type_id)
    .bind(input.root_node_name.trim())
    .execute(&mut **transaction)
    .await?;
    node_ids.insert(input.root_node_external_key.clone(), root_node_id);
    for node in &input.additional_nodes {
        let node_id = core_bootstrap_resource_id(installation_id, "node", &node.external_key);
        let parent_id = node
            .parent_node_key
            .as_ref()
            .and_then(|key| node_ids.get(key))
            .copied();
        sqlx::query(
            "INSERT INTO nodes(id,node_type_id,parent_node_id,name)
             VALUES($1,$2,$3,$4)
             ON CONFLICT(id) DO UPDATE SET node_type_id=EXCLUDED.node_type_id,
                 parent_node_id=EXCLUDED.parent_node_id,name=EXCLUDED.name",
        )
        .bind(node_id)
        .bind(node_type_id)
        .bind(parent_id)
        .bind(node.name.trim())
        .execute(&mut **transaction)
        .await?;
        node_ids.insert(node.external_key.clone(), node_id);
    }

    let mut resources = node_ids
        .iter()
        .map(|(key, id)| (key.clone(), id.to_string()))
        .collect::<BTreeMap<_, _>>();
    resources.insert("actor.admin".into(), apply_actor_id.to_string());
    materialize_core_bootstrap_actors(
        transaction,
        installation_id,
        &node_ids,
        &input.actors,
        &mut resources,
    )
    .await?;
    let forms = materialize_core_bootstrap_forms(
        transaction,
        installation_id,
        node_type_id,
        &node_ids,
        &input.forms,
        &mut resources,
    )
    .await?;
    materialize_core_bootstrap_responses(
        transaction,
        installation_id,
        apply_actor_id,
        &node_ids,
        &forms,
        &input.responses,
        &mut resources,
    )
    .await?;
    Ok(resources)
}

async fn materialize_core_bootstrap_actors(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    node_ids: &BTreeMap<String, Uuid>,
    actors: &[CoreBootstrapActorV1],
    resources: &mut BTreeMap<String, String>,
) -> ApiResult<()> {
    for actor in actors {
        let account_id =
            core_bootstrap_resource_id(installation_id, "account", &actor.resource_key);
        let role_id = core_bootstrap_resource_id(installation_id, "role", &actor.resource_key);
        let password_hash = auth::hash_password_for_storage(&actor.password)?;
        sqlx::query(
            "INSERT INTO accounts(id,email,display_name,is_active) VALUES($1,$2,$3,true)
             ON CONFLICT(id) DO UPDATE SET email=EXCLUDED.email,
                 display_name=EXCLUDED.display_name,is_active=true",
        )
        .bind(account_id)
        .bind(actor.email.trim().to_ascii_lowercase())
        .bind(actor.display_name.trim())
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            "INSERT INTO account_credentials(account_id,password_hash,password_scheme)
             VALUES($1,$2,$3)
             ON CONFLICT(account_id) DO UPDATE SET password_hash=EXCLUDED.password_hash,
                 password_scheme=EXCLUDED.password_scheme,password_updated_at=now()",
        )
        .bind(account_id)
        .bind(password_hash)
        .bind(auth::password_scheme())
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            "INSERT INTO roles(id,name,description) VALUES($1,$2,$3)
             ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,description=EXCLUDED.description",
        )
        .bind(role_id)
        .bind(format!(
            "sprint-8b-{}",
            actor.resource_key.replace('.', "-")
        ))
        .bind("Sprint 8B owner-bootstrap fixture role")
        .execute(&mut **transaction)
        .await?;
        sqlx::query("DELETE FROM role_capabilities WHERE role_id=$1")
            .bind(role_id)
            .execute(&mut **transaction)
            .await?;
        for capability in &actor.capabilities {
            let capability_id: Uuid =
                sqlx::query_scalar("SELECT id FROM capabilities WHERE key=$1")
                    .bind(capability)
                    .fetch_optional(&mut **transaction)
                    .await?
                    .ok_or_else(|| {
                        ApiError::BadRequest(format!(
                            "Core bootstrap actor capability '{capability}' is not enrolled"
                        ))
                    })?;
            sqlx::query("INSERT INTO role_capabilities(role_id,capability_id) VALUES($1,$2)")
                .bind(role_id)
                .bind(capability_id)
                .execute(&mut **transaction)
                .await?;
        }
        sqlx::query("DELETE FROM role_assignments WHERE account_id=$1 AND role_id=$2")
            .bind(account_id)
            .bind(role_id)
            .execute(&mut **transaction)
            .await?;
        for node_key in &actor.scope_node_keys {
            let node_id = node_ids[node_key];
            let assignment_id = core_bootstrap_resource_id(
                installation_id,
                "role_assignment",
                &format!("{}:{node_key}", actor.resource_key),
            );
            sqlx::query(
                "INSERT INTO role_assignments(id,account_id,role_id,node_id)
                 VALUES($1,$2,$3,$4)",
            )
            .bind(assignment_id)
            .bind(account_id)
            .bind(role_id)
            .bind(node_id)
            .execute(&mut **transaction)
            .await?;
        }
        resources.insert(actor.resource_key.clone(), account_id.to_string());
    }
    Ok(())
}

async fn materialize_core_bootstrap_forms(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    node_type_id: Uuid,
    node_ids: &BTreeMap<String, Uuid>,
    forms: &[CoreBootstrapFormV1],
    resources: &mut BTreeMap<String, String>,
) -> ApiResult<BTreeMap<String, BootstrappedForm>> {
    let mut result = BTreeMap::new();
    for form in forms {
        let form_id = core_bootstrap_resource_id(installation_id, "form", &form.resource_key);
        let compatibility_group_id =
            core_bootstrap_resource_id(installation_id, "form_compatibility", &form.resource_key);
        let form_version_id =
            core_bootstrap_resource_id(installation_id, "form_version", &form.resource_key);
        let section_id =
            core_bootstrap_resource_id(installation_id, "form_section", &form.resource_key);
        let version = semver::Version::parse(&form.version_label)
            .map_err(|error| ApiError::BadRequest(error.to_string()))?;
        let version_major = i32::try_from(version.major).map_err(|_| {
            ApiError::BadRequest("Core bootstrap FormVersion major is too large".into())
        })?;
        let version_minor = i32::try_from(version.minor).map_err(|_| {
            ApiError::BadRequest("Core bootstrap FormVersion minor is too large".into())
        })?;
        let version_patch = i32::try_from(version.patch).map_err(|_| {
            ApiError::BadRequest("Core bootstrap FormVersion patch is too large".into())
        })?;
        sqlx::query(
            "INSERT INTO forms(id,name,slug,scope_node_type_id) VALUES($1,$2,$3,$4)
             ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,
                 scope_node_type_id=EXCLUDED.scope_node_type_id",
        )
        .bind(form_id)
        .bind(form.name.trim())
        .bind(&form.slug)
        .bind(node_type_id)
        .execute(&mut **transaction)
        .await?;
        for node_key in &form.scope_node_keys {
            sqlx::query(
                "INSERT INTO form_scope_nodes(form_id,node_id) VALUES($1,$2)
                 ON CONFLICT DO NOTHING",
            )
            .bind(form_id)
            .bind(node_ids[node_key])
            .execute(&mut **transaction)
            .await?;
        }
        sqlx::query(
            "INSERT INTO compatibility_groups(id,form_id,name) VALUES($1,$2,'Initial')
             ON CONFLICT(id) DO UPDATE SET form_id=EXCLUDED.form_id,name=EXCLUDED.name",
        )
        .bind(compatibility_group_id)
        .bind(form_id)
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            "INSERT INTO form_versions(
                 id,form_id,compatibility_group_id,version_label,status,
                 version_major,version_minor,version_patch,semantic_bump,
                 started_new_major_line,published_at)
             VALUES($1,$2,$3,$4,'published'::form_version_status,$5,$6,$7,'INITIAL',true,$8)
             ON CONFLICT(id) DO UPDATE SET compatibility_group_id=EXCLUDED.compatibility_group_id,
                 version_label=EXCLUDED.version_label,status=EXCLUDED.status,
                 version_major=EXCLUDED.version_major,version_minor=EXCLUDED.version_minor,
                 version_patch=EXCLUDED.version_patch,semantic_bump=EXCLUDED.semantic_bump,
                 started_new_major_line=true,published_at=EXCLUDED.published_at",
        )
        .bind(form_version_id)
        .bind(form_id)
        .bind(compatibility_group_id)
        .bind(&form.version_label)
        .bind(version_major)
        .bind(version_minor)
        .bind(version_patch)
        .bind(form.published_at)
        .execute(&mut **transaction)
        .await?;
        sqlx::query(
            "INSERT INTO form_sections(id,form_version_id,title,description,position)
             VALUES($1,$2,'Response','Sprint 8B source fields',0)
             ON CONFLICT(id) DO UPDATE SET title=EXCLUDED.title,
                 description=EXCLUDED.description,position=EXCLUDED.position",
        )
        .bind(section_id)
        .bind(form_version_id)
        .execute(&mut **transaction)
        .await?;
        let mut field_ids = BTreeMap::new();
        for field in &form.fields {
            let field_id = core_bootstrap_resource_id(
                installation_id,
                "form_field",
                &format!("{}:{}", form.resource_key, field.key),
            );
            sqlx::query(
                "INSERT INTO form_fields(
                     field_id,form_version_id,section_id,key,label,field_type,required,
                     position,grid_row,grid_column,grid_width,grid_height)
                 VALUES($1,$2,$3,$4,$5,$6::field_type,$7,$8,$9,$10,1,1)
                 ON CONFLICT(form_version_id,field_id) DO UPDATE SET section_id=EXCLUDED.section_id,
                     key=EXCLUDED.key,label=EXCLUDED.label,field_type=EXCLUDED.field_type,
                     required=EXCLUDED.required,position=EXCLUDED.position,
                     grid_row=EXCLUDED.grid_row,grid_column=EXCLUDED.grid_column",
            )
            .bind(field_id)
            .bind(form_version_id)
            .bind(section_id)
            .bind(&field.key)
            .bind(field.label.trim())
            .bind(&field.field_type)
            .bind(field.required)
            .bind(field.position)
            .bind(field.grid_row)
            .bind(field.grid_column)
            .execute(&mut **transaction)
            .await?;
            field_ids.insert(field.key.clone(), field_id);
        }
        let (_, workflow_version_id, workflow_step_id) =
            crate::workflows::ensure_workflow_for_published_form_version_tx(
                transaction,
                form_version_id,
            )
            .await?;
        let source = tessara_datasets_contract::DatasetProductSourceV1::Form {
            alias: form.source_alias.clone(),
            form_id: form_id.to_string(),
            form_version_id: form_version_id.to_string(),
        };
        let mut source_scope_node_ids = form
            .scope_node_keys
            .iter()
            .map(|key| node_ids[key])
            .collect::<Vec<_>>();
        source_scope_node_ids.sort_unstable();
        let mut schema_fields = form
            .fields
            .iter()
            .map(|field| tessara_forms_contract::FormVersionField {
                field_id: field_ids[&field.key],
                key: field.key.clone(),
                label: field.label.trim().to_string(),
                field_type: field.field_type.clone(),
                required: field.required,
                options: Vec::new(),
                section_id: Some(section_id),
                position: field.position,
                grid_row: field.grid_row,
                grid_column: field.grid_column,
            })
            .collect::<Vec<_>>();
        schema_fields.sort_by(|left, right| {
            (
                left.position,
                left.grid_row,
                left.grid_column,
                &left.label,
                &left.key,
                left.field_id,
            )
                .cmp(&(
                    right.position,
                    right.grid_row,
                    right.grid_column,
                    &right.label,
                    &right.key,
                    right.field_id,
                ))
        });
        let schema = tessara_forms_contract::FormVersionSchemaResponse {
            schema_version: tessara_forms_contract::FORM_VERSION_SCHEMA_VERSION,
            form_id,
            form_version_id,
            form_name: form.name.trim().to_string(),
            form_slug: form.slug.clone(),
            version_label: Some(form.version_label.clone()),
            version_major: Some(version_major),
            source_scope_node_ids,
            source_scope_revision: String::new(),
            source_scope_digest: String::new(),
            content_revision: String::new(),
            content_digest: String::new(),
            sections: vec![tessara_forms_contract::FormVersionSection {
                section_id,
                key: section_id.to_string(),
                label: "Response".into(),
                position: 0,
            }],
            fields: schema_fields,
        }
        .with_recomputed_digests()
        .map_err(|error| ApiError::Internal(error.into()))?;
        resources.insert(
            format!("{}.form_id", form.resource_key),
            form_id.to_string(),
        );
        resources.insert(
            format!("{}.form_version_id", form.resource_key),
            form_version_id.to_string(),
        );
        resources.insert(
            format!("{}.dataset_source", form.resource_key),
            serde_json::to_string(&source).map_err(|error| ApiError::Internal(error.into()))?,
        );
        resources.insert(
            format!("{}.schema", form.resource_key),
            serde_json::to_string(&schema).map_err(|error| ApiError::Internal(error.into()))?,
        );
        result.insert(
            form.resource_key.clone(),
            BootstrappedForm {
                form_version_id,
                workflow_version_id,
                workflow_step_id,
                field_ids,
            },
        );
    }
    Ok(result)
}

async fn materialize_core_bootstrap_responses(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    apply_actor_id: Uuid,
    node_ids: &BTreeMap<String, Uuid>,
    forms: &BTreeMap<String, BootstrappedForm>,
    responses: &[CoreBootstrapResponseV1],
    resources: &mut BTreeMap<String, String>,
) -> ApiResult<()> {
    crate::response_owner_actions::defer_export_capture_tx(transaction).await?;
    for response in responses {
        let form = &forms[&response.form_resource_key];
        let node_id = node_ids[&response.node_key];
        let assignment_key = format!(
            "{}:{}:{apply_actor_id}",
            response.form_resource_key, response.node_key
        );
        let assignment_id =
            core_bootstrap_resource_id(installation_id, "workflow_assignment", &assignment_key);
        let assignment_id: Uuid = sqlx::query_scalar(
            "INSERT INTO workflow_assignments(
                 id,workflow_version_id,workflow_step_id,node_id,account_id,
                 assigned_by_account_id,is_active)
             VALUES($1,$2,$3,$4,$5,$5,true)
             ON CONFLICT(workflow_step_id,node_id,account_id) DO UPDATE SET
                 workflow_version_id=EXCLUDED.workflow_version_id,is_active=true
             RETURNING id",
        )
        .bind(assignment_id)
        .bind(form.workflow_version_id)
        .bind(form.workflow_step_id)
        .bind(node_id)
        .bind(apply_actor_id)
        .fetch_one(&mut **transaction)
        .await?;
        let response_id =
            core_bootstrap_resource_id(installation_id, "response", &response.resource_key);
        sqlx::query(
            "INSERT INTO submissions(
                 id,form_version_id,node_id,workflow_assignment_id,status,submitted_at,created_at)
             VALUES($1,$2,$3,$4,'draft'::submission_status,NULL,$5)",
        )
        .bind(response_id)
        .bind(form.form_version_id)
        .bind(node_id)
        .bind(assignment_id)
        .bind(response.created_at)
        .execute(&mut **transaction)
        .await?;
        for (field_key, value) in &response.values {
            sqlx::query(
                "INSERT INTO submission_values(submission_id,form_version_id,field_id,value)
                 VALUES($1,$2,$3,$4)",
            )
            .bind(response_id)
            .bind(form.form_version_id)
            .bind(form.field_ids[field_key])
            .bind(value)
            .execute(&mut **transaction)
            .await?;
        }
        let audit_id =
            core_bootstrap_resource_id(installation_id, "response_audit", &response.resource_key);
        sqlx::query(
            "INSERT INTO submission_audit_events(id,submission_id,event_type,account_id,created_at)
             VALUES($1,$2,$3,$4,$5)",
        )
        .bind(audit_id)
        .bind(response_id)
        .bind(format!("bootstrap:{}", response.resource_key))
        .bind(apply_actor_id)
        .bind(response.submitted_at)
        .execute(&mut **transaction)
        .await?;
        // Build the complete aggregate while it is a draft so the generic
        // table triggers cannot publish partial value/audit snapshots. The
        // final lifecycle transition emits exactly one immutable upsert with
        // the complete value set and authoritative audit identity.
        sqlx::query(
            "UPDATE submissions
                SET status='submitted'::submission_status,submitted_at=$2
              WHERE id=$1 AND status='draft'::submission_status",
        )
        .bind(response_id)
        .bind(response.submitted_at)
        .execute(&mut **transaction)
        .await?;
        crate::response_owner_actions::append_final_upsert_tx(transaction, response_id).await?;
        resources.insert(response.resource_key.clone(), response_id.to_string());
    }
    Ok(())
}

fn core_bootstrap_resource_id(installation_id: Uuid, kind: &str, key: &str) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(b"tessara.core.owner-bootstrap.v1\0");
    digest.update(installation_id.as_bytes());
    digest.update(b"\0");
    digest.update(kind.as_bytes());
    digest.update(b"\0");
    digest.update(key.as_bytes());
    let digest = digest.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn is_core_bootstrap_key(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 128
        && bytes[0].is_ascii_lowercase()
        && bytes.iter().all(|byte| {
            byte.is_ascii_lowercase()
                || byte.is_ascii_digit()
                || matches!(byte, b'.' | b'-' | b'/' | b'_' | b':')
        })
}

fn is_core_bootstrap_slug(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= 128
        && bytes[0].is_ascii_lowercase()
        && bytes
            .last()
            .is_some_and(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-')
}

async fn adopt_drift(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(finding_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    require_global(&auth, "composition:approve")?;
    let installation_id = installation_id(&state).await?;
    let row = sqlx::query("SELECT path,observed FROM composition_drift_findings WHERE finding_id=$1 AND installation_id=$2 AND disposition='open'")
        .bind(finding_id).bind(installation_id).fetch_optional(&state.pool).await?
        .ok_or_else(|| ApiError::NotFound("Open drift finding was not found".into()))?;
    let path: String = row.try_get("path")?;
    let observed: Value = row.try_get("observed")?;
    let (definition_id, dimension) = drift_target(&path)?;
    let mut blueprint: ApplicationBlueprintV1 = serde_json::from_value(
        sqlx::query_scalar("SELECT document FROM composition_blueprints WHERE installation_id=$1 ORDER BY revision DESC LIMIT 1")
            .bind(installation_id).fetch_one(&state.pool).await?,
    ).map_err(|error| ApiError::Internal(error.into()))?;
    let module = blueprint
        .modules
        .iter_mut()
        .find(|module| module.definition_id == definition_id)
        .ok_or_else(|| {
            ApiError::BadRequest("Drift owner is not in the current Blueprint".into())
        })?;
    match dimension {
        "configuration" => module.configuration = observed,
        "enabled" => {
            module.enabled = observed.as_bool().ok_or_else(|| {
                ApiError::BadRequest("Observed enablement drift is invalid".into())
            })?
        }
        _ => unreachable!(),
    }
    blueprint.revision = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(MAX(revision),0)+1 FROM composition_blueprints WHERE installation_id=$1",
    )
    .bind(installation_id)
    .fetch_one(&state.pool)
    .await? as u64;
    let digest = canonical_digest(&blueprint)
        .map_err(|error| ApiError::Internal(error.into()))?
        .to_string();
    let mut transaction = state.pool.begin().await?;
    sqlx::query("INSERT INTO composition_blueprints(installation_id,revision,digest,document,state,created_by) VALUES($1,$2,$3,$4,'draft',$5)")
        .bind(installation_id).bind(blueprint.revision as i64).bind(digest)
        .bind(serde_json::to_value(&blueprint).map_err(|error| ApiError::Internal(error.into()))?)
        .bind(auth.account_id).execute(&mut *transaction).await?;
    sqlx::query("UPDATE composition_drift_findings SET disposition='adopted',resolved_at=now() WHERE finding_id=$1")
        .bind(finding_id).execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reconcile_drift(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(finding_id): Path<Uuid>,
) -> ApiResult<StatusCode> {
    require_global(&auth, "composition:approve")?;
    let installation_id = installation_id(&state).await?;
    let row = sqlx::query("SELECT path,desired FROM composition_drift_findings WHERE finding_id=$1 AND installation_id=$2 AND disposition='open'")
        .bind(finding_id).bind(installation_id).fetch_optional(&state.pool).await?
        .ok_or_else(|| ApiError::NotFound("Open drift finding was not found".into()))?;
    let path: String = row.try_get("path")?;
    let desired: Value = row.try_get("desired")?;
    let (definition_id, dimension) = drift_target(&path)?;
    if dimension == "enabled" {
        let revision: i64 = sqlx::query_scalar("SELECT blueprint_revision FROM composition_lockfiles WHERE installation_id=$1 ORDER BY blueprint_revision DESC LIMIT 1")
            .bind(installation_id).fetch_one(&state.pool).await?;
        let _ = apply_blueprint(State(state.clone()), auth.clone(), Path(revision)).await?;
        sqlx::query("UPDATE composition_drift_findings SET disposition='reconciled',resolved_at=now() WHERE finding_id=$1")
            .bind(finding_id).execute(&state.pool).await?;
        return Ok(StatusCode::NO_CONTENT);
    }
    let endpoints = module_control_endpoints()?;
    let base = endpoints.get(definition_id).ok_or_else(|| {
        ApiError::BadRequest("Drift owner has no configured control endpoint".into())
    })?;
    let client = reqwest::Client::new();
    client
        .put(format!("{}/api/configuration", base.trim_end_matches('/')))
        .header("x-tessara-module-control-key", module_control_key()?)
        .json(&desired)
        .send()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?
        .error_for_status()
        .map_err(|error| ApiError::Internal(error.into()))?;
    let observed = read_owner_configuration(&client, base).await?;
    if canonical_digest(&observed).map_err(|error| ApiError::Internal(error.into()))?
        != canonical_digest(&desired).map_err(|error| ApiError::Internal(error.into()))?
    {
        return Err(ApiError::BadRequest(
            "Owner read-back does not match desired configuration".into(),
        ));
    }
    sqlx::query("UPDATE composition_drift_findings SET disposition='reconciled',resolved_at=now() WHERE finding_id=$1")
        .bind(finding_id).execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn emergency_disable(
    State(state): State<AppState>,
    auth: AuthenticatedRequest,
    Path(definition_id): Path<String>,
    Json(request): Json<EmergencyDisableRequestV1>,
) -> ApiResult<Json<Value>> {
    require_global(&auth, "composition:approve")?;
    if request.reason.trim().is_empty() || !(1..=1440).contains(&request.expires_in_minutes) {
        return Err(ApiError::BadRequest(
            "Emergency reason and an expiry from 1 to 1440 minutes are required".into(),
        ));
    }
    let installation_id = installation_id(&state).await?;
    let mut lockfile: ApplicationLockfileV1 = serde_json::from_value(
        sqlx::query_scalar("SELECT document FROM composition_lockfiles WHERE installation_id=$1 ORDER BY blueprint_revision DESC LIMIT 1")
            .bind(installation_id).fetch_optional(&state.pool).await?
            .ok_or_else(|| ApiError::BadRequest("Resolve a composition before using emergency disable".into()))?,
    ).map_err(|error| ApiError::Internal(error.into()))?;
    if !lockfile
        .modules
        .iter()
        .any(|module| module.definition_id == definition_id)
    {
        return Err(ApiError::NotFound(
            "Module is not present in the resolved composition".into(),
        ));
    }
    lockfile.materialization_plan = tessara_composition::MaterializationPlanV1 {
        api_version: PLAN_API_V1.into(),
        installation_id,
        desired_revision: lockfile.blueprint_revision,
        actions: vec![
            MaterializationActionV1::SetEnablement {
                definition_id: definition_id.clone(),
                enabled: false,
            },
            MaterializationActionV1::VerifyReadBack,
        ],
    };
    lockfile.materialization_plan_digest = canonical_digest(&lockfile.materialization_plan)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let supervisor_url = std::env::var("TESSARA_SUPERVISOR_URL")
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Supervisor URL is not configured")))?;
    let client = reqwest::Client::new();
    let current = client
        .get(format!(
            "{}/v1/receipts/current",
            supervisor_url.trim_end_matches('/')
        ))
        .send()
        .await
        .map_err(|_| ApiError::NotFound("Supervisor is unavailable".into()))?
        .error_for_status()
        .map_err(|_| {
            ApiError::BadRequest(
                "Emergency disable requires an existing installation receipt".into(),
            )
        })?
        .json::<InstallationReceiptV1>()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    let now = Utc::now();
    let authorization = ApplyAuthorizationV1 {
        api_version: AUTHORIZATION_API_V1.into(),
        operation: ApplyOperationKindV1::EmergencyDisable,
        installation_id,
        base_receipt_digest: Some(
            canonical_digest(&current).map_err(|error| ApiError::Internal(error.into()))?,
        ),
        target_plan_digest: lockfile.materialization_plan_digest.clone(),
        desired_revision: lockfile.blueprint_revision,
        apply_sequence: current.revision + 1,
        nonce: Uuid::new_v4(),
        idempotency_key: format!("emergency-disable-{definition_id}-{}", Uuid::new_v4()),
        initiator: ActorEvidenceV1 {
            actor_id: auth.account_id.to_string(),
            actor_kind: "account".into(),
            authority: "composition:approve".into(),
        },
        approver: ActorEvidenceV1 {
            actor_id: auth.account_id.to_string(),
            actor_kind: "account".into(),
            authority: "composition:approve".into(),
        },
        issued_at: now,
        expires_at: now + Duration::minutes(i64::from(request.expires_in_minutes)),
        approved_effects: BTreeSet::from([ApprovedEffectV1::Disable]),
        reason: Some(request.reason.trim().into()),
    };
    let signer = apply_authorization_signer()?;
    let signed = signer
        .sign(authorization)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let response = client
        .post(format!("{}/v1/apply", supervisor_url.trim_end_matches('/')))
        .json(&serde_json::json!({"lockfile": lockfile, "authorization": signed}))
        .send()
        .await
        .map_err(|_| ApiError::NotFound("Supervisor is unavailable".into()))?;
    let status = response.status();
    let body: Value = response
        .json()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    if !status.is_success() {
        return Err(ApiError::BadRequest(format!(
            "Supervisor rejected emergency disable: {body}"
        )));
    }
    Ok(Json(body))
}

async fn detect_composition_drift(
    state: &AppState,
    installation_id: Uuid,
    document: &Value,
    blueprint_document: Option<&Value>,
    receipt: Option<&Value>,
) -> ApiResult<()> {
    let lockfile: ApplicationLockfileV1 = serde_json::from_value(document.clone())
        .map_err(|error| ApiError::Internal(error.into()))?;
    let desired_blueprint = blueprint_document
        .map(|value| serde_json::from_value::<ApplicationBlueprintV1>(value.clone()))
        .transpose()
        .map_err(|error| ApiError::Internal(error.into()))?;
    let endpoints = module_control_endpoints()?;
    let client = reqwest::Client::new();
    for module in &lockfile.modules {
        let desired_module = desired_blueprint.as_ref().and_then(|blueprint| {
            blueprint
                .modules
                .iter()
                .find(|desired| desired.definition_id == module.definition_id)
        });
        let desired_configuration =
            desired_module.map_or(&module.configuration, |desired| &desired.configuration);
        let Some(base) = endpoints.get(&module.definition_id) else {
            continue;
        };
        let observed = match read_owner_configuration(&client, base).await {
            Ok(value) => value,
            Err(_) => continue,
        };
        let desired_digest = canonical_digest(desired_configuration)
            .map_err(|error| ApiError::Internal(error.into()))?;
        let observed_digest =
            canonical_digest(&observed).map_err(|error| ApiError::Internal(error.into()))?;
        let path = format!("/modules/{}/configuration", module.definition_id);
        if desired_digest != observed_digest {
            sqlx::query("INSERT INTO composition_drift_findings(installation_id,code,path,desired,observed) SELECT $1,'configuration_drift',$2,$3,$4 WHERE NOT EXISTS (SELECT 1 FROM composition_drift_findings WHERE installation_id=$1 AND path=$2 AND disposition='open')")
                .bind(installation_id).bind(path).bind(desired_configuration).bind(observed)
                .execute(&state.pool).await?;
        } else {
            sqlx::query("UPDATE composition_drift_findings SET disposition='reconciled',resolved_at=now() WHERE installation_id=$1 AND path=$2 AND disposition='open'")
                .bind(installation_id).bind(path).execute(&state.pool).await?;
        }
    }
    if let Some(receipt) = receipt {
        for module in &lockfile.modules {
            let desired_enabled = desired_blueprint
                .as_ref()
                .and_then(|blueprint| {
                    blueprint
                        .modules
                        .iter()
                        .find(|desired| desired.definition_id == module.definition_id)
                })
                .map_or(module.enabled, |desired| desired.enabled);
            let observed = receipt
                .pointer(&format!(
                    "/observed_enablement/{}",
                    module.definition_id.replace('~', "~0").replace('/', "~1")
                ))
                .and_then(Value::as_bool);
            let Some(observed) = observed else { continue };
            let path = format!("/modules/{}/enabled", module.definition_id);
            if desired_enabled != observed {
                sqlx::query("INSERT INTO composition_drift_findings(installation_id,code,path,desired,observed) SELECT $1,'enablement_override',$2,$3,$4 WHERE NOT EXISTS (SELECT 1 FROM composition_drift_findings WHERE installation_id=$1 AND path=$2 AND disposition='open')")
                    .bind(installation_id).bind(path).bind(Value::Bool(desired_enabled)).bind(Value::Bool(observed))
                    .execute(&state.pool).await?;
            } else {
                sqlx::query("UPDATE composition_drift_findings SET disposition='reconciled',resolved_at=now() WHERE installation_id=$1 AND path=$2 AND disposition='open'")
                    .bind(installation_id).bind(path).execute(&state.pool).await?;
            }
        }
    }
    Ok(())
}

async fn read_owner_configuration(client: &reqwest::Client, base: &str) -> ApiResult<Value> {
    let mut value: Value = client
        .get(format!("{}/api/configuration", base.trim_end_matches('/')))
        .send()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?
        .error_for_status()
        .map_err(|error| ApiError::Internal(error.into()))?
        .json()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    if let Some(object) = value.as_object_mut() {
        object.remove("updated_at");
    }
    Ok(value)
}

fn module_control_endpoints() -> ApiResult<BTreeMap<String, String>> {
    let Ok(raw) = std::env::var("TESSARA_MODULE_CONTROL_ENDPOINTS") else {
        return Ok(BTreeMap::new());
    };
    serde_json::from_str(&raw).map_err(|error| ApiError::Internal(error.into()))
}

fn module_control_key() -> ApiResult<String> {
    std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .map_err(|_| ApiError::Internal(anyhow::anyhow!("Module control key is not configured")))
}

fn drift_target(path: &str) -> ApiResult<(&str, &str)> {
    let value = path
        .strip_prefix("/modules/")
        .ok_or_else(|| ApiError::BadRequest("Unsupported drift path".into()))?;
    for dimension in ["configuration", "enabled"] {
        if let Some(definition_id) = value.strip_suffix(&format!("/{dimension}"))
            && !definition_id.is_empty()
        {
            return Ok((definition_id, dimension));
        }
    }
    Err(ApiError::BadRequest("Unsupported drift path".into()))
}

fn findings_error(error: CompositionError) -> ApiError {
    ApiError::BadRequest(
        serde_json::to_string(&error.findings)
            .unwrap_or_else(|_| "Composition resolution failed".into()),
    )
}

fn require_global(auth: &AuthenticatedRequest, capability: &str) -> ApiResult<()> {
    if auth.account.has_global_capability(capability) {
        Ok(())
    } else {
        Err(ApiError::Forbidden(capability.into()))
    }
}

fn require_projection_token(headers: &HeaderMap) -> ApiResult<()> {
    let expected = std::env::var("TESSARA_SUPERVISOR_PROJECTION_TOKEN").map_err(|_| {
        ApiError::Internal(anyhow::anyhow!(
            "Supervisor projection token is not configured"
        ))
    })?;
    if headers
        .get("x-tessara-supervisor-token")
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
    {
        Ok(())
    } else {
        Err(ApiError::Forbidden("supervisor:project".into()))
    }
}

async fn installation_id(state: &AppState) -> ApiResult<Uuid> {
    Ok(
        sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
            .fetch_one(&state.pool)
            .await?,
    )
}

pub(crate) async fn native_page(State(state): State<AppState>, headers: HeaderMap) -> Response {
    match auth::authenticate_request(&state.pool, &state.config, &headers).await {
        Ok((account, _)) if account.has_global_capability("composition:read") => crate::native_app(
            "/administration/composition",
            "Application Composition",
            "Plan, approve, apply, and inspect the installation composition.",
        )
        .into_response(),
        Ok(_) => ApiError::Forbidden("composition:read".into()).into_response(),
        Err(ApiError::Unauthorized | ApiError::SessionExpired | ApiError::SessionRevoked) => {
            Redirect::to("/login").into_response()
        }
        Err(error) => error.into_response(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dataset_bootstrap_manifest_fixture()
    -> (tessara_composition::ResolvedModuleReleaseV1, ModuleManifest) {
        let manifest: ModuleManifest = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tessara-dataset-module/manifest.json"
        )))
        .expect("Dataset manifest");
        let manifest_digest = canonical_digest(&manifest).expect("Dataset manifest digest");
        (
            tessara_composition::ResolvedModuleReleaseV1 {
                definition_id: manifest.definition_id.to_string(),
                version: manifest.release_version.clone(),
                manifest_digest,
                runtime_image: ArtifactDigest::new(format!("sha256:{}", "8".repeat(64)))
                    .expect("runtime digest"),
                deployment_profile: "tessara-oci-v1".into(),
                enabled: true,
                configuration_schema_version: "tessara.io/dataset-configuration/v1".into(),
                configuration: serde_json::json!({}),
                configuration_digest: canonical_digest(&serde_json::json!({}))
                    .expect("configuration digest"),
                bootstrap_schema_version: Some("tessara.io/dataset-bootstrap/v1".into()),
                bootstrap: None,
                bootstrap_digest: None,
                dependency_bindings: BTreeMap::new(),
            },
            manifest,
        )
    }

    #[test]
    fn bootstrap_capability_enrollment_fails_closed_on_unlocked_manifest() {
        let (module, manifest) = dataset_bootstrap_manifest_fixture();
        let exact = BTreeMap::from([(module.definition_id.clone(), manifest.clone())]);
        validate_bootstrap_capability_manifests(std::slice::from_ref(&module), &exact)
            .expect("source-exact manifest");

        let mut substituted = manifest;
        substituted.security_capabilities[0]
            .description
            .push_str(" substituted");
        let substituted = BTreeMap::from([(module.definition_id.clone(), substituted)]);
        assert!(
            validate_bootstrap_capability_manifests(std::slice::from_ref(&module), &substituted)
                .is_err(),
            "an unlocked capability declaration must not be enrolled"
        );
        assert!(
            validate_bootstrap_capability_manifests(
                std::slice::from_ref(&module),
                &BTreeMap::new()
            )
            .is_err(),
            "the exact enabled-module manifest set is mandatory"
        );
    }

    #[sqlx::test(migrations = "./migrations")]
    async fn fresh_baseline_enrolls_dataset_capability_before_core_actor_bootstrap(
        pool: sqlx::PgPool,
    ) {
        let (module, manifest) = dataset_bootstrap_manifest_fixture();
        let manifests = BTreeMap::from([(module.definition_id.clone(), manifest)]);
        validate_bootstrap_capability_manifests(std::slice::from_ref(&module), &manifests)
            .expect("source-exact Dataset manifest");
        assert_eq!(
            sqlx::query_scalar::<_, i64>(
                "SELECT COUNT(*) FROM capabilities WHERE key='datasets:read'"
            )
            .fetch_one(&pool)
            .await
            .expect("fresh capability count"),
            0,
            "fresh Core must not rely on a retired built-in Dataset capability"
        );
        let mut transaction = pool.begin().await.expect("capability transaction");
        insert_bootstrap_capabilities(&mut transaction, &manifests)
            .await
            .expect("enroll source-exact capabilities");
        transaction.commit().await.expect("commit capabilities");

        let installation_id: Uuid =
            sqlx::query_scalar("SELECT id FROM application_installations WHERE singleton=true")
                .fetch_one(&pool)
                .await
                .expect("installation identity");
        let apply_actor_id = Uuid::new_v4();
        sqlx::query(
            "INSERT INTO accounts(id,email,display_name,is_active)
             VALUES($1,'bootstrap-applier@example.test','Bootstrap Applier',true)",
        )
        .bind(apply_actor_id)
        .execute(&pool)
        .await
        .expect("apply actor");
        let input = valid_core_bootstrap_fixture();
        validate_core_bootstrap_input(&input).expect("valid Core fixture");
        let mut transaction = pool.begin().await.expect("Core bootstrap transaction");
        let resources =
            materialize_core_bootstrap(&mut transaction, installation_id, apply_actor_id, &input)
                .await
                .expect("Core actor bootstrap after capability enrollment");
        transaction.commit().await.expect("commit Core bootstrap");
        let schema: tessara_forms_contract::FormVersionSchemaResponse = serde_json::from_str(
            resources
                .get("form.primary/v1.schema")
                .expect("signed owner receipt FormVersion schema"),
        )
        .expect("canonical FormVersion schema receipt value");
        schema
            .validate_for(schema.form_version_id)
            .expect("receipt schema digest and shape");
        assert_eq!(schema.fields.len(), 1);
        assert_eq!(schema.fields[0].key, "amount");
        let assigned: bool = sqlx::query_scalar(
            "SELECT EXISTS(
                 SELECT 1 FROM accounts a
                 JOIN role_assignments ra ON ra.account_id=a.id
                 JOIN role_capabilities rc ON rc.role_id=ra.role_id
                 JOIN capabilities c ON c.id=rc.capability_id
                 WHERE a.email='reader@example.test' AND c.key='datasets:read'
             )",
        )
        .fetch_one(&pool)
        .await
        .expect("Dataset actor capability assignment");
        assert!(assigned);
    }

    #[test]
    fn resolved_bootstrap_input_can_change_only_declared_null_receipt_targets() {
        let locked = serde_json::json!({
            "components": [{"dataset_reference": null, "name": "Reference"}]
        });
        let binding = tessara_composition::BootstrapReceiptBindingV1 {
            target_pointer: "/components/0/dataset_reference".into(),
            source_owner: "tessara.datasets".into(),
            resource_key: "dataset.base".into(),
            value_encoding: tessara_composition::BootstrapReceiptValueEncodingV1::Json,
        };
        let resolved = serde_json::json!({
            "components": [{
                "dataset_reference": {"reference": {"resource_id": "opaque"}},
                "name": "Reference"
            }]
        });
        assert!(resolved_bootstrap_input_matches_lock(
            &locked,
            std::slice::from_ref(&binding),
            &resolved,
        ));

        let mut tampered = resolved;
        tampered["components"][0]["name"] = Value::String("Substituted".into());
        assert!(!resolved_bootstrap_input_matches_lock(
            &locked,
            &[binding],
            &tampered,
        ));
    }

    #[test]
    fn core_bootstrap_rejects_retired_dataset_owned_input() {
        let blueprint: ApplicationBlueprintV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8a/blueprints/reference.json"
        )))
        .expect("valid Sprint 8A Blueprint");
        let tessara_composition::BootstrapInputV1::Inline { value, .. } =
            blueprint.core.bootstrap.expect("Core bootstrap")
        else {
            panic!("Sprint 8A Core bootstrap must be inline");
        };
        assert!(
            serde_json::from_value::<CoreBootstrapV1>(value).is_err(),
            "Core bootstrap must fail closed when a historical Blueprint asks Core to own Dataset state"
        );

        let bootstrap: CoreBootstrapV1 = serde_json::from_value(serde_json::json!({
            "schema_version": "tessara.io/core-bootstrap/v1",
            "root_node_type_external_key": "scope.type",
            "root_node_type_name": "Organization",
            "root_node_external_key": "scope.full",
            "root_node_name": "Reference Organization",
            "additional_nodes": []
        }))
        .expect("current Core-only bootstrap");
        assert!(bootstrap.additional_nodes.is_empty());
    }

    fn valid_core_bootstrap_fixture() -> CoreBootstrapV1 {
        serde_json::from_value(serde_json::json!({
            "schema_version": "tessara.io/core-bootstrap/v1",
            "root_node_type_external_key": "scope.type",
            "root_node_type_name": "Organization",
            "root_node_external_key": "scope.full",
            "root_node_name": "Reference Organization",
            "additional_nodes": [{
                "external_key": "scope.restricted",
                "name": "Restricted Division",
                "parent_node_key": "scope.full"
            }],
            "forms": [{
                "resource_key": "form.primary/v1",
                "source_alias": "primary",
                "name": "Primary Responses",
                "slug": "primary-responses",
                "version_label": "1.0.0",
                "published_at": "2026-08-01T12:00:00Z",
                "scope_node_keys": ["scope.full"],
                "fields": [{
                    "key": "amount",
                    "label": "Amount",
                    "field_type": "number",
                    "required": true,
                    "position": 0,
                    "grid_row": 1,
                    "grid_column": 1
                }]
            }],
            "responses": [{
                "resource_key": "response.initial",
                "form_resource_key": "form.primary/v1",
                "node_key": "scope.restricted",
                "created_at": "2026-08-01T12:01:00Z",
                "submitted_at": "2026-08-01T12:02:00Z",
                "values": {"amount": 17}
            }],
            "actors": [{
                "resource_key": "actor.reader",
                "email": "reader@example.test",
                "display_name": "Dataset Reader",
                "password": "fixture-password-123",
                "capabilities": ["datasets:read"],
                "scope_node_keys": ["scope.restricted"]
            }]
        }))
        .expect("canonical Core bootstrap fixture")
    }

    #[test]
    fn core_bootstrap_accepts_logical_owner_fixture_and_derives_typed_ids() {
        let input = valid_core_bootstrap_fixture();
        validate_core_bootstrap_input(&input).expect("logical fixture must validate");

        let installation_id =
            Uuid::parse_str("11111111-2222-4333-8444-555555555555").expect("installation UUID");
        let first = core_bootstrap_resource_id(installation_id, "form", "form.primary/v1");
        let replay = core_bootstrap_resource_id(installation_id, "form", "form.primary/v1");
        let other_kind =
            core_bootstrap_resource_id(installation_id, "form_version", "form.primary/v1");
        let other_key = core_bootstrap_resource_id(installation_id, "form", "form.secondary/v1");

        assert_eq!(
            first, replay,
            "owner-derived identity must be replay-stable"
        );
        assert_ne!(
            first, other_kind,
            "resource kinds must have separate namespaces"
        );
        assert_ne!(
            first, other_key,
            "logical keys must have separate identities"
        );
        assert_eq!(first.as_bytes()[6] >> 4, 8, "owner IDs use UUID version 8");
        assert_eq!(first.as_bytes()[8] >> 6, 2, "owner IDs use the RFC variant");
    }

    #[test]
    fn core_bootstrap_rejects_physical_ids_and_cross_owner_substitution() {
        let mut raw = serde_json::to_value(valid_core_bootstrap_fixture()).expect("fixture JSON");
        raw["root_node_id"] = Value::String(Uuid::new_v4().to_string());
        assert!(
            serde_json::from_value::<CoreBootstrapV1>(raw).is_err(),
            "Blueprint input must not predict a Core resource UUID"
        );

        let mut unknown_parent = valid_core_bootstrap_fixture();
        unknown_parent.additional_nodes[0].parent_node_key = Some("scope.substituted".into());
        assert!(validate_core_bootstrap_input(&unknown_parent).is_err());

        let mut substituted_form = valid_core_bootstrap_fixture();
        substituted_form.responses[0].form_resource_key = "form.substituted/v1".into();
        assert!(validate_core_bootstrap_input(&substituted_form).is_err());

        let mut substituted_field = valid_core_bootstrap_fixture();
        substituted_field.responses[0]
            .values
            .insert("unknown".into(), Value::String("hidden".into()));
        assert!(validate_core_bootstrap_input(&substituted_field).is_err());

        let mut missing_required_field = valid_core_bootstrap_fixture();
        missing_required_field.responses[0].values.clear();
        assert!(validate_core_bootstrap_input(&missing_required_field).is_err());

        let mut wrong_value_type = valid_core_bootstrap_fixture();
        wrong_value_type.responses[0]
            .values
            .insert("amount".into(), Value::String("seventeen".into()));
        assert!(validate_core_bootstrap_input(&wrong_value_type).is_err());

        let mut duplicate_actor_scope = valid_core_bootstrap_fixture();
        duplicate_actor_scope.actors[0]
            .scope_node_keys
            .push("scope.restricted".into());
        assert!(validate_core_bootstrap_input(&duplicate_actor_scope).is_err());

        let mut overlapping_receipt = valid_core_bootstrap_fixture();
        overlapping_receipt.responses[0].resource_key = "scope.restricted".into();
        assert!(validate_core_bootstrap_input(&overlapping_receipt).is_err());
    }

    #[test]
    fn checked_catalog_manifest_digests_match_runtime_manifests() {
        let catalog: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/catalogs/local-release-catalog.json"
        )))
        .expect("valid Sprint 8B catalog");
        let manifests = [
            serde_json::from_str::<ModuleManifest>(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../tessara-dataset-module/manifest.json"
            )))
            .expect("valid Dataset manifest"),
            serde_json::from_str::<ModuleManifest>(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../tessara-component-module/manifest.json"
            )))
            .expect("valid Component manifest"),
            serde_json::from_str::<ModuleManifest>(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../tessara-dashboard-module/manifest.json"
            )))
            .expect("valid Dashboard manifest"),
            serde_json::from_str::<ModuleManifest>(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../tessara-reference-scoped-records/manifest.json"
            )))
            .expect("valid Scoped Records manifest"),
        ];
        for manifest in manifests {
            let release = catalog
                .module_releases
                .iter()
                .find(|release| release.definition_id == manifest.definition_id.as_str())
                .expect("runtime manifest must have a catalog release");
            assert_eq!(
                canonical_digest(&manifest).expect("manifest digest"),
                release.manifest_digest,
                "{} manifest digest must be catalog-bound",
                manifest.definition_id
            );
        }
    }

    #[test]
    fn component_release_identity_is_forward_only_across_sprint_catalogs() {
        let sprint_8a: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8a/catalogs/local-release-catalog.json"
        )))
        .expect("frozen Sprint 8A catalog");
        let sprint_8b: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-8b/catalogs/local-release-catalog.json"
        )))
        .expect("Sprint 8B catalog");
        let release_8a = sprint_8a
            .module_releases
            .iter()
            .find(|release| release.definition_id == "tessara.components")
            .expect("Sprint 8A Component release");
        let release_8b = sprint_8b
            .module_releases
            .iter()
            .find(|release| release.definition_id == "tessara.components")
            .expect("Sprint 8B Component release");
        assert_eq!(release_8a.version, semver::Version::new(1, 0, 1));
        assert_eq!(
            release_8a.manifest_digest.as_str(),
            "sha256:59a78aa01356c5119cc23801b85ba47463940b6bd4237c528ecac7f4824f9d48"
        );
        assert_eq!(release_8b.version, semver::Version::new(1, 1, 0));
        assert_ne!(release_8a.manifest_digest, release_8b.manifest_digest);
    }

    #[test]
    fn emergency_receipt_accepts_only_the_exact_derived_disable_lockfile() {
        let blueprint: ApplicationBlueprintV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-6f/blueprints/reference.json"
        )))
        .expect("valid Sprint 6F Blueprint");
        let catalog: ReleaseCatalogV1 = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../deploy/sprint-6f/catalogs/local-release-catalog.json"
        )))
        .expect("valid Sprint 6F catalog");
        let resolved = tessara_composition::resolve(&blueprint, &catalog)
            .expect("reference composition resolves");
        let definition_id = "tessara.reference.scoped-records".to_string();
        let mut emergency = resolved.clone();
        emergency.materialization_plan = tessara_composition::MaterializationPlanV1 {
            api_version: PLAN_API_V1.into(),
            installation_id: resolved.installation_id,
            desired_revision: resolved.blueprint_revision,
            actions: vec![
                MaterializationActionV1::SetEnablement {
                    definition_id: definition_id.clone(),
                    enabled: false,
                },
                MaterializationActionV1::VerifyReadBack,
            ],
        };
        emergency.materialization_plan_digest =
            canonical_digest(&emergency.materialization_plan).expect("emergency plan digest");
        let desired_enablement = resolved
            .modules
            .iter()
            .map(|module| (module.definition_id.clone(), module.enabled))
            .collect::<BTreeMap<_, _>>();
        let mut observed_enablement = desired_enablement.clone();
        observed_enablement.insert(definition_id.clone(), false);
        let receipt = InstallationReceiptV1 {
            api_version: "tessara.io/installation-receipt/v1".into(),
            installation_id: resolved.installation_id,
            revision: 2,
            lockfile_digest: canonical_digest(&emergency).expect("emergency lockfile digest"),
            plan_digest: emergency.materialization_plan_digest.clone(),
            authorization_digest: canonical_digest(&"authorization").expect("authorization digest"),
            composition_engine_version: resolved.composition_engine_version.clone(),
            supervisor_version: resolved.supervisor_contract_version.clone(),
            deployment_adapter_version: resolved.deployment_adapter_version.clone(),
            desired_enablement,
            observed_enablement,
            observed_artifacts: BTreeMap::new(),
            configuration_digests: BTreeMap::new(),
            bootstrap_receipts: Vec::new(),
            applied_at: Utc::now(),
            previous_receipt_digest: None,
            no_op: false,
        };

        assert!(
            is_constrained_emergency_lockfile(&resolved, &emergency, &receipt)
                .expect("valid emergency check")
        );

        let mut tampered = emergency.clone();
        tampered.navigation.clear();
        assert!(
            !is_constrained_emergency_lockfile(&resolved, &tampered, &receipt)
                .expect("tampered emergency check")
        );
        let mut wrong_observation = receipt.clone();
        wrong_observation
            .observed_enablement
            .insert(definition_id, true);
        assert!(
            !is_constrained_emergency_lockfile(&resolved, &emergency, &wrong_observation)
                .expect("wrong observation check")
        );
    }
}
