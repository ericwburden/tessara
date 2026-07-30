//! Native runtime integration shared by independently deployed Tessara modules.

use std::{collections::BTreeMap, env, net::SocketAddr, sync::Arc};

use async_trait::async_trait;
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_module_contract::{
    ModuleManifest, ProtocolEnvelopeError, ProtocolSignaturePurposeV1, PurposeBoundVerifyingKeyV1,
    ShellContextV1, ShellContextValidationContextV1, ShellContextValidationError, SignedEnvelopeV1,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

/// Current schema for every runtime-owned public error response.
pub const RUNTIME_ERROR_SCHEMA_VERSION: u16 = 1;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeErrorEnvelope {
    pub schema_version: u16,
    pub code: String,
    pub message: String,
    pub correlation_id: Uuid,
    pub retryable: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCheck {
    pub code: String,
    pub passing: bool,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeHealthEnvelope {
    pub schema_version: u16,
    pub status: RuntimeHealthStatus,
    pub checks: Vec<RuntimeCheck>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeHealthStatus {
    Passing,
    Failing,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationValidationEnvelope {
    pub schema_version: u16,
    pub valid: bool,
    pub normalized: Option<Value>,
    pub findings: Vec<RuntimeCheck>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectedSecurityState {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub module_instance_id: Uuid,
    pub authorization_revision: u64,
    pub organization_revision: u64,
    pub enabled: bool,
    pub document_state: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDiagnosticsEnvelope {
    pub schema_version: u16,
    pub release: String,
    pub contract_version: String,
    pub runtime_version: String,
    pub ui_version: Option<String>,
    pub health: RuntimeHealthStatus,
    pub facts: BTreeMap<String, String>,
    pub findings: Vec<RuntimeCheck>,
}

#[async_trait]
pub trait ConfigurationProvider: Send + Sync {
    async fn validate(&self, proposed: Value) -> ConfigurationValidationEnvelope;
    async fn current(&self) -> Result<Value, RuntimeProviderError>;
    async fn apply(&self, normalized: Value) -> Result<Value, RuntimeProviderError>;
}

#[async_trait]
pub trait SecurityStateProvider: Send + Sync {
    async fn current_security_state(&self) -> Result<ProjectedSecurityState, RuntimeProviderError>;
    async fn apply_security_state(
        &self,
        state: ProjectedSecurityState,
    ) -> Result<ProjectedSecurityState, RuntimeProviderError>;
}

#[async_trait]
pub trait ReadinessProvider: Send + Sync {
    async fn readiness_checks(&self) -> Vec<RuntimeCheck>;
}

#[async_trait]
pub trait DiagnosticsProvider: Send + Sync {
    async fn diagnostic_facts(&self) -> BTreeMap<String, String>;
    async fn diagnostic_findings(&self) -> Vec<RuntimeCheck>;
}

pub trait ModuleDefinitionProvider: Send + Sync {
    fn manifest(&self) -> &ModuleManifest;
    fn asset_manifest(&self) -> BTreeMap<String, String>;
    fn asset_bytes(&self, digest: &str) -> Option<(&'static str, &'static [u8])>;
}

#[derive(Debug, thiserror::Error)]
pub enum RuntimeProviderError {
    #[error("runtime state is unavailable")]
    Unavailable,
    #[error("runtime state is invalid")]
    Invalid,
    #[error("runtime state could not be persisted")]
    Persistence,
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleShellError {
    #[error("shell signature validation failed: {0}")]
    Signature(#[from] ProtocolEnvelopeError),
    #[error("shell context validation failed: {0}")]
    Context(#[from] ShellContextValidationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ModuleRequestEnvelopeError {
    #[error("required module projection header is missing")]
    Missing,
    #[error("module projection header encoding is invalid")]
    Encoding,
    #[error("module projection envelope is invalid")]
    Wire,
}

pub fn decode_signed_envelope_header<T: DeserializeOwned>(
    headers: &HeaderMap,
    name: &'static str,
) -> Result<SignedEnvelopeV1<T>, ModuleRequestEnvelopeError> {
    let encoded = headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .ok_or(ModuleRequestEnvelopeError::Missing)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| ModuleRequestEnvelopeError::Encoding)?;
    serde_json::from_slice(&bytes).map_err(|_| ModuleRequestEnvelopeError::Wire)
}

pub fn request_correlation_id(headers: &HeaderMap) -> Result<Uuid, ModuleRequestEnvelopeError> {
    headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(ModuleRequestEnvelopeError::Missing)
}

pub fn verify_shell_context(
    envelope: &SignedEnvelopeV1<ShellContextV1>,
    verifier: &PurposeBoundVerifyingKeyV1,
    expected: &ShellContextValidationContextV1,
) -> Result<(), ModuleShellError> {
    verifier.verify(envelope)?;
    envelope.payload.validate_for(expected)?;
    Ok(())
}

#[derive(Clone)]
pub struct CoreVerifiers {
    pub authorization: PurposeBoundVerifyingKeyV1,
    pub shell: PurposeBoundVerifyingKeyV1,
}

impl CoreVerifiers {
    pub fn from_environment() -> anyhow::Result<Self> {
        let encoded = env::var("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY")
            .map_err(|_| anyhow::anyhow!("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY is required"))?;
        let public_key: [u8; 32] = URL_SAFE_NO_PAD
            .decode(encoded)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("Core authorization public key must contain 32 bytes"))?;
        Ok(Self {
            authorization: PurposeBoundVerifyingKeyV1::from_public_bytes(
                "tessara.core",
                "core-development-v1",
                ProtocolSignaturePurposeV1::AuthorizationGrant,
                public_key,
            )?,
            shell: PurposeBoundVerifyingKeyV1::from_public_bytes(
                "tessara.core",
                "core-development-v1",
                ProtocolSignaturePurposeV1::ShellContext,
                public_key,
            )?,
        })
    }
}

pub fn standard_probe_router<P>() -> Router<Arc<P>>
where
    P: ReadinessProvider + 'static,
{
    Router::new()
        .route("/health/live", get(liveness))
        .route("/health/ready", get(readiness::<P>))
}

pub fn standard_control_router<P>() -> Router<Arc<P>>
where
    P: ConfigurationProvider
        + SecurityStateProvider
        + ReadinessProvider
        + DiagnosticsProvider
        + ModuleDefinitionProvider
        + 'static,
{
    Router::new()
        .route(
            "/api/configuration/validate",
            post(validate_configuration::<P>),
        )
        .route(
            "/api/configuration",
            get(current_configuration::<P>).put(apply_configuration::<P>),
        )
        .route(
            "/api/private/security-state",
            get(current_security::<P>).put(apply_security::<P>),
        )
        .route("/api/diagnostics", get(diagnostics::<P>))
}

fn control_key_is_valid(headers: &HeaderMap) -> bool {
    let expected = env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
}

async fn validate_configuration<P>(
    State(provider): State<Arc<P>>,
    headers: HeaderMap,
    Json(value): Json<Value>,
) -> Response
where
    P: ConfigurationProvider,
{
    if !control_key_is_valid(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    Json(provider.validate(value).await).into_response()
}

async fn current_configuration<P>(State(provider): State<Arc<P>>, headers: HeaderMap) -> Response
where
    P: ConfigurationProvider,
{
    if !control_key_is_valid(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match provider.current().await {
        Ok(value) => Json(value).into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn apply_configuration<P>(
    State(provider): State<Arc<P>>,
    headers: HeaderMap,
    Json(value): Json<Value>,
) -> Response
where
    P: ConfigurationProvider,
{
    if !control_key_is_valid(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let validation = provider.validate(value).await;
    let Some(normalized) = validation.normalized else {
        return (StatusCode::UNPROCESSABLE_ENTITY, Json(validation)).into_response();
    };
    match provider.apply(normalized).await {
        Ok(normalized) => Json(ConfigurationValidationEnvelope {
            schema_version: 1,
            valid: true,
            normalized: Some(normalized),
            findings: Vec::new(),
        })
        .into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn current_security<P>(State(provider): State<Arc<P>>, headers: HeaderMap) -> Response
where
    P: SecurityStateProvider,
{
    if !control_key_is_valid(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match provider.current_security_state().await {
        Ok(state) => Json(state).into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn apply_security<P>(
    State(provider): State<Arc<P>>,
    headers: HeaderMap,
    Json(value): Json<ProjectedSecurityState>,
) -> Response
where
    P: SecurityStateProvider,
{
    if !control_key_is_valid(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match provider.apply_security_state(value).await {
        Ok(state) => Json(state).into_response(),
        Err(RuntimeProviderError::Invalid) => StatusCode::UNPROCESSABLE_ENTITY.into_response(),
        Err(_) => StatusCode::SERVICE_UNAVAILABLE.into_response(),
    }
}

async fn diagnostics<P>(State(provider): State<Arc<P>>, headers: HeaderMap) -> Response
where
    P: ReadinessProvider + DiagnosticsProvider + ModuleDefinitionProvider,
{
    if !control_key_is_valid(&headers) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let findings = provider.diagnostic_findings().await;
    let checks = provider.readiness_checks().await;
    let passing = checks.iter().all(|check| check.passing);
    let manifest = provider.manifest();
    Json(RuntimeDiagnosticsEnvelope {
        schema_version: 1,
        release: manifest.release_version.to_string(),
        contract_version: manifest.linked_packages.module_contract.to_string(),
        runtime_version: manifest
            .linked_packages
            .module_runtime
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| "not-linked".into()),
        ui_version: manifest
            .linked_packages
            .module_ui
            .as_ref()
            .map(ToString::to_string),
        health: if passing {
            RuntimeHealthStatus::Passing
        } else {
            RuntimeHealthStatus::Failing
        },
        facts: provider.diagnostic_facts().await,
        findings,
    })
    .into_response()
}

async fn liveness() -> Json<RuntimeHealthEnvelope> {
    Json(RuntimeHealthEnvelope {
        schema_version: 1,
        status: RuntimeHealthStatus::Passing,
        checks: vec![],
    })
}

async fn readiness<P>(State(provider): State<Arc<P>>) -> Response
where
    P: ReadinessProvider,
{
    let checks = provider.readiness_checks().await;
    let passing = checks.iter().all(|check| check.passing);
    (
        if passing {
            StatusCode::OK
        } else {
            StatusCode::SERVICE_UNAVAILABLE
        },
        Json(RuntimeHealthEnvelope {
            schema_version: 1,
            status: if passing {
                RuntimeHealthStatus::Passing
            } else {
                RuntimeHealthStatus::Failing
            },
            checks,
        }),
    )
        .into_response()
}

pub async fn serve(
    address: SocketAddr,
    router: Router,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "module runtime listening");
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown)
        .await?;
    Ok(())
}

pub fn initialize_tracing() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}

pub fn standard_http_router(router: Router) -> Router {
    router.layer(TraceLayer::new_for_http())
}

pub async fn shutdown_signal() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use tessara_module_contract::{
        ModuleDefinitionId, OriginalActorProjectionV1, ProtocolSignaturePurposeV1,
        PurposeBoundSigningKeyV1, ShellDocumentStateV1, ShellThemeV1,
    };

    use super::*;

    #[test]
    fn shell_verification_is_signature_first_and_context_bound() {
        let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "shell-v1",
            ProtocolSignaturePurposeV1::ShellContext,
            [44; 32],
        )
        .unwrap();
        let now = Utc::now();
        let context = ShellContextV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            module_definition_id: ModuleDefinitionId::new("tessara.reference.module-sdk").unwrap(),
            module_instance_id: Uuid::from_u128(2),
            original_actor: OriginalActorProjectionV1 {
                actor_id: Uuid::from_u128(3),
                display_name: "Operator".into(),
                email: None,
            },
            theme: ShellThemeV1::System,
            navigation: vec![],
            return_destination: "/".into(),
            locale: "en-US".into(),
            time_zone: "UTC".into(),
            correlation_id: Uuid::from_u128(4),
            document_state: ShellDocumentStateV1::Active,
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        };
        let envelope = signer.sign(context.clone()).unwrap();
        verify_shell_context(
            &envelope,
            &signer.verifier(),
            &ShellContextValidationContextV1 {
                installation_id: context.installation_id,
                module_definition_id: context.module_definition_id,
                module_instance_id: context.module_instance_id,
                correlation_id: context.correlation_id,
                now,
            },
        )
        .unwrap();
    }
}
