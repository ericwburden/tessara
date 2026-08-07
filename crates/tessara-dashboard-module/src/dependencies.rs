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
    AuthorizationGrantOperationV1, AuthorizationGrantV3, ContractCompatibilityState,
    ProviderAvailabilityState, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceResolutionV1, ResourceRevision, SignedEnvelopeV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY,
    composition::{
        ComponentAuthorizationContext, ComponentResolutionAttempt, ComponentResolutionOrigin,
        authorization_header, component_authorization_context, load_dashboard_scope,
        resolve_component_since,
    },
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
    pub replacement_component_reference: Option<ComponentVersionReference>,
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

struct VisibleDependencyHealthProjection {
    health: &'static str,
    open_count: i64,
    deferred_count: i64,
    findings: Vec<DependencyFindingResponse>,
}

struct PlacementToRefresh {
    id: Uuid,
    position: i32,
    reference: TypedResourceReference,
    config: Value,
}

const LOAD_AUTHORIZED_ACCESS_BASIS_SQL: &str = "SELECT provider_detail
     FROM dashboard_dependency_observations
     WHERE dashboard_id=$1 AND placement_id=$2 AND reference_digest=$3
       AND authorization_context_digest=$4
       AND authorization_expires_at > now()
       AND resolution_origin='provider_evaluated'
       AND resolution->>'access_state'='authorized'
       AND provider_detail IS NOT NULL
     ORDER BY observed_at DESC,id DESC LIMIT 1";

const UPSERT_DEPENDENCY_FINDING_SQL: &str = "INSERT INTO dashboard_dependency_findings
     (id,dashboard_id,placement_id,observation_id,authorization_context_digest,
      saved_reference,reference_digest,observed_resource_revision,finding_code,impact)
     VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
     ON CONFLICT(placement_id,reference_digest,authorization_context_digest,
                 observed_resource_revision,finding_code)
     DO UPDATE SET observation_id=EXCLUDED.observation_id,
                   impact=EXCLUDED.impact,
                   disposition='open',deferred_by=NULL,deferred_at=NULL,
                   resolved_at=NULL,updated_at=now(),
                   finding_revision=dashboard_dependency_findings.finding_revision+1
     WHERE dashboard_dependency_findings.disposition='resolved'";

const RESOLVE_CURRENT_FINDINGS_SQL: &str = "UPDATE dashboard_dependency_findings
     SET disposition='resolved',resolved_at=now(),updated_at=now(),
         finding_revision=finding_revision+1
     WHERE placement_id=$1 AND reference_digest=$2
       AND authorization_context_digest=$3
       AND disposition IN ('open','deferred')";

const ACTIONABLE_FINDING_VISIBILITY_SQL: &str = "SELECT EXISTS(
         SELECT 1 FROM dashboard_dependency_findings findings
         WHERE findings.id=$1 AND findings.dashboard_id=$2
           AND findings.authorization_context_digest=$3
           AND EXISTS (
               SELECT 1 FROM dashboard_placements current_placement
               WHERE current_placement.id=findings.placement_id
                 AND current_placement.dashboard_id=findings.dashboard_id
                 AND current_placement.component_reference=findings.saved_reference
           )
           AND EXISTS (
               SELECT 1 FROM dashboard_dependency_observations access_basis
               WHERE access_basis.dashboard_id=findings.dashboard_id
                 AND access_basis.placement_id=findings.placement_id
                 AND access_basis.reference_digest=findings.reference_digest
                 AND access_basis.authorization_context_digest=findings.authorization_context_digest
                 AND access_basis.authorization_expires_at > now()
                 AND access_basis.resolution_origin='provider_evaluated'
                 AND access_basis.resolution->>'access_state'='authorized'
           )
     )";

