//! Manifest-driven same-origin gateway for independently deployed modules.
//!
//! Core terminates browser credentials, resolves the installed manifest and
//! service registration, projects current control state, and forwards only
//! short-lived signed authority plus safe request metadata.

use std::collections::BTreeMap;

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde_json::{Value, json};
use sqlx::Row;
use tessara_module_contract::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V2, AuthorizationGrantOperationV1, AuthorizationGrantV2,
    BrowserLifecycleBootstrapV1, CapabilityScopeBindingV1, DependencyBindingKey, DeploymentProfile,
    FunctionalContractId, ModuleDefinitionId, ModuleManifest, NavigationContributionId,
    NavigationProjectionV1, OriginalActorProjectionV1, ProtocolSignaturePurposeV1,
    PublicApiIdempotency, PublicApiMethod, SecurityCapabilityId, ShellContextV1,
    ShellDocumentStateV1, ShellThemeV1,
};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedRequest,
    core_security::{capability_bindings, protocol_signer},
    db::AppState,
    error::{ApiError, ApiResult},
};

struct InstalledModule {
    instance_id: Uuid,
    installation_id: Uuid,
    manifest: ModuleManifest,
    serving: bool,
}

pub(crate) async fn dispatch(
    State(state): State<AppState>,
    actor: AuthenticatedRequest,
    request: Request,
) -> Response {
    let path = request.uri().path().to_string();
    dispatch_result(&state, &actor, request)
        .await
        .unwrap_or_else(|error| {
            tracing::warn!(%error, %path, "Generic module gateway request failed");
            error.into_response()
        })
}

async fn dispatch_result(
    state: &AppState,
    actor: &AuthenticatedRequest,
    request: Request,
) -> ApiResult<Response> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let installed = installed_modules(&state.pool).await?;

    for module in installed {
        if matches!(method, Method::GET | Method::HEAD)
            && let Some(route) = module.manifest.browser_routes.iter().find(|route| {
                route.methods.iter().any(|declared| {
                    matches!(
                        (declared, &method),
                        (
                            tessara_module_contract::BrowserDocumentMethod::Get,
                            &Method::GET
                        ) | (
                            tessara_module_contract::BrowserDocumentMethod::Head,
                            &Method::HEAD
                        )
                    )
                }) && path_template_matches(&route.path_template, &path)
            })
        {
            if !module.serving {
                return Ok(crate::module_unavailable_fallback_response());
            }
            let grant = module_authorization(
                state,
                actor,
                &module,
                AuthorizationRequest {
                    action: &route.authorization_action,
                    dependency_binding: &route.dependency_binding,
                    operation: AuthorizationGrantOperationV1::Read,
                    required_capability: &route.required_capability,
                    contract: &route.functional_contract,
                },
            )
            .await?;
            let shell = shell_context(actor, &module, &path)?;
            return forward(
                &module,
                ForwardRequest {
                    method,
                    path: &path,
                    inbound_headers: request.headers(),
                    body: Bytes::new(),
                    grant: Some(&grant),
                    shell: Some(&shell),
                    idempotent: false,
                },
            )
            .await;
        }

        if module.serving
            && let Some(route) = module.manifest.public_api_routes.iter().find(|route| {
                api_method_matches(route.method, &method)
                    && path_template_matches(&route.path_template, &path)
            })
        {
            let grant = module_authorization(
                state,
                actor,
                &module,
                AuthorizationRequest {
                    action: &route.authorization_action,
                    dependency_binding: &route.dependency_binding,
                    operation: route.operation,
                    required_capability: &route.required_capability,
                    contract: &route.functional_contract,
                },
            )
            .await?;
            let (parts, body) = request.into_parts();
            let bytes = to_bytes(body, 2 * 1024 * 1024)
                .await
                .map_err(|_| ApiError::BadRequest("module request body is too large".into()))?;
            return forward(
                &module,
                ForwardRequest {
                    method,
                    path: &path,
                    inbound_headers: &parts.headers,
                    body: bytes,
                    grant: Some(&grant),
                    shell: None,
                    idempotent: route.idempotency == PublicApiIdempotency::ForwardOrGenerateHeader,
                },
            )
            .await;
        }
    }
    Err(ApiError::NotFound("route not found".into()))
}

