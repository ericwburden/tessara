//! Dashboard-owned composition read boundary.
//!
//! Placement rows contain only typed ComponentVersion references. Every
//! request resolves metadata through Core's action-bound compatibility
//! adapter; the Dashboard database never joins or copies Components tables.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    extract::{Path, State},
    http::HeaderMap,
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;
use tessara_dashboards::{
    DASHBOARD_COMPONENT_RESOURCE_TYPE, DashboardComponentCatalogResponseV1,
    DashboardComponentMetadataV1, DashboardComponentResolutionRequestV1,
    DashboardComponentResolutionResponseV1, DashboardComponentTransitionAction,
    DashboardComponentVersionReferenceV1, DashboardPlacementConfigInput,
    DashboardPlacementConfigState, DashboardPlacementConfigV1, DashboardPlacementOperation,
    DashboardPlacementSizePolicy, GridPlacement, GridRect, encode_dashboard_placement_config,
    parse_dashboard_placement_configs, validate_dashboard_layout,
};
use tessara_module_contract::{
    ContractCompatibilityState, ProviderAvailabilityState, ResourceAccessState,
    ResourceIdentityState, ResourceLifecycleState, ResourceOwner, ResourceTypeId,
    TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY,
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
    pub component: Option<DashboardComponentMetadataV1>,
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
        component_version_id: Uuid,
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
    pub component_version_id: Uuid,
    pub position: i32,
    pub config: Value,
}

struct StoredPlacement {
    id: Uuid,
    position: i32,
    reference: DashboardComponentVersionReferenceV1,
    config: Value,
}

