use std::collections::BTreeSet;

use axum::{
    Json, Router,
    extract::{Path, RawQuery, State},
    http::HeaderMap,
    routing::{delete, get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_components_contract::{COMPONENT_RESOURCE_TYPE, ComponentVersionReference};
use tessara_datasets_contract::{
    DATASET_CONTRACT_SCHEMA_VERSION, DatasetAction, DatasetCatalogRequest, DatasetCatalogResponse,
    DatasetDistinctValuesRequest, DatasetDistinctValuesResponse, DatasetExecutionResponse,
    DatasetMajorLineMetadata, DatasetMajorLineReference, DatasetSchemaRequest,
};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV2, AuthorizationValidationContextV2,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ResourceOwner,
    SecurityCapabilityId, SignedEnvelopeV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    ComponentModuleError, ComponentModuleState, MANAGE_CAPABILITY, READ_CAPABILITY, dataset_client,
    load_security_state,
};

const CORE_COMPONENT_BINDING: &str = "tessara.core.components";
const COMPONENT_RESOURCE_CONTRACT: &str = "tessara.components.component-version";
const COMPONENT_AUTHORING_CONTRACT: &str = "tessara.components.authoring";

pub(super) fn routes() -> Router<ComponentModuleState> {
    Router::new()
        .route("/api/components", get(list_components))
        .route("/api/components/{component_ref}", get(get_component))
        .route(
            "/api/components/{component_ref}/{kind}",
            get(execute_current_component),
        )
        .route(
            "/api/components/{component_ref}/versions/{version_id}/{kind}",
            get(execute_component_version),
        )
        .route(
            "/api/admin/components",
            get(list_manageable_components).post(create_component),
        )
        .route(
            "/api/admin/components/{component_id}",
            get(get_manageable_component).put(update_component),
        )
        .route(
            "/api/admin/components/{component_id}/versions",
            post(create_version),
        )
        .route(
            "/api/admin/components/{component_id}/versions/{version_id}",
            delete(delete_version).put(update_version),
        )
        .route("/api/admin/components/validate", post(validate_version))
        .route("/api/admin/components/preview", post(preview_version))
        .route("/api/admin/components/datasets", get(dataset_catalog))
        .route(
            "/api/admin/components/datasets/distinct-values",
            post(dataset_distinct_values),
        )
        .route(
            "/api/admin/components/{component_id}/versions/{version_id}/publish",
            post(publish_version),
        )
        .route(
            "/api/admin/components/{component_id}/versions/{version_id}/lifecycle",
            post(change_lifecycle),
        )
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentVersionInputV1 {
    pub dataset_reference: DatasetMajorLineReference,
    pub component_type: String,
    pub config: Value,
    #[serde(default)]
    pub version_note: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CreateComponentV1 {
    pub schema_version: u16,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub version: ComponentVersionInputV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct UpdateComponentV1 {
    schema_version: u16,
    name: String,
    slug: String,
    description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CreateComponentVersionV1 {
    schema_version: u16,
    version: ComponentVersionInputV1,
}

#[derive(Clone, Debug, Serialize)]
struct ValidationResponseV1 {
    schema_version: u16,
    valid: bool,
    findings: Vec<ValidationFindingV1>,
}

#[derive(Clone, Debug, Serialize)]
struct ValidationFindingV1 {
    code: String,
    field_path: Option<String>,
    message: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComponentSummaryV1 {
    pub schema_version: u16,
    pub component_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub current_version: ComponentVersionV1,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComponentVersionV1 {
    pub reference: ComponentVersionReference,
    pub component_version_id: Uuid,
    pub dataset_reference: DatasetMajorLineReference,
    pub component_type: String,
    pub publication_state: String,
    pub lifecycle_state: String,
    pub resource_revision: u64,
    pub authority_revision: u64,
    pub version_number: i32,
    pub version_label: String,
    pub version_note: String,
    pub config: Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ComponentDefinitionV1 {
    pub schema_version: u16,
    pub component_id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub versions: Vec<ComponentVersionV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ComponentMutationResponseV1 {
    schema_version: u16,
    component_id: Uuid,
    component_version_id: Option<Uuid>,
    outcome: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct LifecycleRequestV1 {
    schema_version: u16,
    action: LifecycleActionV1,
    expected_resource_revision: u64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum LifecycleActionV1 {
    Activate,
    Deactivate,
    Archive,
    Tombstone,
}

async fn execute_current_component(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_ref, kind)): Path<(String, String)>,
    query: RawQuery,
) -> Result<Json<Value>, ComponentModuleError> {
    let component_id = resolve_component_id(&state, &component_ref).await?;
    let version_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM component_versions
         WHERE component_id=$1 AND status='published'
         ORDER BY version_number DESC LIMIT 1",
    )
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ComponentModuleError::NotFound("Published Component not found".into()))?;
    execute_component(state, headers, component_id, version_id, kind, query.0).await
}

async fn execute_component_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_ref, version_id, kind)): Path<(String, Uuid, String)>,
    query: RawQuery,
) -> Result<Json<Value>, ComponentModuleError> {
    let component_id = resolve_component_id(&state, &component_ref).await?;
    execute_component(state, headers, component_id, version_id, kind, query.0).await
}

async fn execute_component(
    state: ComponentModuleState,
    headers: HeaderMap,
    component_id: Uuid,
    version_id: Uuid,
    kind: String,
    query: Option<String>,
) -> Result<Json<Value>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.execute",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let row = sqlx::query(
        "SELECT dataset_reference,dataset_scope_node_ids,component_type::text AS component_type,
                config,status::text AS status,lifecycle_state::text AS lifecycle_state
         FROM component_versions WHERE id=$1 AND component_id=$2",
    )
    .bind(version_id)
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ComponentModuleError::NotFound("Component version not found".into()))?;
    let scope: Vec<Uuid> = row.try_get("dataset_scope_node_ids")?;
    require_scope(&grant.payload, READ_CAPABILITY, &scope)?;
    let stored_kind: String = row.try_get("component_type")?;
    let requested_kind = kind.replace('-', "_");
    if stored_kind != requested_kind
        || !matches!(
            row.try_get::<String, _>("status")?.as_str(),
            "published" | "superseded"
        )
        || row.try_get::<String, _>("lifecycle_state")? != "active"
    {
        return Err(ComponentModuleError::NotFound(
            "Renderable Component version not found".into(),
        ));
    }
    let dataset_reference: DatasetMajorLineReference =
        serde_json::from_value(row.try_get("dataset_reference")?).map_err(internal)?;
    let config: Value = row.try_get("config")?;
    let execution_request = crate::provider::execution_request(
        dataset_reference.clone(),
        &stored_kind,
        &config,
        query.as_deref().unwrap_or_default(),
    )?;
    let limit = execution_request.limit;
    let execution: DatasetExecutionResponse = dataset_client::post(
        &state,
        authorization_header(&headers)?,
        "/api/private/component-datasets/execute",
        &execution_request,
    )
    .await?;
    Ok(Json(crate::provider::render_execution(
        execution,
        version_id,
        component_id,
        dataset_reference,
        &stored_kind,
        &config,
        limit,
    )))
}

async fn create_component(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<CreateComponentV1>,
) -> Result<Json<ComponentDefinitionV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.create",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let idempotency_key = mutation_idempotency_key(&headers)?;
    if request.schema_version != 1
        || request.name.trim().is_empty()
        || request.slug.trim().is_empty()
        || !request.version.config.is_object()
        || !matches!(
            request.version.component_type.as_str(),
            "table" | "bar" | "line" | "pie" | "donut" | "stat_card"
        )
    {
        return Err(ComponentModuleError::BadRequest(
            "Component create request is invalid".into(),
        ));
    }
    let authorization = authorization_header(&headers)?;
    let metadata: DatasetMajorLineMetadata = dataset_client::post(
        &state,
        authorization,
        "/api/private/component-datasets/schema",
        &DatasetSchemaRequest {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            action: DatasetAction::ResolveSchema,
            reference: request.version.dataset_reference.clone(),
        },
    )
    .await?;
    require_scope(&grant.payload, MANAGE_CAPABILITY, &metadata.scope_node_ids)?;
    let mut scope_node_ids = metadata.scope_node_ids;
    scope_node_ids.sort_unstable();
    scope_node_ids.dedup();
    let payload_digest = format!(
        "sha256:{:x}",
        Sha256::digest(
            serde_json::to_vec(&request)
                .map_err(|error| ComponentModuleError::Internal(error.to_string()))?
        )
    );
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    let component_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();
    let mut transaction = state.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(idempotency_key)
        .execute(&mut *transaction)
        .await?;
    if let Some((actor_id, action, stored_digest, result)) = sqlx::query_as::<_, (Uuid, String, String, Value)>(
        "SELECT original_actor_id,action,payload_digest,result FROM component_mutation_replays WHERE idempotency_key=$1",
    )
    .bind(idempotency_key)
    .fetch_optional(&mut *transaction)
    .await?
    {
        if actor_id != grant.payload.original_actor_id || action != "components.create" || stored_digest != payload_digest {
            return Err(ComponentModuleError::Conflict("Component idempotency key was reused with different input".into()));
        }
        return serde_json::from_value(result)
            .map(Json)
            .map_err(|error| ComponentModuleError::Internal(error.to_string()));
    }
    sqlx::query("INSERT INTO components(id,name,slug,description) VALUES($1,$2,$3,$4)")
        .bind(component_id)
        .bind(request.name.trim())
        .bind(request.slug.trim())
        .bind(
            request
                .description
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .execute(&mut *transaction)
        .await?;
    sqlx::query(
        "INSERT INTO component_versions
         (id,component_id,dataset_reference,dataset_scope_node_ids,component_type,status,lifecycle_state,
          version_number,version_label,version_note,config)
         VALUES($1,$2,$3,$4,$5::component_type,'draft','active',1,'1.0.0',$6,$7)",
    )
    .bind(version_id)
    .bind(component_id)
    .bind(serde_json::to_value(&request.version.dataset_reference).map_err(|error| ComponentModuleError::Internal(error.to_string()))?)
    .bind(&scope_node_ids)
    .bind(&request.version.component_type)
    .bind(request.version.version_note.trim())
    .bind(&request.version.config)
    .execute(&mut *transaction)
    .await?;
    let response = ComponentDefinitionV1 {
        schema_version: 1,
        component_id,
        name: request.name.trim().to_string(),
        slug: request.slug.trim().to_string(),
        description: request
            .description
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        versions: vec![ComponentVersionV1 {
            reference: ComponentVersionReference::new(
                TypedResourceReference::new(
                    security.installation_id,
                    ResourceOwner::ModuleInstance {
                        installation_id: security.installation_id,
                        module_instance_id: security.module_instance_id,
                    },
                    COMPONENT_RESOURCE_TYPE
                        .parse()
                        .map_err(|error| ComponentModuleError::Internal(format!("{error}")))?,
                    version_id.to_string(),
                )
                .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
            )
            .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
            component_version_id: version_id,
            dataset_reference: request.version.dataset_reference.clone(),
            component_type: request.version.component_type.clone(),
            publication_state: "draft".into(),
            lifecycle_state: "active".into(),
            resource_revision: 1,
            authority_revision: 1,
            version_number: 1,
            version_label: "1.0.0".into(),
            version_note: request.version.version_note.trim().into(),
            config: request.version.config.clone(),
        }],
    };
    sqlx::query("INSERT INTO component_mutation_replays(jti,original_actor_id,action,payload_digest,idempotency_key,result) VALUES($1,$2,$3,$4,$5,$6)")
        .bind(grant.payload.jti)
        .bind(grant.payload.original_actor_id)
        .bind("components.create")
        .bind(&payload_digest)
        .bind(idempotency_key)
        .bind(serde_json::to_value(&response).map_err(|error| ComponentModuleError::Internal(error.to_string()))?)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn list_manageable_components(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ComponentDefinitionV1>>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.list_manageable",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT component_id FROM component_versions ORDER BY component_id",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut components = Vec::new();
    for id in ids {
        if let Ok(component) = get_definition_by_id(&state, &grant.payload, id).await {
            components.push(component);
        }
    }
    Ok(Json(components))
}

async fn get_manageable_component(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> Result<Json<ComponentDefinitionV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.edit",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let id = resolve_component_id(&state, &component_ref).await?;
    get_definition_by_id(&state, &grant.payload, id)
        .await
        .map(Json)
}

async fn update_component(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(request): Json<UpdateComponentV1>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.update",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    if request.schema_version != 1
        || request.name.trim().is_empty()
        || request.slug.trim().is_empty()
    {
        return Err(ComponentModuleError::BadRequest(
            "Component update request is invalid".into(),
        ));
    }
    let version_id:Uuid=sqlx::query_scalar("SELECT id FROM component_versions WHERE component_id=$1 ORDER BY version_number DESC LIMIT 1").bind(component_id).fetch_optional(&state.pool).await?.ok_or_else(||ComponentModuleError::NotFound("Component not found".into()))?;
    require_version_scope(
        &state,
        &grant.payload,
        component_id,
        version_id,
        MANAGE_CAPABILITY,
    )
    .await?;
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let digest = mutation_digest("components.update", &[component_id], &request)?;
    let mut transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.update",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }
    sqlx::query(
        "UPDATE components SET name=$1,slug=$2,description=$3,updated_at=now() WHERE id=$4",
    )
    .bind(request.name.trim())
    .bind(request.slug.trim())
    .bind(
        request
            .description
            .as_deref()
            .map(str::trim)
            .filter(|v| !v.is_empty()),
    )
    .bind(component_id)
    .execute(&mut *transaction)
    .await?;
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: None,
        outcome: "updated".into(),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.update",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn create_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_id): Path<Uuid>,
    Json(request): Json<CreateComponentVersionV1>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.save_version",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    if request.schema_version != 1 {
        return Err(ComponentModuleError::BadRequest(
            "Component version schema is invalid".into(),
        ));
    }
    let latest_version_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM component_versions WHERE component_id=$1 ORDER BY version_number DESC LIMIT 1",
    )
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ComponentModuleError::NotFound("Component not found".into()))?;
    require_version_scope(
        &state,
        &grant.payload,
        component_id,
        latest_version_id,
        MANAGE_CAPABILITY,
    )
    .await?;
    let authorization = authorization_header(&headers)?;
    let (metadata, findings) =
        validate_version_input(&state, authorization, &grant.payload, &request.version).await?;
    if !findings.is_empty() {
        return Err(ComponentModuleError::BadRequest(
            findings
                .into_iter()
                .map(|f| f.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let digest = mutation_digest("components.save_version", &[component_id], &request)?;
    let mut transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.save_version",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }
    let existing: Option<Uuid> = sqlx::query_scalar(
        "SELECT id FROM component_versions WHERE component_id=$1 AND status='draft'",
    )
    .bind(component_id)
    .fetch_optional(&mut *transaction)
    .await?;
    if existing.is_some() {
        return Err(ComponentModuleError::Conflict(
            "Publish or delete the current draft before creating another".into(),
        ));
    }
    if !sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM components WHERE id=$1)")
        .bind(component_id)
        .fetch_one(&mut *transaction)
        .await?
    {
        return Err(ComponentModuleError::NotFound("Component not found".into()));
    }
    let number: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number),0)+1 FROM component_versions WHERE component_id=$1",
    )
    .bind(component_id)
    .fetch_one(&mut *transaction)
    .await?;
    let version_id = Uuid::new_v4();
    let mut scope = metadata.scope_node_ids;
    scope.sort_unstable();
    scope.dedup();
    sqlx::query("INSERT INTO component_versions(id,component_id,dataset_reference,dataset_scope_node_ids,component_type,status,lifecycle_state,version_number,version_label,version_note,config) VALUES($1,$2,$3,$4,$5::component_type,'draft','active',$6,$7,$8,$9)").bind(version_id).bind(component_id).bind(serde_json::to_value(&request.version.dataset_reference).map_err(internal)?).bind(scope).bind(&request.version.component_type).bind(number).bind(format!("{number}.0.0")).bind(request.version.version_note.trim()).bind(&request.version.config).execute(&mut *transaction).await?;
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: Some(version_id),
        outcome: "draft_created".into(),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.save_version",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn delete_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.delete_version",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    require_version_scope(
        &state,
        &grant.payload,
        component_id,
        version_id,
        MANAGE_CAPABILITY,
    )
    .await?;
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let digest = mutation_digest(
        "components.delete_version",
        &[component_id, version_id],
        &(),
    )?;
    let mut transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.delete_version",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }
    let deleted = sqlx::query(
        "DELETE FROM component_versions WHERE id=$1 AND component_id=$2 AND status='draft'",
    )
    .bind(version_id)
    .bind(component_id)
    .execute(&mut *transaction)
    .await?;
    if deleted.rows_affected() != 1 {
        return Err(ComponentModuleError::Conflict(
            "Only a draft Component version can be deleted".into(),
        ));
    }
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: Some(version_id),
        outcome: "draft_deleted".into(),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.delete_version",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn update_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<CreateComponentVersionV1>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.save_version",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    if request.schema_version != 1 {
        return Err(ComponentModuleError::BadRequest(
            "Component version schema is invalid".into(),
        ));
    }
    require_version_scope(
        &state,
        &grant.payload,
        component_id,
        version_id,
        MANAGE_CAPABILITY,
    )
    .await?;
    let (metadata, findings) = validate_version_input(
        &state,
        authorization_header(&headers)?,
        &grant.payload,
        &request.version,
    )
    .await?;
    if !findings.is_empty() {
        return Err(ComponentModuleError::BadRequest(
            findings
                .into_iter()
                .map(|f| f.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    let mut scope = metadata.scope_node_ids;
    scope.sort_unstable();
    scope.dedup();
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let digest = mutation_digest(
        "components.save_version",
        &[component_id, version_id],
        &request,
    )?;
    let mut transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.save_version",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }
    let updated=sqlx::query("UPDATE component_versions SET dataset_reference=$1,dataset_scope_node_ids=$2,component_type=$3::component_type,version_note=$4,config=$5,resource_revision=resource_revision+1,authority_revision=authority_revision+1,updated_at=now() WHERE id=$6 AND component_id=$7 AND status='draft'").bind(serde_json::to_value(&request.version.dataset_reference).map_err(internal)?).bind(scope).bind(&request.version.component_type).bind(request.version.version_note.trim()).bind(&request.version.config).bind(version_id).bind(component_id).execute(&mut *transaction).await?;
    if updated.rows_affected() != 1 {
        return Err(ComponentModuleError::Conflict(
            "Only a draft Component version can be edited".into(),
        ));
    }
    sqlx::query("INSERT INTO component_version_change_events(component_version_id,resource_revision,category) SELECT id,resource_revision,'payload' FROM component_versions WHERE id=$1").bind(version_id).execute(&mut *transaction).await?;
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: Some(version_id),
        outcome: "draft_updated".into(),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.save_version",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn validate_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<ComponentVersionInputV1>,
) -> Result<Json<ValidationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.validate",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let (_, findings) = validate_version_input(
        &state,
        authorization_header(&headers)?,
        &grant.payload,
        &request,
    )
    .await?;
    Ok(Json(ValidationResponseV1 {
        schema_version: 1,
        valid: findings.is_empty(),
        findings,
    }))
}

async fn preview_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<ComponentVersionInputV1>,
) -> Result<Json<Value>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.preview",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let authorization = authorization_header(&headers)?;
    let (_, findings) =
        validate_version_input(&state, authorization, &grant.payload, &request).await?;
    if !findings.is_empty() {
        return Err(ComponentModuleError::BadRequest(
            "Component preview configuration is invalid".into(),
        ));
    }
    let execution_request = crate::provider::execution_request(
        request.dataset_reference.clone(),
        &request.component_type,
        &request.config,
        "",
    )?;
    let execution: DatasetExecutionResponse = dataset_client::post(
        &state,
        authorization,
        "/api/private/component-datasets/execute",
        &execution_request,
    )
    .await?;
    Ok(Json(crate::provider::render_execution(
        execution,
        Uuid::nil(),
        Uuid::nil(),
        request.dataset_reference,
        &request.component_type,
        &request.config,
        execution_request.limit,
    )))
}