pub(crate) async fn asset(State(state): State<AppState>, request: Request) -> Response {
    let path = request.uri().path().to_string();
    match installed_modules(&state.pool).await {
        Ok(installed) => {
            for module in installed {
                if module.serving && asset_path_targets_module(&path, &module.manifest) {
                    return forward(
                        &module,
                        ForwardRequest {
                            method: Method::GET,
                            path: &path,
                            inbound_headers: request.headers(),
                            body: Bytes::new(),
                            grant: None,
                            shell: None,
                            idempotent: false,
                        },
                    )
                    .await
                    .unwrap_or_else(|error| error.into_response());
                }
            }
            StatusCode::NOT_FOUND.into_response()
        }
        Err(error) => error.into_response(),
    }
}

async fn installed_modules(pool: &sqlx::PgPool) -> ApiResult<Vec<InstalledModule>> {
    let rows = sqlx::query(
        "SELECT instances.id,instances.installation_id,releases.manifest,
                instances.deployed,instances.configured,instances.enabled,
                instances.ready,instances.healthy
         FROM module_instances instances
         JOIN module_releases releases ON releases.id=instances.release_id
         WHERE instances.identity_state='live' AND instances.installed
           AND releases.manifest IS NOT NULL
         ORDER BY instances.definition_id",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(InstalledModule {
                instance_id: row.try_get("id")?,
                installation_id: row.try_get("installation_id")?,
                serving: row.try_get::<bool, _>("deployed")?
                    && row.try_get::<bool, _>("configured")?
                    && row.try_get::<bool, _>("enabled")?
                    && row.try_get::<bool, _>("ready")?
                    && row.try_get::<bool, _>("healthy")?,
                manifest: row
                    .try_get::<sqlx::types::Json<ModuleManifest>, _>("manifest")?
                    .0,
            })
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ApiError::from)
}

struct AuthorizationRequest<'a> {
    action: &'a str,
    dependency_binding: &'a DependencyBindingKey,
    operation: AuthorizationGrantOperationV1,
    required_capability: &'a SecurityCapabilityId,
    contract: &'a FunctionalContractId,
}

async fn module_authorization(
    state: &AppState,
    actor: &AuthenticatedRequest,
    module: &InstalledModule,
    request: AuthorizationRequest<'_>,
) -> ApiResult<tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV2>> {
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let authorization_revision: i64 = revisions.try_get("authorization_revision")?;
    let organization_revision: i64 = revisions.try_get("organization_revision")?;
    sync_control_projections(state, module, authorization_revision, organization_revision).await?;

    let mut bindings = Vec::new();
    for capability in &module.manifest.security_capabilities {
        bindings.extend(
            capability_bindings(
                &state.pool,
                actor.account.account_id,
                capability.id.as_str(),
            )
            .await?,
        );
        if has_global_capability(
            &state.pool,
            actor.account.account_id,
            capability.id.as_str(),
        )
        .await?
        {
            bindings.push(CapabilityScopeBindingV1 {
                capability: capability.id.clone(),
                organization_root_id: module.installation_id,
                authorized_organization_ids: Vec::new(),
            });
        }
    }
    if !bindings
        .iter()
        .any(|binding| binding.capability == *request.required_capability)
    {
        tracing::warn!(
            actor_id = %actor.account.account_id,
            required_capability = request.required_capability.as_str(),
            binding_count = bindings.len(),
            "Generic module action has no authorized capability binding"
        );
        return Err(ApiError::Forbidden("module action unavailable".into()));
    }

    let now = Utc::now();
    let grant = AuthorizationGrantV2 {
        schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
        installation_id: module.installation_id,
        original_actor_id: actor.account.account_id,
        presenting_service: ModuleDefinitionId::new("tessara.core")
            .map_err(|error| ApiError::Internal(error.into()))?,
        audience_module_instance_id: module.instance_id,
        dependency_binding: request.dependency_binding.clone(),
        functional_contract: request.contract.clone(),
        action: request.action.into(),
        operation: request.operation,
        capability_scope_bindings: bindings,
        resource_assertion: None,
        delegation_basis: Vec::new(),
        authorization_revision: authorization_revision as u64,
        organization_revision: organization_revision as u64,
        jti: Uuid::new_v4(),
        issued_at: now,
        expires_at: now
            + Duration::seconds(
                if request.operation == AuthorizationGrantOperationV1::Read {
                    60
                } else {
                    30
                },
            ),
    };
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(grant)
        .map_err(|error| ApiError::Internal(error.into()))
}

