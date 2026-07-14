//! Authorization-aware Dashboard response projection.
//!
//! This module owns stored-placement decoding, redaction, editor operation
//! envelopes, and Component picker projection. Command orchestration remains
//! in `service`, while SQL remains in `repository`.

use std::collections::{BTreeMap, BTreeSet};

use sqlx::{Postgres, Transaction};
use tessara_dashboards::{
    DashboardPlacementConfigInput, DashboardPlacementConfigState, DashboardPlacementOperation,
    DashboardPlacementSizePolicy, ParsedDashboardPlacementConfig,
    parse_dashboard_placement_configs,
};
use uuid::Uuid;

use crate::{auth::CapabilityBoundary, error::ApiError};

use super::{
    dto::{
        DashboardComponentVersionOption, DashboardComponentVersionSummary,
        DashboardPlacementAvailability, DashboardPlacementResponse, DashboardResponse,
        DashboardVisibilityNodeSummary,
    },
    error::{DashboardResult, DashboardServiceError},
    repository::{self, DashboardPlacementRecord, DashboardRecord},
    scope::{contains as boundary_contains_nodes, overlaps as boundary_allows_dataset},
};

/// Read-only inputs that govern one placement projection pass.
pub(super) struct DashboardProjectionContext<'a> {
    component_boundary: &'a CapabilityBoundary,
    dataset_scopes: &'a BTreeMap<Uuid, Vec<Uuid>>,
    dashboard_scope: &'a [Uuid],
    can_manage: bool,
    editor: bool,
}

impl<'a> DashboardProjectionContext<'a> {
    pub(super) const fn editor(
        component_boundary: &'a CapabilityBoundary,
        dataset_scopes: &'a BTreeMap<Uuid, Vec<Uuid>>,
        dashboard_scope: &'a [Uuid],
    ) -> Self {
        Self {
            component_boundary,
            dataset_scopes,
            dashboard_scope,
            can_manage: true,
            editor: true,
        }
    }
}

pub(super) async fn build_dashboard_response_tx(
    tx: &mut Transaction<'_, Postgres>,
    dashboard: DashboardRecord,
    dashboard_scope: &[Uuid],
    dashboard_boundary: &CapabilityBoundary,
    dashboard_manage_boundary: &CapabilityBoundary,
    component_boundary: &CapabilityBoundary,
    editor: bool,
) -> DashboardResult<DashboardResponse> {
    let mut visibility = repository::load_visibility_nodes_unlocked_tx(tx, dashboard.id).await?;
    match dashboard_boundary {
        CapabilityBoundary::Scoped(node_ids) => {
            visibility.retain(|node| node_ids.contains(&node.node_id));
        }
        CapabilityBoundary::Global => {}
        CapabilityBoundary::None => {
            return Err(ApiError::Forbidden("dashboards:read".to_string()).into());
        }
    }
    let placements = repository::load_placements_tx(tx, dashboard.id).await?;
    let dataset_ids = distinct_dataset_ids(placements.iter().map(|row| row.dataset_id));
    let dataset_scopes = repository::load_dataset_scope_nodes_unlocked_tx(tx, &dataset_ids).await?;
    let parsed = parse_stored_placements(&placements)?;
    let context = DashboardProjectionContext {
        component_boundary,
        dataset_scopes: &dataset_scopes,
        dashboard_scope,
        can_manage: boundary_contains_nodes(dashboard_manage_boundary, dashboard_scope),
        editor,
    };
    Ok(assemble_dashboard_response(
        dashboard,
        visibility,
        &placements,
        parsed,
        context,
    ))
}

pub(super) fn assemble_dashboard_response(
    dashboard: DashboardRecord,
    visibility: Vec<repository::DashboardVisibilityRecord>,
    placements: &[DashboardPlacementRecord],
    parsed: Vec<(Uuid, ParsedDashboardPlacementConfig)>,
    context: DashboardProjectionContext<'_>,
) -> DashboardResponse {
    let placements: Vec<DashboardPlacementResponse> = placements
        .iter()
        .zip(parsed)
        .map(|(row, (placement_id, parsed))| {
            debug_assert_eq!(row.id, placement_id);
            placement_response(row, parsed, &context)
        })
        .collect();
    let placement_count = i64::try_from(placements.len())
        .expect("placement count is bounded by Dashboard grid constraints");
    DashboardResponse {
        id: dashboard.id,
        name: dashboard.name,
        description: dashboard.description,
        visibility_nodes: map_visibility(visibility),
        placement_count,
        can_manage: context.can_manage,
        placements,
    }
}