pub(super) async fn project_component_resolution_for_visibility(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
    placement_id: Uuid,
    reference: &TypedResourceReference,
    attempt: &ComponentResolutionAttempt,
) -> Result<ComponentResolutionResponse, DashboardModuleError> {
    if attempt.origin() != ComponentResolutionOrigin::SyntheticUnavailable {
        return Ok(attempt.response().clone());
    }
    let Some(context) = attempt.authorization_context() else {
        return Ok(restricted_not_evaluated_resolution());
    };
    let reference_digest = digest_json(reference)?;
    let prior_detail = sqlx::query_scalar::<_, Value>(LOAD_AUTHORIZED_ACCESS_BASIS_SQL)
        .bind(dashboard_id)
        .bind(placement_id)
        .bind(reference_digest)
        .bind(context.digest())
        .fetch_optional(&state.pool)
        .await?;
    let Some(prior_detail) = prior_detail else {
        return Ok(restricted_not_evaluated_resolution());
    };
    let prior: ComponentResolutionResponse =
        serde_json::from_value(prior_detail).map_err(|_| {
            DashboardModuleError::Unavailable(
                "stored Component authorization basis is invalid".into(),
            )
        })?;
    if prior.resolution().access_state() != ResourceAccessState::Authorized {
        return Ok(restricted_not_evaluated_resolution());
    }
    provider_unavailable_projection(&prior)
}

fn provider_unavailable_projection(
    prior: &ComponentResolutionResponse,
) -> Result<ComponentResolutionResponse, DashboardModuleError> {
    let prior = prior.resolution();
    ComponentResolutionResponse::new(
        ResourceResolutionV1::authorized(
            prior.owner_state(),
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            prior.compatibility_state(),
            ProviderAvailabilityState::Unavailable,
        )
        .map_err(|_| {
            DashboardModuleError::Unavailable(
                "stored Component authorization basis is invalid".into(),
            )
        })?,
        None,
        None,
        Vec::new(),
        None,
    )
    .map_err(|_| DashboardModuleError::Unavailable("Component outage projection failed".into()))
}

fn restricted_not_evaluated_resolution() -> ComponentResolutionResponse {
    ComponentResolutionResponse::new(
        ResourceResolutionV1::restricted(ResourceAccessState::NotEvaluated)
            .expect("not-evaluated Component projection is valid"),
        None,
        None,
        Vec::new(),
        None,
    )
    .expect("not-evaluated Component projection is non-disclosing")
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
    let authorization = authorization_header(&headers)?;
    let context =
        dashboard_component_authorization_context(&state, authorization, dashboard_id).await?;
    Ok(Json(
        load_health(&state, dashboard_id, context.as_ref()).await?,
    ))
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
    let mut authorization_context = None;
    for placement in placements {
        let observed_context = refresh_placement(
            state,
            authorization,
            dashboard_id,
            placement,
            correlation_id,
        )
        .await?;
        if let Some(observed_context) = observed_context {
            if authorization_context.as_ref().is_some_and(
                |current: &ComponentAuthorizationContext| {
                    current.digest() != observed_context.digest()
                },
            ) {
                return Err(DashboardModuleError::Unavailable(
                    "Component authorization context changed during dependency refresh".into(),
                ));
            }
            authorization_context = Some(observed_context);
        }
    }
    load_health(state, dashboard_id, authorization_context.as_ref()).await
}

