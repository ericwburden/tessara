//! Independently deployed Response module owner.

use async_trait::async_trait;
use axum::{
    Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{FromRow, PgPool};
use std::{
    collections::BTreeMap,
    sync::{Arc, OnceLock},
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, DependencyBindingKey, FunctionalContractId,
    ModuleDefinitionId, ModuleServicePrincipalV1, SecurityCapabilityId,
    ShellContextValidationContextV2,
};
use tessara_module_runtime::{
    ConfigurationProvider, ConfigurationValidationEnvelope, CoreVerifiers, DiagnosticsProvider,
    ModuleDefinitionProvider, ProjectedSecurityState, ReadinessProvider, RuntimeCheck,
    RuntimeProviderError, SecurityStateProvider, decode_signed_envelope_header,
    request_correlation_id, verify_shell_context,
};
use tessara_response_ui::{
    RESPONSE_BINDINGS_JS, RESPONSE_BINDINGS_JS_SHA256, RESPONSE_CSS, RESPONSE_CSS_SHA256,
    RESPONSE_JS, RESPONSE_JS_SHA256, RESPONSE_WASM_SHA256, ResponseRouteBootstrap,
    render_response_document, response_asset_path,
};
use uuid::Uuid;

mod owner;
mod product_store;
pub use owner::{
    CreateResponseCommand, IdempotentCommit, ResponseOwnerError, ResponseOwnerRepository,
    ResponseValueInput, canonical_digest,
};
pub use product_store::{
    ResponseAccess, ResponseListFilter, ResponseMutationCommand, SaveResponseCommand,
};

pub const MODULE_DEFINITION_ID: &str = "tessara.responses";
pub const CURRENT_MODULE_RELEASE_VERSION: &str = "1.0.0";
pub const PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION: &str = "0.9.0";
#[cfg(feature = "sprint-8c-upgrade-baseline")]
pub const MODULE_RELEASE_VERSION: &str = PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION;
#[cfg(not(feature = "sprint-8c-upgrade-baseline"))]
pub const MODULE_RELEASE_VERSION: &str = CURRENT_MODULE_RELEASE_VERSION;
pub const RUNTIME_IDENTITY: &str = "responses-runtime";
pub const MIGRATION_IDENTITY: &str = "responses-migration";

pub fn manifest() -> tessara_module_contract::ModuleManifest {
    let mut manifest: tessara_module_contract::ModuleManifest =
        serde_json::from_str(include_str!("../manifest.json"))
            .expect("Response manifest is valid current JSON");
    if cfg!(feature = "sprint-8c-upgrade-baseline") {
        manifest.release_version = PRIOR_COMPATIBLE_MODULE_RELEASE_VERSION
            .parse()
            .expect("valid prior release");
    }
    manifest
}

#[derive(Clone)]
pub struct ResponseRuntime {
    pool: PgPool,
    verifiers: CoreVerifiers,
}
impl ResponseRuntime {
    pub fn new(pool: PgPool, verifiers: CoreVerifiers) -> Self {
        Self { pool, verifiers }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, FromRow)]
#[serde(deny_unknown_fields)]
pub struct ResponseConfiguration {
    pub schema_version: i16,
    pub display_label: String,
    pub provider_request_timeout_seconds: i16,
    pub workflow_event_page_size: i16,
}
impl Default for ResponseConfiguration {
    fn default() -> Self {
        Self {
            schema_version: 1,
            display_label: "Responses".into(),
            provider_request_timeout_seconds: 5,
            workflow_event_page_size: 250,
        }
    }
}
impl ResponseConfiguration {
    fn normalize(mut self) -> Result<Self, Vec<RuntimeCheck>> {
        self.display_label = self.display_label.trim().into();
        let mut findings = Vec::new();
        if self.schema_version != 1 {
            findings.push(finding(
                "response.configuration.schema_version",
                "schema_version must be 1",
            ));
        }
        if self.display_label.is_empty() || self.display_label.chars().count() > 80 {
            findings.push(finding(
                "response.configuration.display_label",
                "display_label must contain 1 to 80 characters",
            ));
        }
        if !(1..=30).contains(&self.provider_request_timeout_seconds) {
            findings.push(finding(
                "response.configuration.provider_timeout",
                "provider timeout must be between 1 and 30 seconds",
            ));
        }
        if !(1..=1000).contains(&self.workflow_event_page_size) {
            findings.push(finding(
                "response.configuration.event_page_size",
                "event page size must be between 1 and 1000",
            ));
        }
        if findings.is_empty() {
            Ok(self)
        } else {
            Err(findings)
        }
    }
}
fn finding(code: &str, message: &str) -> RuntimeCheck {
    RuntimeCheck {
        code: code.into(),
        passing: false,
        message: message.into(),
    }
}

impl ModuleDefinitionProvider for ResponseRuntime {
    fn manifest(&self) -> &tessara_module_contract::ModuleManifest {
        static MANIFEST: OnceLock<tessara_module_contract::ModuleManifest> = OnceLock::new();
        MANIFEST.get_or_init(manifest)
    }
    fn asset_manifest(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (
                response_asset_path(
                    MODULE_RELEASE_VERSION,
                    tessara_module_ui::MODULE_UI_CSS_SHA256,
                    "module-ui.css",
                ),
                format!("sha256:{}", tessara_module_ui::MODULE_UI_CSS_SHA256),
            ),
            (
                response_asset_path(MODULE_RELEASE_VERSION, RESPONSE_CSS_SHA256, "response.css"),
                format!("sha256:{RESPONSE_CSS_SHA256}"),
            ),
            (
                response_asset_path(MODULE_RELEASE_VERSION, RESPONSE_JS_SHA256, "response.js"),
                format!("sha256:{RESPONSE_JS_SHA256}"),
            ),
            (
                response_asset_path(
                    MODULE_RELEASE_VERSION,
                    RESPONSE_BINDINGS_JS_SHA256,
                    "response-bindings.js",
                ),
                format!("sha256:{RESPONSE_BINDINGS_JS_SHA256}"),
            ),
            (
                response_asset_path(
                    MODULE_RELEASE_VERSION,
                    RESPONSE_WASM_SHA256,
                    "response.wasm",
                ),
                format!("sha256:{RESPONSE_WASM_SHA256}"),
            ),
        ])
    }
    fn asset_bytes(&self, digest: &str) -> Option<(&'static str, &'static [u8])> {
        let module_css = format!("sha256:{}", tessara_module_ui::MODULE_UI_CSS_SHA256);
        if digest == module_css {
            return Some((
                "text/css; charset=utf-8",
                tessara_module_ui::MODULE_UI_CSS.as_bytes(),
            ));
        }
        if digest == format!("sha256:{RESPONSE_CSS_SHA256}") {
            return Some(("text/css; charset=utf-8", RESPONSE_CSS.as_bytes()));
        }
        if digest == format!("sha256:{RESPONSE_JS_SHA256}") {
            return Some(("text/javascript; charset=utf-8", RESPONSE_JS.as_bytes()));
        }
        if digest == format!("sha256:{RESPONSE_BINDINGS_JS_SHA256}") {
            return Some((
                "text/javascript; charset=utf-8",
                RESPONSE_BINDINGS_JS.as_bytes(),
            ));
        }
        if digest == format!("sha256:{RESPONSE_WASM_SHA256}") {
            return Some((
                "application/wasm",
                include_bytes!("../../tessara-web-responses/assets/response.wasm"),
            ));
        }
        None
    }
}

