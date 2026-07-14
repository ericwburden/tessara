//! Transactional full-layout Dashboard reconciliation.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use sqlx::PgPool;
use tessara_core::grid_layout::derive_row_major_positions;
use tessara_dashboards::{
    DASHBOARD_GRID_CONSTRAINTS, DashboardPlacementConfigState, DashboardPlacementConfigV1,
    DashboardPlacementSizePolicy, GridPlacement, GridRect, ParsedDashboardPlacementConfig,
    encode_dashboard_placement_config, validate_dashboard_layout,
};
use uuid::Uuid;

use crate::{
    auth::{self, AccountContext},
    error::ApiError,
};

use super::{
    dto::{
        DashboardCompositionCommand, DashboardCompositionResponse, DashboardPlacementGeometry,
        DashboardPlacementIdMapping, ReconcileDashboardCompositionRequest,
    },
    error::{DashboardResult, DashboardServiceError},
    projection::{
        DashboardProjectionContext, assemble_dashboard_response, distinct_dataset_ids,
        load_component_options_tx, parse_stored_placements,
    },
    repository::{self, DashboardPlacementRecord},
    scope::{overlaps as boundary_allows_dataset, require_contains as require_boundary_contains},
};

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum CandidateKey {
    Existing(Uuid),
    New(String),
}

#[derive(Clone, Debug)]
struct CandidateDraft {
    key: CandidateKey,
    existing_id: Option<Uuid>,
    client_key: Option<String>,
    component_version_id: Uuid,
    geometry: DashboardPlacementGeometry,
    requested_title: Option<String>,
    repair_requested: bool,
    stored: Option<ParsedDashboardPlacementConfig>,
    replaces_binding: bool,
}

#[derive(Clone, Debug)]
struct CandidatePlacement {
    key: CandidateKey,
    existing_id: Option<Uuid>,
    client_key: Option<String>,
    component_version_id: Uuid,
    rect: GridRect,
    config: Value,
}