async fn dataset_catalog(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
) -> Result<Json<DatasetCatalogResponse>, ComponentModuleError> {
    authorize(
        &state,
        &headers,
        "components.list_manageable",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    dataset_client::post(
        &state,
        authorization_header(&headers)?,
        "/api/private/component-datasets/catalog",
        &DatasetCatalogRequest {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            action: DatasetAction::Catalog,
        },
    )
    .await
    .map(Json)
}

async fn dataset_distinct_values(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<DatasetDistinctValuesRequest>,
) -> Result<Json<DatasetDistinctValuesResponse>, ComponentModuleError> {
    authorize(
        &state,
        &headers,
        "components.validate",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    dataset_client::post(
        &state,
        authorization_header(&headers)?,
        "/api/private/component-datasets/distinct-values",
        &request,
    )
    .await
    .map(Json)
}

async fn validate_version_input(
    state: &ComponentModuleState,
    authorization: &str,
    grant: &AuthorizationGrantV2,
    input: &ComponentVersionInputV1,
) -> Result<(DatasetMajorLineMetadata, Vec<ValidationFindingV1>), ComponentModuleError> {
    let metadata: DatasetMajorLineMetadata = dataset_client::post(
        state,
        authorization,
        "/api/private/component-datasets/schema",
        &DatasetSchemaRequest {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            action: DatasetAction::ResolveSchema,
            reference: input.dataset_reference.clone(),
        },
    )
    .await?;
    require_scope(grant, MANAGE_CAPABILITY, &metadata.scope_node_ids)?;
    let mut findings = Vec::new();
    if !matches!(
        input.component_type.as_str(),
        "table" | "bar" | "line" | "pie" | "donut" | "stat_card"
    ) {
        findings.push(finding(
            "component_type.unsupported",
            Some("component_type"),
            "Component kind is unsupported",
        ));
    }
    if !input.config.is_object() {
        findings.push(finding(
            "config.object_required",
            Some("config"),
            "Component configuration must be an object",
        ));
    }
    let known = metadata
        .fields
        .iter()
        .map(|field| field.key.as_str())
        .collect::<BTreeSet<_>>();
    for field in referenced_fields(&input.config) {
        if !known.contains(field.as_str()) {
            findings.push(finding(
                "config.field_unavailable",
                Some(&format!("config.{field}")),
                &format!("Dataset field '{field}' is unavailable"),
            ));
        }
    }
    if input.component_type == "table"
        && input
            .config
            .get("visible_columns")
            .and_then(Value::as_array)
            .is_none()
    {
        findings.push(finding(
            "config.visible_columns.required",
            Some("config.visible_columns"),
            "Table Components require visible columns",
        ));
    }
    if matches!(input.component_type.as_str(), "bar" | "pie" | "donut")
        && input
            .config
            .get("category_field")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .is_none()
    {
        findings.push(finding(
            "config.category_field.required",
            Some("config.category_field"),
            "This Component kind requires a category field",
        ));
    }
    if input.component_type == "line"
        && input
            .config
            .get("x_field")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
            .is_none()
    {
        findings.push(finding(
            "config.x_field.required",
            Some("config.x_field"),
            "Line Components require an x field",
        ));
    }
    Ok((metadata, findings))
}

fn referenced_fields(config: &Value) -> BTreeSet<String> {
    let mut fields = BTreeSet::new();
    for key in [
        "summary_field",
        "category_field",
        "comparison_field",
        "x_field",
    ] {
        if let Some(value) = config
            .get(key)
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty())
        {
            fields.insert(value.into());
        }
    }
    for value in config
        .get("visible_columns")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(field) = value
            .as_str()
            .or_else(|| value.get("key").and_then(Value::as_str))
            .or_else(|| value.get("field").and_then(Value::as_str))
        {
            fields.insert(field.into());
        }
    }
    for value in config
        .get("filters")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(field) = value
            .get("field")
            .or_else(|| value.get("field_key"))
            .and_then(Value::as_str)
        {
            fields.insert(field.into());
        }
    }
    fields
}
fn finding(code: &str, path: Option<&str>, message: &str) -> ValidationFindingV1 {
    ValidationFindingV1 {
        code: code.into(),
        field_path: path.map(str::to_string),
        message: message.into(),
    }
}
fn internal(error: impl std::fmt::Display) -> ComponentModuleError {
    ComponentModuleError::Internal(error.to_string())
}