fn shell_context(
    actor: &AuthenticatedRequest,
    module: &InstalledModule,
    path: &str,
) -> ApiResult<tessara_module_contract::SignedEnvelopeV1<ShellContextV1>> {
    let now = Utc::now();
    let mut navigation = vec![NavigationProjectionV1 {
        contribution_id: NavigationContributionId::new("tessara.core.home")
            .map_err(|error| ApiError::Internal(error.into()))?,
        label: "Home".into(),
        href: "/".into(),
    }];
    for contribution in &module.manifest.navigation {
        if let Some(route) = module
            .manifest
            .browser_routes
            .iter()
            .find(|route| route.destination == contribution.destination)
        {
            navigation.push(NavigationProjectionV1 {
                contribution_id: contribution.id.clone(),
                label: contribution.label.clone(),
                href: route.path_template.clone(),
            });
        }
    }
    let context = ShellContextV1 {
        schema_version: 1,
        installation_id: module.installation_id,
        module_definition_id: module.manifest.definition_id.clone(),
        module_instance_id: module.instance_id,
        original_actor: OriginalActorProjectionV1 {
            actor_id: actor.account.account_id,
            display_name: actor.account.display_name.clone(),
            email: Some(actor.account.email.clone()),
        },
        theme: ShellThemeV1::Dark,
        navigation,
        return_destination: "/".into(),
        locale: "en-US".into(),
        time_zone: "UTC".into(),
        correlation_id: Uuid::new_v4(),
        document_state: ShellDocumentStateV1::Active,
        issued_at: now,
        expires_at: now + Duration::seconds(60),
    };
    let _ = path;
    protocol_signer(ProtocolSignaturePurposeV1::ShellContext)?
        .sign(context)
        .map_err(|error| ApiError::Internal(error.into()))
}

async fn sync_control_projections(
    state: &AppState,
    module: &InstalledModule,
    authorization_revision: i64,
    organization_revision: i64,
) -> ApiResult<()> {
    let endpoint = service_endpoint(&module.manifest)?;
    let control_key = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let client = reqwest::Client::new();
    for projection in &module.manifest.control_projections {
        let payload = match projection.kind {
            tessara_module_contract::ControlProjectionKind::SecurityState => json!({
                "schema_version":1,
                "installation_id":module.installation_id,
                "module_instance_id":module.instance_id,
                "authorization_revision":authorization_revision,
                "organization_revision":organization_revision,
                "enabled":true,
                "document_state":"enabled"
            }),
            tessara_module_contract::ControlProjectionKind::Organization => {
                json!({
                    "schema_version":1,
                    "organization_revision":organization_revision,
                    "nodes":organization_projection(&state.pool).await?
                })
            }
        };
        client
            .put(format!("{}{}", endpoint, projection.path))
            .header("x-tessara-module-control-key", &control_key)
            .json(&payload)
            .send()
            .await
            .map_err(|_| module_unavailable())?
            .error_for_status()
            .map_err(|_| module_unavailable())?;
    }
    Ok(())
}

