use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, put},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use icons::common::{IconType, StaticSvgElement, icon_registry_getter::get_icon_elements};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{FromRow, PgPool, Row};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV1, AuthorizationValidationContextV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, PurposeBoundVerifyingKeyV1,
    SecurityCapabilityId, ShellContextV1, ShellContextValidationContextV1, SignedEnvelopeV1,
};
use tessara_module_runtime::{
    decode_signed_envelope_header, request_correlation_id, verify_shell_context,
};
use tessara_module_ui::{ShellPresentation, render_module_document};
use uuid::Uuid;

pub const MODULE_DEFINITION_ID: &str = "tessara.reference.scoped-records";
pub const MODULE_SHELL_CSS_PATH: &str = "/_tessara/modules/tessara.reference.scoped-records/1.0.0/sha256:fd0c34c22951af76b3c18bcb28d3dfa3641765775dc019bbe50b2a7bce26bee3/module-shell.css";
pub const READ_CAPABILITY: &str = "tessara.reference.scoped-records:read";
pub const MANAGE_CAPABILITY: &str = "tessara.reference.scoped-records:manage";

#[derive(Clone)]
pub struct ModuleState {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScopedRecordsConfigurationV1 {
    pub schema_version: u16,
    pub display_label: String,
    pub retention_mode: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigurationFindingV1 {
    pub code: &'static str,
    pub field: &'static str,
    pub message: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigurationValidationV1 {
    pub schema_version: u16,
    pub valid: bool,
    pub normalized: Option<ScopedRecordsConfigurationV1>,
    pub findings: Vec<ConfigurationFindingV1>,
}

pub fn validate_configuration(input: &ScopedRecordsConfigurationV1) -> ConfigurationValidationV1 {
    let label = input.display_label.trim();
    let mut findings = Vec::new();
    if input.schema_version != 1 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.schema_version.unsupported",
            field: "schema_version",
            message: "Only Scoped Records configuration schema v1 is supported.",
        });
    }
    if label.is_empty() {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.display_label.required",
            field: "display_label",
            message: "Display label is required.",
        });
    } else if label.chars().count() > 80 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.display_label.too_long",
            field: "display_label",
            message: "Display label must contain at most 80 characters.",
        });
    }
    if input.retention_mode != "retain_on_undeploy" {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.retention_mode.unsupported",
            field: "retention_mode",
            message: "Scoped Records v1 retains data when the module is undeployed.",
        });
    }
    ConfigurationValidationV1 {
        schema_version: 1,
        valid: findings.is_empty(),
        normalized: findings.is_empty().then(|| ScopedRecordsConfigurationV1 {
            schema_version: 1,
            display_label: label.to_string(),
            retention_mode: "retain_on_undeploy".into(),
        }),
        findings,
    }
}

