use axum::{
    Json,
    extract::{Path, State},
    http::HeaderMap,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

use crate::{
    auth,
    db::AppState,
    error::{ApiError, ApiResult},
    hierarchy::{IdResponse, require_text},
};

#[derive(Deserialize)]
pub struct CreateDashboardRequest {
    name: String,
    description: Option<String>,
    #[serde(default)]
    visibility_node_ids: Vec<Uuid>,
}

#[derive(Deserialize)]
pub struct AddDashboardComponentRequest {
    component_version_id: Uuid,
    position: i32,
    #[serde(default)]
    config: Value,
}

#[derive(Serialize)]
pub struct DashboardSummary {
    id: Uuid,
    name: String,
    description: Option<String>,
    visibility_nodes: Vec<DashboardVisibilityNodeSummary>,
    component_count: i64,
}

#[derive(Serialize)]
pub struct DashboardResponse {
    id: Uuid,
    name: String,
    description: Option<String>,
    visibility_nodes: Vec<DashboardVisibilityNodeSummary>,
    components: Vec<DashboardComponentResponse>,
}

#[derive(Clone, Serialize)]
pub struct DashboardVisibilityNodeSummary {
    node_id: Uuid,
    node_name: String,
    node_type_name: String,
    parent_node_id: Option<Uuid>,
    node_path: String,
}

#[derive(Serialize)]
pub struct DashboardComponentResponse {
    id: Uuid,
    position: i32,
    config: Value,
    component_version_id: Uuid,
    component_id: Uuid,
    component_name: String,
    component_slug: String,
    component_type: String,
    dataset_revision_id: Uuid,
}

pub async fn create_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    require_text("dashboard name", &payload.name)?;
    require_node_ids_exist(&state.pool, &payload.visibility_node_ids).await?;
    auth::require_capability_contains_nodes(
        &state.pool,
        &account,
        "dashboards:manage",
        &payload.visibility_node_ids,
    )
    .await?;
    let mut tx = state.pool.begin().await?;
    let id = sqlx::query_scalar(
        "INSERT INTO dashboards (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(payload.name.trim())
    .bind(payload.description)
    .fetch_one(&mut *tx)
    .await?;
    replace_dashboard_scope_nodes_tx(&mut tx, id, &payload.visibility_node_ids).await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id }))
}

pub async fn update_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<CreateDashboardRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_dashboard_fully_in_capability_scope(
        &state.pool,
        &account,
        "dashboards:manage",
        dashboard_id,
    )
    .await?;
    require_text("dashboard name", &payload.name)?;
    require_node_ids_exist(&state.pool, &payload.visibility_node_ids).await?;
    auth::require_capability_contains_nodes(
        &state.pool,
        &account,
        "dashboards:manage",
        &payload.visibility_node_ids,
    )
    .await?;
    let mut tx = state.pool.begin().await?;
    sqlx::query("UPDATE dashboards SET name = $1, description = $2 WHERE id = $3")
        .bind(payload.name.trim())
        .bind(payload.description)
        .bind(dashboard_id)
        .execute(&mut *tx)
        .await?;
    replace_dashboard_scope_nodes_tx(&mut tx, dashboard_id, &payload.visibility_node_ids).await?;
    tx.commit().await?;
    Ok(Json(IdResponse { id: dashboard_id }))
}

pub async fn delete_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_dashboard_fully_in_capability_scope(
        &state.pool,
        &account,
        "dashboards:manage",
        dashboard_id,
    )
    .await?;
    sqlx::query("DELETE FROM dashboards WHERE id = $1")
        .bind(dashboard_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(IdResponse { id: dashboard_id }))
}

pub async fn add_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    Json(payload): Json<AddDashboardComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    require_dashboard_exists(&state.pool, dashboard_id).await?;
    require_dashboard_fully_in_capability_scope(
        &state.pool,
        &account,
        "dashboards:manage",
        dashboard_id,
    )
    .await?;
    require_component_version_exists(&state.pool, payload.component_version_id).await?;
    require_component_version_compatible_with_dashboard(
        &state.pool,
        &account,
        dashboard_id,
        payload.component_version_id,
    )
    .await?;
    let id = sqlx::query_scalar(
        r#"
        INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(payload.component_version_id)
    .bind(payload.position)
    .bind(payload.config)
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(IdResponse { id }))
}