async fn organization_projection(pool: &sqlx::PgPool) -> ApiResult<Vec<Value>> {
    let rows = sqlx::query(
        "WITH RECURSIVE organization AS (
           SELECT n.id,n.name,n.node_type_id,n.parent_node_id,n.name::text AS node_path
           FROM nodes n WHERE n.parent_node_id IS NULL
           UNION ALL
           SELECT child.id,child.name,child.node_type_id,child.parent_node_id,
                  parent.node_path || ' / ' || child.name
           FROM nodes child JOIN organization parent ON child.parent_node_id=parent.id
         )
         SELECT organization.id,organization.name,node_types.name AS node_type_name,
                organization.parent_node_id,organization.node_path
         FROM organization JOIN node_types ON node_types.id=organization.node_type_id
         ORDER BY organization.node_path,organization.id",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            Ok(json!({
                "node_id":row.try_get::<Uuid,_>("id")?,
                "node_name":row.try_get::<String,_>("name")?,
                "node_type_name":row.try_get::<String,_>("node_type_name")?,
                "parent_node_id":row.try_get::<Option<Uuid>,_>("parent_node_id")?,
                "node_path":row.try_get::<String,_>("node_path")?
            }))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()
        .map_err(ApiError::from)
}

struct ForwardRequest<'a> {
    method: Method,
    path: &'a str,
    inbound_headers: &'a HeaderMap,
    body: Bytes,
    grant: Option<&'a tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV2>>,
    shell: Option<&'a tessara_module_contract::SignedEnvelopeV1<ShellContextV1>>,
    idempotent: bool,
}

async fn forward(module: &InstalledModule, request: ForwardRequest<'_>) -> ApiResult<Response> {
    let endpoint = service_endpoint(&module.manifest)?;
    let client = reqwest::Client::new();
    let mut outbound = client.request(
        reqwest::Method::from_bytes(request.method.as_str().as_bytes())
            .map_err(|error| ApiError::Internal(error.into()))?,
        format!("{endpoint}{}", request.path),
    );
    if let Some(grant) = request.grant {
        outbound = outbound.header(
            "x-tessara-authorization",
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(grant).map_err(|error| ApiError::Internal(error.into()))?,
            ),
        );
    }
    if let Some(shell) = request.shell {
        outbound = outbound
            .header(
                "x-tessara-shell-context",
                URL_SAFE_NO_PAD.encode(
                    serde_json::to_vec(shell).map_err(|error| ApiError::Internal(error.into()))?,
                ),
            )
            .header(
                "x-tessara-correlation-id",
                shell.payload.correlation_id.to_string(),
            );
    }
    if let Some(content_type) = request
        .inbound_headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
    {
        outbound = outbound.header(reqwest::header::CONTENT_TYPE, content_type);
    }
    if let Some(accept) = request
        .inbound_headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
    {
        outbound = outbound.header(reqwest::header::ACCEPT, accept);
    }
    if request.idempotent {
        outbound = outbound.header(
            "x-idempotency-key",
            idempotency_key(request.inbound_headers),
        );
    }
    if !request.body.is_empty() {
        outbound = outbound.body(request.body.to_vec());
    }
    let response = outbound.send().await.map_err(|_| module_unavailable())?;
    module_response(response, &module.manifest, request.path).await
}

async fn module_response(
    response: reqwest::Response,
    manifest: &ModuleManifest,
    requested_path: &str,
) -> ApiResult<Response> {
    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|error| ApiError::Internal(error.into()))?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let cache_control = response
        .headers()
        .get(reqwest::header::CACHE_CONTROL)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let vary = response
        .headers()
        .get(reqwest::header::VARY)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = response
        .bytes()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    if content_type.starts_with("application/vnd.tessara.module-view+json") {
        let bootstrap = serde_json::from_slice::<BrowserLifecycleBootstrapV1>(&bytes)
            .map_err(|_| incompatible_lifecycle_response())?;
        if !lifecycle_response_matches_manifest(&bootstrap, manifest, requested_path) {
            return Err(incompatible_lifecycle_response());
        }
    }
    let mut builder = Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type);
    if let Some(cache_control) = cache_control {
        builder = builder.header(header::CACHE_CONTROL, cache_control);
    }
    if let Some(vary) = vary {
        builder = builder.header(header::VARY, vary);
    }
    builder
        .body(Body::from(bytes))
        .map_err(|error| ApiError::Internal(error.into()))
}

