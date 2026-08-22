//! Independently deployed Dataset module operational and persistence boundary.

use std::{collections::BTreeMap, sync::Arc, time::Instant};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    middleware::{Next, from_fn},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::{FromRow, PgPool};
use tessara_module_contract::{
    ModuleDefinitionId, ModuleServiceIdentityRegistryV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, ShellContextV2, ShellContextValidationContextV2, SignedEnvelopeV1,
};
use uuid::Uuid;

mod authoring;
mod bootstrap;
mod compatibility;
pub mod dependency_dag;
mod documents;
mod materialization;
mod product;
mod provider;
mod provider_client;
mod refresh;
mod status;
pub mod sync;
mod validation_fault;

pub use validation_fault::DatasetValidationFaultControl;

pub const MODULE_DEFINITION_ID: &str = "tessara.datasets";
pub const CURRENT_MODULE_RELEASE_VERSION: &str = "1.0.0";
pub const PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION: &str = "0.9.0";
#[cfg(feature = "sprint-8b-upgrade-baseline")]
pub const MODULE_RELEASE_VERSION: &str = PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION;
#[cfg(not(feature = "sprint-8b-upgrade-baseline"))]
pub const MODULE_RELEASE_VERSION: &str = CURRENT_MODULE_RELEASE_VERSION;
pub const RUNTIME_IDENTITY: &str = "datasets-runtime";
pub const MIGRATION_IDENTITY: &str = "datasets-migration";
pub const READ_CAPABILITY: &str = "datasets:read";
pub const MANAGE_CAPABILITY: &str = "datasets:manage";
pub const LIVENESS_PATH: &str = "/health/live";
pub const READINESS_PATH: &str = "/health/ready";
pub const DATASET_PROVIDER_ENDPOINTS_ENVIRONMENT: &str = "TESSARA_DATASET_PROVIDER_ENDPOINTS";
pub const REQUIRED_DATASET_PROVIDER_BINDINGS: [&str; 4] = [
    tessara_responses_contract::RESPONSE_EXPORT_BINDING_KEY,
    tessara_forms_contract::FORM_VERSION_SCHEMA_BINDING_KEY,
    tessara_control_plane_contract::SCOPE_CATALOG_BINDING_KEY,
    tessara_control_plane_contract::PRINCIPAL_DISPLAY_BINDING_KEY,
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DatasetServiceEndpoints {
    core_authorization_url: String,
    provider_urls: BTreeMap<String, String>,
}

impl DatasetServiceEndpoints {
    pub fn from_json(
        core_authorization_url: &str,
        configured_provider_endpoints: &str,
    ) -> Result<Self, DatasetServiceEndpointError> {
        let provider_urls: BTreeMap<String, String> =
            serde_json::from_str(configured_provider_endpoints)
                .map_err(DatasetServiceEndpointError::InvalidProviderMap)?;
        Self::new(core_authorization_url, provider_urls)
    }

    pub fn new(
        core_authorization_url: &str,
        provider_urls: BTreeMap<String, String>,
    ) -> Result<Self, DatasetServiceEndpointError> {
        for binding in REQUIRED_DATASET_PROVIDER_BINDINGS {
            if !provider_urls.contains_key(binding) {
                return Err(DatasetServiceEndpointError::MissingBinding(
                    binding.to_string(),
                ));
            }
        }
        if let Some(binding) = provider_urls
            .keys()
            .find(|binding| !REQUIRED_DATASET_PROVIDER_BINDINGS.contains(&binding.as_str()))
        {
            return Err(DatasetServiceEndpointError::UnknownBinding(binding.clone()));
        }
        let provider_urls = provider_urls
            .into_iter()
            .map(|(binding, endpoint)| {
                normalize_service_endpoint(&binding, &endpoint).map(|endpoint| (binding, endpoint))
            })
            .collect::<Result<_, _>>()?;
        Ok(Self {
            core_authorization_url: normalize_service_endpoint(
                "TESSARA_CORE_INTERNAL_URL",
                core_authorization_url,
            )?,
            provider_urls,
        })
    }

    pub fn core_authorization_url(&self) -> &str {
        &self.core_authorization_url
    }

    pub fn provider_url(&self, binding: &str) -> Option<&str> {
        self.provider_urls.get(binding).map(String::as_str)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DatasetServiceEndpointError {
    #[error("TESSARA_DATASET_PROVIDER_ENDPOINTS must be a binding-keyed JSON object: {0}")]
    InvalidProviderMap(serde_json::Error),
    #[error("TESSARA_DATASET_PROVIDER_ENDPOINTS is missing required binding '{0}'")]
    MissingBinding(String),
    #[error("TESSARA_DATASET_PROVIDER_ENDPOINTS contains unknown binding '{0}'")]
    UnknownBinding(String),
    #[error("service endpoint '{name}' is invalid: {reason}")]
    InvalidEndpoint { name: String, reason: &'static str },
}

fn normalize_service_endpoint(
    name: &str,
    endpoint: &str,
) -> Result<String, DatasetServiceEndpointError> {
    let endpoint = endpoint.trim();
    let parsed = reqwest::Url::parse(endpoint).map_err(|_| {
        DatasetServiceEndpointError::InvalidEndpoint {
            name: name.to_string(),
            reason: "an absolute HTTP or HTTPS URL is required",
        }
    })?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        return Err(DatasetServiceEndpointError::InvalidEndpoint {
            name: name.to_string(),
            reason: "an absolute HTTP or HTTPS URL is required",
        });
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(DatasetServiceEndpointError::InvalidEndpoint {
            name: name.to_string(),
            reason: "embedded credentials are forbidden",
        });
    }
    if parsed.path() != "/" || parsed.query().is_some() || parsed.fragment().is_some() {
        return Err(DatasetServiceEndpointError::InvalidEndpoint {
            name: name.to_string(),
            reason: "the endpoint must be an origin without a path, query, or fragment",
        });
    }
    Ok(parsed.as_str().trim_end_matches('/').to_string())
}

#[derive(Clone)]
pub struct DatasetCoreVerifiers {
    pub authorization: PurposeBoundVerifyingKeyV1,
    pub owner_bootstrap: PurposeBoundVerifyingKeyV1,
    pub service_request: PurposeBoundVerifyingKeyV1,
    pub bootstrap_validation: PurposeBoundVerifyingKeyV1,
    pub shell: PurposeBoundVerifyingKeyV1,
}

#[derive(Clone)]
pub struct DatasetModuleState {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_owner_bootstrap_verifier: PurposeBoundVerifyingKeyV1,
    pub core_service_request_verifier: PurposeBoundVerifyingKeyV1,
    pub core_bootstrap_validation_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
    pub service_identity_registry: ModuleServiceIdentityRegistryV1,
    pub service_request_signer: Arc<PurposeBoundSigningKeyV1>,
    pub bootstrap_receipt_signer: Arc<PurposeBoundSigningKeyV1>,
    pub service_endpoints: DatasetServiceEndpoints,
    pub provider_client: reqwest::Client,
    pub validation_fault_control: DatasetValidationFaultControl,
}

impl DatasetModuleState {
    pub fn new(
        pool: PgPool,
        core_verifiers: DatasetCoreVerifiers,
        service_identity_registry: ModuleServiceIdentityRegistryV1,
        service_request_signer: Arc<PurposeBoundSigningKeyV1>,
        bootstrap_receipt_signer: Arc<PurposeBoundSigningKeyV1>,
        service_endpoints: DatasetServiceEndpoints,
        validation_fault_control: DatasetValidationFaultControl,
    ) -> Result<Self, reqwest::Error> {
        Ok(Self {
            pool,
            core_authorization_verifier: core_verifiers.authorization,
            core_owner_bootstrap_verifier: core_verifiers.owner_bootstrap,
            core_service_request_verifier: core_verifiers.service_request,
            core_bootstrap_validation_verifier: core_verifiers.bootstrap_validation,
            core_shell_verifier: core_verifiers.shell,
            service_identity_registry,
            service_request_signer,
            bootstrap_receipt_signer,
            service_endpoints,
            provider_client: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()?,
            validation_fault_control,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DatasetConfigurationV1 {
    pub schema_version: u16,
    pub display_label: String,
    pub provider_request_timeout_seconds: u16,
    pub provider_retry_limit: u16,
    pub response_export_page_size: u16,
}

impl Default for DatasetConfigurationV1 {
    fn default() -> Self {
        Self {
            schema_version: 1,
            display_label: "Datasets".into(),
            provider_request_timeout_seconds: 5,
            provider_retry_limit: 1,
            response_export_page_size: 250,
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
    pub normalized: Option<DatasetConfigurationV1>,
    pub findings: Vec<ConfigurationFindingV1>,
}

pub fn validate_configuration(input: &DatasetConfigurationV1) -> ConfigurationValidationV1 {
    let label = input.display_label.trim();
    let mut findings = Vec::new();
    if input.schema_version != 1 {
        findings.push(ConfigurationFindingV1 {
            code: "dataset.configuration.schema_version.unsupported",
            field: "schema_version",
            message: "Only Dataset configuration schema v1 is supported.",
        });
    }
    if label.is_empty() {
        findings.push(ConfigurationFindingV1 {
            code: "dataset.configuration.display_label.required",
            field: "display_label",
            message: "Display label is required.",
        });
    } else if label.chars().count() > 80 {
        findings.push(ConfigurationFindingV1 {
            code: "dataset.configuration.display_label.too_long",
            field: "display_label",
            message: "Display label must contain at most 80 characters.",
        });
    }
    if !(1..=30).contains(&input.provider_request_timeout_seconds) {
        findings.push(ConfigurationFindingV1 {
            code: "dataset.configuration.provider_request_timeout_seconds.out_of_range",
            field: "provider_request_timeout_seconds",
            message: "Provider request timeout must be between 1 and 30 seconds.",
        });
    }
    if input.provider_retry_limit > 3 {
        findings.push(ConfigurationFindingV1 {
            code: "dataset.configuration.provider_retry_limit.out_of_range",
            field: "provider_retry_limit",
            message: "Provider retry limit must be between 0 and 3.",
        });
    }
    if !(1..=1000).contains(&input.response_export_page_size) {
        findings.push(ConfigurationFindingV1 {
            code: "dataset.configuration.response_export_page_size.out_of_range",
            field: "response_export_page_size",
            message: "Response export page size must be between 1 and 1000.",
        });
    }
    ConfigurationValidationV1 {
        schema_version: 1,
        valid: findings.is_empty(),
        normalized: findings.is_empty().then(|| DatasetConfigurationV1 {
            schema_version: 1,
            display_label: label.to_owned(),
            provider_request_timeout_seconds: input.provider_request_timeout_seconds,
            provider_retry_limit: input.provider_retry_limit,
            response_export_page_size: input.response_export_page_size,
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
pub(crate) struct SecurityState {
    pub(crate) installation_id: Uuid,
    pub(crate) module_instance_id: Uuid,
    pub(crate) authorization_revision: i64,
    pub(crate) organization_revision: i64,
    pub(crate) enabled: bool,
    pub(crate) document_state: String,
    pub(crate) updated_at: DateTime<Utc>,
}

pub fn router(state: DatasetModuleState) -> Router {
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
        .route("/api/private/bootstrap", post(bootstrap::apply_bootstrap))
        .route("/api/diagnostics", get(diagnostics))
        .route(
            "/_tessara/modules/tessara.datasets/{release}/{digest}/{asset}",
            get(dataset_asset),
        )
        .route(LIVENESS_PATH, get(live))
        .route(READINESS_PATH, get(ready))
        .merge(product::routes())
        .merge(documents::routes())
        .merge(provider::routes())
        .layer(from_fn(bind_error_correlation))
        .with_state(state)
}

async fn bind_error_correlation(request: Request, next: Next) -> Response {
    let correlation_id = request
        .headers()
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(Uuid::new_v4);
    let response = next.run(request).await;
    if !response.status().is_client_error() && !response.status().is_server_error() {
        return response;
    }
    let (parts, body) = response.into_parts();
    let Ok(bytes) = to_bytes(body, 1024 * 1024).await else {
        return Response::from_parts(parts, Body::empty());
    };
    let Ok(mut value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return Response::from_parts(parts, Body::from(bytes));
    };
    if value.get("schema_version").is_some() && value.get("error").is_some() {
        value["correlation_id"] = json!(correlation_id);
        if let Ok(encoded) = serde_json::to_vec(&value) {
            return Response::from_parts(parts, Body::from(encoded));
        }
    }
    Response::from_parts(parts, Body::from(bytes))
}

pub(crate) async fn verified_shell_context(
    state: &DatasetModuleState,
    headers: &HeaderMap,
) -> Result<ShellContextV2, DatasetModuleError> {
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};

    let encoded = headers
        .get("x-tessara-shell-context")
        .and_then(|value| value.to_str().ok())
        .ok_or(DatasetModuleError::Forbidden)?;
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(DatasetModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<ShellContextV2> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| DatasetModuleError::Forbidden)?,
    )
    .map_err(|_| DatasetModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DatasetModuleError::Unavailable("security state unavailable".into()))?;
    state
        .core_shell_verifier
        .verify(&envelope)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    envelope
        .payload
        .validate_for(&ShellContextValidationContextV2 {
            installation_id: security.installation_id,
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                .map_err(|_| DatasetModuleError::Forbidden)?,
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    Ok(envelope.payload)
}

pub fn manifest() -> tessara_module_contract::ModuleManifest {
    let mut manifest: tessara_module_contract::ModuleManifest =
        serde_json::from_str(include_str!("../manifest.json"))
            .expect("Dataset manifest must remain valid");
    manifest.release_version = MODULE_RELEASE_VERSION
        .parse()
        .expect("Dataset release version must remain valid");
    manifest
}

async fn get_manifest(
    headers: HeaderMap,
) -> Result<Json<tessara_module_contract::ModuleManifest>, DatasetModuleError> {
    require_control_key(&headers)?;
    Ok(Json(manifest()))
}

async fn validate_configuration_api(
    State(state): State<DatasetModuleState>,
    request: Request,
) -> Result<Json<ConfigurationValidationV1>, DatasetModuleError> {
    let started = Instant::now();
    let headers = request.headers().clone();
    require_json_content_type(request.headers())?;
    let input: DatasetConfigurationV1 =
        decode_bounded_json(request, "Dataset configuration").await?;
    let validation = validate_configuration(&input);
    emit_owner_operation(
        &headers,
        observed_module_instance_id(&state.pool).await,
        "datasets.configuration.validate",
        "dataset.configuration",
        if validation.valid {
            "success"
        } else {
            "validation_failed"
        },
        started,
    );
    Ok(Json(validation))
}

async fn get_configuration(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Json<DatasetConfigurationV1>, DatasetModuleError> {
    let started = Instant::now();
    let row = sqlx::query_as::<_, (i32, String, i32, i32, i32)>(
        "SELECT schema_version,display_label,provider_request_timeout_seconds,provider_retry_limit,\
         response_export_page_size \
         FROM dataset_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let configuration = DatasetConfigurationV1 {
        schema_version: row.0 as u16,
        display_label: row.1,
        provider_request_timeout_seconds: row.2 as u16,
        provider_retry_limit: row.3 as u16,
        response_export_page_size: row.4 as u16,
    };
    emit_owner_operation(
        &headers,
        observed_module_instance_id(&state.pool).await,
        "datasets.configuration.get",
        "dataset.database",
        "success",
        started,
    );
    Ok(Json(configuration))
}

async fn put_configuration(
    State(state): State<DatasetModuleState>,
    request: Request,
) -> Result<Json<ConfigurationValidationV1>, DatasetModuleError> {
    let started = Instant::now();
    let headers = request.headers().clone();
    require_control_key(request.headers())?;
    require_json_content_type(request.headers())?;
    let input: DatasetConfigurationV1 =
        decode_bounded_json(request, "Dataset configuration").await?;
    let validation = validate_configuration(&input);
    let Some(normalized) = &validation.normalized else {
        emit_owner_operation(
            &headers,
            observed_module_instance_id(&state.pool).await,
            "datasets.configuration.save",
            "dataset.database",
            "validation_failed",
            started,
        );
        return Err(DatasetModuleError::InvalidConfiguration(
            validation.findings,
        ));
    };
    sqlx::query(
        "UPDATE dataset_configuration SET schema_version=$1,display_label=$2,\
         provider_request_timeout_seconds=$3,provider_retry_limit=$4,\
         response_export_page_size=$5,updated_at=now() WHERE singleton=true",
    )
    .bind(i32::from(normalized.schema_version))
    .bind(&normalized.display_label)
    .bind(i32::from(normalized.provider_request_timeout_seconds))
    .bind(i32::from(normalized.provider_retry_limit))
    .bind(i32::from(normalized.response_export_page_size))
    .execute(&state.pool)
    .await?;
    emit_owner_operation(
        &headers,
        observed_module_instance_id(&state.pool).await,
        "datasets.configuration.save",
        "dataset.database",
        "success",
        started,
    );
    Ok(Json(validation))
}

async fn update_security_state(
    State(state): State<DatasetModuleState>,
    request: Request,
) -> Result<StatusCode, DatasetModuleError> {
    require_control_key(request.headers())?;
    require_json_content_type(request.headers())?;
    let input: SecurityStateInput = decode_bounded_json(request, "Dataset security state").await?;
    if input.schema_version != 1
        || input.authorization_revision == 0
        || input.organization_revision == 0
        || !matches!(
            input.document_state.as_str(),
            "enabled" | "disabled" | "degraded" | "recovery"
        )
    {
        return Err(DatasetModuleError::BadRequest(
            "Dataset security projection is invalid".into(),
        ));
    }
    let updated = sqlx::query(
        "INSERT INTO dataset_security_state(singleton,installation_id,module_instance_id,\
         authorization_revision,organization_revision,enabled,document_state) \
         VALUES(true,$1,$2,$3,$4,$5,$6) ON CONFLICT(singleton) DO UPDATE SET \
         authorization_revision=GREATEST(dataset_security_state.authorization_revision,EXCLUDED.authorization_revision),\
         organization_revision=GREATEST(dataset_security_state.organization_revision,EXCLUDED.organization_revision),\
         enabled=EXCLUDED.enabled,document_state=EXCLUDED.document_state,updated_at=now() \
         WHERE dataset_security_state.installation_id=EXCLUDED.installation_id \
           AND dataset_security_state.module_instance_id=EXCLUDED.module_instance_id",
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
        return Err(DatasetModuleError::Conflict(
            "Dataset security identity cannot change".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn dataset_asset(
    axum::extract::Path((release, digest, asset)): axum::extract::Path<(String, String, String)>,
) -> Response {
    if release != MODULE_RELEASE_VERSION {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (expected, content_type, bytes): (&str, &str, &'static [u8]) = match asset.as_str() {
        "module-ui.css" => (
            tessara_module_ui::MODULE_UI_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_module_ui::MODULE_UI_CSS.as_bytes(),
        ),
        "dataset.css" => (
            tessara_dataset_ui::DATASET_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_dataset_ui::DATASET_CSS.as_bytes(),
        ),
        "dataset-lifecycle.css" => (
            tessara_dataset_ui::DATASET_LIFECYCLE_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_dataset_ui::DATASET_LIFECYCLE_CSS.as_bytes(),
        ),
        "dataset.js" => (
            tessara_dataset_ui::DATASET_JS_SHA256,
            "text/javascript; charset=utf-8",
            tessara_dataset_ui::DATASET_JS.as_bytes(),
        ),
        "dataset-bindings.js" => (
            tessara_dataset_ui::DATASET_BINDINGS_JS_SHA256,
            "text/javascript; charset=utf-8",
            include_bytes!("../../tessara-web-datasets/assets/dataset-bindings.js"),
        ),
        "dataset.wasm" => (
            tessara_dataset_ui::DATASET_WASM_SHA256,
            "application/wasm",
            include_bytes!("../../tessara-web-datasets/assets/dataset.wasm"),
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    if digest != format!("sha256:{expected}") {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        StatusCode::OK,
        [
            (axum::http::header::CONTENT_TYPE, content_type),
            (
                axum::http::header::CACHE_CONTROL,
                "public, max-age=31536000, immutable",
            ),
        ],
        axum::body::Body::from(bytes),
    )
        .into_response()
}

async fn diagnostics(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> Result<Json<status::DiagnosticsResponse>, DatasetModuleError> {
    let started = Instant::now();
    require_control_key(&headers)?;
    let snapshot = status::operational_snapshot(&state).await;
    emit_owner_operation(
        &headers,
        snapshot.module_instance_id(),
        "datasets.diagnostics",
        "dataset.database",
        snapshot.readiness().status,
        started,
    );
    Ok(Json(snapshot.diagnostics()))
}

async fn live() -> Json<serde_json::Value> {
    Json(json!({
        "schema_version": 1,
        "module_definition_id": MODULE_DEFINITION_ID,
        "module_release_version": MODULE_RELEASE_VERSION,
        "status": "live",
        "database": "not_checked",
        "security_state": "not_checked"
    }))
}

async fn ready(
    State(state): State<DatasetModuleState>,
    headers: HeaderMap,
) -> (StatusCode, Json<status::ReadinessResponse>) {
    let started = Instant::now();
    let snapshot = status::operational_snapshot(&state).await;
    let status_code = snapshot.status_code();
    let response = snapshot.readiness();
    emit_owner_operation(
        &headers,
        snapshot.module_instance_id(),
        "datasets.status",
        "dataset.database",
        response.status,
        started,
    );
    (status_code, Json(response))
}

async fn observed_module_instance_id(pool: &PgPool) -> Option<Uuid> {
    load_security_state(pool)
        .await
        .ok()
        .flatten()
        .map(|value| value.module_instance_id)
}

fn emit_owner_operation(
    headers: &HeaderMap,
    module_instance_id: Option<Uuid>,
    operation: &'static str,
    dependency: &'static str,
    result_code: &str,
    started: Instant,
) {
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .unwrap_or_else(Uuid::new_v4);
    let module_instance_id = module_instance_id
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unavailable".into());
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        target: "tessara_dataset_module::operations",
        event = "dataset.operation",
        correlation_id = %correlation_id,
        module_instance_id = %module_instance_id,
        operation,
        dependency,
        attempt = 1_u16,
        duration_ms,
        result_code,
        retry_classification = "not_retryable",
        "Dataset module operation completed"
    );
}

pub(crate) async fn load_security_state(
    pool: &PgPool,
) -> Result<Option<SecurityState>, DatasetModuleError> {
    Ok(sqlx::query_as::<_, SecurityState>(
        "SELECT installation_id,module_instance_id,authorization_revision,\
         organization_revision,enabled,document_state,updated_at \
         FROM dataset_security_state WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await?)
}

pub(crate) fn require_control_key(headers: &HeaderMap) -> Result<(), DatasetModuleError> {
    let expected = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let actual = headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok());
    if actual != Some(expected.as_str()) {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(())
}

pub(crate) fn require_json_content_type(headers: &HeaderMap) -> Result<(), DatasetModuleError> {
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

pub(crate) async fn decode_bounded_json<T: serde::de::DeserializeOwned>(
    request: Request,
    contract: &str,
) -> Result<T, DatasetModuleError> {
    let bytes = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| DatasetModuleError::BadRequest(format!("{contract} payload is too large")))?;
    serde_json::from_slice(&bytes)
        .map_err(|_| DatasetModuleError::BadRequest(format!("{contract} payload is invalid")))
}

#[derive(Debug, thiserror::Error)]
pub enum DatasetModuleError {
    #[error("forbidden")]
    Forbidden,
    #[error("{0}")]
    BadRequest(String),
    #[error("{0}")]
    ValidationFailed(String),
    #[error("{0}")]
    Conflict(String),
    #[error("idempotency key does not match its original request")]
    IdempotencyMismatch,
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Unavailable(String),
    #[error("{0}")]
    DependencyIncompatible(String),
    #[error("{0}")]
    Internal(String),
    #[error("invalid Dataset configuration")]
    InvalidConfiguration(Vec<ConfigurationFindingV1>),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for DatasetModuleError {
    fn into_response(self) -> Response {
        let (status, code, message) = match self {
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "dataset.not_found_or_forbidden",
                "Forbidden".to_string(),
            ),
            Self::BadRequest(message) => (
                StatusCode::BAD_REQUEST,
                "dataset.malformed_request",
                message,
            ),
            Self::ValidationFailed(message) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "dataset.validation_failed",
                message,
            ),
            Self::Conflict(message) => (StatusCode::CONFLICT, "dataset.conflict", message),
            Self::IdempotencyMismatch => (
                StatusCode::CONFLICT,
                "dataset.idempotency_mismatch",
                "Idempotency key does not match its original request".to_string(),
            ),
            Self::NotFound(_) => (
                StatusCode::NOT_FOUND,
                "dataset.not_found_or_forbidden",
                "Dataset resource was not found".to_string(),
            ),
            Self::Unavailable(message) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "dataset.dependency_unavailable",
                message,
            ),
            Self::DependencyIncompatible(message) => (
                StatusCode::BAD_GATEWAY,
                "dataset.dependency_incompatible",
                message,
            ),
            Self::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "dataset.dependency_unavailable",
                "Dataset request failed".to_string(),
            ),
            Self::InvalidConfiguration(_) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "dataset.validation_failed",
                "Dataset configuration is invalid".to_string(),
            ),
            Self::Database(_) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "dataset.dependency_unavailable",
                "Dataset storage is unavailable".to_string(),
            ),
        };
        (
            status,
            Json(json!({
                "schema_version": 1,
                "error": {
                    "code": code,
                    "message": message
                },
                "correlation_id": Uuid::nil()
            })),
        )
            .into_response()
    }
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use sqlx::postgres::PgPoolOptions;
    use tower::ServiceExt;

    use super::*;

    fn service_endpoints(
        core_authorization_url: &str,
        provider_url: &str,
    ) -> DatasetServiceEndpoints {
        DatasetServiceEndpoints::new(
            core_authorization_url,
            BTreeMap::from([
                (
                    tessara_responses_contract::RESPONSE_EXPORT_BINDING_KEY.to_string(),
                    provider_url.to_string(),
                ),
                (
                    tessara_forms_contract::FORM_VERSION_SCHEMA_BINDING_KEY.to_string(),
                    provider_url.to_string(),
                ),
                (
                    tessara_control_plane_contract::SCOPE_CATALOG_BINDING_KEY.to_string(),
                    provider_url.to_string(),
                ),
                (
                    tessara_control_plane_contract::PRINCIPAL_DISPLAY_BINDING_KEY.to_string(),
                    provider_url.to_string(),
                ),
            ]),
        )
        .expect("valid Dataset service endpoints")
    }

    fn test_state(pool: PgPool) -> DatasetModuleState {
        let authorization_signer =
            tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.core",
                "test-core-v1",
                tessara_module_contract::ProtocolSignaturePurposeV1::AuthorizationGrant,
                [7; 32],
            )
            .unwrap();
        let shell_signer = tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "test-core-v1",
            tessara_module_contract::ProtocolSignaturePurposeV1::ShellContext,
            [7; 32],
        )
        .unwrap();
        let core_service_request_signer =
            tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.core",
                "test-core-v1",
                tessara_module_contract::ProtocolSignaturePurposeV1::ModuleServiceRequest,
                [7; 32],
            )
            .unwrap();
        let bootstrap_signer =
            tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.core",
                "test-core-v1",
                tessara_module_contract::ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
                [7; 32],
            )
            .unwrap();
        let owner_bootstrap_signer =
            tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.core",
                "test-core-v1",
                tessara_module_contract::ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
                [7; 32],
            )
            .unwrap();
        let service_identity_registry =
            tessara_module_contract::ModuleServiceIdentityRegistryV1::from_json(
                r#"{"schema_version":1,"identities":{"tessara.components":{"key_id":"test-component-v1","public_key":"11qYAYKxCrfVS_7TyWQHOg7hcvPapiMlrwIaaPcHURo"}}}"#,
            )
            .unwrap();
        let service_signer = Arc::new(
            tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.datasets",
                "test-dataset-v1",
                tessara_module_contract::ProtocolSignaturePurposeV1::ModuleServiceRequest,
                [8; 32],
            )
            .unwrap(),
        );
        let receipt_signer = Arc::new(
            tessara_module_contract::PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.datasets",
                "test-dataset-v1",
                tessara_module_contract::ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
                [8; 32],
            )
            .unwrap(),
        );
        DatasetModuleState::new(
            pool,
            DatasetCoreVerifiers {
                authorization: authorization_signer.verifier(),
                owner_bootstrap: owner_bootstrap_signer.verifier(),
                service_request: core_service_request_signer.verifier(),
                bootstrap_validation: bootstrap_signer.verifier(),
                shell: shell_signer.verifier(),
            },
            service_identity_registry,
            service_signer,
            receipt_signer,
            service_endpoints("http://127.0.0.1:1", "http://127.0.0.1:2"),
            DatasetValidationFaultControl::disabled(),
        )
        .unwrap()
    }

    fn disconnected_state() -> DatasetModuleState {
        test_state(
            PgPoolOptions::new()
                .acquire_timeout(std::time::Duration::from_millis(10))
                .connect_lazy("postgres://dataset:dataset@127.0.0.1:1/dataset_test")
                .expect("lazy Dataset pool"),
        )
    }

    #[test]
    fn provider_endpoints_are_an_exact_binding_keyed_http_origin_map() {
        let endpoints = DatasetServiceEndpoints::from_json(
            " https://core.internal.example/ ",
            r#"{
                "tessara.datasets.response-export":"http://response-proxy:8080/",
                "tessara.datasets.form-version-schema":"http://form-proxy:8080",
                "tessara.datasets.scope-catalog":"https://scope-proxy.example",
                "tessara.datasets.principal-display-catalog":"http://principal-proxy:8080"
            }"#,
        )
        .expect("exact Dataset provider endpoint map");
        assert_eq!(
            endpoints.core_authorization_url(),
            "https://core.internal.example"
        );
        assert_eq!(
            endpoints.provider_url(tessara_responses_contract::RESPONSE_EXPORT_BINDING_KEY),
            Some("http://response-proxy:8080")
        );
        assert_eq!(
            endpoints.provider_url(tessara_forms_contract::FORM_VERSION_SCHEMA_BINDING_KEY),
            Some("http://form-proxy:8080")
        );
        assert_eq!(
            endpoints.provider_url(tessara_control_plane_contract::SCOPE_CATALOG_BINDING_KEY),
            Some("https://scope-proxy.example")
        );
        assert_eq!(
            endpoints.provider_url(tessara_control_plane_contract::PRINCIPAL_DISPLAY_BINDING_KEY),
            Some("http://principal-proxy:8080")
        );
        assert_eq!(endpoints.provider_url("tessara.datasets.unknown"), None);

        for invalid in [
            "not-json",
            r#"{"tessara.datasets.response-export":"http://response-proxy:8080"}"#,
            r#"{
                "tessara.datasets.response-export":"http://response-proxy:8080",
                "tessara.datasets.form-version-schema":"http://form-proxy:8080",
                "tessara.datasets.scope-catalog":"http://scope-proxy:8080",
                "tessara.datasets.principal-display-catalog":"http://principal-proxy:8080",
                "tessara.datasets.extra":"http://extra-proxy:8080"
            }"#,
            r#"{
                "tessara.datasets.response-export":"response-proxy:8080",
                "tessara.datasets.form-version-schema":"http://form-proxy:8080",
                "tessara.datasets.scope-catalog":"http://scope-proxy:8080",
                "tessara.datasets.principal-display-catalog":"http://principal-proxy:8080"
            }"#,
            r#"{
                "tessara.datasets.response-export":"file:///response-proxy",
                "tessara.datasets.form-version-schema":"http://form-proxy:8080",
                "tessara.datasets.scope-catalog":"http://scope-proxy:8080",
                "tessara.datasets.principal-display-catalog":"http://principal-proxy:8080"
            }"#,
            r#"{
                "tessara.datasets.response-export":"http://response-proxy:8080/private",
                "tessara.datasets.form-version-schema":"http://form-proxy:8080",
                "tessara.datasets.scope-catalog":"http://scope-proxy:8080",
                "tessara.datasets.principal-display-catalog":"http://principal-proxy:8080"
            }"#,
        ] {
            assert!(
                DatasetServiceEndpoints::from_json("http://core:8080", invalid).is_err(),
                "invalid provider endpoint map was accepted: {invalid}"
            );
        }
        assert!(
            DatasetServiceEndpoints::new(
                "core:8080",
                BTreeMap::from([
                    (
                        tessara_responses_contract::RESPONSE_EXPORT_BINDING_KEY.to_string(),
                        "http://response-proxy:8080".to_string(),
                    ),
                    (
                        tessara_forms_contract::FORM_VERSION_SCHEMA_BINDING_KEY.to_string(),
                        "http://form-proxy:8080".to_string(),
                    ),
                    (
                        tessara_control_plane_contract::SCOPE_CATALOG_BINDING_KEY.to_string(),
                        "http://scope-proxy:8080".to_string(),
                    ),
                    (
                        tessara_control_plane_contract::PRINCIPAL_DISPLAY_BINDING_KEY.to_string(),
                        "http://principal-proxy:8080".to_string(),
                    ),
                ]),
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn provider_outage_does_not_remove_the_core_authorization_endpoint() {
        let core_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind Core authorization listener");
        let core_address = core_listener.local_addr().expect("Core listener address");
        let core_server = tokio::spawn(async move {
            axum::serve(
                core_listener,
                Router::new().route(
                    "/api/private/module-authorization/exchange",
                    post(|| async { StatusCode::NO_CONTENT }),
                ),
            )
            .await
            .expect("serve Core authorization listener");
        });
        let unavailable_listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("reserve unavailable provider address");
        let unavailable_address = unavailable_listener
            .local_addr()
            .expect("unavailable provider address");
        drop(unavailable_listener);
        let endpoints = service_endpoints(
            &format!("http://{core_address}"),
            &format!("http://{unavailable_address}"),
        );
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(1))
            .build()
            .expect("test HTTP client");
        let exchange_url = format!(
            "{}/api/private/module-authorization/exchange",
            endpoints.core_authorization_url()
        );
        assert_eq!(
            client
                .post(&exchange_url)
                .send()
                .await
                .expect("Core authorization remains reachable")
                .status(),
            StatusCode::NO_CONTENT
        );
        let provider_url = endpoints
            .provider_url(tessara_responses_contract::RESPONSE_EXPORT_BINDING_KEY)
            .expect("Response provider endpoint");
        assert!(
            client
                .post(format!(
                    "{provider_url}/api/private/responses/submitted-export/checkpoint"
                ))
                .send()
                .await
                .is_err(),
            "the independently unavailable provider must remain unavailable"
        );
        assert_eq!(
            client
                .post(exchange_url)
                .send()
                .await
                .expect("Core authorization survives provider outage")
                .status(),
            StatusCode::NO_CONTENT
        );
        core_server.abort();
        let _ = core_server.await;
    }

    #[test]
    fn configuration_contract_normalizes_and_rejects_bounds_exactly() {
        let valid = validate_configuration(&DatasetConfigurationV1 {
            schema_version: 1,
            display_label: "  Data products  ".into(),
            provider_request_timeout_seconds: 30,
            provider_retry_limit: 3,
            response_export_page_size: 1000,
        });
        assert!(valid.valid);
        assert_eq!(valid.normalized.unwrap().display_label, "Data products");

        let invalid = validate_configuration(&DatasetConfigurationV1 {
            schema_version: 2,
            display_label: " ".into(),
            provider_request_timeout_seconds: 0,
            provider_retry_limit: 4,
            response_export_page_size: 1001,
        });
        assert!(!invalid.valid);
        assert_eq!(
            invalid
                .findings
                .iter()
                .map(|finding| finding.code)
                .collect::<Vec<_>>(),
            [
                "dataset.configuration.schema_version.unsupported",
                "dataset.configuration.display_label.required",
                "dataset.configuration.provider_request_timeout_seconds.out_of_range",
                "dataset.configuration.provider_retry_limit.out_of_range",
                "dataset.configuration.response_export_page_size.out_of_range",
            ]
        );
    }

    #[test]
    fn configuration_json_rejects_unknown_coerced_partial_and_retired_fields() {
        let canonical = json!({
            "schema_version": 1,
            "display_label": "Datasets",
            "provider_request_timeout_seconds": 5,
            "provider_retry_limit": 1,
            "response_export_page_size": 250
        });
        assert!(serde_json::from_value::<DatasetConfigurationV1>(canonical.clone()).is_ok());

        for invalid in [
            {
                let mut value = canonical.clone();
                value["unknown"] = json!(true);
                value
            },
            {
                let mut value = canonical.clone();
                value["provider_request_timeout_seconds"] = json!("5");
                value
            },
            {
                let mut value = canonical.clone();
                value
                    .as_object_mut()
                    .unwrap()
                    .remove("provider_retry_limit");
                value
            },
            json!({
                "schema_version": 1,
                "display_label": "Datasets",
                "provider_timeout_seconds": 5,
                "response_page_size": 250
            }),
        ] {
            assert!(
                serde_json::from_value::<DatasetConfigurationV1>(invalid.clone()).is_err(),
                "noncanonical configuration was accepted: {invalid}"
            );
        }
    }

    #[tokio::test]
    async fn dependency_incompatibility_has_a_distinct_dataset_error_envelope() {
        let response = DatasetModuleError::DependencyIncompatible(
            "FormVersion schema dependency is incompatible".into(),
        )
        .into_response();
        assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(body["error"]["code"], "dataset.dependency_incompatible");
        assert_eq!(
            body["error"]["message"],
            "FormVersion schema dependency is incompatible"
        );
    }

    #[tokio::test]
    async fn control_plane_routes_fail_closed_with_dataset_owned_error_envelope() {
        let response = router(disconnected_state())
            .oneshot(
                Request::builder()
                    .method("PUT")
                    .uri("/api/configuration")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        serde_json::to_vec(&DatasetConfigurationV1::default()).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(body["error"]["code"], "dataset.not_found_or_forbidden");
        assert_eq!(body["error"]["message"], "Forbidden");
        assert!(body.get("message").is_none());
        assert!(body.get("retryable").is_none());
        assert_ne!(body["correlation_id"], Uuid::nil().to_string());
    }

    #[tokio::test]
    async fn liveness_is_exact_json_and_does_not_require_database_readiness() {
        let response = router(disconnected_state())
            .oneshot(
                Request::builder()
                    .uri(LIVENESS_PATH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(
            body,
            json!({
                "schema_version": 1,
                "module_definition_id": MODULE_DEFINITION_ID,
                "module_release_version": MODULE_RELEASE_VERSION,
                "status": "live",
                "database": "not_checked",
                "security_state": "not_checked"
            })
        );
    }

    #[tokio::test]
    async fn readiness_fails_closed_when_database_or_security_state_is_unavailable() {
        let response = router(disconnected_state())
            .oneshot(
                Request::builder()
                    .uri(READINESS_PATH)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        assert_eq!(
            response.headers().get(header::CONTENT_TYPE).unwrap(),
            "application/json"
        );
        assert!(response.headers().get(header::LOCATION).is_none());
        let body: serde_json::Value =
            serde_json::from_slice(&to_bytes(response.into_body(), 16 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(
            body,
            json!({
                "schema_version": 1,
                "module_definition_id": MODULE_DEFINITION_ID,
                "module_release_version": MODULE_RELEASE_VERSION,
                "status": "not_ready",
                "database": "unavailable",
                "security_state": "missing",
                "configuration": "missing",
                "required_bindings": "compatible",
                "product_state": "last_good_missing",
                "freshness": "never_materialized",
                "failures": ["dataset.database.unavailable"]
            })
        );
    }

    #[test]
    fn manifest_is_valid_and_route_action_inventory_is_exact() {
        let manifest = manifest();
        let authority = tessara_module_contract::ManifestNamespaceAuthority::new(
            tessara_module_contract::ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
            tessara_module_contract::PublisherId::new("tessara.first_party").unwrap(),
            ["tessara.datasets", "datasets"],
        )
        .unwrap();
        manifest
            .validate(&authority)
            .expect("Dataset manifest must be semantically valid");
        assert_eq!(manifest.definition_id.as_str(), MODULE_DEFINITION_ID);
        assert_eq!(manifest.release_version.to_string(), MODULE_RELEASE_VERSION);
        let tessara_module_contract::DeploymentProfile::TessaraOciV1(deployment) =
            &manifest.deployment;
        assert_eq!(
            deployment.runtime_image.command,
            ["/usr/local/bin/dataset-module", "serve"]
        );
        assert_eq!(
            deployment
                .migration_image
                .as_ref()
                .expect("Dataset migration image")
                .command,
            ["/usr/local/bin/dataset-module", "migrate"]
        );
        assert_eq!(deployment.listen.protocol, "http");
        assert_eq!(deployment.listen.port, 8093);
        assert_eq!(deployment.listen.registration_name, "datasets");
        assert_eq!(
            deployment.configuration_keys,
            [
                "DISPLAY_LABEL",
                "PROVIDER_REQUEST_TIMEOUT_SECONDS",
                "PROVIDER_RETRY_LIMIT",
                "RESPONSE_EXPORT_PAGE_SIZE",
                "DATASET_MODULE_BIND_ADDR",
                "TESSARA_CORE_INTERNAL_URL",
                "TESSARA_DATASET_PROVIDER_ENDPOINTS",
                "TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY",
                "TESSARA_CORE_AUTHORIZATION_KEY_ID",
                "TESSARA_DATASET_SERVICE_SIGNING_KEY_ID",
                "TESSARA_MODULE_SERVICE_IDENTITIES"
            ]
        );
        assert_eq!(
            deployment.secret_keys,
            [
                "DATABASE_URL",
                "TESSARA_DATASET_SERVICE_SIGNING_KEY",
                "TESSARA_MODULE_CONTROL_SHARED_KEY"
            ]
        );
        assert_eq!(deployment.runtime_identity, RUNTIME_IDENTITY);
        assert_eq!(deployment.migration_identity, MIGRATION_IDENTITY);
        assert_eq!(deployment.readiness_path, READINESS_PATH);
        assert_eq!(deployment.liveness_path, LIVENESS_PATH);
        assert_eq!(manifest.public_api_routes.len(), 21);
        assert_eq!(manifest.browser_routes.len(), 8);
        assert_eq!(manifest.provided_service_actions.len(), 10);
        assert_eq!(manifest.consumed_service_actions.len(), 7);
        assert_eq!(
            manifest.features[0]
                .destinations
                .iter()
                .map(|destination| destination.as_str())
                .collect::<Vec<_>>(),
            [
                "datasets.directory",
                "datasets.create",
                "datasets.detail",
                "datasets.edit",
                "datasets.revisions",
                "datasets.revision_detail",
                "datasets.revision_edit",
            ]
        );
        assert_eq!(
            manifest.features[1]
                .destinations
                .iter()
                .map(|destination| destination.as_str())
                .collect::<Vec<_>>(),
            ["datasets.directory", "datasets.detail", "datasets.preview"]
        );
        let raw: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/manifest.json"
        )))
        .unwrap();
        let tuples = |name: &str, fields: &[&str]| {
            raw[name]
                .as_array()
                .unwrap()
                .iter()
                .map(|item| {
                    fields
                        .iter()
                        .map(|field| item[*field].as_str().unwrap().to_owned())
                        .collect::<Vec<_>>()
                        .join(" ")
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(
            tuples("browser_routes", &["path_template", "destination"]),
            [
                "/datasets datasets.directory",
                "/datasets/new datasets.create",
                "/datasets/{dataset_id}/edit datasets.edit",
                "/datasets/{dataset_id}/preview datasets.preview",
                "/datasets/{dataset_id}/revisions datasets.revisions",
                "/datasets/{dataset_id}/revisions/{revision_id} datasets.revision_detail",
                "/datasets/{dataset_id}/revisions/{revision_id}/edit datasets.revision_edit",
                "/datasets/{dataset_id} datasets.detail",
            ]
        );
        assert_eq!(
            tuples(
                "public_api_routes",
                &["method", "path_template", "authorization_action"]
            ),
            [
                "GET /api/datasets datasets.list",
                "GET /api/datasets/{dataset_id} datasets.get",
                "GET /api/datasets/{dataset_id}/revisions datasets.list_revisions",
                "GET /api/datasets/{dataset_id}/revisions/{revision_id} datasets.get_revision",
                "GET /api/datasets/{dataset_id}/table datasets.preview_table",
                "GET /api/datasets/{dataset_id}/distinct-values datasets.distinct_values",
                "POST /api/admin/datasets datasets.create",
                "DELETE /api/admin/datasets/{dataset_id} datasets.delete",
                "PATCH /api/admin/datasets/{dataset_id}/tags datasets.update_tags",
                "POST /api/admin/datasets/{dataset_id}/draft-revision datasets.save_draft_revision",
                "POST /api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish datasets.publish_revision",
                "PATCH /api/admin/datasets/{dataset_id}/revisions/{revision_id}/label datasets.update_revision_label",
                "PATCH /api/admin/datasets/{dataset_id}/revisions/{revision_id}/options datasets.update_revision_options",
                "DELETE /api/admin/datasets/{dataset_id}/revisions/{revision_id} datasets.delete_revision",
                "POST /api/admin/datasets/sql-preview datasets.preview_sql",
                "POST /api/admin/datasets/{dataset_id}/sql-preview datasets.preview_existing_sql",
                "POST /api/admin/datasets/{dataset_id}/refresh datasets.refresh",
                "GET /api/admin/datasets/editor-options/forms datasets.editor_forms",
                "GET /api/admin/datasets/editor-options/forms/{form_version_id} datasets.editor_form_schema",
                "GET /api/admin/datasets/editor-options/scopes datasets.editor_scopes",
                "GET /api/admin/datasets/editor-options/principals datasets.editor_principals",
            ]
        );
        assert_eq!(
            tuples(
                "provided_service_actions",
                &["method", "path", "authorization_action"]
            ),
            [
                "POST /api/private/datasets/bootstrap-validation datasets.bootstrap_validate",
                "POST /api/private/datasets/catalog datasets.catalog",
                "POST /api/private/datasets/schema datasets.schema",
                "POST /api/private/datasets/distinct-values datasets.distinct_values",
                "POST /api/private/datasets/compatibility datasets.compatibility",
                "POST /api/private/datasets/execute datasets.execute",
                "POST /api/private/datasets/resolve datasets.resolve",
                "POST /api/private/datasets/source-usage datasets.source_usage",
                "POST /api/private/datasets/operations-status datasets.operations_status",
                "POST /api/private/datasets/summary datasets.summary",
            ]
        );
        assert_eq!(
            tuples(
                "consumed_service_actions",
                &["dependency_binding", "authorization_action"]
            ),
            [
                "tessara.datasets.response-export responses.export_checkpoint",
                "tessara.datasets.response-export responses.export_start",
                "tessara.datasets.response-export responses.export_page",
                "tessara.datasets.form-version-schema forms.form_version_catalog",
                "tessara.datasets.form-version-schema forms.form_version_schema",
                "tessara.datasets.scope-catalog core.scope_catalog",
                "tessara.datasets.principal-display-catalog core.principal_display_catalog",
            ]
        );
        assert_eq!(manifest.navigation.len(), 1);
        assert_eq!(
            manifest.navigation[0].id.as_str(),
            "tessara.datasets.navigation"
        );
        assert_eq!(manifest.navigation[0].order_hint, 6);
        let create_route = manifest
            .browser_routes
            .iter()
            .find(|route| route.path_template == "/datasets/new")
            .expect("Dataset create browser route");
        assert_eq!(
            create_route.methods,
            [
                tessara_module_contract::BrowserDocumentMethod::Get,
                tessara_module_contract::BrowserDocumentMethod::Head
            ]
        );
    }

    #[test]
    fn upgrade_baseline_projects_only_release_identity_and_keeps_dataset_v2() {
        let tracked: tessara_module_contract::ModuleManifest = serde_json::from_str(include_str!(
            concat!(env!("CARGO_MANIFEST_DIR"), "/manifest.json")
        ))
        .expect("tracked Dataset candidate manifest must remain valid");
        assert_eq!(
            tracked.release_version.to_string(),
            CURRENT_MODULE_RELEASE_VERSION,
            "the tracked candidate manifest must remain the real 1.0.0 release"
        );

        let projected = manifest();
        let expected_release = if cfg!(feature = "sprint-8b-upgrade-baseline") {
            PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION
        } else {
            CURRENT_MODULE_RELEASE_VERSION
        };
        assert_eq!(MODULE_RELEASE_VERSION, expected_release);
        assert_eq!(projected.release_version.to_string(), expected_release);

        let dataset_v2 = projected
            .provided_contracts
            .iter()
            .find(|contract| contract.id.as_str() == "tessara.datasets.dataset-major-line")
            .expect("Dataset major-line contract must remain declared");
        assert_eq!(dataset_v2.version.to_string(), "2.0.0");

        let mut normalized = projected;
        normalized.release_version = tracked.release_version.clone();
        assert_eq!(
            normalized, tracked,
            "the prior-compatible feature may project release identity only"
        );
    }
}
