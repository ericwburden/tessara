use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{Arc, OnceLock},
};

use async_trait::async_trait;
use axum::{
    Json, Router,
    body::Body,
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use serde_json::{Value, json};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV1, AuthorizationValidationContextV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, SecurityCapabilityId,
    ShellContextValidationContextV1,
};
use tessara_module_runtime::{
    ConfigurationProvider, ConfigurationValidationEnvelope, CoreVerifiers, DiagnosticsProvider,
    ModuleDefinitionProvider, ProjectedSecurityState, ReadinessProvider, RuntimeCheck,
    RuntimeProviderError, SecurityStateProvider, decode_signed_envelope_header,
    request_correlation_id, verify_shell_context,
};
use tessara_module_ui::{MODULE_SHELL_CSS, ShellPresentation};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{
    DEFINITION_ID, MODULE_SHELL_CSS_DIGEST, MODULE_SHELL_CSS_PATH, MODULE_SHELL_JS_DIGEST,
    MODULE_SHELL_JS_PATH, READ_CAPABILITY, RELEASE_VERSION, ROOT_PATH, ReferenceConfiguration,
    ReferenceSecurityState, ReferenceState, manifest, render_reference_document,
};

#[derive(Clone)]
pub struct ReferenceRuntime {
    state_path: PathBuf,
    state: Arc<RwLock<ReferenceState>>,
    verifiers: CoreVerifiers,
}

impl ReferenceRuntime {
    pub async fn open(state_path: PathBuf, verifiers: CoreVerifiers) -> anyhow::Result<Self> {
        let state = if tokio::fs::try_exists(&state_path).await? {
            let bytes = tokio::fs::read(&state_path).await?;
            let state: ReferenceState = serde_json::from_slice(&bytes)?;
            if state.schema_version != 1 {
                anyhow::bail!("unsupported reference state schema");
            }
            state
        } else {
            let state = ReferenceState::default();
            persist(&state_path, &state).await?;
            state
        };
        Ok(Self {
            state_path,
            state: Arc::new(RwLock::new(state)),
            verifiers,
        })
    }

    async fn save(&self, state: ReferenceState) -> Result<ReferenceState, RuntimeProviderError> {
        persist(&self.state_path, &state)
            .await
            .map_err(|_| RuntimeProviderError::Persistence)?;
        *self.state.write().await = state.clone();
        Ok(state)
    }
}

impl ModuleDefinitionProvider for ReferenceRuntime {
    fn manifest(&self) -> &tessara_module_contract::ModuleManifest {
        static MANIFEST: OnceLock<tessara_module_contract::ModuleManifest> = OnceLock::new();
        MANIFEST.get_or_init(manifest)
    }

    fn asset_manifest(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            (MODULE_SHELL_CSS_PATH.into(), MODULE_SHELL_CSS_DIGEST.into()),
            (MODULE_SHELL_JS_PATH.into(), MODULE_SHELL_JS_DIGEST.into()),
        ])
    }

    fn asset_bytes(&self, digest: &str) -> Option<(&'static str, &'static [u8])> {
        match digest {
            MODULE_SHELL_CSS_DIGEST => {
                Some(("text/css; charset=utf-8", MODULE_SHELL_CSS.as_bytes()))
            }
            MODULE_SHELL_JS_DIGEST => Some((
                "text/javascript; charset=utf-8",
                tessara_module_ui::MODULE_SHELL_JS.as_bytes(),
            )),
            _ => None,
        }
    }
}

async fn persist(path: &PathBuf, state: &ReferenceState) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let temporary = path.with_extension(format!("tmp-{}", Uuid::new_v4()));
    tokio::fs::write(&temporary, serde_json::to_vec_pretty(state)?).await?;
    tokio::fs::rename(temporary, path).await?;
    Ok(())
}

#[async_trait]
impl ConfigurationProvider for ReferenceRuntime {
    async fn validate(&self, proposed: Value) -> ConfigurationValidationEnvelope {
        let result = proposed
            .get("display_label")
            .and_then(Value::as_str)
            .ok_or("display_label is required")
            .and_then(ReferenceConfiguration::normalize);
        match result {
            Ok(configuration) => ConfigurationValidationEnvelope {
                schema_version: 1,
                valid: true,
                normalized: Some(json!({"display_label": configuration.display_label})),
                findings: vec![],
            },
            Err(message) => ConfigurationValidationEnvelope {
                schema_version: 1,
                valid: false,
                normalized: None,
                findings: vec![RuntimeCheck {
                    code: "invalid_display_label".into(),
                    passing: false,
                    message: message.into(),
                }],
            },
        }
    }

