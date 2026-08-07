//! Core-owned Dataset compatibility provider. Authorization is evaluated
//! before Dataset identity or metadata is disclosed, and every query executes
//! against Core-owned materializations through this module-neutral boundary.

use std::collections::{BTreeMap, BTreeSet};

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use serde::Deserialize;
use serde_json::Value;
use sqlx::{Postgres, QueryBuilder, Row};
use tessara_composition::{
    BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1,
    BootstrapDependencyValidationAuthorizationV1, BootstrapDependencyValidationContextV1,
    BootstrapDependencyValidationRequestV1, canonical_digest,
};
use tessara_datasets_contract::{
    DATASET_BINDING_KEY, DATASET_BOOTSTRAP_VALIDATION_ACTION, DATASET_BOOTSTRAP_VALIDATION_PATH,
    DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY, DATASET_CONTRACT_ID,
    DATASET_CONTRACT_SCHEMA_VERSION, DATASET_CONTRACT_VERSION, DatasetAction,
    DatasetAggregateFunction, DatasetBootstrapValidationBatch, DatasetBootstrapValidationResponse,
    DatasetBootstrapValidationResult, DatasetCatalogRequest, DatasetCatalogResponse,
    DatasetCompatibilityFinding, DatasetCompatibilityRequest, DatasetCompatibilityResponse,
    DatasetDistinctValuesRequest, DatasetDistinctValuesResponse, DatasetExecutionRequest,
    DatasetExecutionResponse, DatasetExecutionRow, DatasetFieldContract, DatasetFieldRequirement,
    DatasetFilterOperator, DatasetMajorLineMetadata, DatasetMajorLineReference,
    DatasetMissingPolicy, DatasetProvenanceSummary, DatasetSchemaRequest, DatasetSortDirection,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantV3, AuthorizationValidationContextV3,
    ModuleDefinitionId, ModuleServicePrincipalV1, ProtocolSignaturePurposeV1, ServiceActionMethod,
    SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    auth::{self, CapabilityBoundary},
    db::AppState,
    error::{ApiError, ApiResult},
};

const MAX_DATASET_EXECUTION_OFFSET: u32 = 1_000_000;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/private/datasets/catalog", post(catalog))
        .route("/api/private/datasets/schema", post(schema))
        .route(
            "/api/private/datasets/distinct-values",
            post(distinct_values),
        )
        .route("/api/private/datasets/compatibility", post(compatibility))
        .route(DATASET_BOOTSTRAP_VALIDATION_PATH, post(validate_bootstrap))
        .route("/api/private/datasets/execute", post(execute))
}

async fn authorize(
    state: &AppState,
    headers: &HeaderMap,
    authorization_action: &'static str,
    method_path: &'static str,
    body: &[u8],
) -> ApiResult<(SignedEnvelopeV1<AuthorizationGrantV3>, auth::AccountContext)> {
    let inbound = crate::module_service_requests::verified_authorization(headers)?;
    let declaration = crate::core_service_providers::resolve_service_action(
        crate::core_service_providers::DATASET_MAJOR_LINE_CONTRACT,
        authorization_action,
    )
    .filter(|declaration| {
        declaration.method == ServiceActionMethod::Post && declaration.path == method_path
    })
    .ok_or_else(restricted)?;
    let installation_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM application_installations WHERE id=$1)")
            .bind(inbound.payload.installation_id)
            .fetch_one(&state.pool)
            .await?;
    if !installation_exists
        || inbound.payload.audience
            != (AuthorizationAudienceV1::CoreInstallation {
                installation_id: inbound.payload.installation_id,
            })
        || inbound.payload.functional_contract.as_str() != declaration.functional_contract
        || inbound.payload.action != declaration.authorization_action
        || inbound.payload.operation != declaration.operation
        || !inbound
            .payload
            .capability_scope_bindings
            .iter()
            .any(|binding| binding.capability.as_str() == declaration.required_capability)
    {
        return Err(restricted());
    }
    let revisions = sqlx::query_as::<_, (i64, i64)>(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    inbound
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: inbound.payload.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: inbound.payload.presenting_service.clone(),
            audience: AuthorizationAudienceV1::CoreInstallation {
                installation_id: inbound.payload.installation_id,
            },
            dependency_binding: inbound.payload.dependency_binding.clone(),
            functional_contract: inbound.payload.functional_contract.clone(),
            action: declaration.authorization_action.into(),
            operation: declaration.operation,
            resource_assertion: inbound.payload.resource_assertion.clone(),
            authorization_revision: revisions.0 as u64,
            organization_revision: revisions.1 as u64,
            now: chrono::Utc::now(),
        })
        .map_err(|_| restricted())?;
    let presenter = match &inbound.payload.presenting_service {
        ModuleServicePrincipalV1::ModuleInstance { .. } => {
            inbound.payload.presenting_service.clone()
        }
        ModuleServicePrincipalV1::CoreGateway => return Err(restricted()),
    };
    crate::module_service_requests::validate_for_principal(
        state,
        headers,
        &inbound,
        crate::module_service_requests::ModuleServiceRequestExpectation {
            principal: &presenter,
            grant_consumption:
                crate::module_service_requests::AuthorizationGrantConsumption::OneTimeProviderAudience(
                    inbound.payload.jti,
                ),
            method: "POST",
            path: method_path,
            body,
        },
    )
    .await?;
    let account =
        auth::account_context_for_actor(&state.pool, inbound.payload.original_actor_id).await?;
    Ok((inbound, account))
}

