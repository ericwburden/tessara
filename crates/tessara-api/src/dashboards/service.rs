//! Dashboard metadata orchestration and authorization.
//!
//! Full-layout commands live in `reconciliation`, response redaction and
//! projection live in `projection`, and SQL remains in `repository`.

use std::collections::BTreeSet;

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::{
    auth::{self, AccountContext, CapabilityBoundary},
    error::ApiError,
    hierarchy::{IdResponse, require_text},
};

use super::{
    dto::{
        CreateDashboardRequest, DashboardCompositionResponse, DashboardResponse, DashboardSummary,
        DashboardVisibilityNodeOption,
    },
    error::{DashboardResult, DashboardServiceError},
    projection::{
        build_dashboard_response_tx, distinct_dataset_ids, load_component_options_tx,
        map_visibility,
    },
    repository,
    scope::{
        contains as boundary_contains_nodes, overlaps as boundary_overlaps_nodes,
        require_contains as require_boundary_contains,
    },
};

pub(super) async fn list_dashboards(
    pool: &PgPool,
    account: &AccountContext,
) -> DashboardResult<Vec<DashboardSummary>> {
    let boundary = auth::capability_boundary(pool, account, "dashboards:read").await?;
    let scope_filter = match &boundary {
        CapabilityBoundary::Global => None,
        CapabilityBoundary::Scoped(node_ids) => Some(node_ids.as_slice()),
        CapabilityBoundary::None => {
            return Err(ApiError::Forbidden("dashboards:read".to_string()).into());
        }
    };
    let records = repository::list_dashboards(pool, scope_filter).await?;
    let dashboard_ids = records.iter().map(|record| record.id).collect::<Vec<_>>();
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    let dashboard_scopes =
        repository::load_dashboard_scope_node_ids_many(pool, &dashboard_ids).await?;
    let mut visibility =
        repository::load_visibility_nodes(pool, &dashboard_ids, scope_filter).await?;
    Ok(records
        .into_iter()
        .map(|record| DashboardSummary {
            id: record.id,
            name: record.name,
            description: record.description,
            visibility_nodes: map_visibility(visibility.remove(&record.id).unwrap_or_default()),
            placement_count: record.placement_count,
            can_manage: dashboard_scopes
                .get(&record.id)
                .is_some_and(|scope| boundary_contains_nodes(&dashboard_manage_boundary, scope)),
        })
        .collect())
}

pub(super) async fn list_visibility_node_options(
    pool: &PgPool,
    account: &AccountContext,
) -> DashboardResult<Vec<DashboardVisibilityNodeOption>> {
    let boundary = auth::capability_boundary(pool, account, "dashboards:manage").await?;
    let scoped_node_ids = match &boundary {
        CapabilityBoundary::Global => None,
        CapabilityBoundary::Scoped(node_ids) => Some(node_ids.as_slice()),
        CapabilityBoundary::None => {
            return Err(ApiError::Forbidden("dashboards:manage".to_string()).into());
        }
    };
    Ok(
        repository::list_visibility_node_options(pool, scoped_node_ids)
            .await?
            .into_iter()
            .map(|node| DashboardVisibilityNodeOption {
                id: node.id,
                node_type_name: node.node_type_name,
                parent_node_name: node.parent_node_name,
                name: node.name,
            })
            .collect(),
    )
}

