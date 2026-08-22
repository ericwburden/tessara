//! Dataset-owned synchronous source refresh orchestration.
//!
//! Provider pages are staged inside the caller's transaction. The caller can
//! then rebuild every affected Dataset materialization before committing, so
//! imported rows, cursors, receipts, and published tables never diverge.

use serde::Serialize;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_module_contract::{ModuleDefinitionId, ModuleServicePrincipalV1};
use tessara_responses_contract::{
    MAX_EXPORT_PAGE_SIZE, RESPONSE_EXPORT_BINDING_KEY, RESPONSE_EXPORT_CHECKPOINT_ACTION,
    RESPONSE_EXPORT_CHECKPOINT_PATH, RESPONSE_EXPORT_CONTRACT_ID, RESPONSE_EXPORT_MEDIA_TYPE,
    RESPONSE_EXPORT_PAGE_ACTION, RESPONSE_EXPORT_PAGE_PATH, RESPONSE_EXPORT_SCHEMA_VERSION,
    RESPONSE_EXPORT_START_ACTION, RESPONSE_EXPORT_START_PATH, ResponseExportAction,
    ResponseExportCheckpointRequest, ResponseExportCheckpointResponse, ResponseExportPageRequest,
    ResponseExportPageResponse, ResponseExportPartition, ResponseExportStartRequest,
    ResponseExportStartResponse,
};
use uuid::Uuid;

use crate::{
    DatasetModuleError, DatasetModuleState, MODULE_DEFINITION_ID,
    authoring::ValidatedDatasetSource,
    load_security_state,
    provider_client::{self, ProviderAction},
    sync::{
        BeginAttempt, PromotionReceipt, SyncStoreError, begin_attempt_in_transaction,
        promote_in_transaction, stage_page_in_transaction,
    },
};

const CHECKPOINT_PROVIDER: ProviderAction = ProviderAction {
    binding: RESPONSE_EXPORT_BINDING_KEY,
    contract: RESPONSE_EXPORT_CONTRACT_ID,
    action: RESPONSE_EXPORT_CHECKPOINT_ACTION,
    path: RESPONSE_EXPORT_CHECKPOINT_PATH,
    media_type: RESPONSE_EXPORT_MEDIA_TYPE,
    retry_safe_observation: true,
};

const START_PROVIDER: ProviderAction = ProviderAction {
    binding: RESPONSE_EXPORT_BINDING_KEY,
    contract: RESPONSE_EXPORT_CONTRACT_ID,
    action: RESPONSE_EXPORT_START_ACTION,
    path: RESPONSE_EXPORT_START_PATH,
    media_type: RESPONSE_EXPORT_MEDIA_TYPE,
    retry_safe_observation: true,
};