async fn dashboard_component_authorization_context(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
) -> Result<Option<ComponentAuthorizationContext>, DashboardModuleError> {
    let reference = sqlx::query_scalar::<_, Value>(
        "SELECT component_reference FROM dashboard_placements
         WHERE dashboard_id=$1 ORDER BY position,id LIMIT 1",
    )
    .bind(dashboard_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(reference) = reference else {
        return Ok(None);
    };
    let reference: TypedResourceReference = serde_json::from_value(reference).map_err(|_| {
        DashboardModuleError::Conflict("stored Component reference is invalid".into())
    })?;
    let reference = ComponentVersionReference::new(reference)
        .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?;
    component_authorization_context(state, authorization, &reference).await
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
    let action_context =
        ensure_finding_visible_to_authorization(&state, authorization, dashboard_id, finding_id)
            .await?;
    let correlation_id = correlation_id(&headers);
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let request_digest = digest_json(&json!({
        "dashboard_id": dashboard_id,
        "finding_id": finding_id,
        "request": request,
    }))?;
    if let Some(existing) = sqlx::query(
        "SELECT request_digest,actor_id,authorization_context_digest,result
         FROM dashboard_dependency_action_receipts
         WHERE idempotency_key=$1",
    )
    .bind(idempotency_key)
    .fetch_optional(&state.pool)
    .await?
    {
        if existing.try_get::<Uuid, _>("actor_id")? != grant.payload.original_actor_id {
            return Err(DashboardModuleError::Conflict(
                "dependency action idempotency key belongs to another actor".into(),
            ));
        }
        if existing.try_get::<String, _>("authorization_context_digest")? != action_context.digest()
        {
            return Err(DashboardModuleError::Conflict(
                "dependency action idempotency key belongs to another authorization context".into(),
            ));
        }
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
    let replacement = proposed_reference(
        &state,
        authorization,
        dashboard_id,
        finding_id,
        action_context.digest(),
        &request,
    )
    .await?;

    let mut tx = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(idempotency_key)
        .execute(&mut *tx)
        .await?;
    if let Some(existing) = sqlx::query(
        "SELECT request_digest,actor_id,authorization_context_digest,result
         FROM dashboard_dependency_action_receipts
         WHERE idempotency_key=$1 FOR UPDATE",
    )
    .bind(idempotency_key)
    .fetch_optional(&mut *tx)
    .await?
    {
        if existing.try_get::<Uuid, _>("actor_id")? != grant.payload.original_actor_id {
            return Err(DashboardModuleError::Conflict(
                "dependency action idempotency key belongs to another actor".into(),
            ));
        }
        if existing.try_get::<String, _>("authorization_context_digest")? != action_context.digest()
        {
            return Err(DashboardModuleError::Conflict(
                "dependency action idempotency key belongs to another authorization context".into(),
            ));
        }
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
         WHERE id=$1 AND dashboard_id=$2 AND authorization_context_digest=$3 FOR UPDATE",
    )
    .bind(finding_id)
    .bind(dashboard_id)
    .bind(action_context.digest())
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
          actor_id,authorization_context_digest,action,expected_finding_revision,result)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)",
    )
    .bind(idempotency_key)
    .bind(request_digest)
    .bind(dashboard_id)
    .bind(placement_id)
    .bind(finding_id)
    .bind(grant.payload.original_actor_id)
    .bind(action_context.digest())
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
        request.replacement_component_reference.is_some(),
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

async fn ensure_finding_visible_to_authorization(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
    finding_id: Uuid,
) -> Result<ComponentAuthorizationContext, DashboardModuleError> {
    let finding = sqlx::query(
        "SELECT saved_reference
         FROM dashboard_dependency_findings
         WHERE id=$1 AND dashboard_id=$2",
    )
    .bind(finding_id)
    .bind(dashboard_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| DashboardModuleError::NotFound("dependency finding not found".into()))?;
    let reference: TypedResourceReference =
        serde_json::from_value(finding.try_get("saved_reference")?).map_err(|_| {
            DashboardModuleError::Unavailable(
                "stored dependency finding reference is invalid".into(),
            )
        })?;
    let reference = ComponentVersionReference::new(reference)
        .map_err(|_| DashboardModuleError::NotFound("dependency finding not found".into()))?;
    let Some(context) = component_authorization_context(state, authorization, &reference).await?
    else {
        return Err(DashboardModuleError::NotFound(
            "dependency finding not found".into(),
        ));
    };
    let visible = sqlx::query_scalar::<_, bool>(ACTIONABLE_FINDING_VISIBILITY_SQL)
        .bind(finding_id)
        .bind(dashboard_id)
        .bind(context.digest())
        .fetch_one(&state.pool)
        .await?;
    if !visible {
        return Err(DashboardModuleError::NotFound(
            "dependency finding not found".into(),
        ));
    }
    Ok(context)
}

