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
use sqlx::{PgConnection, Postgres, Row, Transaction};
use tessara_components_contract::{COMPONENT_RESOURCE_TYPE, ComponentVersionReference};
use tessara_datasets_contract::{
    DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY, DATASET_CONTRACT_SCHEMA_VERSION,
    DatasetAction, DatasetCatalogRequest, DatasetCatalogResponse, DatasetCompatibilityRequest,
    DatasetCompatibilityResponse, DatasetDistinctValuesRequest, DatasetDistinctValuesResponse,
    DatasetExecutionResponse, DatasetMajorLineMetadata, DatasetMajorLineReference,
    DatasetSchemaRequest,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, DependencyBindingKey, FunctionalContractId,
    ModuleDefinitionId, ModuleServicePrincipalV1, ResourceOwner, SecurityCapabilityId,
    SignedEnvelopeV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    ComponentModuleError, ComponentModuleState, MANAGE_CAPABILITY, READ_CAPABILITY, dataset_client,
    load_security_state, validation,
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
        .route("/api/admin/components/save", post(save_component_edit))
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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SaveComponentEditV1 {
    schema_version: u16,
    #[serde(default)]
    component_id: Option<Uuid>,
    #[serde(default)]
    draft_version_id: Option<Uuid>,
    #[serde(default)]
    published_version_id: Option<Uuid>,
    action: SaveComponentEditActionV1,
    component: UpdateComponentV1,
    version: ComponentVersionInputV1,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum SaveComponentEditActionV1 {
    SaveDraft,
    UpdateExistingVersion,
    CreateNewVersion,
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
    let grant = authorize(
        &state,
        &headers,
        "components.execute",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let component_id = resolve_component_id(&state, &component_ref).await?;
    let version_id: Uuid = sqlx::query_scalar(
        "SELECT id FROM component_versions
         WHERE component_id=$1 AND status='published'
         ORDER BY version_number DESC LIMIT 1",
    )
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(undisclosed_component_resource)?;
    execute_component(
        state,
        headers,
        grant.payload,
        component_id,
        version_id,
        kind,
        query.0,
    )
    .await
}

async fn execute_component_version(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path((component_ref, version_id, kind)): Path<(String, Uuid, String)>,
    query: RawQuery,
) -> Result<Json<Value>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.execute",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let component_id = resolve_component_id(&state, &component_ref).await?;
    execute_component(
        state,
        headers,
        grant.payload,
        component_id,
        version_id,
        kind,
        query.0,
    )
    .await
}

async fn execute_component(
    state: ComponentModuleState,
    headers: HeaderMap,
    grant: AuthorizationGrantV3,
    component_id: Uuid,
    version_id: Uuid,
    kind: String,
    query: Option<String>,
) -> Result<Json<Value>, ComponentModuleError> {
    let row = sqlx::query(
        "SELECT dataset_reference,dataset_scope_node_ids,component_type::text AS component_type,
                config,status::text AS status,lifecycle_state::text AS lifecycle_state
         FROM component_versions WHERE id=$1 AND component_id=$2",
    )
    .bind(version_id)
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(undisclosed_component_resource)?;
    let scope: Vec<Uuid> = row.try_get("dataset_scope_node_ids")?;
    require_scope(&grant, READ_CAPABILITY, &scope).map_err(|_| undisclosed_component_resource())?;
    let stored_kind: String = row.try_get("component_type")?;
    let requested_kind = kind.replace('-', "_");
    if stored_kind != requested_kind
        || !matches!(
            row.try_get::<String, _>("status")?.as_str(),
            "published" | "superseded"
        )
        || row.try_get::<String, _>("lifecycle_state")? != "active"
    {
        return Err(undisclosed_component_resource());
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
        "/api/private/datasets/execute",
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
    let payload_digest = mutation_digest("components.create", &[], &request)?;

    // A committed create result remains authoritative if the Dataset provider
    // is unavailable when the caller retries. Take the same advisory lock used
    // by the write transaction, then release this read-only preflight before
    // any provider I/O.
    let mut replay_transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut replay_transaction,
        &grant.payload,
        "components.create",
        idempotency_key,
        &payload_digest,
    )
    .await?
    {
        replay_transaction.rollback().await?;
        return Ok(Json(response));
    }
    replay_transaction.rollback().await?;

    // Both versioned Dataset checks finish before the Component write
    // transaction begins, so incompatibility or provider outage cannot leave a
    // partial Component definition, version, or replay record.
    let authorization = authorization_header(&headers)?;
    let (metadata, findings) =
        validate_version_input(&state, authorization, &grant.payload, &request.version).await?;
    if !findings.is_empty() {
        return Err(ComponentModuleError::BadRequest(
            findings
                .into_iter()
                .map(|finding| finding.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    let mut scope_node_ids = metadata.scope_node_ids;
    scope_node_ids.sort_unstable();
    scope_node_ids.dedup();
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    let component_id = Uuid::new_v4();
    let version_id = Uuid::new_v4();
    let mut transaction = state.pool.begin().await?;
    // A concurrent original request may have committed while provider
    // validation ran; recheck under the transaction lock before writing.
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.create",
        idempotency_key,
        &payload_digest,
    )
    .await?
    {
        return Ok(Json(response));
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
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.create",
        idempotency_key,
        &payload_digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

async fn save_component_edit(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<SaveComponentEditV1>,
) -> Result<Json<ComponentMutationResponseV1>, ComponentModuleError> {
    let grant = authorize(
        &state,
        &headers,
        "components.save",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    validate_save_component_edit(&request)?;

    let idempotency_key = mutation_idempotency_key(&headers)?;
    let resource_ids = request.component_id.into_iter().collect::<Vec<_>>();
    let digest = mutation_digest("components.save", &resource_ids, &request)?;

    // A successful replay is authoritative even if the Dataset provider is
    // unavailable by the time the client retries. Acquire the same advisory
    // lock used by the write transaction so a concurrent original request can
    // finish, then release this read-only transaction before provider I/O.
    let mut replay_transaction = state.pool.begin().await?;
    if let Some(response) = load_mutation_replay(
        &mut replay_transaction,
        &grant.payload,
        "components.save",
        idempotency_key,
        &digest,
    )
    .await?
    {
        replay_transaction.rollback().await?;
        return Ok(Json(response));
    }
    replay_transaction.rollback().await?;

    // Dataset validation and scope evaluation intentionally complete before the
    // Component transaction starts. An unavailable or invalid provider result
    // therefore cannot partially persist shell metadata or a version payload.
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
                .map(|finding| finding.message)
                .collect::<Vec<_>>()
                .join("; "),
        ));
    }
    let mut scope_node_ids = metadata.scope_node_ids;
    scope_node_ids.sort_unstable();
    scope_node_ids.dedup();

    let mut transaction = state.pool.begin().await?;
    // Recheck after provider I/O while holding the advisory lock. Another
    // request using this identity may have committed while validation ran.
    if let Some(response) = load_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.save",
        idempotency_key,
        &digest,
    )
    .await?
    {
        return Ok(Json(response));
    }

    let component_id = if let Some(component_id) = request.component_id {
        require_component_fully_manageable_in_transaction(
            &mut transaction,
            &grant.payload,
            component_id,
        )
        .await?;
        let ownership = match (
            request.action,
            request.draft_version_id,
            request.published_version_id,
        ) {
            (SaveComponentEditActionV1::UpdateExistingVersion, None, Some(version_id)) => {
                sqlx::query(
                    "SELECT dataset_scope_node_ids FROM component_versions
                     WHERE component_id=$1 AND id=$2 AND status='published' FOR UPDATE",
                )
                .bind(component_id)
                .bind(version_id)
                .fetch_optional(&mut *transaction)
                .await?
            }
            (
                SaveComponentEditActionV1::SaveDraft | SaveComponentEditActionV1::CreateNewVersion,
                Some(version_id),
                None,
            ) => {
                sqlx::query(
                    "SELECT dataset_scope_node_ids FROM component_versions
                     WHERE component_id=$1 AND id=$2 AND status='draft' FOR UPDATE",
                )
                .bind(component_id)
                .bind(version_id)
                .fetch_optional(&mut *transaction)
                .await?
            }
            (
                SaveComponentEditActionV1::SaveDraft | SaveComponentEditActionV1::CreateNewVersion,
                None,
                None,
            ) => {
                // A caller omitting a draft identity will update the existing
                // draft if one exists, so authorization must bind to that exact
                // version before shell metadata changes. Otherwise the current
                // published version owns creation of the next draft.
                sqlx::query(
                    "SELECT dataset_scope_node_ids FROM component_versions
                     WHERE component_id=$1
                     ORDER BY (status='draft') DESC,version_number DESC
                     LIMIT 1 FOR UPDATE",
                )
                .bind(component_id)
                .fetch_optional(&mut *transaction)
                .await?
            }
            _ => None,
        }
        .ok_or_else(undisclosed_component_resource)?;
        let existing_scope: Vec<Uuid> = ownership.try_get("dataset_scope_node_ids")?;
        require_scope(&grant.payload, MANAGE_CAPABILITY, &existing_scope)
            .map_err(|_| undisclosed_component_resource())?;
        let updated = sqlx::query(
            "UPDATE components SET name=$1,slug=$2,description=$3,updated_at=now() WHERE id=$4",
        )
        .bind(request.component.name.trim())
        .bind(request.component.slug.trim())
        .bind(normalized_optional_text(
            request.component.description.as_deref(),
        ))
        .bind(component_id)
        .execute(&mut *transaction)
        .await?;
        if updated.rows_affected() != 1 {
            return Err(undisclosed_component_resource());
        }
        component_id
    } else {
        let component_id = Uuid::new_v4();
        sqlx::query("INSERT INTO components(id,name,slug,description) VALUES($1,$2,$3,$4)")
            .bind(component_id)
            .bind(request.component.name.trim())
            .bind(request.component.slug.trim())
            .bind(normalized_optional_text(
                request.component.description.as_deref(),
            ))
            .execute(&mut *transaction)
            .await?;
        component_id
    };

    let (component_version_id, outcome) = match request.action {
        SaveComponentEditActionV1::SaveDraft => {
            let version_id = save_component_draft_in_transaction(
                &mut transaction,
                component_id,
                request.draft_version_id,
                &scope_node_ids,
                &request.version,
            )
            .await?;
            (version_id, "draft_saved")
        }
        SaveComponentEditActionV1::CreateNewVersion => {
            let version_id = save_component_draft_in_transaction(
                &mut transaction,
                component_id,
                request.draft_version_id,
                &scope_node_ids,
                &request.version,
            )
            .await?;
            publish_component_version_in_transaction(
                &mut transaction,
                &grant.payload,
                component_id,
                version_id,
            )
            .await?;
            (version_id, "published_new_version")
        }
        SaveComponentEditActionV1::UpdateExistingVersion => {
            let version_id = request.published_version_id.ok_or_else(|| {
                ComponentModuleError::BadRequest(
                    "Updating an existing Component version requires its identity".into(),
                )
            })?;
            update_published_component_in_transaction(
                &mut transaction,
                component_id,
                version_id,
                &scope_node_ids,
                &request.version,
            )
            .await?;
            (version_id, "published_version_updated")
        }
    };
    let response = ComponentMutationResponseV1 {
        schema_version: 1,
        component_id,
        component_version_id: Some(component_version_id),
        outcome: outcome.into(),
    };
    record_mutation_replay(
        &mut transaction,
        &grant.payload,
        "components.save",
        idempotency_key,
        &digest,
        &response,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

fn validate_save_component_edit(request: &SaveComponentEditV1) -> Result<(), ComponentModuleError> {
    let identity_is_consistent = matches!(
        (
            request.component_id,
            request.draft_version_id,
            request.published_version_id,
            request.action,
        ),
        (None, None, None, SaveComponentEditActionV1::SaveDraft)
            | (
                None,
                None,
                None,
                SaveComponentEditActionV1::CreateNewVersion
            )
            | (Some(_), None, None, SaveComponentEditActionV1::SaveDraft)
            | (Some(_), Some(_), None, SaveComponentEditActionV1::SaveDraft)
            | (
                Some(_),
                None,
                None,
                SaveComponentEditActionV1::CreateNewVersion
            )
            | (
                Some(_),
                Some(_),
                None,
                SaveComponentEditActionV1::CreateNewVersion
            )
            | (
                Some(_),
                None,
                Some(_),
                SaveComponentEditActionV1::UpdateExistingVersion
            )
    );
    if request.schema_version != 1
        || request.component.schema_version != 1
        || request.component.name.trim().is_empty()
        || request.component.slug.trim().is_empty()
        || !request.version.config.is_object()
        || !matches!(
            request.version.component_type.as_str(),
            "table" | "bar" | "line" | "pie" | "donut" | "stat_card"
        )
        || !identity_is_consistent
        || (request.action == SaveComponentEditActionV1::CreateNewVersion
            && request.version.version_note.trim().is_empty())
    {
        return Err(ComponentModuleError::BadRequest(
            "Component save request is invalid".into(),
        ));
    }
    Ok(())
}

async fn save_component_draft_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    requested_version_id: Option<Uuid>,
    scope_node_ids: &[Uuid],
    version: &ComponentVersionInputV1,
) -> Result<Uuid, ComponentModuleError> {
    let existing_version_id = if let Some(version_id) = requested_version_id {
        Some(version_id)
    } else {
        sqlx::query_scalar(
            "SELECT id FROM component_versions WHERE component_id=$1 AND status='draft' FOR UPDATE",
        )
        .bind(component_id)
        .fetch_optional(&mut **transaction)
        .await?
    };
    if let Some(version_id) = existing_version_id {
        let updated = sqlx::query(
            "UPDATE component_versions
             SET dataset_reference=$1,dataset_scope_node_ids=$2,component_type=$3::component_type,
                 version_note=$4,config=$5,resource_revision=resource_revision+1,
                 authority_revision=authority_revision+1,updated_at=now()
             WHERE id=$6 AND component_id=$7 AND status='draft'",
        )
        .bind(serde_json::to_value(&version.dataset_reference).map_err(internal)?)
        .bind(scope_node_ids)
        .bind(&version.component_type)
        .bind(version.version_note.trim())
        .bind(&version.config)
        .bind(version_id)
        .bind(component_id)
        .execute(&mut **transaction)
        .await?;
        if updated.rows_affected() != 1 {
            return Err(ComponentModuleError::Conflict(
                "The Component draft changed or no longer exists".into(),
            ));
        }
        sqlx::query(
            "INSERT INTO component_version_change_events(component_version_id,resource_revision,category)
             SELECT id,resource_revision,'payload' FROM component_versions WHERE id=$1",
        )
        .bind(version_id)
        .execute(&mut **transaction)
        .await?;
        return Ok(version_id);
    }

    let version_number: i32 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version_number),0)+1 FROM component_versions WHERE component_id=$1",
    )
    .bind(component_id)
    .fetch_one(&mut **transaction)
    .await?;
    let version_id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO component_versions
         (id,component_id,dataset_reference,dataset_scope_node_ids,component_type,status,
          lifecycle_state,version_number,version_label,version_note,config)
         VALUES($1,$2,$3,$4,$5::component_type,'draft','active',$6,$7,$8,$9)",
    )
    .bind(version_id)
    .bind(component_id)
    .bind(serde_json::to_value(&version.dataset_reference).map_err(internal)?)
    .bind(scope_node_ids)
    .bind(&version.component_type)
    .bind(version_number)
    .bind(format!("{version_number}.0.0"))
    .bind(version.version_note.trim())
    .bind(&version.config)
    .execute(&mut **transaction)
    .await?;
    Ok(version_id)
}

async fn publish_component_version_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
    version_id: Uuid,
) -> Result<(), ComponentModuleError> {
    // Publishing changes both the requested draft and every current
    // publication. Hold the Component parent lock and prove containment over
    // the complete history before superseding anything.
    require_component_fully_manageable_in_transaction(transaction, grant, component_id).await?;
    sqlx::query(
        "UPDATE component_versions
         SET status='superseded',successor_version_id=$2,
             resource_revision=resource_revision+1,updated_at=now()
         WHERE component_id=$1 AND status='published' AND id<>$2",
    )
    .bind(component_id)
    .bind(version_id)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "INSERT INTO component_version_change_events
         (component_version_id,resource_revision,category,from_publication_state,to_publication_state)
         SELECT id,resource_revision,'publication','published','superseded'
         FROM component_versions
         WHERE component_id=$1 AND successor_version_id=$2 AND status='superseded'",
    )
    .bind(component_id)
    .bind(version_id)
    .execute(&mut **transaction)
    .await?;
    sqlx::query(
        "INSERT INTO component_version_change_events(component_version_id,resource_revision,category)
         SELECT id,resource_revision,'successor' FROM component_versions
         WHERE component_id=$1 AND successor_version_id=$2 AND status='superseded'",
    )
    .bind(component_id)
    .bind(version_id)
    .execute(&mut **transaction)
    .await?;
    let updated = sqlx::query(
        "UPDATE component_versions
         SET status='published',resource_revision=resource_revision+1,updated_at=now()
         WHERE id=$1 AND component_id=$2 AND status='draft'",
    )
    .bind(version_id)
    .bind(component_id)
    .execute(&mut **transaction)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(ComponentModuleError::Conflict(
            "Only a draft Component version can be published".into(),
        ));
    }
    sqlx::query(
        "INSERT INTO component_version_change_events
         (component_version_id,resource_revision,category,from_publication_state,to_publication_state)
         SELECT id,resource_revision,'publication','draft','published'
         FROM component_versions WHERE id=$1",
    )
    .bind(version_id)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn update_published_component_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    component_id: Uuid,
    version_id: Uuid,
    scope_node_ids: &[Uuid],
    version: &ComponentVersionInputV1,
) -> Result<(), ComponentModuleError> {
    let updated = sqlx::query(
        "UPDATE component_versions
         SET dataset_reference=$1,dataset_scope_node_ids=$2,component_type=$3::component_type,
             version_note=$4,config=$5,resource_revision=resource_revision+1,
             authority_revision=authority_revision+1,updated_at=now()
         WHERE id=$6 AND component_id=$7 AND status='published'",
    )
    .bind(serde_json::to_value(&version.dataset_reference).map_err(internal)?)
    .bind(scope_node_ids)
    .bind(&version.component_type)
    .bind(version.version_note.trim())
    .bind(&version.config)
    .bind(version_id)
    .bind(component_id)
    .execute(&mut **transaction)
    .await?;
    if updated.rows_affected() != 1 {
        return Err(ComponentModuleError::Conflict(
            "Only the current published Component version can be updated".into(),
        ));
    }
    sqlx::query(
        "INSERT INTO component_version_change_events(component_version_id,resource_revision,category)
         SELECT id,resource_revision,'payload' FROM component_versions WHERE id=$1",
    )
    .bind(version_id)
    .execute(&mut **transaction)
    .await?;
    sqlx::query("DELETE FROM component_versions WHERE component_id=$1 AND status='draft'")
        .bind(component_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

fn normalized_optional_text(value: Option<&str>) -> Option<&str> {
    value.map(str::trim).filter(|value| !value.is_empty())
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
    list_manageable_definitions(&state, &grant.payload)
        .await
        .map(Json)
}

pub(super) async fn list_manageable_definitions(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV3,
) -> Result<Vec<ComponentDefinitionV1>, ComponentModuleError> {
    if !has_component_manage_scope(grant) {
        return Ok(Vec::new());
    }
    let ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT DISTINCT component_id FROM component_versions ORDER BY component_id",
    )
    .fetch_all(&state.pool)
    .await?;
    let mut components = Vec::new();
    for id in ids {
        if let Some(component) =
            classify_manageable_lookup(get_manageable_definition_by_id(state, grant, id).await)?
        {
            components.push(component);
        }
    }
    Ok(components)
}

pub(super) fn has_component_manage_scope(grant: &AuthorizationGrantV3) -> bool {
    !authorized_organizations(grant, MANAGE_CAPABILITY).is_empty()
}

fn classify_manageable_lookup(
    result: Result<ComponentDefinitionV1, ComponentModuleError>,
) -> Result<Option<ComponentDefinitionV1>, ComponentModuleError> {
    match result {
        Ok(component) => Ok(Some(component)),
        Err(ComponentModuleError::NotFound(_)) => Ok(None),
        Err(error) => Err(error),
    }
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
    get_manageable_definition_by_id(&state, &grant.payload, id)
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
    require_component_fully_manageable_in_transaction(
        &mut transaction,
        &grant.payload,
        component_id,
    )
    .await?;
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
    require_component_fully_manageable_in_transaction(
        &mut transaction,
        &grant.payload,
        component_id,
    )
    .await?;
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
    require_component_fully_manageable_in_transaction(
        &mut transaction,
        &grant.payload,
        component_id,
    )
    .await?;
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
    require_component_fully_manageable_in_transaction(
        &mut transaction,
        &grant.payload,
        component_id,
    )
    .await?;
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
        "/api/private/datasets/execute",
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
        "/api/private/datasets/catalog",
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
        "/api/private/datasets/distinct-values",
        &request,
    )
    .await
    .map(Json)
}

