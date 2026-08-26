//! Dataset-owned public product APIs.

use std::collections::BTreeSet;

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Path, Query, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::{get, patch, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{Column, Postgres, QueryBuilder, Row, Transaction};
use tessara_control_plane_contract::{
    CONTROL_PLANE_SCHEMA_VERSION, ControlPlaneCatalogAction, PRINCIPAL_DISPLAY_ACTION,
    PRINCIPAL_DISPLAY_BINDING_KEY, PRINCIPAL_DISPLAY_CONTRACT_ID, PRINCIPAL_DISPLAY_MEDIA_TYPE,
    PRINCIPAL_DISPLAY_PATH, PrincipalDisplayCatalogRequest, PrincipalDisplayCatalogResponse,
    SCOPE_CATALOG_ACTION, SCOPE_CATALOG_BINDING_KEY, SCOPE_CATALOG_CONTRACT_ID,
    SCOPE_CATALOG_MEDIA_TYPE, SCOPE_CATALOG_PATH, ScopeCatalogRequest, ScopeCatalogResponse,
};
use tessara_datasets_contract::{
    DATASET_AUTHORING_CONTRACT_ID, DATASET_CONTRACT_ID, DATASET_IDEMPOTENCY_HEADER,
    DatasetAuthoringRequestV1, DatasetDraftRevisionResponseV1, DatasetEditorFormOptionV1,
    DatasetEditorFormVersionOptionV1, DatasetEditorPrincipalOptionV1, DatasetEditorRenderedFieldV1,
    DatasetEditorRenderedFormV1, DatasetEditorRenderedSectionV1, DatasetEditorScopeOptionV1,
    DatasetFreshnessState, DatasetMutationIdResponseV1, DatasetProductCarryForwardStateV1,
    DatasetProductCompatibilityFindingV1, DatasetProductCompatibilityStateV1,
    DatasetProductDefinitionV1, DatasetProductDependencyBindingModeV1,
    DatasetProductDependencyImpactV1, DatasetProductDependencyKindV1,
    DatasetProductDependencySummaryV1, DatasetProductDistinctValuesQueryV1,
    DatasetProductDistinctValuesV1, DatasetProductFieldV1, DatasetProductFreshnessV1,
    DatasetProductLineageNodeV1, DatasetProductOperationV1, DatasetProductProvenanceItemV1,
    DatasetProductProvenanceSummaryV1, DatasetProductRestrictionPolicyV1,
    DatasetProductRevisionDetailV1, DatasetProductRevisionFieldSummaryV1,
    DatasetProductRevisionMetadataV1, DatasetProductRevisionStatusV1,
    DatasetProductRevisionSummaryV1, DatasetProductSemanticBumpV1,
    DatasetProductSourceDefinitionV1, DatasetProductSourceV1, DatasetProductSummaryV1,
    DatasetProductTableRowV1, DatasetProductTableV1, DatasetProductVisibilityNodeV1,
    DatasetPublishRevisionResponseV1, DatasetRefreshResponseV1, DatasetRevisionLabelRequestV1,
    DatasetRevisionLabelResponseV1, DatasetRevisionOptionsRequestV1, DatasetSqlPreviewResponseV1,
    DatasetUpdateTagsRequestV1,
};
use tessara_forms_contract::{
    FORM_VERSION_CATALOG_ACTION, FORM_VERSION_CATALOG_PATH, FORM_VERSION_SCHEMA_ACTION,
    FORM_VERSION_SCHEMA_BINDING_KEY, FORM_VERSION_SCHEMA_CONTRACT_ID,
    FORM_VERSION_SCHEMA_MEDIA_TYPE, FORM_VERSION_SCHEMA_PATH, FORM_VERSION_SCHEMA_VERSION,
    FormVersionCatalogRequest, FormVersionCatalogResponse, FormVersionSchemaAction,
    FormVersionSchemaRequest, FormVersionSchemaResponse,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, DependencyBindingKey, FunctionalContractId,
    ModuleDefinitionId, ModuleServicePrincipalV1, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    DatasetModuleError, DatasetModuleState, MANAGE_CAPABILITY, MODULE_DEFINITION_ID,
    READ_CAPABILITY, load_security_state, provider_client::ProviderAction,
};

const CORE_DATASET_BINDING: &str = "tessara.core.datasets";
const FORM_CATALOG_PROVIDER: ProviderAction = ProviderAction {
    binding: FORM_VERSION_SCHEMA_BINDING_KEY,
    contract: FORM_VERSION_SCHEMA_CONTRACT_ID,
    action: FORM_VERSION_CATALOG_ACTION,
    path: FORM_VERSION_CATALOG_PATH,
    media_type: FORM_VERSION_SCHEMA_MEDIA_TYPE,
    retry_safe_observation: true,
};
const FORM_SCHEMA_PROVIDER: ProviderAction = ProviderAction {
    binding: FORM_VERSION_SCHEMA_BINDING_KEY,
    contract: FORM_VERSION_SCHEMA_CONTRACT_ID,
    action: FORM_VERSION_SCHEMA_ACTION,
    path: FORM_VERSION_SCHEMA_PATH,
    media_type: FORM_VERSION_SCHEMA_MEDIA_TYPE,
    retry_safe_observation: true,
};
const SCOPE_CATALOG_PROVIDER: ProviderAction = ProviderAction {
    binding: SCOPE_CATALOG_BINDING_KEY,
    contract: SCOPE_CATALOG_CONTRACT_ID,
    action: SCOPE_CATALOG_ACTION,
    path: SCOPE_CATALOG_PATH,
    media_type: SCOPE_CATALOG_MEDIA_TYPE,
    retry_safe_observation: true,
};
const PRINCIPAL_CATALOG_PROVIDER: ProviderAction = ProviderAction {
    binding: PRINCIPAL_DISPLAY_BINDING_KEY,
    contract: PRINCIPAL_DISPLAY_CONTRACT_ID,
    action: PRINCIPAL_DISPLAY_ACTION,
    path: PRINCIPAL_DISPLAY_PATH,
    media_type: PRINCIPAL_DISPLAY_MEDIA_TYPE,
    retry_safe_observation: true,
};

pub(super) fn routes() -> Router<DatasetModuleState> {
    Router::new()
        .route(
            "/api/admin/datasets/sql-preview",
            post(preview_new_dataset_sql),
        )
        .route("/api/admin/datasets", post(create_dataset))
        .route(
            "/api/admin/datasets/{dataset_id}/refresh",
            post(refresh_dataset),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/draft-revision",
            post(save_dataset_draft_revision),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish",
            post(publish_dataset_revision),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/sql-preview",
            post(preview_existing_dataset_sql),
        )
        .route("/api/datasets", get(list_datasets))
        .route("/api/datasets/{dataset_id}", get(get_dataset))
        .route("/api/datasets/{dataset_id}/table", get(run_dataset_table))
        .route(
            "/api/datasets/{dataset_id}/distinct-values",
            get(list_dataset_distinct_values),
        )
        .route(
            "/api/admin/datasets/{dataset_id}",
            axum::routing::delete(delete_dataset),
        )
        .route(
            "/api/datasets/{dataset_id}/revisions",
            get(list_dataset_revisions),
        )
        .route(
            "/api/datasets/{dataset_id}/revisions/{revision_id}",
            get(get_dataset_revision),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/tags",
            patch(update_dataset_tags),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/label",
            patch(update_dataset_revision_label),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/options",
            patch(update_dataset_revision_options),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}",
            axum::routing::delete(delete_dataset_revision),
        )
        .route(
            "/api/admin/datasets/editor-options/forms",
            get(editor_form_options),
        )
        .route(
            "/api/admin/datasets/editor-options/forms/{form_version_id}",
            get(editor_form_schema),
        )
        .route(
            "/api/admin/datasets/editor-options/scopes",
            get(editor_scope_options),
        )
        .route(
            "/api/admin/datasets/editor-options/principals",
            get(editor_principal_options),
        )
}

async fn preview_new_dataset_sql(
    State(state): State<DatasetModuleState>,
    request: Request,
) -> Result<Json<DatasetSqlPreviewResponseV1>, DatasetModuleError> {
    preview_dataset_sql(state, None, request).await
}

async fn create_dataset(
    State(state): State<DatasetModuleState>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    require_json_content_type(&headers)?;
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| {
            DatasetModuleError::BadRequest("Dataset authoring payload is too large".into())
        })?;
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.create",
        "POST",
        "/api/admin/datasets",
        &body,
        idempotency_key,
    )
    .await?;
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    crate::dependency_dag::lock_dependency_graph_in_transaction(&mut tx).await?;
    let payload: DatasetAuthoringRequestV1 = serde_json::from_slice(&body).map_err(|error| {
        DatasetModuleError::BadRequest(format!("Invalid Dataset authoring payload: {error}"))
    })?;
    let created =
        create_bootstrap_dataset_in_transaction(&state, &mutation.grant, &mut tx, None, &payload)
            .await?;
    let response_body = serde_json::to_vec(&DatasetMutationIdResponseV1 {
        id: created.dataset_id.to_string(),
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 201, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(201, "application/json", response_body)
}

pub(crate) struct BootstrapDatasetCreated {
    pub(crate) dataset_id: Uuid,
    pub(crate) revision_id: Uuid,
}

/// Creates one published Dataset inside an owner-controlled bootstrap transaction.
///
/// The caller owns commit/rollback and the dependency-graph lock. Provider reads,
/// source promotion, and materialization remain inside the same transaction so a
/// subsequent Dataset in the same bootstrap can safely consume this major line.
pub(crate) async fn create_bootstrap_dataset_in_transaction(
    state: &DatasetModuleState,
    authorization: &dyn crate::provider_client::ProviderAuthorization,
    transaction: &mut Transaction<'_, Postgres>,
    bootstrap_resource_key: Option<&str>,
    payload: &DatasetAuthoringRequestV1,
) -> Result<BootstrapDatasetCreated, DatasetModuleError> {
    validate_authoring_payload(payload)?;
    if sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM datasets WHERE slug=$1)")
        .bind(payload.slug.trim())
        .fetch_one(&mut **transaction)
        .await?
    {
        return Err(DatasetModuleError::Conflict(
            "Dataset slug is already in use".into(),
        ));
    }
    let scope = resolve_authoring_scope(state, authorization, &payload.visibility_node_ids).await?;
    let dataset_id = Uuid::new_v4();
    let compiled = crate::authoring::compile_dataset_definition_in_transaction(
        state,
        authorization,
        transaction,
        Some(dataset_id),
        payload,
    )
    .await
    .map_err(DatasetModuleError::from)?;
    let candidate_sources = compiled
        .sources
        .iter()
        .map(|source| source.reference.clone())
        .collect::<Vec<_>>();
    crate::dependency_dag::validate_candidate_sources_in_transaction(
        transaction,
        dataset_id,
        &candidate_sources,
    )
    .await?;
    crate::refresh::validate_compiled_sources_before_write(state, authorization, &compiled.sources)
        .await?;
    let revision_id = Uuid::new_v4();
    persist_initial_dataset(
        transaction,
        dataset_id,
        revision_id,
        payload,
        &scope,
        &compiled,
    )
    .await?;
    crate::refresh::synchronize_compiled_sources_in_transaction(
        state,
        authorization,
        transaction,
        &compiled.sources,
    )
    .await?;
    if let Some(resource_key) = bootstrap_resource_key {
        state
            .validation_fault_control
            .fail_derived_rebuild(resource_key)?;
    }
    crate::dependency_dag::rebuild_affected_published_closure_in_transaction(
        transaction,
        &[dataset_id],
    )
    .await?;
    Ok(BootstrapDatasetCreated {
        dataset_id,
        revision_id,
    })
}

