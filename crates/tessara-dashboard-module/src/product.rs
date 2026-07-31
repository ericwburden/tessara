use std::collections::BTreeSet;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post, put},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV1, AuthorizationValidationContextV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, SecurityCapabilityId,
    SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY, READ_CAPABILITY,
    load_security_state, require_private_key,
};

const CORE_DASHBOARD_BINDING: &str = "tessara.core.dashboards";
const DASHBOARD_CONTRACT: &str = "tessara.dashboards.dashboard";
const COMPOSITION_CONTRACT: &str = "tessara.dashboards.composition";

fn contract_for_action(action: &str) -> &'static str {
    match action {
        "dashboards.load_composition" | "dashboards.reconcile_composition" => COMPOSITION_CONTRACT,
        _ => DASHBOARD_CONTRACT,
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardInputV1 {
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub visibility_node_ids: Vec<Uuid>,
    #[serde(default)]
    pub idempotency_key: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DashboardIdResponseV1 {
    pub id: Uuid,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardVisibilityNodeV1 {
    pub node_id: Uuid,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<Uuid>,
    pub node_path: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardSummaryV1 {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub visibility_nodes: Vec<DashboardVisibilityNodeV1>,
    pub placement_count: i64,
    pub can_manage: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrganizationProjectionInputV1 {
    schema_version: u16,
    organization_revision: u64,
    nodes: Vec<OrganizationProjectionNodeV1>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct OrganizationProjectionNodeV1 {
    node_id: Uuid,
    node_name: String,
    node_type_name: String,
    parent_node_id: Option<Uuid>,
    node_path: String,
}

pub(super) fn routes() -> Router<DashboardModuleState> {
    Router::new()
        .route(
            "/api/private/organization-projection",
            put(update_organization_projection),
        )
        .route(
            "/api/admin/dashboards",
            post(create_dashboard).get(list_manageable_dashboards),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}",
            put(update_dashboard).delete(delete_dashboard),
        )
        .route("/api/dashboards", get(list_dashboards))
}

async fn update_organization_projection(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Json(input): Json<OrganizationProjectionInputV1>,
) -> Result<axum::http::StatusCode, DashboardModuleError> {
    require_private_key(&headers)?;
    if input.schema_version != 1 || input.organization_revision == 0 {
        return Err(DashboardModuleError::BadRequest(
            "invalid Organization projection".into(),
        ));
    }
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DashboardModuleError::Unavailable("security state unavailable".into()))?;
    if input.organization_revision != security.organization_revision as u64 {
        return Err(DashboardModuleError::Conflict(
            "Organization projection revision does not match Core security state".into(),
        ));
    }
    let mut ids = BTreeSet::new();
    for node in &input.nodes {
        if node.node_id.is_nil()
            || node.node_name.trim().is_empty()
            || node.node_type_name.trim().is_empty()
            || node.node_path.trim().is_empty()
            || !ids.insert(node.node_id)
        {
            return Err(DashboardModuleError::BadRequest(
                "Organization projection contains an invalid or duplicate node".into(),
            ));
        }
    }
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        "UPDATE dashboard_organization_nodes
         SET active=false,projection_revision=$1,updated_at=now()
         WHERE active=true",
    )
    .bind(input.organization_revision as i64)
    .execute(&mut *tx)
    .await?;
    for node in input.nodes {
        sqlx::query(
            "INSERT INTO dashboard_organization_nodes
             (node_id,node_name,node_type_name,parent_node_id,node_path,active,projection_revision)
             VALUES ($1,$2,$3,$4,$5,true,$6)
             ON CONFLICT (node_id) DO UPDATE SET
               node_name=EXCLUDED.node_name,
               node_type_name=EXCLUDED.node_type_name,
               parent_node_id=EXCLUDED.parent_node_id,
               node_path=EXCLUDED.node_path,
               active=true,
               projection_revision=EXCLUDED.projection_revision,
               updated_at=now()",
        )
        .bind(node.node_id)
        .bind(node.node_name.trim())
        .bind(node.node_type_name.trim())
        .bind(node.parent_node_id)
        .bind(node.node_path.trim())
        .bind(input.organization_revision as i64)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub(super) async fn list_dashboards(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DashboardSummaryV1>>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.list",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let read_scope = authorized_organizations(&grant.payload, READ_CAPABILITY);
    let manage_scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    Ok(Json(
        load_dashboards(&state, &read_scope, &manage_scope).await?,
    ))
}

pub(super) async fn list_manageable_dashboards(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DashboardSummaryV1>>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.list_manageable",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let manage_scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    Ok(Json(
        load_dashboards(&state, &manage_scope, &manage_scope).await?,
    ))
}

pub(super) async fn get_dashboard_summary(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DashboardSummaryV1>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let read_scope = authorized_organizations(&grant.payload, READ_CAPABILITY);
    let manage_scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    let dashboard = load_dashboard(&state, dashboard_id, &read_scope, &manage_scope)
        .await?
        .ok_or_else(|| DashboardModuleError::NotFound("Dashboard not found".into()))?;
    Ok(Json(dashboard))
}

async fn create_dashboard(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Json(mut input): Json<DashboardInputV1>,
) -> Result<Json<DashboardIdResponseV1>, DashboardModuleError> {
    input.idempotency_key = mutation_idempotency_key(&headers)?.to_string();
    let grant = authorize(
        &state,
        &headers,
        "dashboards.create",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let scope = normalized_scope(&input.visibility_node_ids)?;
    require_authorized_scope(&grant.payload, MANAGE_CAPABILITY, &scope)?;
    validate_dashboard_input(&input)?;
    let mut tx = state.pool.begin().await?;
    let digest = mutation_digest("dashboards.create", None, &input)?;
    if let Some(result) = load_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.create",
        &input.idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(result));
    }
    require_projected_nodes(&mut tx, &scope).await?;
    let id = Uuid::new_v4();
    sqlx::query("INSERT INTO dashboards (id,name,description) VALUES ($1,$2,$3)")
        .bind(id)
        .bind(input.name.trim())
        .bind(normalized_optional_text(input.description.as_deref()))
        .execute(&mut *tx)
        .await?;
    replace_scope(&mut tx, id, &scope).await?;
    let result = DashboardIdResponseV1 { id };
    record_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.create",
        &input.idempotency_key,
        &digest,
        &result,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(result))
}

async fn update_dashboard(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(mut input): Json<DashboardInputV1>,
) -> Result<Json<DashboardIdResponseV1>, DashboardModuleError> {
    input.idempotency_key = mutation_idempotency_key(&headers)?.to_string();
    let grant = authorize(
        &state,
        &headers,
        "dashboards.update",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let scope = normalized_scope(&input.visibility_node_ids)?;
    require_authorized_scope(&grant.payload, MANAGE_CAPABILITY, &scope)?;
    validate_dashboard_input(&input)?;
    let mut tx = state.pool.begin().await?;
    let digest = mutation_digest("dashboards.update", Some(dashboard_id), &input)?;
    if let Some(result) = load_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.update",
        &input.idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(result));
    }
    lock_dashboard(&mut tx, dashboard_id).await?;
    let current_scope = load_scope_ids(&mut tx, dashboard_id).await?;
    require_authorized_scope(&grant.payload, MANAGE_CAPABILITY, &current_scope)?;
    require_projected_nodes(&mut tx, &scope).await?;
    sqlx::query(
        "UPDATE dashboards
         SET name=$2,description=$3,updated_at=now()
         WHERE id=$1",
    )
    .bind(dashboard_id)
    .bind(input.name.trim())
    .bind(normalized_optional_text(input.description.as_deref()))
    .execute(&mut *tx)
    .await?;
    replace_scope(&mut tx, dashboard_id, &scope).await?;
    let result = DashboardIdResponseV1 { id: dashboard_id };
    record_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.update",
        &input.idempotency_key,
        &digest,
        &result,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(result))
}