async fn catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DatasetCatalogRequest>,
) -> ApiResult<Json<DatasetCatalogResponse>> {
    if request.action != DatasetAction::Catalog {
        return Err(restricted());
    }
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    let (inbound, account) = authorize(
        &state,
        &headers,
        "datasets.catalog",
        "/api/private/datasets/catalog",
        &body,
    )
    .await?;
    let boundary = auth::capability_boundary(&state.pool, &account, "datasets:read").await?;
    let rows = sqlx::query(
        "SELECT DISTINCT ON (d.id,r.version_major)
                d.id,d.name,d.slug,d.grain,r.version_major,r.output_fields,
                COALESCE(m.rebuild_status,'unavailable') AS materialization_state
         FROM datasets d
         JOIN dataset_revisions r ON r.dataset_id=d.id
         LEFT JOIN dataset_major_materializations m
           ON m.dataset_id=d.id AND m.version_major=r.version_major
         WHERE r.version_major IS NOT NULL
           AND r.status IN ('published'::dataset_revision_status,'superseded'::dataset_revision_status)
         ORDER BY d.id,r.version_major,r.version_number DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    let dataset_ids = rows
        .iter()
        .map(|row| row.try_get::<Uuid, _>("id"))
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    let tags_by_dataset = crate::datasets::load_dataset_tags(&state.pool, &dataset_ids).await?;
    let provenance_by_dataset =
        crate::datasets::load_dataset_provenance(&state.pool, &dataset_ids).await?;
    let mut datasets = Vec::new();
    for row in rows {
        let dataset_id: Uuid = row.try_get("id")?;
        let scope_node_ids =
            crate::datasets::load_dataset_scope_node_ids(&state.pool, dataset_id).await?;
        if !boundary_allows(&boundary, &scope_node_ids) {
            continue;
        }
        datasets.push(metadata_from_row(MetadataRow {
            installation_id: inbound.payload.installation_id,
            dataset_id,
            dataset_name: row.try_get("name")?,
            dataset_slug: row.try_get("slug")?,
            grain: row.try_get("grain")?,
            tags: tags_by_dataset
                .get(&dataset_id)
                .cloned()
                .unwrap_or_default(),
            provenance: provenance_by_dataset
                .get(&dataset_id)
                .cloned()
                .unwrap_or_default(),
            major: row.try_get::<i32, _>("version_major")?,
            output_fields: row.try_get("output_fields")?,
            materialization_state: row.try_get("materialization_state")?,
            scope_node_ids,
        })?);
    }
    Ok(Json(DatasetCatalogResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        datasets,
    }))
}

async fn schema(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DatasetSchemaRequest>,
) -> ApiResult<Json<DatasetMajorLineMetadata>> {
    if request.action != DatasetAction::ResolveSchema {
        return Err(restricted());
    }
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    let (inbound, account) = authorize(
        &state,
        &headers,
        "datasets.schema",
        "/api/private/datasets/schema",
        &body,
    )
    .await?;
    Ok(Json(
        load_authorized_metadata(&state, &account, &inbound, &request.reference).await?,
    ))
}

async fn distinct_values(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DatasetDistinctValuesRequest>,
) -> ApiResult<Json<DatasetDistinctValuesResponse>> {
    if request.action != DatasetAction::DistinctValues
        || request.field_key.trim().is_empty()
        || !(1..=200).contains(&request.limit)
    {
        return Err(ApiError::BadRequest(
            "Dataset distinct-value request is invalid".into(),
        ));
    }
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    let (inbound, account) = authorize(
        &state,
        &headers,
        "datasets.distinct_values",
        "/api/private/datasets/distinct-values",
        &body,
    )
    .await?;
    let metadata = load_authorized_metadata(&state, &account, &inbound, &request.reference).await?;
    if !metadata
        .fields
        .iter()
        .any(|field| field.key == request.field_key)
    {
        return Err(restricted());
    }
    let mut values = crate::datasets::load_dataset_major_distinct_values(
        &state.pool,
        &account,
        request.reference.dataset_id(),
        request.reference.major(),
        &request.field_key,
    )
    .await?;
    values.truncate(request.limit as usize);
    Ok(Json(DatasetDistinctValuesResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        values: values.into_iter().map(Value::String).collect(),
    }))
}

async fn compatibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DatasetCompatibilityRequest>,
) -> ApiResult<Json<DatasetCompatibilityResponse>> {
    if request.action != DatasetAction::CheckCompatibility {
        return Err(restricted());
    }
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    let (inbound, account) = authorize(
        &state,
        &headers,
        "datasets.compatibility",
        "/api/private/datasets/compatibility",
        &body,
    )
    .await?;
    let metadata = load_authorized_metadata(&state, &account, &inbound, &request.reference).await?;
    let fields = metadata
        .fields
        .iter()
        .map(|field| (field.key.as_str(), field.field_type.as_str()))
        .collect::<BTreeMap<_, _>>();
    let findings = compatibility_findings(
        &metadata.materialization_state,
        &fields,
        request.required_fields,
    );
    Ok(Json(DatasetCompatibilityResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        compatible: findings.is_empty(),
        findings,
    }))
}

fn compatibility_findings(
    materialization_state: &str,
    fields: &BTreeMap<&str, &str>,
    requirements: Vec<DatasetFieldRequirement>,
) -> Vec<DatasetCompatibilityFinding> {
    if materialization_state != "ready" {
        return vec![DatasetCompatibilityFinding {
            code: DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY.into(),
            field_key: "materialization_state".into(),
        }];
    }
    let mut findings = Vec::new();
    for requirement in requirements {
        match fields.get(requirement.field_key.as_str()) {
            None => findings.push(DatasetCompatibilityFinding {
                code: "field_missing".into(),
                field_key: requirement.field_key,
            }),
            Some(actual)
                if !requirement
                    .accepted_types
                    .iter()
                    .any(|value| value == actual) =>
            {
                findings.push(DatasetCompatibilityFinding {
                    code: "field_type_incompatible".into(),
                    field_key: requirement.field_key,
                });
            }
            Some(_) => {}
        }
    }
    findings
}