pub(super) async fn list_components(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ComponentSummaryV1>>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.list",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let scopes = authorized_organizations(&grant.payload, READ_CAPABILITY);
    if scopes.is_empty() {
        return Err(ComponentModuleError::Forbidden);
    }
    let rows = sqlx::query(
        "SELECT DISTINCT ON (c.id) c.id,c.name,c.slug,c.description,
                v.id AS version_id,v.dataset_reference,v.component_type::text AS component_type,
                v.status::text AS status,v.lifecycle_state::text AS lifecycle_state,
                v.resource_revision,v.authority_revision,v.version_number,v.version_label,v.version_note,v.config
         FROM components c JOIN component_versions v ON v.component_id=c.id
         WHERE v.status IN ('published','superseded') AND v.lifecycle_state <> 'tombstoned'
           AND v.dataset_scope_node_ids && $1
         ORDER BY c.id,(v.status='published') DESC,v.version_number DESC",
    )
    .bind(scopes.into_iter().collect::<Vec<_>>())
    .fetch_all(&state.pool)
    .await?;
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    rows.into_iter()
        .map(|row| {
            let component_id = row.try_get("id")?;
            Ok(ComponentSummaryV1 {
                schema_version: 1,
                component_id,
                name: row.try_get("name")?,
                slug: row.try_get("slug")?,
                description: row.try_get("description")?,
                current_version: version_from_row(
                    &row,
                    security.installation_id,
                    security.module_instance_id,
                    component_id,
                )?,
            })
        })
        .collect::<Result<Vec<_>, ComponentModuleError>>()
        .map(Json)
}