async fn delete_dashboard(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DashboardIdResponseV1>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.delete",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let idempotency_key = headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .ok_or_else(|| {
            DashboardModuleError::BadRequest("valid x-idempotency-key header is required".into())
        })?;
    let mut tx = state.pool.begin().await?;
    let digest = mutation_digest("dashboards.delete", Some(dashboard_id), &())?;
    if let Some(result) = load_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.delete",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(result));
    }
    lock_dashboard(&mut tx, dashboard_id).await?;
    let scope = load_scope_ids(&mut tx, dashboard_id).await?;
    require_authorized_scope(&grant.payload, MANAGE_CAPABILITY, &scope)?;
    sqlx::query("DELETE FROM dashboards WHERE id=$1")
        .bind(dashboard_id)
        .execute(&mut *tx)
        .await?;
    let result = DashboardIdResponseV1 { id: dashboard_id };
    record_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.delete",
        idempotency_key,
        &digest,
        &result,
    )
    .await?;
    tx.commit().await?;
    Ok(Json(result))
}

pub(super) async fn authorize(
    state: &DashboardModuleState,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV1>, DashboardModuleError> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(|| {
            tracing::warn!(
                action,
                "Dashboard authorization header is missing or invalid"
            );
            DashboardModuleError::Forbidden
        })?;
    let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|error| {
        tracing::warn!(%error, action, "Dashboard authorization envelope is not base64url");
        DashboardModuleError::Forbidden
    })?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV1> =
        serde_json::from_slice(&bytes).map_err(|error| {
            tracing::warn!(%error, action, "Dashboard authorization envelope is invalid");
            DashboardModuleError::Forbidden
        })?;
    state
        .core_authorization_verifier
        .verify(&envelope)
        .map_err(|error| {
            tracing::warn!(%error, action, "Dashboard authorization signature is invalid");
            DashboardModuleError::Forbidden
        })?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DashboardModuleError::Unavailable("security state unavailable".into()))?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(DashboardModuleError::Unavailable(
            "Dashboard module is not enabled".into(),
        ));
    }
    let expected_contract = contract_for_action(action);
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV1 {
            installation_id: security.installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core").expect("Core id"),
            audience_module_instance_id: security.module_instance_id,
            dependency_binding: DependencyBindingKey::new(CORE_DASHBOARD_BINDING).expect("binding"),
            functional_contract: FunctionalContractId::new(expected_contract).expect("contract"),
            action: action.into(),
            operation,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|error| {
            tracing::warn!(
                %error,
                action,
                expected_contract,
                presented_contract = envelope.payload.functional_contract.as_str(),
                "Dashboard authorization grant validation failed"
            );
            DashboardModuleError::Forbidden
        })?;
    Ok(envelope)
}