    async fn current(&self) -> Result<Value, RuntimeProviderError> {
        Ok(serde_json::to_value(&self.state.read().await.configuration)
            .map_err(|_| RuntimeProviderError::Invalid)?)
    }

    async fn apply(&self, normalized: Value) -> Result<Value, RuntimeProviderError> {
        let configuration: ReferenceConfiguration =
            serde_json::from_value(normalized).map_err(|_| RuntimeProviderError::Invalid)?;
        let mut state = self.state.read().await.clone();
        state.configuration = configuration;
        let state = self.save(state).await?;
        serde_json::to_value(state.configuration).map_err(|_| RuntimeProviderError::Invalid)
    }
}

#[async_trait]
impl SecurityStateProvider for ReferenceRuntime {
    async fn current_security_state(&self) -> Result<ProjectedSecurityState, RuntimeProviderError> {
        let state = self.state.read().await;
        let security = state
            .security
            .as_ref()
            .ok_or(RuntimeProviderError::Unavailable)?;
        Ok(ProjectedSecurityState {
            schema_version: 1,
            installation_id: security.installation_id,
            module_instance_id: security.module_instance_id,
            authorization_revision: security.authorization_revision,
            organization_revision: security.organization_revision,
            enabled: security.enabled,
            document_state: security.document_state.clone(),
        })
    }

    async fn apply_security_state(
        &self,
        projected: ProjectedSecurityState,
    ) -> Result<ProjectedSecurityState, RuntimeProviderError> {
        if projected.schema_version != 1 {
            return Err(RuntimeProviderError::Invalid);
        }
        let mut state = self.state.read().await.clone();
        state.security = Some(ReferenceSecurityState {
            installation_id: projected.installation_id,
            module_instance_id: projected.module_instance_id,
            authorization_revision: projected.authorization_revision,
            organization_revision: projected.organization_revision,
            enabled: projected.enabled,
            document_state: projected.document_state.clone(),
        });
        self.save(state).await?;
        Ok(projected)
    }
}

#[async_trait]
impl ReadinessProvider for ReferenceRuntime {
    async fn readiness_checks(&self) -> Vec<RuntimeCheck> {
        let state = self.state.read().await;
        vec![
            RuntimeCheck {
                code: "configuration".into(),
                passing: !state.configuration.display_label.is_empty(),
                message: "configuration is valid".into(),
            },
            RuntimeCheck {
                code: "security_state".into(),
                passing: state
                    .security
                    .as_ref()
                    .is_some_and(|security| security.enabled),
                message: "current enabled security projection is required".into(),
            },
        ]
    }
}