async fn validate_bootstrap(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<BootstrapDependencyValidationRequestV1>,
) -> ApiResult<Json<DatasetBootstrapValidationResponse>> {
    if request.schema_version != BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1
        || request.desired_revision == 0
        || request.apply_sequence == 0
    {
        return Err(restricted());
    }
    let encoded = headers
        .get("x-tessara-bootstrap-validation-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted)?;
    let bytes = URL_SAFE_NO_PAD.decode(encoded).map_err(|_| restricted())?;
    let authorization: SignedEnvelopeV1<BootstrapDependencyValidationAuthorizationV1> =
        serde_json::from_slice(&bytes).map_err(|_| restricted())?;
    crate::core_security::protocol_signer(
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
    )?
    .verifier()
    .verify(&authorization)
    .map_err(|_| restricted())?;
    let principal = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: authorization.payload.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(&authorization.payload.module_definition_id)
            .map_err(|_| restricted())?,
    };
    validate_bootstrap_request_authorization(&authorization.payload, &request, Utc::now())?;
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    crate::module_service_requests::validate_materializing_principal_with_authorization(
        &state,
        &headers,
        authorization.payload.installation_id,
        authorization.payload.correlation_id,
        encoded,
        crate::module_service_requests::ModuleServiceRequestExpectation {
            principal: &principal,
            grant_consumption:
                crate::module_service_requests::AuthorizationGrantConsumption::OneTimeProviderAudience(
                    authorization.payload.jti,
                ),
            method: "POST",
            path: DATASET_BOOTSTRAP_VALIDATION_PATH,
            body: &body,
        },
    )
    .await?;

    let batch: DatasetBootstrapValidationBatch =
        serde_json::from_value(request.payload).map_err(|_| restricted())?;
    if batch.schema_version != DATASET_CONTRACT_SCHEMA_VERSION || batch.items.is_empty() {
        return Err(restricted());
    }

    let mut validation_keys = BTreeSet::new();
    let mut results = Vec::with_capacity(batch.items.len());
    for item in batch.items {
        if item.validation_key.trim().is_empty()
            || !validation_keys.insert(item.validation_key.clone())
            || item.reference.reference().installation_id() != authorization.payload.installation_id
        {
            return Err(restricted());
        }
        let metadata = load_metadata(
            &state,
            authorization.payload.installation_id,
            &item.reference,
        )
        .await?;
        let fields = metadata
            .fields
            .iter()
            .map(|field| (field.key.as_str(), field.field_type.as_str()))
            .collect::<BTreeMap<_, _>>();
        let findings = compatibility_findings(
            &metadata.materialization_state,
            &fields,
            item.required_fields,
        );
        results.push(DatasetBootstrapValidationResult {
            validation_key: item.validation_key,
            metadata,
            compatibility: DatasetCompatibilityResponse {
                schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
                compatible: findings.is_empty(),
                findings,
            },
        });
    }
    Ok(Json(DatasetBootstrapValidationResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        results,
    }))
}

fn validate_bootstrap_request_authorization(
    authorization: &BootstrapDependencyValidationAuthorizationV1,
    request: &BootstrapDependencyValidationRequestV1,
    now: chrono::DateTime<Utc>,
) -> ApiResult<()> {
    let audience = AuthorizationAudienceV1::CoreInstallation {
        installation_id: authorization.installation_id,
    };
    let contract_version = DATASET_CONTRACT_VERSION
        .parse()
        .expect("static Dataset contract version");
    let request_digest = canonical_digest(request).map_err(|_| restricted())?;
    authorization
        .validate_for(&BootstrapDependencyValidationContextV1 {
            installation_id: authorization.installation_id,
            module_instance_id: authorization.module_instance_id,
            module_definition_id: authorization.module_definition_id.as_str(),
            input_digest: &request.input_digest,
            desired_revision: request.desired_revision,
            apply_sequence: request.apply_sequence,
            target_plan_digest: &request.target_plan_digest,
            dependency_binding: DATASET_BINDING_KEY,
            functional_contract: DATASET_CONTRACT_ID,
            functional_contract_version: &contract_version,
            action: DATASET_BOOTSTRAP_VALIDATION_ACTION,
            method: ServiceActionMethod::Post,
            path: DATASET_BOOTSTRAP_VALIDATION_PATH,
            audience: &audience,
            request_digest: &request_digest,
            now,
        })
        .map_err(|_| restricted())
}

async fn execute(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DatasetExecutionRequest>,
) -> ApiResult<Json<DatasetExecutionResponse>> {
    if request.action != DatasetAction::Execute || request.limit == 0 || request.limit > 1_000 {
        return Err(ApiError::BadRequest(
            "Dataset execution request is invalid".into(),
        ));
    }
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    let (inbound, account) = authorize(
        &state,
        &headers,
        "datasets.execute",
        "/api/private/datasets/execute",
        &body,
    )
    .await?;
    let metadata = load_authorized_metadata(&state, &account, &inbound, &request.reference).await?;
    if metadata.materialization_state != "ready" {
        return Ok(Json(DatasetExecutionResponse {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            materialization_state: metadata.materialization_state,
            fields: metadata.fields,
            rows: Vec::new(),
            next_cursor: None,
        }));
    }
    execute_materialization(&state, &account, request, metadata)
        .await
        .map(Json)
}

#[derive(Deserialize)]
struct StoredField {
    key: String,
    label: String,
    field_type: String,
}

struct MetadataRow {
    installation_id: Uuid,
    dataset_id: Uuid,
    dataset_name: String,
    dataset_slug: String,
    grain: String,
    tags: Vec<String>,
    provenance: DatasetProvenanceSummary,
    major: i32,
    output_fields: Option<Value>,
    materialization_state: String,
    scope_node_ids: Vec<Uuid>,
}

fn metadata_from_row(row: MetadataRow) -> ApiResult<DatasetMajorLineMetadata> {
    let MetadataRow {
        installation_id,
        dataset_id,
        dataset_name,
        dataset_slug,
        grain,
        tags,
        provenance,
        major,
        output_fields,
        materialization_state,
        scope_node_ids,
    } = row;
    if major <= 0 {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "stored Dataset major is invalid"
        )));
    }
    let fields = serde_json::from_value::<Vec<StoredField>>(
        output_fields.unwrap_or_else(|| Value::Array(Vec::new())),
    )
    .map_err(|error| {
        ApiError::Internal(anyhow::anyhow!(
            "stored Dataset fields are invalid: {error}"
        ))
    })?
    .into_iter()
    .map(|field| DatasetFieldContract {
        key: field.key,
        label: field.label,
        field_type: field.field_type,
        restriction_tier: "provider_enforced".into(),
    })
    .collect();
    Ok(DatasetMajorLineMetadata {
        reference: DatasetMajorLineReference::from_parts(installation_id, dataset_id, major)
            .map_err(|error| ApiError::Internal(error.into()))?,
        dataset_name,
        dataset_slug,
        grain,
        tags,
        provenance,
        materialization_state,
        fields,
        scope_node_ids,
    })
}

async fn load_authorized_metadata(
    state: &AppState,
    account: &auth::AccountContext,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV3>,
    reference: &DatasetMajorLineReference,
) -> ApiResult<DatasetMajorLineMetadata> {
    if reference.reference().installation_id() != inbound.payload.installation_id {
        return Err(restricted());
    }
    let boundary = auth::capability_boundary(&state.pool, account, "datasets:read").await?;
    let dataset_id = reference.dataset_id();
    let scope_node_ids = crate::datasets::load_dataset_scope_node_ids(&state.pool, dataset_id)
        .await
        .map_err(|_| restricted())?;
    if !boundary_allows(&boundary, &scope_node_ids) {
        return Err(restricted());
    }
    load_metadata_with_scope(
        state,
        inbound.payload.installation_id,
        reference,
        scope_node_ids,
    )
    .await
}