async fn refresh_dataset(
    State(state): State<DatasetModuleState>,
    Path(dataset_id): Path<Uuid>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 1)
        .await
        .map_err(|_| DatasetModuleError::BadRequest("Dataset refresh body must be empty".into()))?;
    if !body.is_empty() {
        return Err(DatasetModuleError::BadRequest(
            "Dataset refresh body must be empty".into(),
        ));
    }
    let path = format!("/api/admin/datasets/{dataset_id}/refresh");
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.refresh",
        "POST",
        &path,
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }

    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    crate::dependency_dag::lock_dependency_graph_in_transaction(&mut tx).await?;
    sqlx::query("SELECT id FROM datasets WHERE id=$1 FOR UPDATE")
        .bind(dataset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(undisclosed_dataset)?;
    sqlx::query(
        "SELECT id
         FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' AND lifecycle_state <> 'tombstoned'
         FOR UPDATE",
    )
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| DatasetModuleError::Conflict("Dataset has no published revision".into()))?;
    if !dataset_sources_fully_in_scope_in_transaction(&mut tx, dataset_id, &manage_scopes).await? {
        tx.rollback().await?;
        return Err(undisclosed_dataset());
    }
    let refresh = crate::refresh::synchronize_dataset_sources_in_transaction(
        &state,
        &mutation.grant,
        &mut tx,
        dataset_id,
    )
    .await;
    let outcome = match refresh {
        Ok(outcome) => outcome,
        Err(error) => {
            tx.rollback().await?;
            mark_refresh_failure(&state, dataset_id, &error).await?;
            return Err(error);
        }
    };
    if outcome.changed
        && let Err(error) =
            crate::dependency_dag::rebuild_affected_published_closure_in_transaction(
                &mut tx,
                &[dataset_id],
            )
            .await
    {
        tx.rollback().await?;
        mark_refresh_failure(&state, dataset_id, &error).await?;
        return Err(error);
    }
    let freshness = load_dataset_freshness_in_transaction(&mut tx, dataset_id).await?;
    let response_body = serde_json::to_vec(&DatasetRefreshResponseV1 {
        dataset_id: dataset_id.to_string(),
        changed: outcome.changed,
        freshness,
        materialization_receipt_ids: outcome
            .receipts
            .iter()
            .map(|receipt| receipt.receipt_id.to_string())
            .collect(),
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

async fn mark_refresh_failure(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    error: &DatasetModuleError,
) -> Result<(), DatasetModuleError> {
    let failure_code = match error {
        DatasetModuleError::DependencyIncompatible(_) => "dataset.dependency_incompatible",
        DatasetModuleError::Unavailable(_) | DatasetModuleError::Database(_) => {
            "dataset.dependency_unavailable"
        }
        _ => "dataset.refresh_failed",
    };
    sqlx::query(
        "UPDATE dataset_sync_partitions p
         SET freshness_state='degraded',sanitized_failure_code=$2,
             last_checked_at=now(),updated_at=now()
         FROM dataset_source_bindings b
         WHERE p.source_binding_id=b.id AND b.dataset_id=$1",
    )
    .bind(dataset_id)
    .bind(failure_code)
    .execute(&state.pool)
    .await?;
    Ok(())
}

async fn publish_dataset_revision(
    State(state): State<DatasetModuleState>,
    Path((dataset_id, revision_id)): Path<(Uuid, Uuid)>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 1)
        .await
        .map_err(|_| DatasetModuleError::BadRequest("Dataset publish body must be empty".into()))?;
    if !body.is_empty() {
        return Err(DatasetModuleError::BadRequest(
            "Dataset publish body must be empty".into(),
        ));
    }
    let path = format!("/api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish");
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.publish_revision",
        "POST",
        &path,
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }

    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    crate::dependency_dag::lock_dependency_graph_in_transaction(&mut tx).await?;
    sqlx::query("SELECT id FROM datasets WHERE id=$1 FOR UPDATE")
        .bind(dataset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(undisclosed_dataset)?;
    let draft = sqlx::query(
        "SELECT version_number,version_label,version_major,version_minor,version_patch,
                semantic_bump,started_new_major_line,force_new_major_version,
                initial_source,operations,restriction_policy,definition_metadata,
                compatibility_findings
         FROM dataset_revisions
         WHERE id=$1 AND dataset_id=$2 AND status='draft'
           AND lifecycle_state <> 'tombstoned'
         FOR UPDATE",
    )
    .bind(revision_id)
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| {
        DatasetModuleError::ValidationFailed(
            "Only an existing draft Dataset revision can be published".into(),
        )
    })?;
    let findings: Vec<DatasetProductCompatibilityFindingV1> = parse_json(
        draft.try_get("compatibility_findings")?,
        "draft Dataset compatibility findings",
    )?;
    crate::compatibility::require_publishable_changelog(&findings)?;
    let metadata: DatasetProductRevisionMetadataV1 = parse_json(
        draft.try_get("definition_metadata")?,
        "draft Dataset definition metadata",
    )?;
    let initial_source: DatasetProductSourceV1 = parse_json(
        draft.try_get("initial_source")?,
        "draft Dataset initial source",
    )?;
    let operations: Vec<DatasetProductOperationV1> =
        parse_json(draft.try_get("operations")?, "draft Dataset operations")?;
    let restriction_policy: Option<DatasetProductRestrictionPolicyV1> = draft
        .try_get::<Option<serde_json::Value>, _>("restriction_policy")?
        .map(|value| parse_json(value, "draft Dataset restriction policy"))
        .transpose()?;
    let payload = DatasetAuthoringRequestV1 {
        name: metadata.name.clone(),
        slug: metadata.slug.clone(),
        grain: metadata.grain.clone(),
        version_label: Some(draft.try_get("version_label")?),
        force_new_major_version: draft.try_get("force_new_major_version")?,
        visibility_node_ids: metadata.visibility_node_ids.clone(),
        initial_source,
        operations,
        restriction_policy,
    };
    validate_authoring_payload(&payload)?;
    let duplicate_slug = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM datasets WHERE slug=$1 AND id<>$2)",
    )
    .bind(payload.slug.trim())
    .bind(dataset_id)
    .fetch_one(&mut *tx)
    .await?;
    if duplicate_slug {
        return Err(DatasetModuleError::Conflict(
            "Dataset slug is already in use".into(),
        ));
    }
    let scope =
        resolve_authoring_scope(&state, &mutation.grant, &payload.visibility_node_ids).await?;
    let compiled = crate::authoring::compile_dataset_definition(
        &state,
        &mutation.grant,
        Some(dataset_id),
        &payload,
    )
    .await
    .map_err(DatasetModuleError::from)?;
    let candidate_sources = compiled
        .sources
        .iter()
        .map(|source| source.reference.clone())
        .collect::<Vec<_>>();
    crate::dependency_dag::validate_candidate_sources_in_transaction(
        &mut tx,
        dataset_id,
        &candidate_sources,
    )
    .await?;
    let version_major = draft
        .try_get::<Option<i32>, _>("version_major")?
        .ok_or_else(|| DatasetModuleError::Internal("Draft Dataset major is missing".into()))?;
    let version_minor = draft
        .try_get::<Option<i32>, _>("version_minor")?
        .ok_or_else(|| DatasetModuleError::Internal("Draft Dataset minor is missing".into()))?;
    let version_patch = draft
        .try_get::<Option<i32>, _>("version_patch")?
        .ok_or_else(|| DatasetModuleError::Internal("Draft Dataset patch is missing".into()))?;
    let semantic_bump = semantic_bump(draft.try_get::<Option<String>, _>("semantic_bump")?)?
        .ok_or_else(|| DatasetModuleError::Internal("Draft Dataset bump is missing".into()))?;
    let started_new_major_line: bool = draft.try_get("started_new_major_line")?;
    let superseded_revision_id: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' FOR UPDATE",
    )
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?;

    replace_current_dataset_catalog(&mut tx, dataset_id, &payload, &scope, &compiled).await?;
    sqlx::query(
        "UPDATE dataset_revisions SET status='superseded',updated_at=now()
         WHERE dataset_id=$1 AND status='published'",
    )
    .bind(dataset_id)
    .execute(&mut *tx)
    .await?;
    let output_fields = compiled
        .fields
        .iter()
        .map(|field| DatasetProductFieldV1 {
            key: field.key.clone(),
            label: field.label.clone(),
            source_alias: field.source_alias.clone(),
            source_field_key: field.source_field_key.clone(),
            field_type: field.field_type.clone(),
            position: field.position,
        })
        .collect::<Vec<_>>();
    sqlx::query(
        "UPDATE dataset_revisions
         SET status='published',published_at=now(),initial_source=$1,operations=$2,
             restriction_policy=$3,generated_sql=$4,output_fields=$5,
             definition_metadata=$6,updated_at=now()
         WHERE id=$7 AND dataset_id=$8 AND status='draft'",
    )
    .bind(
        serde_json::to_value(&compiled.initial_source)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .bind(
        serde_json::to_value(&compiled.operations)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .bind(
        compiled
            .restriction_policy
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .bind(&compiled.generated_sql)
    .bind(
        serde_json::to_value(&output_fields)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .bind(
        serde_json::to_value(&metadata)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .bind(revision_id)
    .bind(dataset_id)
    .execute(&mut *tx)
    .await?;
    replace_revision_snapshots(&mut tx, revision_id, &scope, &compiled).await?;
    crate::refresh::synchronize_compiled_sources_in_transaction(
        &state,
        &mutation.grant,
        &mut tx,
        &compiled.sources,
    )
    .await?;
    crate::dependency_dag::rebuild_affected_published_closure_in_transaction(
        &mut tx,
        &[dataset_id],
    )
    .await?;

    let compatibility = crate::compatibility::compatibility_summary(&findings);
    let dataset_dependency_count = compiled
        .sources
        .iter()
        .filter(|source| !matches!(source.reference, DatasetProductSourceV1::Form { .. }))
        .count();
    let dependencies = DatasetProductDependencySummaryV1 {
        dependency_count: dataset_dependency_count,
        dataset_count: dataset_dependency_count,
        carry_forward_state: if compatibility.state == DatasetProductCompatibilityStateV1::Breaking
        {
            DatasetProductCarryForwardStateV1::Blocked
        } else {
            DatasetProductCarryForwardStateV1::Safe
        },
    };
    let version_label: String = draft.try_get("version_label")?;
    let response_body = serde_json::to_vec(&DatasetPublishRevisionResponseV1 {
        dataset_id: dataset_id.to_string(),
        revision_id: revision_id.to_string(),
        superseded_revision_id: superseded_revision_id.map(|id| id.to_string()),
        semantic_version: format!("v{version_major}.{version_minor}.{version_patch}"),
        version_label,
        version_major,
        version_minor,
        version_patch,
        semantic_bump,
        started_new_major_line,
        status: DatasetProductRevisionStatusV1::Published,
        dependencies,
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

async fn save_dataset_draft_revision(
    State(state): State<DatasetModuleState>,
    Path(dataset_id): Path<Uuid>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    require_json_content_type(&headers)?;
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| {
            DatasetModuleError::BadRequest("Dataset authoring payload is too large".into())
        })?;
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.save_draft_revision",
        "POST",
        &format!("/api/admin/datasets/{dataset_id}/draft-revision"),
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    crate::dependency_dag::lock_dependency_graph_in_transaction(&mut tx).await?;
    let payload: DatasetAuthoringRequestV1 = serde_json::from_slice(&body).map_err(|error| {
        DatasetModuleError::BadRequest(format!("Invalid Dataset authoring payload: {error}"))
    })?;
    validate_authoring_payload(&payload)?;
    let duplicate_slug = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM datasets WHERE slug=$1 AND id<>$2)",
    )
    .bind(payload.slug.trim())
    .bind(dataset_id)
    .fetch_one(&mut *tx)
    .await?;
    if duplicate_slug {
        return Err(DatasetModuleError::Conflict(
            "Dataset slug is already in use".into(),
        ));
    }
    let scope =
        resolve_authoring_scope(&state, &mutation.grant, &payload.visibility_node_ids).await?;
    let compiled = crate::authoring::compile_dataset_definition(
        &state,
        &mutation.grant,
        Some(dataset_id),
        &payload,
    )
    .await
    .map_err(DatasetModuleError::from)?;
    let candidate_sources = compiled
        .sources
        .iter()
        .map(|source| source.reference.clone())
        .collect::<Vec<_>>();
    crate::dependency_dag::validate_candidate_sources_in_transaction(
        &mut tx,
        dataset_id,
        &candidate_sources,
    )
    .await?;
    sqlx::query("SELECT id FROM datasets WHERE id=$1 FOR UPDATE")
        .bind(dataset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(undisclosed_dataset)?;
    let published = sqlx::query(
        "SELECT version_major,version_minor,version_patch,initial_source,operations,
                restriction_policy,definition_metadata,output_fields
         FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' AND lifecycle_state <> 'tombstoned'",
    )
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| DatasetModuleError::Conflict("Dataset has no published revision".into()))?;
    let candidate_fields = compiled
        .fields
        .iter()
        .map(|field| DatasetProductFieldV1 {
            key: field.key.clone(),
            label: field.label.clone(),
            source_alias: field.source_alias.clone(),
            source_field_key: field.source_field_key.clone(),
            field_type: field.field_type.clone(),
            position: field.position,
        })
        .collect::<Vec<_>>();
    let metadata = DatasetProductRevisionMetadataV1 {
        name: payload.name.trim().to_owned(),
        slug: payload.slug.trim().to_owned(),
        grain: payload.grain.clone(),
        visibility_node_ids: payload.visibility_node_ids.clone(),
    };
    let published_snapshot = crate::compatibility::DatasetCompatibilitySnapshot {
        metadata: parse_json(
            published.try_get("definition_metadata")?,
            "published Dataset definition metadata",
        )?,
        initial_source: parse_json(
            published.try_get("initial_source")?,
            "published Dataset initial source",
        )?,
        operations: parse_json(
            published.try_get("operations")?,
            "published Dataset operations",
        )?,
        restriction_policy: published
            .try_get::<Option<serde_json::Value>, _>("restriction_policy")?
            .map(|value| parse_json(value, "published Dataset restriction policy"))
            .transpose()?,
        output_fields: parse_json(
            published.try_get("output_fields")?,
            "published Dataset output fields",
        )?,
    };
    let candidate_snapshot = crate::compatibility::DatasetCompatibilitySnapshot {
        metadata: metadata.clone(),
        initial_source: compiled.initial_source.clone(),
        operations: compiled.operations.clone(),
        restriction_policy: compiled.restriction_policy.clone(),
        output_fields: candidate_fields.clone(),
    };
    let findings =
        crate::compatibility::compatibility_findings(&published_snapshot, &candidate_snapshot);
    let compatibility = crate::compatibility::compatibility_summary(&findings);
    let current_major = published
        .try_get::<Option<i32>, _>("version_major")?
        .unwrap_or(1);
    let current_minor = published
        .try_get::<Option<i32>, _>("version_minor")?
        .unwrap_or(0);
    let current_patch = published
        .try_get::<Option<i32>, _>("version_patch")?
        .unwrap_or(0);
    let bump = crate::compatibility::semantic_bump_for_publish(
        &compatibility,
        payload.force_new_major_version,
    );
    let (version_major, version_minor, version_patch) = match bump {
        DatasetProductSemanticBumpV1::Initial => (1, 0, 0),
        DatasetProductSemanticBumpV1::Major => (current_major + 1, 0, 0),
        DatasetProductSemanticBumpV1::Minor => (current_major, current_minor + 1, 0),
        DatasetProductSemanticBumpV1::Patch => (current_major, current_minor, current_patch + 1),
    };
    let existing_draft = sqlx::query(
        "SELECT id,version_number FROM dataset_revisions
         WHERE dataset_id=$1 AND status='draft' FOR UPDATE",
    )
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (revision_id, version_number) = if let Some(row) = existing_draft {
        (row.try_get("id")?, row.try_get("version_number")?)
    } else {
        (
            Uuid::new_v4(),
            sqlx::query_scalar::<_, i32>(
                "SELECT COALESCE(MAX(version_number),0)+1 FROM dataset_revisions WHERE dataset_id=$1",
            )
            .bind(dataset_id)
            .fetch_one(&mut *tx)
            .await?,
        )
    };
    let initial_source = serde_json::to_value(&compiled.initial_source)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let operations = serde_json::to_value(&compiled.operations)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let restriction_policy = compiled
        .restriction_policy
        .as_ref()
        .map(serde_json::to_value)
        .transpose()
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let output_fields = serde_json::to_value(&candidate_fields)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let findings_json = serde_json::to_value(&findings)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let metadata_json = serde_json::to_value(&metadata)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,version_patch,
          semantic_bump,started_new_major_line,force_new_major_version,status,initial_source,
          operations,restriction_policy,definition_metadata,compatibility_findings,generated_sql,
          output_fields)
         VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,'draft',$11,$12,$13,$14,$15,$16,$17)
         ON CONFLICT (dataset_id,version_number) DO UPDATE SET
          version_label=EXCLUDED.version_label,version_major=EXCLUDED.version_major,
          version_minor=EXCLUDED.version_minor,version_patch=EXCLUDED.version_patch,
          semantic_bump=EXCLUDED.semantic_bump,started_new_major_line=EXCLUDED.started_new_major_line,
          force_new_major_version=EXCLUDED.force_new_major_version,initial_source=EXCLUDED.initial_source,
          operations=EXCLUDED.operations,restriction_policy=EXCLUDED.restriction_policy,
          definition_metadata=EXCLUDED.definition_metadata,
          compatibility_findings=EXCLUDED.compatibility_findings,generated_sql=EXCLUDED.generated_sql,
          output_fields=EXCLUDED.output_fields,materialized_schema=NULL,materialized_table=NULL,
          materialized_row_count=NULL,materialized_at=NULL,published_at=NULL,updated_at=now()",
    )
    .bind(revision_id)
    .bind(dataset_id)
    .bind(version_number)
    .bind(
        payload
            .version_label
            .as_deref()
            .map(str::trim)
            .unwrap_or_default(),
    )
    .bind(version_major)
    .bind(version_minor)
    .bind(version_patch)
    .bind(semantic_bump_storage(bump))
    .bind(matches!(bump, DatasetProductSemanticBumpV1::Major))
    .bind(payload.force_new_major_version)
    .bind(initial_source)
    .bind(operations)
    .bind(restriction_policy)
    .bind(metadata_json)
    .bind(findings_json)
    .bind(&compiled.generated_sql)
    .bind(output_fields)
    .execute(&mut *tx)
    .await?;
    replace_revision_snapshots(&mut tx, revision_id, &scope, &compiled).await?;
    let dataset_dependency_count = compiled
        .sources
        .iter()
        .filter(|source| !matches!(source.reference, DatasetProductSourceV1::Form { .. }))
        .count();
    let dependencies = DatasetProductDependencySummaryV1 {
        dependency_count: dataset_dependency_count,
        dataset_count: dataset_dependency_count,
        carry_forward_state: if compatibility.state == DatasetProductCompatibilityStateV1::Breaking
        {
            DatasetProductCarryForwardStateV1::Blocked
        } else {
            DatasetProductCarryForwardStateV1::Safe
        },
    };
    let response_body = serde_json::to_vec(&DatasetDraftRevisionResponseV1 {
        dataset_id: dataset_id.to_string(),
        revision_id: revision_id.to_string(),
        status: DatasetProductRevisionStatusV1::Draft,
        compatibility,
        dependencies,
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

async fn replace_current_dataset_catalog(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    payload: &DatasetAuthoringRequestV1,
    scope: &ScopeCatalogResponse,
    compiled: &crate::authoring::CompiledDataset,
) -> Result<(), DatasetModuleError> {
    sqlx::query("UPDATE datasets SET name=$1,slug=$2,grain=$3,updated_at=now() WHERE id=$4")
        .bind(payload.name.trim())
        .bind(payload.slug.trim())
        .bind(&payload.grain)
        .bind(dataset_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM dataset_scope_nodes WHERE dataset_id=$1")
        .bind(dataset_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM dataset_source_bindings WHERE dataset_id=$1")
        .bind(dataset_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM dataset_fields WHERE dataset_id=$1")
        .bind(dataset_id)
        .execute(&mut **tx)
        .await?;
    for node in &scope.nodes {
        sqlx::query(
            "INSERT INTO dataset_scope_nodes
             (dataset_id,node_id,node_name,node_type_name,parent_node_id,node_path,
              requested_set_revision,requested_set_digest)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(dataset_id)
        .bind(node.node_id)
        .bind(&node.display_label)
        .bind(&node.node_type_name)
        .bind(node.parent_node_id)
        .bind(&node.node_path)
        .bind(&scope.requested_set_revision)
        .bind(&scope.requested_set_digest)
        .execute(&mut **tx)
        .await?;
    }
    for source in &compiled.sources {
        let source_binding_id = source.source_binding_id.ok_or_else(|| {
            DatasetModuleError::Internal("Compiled Dataset source has no binding identity".into())
        })?;
        let source_identity = serde_json::to_value(&source.reference)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
        let binding_kind = if source.form_version_id.is_some() {
            "response_export"
        } else {
            "dataset_major_line"
        };
        sqlx::query(
            "INSERT INTO dataset_source_bindings
             (id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest)
             VALUES($1,$2,$3,$4,$5,$6)",
        )
        .bind(source_binding_id)
        .bind(dataset_id)
        .bind(&source.source_alias)
        .bind(binding_kind)
        .bind(&source_identity)
        .bind(digest_json(&source_identity)?)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "INSERT INTO dataset_sync_partitions(source_binding_id,freshness_state)
             VALUES($1,$2)",
        )
        .bind(source_binding_id)
        .bind(if binding_kind == "response_export" {
            "never_materialized"
        } else {
            "current"
        })
        .execute(&mut **tx)
        .await?;
        let source_kind = match &source.reference {
            DatasetProductSourceV1::Form { .. } => "form_version",
            DatasetProductSourceV1::Dataset { .. } => "dataset_revision",
            DatasetProductSourceV1::DatasetMajor { .. } => "dataset_major_line",
        };
        sqlx::query(
            "INSERT INTO dataset_sources
             (id,dataset_id,source_binding_id,source_alias,source_kind,source_reference,
              source_name,source_slug,source_version_label,source_scope_node_ids,
              source_scope_revision,source_scope_digest,source_content_revision,
              source_content_digest,position)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
        )
        .bind(Uuid::new_v4())
        .bind(dataset_id)
        .bind(source_binding_id)
        .bind(&source.source_alias)
        .bind(source_kind)
        .bind(&source_identity)
        .bind(&source.source_name)
        .bind(&source.source_slug)
        .bind(&source.source_version_label)
        .bind(&source.source_scope_node_ids)
        .bind(&source.source_scope_revision)
        .bind(&source.source_scope_digest)
        .bind(&source.source_content_revision)
        .bind(&source.source_content_digest)
        .bind(source.position)
        .execute(&mut **tx)
        .await?;
    }
    for field in &compiled.fields {
        sqlx::query(
            "INSERT INTO dataset_fields
             (id,dataset_id,key,label,source_alias,source_field_key,source_field_id,field_type,position)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(Uuid::new_v4())
        .bind(dataset_id)
        .bind(&field.key)
        .bind(&field.label)
        .bind(&field.source_alias)
        .bind(&field.source_field_key)
        .bind(field.source_field_id)
        .bind(&field.field_type)
        .bind(field.position)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn persist_initial_dataset(
    tx: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    revision_id: Uuid,
    payload: &DatasetAuthoringRequestV1,
    scope: &ScopeCatalogResponse,
    compiled: &crate::authoring::CompiledDataset,
) -> Result<(), DatasetModuleError> {
    sqlx::query("INSERT INTO datasets(id,name,slug,grain) VALUES($1,$2,$3,$4)")
        .bind(dataset_id)
        .bind(payload.name.trim())
        .bind(payload.slug.trim())
        .bind(&payload.grain)
        .execute(&mut **tx)
        .await?;
    for node in &scope.nodes {
        sqlx::query(
            "INSERT INTO dataset_scope_nodes
             (dataset_id,node_id,node_name,node_type_name,parent_node_id,node_path,
              requested_set_revision,requested_set_digest)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(dataset_id)
        .bind(node.node_id)
        .bind(&node.display_label)
        .bind(&node.node_type_name)
        .bind(node.parent_node_id)
        .bind(&node.node_path)
        .bind(&scope.requested_set_revision)
        .bind(&scope.requested_set_digest)
        .execute(&mut **tx)
        .await?;
    }
    for source in &compiled.sources {
        let source_binding_id = source.source_binding_id.ok_or_else(|| {
            DatasetModuleError::Internal("Compiled Dataset source has no binding identity".into())
        })?;
        let source_identity = serde_json::to_value(&source.reference)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
        let source_identity_digest = digest_json(&source_identity)?;
        let binding_kind = if source.form_version_id.is_some() {
            "response_export"
        } else {
            "dataset_major_line"
        };
        sqlx::query(
            "INSERT INTO dataset_source_bindings
             (id,dataset_id,binding_key,source_kind,source_identity,source_identity_digest)
             VALUES($1,$2,$3,$4,$5,$6)",
        )
        .bind(source_binding_id)
        .bind(dataset_id)
        .bind(&source.source_alias)
        .bind(binding_kind)
        .bind(&source_identity)
        .bind(source_identity_digest)
        .execute(&mut **tx)
        .await?;
        sqlx::query(
            "INSERT INTO dataset_sync_partitions
             (source_binding_id,freshness_state) VALUES($1,$2)",
        )
        .bind(source_binding_id)
        .bind(if binding_kind == "response_export" {
            "never_materialized"
        } else {
            "current"
        })
        .execute(&mut **tx)
        .await?;
        let source_kind = match &source.reference {
            DatasetProductSourceV1::Form { .. } => "form_version",
            DatasetProductSourceV1::Dataset { .. } => "dataset_revision",
            DatasetProductSourceV1::DatasetMajor { .. } => "dataset_major_line",
        };
        sqlx::query(
            "INSERT INTO dataset_sources
             (id,dataset_id,source_binding_id,source_alias,source_kind,source_reference,
              source_name,source_slug,source_version_label,source_scope_node_ids,
              source_scope_revision,source_scope_digest,source_content_revision,
              source_content_digest,position)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)",
        )
        .bind(Uuid::new_v4())
        .bind(dataset_id)
        .bind(source_binding_id)
        .bind(&source.source_alias)
        .bind(source_kind)
        .bind(&source_identity)
        .bind(&source.source_name)
        .bind(&source.source_slug)
        .bind(&source.source_version_label)
        .bind(&source.source_scope_node_ids)
        .bind(&source.source_scope_revision)
        .bind(&source.source_scope_digest)
        .bind(&source.source_content_revision)
        .bind(&source.source_content_digest)
        .bind(source.position)
        .execute(&mut **tx)
        .await?;
    }
    for field in &compiled.fields {
        sqlx::query(
            "INSERT INTO dataset_fields
             (id,dataset_id,key,label,source_alias,source_field_key,source_field_id,field_type,position)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9)",
        )
        .bind(Uuid::new_v4())
        .bind(dataset_id)
        .bind(&field.key)
        .bind(&field.label)
        .bind(&field.source_alias)
        .bind(&field.source_field_key)
        .bind(field.source_field_id)
        .bind(&field.field_type)
        .bind(field.position)
        .execute(&mut **tx)
        .await?;
    }
    let output_fields = compiled
        .fields
        .iter()
        .map(|field| DatasetProductFieldV1 {
            key: field.key.clone(),
            label: field.label.clone(),
            source_alias: field.source_alias.clone(),
            source_field_key: field.source_field_key.clone(),
            field_type: field.field_type.clone(),
            position: field.position,
        })
        .collect::<Vec<_>>();
    let metadata = DatasetProductRevisionMetadataV1 {
        name: payload.name.trim().to_owned(),
        slug: payload.slug.trim().to_owned(),
        grain: payload.grain.clone(),
        visibility_node_ids: payload.visibility_node_ids.clone(),
    };
    sqlx::query(
        "INSERT INTO dataset_revisions
         (id,dataset_id,version_number,version_label,version_major,version_minor,version_patch,
          semantic_bump,started_new_major_line,force_new_major_version,status,published_at,
          initial_source,operations,restriction_policy,definition_metadata,generated_sql,output_fields)
         VALUES($1,$2,1,$3,1,0,0,'initial',TRUE,$4,'published',now(),$5,$6,$7,$8,$9,$10)",
    )
    .bind(revision_id)
    .bind(dataset_id)
    .bind(
        payload
            .version_label
            .as_deref()
            .map(str::trim)
            .unwrap_or_default(),
    )
    .bind(payload.force_new_major_version)
    .bind(serde_json::to_value(&compiled.initial_source).map_err(|error| {
        DatasetModuleError::Internal(error.to_string())
    })?)
    .bind(serde_json::to_value(&compiled.operations).map_err(|error| {
        DatasetModuleError::Internal(error.to_string())
    })?)
    .bind(
        compiled
            .restriction_policy
            .as_ref()
            .map(serde_json::to_value)
            .transpose()
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
    )
    .bind(serde_json::to_value(metadata).map_err(|error| {
        DatasetModuleError::Internal(error.to_string())
    })?)
    .bind(&compiled.generated_sql)
    .bind(serde_json::to_value(output_fields).map_err(|error| {
        DatasetModuleError::Internal(error.to_string())
    })?)
    .execute(&mut **tx)
    .await?;
    replace_revision_snapshots(tx, revision_id, scope, compiled).await?;
    sqlx::query(
        "INSERT INTO dataset_major_materializations
         (dataset_id,version_major,rebuild_status) VALUES($1,1,'pending')",
    )
    .bind(dataset_id)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn replace_revision_snapshots(
    tx: &mut Transaction<'_, Postgres>,
    revision_id: Uuid,
    scope: &ScopeCatalogResponse,
    compiled: &crate::authoring::CompiledDataset,
) -> Result<(), DatasetModuleError> {
    sqlx::query("DELETE FROM dataset_revision_scope_nodes WHERE revision_id=$1")
        .bind(revision_id)
        .execute(&mut **tx)
        .await?;
    sqlx::query("DELETE FROM dataset_revision_sources WHERE revision_id=$1")
        .bind(revision_id)
        .execute(&mut **tx)
        .await?;
    for node in &scope.nodes {
        sqlx::query(
            "INSERT INTO dataset_revision_scope_nodes
             (revision_id,node_id,node_name,node_type_name,parent_node_id,node_path,
              requested_set_revision,requested_set_digest)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8)",
        )
        .bind(revision_id)
        .bind(node.node_id)
        .bind(&node.display_label)
        .bind(&node.node_type_name)
        .bind(node.parent_node_id)
        .bind(&node.node_path)
        .bind(&scope.requested_set_revision)
        .bind(&scope.requested_set_digest)
        .execute(&mut **tx)
        .await?;
    }
    for source in &compiled.sources {
        let source_kind = match &source.reference {
            DatasetProductSourceV1::Form { .. } => "form_version",
            DatasetProductSourceV1::Dataset { .. } => "dataset_revision",
            DatasetProductSourceV1::DatasetMajor { .. } => "dataset_major_line",
        };
        sqlx::query(
            "INSERT INTO dataset_revision_sources
             (revision_id,source_alias,source_kind,source_reference,source_name,source_slug,
              source_version_label,source_scope_node_ids,source_scope_revision,
              source_scope_digest,source_content_revision,source_content_digest,position)
             VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)",
        )
        .bind(revision_id)
        .bind(&source.source_alias)
        .bind(source_kind)
        .bind(
            serde_json::to_value(&source.reference)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        )
        .bind(&source.source_name)
        .bind(&source.source_slug)
        .bind(&source.source_version_label)
        .bind(&source.source_scope_node_ids)
        .bind(&source.source_scope_revision)
        .bind(&source.source_scope_digest)
        .bind(&source.source_content_revision)
        .bind(&source.source_content_digest)
        .bind(source.position)
        .execute(&mut **tx)
        .await?;
    }
    Ok(())
}

async fn preview_existing_dataset_sql(
    State(state): State<DatasetModuleState>,
    Path(dataset_id): Path<Uuid>,
    request: Request,
) -> Result<Json<DatasetSqlPreviewResponseV1>, DatasetModuleError> {
    preview_dataset_sql(state, Some(dataset_id), request).await
}

async fn preview_dataset_sql(
    state: DatasetModuleState,
    dataset_id: Option<Uuid>,
    request: Request,
) -> Result<Json<DatasetSqlPreviewResponseV1>, DatasetModuleError> {
    let headers = request.headers().clone();
    require_json_content_type(&headers)?;
    let body = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| {
            DatasetModuleError::BadRequest("Dataset authoring payload is too large".into())
        })?;
    let action = if dataset_id.is_some() {
        "datasets.preview_existing_sql"
    } else {
        "datasets.preview_sql"
    };
    let grant = authorize_product(
        &state,
        &headers,
        action,
        AuthorizationGrantOperationV1::Read,
        DATASET_AUTHORING_CONTRACT_ID,
    )
    .await?;
    if let Some(dataset_id) = dataset_id {
        let manage_scopes = authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
        if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
            return Err(undisclosed_dataset());
        }
    }
    let payload: DatasetAuthoringRequestV1 = serde_json::from_slice(&body).map_err(|error| {
        DatasetModuleError::BadRequest(format!("Invalid Dataset authoring payload: {error}"))
    })?;
    validate_authoring_payload(&payload)?;
    resolve_authoring_scope(&state, &grant, &payload.visibility_node_ids).await?;
    let compiled =
        crate::authoring::compile_dataset_definition(&state, &grant, dataset_id, &payload)
            .await
            .map_err(DatasetModuleError::from)?;
    Ok(Json(DatasetSqlPreviewResponseV1 {
        generated_sql: compiled.generated_sql,
    }))
}

fn validate_authoring_payload(
    payload: &DatasetAuthoringRequestV1,
) -> Result<(), DatasetModuleError> {
    if payload.name.trim().is_empty() {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset name is required".into(),
        ));
    }
    if payload.slug.trim().is_empty() {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset slug is required".into(),
        ));
    }
    if payload.grain != "submission" {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset query designer currently supports submission grain".into(),
        ));
    }
    if payload
        .version_label
        .as_deref()
        .is_some_and(|label| label.trim().chars().count() > 80)
    {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset revision label must be 80 characters or fewer".into(),
        ));
    }
    if payload.visibility_node_ids.is_empty() {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset visibility requires at least one scope node".into(),
        ));
    }
    let mut nodes = BTreeSet::new();
    for node_id in &payload.visibility_node_ids {
        let parsed = Uuid::parse_str(node_id).map_err(|_| {
            DatasetModuleError::ValidationFailed(
                "Dataset visibility node IDs must be canonical UUIDs".into(),
            )
        })?;
        if parsed.is_nil() || parsed.to_string() != *node_id || !nodes.insert(parsed) {
            return Err(DatasetModuleError::ValidationFailed(
                "Dataset visibility node IDs must be unique canonical non-nil UUIDs".into(),
            ));
        }
    }
    Ok(())
}

