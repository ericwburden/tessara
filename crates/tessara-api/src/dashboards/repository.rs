//! PostgreSQL persistence for Dashboards.

use std::collections::BTreeMap;

use serde_json::Value;
use sqlx::{PgPool, Postgres, Row, Transaction};
use uuid::Uuid;

use super::error::DashboardResult;

#[derive(Clone, Debug)]
pub(super) struct DashboardRecord {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub placement_count: i64,
}

#[derive(Clone, Debug)]
pub(super) struct DashboardVisibilityRecord {
    pub dashboard_id: Uuid,
    pub node_id: Uuid,
    pub node_name: String,
    pub node_type_name: String,
    pub parent_node_id: Option<Uuid>,
    pub node_path: String,
}

#[derive(Clone, Debug)]
pub(super) struct DashboardVisibilityOptionRecord {
    pub id: Uuid,
    pub node_type_name: String,
    pub parent_node_name: Option<String>,
    pub name: String,
}

#[derive(Clone, Debug)]
pub(super) struct DashboardPlacementRecord {
    pub id: Uuid,
    pub position: i32,
    pub config: Value,
    pub component_version_id: Uuid,
    pub component_id: Uuid,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_status: String,
    pub dataset_id: Uuid,
}

#[derive(Clone, Debug)]
pub(super) struct ComponentVersionRecord {
    pub id: Uuid,
    pub component_id: Uuid,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_status: String,
    pub dataset_id: Uuid,
}

pub(super) async fn list_dashboards(
    pool: &PgPool,
    scoped_node_ids: Option<&[Uuid]>,
) -> DashboardResult<Vec<DashboardRecord>> {
    let rows = if let Some(node_ids) = scoped_node_ids {
        sqlx::query(
            r#"
            SELECT dashboards.id, dashboards.name, dashboards.description,
                   (SELECT COUNT(*)
                    FROM dashboard_components
                    WHERE dashboard_components.dashboard_id = dashboards.id) AS placement_count
            FROM dashboards
            WHERE EXISTS (
                SELECT 1
                FROM dashboard_scope_nodes
                WHERE dashboard_scope_nodes.dashboard_id = dashboards.id
                  AND dashboard_scope_nodes.node_id = ANY($1)
            )
            ORDER BY dashboards.name, dashboards.id
            "#,
        )
        .bind(node_ids)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT dashboards.id, dashboards.name, dashboards.description,
                   (SELECT COUNT(*)
                    FROM dashboard_components
                    WHERE dashboard_components.dashboard_id = dashboards.id) AS placement_count
            FROM dashboards
            ORDER BY dashboards.name, dashboards.id
            "#,
        )
        .fetch_all(pool)
        .await?
    };
    rows.into_iter().map(map_dashboard).collect()
}

pub(super) async fn list_visibility_node_options(
    pool: &PgPool,
    scoped_node_ids: Option<&[Uuid]>,
) -> DashboardResult<Vec<DashboardVisibilityOptionRecord>> {
    let rows = if let Some(node_ids) = scoped_node_ids {
        sqlx::query(
            r#"
            SELECT nodes.id, node_types.name AS node_type_name,
                   parent_nodes.name AS parent_node_name, nodes.name
            FROM nodes
            JOIN node_types ON node_types.id = nodes.node_type_id
            LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
            WHERE nodes.id = ANY($1)
            ORDER BY node_types.name, parent_nodes.name NULLS FIRST, nodes.name, nodes.id
            "#,
        )
        .bind(node_ids)
        .fetch_all(pool)
        .await?
    } else {
        sqlx::query(
            r#"
            SELECT nodes.id, node_types.name AS node_type_name,
                   parent_nodes.name AS parent_node_name, nodes.name
            FROM nodes
            JOIN node_types ON node_types.id = nodes.node_type_id
            LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
            ORDER BY node_types.name, parent_nodes.name NULLS FIRST, nodes.name, nodes.id
            "#,
        )
        .fetch_all(pool)
        .await?
    };
    rows.into_iter()
        .map(|row| {
            Ok(DashboardVisibilityOptionRecord {
                id: row.try_get("id")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_name: row.try_get("parent_node_name")?,
                name: row.try_get("name")?,
            })
        })
        .collect()
}