const PAGE_PROVIDER: ProviderAction = ProviderAction {
    binding: RESPONSE_EXPORT_BINDING_KEY,
    contract: RESPONSE_EXPORT_CONTRACT_ID,
    action: RESPONSE_EXPORT_PAGE_ACTION,
    path: RESPONSE_EXPORT_PAGE_PATH,
    media_type: RESPONSE_EXPORT_MEDIA_TYPE,
    retry_safe_observation: true,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct SourceRefreshOutcome {
    pub(crate) changed: bool,
    pub(crate) receipts: Vec<PromotionReceipt>,
}

/// Synchronizes every Response-backed source in a compiled Dataset definition.
/// No commit occurs here; callers own the surrounding definition/publication
/// and materialization transaction.
pub(crate) async fn synchronize_compiled_sources_in_transaction(
    state: &DatasetModuleState,
    grant: &dyn provider_client::ProviderAuthorization,
    transaction: &mut Transaction<'_, Postgres>,
    sources: &[ValidatedDatasetSource],
) -> Result<SourceRefreshOutcome, DatasetModuleError> {
    let mut outcome = SourceRefreshOutcome::default();
    for source in sources
        .iter()
        .filter(|source| source.form_version_id.is_some())
    {
        let refresh = synchronize_response_source_in_transaction(
            state,
            grant,
            transaction,
            source.source_binding_id.ok_or_else(|| {
                DatasetModuleError::Internal("Response source binding identity is missing".into())
            })?,
            source.form_version_id.ok_or_else(|| {
                DatasetModuleError::Internal(
                    "Response source FormVersion identity is missing".into(),
                )
            })?,
            &source.source_scope_node_ids,
        )
        .await?;
        outcome.changed |= refresh.is_some();
        outcome.receipts.extend(refresh);
    }
    Ok(outcome)
}

/// Validates every Response-backed provider partition before the owner begins
/// writing the Dataset definition. This is intentionally observation-only:
/// an incompatible provider therefore rejects bootstrap before Dataset product
/// state exists, while the subsequent fixed-bound synchronization remains the
/// authoritative import performed inside the mutation transaction.
pub(crate) async fn validate_compiled_sources_before_write(
    state: &DatasetModuleState,
    grant: &dyn provider_client::ProviderAuthorization,
    sources: &[ValidatedDatasetSource],
) -> Result<(), DatasetModuleError> {
    for source in sources {
        if source.form_version_id.is_none() {
            continue;
        }
        let partition = response_partition(
            state,
            source.source_binding_id.ok_or_else(|| {
                DatasetModuleError::Internal("Response source binding identity is missing".into())
            })?,
            source.form_version_id.ok_or_else(|| {
                DatasetModuleError::Internal(
                    "Response source FormVersion identity is missing".into(),
                )
            })?,
            &source.source_scope_node_ids,
        )
        .await?;
        let checkpoint: ResponseExportCheckpointResponse = provider_client::post(
            state,
            grant,
            CHECKPOINT_PROVIDER,
            &ResponseExportCheckpointRequest {
                schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
                action: ResponseExportAction::Checkpoint,
                partition,
                committed_cursor: None,
            },
        )
        .await?;
        if checkpoint.provider_epoch.is_nil()
            || !checkpoint.changed
            || !checkpoint.committed_cursor_valid
        {
            return Err(DatasetModuleError::DependencyIncompatible(
                "Response provider pre-write checkpoint is inconsistent".into(),
            ));
        }
    }
    Ok(())
}

/// Loads the currently published Response bindings for a Dataset and performs
/// the same fixed-bound synchronization used during create and publish.
pub(crate) async fn synchronize_dataset_sources_in_transaction(
    state: &DatasetModuleState,
    grant: &tessara_module_contract::SignedEnvelopeV1<
        tessara_module_contract::AuthorizationGrantV3,
    >,
    transaction: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
) -> Result<SourceRefreshOutcome, DatasetModuleError> {
    let rows = sqlx::query(
        "SELECT source_binding_id,source_reference,source_scope_node_ids
         FROM dataset_sources
         WHERE dataset_id=$1 AND source_kind='form_version'
         ORDER BY position,source_alias",
    )
    .bind(dataset_id)
    .fetch_all(&mut **transaction)
    .await?;
    let mut outcome = SourceRefreshOutcome::default();
    for row in rows {
        let source_binding_id: Uuid = row.try_get("source_binding_id")?;
        let source_reference: serde_json::Value = row.try_get("source_reference")?;
        let source_reference = serde_json::from_value::<
            tessara_datasets_contract::DatasetProductSourceV1,
        >(source_reference)
        .map_err(|error| {
            DatasetModuleError::Internal(format!(
                "Stored Dataset source reference is invalid: {error}"
            ))
        })?;
        let form_version_id = match source_reference {
            tessara_datasets_contract::DatasetProductSourceV1::Form {
                form_version_id, ..
            } => Uuid::parse_str(&form_version_id).map_err(|_| {
                DatasetModuleError::Internal(
                    "Stored Response source FormVersion identity is invalid".into(),
                )
            })?,
            _ => {
                return Err(DatasetModuleError::Internal(
                    "Stored form source has a non-Form reference".into(),
                ));
            }
        };
        let source_scope_node_ids: Vec<Uuid> = row.try_get("source_scope_node_ids")?;
        let refresh = synchronize_response_source_in_transaction(
            state,
            grant,
            transaction,
            source_binding_id,
            form_version_id,
            &source_scope_node_ids,
        )
        .await?;
        outcome.changed |= refresh.is_some();
        outcome.receipts.extend(refresh);
    }
    Ok(outcome)
}

async fn synchronize_response_source_in_transaction(
    state: &DatasetModuleState,
    grant: &dyn provider_client::ProviderAuthorization,
    transaction: &mut Transaction<'_, Postgres>,
    source_binding_id: Uuid,
    form_version_id: Uuid,
    source_scope_node_ids: &[Uuid],
) -> Result<Option<PromotionReceipt>, DatasetModuleError> {
    let partition = response_partition(
        state,
        source_binding_id,
        form_version_id,
        source_scope_node_ids,
    )
    .await?;

    let committed_cursor_text: Option<String> = sqlx::query_scalar(
        "SELECT committed_cursor FROM dataset_sync_partitions WHERE source_binding_id=$1 FOR UPDATE",
    )
    .bind(source_binding_id)
    .fetch_optional(&mut **transaction)
    .await?
    .ok_or_else(|| {
        DatasetModuleError::Internal("Response synchronization partition is missing".into())
    })?;
    let committed_cursor = committed_cursor_text
        .as_deref()
        .map(tessara_responses_contract::ResponseExportCursor::parse)
        .transpose()
        .map_err(|_| {
            DatasetModuleError::DependencyIncompatible(
                "Stored Response cursor is not canonical".into(),
            )
        })?;
    let checkpoint: ResponseExportCheckpointResponse = provider_client::post(
        state,
        grant,
        CHECKPOINT_PROVIDER,
        &ResponseExportCheckpointRequest {
            schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
            action: ResponseExportAction::Checkpoint,
            partition: partition.clone(),
            committed_cursor: committed_cursor.clone(),
        },
    )
    .await?;
    if checkpoint.provider_epoch.is_nil() {
        return Err(DatasetModuleError::DependencyIncompatible(
            "Response checkpoint identity is invalid".into(),
        ));
    }
    let checkpoint_should_change = !checkpoint.committed_cursor_valid
        || committed_cursor
            .as_ref()
            .is_none_or(|cursor| cursor != &checkpoint.authenticated_head);
    if checkpoint.changed != checkpoint_should_change {
        return Err(DatasetModuleError::DependencyIncompatible(
            "Response checkpoint change state is inconsistent".into(),
        ));
    }
    if !checkpoint.changed && checkpoint.committed_cursor_valid {
        sqlx::query(
            "UPDATE dataset_sync_partitions
             SET freshness_state='current',sanitized_failure_code=NULL,
                 last_checked_at=now(),updated_at=now()
             WHERE source_binding_id=$1",
        )
        .bind(source_binding_id)
        .execute(&mut **transaction)
        .await?;
        return Ok(None);
    }

    let full_snapshot_rebase = committed_cursor.is_none() || !checkpoint.committed_cursor_valid;
    let page_size: i32 = sqlx::query_scalar(
        "SELECT response_export_page_size FROM dataset_configuration WHERE singleton=true",
    )
    .fetch_one(&mut **transaction)
    .await?;
    let page_size = u16::try_from(page_size)
        .ok()
        .filter(|size| *size > 0 && *size <= MAX_EXPORT_PAGE_SIZE)
        .ok_or_else(|| {
            DatasetModuleError::Internal("Dataset Response export page size is invalid".into())
        })?;
    let start_request = ResponseExportStartRequest {
        schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
        action: ResponseExportAction::Start,
        partition: partition.clone(),
        provider_epoch: checkpoint.provider_epoch,
        committed_cursor: if full_snapshot_rebase {
            None
        } else {
            committed_cursor.clone()
        },
        authenticated_head: checkpoint.authenticated_head.clone(),
        full_snapshot_rebase,
        page_size,
    };
    start_request.validate().map_err(|_| {
        DatasetModuleError::Internal("Dataset built an invalid Response start request".into())
    })?;
    let start: ResponseExportStartResponse =
        provider_client::post(state, grant, START_PROVIDER, &start_request).await?;
    if start.provider_epoch != checkpoint.provider_epoch
        || start.snapshot_upper_bound != checkpoint.authenticated_head
        || start.full_snapshot_rebase != full_snapshot_rebase
        || start.start_after_cursor
            != if full_snapshot_rebase {
                None
            } else {
                committed_cursor.clone()
            }
    {
        return Err(DatasetModuleError::DependencyIncompatible(
            "Response snapshot start was substituted".into(),
        ));
    }

    let attempt_id = Uuid::new_v4();
    begin_attempt_in_transaction(
        transaction,
        BeginAttempt {
            attempt_id,
            source_binding_id,
            provider_epoch: start.provider_epoch,
            expected_committed_cursor: committed_cursor_text,
            snapshot_upper_bound: start.snapshot_upper_bound.as_str().to_owned(),
            full_snapshot_rebase,
        },
    )
    .await
    .map_err(sync_error)?;

    let mut after_cursor = start.start_after_cursor.clone();
    let mut page_digests = Vec::new();
    loop {
        let page: ResponseExportPageResponse = provider_client::post(
            state,
            grant,
            PAGE_PROVIDER,
            &ResponseExportPageRequest {
                schema_version: RESPONSE_EXPORT_SCHEMA_VERSION,
                action: ResponseExportAction::Page,
                partition: partition.clone(),
                provider_epoch: start.provider_epoch,
                snapshot_upper_bound: start.snapshot_upper_bound.clone(),
                after_cursor: after_cursor.clone(),
                page_size,
            },
        )
        .await?;
        if page.provider_epoch != start.provider_epoch
            || page.snapshot_upper_bound != start.snapshot_upper_bound
        {
            return Err(DatasetModuleError::DependencyIncompatible(
                "Response page snapshot identity was substituted".into(),
            ));
        }
        stage_page_in_transaction(
            transaction,
            attempt_id,
            after_cursor.as_ref().map(|cursor| cursor.as_str()),
            &page,
        )
        .await
        .map_err(sync_error)?;
        page_digests.push(page.page_digest.clone());
        if page.complete {
            break;
        }
        after_cursor = page.next_after_cursor;
    }

    let input_digest = canonical_digest(&(
        &partition,
        &checkpoint,
        &start,
        full_snapshot_rebase,
        page_size,
    ))?;
    let result_digest = canonical_digest(&page_digests)?;
    promote_in_transaction(
        transaction,
        attempt_id,
        Uuid::new_v4(),
        &input_digest,
        &result_digest,
    )
    .await
    .map(Some)
    .map_err(sync_error)
}

async fn response_partition(
    state: &DatasetModuleState,
    source_binding_id: Uuid,
    form_version_id: Uuid,
    source_scope_node_ids: &[Uuid],
) -> Result<ResponseExportPartition, DatasetModuleError> {
    if source_scope_node_ids.is_empty() {
        return Err(DatasetModuleError::DependencyIncompatible(
            "Response source scope is empty".into(),
        ));
    }
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
    })?;
    let presenting_service = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    };
    let mut partition = ResponseExportPartition {
        source_binding_id,
        form_version_ids: vec![form_version_id],
        authorized_scope_node_ids: source_scope_node_ids.to_vec(),
        authorization_digest: String::new(),
    };
    partition.form_version_ids.sort_unstable();
    partition.form_version_ids.dedup();
    partition.authorized_scope_node_ids.sort_unstable();
    partition.authorized_scope_node_ids.dedup();
    partition
        .recompute_authorization_digest(&presenting_service, security.installation_id)
        .map_err(|_| {
            DatasetModuleError::Internal(
                "Response partition authorization could not be canonicalized".into(),
            )
        })?;
    Ok(partition)
}

fn canonical_digest<T: Serialize>(value: &T) -> Result<String, DatasetModuleError> {
    let bytes = serde_jcs::to_vec(value)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn sync_error(error: SyncStoreError) -> DatasetModuleError {
    match error {
        SyncStoreError::Database(error) => DatasetModuleError::Database(error),
        SyncStoreError::InvalidProviderPage | SyncStoreError::InvalidStagedChange => {
            DatasetModuleError::DependencyIncompatible(
                "Response synchronization data is incompatible".into(),
            )
        }
        SyncStoreError::CursorConflict | SyncStoreError::StalePromotion => {
            DatasetModuleError::Conflict("A concurrent Dataset refresh won promotion".into())
        }
        SyncStoreError::MissingPartition
        | SyncStoreError::InactiveAttempt
        | SyncStoreError::PageCheckpointMismatch => DatasetModuleError::Internal(error.to_string()),
    }
}