pub async fn update_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(payload): Json<AddDashboardComponentRequest>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    require_component_version_exists(&state.pool, payload.component_version_id).await?;
    let dashboard_id = require_dashboard_component_dashboard_id(&state.pool, component_id).await?;
    require_dashboard_fully_in_capability_scope(
        &state.pool,
        &account,
        "dashboards:manage",
        dashboard_id,
    )
    .await?;
    require_component_version_compatible_with_dashboard(
        &state.pool,
        &account,
        dashboard_id,
        payload.component_version_id,
    )
    .await?;
    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET component_version_id = $2, position = $3, config = $4
        WHERE id = $1
        "#,
    )
    .bind(component_id)
    .bind(payload.component_version_id)
    .bind(payload.position)
    .bind(payload.config)
    .execute(&state.pool)
    .await?;
    Ok(Json(IdResponse { id: component_id }))
}

pub async fn delete_dashboard_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:manage").await?;
    let dashboard_id = require_dashboard_component_dashboard_id(&state.pool, component_id).await?;
    require_dashboard_fully_in_capability_scope(
        &state.pool,
        &account,
        "dashboards:manage",
        dashboard_id,
    )
    .await?;
    sqlx::query("DELETE FROM dashboard_components WHERE id = $1")
        .bind(component_id)
        .execute(&state.pool)
        .await?;
    Ok(Json(IdResponse { id: component_id }))
}

pub async fn list_dashboards(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<DashboardSummary>>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:read").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "dashboards:read").await?;
    let rows = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT dashboards.id, dashboards.name, dashboards.description,
               COUNT(dashboard_components.id) AS component_count
        FROM dashboards
        JOIN dashboard_scope_nodes ON dashboard_scope_nodes.dashboard_id = dashboards.id
        LEFT JOIN dashboard_components ON dashboard_components.dashboard_id = dashboards.id
        WHERE dashboard_scope_nodes.node_id = ANY($1)
        GROUP BY dashboards.id, dashboards.name, dashboards.description
        ORDER BY dashboards.name, dashboards.id
        "#,
            )
            .bind(scope_ids)
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT dashboards.id, dashboards.name, dashboards.description,
               COUNT(dashboard_components.id) AS component_count
        FROM dashboards
        LEFT JOIN dashboard_components ON dashboard_components.dashboard_id = dashboards.id
        GROUP BY dashboards.id, dashboards.name, dashboards.description
        ORDER BY dashboards.name, dashboards.id
        "#,
            )
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::None => {
            return Err(ApiError::Forbidden("dashboards:read".into()));
        }
    };
    let dashboard_ids = rows
        .iter()
        .map(|row| row.try_get::<Uuid, _>("id"))
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let visible_node_filter = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => Some(scope_ids.as_slice()),
        _ => None,
    };
    let visibility_nodes =
        load_dashboard_visibility_nodes(&state.pool, &dashboard_ids, visible_node_filter).await?;
    Ok(Json(
        rows.into_iter()
            .map(|row| {
                let id: Uuid = row.try_get("id")?;
                Ok(DashboardSummary {
                    id,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    visibility_nodes: visibility_nodes.get(&id).cloned().unwrap_or_default(),
                    component_count: row.try_get("component_count")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    ))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Json<DashboardResponse>> {
    let account = auth::require_capability(&state.pool, &headers, "dashboards:read").await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "dashboards:read").await?;
    require_dashboard_visible_for_boundary(&state.pool, dashboard_id, &boundary, "dashboards:read")
        .await?;
    let visible_node_filter = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => Some(scope_ids.as_slice()),
        _ => None,
    };
    let dashboard = sqlx::query("SELECT id, name, description FROM dashboards WHERE id = $1")
        .bind(dashboard_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let component_rows = match &boundary {
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            sqlx::query(
                r#"
        SELECT
            dashboard_components.id,
            dashboard_components.position,
            dashboard_components.config,
            component_versions.id AS component_version_id,
            component_versions.component_id,
            component_versions.component_type::text AS component_type,
            component_versions.dataset_revision_id,
            components.name AS component_name,
            components.slug AS component_slug
        FROM dashboard_components
        JOIN component_versions ON component_versions.id = dashboard_components.component_version_id
        JOIN dataset_revisions ON dataset_revisions.id = component_versions.dataset_revision_id
        JOIN components ON components.id = component_versions.component_id
        WHERE dashboard_components.dashboard_id = $1
          AND EXISTS (
              SELECT 1
              FROM dataset_scope_nodes
              WHERE dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
                AND dataset_scope_nodes.node_id = ANY($2)
          )
          AND NOT EXISTS (
              SELECT 1
              FROM dataset_scope_nodes
              WHERE dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
                AND NOT EXISTS (
                    SELECT 1
                    FROM dashboard_scope_nodes
                    WHERE dashboard_scope_nodes.dashboard_id = dashboard_components.dashboard_id
                      AND dashboard_scope_nodes.node_id = dataset_scope_nodes.node_id
                )
          )
        ORDER BY dashboard_components.position, dashboard_components.id
        "#,
            )
            .bind(dashboard_id)
            .bind(scope_ids)
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::Global => {
            sqlx::query(
                r#"
        SELECT
            dashboard_components.id,
            dashboard_components.position,
            dashboard_components.config,
            component_versions.id AS component_version_id,
            component_versions.component_id,
            component_versions.component_type::text AS component_type,
            component_versions.dataset_revision_id,
            components.name AS component_name,
            components.slug AS component_slug
        FROM dashboard_components
        JOIN component_versions ON component_versions.id = dashboard_components.component_version_id
        JOIN dataset_revisions ON dataset_revisions.id = component_versions.dataset_revision_id
        JOIN components ON components.id = component_versions.component_id
        WHERE dashboard_components.dashboard_id = $1
          AND NOT EXISTS (
              SELECT 1
              FROM dataset_scope_nodes
              WHERE dataset_scope_nodes.dataset_id = dataset_revisions.dataset_id
                AND NOT EXISTS (
                    SELECT 1
                    FROM dashboard_scope_nodes
                    WHERE dashboard_scope_nodes.dashboard_id = dashboard_components.dashboard_id
                      AND dashboard_scope_nodes.node_id = dataset_scope_nodes.node_id
                )
          )
        ORDER BY dashboard_components.position, dashboard_components.id
        "#,
            )
            .bind(dashboard_id)
            .fetch_all(&state.pool)
            .await?
        }
        auth::CapabilityBoundary::None => {
            return Err(ApiError::Forbidden("dashboards:read".into()));
        }
    };

    let visibility_nodes =
        load_dashboard_visibility_nodes(&state.pool, &[dashboard_id], visible_node_filter).await?;

    Ok(Json(DashboardResponse {
        id: dashboard.try_get("id")?,
        name: dashboard.try_get("name")?,
        description: dashboard.try_get("description")?,
        visibility_nodes: visibility_nodes
            .get(&dashboard_id)
            .cloned()
            .unwrap_or_default(),
        components: component_rows
            .into_iter()
            .map(|row| {
                Ok(DashboardComponentResponse {
                    id: row.try_get("id")?,
                    position: row.try_get("position")?,
                    config: row.try_get("config")?,
                    component_version_id: row.try_get("component_version_id")?,
                    component_id: row.try_get("component_id")?,
                    component_name: row.try_get("component_name")?,
                    component_slug: row.try_get("component_slug")?,
                    component_type: row.try_get("component_type")?,
                    dataset_revision_id: row.try_get("dataset_revision_id")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?,
    }))
}

async fn require_dashboard_exists(pool: &sqlx::PgPool, dashboard_id: Uuid) -> ApiResult<()> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM dashboards WHERE id = $1)")
        .bind(dashboard_id)
        .fetch_one(pool)
        .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!("dashboard {dashboard_id}")))
    }
}

async fn require_component_version_exists(
    pool: &sqlx::PgPool,
    component_version_id: Uuid,
) -> ApiResult<()> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM component_versions WHERE id = $1)")
            .bind(component_version_id)
            .fetch_one(pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::NotFound(format!(
            "component version {component_version_id}"
        )))
    }
}