async fn validate_version_input(
    state: &ComponentModuleState,
    authorization: &str,
    grant: &AuthorizationGrantV3,
    input: &ComponentVersionInputV1,
) -> Result<(DatasetMajorLineMetadata, Vec<ValidationFindingV1>), ComponentModuleError> {
    let metadata: DatasetMajorLineMetadata = dataset_client::post(
        state,
        authorization,
        "/api/private/datasets/schema",
        &DatasetSchemaRequest {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            action: DatasetAction::ResolveSchema,
            reference: input.dataset_reference.clone(),
        },
    )
    .await?;
    crate::require_ready_dataset_metadata(&metadata, &input.dataset_reference)?;
    require_scope(grant, MANAGE_CAPABILITY, &metadata.scope_node_ids)?;
    let required_fields =
        validation::required_field_requirements(&input.component_type, &input.config);
    let compatibility: DatasetCompatibilityResponse = dataset_client::post(
        state,
        authorization,
        "/api/private/datasets/compatibility",
        &DatasetCompatibilityRequest {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            action: DatasetAction::CheckCompatibility,
            reference: input.dataset_reference.clone(),
            required_fields,
        },
    )
    .await?;
    if compatibility.schema_version != DATASET_CONTRACT_SCHEMA_VERSION {
        return Err(ComponentModuleError::Unavailable(
            "Dataset compatibility response uses an unsupported schema".into(),
        ));
    }
    if compatibility
        .findings
        .iter()
        .any(|finding| finding.code == DATASET_COMPATIBILITY_MATERIALIZATION_NOT_READY)
    {
        return Err(ComponentModuleError::Unavailable(
            "Dataset materialization is not ready; retry the request".into(),
        ));
    }
    let mut findings = component_input_findings(input, &metadata);
    findings.extend(
        compatibility
            .findings
            .into_iter()
            .map(|finding| ValidationFindingV1 {
                code: format!("dataset.compatibility.{}", finding.code),
                field_path: Some(format!("config.fields.{}", finding.field_key)),
                message: format!(
                    "Dataset field '{}' does not satisfy the Component compatibility contract",
                    finding.field_key
                ),
            }),
    );
    if !compatibility.compatible && findings.is_empty() {
        findings.push(ValidationFindingV1 {
            code: "dataset.compatibility.rejected".into(),
            field_path: Some("dataset_reference".into()),
            message: "Dataset rejected the Component compatibility requirements".into(),
        });
    }
    Ok((metadata, findings))
}

