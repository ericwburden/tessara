//! Dashboard-owned composition read boundary.
//!
//! Placement rows contain only typed ComponentVersion references. Every
//! request resolves metadata through Core's action-bound compatibility
//! adapter; the Dashboard database never joins or copies Components tables.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    body::Body,
    extract::{Path, RawQuery, State},
    http::{HeaderMap, header},
    response::Response,
    routing::get,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_components_contract::{
    COMPONENT_BINDING_KEY, COMPONENT_CONTRACT_ID, COMPONENT_CONTRACT_SCHEMA_VERSION,
    COMPONENT_MODULE_DEFINITION_ID, ComponentAction, ComponentCatalogResponse, ComponentMetadata,
    ComponentPublicationState, ComponentRenderKind, ComponentRenderRequest,
    ComponentResolutionRequest, ComponentResolutionResponse, ComponentVersionReference,
};
use tessara_dashboards::{
    DashboardPlacementConfigInput, DashboardPlacementConfigState, DashboardPlacementConfigV1,
    DashboardPlacementOperation, DashboardPlacementSizePolicy, GridPlacement, GridRect,
    encode_dashboard_placement_config, parse_dashboard_placement_configs,
    validate_dashboard_layout,
};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AuthorizationAudienceV1,
    AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, AuthorizationValidationContextV3, ContractCompatibilityState,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ModuleInstanceOwnerState,
    ModuleServicePrincipalV1, ModuleServiceRequestV1, OwnerDataState, ProviderAvailabilityState,
    ResourceAccessState, ResourceIdentityState, ResourceLifecycleState, ResourceOwner,
    ResourceOwnerState, ResourceResolutionV1, ResourceRevision, SignedEnvelopeV1,
    TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY, READ_CAPABILITY,
    product::{
        DashboardSummaryV1, authorize, authorized_organizations, get_dashboard_summary,
        load_mutation_replay, mutation_digest, record_mutation_replay,
    },
};

