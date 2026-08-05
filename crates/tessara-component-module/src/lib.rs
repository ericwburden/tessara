//! Independently deployed Component Module operational boundary.

use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use axum::{
    Json, Router,
    body::Body,
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool};
use tessara_module_contract::{
    ModuleDefinitionId, ModuleManifest, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
    ShellContextV1, ShellContextValidationContextV1, SignedEnvelopeV1,
};
use uuid::Uuid;

mod dataset_client;
mod documents;
mod product;
mod provider;

pub const MODULE_DEFINITION_ID: &str = "tessara.components";
pub const MODULE_RELEASE_VERSION: &str = "1.0.0";
pub const READ_CAPABILITY: &str = "components:read";
pub const MANAGE_CAPABILITY: &str = "components:manage";

#[derive(Clone)]
pub struct ComponentModuleState {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
    pub dashboard_service_verifier: PurposeBoundVerifyingKeyV1,
    pub service_request_signer: Arc<PurposeBoundSigningKeyV1>,
    pub core_internal_url: String,
    pub dataset_client: reqwest::Client,
    pub(crate) dataset_health: Arc<RwLock<DatasetHealthObservation>>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DatasetHealthObservation {
    status: &'static str,
    failure_code: Option<&'static str>,
    observed_at: Option<DateTime<Utc>>,
}

impl Default for DatasetHealthObservation {
    fn default() -> Self {
        Self {
            status: "not_evaluated",
            failure_code: None,
            observed_at: None,
        }
    }
}

impl ComponentModuleState {
    pub fn new(
        pool: PgPool,
        core_authorization_verifier: PurposeBoundVerifyingKeyV1,
        core_shell_verifier: PurposeBoundVerifyingKeyV1,
        dashboard_service_verifier: PurposeBoundVerifyingKeyV1,
        service_request_signer: Arc<PurposeBoundSigningKeyV1>,
        core_internal_url: String,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self {
            pool,
            core_authorization_verifier,
            core_shell_verifier,
            dashboard_service_verifier,
            service_request_signer,
            core_internal_url: core_internal_url.trim_end_matches('/').to_string(),
            dataset_client: reqwest::Client::builder()
                .timeout(Duration::from_secs(30))
                .build()?,
            dataset_health: Arc::new(RwLock::new(DatasetHealthObservation::default())),
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentConfigurationV1 {
    pub schema_version: u16,
    pub display_label: String,
    pub dataset_request_timeout_seconds: u16,
}

impl Default for ComponentConfigurationV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            display_label: "Components".into(),
            dataset_request_timeout_seconds: 5,
        }
    }
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
    pub normalized: Option<ComponentConfigurationV1>,
    pub findings: Vec<ConfigurationFindingV1>,
}

pub fn validate_configuration(input: &ComponentConfigurationV1) -> ConfigurationValidationV1 {
    let label = input.display_label.trim();
    let mut findings = Vec::new();
    if input.schema_version != 1 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.schema_version.unsupported",
            field: "schema_version",
            message: "Only Component configuration schema v1 is supported.",
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
    if !(1..=30).contains(&input.dataset_request_timeout_seconds) {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.dataset_request_timeout_seconds.out_of_range",
            field: "dataset_request_timeout_seconds",
            message: "Dataset request timeout must be between 1 and 30 seconds.",
        });
    }
    ConfigurationValidationV1 {
        schema_version: 1,
        valid: findings.is_empty(),
        normalized: findings.is_empty().then(|| ComponentConfigurationV1 {
            schema_version: 1,
            display_label: label.to_string(),
            dataset_request_timeout_seconds: input.dataset_request_timeout_seconds,
        }),
        findings,
    }
}

#[derive(Clone, Debug, Deserialize)]
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

#[derive(Clone, Debug, FromRow, Serialize)]
struct SecurityState {
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: String,
    updated_at: DateTime<Utc>,
}