fn component_input_findings(
    input: &ComponentVersionInputV1,
    metadata: &DatasetMajorLineMetadata,
) -> Vec<ValidationFindingV1> {
    validation::validate_component_config(&input.component_type, &input.config, &metadata.fields)
        .into_iter()
        .chain(validation::validate_version_note(&input.version_note))
        .map(|finding| ValidationFindingV1 {
            code: finding.code.into(),
            field_path: finding.field_path,
            message: finding.message,
        })
        .collect()
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
    list_component_summaries(&state, &grant.payload)
        .await
        .map(Json)
}

pub(super) async fn list_component_summaries(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV3,
) -> Result<Vec<ComponentSummaryV1>, ComponentModuleError> {
    let scopes = authorized_organizations(grant, READ_CAPABILITY);
    if scopes.is_empty() {
        return Err(ComponentModuleError::Forbidden);
    }
    let rows = sqlx::query(
        "SELECT DISTINCT ON (c.id) c.id,c.name,c.slug,c.description,
                v.id AS version_id,v.dataset_reference,v.component_type::text AS component_type,
                v.status::text AS status,v.lifecycle_state::text AS lifecycle_state,
                v.resource_revision,v.authority_revision,v.version_number,v.version_label,v.version_note,v.config
         FROM components c JOIN component_versions v ON v.component_id=c.id
         WHERE v.status = 'published' AND v.lifecycle_state <> 'tombstoned'
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
    let component_id = resolve_component_id(&state, &component_ref).await?;
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
    publish_component_version_in_transaction(
        &mut transaction,
        &grant.payload,
        component_id,
        version_id,
    )
    .await?;
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
    require_component_fully_manageable_in_transaction(
        &mut transaction,
        &grant.payload,
        component_id,
    )
    .await?;
    let (publication, previous): (String, String) = sqlx::query_as(
        "SELECT status::text,lifecycle_state::text FROM component_versions \
         WHERE id=$1 AND component_id=$2 FOR UPDATE",
    )
    .bind(version_id)
    .bind(component_id)
    .fetch_optional(&mut *transaction)
    .await?
    .ok_or_else(|| ComponentModuleError::NotFound("Component version not found".into()))?;
    if publication == "draft" {
        return Err(ComponentModuleError::BadRequest(
            "Draft Component versions do not have lifecycle actions".into(),
        ));
    }
    let next = lifecycle_transition(&previous, request.action)?;
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

fn lifecycle_transition(
    current: &str,
    action: LifecycleActionV1,
) -> Result<&'static str, ComponentModuleError> {
    match (current, action) {
        ("active", LifecycleActionV1::Deactivate) => Ok("inactive"),
        ("inactive", LifecycleActionV1::Activate) => Ok("active"),
        ("active" | "inactive", LifecycleActionV1::Archive) => Ok("archived"),
        ("archived", LifecycleActionV1::Tombstone) => Ok("tombstoned"),
        _ => Err(ComponentModuleError::BadRequest(format!(
            "Lifecycle action is not allowed from '{current}'"
        ))),
    }
}

pub(super) async fn get_definition_by_id(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
) -> Result<ComponentDefinitionV1, ComponentModuleError> {
    load_component_definition(state, grant, component_id, false).await
}

pub(super) async fn get_manageable_definition_by_id(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
) -> Result<ComponentDefinitionV1, ComponentModuleError> {
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    let mut transaction = state.pool.begin().await?;
    require_component_fully_manageable_in_transaction(&mut transaction, grant, component_id)
        .await?;
    let definition = load_component_definition_from_connection(
        &mut transaction,
        grant,
        component_id,
        true,
        security.installation_id,
        security.module_instance_id,
    )
    .await?;
    transaction.commit().await?;
    Ok(definition)
}

async fn load_component_definition(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
    manageable: bool,
) -> Result<ComponentDefinitionV1, ComponentModuleError> {
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    let mut connection = state.pool.acquire().await?;
    load_component_definition_from_connection(
        &mut connection,
        grant,
        component_id,
        manageable,
        security.installation_id,
        security.module_instance_id,
    )
    .await
}

async fn load_component_definition_from_connection(
    connection: &mut PgConnection,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
    manageable: bool,
    installation_id: Uuid,
    module_instance_id: Uuid,
) -> Result<ComponentDefinitionV1, ComponentModuleError> {
    let component = sqlx::query("SELECT name,slug,description FROM components WHERE id=$1")
        .bind(component_id)
        .fetch_optional(&mut *connection)
        .await?
        .ok_or_else(undisclosed_component_resource)?;
    let mut version_rows = sqlx::query("SELECT id AS version_id,dataset_reference,component_type::text AS component_type,status::text AS status,lifecycle_state::text AS lifecycle_state,resource_revision,authority_revision,version_number,version_label,version_note,config,dataset_scope_node_ids FROM component_versions WHERE component_id=$1 ORDER BY version_number DESC")
        .bind(component_id).fetch_all(&mut *connection).await?;
    if !manageable {
        version_rows.retain(|row| {
            let Ok(scope) = row.try_get::<Vec<Uuid>, _>("dataset_scope_node_ids") else {
                return false;
            };
            let readable = require_scope(grant, READ_CAPABILITY, &scope).is_ok();
            let published = row
                .try_get::<String, _>("status")
                .is_ok_and(|status| status == "published");
            readable && published
        });
    }
    if version_rows.is_empty() {
        // Each version owns its Dataset scope. Access to one version must not
        // disclose sibling drafts or history bound to another scope.
        return Err(undisclosed_component_resource());
    }
    let versions = version_rows
        .iter()
        .map(|row| version_from_row(row, installation_id, module_instance_id, component_id))
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
    sqlx::query_scalar(
        "SELECT id FROM components
         WHERE id::text=$1 OR slug=$1
         ORDER BY (id::text=$1) DESC
         LIMIT 1",
    )
    .bind(component_ref)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(undisclosed_component_resource)
}

fn undisclosed_component_resource() -> ComponentModuleError {
    ComponentModuleError::NotFound("Component resource was not found".into())
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
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
    version_id: Uuid,
    capability: &str,
) -> Result<(), ComponentModuleError> {
    if capability == MANAGE_CAPABILITY {
        // Establish complete-history authority before looking up the requested
        // target so mixed-scope history and target-scope differences remain
        // nondisclosing.
        require_component_fully_manageable(state, grant, component_id).await?;
    }
    let scope: Vec<Uuid> = sqlx::query_scalar(
        "SELECT dataset_scope_node_ids FROM component_versions WHERE id=$1 AND component_id=$2",
    )
    .bind(version_id)
    .bind(component_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(|| ComponentModuleError::NotFound("Component version not found".into()))?;
    require_scope(grant, capability, &scope).map_err(|_| undisclosed_component_resource())?;
    Ok(())
}

async fn require_component_fully_manageable(
    state: &ComponentModuleState,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
) -> Result<(), ComponentModuleError> {
    let scopes = sqlx::query_scalar::<_, Vec<Uuid>>(
        "SELECT dataset_scope_node_ids FROM component_versions
         WHERE component_id=$1 ORDER BY id",
    )
    .bind(component_id)
    .fetch_all(&state.pool)
    .await?;
    if scopes.is_empty()
        || scopes
            .iter()
            .any(|scope| require_scope(grant, MANAGE_CAPABILITY, scope).is_err())
    {
        return Err(undisclosed_component_resource());
    }
    Ok(())
}

async fn require_component_fully_manageable_in_transaction(
    transaction: &mut Transaction<'_, Postgres>,
    grant: &AuthorizationGrantV3,
    component_id: Uuid,
) -> Result<(), ComponentModuleError> {
    // Lock the Component parent as the serialization point for every existing
    // Component mutation. This also prevents a sibling version from being
    // inserted between the complete-history check and the protected write.
    sqlx::query_scalar::<_, Uuid>("SELECT id FROM components WHERE id=$1 FOR UPDATE")
        .bind(component_id)
        .fetch_optional(&mut **transaction)
        .await?
        .ok_or_else(undisclosed_component_resource)?;
    let scopes = sqlx::query_scalar::<_, Vec<Uuid>>(
        "SELECT dataset_scope_node_ids FROM component_versions
         WHERE component_id=$1 ORDER BY id FOR UPDATE",
    )
    .bind(component_id)
    .fetch_all(&mut **transaction)
    .await?;
    if scopes.is_empty()
        || scopes
            .iter()
            .any(|scope| require_scope(grant, MANAGE_CAPABILITY, scope).is_err())
    {
        return Err(undisclosed_component_resource());
    }
    Ok(())
}

async fn revalidate_stored_version(
    state: &ComponentModuleState,
    headers: &HeaderMap,
    grant: &AuthorizationGrantV3,
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
    grant: &AuthorizationGrantV3,
    capability: &str,
    scope: &[Uuid],
) -> Result<(), ComponentModuleError> {
    let capability = SecurityCapabilityId::new(capability)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let authorized = if capability.as_str() == MANAGE_CAPABILITY {
        !scope.is_empty()
            && scope
                .iter()
                .all(|node_id| grant.authorizes(&capability, *node_id))
    } else {
        scope
            .iter()
            .any(|node_id| grant.authorizes(&capability, *node_id))
    };
    if authorized {
        Ok(())
    } else {
        Err(ComponentModuleError::Forbidden)
    }
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

pub(super) async fn authorize(
    state: &ComponentModuleState,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, ComponentModuleError> {
    let encoded = authorization_header(headers)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV3> =
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
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(ComponentModuleError::Forbidden)?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
                    .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
            },
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
    grant: &AuthorizationGrantV3,
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
    grant: &AuthorizationGrantV3,
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

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};
    use serde_json::json;
    use tessara_datasets_contract::DatasetMajorLineReference;
    use uuid::Uuid;

    use super::{
        ComponentVersionInputV1, LifecycleActionV1, SaveComponentEditActionV1, SaveComponentEditV1,
        UpdateComponentV1, classify_manageable_lookup, lifecycle_transition,
        undisclosed_component_resource, validate_save_component_edit,
    };
    use crate::ComponentModuleError;

    fn save_request(
        component_id: Option<Uuid>,
        draft_version_id: Option<Uuid>,
        published_version_id: Option<Uuid>,
        action: SaveComponentEditActionV1,
    ) -> SaveComponentEditV1 {
        SaveComponentEditV1 {
            schema_version: 1,
            component_id,
            draft_version_id,
            published_version_id,
            action,
            component: UpdateComponentV1 {
                schema_version: 1,
                name: "Orders".into(),
                slug: "orders".into(),
                description: None,
            },
            version: ComponentVersionInputV1 {
                dataset_reference: DatasetMajorLineReference::from_parts(
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    1,
                )
                .expect("valid Dataset reference"),
                component_type: "table".into(),
                config: json!({}),
                version_note: "Validated save".into(),
            },
        }
    }

    #[test]
    fn lifecycle_state_machine_retains_the_closed_sprint_7b_contract() {
        assert_eq!(
            lifecycle_transition("active", LifecycleActionV1::Deactivate).expect("deactivate"),
            "inactive"
        );
        assert_eq!(
            lifecycle_transition("inactive", LifecycleActionV1::Activate).expect("activate"),
            "active"
        );
        assert_eq!(
            lifecycle_transition("active", LifecycleActionV1::Archive).expect("archive"),
            "archived"
        );
        assert_eq!(
            lifecycle_transition("archived", LifecycleActionV1::Tombstone).expect("tombstone"),
            "tombstoned"
        );
        for action in [
            LifecycleActionV1::Activate,
            LifecycleActionV1::Deactivate,
            LifecycleActionV1::Archive,
            LifecycleActionV1::Tombstone,
        ] {
            assert!(lifecycle_transition("tombstoned", action).is_err());
        }
        assert!(lifecycle_transition("active", LifecycleActionV1::Tombstone).is_err());
        assert!(lifecycle_transition("archived", LifecycleActionV1::Activate).is_err());
    }

    #[test]
    fn component_save_accepts_only_action_specific_identity_shapes() {
        let component_id = Uuid::from_u128(10);
        let draft_id = Uuid::from_u128(11);
        let published_id = Uuid::from_u128(12);
        let valid = [
            save_request(None, None, None, SaveComponentEditActionV1::SaveDraft),
            save_request(
                None,
                None,
                None,
                SaveComponentEditActionV1::CreateNewVersion,
            ),
            save_request(
                Some(component_id),
                None,
                None,
                SaveComponentEditActionV1::SaveDraft,
            ),
            save_request(
                Some(component_id),
                Some(draft_id),
                None,
                SaveComponentEditActionV1::SaveDraft,
            ),
            save_request(
                Some(component_id),
                None,
                None,
                SaveComponentEditActionV1::CreateNewVersion,
            ),
            save_request(
                Some(component_id),
                Some(draft_id),
                None,
                SaveComponentEditActionV1::CreateNewVersion,
            ),
            save_request(
                Some(component_id),
                None,
                Some(published_id),
                SaveComponentEditActionV1::UpdateExistingVersion,
            ),
        ];
        for request in valid {
            assert!(validate_save_component_edit(&request).is_ok());
        }

        let invalid = [
            // A new Component cannot reference an existing version identity.
            save_request(
                None,
                Some(draft_id),
                None,
                SaveComponentEditActionV1::SaveDraft,
            ),
            save_request(
                None,
                None,
                Some(published_id),
                SaveComponentEditActionV1::CreateNewVersion,
            ),
            // Draft/new-version actions cannot silently ignore a published id.
            save_request(
                Some(component_id),
                None,
                Some(published_id),
                SaveComponentEditActionV1::SaveDraft,
            ),
            save_request(
                Some(component_id),
                Some(draft_id),
                Some(published_id),
                SaveComponentEditActionV1::CreateNewVersion,
            ),
            // Updating a published version requires exactly that identity.
            save_request(
                Some(component_id),
                None,
                None,
                SaveComponentEditActionV1::UpdateExistingVersion,
            ),
            save_request(
                Some(component_id),
                Some(draft_id),
                Some(published_id),
                SaveComponentEditActionV1::UpdateExistingVersion,
            ),
            save_request(
                None,
                None,
                Some(published_id),
                SaveComponentEditActionV1::UpdateExistingVersion,
            ),
        ];
        for request in invalid {
            assert!(validate_save_component_edit(&request).is_err());
        }
    }

    #[test]
    fn manageable_catalog_skips_only_nondisclosing_not_found_results() {
        assert!(matches!(
            classify_manageable_lookup(Err(ComponentModuleError::NotFound("hidden".into()))),
            Ok(None)
        ));
        assert!(matches!(
            classify_manageable_lookup(Err(ComponentModuleError::Unavailable("provider".into()))),
            Err(ComponentModuleError::Unavailable(_))
        ));
        assert!(matches!(
            classify_manageable_lookup(Err(ComponentModuleError::Internal("database".into()))),
            Err(ComponentModuleError::Internal(_))
        ));
    }

    #[tokio::test]
    async fn missing_and_unauthorized_product_references_share_the_exact_not_found_response() {
        let missing = undisclosed_component_resource().into_response();
        let unauthorized = undisclosed_component_resource().into_response();

        assert_eq!(missing.status(), StatusCode::NOT_FOUND);
        assert_eq!(missing.status(), unauthorized.status());
        assert_eq!(
            missing.headers().get(axum::http::header::CONTENT_TYPE),
            unauthorized.headers().get(axum::http::header::CONTENT_TYPE)
        );
        let missing_body = to_bytes(missing.into_body(), usize::MAX)
            .await
            .expect("missing response body");
        let unauthorized_body = to_bytes(unauthorized.into_body(), usize::MAX)
            .await
            .expect("unauthorized response body");
        assert_eq!(missing_body, unauthorized_body);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&missing_body)
                .expect("not-found response JSON"),
            serde_json::json!({
                "schema_version": 1,
                "code": "component.not_found",
                "message": "Component resource was not found",
                "retryable": false,
                "findings": null
            })
        );
    }
}
