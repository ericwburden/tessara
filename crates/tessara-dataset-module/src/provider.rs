//! Dataset-owned private provider surface.
//!
//! Every handler authenticates the raw request before typed deserialization,
//! consumes the caller's one-use service nonce in Dataset storage, and reads
//! only Dataset-owned metadata and materializations.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    body::Bytes,
    extract::{DefaultBodyLimit, State},
    http::HeaderMap,
    routing::post,
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use semver::Version;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, QueryBuilder, Row};
use tessara_composition::{
    BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1,
    BootstrapDependencyValidationAuthorizationV1, BootstrapDependencyValidationContextV1,
    BootstrapDependencyValidationRequestV1, canonical_digest,
};
use tessara_datasets_contract::{
    DATASET_BINDING_KEY, DATASET_BOOTSTRAP_VALIDATION_ACTION, DATASET_BOOTSTRAP_VALIDATION_PATH,
    DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY, DATASET_CONTRACT_ID,
    DATASET_CONTRACT_SCHEMA_VERSION, DATASET_CONTRACT_VERSION, DATASET_MAJOR_LINE_RESOURCE_TYPE,
    DATASET_OPERATIONAL_STATUS_CONTRACT_ID, DATASET_OPERATIONS_STATUS_ACTION,
    DATASET_OPERATIONS_STATUS_PATH, DATASET_RESOLVE_ACTION, DATASET_RESOLVE_PATH,
    DATASET_RESOURCE_OBSERVATION_CONTRACT_ID, DATASET_RESOURCE_TYPE,
    DATASET_REVERSE_CONTRACT_VERSION, DATASET_REVISION_RESOURCE_TYPE, DATASET_SOURCE_USAGE_ACTION,
    DATASET_SOURCE_USAGE_CONTRACT_ID, DATASET_SOURCE_USAGE_PATH, DATASET_SUMMARY_ACTION,
    DATASET_SUMMARY_PATH, DatasetAction, DatasetAggregateFunction, DatasetBootstrapValidationBatch,
    DatasetBootstrapValidationResponse, DatasetBootstrapValidationResult, DatasetCatalogRequest,
    DatasetCatalogResponse, DatasetCompatibilityFinding, DatasetCompatibilityRequest,
    DatasetCompatibilityResponse, DatasetDistinctValuesRequest, DatasetDistinctValuesResponse,
    DatasetExecutionRequest, DatasetExecutionResponse, DatasetExecutionRow, DatasetFieldContract,
    DatasetFieldRequirement, DatasetFilterOperator, DatasetFreshnessState,
    DatasetMajorLineMetadata, DatasetMajorLineReference, DatasetMissingPolicy,
    DatasetOperationsStatusItem, DatasetOperationsStatusRequest, DatasetOperationsStatusResponse,
    DatasetProductSourceV1, DatasetProvenanceItem, DatasetProvenanceSummary,
    DatasetProviderResultState, DatasetReadinessLabel, DatasetReference,
    DatasetResourceObservationRequest, DatasetResourceObservationResponse,
    DatasetRevisionReference, DatasetSchemaRequest, DatasetSortDirection, DatasetSourceUsageItem,
    DatasetSourceUsageRequest, DatasetSourceUsageResponse, DatasetSummaryRequest,
    DatasetSummaryResponse,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, ContractCompatibilityState, CoreServiceRequestV1,
    CoreServiceRequestValidationContextV1, FunctionalContractId, ModuleDefinitionId,
    ModuleInstanceOwnerState, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, OwnerDataState, ProviderAvailabilityState,
    ProviderContractIdentity, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationStrategy, ResourceObservationV1, ResourceOwner, ResourceOwnerState,
    ResourceResolutionV1, ResourceRevision, SecurityCapabilityId, ServiceActionMethod,
    SignedEnvelopeV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{DatasetModuleError, DatasetModuleState, READ_CAPABILITY, load_security_state};

const MAX_PROVIDER_BODY_BYTES: usize = 1024 * 1024;
const MAX_DATASET_EXECUTION_OFFSET: u32 = 1_000_000;

pub(super) fn routes() -> Router<DatasetModuleState> {
    Router::new()
        .route(DATASET_BOOTSTRAP_VALIDATION_PATH, post(validate_bootstrap))
        .route("/api/private/datasets/catalog", post(catalog))
        .route("/api/private/datasets/schema", post(schema))
        .route(
            "/api/private/datasets/distinct-values",
            post(distinct_values),
        )
        .route("/api/private/datasets/compatibility", post(compatibility))
        .route("/api/private/datasets/execute", post(execute))
        .route(DATASET_RESOLVE_PATH, post(resolve))
        .route(DATASET_SOURCE_USAGE_PATH, post(source_usage))
        .route(DATASET_OPERATIONS_STATUS_PATH, post(operations_status))
        .route(DATASET_SUMMARY_PATH, post(summary))
        .layer(DefaultBodyLimit::max(MAX_PROVIDER_BODY_BYTES))
}

struct ProviderAuthorization {
    grant: Option<SignedEnvelopeV1<AuthorizationGrantV3>>,
    installation_id: Uuid,
    module_instance_id: Uuid,
}

async fn authorize(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    path: &'static str,
    body: &[u8],
    expected_action: &'static str,
    expected_contract: &'static str,
) -> Result<ProviderAuthorization, DatasetModuleError> {
    let encoded = header(headers, "x-tessara-authorization")?;
    let grant: SignedEnvelopeV1<AuthorizationGrantV3> = decode(encoded)?;
    state
        .core_authorization_verifier
        .verify(&grant)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .filter(|security| security.enabled && security.document_state == "enabled")
        .ok_or_else(unavailable_security)?;
    let correlation_id = correlation_id(headers)?;
    let presenting_service = match &grant.payload.presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: *module_instance_id,
            module_definition_id: module_definition_id.clone(),
        },
        ModuleServicePrincipalV1::CoreGateway => ModuleServicePrincipalV1::CoreGateway,
    };
    if grant.payload.functional_contract.as_str() != expected_contract
        || grant.payload.resource_assertion.is_some()
        || !grant
            .payload
            .capability_scope_bindings
            .iter()
            .any(|binding| binding.capability.as_str() == READ_CAPABILITY)
    {
        return Err(DatasetModuleError::Forbidden);
    }
    grant
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: presenting_service.clone(),
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
                    .map_err(internal)?,
            },
            dependency_binding: grant.payload.dependency_binding.clone(),
            functional_contract: FunctionalContractId::new(expected_contract).map_err(internal)?,
            action: expected_action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    match &presenting_service {
        ModuleServicePrincipalV1::ModuleInstance { .. } => {
            validate_and_consume_service_request(
                state,
                headers,
                path,
                body,
                encoded,
                &presenting_service,
                grant.payload.jti,
                security.installation_id,
                correlation_id,
            )
            .await?;
        }
        ModuleServicePrincipalV1::CoreGateway => {
            if !core_gateway_action_allowed(path, expected_action) {
                return Err(DatasetModuleError::Forbidden);
            }
            validate_and_consume_core_service_request(
                state,
                headers,
                path,
                body,
                encoded,
                grant.payload.jti,
                security.installation_id,
                correlation_id,
            )
            .await?;
        }
    }
    Ok(ProviderAuthorization {
        grant: Some(grant),
        installation_id: security.installation_id,
        module_instance_id: security.module_instance_id,
    })
}

fn core_gateway_action_allowed(path: &str, action: &str) -> bool {
    matches!(
        (path, action),
        (DATASET_RESOLVE_PATH, DATASET_RESOLVE_ACTION)
            | (DATASET_SOURCE_USAGE_PATH, DATASET_SOURCE_USAGE_ACTION)
            | (
                DATASET_OPERATIONS_STATUS_PATH,
                DATASET_OPERATIONS_STATUS_ACTION
            )
            | (DATASET_SUMMARY_PATH, DATASET_SUMMARY_ACTION)
    )
}