fn lifecycle_response_matches_manifest(
    bootstrap: &BrowserLifecycleBootstrapV1,
    manifest: &ModuleManifest,
    requested_path: &str,
) -> bool {
    let Some(lifecycle) = &manifest.browser_lifecycle else {
        return false;
    };
    if !bootstrap.is_supported()
        || bootstrap.definition_id != manifest.definition_id
        || bootstrap.release_version != manifest.release_version
        || bootstrap.lifecycle_abi != lifecycle.lifecycle_abi
        || bootstrap.path != requested_path
        || !manifest.browser_routes.iter().any(|route| {
            route.destination == bootstrap.destination
                && path_template_matches(&route.path_template, requested_path)
        })
    {
        return false;
    }
    let asset_matches = |projected: &tessara_module_contract::BrowserLifecycleAssetV1,
                         declared_path: &str| {
        manifest.assets.iter().any(|asset| {
            asset.path == declared_path
                && asset.digest == projected.digest
                && asset.content_type == projected.content_type
                && projected.url.ends_with(declared_path)
        })
    };
    asset_matches(&bootstrap.entry_asset, &lifecycle.entry_asset)
        && bootstrap.stylesheet_assets.len() == lifecycle.stylesheet_assets.len()
        && bootstrap
            .stylesheet_assets
            .iter()
            .zip(&lifecycle.stylesheet_assets)
            .all(|(projected, declared)| asset_matches(projected, declared))
}

fn incompatible_lifecycle_response() -> ApiError {
    ApiError::ServiceUnavailable("module lifecycle response is incompatible".into())
}

fn service_endpoint(manifest: &ModuleManifest) -> ApiResult<String> {
    let DeploymentProfile::TessaraOciV1(deployment) = &manifest.deployment;
    let configured = std::env::var("TESSARA_MODULE_SERVICE_ENDPOINTS")
        .ok()
        .and_then(|value| serde_json::from_str::<BTreeMap<String, String>>(&value).ok())
        .and_then(|map| map.get(&deployment.listen.registration_name).cloned());
    Ok(configured
        .unwrap_or_else(|| {
            format!(
                "http://{}:{}",
                deployment.listen.registration_name, deployment.listen.port
            )
        })
        .trim_end_matches('/')
        .to_string())
}

fn asset_path_targets_module(path: &str, manifest: &ModuleManifest) -> bool {
    path.strip_prefix("/_tessara/modules/")
        .and_then(|path| path.split_once('/'))
        .is_some_and(|(definition_id, remaining)| {
            definition_id == manifest.definition_id.as_str() && !remaining.is_empty()
        })
}

fn path_template_matches(template: &str, path: &str) -> bool {
    let template = template.trim_matches('/').split('/').collect::<Vec<_>>();
    let path = path.trim_matches('/').split('/').collect::<Vec<_>>();
    template.len() == path.len()
        && template.iter().zip(path).all(|(template, value)| {
            template == &value
                || (template.starts_with('{')
                    && template.ends_with('}')
                    && !value.is_empty()
                    && !value.contains(['.', '/']))
        })
}

fn api_method_matches(declared: PublicApiMethod, actual: &Method) -> bool {
    matches!(
        (declared, actual),
        (PublicApiMethod::Get, &Method::GET)
            | (PublicApiMethod::Post, &Method::POST)
            | (PublicApiMethod::Put, &Method::PUT)
            | (PublicApiMethod::Delete, &Method::DELETE)
    )
}

