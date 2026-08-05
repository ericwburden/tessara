//! Dashboard-owned dependency observations, findings, and manager refresh.

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_components_contract::{
    COMPONENT_CONTRACT_ID, COMPONENT_CONTRACT_VERSION, ComponentResolutionResponse,
    ComponentVersionReference,
};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV2, ContractCompatibilityState,
    ProviderAvailabilityState, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceRevision, SignedEnvelopeV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY,
    composition::{authorization_header, load_dashboard_scope, resolve_component_since},
    product::{authorize, authorized_organizations},
};

#[derive(Clone, Debug, Serialize)]
pub struct DependencyHealthResponse {
    pub dashboard_id: Uuid,
    pub health: &'static str,
    pub open_count: i64,
    pub deferred_count: i64,
    pub findings: Vec<DependencyFindingResponse>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DependencyFindingResponse {
    pub id: Uuid,
    pub placement_id: Uuid,
    pub finding_code: String,
    pub disposition: String,
    pub finding_revision: i64,
    pub observed_resource_revision: i64,
    pub saved_reference: Value,
    pub observed_lifecycle: Option<String>,
    pub publication_state: Option<String>,
    pub change_categories: Vec<String>,
    pub successor_available: bool,
    pub impact: Value,
    pub observed_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyAction {
    Defer,
    Upgrade,
    Replace,
    Remove,
}

impl DependencyAction {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Defer => "defer",
            Self::Upgrade => "upgrade",
            Self::Replace => "replace",
            Self::Remove => "remove",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyActionRequest {
    pub action: DependencyAction,
    pub expected_finding_revision: i64,
    pub replacement_component_version_id: Option<Uuid>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DependencyActionResponse {
    pub dashboard_id: Uuid,
    pub finding_id: Uuid,
    pub placement_id: Uuid,
    pub action: String,
    pub disposition: String,
    pub finding_revision: i64,
}

struct PlacementToRefresh {
    id: Uuid,
    position: i32,
    reference: TypedResourceReference,
    config: Value,
}

pub(super) fn routes() -> Router<DashboardModuleState> {
    Router::new()
        .route(
            "/api/admin/dashboards/{dashboard_id}/dependencies",
            get(read_dependencies),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}/dependencies/refresh",
            post(refresh_dependencies),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}/dependencies/{finding_id}/actions",
            post(act_on_dependency),
        )
}

async fn read_dependencies(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DependencyHealthResponse>, DashboardModuleError> {
    require_manager_scope(
        &state,
        &headers,
        dashboard_id,
        "dashboards.read_dependencies",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    Ok(Json(load_health(&state, dashboard_id).await?))
}

async fn refresh_dependencies(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DependencyHealthResponse>, DashboardModuleError> {
    require_manager_scope(
        &state,
        &headers,
        dashboard_id,
        "dashboards.refresh_dependencies",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let health = refresh_for_editor(&state, &headers, dashboard_id).await?;
    Ok(Json(health))
}

pub(super) async fn refresh_for_editor(
    state: &DashboardModuleState,
    headers: &HeaderMap,
    dashboard_id: Uuid,
) -> Result<DependencyHealthResponse, DashboardModuleError> {
    let authorization = authorization_header(headers)?;
    let correlation_id = correlation_id(headers);
    let placements = load_placements(state, dashboard_id).await?;
    for placement in placements {
        refresh_placement(
            state,
            authorization,
            dashboard_id,
            placement,
            correlation_id,
        )
        .await?;
    }
    load_health(state, dashboard_id).await
}

async fn act_on_dependency(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path((dashboard_id, finding_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<DependencyActionRequest>,
) -> Result<Json<DependencyActionResponse>, DashboardModuleError> {
    validate_action_request(&request)?;
    let grant = require_manager_scope(
        &state,
        &headers,
        dashboard_id,
        "dashboards.act_on_dependency",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let authorization = authorization_header(&headers)?;
    let correlation_id = correlation_id(&headers);
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let request_digest = digest_json(&json!({
        "dashboard_id": dashboard_id,
        "finding_id": finding_id,
        "request": request,
    }))?;
    if let Some(existing) = sqlx::query(
        "SELECT request_digest,result FROM dashboard_dependency_action_receipts
         WHERE idempotency_key=$1",
    )
    .bind(idempotency_key)
    .fetch_optional(&state.pool)
    .await?
    {
        if existing.try_get::<String, _>("request_digest")? != request_digest {
            return Err(DashboardModuleError::Conflict(
                "dependency action idempotency key was reused for different input".into(),
            ));
        }
        let result = serde_json::from_value(existing.try_get("result")?).map_err(|_| {
            DashboardModuleError::Unavailable("stored dependency action result is invalid".into())
        })?;
        tracing::info!(
            correlation_id,
            actor_class = "authorized_dashboard_manager",
            reference_digest = "receipt_replay",
            prior_revision = request.expected_finding_revision,
            current_revision = request.expected_finding_revision + 1,
            action = request.action.as_str(),
            result_code = "idempotent_replay",
            provider_contract_id = COMPONENT_CONTRACT_ID,
            provider_contract_version = COMPONENT_CONTRACT_VERSION,
            "dashboard dependency action"
        );
        return Ok(Json(result));
    }
    let replacement =
        proposed_reference(&state, authorization, dashboard_id, finding_id, &request).await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(idempotency_key)
        .execute(&mut *tx)
        .await?;
    if let Some(existing) = sqlx::query(
        "SELECT request_digest,result FROM dashboard_dependency_action_receipts
         WHERE idempotency_key=$1 FOR UPDATE",
    )
    .bind(idempotency_key)
    .fetch_optional(&mut *tx)
    .await?
    {
        if existing.try_get::<String, _>("request_digest")? != request_digest {
            return Err(DashboardModuleError::Conflict(
                "dependency action idempotency key was reused for different input".into(),
            ));
        }
        let result = serde_json::from_value(existing.try_get("result")?).map_err(|_| {
            DashboardModuleError::Unavailable("stored dependency action result is invalid".into())
        })?;
        tx.commit().await?;
        return Ok(Json(result));
    }

    let finding = sqlx::query(
        "SELECT placement_id,reference_digest,disposition,finding_revision
         FROM dashboard_dependency_findings
         WHERE id=$1 AND dashboard_id=$2 FOR UPDATE",
    )
    .bind(finding_id)
    .bind(dashboard_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| DashboardModuleError::NotFound("dependency finding not found".into()))?;
    let placement_id: Uuid = finding.try_get("placement_id")?;
    let disposition: String = finding.try_get("disposition")?;
    let finding_revision: i64 = finding.try_get("finding_revision")?;
    if disposition == "resolved" || finding_revision != request.expected_finding_revision {
        return Err(DashboardModuleError::Conflict(
            "dependency finding is no longer actionable at the expected revision".into(),
        ));
    }
    if matches!(request.action, DependencyAction::Defer) && disposition != "open" {
        return Err(DashboardModuleError::Conflict(
            "only an open dependency finding can be deferred".into(),
        ));
    }

    let placement = sqlx::query(
        "SELECT component_reference FROM dashboard_placements
         WHERE id=$1 AND dashboard_id=$2 FOR UPDATE",
    )
    .bind(placement_id)
    .bind(dashboard_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| DashboardModuleError::Conflict("placement no longer exists".into()))?;
    let current_reference: TypedResourceReference =
        serde_json::from_value(placement.try_get("component_reference")?).map_err(|_| {
            DashboardModuleError::Conflict("stored Component reference is invalid".into())
        })?;
    let reference_digest: String = finding.try_get("reference_digest")?;
    if digest_json(&current_reference)? != reference_digest {
        return Err(DashboardModuleError::Conflict(
            "placement reference changed after the dependency finding was observed".into(),
        ));
    }

    match request.action {
        DependencyAction::Defer => {
            sqlx::query(
                "UPDATE dashboard_dependency_findings
                 SET disposition='deferred',deferred_by=$2,deferred_at=now(),
                     updated_at=now(),finding_revision=finding_revision+1
                 WHERE id=$1",
            )
            .bind(finding_id)
            .bind(grant.payload.original_actor_id)
            .execute(&mut *tx)
            .await?;
        }
        DependencyAction::Upgrade | DependencyAction::Replace => {
            let replacement = replacement.as_ref().ok_or_else(|| {
                DashboardModuleError::Conflict("validated replacement reference is missing".into())
            })?;
            if replacement == &current_reference {
                return Err(DashboardModuleError::Conflict(
                    "replacement must differ from the current Component reference".into(),
                ));
            }
            sqlx::query(
                "UPDATE dashboard_placements
                 SET component_reference=$2,updated_at=now() WHERE id=$1",
            )
            .bind(placement_id)
            .bind(serde_json::to_value(replacement).map_err(|_| {
                DashboardModuleError::BadRequest("replacement reference is invalid".into())
            })?)
            .execute(&mut *tx)
            .await?;
            resolve_finding(&mut tx, finding_id).await?;
        }
        DependencyAction::Remove => {
            sqlx::query("DELETE FROM dashboard_placements WHERE id=$1")
                .bind(placement_id)
                .execute(&mut *tx)
                .await?;
            resolve_finding(&mut tx, finding_id).await?;
        }
    }

    let result = DependencyActionResponse {
        dashboard_id,
        finding_id,
        placement_id,
        action: request.action.as_str().into(),
        disposition: if matches!(request.action, DependencyAction::Defer) {
            "deferred"
        } else {
            "resolved"
        }
        .into(),
        finding_revision: finding_revision + 1,
    };
    sqlx::query(
        "INSERT INTO dashboard_dependency_action_receipts
         (idempotency_key,request_digest,dashboard_id,placement_id,finding_id,
          actor_id,action,expected_finding_revision,result)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
    )
    .bind(idempotency_key)
    .bind(request_digest)
    .bind(dashboard_id)
    .bind(placement_id)
    .bind(finding_id)
    .bind(grant.payload.original_actor_id)
    .bind(request.action.as_str())
    .bind(request.expected_finding_revision)
    .bind(serde_json::to_value(&result).map_err(|_| {
        DashboardModuleError::Unavailable("dependency action result encoding failed".into())
    })?)
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    tracing::info!(
        correlation_id,
        actor_class = "authorized_dashboard_manager",
        reference_digest,
        prior_revision = request.expected_finding_revision,
        current_revision = result.finding_revision,
        action = request.action.as_str(),
        result_code = result.disposition,
        provider_contract_id = COMPONENT_CONTRACT_ID,
        provider_contract_version = COMPONENT_CONTRACT_VERSION,
        "dashboard dependency action"
    );
    Ok(Json(result))
}

fn validate_action_request(request: &DependencyActionRequest) -> Result<(), DashboardModuleError> {
    if request.expected_finding_revision <= 0 {
        return Err(DashboardModuleError::BadRequest(
            "expected finding revision must be positive".into(),
        ));
    }
    match (
        request.action,
        request.replacement_component_version_id.is_some(),
    ) {
        (DependencyAction::Replace, true)
        | (DependencyAction::Defer | DependencyAction::Upgrade | DependencyAction::Remove, false) => {
            Ok(())
        }
        (DependencyAction::Replace, false) => Err(DashboardModuleError::BadRequest(
            "replace requires a replacement Component reference".into(),
        )),
        _ => Err(DashboardModuleError::BadRequest(
            "replacement reference is only valid for replace".into(),
        )),
    }
}

async fn proposed_reference(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
    finding_id: Uuid,
    request: &DependencyActionRequest,
) -> Result<Option<TypedResourceReference>, DashboardModuleError> {
    let reference = match request.action {
        DependencyAction::Upgrade => {
            let provider_detail = sqlx::query_scalar::<_, Option<Value>>(
                "SELECT observations.provider_detail
                 FROM dashboard_dependency_findings findings
                 JOIN dashboard_dependency_observations observations
                   ON observations.id=findings.observation_id
                 WHERE findings.id=$1 AND findings.dashboard_id=$2",
            )
            .bind(finding_id)
            .bind(dashboard_id)
            .fetch_optional(&state.pool)
            .await?
            .flatten()
            .ok_or_else(|| {
                DashboardModuleError::Conflict(
                    "finding has no disclosed provider-declared successor".into(),
                )
            })?;
            let observation: ComponentResolutionResponse = serde_json::from_value(provider_detail)
                .map_err(|_| {
                    DashboardModuleError::Conflict("stored Component observation is invalid".into())
                })?;
            observation
                .successor()
                .map(|successor| successor.reference.reference().clone())
                .ok_or_else(|| {
                    DashboardModuleError::Conflict(
                        "provider did not declare a successor for this Component version".into(),
                    )
                })?
        }
        DependencyAction::Replace => {
            let version_id = request.replacement_component_version_id.ok_or_else(|| {
                DashboardModuleError::BadRequest(
                    "replace requires a replacement Component version".into(),
                )
            })?;
            let saved_reference = sqlx::query_scalar::<_, Value>(
                "SELECT saved_reference FROM dashboard_dependency_findings
                 WHERE id=$1 AND dashboard_id=$2",
            )
            .bind(finding_id)
            .bind(dashboard_id)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| DashboardModuleError::NotFound("dependency finding not found".into()))?;
            let saved_reference: TypedResourceReference = serde_json::from_value(saved_reference)
                .map_err(|_| {
                DashboardModuleError::Conflict(
                    "stored dependency finding reference is invalid".into(),
                )
            })?;
            TypedResourceReference::new(
                saved_reference.installation_id(),
                saved_reference.owner().clone(),
                saved_reference.resource_type().clone(),
                version_id.to_string(),
            )
            .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?
        }
        DependencyAction::Defer | DependencyAction::Remove => return Ok(None),
    };
    let wrapped = ComponentVersionReference::new(reference.clone())
        .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
    let response = resolve_component_since(state, authorization, wrapped, None).await?;
    if !response
        .metadata()
        .is_some_and(|metadata| metadata.renderable())
    {
        return Err(DashboardModuleError::Conflict(
            "replacement Component version is not currently renderable".into(),
        ));
    }
    Ok(Some(reference))
}

async fn resolve_finding(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    finding_id: Uuid,
) -> Result<(), DashboardModuleError> {
    sqlx::query(
        "UPDATE dashboard_dependency_findings
         SET disposition='resolved',resolved_at=now(),updated_at=now(),
             finding_revision=finding_revision+1 WHERE id=$1",
    )
    .bind(finding_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn mutation_idempotency_key(headers: &HeaderMap) -> Result<&str, DashboardModuleError> {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .ok_or_else(|| DashboardModuleError::BadRequest("missing X-Idempotency-Key".into()))
}

async fn require_manager_scope(
    state: &DashboardModuleState,
    headers: &HeaderMap,
    dashboard_id: Uuid,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV2>, DashboardModuleError> {
    let grant = authorize(state, headers, action, operation).await?;
    let manage_scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    let dashboard_scope = load_dashboard_scope(state, dashboard_id).await?;
    if dashboard_scope.is_empty()
        || !dashboard_scope
            .iter()
            .all(|node_id| manage_scope.contains(node_id))
    {
        return Err(DashboardModuleError::Forbidden);
    }
    Ok(grant)
}

async fn load_placements(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
) -> Result<Vec<PlacementToRefresh>, DashboardModuleError> {
    let rows = sqlx::query(
        "SELECT id,position,component_reference,config
         FROM dashboard_placements WHERE dashboard_id=$1 ORDER BY position,id",
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let reference =
                serde_json::from_value(row.try_get("component_reference")?).map_err(|_| {
                    DashboardModuleError::Conflict("stored Component reference is invalid".into())
                })?;
            Ok(PlacementToRefresh {
                id: row.try_get("id")?,
                position: row.try_get("position")?,
                reference,
                config: row.try_get("config")?,
            })
        })
        .collect()
}

async fn refresh_placement(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
    placement: PlacementToRefresh,
    correlation_id: &str,
) -> Result<(), DashboardModuleError> {
    let reference_digest = digest_json(&placement.reference)?;
    let prior_revision = sqlx::query_scalar::<_, Option<i64>>(
        "SELECT MAX(resource_revision) FROM dashboard_dependency_observations
         WHERE placement_id=$1 AND reference_digest=$2",
    )
    .bind(placement.id)
    .bind(&reference_digest)
    .fetch_one(&state.pool)
    .await?
    .map(|revision| ResourceRevision::new(revision as u64))
    .transpose()
    .map_err(|_| DashboardModuleError::Conflict("stored observation revision is invalid".into()))?;
    let wrapped = ComponentVersionReference::new(placement.reference.clone())
        .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?;
    let response = resolve_component_since(state, authorization, wrapped, prior_revision).await?;
    let response_json = serde_json::to_value(&response).map_err(|_| {
        DashboardModuleError::Unavailable("Component observation encoding failed".into())
    })?;
    let observation_fingerprint = digest_json(&response_json)?;
    let resource_revision = response
        .observation()
        .map(|observation| observation.resource_revision().get() as i64);
    let provider_detail = (response.resolution().access_state() == ResourceAccessState::Authorized)
        .then_some(response_json.clone());
    let observation_id = Uuid::new_v4();
    let inserted = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO dashboard_dependency_observations
         (id,dashboard_id,placement_id,saved_reference,reference_digest,
          provider_contract_id,provider_contract_version,resource_revision,
          observation_fingerprint,resolution,provider_detail)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
         ON CONFLICT(placement_id,observation_fingerprint) DO NOTHING
         RETURNING id",
    )
    .bind(observation_id)
    .bind(dashboard_id)
    .bind(placement.id)
    .bind(serde_json::to_value(&placement.reference).map_err(|_| {
        DashboardModuleError::Conflict("stored Component reference is invalid".into())
    })?)
    .bind(&reference_digest)
    .bind(COMPONENT_CONTRACT_ID)
    .bind(COMPONENT_CONTRACT_VERSION)
    .bind(resource_revision)
    .bind(&observation_fingerprint)
    .bind(serde_json::to_value(response.resolution()).map_err(|_| {
        DashboardModuleError::Unavailable("Component resolution encoding failed".into())
    })?)
    .bind(provider_detail)
    .fetch_optional(&state.pool)
    .await?;
    let observation_id = if let Some(id) = inserted {
        id
    } else {
        sqlx::query_scalar(
            "SELECT id FROM dashboard_dependency_observations
             WHERE placement_id=$1 AND observation_fingerprint=$2",
        )
        .bind(placement.id)
        .bind(&observation_fingerprint)
        .fetch_one(&state.pool)
        .await?
    };

    let finding = classify_finding(&response, &placement);
    let result_code = finding.as_ref().map_or("healthy", |(code, _)| *code);
    if let Some((finding_code, impact)) = finding {
        sqlx::query(
            "UPDATE dashboard_dependency_findings
             SET disposition='resolved',resolved_at=now(),updated_at=now(),
                 finding_revision=finding_revision+1
             WHERE placement_id=$1 AND reference_digest=$2
               AND disposition IN ('open','deferred')
               AND (observed_resource_revision IS DISTINCT FROM $3
                    OR finding_code IS DISTINCT FROM $4)",
        )
        .bind(placement.id)
        .bind(&reference_digest)
        .bind(resource_revision.unwrap_or(0))
        .bind(finding_code)
        .execute(&state.pool)
        .await?;
        sqlx::query(
            "INSERT INTO dashboard_dependency_findings
             (id,dashboard_id,placement_id,observation_id,saved_reference,reference_digest,
              observed_resource_revision,finding_code,impact)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)
             ON CONFLICT(placement_id,reference_digest,observed_resource_revision,finding_code)
             DO NOTHING",
        )
        .bind(Uuid::new_v4())
        .bind(dashboard_id)
        .bind(placement.id)
        .bind(observation_id)
        .bind(serde_json::to_value(&placement.reference).map_err(|_| {
            DashboardModuleError::Conflict("stored Component reference is invalid".into())
        })?)
        .bind(&reference_digest)
        .bind(resource_revision.unwrap_or(0))
        .bind(finding_code)
        .bind(impact)
        .execute(&state.pool)
        .await?;
    } else {
        sqlx::query(
            "UPDATE dashboard_dependency_findings
             SET disposition='resolved',resolved_at=now(),updated_at=now(),
                 finding_revision=finding_revision+1
             WHERE placement_id=$1 AND reference_digest=$2
               AND disposition IN ('open','deferred')",
        )
        .bind(placement.id)
        .bind(&reference_digest)
        .execute(&state.pool)
        .await?;
    }
    tracing::info!(
        correlation_id,
        actor_class = "authorized_dashboard_manager",
        reference_digest,
        prior_revision = prior_revision.map(|revision| revision.get()),
        current_revision = resource_revision,
        action = "refresh",
        result_code,
        provider_contract_id = COMPONENT_CONTRACT_ID,
        provider_contract_version = COMPONENT_CONTRACT_VERSION,
        "dashboard dependency refresh"
    );
    Ok(())
}

fn correlation_id(headers: &HeaderMap) -> &str {
    headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("unavailable")
}

fn classify_finding(
    response: &ComponentResolutionResponse,
    placement: &PlacementToRefresh,
) -> Option<(&'static str, Value)> {
    let resolution = response.resolution();
    let code = if resolution.access_state() != ResourceAccessState::Authorized {
        "restricted"
    } else if resolution.availability_state() != ProviderAvailabilityState::Available {
        "provider_unavailable"
    } else if resolution.compatibility_state() != ContractCompatibilityState::Compatible {
        "contract_incompatible"
    } else if resolution.resource_identity_state() != ResourceIdentityState::Resolved {
        "resource_unresolved"
    } else if matches!(
        resolution.resource_lifecycle_state(),
        ResourceLifecycleState::ProviderDefined { state } if state != "active"
    ) {
        "lifecycle_unrenderable"
    } else if response
        .metadata()
        .is_some_and(|metadata| !metadata.renderable())
    {
        "publication_unrenderable"
    } else if !response.changes().is_empty() {
        "resource_changed"
    } else {
        return None;
    };
    Some((
        code,
        json!({
            "placement_id": placement.id,
            "position": placement.position,
            "saved_layout": placement.config,
            "consumer": "dashboard"
        }),
    ))
}

async fn load_health(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
) -> Result<DependencyHealthResponse, DashboardModuleError> {
    let rows = sqlx::query(
        "SELECT findings.id,findings.placement_id,findings.finding_code,
                findings.disposition,findings.finding_revision,
                findings.observed_resource_revision,findings.saved_reference,findings.impact,
                observations.provider_detail,observations.observed_at
         FROM dashboard_dependency_findings findings
         JOIN dashboard_dependency_observations observations ON observations.id=findings.observation_id
         WHERE findings.dashboard_id=$1 AND findings.disposition IN ('open','deferred')
         ORDER BY findings.created_at,findings.id",
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?;
    let all_findings = rows
        .into_iter()
        .map(|row| {
            let provider_detail: Option<Value> = row.try_get("provider_detail")?;
            let observed_lifecycle = provider_detail
                .as_ref()
                .and_then(|detail| detail.pointer("/resolution/resource_lifecycle_state/state"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let publication_state = provider_detail
                .as_ref()
                .and_then(|detail| detail.pointer("/metadata/publication_state"))
                .and_then(Value::as_str)
                .map(str::to_string);
            let change_categories = provider_detail
                .as_ref()
                .and_then(|detail| detail.get("changes"))
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(|change| change.get("categories").and_then(Value::as_array))
                .flatten()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect();
            let successor_available = provider_detail
                .as_ref()
                .and_then(|detail| detail.get("successor"))
                .is_some_and(|successor| !successor.is_null());
            Ok(DependencyFindingResponse {
                id: row.try_get("id")?,
                placement_id: row.try_get("placement_id")?,
                finding_code: row.try_get("finding_code")?,
                disposition: row.try_get("disposition")?,
                finding_revision: row.try_get("finding_revision")?,
                observed_resource_revision: row.try_get("observed_resource_revision")?,
                saved_reference: row.try_get("saved_reference")?,
                observed_lifecycle,
                publication_state,
                change_categories,
                successor_available,
                impact: row.try_get("impact")?,
                observed_at: row.try_get("observed_at")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let degraded = !all_findings.is_empty();
    let open_count = all_findings
        .iter()
        .filter(|finding| finding.disposition == "open")
        .count() as i64;
    let deferred_count = all_findings.len() as i64 - open_count;
    let findings = all_findings
        .into_iter()
        .filter(|finding| finding.finding_code != "restricted")
        .collect();
    Ok(DependencyHealthResponse {
        dashboard_id,
        health: if degraded { "degraded" } else { "healthy" },
        open_count,
        deferred_count,
        findings,
    })
}

fn digest_json(value: &impl Serialize) -> Result<String, DashboardModuleError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| DashboardModuleError::BadRequest("dependency value is invalid".into()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dependency_api_has_no_manual_resolve_action() {
        for action in [
            DependencyAction::Defer,
            DependencyAction::Upgrade,
            DependencyAction::Replace,
            DependencyAction::Remove,
        ] {
            assert_ne!(action.as_str(), "resolve");
        }
    }

    #[test]
    fn replacement_reference_is_exclusive_to_replace() {
        let request = DependencyActionRequest {
            action: DependencyAction::Upgrade,
            expected_finding_revision: 1,
            replacement_component_version_id: None,
        };
        assert!(validate_action_request(&request).is_ok());

        let mut invalid = request;
        invalid.expected_finding_revision = 0;
        assert!(validate_action_request(&invalid).is_err());
    }
}