async fn proposed_reference(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
    finding_id: Uuid,
    authorization_context_digest: &str,
    request: &DependencyActionRequest,
) -> Result<Option<TypedResourceReference>, DashboardModuleError> {
    let reference = match request.action {
        DependencyAction::Upgrade => {
            let provider_detail = sqlx::query_scalar::<_, Option<Value>>(
                "SELECT observations.provider_detail
                 FROM dashboard_dependency_findings findings
                 JOIN dashboard_dependency_observations observations
                   ON observations.id=findings.observation_id
                 WHERE findings.id=$1 AND findings.dashboard_id=$2
                   AND findings.authorization_context_digest=$3
                   AND observations.authorization_context_digest=$3",
            )
            .bind(finding_id)
            .bind(dashboard_id)
            .bind(authorization_context_digest)
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
            let replacement = request
                .replacement_component_reference
                .as_ref()
                .ok_or_else(|| {
                    DashboardModuleError::BadRequest(
                        "replace requires a replacement Component version".into(),
                    )
                })?;
            let saved_reference = sqlx::query_scalar::<_, Value>(
                "SELECT saved_reference FROM dashboard_dependency_findings
                 WHERE id=$1 AND dashboard_id=$2 AND authorization_context_digest=$3",
            )
            .bind(finding_id)
            .bind(dashboard_id)
            .bind(authorization_context_digest)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| DashboardModuleError::NotFound("dependency finding not found".into()))?;
            let saved_reference: TypedResourceReference = serde_json::from_value(saved_reference)
                .map_err(|_| {
                DashboardModuleError::Conflict(
                    "stored dependency finding reference is invalid".into(),
                )
            })?;
            let replacement = replacement.reference();
            if replacement.installation_id() != saved_reference.installation_id()
                || replacement.owner() != saved_reference.owner()
            {
                return Err(DashboardModuleError::BadRequest(
                    "replacement Component reference must use the selected provider instance"
                        .into(),
                ));
            }
            replacement.clone()
        }
        DependencyAction::Defer | DependencyAction::Remove => return Ok(None),
    };
    let wrapped = ComponentVersionReference::new(reference.clone())
        .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
    let attempt = resolve_component_since(state, authorization, wrapped, None).await?;
    if attempt.origin() == ComponentResolutionOrigin::SyntheticUnavailable {
        return Err(DashboardModuleError::Unavailable(
            "Component provider unavailable".into(),
        ));
    }
    if attempt.origin() != ComponentResolutionOrigin::ProviderEvaluated
        || !attempt
            .response()
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
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, DashboardModuleError> {
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
) -> Result<Option<ComponentAuthorizationContext>, DashboardModuleError> {
    let reference_digest = digest_json(&placement.reference)?;
    let wrapped = ComponentVersionReference::new(placement.reference.clone())
        .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?;
    let requested_context = component_authorization_context(state, authorization, &wrapped).await?;
    let prior_revision = if let Some(context) = &requested_context {
        sqlx::query_scalar::<_, Option<i64>>(
            "SELECT MAX(resource_revision) FROM dashboard_dependency_observations
             WHERE dashboard_id=$1 AND placement_id=$2 AND reference_digest=$3
               AND authorization_context_digest=$4
               AND authorization_expires_at > now()
               AND resolution_origin='provider_evaluated'
               AND resolution->>'access_state'='authorized'",
        )
        .bind(dashboard_id)
        .bind(placement.id)
        .bind(&reference_digest)
        .bind(context.digest())
        .fetch_one(&state.pool)
        .await?
    } else {
        None
    }
    .map(|revision| ResourceRevision::new(revision as u64))
    .transpose()
    .map_err(|_| DashboardModuleError::Conflict("stored observation revision is invalid".into()))?;
    let attempt = resolve_component_since(state, authorization, wrapped, prior_revision).await?;
    if let (Some(requested), Some(observed)) =
        (requested_context.as_ref(), attempt.authorization_context())
        && requested.digest() != observed.digest()
    {
        return Err(DashboardModuleError::Unavailable(
            "Component authorization context changed during dependency observation".into(),
        ));
    }
    let response = project_component_resolution_for_visibility(
        state,
        dashboard_id,
        placement.id,
        &placement.reference,
        &attempt,
    )
    .await?;
    let authorization_context = attempt.authorization_context().cloned();
    if response.resolution().access_state() != ResourceAccessState::Authorized {
        tracing::info!(
            correlation_id,
            actor_class = "authorized_dashboard_manager",
            reference_digest,
            prior_revision = prior_revision.map(|revision| revision.get()),
            current_revision = Option::<u64>::None,
            action = "refresh",
            result_code = "restricted",
            resolution_origin = attempt.origin().as_str(),
            provider_contract_id = COMPONENT_CONTRACT_ID,
            provider_contract_version = COMPONENT_CONTRACT_VERSION,
            "dashboard dependency refresh"
        );
        return Ok(authorization_context);
    }
    let context = authorization_context.as_ref().ok_or_else(|| {
        DashboardModuleError::Unavailable(
            "authorized Component observation omitted its authorization context".into(),
        )
    })?;
    let response_json = serde_json::to_value(&response).map_err(|_| {
        DashboardModuleError::Unavailable("Component observation encoding failed".into())
    })?;
    let observation_fingerprint = digest_json(&response_json)?;
    let resource_revision = response
        .observation()
        .map(|observation| observation.resource_revision().get() as i64);
    let provider_detail = (attempt.origin() == ComponentResolutionOrigin::ProviderEvaluated)
        .then(|| serde_json::to_value(attempt.response()))
        .transpose()
        .map_err(|_| {
            DashboardModuleError::Unavailable("Component observation encoding failed".into())
        })?;
    let observation_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO dashboard_dependency_observations
         (id,dashboard_id,placement_id,saved_reference,reference_digest,
          provider_contract_id,provider_contract_version,authorization_context_digest,
          authorization_expires_at,resolution_origin,resource_revision,
          observation_fingerprint,resolution,provider_detail)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14)",
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
    .bind(context.digest())
    .bind(context.expires_at())
    .bind(attempt.origin().as_str())
    .bind(resource_revision)
    .bind(&observation_fingerprint)
    .bind(serde_json::to_value(response.resolution()).map_err(|_| {
        DashboardModuleError::Unavailable("Component resolution encoding failed".into())
    })?)
    .bind(provider_detail)
    .execute(&state.pool)
    .await?;

    let finding = classify_finding(&response, &placement);
    let result_code = finding.as_ref().map_or("healthy", |(code, _)| *code);
    if let Some((finding_code, impact)) = finding {
        sqlx::query(
            "UPDATE dashboard_dependency_findings
             SET disposition='resolved',resolved_at=now(),updated_at=now(),
                 finding_revision=finding_revision+1
             WHERE placement_id=$1 AND reference_digest=$2
               AND authorization_context_digest=$5
               AND disposition IN ('open','deferred')
               AND (observed_resource_revision IS DISTINCT FROM $3
                    OR finding_code IS DISTINCT FROM $4)",
        )
        .bind(placement.id)
        .bind(&reference_digest)
        .bind(resource_revision.unwrap_or(0))
        .bind(finding_code)
        .bind(context.digest())
        .execute(&state.pool)
        .await?;
        sqlx::query(UPSERT_DEPENDENCY_FINDING_SQL)
            .bind(Uuid::new_v4())
            .bind(dashboard_id)
            .bind(placement.id)
            .bind(observation_id)
            .bind(context.digest())
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
        sqlx::query(RESOLVE_CURRENT_FINDINGS_SQL)
            .bind(placement.id)
            .bind(&reference_digest)
            .bind(context.digest())
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
        resolution_origin = attempt.origin().as_str(),
        provider_contract_id = COMPONENT_CONTRACT_ID,
        provider_contract_version = COMPONENT_CONTRACT_VERSION,
        "dashboard dependency refresh"
    );
    Ok(authorization_context)
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
    let code = if resolution.availability_state() != ProviderAvailabilityState::Available {
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
    authorization_context: Option<&ComponentAuthorizationContext>,
) -> Result<DependencyHealthResponse, DashboardModuleError> {
    let Some(authorization_context) = authorization_context else {
        return Ok(DependencyHealthResponse {
            dashboard_id,
            health: "healthy",
            open_count: 0,
            deferred_count: 0,
            findings: Vec::new(),
        });
    };
    let rows = sqlx::query(
        "SELECT findings.id,findings.placement_id,findings.finding_code,
                findings.disposition,findings.finding_revision,
                findings.observed_resource_revision,findings.saved_reference,findings.impact,
                observations.provider_detail,observations.observed_at
         FROM dashboard_dependency_findings findings
         JOIN dashboard_dependency_observations observations
           ON observations.id=findings.observation_id
          AND observations.authorization_context_digest=findings.authorization_context_digest
         WHERE findings.dashboard_id=$1 AND findings.disposition IN ('open','deferred')
           AND findings.authorization_context_digest=$2
           AND EXISTS (
               SELECT 1 FROM dashboard_placements current_placement
               WHERE current_placement.id=findings.placement_id
                 AND current_placement.dashboard_id=findings.dashboard_id
                 AND current_placement.component_reference=findings.saved_reference
           )
           AND EXISTS (
               SELECT 1 FROM dashboard_dependency_observations access_basis
               WHERE access_basis.dashboard_id=findings.dashboard_id
                 AND access_basis.placement_id=findings.placement_id
                 AND access_basis.reference_digest=findings.reference_digest
                 AND access_basis.authorization_context_digest=$2
                 AND access_basis.authorization_expires_at > now()
                 AND access_basis.resolution_origin='provider_evaluated'
                 AND access_basis.resolution->>'access_state'='authorized'
           )
         ORDER BY findings.created_at,findings.id",
    )
    .bind(dashboard_id)
    .bind(authorization_context.digest())
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
    let visible = project_visible_dependency_health(all_findings);
    Ok(DependencyHealthResponse {
        dashboard_id,
        health: visible.health,
        open_count: visible.open_count,
        deferred_count: visible.deferred_count,
        findings: visible.findings,
    })
}