fn idempotency_key(headers: &HeaderMap) -> String {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

async fn has_global_capability(
    pool: &sqlx::PgPool,
    account_id: Uuid,
    capability: &str,
) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(
           SELECT 1 FROM role_assignments ra
           JOIN role_capabilities rc ON rc.role_id=ra.role_id
           JOIN capabilities c ON c.id=rc.capability_id
           WHERE ra.account_id=$1 AND ra.node_id IS NULL
             AND (c.key=$2 OR c.key='admin:all')
         )",
    )
    .bind(account_id)
    .bind(capability)
    .fetch_one(pool)
    .await?)
}

fn module_unavailable() -> ApiError {
    ApiError::ServiceUnavailable("module temporarily unavailable".into())
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    #[test]
    fn manifest_path_matching_distinguishes_static_and_parameter_segments() {
        assert!(path_template_matches(
            "/dashboards/{dashboard_id}/view",
            "/dashboards/1d812771-4bd5-4344-81e1-b32b017061c9/view"
        ));
        assert!(!path_template_matches(
            "/dashboards/{dashboard_id}/view",
            "/dashboards/new"
        ));
    }

    #[test]
    fn mutation_idempotency_is_header_preserving_or_generated() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-idempotency-key",
            HeaderValue::from_static("dashboard-save-42"),
        );
        assert_eq!(idempotency_key(&headers), "dashboard-save-42");
        assert!(!idempotency_key(&HeaderMap::new()).is_empty());
    }

    #[test]
    fn candidate_assets_route_to_the_installed_module_identity() {
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-dashboard-module/manifest.json"))
                .expect("Dashboard manifest");
        assert!(asset_path_targets_module(
            "/_tessara/modules/tessara.dashboards/2.0.2/sha256:candidate/dashboard.js",
            &manifest
        ));
        assert!(!asset_path_targets_module(
            "/_tessara/modules/tessara.other/2.0.2/sha256:candidate/dashboard.js",
            &manifest
        ));
    }

    #[test]
    fn lifecycle_bootstrap_must_match_the_installed_manifest_and_route() {
        use tessara_module_contract::{BrowserLifecycleAssetV1, ShellDocumentStateV1};

        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-dashboard-module/manifest.json"))
                .expect("Dashboard manifest");
        let lifecycle = manifest.browser_lifecycle.as_ref().unwrap();
        let projected = |path: &str| {
            let asset = manifest
                .assets
                .iter()
                .find(|asset| asset.path == path)
                .unwrap();
            BrowserLifecycleAssetV1 {
                url: format!(
                    "/_tessara/modules/{}/{}/{}/{}",
                    manifest.definition_id,
                    manifest.release_version,
                    asset.digest,
                    path.trim_start_matches('/')
                ),
                digest: asset.digest.clone(),
                content_type: asset.content_type.clone(),
            }
        };
        let mut bootstrap = BrowserLifecycleBootstrapV1 {
            schema_version: 1,
            definition_id: manifest.definition_id.clone(),
            release_version: manifest.release_version.clone(),
            lifecycle_abi: lifecycle.lifecycle_abi.clone(),
            destination: tessara_module_contract::SemanticRouteName::new("dashboards.directory")
                .unwrap(),
            path: "/dashboards".into(),
            title: "Dashboards".into(),
            document_state: ShellDocumentStateV1::Active,
            entry_asset: projected(&lifecycle.entry_asset),
            stylesheet_assets: lifecycle
                .stylesheet_assets
                .iter()
                .map(|path| projected(path))
                .collect(),
            payload: json!({"route":"directory"}),
        };
        assert!(lifecycle_response_matches_manifest(
            &bootstrap,
            &manifest,
            "/dashboards"
        ));

        bootstrap.release_version = "9.0.0".parse().unwrap();
        assert!(!lifecycle_response_matches_manifest(
            &bootstrap,
            &manifest,
            "/dashboards"
        ));
    }
}