#[async_trait]
impl ConfigurationProvider for ResponseRuntime {
    async fn validate(&self, proposed: Value) -> ConfigurationValidationEnvelope {
        let parsed = serde_json::from_value::<ResponseConfiguration>(proposed)
            .map_err(|_| {
                vec![finding(
                    "response.configuration.shape",
                    "configuration shape is invalid",
                )]
            })
            .and_then(ResponseConfiguration::normalize);
        match parsed {
            Ok(value) => ConfigurationValidationEnvelope {
                schema_version: 1,
                valid: true,
                normalized: serde_json::to_value(value).ok(),
                findings: Vec::new(),
            },
            Err(findings) => ConfigurationValidationEnvelope {
                schema_version: 1,
                valid: false,
                normalized: None,
                findings,
            },
        }
    }
    async fn current(&self) -> Result<Value, RuntimeProviderError> {
        let value = sqlx::query_as::<_, ResponseConfiguration>("SELECT schema_version, display_label, provider_request_timeout_seconds, workflow_event_page_size FROM response_module_configuration WHERE singleton=TRUE").fetch_one(&self.pool).await.map_err(|_| RuntimeProviderError::Persistence)?;
        serde_json::to_value(value).map_err(|_| RuntimeProviderError::Invalid)
    }
    async fn apply(&self, normalized: Value) -> Result<Value, RuntimeProviderError> {
        let value = serde_json::from_value::<ResponseConfiguration>(normalized)
            .map_err(|_| RuntimeProviderError::Invalid)?
            .normalize()
            .map_err(|_| RuntimeProviderError::Invalid)?;
        sqlx::query("UPDATE response_module_configuration SET schema_version=$1,display_label=$2,provider_request_timeout_seconds=$3,workflow_event_page_size=$4,updated_at=now() WHERE singleton=TRUE")
            .bind(value.schema_version).bind(&value.display_label).bind(value.provider_request_timeout_seconds).bind(value.workflow_event_page_size).execute(&self.pool).await.map_err(|_| RuntimeProviderError::Persistence)?;
        serde_json::to_value(value).map_err(|_| RuntimeProviderError::Invalid)
    }
}