pub(super) fn routes() -> Router<DashboardModuleState> {
    Router::new()
        .route("/api/dashboards/{dashboard_id}", get(get_dashboard))
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
            resolve_component(authorization, placement.reference.clone()).await?,
        );
    }
    let policy = DashboardPlacementSizePolicy::new();
    let parsed = parse_dashboard_placement_configs(
        &stored
            .iter()
            .map(|placement| {
                let kind = current_resolutions
                    .get(&placement.id)
                    .and_then(DashboardComponentResolutionResponseV1::metadata)
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
                component_version_id,
                geometry,
                title,
            } => {
                if placement_id.is_some() == client_key.is_some() {
                    return Err(DashboardModuleError::BadRequest(
                        "bind requires exactly one placement_id or client_key".into(),
                    ));
                }
                if let Some(placement_id) = placement_id {
                    if !current.contains_key(&placement_id) || !seen.insert(placement_id) {
                        return Err(DashboardModuleError::Conflict(
                            "replacement placement is stale or repeated".into(),
                        ));
                    }
                }
                if let Some(client_key) = &client_key {
                    if client_key.trim().is_empty()
                        || client_key.chars().count() > 200
                        || !client_keys.insert(client_key.clone())
                    {
                        return Err(DashboardModuleError::BadRequest(
                            "client_key is invalid or repeated".into(),
                        ));
                    }
                }
                let reference =
                    component_reference(grant.payload.installation_id, component_version_id)?;
                let wrapped = DashboardComponentVersionReferenceV1::new(reference.clone())
                    .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
                let resolution = resolve_component(authorization, wrapped).await?;
                let metadata = resolution.metadata().ok_or_else(|| {
                    DashboardModuleError::Conflict(
                        "ComponentVersion cannot be bound in its current state".into(),
                    )
                })?;
                if !matches!(metadata.version_status.as_str(), "published" | "superseded")
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
    let catalog = component_catalog(authorization).await?;
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
                component_version_id: component.component_version_id,
                component_id: component.component_id,
                component_name: component.component_name,
                component_slug: component.component_slug,
                component_type: component.component_type,
                version_number: component.version_number,
                version_label: component.version_label,
                version_status: component.version_status,
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

async fn get_dashboard(
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

async fn get_composition(
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
    let catalog = component_catalog(authorization).await?;
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
                component_version_id: component.component_version_id,
                component_id: component.component_id,
                component_name: component.component_name,
                component_slug: component.component_slug,
                component_type: component.component_type,
                version_number: component.version_number,
                version_label: component.version_label,
                version_status: component.version_status,
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

async fn list_visibility_nodes(
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
    Ok(Json(options))
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
        let response = resolve_component(authorization, placement.reference.clone()).await?;
        resolutions.insert(placement.id, response);
    }
    let policy = DashboardPlacementSizePolicy::new();
    let config_inputs = stored
        .iter()
        .map(|placement| {
            let kind = resolutions
                .get(&placement.id)
                .and_then(DashboardComponentResolutionResponseV1::metadata)
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
                title: available.then(|| parsed.title.clone()).flatten(),
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
            let reference = DashboardComponentVersionReferenceV1::new(reference)
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
    authorization: &str,
    reference: DashboardComponentVersionReferenceV1,
) -> Result<DashboardComponentResolutionResponseV1, DashboardModuleError> {
    reqwest::Client::new()
        .post(format!(
            "{}/api/private/dashboard-components/resolve",
            core_url()
        ))
        .header("x-tessara-authorization", authorization)
        .json(&DashboardComponentResolutionRequestV1::new(
            DashboardComponentTransitionAction::ResolveMetadata,
            reference,
        ))
        .send()
        .await
        .map_err(|_| DashboardModuleError::Unavailable("Components provider unavailable".into()))?
        .error_for_status()
        .map_err(|_| DashboardModuleError::Unavailable("Component resolution unavailable".into()))?
        .json()
        .await
        .map_err(|_| DashboardModuleError::Unavailable("Component resolution invalid".into()))
}

async fn component_catalog(
    authorization: &str,
) -> Result<DashboardComponentCatalogResponseV1, DashboardModuleError> {
    reqwest::Client::new()
        .post(format!(
            "{}/api/private/dashboard-components/catalog",
            core_url()
        ))
        .header("x-tessara-authorization", authorization)
        .send()
        .await
        .map_err(|_| DashboardModuleError::Unavailable("Components catalog unavailable".into()))?
        .error_for_status()
        .map_err(|_| DashboardModuleError::Unavailable("Components catalog unavailable".into()))?
        .json()
        .await
        .map_err(|_| DashboardModuleError::Unavailable("Components catalog invalid".into()))
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

async fn load_dashboard_scope(
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

fn authorization_header(headers: &HeaderMap) -> Result<&str, DashboardModuleError> {
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

fn core_url() -> String {
    std::env::var("TESSARA_CORE_INTERNAL_URL")
        .unwrap_or_else(|_| "http://core:8080".into())
        .trim_end_matches('/')
        .to_string()
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
    if resolution.resource_identity_state() == ResourceIdentityState::UnknownResource {
        return "missing";
    }
    match resolution.resource_lifecycle_state() {
        ResourceLifecycleState::ProviderDefined { state } if state == "draft" => "inactive",
        ResourceLifecycleState::ProviderDefined { state } if state == "superseded" => "superseded",
        ResourceLifecycleState::ProviderDefined { state } if state == "tombstoned" => "tombstoned",
        ResourceLifecycleState::ProviderDefined { .. } => "available",
        ResourceLifecycleState::NotEvaluated => "not_evaluated",
        ResourceLifecycleState::Undisclosed => "restricted",
    }
}

pub(super) fn component_reference(
    installation_id: Uuid,
    component_version_id: Uuid,
) -> Result<TypedResourceReference, DashboardModuleError> {
    TypedResourceReference::new(
        installation_id,
        ResourceOwner::CoreInstallation { installation_id },
        ResourceTypeId::new(DASHBOARD_COMPONENT_RESOURCE_TYPE)
            .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?,
        component_version_id.to_string(),
    )
    .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))
}

#[cfg(test)]
mod tests {
    use tessara_module_contract::{
        ContractCompatibilityState, CoreInstallationOwnerState, ProviderAvailabilityState,
        ResourceIdentityState, ResourceLifecycleState, ResourceOwnerState, ResourceResolutionV1,
    };

    use super::{reconciled_title, resolution_state};

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
        let unavailable = ResourceResolutionV1::authorized(
            ResourceOwnerState::CoreInstallation {
                state: CoreInstallationOwnerState::Live,
            },
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Unavailable,
        )
        .expect("valid");
        assert_eq!(resolution_state(&unavailable), "provider_unavailable");
    }
}