pub(super) async fn get_component(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> Result<Json<ComponentDefinitionV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let component_id = Uuid::parse_str(&component_ref).ok();
    let component_id = if let Some(id) = component_id {
        id
    } else {
        sqlx::query_scalar("SELECT id FROM components WHERE slug=$1")
            .bind(&component_ref)
            .fetch_optional(&state.pool)
            .await?
            .ok_or_else(|| ComponentModuleError::NotFound("Component not found".into()))?
    };
    get_definition_by_id(&state, &grant.payload, component_id)
        .await
        .map(Json)
}

async fn publish_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.publish_version",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    require_version_scope(
        &state,
        &grant.payload,
        component_id,
        version_id,
        MANAGE_CAPABILITY,
    )
    .await?;
    revalidate_stored_version(&state, &headers, &grant.payload, component_id, version_id).await?;
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let digest = mutation_digest(
        "components.publish_version",
        &[component_id, version_id],
        &(),
    )?;
    let mut transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.publish_version",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }
    sqlx::query("UPDATE component_versions SET status='superseded',successor_version_id=$2,resource_revision=resource_revision+1,updated_at=now() WHERE component_id=$1 AND status='published' AND id<>$2")
        .bind(component_id).bind(version_id).execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO component_version_change_events(component_version_id,resource_revision,category,from_publication_state,to_publication_state) SELECT id,resource_revision,'publication','published','superseded' FROM component_versions WHERE component_id=$1 AND successor_version_id=$2 AND status='superseded'")
        .bind(component_id).bind(version_id).execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO component_version_change_events(component_version_id,resource_revision,category) SELECT id,resource_revision,'successor' FROM component_versions WHERE component_id=$1 AND successor_version_id=$2 AND status='superseded'")
        .bind(component_id).bind(version_id).execute(&mut *transaction).await?;
    let updated = sqlx::query("UPDATE component_versions SET status='published',resource_revision=resource_revision+1,updated_at=now() WHERE id=$1 AND component_id=$2 AND status='draft'")
        .bind(version_id).bind(component_id).execute(&mut *transaction).await?;
    if updated.rows_affected() != 1 {
        return Err(ComponentModuleError::Conflict(
            "Only a draft Component version can be published".into(),
        ));
    }
    sqlx::query("INSERT INTO component_version_change_events(component_version_id,resource_revision,category,from_publication_state,to_publication_state) SELECT id,resource_revision,'publication','draft','published' FROM component_versions WHERE id=$1")
        .bind(version_id).execute(&mut *transaction).await?;
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: Some(version_id),
        outcome: "published".into(),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.publish_version",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn change_lifecycle(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_id, version_id)): Path<(Uuid, Uuid)>,
    Json(request): Json<LifecycleRequestV1>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.change_lifecycle",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    if request.schema_version != 1 || request.expected_resource_revision == 0 {
        return Err(ComponentModuleError::BadRequest(
            "Component lifecycle request is invalid".into(),
        ));
    }
    require_version_scope(
        &state,
        &grant.payload,
        component_id,
        version_id,
        MANAGE_CAPABILITY,
    )
    .await?;
    revalidate_stored_version(&state, &headers, &grant.payload, component_id, version_id).await?;
    let next = match request.action {
        LifecycleActionV1::Activate => "active",
        LifecycleActionV1::Deactivate => "inactive",
        LifecycleActionV1::Archive => "archived",
        LifecycleActionV1::Tombstone => "tombstoned",
    };
    let idempotency_key = mutation_idempotency_key(&headers)?;
    let digest = mutation_digest(
        "components.change_lifecycle",
        &[component_id, version_id],
        &request,
    )?;
    let mut transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.change_lifecycle",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }
    let previous: String = sqlx::query_scalar("SELECT lifecycle_state::text FROM component_versions WHERE id=$1 AND component_id=$2 FOR UPDATE")
        .bind(version_id).bind(component_id).fetch_optional(&mut *transaction).await?.ok_or_else(|| ComponentModuleError::NotFound("Component version not found".into()))?;
    if previous == next {
        return Err(ComponentModuleError::Conflict(
            "Component version already has that lifecycle state".into(),
        ));
    }
    let updated = sqlx::query("UPDATE component_versions SET lifecycle_state=$1::component_lifecycle_state,resource_revision=resource_revision+1,updated_at=now() WHERE id=$2 AND resource_revision=$3")
        .bind(next).bind(version_id).bind(request.expected_resource_revision as i64).execute(&mut *transaction).await?;
    if updated.rows_affected() != 1 {
        return Err(ComponentModuleError::Conflict(
            "Component version revision changed".into(),
        ));
    }
    sqlx::query("INSERT INTO component_version_change_events(component_version_id,resource_revision,category,from_lifecycle_state,to_lifecycle_state) SELECT id,resource_revision,'lifecycle',$1::component_lifecycle_state,$2::component_lifecycle_state FROM component_versions WHERE id=$3")
        .bind(previous).bind(next).bind(version_id).execute(&mut *transaction).await?;
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: Some(version_id),
        outcome: format!("lifecycle_{next}"),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.change_lifecycle",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