#[async_trait]
impl DiagnosticsProvider for ReferenceRuntime {
    async fn diagnostic_facts(&self) -> BTreeMap<String, String> {
        BTreeMap::from([
            ("definition".into(), DEFINITION_ID.into()),
            ("release".into(), RELEASE_VERSION.into()),
            ("state_schema".into(), "1".into()),
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

pub fn router(runtime: Arc<ReferenceRuntime>) -> Router {
    Router::new()
        .route(ROOT_PATH, get(document))
        .route(
            "/reference/module-sdk/scopes/{organization_id}",
            get(scoped_probe),
        )
        .route(
            "/reference/module-sdk/diagnostics",
            get(diagnostics_document),
        )
        .route(MODULE_SHELL_CSS_PATH, get(stylesheet))
        .route(MODULE_SHELL_JS_PATH, get(hydration_script))
        .merge(tessara_module_runtime::standard_control_router::<
            ReferenceRuntime,
        >())
        .merge(tessara_module_runtime::standard_probe_router::<
            ReferenceRuntime,
        >())
        .with_state(runtime)
}

async fn document(State(runtime): State<Arc<ReferenceRuntime>>, headers: HeaderMap) -> Response {
    let Ok((presentation, state)) = verified_document(
        &runtime,
        &headers,
        "reference.module-sdk.read",
        ROOT_PATH,
        "Module SDK Reference",
    )
    .await
    else {
        return (StatusCode::FORBIDDEN, "module action unavailable").into_response();
    };
    Html(render_reference_document(
        &presentation,
        &state.configuration,
    ))
    .into_response()
}

async fn verified_document(
    runtime: &ReferenceRuntime,
    headers: &HeaderMap,
    action: &str,
    path: &str,
    title: &str,
) -> Result<(ShellPresentation, ReferenceState), ()> {
    verify_authorization(runtime, headers, action, None).await?;
    let correlation_id = request_correlation_id(headers).map_err(|_| ())?;
    let envelope =
        decode_signed_envelope_header(headers, "x-tessara-shell-context").map_err(|_| ())?;
    let state = runtime.state.read().await.clone();
    let Some(ref security) = state.security else {
        return Err(());
    };
    if verify_shell_context(
        &envelope,
        &runtime.verifiers.shell,
        &ShellContextValidationContextV1 {
            installation_id: security.installation_id,
            module_definition_id: envelope.payload.module_definition_id.clone(),
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: chrono::Utc::now(),
        },
    )
    .is_err()
        || envelope.payload.module_definition_id.as_str() != DEFINITION_ID
    {
        return Err(());
    }
    Ok((
        ShellPresentation::from_verified_context(&envelope.payload, path, title),
        state,
    ))
}

async fn diagnostics_document(
    State(runtime): State<Arc<ReferenceRuntime>>,
    headers: HeaderMap,
) -> Response {
    let Ok((presentation, _)) = verified_document(
        &runtime,
        &headers,
        "reference.module-sdk.diagnostics.read",
        "/reference/module-sdk/diagnostics",
        "Module SDK diagnostics",
    )
    .await
    else {
        return (StatusCode::FORBIDDEN, "module action unavailable").into_response();
    };
    let findings = runtime.diagnostic_findings().await;
    let body = format!(
        "<section aria-labelledby=\"diagnostics-title\"><h1 id=\"diagnostics-title\">Module SDK diagnostics</h1><p>Release {} · health {}</p><p>{} active finding(s). No credentials or policy payloads are exposed.</p><p><a href=\"{}\">Return to reference module</a></p></section>",
        RELEASE_VERSION,
        if findings.is_empty() {
            "passing"
        } else {
            "failing"
        },
        findings.len(),
        ROOT_PATH,
    );
    Html(tessara_module_ui::render_module_document(
        &presentation,
        MODULE_SHELL_CSS_PATH,
        Some(MODULE_SHELL_JS_PATH),
        &body,
    ))
    .into_response()
}

#[cfg(test)]
mod tests {
    use axum::{
        body::{Body, to_bytes},
        http::Request,
    };
    use tessara_module_contract::{ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1};
    use tower::ServiceExt;

    use super::*;

    fn test_verifiers() -> CoreVerifiers {
        let authorization = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "reference-test",
            ProtocolSignaturePurposeV1::AuthorizationGrant,
            [31; 32],
        )
        .unwrap()
        .verifier();
        let shell = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "reference-test",
            ProtocolSignaturePurposeV1::ShellContext,
            [31; 32],
        )
        .unwrap()
        .verifier();
        CoreVerifiers {
            authorization,
            shell,
        }
    }

    async fn request(
        app: &Router,
        method: &str,
        path: &str,
        body: Value,
        controlled: bool,
    ) -> Response {
        let mut builder = Request::builder()
            .method(method)
            .uri(path)
            .header(header::CONTENT_TYPE, "application/json");
        if controlled {
            builder = builder.header(
                "x-tessara-module-control-key",
                "development-module-control-only",
            );
        }
        app.clone()
            .oneshot(builder.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn standard_control_probes_persistence_and_assets_are_conformant() {
        let path = std::env::temp_dir().join(format!(
            "tessara-reference-module-sdk-{}.json",
            Uuid::new_v4()
        ));
        let runtime = Arc::new(
            ReferenceRuntime::open(path.clone(), test_verifiers())
                .await
                .unwrap(),
        );
        let app = router(runtime);

        let denied = request(&app, "GET", "/api/configuration", json!({}), false).await;
        assert_eq!(denied.status(), StatusCode::UNAUTHORIZED);

        let applied = request(
            &app,
            "PUT",
            "/api/configuration",
            json!({"display_label":"  Conformance  "}),
            true,
        )
        .await;
        assert_eq!(applied.status(), StatusCode::OK);
        let applied: Value =
            serde_json::from_slice(&to_bytes(applied.into_body(), 64 * 1024).await.unwrap())
                .unwrap();
        assert_eq!(applied["normalized"]["display_label"], "Conformance");

        let not_ready = request(&app, "GET", "/health/ready", json!({}), false).await;
        assert_eq!(not_ready.status(), StatusCode::SERVICE_UNAVAILABLE);
        let security = request(
            &app,
            "PUT",
            "/api/private/security-state",
            json!({
                "schema_version":1,
                "installation_id":Uuid::from_u128(1),
                "module_instance_id":Uuid::from_u128(2),
                "authorization_revision":1,
                "organization_revision":1,
                "enabled":true,
                "document_state":"enabled"
            }),
            true,
        )
        .await;
        assert_eq!(security.status(), StatusCode::OK);
        let ready = request(&app, "GET", "/health/ready", json!({}), false).await;
        assert_eq!(ready.status(), StatusCode::OK);

        let asset = request(&app, "GET", MODULE_SHELL_CSS_PATH, json!({}), false).await;
        assert_eq!(asset.status(), StatusCode::OK);
        assert_eq!(
            asset.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );
        let script = request(&app, "GET", MODULE_SHELL_JS_PATH, json!({}), false).await;
        assert_eq!(script.status(), StatusCode::OK);
        assert_eq!(
            script.headers()[header::CACHE_CONTROL],
            "public, max-age=31536000, immutable"
        );

        let diagnostics = request(&app, "GET", "/api/diagnostics", json!({}), true).await;
        assert_eq!(diagnostics.status(), StatusCode::OK);
        let diagnostics = String::from_utf8(
            to_bytes(diagnostics.into_body(), 64 * 1024)
                .await
                .unwrap()
                .to_vec(),
        )
        .unwrap();
        assert!(!diagnostics.contains(path.to_string_lossy().as_ref()));
        assert!(!diagnostics.contains("development-module-control-only"));

        tokio::fs::remove_file(path).await.unwrap();
    }
}

async fn scoped_probe(
    State(runtime): State<Arc<ReferenceRuntime>>,
    Path(organization_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    if verify_authorization(
        &runtime,
        &headers,
        "reference.module-sdk.scope.read",
        Some(organization_id),
    )
    .await
    .is_err()
    {
        return (StatusCode::FORBIDDEN, "module action unavailable").into_response();
    }
    Json(json!({
        "schema_version": 1,
        "organization_id": organization_id,
        "status": "authorized"
    }))
    .into_response()
}

async fn verify_authorization(
    runtime: &ReferenceRuntime,
    headers: &HeaderMap,
    action: &str,
    organization_id: Option<Uuid>,
) -> Result<(), ()> {
    let envelope: tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV1> =
        decode_signed_envelope_header(headers, "x-tessara-authorization").map_err(|_| ())?;
    runtime
        .verifiers
        .authorization
        .verify(&envelope)
        .map_err(|_| ())?;
    let security = runtime.current_security_state().await.map_err(|_| ())?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV1 {
            installation_id: security.installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core").map_err(|_| ())?,
            audience_module_instance_id: security.module_instance_id,
            dependency_binding: DependencyBindingKey::new("tessara.core.module-document")
                .map_err(|_| ())?,
            functional_contract: FunctionalContractId::new(
                "tessara.reference.module-sdk.conformance",
            )
            .map_err(|_| ())?,
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            authorization_revision: security.authorization_revision,
            organization_revision: security.organization_revision,
            now: chrono::Utc::now(),
        })
        .map_err(|_| ())?;
    let capability = SecurityCapabilityId::new(READ_CAPABILITY).map_err(|_| ())?;
    if !envelope
        .payload
        .capability_scope_bindings
        .iter()
        .any(|binding| binding.capability == capability)
    {
        return Err(());
    }
    if let Some(organization_id) = organization_id
        && !envelope.payload.authorizes(&capability, organization_id)
    {
        return Err(());
    }
    Ok(())
}

async fn stylesheet() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/css; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        Body::from(MODULE_SHELL_CSS),
    )
        .into_response()
}

async fn hydration_script() -> Response {
    (
        [
            (header::CONTENT_TYPE, "text/javascript; charset=utf-8"),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        Body::from(tessara_module_ui::MODULE_SHELL_JS),
    )
        .into_response()
}