async fn load_metadata(
    state: &AppState,
    installation_id: Uuid,
    reference: &DatasetMajorLineReference,
) -> ApiResult<DatasetMajorLineMetadata> {
    if reference.reference().installation_id() != installation_id {
        return Err(restricted());
    }
    let scope_node_ids =
        crate::datasets::load_dataset_scope_node_ids(&state.pool, reference.dataset_id())
            .await
            .map_err(|_| restricted())?;
    load_metadata_with_scope(state, installation_id, reference, scope_node_ids).await
}

async fn load_metadata_with_scope(
    state: &AppState,
    installation_id: Uuid,
    reference: &DatasetMajorLineReference,
    scope_node_ids: Vec<Uuid>,
) -> ApiResult<DatasetMajorLineMetadata> {
    let dataset_id = reference.dataset_id();
    let row = sqlx::query(
        "SELECT d.name,d.slug,d.grain,r.output_fields,
                COALESCE(m.rebuild_status,'unavailable') AS materialization_state
         FROM datasets d
         JOIN LATERAL (
           SELECT output_fields FROM dataset_revisions
           WHERE dataset_id=d.id AND version_major=$2
             AND status IN ('published'::dataset_revision_status,'superseded'::dataset_revision_status)
           ORDER BY version_number DESC LIMIT 1
         ) r ON true
         LEFT JOIN dataset_major_materializations m
           ON m.dataset_id=d.id AND m.version_major=$2
         WHERE d.id=$1",
    )
    .bind(dataset_id)
    .bind(reference.major())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted)?;
    let tags = crate::datasets::load_dataset_tags(&state.pool, &[dataset_id])
        .await?
        .remove(&dataset_id)
        .unwrap_or_default();
    let provenance = crate::datasets::load_dataset_provenance(&state.pool, &[dataset_id])
        .await?
        .remove(&dataset_id)
        .unwrap_or_default();
    metadata_from_row(MetadataRow {
        installation_id,
        dataset_id,
        dataset_name: row.try_get("name")?,
        dataset_slug: row.try_get("slug")?,
        grain: row.try_get("grain")?,
        tags,
        provenance,
        major: reference.major(),
        output_fields: row.try_get("output_fields")?,
        materialization_state: row.try_get("materialization_state")?,
        scope_node_ids,
    })
}

fn boundary_allows(boundary: &CapabilityBoundary, scope_node_ids: &[Uuid]) -> bool {
    match boundary {
        CapabilityBoundary::Global => true,
        CapabilityBoundary::Scoped(allowed) => scope_node_ids.iter().any(|id| allowed.contains(id)),
        CapabilityBoundary::None => false,
    }
}