pub(super) async fn load_dashboard_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Option<DashboardRecord>> {
    sqlx::query(
        r#"
        SELECT dashboards.id, dashboards.name, dashboards.description,
               (SELECT COUNT(*)
                FROM dashboard_components
                WHERE dashboard_components.dashboard_id = dashboards.id) AS placement_count
        FROM dashboards
        WHERE dashboards.id = $1
        "#,
    )
    .bind(dashboard_id)
    .fetch_optional(&mut **tx)
    .await?
    .map(map_dashboard)
    .transpose()
}

pub(super) async fn load_visibility_nodes(
    pool: &PgPool,
    dashboard_ids: &[Uuid],
    visible_node_filter: Option<&[Uuid]>,
) -> DashboardResult<BTreeMap<Uuid, Vec<DashboardVisibilityRecord>>> {
    if dashboard_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = if let Some(node_ids) = visible_node_filter {
        sqlx::query(
            r#"
            SELECT dashboard_scope_nodes.dashboard_id,
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
            SELECT dashboard_scope_nodes.dashboard_id,
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
    let mut mapped = BTreeMap::<Uuid, Vec<DashboardVisibilityRecord>>::new();
    for row in rows {
        let record = DashboardVisibilityRecord {
            dashboard_id: row.try_get("dashboard_id")?,
            node_id: row.try_get("node_id")?,
            node_name: row.try_get("node_name")?,
            node_type_name: row.try_get("node_type_name")?,
            parent_node_id: row.try_get("parent_node_id")?,
            node_path: row.try_get("node_path")?,
        };
        mapped.entry(record.dashboard_id).or_default().push(record);
    }
    Ok(mapped)
}

pub(super) async fn load_visibility_nodes_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Vec<DashboardVisibilityRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT dashboard_scope_nodes.dashboard_id,
               nodes.id AS node_id,
               nodes.name AS node_name,
               nodes.parent_node_id,
               node_types.name AS node_type_name,
               COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path
        FROM dashboard_scope_nodes
        JOIN nodes ON nodes.id = dashboard_scope_nodes.node_id
        JOIN node_types ON node_types.id = nodes.node_type_id
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        WHERE dashboard_scope_nodes.dashboard_id = $1
        ORDER BY node_path, nodes.name, nodes.id
        FOR SHARE OF dashboard_scope_nodes
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DashboardVisibilityRecord {
                dashboard_id: row.try_get("dashboard_id")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_id: row.try_get("parent_node_id")?,
                node_path: row.try_get("node_path")?,
            })
        })
        .collect()
}

/// Loads Dashboard visibility inside a consistent read transaction without
/// taking row locks. Mutation paths use [`load_visibility_nodes_tx`] instead.
pub(super) async fn load_visibility_nodes_unlocked_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Vec<DashboardVisibilityRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT dashboard_scope_nodes.dashboard_id,
               nodes.id AS node_id,
               nodes.name AS node_name,
               nodes.parent_node_id,
               node_types.name AS node_type_name,
               COALESCE(parent_nodes.name || ' / ' || nodes.name, nodes.name) AS node_path
        FROM dashboard_scope_nodes
        JOIN nodes ON nodes.id = dashboard_scope_nodes.node_id
        JOIN node_types ON node_types.id = nodes.node_type_id
        LEFT JOIN nodes AS parent_nodes ON parent_nodes.id = nodes.parent_node_id
        WHERE dashboard_scope_nodes.dashboard_id = $1
        ORDER BY node_path, nodes.name, nodes.id
        "#,
    )
    .bind(dashboard_id)
    .fetch_all(&mut **tx)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(DashboardVisibilityRecord {
                dashboard_id: row.try_get("dashboard_id")?,
                node_id: row.try_get("node_id")?,
                node_name: row.try_get("node_name")?,
                node_type_name: row.try_get("node_type_name")?,
                parent_node_id: row.try_get("parent_node_id")?,
                node_path: row.try_get("node_path")?,
            })
        })
        .collect()
}