pub(super) async fn get_dashboard(
    pool: &PgPool,
    account: &AccountContext,
    dashboard_id: Uuid,
) -> DashboardResult<DashboardResponse> {
    let dashboard_boundary = auth::capability_boundary(pool, account, "dashboards:read").await?;
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    let component_boundary = auth::capability_boundary(pool, account, "components:read").await?;
    let mut tx = pool.begin().await?;
    begin_repeatable_read(&mut tx).await?;
    let dashboard = repository::load_dashboard_tx(&mut tx, dashboard_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let dashboard_scope =
        repository::load_dashboard_scope_node_ids_unlocked_tx(&mut tx, dashboard_id).await?;
    if !boundary_overlaps_nodes(&dashboard_boundary, &dashboard_scope) {
        return Err(ApiError::Forbidden("dashboards:read".to_string()).into());
    }
    let response = build_dashboard_response_tx(
        &mut tx,
        dashboard,
        &dashboard_scope,
        &dashboard_boundary,
        &dashboard_manage_boundary,
        &component_boundary,
        false,
    )
    .await?;
    tx.commit().await?;
    Ok(response)
}

pub(super) async fn load_composition(
    pool: &PgPool,
    account: &AccountContext,
    dashboard_id: Uuid,
) -> DashboardResult<DashboardCompositionResponse> {
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    let component_boundary = auth::capability_boundary(pool, account, "components:read").await?;
    let mut tx = pool.begin().await?;
    begin_repeatable_read(&mut tx).await?;
    let dashboard = repository::load_dashboard_tx(&mut tx, dashboard_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let dashboard_scope =
        repository::load_dashboard_scope_node_ids_unlocked_tx(&mut tx, dashboard_id).await?;
    if dashboard_scope.is_empty() {
        return Err(ApiError::BadRequest("Dashboard has no visibility scope".to_string()).into());
    }
    require_boundary_contains(
        &dashboard_manage_boundary,
        &dashboard_scope,
        "dashboards:manage",
    )?;
    let dashboard_boundary = CapabilityBoundary::Global;
    let dashboard = build_dashboard_response_tx(
        &mut tx,
        dashboard,
        &dashboard_scope,
        &dashboard_boundary,
        &dashboard_manage_boundary,
        &component_boundary,
        true,
    )
    .await?;
    let available_component_versions =
        load_component_options_tx(&mut tx, &component_boundary, &dashboard_scope).await?;
    let response = DashboardCompositionResponse {
        dashboard,
        available_component_versions,
        new_placement_ids: Vec::new(),
    };
    tx.commit().await?;
    Ok(response)
}

pub(super) async fn create_dashboard(
    pool: &PgPool,
    account: &AccountContext,
    payload: CreateDashboardRequest,
) -> DashboardResult<IdResponse> {
    require_text("dashboard name", &payload.name)?;
    let visibility_node_ids = normalized_node_ids(&payload.visibility_node_ids)?;
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    require_boundary_contains(
        &dashboard_manage_boundary,
        &visibility_node_ids,
        "dashboards:manage",
    )?;
    let mut tx = pool.begin().await?;
    if !repository::node_ids_exist(&mut tx, &visibility_node_ids).await? {
        return Err(
            ApiError::BadRequest("one or more visibility nodes do not exist".to_string()).into(),
        );
    }
    let description = normalized_optional_text(payload.description.as_deref());
    let dashboard_id =
        repository::insert_dashboard(&mut tx, payload.name.trim(), description.as_deref()).await?;
    repository::replace_scope_nodes(&mut tx, dashboard_id, &visibility_node_ids).await?;
    tx.commit().await?;
    Ok(IdResponse { id: dashboard_id })
}

pub(super) async fn update_dashboard(
    pool: &PgPool,
    account: &AccountContext,
    dashboard_id: Uuid,
    payload: CreateDashboardRequest,
) -> DashboardResult<IdResponse> {
    require_text("dashboard name", &payload.name)?;
    let visibility_node_ids = normalized_node_ids(&payload.visibility_node_ids)?;
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    require_boundary_contains(
        &dashboard_manage_boundary,
        &visibility_node_ids,
        "dashboards:manage",
    )?;

    let mut tx = pool.begin().await?;
    repository::lock_dashboard(&mut tx, dashboard_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let current_scope = repository::load_dashboard_scope_node_ids_tx(&mut tx, dashboard_id).await?;
    require_boundary_contains(
        &dashboard_manage_boundary,
        &current_scope,
        "dashboards:manage",
    )?;
    if !repository::node_ids_exist(&mut tx, &visibility_node_ids).await? {
        return Err(
            ApiError::BadRequest("one or more visibility nodes do not exist".to_string()).into(),
        );
    }

    let placements = repository::load_locked_placements(&mut tx, dashboard_id).await?;
    let version_ids = distinct_version_ids(placements.iter().map(|row| row.component_version_id));
    let versions = repository::load_component_versions_locked(&mut tx, &version_ids).await?;
    let dataset_ids = distinct_dataset_ids(versions.iter().map(|version| version.dataset_id));
    let dataset_scopes = repository::load_dataset_scope_nodes_tx(&mut tx, &dataset_ids).await?;
    for version in versions {
        let nodes = dataset_scopes
            .get(&version.dataset_id)
            .cloned()
            .unwrap_or_default();
        if nodes.is_empty()
            || !nodes
                .iter()
                .all(|node_id| visibility_node_ids.contains(node_id))
        {
            return Err(DashboardServiceError::ScopeIncompatible);
        }
    }

    let description = normalized_optional_text(payload.description.as_deref());
    repository::update_dashboard(
        &mut tx,
        dashboard_id,
        payload.name.trim(),
        description.as_deref(),
    )
    .await?;
    repository::replace_scope_nodes(&mut tx, dashboard_id, &visibility_node_ids).await?;
    tx.commit().await?;
    Ok(IdResponse { id: dashboard_id })
}

pub(super) async fn delete_dashboard(
    pool: &PgPool,
    account: &AccountContext,
    dashboard_id: Uuid,
) -> DashboardResult<IdResponse> {
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    let mut tx = pool.begin().await?;
    repository::lock_dashboard(&mut tx, dashboard_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let scope = repository::load_dashboard_scope_node_ids_tx(&mut tx, dashboard_id).await?;
    require_boundary_contains(&dashboard_manage_boundary, &scope, "dashboards:manage")?;
    repository::delete_dashboard(&mut tx, dashboard_id).await?;
    tx.commit().await?;
    Ok(IdResponse { id: dashboard_id })
}

async fn begin_repeatable_read(tx: &mut Transaction<'_, Postgres>) -> DashboardResult<()> {
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ")
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn normalized_node_ids(node_ids: &[Uuid]) -> DashboardResult<Vec<Uuid>> {
    if node_ids.is_empty() {
        return Err(
            ApiError::BadRequest("at least one visibility node is required".to_string()).into(),
        );
    }
    let normalized = node_ids.iter().copied().collect::<BTreeSet<_>>();
    if normalized.len() != node_ids.len() {
        return Err(ApiError::BadRequest("visibility nodes must be unique".to_string()).into());
    }
    Ok(normalized.into_iter().collect())
}

fn normalized_optional_text(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn distinct_version_ids(ids: impl Iterator<Item = Uuid>) -> Vec<Uuid> {
    ids.collect::<BTreeSet<_>>().into_iter().collect()
}