async fn execute_materialization(
    state: &AppState,
    account: &auth::AccountContext,
    request: DatasetExecutionRequest,
    metadata: DatasetMajorLineMetadata,
) -> ApiResult<DatasetExecutionResponse> {
    let offset = execution_offset(request.cursor.as_deref())?;
    if offset > 0 && !request.aggregates.is_empty() {
        return Err(ApiError::BadRequest(
            "Dataset aggregate execution does not accept a cursor".into(),
        ));
    }
    let known = metadata
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<BTreeSet<_>>();
    let projection = if request.projection.is_empty() {
        metadata
            .fields
            .iter()
            .map(|field| field.key.clone())
            .collect::<Vec<_>>()
    } else {
        request.projection.clone()
    };
    for key in projection
        .iter()
        .chain(&request.group_by)
        .chain(request.filters.iter().map(|f| &f.field_key))
        .chain(
            request
                .search
                .iter()
                .flat_map(|search| search.field_keys.iter()),
        )
    {
        if !known.contains(key.as_str()) {
            return Err(ApiError::BadRequest(format!(
                "Dataset field '{key}' is unavailable"
            )));
        }
    }
    for filter in &request.filters {
        let field_type = metadata
            .fields
            .iter()
            .find(|field| field.key == filter.field_key)
            .map(|field| field.field_type.as_str())
            .expect("known filter field");
        if !filter_operator_supported(filter.operator, field_type) {
            return Err(ApiError::BadRequest(format!(
                "Dataset filter operator is unavailable for field '{}'",
                filter.field_key
            )));
        }
    }
    let output_keys = if request.aggregates.is_empty() {
        if !request.group_by.is_empty() {
            return Err(ApiError::BadRequest("group_by requires aggregates".into()));
        }
        projection.clone()
    } else {
        let mut keys = request.group_by.clone();
        for aggregate in &request.aggregates {
            let field_type = aggregate.field_key.as_ref().and_then(|key| {
                metadata
                    .fields
                    .iter()
                    .find(|field| field.key == *key)
                    .map(|field| field.field_type.as_str())
            });
            let valid_function = aggregate_function_supported(aggregate.function, field_type);
            if aggregate.output_key.trim().is_empty()
                || aggregate
                    .field_key
                    .as_ref()
                    .is_some_and(|key| !known.contains(key.as_str()))
                || keys.contains(&aggregate.output_key)
                || !valid_function
            {
                return Err(ApiError::BadRequest("Dataset aggregate is invalid".into()));
            }
            keys.push(aggregate.output_key.clone());
        }
        keys
    };
    if request
        .group_missing_policies
        .keys()
        .any(|key| !request.group_by.contains(key))
    {
        return Err(ApiError::BadRequest(
            "Dataset grouping missing-value policy is invalid".into(),
        ));
    }
    for sort in &request.order_by {
        if !output_keys.contains(&sort.field_key) {
            return Err(ApiError::BadRequest(
                "Dataset sort field is unavailable".into(),
            ));
        }
    }
    let materialization = sqlx::query(
        "SELECT materialized_schema,materialized_table FROM dataset_major_materializations
         WHERE dataset_id=$1 AND version_major=$2 AND rebuild_status='ready'",
    )
    .bind(request.reference.dataset_id())
    .bind(request.reference.major())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ApiError::ServiceUnavailable("dataset_materialization_not_ready".into()))?;
    let schema: String = materialization.try_get("materialized_schema")?;
    let table: String = materialization.try_get("materialized_table")?;
    let tier_predicate = crate::analytics_authorization::tier_access_predicate_for_dataset(
        &state.pool,
        account,
        request.reference.dataset_id(),
        "datasets:read",
    )
    .await?;
    let mut query = QueryBuilder::<Postgres>::new("SELECT ");
    if request.aggregates.is_empty() {
        query.push("__row_id::text AS row_id");
        for key in &projection {
            query
                .push(",to_jsonb(")
                .push(quoted(key))
                .push(") AS ")
                .push(quoted(key));
        }
    } else {
        query.push("md5(concat_ws('|',");
        if request.group_by.is_empty() {
            query.push("'aggregate'");
        }
        for (index, key) in request.group_by.iter().enumerate() {
            if index > 0 {
                query.push(",");
            }
            push_group_expression(
                &mut query,
                key,
                request
                    .group_missing_policies
                    .get(key)
                    .copied()
                    .unwrap_or_default(),
            );
            query.push("::text");
        }
        query.push(")) AS row_id");
        for key in &request.group_by {
            query.push(",to_jsonb(");
            push_group_expression(
                &mut query,
                key,
                request
                    .group_missing_policies
                    .get(key)
                    .copied()
                    .unwrap_or_default(),
            );
            query.push(") AS ").push(quoted(key));
        }
        for aggregate in &request.aggregates {
            query.push(",to_jsonb(");
            match aggregate.function {
                DatasetAggregateFunction::Count => {
                    if let Some(key) = &aggregate.field_key {
                        if aggregate.missing_policy == DatasetMissingPolicy::Omit {
                            query
                                .push("COUNT(NULLIF(BTRIM(")
                                .push(quoted(key))
                                .push("::text),''))");
                        } else {
                            query.push("COUNT(*)");
                        }
                    } else {
                        query.push("COUNT(*)");
                    }
                }
                DatasetAggregateFunction::UniqueCount => {
                    let field = quoted(aggregate.field_key.as_deref().ok_or_else(|| {
                        ApiError::BadRequest("unique_count requires a field".into())
                    })?);
                    if aggregate.missing_policy == DatasetMissingPolicy::ExplicitMissing {
                        query
                            .push("COUNT(DISTINCT CASE WHEN NULLIF(BTRIM(")
                            .push(field.clone())
                            .push("::text),'') IS NULL THEN jsonb_build_array('missing') ELSE jsonb_build_array('value',NULLIF(BTRIM(")
                            .push(field)
                            .push("::text),'')) END)");
                    } else {
                        query
                            .push("COUNT(DISTINCT NULLIF(BTRIM(")
                            .push(field)
                            .push("::text),''))");
                    }
                }
                DatasetAggregateFunction::Sum => {
                    query.push("SUM(");
                    push_numeric_aggregate_operand(
                        &mut query,
                        aggregate
                            .field_key
                            .as_deref()
                            .ok_or_else(|| ApiError::BadRequest("sum requires a field".into()))?,
                        aggregate.missing_policy,
                    );
                    query.push(")");
                }
                DatasetAggregateFunction::Average => {
                    query.push("AVG(");
                    push_numeric_aggregate_operand(
                        &mut query,
                        aggregate.field_key.as_deref().ok_or_else(|| {
                            ApiError::BadRequest("average requires a field".into())
                        })?,
                        aggregate.missing_policy,
                    );
                    query.push(")");
                }
                DatasetAggregateFunction::Median => {
                    query.push("PERCENTILE_CONT(0.5) WITHIN GROUP (ORDER BY ");
                    push_numeric_aggregate_operand(
                        &mut query,
                        aggregate.field_key.as_deref().ok_or_else(|| {
                            ApiError::BadRequest("median requires a field".into())
                        })?,
                        aggregate.missing_policy,
                    );
                    query.push(")");
                }
                DatasetAggregateFunction::Minimum => {
                    query
                        .push("MIN(")
                        .push(quoted(aggregate.field_key.as_deref().ok_or_else(|| {
                            ApiError::BadRequest("minimum requires a field".into())
                        })?))
                        .push(")");
                }
                DatasetAggregateFunction::Maximum => {
                    query
                        .push("MAX(")
                        .push(quoted(aggregate.field_key.as_deref().ok_or_else(|| {
                            ApiError::BadRequest("maximum requires a field".into())
                        })?))
                        .push(")");
                }
                DatasetAggregateFunction::SingleValue => {
                    aggregate.field_key.as_deref().ok_or_else(|| {
                        ApiError::BadRequest("single_value requires a field".into())
                    })?;
                    query.push("CASE WHEN COUNT(*)=1 THEN MAX(");
                    push_numeric_aggregate_operand(
                        &mut query,
                        aggregate.field_key.as_deref().expect("field checked above"),
                        aggregate.missing_policy,
                    );
                    query.push(") ELSE NULL END");
                }
            };
            query.push(") AS ").push(quoted(&aggregate.output_key));
        }
    }
    query
        .push(" FROM ")
        .push(quoted(&schema))
        .push(".")
        .push(quoted(&table));
    query.push(" WHERE ").push(tier_predicate);
    for key in &request.group_by {
        if request
            .group_missing_policies
            .get(key)
            .copied()
            .unwrap_or_default()
            != DatasetMissingPolicy::ExplicitMissing
        {
            query
                .push(" AND NULLIF(BTRIM(")
                .push(quoted(key))
                .push("::text),'') IS NOT NULL");
        }
    }
    for filter in &request.filters {
        query.push(" AND ");
        let field = quoted(&filter.field_key);
        match filter.operator {
            DatasetFilterOperator::IsNull => {
                query.push(field).push(" IS NULL");
            }
            DatasetFilterOperator::IsNotNull => {
                query.push(field).push(" IS NOT NULL");
            }
            DatasetFilterOperator::IsEmpty => {
                query.push("NULLIF(").push(field).push("::text,'') IS NULL");
            }
            DatasetFilterOperator::IsNotEmpty => {
                query
                    .push("NULLIF(")
                    .push(field)
                    .push("::text,'') IS NOT NULL");
            }
            operator => {
                let value = filter.value.as_ref().ok_or_else(|| {
                    ApiError::BadRequest("Dataset filter value is required".into())
                })?;
                let text = match value {
                    Value::String(value) => value.clone(),
                    _ => value.to_string(),
                };
                let field_type = metadata
                    .fields
                    .iter()
                    .find(|candidate| candidate.key == filter.field_key)
                    .map(|candidate| candidate.field_type.as_str())
                    .unwrap_or("text");
                match operator {
                    DatasetFilterOperator::Eq => {
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(" = ");
                        push_comparison_bind(&mut query, text, field_type);
                    }
                    DatasetFilterOperator::NotEq => {
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(" <> ");
                        push_comparison_bind(&mut query, text, field_type);
                    }
                    DatasetFilterOperator::Contains => {
                        query
                            .push(field)
                            .push("::text ILIKE ")
                            .push_bind(format!("%{text}%"));
                    }
                    DatasetFilterOperator::NotContains => {
                        query
                            .push(field)
                            .push("::text NOT ILIKE ")
                            .push_bind(format!("%{text}%"));
                    }
                    DatasetFilterOperator::StartsWith => {
                        query
                            .push(field)
                            .push("::text ILIKE ")
                            .push_bind(format!("{text}%"));
                    }
                    DatasetFilterOperator::EndsWith => {
                        query
                            .push(field)
                            .push("::text ILIKE ")
                            .push_bind(format!("%{text}"));
                    }
                    DatasetFilterOperator::GreaterThan => {
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(" > ");
                        push_comparison_bind(&mut query, text, field_type);
                    }
                    DatasetFilterOperator::GreaterThanOrEqual => {
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(" >= ");
                        push_comparison_bind(&mut query, text, field_type);
                    }
                    DatasetFilterOperator::LessThan => {
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(" < ");
                        push_comparison_bind(&mut query, text, field_type);
                    }
                    DatasetFilterOperator::LessThanOrEqual => {
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(" <= ");
                        push_comparison_bind(&mut query, text, field_type);
                    }
                    DatasetFilterOperator::Between | DatasetFilterOperator::NotBetween => {
                        let (lower, upper) = text
                            .split_once("..")
                            .or_else(|| text.split_once(','))
                            .ok_or_else(|| {
                                ApiError::BadRequest(
                                    "Dataset between filter requires two bounds".into(),
                                )
                            })?;
                        push_comparison_operand(&mut query, &field, field_type);
                        if operator == DatasetFilterOperator::NotBetween {
                            query.push(" NOT BETWEEN ");
                        } else {
                            query.push(" BETWEEN ");
                        }
                        push_comparison_bind(&mut query, lower.trim().to_string(), field_type);
                        query.push(" AND ");
                        push_comparison_bind(&mut query, upper.trim().to_string(), field_type);
                    }
                    DatasetFilterOperator::IsEmpty
                    | DatasetFilterOperator::IsNotEmpty
                    | DatasetFilterOperator::IsNull
                    | DatasetFilterOperator::IsNotNull => {
                        unreachable!()
                    }
                };
            }
        }
    }
    if let Some(search) = &request.search {
        if search.field_keys.is_empty() || search.query.trim().is_empty() {
            return Err(ApiError::BadRequest("Dataset search is invalid".into()));
        }
        query.push(" AND (");
        for (index, key) in search.field_keys.iter().enumerate() {
            if index > 0 {
                query.push(" OR ");
            }
            query
                .push(quoted(key))
                .push("::text ILIKE ")
                .push_bind(format!("%{}%", search.query));
        }
        query.push(")");
    }
    if !request.aggregates.is_empty() && !request.group_by.is_empty() {
        query.push(" GROUP BY ");
        for (index, key) in request.group_by.iter().enumerate() {
            if index > 0 {
                query.push(",");
            }
            push_group_expression(
                &mut query,
                key,
                request
                    .group_missing_policies
                    .get(key)
                    .copied()
                    .unwrap_or_default(),
            );
        }
    }
    if !request.order_by.is_empty() {
        query.push(" ORDER BY ");
        for (index, sort) in request.order_by.iter().enumerate() {
            if index > 0 {
                query.push(",");
            }
            query
                .push(quoted(&sort.field_key))
                .push(match sort.direction {
                    DatasetSortDirection::Asc => " ASC",
                    DatasetSortDirection::Desc => " DESC",
                });
        }
    } else if request.aggregates.is_empty() {
        query.push(" ORDER BY __row_id ASC");
    }
    query
        .push(" LIMIT ")
        .push_bind(i64::from(request.limit) + 1)
        .push(" OFFSET ")
        .push_bind(i64::from(offset));
    let result = query.build().fetch_all(&state.pool).await?;
    let has_more = result.len() > request.limit as usize;
    let mut rows = result
        .into_iter()
        .map(|row| {
            let row_id = row.try_get("row_id")?;
            let values = output_keys
                .iter()
                .map(|key| {
                    row.try_get::<Option<Value>, _>(key.as_str())
                        .map(|value| (key.clone(), value))
                })
                .collect::<Result<BTreeMap<_, _>, sqlx::Error>>()?;
            Ok(DatasetExecutionRow { row_id, values })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    if has_more {
        rows.truncate(request.limit as usize);
    }
    if request
        .aggregates
        .iter()
        .any(|aggregate| aggregate.function == DatasetAggregateFunction::SingleValue)
        && rows
            .iter()
            .any(|row| row.values.get("summary_value").is_none_or(Option::is_none))
    {
        return Err(ApiError::BadRequest(
            "Dataset single-value execution requires exactly one value per result".into(),
        ));
    }
    let fields = output_keys
        .iter()
        .map(|key| {
            metadata
                .fields
                .iter()
                .find(|field| field.key == *key)
                .cloned()
                .unwrap_or(DatasetFieldContract {
                    key: key.clone(),
                    label: key.clone(),
                    field_type: "number".into(),
                    restriction_tier: "provider_enforced".into(),
                })
        })
        .collect();
    Ok(DatasetExecutionResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        materialization_state: "ready".into(),
        fields,
        rows,
        next_cursor: has_more.then(|| format!("offset:{}", offset + request.limit)),
    })
}