pub fn router(state: ComponentModuleState) -> Router {
    Router::new()
        .route("/api/manifest", get(get_manifest))
        .route(
            "/api/configuration/validate",
            post(validate_configuration_api),
        )
        .route(
            "/api/configuration",
            get(get_configuration).put(put_configuration),
        )
        .route("/api/private/security-state", put(update_security_state))
        .route("/api/private/bootstrap", post(apply_bootstrap))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/api/diagnostics", get(diagnostics))
        .merge(product::routes())
        .merge(provider::routes())
        .merge(documents::routes())
        .route(
            "/_tessara/modules/tessara.components/{release}/{digest}/{asset}",
            get(component_asset),
        )
        .with_state(state)
}

pub(crate) async fn verified_shell_context(
    state: &ComponentModuleState,
    headers: &HeaderMap,
) -> Result<ShellContextV1, ComponentModuleError> {
    let encoded = headers
        .get("x-tessara-shell-context")
        .and_then(|value| value.to_str().ok())
        .ok_or(ComponentModuleError::Forbidden)?;
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(ComponentModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<ShellContextV1> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| ComponentModuleError::Forbidden)?,
    )
    .map_err(|_| ComponentModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| ComponentModuleError::Unavailable("security state unavailable".into()))?;
    state
        .core_shell_verifier
        .verify(&envelope)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    envelope
        .payload
        .validate_for(&ShellContextValidationContextV1 {
            installation_id: security.installation_id,
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                .map_err(|_| ComponentModuleError::Forbidden)?,
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: Utc::now(),
        })
        .map_err(|_| ComponentModuleError::Forbidden)?;
    Ok(envelope.payload)
}

