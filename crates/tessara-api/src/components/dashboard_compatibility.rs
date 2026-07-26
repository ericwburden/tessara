//! Guards mutable published Component payloads against invalidating pinned
//! Dashboard layouts.
//!
//! Dashboard rows live in a separate database. Core therefore consumes the
//! module's bounded dependency projection and never joins Dashboard storage.
//! A Core advisory lock serializes concurrent published Component updates.

use std::collections::BTreeMap;

use serde_json::Value;
use sqlx::{Postgres, Row, Transaction};
use tessara_dashboards::{
    DashboardPlacementConfigInput, DashboardPlacementSizePolicy, parse_dashboard_placement_configs,
};
use uuid::Uuid;

use crate::{
    dashboard_dependencies,
    error::{ApiError, ApiResult},
};

const PUBLISHED_UPDATE_ADVISORY_LOCK: i64 = 0x5445_5353_4152_4135;
const DASHBOARD_COMPATIBILITY_ERROR: &str =
    "This Component update would make a pinned Dashboard layout undisplayable.";

#[derive(Clone, Debug)]
struct StoredDashboardPlacement {
    dashboard_id: Uuid,
    placement_id: Uuid,
    position: i32,
    config: Value,
    component_type: String,
}

/// Serializes update-in-place compatibility checks before the caller locks the
/// Component/ComponentVersion rows.
pub(super) async fn prepare_published_update(
    tx: &mut Transaction<'_, Postgres>,
    component_version_id: Uuid,
) -> ApiResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock($1)")
        .bind(PUBLISHED_UPDATE_ADVISORY_LOCK)
        .execute(&mut **tx)
        .await?;
    let _ = component_version_id;
    Ok(())
}

/// Re-reads every complete affected Dashboard after the candidate version row
/// has been updated. The transaction sees its own candidate kind, while the
/// version row lock prevents a concurrent app reconciliation from pinning an
/// unchecked payload before this transaction completes.
pub(super) async fn validate_published_update(
    tx: &mut Transaction<'_, Postgres>,
    component_version_id: Uuid,
) -> ApiResult<()> {
    let projection = dashboard_dependencies::load().await?;
    let affected = projection
        .dashboards
        .into_iter()
        .filter(|dashboard| {
            dashboard
                .placements
                .iter()
                .any(|placement| placement.component_version_id == component_version_id)
        })
        .collect::<Vec<_>>();
    let component_ids = affected
        .iter()
        .flat_map(|dashboard| {
            dashboard
                .placements
                .iter()
                .map(|placement| placement.component_version_id)
        })
        .collect::<Vec<_>>();
    let component_types = sqlx::query(
        "SELECT id,component_type::text AS component_type
         FROM component_versions WHERE id=ANY($1)",
    )
    .bind(&component_ids)
    .fetch_all(&mut **tx)
    .await?
    .into_iter()
    .map(|row| {
        Ok((
            row.try_get::<Uuid, _>("id")?,
            row.try_get::<String, _>("component_type")?,
        ))
    })
    .collect::<Result<BTreeMap<_, _>, sqlx::Error>>()?;
    let mut rows = Vec::new();
    for dashboard in affected {
        for placement in dashboard.placements {
            if let Some(component_type) = component_types.get(&placement.component_version_id) {
                rows.push(StoredDashboardPlacement {
                    dashboard_id: dashboard.dashboard_id,
                    placement_id: placement.placement_id,
                    position: placement.position,
                    config: placement.config,
                    component_type: component_type.clone(),
                });
            }
        }
    }

    validate_candidate_layouts(&rows)
}

fn validate_candidate_layouts(rows: &[StoredDashboardPlacement]) -> ApiResult<()> {
    let size_policy = DashboardPlacementSizePolicy::new();
    let mut dashboards = BTreeMap::<Uuid, Vec<DashboardPlacementConfigInput<Uuid>>>::new();
    for row in rows {
        dashboards
            .entry(row.dashboard_id)
            .or_default()
            .push(DashboardPlacementConfigInput::new(
                row.placement_id,
                row.position,
                row.config.clone(),
                size_policy.minimum_for(&row.component_type),
            ));
    }
    for placements in dashboards.values() {
        parse_dashboard_placement_configs(placements)
            .map_err(|_| ApiError::BadRequest(DASHBOARD_COMPATIBILITY_ERROR.into()))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use uuid::Uuid;

    use super::{
        DASHBOARD_COMPATIBILITY_ERROR, StoredDashboardPlacement, validate_candidate_layouts,
    };

    fn placement(
        dashboard_id: Uuid,
        position: i32,
        component_type: &str,
        row: i32,
        column: i32,
        width: i32,
        height: i32,
    ) -> StoredDashboardPlacement {
        StoredDashboardPlacement {
            dashboard_id,
            placement_id: Uuid::new_v4(),
            position,
            config: json!({
                "schema_version": 1,
                "grid_row": row,
                "grid_column": column,
                "grid_width": width,
                "grid_height": height,
            }),
            component_type: component_type.into(),
        }
    }

    #[test]
    fn candidate_table_kind_is_rejected_when_reclassification_has_no_fallback_row() {
        let dashboard_id = Uuid::new_v4();
        let rows = vec![
            placement(dashboard_id, 0, "bar", 1, 1, 1, 240),
            // This geometry was valid for a chart. With the candidate Table
            // kind it needs repair, but every fallback row is occupied.
            placement(dashboard_id, 1, "table", 1, 2, 1, 1),
        ];

        let error = validate_candidate_layouts(&rows)
            .expect_err("candidate kind must preserve a displayable full layout");
        assert_eq!(
            error.to_string(),
            format!("bad request: {DASHBOARD_COMPATIBILITY_ERROR}")
        );
    }

    #[test]
    fn safe_candidate_kind_update_keeps_the_dashboard_displayable() {
        let dashboard_id = Uuid::new_v4();
        let rows = vec![
            placement(dashboard_id, 0, "bar", 1, 1, 1, 240),
            // A Table-valid rectangle remains explicit and does not need a
            // full-width fallback row.
            placement(dashboard_id, 1, "table", 1, 2, 6, 4),
        ];

        validate_candidate_layouts(&rows).expect("safe kind update should remain displayable");
    }

    #[test]
    fn layouts_are_validated_per_dashboard_not_as_one_global_grid() {
        let rows = vec![
            placement(Uuid::new_v4(), 0, "bar", 1, 1, 12, 1),
            placement(Uuid::new_v4(), 0, "bar", 1, 1, 12, 1),
        ];

        validate_candidate_layouts(&rows).expect("independent Dashboards may reuse coordinates");
    }
}
