//! Transactional synchronization for the Core-owned transition catalog.

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use super::{
    catalog::{
        CatalogContractError, CatalogInput, PreparedCatalogSource, canonical_inputs,
        ensure_compatible_source_change, frozen_definition_ids, frozen_source_input,
        prepare_catalog, prepare_source, source_digest,
    },
    repository::{self, NavigationPolicyEntryRow, NavigationRecord},
};

const MODULE_CAPABILITIES: [(&str, &str); 2] = [
    (
        "modules:read",
        "Inspect module inventory and transition contribution metadata",
    ),
    (
        "modules:manage_navigation",
        "Manage installation navigation visibility and ordering",
    ),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CatalogSyncOutcome {
    pub(crate) installation_id: Uuid,
    pub(crate) changed_definition_ids: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct CoreRuntimeReadModel {
    pub(crate) provenance: String,
    pub(crate) observed_version: String,
    pub(crate) finding_code: String,
    pub(crate) observed_at: DateTime<Utc>,
}

#[derive(Clone, Debug)]
pub(crate) struct TransitionCatalogReadModel {
    pub(crate) definition_id: String,
    /// Retained for persistence identity/no-op proof; not exposed on the wire.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) source_id: Uuid,
    /// Retained for persistence identity/no-op proof; not exposed on the wire.
    #[cfg_attr(not(test), allow(dead_code))]
    pub(crate) projection_id: Uuid,
    pub(crate) source_digest: String,
    pub(crate) content_type: String,
    pub(crate) source_bytes: Vec<u8>,
    pub(crate) normalized_projection: Value,
}

#[derive(Clone, Debug)]
pub(crate) struct ModuleInventoryReadModel {
    pub(crate) installation_id: Uuid,
    pub(crate) installation_created_at: DateTime<Utc>,
    pub(crate) core_runtime: CoreRuntimeReadModel,
    pub(crate) transitions: Vec<TransitionCatalogReadModel>,
}

#[derive(Clone, Debug)]
pub(crate) struct DescriptorDocumentReadModel {
    pub(crate) source_digest: String,
    pub(crate) content_type: String,
    pub(crate) source_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationPolicyReadModel {
    pub(crate) installation_id: Uuid,
    pub(crate) revision: i64,
    pub(crate) entries: Vec<NavigationPolicyEntry>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationPolicyEntry {
    pub(crate) contribution_id: String,
    pub(crate) definition_id: String,
    pub(crate) destination: String,
    pub(crate) label: String,
    pub(crate) group: String,
    pub(crate) reorder_band: String,
    pub(crate) source_order_hint: i32,
    pub(crate) default_policy_order: i32,
    pub(crate) required_capabilities_any_of: Vec<String>,
    pub(crate) visible: bool,
    pub(crate) order: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NavigationPolicyUpdateEntry {
    pub(crate) contribution_id: String,
    pub(crate) group: String,
    pub(crate) reorder_band: String,
    pub(crate) visible: bool,
    pub(crate) order: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SyncFailurePoint {
    Sources,
    Projections,
    Capabilities,
    Navigation,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CatalogSyncError {
    #[error(transparent)]
    Contract(#[from] CatalogContractError),
    #[error("stored source for '{definition_id}' does not match its recorded digest")]
    StoredSourceDigestMismatch { definition_id: String },
    #[error("stored reservation metadata for '{definition_id}' does not match the catalog")]
    StoredReservationMismatch { definition_id: String },
    #[error("stored normalized projection for '{definition_id}' does not match its source")]
    StoredProjectionMismatch { definition_id: String },
    #[error("stored findings for '{definition_id}' do not match its normalized projection")]
    StoredFindingsMismatch { definition_id: String },
    #[error("stored navigation contribution for '{contribution_id}' has immutable metadata drift")]
    StoredNavigationMismatch { contribution_id: String },
    #[error("stored transition catalog does not contain exactly the seven frozen definitions")]
    StoredCatalogShapeMismatch,
    #[error("Core capability '{key}' required by '{definition_id}' is not registered")]
    CapabilityNotRegistered { definition_id: String, key: String },
    #[error("transition capability description for '{key}' in '{definition_id}' differs from Core")]
    CapabilityDescriptionMismatch { definition_id: String, key: String },
    #[error("Core capability '{key}' has incompatible module-control-plane metadata")]
    CapabilityMetadataMismatch { key: String },
    #[error("catalog synchronization failed at injected checkpoint '{0:?}'")]
    InjectedFailure(SyncFailurePoint),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl CatalogSyncError {
    pub(crate) fn stable_code(&self) -> &'static str {
        match self {
            Self::Contract(error) => error.stable_code(),
            Self::StoredSourceDigestMismatch { .. } => "stored_transition_source_digest_mismatch",
            Self::StoredReservationMismatch { .. } => "stored_transition_reservation_mismatch",
            Self::StoredProjectionMismatch { .. } => "stored_transition_projection_mismatch",
            Self::StoredFindingsMismatch { .. } => "stored_transition_findings_mismatch",
            Self::StoredNavigationMismatch { .. } => "stored_navigation_contribution_mismatch",
            Self::StoredCatalogShapeMismatch => "stored_transition_catalog_shape_mismatch",
            Self::CapabilityNotRegistered { .. } => "transition_capability_not_registered",
            Self::CapabilityDescriptionMismatch { .. } => {
                "transition_capability_description_mismatch"
            }
            Self::CapabilityMetadataMismatch { .. } => "module_capability_metadata_mismatch",
            Self::InjectedFailure(_) => "module_catalog_sync_injected_failure",
            Self::Database(_) => "module_catalog_sync_database_error",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum CatalogReadError {
    #[error("module catalog integrity validation failed: {code}")]
    Integrity { code: &'static str },
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl CatalogReadError {
    pub(crate) fn stable_code(&self) -> &'static str {
        match self {
            Self::Integrity { code } => code,
            Self::Database(_) => "module_catalog_read_database_error",
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum NavigationPolicyUpdateError {
    #[error("navigation policy revision {presented} conflicts with current revision {current}")]
    RevisionConflict { presented: i64, current: i64 },
    #[error("permanent Core navigation item '{contribution_id}' is policy-immutable")]
    CoreItemImmutable { contribution_id: String },
    #[error("navigation policy contains duplicate contribution '{contribution_id}'")]
    DuplicateContribution { contribution_id: String },
    #[error("navigation policy contains unknown contribution '{contribution_id}'")]
    UnknownContribution { contribution_id: String },
    #[error("navigation policy omits contribution '{contribution_id}'")]
    MissingContribution { contribution_id: String },
    #[error("navigation contribution '{contribution_id}' cannot change group")]
    GroupChangeForbidden { contribution_id: String },
    #[error("navigation contribution '{contribution_id}' cannot change reorder band")]
    BandChangeForbidden { contribution_id: String },
    #[error("navigation band '{reorder_band}' must contain a dense zero-based order")]
    InvalidBandOrder { reorder_band: String },
    #[error("navigation policy revision must be non-negative")]
    InvalidRevision,
    #[error("stored navigation policy failed integrity validation")]
    Integrity,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl NavigationPolicyUpdateError {
    pub(crate) fn stable_code(&self) -> &'static str {
        match self {
            Self::RevisionConflict { .. } => "navigation_policy_revision_conflict",
            Self::CoreItemImmutable { .. } => "navigation_policy_core_item_immutable",
            Self::DuplicateContribution { .. } => "navigation_policy_duplicate_contribution",
            Self::UnknownContribution { .. } => "navigation_policy_unknown_contribution",
            Self::MissingContribution { .. } => "navigation_policy_missing_contribution",
            Self::GroupChangeForbidden { .. } => "navigation_policy_group_change_forbidden",
            Self::BandChangeForbidden { .. } => "navigation_policy_band_change_forbidden",
            Self::InvalidBandOrder { .. } => "navigation_policy_order_invalid",
            Self::InvalidRevision => "navigation_policy_revision_invalid",
            Self::Integrity => "navigation_policy_integrity_mismatch",
            Self::Database(_) => "navigation_policy_database_error",
        }
    }

    fn auditable_denial(&self) -> bool {
        !matches!(self, Self::Database(_) | Self::Integrity)
    }
}

struct WorkingSource {
    prepared: PreparedCatalogSource,
    source_id: Option<Uuid>,
    projection_id: Option<Uuid>,
    before_digest: Option<String>,
    before_finding_codes: Vec<String>,
    changed: bool,
}

pub(crate) async fn load_module_inventory(
    pool: &PgPool,
) -> Result<ModuleInventoryReadModel, CatalogReadError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let inventory = load_module_inventory_in_transaction(&mut tx).await?;
    tx.commit().await?;
    Ok(inventory)
}

pub(crate) async fn load_transition_detail(
    pool: &PgPool,
    definition_id: &str,
) -> Result<Option<TransitionCatalogReadModel>, CatalogReadError> {
    Ok(load_module_inventory(pool)
        .await?
        .transitions
        .into_iter()
        .find(|entry| entry.definition_id == definition_id))
}

pub(crate) async fn load_descriptor_document(
    pool: &PgPool,
    definition_id: &str,
) -> Result<Option<DescriptorDocumentReadModel>, CatalogReadError> {
    Ok(load_transition_detail(pool, definition_id)
        .await?
        .map(|entry| DescriptorDocumentReadModel {
            source_digest: entry.source_digest,
            content_type: entry.content_type,
            source_bytes: entry.source_bytes,
        }))
}

pub(crate) async fn load_navigation_policy(
    pool: &PgPool,
) -> Result<NavigationPolicyReadModel, CatalogReadError> {
    let mut tx = pool.begin().await?;
    sqlx::query("SET TRANSACTION ISOLATION LEVEL REPEATABLE READ, READ ONLY")
        .execute(&mut *tx)
        .await?;
    let installation_id = repository::installation_id(&mut tx).await?;
    let revision = repository::load_navigation_policy_revision(&mut tx, installation_id).await?;
    let rows = repository::load_navigation_policy_entries(&mut tx, installation_id).await?;
    let policy = navigation_policy_model(installation_id, revision, rows).map_err(|()| {
        CatalogReadError::Integrity {
            code: "navigation_policy_integrity_mismatch",
        }
    })?;
    tx.commit().await?;
    Ok(policy)
}

pub(crate) async fn update_navigation_policy(
    pool: &PgPool,
    actor_account_id: Uuid,
    correlation_id: Uuid,
    expected_revision: i64,
    entries: Vec<NavigationPolicyUpdateEntry>,
) -> Result<NavigationPolicyReadModel, NavigationPolicyUpdateError> {
    let result = update_navigation_policy_transaction(
        pool,
        actor_account_id,
        correlation_id,
        expected_revision,
        entries,
    )
    .await;
    if let Err(error) = &result
        && error.auditable_denial()
    {
        repository::record_navigation_policy_denial(
            pool,
            actor_account_id,
            correlation_id,
            Some(expected_revision),
            error.stable_code(),
        )
        .await?;
    }
    result
}

pub(crate) async fn record_navigation_policy_authorization_denial(
    pool: &PgPool,
    actor_account_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), sqlx::Error> {
    repository::record_navigation_policy_denial(
        pool,
        actor_account_id,
        correlation_id,
        None,
        "modules_manage_navigation_global_required",
    )
    .await
}

async fn load_module_inventory_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
) -> Result<ModuleInventoryReadModel, CatalogReadError> {
    let installation = repository::load_installation(tx).await?;
    let observation = repository::load_core_runtime_observation(tx, installation.id).await?;
    if observation.installation_id != installation.id
        || observation.provenance != "development_unresolved"
        || observation.finding_code != "core_release_provenance_unresolved"
    {
        return Err(CatalogReadError::Integrity {
            code: "core_runtime_observation_integrity_mismatch",
        });
    }

    let rows = repository::load_catalog_projections(tx).await?;
    let mut by_definition = rows
        .into_iter()
        .map(|row| (row.definition_id.clone(), row))
        .collect::<BTreeMap<_, _>>();
    let expected_ids = frozen_definition_ids();
    if by_definition.len() != expected_ids.len()
        || expected_ids
            .iter()
            .any(|definition_id| !by_definition.contains_key(definition_id))
    {
        return Err(CatalogReadError::Integrity {
            code: "stored_transition_catalog_shape_mismatch",
        });
    }

    let mut transitions = Vec::with_capacity(expected_ids.len());
    for definition_id in expected_ids {
        let row = by_definition
            .remove(&definition_id)
            .ok_or(CatalogReadError::Integrity {
                code: "stored_transition_catalog_shape_mismatch",
            })?;
        let input = frozen_source_input(
            &definition_id,
            row.current.source_bytes.clone(),
            row.current.source_digest.clone(),
        )
        .ok_or(CatalogReadError::Integrity {
            code: "stored_transition_catalog_shape_mismatch",
        })?;
        let prepared = prepare_source(&input).map_err(|error| CatalogReadError::Integrity {
            code: error.stable_code(),
        })?;
        if row.display_name != prepared.display_name
            || source_digest(&row.current.source_bytes) != row.current.source_digest
        {
            return Err(CatalogReadError::Integrity {
                code: "stored_transition_source_digest_mismatch",
            });
        }
        let expected_projection =
            prepared
                .normalized_projection(installation.id)
                .map_err(|error| CatalogReadError::Integrity {
                    code: error.stable_code(),
                })?;
        if row.current.schema_version != 1
            || row.current.content_type != "application/json"
            || row.current.projection_source_id != row.current.source_id
            || row.current.projection_installation_id != installation.id
            || row.current.provider_eligible
            || row.current.supervisor_materializable
            || row.current.normalized_projection != expected_projection
        {
            return Err(CatalogReadError::Integrity {
                code: "stored_transition_projection_mismatch",
            });
        }
        let findings = repository::load_findings(tx, row.current.projection_id).await?;
        if findings != prepared.findings {
            return Err(CatalogReadError::Integrity {
                code: "stored_transition_findings_mismatch",
            });
        }
        transitions.push(TransitionCatalogReadModel {
            definition_id,
            source_id: row.current.source_id,
            projection_id: row.current.projection_id,
            source_digest: row.current.source_digest,
            content_type: row.current.content_type,
            source_bytes: row.current.source_bytes,
            normalized_projection: row.current.normalized_projection,
        });
    }

    Ok(ModuleInventoryReadModel {
        installation_id: installation.id,
        installation_created_at: installation.created_at,
        core_runtime: CoreRuntimeReadModel {
            provenance: observation.provenance,
            observed_version: observation.observed_version,
            finding_code: observation.finding_code,
            observed_at: observation.observed_at,
        },
        transitions,
    })
}

async fn update_navigation_policy_transaction(
    pool: &PgPool,
    actor_account_id: Uuid,
    correlation_id: Uuid,
    expected_revision: i64,
    entries: Vec<NavigationPolicyUpdateEntry>,
) -> Result<NavigationPolicyReadModel, NavigationPolicyUpdateError> {
    if expected_revision < 0 {
        return Err(NavigationPolicyUpdateError::InvalidRevision);
    }

    let mut tx = pool.begin().await?;
    let installation_id = repository::installation_id(&mut tx).await?;
    let current_revision = repository::lock_navigation_policy(&mut tx, installation_id).await?;
    let current_rows = repository::load_navigation_policy_entries(&mut tx, installation_id).await?;
    let current = navigation_policy_model(installation_id, current_revision, current_rows)
        .map_err(|()| NavigationPolicyUpdateError::Integrity)?;
    let requested = validate_navigation_policy_request(&current, entries)?;

    let unchanged = current.entries.iter().all(|entry| {
        requested
            .get(&entry.contribution_id)
            .is_some_and(|requested| {
                requested.visible == entry.visible && requested.order == entry.order
            })
    });
    if unchanged {
        tx.commit().await?;
        return Ok(current);
    }
    if expected_revision != current_revision {
        return Err(NavigationPolicyUpdateError::RevisionConflict {
            presented: expected_revision,
            current: current_revision,
        });
    }

    let mut changes = Vec::new();
    for before in &current.entries {
        let after = requested
            .get(&before.contribution_id)
            .expect("validated policy is a complete collection");
        if before.visible == after.visible && before.order == after.order {
            continue;
        }
        repository::update_navigation_policy_entry(
            &mut tx,
            installation_id,
            &before.contribution_id,
            after.visible,
            after.order,
        )
        .await?;
        changes.push(json!({
            "contribution_id": before.contribution_id,
            "before": {
                "group": before.group,
                "reorder_band": before.reorder_band,
                "visible": before.visible,
                "order": before.order,
            },
            "after": {
                "group": after.group,
                "reorder_band": after.reorder_band,
                "visible": after.visible,
                "order": after.order,
            },
        }));
    }
    let next_revision = repository::increment_navigation_policy_revision(
        &mut tx,
        installation_id,
        current_revision,
    )
    .await?;
    let updated_rows = repository::load_navigation_policy_entries(&mut tx, installation_id).await?;
    let updated = navigation_policy_model(installation_id, next_revision, updated_rows)
        .map_err(|()| NavigationPolicyUpdateError::Integrity)?;
    let payload = json!({
        "schema_version": 1,
        "installation_id": installation_id,
        "before_revision": current_revision,
        "after_revision": next_revision,
        "changes": changes,
        "success": true,
    });
    repository::insert_account_audit_event(
        &mut tx,
        installation_id,
        "navigation_policy.updated",
        actor_account_id,
        correlation_id,
        &payload,
    )
    .await?;
    tx.commit().await?;
    Ok(updated)
}

fn navigation_policy_model(
    installation_id: Uuid,
    revision: i64,
    rows: Vec<NavigationPolicyEntryRow>,
) -> Result<NavigationPolicyReadModel, ()> {
    if revision < 0 || rows.len() != 6 {
        return Err(());
    }
    let mut seen = BTreeSet::new();
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        if !seen.insert(row.contribution_id.clone()) {
            return Err(());
        }
        let required_capabilities_any_of = row
            .required_capabilities_any_of
            .as_array()
            .ok_or(())?
            .iter()
            .map(|value| value.as_str().filter(|value| !value.is_empty()).ok_or(()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>();
        if required_capabilities_any_of.is_empty() {
            return Err(());
        }
        entries.push(NavigationPolicyEntry {
            contribution_id: row.contribution_id,
            definition_id: row.definition_id,
            destination: row.destination,
            label: row.label,
            group: row.group_name,
            reorder_band: row.reorder_band,
            source_order_hint: row.source_order_hint,
            default_policy_order: row.default_policy_order,
            required_capabilities_any_of,
            visible: row.visible,
            order: row.policy_order,
        });
    }
    validate_dense_policy_orders(&entries).map_err(|_| ())?;
    Ok(NavigationPolicyReadModel {
        installation_id,
        revision,
        entries,
    })
}

fn validate_navigation_policy_request(
    current: &NavigationPolicyReadModel,
    entries: Vec<NavigationPolicyUpdateEntry>,
) -> Result<BTreeMap<String, NavigationPolicyUpdateEntry>, NavigationPolicyUpdateError> {
    const CORE_ITEMS: [&str; 5] = [
        "home",
        "organization",
        "operations",
        "administration",
        "module_management",
    ];
    let current_by_id = current
        .entries
        .iter()
        .map(|entry| (entry.contribution_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    let mut requested = BTreeMap::new();
    for entry in entries {
        if CORE_ITEMS.contains(&entry.contribution_id.as_str()) {
            return Err(NavigationPolicyUpdateError::CoreItemImmutable {
                contribution_id: entry.contribution_id,
            });
        }
        let Some(existing) = current_by_id.get(entry.contribution_id.as_str()) else {
            return Err(NavigationPolicyUpdateError::UnknownContribution {
                contribution_id: entry.contribution_id,
            });
        };
        if requested.contains_key(&entry.contribution_id) {
            return Err(NavigationPolicyUpdateError::DuplicateContribution {
                contribution_id: entry.contribution_id,
            });
        }
        if entry.group != existing.group {
            return Err(NavigationPolicyUpdateError::GroupChangeForbidden {
                contribution_id: entry.contribution_id,
            });
        }
        if entry.reorder_band != existing.reorder_band {
            return Err(NavigationPolicyUpdateError::BandChangeForbidden {
                contribution_id: entry.contribution_id,
            });
        }
        requested.insert(entry.contribution_id.clone(), entry);
    }
    for contribution_id in current_by_id.keys() {
        if !requested.contains_key(*contribution_id) {
            return Err(NavigationPolicyUpdateError::MissingContribution {
                contribution_id: (*contribution_id).to_string(),
            });
        }
    }
    validate_dense_requested_orders(requested.values())?;
    Ok(requested)
}

fn validate_dense_policy_orders(
    entries: &[NavigationPolicyEntry],
) -> Result<(), NavigationPolicyUpdateError> {
    let mut by_band = BTreeMap::<&str, Vec<i32>>::new();
    for entry in entries {
        by_band
            .entry(&entry.reorder_band)
            .or_default()
            .push(entry.order);
    }
    validate_dense_orders(by_band)
}

fn validate_dense_requested_orders<'a>(
    entries: impl Iterator<Item = &'a NavigationPolicyUpdateEntry>,
) -> Result<(), NavigationPolicyUpdateError> {
    let mut by_band = BTreeMap::<&str, Vec<i32>>::new();
    for entry in entries {
        by_band
            .entry(&entry.reorder_band)
            .or_default()
            .push(entry.order);
    }
    validate_dense_orders(by_band)
}

fn validate_dense_orders(
    by_band: BTreeMap<&str, Vec<i32>>,
) -> Result<(), NavigationPolicyUpdateError> {
    for (reorder_band, mut orders) in by_band {
        orders.sort_unstable();
        if orders
            .iter()
            .copied()
            .ne((0..orders.len()).map(|order| order as i32))
        {
            return Err(NavigationPolicyUpdateError::InvalidBandOrder {
                reorder_band: reorder_band.to_string(),
            });
        }
    }
    Ok(())
}

pub(crate) async fn synchronize_catalog(
    pool: &PgPool,
) -> Result<CatalogSyncOutcome, CatalogSyncError> {
    synchronize_catalog_inputs(pool, canonical_inputs(), None).await
}

pub(crate) async fn synchronize_catalog_inputs(
    pool: &PgPool,
    inputs: Vec<CatalogInput>,
    failure_point: Option<SyncFailurePoint>,
) -> Result<CatalogSyncOutcome, CatalogSyncError> {
    let correlation_id = Uuid::new_v4();
    let attempted_digests = inputs
        .iter()
        .map(|input| source_digest(&input.bytes))
        .collect::<Vec<_>>();
    let prepared = match prepare_catalog(&inputs) {
        Ok(prepared) => prepared,
        Err(error) => {
            let sync_error = CatalogSyncError::Contract(error);
            record_rejection(
                pool,
                correlation_id,
                sync_error.stable_code(),
                &attempted_digests,
            )
            .await;
            return Err(sync_error);
        }
    };

    let result = synchronize_prepared(pool, prepared, correlation_id, failure_point).await;
    if let Err(error) = &result {
        record_rejection(
            pool,
            correlation_id,
            error.stable_code(),
            &attempted_digests,
        )
        .await;
    }
    result
}

async fn record_rejection(
    pool: &PgPool,
    correlation_id: Uuid,
    stable_code: &str,
    attempted_digests: &[String],
) {
    if let Err(error) =
        repository::record_rejected_sync(pool, correlation_id, stable_code, attempted_digests).await
    {
        tracing::warn!(
            error = ?error,
            stable_code,
            "could not persist rejected module catalog synchronization audit event"
        );
    }
}

async fn synchronize_prepared(
    pool: &PgPool,
    prepared: Vec<PreparedCatalogSource>,
    correlation_id: Uuid,
    failure_point: Option<SyncFailurePoint>,
) -> Result<CatalogSyncOutcome, CatalogSyncError> {
    let mut tx = pool.begin().await?;
    let outcome =
        synchronize_in_transaction(&mut tx, prepared, correlation_id, failure_point).await;

    match outcome {
        Ok(outcome) => {
            tx.commit().await?;
            Ok(outcome)
        }
        Err(error) => {
            tx.rollback().await?;
            Err(error)
        }
    }
}

async fn synchronize_in_transaction(
    tx: &mut Transaction<'_, Postgres>,
    prepared: Vec<PreparedCatalogSource>,
    correlation_id: Uuid,
    failure_point: Option<SyncFailurePoint>,
) -> Result<CatalogSyncOutcome, CatalogSyncError> {
    repository::acquire_catalog_sync_lock(tx).await?;
    let installation_id = repository::installation_id(tx).await?;
    repository::ensure_core_runtime_observation(tx, installation_id, env!("CARGO_PKG_VERSION"))
        .await?;

    let mut working = Vec::with_capacity(prepared.len());
    for source in prepared {
        let stored_display_name =
            repository::ensure_reservation(tx, &source.definition_id, &source.display_name).await?;
        if stored_display_name != source.display_name {
            return Err(CatalogSyncError::StoredReservationMismatch {
                definition_id: source.definition_id,
            });
        }
        let current = repository::load_current_catalog_entry(tx, &source.definition_id).await?;
        let mut entry = WorkingSource {
            prepared: source,
            source_id: None,
            projection_id: None,
            before_digest: None,
            before_finding_codes: Vec::new(),
            changed: true,
        };

        if let Some(current) = current {
            if source_digest(&current.source_bytes) != current.source_digest {
                return Err(CatalogSyncError::StoredSourceDigestMismatch {
                    definition_id: entry.prepared.definition_id.clone(),
                });
            }
            let previous_input = frozen_source_input(
                &entry.prepared.definition_id,
                current.source_bytes.clone(),
                current.source_digest.clone(),
            )
            .ok_or(CatalogSyncError::StoredCatalogShapeMismatch)?;
            let previous = prepare_source(&previous_input)?;
            validate_current_projection(tx, &current, &previous, installation_id).await?;
            entry.before_finding_codes = previous
                .findings
                .iter()
                .map(|finding| finding.code.clone())
                .collect();
            entry.before_digest = Some(current.source_digest.clone());
            if current.source_digest == entry.prepared.source_digest {
                if current.source_bytes != entry.prepared.source_bytes {
                    return Err(CatalogSyncError::StoredSourceDigestMismatch {
                        definition_id: entry.prepared.definition_id.clone(),
                    });
                }
                entry.source_id = Some(current.source_id);
                entry.projection_id = Some(current.projection_id);
                entry.changed = false;
            } else {
                ensure_compatible_source_change(&previous.descriptor, &entry.prepared.descriptor)?;
            }
        }
        working.push(entry);
    }

    for source in working.iter_mut().filter(|source| source.changed) {
        let stored = repository::ensure_source(
            tx,
            &source.prepared.definition_id,
            &source.prepared.source_digest,
            &source.prepared.source_bytes,
        )
        .await?;
        if stored.definition_id != source.prepared.definition_id
            || stored.schema_version != 1
            || stored.source_digest != source.prepared.source_digest
            || stored.source_bytes != source.prepared.source_bytes
            || stored.content_type != "application/json"
        {
            return Err(CatalogSyncError::StoredSourceDigestMismatch {
                definition_id: source.prepared.definition_id.clone(),
            });
        }
        source.source_id = Some(stored.id);
    }
    fail_if_requested(failure_point, SyncFailurePoint::Sources)?;

    for source in working.iter_mut().filter(|source| source.changed) {
        let source_id = source
            .source_id
            .expect("changed sources have an inserted row");
        let projection = source.prepared.normalized_projection(installation_id)?;
        let projection_id = if let Some(stored) =
            repository::load_projection_by_source(tx, source_id).await?
        {
            if stored.source_id != source_id
                || stored.installation_id != installation_id
                || stored.provider_eligible
                || stored.supervisor_materializable
                || stored.normalized_projection != projection
            {
                return Err(CatalogSyncError::StoredProjectionMismatch {
                    definition_id: source.prepared.definition_id.clone(),
                });
            }
            let stored_findings = repository::load_findings(tx, stored.id).await?;
            if stored_findings != source.prepared.findings {
                return Err(CatalogSyncError::StoredFindingsMismatch {
                    definition_id: source.prepared.definition_id.clone(),
                });
            }
            stored.id
        } else {
            let projection_id =
                repository::insert_projection(tx, source_id, installation_id, &projection).await?;
            repository::insert_findings(tx, projection_id, &source.prepared.findings).await?;
            projection_id
        };
        repository::set_current_catalog_entry(
            tx,
            &source.prepared.definition_id,
            source_id,
            projection_id,
        )
        .await?;
        source.projection_id = Some(projection_id);
    }
    fail_if_requested(failure_point, SyncFailurePoint::Projections)?;

    for (key, description) in MODULE_CAPABILITIES {
        let capability = repository::ensure_module_capability(tx, key, description).await?;
        if capability.key != key
            || capability.description != description
            || capability.scope_mode != "installation_global"
        {
            return Err(CatalogSyncError::CapabilityMetadataMismatch {
                key: key.to_string(),
            });
        }
    }
    let admin_all = repository::load_capability(tx, "admin:all")
        .await?
        .ok_or_else(|| CatalogSyncError::CapabilityMetadataMismatch {
            key: "admin:all".to_string(),
        })?;
    if admin_all.description != "Full administration access"
        || admin_all.scope_mode != "installation_global"
    {
        return Err(CatalogSyncError::CapabilityMetadataMismatch {
            key: "admin:all".to_string(),
        });
    }
    repository::ensure_core_capability_provenance(tx).await?;

    for source in &working {
        let source_id = source.source_id.expect("every prepared source has a row");
        for declaration in &source.prepared.descriptor.security_capabilities {
            let key = declaration.id.as_str();
            let capability = repository::load_capability(tx, key).await?.ok_or_else(|| {
                CatalogSyncError::CapabilityNotRegistered {
                    definition_id: source.prepared.definition_id.clone(),
                    key: key.to_string(),
                }
            })?;
            if capability.description != declaration.description {
                return Err(CatalogSyncError::CapabilityDescriptionMismatch {
                    definition_id: source.prepared.definition_id.clone(),
                    key: key.to_string(),
                });
            }
            if capability.scope_mode != "scope_aware" {
                return Err(CatalogSyncError::CapabilityMetadataMismatch {
                    key: key.to_string(),
                });
            }
            repository::upsert_transition_capability_provenance(
                tx,
                capability.id,
                &source.prepared.definition_id,
                source_id,
            )
            .await?;
        }
    }
    fail_if_requested(failure_point, SyncFailurePoint::Capabilities)?;

    repository::ensure_navigation_policy(tx, installation_id).await?;
    for source in &working {
        let Some(defaults) = source.prepared.navigation_defaults.as_ref() else {
            continue;
        };
        let [navigation] = source.prepared.descriptor.navigation.as_slice() else {
            unreachable!("catalog preparation enforces one navigation contribution")
        };
        let required_capabilities_any_of = serde_json::to_value(
            navigation
                .required_capabilities_any_of
                .iter()
                .map(|capability| capability.as_str())
                .collect::<Vec<_>>(),
        )
        .map_err(CatalogContractError::from)?;
        let stored = repository::ensure_navigation_contribution(
            tx,
            NavigationRecord {
                contribution_id: navigation.id.as_str(),
                definition_id: &source.prepared.definition_id,
                descriptor_source_id: source.source_id.expect("source row"),
                destination: navigation.destination.as_str(),
                label: &navigation.label,
                group_name: &navigation.group,
                reorder_band: &defaults.reorder_band,
                source_order_hint: navigation.order_hint,
                default_policy_order: defaults.policy_order,
                required_capabilities_any_of: required_capabilities_any_of.clone(),
            },
        )
        .await?;
        if stored.contribution_id != navigation.id.as_str()
            || stored.definition_id != source.prepared.definition_id
            || stored.destination != navigation.destination.as_str()
            || stored.label != navigation.label
            || stored.group_name != navigation.group
            || stored.reorder_band != defaults.reorder_band
            || stored.source_order_hint != navigation.order_hint
            || stored.default_policy_order != defaults.policy_order
            || stored.required_capabilities_any_of != required_capabilities_any_of
        {
            return Err(CatalogSyncError::StoredNavigationMismatch {
                contribution_id: navigation.id.as_str().to_string(),
            });
        }
        repository::update_navigation_descriptor_source(
            tx,
            navigation.id.as_str(),
            source.source_id.expect("source row"),
        )
        .await?;
        repository::ensure_navigation_policy_entry(
            tx,
            installation_id,
            navigation.id.as_str(),
            defaults.policy_order,
        )
        .await?;
    }
    fail_if_requested(failure_point, SyncFailurePoint::Navigation)?;

    let mut stored_definition_ids = repository::load_current_definition_ids(tx).await?;
    let mut expected_definition_ids = frozen_definition_ids();
    stored_definition_ids.sort();
    expected_definition_ids.sort();
    if stored_definition_ids != expected_definition_ids {
        return Err(CatalogSyncError::StoredCatalogShapeMismatch);
    }

    let changed = working
        .iter()
        .filter(|source| source.changed)
        .collect::<Vec<_>>();
    let changed_definition_ids = changed
        .iter()
        .map(|source| source.prepared.definition_id.clone())
        .collect::<Vec<_>>();
    if !changed.is_empty() {
        let source_digest_changes = changed
            .iter()
            .map(|source| {
                json!({
                    "definition_id": source.prepared.definition_id,
                    "before": source.before_digest,
                    "after": source.prepared.source_digest,
                })
            })
            .collect::<Vec<_>>();
        let finding_summary_changes = changed
            .iter()
            .map(|source| {
                json!({
                    "definition_id": source.prepared.definition_id,
                    "before_codes": source.before_finding_codes,
                    "after_codes": source.prepared.findings.iter()
                        .map(|finding| finding.code.as_str())
                        .collect::<Vec<_>>(),
                })
            })
            .collect::<Vec<_>>();
        let payload: Value = json!({
            "schema_version": 1,
            "installation_id": installation_id,
            "affected_reserved_definition_ids": changed_definition_ids,
            "source_digest_changes": source_digest_changes,
            "finding_summary_changes": finding_summary_changes,
            "success": true,
        });
        repository::insert_audit_event(
            tx,
            Some(installation_id),
            "module_catalog.synchronized",
            correlation_id,
            &payload,
        )
        .await?;
    }

    Ok(CatalogSyncOutcome {
        installation_id,
        changed_definition_ids,
    })
}

fn fail_if_requested(
    requested: Option<SyncFailurePoint>,
    current: SyncFailurePoint,
) -> Result<(), CatalogSyncError> {
    if requested == Some(current) {
        Err(CatalogSyncError::InjectedFailure(current))
    } else {
        Ok(())
    }
}

async fn validate_current_projection(
    tx: &mut Transaction<'_, Postgres>,
    current: &repository::CurrentCatalogEntry,
    prepared: &PreparedCatalogSource,
    installation_id: Uuid,
) -> Result<(), CatalogSyncError> {
    let expected_projection = prepared.normalized_projection(installation_id)?;
    if current.schema_version != 1
        || current.content_type != "application/json"
        || current.projection_source_id != current.source_id
        || current.projection_installation_id != installation_id
        || current.provider_eligible
        || current.supervisor_materializable
        || current.normalized_projection != expected_projection
    {
        return Err(CatalogSyncError::StoredProjectionMismatch {
            definition_id: prepared.definition_id.clone(),
        });
    }
    let stored_findings = repository::load_findings(tx, current.projection_id).await?;
    if stored_findings != prepared.findings {
        return Err(CatalogSyncError::StoredFindingsMismatch {
            definition_id: prepared.definition_id.clone(),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};
    use sqlx::{Row, postgres::PgPoolOptions};

    use super::*;

    const DISPOSABLE_DATABASE_NAME_TOKENS: &[&str] = &[
        "test", "tests", "testing", "upgrade", "clone", "rollback", "sprint6a",
    ];

    fn is_disposable_database_name(database_name: &str) -> bool {
        let tokens = database_name
            .split(|character: char| !character.is_alphanumeric())
            .filter(|token| !token.is_empty())
            .map(str::to_ascii_lowercase)
            .collect::<Vec<_>>();

        tokens.iter().any(|token| {
            DISPOSABLE_DATABASE_NAME_TOKENS
                .iter()
                .any(|marker| token == marker)
        }) || tokens
            .windows(2)
            .any(|pair| pair[0] == "sprint" && pair[1] == "6a")
    }

    #[test]
    fn historical_migrations_remain_byte_identical() {
        let baseline = include_bytes!("../../migrations/001_baseline.sql");
        let dashboard_capacity =
            include_bytes!("../../migrations/002_dashboard_placement_capacity.sql");
        assert_eq!(
            format!("{:x}", Sha256::digest(baseline)),
            "a61f5192ad8e14bdcbbd26203301030fd57b647a237218c1e5443936944e9ca0"
        );
        assert_eq!(
            format!("{:x}", Sha256::digest(dashboard_capacity)),
            "c26a100e7fcd7aba4a74622c03f6c8e809219022595206da3ba7ddc86313550e"
        );
    }

    #[test]
    fn policy_collection_validation_rejects_every_immutable_shape_change() {
        let current = example_policy();

        let mut duplicate = update_entries(&current);
        duplicate.push(duplicate[0].clone());
        assert_error_code(
            validate_navigation_policy_request(&current, duplicate),
            "navigation_policy_duplicate_contribution",
        );

        let mut core_item = update_entries(&current);
        core_item.push(NavigationPolicyUpdateEntry {
            contribution_id: "module_management".to_string(),
            group: "Admin".to_string(),
            reorder_band: "core_anchor".to_string(),
            visible: false,
            order: 0,
        });
        assert_error_code(
            validate_navigation_policy_request(&current, core_item),
            "navigation_policy_core_item_immutable",
        );

        let mut unknown = update_entries(&current);
        unknown.push(NavigationPolicyUpdateEntry {
            contribution_id: "tessara.unknown.navigation".to_string(),
            group: "Main".to_string(),
            reorder_band: "main_after_operations".to_string(),
            visible: true,
            order: 2,
        });
        assert_error_code(
            validate_navigation_policy_request(&current, unknown),
            "navigation_policy_unknown_contribution",
        );

        let mut missing = update_entries(&current);
        missing.pop();
        assert_error_code(
            validate_navigation_policy_request(&current, missing),
            "navigation_policy_missing_contribution",
        );

        let mut group_change = update_entries(&current);
        group_change[0].group = "Admin".to_string();
        assert_error_code(
            validate_navigation_policy_request(&current, group_change),
            "navigation_policy_group_change_forbidden",
        );

        let mut band_change = update_entries(&current);
        band_change[0].reorder_band = "main_after_operations".to_string();
        assert_error_code(
            validate_navigation_policy_request(&current, band_change),
            "navigation_policy_band_change_forbidden",
        );

        let mut invalid_order = update_entries(&current);
        invalid_order[0].order = 1;
        assert_error_code(
            validate_navigation_policy_request(&current, invalid_order),
            "navigation_policy_order_invalid",
        );
    }

    #[test]
    fn policy_collection_validation_accepts_dense_within_band_reordering() {
        let current = example_policy();
        let mut requested = update_entries(&current);
        requested[0].order = 1;
        requested[1].order = 0;
        requested[4].visible = false;
        let validated = validate_navigation_policy_request(&current, requested)
            .expect("same-band dense reorder is valid");
        assert_eq!(validated["tessara.forms.navigation"].order, 1);
        assert_eq!(validated["tessara.workflows.navigation"].order, 0);
        assert!(!validated["tessara.dashboards.navigation"].visible);
    }

    #[test]
    fn catalog_sync_database_guard_requires_token_bounded_disposable_names() {
        for accepted in [
            "tessara_test",
            "tessara-tests",
            "tessara_testing",
            "tessara_upgrade",
            "tessara_clone",
            "tessara_rollback",
            "tessara_sprint6a",
            "tessara_sprint_6a",
        ] {
            assert!(
                is_disposable_database_name(accepted),
                "'{accepted}' should be recognized as disposable"
            );
        }

        for rejected in [
            "tessara",
            "latest",
            "contest",
            "testingground",
            "upgraded",
            "cloned",
            "rollbacks",
            "sprint6alpha",
        ] {
            assert!(
                !is_disposable_database_name(rejected),
                "'{rejected}' must not pass by substring"
            );
        }
    }

    #[tokio::test]
    async fn catalog_sync_is_repeatable_concurrent_and_rolls_back_injected_failure() {
        let database_url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL is required; database integration tests must never skip");
        assert!(
            !database_url.trim().is_empty(),
            "TEST_DATABASE_URL is required and must not be empty"
        );

        let preflight_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await
            .expect("TEST_DATABASE_URL must be reachable for the read-only safety preflight");
        let database_name: String = sqlx::query_scalar("SELECT current_database()")
            .fetch_one(&preflight_pool)
            .await
            .expect("the preflight must read the current database name");
        assert!(
            is_disposable_database_name(&database_name),
            "TEST_DATABASE_URL must name a token-bounded disposable database ({}); got '{database_name}'",
            DISPOSABLE_DATABASE_NAME_TOKENS.join(", ")
        );
        preflight_pool.close().await;

        let config = crate::config::Config {
            database_url: database_url.clone(),
            bind_addr: "127.0.0.1:0".to_string(),
            dev_admin_email: "module-persistence-test@tessara.local".to_string(),
            dev_admin_password: "module-persistence-test-password".to_string(),
            auth_cookie_name: "tessara_module_persistence_test".to_string(),
            auth_cookie_secure: false,
            auth_session_ttl_hours: 1,
        };
        let pool = crate::db::connect_and_prepare(&config)
            .await
            .expect("disposable database prepares");

        let before = load_module_inventory(&pool).await.expect("inventory reads");
        assert_eq!(before.transitions.len(), 7);
        let canonical_forms_digest = before
            .transitions
            .iter()
            .find(|transition| transition.definition_id == "tessara.forms")
            .map(|transition| transition.source_digest.clone())
            .expect("canonical Forms transition");
        let before_identity = inventory_identity(&before);
        let before_policy = load_navigation_policy(&pool).await.expect("policy reads");
        let capability_ids_before = capability_ids(&pool).await;
        let role_capabilities_before = role_capabilities(&pool).await;
        let successful_audits_before = audit_count(&pool, "module_catalog.synchronized").await;

        let repeated = synchronize_catalog(&pool)
            .await
            .expect("repeat sync succeeds");
        assert_eq!(repeated.installation_id, before.installation_id);
        assert!(repeated.changed_definition_ids.is_empty());
        assert_eq!(
            audit_count(&pool, "module_catalog.synchronized").await,
            successful_audits_before
        );

        let (left, right) = tokio::join!(synchronize_catalog(&pool), synchronize_catalog(&pool));
        assert!(
            left.expect("left concurrent sync")
                .changed_definition_ids
                .is_empty(),
            "concurrent no-op sync must not report changes"
        );
        assert!(
            right
                .expect("right concurrent sync")
                .changed_definition_ids
                .is_empty(),
            "concurrent no-op sync must not report changes"
        );
        assert_eq!(
            audit_count(&pool, "module_catalog.synchronized").await,
            successful_audits_before
        );

        let source_count_before: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM transition_descriptor_sources")
                .fetch_one(&pool)
                .await
                .expect("source count");
        let rejected_audits_before = audit_count(&pool, "module_catalog.sync_rejected").await;
        let mut changed_inputs = canonical_inputs();
        let forms = &mut changed_inputs[0];
        forms.bytes.insert(forms.bytes.len() - 1, b' ');
        forms.expected_digest = source_digest(&forms.bytes);
        let changed_forms_digest = forms.expected_digest.clone();
        let control_plane_before_injected_failures = control_plane_snapshot(&pool).await;
        for failure_point in [
            SyncFailurePoint::Sources,
            SyncFailurePoint::Projections,
            SyncFailurePoint::Capabilities,
            SyncFailurePoint::Navigation,
        ] {
            let failure =
                synchronize_catalog_inputs(&pool, changed_inputs.clone(), Some(failure_point))
                    .await
                    .expect_err("injected synchronization failure is returned");
            assert_eq!(
                failure.stable_code(),
                "module_catalog_sync_injected_failure"
            );
            assert_eq!(
                control_plane_snapshot(&pool).await,
                control_plane_before_injected_failures,
                "the complete module control plane must roll back at {failure_point:?}; the separately committed rejection audit is intentionally excluded from this state snapshot"
            );
        }
        let source_count_after: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM transition_descriptor_sources")
                .fetch_one(&pool)
                .await
                .expect("source count after rollback");
        assert_eq!(source_count_after, source_count_before);
        assert_eq!(
            audit_count(&pool, "module_catalog.synchronized").await,
            successful_audits_before
        );
        assert_eq!(
            audit_count(&pool, "module_catalog.sync_rejected").await,
            rejected_audits_before + 4
        );

        let corrupted_capability_rows =
            sqlx::query("UPDATE capabilities SET description = $2 WHERE key = $1")
                .bind("forms:read")
                .bind("Conflicting stored Forms read description")
                .execute(&pool)
                .await
                .expect("the capability-description conflict fixture is installed")
                .rows_affected();
        assert_eq!(corrupted_capability_rows, 1);
        let control_plane_before_description_mismatch = control_plane_snapshot(&pool).await;
        let expected_attempted_digests = changed_inputs
            .iter()
            .map(|input| source_digest(&input.bytes))
            .collect::<Vec<_>>();
        let description_mismatch_result =
            synchronize_catalog_inputs(&pool, changed_inputs.clone(), None).await;
        let control_plane_after_description_mismatch = control_plane_snapshot(&pool).await;
        let rejected_audits_after_description_mismatch =
            audit_count(&pool, "module_catalog.sync_rejected").await;
        let restored_capability_rows =
            sqlx::query("UPDATE capabilities SET description = $2 WHERE key = $1")
                .bind("forms:read")
                .bind("Browse top-level form records")
                .execute(&pool)
                .await
                .expect("the capability-description conflict fixture is removed")
                .rows_affected();
        assert_eq!(restored_capability_rows, 1);
        let description_mismatch_audit: Value = sqlx::query_scalar(
            r#"
            SELECT payload
            FROM core_control_plane_audit_events
            WHERE event_type = 'module_catalog.sync_rejected'
              AND payload ->> 'rejection_code' = 'transition_capability_description_mismatch'
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("description mismatch rejection audit");
        let description_mismatch = description_mismatch_result
            .expect_err("a transition declaration that conflicts with Core metadata is rejected");
        assert_eq!(
            description_mismatch.stable_code(),
            "transition_capability_description_mismatch"
        );
        assert_eq!(
            control_plane_after_description_mismatch, control_plane_before_description_mismatch,
            "description mismatch rejection must roll back the new source, projection, findings, current pointer, provenance, and navigation writes"
        );
        assert_eq!(
            rejected_audits_after_description_mismatch,
            rejected_audits_before + 5
        );
        assert_eq!(
            description_mismatch_audit,
            json!({
                "schema_version": 1,
                "attempted_source_digests": expected_attempted_digests,
                "rejection_code": "transition_capability_description_mismatch",
                "success": false,
            })
        );

        let changed = synchronize_catalog_inputs(&pool, changed_inputs, None)
            .await
            .expect("compatible exact-byte source change succeeds");
        assert_eq!(changed.changed_definition_ids, ["tessara.forms"]);
        let (
            success_installation_id,
            success_event_type,
            success_actor_kind,
            success_actor_account_id,
            success_correlation_id,
            success_payload,
        ): (Option<Uuid>, String, String, Option<Uuid>, Uuid, Value) = sqlx::query_as(
            r#"
            SELECT installation_id, event_type, actor_kind, actor_account_id, correlation_id, payload
            FROM core_control_plane_audit_events
            WHERE event_type = 'module_catalog.synchronized'
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .fetch_one(&pool)
        .await
        .expect("successful catalog synchronization audit event");
        assert_eq!(success_installation_id, Some(before.installation_id));
        assert_eq!(success_event_type, "module_catalog.synchronized");
        assert_eq!(success_actor_kind, "system");
        assert_eq!(success_actor_account_id, None);
        assert_ne!(success_correlation_id, Uuid::nil());
        assert_eq!(
            success_payload,
            json!({
                "schema_version": 1,
                "installation_id": before.installation_id,
                "affected_reserved_definition_ids": ["tessara.forms"],
                "source_digest_changes": [{
                    "definition_id": "tessara.forms",
                    "before": canonical_forms_digest,
                    "after": changed_forms_digest,
                }],
                "finding_summary_changes": [{
                    "definition_id": "tessara.forms",
                    "before_codes": [],
                    "after_codes": [],
                }],
                "success": true,
            })
        );
        let restored = synchronize_catalog(&pool)
            .await
            .expect("canonical source can be restored from immutable history");
        assert_eq!(restored.changed_definition_ids, ["tessara.forms"]);
        assert_eq!(
            role_capabilities(&pool).await,
            role_capabilities_before,
            "compatible synchronization and canonical restoration must preserve every exact role-capability mapping"
        );
        assert_eq!(
            audit_count(&pool, "module_catalog.synchronized").await,
            successful_audits_before + 2
        );

        let mut malformed_inputs = canonical_inputs();
        malformed_inputs[0].bytes = b"{\n".to_vec();
        malformed_inputs[0].expected_digest = source_digest(&malformed_inputs[0].bytes);
        let malformed = synchronize_catalog_inputs(&pool, malformed_inputs, None)
            .await
            .expect_err("malformed source is rejected before catalog writes");
        assert_eq!(malformed.stable_code(), "transition_source_decode_failed");
        assert_eq!(
            audit_count(&pool, "module_catalog.sync_rejected").await,
            rejected_audits_before + 6
        );

        let actor_account_id: Uuid = sqlx::query_scalar("SELECT id FROM accounts WHERE email = $1")
            .bind(&config.dev_admin_email)
            .fetch_one(&pool)
            .await
            .expect("seeded test administrator");
        let policy_before_authorization_denial = load_navigation_policy(&pool)
            .await
            .expect("policy reads before authorization denial");
        let authorization_denials_before =
            audit_count(&pool, "navigation_policy.update_denied").await;
        let authorization_correlation_id = Uuid::new_v4();
        record_navigation_policy_authorization_denial(
            &pool,
            actor_account_id,
            authorization_correlation_id,
        )
        .await
        .expect("authenticated authorization denial is audited");
        let authorization_event = sqlx::query(
            r#"
            SELECT event_type, actor_kind, actor_account_id, correlation_id, payload
            FROM core_control_plane_audit_events
            WHERE correlation_id = $1
            "#,
        )
        .bind(authorization_correlation_id)
        .fetch_one(&pool)
        .await
        .expect("authorization denial audit event");
        assert_eq!(
            authorization_event
                .try_get::<String, _>("event_type")
                .expect("event type"),
            "navigation_policy.update_denied"
        );
        assert_eq!(
            authorization_event
                .try_get::<String, _>("actor_kind")
                .expect("actor kind"),
            "account"
        );
        assert_eq!(
            authorization_event
                .try_get::<Uuid, _>("actor_account_id")
                .expect("actor account"),
            actor_account_id
        );
        assert_eq!(
            authorization_event
                .try_get::<Uuid, _>("correlation_id")
                .expect("correlation ID"),
            authorization_correlation_id
        );
        assert_eq!(
            authorization_event
                .try_get::<Value, _>("payload")
                .expect("audit payload"),
            json!({
                "schema_version": 1,
                "action": "navigation_policy.update",
                "presented_revision": null,
                "denial_code": "modules_manage_navigation_global_required",
                "success": false,
            })
        );
        assert_eq!(
            load_navigation_policy(&pool)
                .await
                .expect("policy reads after authorization denial"),
            policy_before_authorization_denial
        );
        assert_eq!(
            audit_count(&pool, "navigation_policy.update_denied").await,
            authorization_denials_before + 1
        );
        let policy_audits_before = audit_count(&pool, "navigation_policy.updated").await;
        let denied_policy_audits_before =
            audit_count(&pool, "navigation_policy.update_denied").await;
        let mut changed_policy_request = update_entries(&before_policy);
        changed_policy_request[0].order = 1;
        changed_policy_request[1].order = 0;
        changed_policy_request[4].visible = false;
        let original_policy_request = update_entries(&before_policy);
        let policy_exercise: Result<_, String> = async {
            let updated = update_navigation_policy(
                &pool,
                actor_account_id,
                Uuid::new_v4(),
                before_policy.revision,
                changed_policy_request.clone(),
            )
            .await
            .map_err(|error| error.to_string())?;
            let retried = update_navigation_policy(
                &pool,
                actor_account_id,
                Uuid::new_v4(),
                before_policy.revision,
                changed_policy_request,
            )
            .await
            .map_err(|error| error.to_string())?;
            let conflict = update_navigation_policy(
                &pool,
                actor_account_id,
                Uuid::new_v4(),
                before_policy.revision,
                original_policy_request.clone(),
            )
            .await
            .err()
            .ok_or_else(|| "stale conflicting update unexpectedly succeeded".to_string())?;
            Ok((updated, retried, conflict.stable_code()))
        }
        .await;

        let policy_before_restore = load_navigation_policy(&pool)
            .await
            .expect("policy reads before cleanup");
        if !same_policy_values(&policy_before_restore, &before_policy) {
            update_navigation_policy(
                &pool,
                actor_account_id,
                Uuid::new_v4(),
                policy_before_restore.revision,
                original_policy_request,
            )
            .await
            .expect("policy cleanup restores the original values");
        }
        let (updated, retried, conflict_code) =
            policy_exercise.expect("navigation policy exercise succeeds");
        assert_eq!(updated.revision, before_policy.revision + 1);
        assert_eq!(retried, updated);
        assert_eq!(conflict_code, "navigation_policy_revision_conflict");
        assert_eq!(
            audit_count(&pool, "navigation_policy.updated").await,
            policy_audits_before + 2
        );
        assert_eq!(
            audit_count(&pool, "navigation_policy.update_denied").await,
            denied_policy_audits_before + 1
        );

        let after = load_module_inventory(&pool)
            .await
            .expect("inventory remains readable");
        assert_eq!(inventory_identity(&after), before_identity);
        assert!(same_policy_values(
            &load_navigation_policy(&pool)
                .await
                .expect("policy remains readable"),
            &before_policy
        ));
        assert_eq!(capability_ids(&pool).await, capability_ids_before);
        assert_eq!(
            role_capabilities(&pool).await,
            role_capabilities_before,
            "the complete exercise must preserve every exact role-capability mapping"
        );
    }

    fn assert_error_code(
        result: Result<BTreeMap<String, NavigationPolicyUpdateEntry>, NavigationPolicyUpdateError>,
        expected: &str,
    ) {
        let error = result.expect_err("request must be rejected");
        assert_eq!(error.stable_code(), expected);
    }

    fn example_policy() -> NavigationPolicyReadModel {
        let definitions = [
            (
                "tessara.forms.navigation",
                "tessara.forms",
                "Main",
                "main_between_organization_and_operations",
                0,
            ),
            (
                "tessara.workflows.navigation",
                "tessara.workflows",
                "Main",
                "main_between_organization_and_operations",
                1,
            ),
            (
                "tessara.responses.navigation",
                "tessara.responses",
                "Main",
                "main_between_organization_and_operations",
                2,
            ),
            (
                "tessara.components.navigation",
                "tessara.components",
                "Main",
                "main_after_operations",
                0,
            ),
            (
                "tessara.dashboards.navigation",
                "tessara.dashboards",
                "Main",
                "main_after_operations",
                1,
            ),
            (
                "tessara.datasets.navigation",
                "tessara.datasets",
                "Admin",
                "admin_between_administration_and_module_management",
                0,
            ),
        ];
        NavigationPolicyReadModel {
            installation_id: Uuid::nil(),
            revision: 0,
            entries: definitions
                .into_iter()
                .map(
                    |(contribution_id, definition_id, group, reorder_band, order)| {
                        NavigationPolicyEntry {
                            contribution_id: contribution_id.to_string(),
                            definition_id: definition_id.to_string(),
                            destination: format!(
                                "{}.directory",
                                definition_id.replace("tessara.", "")
                            ),
                            label: definition_id.to_string(),
                            group: group.to_string(),
                            reorder_band: reorder_band.to_string(),
                            source_order_hint: order * 10,
                            default_policy_order: order,
                            required_capabilities_any_of: vec!["example:read".to_string()],
                            visible: true,
                            order,
                        }
                    },
                )
                .collect(),
        }
    }

    fn update_entries(policy: &NavigationPolicyReadModel) -> Vec<NavigationPolicyUpdateEntry> {
        policy
            .entries
            .iter()
            .map(|entry| NavigationPolicyUpdateEntry {
                contribution_id: entry.contribution_id.clone(),
                group: entry.group.clone(),
                reorder_band: entry.reorder_band.clone(),
                visible: entry.visible,
                order: entry.order,
            })
            .collect()
    }

    fn same_policy_values(
        left: &NavigationPolicyReadModel,
        right: &NavigationPolicyReadModel,
    ) -> bool {
        left.installation_id == right.installation_id && left.entries == right.entries
    }

    fn inventory_identity(inventory: &ModuleInventoryReadModel) -> Vec<(String, Uuid, Uuid)> {
        inventory
            .transitions
            .iter()
            .map(|entry| {
                (
                    entry.definition_id.clone(),
                    entry.source_id,
                    entry.projection_id,
                )
            })
            .collect()
    }

    async fn audit_count(pool: &PgPool, event_type: &str) -> i64 {
        sqlx::query_scalar(
            "SELECT COUNT(*) FROM core_control_plane_audit_events WHERE event_type = $1",
        )
        .bind(event_type)
        .fetch_one(pool)
        .await
        .expect("audit count")
    }

    async fn capability_ids(pool: &PgPool) -> Vec<(String, Uuid, String, String)> {
        sqlx::query_as("SELECT key, id, description, scope_mode FROM capabilities ORDER BY key")
            .fetch_all(pool)
            .await
            .expect("capability identity snapshot")
    }

    async fn role_capabilities(pool: &PgPool) -> Vec<(Uuid, Uuid)> {
        sqlx::query_as(
            "SELECT role_id, capability_id FROM role_capabilities ORDER BY role_id, capability_id",
        )
        .fetch_all(pool)
        .await
        .expect("exact role-capability mapping snapshot")
    }

    async fn control_plane_snapshot(pool: &PgPool) -> Value {
        sqlx::query_scalar(
            r#"
            SELECT jsonb_build_object(
                'application_installations', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.id), '[]'::jsonb)
                    FROM application_installations AS row
                ),
                'core_runtime_observations', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.installation_id), '[]'::jsonb)
                    FROM core_runtime_observations AS row
                ),
                'module_definition_reservations', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.definition_id), '[]'::jsonb)
                    FROM module_definition_reservations AS row
                ),
                'transition_descriptor_sources', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.definition_id, row.created_at, row.id),
                        '[]'::jsonb
                    )
                    FROM transition_descriptor_sources AS row
                ),
                'transition_catalog_projections', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.source_id), '[]'::jsonb)
                    FROM transition_catalog_projections AS row
                ),
                'transition_catalog_current', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.definition_id), '[]'::jsonb)
                    FROM transition_catalog_current AS row
                ),
                'module_catalog_findings', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.projection_id, row.ordinal, row.id),
                        '[]'::jsonb
                    )
                    FROM module_catalog_findings AS row
                ),
                'capabilities', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.key), '[]'::jsonb)
                    FROM capabilities AS row
                ),
                'capability_provenance', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.capability_id, row.source_key),
                        '[]'::jsonb
                    )
                    FROM capability_provenance AS row
                ),
                'role_capabilities', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.role_id, row.capability_id),
                        '[]'::jsonb
                    )
                    FROM role_capabilities AS row
                ),
                'module_navigation_contributions', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.contribution_id),
                        '[]'::jsonb
                    )
                    FROM module_navigation_contributions AS row
                ),
                'navigation_policies', (
                    SELECT COALESCE(jsonb_agg(to_jsonb(row) ORDER BY row.installation_id), '[]'::jsonb)
                    FROM navigation_policies AS row
                ),
                'navigation_policy_entries', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.installation_id, row.contribution_id),
                        '[]'::jsonb
                    )
                    FROM navigation_policy_entries AS row
                ),
                'non_rejection_audit_events', (
                    SELECT COALESCE(
                        jsonb_agg(to_jsonb(row) ORDER BY row.created_at, row.id),
                        '[]'::jsonb
                    )
                    FROM core_control_plane_audit_events AS row
                    WHERE row.event_type <> 'module_catalog.sync_rejected'
                )
            )
            "#,
        )
        .fetch_one(pool)
        .await
        .expect("complete module control-plane snapshot")
    }
}
