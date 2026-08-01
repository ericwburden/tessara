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
use sqlx::Row;
use tessara_composition::{
    AUTHORIZATION_API_V1, ActorEvidenceV1, ApplicationBlueprintV1, ApplicationLockfileV1,
    ApplyAuthorizationV1, ApplyOperationKindV1, ApprovedEffectV1, CompositionError,
    CompositionOperationV1, InstallationReceiptV1, MaterializationActionV1, PLAN_API_V1,
    ReleaseCatalogV1, canonical_digest, required_effects, resolve,
};
use tessara_module_contract::{ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1};
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
            "/api/internal/composition/bootstrap/core",
            post(apply_core_bootstrap),
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
    receipt: InstallationReceiptV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapV1 {
    schema_version: String,
    root_node_type_id: Uuid,
    root_node_type_external_key: String,
    root_node_type_name: String,
    root_node_id: Uuid,
    root_node_external_key: String,
    root_node_name: String,
    dataset_id: Uuid,
    dataset_external_key: String,
    components: Vec<CoreBootstrapComponentV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CoreBootstrapComponentV1 {
    external_key: String,
    component_id: Uuid,
    component_version_id: Uuid,
    name: String,
    slug: String,
    component_type: String,
    config: Value,
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
    if latest_receipt.is_none() {
        if let Ok(supervisor_url) = std::env::var("TESSARA_SUPERVISOR_URL") {
            if let Ok(response) = reqwest::Client::new()
                .get(format!(
                    "{}/v1/receipts/current",
                    supervisor_url.trim_end_matches('/')
                ))
                .send()
                .await
            {
                if response.status().is_success() {
                    latest_receipt = response.json::<Value>().await.ok();
                }
            }
        }
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
    let lockfile = resolve(&blueprint, &request.catalog).map_err(findings_error)?;
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
    let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        std::env::var("TESSARA_COMPOSITION_APPLY_SIGNING_ISSUER")
            .unwrap_or_else(|_| "tessara.local.sprint-6f".into()),
        std::env::var("TESSARA_COMPOSITION_APPLY_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "apply-dev-v1".into()),
        ProtocolSignaturePurposeV1::ApplyAuthorization,
        decode_secret_hex("TESSARA_COMPOSITION_APPLY_SIGNING_SECRET_HEX")?,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
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
    let digest = canonical_digest(&request.receipt)
        .map_err(|error| ApiError::Internal(error.into()))?
        .to_string();
    let value =
        serde_json::to_value(&request.receipt).map_err(|error| ApiError::Internal(error.into()))?;
    sqlx::query("INSERT INTO composition_receipt_projections(installation_id,revision,digest,receipt) VALUES($1,$2,$3,$4) ON CONFLICT(installation_id,revision) DO UPDATE SET digest=EXCLUDED.digest,receipt=EXCLUDED.receipt,observed_at=now()")
        .bind(request.receipt.installation_id).bind(request.receipt.revision as i64).bind(digest).bind(value)
        .execute(&state.pool).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn apply_core_bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<tessara_composition::OwnerBootstrapRequestV1<CoreBootstrapV1>>,
) -> ApiResult<Json<tessara_composition::OwnerBootstrapResponseV1>> {
    require_projection_token(&headers)?;
    if request.input.schema_version != "tessara.io/core-bootstrap/v1"
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|error| ApiError::Internal(error.into()))?
    {
        return Err(ApiError::BadRequest(
            "Core bootstrap contract or digest is invalid".into(),
        ));
    }
    if let Some((digest, receipt)) = sqlx::query_as::<_, (String, Value)>(
        "SELECT input_digest,receipt FROM core_bootstrap_receipts WHERE idempotency_key=$1",
    )
    .bind(&request.idempotency_key)
    .fetch_optional(&state.pool)
    .await?
    {
        if digest != request.input_digest.to_string() {
            return Err(ApiError::BadRequest(
                "Core bootstrap idempotency key was reused with different input".into(),
            ));
        }
        let mut response: tessara_composition::OwnerBootstrapResponseV1 =
            serde_json::from_value(receipt).map_err(|error| ApiError::Internal(error.into()))?;
        response.receipt.changed = false;
        return Ok(Json(response));
    }
    let installation_id = installation_id(&state).await?;
    if installation_id != request.installation_id
        || request.input.root_node_type_name.trim().is_empty()
        || request.input.root_node_name.trim().is_empty()
        || request.input.components.iter().any(|component| {
            component.external_key.trim().is_empty()
                || component.name.trim().is_empty()
                || !matches!(
                    component.component_type.as_str(),
                    "table" | "bar" | "line" | "pie" | "donut" | "stat_card"
                )
        })
    {
        return Err(ApiError::BadRequest(
            "Core bootstrap input is invalid".into(),
        ));
    }
    let mut transaction = state.pool.begin().await?;
    sqlx::query("INSERT INTO node_types(id,name,slug,plural_label,description) VALUES($1,$2,$3,$4,$5) ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,plural_label=EXCLUDED.plural_label,description=EXCLUDED.description")
        .bind(request.input.root_node_type_id).bind(request.input.root_node_type_name.trim())
        .bind(&request.input.root_node_type_external_key).bind("Organizations").bind("Application composition bootstrap root")
        .execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO nodes(id,node_type_id,parent_node_id,name) VALUES($1,$2,NULL,$3) ON CONFLICT(id) DO UPDATE SET node_type_id=EXCLUDED.node_type_id,name=EXCLUDED.name")
        .bind(request.input.root_node_id).bind(request.input.root_node_type_id).bind(request.input.root_node_name.trim())
        .execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,$2,$3,'node') ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug")
        .bind(request.input.dataset_id).bind("Composition Bootstrap Dataset").bind(&request.input.dataset_external_key)
        .execute(&mut *transaction).await?;
    sqlx::query(
        "INSERT INTO dataset_scope_nodes(dataset_id,node_id) VALUES($1,$2) ON CONFLICT DO NOTHING",
    )
    .bind(request.input.dataset_id)
    .bind(request.input.root_node_id)
    .execute(&mut *transaction)
    .await?;
    for component in &request.input.components {
        sqlx::query("INSERT INTO components(id,name,slug,description) VALUES($1,$2,$3,$4) ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,description=EXCLUDED.description")
            .bind(component.component_id).bind(component.name.trim()).bind(&component.slug)
            .bind("Application composition bootstrap fixture").execute(&mut *transaction).await?;
        sqlx::query("INSERT INTO component_versions(id,component_id,dataset_id,dataset_version_major,binding_mode,component_type,version_number,version_label,version_note,status,config,published_at) VALUES($1,$2,$3,1,'major_line',$4::component_type,1,'1.0.0','Application composition bootstrap','published',$5,now()) ON CONFLICT(id) DO UPDATE SET component_id=EXCLUDED.component_id,dataset_id=EXCLUDED.dataset_id,component_type=EXCLUDED.component_type,config=EXCLUDED.config")
            .bind(component.component_version_id).bind(component.component_id).bind(request.input.dataset_id)
            .bind(&component.component_type).bind(&component.config).execute(&mut *transaction).await?;
    }
    let mut resource_ids = std::collections::BTreeMap::from([
        (
            request.input.root_node_external_key.clone(),
            request.input.root_node_id.to_string(),
        ),
        (
            request.input.dataset_external_key.clone(),
            request.input.dataset_id.to_string(),
        ),
    ]);
    resource_ids.extend(request.input.components.iter().map(|component| {
        (
            component.external_key.clone(),
            component.component_version_id.to_string(),
        )
    }));
    let result_digest =
        canonical_digest(&resource_ids).map_err(|error| ApiError::Internal(error.into()))?;
    let response = tessara_composition::OwnerBootstrapResponseV1 {
        receipt: tessara_composition::BootstrapReceiptV1 {
            owner: "core".into(),
            schema_version: request.input.schema_version.clone(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids,
        },
    };
    sqlx::query("INSERT INTO core_bootstrap_receipts(idempotency_key,input_digest,desired_revision,receipt) VALUES($1,$2,$3,$4)")
        .bind(&request.idempotency_key).bind(request.input_digest.to_string()).bind(request.desired_revision as i64)
        .bind(serde_json::to_value(&response).map_err(|error| ApiError::Internal(error.into()))?)
        .execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(response))
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
    let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
        std::env::var("TESSARA_COMPOSITION_APPLY_SIGNING_ISSUER")
            .unwrap_or_else(|_| "tessara.local.sprint-6f".into()),
        std::env::var("TESSARA_COMPOSITION_APPLY_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "apply-dev-v1".into()),
        ProtocolSignaturePurposeV1::ApplyAuthorization,
        decode_secret_hex("TESSARA_COMPOSITION_APPLY_SIGNING_SECRET_HEX")?,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
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
        if let Some(definition_id) = value.strip_suffix(&format!("/{dimension}")) {
            if !definition_id.is_empty() {
                return Ok((definition_id, dimension));
            }
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