pub(super) async fn reconcile_composition(
    pool: &PgPool,
    account: &AccountContext,
    dashboard_id: Uuid,
    payload: ReconcileDashboardCompositionRequest,
) -> DashboardResult<DashboardCompositionResponse> {
    let component_boundary = auth::capability_boundary(pool, account, "components:read").await?;
    let dashboard_manage_boundary =
        auth::capability_boundary(pool, account, "dashboards:manage").await?;
    let mut tx = pool.begin().await?;
    let mut locked_dashboard = repository::lock_dashboard(&mut tx, dashboard_id)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("dashboard {dashboard_id}")))?;
    let dashboard_scope =
        repository::load_dashboard_scope_node_ids_tx(&mut tx, dashboard_id).await?;
    require_boundary_contains(
        &dashboard_manage_boundary,
        &dashboard_scope,
        "dashboards:manage",
    )?;
    let current = repository::load_locked_placements(&mut tx, dashboard_id).await?;
    let parsed_current = parse_stored_placements(&current)?;
    let current_by_id = current
        .iter()
        .map(|row| (row.id, row))
        .collect::<BTreeMap<_, _>>();
    let parsed_by_id = parsed_current.into_iter().collect::<BTreeMap<_, _>>();

    let mut seen_existing = BTreeSet::new();
    let mut seen_client_keys = BTreeSet::new();
    let mut removed_ids = Vec::new();
    let mut drafts = Vec::new();
    for command in payload.commands {
        match command {
            DashboardCompositionCommand::Retain {
                placement_id,
                geometry,
                title,
                repair,
            } => {
                let row =
                    require_current_placement(&current_by_id, &mut seen_existing, placement_id)?;
                drafts.push(CandidateDraft {
                    key: CandidateKey::Existing(placement_id),
                    existing_id: Some(placement_id),
                    client_key: None,
                    component_version_id: row.component_version_id,
                    geometry,
                    requested_title: title,
                    repair_requested: repair,
                    stored: Some(require_parsed_config(&parsed_by_id, placement_id)?.clone()),
                    replaces_binding: false,
                });
            }
            DashboardCompositionCommand::Bind {
                placement_id,
                client_key,
                component_version_id,
                geometry,
                title,
            } => match (placement_id, client_key) {
                (Some(placement_id), None) => {
                    require_current_placement(&current_by_id, &mut seen_existing, placement_id)?;
                    drafts.push(CandidateDraft {
                        key: CandidateKey::Existing(placement_id),
                        existing_id: Some(placement_id),
                        client_key: None,
                        component_version_id,
                        geometry,
                        requested_title: title,
                        repair_requested: false,
                        stored: Some(require_parsed_config(&parsed_by_id, placement_id)?.clone()),
                        // Every Bind is an explicit candidate-binding request.
                        // Treating a same-id Bind as Retain would expose a UUID
                        // equality oracle for an otherwise redacted binding.
                        replaces_binding: true,
                    });
                }
                (None, Some(client_key)) => {
                    let client_key = client_key.trim().to_string();
                    if client_key.is_empty() || !seen_client_keys.insert(client_key.clone()) {
                        return Err(ApiError::BadRequest(
                            "new Dashboard placements require unique non-empty client_key values"
                                .to_string(),
                        )
                        .into());
                    }
                    drafts.push(CandidateDraft {
                        key: CandidateKey::New(client_key.clone()),
                        existing_id: None,
                        client_key: Some(client_key),
                        component_version_id,
                        geometry,
                        requested_title: title,
                        repair_requested: false,
                        stored: None,
                        replaces_binding: true,
                    });
                }
                _ => {
                    return Err(ApiError::BadRequest(
                        "bind commands require exactly one of placement_id or client_key"
                            .to_string(),
                    )
                    .into());
                }
            },
            DashboardCompositionCommand::Remove { placement_id } => {
                require_current_placement(&current_by_id, &mut seen_existing, placement_id)?;
                removed_ids.push(placement_id);
            }
        }
    }

    let current_ids = current_by_id.keys().copied().collect::<BTreeSet<_>>();
    if seen_existing != current_ids {
        return Err(DashboardServiceError::CompositionStale);
    }
    if drafts.len() > DASHBOARD_GRID_CONSTRAINTS.max_placements() {
        return Err(DashboardServiceError::PlacementLimit);
    }

    let version_ids = distinct_version_ids(drafts.iter().map(|draft| draft.component_version_id));
    let versions = repository::load_component_versions_locked(&mut tx, &version_ids).await?;
    let versions_by_id = versions
        .into_iter()
        .map(|version| (version.id, version))
        .collect::<BTreeMap<_, _>>();
    if versions_by_id.len() != version_ids.len() {
        return Err(DashboardServiceError::ComponentVersionUnavailable);
    }
    let dataset_ids =
        distinct_dataset_ids(versions_by_id.values().map(|version| version.dataset_id));
    let dataset_scopes = repository::load_dataset_scope_nodes_tx(&mut tx, &dataset_ids).await?;

    let current_dataset_scopes = current
        .iter()
        .map(|row| row.dataset_id)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let mut all_dataset_scopes =
        repository::load_dataset_scope_nodes_tx(&mut tx, &current_dataset_scopes).await?;
    for (dataset_id, nodes) in &dataset_scopes {
        all_dataset_scopes.insert(*dataset_id, nodes.clone());
    }

    let size_policy = DashboardPlacementSizePolicy::new();
    let mut candidates = Vec::with_capacity(drafts.len());
    for draft in drafts {
        let version = versions_by_id
            .get(&draft.component_version_id)
            .ok_or(DashboardServiceError::ComponentVersionUnavailable)?;
        if !matches!(version.version_status.as_str(), "published" | "superseded")
            && (draft.existing_id.is_none() || draft.replaces_binding)
        {
            return Err(DashboardServiceError::ComponentVersionUnavailable);
        }
        let dataset_nodes = dataset_scopes
            .get(&version.dataset_id)
            .cloned()
            .unwrap_or_default();
        let binding_changed = draft.existing_id.is_none() || draft.replaces_binding;
        let candidate_readable = boundary_allows_dataset(&component_boundary, &dataset_nodes);
        if binding_changed && !candidate_readable {
            return Err(DashboardServiceError::ComponentVersionUnavailable);
        }
        if binding_changed {
            if dataset_nodes.is_empty()
                || !dataset_nodes
                    .iter()
                    .all(|node_id| dashboard_scope.contains(node_id))
            {
                return Err(DashboardServiceError::ScopeIncompatible);
            }
            require_boundary_contains(
                &dashboard_manage_boundary,
                &dataset_nodes,
                "dashboards:manage",
            )?;
        }
        let mut current_metadata_readable = false;
        if let Some(existing_id) = draft.existing_id {
            let current_row = current_by_id
                .get(&existing_id)
                .copied()
                .ok_or(DashboardServiceError::CompositionStale)?;
            let current_nodes = all_dataset_scopes
                .get(&current_row.dataset_id)
                .cloned()
                .unwrap_or_default();
            current_metadata_readable = matches!(
                current_row.version_status.as_str(),
                "published" | "superseded"
            ) && !current_nodes.is_empty()
                && current_nodes
                    .iter()
                    .all(|node_id| dashboard_scope.contains(node_id))
                && boundary_allows_dataset(&component_boundary, &current_nodes);
            if !draft.replaces_binding
                && draft.requested_title.is_some()
                && !current_metadata_readable
            {
                return Err(DashboardServiceError::ComponentVersionUnavailable);
            }
        }

        let requested_rect = geometry_rect(draft.geometry);
        let (rect, config) = if let Some(stored) = draft.stored {
            match stored.config_state {
                DashboardPlacementConfigState::FutureSchema => {
                    if draft.repair_requested
                        || draft.replaces_binding
                        || draft.requested_title.is_some()
                        || requested_rect != stored.display_rect
                    {
                        return Err(DashboardServiceError::InvalidGeometry(
                            "future-schema placements may only be retained unchanged or removed"
                                .to_string(),
                        ));
                    }
                    (stored.display_rect, stored.raw_config)
                }
                DashboardPlacementConfigState::NeedsRepair if !draft.repair_requested => {
                    if draft.replaces_binding
                        || draft.requested_title.is_some()
                        || requested_rect != stored.display_rect
                    {
                        return Err(DashboardServiceError::InvalidGeometry(
                            "malformed V1 placements require repair: true before they may be changed"
                                .to_string(),
                        ));
                    }
                    (stored.display_rect, stored.raw_config)
                }
                DashboardPlacementConfigState::Valid
                | DashboardPlacementConfigState::Legacy
                | DashboardPlacementConfigState::NeedsRepair => {
                    let title = draft
                        .requested_title
                        .as_deref()
                        .map(normalized_title)
                        .unwrap_or_else(|| {
                            if draft.replaces_binding && !current_metadata_readable {
                                None
                            } else {
                                stored.title.clone()
                            }
                        });
                    let config = DashboardPlacementConfigV1::new_with_minimum(
                        title,
                        requested_rect,
                        size_policy.minimum_for(&version.component_type),
                    )
                    .map_err(DashboardServiceError::from)?;
                    let encoded = encode_dashboard_placement_config(&config)
                        .map_err(DashboardServiceError::from)?;
                    (config.rect(), encoded)
                }
            }
        } else {
            let config = DashboardPlacementConfigV1::new_with_minimum(
                draft.requested_title.as_deref().and_then(normalized_title),
                requested_rect,
                size_policy.minimum_for(&version.component_type),
            )
            .map_err(DashboardServiceError::from)?;
            let encoded =
                encode_dashboard_placement_config(&config).map_err(DashboardServiceError::from)?;
            (config.rect(), encoded)
        };
        candidates.push(CandidatePlacement {
            key: draft.key,
            existing_id: draft.existing_id,
            client_key: draft.client_key,
            component_version_id: draft.component_version_id,
            rect,
            config,
        });
    }

    let layout = candidates
        .iter()
        .map(|candidate| GridPlacement::new(candidate.key.clone(), candidate.rect))
        .collect::<Vec<_>>();
    validate_dashboard_layout(&layout).map_err(DashboardServiceError::from)?;
    let positions = derive_row_major_positions(&layout)
        .into_iter()
        .map(|(key, position)| {
            i32::try_from(position)
                .map(|position| (key, position))
                .map_err(|_| DashboardServiceError::PlacementLimit)
        })
        .collect::<DashboardResult<BTreeMap<_, _>>>()?;

    for placement_id in removed_ids {
        repository::delete_placement(&mut tx, placement_id).await?;
    }
    let mut new_placement_ids = Vec::new();
    for candidate in &candidates {
        let position = positions
            .get(&candidate.key)
            .copied()
            .ok_or(DashboardServiceError::CompositionStale)?;
        if let Some(placement_id) = candidate.existing_id {
            repository::update_placement(
                &mut tx,
                placement_id,
                candidate.component_version_id,
                position,
                &candidate.config,
            )
            .await?;
        } else {
            let placement_id = repository::insert_placement(
                &mut tx,
                dashboard_id,
                candidate.component_version_id,
                position,
                &candidate.config,
            )
            .await?;
            let client_key = candidate
                .client_key
                .clone()
                .ok_or(DashboardServiceError::CompositionStale)?;
            new_placement_ids.push(DashboardPlacementIdMapping {
                client_key,
                placement_id,
            });
        }
    }
    let canonical_placements = repository::load_locked_placements(&mut tx, dashboard_id).await?;
    let canonical_parsed = parse_stored_placements(&canonical_placements)?;
    let canonical_dataset_ids =
        distinct_dataset_ids(canonical_placements.iter().map(|row| row.dataset_id));
    let canonical_dataset_scopes =
        repository::load_dataset_scope_nodes_tx(&mut tx, &canonical_dataset_ids).await?;
    let canonical_visibility = repository::load_visibility_nodes_tx(&mut tx, dashboard_id).await?;
    let available_component_versions =
        load_component_options_tx(&mut tx, &component_boundary, &dashboard_scope).await?;
    locked_dashboard.placement_count = i64::try_from(canonical_placements.len())
        .map_err(|_| DashboardServiceError::PlacementLimit)?;
    let dashboard = assemble_dashboard_response(
        locked_dashboard,
        canonical_visibility,
        &canonical_placements,
        canonical_parsed,
        DashboardProjectionContext::editor(
            &component_boundary,
            &canonical_dataset_scopes,
            &dashboard_scope,
        ),
    );
    let response = DashboardCompositionResponse {
        dashboard,
        available_component_versions,
        new_placement_ids,
    };
    tx.commit().await?;
    Ok(response)
}