async fn component_asset(
    axum::extract::Path((release, digest, asset)): axum::extract::Path<(String, String, String)>,
) -> Response {
    if release != MODULE_RELEASE_VERSION {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (expected, content_type, bytes): (&str, &str, &'static [u8]) = match asset.as_str() {
        "component.css" => (
            documents::COMPONENT_CSS_SHA256,
            "text/css; charset=utf-8",
            documents::COMPONENT_CSS.as_bytes(),
        ),
        "component-lifecycle.css" => (
            documents::COMPONENT_LIFECYCLE_CSS_SHA256,
            "text/css; charset=utf-8",
            documents::COMPONENT_LIFECYCLE_CSS.as_bytes(),
        ),
        "component.js" => (
            documents::COMPONENT_JS_SHA256,
            "text/javascript; charset=utf-8",
            documents::COMPONENT_JS.as_bytes(),
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    if digest != format!("sha256:{expected}") {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        Body::from(bytes),
    )
        .into_response()
}

pub fn manifest() -> ModuleManifest {
    serde_json::from_str(include_str!("../manifest.json"))
        .expect("Component manifest must remain valid")
}

async fn get_manifest(headers: HeaderMap) -> Result<Json<ModuleManifest>, ComponentModuleError> {
    require_control_key(&headers)?;
    Ok(Json(manifest()))
}

async fn validate_configuration_api(
    Json(input): Json<ComponentConfigurationV1>,
) -> Json<ConfigurationValidationV1> {
    Json(validate_configuration(&input))
}

async fn get_configuration(
    State(state): State<ComponentModuleState>,
) -> Result<Json<ComponentConfigurationV1>, ComponentModuleError> {
    let row = sqlx::query_as::<_, (i32, String, i32)>(
        "SELECT schema_version,display_label,dataset_request_timeout_seconds \
         FROM component_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(ComponentConfigurationV1 {
        schema_version: row.0 as u16,
        display_label: row.1,
        dataset_request_timeout_seconds: row.2 as u16,
    }))
}

async fn put_configuration(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(input): Json<ComponentConfigurationV1>,
) -> Result<Json<ComponentConfigurationV1>, ComponentModuleError> {
    require_control_key(&headers)?;
    let validation = validate_configuration(&input);
    let normalized = validation
        .normalized
        .ok_or(ComponentModuleError::InvalidConfiguration(
            validation.findings,
        ))?;
    sqlx::query(
        "UPDATE component_configuration SET schema_version=$1,display_label=$2,\
         dataset_request_timeout_seconds=$3,updated_at=now() WHERE singleton=true",
    )
    .bind(i32::from(normalized.schema_version))
    .bind(&normalized.display_label)
    .bind(i32::from(normalized.dataset_request_timeout_seconds))
    .execute(&state.pool)
    .await?;
    Ok(Json(normalized))
}

async fn update_security_state(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(input): Json<SecurityStateInput>,
) -> Result<StatusCode, ComponentModuleError> {
    require_control_key(&headers)?;
    if input.schema_version != 1
        || input.authorization_revision == 0
        || input.organization_revision == 0
        || !matches!(
            input.document_state.as_str(),
            "enabled" | "disabled" | "degraded" | "recovery"
        )
    {
        return Err(ComponentModuleError::BadRequest(
            "Component security projection is invalid".into(),
        ));
    }
    let updated = sqlx::query(
        "INSERT INTO component_security_state(singleton,installation_id,module_instance_id,\
         authorization_revision,organization_revision,enabled,document_state) \
         VALUES(true,$1,$2,$3,$4,$5,$6) ON CONFLICT(singleton) DO UPDATE SET \
         authorization_revision=GREATEST(component_security_state.authorization_revision,EXCLUDED.authorization_revision),\
         organization_revision=GREATEST(component_security_state.organization_revision,EXCLUDED.organization_revision),\
         enabled=EXCLUDED.enabled,document_state=EXCLUDED.document_state,updated_at=now() \
         WHERE component_security_state.installation_id=EXCLUDED.installation_id \
           AND component_security_state.module_instance_id=EXCLUDED.module_instance_id",
    )
    .bind(input.installation_id)
    .bind(input.module_instance_id)
    .bind(input.authorization_revision as i64)
    .bind(input.organization_revision as i64)
    .bind(input.enabled)
    .bind(input.document_state)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(ComponentModuleError::Conflict(
            "Component security identity cannot change".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ComponentBootstrapV1 {
    schema_version: String,
    components: Vec<ComponentBootstrapItemV1>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ComponentBootstrapItemV1 {
    external_key: String,
    component_id: Uuid,
    component_version_id: Uuid,
    name: String,
    slug: String,
    component_type: String,
    dataset_reference: tessara_datasets_contract::DatasetMajorLineReference,
    dataset_scope_node_ids: Vec<Uuid>,
    config: Value,
}

async fn apply_bootstrap(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Json(request): Json<tessara_composition::OwnerBootstrapRequestV1<ComponentBootstrapV1>>,
) -> Result<Json<tessara_composition::OwnerBootstrapResponseV1>, ComponentModuleError> {
    require_control_key(&headers)?;
    if request.input.schema_version != "tessara.io/component-bootstrap/v1"
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|error| ComponentModuleError::BadRequest(error.to_string()))?
    {
        return Err(ComponentModuleError::BadRequest(
            "Component bootstrap contract or digest is invalid".into(),
        ));
    }
    if let Some((digest, receipt)) = sqlx::query_as::<_, (String, Value)>(
        "SELECT input_digest,receipt FROM component_bootstrap_receipts WHERE idempotency_key=$1",
    )
    .bind(&request.idempotency_key)
    .fetch_optional(&state.pool)
    .await?
    {
        if digest != request.input_digest.to_string() {
            return Err(ComponentModuleError::Conflict(
                "Component bootstrap idempotency key was reused with different input".into(),
            ));
        }
        let mut response: tessara_composition::OwnerBootstrapResponseV1 =
            serde_json::from_value(receipt)
                .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
        response.receipt.changed = false;
        return Ok(Json(response));
    }
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    if security.installation_id != request.installation_id
        || request.input.components.is_empty()
        || request.input.components.iter().any(|component| {
            component.external_key.trim().is_empty()
                || component.name.trim().is_empty()
                || component.slug.trim().is_empty()
                || component.dataset_scope_node_ids.is_empty()
                || component.dataset_reference.reference().installation_id()
                    != request.installation_id
                || !component.config.is_object()
                || !matches!(
                    component.component_type.as_str(),
                    "table" | "bar" | "line" | "pie" | "donut" | "stat_card"
                )
        })
    {
        return Err(ComponentModuleError::BadRequest(
            "Component bootstrap input is invalid".into(),
        ));
    }
    let mut transaction = state.pool.begin().await?;
    let mut resource_ids = std::collections::BTreeMap::new();
    for component in &request.input.components {
        let mut scope = component.dataset_scope_node_ids.clone();
        scope.sort_unstable();
        scope.dedup();
        sqlx::query("INSERT INTO components(id,external_key,name,slug,description) VALUES($1,$2,$3,$4,$5) ON CONFLICT(external_key) DO UPDATE SET name=EXCLUDED.name,slug=EXCLUDED.slug,description=EXCLUDED.description,updated_at=now()")
            .bind(component.component_id)
            .bind(&component.external_key)
            .bind(component.name.trim())
            .bind(component.slug.trim())
            .bind("Reference application seed")
            .execute(&mut *transaction)
            .await?;
        sqlx::query("INSERT INTO component_versions(id,component_id,dataset_reference,dataset_scope_node_ids,component_type,status,lifecycle_state,version_number,version_label,version_note,config) VALUES($1,$2,$3,$4,$5::component_type,'published','active',1,'1.0.0','Reference application seed',$6) ON CONFLICT(id) DO UPDATE SET dataset_reference=EXCLUDED.dataset_reference,dataset_scope_node_ids=EXCLUDED.dataset_scope_node_ids,component_type=EXCLUDED.component_type,config=EXCLUDED.config,updated_at=now()")
            .bind(component.component_version_id)
            .bind(component.component_id)
            .bind(serde_json::to_value(&component.dataset_reference).map_err(|error| ComponentModuleError::Internal(error.to_string()))?)
            .bind(scope)
            .bind(&component.component_type)
            .bind(&component.config)
            .execute(&mut *transaction)
            .await?;
        resource_ids.insert(
            component.external_key.clone(),
            serde_json::to_string(&component_reference(
                &security,
                component.component_version_id,
            )?)
            .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
        );
    }
    let result_digest = tessara_composition::canonical_digest(&resource_ids)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let response = tessara_composition::OwnerBootstrapResponseV1 {
        receipt: tessara_composition::BootstrapReceiptV1 {
            owner: MODULE_DEFINITION_ID.into(),
            schema_version: request.input.schema_version.clone(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids,
        },
    };
    sqlx::query("INSERT INTO component_bootstrap_receipts(idempotency_key,input_digest,desired_revision,receipt) VALUES($1,$2,$3,$4)")
        .bind(&request.idempotency_key)
        .bind(request.input_digest.to_string())
        .bind(request.desired_revision as i64)
        .bind(serde_json::to_value(&response).map_err(|error| ComponentModuleError::Internal(error.to_string()))?)
        .execute(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

fn component_reference(
    security: &SecurityState,
    version_id: Uuid,
) -> Result<tessara_components_contract::ComponentVersionReference, ComponentModuleError> {
    use tessara_module_contract::{ResourceOwner, TypedResourceReference};

    tessara_components_contract::ComponentVersionReference::new(
        TypedResourceReference::new(
            security.installation_id,
            ResourceOwner::ModuleInstance {
                installation_id: security.installation_id,
                module_instance_id: security.module_instance_id,
            },
            tessara_components_contract::COMPONENT_RESOURCE_TYPE
                .parse()
                .map_err(|error| ComponentModuleError::Internal(format!("{error}")))?,
            version_id.to_string(),
        )
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
    )
    .map_err(|error| ComponentModuleError::Internal(error.to_string()))
}

async fn live() -> Json<Value> {
    Json(json!({"schema_version":1,"status":"live"}))
}

async fn ready(
    State(state): State<ComponentModuleState>,
) -> Result<(StatusCode, Json<Value>), ComponentModuleError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await?;
    let security = load_security_state(&state.pool).await?;
    let ready = security
        .as_ref()
        .is_some_and(|value| value.enabled && value.document_state == "enabled");
    Ok((
        if ready {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(json!({
            "schema_version": 1,
            "status": if ready { "ready" } else { "not_ready" },
            "checks": {
                "database": "ready",
                "security_projection": if security.is_some() { "available" } else { "missing" },
                "enabled": security.as_ref().is_some_and(|value| value.enabled),
                "document_state": security.as_ref().map(|value| value.document_state.as_str()).unwrap_or("not_evaluated")
            }
        })),
    ))
}

async fn diagnostics(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ComponentModuleError> {
    require_control_key(&headers)?;
    let configuration = get_configuration(State(state.clone())).await?.0;
    let security = load_security_state(&state.pool).await?;
    let dataset_health = state
        .dataset_health
        .read()
        .map_err(|_| {
            ComponentModuleError::Internal("Dataset health observation lock is poisoned".into())
        })?
        .clone();
    Ok(Json(json!({
        "schema_version": 1,
        "module": MODULE_DEFINITION_ID,
        "release": MODULE_RELEASE_VERSION,
        "manifest_schema": 3,
        "contracts": {
            "components": tessara_components_contract::COMPONENT_CONTRACT_VERSION,
            "dataset_dependency": tessara_datasets_contract::DATASET_CONTRACT_VERSION
        },
        "configuration": configuration,
        "database": {"status":"connected","binding":"component_module_instance"},
        "authorization": security,
        "dataset_dependency": {
            "binding_key": tessara_datasets_contract::DATASET_BINDING_KEY,
            "contract_id": tessara_datasets_contract::DATASET_CONTRACT_ID,
            "provider_owner": "core_installation",
            "last_health": dataset_health
        },
        "probes": {"liveness":"live","readiness":"see_health_ready"},
        "failures": []
    })))
}

async fn load_security_state(pool: &PgPool) -> Result<Option<SecurityState>, ComponentModuleError> {
    Ok(sqlx::query_as::<_, SecurityState>(
        "SELECT installation_id,module_instance_id,authorization_revision,\
         organization_revision,enabled,document_state,updated_at \
         FROM component_security_state WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await?)
}

fn require_control_key(headers: &HeaderMap) -> Result<(), ComponentModuleError> {
    let expected = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let actual = headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok());
    if actual != Some(expected.as_str()) {
        return Err(ComponentModuleError::Forbidden);
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum ComponentModuleError {
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    Internal(String),
    #[error("invalid Component configuration")]
    InvalidConfiguration(Vec<ConfigurationFindingV1>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for ComponentModuleError {
    fn into_response(self) -> Response {
        let (status, code, message, findings) = match self {
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "component.forbidden",
                "Forbidden".to_string(),
                None,
            ),
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                "component.bad_request",
                message,
                None,
            ),
            Self::Conflict(message) => (StatusCode::CONFLICT, "component.conflict", message, None),
            Self::NotFound(_) => (
                StatusCode::NOT_FOUND,
                "component.not_found",
                "Component resource was not found".to_string(),
                None,
            ),
            Self::Unavailable(message) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "component.dependency_unavailable",
                message,
                None,
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "component.internal",
                "Component request failed".to_string(),
                None,
            ),
            Self::InvalidConfiguration(findings) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "component.configuration_invalid",
                "Component configuration is invalid".to_string(),
                Some(findings),
            ),
            Self::Database(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "component.storage_unavailable",
                "Component storage is unavailable".to_string(),
                None,
            ),
        };
        (
            status,
            Json(json!({
                "schema_version": 1,
                "code": code,
                "message": message,
                "retryable": status == StatusCode::SERVICE_UNAVAILABLE,
                "findings": findings
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    const BASELINE: &[u8] = include_bytes!("../migrations/001_component_module.sql");

    #[test]
    fn exact_configuration_defaults_and_boundaries() {
        let default = ComponentConfigurationV1::default();
        let valid = validate_configuration(&default);
        assert!(valid.valid);
        assert_eq!(valid.normalized.unwrap(), default);

        for timeout in [0, 31] {
            let invalid = validate_configuration(&ComponentConfigurationV1 {
                dataset_request_timeout_seconds: timeout,
                ..ComponentConfigurationV1::default()
            });
            assert!(!invalid.valid);
            assert_eq!(
                invalid.findings[0].code,
                "configuration.dataset_request_timeout_seconds.out_of_range"
            );
        }

        let trimmed = validate_configuration(&ComponentConfigurationV1 {
            display_label: "  Analysis Components  ".into(),
            ..ComponentConfigurationV1::default()
        });
        assert_eq!(
            trimmed.normalized.unwrap().display_label,
            "Analysis Components"
        );
    }

    #[test]
    fn configuration_rejects_blank_long_and_wrong_schema() {
        for input in [
            ComponentConfigurationV1 {
                display_label: "  ".into(),
                ..ComponentConfigurationV1::default()
            },
            ComponentConfigurationV1 {
                display_label: "x".repeat(81),
                ..ComponentConfigurationV1::default()
            },
            ComponentConfigurationV1 {
                schema_version: 2,
                ..ComponentConfigurationV1::default()
            },
        ] {
            assert!(!validate_configuration(&input).valid);
        }
    }

    #[test]
    fn manifest_declares_exact_release_contract_and_configuration() {
        let manifest = manifest();
        assert_eq!(manifest.definition_id.as_str(), MODULE_DEFINITION_ID);
        assert_eq!(manifest.release_version.to_string(), MODULE_RELEASE_VERSION);
        let wire = serde_json::to_value(&manifest).unwrap();
        assert_eq!(
            wire["provided_contracts"][0]["version"],
            tessara_components_contract::COMPONENT_CONTRACT_VERSION
        );
        assert_eq!(
            wire["configuration_schema"]["required"],
            json!(["display_label", "dataset_request_timeout_seconds"])
        );
        let authority = tessara_module_contract::ManifestNamespaceAuthority::new(
            tessara_module_contract::ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
            tessara_module_contract::PublisherId::new("tessara.first_party").unwrap(),
            ["tessara.components", "components"],
        )
        .unwrap();
        manifest.validate(&authority).unwrap();
    }

    #[test]
    fn fresh_baseline_is_component_owned_and_source_exact() {
        assert_eq!(
            format!("{:x}", Sha256::digest(BASELINE)),
            "1b6174fa52d648d924fe7fa58423dc5a35b897d88a2d6b9992a291749ce96238"
        );
        let baseline = std::str::from_utf8(BASELINE).expect("baseline migration is UTF-8");
        for required in [
            "CREATE TABLE components",
            "CREATE TABLE component_versions",
            "CREATE TABLE component_version_change_events",
            "CREATE TABLE component_mutation_replays",
            "CREATE TABLE component_bootstrap_receipts",
        ] {
            assert!(baseline.contains(required), "missing {required}");
        }
        for forbidden in [
            "CREATE TABLE datasets",
            "CREATE TABLE dashboards",
            "REFERENCES datasets",
            "REFERENCES dashboards",
        ] {
            assert!(!baseline.contains(forbidden), "forbidden {forbidden}");
        }
    }
}