#[derive(Clone, Debug, Serialize)]
pub struct DashboardResponseV1 {
    #[serde(flatten)]
    pub summary: DashboardSummaryV1,
    pub placements: Vec<DashboardPlacementResponseV1>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardPlacementResponseV1 {
    pub placement_id: Uuid,
    pub position: i32,
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
    pub availability: DashboardPlacementAvailabilityV1,
    pub resolution_state: &'static str,
    pub resolution: tessara_module_contract::ResourceResolutionV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_state: Option<DashboardPlacementConfigState>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub component: Option<ComponentMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allowed_operations: Option<Vec<DashboardPlacementOperation>>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardPlacementAvailabilityV1 {
    Available,
    Unavailable,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardComponentVersionOptionV1 {
    pub component_reference: ComponentVersionReference,
    pub component_version_id: Uuid,
    pub component_id: Uuid,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_status: String,
    pub default_grid_width: i32,
    pub default_grid_height: i32,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardCompositionResponseV1 {
    pub dashboard: DashboardResponseV1,
    pub available_component_versions: Vec<DashboardComponentVersionOptionV1>,
    pub new_placement_ids: Vec<DashboardPlacementIdMappingV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DashboardPlacementIdMappingV1 {
    pub client_key: String,
    pub placement_id: Uuid,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct DashboardCompositionReplayV1 {
    dashboard_id: Uuid,
    new_placement_ids: Vec<DashboardPlacementIdMappingV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct DashboardPlacementGeometryV1 {
    pub grid_row: i32,
    pub grid_column: i32,
    pub grid_width: i32,
    pub grid_height: i32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum DashboardCompositionCommandV1 {
    Retain {
        placement_id: Uuid,
        geometry: DashboardPlacementGeometryV1,
        #[serde(default)]
        title: Option<String>,
        #[serde(default)]
        repair: bool,
    },
    Bind {
        #[serde(default)]
        placement_id: Option<Uuid>,
        #[serde(default)]
        client_key: Option<String>,
        component_reference: ComponentVersionReference,
        geometry: DashboardPlacementGeometryV1,
        #[serde(default)]
        title: Option<String>,
    },
    Remove {
        placement_id: Uuid,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReconcileDashboardCompositionRequestV1 {
    #[serde(default)]
    pub commands: Vec<DashboardCompositionCommandV1>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardVisibilityNodeOptionV1 {
    pub id: Uuid,
    pub node_type_name: String,
    pub parent_node_name: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardDependencyProjectionV1 {
    pub schema_version: u16,
    pub dashboards: Vec<DashboardDependencyV1>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardDependencyV1 {
    pub dashboard_id: Uuid,
    pub dashboard_name: String,
    pub description: Option<String>,
    pub scope_node_ids: Vec<Uuid>,
    pub placements: Vec<DashboardPlacementDependencyV1>,
}

#[derive(Clone, Debug, Serialize)]
pub struct DashboardPlacementDependencyV1 {
    pub placement_id: Uuid,
    pub component_reference: ComponentVersionReference,
    pub component_version_id: Uuid,
    pub position: i32,
    pub config: Value,
}

struct StoredPlacement {
    id: Uuid,
    position: i32,
    reference: ComponentVersionReference,
    config: Value,
}

pub(super) fn routes() -> Router<DashboardModuleState> {
    Router::new()
        .route("/api/dashboards/{dashboard_id}", get(get_dashboard))
        .route(
            "/api/dashboards/{dashboard_id}/placements/{placement_id}/render/{kind}",
            get(render_placement),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}/composition",
            get(get_composition).put(reconcile_composition),
        )
        .route(
            "/api/admin/dashboards/visibility-nodes",
            get(list_visibility_nodes),
        )
        .route(
            "/api/private/dependency-projection",
            get(dependency_projection),
        )
}

async fn render_placement(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path((dashboard_id, placement_id, kind)): Path<(Uuid, Uuid, String)>,
    RawQuery(query): RawQuery,
) -> Result<Response, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.render_placement",
        tessara_module_contract::AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let dashboard_scope = load_dashboard_scope(&state, dashboard_id).await?;
    let read_scope = authorized_organizations(&grant.payload, READ_CAPABILITY);
    if dashboard_scope.is_empty()
        || !dashboard_scope
            .iter()
            .any(|node_id| read_scope.contains(node_id))
    {
        return Err(DashboardModuleError::Forbidden);
    }
    let row = sqlx::query(
        "SELECT component_reference FROM dashboard_placements
         WHERE id=$1 AND dashboard_id=$2",
    )
    .bind(placement_id)
    .bind(dashboard_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| DashboardModuleError::NotFound("placement not found".into()))?;
    let reference: TypedResourceReference =
        serde_json::from_value(row.try_get("component_reference")?).map_err(|_| {
            DashboardModuleError::Conflict("stored Component reference is invalid".into())
        })?;
    let reference = ComponentVersionReference::new(reference)
        .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?;
    let kind = match kind.as_str() {
        "table" => ComponentRenderKind::Table,
        "bar" => ComponentRenderKind::Bar,
        "line" => ComponentRenderKind::Line,
        "pie" => ComponentRenderKind::Pie,
        "donut" => ComponentRenderKind::Donut,
        "stat-card" => ComponentRenderKind::StatCard,
        _ => {
            return Err(DashboardModuleError::NotFound(
                "render kind not found".into(),
            ));
        }
    };
    let authorization = authorization_header(&headers)?;
    let attempt = resolve_component_since(&state, authorization, reference.clone(), None).await?;
    let resolution = crate::dependencies::project_component_resolution_for_visibility(
        &state,
        dashboard_id,
        placement_id,
        reference.reference(),
        &attempt,
    )
    .await?;
    let metadata = renderable_component_metadata(&resolution)?;
    if metadata.component_type != kind.component_type() {
        return Err(DashboardModuleError::NotFound(
            "render kind not found".into(),
        ));
    }
    let path = "/api/private/components/render";
    let request = ComponentRenderRequest {
        schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
        action: ComponentAction::Render,
        reference,
        kind,
        resource_authority_revision: metadata.authority_revision,
        query: query.unwrap_or_default(),
        dashboard_scope_node_ids: dashboard_scope,
    };
    let body = serde_json::to_vec(&request)
        .map_err(|_| DashboardModuleError::Unavailable("render request encoding failed".into()))?;
    let target_module_instance_id = component_target_instance(authorization, &request.reference)?;
    let downstream = exchange_component_authorization(
        &state,
        authorization,
        target_module_instance_id,
        "components.render",
    )
    .await?
    .ok_or(DashboardModuleError::Forbidden)?;
    let service_request = signed_service_request(&state, &downstream, "POST", path, &body)?;
    let response = state
        .service_client
        .post(format!("{}{path}", state.component_provider_url))
        .header("x-tessara-authorization", &downstream.encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            downstream.correlation_id.to_string(),
        )
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|_| DashboardModuleError::Unavailable("Component render unavailable".into()))?;
    let status = response.status();
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .cloned();
    let bytes = response
        .bytes()
        .await
        .map_err(|_| DashboardModuleError::Unavailable("Component render unavailable".into()))?;
    let mut builder = Response::builder().status(status);
    if let Some(content_type) = content_type
        && let Ok(content_type) = content_type.to_str()
    {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }
    builder
        .body(Body::from(bytes))
        .map_err(|_| DashboardModuleError::Unavailable("Component render response failed".into()))
}

async fn dependency_projection(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
) -> Result<Json<DashboardDependencyProjectionV1>, DashboardModuleError> {
    crate::require_private_key(&headers)?;
    let rows = sqlx::query(
        "SELECT dashboards.id,dashboards.name,dashboards.description,
                COALESCE(array_agg(DISTINCT scope.node_id)
                  FILTER (WHERE scope.node_id IS NOT NULL),'{}') AS scope_node_ids
         FROM dashboards
         LEFT JOIN dashboard_scope_nodes scope ON scope.dashboard_id=dashboards.id
         GROUP BY dashboards.id,dashboards.name,dashboards.description
         ORDER BY dashboards.name,dashboards.id",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut dashboards = Vec::with_capacity(rows.len());
    for row in rows {
        let dashboard_id: Uuid = row.try_get("id")?;
        let placement_rows = sqlx::query(
            "SELECT id,component_reference,position,config
             FROM dashboard_placements
             WHERE dashboard_id=$1
             ORDER BY position,id",
        )
        .bind(dashboard_id)
        .fetch_all(&state.pool)
        .await?;
        let placements = placement_rows
            .into_iter()
            .map(|placement| {
                let reference: TypedResourceReference =
                    serde_json::from_value(placement.try_get("component_reference")?)
                        .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
                let component_version_id = Uuid::parse_str(reference.resource_id())
                    .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
                Ok(DashboardPlacementDependencyV1 {
                    placement_id: placement.try_get("id")?,
                    component_reference: ComponentVersionReference::new(reference)
                        .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
                    component_version_id,
                    position: placement.try_get("position")?,
                    config: placement.try_get("config")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        dashboards.push(DashboardDependencyV1 {
            dashboard_id,
            dashboard_name: row.try_get("name")?,
            description: row.try_get("description")?,
            scope_node_ids: row.try_get("scope_node_ids")?,
            placements,
        });
    }
    Ok(Json(DashboardDependencyProjectionV1 {
        schema_version: 1,
        dashboards,
    }))
}

async fn reconcile_composition(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(input): Json<ReconcileDashboardCompositionRequestV1>,
) -> Result<Json<DashboardCompositionResponseV1>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.reconcile_composition",
        tessara_module_contract::AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let payload_digest = mutation_digest(
        "dashboards.reconcile_composition",
        Some(dashboard_id),
        &input,
    )?;
    let manage_scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    let dashboard_scope = load_dashboard_scope(&state, dashboard_id).await?;
    if dashboard_scope.is_empty()
        || !dashboard_scope
            .iter()
            .all(|node_id| manage_scope.contains(node_id))
    {
        tracing::warn!(
            dashboard_scope_count = dashboard_scope.len(),
            manage_scope_count = manage_scope.len(),
            missing_scope_count = dashboard_scope
                .iter()
                .filter(|node_id| !manage_scope.contains(node_id))
                .count(),
            "Dashboard composition scope is not manageable"
        );
        return Err(DashboardModuleError::Forbidden);
    }
    let authorization = authorization_header(&headers)?;
    let stored = load_stored_placements(&state, dashboard_id).await?;
    let current = stored
        .iter()
        .map(|placement| (placement.id, placement))
        .collect::<BTreeMap<_, _>>();
    let mut current_resolutions = BTreeMap::new();
    for placement in &stored {
        current_resolutions.insert(
            placement.id,
            resolve_component(&state, authorization, placement.reference.clone()).await?,
        );
    }
    let policy = DashboardPlacementSizePolicy::new();
    let parsed = parse_dashboard_placement_configs(
        &stored
            .iter()
            .map(|placement| {
                let kind = current_resolutions
                    .get(&placement.id)
                    .and_then(ComponentResolutionResponse::metadata)
                    .map(|metadata| metadata.component_type.as_str())
                    .unwrap_or("redacted");
                DashboardPlacementConfigInput::new(
                    placement.id,
                    placement.position,
                    placement.config.clone(),
                    policy.minimum_for(kind),
                )
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?
    .into_iter()
    .map(|placement| (placement.placement_id, placement.config))
    .collect::<BTreeMap<_, _>>();

    struct Candidate {
        id: Uuid,
        client_key: Option<String>,
        reference: TypedResourceReference,
        rect: GridRect,
        config: Value,
    }
    let mut candidates = Vec::new();
    let mut seen = BTreeSet::new();
    let mut removed = Vec::new();
    let mut client_keys = BTreeSet::new();
    for command in input.commands {
        match command {
            DashboardCompositionCommandV1::Retain {
                placement_id,
                geometry,
                title,
                repair,
            } => {
                let placement = current.get(&placement_id).ok_or_else(|| {
                    DashboardModuleError::Conflict("placement no longer exists".into())
                })?;
                if !seen.insert(placement_id) {
                    return Err(DashboardModuleError::Conflict(
                        "placement appears more than once".into(),
                    ));
                }
                let parsed = parsed.get(&placement_id).ok_or_else(|| {
                    DashboardModuleError::Conflict("placement configuration missing".into())
                })?;
                let requested = geometry_rect(geometry);
                let resolution = current_resolutions.get(&placement_id).ok_or_else(|| {
                    DashboardModuleError::Unavailable("Component resolution missing".into())
                })?;
                let kind = resolution
                    .metadata()
                    .map(|metadata| metadata.component_type.as_str())
                    .unwrap_or("redacted");
                let config = match parsed.config_state {
                    DashboardPlacementConfigState::FutureSchema => {
                        if repair || title.is_some() || requested != parsed.display_rect {
                            return Err(DashboardModuleError::Conflict(
                                "future-schema placement may only be retained unchanged".into(),
                            ));
                        }
                        parsed.raw_config.clone()
                    }
                    DashboardPlacementConfigState::NeedsRepair if !repair => {
                        if title.is_some() || requested != parsed.display_rect {
                            return Err(DashboardModuleError::Conflict(
                                "malformed placement requires repair before changes".into(),
                            ));
                        }
                        parsed.raw_config.clone()
                    }
                    _ => encode_dashboard_placement_config(
                        &DashboardPlacementConfigV1::new_with_minimum(
                            reconciled_title(title.as_deref(), parsed.title.as_deref()),
                            requested,
                            policy.minimum_for(kind),
                        )
                        .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?,
                    )
                    .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?,
                };
                candidates.push(Candidate {
                    id: placement_id,
                    client_key: None,
                    reference: placement.reference.reference().clone(),
                    rect: if matches!(
                        parsed.config_state,
                        DashboardPlacementConfigState::FutureSchema
                            | DashboardPlacementConfigState::NeedsRepair
                    ) && !repair
                    {
                        parsed.display_rect
                    } else {
                        requested
                    },
                    config,
                });
            }
            DashboardCompositionCommandV1::Bind {
                placement_id,
                client_key,
                component_reference,
                geometry,
                title,
            } => {
                if placement_id.is_some() == client_key.is_some() {
                    return Err(DashboardModuleError::BadRequest(
                        "bind requires exactly one placement_id or client_key".into(),
                    ));
                }
                if let Some(placement_id) = placement_id
                    && (!current.contains_key(&placement_id) || !seen.insert(placement_id))
                {
                    return Err(DashboardModuleError::Conflict(
                        "replacement placement is stale or repeated".into(),
                    ));
                }
                if let Some(client_key) = &client_key
                    && (client_key.trim().is_empty()
                        || client_key.chars().count() > 200
                        || !client_keys.insert(client_key.clone()))
                {
                    return Err(DashboardModuleError::BadRequest(
                        "client_key is invalid or repeated".into(),
                    ));
                }
                if component_reference.reference().installation_id()
                    != grant.payload.installation_id
                {
                    return Err(DashboardModuleError::BadRequest(
                        "ComponentVersion belongs to another installation".into(),
                    ));
                }
                let reference = component_reference.reference().clone();
                let resolution =
                    resolve_component(&state, authorization, component_reference).await?;
                let metadata = resolution.metadata().ok_or_else(|| {
                    DashboardModuleError::Conflict(
                        "ComponentVersion cannot be bound in its current state".into(),
                    )
                })?;
                if !metadata.renderable()
                    || metadata.scope_node_ids.is_empty()
                    || !metadata
                        .scope_node_ids
                        .iter()
                        .all(|node_id| dashboard_scope.contains(node_id))
                {
                    return Err(DashboardModuleError::Conflict(
                        "ComponentVersion scope or lifecycle is incompatible".into(),
                    ));
                }
                let rect = geometry_rect(geometry);
                let config = encode_dashboard_placement_config(
                    &DashboardPlacementConfigV1::new_with_minimum(
                        title
                            .as_deref()
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(str::to_string),
                        rect,
                        policy.minimum_for(&metadata.component_type),
                    )
                    .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?,
                )
                .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
                candidates.push(Candidate {
                    id: placement_id.unwrap_or_else(Uuid::new_v4),
                    client_key,
                    reference,
                    rect,
                    config,
                });
            }
            DashboardCompositionCommandV1::Remove { placement_id } => {
                if !current.contains_key(&placement_id) || !seen.insert(placement_id) {
                    return Err(DashboardModuleError::Conflict(
                        "removed placement is stale or repeated".into(),
                    ));
                }
                removed.push(placement_id);
            }
        }
    }
    if seen.len() != current.len() {
        return Err(DashboardModuleError::Conflict(
            "full-layout request omitted a stored placement".into(),
        ));
    }
    let layout = candidates
        .iter()
        .map(|candidate| GridPlacement::new(candidate.id, candidate.rect))
        .collect::<Vec<_>>();
    validate_dashboard_layout(&layout)
        .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
    let mut order = candidates
        .iter()
        .map(|candidate| (candidate.rect.row, candidate.rect.column, candidate.id))
        .collect::<Vec<_>>();
    order.sort();
    let positions = order
        .into_iter()
        .enumerate()
        .map(|(position, (_, _, id))| (id, position as i32))
        .collect::<BTreeMap<_, _>>();

    let mut tx = state.pool.begin().await?;
    if let Some(replay) = load_mutation_replay::<DashboardCompositionReplayV1>(
        &mut tx,
        &grant.payload,
        "dashboards.reconcile_composition",
        idempotency_key,
        &payload_digest,
    )
    .await?
    {
        if replay.dashboard_id != dashboard_id {
            return Err(DashboardModuleError::Conflict(
                "stored composition replay targets a different Dashboard".into(),
            ));
        }
        tx.commit().await?;
        return load_composition_response(
            &state,
            authorization,
            dashboard_id,
            &manage_scope,
            &dashboard_scope,
            replay.new_placement_ids,
        )
        .await;
    }
    let locked = sqlx::query_scalar::<_, Uuid>("SELECT id FROM dashboards WHERE id=$1 FOR UPDATE")
        .bind(dashboard_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| DashboardModuleError::NotFound("Dashboard not found".into()))?;
    debug_assert_eq!(locked, dashboard_id);
    let locked_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM dashboard_placements WHERE dashboard_id=$1 ORDER BY id FOR UPDATE",
    )
    .bind(dashboard_id)
    .fetch_all(&mut *tx)
    .await?;
    let expected_ids = current.keys().copied().collect::<Vec<_>>();
    if locked_ids != expected_ids {
        return Err(DashboardModuleError::Conflict(
            "Dashboard composition changed during reconciliation".into(),
        ));
    }
    for id in removed {
        sqlx::query("DELETE FROM dashboard_placements WHERE id=$1")
            .bind(id)
            .execute(&mut *tx)
            .await?;
    }
    let mut new_placement_ids = Vec::new();
    for candidate in candidates {
        let position = positions[&candidate.id];
        if current.contains_key(&candidate.id) {
            sqlx::query(
                "UPDATE dashboard_placements
                 SET component_reference=$2,position=$3,config=$4,updated_at=now()
                 WHERE id=$1",
            )
            .bind(candidate.id)
            .bind(serde_json::to_value(&candidate.reference).map_err(|_| {
                DashboardModuleError::BadRequest("Component reference is invalid".into())
            })?)
            .bind(position)
            .bind(candidate.config)
            .execute(&mut *tx)
            .await?;
        } else {
            sqlx::query(
                "INSERT INTO dashboard_placements
                 (id,dashboard_id,component_reference,position,config)
                 VALUES ($1,$2,$3,$4,$5)",
            )
            .bind(candidate.id)
            .bind(dashboard_id)
            .bind(serde_json::to_value(&candidate.reference).map_err(|_| {
                DashboardModuleError::BadRequest("Component reference is invalid".into())
            })?)
            .bind(position)
            .bind(candidate.config)
            .execute(&mut *tx)
            .await?;
            if let Some(client_key) = candidate.client_key {
                new_placement_ids.push(DashboardPlacementIdMappingV1 {
                    client_key,
                    placement_id: candidate.id,
                });
            }
        }
    }
    record_mutation_replay(
        &mut tx,
        &grant.payload,
        "dashboards.reconcile_composition",
        idempotency_key,
        &payload_digest,
        &DashboardCompositionReplayV1 {
            dashboard_id,
            new_placement_ids: new_placement_ids.clone(),
        },
    )
    .await?;
    tx.commit().await?;

    load_composition_response(
        &state,
        authorization,
        dashboard_id,
        &manage_scope,
        &dashboard_scope,
        new_placement_ids,
    )
    .await
}

async fn load_composition_response(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
    manage_scope: &BTreeSet<Uuid>,
    dashboard_scope: &[Uuid],
    new_placement_ids: Vec<DashboardPlacementIdMappingV1>,
) -> Result<Json<DashboardCompositionResponseV1>, DashboardModuleError> {
    let summary = get_dashboard_summary_with_grant(state, dashboard_id, manage_scope).await?;
    let placements =
        load_placements_with_authorization(state, authorization, dashboard_id, true).await?;
    let policy = DashboardPlacementSizePolicy::new();
    let catalog = component_catalog(state, authorization).await?;
    let available_component_versions = catalog
        .components
        .into_iter()
        .filter(|component| {
            !component.scope_node_ids.is_empty()
                && component
                    .scope_node_ids
                    .iter()
                    .all(|node_id| dashboard_scope.contains(node_id))
        })
        .map(|component| {
            let recommended = policy.recommended_for(&component.component_type);
            DashboardComponentVersionOptionV1 {
                component_reference: component.reference,
                component_version_id: component.component_version_id,
                component_id: component.component_id,
                component_name: component.component_name,
                component_slug: component.component_slug,
                component_type: component.component_type,
                version_number: component.version_number,
                version_label: component.version_label,
                version_status: publication_state_label(component.publication_state).into(),
                default_grid_width: recommended.width,
                default_grid_height: recommended.height,
            }
        })
        .collect();
    Ok(Json(DashboardCompositionResponseV1 {
        dashboard: DashboardResponseV1 {
            summary,
            placements,
        },
        available_component_versions,
        new_placement_ids,
    }))
}

pub(super) async fn get_dashboard(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DashboardResponseV1>, DashboardModuleError> {
    let summary = get_dashboard_summary(State(state.clone()), headers.clone(), Path(dashboard_id))
        .await?
        .0;
    let placements = load_placements(&state, &headers, dashboard_id, false).await?;
    Ok(Json(DashboardResponseV1 {
        summary,
        placements,
    }))
}

pub(super) async fn get_composition(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<DashboardCompositionResponseV1>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.load_composition",
        tessara_module_contract::AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let manage_scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    let dashboard_scope = load_dashboard_scope(&state, dashboard_id).await?;
    if dashboard_scope.is_empty()
        || !dashboard_scope
            .iter()
            .all(|node_id| manage_scope.contains(node_id))
    {
        return Err(DashboardModuleError::Forbidden);
    }
    let summary = get_dashboard_summary_with_grant(&state, dashboard_id, &manage_scope).await?;
    let authorization = authorization_header(&headers)?;
    let placements =
        load_placements_with_authorization(&state, authorization, dashboard_id, true).await?;
    let catalog = component_catalog(&state, authorization).await?;
    let policy = DashboardPlacementSizePolicy::new();
    let available_component_versions = catalog
        .components
        .into_iter()
        .filter(|component| {
            !component.scope_node_ids.is_empty()
                && component
                    .scope_node_ids
                    .iter()
                    .all(|node_id| dashboard_scope.contains(node_id))
        })
        .map(|component| {
            let recommended = policy.recommended_for(&component.component_type);
            DashboardComponentVersionOptionV1 {
                component_reference: component.reference,
                component_version_id: component.component_version_id,
                component_id: component.component_id,
                component_name: component.component_name,
                component_slug: component.component_slug,
                component_type: component.component_type,
                version_number: component.version_number,
                version_label: component.version_label,
                version_status: publication_state_label(component.publication_state).into(),
                default_grid_width: recommended.width,
                default_grid_height: recommended.height,
            }
        })
        .collect();
    Ok(Json(DashboardCompositionResponseV1 {
        dashboard: DashboardResponseV1 {
            summary,
            placements,
        },
        available_component_versions,
        new_placement_ids: Vec::new(),
    }))
}

pub(super) async fn list_visibility_nodes(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DashboardVisibilityNodeOptionV1>>, DashboardModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "dashboards.list_manageable",
        tessara_module_contract::AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let scope = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    if scope.is_empty() {
        return Err(DashboardModuleError::Forbidden);
    }
    Ok(Json(load_visibility_nodes_for_scope(&state, scope).await?))
}

pub(super) async fn load_visibility_nodes_for_scope(
    state: &DashboardModuleState,
    scope: BTreeSet<Uuid>,
) -> Result<Vec<DashboardVisibilityNodeOptionV1>, DashboardModuleError> {
    let ids = scope.into_iter().collect::<Vec<_>>();
    let rows = sqlx::query(
        "SELECT child.node_id AS id,child.node_type_name,
                parent.node_name AS parent_node_name,child.node_name AS name
         FROM dashboard_organization_nodes child
         LEFT JOIN dashboard_organization_nodes parent
           ON parent.node_id=child.parent_node_id
         WHERE child.node_id=ANY($1) AND child.active=true
         ORDER BY child.node_path,child.node_id",
    )
    .bind(ids)
    .fetch_all(&state.pool)
    .await?;
    let options = rows
        .into_iter()
        .map(|row| {
            Ok(DashboardVisibilityNodeOptionV1 {
                id: row.try_get("id")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_name: row.try_get("parent_node_name")?,
                name: row.try_get("name")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(options)
}

async fn load_placements(
    state: &DashboardModuleState,
    headers: &HeaderMap,
    dashboard_id: Uuid,
    editor: bool,
) -> Result<Vec<DashboardPlacementResponseV1>, DashboardModuleError> {
    load_placements_with_authorization(state, authorization_header(headers)?, dashboard_id, editor)
        .await
}

async fn load_placements_with_authorization(
    state: &DashboardModuleState,
    authorization: &str,
    dashboard_id: Uuid,
    editor: bool,
) -> Result<Vec<DashboardPlacementResponseV1>, DashboardModuleError> {
    let stored = load_stored_placements(state, dashboard_id).await?;
    let mut resolutions = BTreeMap::new();
    for placement in &stored {
        let attempt =
            resolve_component_since(state, authorization, placement.reference.clone(), None)
                .await?;
        let response = crate::dependencies::project_component_resolution_for_visibility(
            state,
            dashboard_id,
            placement.id,
            placement.reference.reference(),
            &attempt,
        )
        .await?;
        resolutions.insert(placement.id, response);
    }
    let policy = DashboardPlacementSizePolicy::new();
    let config_inputs = stored
        .iter()
        .map(|placement| {
            let kind = resolutions
                .get(&placement.id)
                .and_then(ComponentResolutionResponse::metadata)
                .map(|metadata| metadata.component_type.as_str())
                .unwrap_or("redacted");
            DashboardPlacementConfigInput::new(
                placement.id,
                placement.position,
                placement.config.clone(),
                policy.minimum_for(kind),
            )
        })
        .collect::<Vec<_>>();
    let parsed = parse_dashboard_placement_configs(&config_inputs)
        .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?
        .into_iter()
        .map(|placement| (placement.placement_id, placement.config))
        .collect::<BTreeMap<_, _>>();
    stored
        .into_iter()
        .map(|placement| {
            let resolution = resolutions.remove(&placement.id).ok_or_else(|| {
                DashboardModuleError::Unavailable("Component resolution missing".into())
            })?;
            let metadata = resolution.metadata().cloned();
            let parsed = parsed.get(&placement.id).ok_or_else(|| {
                DashboardModuleError::Conflict("placement configuration missing".into())
            })?;
            let state = resolution_state(resolution.resolution());
            let available = parsed.is_executable() && matches!(state, "available" | "superseded");
            let allowed_operations = editor.then(|| match parsed.config_state {
                DashboardPlacementConfigState::FutureSchema => vec![
                    DashboardPlacementOperation::Retain,
                    DashboardPlacementOperation::Remove,
                ],
                DashboardPlacementConfigState::NeedsRepair => vec![
                    DashboardPlacementOperation::Retain,
                    DashboardPlacementOperation::Repair,
                    DashboardPlacementOperation::Remove,
                ],
                DashboardPlacementConfigState::Valid | DashboardPlacementConfigState::Legacy
                    if available =>
                {
                    vec![
                        DashboardPlacementOperation::Retain,
                        DashboardPlacementOperation::Move,
                        DashboardPlacementOperation::Resize,
                        DashboardPlacementOperation::Retitle,
                        DashboardPlacementOperation::Replace,
                        DashboardPlacementOperation::Preview,
                        DashboardPlacementOperation::Remove,
                    ]
                }
                _ => vec![
                    DashboardPlacementOperation::Retain,
                    DashboardPlacementOperation::Move,
                    DashboardPlacementOperation::Resize,
                    DashboardPlacementOperation::Remove,
                ],
            });
            Ok(DashboardPlacementResponseV1 {
                placement_id: placement.id,
                position: placement.position,
                grid_row: parsed.display_rect.row,
                grid_column: parsed.display_rect.column,
                grid_width: parsed.display_rect.width,
                grid_height: parsed.display_rect.height,
                availability: if available {
                    DashboardPlacementAvailabilityV1::Available
                } else {
                    DashboardPlacementAvailabilityV1::Unavailable
                },
                resolution_state: state,
                resolution: resolution.resolution().clone(),
                config_state: editor.then_some(parsed.config_state),
                title: disclosed_title(resolution.resolution(), &parsed.title),
                component: metadata,
                allowed_operations,
            })
        })
        .collect()
}

async fn load_stored_placements(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
) -> Result<Vec<StoredPlacement>, DashboardModuleError> {
    let rows = sqlx::query(
        "SELECT id,position,component_reference,config
         FROM dashboard_placements WHERE dashboard_id=$1 ORDER BY position,id",
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let reference: TypedResourceReference =
                serde_json::from_value(row.try_get("component_reference")?).map_err(|_| {
                    DashboardModuleError::Conflict("stored Component reference is invalid".into())
                })?;
            let reference = ComponentVersionReference::new(reference)
                .map_err(|error| DashboardModuleError::Conflict(error.to_string()))?;
            Ok(StoredPlacement {
                id: row.try_get("id")?,
                position: row.try_get("position")?,
                reference,
                config: row.try_get("config")?,
            })
        })
        .collect()
}

async fn resolve_component(
    state: &DashboardModuleState,
    authorization: &str,
    reference: ComponentVersionReference,
) -> Result<ComponentResolutionResponse, DashboardModuleError> {
    let attempt = resolve_component_since(state, authorization, reference, None).await?;
    if attempt.origin == ComponentResolutionOrigin::SyntheticUnavailable {
        return Err(DashboardModuleError::Unavailable(
            "Component provider unavailable".into(),
        ));
    }
    Ok(attempt.response)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct ComponentAuthorizationContext {
    digest: String,
    expires_at: DateTime<Utc>,
}

impl ComponentAuthorizationContext {
    pub(super) fn digest(&self) -> &str {
        &self.digest
    }

    pub(super) const fn expires_at(&self) -> DateTime<Utc> {
        self.expires_at
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ComponentResolutionOrigin {
    ProviderEvaluated,
    SyntheticUnavailable,
    AuthorizationRestricted,
}

impl ComponentResolutionOrigin {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            Self::ProviderEvaluated => "provider_evaluated",
            Self::SyntheticUnavailable => "synthetic_unavailable",
            Self::AuthorizationRestricted => "authorization_restricted",
        }
    }
}

#[derive(Clone, Debug)]
pub(super) struct ComponentResolutionAttempt {
    response: ComponentResolutionResponse,
    authorization_context: Option<ComponentAuthorizationContext>,
    origin: ComponentResolutionOrigin,
}

impl ComponentResolutionAttempt {
    pub(super) const fn response(&self) -> &ComponentResolutionResponse {
        &self.response
    }

    pub(super) const fn authorization_context(&self) -> Option<&ComponentAuthorizationContext> {
        self.authorization_context.as_ref()
    }

    pub(super) const fn origin(&self) -> ComponentResolutionOrigin {
        self.origin
    }
}

struct DownstreamAuthorization {
    encoded: String,
    installation_id: Uuid,
    caller_module_instance_id: Uuid,
    correlation_id: Uuid,
    context: ComponentAuthorizationContext,
}

pub(super) async fn resolve_component_since(
    state: &DashboardModuleState,
    authorization: &str,
    reference: ComponentVersionReference,
    changes_since_revision: Option<ResourceRevision>,
) -> Result<ComponentResolutionAttempt, DashboardModuleError> {
    let path = "/api/private/components/resolve";
    let target_module_instance_id = component_target_instance(authorization, &reference)?;
    let request = ComponentResolutionRequest::new(
        ComponentAction::ResolveMetadata,
        reference,
        changes_since_revision,
    );
    let body = serde_json::to_vec(&request)
        .map_err(|_| DashboardModuleError::Unavailable("service request encoding failed".into()))?;
    let Some(downstream) = exchange_component_authorization(
        state,
        authorization,
        target_module_instance_id,
        "components.resolve",
    )
    .await?
    else {
        return Ok(ComponentResolutionAttempt {
            response: restricted_component_resolution(ResourceAccessState::Unauthorized),
            authorization_context: None,
            origin: ComponentResolutionOrigin::AuthorizationRestricted,
        });
    };
    let service_request = signed_service_request(state, &downstream, "POST", path, &body)?;
    let response = state
        .service_client
        .post(format!("{}{path}", state.component_provider_url))
        .header("x-tessara-authorization", &downstream.encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            downstream.correlation_id.to_string(),
        )
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await;
    let Ok(response) = response else {
        tracing::warn!("Components provider could not be reached; degrading Dashboard placement");
        return Ok(ComponentResolutionAttempt {
            response: restricted_component_resolution(ResourceAccessState::NotEvaluated),
            authorization_context: Some(downstream.context),
            origin: ComponentResolutionOrigin::SyntheticUnavailable,
        });
    };
    if response.status() == reqwest::StatusCode::FORBIDDEN {
        return Ok(ComponentResolutionAttempt {
            response: restricted_component_resolution(ResourceAccessState::Unauthorized),
            authorization_context: Some(downstream.context),
            origin: ComponentResolutionOrigin::ProviderEvaluated,
        });
    }
    let Ok(response) = response.error_for_status() else {
        tracing::warn!("Components provider rejected resolution; degrading Dashboard placement");
        return Ok(ComponentResolutionAttempt {
            response: restricted_component_resolution(ResourceAccessState::NotEvaluated),
            authorization_context: Some(downstream.context),
            origin: ComponentResolutionOrigin::SyntheticUnavailable,
        });
    };
    match response.json().await {
        Ok(resolution) => Ok(ComponentResolutionAttempt {
            response: resolution,
            authorization_context: Some(downstream.context),
            origin: ComponentResolutionOrigin::ProviderEvaluated,
        }),
        Err(_) => {
            tracing::warn!(
                "Components provider returned invalid resolution; degrading Dashboard placement"
            );
            Ok(ComponentResolutionAttempt {
                response: restricted_component_resolution(ResourceAccessState::NotEvaluated),
                authorization_context: Some(downstream.context),
                origin: ComponentResolutionOrigin::SyntheticUnavailable,
            })
        }
    }
}

pub(super) async fn component_authorization_context(
    state: &DashboardModuleState,
    authorization: &str,
    reference: &ComponentVersionReference,
) -> Result<Option<ComponentAuthorizationContext>, DashboardModuleError> {
    let target_module_instance_id = component_target_instance(authorization, reference)?;
    Ok(exchange_component_authorization(
        state,
        authorization,
        target_module_instance_id,
        "components.resolve",
    )
    .await?
    .map(|downstream| downstream.context))
}

async fn component_catalog(
    state: &DashboardModuleState,
    authorization: &str,
) -> Result<ComponentCatalogResponse, DashboardModuleError> {
    let path = "/api/private/components/catalog";
    let inbound: SignedEnvelopeV1<AuthorizationGrantV3> = decode_header_envelope(authorization)?;
    let target_module_instance_id = tessara_composition::module_instance_id(
        inbound.payload.installation_id,
        COMPONENT_MODULE_DEFINITION_ID,
    );
    let Some(downstream) = exchange_component_authorization(
        state,
        authorization,
        target_module_instance_id,
        "components.catalog",
    )
    .await?
    else {
        return Ok(unavailable_component_catalog());
    };
    let service_request = signed_service_request(state, &downstream, "POST", path, &[])?;
    let response = state
        .service_client
        .post(format!("{}{path}", state.component_provider_url))
        .header("x-tessara-authorization", &downstream.encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            downstream.correlation_id.to_string(),
        )
        .send()
        .await;
    let Ok(response) = response else {
        tracing::warn!("Components catalog could not be reached; continuing without add options");
        return Ok(unavailable_component_catalog());
    };
    let Ok(response) = response.error_for_status() else {
        tracing::warn!("Components catalog request failed; continuing without add options");
        return Ok(unavailable_component_catalog());
    };
    match response.json().await {
        Ok(catalog) => Ok(catalog),
        Err(_) => {
            tracing::warn!(
                "Components catalog response was invalid; continuing without add options"
            );
            Ok(unavailable_component_catalog())
        }
    }
}

fn signed_service_request(
    state: &DashboardModuleState,
    authorization: &DownstreamAuthorization,
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<String, DashboardModuleError> {
    signed_service_request_for_identity(
        state,
        &authorization.encoded,
        authorization.installation_id,
        authorization.caller_module_instance_id,
        authorization.correlation_id,
        method,
        path,
        body,
    )
}

#[allow(clippy::too_many_arguments)]
fn signed_service_request_for_identity(
    state: &DashboardModuleState,
    authorization: &str,
    installation_id: Uuid,
    caller_module_instance_id: Uuid,
    correlation_id: Uuid,
    method: &str,
    path: &str,
    body: &[u8],
) -> Result<String, DashboardModuleError> {
    let now = Utc::now();
    let request = ModuleServiceRequestV1 {
        schema_version: 1,
        installation_id,
        module_instance_id: caller_module_instance_id,
        module_definition_id: tessara_module_contract::ModuleDefinitionId::new(
            crate::MODULE_DEFINITION_ID,
        )
        .map_err(|_| DashboardModuleError::Unavailable("service identity is invalid".into()))?,
        method: method.into(),
        path: path.into(),
        canonical_body_digest: sha256_hex(body),
        inbound_grant_digest: sha256_hex(authorization.as_bytes()),
        correlation_id: correlation_id.to_string(),
        nonce: Uuid::new_v4(),
        issued_at: now,
        expires_at: now + Duration::seconds(30),
    };
    let envelope = state
        .service_request_signer
        .sign(request)
        .map_err(|_| DashboardModuleError::Unavailable("service request signing failed".into()))?;
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|_| DashboardModuleError::Unavailable("service request encoding failed".into()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn component_target_instance(
    authorization: &str,
    reference: &ComponentVersionReference,
) -> Result<Uuid, DashboardModuleError> {
    let inbound: SignedEnvelopeV1<AuthorizationGrantV3> = decode_header_envelope(authorization)?;
    let typed = reference.reference();
    if typed.installation_id() != inbound.payload.installation_id {
        return Err(DashboardModuleError::Forbidden);
    }
    match typed.owner() {
        ResourceOwner::ModuleInstance {
            installation_id,
            module_instance_id,
        } if *installation_id == inbound.payload.installation_id => Ok(*module_instance_id),
        _ => Err(DashboardModuleError::Forbidden),
    }
}

async fn exchange_component_authorization(
    state: &DashboardModuleState,
    inbound_authorization: &str,
    target_module_instance_id: Uuid,
    action: &str,
) -> Result<Option<DownstreamAuthorization>, DashboardModuleError> {
    let inbound: SignedEnvelopeV1<AuthorizationGrantV3> =
        decode_header_envelope(inbound_authorization)?;
    state
        .core_authorization_verifier
        .verify(&inbound)
        .map_err(|_| DashboardModuleError::Forbidden)?;
    let security = crate::load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DashboardModuleError::Unavailable("security state unavailable".into()))?;
    inbound
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
                    .map_err(|_| DashboardModuleError::Forbidden)?,
            },
            dependency_binding: inbound.payload.dependency_binding.clone(),
            functional_contract: inbound.payload.functional_contract.clone(),
            action: inbound.payload.action.clone(),
            operation: inbound.payload.operation,
            resource_assertion: inbound.payload.resource_assertion.clone(),
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| DashboardModuleError::Forbidden)?;
    let target = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: target_module_instance_id,
        module_definition_id: ModuleDefinitionId::new(COMPONENT_MODULE_DEFINITION_ID)
            .map_err(|_| DashboardModuleError::Forbidden)?,
    };
    let request = AuthorizationExchangeRequestV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        target: target.clone(),
        dependency_binding: DependencyBindingKey::new(COMPONENT_BINDING_KEY)
            .map_err(|_| DashboardModuleError::Forbidden)?,
        functional_contract: FunctionalContractId::new(COMPONENT_CONTRACT_ID)
            .map_err(|_| DashboardModuleError::Forbidden)?,
        action: action.into(),
        resource_assertion: None,
    };
    request
        .validate()
        .map_err(|_| DashboardModuleError::Forbidden)?;
    let path = "/api/private/module-authorization/exchange";
    let body = serde_json::to_vec(&request).map_err(|_| {
        DashboardModuleError::Unavailable("authorization exchange encoding failed".into())
    })?;
    let service_request = signed_service_request_for_identity(
        state,
        inbound_authorization,
        security.installation_id,
        security.module_instance_id,
        inbound.payload.correlation_id,
        "POST",
        path,
        &body,
    )?;
    let response = state
        .service_client
        .post(format!("{}{path}", state.core_internal_url))
        .header("content-type", "application/json")
        .header("x-tessara-authorization", inbound_authorization)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            inbound.payload.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
        .map_err(|_| {
            DashboardModuleError::Unavailable("Core authorization exchange unavailable".into())
        })?;
    if matches!(
        response.status(),
        reqwest::StatusCode::FORBIDDEN | reqwest::StatusCode::NOT_FOUND
    ) {
        return Ok(None);
    }
    let response = response.error_for_status().map_err(|error| {
        if error
            .status()
            .is_some_and(|status| status.is_server_error())
        {
            DashboardModuleError::Unavailable("Core authorization exchange unavailable".into())
        } else {
            DashboardModuleError::Forbidden
        }
    })?;
    let response: AuthorizationExchangeResponseV2 = response.json().await.map_err(|_| {
        DashboardModuleError::Unavailable("Core authorization exchange response is invalid".into())
    })?;
    if response.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2 {
        return Err(DashboardModuleError::Forbidden);
    }
    state
        .core_authorization_verifier
        .verify(&response.authorization)
        .map_err(|_| DashboardModuleError::Forbidden)?;
    let presenting_service = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
            .map_err(|_| DashboardModuleError::Forbidden)?,
    };
    response
        .authorization
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service,
            audience: target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| DashboardModuleError::Forbidden)?;
    if response.authorization.payload.original_actor_id != inbound.payload.original_actor_id {
        return Err(DashboardModuleError::Forbidden);
    }
    let context = semantic_authorization_context(&response.authorization.payload)?;
    let encoded = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&response.authorization).map_err(|_| DashboardModuleError::Forbidden)?,
    );
    Ok(Some(DownstreamAuthorization {
        encoded,
        installation_id: security.installation_id,
        caller_module_instance_id: security.module_instance_id,
        correlation_id: response.authorization.payload.correlation_id,
        context,
    }))
}

fn semantic_authorization_context(
    grant: &AuthorizationGrantV3,
) -> Result<ComponentAuthorizationContext, DashboardModuleError> {
    let mut capability_scope_bindings = grant.capability_scope_bindings.clone();
    for binding in &mut capability_scope_bindings {
        binding.authorized_organization_ids.sort_unstable();
    }
    capability_scope_bindings.sort_by_cached_key(|binding| {
        serde_json::to_string(binding).unwrap_or_else(|_| format!("{binding:?}"))
    });

    let mut delegation_basis = grant.delegation_basis.clone();
    delegation_basis.sort_by_cached_key(|basis| {
        serde_json::to_string(basis).unwrap_or_else(|_| format!("{basis:?}"))
    });

    let mut resource_assertion = grant.resource_assertion.clone();
    if let Some(assertion) = &mut resource_assertion {
        assertion.governing_organization_ids.sort_unstable();
    }

    let projection = json!({
        "schema_version": grant.schema_version,
        "installation_id": grant.installation_id,
        "original_actor_id": grant.original_actor_id,
        "presenting_service": grant.presenting_service,
        "audience": grant.audience,
        "dependency_binding": grant.dependency_binding,
        "functional_contract": grant.functional_contract,
        "action": grant.action,
        "operation": grant.operation,
        "capability_scope_bindings": capability_scope_bindings,
        "resource_assertion": resource_assertion,
        "delegation_basis": delegation_basis,
        "authorization_revision": grant.authorization_revision,
        "organization_revision": grant.organization_revision,
    });
    let bytes = serde_json::to_vec(&projection).map_err(|_| {
        DashboardModuleError::Unavailable("Component authorization context encoding failed".into())
    })?;
    Ok(ComponentAuthorizationContext {
        digest: format!("sha256:{}", sha256_hex(&bytes)),
        expires_at: grant.expires_at,
    })
}

fn decode_header_envelope<T: serde::de::DeserializeOwned>(
    value: &str,
) -> Result<T, DashboardModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| DashboardModuleError::Forbidden)?;
    serde_json::from_slice(&bytes).map_err(|_| DashboardModuleError::Forbidden)
}

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn restricted_component_resolution(
    access_state: ResourceAccessState,
) -> ComponentResolutionResponse {
    ComponentResolutionResponse::new(
        ResourceResolutionV1::restricted(access_state)
            .expect("restricted Component resolution is valid"),
        None,
        None,
        Vec::new(),
        None,
    )
    .expect("restricted Dashboard response is metadata-free")
}

fn renderable_component_metadata(
    response: &ComponentResolutionResponse,
) -> Result<&ComponentMetadata, DashboardModuleError> {
    let resolution = response.resolution();
    if resolution.access_state() != ResourceAccessState::Authorized {
        return Err(DashboardModuleError::Forbidden);
    }
    if resolution.availability_state() == ProviderAvailabilityState::Unavailable {
        return Err(DashboardModuleError::Unavailable(
            "Component provider unavailable".into(),
        ));
    }
    response
        .metadata()
        .filter(|metadata| metadata.renderable())
        .ok_or(DashboardModuleError::Forbidden)
}

fn unavailable_component_catalog() -> ComponentCatalogResponse {
    ComponentCatalogResponse {
        schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
        components: Vec::new(),
    }
}

const fn publication_state_label(state: ComponentPublicationState) -> &'static str {
    match state {
        ComponentPublicationState::Draft => "draft",
        ComponentPublicationState::Published => "published",
        ComponentPublicationState::Superseded => "superseded",
    }
}

async fn get_dashboard_summary_with_grant(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
    manage_scope: &BTreeSet<Uuid>,
) -> Result<DashboardSummaryV1, DashboardModuleError> {
    let scope = load_dashboard_scope(state, dashboard_id).await?;
    let row = sqlx::query(
        "SELECT id,name,description,
                (SELECT COUNT(*) FROM dashboard_placements
                 WHERE dashboard_id=dashboards.id) AS placement_count
         FROM dashboards WHERE id=$1",
    )
    .bind(dashboard_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| DashboardModuleError::NotFound("Dashboard not found".into()))?;
    let visibility_rows = sqlx::query(
        "SELECT node_id,node_name,node_type_name,parent_node_id,node_path
         FROM dashboard_organization_nodes WHERE node_id=ANY($1)
         ORDER BY node_path,node_id",
    )
    .bind(&scope)
    .fetch_all(&state.pool)
    .await?;
    let visibility_nodes = visibility_rows
        .into_iter()
        .map(|row| {
            Ok(crate::product::DashboardVisibilityNodeV1 {
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_id: row.try_get("parent_node_id")?,
                node_path: row.try_get("node_path")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(DashboardSummaryV1 {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        visibility_nodes,
        placement_count: row.try_get("placement_count")?,
        can_manage: !scope.is_empty() && scope.iter().all(|id| manage_scope.contains(id)),
    })
}

pub(super) async fn load_dashboard_scope(
    state: &DashboardModuleState,
    dashboard_id: Uuid,
) -> Result<Vec<Uuid>, DashboardModuleError> {
    Ok(sqlx::query_scalar(
        "SELECT node_id FROM dashboard_scope_nodes WHERE dashboard_id=$1 ORDER BY node_id",
    )
    .bind(dashboard_id)
    .fetch_all(&state.pool)
    .await?)
}

pub(super) fn authorization_header(headers: &HeaderMap) -> Result<&str, DashboardModuleError> {
    headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(DashboardModuleError::Forbidden)
}

fn mutation_idempotency_key(headers: &HeaderMap) -> Result<&str, DashboardModuleError> {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .ok_or_else(|| {
            DashboardModuleError::BadRequest("valid x-idempotency-key header is required".into())
        })
}

fn geometry_rect(geometry: DashboardPlacementGeometryV1) -> GridRect {
    GridRect::new(
        geometry.grid_row,
        geometry.grid_column,
        geometry.grid_width,
        geometry.grid_height,
    )
}

fn reconciled_title(requested: Option<&str>, current: Option<&str>) -> Option<String> {
    match requested {
        Some(title) => {
            let title = title.trim();
            (!title.is_empty()).then(|| title.to_string())
        }
        None => current.map(str::to_owned),
    }
}

fn resolution_state(resolution: &tessara_module_contract::ResourceResolutionV1) -> &'static str {
    if resolution.access_state() != ResourceAccessState::Authorized {
        return "restricted";
    }
    if resolution.availability_state() == ProviderAvailabilityState::Unavailable {
        return "provider_unavailable";
    }
    if resolution.compatibility_state() == ContractCompatibilityState::Incompatible {
        return "incompatible";
    }
    match resolution.owner_state() {
        ResourceOwnerState::ModuleInstance {
            data_state: OwnerDataState::OwnerDataDestroyed,
            ..
        } => return "owner_data_destroyed",
        ResourceOwnerState::ModuleInstance {
            instance_state: ModuleInstanceOwnerState::OwnerModuleInstanceTombstoned,
            ..
        } => return "owner_tombstoned",
        _ => {}
    }
    if resolution.resource_identity_state() == ResourceIdentityState::UnknownResource {
        return "missing";
    }
    match resolution.resource_lifecycle_state() {
        ResourceLifecycleState::ProviderDefined { state }
            if matches!(state.as_str(), "draft" | "inactive") =>
        {
            "inactive"
        }
        ResourceLifecycleState::ProviderDefined { state } if state == "superseded" => "superseded",
        ResourceLifecycleState::ProviderDefined { state } if state == "tombstoned" => "tombstoned",
        ResourceLifecycleState::ProviderDefined { .. } => "available",
        ResourceLifecycleState::NotEvaluated => "not_evaluated",
        ResourceLifecycleState::Undisclosed => "restricted",
    }
}

fn disclosed_title(
    resolution: &tessara_module_contract::ResourceResolutionV1,
    title: &Option<String>,
) -> Option<String> {
    (resolution.access_state() == ResourceAccessState::Authorized)
        .then(|| title.clone())
        .flatten()
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use tessara_module_contract::{
        AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, AuthorizationAudienceV1,
        AuthorizationGrantOperationV1, AuthorizationGrantV3, CapabilityScopeBindingV1,
        DelegationBasisV1, DependencyBindingKey, FunctionalContractId, ModuleDefinitionId,
        ModuleServicePrincipalV1, ResourceAccessState, ResourceAuthorizationAssertionV2,
        ResourceResolutionV1, ResourceTypeId, SecurityCapabilityId,
    };
    use uuid::Uuid;

    use super::{
        ComponentResolutionResponse, disclosed_title, reconciled_title,
        renderable_component_metadata, semantic_authorization_context,
        unavailable_component_catalog,
    };

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn authorization_grant() -> AuthorizationGrantV3 {
        let now = Utc::now();
        AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: id(1),
            original_actor_id: id(2),
            correlation_id: id(3),
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: id(4),
                module_definition_id: ModuleDefinitionId::new("tessara.dashboards")
                    .expect("module definition"),
            },
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: id(5),
                module_definition_id: ModuleDefinitionId::new("tessara.components")
                    .expect("module definition"),
            },
            dependency_binding: DependencyBindingKey::new("tessara.dashboards.component-version")
                .expect("dependency binding"),
            functional_contract: FunctionalContractId::new(
                "tessara.components.component-resolution",
            )
            .expect("functional contract"),
            action: "components.resolve".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![
                CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("components:read").expect("capability"),
                    organization_root_id: id(10),
                    authorized_organization_ids: vec![id(11), id(12)],
                },
                CapabilityScopeBindingV1 {
                    capability: SecurityCapabilityId::new("datasets:read").expect("capability"),
                    organization_root_id: id(20),
                    authorized_organization_ids: vec![id(21)],
                },
            ],
            resource_assertion: Some(ResourceAuthorizationAssertionV2 {
                resource_type: ResourceTypeId::new("tessara.components.component_version")
                    .expect("resource type"),
                resource_id: id(30).to_string(),
                authority_revision: 7,
                governing_organization_ids: vec![id(10), id(11)],
            }),
            delegation_basis: vec![DelegationBasisV1 {
                delegation_id: id(40),
                delegated_by_actor_id: id(41),
                capability: SecurityCapabilityId::new("components:read").expect("capability"),
                organization_root_id: id(10),
            }],
            authorization_revision: 42,
            organization_revision: 17,
            jti: id(50),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        }
    }

    #[test]
    fn omitted_title_is_retained_and_explicit_blank_title_is_cleared() {
        assert_eq!(
            reconciled_title(None, Some("Current title")),
            Some("Current title".into())
        );
        assert_eq!(reconciled_title(Some("  "), Some("Current title")), None);
        assert_eq!(
            reconciled_title(Some("  Revised  "), Some("Current title")),
            Some("Revised".into())
        );
    }

    #[test]
    fn approved_resolution_states_have_stable_ui_vocabulary() {
        assert!(unavailable_component_catalog().components.is_empty());
        let restricted = ComponentResolutionResponse::new(
            ResourceResolutionV1::restricted(ResourceAccessState::Unauthorized)
                .expect("valid restricted resolution"),
            None,
            None,
            Vec::new(),
            None,
        )
        .expect("metadata-free restricted response");
        assert!(matches!(
            renderable_component_metadata(&restricted),
            Err(crate::DashboardModuleError::Forbidden)
        ));
        assert_eq!(
            disclosed_title(
                &ResourceResolutionV1::restricted(ResourceAccessState::Unauthorized)
                    .expect("valid restricted resolution"),
                &Some("Restricted title".into())
            ),
            None
        );
    }

    #[test]
    fn authorization_context_binds_semantics_but_not_per_request_fields() {
        let grant = authorization_grant();
        let baseline = semantic_authorization_context(&grant).expect("authorization context");

        let mut per_request = grant.clone();
        per_request.correlation_id = id(51);
        per_request.jti = id(52);
        per_request.issued_at += Duration::seconds(1);
        per_request.expires_at += Duration::seconds(5);
        let renewed = semantic_authorization_context(&per_request).expect("renewed context");
        assert_eq!(baseline.digest(), renewed.digest());
        assert_ne!(baseline.expires_at(), renewed.expires_at());

        let mut reordered = grant.clone();
        reordered.capability_scope_bindings.reverse();
        reordered.capability_scope_bindings[1]
            .authorized_organization_ids
            .reverse();
        assert_eq!(
            baseline.digest(),
            semantic_authorization_context(&reordered)
                .expect("reordered semantic context")
                .digest()
        );

        let mut variants = Vec::new();
        let mut actor = grant.clone();
        actor.original_actor_id = id(60);
        variants.push(actor);
        let mut scope = grant.clone();
        scope.capability_scope_bindings[0]
            .authorized_organization_ids
            .push(id(13));
        variants.push(scope);
        let mut delegation = grant.clone();
        delegation.delegation_basis[0].delegated_by_actor_id = id(61);
        variants.push(delegation);
        let mut resource = grant.clone();
        resource
            .resource_assertion
            .as_mut()
            .expect("resource assertion")
            .resource_id = id(62).to_string();
        variants.push(resource);
        let mut authorization_revision = grant.clone();
        authorization_revision.authorization_revision += 1;
        variants.push(authorization_revision);
        let mut organization_revision = grant.clone();
        organization_revision.organization_revision += 1;
        variants.push(organization_revision);
        let mut action = grant.clone();
        action.action = "components.render".into();
        variants.push(action);
        let mut audience = grant;
        audience.audience = AuthorizationAudienceV1::ModuleInstance {
            module_instance_id: id(63),
            module_definition_id: ModuleDefinitionId::new("tessara.components")
                .expect("module definition"),
        };
        variants.push(audience);

        for semantic_variant in variants {
            assert_ne!(
                baseline.digest(),
                semantic_authorization_context(&semantic_variant)
                    .expect("semantic variant")
                    .digest()
            );
        }
    }
}