pub(super) async fn load_dashboard_scope_node_ids_many(
    pool: &PgPool,
    dashboard_ids: &[Uuid],
) -> DashboardResult<BTreeMap<Uuid, Vec<Uuid>>> {
    if dashboard_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT dashboard_id, node_id
        FROM dashboard_scope_nodes
        WHERE dashboard_id = ANY($1)
        ORDER BY dashboard_id, node_id
        "#,
    )
    .bind(dashboard_ids)
    .fetch_all(pool)
    .await?;
    let mut scopes = BTreeMap::<Uuid, Vec<Uuid>>::new();
    for row in rows {
        scopes
            .entry(row.try_get("dashboard_id")?)
            .or_default()
            .push(row.try_get("node_id")?);
    }
    Ok(scopes)
}

pub(super) async fn load_placements_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Vec<DashboardPlacementRecord>> {
    let rows = sqlx::query(PLACEMENT_SELECT)
        .bind(dashboard_id)
        .fetch_all(&mut **tx)
        .await?;
    rows.into_iter().map(map_placement).collect()
}

pub(super) async fn load_placeable_component_versions_tx(
    tx: &mut Transaction<'_, Postgres>,
) -> DashboardResult<Vec<ComponentVersionRecord>> {
    let rows = sqlx::query(
        r#"
        SELECT component_versions.id,
               component_versions.component_id,
               components.name AS component_name,
               components.slug AS component_slug,
               component_versions.component_type::text AS component_type,
               component_versions.version_number,
               component_versions.version_label,
               component_versions.status::text AS version_status,
               component_versions.dataset_id
        FROM component_versions
        JOIN components ON components.id = component_versions.component_id
        WHERE component_versions.status IN (
            'published'::component_version_status,
            'superseded'::component_version_status
        )
        ORDER BY components.name, component_versions.version_number DESC, component_versions.id
        "#,
    )
    .fetch_all(&mut **tx)
    .await?;
    rows.into_iter().map(map_component_version).collect()
}

pub(super) async fn insert_dashboard(
    tx: &mut Transaction<'_, Postgres>,
    name: &str,
    description: Option<&str>,
) -> DashboardResult<Uuid> {
    Ok(sqlx::query_scalar(
        "INSERT INTO dashboards (name, description) VALUES ($1, $2) RETURNING id",
    )
    .bind(name)
    .bind(description)
    .fetch_one(&mut **tx)
    .await?)
}