pub(super) async fn load_component_options_tx(
    tx: &mut Transaction<'_, Postgres>,
    component_boundary: &CapabilityBoundary,
    dashboard_scope: &[Uuid],
) -> DashboardResult<Vec<DashboardComponentVersionOption>> {
    if component_boundary == &CapabilityBoundary::None {
        return Ok(Vec::new());
    }
    let versions = repository::load_placeable_component_versions_tx(tx).await?;
    let dataset_ids = distinct_dataset_ids(versions.iter().map(|version| version.dataset_id));
    let dataset_scopes = repository::load_dataset_scope_nodes_unlocked_tx(tx, &dataset_ids).await?;
    let policy = DashboardPlacementSizePolicy::new();
    Ok(versions
        .into_iter()
        .filter(|version| {
            let nodes = dataset_scopes
                .get(&version.dataset_id)
                .map(Vec::as_slice)
                .unwrap_or_default();
            !nodes.is_empty()
                && boundary_allows_dataset(component_boundary, nodes)
                && nodes
                    .iter()
                    .all(|node_id| dashboard_scope.contains(node_id))
        })
        .map(|version| {
            let recommended = policy.recommended_for(&version.component_type);
            DashboardComponentVersionOption {
                component_version_id: version.id,
                component_id: version.component_id,
                component_name: version.component_name,
                component_slug: version.component_slug,
                component_type: version.component_type,
                version_number: version.version_number,
                version_label: version.version_label,
                version_status: version.version_status,
                default_grid_width: u16::try_from(recommended.width)
                    .expect("validated recommended width"),
                default_grid_height: u16::try_from(recommended.height)
                    .expect("validated recommended height"),
            }
        })
        .collect())
}

fn placement_response(
    row: &DashboardPlacementRecord,
    parsed: ParsedDashboardPlacementConfig,
    context: &DashboardProjectionContext<'_>,
) -> DashboardPlacementResponse {
    let dataset_nodes = context
        .dataset_scopes
        .get(&row.dataset_id)
        .map(Vec::as_slice)
        .unwrap_or_default();
    let readable = matches!(row.version_status.as_str(), "published" | "superseded")
        && parsed.is_executable()
        && !dataset_nodes.is_empty()
        && dataset_nodes
            .iter()
            .all(|node_id| context.dashboard_scope.contains(node_id))
        && boundary_allows_dataset(context.component_boundary, dataset_nodes);
    let allowed_operations = context.editor.then(|| match parsed.config_state {
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
            if readable =>
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
        DashboardPlacementConfigState::Valid | DashboardPlacementConfigState::Legacy => vec![
            DashboardPlacementOperation::Retain,
            DashboardPlacementOperation::Move,
            DashboardPlacementOperation::Resize,
            DashboardPlacementOperation::Remove,
        ],
    });
    DashboardPlacementResponse {
        placement_id: row.id,
        position: row.position,
        grid_row: parsed.display_rect.row,
        grid_column: parsed.display_rect.column,
        grid_width: parsed.display_rect.width,
        grid_height: parsed.display_rect.height,
        availability: if readable {
            DashboardPlacementAvailability::Available
        } else {
            DashboardPlacementAvailability::Unavailable
        },
        config_state: context.editor.then_some(parsed.config_state),
        title: readable.then_some(parsed.title).flatten(),
        component: readable.then(|| DashboardComponentVersionSummary {
            component_version_id: row.component_version_id,
            component_id: row.component_id,
            component_name: row.component_name.clone(),
            component_slug: row.component_slug.clone(),
            component_type: row.component_type.clone(),
            version_number: row.version_number,
            version_label: row.version_label.clone(),
            version_status: row.version_status.clone(),
        }),
        allowed_operations,
    }
}

pub(super) fn parse_stored_placements(
    placements: &[DashboardPlacementRecord],
) -> DashboardResult<Vec<(Uuid, ParsedDashboardPlacementConfig)>> {
    let size_policy = DashboardPlacementSizePolicy::new();
    let inputs = placements
        .iter()
        .map(|row| {
            DashboardPlacementConfigInput::new(
                row.id,
                row.position,
                row.config.clone(),
                size_policy.minimum_for(&row.component_type),
            )
        })
        .collect::<Vec<_>>();
    Ok(parse_dashboard_placement_configs(&inputs)
        .map_err(DashboardServiceError::from)?
        .into_iter()
        .map(|placement| (placement.placement_id, placement.config))
        .collect())
}

pub(super) fn distinct_dataset_ids(ids: impl Iterator<Item = Uuid>) -> Vec<Uuid> {
    ids.collect::<BTreeSet<_>>().into_iter().collect()
}

pub(super) fn map_visibility(
    records: Vec<repository::DashboardVisibilityRecord>,
) -> Vec<DashboardVisibilityNodeSummary> {
    records
        .into_iter()
        .map(|record| DashboardVisibilityNodeSummary {
            node_id: record.node_id,
            node_name: record.node_name,
            node_type_name: record.node_type_name,
            parent_node_id: record.parent_node_id,
            node_path: record.node_path,
        })
        .collect()
}