#[derive(FromRow)]
struct SecurityRow {
    schema_version: i16,
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: String,
}

#[async_trait]
impl SecurityStateProvider for ResponseRuntime {
    async fn current_security_state(&self) -> Result<ProjectedSecurityState, RuntimeProviderError> {
        let row = sqlx::query_as::<_, SecurityRow>("SELECT schema_version,installation_id,module_instance_id,authorization_revision,organization_revision,enabled,document_state FROM response_module_security_state WHERE singleton=TRUE").fetch_optional(&self.pool).await.map_err(|_| RuntimeProviderError::Persistence)?.ok_or(RuntimeProviderError::Unavailable)?;
        Ok(ProjectedSecurityState {
            schema_version: row.schema_version as u16,
            installation_id: row.installation_id,
            module_instance_id: row.module_instance_id,
            authorization_revision: row.authorization_revision as u64,
            organization_revision: row.organization_revision as u64,
            enabled: row.enabled,
            document_state: row.document_state,
        })
    }
    async fn apply_security_state(
        &self,
        value: ProjectedSecurityState,
    ) -> Result<ProjectedSecurityState, RuntimeProviderError> {
        if value.schema_version != 1
            || value.installation_id.is_nil()
            || value.module_instance_id.is_nil()
        {
            return Err(RuntimeProviderError::Invalid);
        }
        let result = sqlx::query("INSERT INTO response_module_security_state (singleton,schema_version,installation_id,module_instance_id,authorization_revision,organization_revision,enabled,document_state) VALUES (TRUE,$1,$2,$3,$4,$5,$6,$7) ON CONFLICT(singleton) DO UPDATE SET authorization_revision=EXCLUDED.authorization_revision,organization_revision=EXCLUDED.organization_revision,enabled=EXCLUDED.enabled,document_state=EXCLUDED.document_state,updated_at=now() WHERE response_module_security_state.installation_id=EXCLUDED.installation_id AND response_module_security_state.module_instance_id=EXCLUDED.module_instance_id")
            .bind(value.schema_version as i16).bind(value.installation_id).bind(value.module_instance_id).bind(value.authorization_revision as i64).bind(value.organization_revision as i64).bind(value.enabled).bind(&value.document_state).execute(&self.pool).await.map_err(|_| RuntimeProviderError::Persistence)?;
        if result.rows_affected() != 1 {
            return Err(RuntimeProviderError::Invalid);
        }
        Ok(value)
    }
}

#[async_trait]
impl ReadinessProvider for ResponseRuntime {
    async fn readiness_checks(&self) -> Vec<RuntimeCheck> {
        let database = sqlx::query_scalar::<_, i32>("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .is_ok();
        let configuration = self.current().await.is_ok();
        let enabled = self
            .current_security_state()
            .await
            .ok()
            .is_some_and(|state| state.enabled);
        vec![
            RuntimeCheck {
                code: "response.database".into(),
                passing: database,
                message: "Response database must be available".into(),
            },
            RuntimeCheck {
                code: "response.configuration".into(),
                passing: configuration,
                message: "Response configuration must be available".into(),
            },
            RuntimeCheck {
                code: "response.security_state".into(),
                passing: enabled,
                message: "Response security projection must be enabled".into(),
            },
        ]
    }
}

#[async_trait]
impl DiagnosticsProvider for ResponseRuntime {
    async fn diagnostic_facts(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("definition".into(), MODULE_DEFINITION_ID.into()),
            ("release".into(), MODULE_RELEASE_VERSION.into()),
            ("database_owner".into(), RUNTIME_IDENTITY.into()),
            (
                "resource_contract".into(),
                tessara_responses_contract::RESPONSE_CONTRACT_VERSION.into(),
            ),
        ])
    }
    async fn diagnostic_findings(&self) -> Vec<RuntimeCheck> {
        self.readiness_checks()
            .await
            .into_iter()
            .filter(|check| !check.passing)
            .collect()
    }
}