pub(super) async fn lock_dashboard(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Option<DashboardRecord>> {
    sqlx::query(
        r#"
        SELECT dashboards.id, dashboards.name, dashboards.description,
               (SELECT COUNT(*)
                FROM dashboard_components
                WHERE dashboard_components.dashboard_id = dashboards.id) AS placement_count
        FROM dashboards
        WHERE dashboards.id = $1
        FOR UPDATE OF dashboards
        "#,
    )
    .bind(dashboard_id)
    .fetch_optional(&mut **tx)
    .await?
    .map(map_dashboard)
    .transpose()
}

pub(super) async fn update_dashboard(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
    name: &str,
    description: Option<&str>,
) -> DashboardResult<()> {
    sqlx::query("UPDATE dashboards SET name = $2, description = $3 WHERE id = $1")
        .bind(dashboard_id)
        .bind(name)
        .bind(description)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn delete_dashboard(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<()> {
    sqlx::query("DELETE FROM dashboards WHERE id = $1")
        .bind(dashboard_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

pub(super) async fn node_ids_exist(
    tx: &mut Transaction<'_, Postgres>,
    node_ids: &[Uuid],
) -> DashboardResult<bool> {
    let count: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT id) FROM nodes WHERE id = ANY($1)")
        .bind(node_ids)
        .fetch_one(&mut **tx)
        .await?;
    Ok(count as usize == node_ids.len())
}

pub(super) async fn replace_scope_nodes(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
    node_ids: &[Uuid],
) -> DashboardResult<()> {
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

pub(super) async fn load_locked_placements(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Vec<DashboardPlacementRecord>> {
    let query = format!("{PLACEMENT_SELECT} FOR UPDATE OF dashboard_components");
    let rows = sqlx::query(&query)
        .bind(dashboard_id)
        .fetch_all(&mut **tx)
        .await?;
    rows.into_iter().map(map_placement).collect()
}

pub(super) async fn load_dashboard_scope_node_ids_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT node_id FROM dashboard_scope_nodes WHERE dashboard_id = $1 ORDER BY node_id FOR SHARE",
    )
    .bind(dashboard_id)
    .fetch_all(&mut **tx)
    .await?)
}

/// Loads Dashboard visibility IDs without locking rows. This is intended for
/// REPEATABLE READ response projections; save paths use the locking variant.
pub(super) async fn load_dashboard_scope_node_ids_unlocked_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
) -> DashboardResult<Vec<Uuid>> {
    Ok(sqlx::query_scalar(
        "SELECT node_id FROM dashboard_scope_nodes WHERE dashboard_id = $1 ORDER BY node_id",
    )
    .bind(dashboard_id)
    .fetch_all(&mut **tx)
    .await?)
}

pub(super) async fn load_component_versions_locked(
    tx: &mut Transaction<'_, Postgres>,
    version_ids: &[Uuid],
) -> DashboardResult<Vec<ComponentVersionRecord>> {
    if version_ids.is_empty() {
        return Ok(Vec::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT component_versions.id,
               component_versions.component_id,
               components.name AS component_name,
               components.slug AS component_slug,
               component_versions.component_type::text AS component_type,
               component_versions.version_number,
               component_versions.version_label,
               component_versions.status::text AS version_status,
               component_versions.dataset_id
        FROM component_versions
        JOIN components ON components.id = component_versions.component_id
        WHERE component_versions.id = ANY($1)
        ORDER BY component_versions.id
        FOR SHARE OF component_versions
        "#,
    )
    .bind(version_ids)
    .fetch_all(&mut **tx)
    .await?;
    rows.into_iter().map(map_component_version).collect()
}

pub(super) async fn load_dataset_scope_nodes_tx(
    tx: &mut Transaction<'_, Postgres>,
    dataset_ids: &[Uuid],
) -> DashboardResult<BTreeMap<Uuid, Vec<Uuid>>> {
    if dataset_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT dataset_id, node_id
        FROM dataset_scope_nodes
        WHERE dataset_id = ANY($1)
        ORDER BY dataset_id, node_id
        FOR SHARE OF dataset_scope_nodes
        "#,
    )
    .bind(dataset_ids)
    .fetch_all(&mut **tx)
    .await?;
    map_dataset_scopes(rows)
}

/// Loads Dataset visibility for response decoration without adding row locks.
///
/// Candidate validation uses [`load_dataset_scope_nodes_tx`] so scope changes
/// cannot race a save. Picker decoration deliberately does not widen that lock
/// set after candidate rows have already been locked; doing so would allow two
/// unrelated Dashboard saves to deadlock while walking every placeable Dataset.
pub(super) async fn load_dataset_scope_nodes_unlocked_tx(
    tx: &mut Transaction<'_, Postgres>,
    dataset_ids: &[Uuid],
) -> DashboardResult<BTreeMap<Uuid, Vec<Uuid>>> {
    if dataset_ids.is_empty() {
        return Ok(BTreeMap::new());
    }
    let rows = sqlx::query(
        r#"
        SELECT dataset_id, node_id
        FROM dataset_scope_nodes
        WHERE dataset_id = ANY($1)
        ORDER BY dataset_id, node_id
        "#,
    )
    .bind(dataset_ids)
    .fetch_all(&mut **tx)
    .await?;
    map_dataset_scopes(rows)
}

pub(super) async fn update_placement(
    tx: &mut Transaction<'_, Postgres>,
    placement_id: Uuid,
    component_version_id: Uuid,
    position: i32,
    config: &Value,
) -> DashboardResult<()> {
    sqlx::query(
        r#"
        UPDATE dashboard_components
        SET component_version_id = $2, position = $3, config = $4
        WHERE id = $1
        "#,
    )
    .bind(placement_id)
    .bind(component_version_id)
    .bind(position)
    .bind(config)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

pub(super) async fn insert_placement(
    tx: &mut Transaction<'_, Postgres>,
    dashboard_id: Uuid,
    component_version_id: Uuid,
    position: i32,
    config: &Value,
) -> DashboardResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(dashboard_id)
    .bind(component_version_id)
    .bind(position)
    .bind(config)
    .fetch_one(&mut **tx)
    .await?)
}

pub(super) async fn delete_placement(
    tx: &mut Transaction<'_, Postgres>,
    placement_id: Uuid,
) -> DashboardResult<()> {
    sqlx::query("DELETE FROM dashboard_components WHERE id = $1")
        .bind(placement_id)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

const PLACEMENT_SELECT: &str = r#"
    SELECT dashboard_components.id,
           dashboard_components.position,
           dashboard_components.config,
           component_versions.id AS component_version_id,
           component_versions.component_id,
           components.name AS component_name,
           components.slug AS component_slug,
           component_versions.component_type::text AS component_type,
           component_versions.version_number,
           component_versions.version_label,
           component_versions.status::text AS version_status,
           component_versions.dataset_id
    FROM dashboard_components
    JOIN component_versions ON component_versions.id = dashboard_components.component_version_id
    JOIN components ON components.id = component_versions.component_id
    WHERE dashboard_components.dashboard_id = $1
    ORDER BY dashboard_components.position, dashboard_components.id
"#;

fn map_dashboard(row: sqlx::postgres::PgRow) -> DashboardResult<DashboardRecord> {
    Ok(DashboardRecord {
        id: row.try_get("id")?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        placement_count: row.try_get("placement_count")?,
    })
}

fn map_placement(row: sqlx::postgres::PgRow) -> DashboardResult<DashboardPlacementRecord> {
    Ok(DashboardPlacementRecord {
        id: row.try_get("id")?,
        position: row.try_get("position")?,
        config: row.try_get("config")?,
        component_version_id: row.try_get("component_version_id")?,
        component_id: row.try_get("component_id")?,
        component_name: row.try_get("component_name")?,
        component_slug: row.try_get("component_slug")?,
        component_type: row.try_get("component_type")?,
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        version_status: row.try_get("version_status")?,
        dataset_id: row.try_get("dataset_id")?,
    })
}

fn map_component_version(row: sqlx::postgres::PgRow) -> DashboardResult<ComponentVersionRecord> {
    Ok(ComponentVersionRecord {
        id: row.try_get("id")?,
        component_id: row.try_get("component_id")?,
        component_name: row.try_get("component_name")?,
        component_slug: row.try_get("component_slug")?,
        component_type: row.try_get("component_type")?,
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        version_status: row.try_get("version_status")?,
        dataset_id: row.try_get("dataset_id")?,
    })
}

fn map_dataset_scopes(
    rows: Vec<sqlx::postgres::PgRow>,
) -> DashboardResult<BTreeMap<Uuid, Vec<Uuid>>> {
    let mut mapped = BTreeMap::<Uuid, Vec<Uuid>>::new();
    for row in rows {
        mapped
            .entry(row.try_get("dataset_id")?)
            .or_default()
            .push(row.try_get("node_id")?);
    }
    Ok(mapped)
}