pub(super) async fn get_definition_by_id(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV2,
    component_id: Uuid,
) -> Result<ComponentDefinitionV1, ComponentModuleError> {
    let component = sqlx::query("SELECT name,slug,description FROM components WHERE id=$1")
        .bind(component_id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ComponentModuleError::NotFound("Component not found".into()))?;
    let mut version_rows = sqlx::query("SELECT id AS version_id,dataset_reference,component_type::text AS component_type,status::text AS status,lifecycle_state::text AS lifecycle_state,resource_revision,authority_revision,version_number,version_label,version_note,config,dataset_scope_node_ids FROM component_versions WHERE component_id=$1 ORDER BY version_number DESC")
        .bind(component_id).fetch_all(&state.pool).await?;
    let manages = version_rows.iter().any(|row| {
        row.try_get::<Vec<Uuid>, _>("dataset_scope_node_ids")
            .ok()
            .is_some_and(|scope| require_scope(grant, MANAGE_CAPABILITY, &scope).is_ok())
    });
    if !manages {
        if !version_rows.iter().any(|row| {
            row.try_get::<Vec<Uuid>, _>("dataset_scope_node_ids")
                .ok()
                .is_some_and(|scope| require_scope(grant, READ_CAPABILITY, &scope).is_ok())
        }) {
            return Err(ComponentModuleError::Forbidden);
        }
        version_rows.retain(|row| {
            row.try_get::<String, _>("status")
                .ok()
                .is_some_and(|status| status != "draft")
        });
        if version_rows.is_empty() {
            return Err(ComponentModuleError::NotFound("Component not found".into()));
        }
    }
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    let versions = version_rows
        .iter()
        .map(|row| {
            version_from_row(
                row,
                security.installation_id,
                security.module_instance_id,
                component_id,
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(ComponentDefinitionV1 {
        schema_version: 1,
        component_id,
        name: component.try_get("name")?,
        slug: component.try_get("slug")?,
        description: component.try_get("description")?,
        versions,
    })
}

pub(super) async fn resolve_component_id(
    state: &ComponentModuleState,
    component_ref: &str,
) -> Result<Uuid, ComponentModuleError> {
    if let Ok(id) = Uuid::parse_str(component_ref) {
        return Ok(id);
    }
    sqlx::query_scalar("SELECT id FROM components WHERE slug=$1")
        .bind(component_ref)
        .fetch_optional(&state.pool)
        .await?
        .ok_or_else(|| ComponentModuleError::NotFound("Component not found".into()))
}

fn version_from_row(
    row: &sqlx::postgres::PgRow,
    installation_id: Uuid,
    module_instance_id: Uuid,
    component_id: Uuid,
) -> Result<ComponentVersionV1, ComponentModuleError> {
    let version_id: Uuid = row.try_get("version_id")?;
    let reference = ComponentVersionReference::new(
        TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            COMPONENT_RESOURCE_TYPE
                .parse()
                .map_err(|error| ComponentModuleError::Internal(format!("{error}")))?,
            version_id.to_string(),
        )
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
    )
    .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let dataset_reference = serde_json::from_value(row.try_get("dataset_reference")?)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let _ = component_id;
    Ok(ComponentVersionV1 {
        reference,
        component_version_id: version_id,
        dataset_reference,
        component_type: row.try_get("component_type")?,
        publication_state: row.try_get("status")?,
        lifecycle_state: row.try_get("lifecycle_state")?,
        resource_revision: row.try_get::<i64, _>("resource_revision")? as u64,
        authority_revision: row.try_get::<i64, _>("authority_revision")? as u64,
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        version_note: row.try_get("version_note")?,
        config: row.try_get("config")?,
    })
}

async fn require_version_scope(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV2,
    component_id: Uuid,
    version_id: Uuid,
    capability: &str,
) -> Result<(), ComponentModuleError> {
    let scope: Vec<Uuid> = sqlx::query_scalar(
        "SELECT dataset_scope_node_ids FROM component_versions WHERE id=$1 AND component_id=$2",
    )
    .bind(version_id)
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ComponentModuleError::NotFound("Component version not found".into()))?;
    require_scope(grant, capability, &scope)
}

async fn revalidate_stored_version(
    state: &ComponentModuleState,
    headers: &HeaderMap,
    grant: &AuthorizationGrantV2,
    component_id: Uuid,
    version_id: Uuid,
) -> Result<(), ComponentModuleError> {
    let row=sqlx::query("SELECT dataset_reference,component_type::text AS component_type,config,version_note FROM component_versions WHERE id=$1 AND component_id=$2").bind(version_id).bind(component_id).fetch_optional(&state.pool).await?.ok_or_else(||ComponentModuleError::NotFound("Component version not found".into()))?;
    let input = ComponentVersionInputV1 {
        dataset_reference: serde_json::from_value(row.try_get("dataset_reference")?)
            .map_err(internal)?,
        component_type: row.try_get("component_type")?,
        config: row.try_get("config")?,
        version_note: row.try_get("version_note")?,
    };
    let (_, findings) =
        validate_version_input(state, authorization_header(headers)?, grant, &input).await?;
    if findings.is_empty() {
        Ok(())
    } else {
        Err(ComponentModuleError::BadRequest(
            "Stored Component version is incompatible with its Dataset major line".into(),
        ))
    }
}

fn require_scope(
    grant: &AuthorizationGrantV2,
    capability: &str,
    scope: &[Uuid],
) -> Result<(), ComponentModuleError> {
    let capability = SecurityCapabilityId::new(capability)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    if scope
        .iter()
        .any(|node_id| grant.authorizes(&capability, *node_id))
    {
        Ok(())
    } else {
        Err(ComponentModuleError::Forbidden)
    }
}

fn authorized_organizations(grant: &AuthorizationGrantV2, capability: &str) -> BTreeSet<Uuid> {
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

pub(super) async fn authorize(
    state: &ComponentModuleState,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV2>, ComponentModuleError> {
    let encoded = authorization_header(headers)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV2> =
        serde_json::from_slice(&bytes).map_err(|_| ComponentModuleError::Forbidden)?;
    state
        .core_authorization_verifier
        .verify(&envelope)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(ComponentModuleError::Unavailable(
            "Component module is not enabled".into(),
        ));
    }
    let contract = if matches!(
        action,
        "components.list" | "components.get" | "components.resolve" | "components.execute"
    ) {
        COMPONENT_RESOURCE_CONTRACT
    } else {
        COMPONENT_AUTHORING_CONTRACT
    };
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV2 {
            installation_id: security.installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
            audience_module_instance_id: security.module_instance_id,
            dependency_binding: DependencyBindingKey::new(CORE_COMPONENT_BINDING)
                .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
            functional_contract: FunctionalContractId::new(contract)
                .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
            action: action.into(),
            operation,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| ComponentModuleError::Forbidden)?;
    Ok(envelope)
}

fn authorization_header(headers: &HeaderMap) -> Result<&str, ComponentModuleError> {
    headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ComponentModuleError::Forbidden)
}

fn mutation_idempotency_key(headers: &HeaderMap) -> Result<&str, ComponentModuleError> {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .ok_or_else(|| ComponentModuleError::BadRequest("missing X-Idempotency-Key".into()))
}

fn mutation_digest<T: Serialize>(
    action: &str,
    resource_ids: &[Uuid],
    input: &T,
) -> Result<String, ComponentModuleError> {
    let mut digest = Sha256::new();
    digest.update(action.as_bytes());
    for resource_id in resource_ids {
        digest.update([0]);
        digest.update(resource_id.as_bytes());
    }
    digest.update([0]);
    digest.update(serde_json::to_vec(input).map_err(internal)?);
    Ok(format!("sha256:{:x}", digest.finalize()))
}

async fn load_mutation_replay<T: DeserializeOwned>(
    transaction: &mut Transaction<'_, Postgres>,
    grant: &AuthorizationGrantV2,
    action: &str,
    idempotency_key: &str,
    payload_digest: &str,
) -> Result<Option<T>, ComponentModuleError> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1,0))")
        .bind(idempotency_key)
        .execute(&mut **transaction)
        .await?;
    let existing = sqlx::query(
        "SELECT jti,original_actor_id,action,payload_digest,idempotency_key,result
         FROM component_mutation_replays WHERE jti=$1 OR idempotency_key=$2 FOR UPDATE",
    )
    .bind(grant.jti)
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await?;
    let Some(existing) = existing else {
        return Ok(None);
    };
    if existing.try_get::<Uuid, _>("original_actor_id")? != grant.original_actor_id
        || existing.try_get::<String, _>("action")? != action
        || existing.try_get::<String, _>("payload_digest")? != payload_digest
        || existing.try_get::<String, _>("idempotency_key")? != idempotency_key
    {
        return Err(ComponentModuleError::Conflict(
            "Component mutation replay identity was reused with different input".into(),
        ));
    }
    serde_json::from_value(existing.try_get("result")?)
        .map(Some)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))
}

async fn record_mutation_replay<T: Serialize>(
    transaction: &mut Transaction<'_, Postgres>,
    grant: &AuthorizationGrantV2,
    action: &str,
    idempotency_key: &str,
    payload_digest: &str,
    result: &T,
) -> Result<(), ComponentModuleError> {
    sqlx::query(
        "INSERT INTO component_mutation_replays
         (jti,original_actor_id,action,payload_digest,idempotency_key,result)
         VALUES($1,$2,$3,$4,$5,$6)",
    )
    .bind(grant.jti)
    .bind(grant.original_actor_id)
    .bind(action)
    .bind(payload_digest)
    .bind(idempotency_key)
    .bind(serde_json::to_value(result).map_err(internal)?)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}