pub fn router(runtime: Arc<ResponseRuntime>) -> Router {
    Router::new()
        .route("/responses", get(directory_document))
        .route("/responses/new", get(start_document))
        .route("/responses/{response_id}", get(detail_document))
        .route("/responses/{response_id}/edit", get(edit_document))
        .route(
            "/_tessara/modules/tessara.responses/{release}/{digest}/{asset}",
            get(asset),
        )
        .merge(tessara_module_runtime::standard_control_router::<
            ResponseRuntime,
        >())
        .merge(tessara_module_runtime::standard_probe_router::<
            ResponseRuntime,
        >())
        .with_state(runtime)
}

async fn directory_document(
    State(runtime): State<Arc<ResponseRuntime>>,
    headers: HeaderMap,
) -> Response {
    document(
        &runtime,
        &headers,
        "responses.list",
        "submissions:read_own",
        "/responses",
        "Responses",
        ResponseRouteBootstrap::Directory,
    )
    .await
}
async fn start_document(
    State(runtime): State<Arc<ResponseRuntime>>,
    headers: HeaderMap,
) -> Response {
    document(
        &runtime,
        &headers,
        "responses.start",
        "submissions:respond",
        "/responses/new",
        "Start Response",
        ResponseRouteBootstrap::Start,
    )
    .await
}
async fn detail_document(
    State(runtime): State<Arc<ResponseRuntime>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let path = format!("/responses/{id}");
    document(
        &runtime,
        &headers,
        "responses.get",
        "submissions:read_own",
        &path,
        "Response Detail",
        ResponseRouteBootstrap::Detail {
            response_id: id.to_string(),
        },
    )
    .await
}
async fn edit_document(
    State(runtime): State<Arc<ResponseRuntime>>,
    Path(id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    let path = format!("/responses/{id}/edit");
    document(
        &runtime,
        &headers,
        "responses.edit",
        "submissions:respond",
        &path,
        "Edit Response",
        ResponseRouteBootstrap::Edit {
            response_id: id.to_string(),
        },
    )
    .await
}

async fn document(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    action: &str,
    capability: &str,
    path: &str,
    title: &str,
    bootstrap: ResponseRouteBootstrap,
) -> Response {
    let Ok(shell) = verified_document(runtime, headers, action, capability).await else {
        return (StatusCode::FORBIDDEN, "module action unavailable").into_response();
    };
    Html(render_response_document(
        &shell,
        path,
        title,
        &bootstrap,
        MODULE_RELEASE_VERSION,
    ))
    .into_response()
}

async fn verified_document(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    action: &str,
    capability: &str,
) -> Result<tessara_module_contract::ShellContextV2, ()> {
    let authorization: tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3> =
        decode_signed_envelope_header(headers, "x-tessara-authorization").map_err(|_| ())?;
    runtime
        .verifiers
        .authorization
        .verify(&authorization)
        .map_err(|_| ())?;
    let security = runtime.current_security_state().await.map_err(|_| ())?;
    let correlation_id = request_correlation_id(headers).map_err(|_| ())?;
    authorization
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                    .map_err(|_| ())?,
            },
            dependency_binding: DependencyBindingKey::new("tessara.core.module-document")
                .map_err(|_| ())?,
            functional_contract: FunctionalContractId::new(
                tessara_responses_contract::RESPONSE_LIFECYCLE_CONTRACT_ID,
            )
            .map_err(|_| ())?,
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision,
            organization_revision: security.organization_revision,
            now: chrono::Utc::now(),
        })
        .map_err(|_| ())?;
    let capability = SecurityCapabilityId::new(capability).map_err(|_| ())?;
    if !authorization
        .payload
        .capability_scope_bindings
        .iter()
        .any(|binding| binding.capability == capability)
    {
        return Err(());
    }
    let shell =
        decode_signed_envelope_header(headers, "x-tessara-shell-context").map_err(|_| ())?;
    verify_shell_context(
        &shell,
        &runtime.verifiers.shell,
        &ShellContextValidationContextV2 {
            installation_id: security.installation_id,
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).map_err(|_| ())?,
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: chrono::Utc::now(),
        },
    )
    .map_err(|_| ())?;
    Ok(shell.payload)
}