#[derive(Clone, Debug, FromRow, Serialize)]
pub struct ScopedRecord {
    pub id: Uuid,
    pub label: String,
    pub organization_owner_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OrganizationAccessProjectionV1 {
    pub organization_id: Uuid,
    pub label: String,
    pub can_manage: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct DirectoryQuery {
    #[serde(default)]
    q: String,
    organization: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RecordInput {
    pub label: String,
    pub organization_owner_id: Uuid,
    pub idempotency_key: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityStateInput {
    schema_version: u16,
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: u64,
    organization_revision: u64,
    enabled: bool,
    document_state: String,
}

#[derive(FromRow)]
struct SecurityState {
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: String,
}

pub fn router(state: ModuleState) -> Router {
    Router::new()
        .route("/", get(directory_page))
        .route("/records/new", get(create_page))
        .route("/records/{record_id}", get(detail_page))
        .route("/records/{record_id}/edit", get(edit_page))
        .route(
            "/api/configuration/validate",
            axum::routing::post(validate_configuration_api),
        )
        .route(
            "/api/configuration",
            get(get_configuration).put(put_configuration),
        )
        .route("/api/private/security-state", put(update_security_state))
        .route("/api/records", get(list_records).post(create_record))
        .route(
            "/api/records/{record_id}",
            get(get_record).put(update_record),
        )
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/health", get(health_page))
        .route("/diagnostics", get(diagnostics_page))
        .route(MODULE_SHELL_CSS_PATH, get(module_shell_stylesheet))
        .with_state(state)
}

async fn validate_configuration_api(
    Json(input): Json<ScopedRecordsConfigurationV1>,
) -> Json<ConfigurationValidationV1> {
    Json(validate_configuration(&input))
}

async fn get_configuration(State(state): State<ModuleState>) -> Result<Json<Value>, ApiError> {
    let row = sqlx::query(
        "SELECT schema_version, display_label, updated_at
         FROM scoped_records_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({
        "schema_version": row.try_get::<i32,_>("schema_version")?,
        "display_label": row.try_get::<String,_>("display_label")?,
        "retention_mode": "retain_on_undeploy",
        "updated_at": row.try_get::<DateTime<Utc>,_>("updated_at")?,
    })))
}

async fn put_configuration(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Json(input): Json<ScopedRecordsConfigurationV1>,
) -> Result<Json<ConfigurationValidationV1>, ApiError> {
    require_private_key(&headers)?;
    let validation = validate_configuration(&input);
    let Some(normalized) = &validation.normalized else {
        return Ok(Json(validation));
    };
    sqlx::query(
        "UPDATE scoped_records_configuration
         SET display_label=$1, updated_at=now() WHERE singleton=true",
    )
    .bind(&normalized.display_label)
    .execute(&state.pool)
    .await?;
    Ok(Json(validation))
}

async fn update_security_state(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Json(input): Json<SecurityStateInput>,
) -> Result<StatusCode, ApiError> {
    require_private_key(&headers)?;
    if input.schema_version != 1
        || !matches!(
            input.document_state.as_str(),
            "enabled" | "disabled" | "degraded" | "recovery"
        )
        || input.authorization_revision == 0
        || input.organization_revision == 0
    {
        return Err(ApiError::bad_request("invalid security state"));
    }
    sqlx::query(
        "INSERT INTO scoped_records_security_state
         (singleton, installation_id, module_instance_id, authorization_revision,
          organization_revision, enabled, document_state)
         VALUES (true,$1,$2,$3,$4,$5,$6)
         ON CONFLICT (singleton) DO UPDATE SET
           installation_id=EXCLUDED.installation_id,
           module_instance_id=EXCLUDED.module_instance_id,
           authorization_revision=EXCLUDED.authorization_revision,
           organization_revision=EXCLUDED.organization_revision,
           enabled=EXCLUDED.enabled, document_state=EXCLUDED.document_state,
           updated_at=now()",
    )
    .bind(input.installation_id)
    .bind(input.module_instance_id)
    .bind(input.authorization_revision as i64)
    .bind(input.organization_revision as i64)
    .bind(input.enabled)
    .bind(input.document_state)
    .execute(&state.pool)
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn list_records(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Json<Vec<ScopedRecord>>, ApiError> {
    Ok(Json(authorized_records(&state, &headers).await?))
}

async fn authorized_records(
    state: &ModuleState,
    headers: &HeaderMap,
) -> Result<Vec<ScopedRecord>, ApiError> {
    let auth = authorize(
        state,
        headers,
        "records.list",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let owners = authorized_owners(&auth, READ_CAPABILITY);
    let records = sqlx::query_as::<_, ScopedRecord>(
        "SELECT id,label,organization_owner_id,created_at,updated_at
         FROM scoped_records WHERE organization_owner_id=ANY($1)
         ORDER BY updated_at DESC,id",
    )
    .bind(owners)
    .fetch_all(&state.pool)
    .await?;
    Ok(records)
}

async fn get_record(
    State(state): State<ModuleState>,
    Path(record_id): Path<Uuid>,
    headers: HeaderMap,
) -> Result<Json<ScopedRecord>, ApiError> {
    let auth = authorize(
        &state,
        &headers,
        "records.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let record = load_record(&state.pool, record_id)
        .await?
        .filter(|record| authorization_allows(&auth, READ_CAPABILITY, record.organization_owner_id))
        .ok_or_else(ApiError::restricted)?;
    Ok(Json(record))
}

async fn create_record(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Json(input): Json<RecordInput>,
) -> Result<impl IntoResponse, ApiError> {
    require_record_input(&input)?;
    let auth = authorize(
        &state,
        &headers,
        "records.create",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    if !authorization_allows(&auth, MANAGE_CAPABILITY, input.organization_owner_id) {
        return Err(ApiError::restricted());
    }
    let payload_digest = payload_digest(&input)?;
    let mut transaction = state.pool.begin().await?;
    if let Some(result) = replay_result(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        &input.idempotency_key,
        &payload_digest,
        "records.create",
    )
    .await?
    {
        transaction.commit().await?;
        return Ok((StatusCode::OK, Json(result)));
    }
    let record = sqlx::query_as::<_, ScopedRecord>(
        "INSERT INTO scoped_records
         (id,label,scope,organization_owner_id)
         VALUES ($1,$2,'sprint-6b2',$3)
         RETURNING id,label,organization_owner_id,created_at,updated_at",
    )
    .bind(Uuid::new_v4())
    .bind(input.label.trim())
    .bind(input.organization_owner_id)
    .fetch_one(&mut *transaction)
    .await?;
    let result = serde_json::to_value(&record)?;
    consume_replay(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        "records.create",
        &payload_digest,
        &input.idempotency_key,
        &result,
    )
    .await?;
    transaction.commit().await?;
    Ok((StatusCode::CREATED, Json(result)))
}

async fn update_record(
    State(state): State<ModuleState>,
    Path(record_id): Path<Uuid>,
    headers: HeaderMap,
    Json(input): Json<RecordInput>,
) -> Result<Json<Value>, ApiError> {
    require_record_input(&input)?;
    let auth = authorize(
        &state,
        &headers,
        "records.update",
        AuthorizationGrantOperationV1::Mutation,
    )
    .await?;
    let current = load_record(&state.pool, record_id)
        .await?
        .ok_or_else(ApiError::restricted)?;
    if !authorization_allows(&auth, MANAGE_CAPABILITY, current.organization_owner_id)
        || !authorization_allows(&auth, MANAGE_CAPABILITY, input.organization_owner_id)
    {
        return Err(ApiError::restricted());
    }
    let payload_digest = payload_digest(&(&record_id, &input))?;
    let mut transaction = state.pool.begin().await?;
    if let Some(result) = replay_result(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        &input.idempotency_key,
        &payload_digest,
        "records.update",
    )
    .await?
    {
        transaction.commit().await?;
        return Ok(Json(result));
    }
    let record = sqlx::query_as::<_, ScopedRecord>(
        "UPDATE scoped_records SET label=$2,organization_owner_id=$3,updated_at=now()
         WHERE id=$1 RETURNING id,label,organization_owner_id,created_at,updated_at",
    )
    .bind(record_id)
    .bind(input.label.trim())
    .bind(input.organization_owner_id)
    .fetch_one(&mut *transaction)
    .await?;
    let result = serde_json::to_value(&record)?;
    consume_replay(
        &mut transaction,
        auth.payload.jti,
        auth.payload.original_actor_id,
        "records.update",
        &payload_digest,
        &input.idempotency_key,
        &result,
    )
    .await?;
    transaction.commit().await?;
    Ok(Json(result))
}

async fn authorize(
    state: &ModuleState,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV1>, ApiError> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::restricted)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ApiError::restricted())?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV1> =
        serde_json::from_slice(&bytes).map_err(|_| ApiError::restricted())?;
    state
        .core_authorization_verifier
        .verify(&envelope)
        .map_err(|_| ApiError::restricted())?;
    let security = load_security_state(&state.pool).await?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(ApiError::unavailable("module is not enabled"));
    }
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV1 {
            installation_id: security.installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core").unwrap(),
            audience_module_instance_id: security.module_instance_id,
            dependency_binding: DependencyBindingKey::new("tessara.core.scoped-records").unwrap(),
            functional_contract: FunctionalContractId::new(
                "tessara.reference.scoped-records.record",
            )
            .unwrap(),
            action: action.to_string(),
            operation,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| ApiError::stale_or_restricted())?;
    Ok(envelope)
}

fn authorized_owners(
    envelope: &SignedEnvelopeV1<AuthorizationGrantV1>,
    capability: &str,
) -> Vec<Uuid> {
    let mut owners = BTreeSet::new();
    for binding in &envelope.payload.capability_scope_bindings {
        if binding.capability.as_str() == capability {
            owners.insert(binding.organization_root_id);
            owners.extend(binding.authorized_organization_ids.iter().copied());
        }
    }
    owners.into_iter().collect()
}

fn authorization_allows(
    envelope: &SignedEnvelopeV1<AuthorizationGrantV1>,
    capability: &str,
    organization_id: Uuid,
) -> bool {
    SecurityCapabilityId::new(capability)
        .ok()
        .is_some_and(|capability| envelope.payload.authorizes(&capability, organization_id))
}

async fn load_security_state(pool: &PgPool) -> Result<SecurityState, ApiError> {
    sqlx::query_as(
        "SELECT installation_id,module_instance_id,authorization_revision,
                organization_revision,enabled,document_state
         FROM scoped_records_security_state WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| ApiError::unavailable("module security state is unavailable"))
}

async fn load_record(pool: &PgPool, id: Uuid) -> Result<Option<ScopedRecord>, ApiError> {
    Ok(sqlx::query_as(
        "SELECT id,label,organization_owner_id,created_at,updated_at
         FROM scoped_records WHERE id=$1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?)
}

fn require_record_input(input: &RecordInput) -> Result<(), ApiError> {
    if input.label.trim().is_empty()
        || input.label.chars().count() > 160
        || input.organization_owner_id.is_nil()
        || input.idempotency_key.trim().is_empty()
    {
        return Err(ApiError::bad_request("record input is invalid"));
    }
    Ok(())
}

fn payload_digest(value: &impl Serialize) -> Result<String, ApiError> {
    Ok(format!(
        "sha256:{:x}",
        Sha256::digest(serde_json::to_vec(value)?)
    ))
}

async fn replay_result(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    jti: Uuid,
    original_actor_id: Uuid,
    idempotency_key: &str,
    digest: &str,
    action: &str,
) -> Result<Option<Value>, ApiError> {
    let row = sqlx::query(
        "SELECT original_actor_id,action,payload_digest,idempotency_key,result
         FROM scoped_records_mutation_replays
         WHERE jti=$1 OR idempotency_key=$2 FOR UPDATE",
    )
    .bind(jti)
    .bind(idempotency_key)
    .fetch_optional(&mut **transaction)
    .await?;
    match row {
        None => Ok(None),
        Some(row)
            if row.try_get::<Uuid, _>("original_actor_id")? == original_actor_id
                && row.try_get::<String, _>("action")? == action
                && row.try_get::<String, _>("payload_digest")? == digest
                && row.try_get::<String, _>("idempotency_key")? == idempotency_key =>
        {
            Ok(Some(row.try_get("result")?))
        }
        Some(_) => Err(ApiError::restricted()),
    }
}

async fn consume_replay(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    jti: Uuid,
    original_actor_id: Uuid,
    action: &str,
    digest: &str,
    idempotency_key: &str,
    result: &Value,
) -> Result<(), ApiError> {
    sqlx::query(
        "INSERT INTO scoped_records_mutation_replays
         (jti,original_actor_id,action,payload_digest,idempotency_key,result)
         VALUES ($1,$2,$3,$4,$5,$6)",
    )
    .bind(jti)
    .bind(original_actor_id)
    .bind(action)
    .bind(digest)
    .bind(idempotency_key)
    .bind(result)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn directory_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Query(query): Query<DirectoryQuery>,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_html(&load_security_state(&state.pool).await?) {
        return shell_page(
            &state,
            &headers,
            &configuration_label(&state.pool).await,
            &content,
        )
        .await;
    }
    let organizations = organization_access_projection(&headers)?;
    let records = authorized_records(&state, &headers).await?;
    let records = filter_directory_records(records, &organizations, &query);
    shell_page(
        &state,
        &headers,
        &configuration_label(&state.pool).await,
        &directory_html(&records, &organizations, &query),
    )
    .await
}

fn directory_html(
    records: &[ScopedRecord],
    organizations: &[OrganizationAccessProjectionV1],
    query: &DirectoryQuery,
) -> String {
    let organization_map = organizations
        .iter()
        .map(|organization| (organization.organization_id, organization))
        .collect::<BTreeMap<_, _>>();
    let rows = records
        .iter()
        .map(|record| {
            let organization = organization_map.get(&record.organization_owner_id);
            let owner_label = organization
                .map(|value| escape(&value.label))
                .unwrap_or_else(|| "Unavailable Organization".into());
            let authority = if organization.is_some_and(|value| value.can_manage) {
                "<span class=\"status-badge is-success\">Read · Manage</span>"
            } else {
                "<span class=\"status-badge is-info\">Read</span>"
            };
            format!(
                "<tr><th><a class=\"scoped-records-primary-link\" href=\"/reference/scoped-records/records/{id}\">{label}</a><code>{id}</code></th><td>{owner_label}</td><td>{updated}</td><td>{authority}</td></tr>",
                id = record.id,
                label = escape(&record.label),
                updated = record.updated_at.format("%b %e, %Y · %l:%M %p UTC"),
            )
        })
        .collect::<String>();
    let directory = if records.is_empty() {
        "<div class=\"organization-detail-card empty-state\"><h2>No scoped records</h2><p>No records are owned by an Organization in your current read scope.</p></div>".to_string()
    } else {
        format!(
            "<div class=\"scoped-records-table-wrap\"><table class=\"scoped-records-table\"><thead><tr><th>Record</th><th>Organization owner</th><th>Updated</th><th>Authority</th></tr></thead><tbody>{rows}</tbody></table></div><div class=\"scoped-records-pagination\"><span>Showing 1-{} of {} records</span><span>Rows <strong>10</strong> · Page 1 of 1</span></div>",
            records.len(),
            records.len(),
        )
    };
    let options = organizations
        .iter()
        .map(|organization| {
            let selected = if query
                .organization
                .as_deref()
                .and_then(|value| Uuid::parse_str(value).ok())
                == Some(organization.organization_id)
            {
                " selected"
            } else {
                ""
            };
            format!(
                "<option value=\"{}\"{}>{}</option>",
                organization.organization_id,
                selected,
                escape(&organization.label)
            )
        })
        .collect::<String>();
    let readable = organizations.len();
    let manageable = organizations
        .iter()
        .filter(|organization| organization.can_manage)
        .count();
    let create_action = if manageable > 0 {
        format!(
            "<a class=\"button\" href=\"/reference/scoped-records/records/new\">{}New Record</a>",
            icon_html(IconType::Plus, "button__icon")
        )
    } else {
        String::new()
    };
    let breadcrumb = scoped_records_breadcrumb(&[("Home", Some("/")), ("Scoped Records", None)]);
    format!(
        "{breadcrumb}<div class=\"scoped-records-heading\"><div><h1>Scoped Records</h1><p>Organization-owned reference records available within your assigned read scope.</p></div>{create_action}</div>\
         <div class=\"scoped-records-scope-summary\"><div><strong>Read access across {readable} accessible Organizations</strong><span>{manageable} include manage authority</span></div><a href=\"/administration/roles\">View access</a></div>\
         <form class=\"scoped-records-toolbar\" method=\"get\" action=\"/reference/scoped-records\"><input type=\"search\" name=\"q\" value=\"{}\" placeholder=\"Search record label, ID, or Organization\"><select name=\"organization\" aria-label=\"Filter by Organization\"><option value=\"\">All accessible Organizations</option>{options}</select><button class=\"button button--secondary\" type=\"submit\">Filter</button></form>{directory}",
        escape(&query.q)
    )
}

fn filter_directory_records(
    records: Vec<ScopedRecord>,
    organizations: &[OrganizationAccessProjectionV1],
    query: &DirectoryQuery,
) -> Vec<ScopedRecord> {
    let labels = organizations
        .iter()
        .map(|organization| {
            (
                organization.organization_id,
                organization.label.to_lowercase(),
            )
        })
        .collect::<BTreeMap<_, _>>();
    let needle = query.q.trim().to_lowercase();
    let selected_organization = query
        .organization
        .as_deref()
        .filter(|value| !value.is_empty())
        .and_then(|value| Uuid::parse_str(value).ok());
    records
        .into_iter()
        .filter(|record| {
            selected_organization
                .is_none_or(|organization| organization == record.organization_owner_id)
                && (needle.is_empty()
                    || record.label.to_lowercase().contains(&needle)
                    || record.id.to_string().contains(&needle)
                    || labels
                        .get(&record.organization_owner_id)
                        .is_some_and(|label| label.contains(&needle)))
        })
        .collect()
}

fn organization_access_projection(
    headers: &HeaderMap,
) -> Result<Vec<OrganizationAccessProjectionV1>, ApiError> {
    let encoded = headers
        .get("x-tessara-organization-access")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(ApiError::restricted)?;
    serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| ApiError::restricted())?,
    )
    .map_err(|_| ApiError::restricted())
}

fn has_manage_authority(organizations: &[OrganizationAccessProjectionV1]) -> bool {
    organizations
        .iter()
        .any(|organization| organization.can_manage)
}

async fn detail_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Path(record_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_html(&load_security_state(&state.pool).await?) {
        return shell_page(&state, &headers, "Scoped Records", &content).await;
    }
    let auth = authorize(
        &state,
        &headers,
        "records.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let record = load_record(&state.pool, record_id)
        .await?
        .filter(|record| authorization_allows(&auth, READ_CAPABILITY, record.organization_owner_id))
        .ok_or_else(ApiError::restricted)?;
    let organizations = organization_access_projection(&headers)?;
    let organization = organizations
        .iter()
        .find(|value| value.organization_id == record.organization_owner_id);
    let owner_label = organization
        .map(|value| escape(&value.label))
        .unwrap_or_else(|| "Unavailable Organization".into());
    let can_manage = organization.is_some_and(|value| value.can_manage);
    let edit_action = if can_manage {
        format!(
            "<a class=\"button\" href=\"/reference/scoped-records/records/{record_id}/edit\">{}Edit Record</a>",
            icon_html(IconType::Pencil, "button__icon")
        )
    } else {
        String::new()
    };
    let record_id_text = record_id.to_string();
    let breadcrumb = scoped_records_breadcrumb(&[
        ("Home", Some("/")),
        ("Scoped Records", Some("/reference/scoped-records")),
        (&record_id_text, None),
    ]);
    let body = format!(
        "{breadcrumb}<div class=\"scoped-records-heading\"><div><h1>{label}</h1><p>{record_id}</p></div><div class=\"scoped-records-actions\"><a class=\"button button--secondary\" href=\"/reference/scoped-records\">Back to Records</a>{edit_action}</div></div>\
         <div class=\"scoped-records-detail-grid\"><section class=\"scoped-records-card\"><header><div><h2>Record</h2><p>Product data owned by the Scoped Records Module Instance.</p></div><span class=\"status-badge {badge_class}\">{authority}</span></header><dl><div><dt>Record ID</dt><dd><code>{record_id}</code></dd></div><div><dt>Label</dt><dd>{label}</dd></div><div><dt>Organization owner</dt><dd>{owner_label} <code>{owner_id}</code></dd></div><div><dt>Created</dt><dd>{created}</dd></div><div><dt>Last updated</dt><dd>{updated}</dd></div></dl></section>\
         <aside class=\"scoped-records-card\"><header><div><h2>Authorization context</h2><p>Current Core decision for this module action.</p></div></header><div class=\"scoped-records-auth-context\"><div><span>Capability</span><code>{read_capability}</code></div><div><span>Authorized Organization</span><strong>{owner_label}</strong></div><div><span>Decision freshness</span><span class=\"status-badge is-success\">Current</span></div><div><span>Presenting service</span><code>tessara.reference.scoped-records</code></div></div><div class=\"scoped-records-notice\"><strong>Core credentials are not shared</strong><span>This module received only a short-lived, audience-bound decision.</span></div></aside></div>",
        label = escape(&record.label),
        owner_id = record.organization_owner_id,
        created = record.created_at.format("%b %e, %Y · %l:%M %p UTC"),
        updated = record.updated_at.format("%b %e, %Y · %l:%M %p UTC"),
        badge_class = if can_manage { "is-success" } else { "is-info" },
        authority = if can_manage { "Read · Manage" } else { "Read" },
        read_capability = READ_CAPABILITY,
    );
    shell_page(&state, &headers, &record.label, &body).await
}

async fn create_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_html(&load_security_state(&state.pool).await?) {
        return shell_page(&state, &headers, "Scoped Records", &content).await;
    }
    let organizations = organization_access_projection(&headers)?;
    if !has_manage_authority(&organizations) {
        return shell_page(&state, &headers, "Scoped Records", &manage_denied_html()).await;
    }
    shell_page(
        &state,
        &headers,
        "Create Record",
        &record_form_html(None, &organizations),
    )
    .await
}

async fn edit_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
    Path(record_id): Path<Uuid>,
) -> Result<Response, ApiError> {
    if let Some(content) = product_state_html(&load_security_state(&state.pool).await?) {
        return shell_page(&state, &headers, "Scoped Records", &content).await;
    }
    let auth = authorize(
        &state,
        &headers,
        "records.get",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let record = load_record(&state.pool, record_id)
        .await?
        .filter(|record| authorization_allows(&auth, READ_CAPABILITY, record.organization_owner_id))
        .ok_or_else(ApiError::restricted)?;
    let organizations = organization_access_projection(&headers)?;
    if !organizations
        .iter()
        .any(|value| value.organization_id == record.organization_owner_id && value.can_manage)
    {
        return shell_page(&state, &headers, "Scoped Records", &manage_denied_html()).await;
    }
    shell_page(
        &state,
        &headers,
        "Edit Record",
        &record_form_html(Some(&record), &organizations),
    )
    .await
}

fn record_form_html(
    record: Option<&ScopedRecord>,
    organizations: &[OrganizationAccessProjectionV1],
) -> String {
    let editing = record.is_some();
    let title = if editing { "Edit Record" } else { "New Record" };
    let submit_label = if editing {
        "Save Record"
    } else {
        "Create Record"
    };
    let action = record.map_or_else(
        || "/reference/scoped-records/records".to_string(),
        |record| format!("/reference/scoped-records/records/{}", record.id),
    );
    let cancel = record.map_or_else(
        || "/reference/scoped-records".to_string(),
        |record| format!("/reference/scoped-records/records/{}", record.id),
    );
    let options = organizations
        .iter()
        .filter(|organization| organization.can_manage)
        .map(|organization| {
            let selected = if record
                .is_some_and(|record| record.organization_owner_id == organization.organization_id)
            {
                " selected"
            } else {
                ""
            };
            format!(
                "<option value=\"{}\"{}>{}</option>",
                organization.organization_id,
                selected,
                escape(&organization.label)
            )
        })
        .collect::<String>();
    let label = record
        .map(|record| escape(&record.label))
        .unwrap_or_default();
    let record_code = record
        .map(|record| format!("<code>{}</code>", record.id))
        .unwrap_or_default();
    let record_id_text = record.map(|record| record.id.to_string());
    let breadcrumb = if let Some(record_id) = record_id_text.as_deref() {
        scoped_records_breadcrumb(&[
            ("Home", Some("/")),
            ("Scoped Records", Some("/reference/scoped-records")),
            (
                record_id,
                Some(&format!("/reference/scoped-records/records/{record_id}")),
            ),
            ("Edit", None),
        ])
    } else {
        scoped_records_breadcrumb(&[
            ("Home", Some("/")),
            ("Scoped Records", Some("/reference/scoped-records")),
            ("New Record", None),
        ])
    };
    format!(
        "{breadcrumb}<div class=\"scoped-records-heading\"><div><h1>{title}</h1><p>Manage authority is checked against the selected Organization subtree when saved.</p></div></div>\
         <form class=\"scoped-records-card scoped-records-form\" method=\"post\" action=\"{action}\"><header><div><h2>Record details</h2><p>Fields and validation belong to Scoped Records.</p></div>{record_code}</header><div class=\"scoped-records-form-grid\"><label><span>Label</span><input name=\"label\" value=\"{label}\" placeholder=\"Enter a clear record label\" maxlength=\"200\" required></label><label><span>Organization owner</span><select name=\"organization_owner_id\" required>{options}</select></label></div><input type=\"hidden\" name=\"idempotency_key\" value=\"{}\"><div class=\"scoped-records-validation\"><strong>Manage authority confirmed</strong><span>Only Organizations covered by your current manage authority are available.</span></div><div class=\"scoped-records-form-actions\"><a class=\"button button--secondary\" href=\"{cancel}\">Cancel</a><button class=\"button\" type=\"submit\">{submit_label}</button></div></form>",
        Uuid::new_v4(),
    )
}

async fn health_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let status = if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_ok()
    {
        "Passing"
    } else {
        "Unavailable"
    };
    shell_page(
        &state,
        &headers,
        "Scoped Records",
        &format!(
            "{}<div class=\"scoped-records-heading\"><div><h1>Scoped Records health</h1><p>Module-owned operational detail with Core installation context.</p></div><a class=\"button button--secondary\" href=\"/reference/scoped-records/health\">{}Refresh status</a></div><nav class=\"scoped-records-tabs\"><a class=\"is-active\" href=\"/reference/scoped-records/health\">Health</a><a href=\"/reference/scoped-records/diagnostics\">Diagnostics</a></nav><div class=\"scoped-records-diagnostic-grid\"><article>{}<div><h2>Readiness</h2><strong>{status}</strong><small>Module can serve authorized product requests.</small></div></article><article>{}<div><h2>Liveness</h2><strong>Passing</strong><small>Module process is responding.</small></div></article><article>{}<div><h2>Configuration</h2><strong>Valid</strong><small>Schema v1 · no findings.</small></div></article><article>{}<div><h2>Core authorization</h2><strong>Connected</strong><small>Signed decision exchange is available.</small></div></article></div>",
            scoped_records_breadcrumb(&[
                ("Home", Some("/")),
                ("Scoped Records", Some("/reference/scoped-records")),
                ("Health & diagnostics", None),
            ]),
            icon_html(IconType::RefreshCw, "button__icon"),
            icon_html(IconType::Activity, "scoped-records-diagnostic-icon"),
            icon_html(IconType::Activity, "scoped-records-diagnostic-icon"),
            icon_html(IconType::Activity, "scoped-records-diagnostic-icon"),
            icon_html(IconType::Activity, "scoped-records-diagnostic-icon"),
        ),
    )
    .await
}

async fn diagnostics_page(
    State(state): State<ModuleState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let security = load_security_state(&state.pool).await.ok();
    let module_instance = security
        .as_ref()
        .map(|value| value.module_instance_id.to_string())
        .unwrap_or_else(|| "Unavailable".into());
    shell_page(
        &state,
        &headers,
        "Scoped Records",
        &format!(
            "{}<div class=\"scoped-records-heading\"><div><h1>Scoped Records health</h1><p>Module-owned operational detail with Core installation context.</p></div><a class=\"button button--secondary\" href=\"/reference/scoped-records/diagnostics\">{}Refresh status</a></div><nav class=\"scoped-records-tabs\"><a href=\"/reference/scoped-records/health\">Health</a><a class=\"is-active\" href=\"/reference/scoped-records/diagnostics\">Diagnostics</a></nav><div class=\"scoped-records-detail-grid scoped-records-diagnostics\"><section class=\"scoped-records-card\"><header><div><h2>Diagnostic context</h2><p>Shareable values are sanitized and contain no claim secrets or Core credentials.</p></div></header><dl><div><dt>Module version</dt><dd>1.0.0</dd></div><div><dt>Module Instance</dt><dd><code>{module_instance}</code></dd></div><div><dt>Database binding</dt><dd><code>tessara_module_scoped_records</code></dd></div><div><dt>Authorization revision</dt><dd><code>auth:{authorization_revision}</code></dd></div><div><dt>Organization revision</dt><dd><code>org:{organization_revision}</code></dd></div></dl></section><aside class=\"scoped-records-card\"><header><div><h2>Recent findings</h2><p>Stable codes from module-owned validation and health checks.</p></div></header><div class=\"scoped-records-empty\">{}<strong>No active findings</strong><span>All required contracts and probes currently pass.</span></div><a class=\"button button--secondary\" download=\"scoped-records-diagnostics.json\" href=\"data:application/json,%7B%22schema_version%22%3A1%2C%22module%22%3A%22tessara.reference.scoped-records%22%7D\">Download sanitized diagnostics</a></aside></div>",
            scoped_records_breadcrumb(&[
                ("Home", Some("/")),
                ("Scoped Records", Some("/reference/scoped-records")),
                ("Health & diagnostics", None),
            ]),
            icon_html(IconType::RefreshCw, "button__icon"),
            icon_html(IconType::CircleCheck, "scoped-records-empty-icon"),
            authorization_revision = security.as_ref().map(|value| value.authorization_revision).unwrap_or_default(),
            organization_revision = security.as_ref().map(|value| value.organization_revision).unwrap_or_default(),
        ),
    )
    .await
}

async fn live() -> StatusCode {
    StatusCode::NO_CONTENT
}

async fn ready(State(state): State<ModuleState>) -> StatusCode {
    let config = get_configuration(State(state.clone())).await;
    let security = load_security_state(&state.pool).await;
    if config.is_ok()
        && security.is_ok_and(|value| value.enabled && value.document_state == "enabled")
    {
        StatusCode::NO_CONTENT
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    }
}

async fn configuration_label(pool: &PgPool) -> String {
    sqlx::query_scalar(
        "SELECT display_label FROM scoped_records_configuration WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .unwrap_or_else(|| "Scoped Records".into())
}

fn product_state_html(security: &SecurityState) -> Option<String> {
    let (eyebrow, title, message, action_label, action_href) = if !security.enabled
        || security.document_state == "disabled"
    {
        (
            "Module disabled",
            "Scoped Records is not serving product routes",
            "Configuration, health, and diagnostics remain available while product access is disabled.",
            "Open Module Management",
            "/administration/modules/tessara.reference.scoped-records",
        )
    } else if security.document_state == "degraded" {
        (
            "Module needs attention",
            "Scoped Records cannot serve this route reliably",
            "Review module diagnostics before continuing with product work.",
            "Open diagnostics",
            "/reference/scoped-records/diagnostics",
        )
    } else {
        return None;
    };

    Some(format!(
        "{}<section class=\"scoped-records-state-treatment\"><span class=\"scoped-records-state-treatment__eyebrow\">{eyebrow}</span><h1>{title}</h1><p>{message}</p><a class=\"button\" href=\"{action_href}\">{action_label}</a></section>\
         <section class=\"scoped-records-state-context\"><h2>Protected context</h2><dl><div><dt>Module</dt><dd>tessara.reference.scoped-records</dd></div><div><dt>Lifecycle state</dt><dd>{state}</dd></div><div><dt>Product data</dt><dd>Retained</dd></div></dl></section>",
        scoped_records_breadcrumb(&[
            ("Home", Some("/")),
            ("Scoped Records", Some("/reference/scoped-records")),
            ("State", None),
        ]),
        state = escape(&security.document_state),
    ))
}

fn manage_denied_html() -> String {
    format!(
        "{}<section class=\"scoped-records-state-treatment\"><span class=\"scoped-records-state-treatment__eyebrow\">Scoped action unavailable</span><h1>You can’t manage this record</h1><p>Your current access permits reading Scoped Records, but not creating or changing them.</p><a class=\"button\" href=\"/reference/scoped-records\">Back to Records</a></section>\
         <section class=\"scoped-records-state-context\"><h2>Protected context</h2><dl><div><dt>Required capability</dt><dd><code>tessara.reference.scoped-records:manage</code></dd></div><div><dt>Current route</dt><dd>Read-only</dd></div><div><dt>Disclosure</dt><dd>No unavailable record details shown</dd></div></dl></section>",
        scoped_records_breadcrumb(&[
            ("Home", Some("/")),
            ("Scoped Records", Some("/reference/scoped-records")),
            ("Manage access", None),
        ])
    )
}

async fn shell_page(
    state: &ModuleState,
    headers: &HeaderMap,
    heading: &str,
    body: &str,
) -> Result<Response, ApiError> {
    let correlation_id =
        request_correlation_id(headers).map_err(|_| ApiError::shell_unavailable())?;
    let envelope: SignedEnvelopeV1<ShellContextV1> =
        decode_signed_envelope_header(headers, "x-tessara-shell-context")
            .map_err(|_| ApiError::shell_unavailable())?;
    let security = load_security_state(&state.pool).await?;
    verify_shell_context(
        &envelope,
        &state.core_shell_verifier,
        &ShellContextValidationContextV1 {
            installation_id: security.installation_id,
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                .map_err(|_| ApiError::shell_unavailable())?,
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: Utc::now(),
        },
    )
    .map_err(|_| ApiError::shell_unavailable())?;
    let presentation = ShellPresentation::from_verified_context(
        &envelope.payload,
        headers
            .get("x-tessara-original-path")
            .and_then(|value| value.to_str().ok())
            .unwrap_or("/reference/scoped-records"),
        heading,
    );
    Ok(Html(render_module_document(
        &presentation,
        MODULE_SHELL_CSS_PATH,
        None,
        body,
    ))
    .into_response())
}

async fn module_shell_stylesheet() -> Response {
    (
        [(header::CONTENT_TYPE, "text/css; charset=utf-8")],
        tessara_module_ui::MODULE_SHELL_CSS,
    )
        .into_response()
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn scoped_records_breadcrumb(items: &[(&str, Option<&str>)]) -> String {
    let mut html =
        "<nav class=\"breadcrumb\" aria-label=\"Breadcrumb\"><ol class=\"breadcrumb__list\">"
            .to_string();
    for (index, (label, href)) in items.iter().enumerate() {
        if index > 0 {
            html.push_str("<li class=\"breadcrumb__separator\" aria-hidden=\"true\">");
            html.push_str(&icon_html(
                IconType::ChevronRight,
                "breadcrumb__separator-icon",
            ));
            html.push_str("</li>");
        }
        html.push_str("<li class=\"breadcrumb__item\">");
        if let Some(href) = href {
            html.push_str(&format!(
                "<a class=\"breadcrumb__link\" href=\"{}\">{}</a>",
                escape(href),
                escape(label)
            ));
        } else {
            html.push_str(&format!(
                "<span class=\"breadcrumb__page\" aria-current=\"page\">{}</span>",
                escape(label)
            ));
        }
        html.push_str("</li>");
    }
    html.push_str("</ol></nav>");
    html
}

fn icon_html(icon: IconType, class: &str) -> String {
    let mut html = format!(
        "<svg class=\"{}\" xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\">",
        escape(class)
    );
    if let Some(elements) = get_icon_elements(icon) {
        for element in elements {
            match element {
                StaticSvgElement::Path { d } => {
                    html.push_str(&format!("<path d=\"{d}\"></path>"));
                }
                StaticSvgElement::Circle { cx, cy, r } => {
                    html.push_str(&format!(
                        "<circle cx=\"{cx}\" cy=\"{cy}\" r=\"{r}\"></circle>"
                    ));
                }
                StaticSvgElement::Rect {
                    x,
                    y,
                    width,
                    height,
                    rx,
                    ry,
                } => {
                    html.push_str(&format!(
                        "<rect x=\"{x}\" y=\"{y}\" width=\"{width}\" height=\"{height}\"{}{}></rect>",
                        rx.map(|value| format!(" rx=\"{value}\"")).unwrap_or_default(),
                        ry.map(|value| format!(" ry=\"{value}\"")).unwrap_or_default(),
                    ));
                }
                StaticSvgElement::Ellipse { cx, cy, rx, ry } => {
                    html.push_str(&format!(
                        "<ellipse cx=\"{cx}\" cy=\"{cy}\" rx=\"{rx}\" ry=\"{ry}\"></ellipse>"
                    ));
                }
                StaticSvgElement::Line { x1, y1, x2, y2 } => {
                    html.push_str(&format!(
                        "<line x1=\"{x1}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\"></line>"
                    ));
                }
                StaticSvgElement::Polyline { points } => {
                    html.push_str(&format!("<polyline points=\"{points}\"></polyline>"));
                }
                StaticSvgElement::Polygon { points } => {
                    html.push_str(&format!("<polygon points=\"{points}\"></polygon>"));
                }
            }
        }
    }
    html.push_str("</svg>");
    html
}

fn require_private_key(headers: &HeaderMap) -> Result<(), ApiError> {
    let expected = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    if headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
    {
        Ok(())
    } else {
        Err(ApiError {
            status: StatusCode::UNAUTHORIZED,
            code: "unauthorized",
            message: "Request unavailable".into(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{message}")]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn shell_unavailable() -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code: "shell_context_unavailable",
            message: "Open this module through Core.".into(),
        }
    }
    fn bad_request(message: &str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code: "invalid_request",
            message: message.into(),
        }
    }
    fn restricted() -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code: "record_unavailable",
            message: "The requested record or action is unavailable.".into(),
        }
    }
    fn stale_or_restricted() -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code: "authorization_stale",
            message: "Authorization changed. Refresh through Core and try again.".into(),
        }
    }
    fn unavailable(message: &str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code: "module_unavailable",
            message: message.into(),
        }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(error: sqlx::Error) -> Self {
        tracing::error!(%error, "scoped records database request failed");
        Self::unavailable("The module could not complete the request.")
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(_: serde_json::Error) -> Self {
        Self::bad_request("JSON input is invalid")
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        (
            self.status,
            [(header::CONTENT_TYPE, "application/json")],
            Json(json!({"code": self.code, "message": self.message})),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configuration_validator_normalizes_and_returns_stable_findings() {
        let valid = validate_configuration(&ScopedRecordsConfigurationV1 {
            schema_version: 1,
            display_label: "  Regional Records  ".into(),
            retention_mode: "retain_on_undeploy".into(),
        });
        assert_eq!(valid.normalized.unwrap().display_label, "Regional Records");
        let invalid = validate_configuration(&ScopedRecordsConfigurationV1 {
            schema_version: 1,
            display_label: " ".into(),
            retention_mode: "retain_on_undeploy".into(),
        });
        assert_eq!(
            invalid.findings[0].code,
            "configuration.display_label.required"
        );
    }

    #[test]
    fn capability_owner_projection_does_not_cross_bindings() {
        let read = SecurityCapabilityId::new(READ_CAPABILITY).unwrap();
        let manage = SecurityCapabilityId::new(MANAGE_CAPABILITY).unwrap();
        let root_a = Uuid::from_u128(1);
        let root_b = Uuid::from_u128(2);
        let grant = AuthorizationGrantV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(3),
            original_actor_id: Uuid::from_u128(4),
            presenting_service: ModuleDefinitionId::new("tessara.core").unwrap(),
            audience_module_instance_id: Uuid::from_u128(5),
            dependency_binding: DependencyBindingKey::new("tessara.core.scoped-records").unwrap(),
            functional_contract: FunctionalContractId::new(
                "tessara.reference.scoped-records.record",
            )
            .unwrap(),
            action: "records.list".into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings: vec![
                tessara_module_contract::CapabilityScopeBindingV1 {
                    capability: read.clone(),
                    organization_root_id: root_a,
                    authorized_organization_ids: vec![],
                },
                tessara_module_contract::CapabilityScopeBindingV1 {
                    capability: manage.clone(),
                    organization_root_id: root_b,
                    authorized_organization_ids: vec![],
                },
            ],
            resource_assertion: None,
            delegation_basis: vec![],
            authorization_revision: 1,
            organization_revision: 1,
            jti: Uuid::from_u128(6),
            issued_at: Utc::now(),
            expires_at: Utc::now(),
        };
        assert!(grant.authorizes(&read, root_a));
        assert!(grant.authorizes(&manage, root_b));
        assert!(!grant.authorizes(&read, root_b));
        assert!(!grant.authorizes(&manage, root_a));
    }

    #[test]
    fn directory_renders_only_supplied_authorized_records_and_escapes_labels() {
        let record_id = Uuid::from_u128(7);
        let owner_id = Uuid::from_u128(8);
        let organizations = vec![OrganizationAccessProjectionV1 {
            organization_id: owner_id,
            label: "North Region".into(),
            can_manage: true,
        }];
        let html = directory_html(
            &[ScopedRecord {
                id: record_id,
                label: "<North & Central>".into(),
                organization_owner_id: owner_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }],
            &organizations,
            &DirectoryQuery::default(),
        );

        assert!(html.contains(&format!(
            "href=\"/reference/scoped-records/records/{record_id}\""
        )));
        assert!(html.contains("&lt;North &amp; Central&gt;"));
        assert!(html.contains("North Region"));
        assert!(html.contains("Read · Manage"));
        assert!(html.contains("Showing 1-1 of 1 records"));
        assert!(html.contains("class=\"breadcrumb\" aria-label=\"Breadcrumb\""));
        assert!(html.contains("class=\"breadcrumb__separator-icon\""));
        assert!(html.contains("class=\"button__icon\""));
        assert!(html.contains("<path"));
        assert!(!html.contains("<North & Central>"));

        let empty = directory_html(&[], &organizations, &DirectoryQuery::default());
        assert!(empty.contains("No scoped records"));
        assert!(!empty.contains("<tbody>"));
    }

    #[test]
    fn create_and_edit_forms_are_distinct_and_offer_only_manageable_organizations() {
        let managed = OrganizationAccessProjectionV1 {
            organization_id: Uuid::from_u128(9),
            label: "North Region".into(),
            can_manage: true,
        };
        let read_only = OrganizationAccessProjectionV1 {
            organization_id: Uuid::from_u128(10),
            label: "West Region".into(),
            can_manage: false,
        };
        let organizations = vec![managed.clone(), read_only];
        let create = record_form_html(None, &organizations);
        assert!(create.contains("<h1>New Record</h1>"));
        assert!(
            create.contains("<span class=\"breadcrumb__page\" aria-current=\"page\">New Record")
        );
        assert!(create.contains("action=\"/reference/scoped-records/records\""));
        assert!(create.contains("North Region"));
        assert!(!create.contains("West Region"));

        let record_id = Uuid::from_u128(11);
        let edit = record_form_html(
            Some(&ScopedRecord {
                id: record_id,
                label: "North intake review".into(),
                organization_owner_id: managed.organization_id,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }),
            &organizations,
        );
        assert!(edit.contains("<h1>Edit Record</h1>"));
        assert!(edit.contains("<span class=\"breadcrumb__page\" aria-current=\"page\">Edit"));
        assert!(edit.contains(&format!(
            "action=\"/reference/scoped-records/records/{record_id}\""
        )));
        assert!(edit.contains("North intake review"));
        assert!(edit.contains(" selected"));
    }

    #[test]
    fn read_only_directory_does_not_advertise_record_creation() {
        let organizations = vec![OrganizationAccessProjectionV1 {
            organization_id: Uuid::from_u128(12),
            label: "North Region".into(),
            can_manage: false,
        }];
        let html = directory_html(&[], &organizations, &DirectoryQuery::default());

        assert!(html.contains("0 include manage authority"));
        assert!(!html.contains("/reference/scoped-records/records/new"));
        assert!(!html.contains(">New Record<"));
        assert!(!has_manage_authority(&organizations));
    }

    #[test]
    fn lifecycle_and_manage_denial_states_preserve_the_core_shell_contract() {
        let disabled = SecurityState {
            installation_id: Uuid::from_u128(13),
            module_instance_id: Uuid::from_u128(14),
            authorization_revision: 1,
            organization_revision: 1,
            enabled: false,
            document_state: "disabled".into(),
        };
        let disabled_html = product_state_html(&disabled).expect("disabled state treatment");
        assert!(disabled_html.contains("Scoped Records is not serving product routes"));
        assert!(disabled_html.contains("/administration/modules/tessara.reference.scoped-records"));
        assert!(disabled_html.contains("Product data</dt><dd>Retained"));

        let enabled = SecurityState {
            enabled: true,
            document_state: "enabled".into(),
            ..disabled
        };
        assert!(product_state_html(&enabled).is_none());

        let denied_html = manage_denied_html();
        assert!(denied_html.contains("You can’t manage this record"));
        assert!(denied_html.contains("tessara.reference.scoped-records:manage"));
        assert!(denied_html.contains("Back to Records"));
    }
}