async fn resolve_authoring_scope(
    state: &DatasetModuleState,
    grant: &dyn crate::provider_client::ProviderAuthorization,
    node_ids: &[String],
) -> Result<ScopeCatalogResponse, DatasetModuleError> {
    let requested = node_ids
        .iter()
        .map(|id| Uuid::parse_str(id).map_err(|_| DatasetModuleError::Forbidden))
        .collect::<Result<Vec<_>, _>>()?;
    let response: ScopeCatalogResponse = crate::provider_client::post(
        state,
        grant,
        SCOPE_CATALOG_PROVIDER,
        &ScopeCatalogRequest {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            action: ControlPlaneCatalogAction::ResolveRequestedSet,
            node_ids: requested.clone(),
        },
    )
    .await?;
    let returned = response
        .nodes
        .iter()
        .map(|node| node.node_id)
        .collect::<BTreeSet<_>>();
    if returned != requested.into_iter().collect() {
        return Err(DatasetModuleError::Unavailable(
            "Scope provider returned a substituted requested set".into(),
        ));
    }
    Ok(response)
}

async fn editor_scope_options(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Response, DatasetModuleError> {
    let grant = authorize_product(
        &state,
        &headers,
        "datasets.editor_scopes",
        AuthorizationGrantOperationV1::Read,
        DATASET_AUTHORING_CONTRACT_ID,
    )
    .await?;
    let (options, digest) = editor_scope_option_projection(&state, &grant).await?;
    json_etag_response(&options, &digest)
}

pub(crate) async fn editor_scope_option_projection(
    state: &DatasetModuleState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV3>,
) -> Result<(Vec<DatasetEditorScopeOptionV1>, String), DatasetModuleError> {
    let response: ScopeCatalogResponse = crate::provider_client::post(
        state,
        grant,
        SCOPE_CATALOG_PROVIDER,
        &ScopeCatalogRequest {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            action: ControlPlaneCatalogAction::Catalog,
            node_ids: Vec::new(),
        },
    )
    .await?;
    let names = response
        .nodes
        .iter()
        .map(|node| (node.node_id, node.display_label.clone()))
        .collect::<std::collections::BTreeMap<_, _>>();
    let options = response
        .nodes
        .into_iter()
        .map(|node| DatasetEditorScopeOptionV1 {
            id: node.node_id.to_string(),
            node_type_name: node.node_type_name,
            parent_node_id: node.parent_node_id.map(|id| id.to_string()),
            parent_node_name: node
                .parent_node_id
                .and_then(|parent_id| names.get(&parent_id).cloned()),
            name: node.display_label,
        })
        .collect::<Vec<_>>();
    Ok((options, response.requested_set_digest))
}

async fn editor_principal_options(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Response, DatasetModuleError> {
    let grant = authorize_product(
        &state,
        &headers,
        "datasets.editor_principals",
        AuthorizationGrantOperationV1::Read,
        DATASET_AUTHORING_CONTRACT_ID,
    )
    .await?;
    let (options, digest) = editor_principal_option_projection(&state, &grant).await?;
    json_etag_response(&options, &digest)
}

pub(crate) async fn editor_principal_option_projection(
    state: &DatasetModuleState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV3>,
) -> Result<(Vec<DatasetEditorPrincipalOptionV1>, String), DatasetModuleError> {
    let response: PrincipalDisplayCatalogResponse = crate::provider_client::post(
        state,
        grant,
        PRINCIPAL_CATALOG_PROVIDER,
        &PrincipalDisplayCatalogRequest {
            schema_version: CONTROL_PLANE_SCHEMA_VERSION,
            action: ControlPlaneCatalogAction::Catalog,
            principal_ids: Vec::new(),
        },
    )
    .await?;
    let options = response
        .principals
        .into_iter()
        .map(|principal| DatasetEditorPrincipalOptionV1 {
            display_name: principal.display_name,
        })
        .collect::<Vec<_>>();
    Ok((options, response.requested_set_digest))
}

async fn editor_form_options(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Response, DatasetModuleError> {
    let grant = authorize_product(
        &state,
        &headers,
        "datasets.editor_forms",
        AuthorizationGrantOperationV1::Read,
        DATASET_AUTHORING_CONTRACT_ID,
    )
    .await?;
    let (forms, digest) = editor_form_option_projection(&state, &grant).await?;
    json_etag_response(&forms, &digest)
}

pub(crate) async fn editor_form_option_projection(
    state: &DatasetModuleState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV3>,
) -> Result<(Vec<DatasetEditorFormOptionV1>, String), DatasetModuleError> {
    let mut cursor = None;
    let mut seen_cursors = BTreeSet::new();
    let mut catalog_items = Vec::new();
    let requested_set_digest = loop {
        let response: FormVersionCatalogResponse = crate::provider_client::post(
            state,
            grant,
            FORM_CATALOG_PROVIDER,
            &FormVersionCatalogRequest {
                schema_version: FORM_VERSION_SCHEMA_VERSION,
                action: FormVersionSchemaAction::Catalog,
                cursor: cursor.clone(),
                limit: 100,
            },
        )
        .await?;
        catalog_items.extend(response.items);
        let Some(next) = response.next_cursor else {
            break response.requested_set_digest;
        };
        if !seen_cursors.insert(next.clone()) {
            return Err(DatasetModuleError::Unavailable(
                "FormVersion catalog pagination is invalid".into(),
            ));
        }
        cursor = Some(next);
    };
    let mut forms = Vec::<DatasetEditorFormOptionV1>::new();
    for item in catalog_items {
        if let Some(form) = forms
            .iter_mut()
            .find(|form| form.id == item.form_id.to_string())
        {
            form.versions.push(DatasetEditorFormVersionOptionV1 {
                id: item.form_version_id.to_string(),
                version_label: item.version_label,
                status: "published".into(),
                version_major: item.version_major,
                field_count: item.field_count,
            });
        } else {
            forms.push(DatasetEditorFormOptionV1 {
                id: item.form_id.to_string(),
                name: item.form_name,
                versions: vec![DatasetEditorFormVersionOptionV1 {
                    id: item.form_version_id.to_string(),
                    version_label: item.version_label,
                    status: "published".into(),
                    version_major: item.version_major,
                    field_count: item.field_count,
                }],
            });
        }
    }
    forms.sort_by(|left, right| left.name.cmp(&right.name).then(left.id.cmp(&right.id)));
    Ok((forms, requested_set_digest))
}

async fn editor_form_schema(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(form_version_id): Path<Uuid>,
) -> Result<Response, DatasetModuleError> {
    let grant = authorize_product(
        &state,
        &headers,
        "datasets.editor_form_schema",
        AuthorizationGrantOperationV1::Read,
        DATASET_AUTHORING_CONTRACT_ID,
    )
    .await?;
    let (rendered, digest) = editor_form_schema_projection(&state, &grant, form_version_id).await?;
    json_etag_response(&rendered, &digest)
}

pub(crate) async fn editor_form_schema_projection(
    state: &DatasetModuleState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV3>,
    form_version_id: Uuid,
) -> Result<(DatasetEditorRenderedFormV1, String), DatasetModuleError> {
    let schema: FormVersionSchemaResponse = crate::provider_client::post(
        state,
        grant,
        FORM_SCHEMA_PROVIDER,
        &FormVersionSchemaRequest {
            schema_version: FORM_VERSION_SCHEMA_VERSION,
            action: FormVersionSchemaAction::ResolveSchema,
            form_version_id,
        },
    )
    .await?;
    if schema.form_version_id != form_version_id {
        return Err(DatasetModuleError::Unavailable(
            "FormVersion schema identity is invalid".into(),
        ));
    }
    let rendered = DatasetEditorRenderedFormV1 {
        form_version_id: schema.form_version_id.to_string(),
        form_id: schema.form_id.to_string(),
        form_name: schema.form_name,
        sections: schema
            .sections
            .iter()
            .map(|section| DatasetEditorRenderedSectionV1 {
                fields: schema
                    .fields
                    .iter()
                    .filter(|field| field.section_id == Some(section.section_id))
                    .map(|field| DatasetEditorRenderedFieldV1 {
                        key: field.key.clone(),
                        label: field.label.clone(),
                        field_type: field.field_type.clone(),
                        value_options: field
                            .options
                            .iter()
                            .filter_map(option_display_value)
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
    };
    Ok((rendered, schema.content_digest))
}

fn option_display_value(value: &serde_json::Value) -> Option<String> {
    value.as_str().map(str::to_owned).or_else(|| {
        value
            .get("value")
            .and_then(serde_json::Value::as_str)
            .map(str::to_owned)
    })
}

fn json_etag_response<T: serde::Serialize>(
    value: &T,
    digest: &str,
) -> Result<Response, DatasetModuleError> {
    let body = serde_json::to_vec(value)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::ETAG, format!("\"{digest}\""))
        .body(Body::from(body))
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))
}

async fn list_datasets(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<DatasetProductSummaryV1>>, DatasetModuleError> {
    let grant = authorize_read(&state, &headers, "datasets.list").await?;
    Ok(Json(list_dataset_summaries(&state, &grant.payload).await?))
}

pub(crate) async fn list_dataset_summaries(
    state: &DatasetModuleState,
    grant: &AuthorizationGrantV3,
) -> Result<Vec<DatasetProductSummaryV1>, DatasetModuleError> {
    let scopes = readable_organizations(grant);
    if scopes.is_empty() {
        return Err(DatasetModuleError::Forbidden);
    }

    let rows = sqlx::query(
        "SELECT DISTINCT d.id,d.name,d.slug,d.grain,
                r.id AS current_revision_id,r.version_major,
                r.version_minor,r.version_patch,r.materialized_row_count,r.materialized_at
         FROM datasets d
         JOIN dataset_scope_nodes s ON s.dataset_id=d.id
         LEFT JOIN dataset_revisions r ON r.dataset_id=d.id AND r.status='published'
         WHERE d.lifecycle_state <> 'tombstoned' AND s.node_id = ANY($1)
         ORDER BY d.name,d.id",
    )
    .bind(scopes.into_iter().collect::<Vec<_>>())
    .fetch_all(&state.pool)
    .await?;

    let mut datasets = Vec::with_capacity(rows.len());
    for row in rows {
        let dataset_id: Uuid = row.try_get("id")?;
        let tags = sqlx::query_scalar::<_, String>(
            "SELECT tag FROM dataset_tags WHERE dataset_id=$1 ORDER BY tag",
        )
        .bind(dataset_id)
        .fetch_all(&state.pool)
        .await?;
        let major_versions = sqlx::query_scalar::<_, i32>(
            "SELECT DISTINCT version_major FROM dataset_revisions
             WHERE dataset_id=$1 AND version_major IS NOT NULL ORDER BY version_major",
        )
        .bind(dataset_id)
        .fetch_all(&state.pool)
        .await?;
        let revision_rows = sqlx::query(
            "SELECT id,version_number,version_major,version_minor,version_patch,output_fields
             FROM dataset_revisions
             WHERE dataset_id=$1 AND status IN ('published','superseded')
             ORDER BY version_number,id",
        )
        .bind(dataset_id)
        .fetch_all(&state.pool)
        .await?;
        let revisions = revision_rows
            .into_iter()
            .map(|revision| {
                let output_fields = serde_json::from_value::<Vec<DatasetProductFieldV1>>(
                    revision.try_get("output_fields")?,
                )
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
                Ok(DatasetProductRevisionFieldSummaryV1 {
                    id: revision.try_get::<Uuid, _>("id")?.to_string(),
                    version_number: revision.try_get("version_number")?,
                    version_major: revision.try_get("version_major")?,
                    version_minor: revision.try_get("version_minor")?,
                    version_patch: revision.try_get("version_patch")?,
                    output_fields,
                })
            })
            .collect::<Result<Vec<_>, DatasetModuleError>>()?;
        let scope_rows = sqlx::query(
            "SELECT node_id,node_name,node_type_name,parent_node_id,node_path
             FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_path,node_id",
        )
        .bind(dataset_id)
        .fetch_all(&state.pool)
        .await?;
        let visibility_nodes = scope_rows
            .into_iter()
            .map(|scope| {
                Ok(DatasetProductVisibilityNodeV1 {
                    node_id: scope.try_get::<Uuid, _>("node_id")?.to_string(),
                    node_name: scope.try_get("node_name")?,
                    node_type_name: scope.try_get("node_type_name")?,
                    parent_node_id: scope
                        .try_get::<Option<Uuid>, _>("parent_node_id")?
                        .map(|id| id.to_string()),
                    node_path: scope.try_get("node_path")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        let field_rows = sqlx::query(
            "SELECT key,label,source_alias,source_field_key,field_type,position
             FROM dataset_fields WHERE dataset_id=$1 ORDER BY position,key",
        )
        .bind(dataset_id)
        .fetch_all(&state.pool)
        .await?;
        let output_fields = field_rows
            .into_iter()
            .map(|field| {
                Ok(DatasetProductFieldV1 {
                    key: field.try_get("key")?,
                    label: field.try_get("label")?,
                    source_alias: field.try_get("source_alias")?,
                    source_field_key: field.try_get("source_field_key")?,
                    field_type: field.try_get("field_type")?,
                    position: field.try_get("position")?,
                })
            })
            .collect::<Result<Vec<_>, sqlx::Error>>()?;
        let source_count = sqlx::query_scalar::<_, i64>(
            "SELECT count(*) FROM dataset_sources WHERE dataset_id=$1",
        )
        .bind(dataset_id)
        .fetch_one(&state.pool)
        .await?;
        let provenance = load_dataset_provenance(state, dataset_id).await?;
        let field_count = output_fields.len() as i64;
        let current_revision_id = row.try_get::<Option<Uuid>, _>("current_revision_id")?;
        let version_major = row.try_get::<Option<i32>, _>("version_major")?;
        let version_minor = row.try_get::<Option<i32>, _>("version_minor")?;
        let version_patch = row.try_get::<Option<i32>, _>("version_patch")?;
        let materialized_at: Option<chrono::DateTime<Utc>> = row.try_get("materialized_at")?;
        let freshness = load_dataset_freshness(state, dataset_id, materialized_at).await?;
        datasets.push(DatasetProductSummaryV1 {
            id: dataset_id.to_string(),
            current_revision_id: current_revision_id.map(|id| id.to_string()),
            current_version_major: version_major,
            current_version_minor: version_minor,
            current_version_patch: version_patch,
            major_versions,
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
            grain: row.try_get("grain")?,
            tags,
            provenance,
            materialized_row_count: row.try_get("materialized_row_count")?,
            materialized_at: materialized_at.map(|value| value.to_rfc3339()),
            freshness,
            visibility_nodes,
            source_count,
            field_count,
            output_fields,
            revisions,
        });
    }
    Ok(datasets)
}

async fn load_dataset_provenance(
    state: &DatasetModuleState,
    dataset_id: Uuid,
) -> Result<DatasetProductProvenanceSummaryV1, DatasetModuleError> {
    let source_rows = sqlx::query(
        "SELECT source_reference,source_name,source_slug
         FROM dataset_sources WHERE dataset_id=$1 ORDER BY position,source_alias",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let mut provenance = DatasetProductProvenanceSummaryV1::default();
    for source_row in source_rows {
        let reference: DatasetProductSourceV1 =
            serde_json::from_value(source_row.try_get("source_reference")?)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
        append_source_provenance(
            &mut provenance,
            &reference,
            source_row.try_get("source_name")?,
            source_row.try_get("source_slug")?,
        );
    }
    Ok(provenance)
}

fn append_source_provenance(
    provenance: &mut DatasetProductProvenanceSummaryV1,
    reference: &DatasetProductSourceV1,
    source_name: String,
    source_slug: Option<String>,
) {
    let (items, id) = match reference {
        DatasetProductSourceV1::Form { form_id, .. } => (&mut provenance.forms, form_id),
        DatasetProductSourceV1::Dataset { dataset_id, .. }
        | DatasetProductSourceV1::DatasetMajor { dataset_id, .. } => {
            (&mut provenance.datasets, dataset_id)
        }
    };
    items.push(DatasetProductProvenanceItemV1 {
        id: id.clone(),
        name: source_name,
        slug: source_slug,
    });
}

async fn get_dataset(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<DatasetProductDefinitionV1>, DatasetModuleError> {
    let grant = authorize_read(&state, &headers, "datasets.get").await?;
    Ok(Json(
        dataset_definition(&state, &grant.payload, dataset_id).await?,
    ))
}

pub(crate) async fn dataset_definition(
    state: &DatasetModuleState,
    grant: &AuthorizationGrantV3,
    dataset_id: Uuid,
) -> Result<DatasetProductDefinitionV1, DatasetModuleError> {
    let scopes = readable_organizations(grant);
    if scopes.is_empty() {
        return Err(undisclosed_dataset());
    }
    let row = sqlx::query(
        "SELECT d.id,d.name,d.slug,d.grain,
                r.id AS revision_id,r.version_number,r.version_label,r.version_major,
                r.version_minor,r.version_patch,r.initial_source,r.operations,
                r.restriction_policy,r.generated_sql,r.materialized_schema,
                r.materialized_table,r.materialized_row_count,r.materialized_at
         FROM datasets d
         JOIN dataset_revisions r ON r.dataset_id=d.id AND r.status='published'
         WHERE d.id=$1 AND d.lifecycle_state <> 'tombstoned'
           AND EXISTS (
             SELECT 1 FROM dataset_scope_nodes s
             WHERE s.dataset_id=d.id AND s.node_id=ANY($2)
           )",
    )
    .bind(dataset_id)
    .bind(scopes.into_iter().collect::<Vec<_>>())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(undisclosed_dataset)?;

    let tags = sqlx::query_scalar::<_, String>(
        "SELECT tag FROM dataset_tags WHERE dataset_id=$1 ORDER BY tag",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let scope_rows = sqlx::query(
        "SELECT node_id,node_name,node_type_name,parent_node_id,node_path
         FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_path,node_id",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let visibility_nodes = scope_rows
        .into_iter()
        .map(|scope| {
            Ok(DatasetProductVisibilityNodeV1 {
                node_id: scope.try_get::<Uuid, _>("node_id")?.to_string(),
                node_name: scope.try_get("node_name")?,
                node_type_name: scope.try_get("node_type_name")?,
                parent_node_id: scope
                    .try_get::<Option<Uuid>, _>("parent_node_id")?
                    .map(|id| id.to_string()),
                node_path: scope.try_get("node_path")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let field_rows = sqlx::query(
        "SELECT key,label,source_alias,source_field_key,field_type,position
         FROM dataset_fields WHERE dataset_id=$1 ORDER BY position,key",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let fields = field_rows
        .into_iter()
        .map(|field| {
            Ok(DatasetProductFieldV1 {
                key: field.try_get("key")?,
                label: field.try_get("label")?,
                source_alias: field.try_get("source_alias")?,
                source_field_key: field.try_get("source_field_key")?,
                field_type: field.try_get("field_type")?,
                position: field.try_get("position")?,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let source_rows = sqlx::query(
        "SELECT source_alias,source_reference,source_name,source_slug,
                source_version_label,position
         FROM dataset_sources WHERE dataset_id=$1 ORDER BY position,source_alias",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let mut sources = Vec::with_capacity(source_rows.len());
    let mut provenance = DatasetProductProvenanceSummaryV1::default();
    let mut lineage_children = Vec::with_capacity(source_rows.len());
    for source_row in source_rows {
        let reference: DatasetProductSourceV1 =
            serde_json::from_value(source_row.try_get("source_reference")?)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
        let source_name: String = source_row.try_get("source_name")?;
        let source_slug: Option<String> = source_row.try_get("source_slug")?;
        let source_version_label: Option<String> = source_row.try_get("source_version_label")?;
        let position: i32 = source_row.try_get("position")?;
        append_source_provenance(
            &mut provenance,
            &reference,
            source_name.clone(),
            source_slug.clone(),
        );
        let (definition, lineage) = match reference {
            DatasetProductSourceV1::Form {
                alias,
                form_id,
                form_version_id,
            } => (
                DatasetProductSourceDefinitionV1 {
                    source_alias: alias,
                    form_id: Some(form_id.clone()),
                    form_name: Some(source_name.clone()),
                    form_version_id: Some(form_version_id),
                    form_version_label: source_version_label.clone(),
                    source_dataset_id: None,
                    source_dataset_name: None,
                    source_dataset_slug: None,
                    dataset_revision_id: None,
                    dataset_revision_label: None,
                    dataset_version_major: None,
                    position,
                },
                DatasetProductLineageNodeV1 {
                    id: form_id,
                    name: source_name.clone(),
                    slug: source_slug.clone(),
                    source_type: "form".into(),
                    version_label: source_version_label.clone(),
                    children: Vec::new(),
                },
            ),
            DatasetProductSourceV1::Dataset {
                alias,
                dataset_id: source_dataset_id,
                dataset_revision_id,
            } => (
                DatasetProductSourceDefinitionV1 {
                    source_alias: alias,
                    form_id: None,
                    form_name: None,
                    form_version_id: None,
                    form_version_label: None,
                    source_dataset_id: Some(source_dataset_id.clone()),
                    source_dataset_name: Some(source_name.clone()),
                    source_dataset_slug: source_slug.clone(),
                    dataset_revision_id: Some(dataset_revision_id),
                    dataset_revision_label: source_version_label.clone(),
                    dataset_version_major: None,
                    position,
                },
                DatasetProductLineageNodeV1 {
                    id: source_dataset_id,
                    name: source_name.clone(),
                    slug: source_slug.clone(),
                    source_type: "dataset".into(),
                    version_label: source_version_label.clone(),
                    children: Vec::new(),
                },
            ),
            DatasetProductSourceV1::DatasetMajor {
                alias,
                dataset_id: source_dataset_id,
                version_major,
            } => (
                DatasetProductSourceDefinitionV1 {
                    source_alias: alias,
                    form_id: None,
                    form_name: None,
                    form_version_id: None,
                    form_version_label: None,
                    source_dataset_id: Some(source_dataset_id.clone()),
                    source_dataset_name: Some(source_name.clone()),
                    source_dataset_slug: source_slug.clone(),
                    dataset_revision_id: None,
                    dataset_revision_label: None,
                    dataset_version_major: Some(version_major),
                    position,
                },
                DatasetProductLineageNodeV1 {
                    id: source_dataset_id,
                    name: source_name.clone(),
                    slug: source_slug.clone(),
                    source_type: "dataset".into(),
                    version_label: Some(format!("v{version_major}")),
                    children: Vec::new(),
                },
            ),
        };
        sources.push(definition);
        lineage_children.push(lineage);
    }

    let revision_id: Uuid = row.try_get("revision_id")?;
    let version_label: String = row.try_get("version_label")?;
    let name: String = row.try_get("name")?;
    let slug: String = row.try_get("slug")?;
    let initial_source = row
        .try_get::<Option<serde_json::Value>, _>("initial_source")?
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let operations: Vec<DatasetProductOperationV1> =
        serde_json::from_value(row.try_get("operations")?)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let restriction_policy: Option<DatasetProductRestrictionPolicyV1> = row
        .try_get::<Option<serde_json::Value>, _>("restriction_policy")?
        .map(serde_json::from_value)
        .transpose()
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let materialized_at: Option<chrono::DateTime<Utc>> = row.try_get("materialized_at")?;
    let freshness = load_dataset_freshness(state, dataset_id, materialized_at).await?;
    Ok(DatasetProductDefinitionV1 {
        id: dataset_id.to_string(),
        current_revision_id: Some(revision_id.to_string()),
        current_revision_number: Some(row.try_get("version_number")?),
        current_revision_label: Some(version_label.clone()),
        current_version_major: row.try_get("version_major")?,
        current_version_minor: row.try_get("version_minor")?,
        current_version_patch: row.try_get("version_patch")?,
        name: name.clone(),
        slug: slug.clone(),
        grain: row.try_get("grain")?,
        tags,
        provenance,
        lineage: DatasetProductLineageNodeV1 {
            id: dataset_id.to_string(),
            name,
            slug: Some(slug),
            source_type: "dataset".into(),
            version_label: Some(version_label),
            children: lineage_children,
        },
        initial_source,
        operations,
        restriction_policy,
        generated_sql: row.try_get("generated_sql")?,
        materialized_schema: row.try_get("materialized_schema")?,
        materialized_table: row.try_get("materialized_table")?,
        materialized_row_count: row.try_get("materialized_row_count")?,
        materialized_at: materialized_at.map(|value| value.to_rfc3339()),
        freshness,
        visibility_nodes,
        sources,
        fields: fields.clone(),
        output_fields: fields,
    })
}

async fn run_dataset_table(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<DatasetProductTableV1>, DatasetModuleError> {
    let grant = authorize_read(&state, &headers, "datasets.preview_table").await?;
    Ok(Json(
        dataset_table(&state, &grant.payload, dataset_id).await?,
    ))
}

pub(crate) async fn dataset_table(
    state: &DatasetModuleState,
    grant: &AuthorizationGrantV3,
    dataset_id: Uuid,
) -> Result<DatasetProductTableV1, DatasetModuleError> {
    let read_scopes = readable_organizations(grant);
    require_visible_dataset(state, dataset_id, &read_scopes).await?;
    let materialization = sqlx::query(
        "SELECT materialized_schema,materialized_table
         FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published'
           AND lifecycle_state <> 'tombstoned'
           AND materialized_schema IS NOT NULL AND materialized_table IS NOT NULL",
    )
    .bind(dataset_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        DatasetModuleError::Conflict(
            "Dataset preview requires a ready published materialization".into(),
        )
    })?;
    let schema: String = materialization.try_get("materialized_schema")?;
    let table: String = materialization.try_get("materialized_table")?;
    let mut materialized_query = QueryBuilder::<Postgres>::new("SELECT * FROM ");
    materialized_query
        .push(quote_dataset_identifier(&schema))
        .push(".")
        .push(quote_dataset_identifier(&table))
        .push(" WHERE ");
    crate::provider::push_row_access_predicate(&mut materialized_query, grant);
    materialized_query.push(" ORDER BY __row_id LIMIT 200");
    let rows = materialized_query.build().fetch_all(&state.pool).await?;
    let rows = rows
        .into_iter()
        .map(|row| {
            let submission_id: String = row.try_get("__row_id")?;
            let mut values = std::collections::BTreeMap::new();
            for column in row.columns() {
                let name = column.name();
                if name.starts_with("__") {
                    continue;
                }
                values.insert(name.to_owned(), row.try_get::<Option<String>, _>(name)?);
            }
            Ok(DatasetProductTableRowV1 {
                submission_id,
                node_name: String::new(),
                source_alias: "materialized".into(),
                values,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    Ok(DatasetProductTableV1 { rows })
}

async fn list_dataset_distinct_values(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
    Query(query): Query<DatasetProductDistinctValuesQueryV1>,
) -> Result<Json<DatasetProductDistinctValuesV1>, DatasetModuleError> {
    let grant = authorize_read(&state, &headers, "datasets.distinct_values").await?;
    let read_scopes = readable_organizations(&grant.payload);
    require_visible_dataset(&state, dataset_id, &read_scopes).await?;
    if query.version_major < 1 {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset version major must be at least 1".into(),
        ));
    }
    let field = query.field.trim();
    if !is_dataset_identifier(field) {
        return Err(DatasetModuleError::ValidationFailed(
            "Dataset field must be a canonical identifier".into(),
        ));
    }
    let row = sqlx::query(
        "SELECT m.materialized_schema,m.materialized_table,
                (SELECT output_fields FROM dataset_revisions r
                 WHERE r.dataset_id=m.dataset_id AND r.version_major=m.version_major
                   AND r.status IN ('published','superseded')
                 ORDER BY r.version_minor DESC,r.version_patch DESC,r.version_number DESC
                 LIMIT 1) AS output_fields
         FROM dataset_major_materializations m
         WHERE m.dataset_id=$1 AND m.version_major=$2 AND m.rebuild_status='ready'
           AND m.materialized_schema IS NOT NULL AND m.materialized_table IS NOT NULL",
    )
    .bind(dataset_id)
    .bind(query.version_major)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| {
        DatasetModuleError::Conflict(format!(
            "Dataset major version {} is not ready",
            query.version_major
        ))
    })?;
    let output_fields: Vec<DatasetProductFieldV1> =
        parse_json(row.try_get("output_fields")?, "Dataset major output fields")?;
    if !output_fields.iter().any(|candidate| candidate.key == field) {
        return Err(DatasetModuleError::ValidationFailed(format!(
            "Dataset field '{field}' is not in major version {}",
            query.version_major
        )));
    }
    let schema: String = row.try_get("materialized_schema")?;
    let table: String = row.try_get("materialized_table")?;
    let quoted_field = quote_dataset_identifier(field);
    let mut distinct_query = QueryBuilder::<Postgres>::new("SELECT DISTINCT BTRIM(");
    distinct_query
        .push(&quoted_field)
        .push("::text) FROM ")
        .push(quote_dataset_identifier(&schema))
        .push(".")
        .push(quote_dataset_identifier(&table))
        .push(" WHERE ");
    crate::provider::push_row_access_predicate(&mut distinct_query, &grant.payload);
    distinct_query
        .push(" AND ")
        .push(&quoted_field)
        .push(" IS NOT NULL AND BTRIM(")
        .push(&quoted_field)
        .push("::text) <> '' ORDER BY 1");
    let values = distinct_query
        .build_query_scalar::<String>()
        .fetch_all(&state.pool)
        .await?;
    Ok(Json(DatasetProductDistinctValuesV1 {
        dataset_id: dataset_id.to_string(),
        version_major: query.version_major,
        field: field.to_owned(),
        values,
    }))
}

async fn load_dataset_freshness(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    materialized_at: Option<chrono::DateTime<Utc>>,
) -> Result<DatasetProductFreshnessV1, DatasetModuleError> {
    let rows = sqlx::query_as::<
        _,
        (
            String,
            Option<chrono::DateTime<Utc>>,
            Option<chrono::DateTime<Utc>>,
            Option<String>,
        ),
    >(
        "SELECT p.freshness_state,p.last_checked_at,p.last_succeeded_at,
                 p.sanitized_failure_code
         FROM dataset_source_bindings b
         JOIN dataset_sync_partitions p ON p.source_binding_id=b.id
         WHERE b.dataset_id=$1 ORDER BY b.binding_key",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    Ok(compose_dataset_freshness(rows, materialized_at))
}

async fn load_dataset_freshness_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
) -> Result<DatasetProductFreshnessV1, DatasetModuleError> {
    let materialized_at = sqlx::query_scalar::<_, Option<chrono::DateTime<Utc>>>(
        "SELECT materialized_at FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' AND lifecycle_state <> 'tombstoned'",
    )
    .bind(dataset_id)
    .fetch_optional(&mut **transaction)
    .await?
    .flatten();
    let rows = sqlx::query_as::<
        _,
        (
            String,
            Option<chrono::DateTime<Utc>>,
            Option<chrono::DateTime<Utc>>,
            Option<String>,
        ),
    >(
        "SELECT p.freshness_state,p.last_checked_at,p.last_succeeded_at,
                p.sanitized_failure_code
         FROM dataset_source_bindings b
         JOIN dataset_sync_partitions p ON p.source_binding_id=b.id
         WHERE b.dataset_id=$1 ORDER BY b.binding_key",
    )
    .bind(dataset_id)
    .fetch_all(&mut **transaction)
    .await?;
    Ok(compose_dataset_freshness(rows, materialized_at))
}

type DatasetFreshnessRow = (
    String,
    Option<chrono::DateTime<Utc>>,
    Option<chrono::DateTime<Utc>>,
    Option<String>,
);

fn compose_dataset_freshness(
    rows: Vec<DatasetFreshnessRow>,
    materialized_at: Option<chrono::DateTime<Utc>>,
) -> DatasetProductFreshnessV1 {
    let mut states = BTreeSet::new();
    let mut last_checked_at = None;
    let mut last_succeeded_at = materialized_at;
    let mut failure_code = None;
    for (freshness_state, checked, succeeded, sanitized_failure_code) in rows {
        states.insert(freshness_state);
        if checked > last_checked_at {
            last_checked_at = checked;
        }
        if succeeded > last_succeeded_at {
            last_succeeded_at = succeeded;
        }
        if failure_code.is_none() {
            failure_code = sanitized_failure_code;
        }
    }
    let state = if materialized_at.is_none() {
        if states.contains("failed") || states.contains("degraded") {
            DatasetFreshnessState::Failed
        } else if states.contains("refreshing") {
            DatasetFreshnessState::Refreshing
        } else {
            DatasetFreshnessState::NeverMaterialized
        }
    } else if states.contains("degraded") || states.contains("failed") {
        DatasetFreshnessState::Degraded
    } else if states.contains("refreshing") {
        DatasetFreshnessState::Refreshing
    } else if states.contains("stale") || states.contains("never_materialized") {
        DatasetFreshnessState::Stale
    } else {
        DatasetFreshnessState::Current
    };
    DatasetProductFreshnessV1 {
        state,
        last_checked_at: last_checked_at.map(|value| value.to_rfc3339()),
        last_succeeded_at: last_succeeded_at.map(|value| value.to_rfc3339()),
        sanitized_failure_code: failure_code,
    }
}

fn is_dataset_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    characters
        .next()
        .is_some_and(|character| character.is_ascii_lowercase() || character == '_')
        && characters.all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

fn quote_dataset_identifier(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

async fn list_dataset_revisions(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path(dataset_id): Path<Uuid>,
) -> Result<Json<Vec<DatasetProductRevisionSummaryV1>>, DatasetModuleError> {
    let grant = authorize_read(&state, &headers, "datasets.list_revisions").await?;
    Ok(Json(
        dataset_revision_summaries(&state, &grant.payload, dataset_id).await?,
    ))
}

pub(crate) async fn dataset_revision_summaries(
    state: &DatasetModuleState,
    grant: &AuthorizationGrantV3,
    dataset_id: Uuid,
) -> Result<Vec<DatasetProductRevisionSummaryV1>, DatasetModuleError> {
    let read_scopes = readable_organizations(grant);
    require_visible_dataset(state, dataset_id, &read_scopes).await?;
    let can_manage = dataset_fully_in_scope(
        state,
        dataset_id,
        &authorized_organizations(grant, MANAGE_CAPABILITY),
    )
    .await?;
    let rows = sqlx::query(
        "SELECT id,dataset_id,version_number,version_label,version_major,version_minor,
                version_patch,semantic_bump,started_new_major_line,force_new_major_version,
                status,created_at,published_at,materialized_at,materialized_row_count,
                compatibility_findings,output_fields
         FROM dataset_revisions
         WHERE dataset_id=$1 AND lifecycle_state <> 'tombstoned'
           AND ($2 OR status <> 'draft')
         ORDER BY version_number DESC",
    )
    .bind(dataset_id)
    .bind(can_manage)
    .fetch_all(&state.pool)
    .await?;
    let current = sqlx::query(
        "SELECT id,version_major FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' AND lifecycle_state <> 'tombstoned'",
    )
    .bind(dataset_id)
    .fetch_optional(&state.pool)
    .await?;
    let current_revision_id = current.as_ref().map(|row| row.get::<Uuid, _>("id"));
    let current_major = current
        .as_ref()
        .and_then(|row| row.get::<Option<i32>, _>("version_major"));
    let mut revisions = Vec::with_capacity(rows.len());
    for row in rows {
        let status = revision_status(row.try_get("status")?)?;
        let findings = revision_findings(row.try_get("compatibility_findings")?)?;
        let compatibility = crate::compatibility::compatibility_summary(&findings);
        let revision_id: Uuid = row.try_get("id")?;
        let revision_major: Option<i32> = row.try_get("version_major")?;
        let (dependency_revision_id, dependency_major) =
            if status == DatasetProductRevisionStatusV1::Draft {
                (current_revision_id, current_major)
            } else {
                (Some(revision_id), revision_major)
            };
        let (_, dependencies) = load_dataset_dependencies(
            state,
            dataset_id,
            dependency_revision_id,
            dependency_major,
            &read_scopes,
            compatibility.state,
        )
        .await?;
        let output_fields: Vec<DatasetProductFieldV1> =
            parse_json(row.try_get("output_fields")?, "revision output fields")?;
        revisions.push(DatasetProductRevisionSummaryV1 {
            id: revision_id.to_string(),
            dataset_id: dataset_id.to_string(),
            version_number: row.try_get("version_number")?,
            version_label: row.try_get("version_label")?,
            version_major: revision_major,
            version_minor: row.try_get("version_minor")?,
            version_patch: row.try_get("version_patch")?,
            semantic_bump: semantic_bump(row.try_get("semantic_bump")?)?,
            started_new_major_line: row.try_get("started_new_major_line")?,
            force_new_major_version: row.try_get("force_new_major_version")?,
            status,
            is_current: current_revision_id == Some(revision_id),
            created_at: row
                .try_get::<chrono::DateTime<Utc>, _>("created_at")?
                .to_rfc3339(),
            published_at: optional_time(&row, "published_at")?,
            materialized_at: optional_time(&row, "materialized_at")?,
            materialized_row_count: row.try_get("materialized_row_count")?,
            output_field_count: output_fields.len(),
            compatibility,
            dependencies,
        });
    }
    Ok(revisions)
}

async fn get_dataset_revision(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    Path((dataset_id, revision_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<DatasetProductRevisionDetailV1>, DatasetModuleError> {
    let grant = authorize_read(&state, &headers, "datasets.get_revision").await?;
    Ok(Json(
        dataset_revision_detail(&state, &grant.payload, dataset_id, revision_id).await?,
    ))
}

pub(crate) async fn dataset_revision_detail(
    state: &DatasetModuleState,
    grant: &AuthorizationGrantV3,
    dataset_id: Uuid,
    revision_id: Uuid,
) -> Result<DatasetProductRevisionDetailV1, DatasetModuleError> {
    let read_scopes = readable_organizations(grant);
    require_visible_dataset(state, dataset_id, &read_scopes).await?;
    let can_manage = dataset_fully_in_scope(
        state,
        dataset_id,
        &authorized_organizations(grant, MANAGE_CAPABILITY),
    )
    .await?;
    load_dataset_revision_detail(state, dataset_id, revision_id, &read_scopes, can_manage).await
}

async fn load_dataset_revision_detail(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    revision_id: Uuid,
    read_scopes: &BTreeSet<Uuid>,
    can_manage: bool,
) -> Result<DatasetProductRevisionDetailV1, DatasetModuleError> {
    let row = sqlx::query(
        "SELECT r.id,r.dataset_id,r.version_number,r.version_label,r.revision_notes,
                r.version_major,r.version_minor,r.version_patch,r.semantic_bump,
                r.started_new_major_line,r.force_new_major_version,r.status,r.created_at,
                r.published_at,r.materialized_schema,r.materialized_table,
                r.materialized_row_count,r.materialized_at,r.definition_metadata,
                r.initial_source,r.operations,r.restriction_policy,r.generated_sql,
                r.output_fields,r.compatibility_findings,d.name,d.slug,d.grain
         FROM dataset_revisions r JOIN datasets d ON d.id=r.dataset_id
         WHERE r.dataset_id=$1 AND r.id=$2 AND r.lifecycle_state <> 'tombstoned'
           AND d.lifecycle_state <> 'tombstoned' AND ($3 OR r.status <> 'draft')",
    )
    .bind(dataset_id)
    .bind(revision_id)
    .bind(can_manage)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(undisclosed_dataset)?;
    let status = revision_status(row.try_get("status")?)?;
    let findings = revision_findings(row.try_get("compatibility_findings")?)?;
    let compatibility = crate::compatibility::compatibility_summary(&findings);
    let current = sqlx::query(
        "SELECT id,version_major FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' AND lifecycle_state <> 'tombstoned'",
    )
    .bind(dataset_id)
    .fetch_optional(&state.pool)
    .await?;
    let current_revision_id = current.as_ref().map(|value| value.get::<Uuid, _>("id"));
    let version_major: Option<i32> = row.try_get("version_major")?;
    let dependency_revision_id = if status == DatasetProductRevisionStatusV1::Draft {
        current_revision_id
    } else {
        Some(revision_id)
    };
    let dependency_major = if status == DatasetProductRevisionStatusV1::Draft {
        current
            .as_ref()
            .and_then(|value| value.get::<Option<i32>, _>("version_major"))
    } else {
        version_major
    };
    let (dependency_impacts, dependencies) = load_dataset_dependencies(
        state,
        dataset_id,
        dependency_revision_id,
        dependency_major,
        read_scopes,
        compatibility.state,
    )
    .await?;
    let metadata = match row.try_get::<Option<serde_json::Value>, _>("definition_metadata")? {
        Some(value) => parse_json(value, "revision metadata")?,
        None => DatasetProductRevisionMetadataV1 {
            name: row.try_get("name")?,
            slug: row.try_get("slug")?,
            grain: row.try_get("grain")?,
            visibility_node_ids: sqlx::query_scalar::<_, Uuid>(
                "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_id",
            )
            .bind(dataset_id)
            .fetch_all(&state.pool)
            .await?
            .into_iter()
            .map(|id| id.to_string())
            .collect(),
        },
    };
    Ok(DatasetProductRevisionDetailV1 {
        id: revision_id.to_string(),
        dataset_id: dataset_id.to_string(),
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        revision_notes: row.try_get("revision_notes")?,
        version_major,
        version_minor: row.try_get("version_minor")?,
        version_patch: row.try_get("version_patch")?,
        semantic_bump: semantic_bump(row.try_get("semantic_bump")?)?,
        started_new_major_line: row.try_get("started_new_major_line")?,
        force_new_major_version: row.try_get("force_new_major_version")?,
        status,
        is_current: current_revision_id == Some(revision_id),
        created_at: row
            .try_get::<chrono::DateTime<Utc>, _>("created_at")?
            .to_rfc3339(),
        published_at: optional_time(&row, "published_at")?,
        materialized_schema: row.try_get("materialized_schema")?,
        materialized_table: row.try_get("materialized_table")?,
        materialized_row_count: row.try_get("materialized_row_count")?,
        materialized_at: optional_time(&row, "materialized_at")?,
        metadata,
        initial_source: parse_json(
            row.try_get::<Option<serde_json::Value>, _>("initial_source")?
                .ok_or_else(|| {
                    DatasetModuleError::Internal("revision initial source is missing".into())
                })?,
            "revision initial source",
        )?,
        operations: parse_json(row.try_get("operations")?, "revision operations")?,
        restriction_policy: row
            .try_get::<Option<serde_json::Value>, _>("restriction_policy")?
            .map(|value| parse_json(value, "revision restriction policy"))
            .transpose()?,
        generated_sql: row.try_get("generated_sql")?,
        output_fields: parse_json(row.try_get("output_fields")?, "revision output fields")?,
        compatibility,
        compatibility_findings: findings,
        dependencies,
        dependency_impacts,
    })
}

async fn update_dataset_revision_label(
    State(state): State<DatasetModuleState>,
    Path((dataset_id, revision_id)): Path<(Uuid, Uuid)>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    require_json_content_type(&headers)?;
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 64 * 1024)
        .await
        .map_err(|_| {
            DatasetModuleError::BadRequest("Dataset revision label payload is too large".into())
        })?;
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.update_revision_label",
        "PATCH",
        &format!("/api/admin/datasets/{dataset_id}/revisions/{revision_id}/label"),
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }
    let payload: DatasetRevisionLabelRequestV1 =
        serde_json::from_slice(&body).map_err(|error| {
            DatasetModuleError::BadRequest(format!(
                "Invalid Dataset revision label payload: {error}"
            ))
        })?;
    validate_revision_label(&payload)?;
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    let row = sqlx::query(
        "SELECT version_label,revision_notes FROM dataset_revisions
         WHERE dataset_id=$1 AND id=$2 AND lifecycle_state <> 'tombstoned'
         FOR UPDATE",
    )
    .bind(dataset_id)
    .bind(revision_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(undisclosed_dataset)?;
    let version_label = payload
        .version_label
        .as_deref()
        .map(str::trim)
        .map(str::to_owned)
        .unwrap_or(row.try_get("version_label")?);
    let revision_notes = payload
        .revision_notes
        .as_deref()
        .map(str::trim)
        .map(str::to_owned)
        .unwrap_or(row.try_get("revision_notes")?);
    sqlx::query(
        "UPDATE dataset_revisions SET version_label=$1,revision_notes=$2,updated_at=now()
         WHERE dataset_id=$3 AND id=$4",
    )
    .bind(&version_label)
    .bind(&revision_notes)
    .bind(dataset_id)
    .bind(revision_id)
    .execute(&mut *tx)
    .await?;
    let response_body = serde_json::to_vec(&DatasetRevisionLabelResponseV1 {
        dataset_id: dataset_id.to_string(),
        revision_id: revision_id.to_string(),
        version_label,
        revision_notes,
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

async fn update_dataset_revision_options(
    State(state): State<DatasetModuleState>,
    Path((dataset_id, revision_id)): Path<(Uuid, Uuid)>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    require_json_content_type(&headers)?;
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 64 * 1024)
        .await
        .map_err(|_| {
            DatasetModuleError::BadRequest("Dataset revision options payload is too large".into())
        })?;
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.update_revision_options",
        "PATCH",
        &format!("/api/admin/datasets/{dataset_id}/revisions/{revision_id}/options"),
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }
    let payload: DatasetRevisionOptionsRequestV1 =
        serde_json::from_slice(&body).map_err(|error| {
            DatasetModuleError::BadRequest(format!(
                "Invalid Dataset revision options payload: {error}"
            ))
        })?;
    let mut detail =
        load_dataset_revision_detail(&state, dataset_id, revision_id, &manage_scopes, true).await?;
    if detail.status != DatasetProductRevisionStatusV1::Draft {
        return Err(DatasetModuleError::ValidationFailed(
            "Only draft Dataset revisions can change revision options".into(),
        ));
    }
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    sqlx::query("SELECT id FROM datasets WHERE id=$1 FOR UPDATE")
        .bind(dataset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(undisclosed_dataset)?;
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM dataset_revisions
         WHERE dataset_id=$1 AND id=$2 AND lifecycle_state <> 'tombstoned' FOR UPDATE",
    )
    .bind(dataset_id)
    .bind(revision_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(undisclosed_dataset)?;
    if status != "draft" {
        return Err(DatasetModuleError::ValidationFailed(
            "Only draft Dataset revisions can change revision options".into(),
        ));
    }
    let current = sqlx::query(
        "SELECT version_major,version_minor,version_patch
         FROM dataset_revisions
         WHERE dataset_id=$1 AND status='published' AND lifecycle_state <> 'tombstoned'",
    )
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?;
    let (version_major, version_minor, version_patch, bump) = match current {
        None => (1, 0, 0, DatasetProductSemanticBumpV1::Initial),
        Some(row) => {
            let current_major = row.try_get::<Option<i32>, _>("version_major")?.unwrap_or(1);
            let current_minor = row.try_get::<Option<i32>, _>("version_minor")?.unwrap_or(0);
            let current_patch = row.try_get::<Option<i32>, _>("version_patch")?.unwrap_or(0);
            let bump = if payload.force_new_major_version || detail.compatibility.major_count > 0 {
                DatasetProductSemanticBumpV1::Major
            } else if detail.compatibility.minor_count > 0 {
                DatasetProductSemanticBumpV1::Minor
            } else {
                DatasetProductSemanticBumpV1::Patch
            };
            match bump {
                DatasetProductSemanticBumpV1::Initial => (1, 0, 0, bump),
                DatasetProductSemanticBumpV1::Major => (current_major + 1, 0, 0, bump),
                DatasetProductSemanticBumpV1::Minor => (current_major, current_minor + 1, 0, bump),
                DatasetProductSemanticBumpV1::Patch => {
                    (current_major, current_minor, current_patch + 1, bump)
                }
            }
        }
    };
    detail.force_new_major_version = payload.force_new_major_version;
    detail.version_major = Some(version_major);
    detail.version_minor = Some(version_minor);
    detail.version_patch = Some(version_patch);
    detail.semantic_bump = Some(bump);
    detail.started_new_major_line = Some(matches!(
        bump,
        DatasetProductSemanticBumpV1::Initial | DatasetProductSemanticBumpV1::Major
    ));
    let response_body = serde_json::to_vec(&detail)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    sqlx::query(
        "UPDATE dataset_revisions
         SET force_new_major_version=$1,version_major=$2,version_minor=$3,version_patch=$4,
             semantic_bump=$5,started_new_major_line=$6,updated_at=now()
         WHERE dataset_id=$7 AND id=$8 AND status='draft'",
    )
    .bind(payload.force_new_major_version)
    .bind(version_major)
    .bind(version_minor)
    .bind(version_patch)
    .bind(semantic_bump_storage(bump))
    .bind(detail.started_new_major_line)
    .bind(dataset_id)
    .bind(revision_id)
    .execute(&mut *tx)
    .await?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

fn semantic_bump_storage(bump: DatasetProductSemanticBumpV1) -> &'static str {
    match bump {
        DatasetProductSemanticBumpV1::Initial => "initial",
        DatasetProductSemanticBumpV1::Major => "major",
        DatasetProductSemanticBumpV1::Minor => "minor",
        DatasetProductSemanticBumpV1::Patch => "patch",
    }
}

async fn delete_dataset_revision(
    State(state): State<DatasetModuleState>,
    Path((dataset_id, revision_id)): Path<(Uuid, Uuid)>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 1).await.map_err(|_| {
        DatasetModuleError::BadRequest("Dataset revision delete body must be empty".into())
    })?;
    if !body.is_empty() {
        return Err(DatasetModuleError::BadRequest(
            "Dataset revision delete body must be empty".into(),
        ));
    }
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.delete_revision",
        "DELETE",
        &format!("/api/admin/datasets/{dataset_id}/revisions/{revision_id}"),
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if manage_scopes.is_empty() {
        return Err(undisclosed_dataset());
    }
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }
    let status = sqlx::query_scalar::<_, String>(
        "SELECT status FROM dataset_revisions
         WHERE dataset_id=$1 AND id=$2 AND lifecycle_state <> 'tombstoned'
         FOR UPDATE",
    )
    .bind(dataset_id)
    .bind(revision_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(undisclosed_dataset)?;
    if status != "draft" {
        return Err(DatasetModuleError::ValidationFailed(
            "Only draft Dataset revisions can be deleted".into(),
        ));
    }
    sqlx::query("DELETE FROM dataset_revisions WHERE dataset_id=$1 AND id=$2 AND status='draft'")
        .bind(dataset_id)
        .bind(revision_id)
        .execute(&mut *tx)
        .await?;
    let response_body = serde_json::to_vec(&DatasetMutationIdResponseV1 {
        id: revision_id.to_string(),
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

async fn delete_dataset(
    State(state): State<DatasetModuleState>,
    Path(dataset_id): Path<Uuid>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 1)
        .await
        .map_err(|_| DatasetModuleError::BadRequest("Dataset delete body must be empty".into()))?;
    if !body.is_empty() {
        return Err(DatasetModuleError::BadRequest(
            "Dataset delete body must be empty".into(),
        ));
    }
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.delete",
        "DELETE",
        &format!("/api/admin/datasets/{dataset_id}"),
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if manage_scopes.is_empty() {
        return Err(undisclosed_dataset());
    }
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }
    sqlx::query(
        "SELECT id FROM datasets WHERE id=$1 AND lifecycle_state <> 'tombstoned' FOR UPDATE",
    )
    .bind(dataset_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(undisclosed_dataset)?;
    sqlx::query("DELETE FROM datasets WHERE id=$1")
        .bind(dataset_id)
        .execute(&mut *tx)
        .await?;
    let response_body = serde_json::to_vec(&DatasetMutationIdResponseV1 {
        id: dataset_id.to_string(),
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

fn validate_revision_label(
    payload: &DatasetRevisionLabelRequestV1,
) -> Result<(), DatasetModuleError> {
    if payload
        .version_label
        .as_deref()
        .is_some_and(|value| value.trim().chars().count() > 80)
    {
        return Err(DatasetModuleError::ValidationFailed(
            "Revision label must be 80 characters or fewer".into(),
        ));
    }
    if payload
        .revision_notes
        .as_deref()
        .is_some_and(|value| value.trim().chars().count() > 2_000)
    {
        return Err(DatasetModuleError::ValidationFailed(
            "Revision notes must be 2000 characters or fewer".into(),
        ));
    }
    Ok(())
}

async fn update_dataset_tags(
    State(state): State<DatasetModuleState>,
    Path(dataset_id): Path<Uuid>,
    request: Request,
) -> Result<Response, DatasetModuleError> {
    let headers = request.headers().clone();
    require_json_content_type(&headers)?;
    let idempotency_key = require_idempotency_key(&headers)?;
    let body = to_bytes(request.into_body(), 64 * 1024)
        .await
        .map_err(|_| DatasetModuleError::BadRequest("Dataset tag payload is too large".into()))?;
    let mutation = prepare_mutation(
        &state,
        &headers,
        "datasets.update_tags",
        "PATCH",
        &format!("/api/admin/datasets/{dataset_id}/tags"),
        &body,
        idempotency_key,
    )
    .await?;
    let manage_scopes = authorized_organizations(&mutation.grant.payload, MANAGE_CAPABILITY);
    if !dataset_fully_in_scope(&state, dataset_id, &manage_scopes).await? {
        return Err(undisclosed_dataset());
    }
    let payload: DatasetUpdateTagsRequestV1 = serde_json::from_slice(&body).map_err(|error| {
        DatasetModuleError::BadRequest(format!("Invalid Dataset tag payload: {error}"))
    })?;
    let tags = normalize_dataset_tags(payload.tags)?;
    let mut tx = state.pool.begin().await?;
    if let Some(response) = lock_and_load_replay(&mut tx, &mutation).await? {
        tx.commit().await?;
        return Ok(response);
    }
    sqlx::query("SELECT id FROM datasets WHERE id=$1 FOR UPDATE")
        .bind(dataset_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(undisclosed_dataset)?;
    sqlx::query("DELETE FROM dataset_tags WHERE dataset_id=$1")
        .bind(dataset_id)
        .execute(&mut *tx)
        .await?;
    for tag in &tags {
        sqlx::query("INSERT INTO dataset_tags(dataset_id,tag) VALUES($1,$2)")
            .bind(dataset_id)
            .bind(tag)
            .execute(&mut *tx)
            .await?;
    }
    let response_body = serde_json::to_vec(&DatasetMutationIdResponseV1 {
        id: dataset_id.to_string(),
    })
    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    store_mutation_receipt(&mut tx, &mutation, 200, "application/json", &response_body).await?;
    tx.commit().await?;
    stored_response(200, "application/json", response_body)
}

struct MutationIdentity {
    grant: SignedEnvelopeV1<AuthorizationGrantV3>,
    action: String,
    idempotency_key: String,
    request_digest: String,
}

async fn prepare_mutation(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    action: &str,
    method: &str,
    path: &str,
    body: &[u8],
    idempotency_key: &str,
) -> Result<MutationIdentity, DatasetModuleError> {
    let grant = authorize_product(
        state,
        headers,
        action,
        AuthorizationGrantOperationV1::Mutation,
        DATASET_AUTHORING_CONTRACT_ID,
    )
    .await?;
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
    })?;
    let request_digest = digest_json(&serde_json::json!({
        "module_instance_id": security.module_instance_id,
        "actor_id": grant.payload.original_actor_id,
        "authorization_jti": grant.payload.jti,
        "action": action,
        "method": method,
        "path": path,
        "raw_body_sha256": digest_bytes(body),
        "idempotency_key": idempotency_key,
    }))?;
    Ok(MutationIdentity {
        grant,
        action: action.to_owned(),
        idempotency_key: idempotency_key.to_owned(),
        request_digest,
    })
}

async fn lock_and_load_replay(
    tx: &mut Transaction<'_, Postgres>,
    mutation: &MutationIdentity,
) -> Result<Option<Response>, DatasetModuleError> {
    let lock_identity = &mutation.idempotency_key;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(lock_identity)
        .execute(&mut **tx)
        .await?;
    let Some(row) = sqlx::query(
        "SELECT request_digest,response_status,response_media_type,response_body
         FROM dataset_idempotency_receipts
         WHERE idempotency_key=$1",
    )
    .bind(&mutation.idempotency_key)
    .fetch_optional(&mut **tx)
    .await?
    else {
        return Ok(None);
    };
    if row.try_get::<String, _>("request_digest")? != mutation.request_digest {
        return Err(DatasetModuleError::IdempotencyMismatch);
    }
    Ok(Some(stored_response(
        row.try_get("response_status")?,
        row.try_get::<String, _>("response_media_type")?.as_str(),
        row.try_get("response_body")?,
    )?))
}

async fn store_mutation_receipt(
    tx: &mut Transaction<'_, Postgres>,
    mutation: &MutationIdentity,
    status: i32,
    media_type: &str,
    body: &[u8],
) -> Result<(), DatasetModuleError> {
    sqlx::query(
        "INSERT INTO dataset_idempotency_receipts
         (actor_id,action,idempotency_key,request_digest,response_status,response_media_type,response_body)
         VALUES($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(mutation.grant.payload.original_actor_id)
    .bind(&mutation.action)
    .bind(&mutation.idempotency_key)
    .bind(&mutation.request_digest)
    .bind(status)
    .bind(media_type)
    .bind(body)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn require_json_content_type(headers: &HeaderMap) -> Result<(), DatasetModuleError> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return Err(DatasetModuleError::BadRequest(
            "Content-Type must be application/json".into(),
        ));
    }
    Ok(())
}

fn require_idempotency_key(headers: &HeaderMap) -> Result<&str, DatasetModuleError> {
    let key = headers
        .get(DATASET_IDEMPOTENCY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.len() <= 200)
        .ok_or_else(|| {
            DatasetModuleError::BadRequest("A valid X-Idempotency-Key is required".into())
        })?;
    Ok(key)
}

fn normalize_dataset_tags(tags: Vec<String>) -> Result<Vec<String>, DatasetModuleError> {
    let mut normalized = Vec::new();
    let mut seen = BTreeSet::new();
    for tag in tags {
        let value = tag.trim();
        if value.is_empty() {
            continue;
        }
        if value.chars().count() > 48 {
            return Err(DatasetModuleError::BadRequest(
                "Dataset tags must be 48 characters or fewer".into(),
            ));
        }
        if seen.insert(value.to_ascii_lowercase()) {
            normalized.push(value.to_owned());
        }
    }
    if normalized.len() > 20 {
        return Err(DatasetModuleError::BadRequest(
            "Datasets can have at most 20 tags".into(),
        ));
    }
    Ok(normalized)
}

fn digest_bytes(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn digest_json(value: &serde_json::Value) -> Result<String, DatasetModuleError> {
    serde_jcs::to_vec(value)
        .map(|bytes| digest_bytes(&bytes))
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))
}

fn stored_response(
    status: i32,
    media_type: &str,
    body: Vec<u8>,
) -> Result<Response, DatasetModuleError> {
    let status = u16::try_from(status)
        .ok()
        .and_then(|value| StatusCode::from_u16(value).ok())
        .ok_or_else(|| DatasetModuleError::Internal("stored response status is invalid".into()))?;
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, media_type)
        .body(Body::from(body))
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))
}

async fn require_visible_dataset(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    scopes: &BTreeSet<Uuid>,
) -> Result<(), DatasetModuleError> {
    if scopes.is_empty() {
        return Err(undisclosed_dataset());
    }
    let visible = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(
           SELECT 1 FROM datasets d JOIN dataset_scope_nodes s ON s.dataset_id=d.id
           WHERE d.id=$1 AND d.lifecycle_state <> 'tombstoned' AND s.node_id=ANY($2)
         )",
    )
    .bind(dataset_id)
    .bind(scopes.iter().copied().collect::<Vec<_>>())
    .fetch_one(&state.pool)
    .await?;
    if !visible {
        return Err(undisclosed_dataset());
    }
    Ok(())
}

async fn dataset_fully_in_scope(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    scopes: &BTreeSet<Uuid>,
) -> Result<bool, DatasetModuleError> {
    if scopes.is_empty() {
        return Ok(false);
    }
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM dataset_scope_nodes WHERE dataset_id=$1)
           AND NOT EXISTS(
             SELECT 1 FROM dataset_scope_nodes
             WHERE dataset_id=$1 AND NOT(node_id=ANY($2))
           )",
    )
    .bind(dataset_id)
    .bind(scopes.iter().copied().collect::<Vec<_>>())
    .fetch_one(&state.pool)
    .await?)
}

async fn dataset_sources_fully_in_scope_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    dataset_id: Uuid,
    scopes: &BTreeSet<Uuid>,
) -> Result<bool, DatasetModuleError> {
    if scopes.is_empty() {
        return Ok(false);
    }
    Ok(sqlx::query_scalar::<_, bool>(
        "SELECT NOT EXISTS(
           SELECT 1
           FROM dataset_sources source
           WHERE source.dataset_id=$1
             AND (
               cardinality(source.source_scope_node_ids)=0
               OR EXISTS(
                 SELECT 1 FROM unnest(source.source_scope_node_ids) AS source_node(node_id)
                 WHERE NOT(source_node.node_id=ANY($2))
               )
             )
         )",
    )
    .bind(dataset_id)
    .bind(scopes.iter().copied().collect::<Vec<_>>())
    .fetch_one(&mut **transaction)
    .await?)
}

async fn load_dataset_dependencies(
    state: &DatasetModuleState,
    source_dataset_id: Uuid,
    source_revision_id: Option<Uuid>,
    source_version_major: Option<i32>,
    scopes: &BTreeSet<Uuid>,
    compatibility: DatasetProductCompatibilityStateV1,
) -> Result<
    (
        Vec<DatasetProductDependencyImpactV1>,
        DatasetProductDependencySummaryV1,
    ),
    DatasetModuleError,
> {
    let rows = sqlx::query(
        "SELECT DISTINCT d.id,d.name,ds.source_reference
         FROM dataset_sources ds JOIN datasets d ON d.id=ds.dataset_id
         WHERE d.lifecycle_state <> 'tombstoned'
           AND EXISTS(SELECT 1 FROM dataset_scope_nodes s WHERE s.dataset_id=d.id AND s.node_id=ANY($1))
           AND (
             ($2::uuid IS NOT NULL AND ds.source_reference->>'kind'='dataset'
               AND ds.source_reference->>'dataset_revision_id'=$2::text)
             OR
             ($3::integer IS NOT NULL AND ds.source_reference->>'kind'='dataset_major'
               AND ds.source_reference->>'dataset_id'=$4::text
               AND (ds.source_reference->>'version_major')::integer=$3)
           )
         ORDER BY d.name,d.id",
    )
    .bind(scopes.iter().copied().collect::<Vec<_>>())
    .bind(source_revision_id)
    .bind(source_version_major)
    .bind(source_dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let mut impacts = Vec::with_capacity(rows.len());
    for row in rows {
        let reference: DatasetProductSourceV1 = parse_json(
            row.try_get("source_reference")?,
            "downstream Dataset source",
        )?;
        let (binding_mode, pinned_revision_id, pinned_version_major, carry_forward_state) =
            match reference {
                DatasetProductSourceV1::Dataset {
                    dataset_revision_id,
                    ..
                } => (
                    DatasetProductDependencyBindingModeV1::ExactRevision,
                    Some(dataset_revision_id),
                    None,
                    DatasetProductCarryForwardStateV1::Safe,
                ),
                DatasetProductSourceV1::DatasetMajor { version_major, .. } => (
                    DatasetProductDependencyBindingModeV1::MajorLine,
                    None,
                    Some(version_major),
                    carry_forward_state(compatibility),
                ),
                DatasetProductSourceV1::Form { .. } => {
                    return Err(DatasetModuleError::Internal(
                        "downstream Dataset dependency contained a Form source".into(),
                    ));
                }
            };
        impacts.push(DatasetProductDependencyImpactV1 {
            kind: DatasetProductDependencyKindV1::Dataset,
            id: row.try_get::<Uuid, _>("id")?.to_string(),
            name: row.try_get("name")?,
            pinned_revision_id,
            pinned_version_major,
            binding_mode,
            carry_forward_state,
            message: match binding_mode {
                DatasetProductDependencyBindingModeV1::ExactRevision => {
                    "Exact revision binding remains pinned until explicitly updated".into()
                }
                DatasetProductDependencyBindingModeV1::MajorLine => {
                    "Major-line binding follows compatible publications in its selected line".into()
                }
            },
        });
    }
    let state = impacts
        .iter()
        .map(|impact| impact.carry_forward_state)
        .max_by_key(|state| match state {
            DatasetProductCarryForwardStateV1::Safe => 0,
            DatasetProductCarryForwardStateV1::ManualReview => 1,
            DatasetProductCarryForwardStateV1::Blocked => 2,
        })
        .unwrap_or(DatasetProductCarryForwardStateV1::Safe);
    let count = impacts.len();
    Ok((
        impacts,
        DatasetProductDependencySummaryV1 {
            dependency_count: count,
            dataset_count: count,
            carry_forward_state: state,
        },
    ))
}

fn revision_status(value: String) -> Result<DatasetProductRevisionStatusV1, DatasetModuleError> {
    match value.as_str() {
        "draft" => Ok(DatasetProductRevisionStatusV1::Draft),
        "published" => Ok(DatasetProductRevisionStatusV1::Published),
        "superseded" => Ok(DatasetProductRevisionStatusV1::Superseded),
        _ => Err(DatasetModuleError::Internal(
            "stored revision status is invalid".into(),
        )),
    }
}

fn semantic_bump(
    value: Option<String>,
) -> Result<Option<DatasetProductSemanticBumpV1>, DatasetModuleError> {
    value
        .map(|value| match value.as_str() {
            "initial" => Ok(DatasetProductSemanticBumpV1::Initial),
            "major" => Ok(DatasetProductSemanticBumpV1::Major),
            "minor" => Ok(DatasetProductSemanticBumpV1::Minor),
            "patch" => Ok(DatasetProductSemanticBumpV1::Patch),
            _ => Err(DatasetModuleError::Internal(
                "stored semantic bump is invalid".into(),
            )),
        })
        .transpose()
}

fn revision_findings(
    value: serde_json::Value,
) -> Result<Vec<DatasetProductCompatibilityFindingV1>, DatasetModuleError> {
    parse_json(value, "revision compatibility findings")
}

fn carry_forward_state(
    state: DatasetProductCompatibilityStateV1,
) -> DatasetProductCarryForwardStateV1 {
    match state {
        DatasetProductCompatibilityStateV1::Compatible => DatasetProductCarryForwardStateV1::Safe,
        DatasetProductCompatibilityStateV1::Review => {
            DatasetProductCarryForwardStateV1::ManualReview
        }
        DatasetProductCompatibilityStateV1::Breaking => DatasetProductCarryForwardStateV1::Blocked,
    }
}

fn optional_time(
    row: &sqlx::postgres::PgRow,
    column: &str,
) -> Result<Option<String>, DatasetModuleError> {
    Ok(row
        .try_get::<Option<chrono::DateTime<Utc>>, _>(column)?
        .map(|value| value.to_rfc3339()))
}

fn parse_json<T: serde::de::DeserializeOwned>(
    value: serde_json::Value,
    label: &str,
) -> Result<T, DatasetModuleError> {
    serde_json::from_value(value).map_err(|error| {
        DatasetModuleError::Internal(format!("stored {label} is invalid: {error}"))
    })
}

fn undisclosed_dataset() -> DatasetModuleError {
    DatasetModuleError::NotFound("Dataset resource was not found".into())
}

async fn authorize_read(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    action: &str,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, DatasetModuleError> {
    authorize_product(
        state,
        headers,
        action,
        AuthorizationGrantOperationV1::Read,
        DATASET_CONTRACT_ID,
    )
    .await
}

async fn authorize_product(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
    functional_contract: &str,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, DatasetModuleError> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(DatasetModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV3> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| DatasetModuleError::Forbidden)?,
    )
    .map_err(|_| DatasetModuleError::Forbidden)?;
    state
        .core_authorization_verifier
        .verify(&envelope)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
    })?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(DatasetModuleError::Unavailable(
            "Dataset module is not enabled".into(),
        ));
    }
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(DatasetModuleError::Forbidden)?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            },
            dependency_binding: DependencyBindingKey::new(CORE_DATASET_BINDING)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            functional_contract: FunctionalContractId::new(functional_contract)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            action: action.into(),
            operation,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    Ok(envelope)
}

fn authorized_organizations(grant: &AuthorizationGrantV3, capability: &str) -> BTreeSet<Uuid> {
    grant
        .capability_scope_bindings
        .iter()
        .filter(|binding| binding.capability.as_str() == capability)
        .flat_map(|binding| {
            std::iter::once(binding.organization_root_id)
                .chain(binding.authorized_organization_ids.iter().copied())
        })
        .collect()
}

fn readable_organizations(grant: &AuthorizationGrantV3) -> BTreeSet<Uuid> {
    let mut organizations = authorized_organizations(grant, READ_CAPABILITY);
    organizations.extend(authorized_organizations(grant, MANAGE_CAPABILITY));
    organizations
}