async fn asset(Path((release, digest, asset)): Path<(String, String, String)>) -> Response {
    if release != MODULE_RELEASE_VERSION {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (expected, content_type, bytes): (&str, &str, &'static [u8]) = match asset.as_str() {
        "module-ui.css" => (
            tessara_module_ui::MODULE_UI_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_module_ui::MODULE_UI_CSS.as_bytes(),
        ),
        "response.css" => (
            RESPONSE_CSS_SHA256,
            "text/css; charset=utf-8",
            RESPONSE_CSS.as_bytes(),
        ),
        "response.js" => (
            RESPONSE_JS_SHA256,
            "text/javascript; charset=utf-8",
            RESPONSE_JS.as_bytes(),
        ),
        "response-bindings.js" => (
            RESPONSE_BINDINGS_JS_SHA256,
            "text/javascript; charset=utf-8",
            RESPONSE_BINDINGS_JS.as_bytes(),
        ),
        "response.wasm" => (
            RESPONSE_WASM_SHA256,
            "application/wasm",
            include_bytes!("../../tessara-web-responses/assets/response.wasm"),
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    if digest != format!("sha256:{expected}") {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        Body::from(bytes),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::*;

    const BASELINE: &[u8] = include_bytes!("../migrations/001_response_module.sql");

    #[test]
    fn configuration_normalizes_and_rejects_every_bound() {
        let valid = ResponseConfiguration {
            display_label: "  Customer responses  ".into(),
            provider_request_timeout_seconds: 30,
            workflow_event_page_size: 1000,
            ..ResponseConfiguration::default()
        }
        .normalize()
        .expect("valid configuration");
        assert_eq!(valid.display_label, "Customer responses");

        let findings = ResponseConfiguration {
            schema_version: 2,
            display_label: " ".into(),
            provider_request_timeout_seconds: 0,
            workflow_event_page_size: 1001,
        }
        .normalize()
        .expect_err("invalid configuration");
        assert_eq!(
            findings
                .iter()
                .map(|finding| finding.code.as_str())
                .collect::<Vec<_>>(),
            [
                "response.configuration.schema_version",
                "response.configuration.display_label",
                "response.configuration.provider_timeout",
                "response.configuration.event_page_size",
            ]
        );
    }

    #[test]
    fn manifest_is_semantically_valid_and_declares_independent_ownership() {
        let manifest = manifest();
        let authority = tessara_module_contract::ManifestNamespaceAuthority::new(
            tessara_module_contract::ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
            tessara_module_contract::PublisherId::new("tessara.first_party").unwrap(),
            ["tessara.responses", "responses", "submissions"],
        )
        .unwrap();
        manifest
            .validate(&authority)
            .expect("Response manifest must be semantically valid");
        assert_eq!(manifest.definition_id.as_str(), MODULE_DEFINITION_ID);
        assert_eq!(manifest.release_version.to_string(), MODULE_RELEASE_VERSION);
        let tessara_module_contract::DeploymentProfile::TessaraOciV1(deployment) =
            &manifest.deployment;
        assert_eq!(
            deployment.runtime_image.command,
            ["/usr/local/bin/response-module", "serve"]
        );
        assert_eq!(
            deployment
                .migration_image
                .as_ref()
                .expect("Response migration image")
                .command,
            ["/usr/local/bin/response-module", "migrate"]
        );
        assert_eq!(deployment.listen.port, 8094);
        assert_eq!(deployment.listen.registration_name, "responses");
        assert!(manifest.provided_contracts.iter().any(|contract| {
            contract.id.as_str() == tessara_responses_contract::RESPONSE_RESOURCE_CONTRACT_ID
                && contract.version.to_string()
                    == tessara_responses_contract::RESPONSE_CONTRACT_VERSION
        }));
    }

    #[test]
    fn fresh_baseline_is_response_owned_and_has_no_cross_database_constraints() {
        let sql = std::str::from_utf8(BASELINE).expect("baseline migration is UTF-8");
        for required in [
            "CREATE TABLE responses",
            "CREATE TABLE response_values",
            "CREATE TABLE response_audit_events",
            "CREATE TABLE response_idempotency_receipts",
            "CREATE TABLE response_workflow_events",
            "CREATE TABLE response_export_changes",
            "form_snapshot JSONB NOT NULL",
            "workflow_context JSONB NOT NULL",
        ] {
            assert!(
                sql.contains(required),
                "missing baseline owner clause: {required}"
            );
        }
        assert!(!sql.contains("REFERENCES forms"));
        assert!(!sql.contains("REFERENCES workflow"));
        assert!(!sql.contains("submission_value_multi"));
        assert_ne!(
            format!("{:x}", Sha256::digest(BASELINE)),
            format!("{:064x}", 0)
        );
    }
}