fn push_comparison_operand<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    field: &str,
    field_type: &str,
) {
    match field_type {
        "number" | "integer" | "decimal" => {
            query
                .push("NULLIF(")
                .push(field)
                .push("::text,'')::numeric");
        }
        "date" => {
            query.push("NULLIF(").push(field).push("::text,'')::date");
        }
        "datetime" | "timestamp" => {
            query
                .push("NULLIF(")
                .push(field)
                .push("::text,'')::timestamptz");
        }
        _ => {
            query.push(field).push("::text");
        }
    }
}

fn filter_operator_supported(operator: DatasetFilterOperator, field_type: &str) -> bool {
    match operator {
        DatasetFilterOperator::Contains
        | DatasetFilterOperator::NotContains
        | DatasetFilterOperator::StartsWith
        | DatasetFilterOperator::EndsWith
        | DatasetFilterOperator::IsEmpty
        | DatasetFilterOperator::IsNotEmpty => matches!(field_type, "text" | "string"),
        DatasetFilterOperator::GreaterThan
        | DatasetFilterOperator::GreaterThanOrEqual
        | DatasetFilterOperator::LessThan
        | DatasetFilterOperator::LessThanOrEqual
        | DatasetFilterOperator::Between
        | DatasetFilterOperator::NotBetween => matches!(
            field_type,
            "number" | "integer" | "decimal" | "date" | "datetime" | "timestamp"
        ),
        DatasetFilterOperator::Eq
        | DatasetFilterOperator::NotEq
        | DatasetFilterOperator::IsNull
        | DatasetFilterOperator::IsNotNull => true,
    }
}