#[allow(clippy::too_many_arguments)]
async fn validate_and_consume_service_request(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    path: &str,
    body: &[u8],
    inbound_authorization: &str,
    presenting_service: &ModuleServicePrincipalV1,
    authorization_jti: Uuid,
    installation_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), DatasetModuleError> {
    let (module_instance_id, module_definition_id) = match presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => (*module_instance_id, module_definition_id.clone()),
        ModuleServicePrincipalV1::CoreGateway => return Err(DatasetModuleError::Forbidden),
    };
    let service: SignedEnvelopeV1<ModuleServiceRequestV1> =
        decode(header(headers, "x-tessara-module-service-request")?)?;
    state
        .service_identity_registry
        .module_service_verifier(&module_definition_id)
        .map_err(|_| DatasetModuleError::Forbidden)?
        .verify(&service)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    service
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id,
            module_instance_id,
            module_definition_id,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(inbound_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let consumed = sqlx::query(
        "INSERT INTO dataset_consumed_service_nonces
         (module_instance_id,nonce,authorization_jti,correlation_id,issued_at)
         VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(service.payload.module_instance_id)
    .bind(service.payload.nonce)
    .bind(authorization_jti)
    .bind(&service.payload.correlation_id)
    .bind(service.payload.issued_at)
    .execute(&state.pool)
    .await?;
    if consumed.rows_affected() != 1 {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn validate_and_consume_core_service_request(
    state: &DatasetModuleState,
    headers: &HeaderMap,
    path: &str,
    body: &[u8],
    inbound_authorization: &str,
    authorization_jti: Uuid,
    installation_id: Uuid,
    correlation_id: Uuid,
) -> Result<(), DatasetModuleError> {
    let service: SignedEnvelopeV1<CoreServiceRequestV1> =
        decode(header(headers, "x-tessara-core-service-request")?)?;
    state
        .core_service_request_verifier
        .verify(&service)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    service
        .payload
        .validate_for(&CoreServiceRequestValidationContextV1 {
            installation_id,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(inbound_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let consumed = sqlx::query(
        "INSERT INTO dataset_consumed_core_service_nonces
         (installation_id,nonce,authorization_jti,correlation_id,issued_at)
         VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(service.payload.installation_id)
    .bind(service.payload.nonce)
    .bind(authorization_jti)
    .bind(&service.payload.correlation_id)
    .bind(service.payload.issued_at)
    .execute(&state.pool)
    .await?;
    if consumed.rows_affected() != 1 {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(())
}

async fn catalog(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetCatalogResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        "/api/private/datasets/catalog",
        &body,
        "datasets.catalog",
        DATASET_CONTRACT_ID,
    )
    .await?;
    let request: DatasetCatalogRequest = parse(&body, "Dataset catalog request is invalid")?;
    if request.action != DatasetAction::Catalog {
        return Err(DatasetModuleError::Forbidden);
    }
    let rows = sqlx::query(
        "SELECT DISTINCT ON (d.id,r.version_major)
                d.id AS dataset_id,d.name,d.slug,d.grain,r.id AS revision_id,
                r.version_major,r.output_fields,
                COALESCE(m.rebuild_status,'unavailable') AS materialization_state
         FROM datasets d
         JOIN dataset_revisions r ON r.dataset_id=d.id
         LEFT JOIN dataset_major_materializations m
           ON m.dataset_id=d.id AND m.version_major=r.version_major
         WHERE d.lifecycle_state <> 'tombstoned'
           AND r.lifecycle_state <> 'tombstoned'
           AND r.version_major IS NOT NULL
           AND r.status IN ('published','superseded')
         ORDER BY d.id,r.version_major,r.version_number DESC",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut datasets = Vec::new();
    for row in rows {
        let revision_id: Uuid = row.try_get("revision_id")?;
        let scope_node_ids = revision_scope(&state, revision_id).await?;
        if !scope_authorized(&normal_grant(&authorization)?.payload, &scope_node_ids) {
            continue;
        }
        datasets.push(
            metadata_from_row(
                &state,
                &authorization,
                MetadataRow {
                    dataset_id: row.try_get("dataset_id")?,
                    revision_id,
                    dataset_name: row.try_get("name")?,
                    dataset_slug: row.try_get("slug")?,
                    grain: row.try_get("grain")?,
                    major: row.try_get("version_major")?,
                    output_fields: row.try_get("output_fields")?,
                    materialization_state: row.try_get("materialization_state")?,
                    scope_node_ids,
                },
            )
            .await?,
        );
    }
    Ok(Json(DatasetCatalogResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        datasets,
    }))
}

async fn schema(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetMajorLineMetadata>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        "/api/private/datasets/schema",
        &body,
        "datasets.schema",
        DATASET_CONTRACT_ID,
    )
    .await?;
    let request: DatasetSchemaRequest = parse(&body, "Dataset schema request is invalid")?;
    if request.action != DatasetAction::ResolveSchema {
        return Err(DatasetModuleError::Forbidden);
    }
    load_authorized_metadata(&state, &authorization, &request.reference)
        .await
        .map(Json)
}

async fn distinct_values(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetDistinctValuesResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        "/api/private/datasets/distinct-values",
        &body,
        "datasets.distinct_values",
        DATASET_CONTRACT_ID,
    )
    .await?;
    let request: DatasetDistinctValuesRequest =
        parse(&body, "Dataset distinct-value request is invalid")?;
    if request.action != DatasetAction::DistinctValues
        || request.field_key.trim().is_empty()
        || !(1..=200).contains(&request.limit)
    {
        return Err(DatasetModuleError::BadRequest(
            "Dataset distinct-value request is invalid".into(),
        ));
    }
    let metadata = load_authorized_metadata(&state, &authorization, &request.reference).await?;
    if metadata.materialization_state != "ready"
        || !metadata
            .fields
            .iter()
            .any(|field| field.key == request.field_key)
    {
        return Err(DatasetModuleError::Forbidden);
    }
    let materialization = load_ready_materialization(
        &state,
        request.reference.dataset_id(),
        request.reference.major(),
    )
    .await?;
    let mut query = QueryBuilder::<Postgres>::new("SELECT DISTINCT to_jsonb(");
    query
        .push(quoted(&request.field_key))
        .push(") AS value FROM ")
        .push(quoted(&materialization.0))
        .push(".")
        .push(quoted(&materialization.1))
        .push(" WHERE ");
    push_row_access_predicate(&mut query, &normal_grant(&authorization)?.payload);
    query
        .push(" AND ")
        .push(quoted(&request.field_key))
        .push(" IS NOT NULL ORDER BY value LIMIT ")
        .push_bind(i64::from(request.limit));
    let values = query
        .build()
        .fetch_all(&state.pool)
        .await?
        .into_iter()
        .map(|row| row.try_get("value"))
        .collect::<Result<Vec<Value>, sqlx::Error>>()?;
    Ok(Json(DatasetDistinctValuesResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        values,
    }))
}

async fn compatibility(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetCompatibilityResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        "/api/private/datasets/compatibility",
        &body,
        "datasets.compatibility",
        DATASET_CONTRACT_ID,
    )
    .await?;
    let request: DatasetCompatibilityRequest =
        parse(&body, "Dataset compatibility request is invalid")?;
    if request.action != DatasetAction::CheckCompatibility {
        return Err(DatasetModuleError::Forbidden);
    }
    let metadata = load_authorized_metadata(&state, &authorization, &request.reference).await?;
    Ok(Json(compatibility_response(
        &metadata,
        request.required_fields,
    )))
}

fn compatibility_response(
    metadata: &DatasetMajorLineMetadata,
    requirements: Vec<DatasetFieldRequirement>,
) -> DatasetCompatibilityResponse {
    let fields = metadata
        .fields
        .iter()
        .map(|field| (field.key.as_str(), field.field_type.as_str()))
        .collect::<BTreeMap<_, _>>();
    let findings = compatibility_findings(&metadata.materialization_state, &fields, requirements);
    DatasetCompatibilityResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        compatible: findings.is_empty(),
        findings,
    }
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
    requirements
        .into_iter()
        .filter_map(
            |requirement| match fields.get(requirement.field_key.as_str()) {
                None => Some(DatasetCompatibilityFinding {
                    code: "field_missing".into(),
                    field_key: requirement.field_key,
                }),
                Some(actual)
                    if !requirement
                        .accepted_types
                        .iter()
                        .any(|accepted| accepted == actual) =>
                {
                    Some(DatasetCompatibilityFinding {
                        code: "field_type_incompatible".into(),
                        field_key: requirement.field_key,
                    })
                }
                Some(_) => None,
            },
        )
        .collect()
}

async fn validate_bootstrap(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetBootstrapValidationResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let encoded = header(&headers, "x-tessara-bootstrap-validation-authorization")?;
    let authorization: SignedEnvelopeV1<BootstrapDependencyValidationAuthorizationV1> =
        decode(encoded)?;
    state
        .core_bootstrap_validation_verifier
        .verify(&authorization)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .filter(|security| security.enabled && security.document_state == "enabled")
        .ok_or_else(unavailable_security)?;
    let correlation_id = correlation_id(&headers)?;
    if correlation_id != authorization.payload.correlation_id {
        return Err(DatasetModuleError::Forbidden);
    }
    let presenting_service = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: authorization.payload.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(&authorization.payload.module_definition_id)
            .map_err(|_| DatasetModuleError::Forbidden)?,
    };
    validate_and_consume_service_request(
        &state,
        &headers,
        DATASET_BOOTSTRAP_VALIDATION_PATH,
        &body,
        encoded,
        &presenting_service,
        authorization.payload.jti,
        security.installation_id,
        correlation_id,
    )
    .await?;
    let request: BootstrapDependencyValidationRequestV1 = parse_restricted(&body)?;
    if request.schema_version != BOOTSTRAP_DEPENDENCY_VALIDATION_REQUEST_SCHEMA_VERSION_V1
        || request.desired_revision == 0
        || request.apply_sequence == 0
    {
        return Err(DatasetModuleError::Forbidden);
    }
    let request_digest = canonical_digest(&request).map_err(|_| DatasetModuleError::Forbidden)?;
    let audience = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
            .map_err(internal)?,
    };
    authorization
        .payload
        .validate_for(&BootstrapDependencyValidationContextV1 {
            installation_id: security.installation_id,
            module_instance_id: authorization.payload.module_instance_id,
            module_definition_id: &authorization.payload.module_definition_id,
            input_digest: &request.input_digest,
            desired_revision: request.desired_revision,
            apply_sequence: request.apply_sequence,
            target_plan_digest: &request.target_plan_digest,
            dependency_binding: DATASET_BINDING_KEY,
            functional_contract: DATASET_CONTRACT_ID,
            functional_contract_version: &Version::parse(DATASET_CONTRACT_VERSION)
                .expect("static Dataset contract version"),
            action: DATASET_BOOTSTRAP_VALIDATION_ACTION,
            method: ServiceActionMethod::Post,
            path: DATASET_BOOTSTRAP_VALIDATION_PATH,
            audience: &audience,
            request_digest: &request_digest,
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let batch: DatasetBootstrapValidationBatch =
        serde_json::from_value(request.payload).map_err(|_| DatasetModuleError::Forbidden)?;
    if batch.schema_version != DATASET_CONTRACT_SCHEMA_VERSION || batch.items.is_empty() {
        return Err(DatasetModuleError::Forbidden);
    }
    let bootstrap_authorization = ProviderAuthorization {
        grant: None,
        installation_id: security.installation_id,
        module_instance_id: security.module_instance_id,
    };
    let mut validation_keys = BTreeSet::new();
    let mut results = Vec::with_capacity(batch.items.len());
    for item in batch.items {
        if item.validation_key.trim().is_empty()
            || !validation_keys.insert(item.validation_key.clone())
        {
            return Err(DatasetModuleError::Forbidden);
        }
        let metadata =
            load_metadata_unscoped(&state, &bootstrap_authorization, &item.reference).await?;
        let compatibility = compatibility_response(&metadata, item.required_fields);
        results.push(DatasetBootstrapValidationResult {
            validation_key: item.validation_key,
            metadata,
            compatibility,
        });
    }
    Ok(Json(DatasetBootstrapValidationResponse {
        schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
        results,
    }))
}

#[derive(Deserialize)]
struct StoredField {
    key: String,
    label: String,
    field_type: String,
}

struct MetadataRow {
    dataset_id: Uuid,
    revision_id: Uuid,
    dataset_name: String,
    dataset_slug: String,
    grain: String,
    major: i32,
    output_fields: Value,
    materialization_state: String,
    scope_node_ids: Vec<Uuid>,
}

async fn load_authorized_metadata(
    state: &DatasetModuleState,
    authorization: &ProviderAuthorization,
    reference: &DatasetMajorLineReference,
) -> Result<DatasetMajorLineMetadata, DatasetModuleError> {
    let metadata = load_metadata_unscoped(state, authorization, reference).await?;
    if !scope_authorized(
        &normal_grant(authorization)?.payload,
        &metadata.scope_node_ids,
    ) {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(metadata)
}

fn normal_grant(
    authorization: &ProviderAuthorization,
) -> Result<&SignedEnvelopeV1<AuthorizationGrantV3>, DatasetModuleError> {
    authorization
        .grant
        .as_ref()
        .ok_or_else(|| DatasetModuleError::Internal("normal provider grant is unavailable".into()))
}

async fn load_metadata_unscoped(
    state: &DatasetModuleState,
    authorization: &ProviderAuthorization,
    reference: &DatasetMajorLineReference,
) -> Result<DatasetMajorLineMetadata, DatasetModuleError> {
    require_major_reference_owner(reference, authorization)?;
    let row = sqlx::query(
        "SELECT d.name,d.slug,d.grain,r.id AS revision_id,r.output_fields,
                COALESCE(m.rebuild_status,'unavailable') AS materialization_state
         FROM datasets d
         JOIN LATERAL (
           SELECT id,output_fields FROM dataset_revisions
           WHERE dataset_id=d.id AND version_major=$2
             AND lifecycle_state <> 'tombstoned'
             AND status IN ('published','superseded')
           ORDER BY version_number DESC LIMIT 1
         ) r ON true
         LEFT JOIN dataset_major_materializations m
           ON m.dataset_id=d.id AND m.version_major=$2
         WHERE d.id=$1 AND d.lifecycle_state <> 'tombstoned'",
    )
    .bind(reference.dataset_id())
    .bind(reference.major())
    .fetch_optional(&state.pool)
    .await?
    .ok_or(DatasetModuleError::Forbidden)?;
    let revision_id: Uuid = row.try_get("revision_id")?;
    let scope_node_ids = revision_scope(state, revision_id).await?;
    metadata_from_row(
        state,
        authorization,
        MetadataRow {
            dataset_id: reference.dataset_id(),
            revision_id,
            dataset_name: row.try_get("name")?,
            dataset_slug: row.try_get("slug")?,
            grain: row.try_get("grain")?,
            major: reference.major(),
            output_fields: row.try_get("output_fields")?,
            materialization_state: row.try_get("materialization_state")?,
            scope_node_ids,
        },
    )
    .await
}

async fn metadata_from_row(
    state: &DatasetModuleState,
    authorization: &ProviderAuthorization,
    mut row: MetadataRow,
) -> Result<DatasetMajorLineMetadata, DatasetModuleError> {
    row.scope_node_ids.sort_unstable();
    row.scope_node_ids.dedup();
    let fields = serde_json::from_value::<Vec<StoredField>>(row.output_fields)
        .map_err(internal)?
        .into_iter()
        .map(|field| DatasetFieldContract {
            key: field.key,
            label: field.label,
            field_type: field.field_type,
            restriction_tier: "provider_enforced".into(),
        })
        .collect();
    let tags = sqlx::query_scalar::<_, String>(
        "SELECT tag FROM dataset_tags WHERE dataset_id=$1 ORDER BY tag",
    )
    .bind(row.dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let provenance = revision_provenance(state, row.revision_id).await?;
    Ok(DatasetMajorLineMetadata {
        reference: DatasetMajorLineReference::from_parts(
            authorization.installation_id,
            authorization.module_instance_id,
            row.dataset_id,
            row.major,
        )
        .map_err(internal)?,
        dataset_name: row.dataset_name,
        dataset_slug: row.dataset_slug,
        grain: row.grain,
        tags,
        provenance,
        materialization_state: row.materialization_state,
        fields,
        scope_node_ids: row.scope_node_ids,
    })
}

async fn revision_scope(
    state: &DatasetModuleState,
    revision_id: Uuid,
) -> Result<Vec<Uuid>, DatasetModuleError> {
    Ok(sqlx::query_scalar(
        "SELECT node_id FROM dataset_revision_scope_nodes
         WHERE revision_id=$1 ORDER BY node_id",
    )
    .bind(revision_id)
    .fetch_all(&state.pool)
    .await?)
}

async fn revision_provenance(
    state: &DatasetModuleState,
    revision_id: Uuid,
) -> Result<DatasetProvenanceSummary, DatasetModuleError> {
    let rows = sqlx::query(
        "SELECT source_reference,source_name,source_slug
         FROM dataset_revision_sources WHERE revision_id=$1 ORDER BY position,source_alias",
    )
    .bind(revision_id)
    .fetch_all(&state.pool)
    .await?;
    let mut forms = BTreeMap::<Uuid, DatasetProvenanceItem>::new();
    let mut datasets = BTreeMap::<Uuid, DatasetProvenanceItem>::new();
    for row in rows {
        let reference: DatasetProductSourceV1 =
            serde_json::from_value(row.try_get("source_reference")?).map_err(internal)?;
        let name: String = row.try_get("source_name")?;
        let slug: Option<String> = row.try_get("source_slug")?;
        match reference {
            DatasetProductSourceV1::Form { form_id, .. } => {
                let id = Uuid::parse_str(&form_id).map_err(internal)?;
                forms
                    .entry(id)
                    .or_insert(DatasetProvenanceItem { id, name, slug });
            }
            DatasetProductSourceV1::Dataset { dataset_id, .. }
            | DatasetProductSourceV1::DatasetMajor { dataset_id, .. } => {
                let id = Uuid::parse_str(&dataset_id).map_err(internal)?;
                datasets
                    .entry(id)
                    .or_insert(DatasetProvenanceItem { id, name, slug });
            }
        }
    }
    Ok(DatasetProvenanceSummary {
        forms: forms.into_values().collect(),
        datasets: datasets.into_values().collect(),
    })
}

fn require_major_reference_owner(
    reference: &DatasetMajorLineReference,
    authorization: &ProviderAuthorization,
) -> Result<(), DatasetModuleError> {
    if reference.reference().installation_id() != authorization.installation_id
        || reference.reference().owner()
            != &(ResourceOwner::ModuleInstance {
                installation_id: authorization.installation_id,
                module_instance_id: authorization.module_instance_id,
            })
    {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(())
}

async fn execute(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetExecutionResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        "/api/private/datasets/execute",
        &body,
        "datasets.execute",
        DATASET_CONTRACT_ID,
    )
    .await?;
    let request: DatasetExecutionRequest = parse(&body, "Dataset execution request is invalid")?;
    if request.action != DatasetAction::Execute || request.limit == 0 || request.limit > 1_000 {
        return Err(DatasetModuleError::BadRequest(
            "Dataset execution request is invalid".into(),
        ));
    }
    let metadata = load_authorized_metadata(&state, &authorization, &request.reference).await?;
    if metadata.materialization_state != "ready" {
        return Ok(Json(DatasetExecutionResponse {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            materialization_state: metadata.materialization_state,
            fields: metadata.fields,
            rows: Vec::new(),
            next_cursor: None,
        }));
    }
    execute_materialization(&state, &authorization, request, metadata)
        .await
        .map(Json)
}

async fn execute_materialization(
    state: &DatasetModuleState,
    authorization: &ProviderAuthorization,
    request: DatasetExecutionRequest,
    metadata: DatasetMajorLineMetadata,
) -> Result<DatasetExecutionResponse, DatasetModuleError> {
    let offset = execution_offset(request.cursor.as_deref())?;
    if offset > 0 && !request.aggregates.is_empty() {
        return Err(DatasetModuleError::BadRequest(
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
        .chain(request.filters.iter().map(|filter| &filter.field_key))
        .chain(
            request
                .search
                .iter()
                .flat_map(|search| search.field_keys.iter()),
        )
    {
        if !known.contains(key.as_str()) {
            return Err(DatasetModuleError::BadRequest(format!(
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
            return Err(DatasetModuleError::BadRequest(format!(
                "Dataset filter operator is unavailable for field '{}'",
                filter.field_key
            )));
        }
    }
    let output_keys = if request.aggregates.is_empty() {
        if !request.group_by.is_empty() {
            return Err(DatasetModuleError::BadRequest(
                "group_by requires aggregates".into(),
            ));
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
            if aggregate.output_key.trim().is_empty()
                || aggregate
                    .field_key
                    .as_ref()
                    .is_some_and(|key| !known.contains(key.as_str()))
                || keys.contains(&aggregate.output_key)
                || !aggregate_function_supported(aggregate.function, field_type)
            {
                return Err(DatasetModuleError::BadRequest(
                    "Dataset aggregate is invalid".into(),
                ));
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
        return Err(DatasetModuleError::BadRequest(
            "Dataset grouping missing-value policy is invalid".into(),
        ));
    }
    for sort in &request.order_by {
        if !output_keys.contains(&sort.field_key) {
            return Err(DatasetModuleError::BadRequest(
                "Dataset sort field is unavailable".into(),
            ));
        }
    }
    let (schema, table) = load_ready_materialization(
        state,
        request.reference.dataset_id(),
        request.reference.major(),
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
                        DatasetModuleError::BadRequest("unique_count requires a field".into())
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
                        aggregate.field_key.as_deref().ok_or_else(|| {
                            DatasetModuleError::BadRequest("sum requires a field".into())
                        })?,
                        aggregate.missing_policy,
                    );
                    query.push(")");
                }
                DatasetAggregateFunction::Average => {
                    query.push("AVG(");
                    push_numeric_aggregate_operand(
                        &mut query,
                        aggregate.field_key.as_deref().ok_or_else(|| {
                            DatasetModuleError::BadRequest("average requires a field".into())
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
                            DatasetModuleError::BadRequest("median requires a field".into())
                        })?,
                        aggregate.missing_policy,
                    );
                    query.push(")");
                }
                DatasetAggregateFunction::Minimum => {
                    query
                        .push("MIN(")
                        .push(quoted(aggregate.field_key.as_deref().ok_or_else(|| {
                            DatasetModuleError::BadRequest("minimum requires a field".into())
                        })?))
                        .push(")");
                }
                DatasetAggregateFunction::Maximum => {
                    query
                        .push("MAX(")
                        .push(quoted(aggregate.field_key.as_deref().ok_or_else(|| {
                            DatasetModuleError::BadRequest("maximum requires a field".into())
                        })?))
                        .push(")");
                }
                DatasetAggregateFunction::SingleValue => {
                    let field = aggregate.field_key.as_deref().ok_or_else(|| {
                        DatasetModuleError::BadRequest("single_value requires a field".into())
                    })?;
                    query.push("CASE WHEN COUNT(*)=1 THEN MAX(");
                    push_numeric_aggregate_operand(&mut query, field, aggregate.missing_policy);
                    query.push(") ELSE NULL END");
                }
            }
            query.push(") AS ").push(quoted(&aggregate.output_key));
        }
    }
    query
        .push(" FROM ")
        .push(quoted(&schema))
        .push(".")
        .push(quoted(&table))
        .push(" WHERE ");
    push_row_access_predicate(&mut query, &normal_grant(authorization)?.payload);
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
                    DatasetModuleError::BadRequest("Dataset filter value is required".into())
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
                    .expect("known filter field");
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
                                DatasetModuleError::BadRequest(
                                    "Dataset between filter requires two bounds".into(),
                                )
                            })?;
                        push_comparison_operand(&mut query, &field, field_type);
                        query.push(if operator == DatasetFilterOperator::NotBetween {
                            " NOT BETWEEN "
                        } else {
                            " BETWEEN "
                        });
                        push_comparison_bind(&mut query, lower.trim().to_string(), field_type);
                        query.push(" AND ");
                        push_comparison_bind(&mut query, upper.trim().to_string(), field_type);
                    }
                    DatasetFilterOperator::IsEmpty
                    | DatasetFilterOperator::IsNotEmpty
                    | DatasetFilterOperator::IsNull
                    | DatasetFilterOperator::IsNotNull => unreachable!(),
                }
            }
        }
    }
    if let Some(search) = &request.search {
        if search.field_keys.is_empty() || search.query.trim().is_empty() {
            return Err(DatasetModuleError::BadRequest(
                "Dataset search is invalid".into(),
            ));
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
        && rows.iter().any(|row| {
            request.aggregates.iter().any(|aggregate| {
                aggregate.function == DatasetAggregateFunction::SingleValue
                    && row
                        .values
                        .get(&aggregate.output_key)
                        .is_none_or(Option::is_none)
            })
        })
    {
        return Err(DatasetModuleError::BadRequest(
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

async fn load_ready_materialization(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    major: i32,
) -> Result<(String, String), DatasetModuleError> {
    sqlx::query_as(
        "SELECT materialized_schema,materialized_table
         FROM dataset_major_materializations
         WHERE dataset_id=$1 AND version_major=$2 AND rebuild_status='ready'
           AND materialized_schema IS NOT NULL AND materialized_table IS NOT NULL",
    )
    .bind(dataset_id)
    .bind(major)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| DatasetModuleError::Unavailable("dataset_materialization_not_ready".into()))
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

fn execution_offset(cursor: Option<&str>) -> Result<u32, DatasetModuleError> {
    match cursor {
        None => Ok(0),
        Some(cursor) => cursor
            .strip_prefix("offset:")
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| (1..=MAX_DATASET_EXECUTION_OFFSET).contains(value))
            .ok_or_else(|| {
                DatasetModuleError::BadRequest("Dataset execution cursor is invalid".into())
            }),
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

async fn resolve(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetResourceObservationResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        DATASET_RESOLVE_PATH,
        &body,
        "datasets.resolve",
        DATASET_RESOURCE_OBSERVATION_CONTRACT_ID,
    )
    .await?;
    let request: DatasetResourceObservationRequest =
        parse(&body, "Dataset resource-observation request is invalid")?;
    if request.schema_version != 1 {
        return Err(DatasetModuleError::BadRequest(
            "Dataset resource-observation request is invalid".into(),
        ));
    }
    let reference = &request.reference;
    if !reference_targets_provider(reference, &authorization) {
        return restricted_observation(ResourceAccessState::NotEvaluated).map(Json);
    }
    let loaded = match reference.resource_type().as_str() {
        DATASET_RESOURCE_TYPE => {
            let Ok(reference) = DatasetReference::new(reference.clone()) else {
                return restricted_observation(ResourceAccessState::NotEvaluated).map(Json);
            };
            load_dataset_observation(&state, reference.dataset_id()).await?
        }
        DATASET_REVISION_RESOURCE_TYPE => {
            let Ok(reference) = DatasetRevisionReference::new(reference.clone()) else {
                return restricted_observation(ResourceAccessState::NotEvaluated).map(Json);
            };
            load_revision_observation(&state, reference.revision_id()).await?
        }
        DATASET_MAJOR_LINE_RESOURCE_TYPE => {
            let Ok(reference) = DatasetMajorLineReference::new(reference.clone()) else {
                return restricted_observation(ResourceAccessState::NotEvaluated).map(Json);
            };
            load_major_observation(&state, reference.dataset_id(), reference.major()).await?
        }
        _ => return restricted_observation(ResourceAccessState::NotEvaluated).map(Json),
    };
    let Some(loaded) = loaded else {
        return restricted_observation(ResourceAccessState::Unauthorized).map(Json);
    };
    if !scope_authorized(
        &normal_grant(&authorization)?.payload,
        &loaded.scope_node_ids,
    ) {
        return restricted_observation(ResourceAccessState::Unauthorized).map(Json);
    }
    let contract = match reference.resource_type().as_str() {
        DATASET_MAJOR_LINE_RESOURCE_TYPE => DATASET_CONTRACT_ID,
        DATASET_RESOURCE_TYPE => "tessara.datasets.dataset",
        DATASET_REVISION_RESOURCE_TYPE => "tessara.datasets.dataset-revision",
        _ => unreachable!("resource type was matched above"),
    };
    let observation = ResourceObservationV1::new(
        reference.clone(),
        ProviderContractIdentity::new(
            FunctionalContractId::new(contract).map_err(internal)?,
            Version::parse(if contract == DATASET_CONTRACT_ID {
                DATASET_CONTRACT_VERSION
            } else {
                DATASET_REVERSE_CONTRACT_VERSION
            })
            .map_err(internal)?,
        ),
        ResourceObservationStrategy::LiveResolutionWithRevision,
        ResourceRevision::new(loaded.resource_revision as u64).map_err(internal)?,
    );
    Ok(Json(DatasetResourceObservationResponse {
        schema_version: 1,
        resolution: ResourceResolutionV1::authorized(
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::Live,
                data_state: OwnerDataState::Retained,
            },
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: loaded.lifecycle_state,
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        )
        .map_err(internal)?,
        observation: Some(observation),
    }))
}

struct ObservationRow {
    resource_revision: i64,
    lifecycle_state: String,
    scope_node_ids: Vec<Uuid>,
}

async fn load_dataset_observation(
    state: &DatasetModuleState,
    dataset_id: Uuid,
) -> Result<Option<ObservationRow>, DatasetModuleError> {
    let row = sqlx::query(
        "SELECT d.resource_revision,d.lifecycle_state,
                COALESCE(array_agg(s.node_id ORDER BY s.node_id)
                    FILTER (WHERE s.node_id IS NOT NULL),'{}'::uuid[]) AS scope_node_ids
         FROM datasets d LEFT JOIN dataset_scope_nodes s ON s.dataset_id=d.id
         WHERE d.id=$1 GROUP BY d.id",
    )
    .bind(dataset_id)
    .fetch_optional(&state.pool)
    .await?;
    row.map(observation_row).transpose()
}

async fn load_revision_observation(
    state: &DatasetModuleState,
    revision_id: Uuid,
) -> Result<Option<ObservationRow>, DatasetModuleError> {
    let row = sqlx::query(
        "SELECT r.resource_revision,r.lifecycle_state,
                COALESCE(array_agg(s.node_id ORDER BY s.node_id)
                    FILTER (WHERE s.node_id IS NOT NULL),'{}'::uuid[]) AS scope_node_ids
         FROM dataset_revisions r
         LEFT JOIN dataset_revision_scope_nodes s ON s.revision_id=r.id
         WHERE r.id=$1 GROUP BY r.id",
    )
    .bind(revision_id)
    .fetch_optional(&state.pool)
    .await?;
    row.map(observation_row).transpose()
}

async fn load_major_observation(
    state: &DatasetModuleState,
    dataset_id: Uuid,
    major: i32,
) -> Result<Option<ObservationRow>, DatasetModuleError> {
    let row = sqlx::query(
        "SELECT r.resource_revision,r.lifecycle_state,
                COALESCE(array_agg(s.node_id ORDER BY s.node_id)
                    FILTER (WHERE s.node_id IS NOT NULL),'{}'::uuid[]) AS scope_node_ids
         FROM dataset_revisions r
         LEFT JOIN dataset_revision_scope_nodes s ON s.revision_id=r.id
         WHERE r.dataset_id=$1 AND r.version_major=$2
           AND r.status IN ('published','superseded')
         GROUP BY r.id ORDER BY r.version_number DESC LIMIT 1",
    )
    .bind(dataset_id)
    .bind(major)
    .fetch_optional(&state.pool)
    .await?;
    row.map(observation_row).transpose()
}

fn observation_row(row: sqlx::postgres::PgRow) -> Result<ObservationRow, DatasetModuleError> {
    Ok(ObservationRow {
        resource_revision: row.try_get("resource_revision")?,
        lifecycle_state: row.try_get("lifecycle_state")?,
        scope_node_ids: row.try_get("scope_node_ids")?,
    })
}

fn restricted_observation(
    access: ResourceAccessState,
) -> Result<DatasetResourceObservationResponse, DatasetModuleError> {
    Ok(DatasetResourceObservationResponse {
        schema_version: 1,
        resolution: ResourceResolutionV1::restricted(access).map_err(internal)?,
        observation: None,
    })
}

async fn source_usage(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetSourceUsageResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        DATASET_SOURCE_USAGE_PATH,
        &body,
        DATASET_SOURCE_USAGE_ACTION,
        DATASET_SOURCE_USAGE_CONTRACT_ID,
    )
    .await?;
    let request: DatasetSourceUsageRequest =
        parse(&body, "Dataset source-usage request is invalid")?;
    if request.schema_version != 1 || request.form_id.is_nil() {
        return Err(DatasetModuleError::BadRequest(
            "Dataset source-usage request is invalid".into(),
        ));
    }
    let grant = &normal_grant(&authorization)?.payload;
    let rows = sqlx::query(
        "SELECT DISTINCT d.id AS dataset_id,d.name,rs.source_alias,rs.source_reference,
                d.lifecycle_state,r.id AS revision_id
         FROM dataset_revision_sources rs
         JOIN dataset_revisions r ON r.id=rs.revision_id
         JOIN datasets d ON d.id=r.dataset_id
         WHERE rs.source_kind='form_version'
           AND r.status IN ('published','superseded')
           AND r.lifecycle_state <> 'tombstoned'
           AND d.lifecycle_state <> 'tombstoned'
         ORDER BY d.name,d.id,rs.source_alias",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut candidates = Vec::new();
    for row in rows {
        let source: DatasetProductSourceV1 =
            serde_json::from_value(row.try_get("source_reference")?).map_err(internal)?;
        let DatasetProductSourceV1::Form {
            form_id,
            form_version_id,
            ..
        } = source
        else {
            continue;
        };
        let Ok(form_id) = Uuid::parse_str(&form_id) else {
            return Err(DatasetModuleError::Internal(
                "Stored Form source identity is invalid".into(),
            ));
        };
        let Ok(form_version_id) = Uuid::parse_str(&form_version_id) else {
            return Err(DatasetModuleError::Internal(
                "Stored FormVersion source identity is invalid".into(),
            ));
        };
        if form_id != request.form_id
            || request
                .form_version_id
                .is_some_and(|requested| requested != form_version_id)
        {
            continue;
        }
        let scope = revision_scope(&state, row.try_get("revision_id")?).await?;
        candidates.push((
            SourceUsageCandidate {
                dataset_name: row.try_get("name")?,
                dataset_id: row.try_get("dataset_id")?,
                source_alias: row.try_get("source_alias")?,
                pinned_form_version_id: form_version_id,
                lifecycle_state: row.try_get("lifecycle_state")?,
            },
            scope_authorized(grant, &scope),
        ));
    }
    Ok(Json(project_source_usage(
        authorization.installation_id,
        authorization.module_instance_id,
        candidates,
    )?))
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct SourceUsageCandidate {
    // Field order is the stable product sort and the exact output deduplication
    // identity. Revision ID is deliberately absent because it is not part of
    // the source-usage contract.
    dataset_name: String,
    dataset_id: Uuid,
    source_alias: String,
    pinned_form_version_id: Uuid,
    lifecycle_state: String,
}

fn project_source_usage(
    installation_id: Uuid,
    module_instance_id: Uuid,
    candidates: Vec<(SourceUsageCandidate, bool)>,
) -> Result<DatasetSourceUsageResponse, DatasetModuleError> {
    let matched_usage = !candidates.is_empty();
    let mut authorized = BTreeSet::new();
    for (candidate, is_authorized) in candidates {
        if is_authorized {
            authorized.insert(candidate);
        }
    }
    let items = authorized
        .into_iter()
        .map(|candidate| {
            Ok(DatasetSourceUsageItem {
                dataset: DatasetReference::from_parts(
                    installation_id,
                    module_instance_id,
                    candidate.dataset_id,
                )
                .map_err(internal)?,
                dataset_name: candidate.dataset_name,
                source_alias: candidate.source_alias,
                pinned_form_version_id: candidate.pinned_form_version_id,
                lifecycle_state: candidate.lifecycle_state,
                semantic_destination: format!("datasets.detail:{}", candidate.dataset_id),
            })
        })
        .collect::<Result<Vec<_>, DatasetModuleError>>()?;
    let state = if !items.is_empty() {
        DatasetProviderResultState::Available
    } else if matched_usage {
        DatasetProviderResultState::Undisclosed
    } else {
        DatasetProviderResultState::Empty
    };
    Ok(DatasetSourceUsageResponse {
        schema_version: 1,
        state,
        items,
    })
}

async fn operations_status(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetOperationsStatusResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        DATASET_OPERATIONS_STATUS_PATH,
        &body,
        "datasets.operations_status",
        DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
    )
    .await?;
    let request: DatasetOperationsStatusRequest =
        parse(&body, "Dataset operations-status request is invalid")?;
    if request.schema_version != 1 || !canonical_scope(&request.requested_scope_node_ids) {
        return Err(DatasetModuleError::BadRequest(
            "Dataset operations-status request is invalid".into(),
        ));
    }
    if request
        .requested_scope_node_ids
        .iter()
        .any(|id| !grant_authorizes(authorization.grant.as_ref(), READ_CAPABILITY, *id))
    {
        return Err(DatasetModuleError::Forbidden);
    }
    let rows = sqlx::query(
        "SELECT d.id AS dataset_id,d.name,r.id AS revision_id,r.status AS revision_status,
                COUNT(DISTINCT rs.source_alias) AS source_count,
                jsonb_array_length(COALESCE(r.output_fields,'[]'::jsonb))::bigint AS field_count,
                COALESCE(m.materialized_row_count,0)::bigint AS ready_response_count,
                CASE
                  WHEN bool_or(p.freshness_state IN ('degraded','failed')) THEN 'degraded'
                  WHEN bool_or(p.freshness_state='refreshing') THEN 'refreshing'
                  WHEN bool_or(p.freshness_state='stale') THEN 'stale'
                  WHEN bool_or(p.freshness_state='never_materialized') THEN 'never_materialized'
                  WHEN COUNT(p.source_binding_id)=0 AND m.materialized_at IS NULL THEN 'never_materialized'
                  ELSE 'current'
                END AS freshness_state,
                MIN(p.sanitized_failure_code) FILTER (WHERE p.sanitized_failure_code IS NOT NULL)
                  AS sanitized_failure_code
         FROM datasets d
         JOIN dataset_scope_nodes ds ON ds.dataset_id=d.id
         LEFT JOIN LATERAL (
           SELECT * FROM dataset_revisions
           WHERE dataset_id=d.id AND status IN ('published','superseded','draft')
             AND lifecycle_state <> 'tombstoned'
           ORDER BY CASE status WHEN 'published' THEN 0 WHEN 'draft' THEN 1 ELSE 2 END,
                    version_number DESC LIMIT 1
         ) r ON true
         LEFT JOIN dataset_revision_sources rs ON rs.revision_id=r.id
         LEFT JOIN dataset_major_materializations m
           ON m.dataset_id=d.id AND m.version_major=r.version_major
         LEFT JOIN dataset_sources s ON s.dataset_id=d.id
         LEFT JOIN dataset_sync_partitions p ON p.source_binding_id=s.source_binding_id
         WHERE d.lifecycle_state <> 'tombstoned' AND ds.node_id=ANY($1)
         GROUP BY d.id,d.name,r.id,r.status,r.output_fields,m.materialized_row_count,m.materialized_at
         ORDER BY d.name,d.id",
    )
    .bind(&request.requested_scope_node_ids)
    .fetch_all(&state.pool)
    .await?;
    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let dataset_id: Uuid = row.try_get("dataset_id")?;
        let revision_status: Option<String> = row.try_get("revision_status")?;
        let ready_response_count: i64 = row.try_get("ready_response_count")?;
        let readiness = match revision_status.as_deref() {
            Some("published") if ready_response_count > 0 => DatasetReadinessLabel::Ready,
            Some("published") => DatasetReadinessLabel::NoReadyResponses,
            Some("draft") => DatasetReadinessLabel::Draft,
            Some("superseded") => DatasetReadinessLabel::Superseded,
            Some(_) => DatasetReadinessLabel::Unavailable,
            None => DatasetReadinessLabel::NoPublishedRevision,
        };
        items.push(DatasetOperationsStatusItem {
            dataset: DatasetReference::from_parts(
                authorization.installation_id,
                authorization.module_instance_id,
                dataset_id,
            )
            .map_err(internal)?,
            dataset_name: row.try_get("name")?,
            readiness,
            revision_status,
            source_count: row.try_get::<i64, _>("source_count")? as u64,
            field_count: row.try_get::<i64, _>("field_count")? as u64,
            ready_response_count: ready_response_count as u64,
            freshness: freshness_state(&row.try_get::<String, _>("freshness_state")?)?,
            sanitized_failure_code: row.try_get("sanitized_failure_code")?,
        });
    }
    Ok(Json(DatasetOperationsStatusResponse {
        schema_version: 1,
        state: if items.is_empty() {
            DatasetProviderResultState::Empty
        } else {
            DatasetProviderResultState::Available
        },
        items,
    }))
}

async fn summary(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<DatasetSummaryResponse>, DatasetModuleError> {
    require_json_content_type(&headers)?;
    let authorization = authorize(
        &state,
        &headers,
        DATASET_SUMMARY_PATH,
        &body,
        "datasets.summary",
        DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
    )
    .await?;
    let request: DatasetSummaryRequest = parse(&body, "Dataset summary request is invalid")?;
    if request.schema_version != 1 {
        return Err(DatasetModuleError::BadRequest(
            "Dataset summary request is invalid".into(),
        ));
    }
    let scopes = authorized_scope(&authorization.grant, READ_CAPABILITY);
    if scopes.is_empty() {
        return Err(DatasetModuleError::Forbidden);
    }
    let counts: (i64, i64) = sqlx::query_as(
        "SELECT COUNT(DISTINCT d.id),
                COUNT(DISTINCT r.id) FILTER (WHERE r.status='published')
         FROM datasets d JOIN dataset_scope_nodes s ON s.dataset_id=d.id
         LEFT JOIN dataset_revisions r ON r.dataset_id=d.id AND r.lifecycle_state <> 'tombstoned'
         WHERE d.lifecycle_state <> 'tombstoned' AND s.node_id=ANY($1)",
    )
    .bind(scopes.into_iter().collect::<Vec<_>>())
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(DatasetSummaryResponse {
        schema_version: 1,
        state: if counts.0 == 0 {
            DatasetProviderResultState::Empty
        } else {
            DatasetProviderResultState::Available
        },
        dataset_count: Some(counts.0 as u64),
        published_revision_count: Some(counts.1 as u64),
    }))
}

fn freshness_state(value: &str) -> Result<DatasetFreshnessState, DatasetModuleError> {
    match value {
        "current" => Ok(DatasetFreshnessState::Current),
        "stale" => Ok(DatasetFreshnessState::Stale),
        "degraded" => Ok(DatasetFreshnessState::Degraded),
        "refreshing" => Ok(DatasetFreshnessState::Refreshing),
        "failed" => Ok(DatasetFreshnessState::Failed),
        "never_materialized" => Ok(DatasetFreshnessState::NeverMaterialized),
        _ => Err(DatasetModuleError::Internal(
            "Stored Dataset freshness state is invalid".into(),
        )),
    }
}

pub(crate) fn push_row_access_predicate(
    query: &mut QueryBuilder<'_, Postgres>,
    grant: &AuthorizationGrantV3,
) {
    let read = authorized_organizations(grant, READ_CAPABILITY);
    let confidential = authorized_organizations(grant, "datasets:read_confidential");
    let restricted = authorized_organizations(grant, "datasets:read_restricted");
    let read = read.into_iter().collect::<Vec<_>>();
    let restricted = restricted
        .into_iter()
        .filter(|node| read.binary_search(node).is_ok())
        .collect::<Vec<_>>();
    let confidential = confidential
        .into_iter()
        .filter(|node| read.binary_search(node).is_ok())
        .collect::<Vec<_>>();
    push_row_scope_predicate(query, read, restricted, confidential);
}

fn push_row_scope_predicate(
    query: &mut QueryBuilder<'_, Postgres>,
    read: Vec<Uuid>,
    restricted: Vec<Uuid>,
    confidential: Vec<Uuid>,
) {
    query
        .push("(__scope_node_ids && ")
        .push_bind(read)
        .push("::uuid[] AND (__restriction_tier IN ('public','internal')");
    if !restricted.is_empty() {
        query
            .push(" OR (__restriction_tier='restricted' AND __scope_node_ids && ")
            .push_bind(restricted)
            .push("::uuid[])");
    }
    if !confidential.is_empty() {
        query
            .push(
                " OR (__restriction_tier IN ('restricted','confidential') AND __scope_node_ids && ",
            )
            .push_bind(confidential)
            .push("::uuid[])");
    }
    query.push("))");
}

fn scope_authorized(grant: &AuthorizationGrantV3, scope: &[Uuid]) -> bool {
    SecurityCapabilityId::new(READ_CAPABILITY)
        .ok()
        .is_some_and(|capability| scope.iter().any(|id| grant.authorizes(&capability, *id)))
}

fn grant_authorizes(
    grant: Option<&SignedEnvelopeV1<AuthorizationGrantV3>>,
    capability: &str,
    organization_id: Uuid,
) -> bool {
    let Some(grant) = grant else {
        return false;
    };
    SecurityCapabilityId::new(capability)
        .ok()
        .is_some_and(|capability| grant.payload.authorizes(&capability, organization_id))
}

fn authorized_scope(
    grant: &Option<SignedEnvelopeV1<AuthorizationGrantV3>>,
    capability: &str,
) -> BTreeSet<Uuid> {
    grant
        .as_ref()
        .map(|grant| authorized_organizations(&grant.payload, capability))
        .unwrap_or_default()
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

fn canonical_scope(scope: &[Uuid]) -> bool {
    scope.windows(2).all(|pair| pair[0] < pair[1])
}

fn reference_targets_provider(
    reference: &TypedResourceReference,
    authorization: &ProviderAuthorization,
) -> bool {
    reference.validate().is_ok()
        && reference.installation_id() == authorization.installation_id
        && reference.owner()
            == &(ResourceOwner::ModuleInstance {
                installation_id: authorization.installation_id,
                module_instance_id: authorization.module_instance_id,
            })
}

fn require_json_content_type(headers: &HeaderMap) -> Result<(), DatasetModuleError> {
    let is_json = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        == Some("application/json");
    if !is_json {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(())
}

fn parse<T: serde::de::DeserializeOwned>(
    body: &[u8],
    message: &'static str,
) -> Result<T, DatasetModuleError> {
    serde_json::from_slice(body).map_err(|_| DatasetModuleError::BadRequest(message.into()))
}

fn parse_restricted<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, DatasetModuleError> {
    serde_json::from_slice(body).map_err(|_| DatasetModuleError::Forbidden)
}

fn decode<T: serde::de::DeserializeOwned>(value: &str) -> Result<T, DatasetModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    serde_json::from_slice(&bytes).map_err(|_| DatasetModuleError::Forbidden)
}

fn header<'a>(headers: &'a HeaderMap, name: &str) -> Result<&'a str, DatasetModuleError> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or(DatasetModuleError::Forbidden)
}

fn correlation_id(headers: &HeaderMap) -> Result<Uuid, DatasetModuleError> {
    header(headers, "x-tessara-correlation-id")?
        .parse()
        .map_err(|_| DatasetModuleError::Forbidden)
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

fn unavailable_security() -> DatasetModuleError {
    DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
}

fn internal(error: impl std::fmt::Display) -> DatasetModuleError {
    DatasetModuleError::Internal(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use axum::http::{HeaderMap, HeaderValue};
    use sqlx::{Execute, Postgres, QueryBuilder};
    use tessara_datasets_contract::{
        DATASET_OPERATIONS_STATUS_ACTION, DATASET_OPERATIONS_STATUS_PATH, DATASET_RESOLVE_ACTION,
        DATASET_RESOLVE_PATH, DATASET_SOURCE_USAGE_ACTION, DATASET_SOURCE_USAGE_PATH,
        DATASET_SUMMARY_ACTION, DATASET_SUMMARY_PATH, DatasetAggregateFunction,
        DatasetCompatibilityFinding, DatasetFieldRequirement, DatasetFilterOperator,
        DatasetMissingPolicy, DatasetProviderResultState,
    };
    use uuid::Uuid;

    use super::{
        MAX_DATASET_EXECUTION_OFFSET, SourceUsageCandidate, aggregate_function_supported,
        compatibility_findings, core_gateway_action_allowed, execution_offset,
        filter_operator_supported, project_source_usage, push_row_scope_predicate,
        require_json_content_type,
    };

    #[test]
    fn core_gateway_is_limited_to_the_four_reverse_provider_actions() {
        for (path, action) in [
            (DATASET_RESOLVE_PATH, DATASET_RESOLVE_ACTION),
            (DATASET_SOURCE_USAGE_PATH, DATASET_SOURCE_USAGE_ACTION),
            (
                DATASET_OPERATIONS_STATUS_PATH,
                DATASET_OPERATIONS_STATUS_ACTION,
            ),
            (DATASET_SUMMARY_PATH, DATASET_SUMMARY_ACTION),
        ] {
            assert!(core_gateway_action_allowed(path, action));
        }
        for (path, action) in [
            ("/api/private/datasets/catalog", "datasets.catalog"),
            ("/api/private/datasets/schema", "datasets.schema"),
            ("/api/private/datasets/execute", "datasets.execute"),
            (DATASET_SUMMARY_PATH, DATASET_OPERATIONS_STATUS_ACTION),
        ] {
            assert!(!core_gateway_action_allowed(path, action));
        }
    }

    fn source_usage_candidate(dataset_id: u128) -> SourceUsageCandidate {
        SourceUsageCandidate {
            dataset_name: "Cases".into(),
            dataset_id: Uuid::from_u128(dataset_id),
            source_alias: "case_form".into(),
            pinned_form_version_id: Uuid::from_u128(31),
            lifecycle_state: "active".into(),
        }
    }

    #[test]
    fn source_usage_distinguishes_true_empty_from_matched_but_disjoint() {
        let installation_id = Uuid::from_u128(1);
        let module_instance_id = Uuid::from_u128(2);
        let empty = project_source_usage(installation_id, module_instance_id, Vec::new()).unwrap();
        assert_eq!(empty.state, DatasetProviderResultState::Empty);
        assert!(empty.items.is_empty());
        empty.validate().unwrap();

        let disjoint = project_source_usage(
            installation_id,
            module_instance_id,
            vec![(source_usage_candidate(21), false)],
        )
        .unwrap();
        assert_eq!(disjoint.state, DatasetProviderResultState::Undisclosed);
        assert!(disjoint.items.is_empty());
        disjoint.validate().unwrap();
    }

    #[test]
    fn source_usage_deduplicates_identical_output_across_revisions() {
        let candidate = source_usage_candidate(21);
        let response = project_source_usage(
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            vec![(candidate.clone(), true), (candidate, true)],
        )
        .unwrap();

        assert_eq!(response.state, DatasetProviderResultState::Available);
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].dataset.dataset_id(), Uuid::from_u128(21));
        assert_eq!(
            response.items[0].pinned_form_version_id,
            Uuid::from_u128(31)
        );
        assert_eq!(
            response.items[0].semantic_destination,
            "datasets.detail:00000000-0000-0000-0000-000000000015"
        );
        response.validate().unwrap();
    }

    #[test]
    fn provider_media_type_is_exactly_json() {
        let mut headers = HeaderMap::new();
        headers.insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        require_json_content_type(&headers).unwrap();

        for content_type in [
            None,
            Some("application/json; charset=utf-8"),
            Some("application/vnd.tessara+json"),
            Some("text/json"),
            Some("application/octet-stream"),
        ] {
            let mut headers = HeaderMap::new();
            if let Some(content_type) = content_type {
                headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    HeaderValue::from_str(content_type).unwrap(),
                );
            }
            assert!(require_json_content_type(&headers).is_err());
        }
    }

    #[test]
    fn compatibility_reports_materialization_before_fields() {
        let fields = BTreeMap::from([("label", "text")]);
        assert_eq!(
            compatibility_findings(
                "pending",
                &fields,
                vec![DatasetFieldRequirement {
                    field_key: "missing".into(),
                    accepted_types: vec!["text".into()],
                }],
            ),
            vec![DatasetCompatibilityFinding {
                code: tessara_datasets_contract::DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY
                    .into(),
                field_key: "materialization_state".into(),
            }]
        );
    }

    #[test]
    fn cursor_and_operation_bounds_match_dataset_v2() {
        assert_eq!(execution_offset(None).unwrap(), 0);
        assert_eq!(execution_offset(Some("offset:25")).unwrap(), 25);
        assert!(execution_offset(Some("offset:0")).is_err());
        assert!(
            execution_offset(Some(&format!(
                "offset:{}",
                MAX_DATASET_EXECUTION_OFFSET + 1
            )))
            .is_err()
        );
        assert!(filter_operator_supported(
            DatasetFilterOperator::Contains,
            "text"
        ));
        assert!(!filter_operator_supported(
            DatasetFilterOperator::Contains,
            "number"
        ));
        assert!(aggregate_function_supported(
            DatasetAggregateFunction::Median,
            Some("number")
        ));
        assert!(!aggregate_function_supported(
            DatasetAggregateFunction::Median,
            Some("text")
        ));
        assert_eq!(DatasetMissingPolicy::default(), DatasetMissingPolicy::Omit);
    }

    #[test]
    fn row_access_is_bound_to_hidden_scope_and_same_scope_tier_grants() {
        let mut query = QueryBuilder::<Postgres>::new("SELECT 1 WHERE ");
        push_row_scope_predicate(
            &mut query,
            vec![Uuid::from_u128(1), Uuid::from_u128(2)],
            vec![Uuid::from_u128(1)],
            vec![Uuid::from_u128(2)],
        );
        let sql = query.build().sql().to_string();
        assert!(sql.contains("__scope_node_ids && $1::uuid[]"));
        assert!(sql.contains("='restricted' AND __scope_node_ids && $2::uuid[]"));
        assert!(
            sql.contains("IN ('restricted','confidential') AND __scope_node_ids && $3::uuid[]")
        );
        assert!(sql.contains("IN ('public','internal')"));
        assert!(!sql.contains("COALESCE"));
    }
}