pub(super) fn authorized_organizations(
    grant: &AuthorizationGrantV1,
    capability: &str,
) -> BTreeSet<Uuid> {
    grant
        .capability_scope_bindings
        .iter()
        .filter(|binding| binding.capability.as_str() == capability)
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect()
}

fn require_authorized_scope(
    grant: &AuthorizationGrantV1,
    capability: &str,
    requested: &[Uuid],
) -> Result<(), DashboardModuleError> {
    let capability = SecurityCapabilityId::new(capability).expect("capability");
    if !requested.is_empty()
        && requested
            .iter()
            .all(|organization_id| grant.authorizes(&capability, *organization_id))
    {
        Ok(())
    } else {
        Err(DashboardModuleError::Forbidden)
    }
}

async fn load_dashboards(
    state: &DashboardModuleState,
    read_scope: &BTreeSet<Uuid>,
    manage_scope: &BTreeSet<Uuid>,
) -> Result<Vec<DashboardSummaryV1>, DashboardModuleError> {
    if read_scope.is_empty() {
        return Err(DashboardModuleError::Forbidden);
    }
    let ids = read_scope.iter().copied().collect::<Vec<_>>();
    let rows = sqlx::query(
        "SELECT dashboards.id,dashboards.name,dashboards.description,
                (SELECT COUNT(*) FROM dashboard_placements
                 WHERE dashboard_id=dashboards.id) AS placement_count
         FROM dashboards
         WHERE EXISTS (
           SELECT 1 FROM dashboard_scope_nodes
           WHERE dashboard_id=dashboards.id AND node_id=ANY($1)
         )
         ORDER BY dashboards.name,dashboards.id",
    )
    .bind(&ids)
    .fetch_all(&state.pool)
    .await?;
    let mut dashboards = Vec::with_capacity(rows.len());
    for row in rows {
        let id: Uuid = row.try_get("id")?;
        let visibility = load_visibility(&state.pool, id, Some(read_scope)).await?;
        let all_scope = load_scope_ids_pool(&state.pool, id).await?;
        dashboards.push(DashboardSummaryV1 {
            id,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            visibility_nodes: visibility,
            placement_count: row.try_get("placement_count")?,
            can_manage: !all_scope.is_empty()
                && all_scope
                    .iter()
                    .all(|node_id| manage_scope.contains(node_id)),
        });
    }
    Ok(dashboards)
}