async fn require_node_ids_exist(pool: &sqlx::PgPool, node_ids: &[Uuid]) -> ApiResult<()> {
    if node_ids.is_empty() {
        return Err(ApiError::BadRequest(
            "at least one visibility node is required".into(),
        ));
    }
    let existing_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM nodes WHERE id = ANY($1)")
        .bind(node_ids)
        .fetch_one(pool)
        .await?;
    if existing_count as usize == node_ids.len() {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "one or more visibility nodes do not exist".into(),
        ))
    }
}

async fn replace_dashboard_scope_nodes_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
    node_ids: &[Uuid],
) -> ApiResult<()> {
    sqlx::query("DELETE FROM dashboard_scope_nodes WHERE dashboard_id = $1")
        .bind(dashboard_id)
        .execute(&mut **tx)
        .await?;
    for node_id in node_ids {
        sqlx::query(
            "INSERT INTO dashboard_scope_nodes (dashboard_id, node_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
        )
        .bind(dashboard_id)
        .bind(node_id)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn load_dashboard_visibility_nodes(
    pool: &sqlx::PgPool,
    dashboard_ids: &[Uuid],
    visible_node_filter: Option<&[Uuid]>,
) -> ApiResult<std::collections::BTreeMap<Uuid, Vec<DashboardVisibilityNodeSummary>>> {
    if dashboard_ids.is_empty() {
        return Ok(std::collections::BTreeMap::new());
    }
    let rows = if let Some(node_ids) = visible_node_filter {
        sqlx::query(
            r#"
            SELECT
                dashboard_scope_nodes.dashboard_id,
                nodes.id AS node_id,
                nodes.name AS node_name,
                nodes.parent_node_id,
                node_types.name AS node_type_name,
                COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path
            FROM dashboard_scope_nodes
            JOIN nodes ON nodes.id = dashboard_scope_nodes.node_id
            JOIN node_types ON node_types.id = nodes.node_type_id
            LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
            WHERE dashboard_scope_nodes.dashboard_id = ANY($1)
              AND dashboard_scope_nodes.node_id = ANY($2)
            ORDER BY node_path, nodes.name, nodes.id
            "#,
        )
        .bind(dashboard_ids)
        .bind(node_ids)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT
                dashboard_scope_nodes.dashboard_id,
                nodes.id AS node_id,
                nodes.name AS node_name,
                nodes.parent_node_id,
                node_types.name AS node_type_name,
                COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path
            FROM dashboard_scope_nodes
            JOIN nodes ON nodes.id = dashboard_scope_nodes.node_id
            JOIN node_types ON node_types.id = nodes.node_type_id
            LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
            WHERE dashboard_scope_nodes.dashboard_id = ANY($1)
            ORDER BY node_path, nodes.name, nodes.id
            "#,
        )
        .bind(dashboard_ids)
        .fetch_all(pool)
        .await?
    };
    let mut visibility_nodes =
        std::collections::BTreeMap::<Uuid, Vec<DashboardVisibilityNodeSummary>>::new();
    for row in rows {
        let dashboard_id: Uuid = row.try_get("dashboard_id")?;
        visibility_nodes
            .entry(dashboard_id)
            .or_default()
            .push(DashboardVisibilityNodeSummary {
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_id: row.try_get("parent_node_id")?,
                node_path: row.try_get("node_path")?,
            });
    }
    Ok(visibility_nodes)
}

async fn require_dashboard_fully_in_capability_scope(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    capability: &str,
    dashboard_id: Uuid,
) -> ApiResult<()> {
    let node_ids = load_dashboard_scope_node_ids(pool, dashboard_id).await?;
    auth::require_capability_contains_nodes(pool, account, capability, &node_ids).await
}

async fn require_dashboard_visible_for_boundary(
    pool: &sqlx::PgPool,
    dashboard_id: Uuid,
    boundary: &auth::CapabilityBoundary,
    capability: &str,
) -> ApiResult<()> {
    match boundary {
        auth::CapabilityBoundary::Global => Ok(()),
        auth::CapabilityBoundary::Scoped(scope_ids) => {
            let node_ids = load_dashboard_scope_node_ids(pool, dashboard_id).await?;
            if node_ids.iter().any(|node_id| scope_ids.contains(node_id)) {
                Ok(())
            } else {
                Err(ApiError::Forbidden(capability.into()))
            }
        }
        auth::CapabilityBoundary::None => Err(ApiError::Forbidden(capability.into())),
    }
}

async fn load_dashboard_scope_node_ids(
    pool: &sqlx::PgPool,
    dashboard_id: Uuid,
) -> ApiResult<Vec<Uuid>> {
    sqlx::query_scalar(
        "SELECT node_id FROM dashboard_scope_nodes WHERE dashboard_id = $1 ORDER BY node_id",
    )
    .bind(dashboard_id)
    .fetch_all(pool)
    .await
    .map_err(Into::into)
}

async fn require_dashboard_component_dashboard_id(
    pool: &sqlx::PgPool,
    dashboard_component_id: Uuid,
) -> ApiResult<Uuid> {
    sqlx::query_scalar("SELECT dashboard_id FROM dashboard_components WHERE id = $1")
        .bind(dashboard_component_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard component {dashboard_component_id}")))
}

async fn require_component_version_compatible_with_dashboard(
    pool: &sqlx::PgPool,
    account: &auth::AccountContext,
    dashboard_id: Uuid,
    component_version_id: Uuid,
) -> ApiResult<()> {
    let dataset_id: Uuid = sqlx::query_scalar(
        r#"
        SELECT dataset_revisions.dataset_id
        FROM component_versions
        JOIN dataset_revisions ON dataset_revisions.id = component_versions.dataset_revision_id
        WHERE component_versions.id = $1
        "#,
    )
    .bind(component_version_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::NotFound(format!("component version {component_version_id}")))?;
    let dashboard_node_ids = load_dashboard_scope_node_ids(pool, dashboard_id).await?;
    let dataset_node_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id = $1",
    )
    .bind(dataset_id)
    .fetch_all(pool)
    .await?;
    if !dataset_node_ids
        .iter()
        .all(|node_id| dashboard_node_ids.contains(node_id))
    {
        return Err(ApiError::BadRequest(
            "dashboard visibility must encompass component dataset visibility".into(),
        ));
    }
    auth::require_capability_contains_nodes(
        pool,
        account,
        "dashboards:manage",
        &dashboard_node_ids,
    )
    .await?;
    Ok(())
}