fn require_current_placement<'a>(
    current: &'a BTreeMap<Uuid, &'a DashboardPlacementRecord>,
    seen: &mut BTreeSet<Uuid>,
    placement_id: Uuid,
) -> DashboardResult<&'a DashboardPlacementRecord> {
    let row = current
        .get(&placement_id)
        .copied()
        .ok_or(DashboardServiceError::PlacementNotFound(placement_id))?;
    if !seen.insert(placement_id) {
        return Err(DashboardServiceError::CompositionStale);
    }
    Ok(row)
}

fn require_parsed_config(
    parsed: &BTreeMap<Uuid, ParsedDashboardPlacementConfig>,
    placement_id: Uuid,
) -> DashboardResult<&ParsedDashboardPlacementConfig> {
    parsed
        .get(&placement_id)
        .ok_or(DashboardServiceError::CompositionStale)
}

fn geometry_rect(geometry: DashboardPlacementGeometry) -> GridRect {
    GridRect::new(
        geometry.grid_row,
        geometry.grid_column,
        geometry.grid_width,
        geometry.grid_height,
    )
}

fn normalized_title(value: &str) -> Option<String> {
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_string())
}

fn distinct_version_ids(ids: impl Iterator<Item = Uuid>) -> Vec<Uuid> {
    ids.collect::<BTreeSet<_>>().into_iter().collect()
}