async fn load_dashboard(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
    read_scope: &BTreeSet<Uuid>,
    manage_scope: &BTreeSet<Uuid>,
) -> Result<Option<DashboardSummaryV1>, DashboardModuleError> {
    let scope = load_scope_ids_pool(&state.pool, dashboard_id).await?;
    if scope.is_empty() || !scope.iter().any(|id| read_scope.contains(id)) {
        return Ok(None);
    }
    let row = sqlx::query(
        "SELECT id,name,description,
                (SELECT COUNT(*) FROM dashboard_placements
                 WHERE dashboard_id=dashboards.id) AS placement_count
         FROM dashboards WHERE id=$1",
    )
    .bind(dashboard_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(row) = row else {
        return Ok(None);
    };
    Ok(Some(DashboardSummaryV1 {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        visibility_nodes: load_visibility(&state.pool, dashboard_id, Some(read_scope)).await?,
        placement_count: row.try_get("placement_count")?,
        can_manage: scope.iter().all(|node_id| manage_scope.contains(node_id)),
    }))
}

async fn load_visibility(
    pool: &sqlx::PgPool,
    dashboard_id: Uuid,
    filter: Option<&BTreeSet<Uuid>>,
) -> Result<Vec<DashboardVisibilityNodeV1>, DashboardModuleError> {
    let rows = sqlx::query(
        "SELECT nodes.node_id,nodes.node_name,nodes.node_type_name,
                nodes.parent_node_id,nodes.node_path
         FROM dashboard_scope_nodes AS scope
         JOIN dashboard_organization_nodes AS nodes ON nodes.node_id=scope.node_id
         WHERE scope.dashboard_id=$1
         ORDER BY nodes.node_path,nodes.node_id",
    )
    .bind(dashboard_id)
    .fetch_all(pool)
    .await?;
    let mut visibility = Vec::new();
    for row in rows {
        let node_id = row.try_get::<Uuid, _>("node_id")?;
        if filter.is_some_and(|ids| !ids.contains(&node_id)) {
            continue;
        }
        visibility.push(DashboardVisibilityNodeV1 {
            node_id,
            node_name: row.try_get("node_name")?,
            node_type_name: row.try_get("node_type_name")?,
            parent_node_id: row.try_get("parent_node_id")?,
            node_path: row.try_get("node_path")?,
        });
    }
    Ok(visibility)
}

fn validate_dashboard_input(input: &DashboardInputV1) -> Result<(), DashboardModuleError> {
    if input.name.trim().is_empty()
        || input.name.chars().count() > 160
        || input.idempotency_key.trim().is_empty()
        || input.idempotency_key.chars().count() > 200
    {
        return Err(DashboardModuleError::BadRequest(
            "Dashboard name or idempotency key is invalid".into(),
        ));
    }
    Ok(())
}

fn normalized_scope(node_ids: &[Uuid]) -> Result<Vec<Uuid>, DashboardModuleError> {
    if node_ids.is_empty() || node_ids.iter().any(Uuid::is_nil) {
        return Err(DashboardModuleError::BadRequest(
            "at least one valid visibility node is required".into(),
        ));
    }
    let normalized = node_ids.iter().copied().collect::<BTreeSet<_>>();
    if normalized.len() != node_ids.len() {
        return Err(DashboardModuleError::BadRequest(
            "visibility nodes must be unique".into(),
        ));
    }
    Ok(normalized.into_iter().collect())
}

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn mutation_idempotency_key(headers: &HeaderMap) -> Result<&str, DashboardModuleError> {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .ok_or_else(|| DashboardModuleError::BadRequest("missing X-Idempotency-Key".into()))
}

pub(super) fn mutation_digest<T: Serialize>(
    action: &str,
    dashboard_id: Option<Uuid>,
    input: &T,
) -> Result<String, DashboardModuleError> {
    let mut digest = Sha256::new();
    digest.update(action.as_bytes());
    digest.update([0]);
    if let Some(dashboard_id) = dashboard_id {
        digest.update(dashboard_id.as_bytes());
    }
    digest.update([0]);
    digest.update(
        serde_json::to_vec(input)
            .map_err(|_| DashboardModuleError::BadRequest("invalid mutation payload".into()))?,
    );
    Ok(format!("sha256:{:x}", digest.finalize()))
}

pub(super) async fn load_mutation_replay<T: DeserializeOwned>(
    tx: &mut Transaction<'_, Postgres>,
    grant: &AuthorizationGrantV1,
    action: &str,
    idempotency_key: &str,
    payload_digest: &str,
) -> Result<Option<T>, DashboardModuleError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(idempotency_key)
        .execute(&mut **tx)
        .await?;
    let existing = sqlx::query(
        "SELECT jti,original_actor_id,action,payload_digest,idempotency_key,result
         FROM dashboard_mutation_replays
         WHERE jti=$1 OR idempotency_key=$2
         FOR UPDATE",
    )
    .bind(grant.jti)
    .bind(idempotency_key)
    .fetch_optional(&mut **tx)
    .await?;
    let Some(existing) = existing else {
        return Ok(None);
    };
    if existing.try_get::<Uuid, _>("original_actor_id")? != grant.original_actor_id
        || existing.try_get::<String, _>("action")? != action
        || existing.try_get::<String, _>("payload_digest")? != payload_digest
        || existing.try_get::<String, _>("idempotency_key")? != idempotency_key
    {
        return Err(DashboardModuleError::Conflict(
            "mutation replay identity was reused for a different request".into(),
        ));
    }
    let result = existing.try_get::<Value, _>("result")?;
    serde_json::from_value(result)
        .map(Some)
        .map_err(|_| DashboardModuleError::Unavailable("stored mutation result is invalid".into()))
}