fn aggregate_function_supported(
    function: DatasetAggregateFunction,
    field_type: Option<&str>,
) -> bool {
    match function {
        DatasetAggregateFunction::Count => true,
        DatasetAggregateFunction::UniqueCount
        | DatasetAggregateFunction::Minimum
        | DatasetAggregateFunction::Maximum => field_type.is_some(),
        DatasetAggregateFunction::Sum
        | DatasetAggregateFunction::Average
        | DatasetAggregateFunction::Median
        | DatasetAggregateFunction::SingleValue => {
            matches!(field_type, Some("number" | "integer" | "decimal"))
        }
    }
}

fn push_comparison_bind<'a>(
    query: &mut QueryBuilder<'a, Postgres>,
    value: String,
    field_type: &str,
) {
    query.push_bind(value);
    match field_type {
        "number" | "integer" | "decimal" => {
            query.push("::numeric");
        }
        "date" => {
            query.push("::date");
        }
        "datetime" | "timestamp" => {
            query.push("::timestamptz");
        }
        _ => {}
    }
}

fn execution_offset(cursor: Option<&str>) -> ApiResult<u32> {
    match cursor {
        None => Ok(0),
        Some(cursor) => cursor
            .strip_prefix("offset:")
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| (1..=MAX_DATASET_EXECUTION_OFFSET).contains(value))
            .ok_or_else(|| ApiError::BadRequest("Dataset execution cursor is invalid".into())),
    }
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn push_group_expression(
    query: &mut QueryBuilder<'_, Postgres>,
    field_key: &str,
    policy: DatasetMissingPolicy,
) {
    if policy == DatasetMissingPolicy::ExplicitMissing {
        query
            .push("COALESCE(NULLIF(BTRIM(")
            .push(quoted(field_key))
            .push("::text),''),'(Missing)')");
    } else {
        query
            .push("NULLIF(BTRIM(")
            .push(quoted(field_key))
            .push("::text),'')");
    }
}

fn push_numeric_aggregate_operand(
    query: &mut QueryBuilder<'_, Postgres>,
    field_key: &str,
    policy: DatasetMissingPolicy,
) {
    if policy == DatasetMissingPolicy::Zero {
        query.push("COALESCE(");
    }
    query
        .push("NULLIF(BTRIM(")
        .push(quoted(field_key))
        .push("::text),'')::numeric");
    if policy == DatasetMissingPolicy::Zero {
        query.push(",0::numeric)");
    }
}