fn project_visible_dependency_health(
    all_findings: Vec<DependencyFindingResponse>,
) -> VisibleDependencyHealthProjection {
    let findings = all_findings
        .into_iter()
        .filter(|finding| finding.finding_code != "restricted")
        .collect::<Vec<_>>();
    let open_count = findings
        .iter()
        .filter(|finding| finding.disposition == "open")
        .count() as i64;
    let deferred_count = findings.len() as i64 - open_count;
    VisibleDependencyHealthProjection {
        health: if findings.is_empty() {
            "healthy"
        } else {
            "degraded"
        },
        open_count,
        deferred_count,
        findings,
    }
}

fn digest_json(value: &impl Serialize) -> Result<String, DashboardModuleError> {
    let bytes = serde_json::to_vec(value)
        .map_err(|_| DashboardModuleError::BadRequest("dependency value is invalid".into()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tessara_module_contract::{ModuleInstanceOwnerState, OwnerDataState, ResourceOwnerState};

    fn provider_availability_response(
        availability_state: ProviderAvailabilityState,
    ) -> ComponentResolutionResponse {
        ComponentResolutionResponse::new(
            ResourceResolutionV1::authorized(
                ResourceOwnerState::ModuleInstance {
                    instance_state: ModuleInstanceOwnerState::Live,
                    data_state: OwnerDataState::Retained,
                },
                ResourceIdentityState::NotEvaluated,
                ResourceLifecycleState::NotEvaluated,
                ContractCompatibilityState::Compatible,
                availability_state,
            )
            .expect("test provider resolution must be valid"),
            None,
            None,
            Vec::new(),
            None,
        )
        .expect("test provider response must be non-disclosing")
    }

    fn finding(
        finding_id: Uuid,
        placement_id: Uuid,
        finding_code: &str,
        disposition: &str,
    ) -> DependencyFindingResponse {
        DependencyFindingResponse {
            id: finding_id,
            placement_id,
            finding_code: finding_code.into(),
            disposition: disposition.into(),
            finding_revision: 1,
            observed_resource_revision: 1,
            saved_reference: json!({ "opaque": "test-only" }),
            observed_lifecycle: None,
            publication_state: None,
            change_categories: Vec::new(),
            successor_available: false,
            impact: json!({ "placement_id": placement_id }),
            observed_at: Utc::now(),
        }
    }

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
            replacement_component_reference: None,
        };
        assert!(validate_action_request(&request).is_ok());

        let mut invalid = request;
        invalid.expected_finding_revision = 0;
        assert!(validate_action_request(&invalid).is_err());
    }

    #[test]
    fn restricted_findings_cannot_affect_visible_health_or_recovery() {
        let blocked_placement_id = Uuid::parse_str("01980000-0003-7000-8000-000000000005")
            .expect("blocked placement fixture must be a UUID");
        let restricted = finding(Uuid::new_v4(), blocked_placement_id, "restricted", "open");

        let recovered = project_visible_dependency_health(vec![restricted]);
        assert_eq!(recovered.health, "healthy");
        assert_eq!(recovered.open_count, 0);
        assert_eq!(recovered.deferred_count, 0);
        assert!(recovered.findings.is_empty());

        let visible_open_placement = Uuid::new_v4();
        let visible_deferred_placement = Uuid::new_v4();
        let projected = project_visible_dependency_health(vec![
            finding(Uuid::new_v4(), blocked_placement_id, "restricted", "open"),
            finding(
                Uuid::new_v4(),
                visible_open_placement,
                "provider_unavailable",
                "open",
            ),
            finding(
                Uuid::new_v4(),
                visible_deferred_placement,
                "provider_unavailable",
                "deferred",
            ),
        ]);
        assert_eq!(projected.health, "degraded");
        assert_eq!(projected.open_count, 1);
        assert_eq!(projected.deferred_count, 1);
        assert_eq!(projected.findings.len(), 2);
        assert!(
            projected
                .findings
                .iter()
                .all(|finding| finding.placement_id != blocked_placement_id)
        );
    }

    #[test]
    fn provider_outage_projection_requires_a_provider_evaluated_access_basis() {
        let first_outage = restricted_not_evaluated_resolution();
        assert_eq!(
            first_outage.resolution().access_state(),
            ResourceAccessState::NotEvaluated
        );
        assert_eq!(
            first_outage.resolution().availability_state(),
            ProviderAvailabilityState::Undisclosed
        );
        let encoded = serde_json::to_string(&first_outage)
            .expect("restricted response must remain serializable");
        assert!(!encoded.contains("\"saved_reference\":"));
        assert!(!encoded.contains("\"resource_id\":"));

        let provider_evaluated =
            provider_availability_response(ProviderAvailabilityState::Available);
        let previously_authorized = provider_unavailable_projection(&provider_evaluated)
            .expect("an exact provider-evaluated access basis permits outage projection");
        assert_eq!(
            previously_authorized.resolution().access_state(),
            ResourceAccessState::Authorized
        );
        assert_eq!(
            previously_authorized.resolution().availability_state(),
            ProviderAvailabilityState::Unavailable
        );

        let recovered = provider_availability_response(ProviderAvailabilityState::Available);
        assert_eq!(
            recovered.resolution().access_state(),
            ResourceAccessState::Authorized
        );
        assert_eq!(
            recovered.resolution().availability_state(),
            ProviderAvailabilityState::Available
        );
    }

    #[test]
    fn viewer_outage_projection_is_read_only_and_exactly_context_bound() {
        let sql = LOAD_AUTHORIZED_ACCESS_BASIS_SQL.to_ascii_lowercase();
        for identity in [
            "dashboard_id=$1",
            "placement_id=$2",
            "reference_digest=$3",
            "authorization_context_digest=$4",
            "authorization_expires_at > now()",
            "resolution_origin='provider_evaluated'",
            "resolution->>'access_state'='authorized'",
        ] {
            assert!(
                sql.contains(identity),
                "missing exact access basis: {identity}"
            );
        }
        for mutation in ["insert ", "update ", "delete ", "merge "] {
            assert!(
                !sql.contains(mutation),
                "viewer projection must not mutate dependency state"
            );
        }
        assert!(!sql.contains("synthetic_unavailable"));
    }

    #[test]
    fn identical_finding_reopens_only_after_recovery_and_advances_revision() {
        let upsert = UPSERT_DEPENDENCY_FINDING_SQL.to_ascii_lowercase();
        assert!(upsert.contains("on conflict"));
        assert!(
            upsert.contains(
                "on conflict(placement_id,reference_digest,authorization_context_digest,"
            )
        );
        assert!(upsert.contains("disposition='open'"));
        assert!(
            upsert.contains("finding_revision=dashboard_dependency_findings.finding_revision+1")
        );
        assert!(upsert.contains("where dashboard_dependency_findings.disposition='resolved'"));

        let recovery = RESOLVE_CURRENT_FINDINGS_SQL.to_ascii_lowercase();
        assert!(recovery.contains("disposition='resolved'"));
        assert!(recovery.contains("finding_revision=finding_revision+1"));
        assert!(recovery.contains("authorization_context_digest=$3"));
        assert!(recovery.contains("disposition in ('open','deferred')"));
    }

    #[test]
    fn finding_actions_require_the_finding_and_access_basis_in_one_exact_context() {
        let sql = ACTIONABLE_FINDING_VISIBILITY_SQL.to_ascii_lowercase();
        assert!(sql.contains("findings.authorization_context_digest=$3"));
        assert!(sql.contains(
            "access_basis.authorization_context_digest=findings.authorization_context_digest"
        ));
        assert!(sql.contains("current_placement.component_reference=findings.saved_reference"));
        assert!(sql.contains("access_basis.authorization_expires_at > now()"));
        assert!(sql.contains("access_basis.resolution_origin='provider_evaluated'"));
    }

    #[test]
    fn fresh_baseline_records_context_expiry_origin_and_append_only_recency() {
        let baseline = include_str!("../migrations/001_dashboard_module.sql");
        for field in [
            "authorization_context_digest TEXT NOT NULL",
            "authorization_expires_at TIMESTAMPTZ NOT NULL",
            "CHECK (authorization_expires_at > observed_at)",
            "resolution_origin TEXT NOT NULL",
            "UNIQUE (placement_id, reference_digest, authorization_context_digest,",
            "FOREIGN KEY (observation_id, authorization_context_digest)",
            "'provider_evaluated'",
            "'synthetic_unavailable'",
        ] {
            assert!(baseline.contains(field), "baseline omitted {field}");
        }
        assert!(!baseline.contains("UNIQUE (placement_id, observation_fingerprint)"));
        assert!(baseline.contains("dashboard_dependency_observations_immutable"));
    }
}