pub(super) async fn record_mutation_replay<T: Serialize>(
    tx: &mut Transaction<'_, Postgres>,
    grant: &AuthorizationGrantV1,
    action: &str,
    idempotency_key: &str,
    payload_digest: &str,
    result: &T,
) -> Result<(), DashboardModuleError> {
    sqlx::query(
        "INSERT INTO dashboard_mutation_replays
         (jti,original_actor_id,action,payload_digest,idempotency_key,result)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(grant.jti)
    .bind(grant.original_actor_id)
    .bind(action)
    .bind(payload_digest)
    .bind(idempotency_key)
    .bind(
        serde_json::to_value(result)
            .map_err(|_| DashboardModuleError::BadRequest("invalid mutation result".into()))?,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn require_projected_nodes(
    tx: &mut Transaction<'_, Postgres>,
    ids: &[Uuid],
) -> Result<(), DashboardModuleError> {
    let count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM dashboard_organization_nodes
         WHERE node_id=ANY($1) AND active=true",
    )
    .bind(ids)
    .fetch_one(&mut **tx)
    .await?;
    if count == ids.len() as i64 {
        Ok(())
    } else {
        Err(DashboardModuleError::BadRequest(
            "one or more visibility nodes are not in the current Core projection".into(),
        ))
    }
}

async fn replace_scope(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
    ids: &[Uuid],
) -> Result<(), DashboardModuleError> {
    sqlx::query("DELETE FROM dashboard_scope_nodes WHERE dashboard_id=$1")
        .bind(dashboard_id)
        .execute(&mut **tx)
        .await?;
    for id in ids {
        sqlx::query("INSERT INTO dashboard_scope_nodes (dashboard_id,node_id) VALUES ($1,$2)")
            .bind(dashboard_id)
            .bind(id)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn lock_dashboard(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> Result<(), DashboardModuleError> {
    let exists = sqlx::query_scalar::<_, Uuid>("SELECT id FROM dashboards WHERE id=$1 FOR UPDATE")
        .bind(dashboard_id)
        .fetch_optional(&mut **tx)
        .await?;
    if exists.is_some() {
        Ok(())
    } else {
        Err(DashboardModuleError::NotFound("Dashboard not found".into()))
    }
}

async fn load_scope_ids(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> Result<Vec<Uuid>, DashboardModuleError> {
    Ok(sqlx::query_scalar(
        "SELECT node_id FROM dashboard_scope_nodes
         WHERE dashboard_id=$1 ORDER BY node_id FOR SHARE",
    )
    .bind(dashboard_id)
    .fetch_all(&mut **tx)
    .await?)
}

async fn load_scope_ids_pool(
    pool: &sqlx::PgPool,
    dashboard_id: Uuid,
) -> Result<Vec<Uuid>, DashboardModuleError> {
    Ok(sqlx::query_scalar(
        "SELECT node_id FROM dashboard_scope_nodes
         WHERE dashboard_id=$1 ORDER BY node_id",
    )
    .bind(dashboard_id)
    .fetch_all(pool)
    .await?)
}

#[cfg(test)]
mod tests {
    use super::{
        COMPOSITION_CONTRACT, DASHBOARD_CONTRACT, DashboardInputV1, contract_for_action,
        normalized_scope, validate_dashboard_input,
    };
    use uuid::Uuid;

    #[test]
    fn metadata_requires_nonempty_unique_visibility_and_idempotency() {
        let node = Uuid::new_v4();
        assert!(normalized_scope(&[node]).is_ok());
        assert!(normalized_scope(&[node, node]).is_err());
        assert!(normalized_scope(&[]).is_err());

        assert!(
            validate_dashboard_input(&DashboardInputV1 {
                name: "Operations".into(),
                description: None,
                visibility_node_ids: vec![node],
                idempotency_key: "create-operations".into(),
            })
            .is_ok()
        );
    }

    #[test]
    fn authorization_contract_matches_the_manifest_action_family() {
        assert_eq!(
            contract_for_action("dashboards.load_composition"),
            COMPOSITION_CONTRACT
        );
        assert_eq!(
            contract_for_action("dashboards.reconcile_composition"),
            COMPOSITION_CONTRACT
        );
        assert_eq!(contract_for_action("dashboards.list"), DASHBOARD_CONTRACT);
        assert_eq!(contract_for_action("dashboards.update"), DASHBOARD_CONTRACT);
    }
}