fn restricted() -> ApiError {
    ApiError::Forbidden("dataset action unavailable".into())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use sqlx::{Execute, Postgres, QueryBuilder};
    use tessara_datasets_contract::{
        DatasetAggregateFunction, DatasetBootstrapValidationBatch, DatasetBootstrapValidationItem,
        DatasetCompatibilityFinding, DatasetFieldRequirement, DatasetFilterOperator,
        DatasetMajorLineReference, DatasetMissingPolicy, DatasetProvenanceItem,
        DatasetProvenanceSummary,
    };
    use tessara_module_contract::{AuthorizationAudienceV1, ServiceActionMethod};
    use uuid::Uuid;

    use super::{
        MAX_DATASET_EXECUTION_OFFSET, MetadataRow, aggregate_function_supported,
        compatibility_findings, execution_offset, filter_operator_supported, metadata_from_row,
        push_group_expression, push_numeric_aggregate_operand,
        validate_bootstrap_request_authorization,
    };

    #[test]
    fn compatibility_marks_non_ready_materialization_before_field_evaluation() {
        let fields = BTreeMap::from([("label", "text")]);
        let findings = compatibility_findings(
            "rebuilding",
            &fields,
            vec![
                DatasetFieldRequirement {
                    field_key: "missing".into(),
                    accepted_types: vec!["text".into()],
                },
                DatasetFieldRequirement {
                    field_key: "label".into(),
                    accepted_types: vec!["number".into()],
                },
            ],
        );

        assert_eq!(
            findings,
            vec![DatasetCompatibilityFinding {
                code: tessara_datasets_contract::DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY
                    .into(),
                field_key: "materialization_state".into(),
            }]
        );
    }

    #[test]
    fn core_provider_rejects_known_and_random_bootstrap_item_substitution_before_lookup() {
        let now = chrono::Utc::now();
        let installation_id = Uuid::from_u128(1);
        let module_instance_id = Uuid::from_u128(2);
        let input_digest =
            tessara_module_contract::ArtifactDigest::new(format!("sha256:{}", "1".repeat(64)))
                .expect("input digest");
        let plan_digest =
            tessara_module_contract::ArtifactDigest::new(format!("sha256:{}", "2".repeat(64)))
                .expect("plan digest");
        let reference = |dataset_id| {
            DatasetMajorLineReference::from_parts(installation_id, dataset_id, 1)
                .expect("Dataset reference")
        };
        let request_for =
            |dataset_id| tessara_composition::BootstrapDependencyValidationRequestV1 {
                schema_version:
                    tessara_composition::BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1,
                input_digest: input_digest.clone(),
                desired_revision: 8,
                apply_sequence: 3,
                target_plan_digest: plan_digest.clone(),
                payload: serde_json::to_value(DatasetBootstrapValidationBatch {
                    schema_version: tessara_datasets_contract::DATASET_CONTRACT_SCHEMA_VERSION,
                    items: vec![DatasetBootstrapValidationItem {
                        validation_key: "locked-item".into(),
                        reference: reference(dataset_id),
                        required_fields: Vec::new(),
                    }],
                })
                .expect("validation batch"),
            };
        let locked_request = request_for(Uuid::from_u128(10));
        let request_digest =
            tessara_composition::canonical_digest(&locked_request).expect("locked request digest");
        let authorization = tessara_composition::BootstrapDependencyValidationAuthorizationV1 {
            schema_version:
                tessara_composition::BOOTSTRAP_DEPENDENCY_VALIDATION_AUTHORIZATION_SCHEMA_VERSION_V1,
            installation_id,
            module_instance_id,
            module_definition_id: "tessara.components".into(),
            input_digest: input_digest.clone(),
            desired_revision: 8,
            apply_sequence: 3,
            target_plan_digest: plan_digest.clone(),
            dependency_binding: tessara_datasets_contract::DATASET_BINDING_KEY.into(),
            functional_contract: tessara_datasets_contract::DATASET_CONTRACT_ID.into(),
            functional_contract_version: tessara_datasets_contract::DATASET_CONTRACT_VERSION
                .parse()
                .expect("Dataset contract version"),
            action: tessara_datasets_contract::DATASET_BOOTSTRAP_VALIDATION_ACTION.into(),
            method: ServiceActionMethod::Post,
            path: tessara_datasets_contract::DATASET_BOOTSTRAP_VALIDATION_PATH.into(),
            audience: AuthorizationAudienceV1::CoreInstallation { installation_id },
            request_digest,
            correlation_id: Uuid::from_u128(3),
            jti: Uuid::from_u128(4),
            issued_at: now,
            expires_at: now + chrono::Duration::seconds(30),
        };
        assert!(
            validate_bootstrap_request_authorization(&authorization, &locked_request, now).is_ok()
        );

        for substituted in [
            request_for(Uuid::from_u128(11)),
            request_for(Uuid::new_v4()),
        ] {
            let error = validate_bootstrap_request_authorization(&authorization, &substituted, now)
                .expect_err("substituted validation item must fail before provider lookup");
            assert!(matches!(
                error,
                crate::error::ApiError::Forbidden(message)
                    if message == "dataset action unavailable"
            ));
        }
    }

    #[test]
    fn major_line_mapping_exposes_the_complete_picker_metadata() {
        let installation_id = Uuid::from_u128(1);
        let dataset_id = Uuid::from_u128(2);
        let metadata = metadata_from_row(MetadataRow {
            installation_id,
            dataset_id,
            dataset_name: "Enrollment outcomes".into(),
            dataset_slug: "enrollment-outcomes".into(),
            grain: "submission".into(),
            tags: vec!["outcomes".into()],
            provenance: DatasetProvenanceSummary {
                forms: vec![DatasetProvenanceItem {
                    id: Uuid::from_u128(3),
                    name: "Enrollment intake".into(),
                    slug: None,
                }],
                datasets: vec![DatasetProvenanceItem {
                    id: Uuid::from_u128(4),
                    name: "Enrollment activity".into(),
                    slug: Some("enrollment-activity".into()),
                }],
            },
            major: 3,
            output_fields: Some(serde_json::json!([{
                "key": "program",
                "label": "Program",
                "field_type": "text"
            }])),
            materialization_state: "ready".into(),
            scope_node_ids: vec![Uuid::from_u128(5)],
        })
        .expect("valid stored Dataset metadata");

        assert_eq!(metadata.reference.dataset_id(), dataset_id);
        assert_eq!(metadata.reference.major(), 3);
        assert_eq!(metadata.dataset_name, "Enrollment outcomes");
        assert_eq!(metadata.grain, "submission");
        assert_eq!(metadata.tags, ["outcomes"]);
        assert_eq!(metadata.provenance.forms[0].name, "Enrollment intake");
        assert_eq!(
            metadata.provenance.datasets[0].slug.as_deref(),
            Some("enrollment-activity")
        );
        assert_eq!(metadata.fields[0].label, "Program");
    }

    #[test]
    fn major_line_mapping_rejects_nonpositive_storage_versions() {
        for major in [i32::MIN, -1, 0] {
            let result = metadata_from_row(MetadataRow {
                installation_id: Uuid::from_u128(1),
                dataset_id: Uuid::from_u128(2),
                dataset_name: "Invalid".into(),
                dataset_slug: "invalid".into(),
                grain: "submission".into(),
                tags: Vec::new(),
                provenance: DatasetProvenanceSummary::default(),
                major,
                output_fields: None,
                materialization_state: "ready".into(),
                scope_node_ids: Vec::new(),
            });
            assert!(result.is_err(), "storage major {major} must be rejected");
        }
    }

    #[test]
    fn execution_cursor_is_exact_and_forward_only() {
        assert_eq!(execution_offset(None).expect("first page"), 0);
        assert_eq!(execution_offset(Some("offset:25")).expect("next page"), 25);
        let beyond_limit = format!("offset:{}", MAX_DATASET_EXECUTION_OFFSET + 1);
        for invalid in [
            "offset:0",
            "offset:-1",
            "offset:25:extra",
            "unexpected",
            &beyond_limit,
        ] {
            assert!(execution_offset(Some(invalid)).is_err(), "{invalid}");
        }
    }

    #[test]
    fn missing_policy_sql_preserves_explicit_groups_and_numeric_zeroes() {
        let mut query = QueryBuilder::<Postgres>::new("SELECT ");
        push_group_expression(&mut query, "status", DatasetMissingPolicy::ExplicitMissing);
        query.push(",");
        push_numeric_aggregate_operand(&mut query, "score", DatasetMissingPolicy::Zero);
        assert_eq!(
            query.build().sql(),
            "SELECT COALESCE(NULLIF(BTRIM(\"status\"::text),''),'(Missing)'),COALESCE(NULLIF(BTRIM(\"score\"::text),'')::numeric,0::numeric)"
        );
    }

    #[test]
    fn core_rejects_dataset_execution_type_mismatches_at_the_owner_boundary() {
        assert!(filter_operator_supported(
            DatasetFilterOperator::Contains,
            "text"
        ));
        assert!(!filter_operator_supported(
            DatasetFilterOperator::Contains,
            "number"
        ));
        assert!(filter_operator_supported(
            DatasetFilterOperator::Between,
            "date"
        ));
        assert!(!filter_operator_supported(
            DatasetFilterOperator::Between,
            "text"
        ));
        assert!(aggregate_function_supported(
            DatasetAggregateFunction::Median,
            Some("number")
        ));
        assert!(!aggregate_function_supported(
            DatasetAggregateFunction::Median,
            Some("text")
        ));
        assert!(!aggregate_function_supported(
            DatasetAggregateFunction::UniqueCount,
            None
        ));
        assert!(aggregate_function_supported(
            DatasetAggregateFunction::Count,
            None
        ));
    }
}
