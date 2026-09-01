//! Manifest-driven same-origin gateway for independently deployed modules.
//!
//! Core terminates browser credentials, resolves the installed manifest and
//! service registration, projects current control state, and forwards only
//! short-lived signed authority plus safe request metadata.

use std::{
    cmp::Reverse,
    collections::{BTreeMap, BTreeSet},
};

use axum::{
    body::{Body, Bytes, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, Method, StatusCode, header},
    response::{IntoResponse, Response},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_module_contract::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, AuthorizationAudienceV1, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, BrowserLifecycleBootstrapV1, CapabilityScopeBindingV1,
    CoreServiceRequestV1, DelegationBasisV1, DependencyBindingKey, DeploymentProfile,
    FunctionalContractId, ModuleManifest, ModuleServicePrincipalV1, OriginalActorProjectionV1,
    ProtocolSignaturePurposeV1, PublicApiIdempotency, PublicApiMethod,
    SHELL_CONTEXT_SCHEMA_VERSION_V2, SecurityCapabilityId, ServiceActionMethod, ShellContextV2,
    ShellDocumentStateV1, ShellNavigationGroupProjectionV2, ShellThemeV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedRequest,
    core_security::{capability_bindings, protocol_signer, request_correlation_id_or_new},
    db::AppState,
    error::{ApiError, ApiResult},
};

struct InstalledModule {
    instance_id: Uuid,
    installation_id: Uuid,
    manifest: ModuleManifest,
    serving: bool,
    // Recovery jobs must remain able to repair readiness/health-derived lag.
    system_job_reachable: bool,
}

const CORE_PRIVATE_PROVIDER_RESPONSE_LIMIT_BYTES: usize = 1024 * 1024;
const CORE_PRIVATE_PROVIDER_MEDIA_TYPE: &str = "application/json";
const RESOURCE_OBSERVATION_PROVIDER_CONTRACT_VERSION: &str = "1.0.0";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CorePrivateProviderOwner {
    pub(crate) installation_id: Uuid,
    pub(crate) module_instance_id: Uuid,
}

pub(crate) struct CorePrivateProviderRequest<'a, T> {
    pub(crate) module_definition_id: &'a str,
    /// When present, a generic resource call must reach this exact owner rather
    /// than any installed instance of the same module definition.
    pub(crate) expected_owner: Option<CorePrivateProviderOwner>,
    pub(crate) dependency_binding: &'a str,
    pub(crate) functional_contract: &'a str,
    pub(crate) contract_version: &'a str,
    pub(crate) authorization_action: &'a str,
    pub(crate) path: &'a str,
    pub(crate) media_type: &'a str,
    pub(crate) correlation_id: Uuid,
    /// Core-owned capability whose effective scope is delegated to this exact
    /// provider action (for example `operations:view` or `admin:all`).
    pub(crate) actor_capability: &'a str,
    pub(crate) body: &'a T,
}

/// One explicitly authorized Core-owned system job calling a manifest-declared
/// private provider action. System jobs use the same signed CoreGateway wire
/// boundary as actor-mediated calls, but their least-privilege authority is
/// declared at the call site instead of being borrowed from a browser account.
pub(crate) struct CoreSystemJobProviderRequest<'a, T> {
    pub(crate) system_job_id: &'a str,
    pub(crate) module_definition_id: &'a str,
    pub(crate) expected_owner: Option<CorePrivateProviderOwner>,
    pub(crate) dependency_binding: &'a str,
    pub(crate) functional_contract: &'a str,
    pub(crate) contract_version: &'a str,
    pub(crate) authorization_action: &'a str,
    pub(crate) path: &'a str,
    pub(crate) media_type: &'a str,
    pub(crate) correlation_id: Uuid,
    pub(crate) body: &'a T,
}

#[derive(Clone, Copy)]
struct PrivateProviderTarget<'a> {
    dependency_binding: &'a str,
    authorization_action: &'a str,
    path: &'a str,
    media_type: &'a str,
    correlation_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResourceObservationProviderRoute {
    owner: CorePrivateProviderOwner,
    module_definition_id: String,
    dependency_binding: String,
    functional_contract: String,
    contract_version: String,
    authorization_action: String,
    path: String,
    actor_capability: String,
}

impl ResourceObservationProviderRoute {
    pub(crate) fn actor_capability(&self) -> &str {
        &self.actor_capability
    }

    pub(crate) fn private_request<'a, T>(
        &'a self,
        correlation_id: Uuid,
        body: &'a T,
    ) -> CorePrivateProviderRequest<'a, T> {
        CorePrivateProviderRequest {
            module_definition_id: &self.module_definition_id,
            expected_owner: Some(self.owner),
            dependency_binding: &self.dependency_binding,
            functional_contract: &self.functional_contract,
            contract_version: &self.contract_version,
            authorization_action: &self.authorization_action,
            path: &self.path,
            media_type: CORE_PRIVATE_PROVIDER_MEDIA_TYPE,
            correlation_id,
            actor_capability: &self.actor_capability,
            body,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ResourceObservationProviderLookup {
    Registered(ResourceObservationProviderRoute),
    OwnerUnavailable,
    ReferenceUnsupported,
    ContractUnavailable,
}

/// Resolves a module-owned typed reference to its manifest-declared generic
/// resource-observation provider. The platform recognizes only the policy-
/// neutral `resource-observation` v1 service convention; product definition,
/// action, route, capability, and Core binding identities all come from the
/// exact installed manifest.
pub(crate) async fn resource_observation_provider(
    pool: &sqlx::PgPool,
    reference: &TypedResourceReference,
) -> ApiResult<ResourceObservationProviderLookup> {
    let installed = installed_modules(pool).await?;
    Ok(resource_observation_provider_from_installed(
        &installed, reference,
    ))
}

fn resource_observation_provider_from_installed(
    installed: &[InstalledModule],
    reference: &TypedResourceReference,
) -> ResourceObservationProviderLookup {
    let tessara_module_contract::ResourceOwner::ModuleInstance {
        installation_id,
        module_instance_id,
    } = reference.owner()
    else {
        return ResourceObservationProviderLookup::OwnerUnavailable;
    };
    if reference.installation_id() != *installation_id {
        return ResourceObservationProviderLookup::OwnerUnavailable;
    }

    let mut owners = installed.iter().filter(|module| {
        module.installation_id == *installation_id && module.instance_id == *module_instance_id
    });
    let Some(module) = owners.next() else {
        return ResourceObservationProviderLookup::OwnerUnavailable;
    };
    if owners.next().is_some() {
        return ResourceObservationProviderLookup::OwnerUnavailable;
    }

    let mut reference_schemas = module
        .manifest
        .typed_reference_schemas
        .iter()
        .filter(|schema| schema.resource_type == *reference.resource_type());
    let Some(reference_schema) = reference_schemas.next() else {
        return ResourceObservationProviderLookup::ReferenceUnsupported;
    };
    if reference_schemas.next().is_some() || reference_schema.validate_reference(reference).is_err()
    {
        return ResourceObservationProviderLookup::ReferenceUnsupported;
    }

    let mut actions = module
        .manifest
        .provided_service_actions
        .iter()
        .filter(|action| {
            action.method == ServiceActionMethod::Post
                && action.operation == AuthorizationGrantOperationV1::Read
                && action.authorization_action.rsplit('.').next() == Some("resolve")
                && action.path.rsplit('/').next() == Some("resolve")
                && action.functional_contract.as_str().rsplit('.').next()
                    == Some("resource-observation")
        });
    let Some(action) = actions.next() else {
        return ResourceObservationProviderLookup::ContractUnavailable;
    };
    if actions.next().is_some() {
        return ResourceObservationProviderLookup::ContractUnavailable;
    }

    let mut contracts = module
        .manifest
        .provided_contracts
        .iter()
        .filter(|contract| {
            contract.id == action.functional_contract
                && contract.version.to_string() == RESOURCE_OBSERVATION_PROVIDER_CONTRACT_VERSION
        });
    let Some(contract) = contracts.next() else {
        return ResourceObservationProviderLookup::ContractUnavailable;
    };
    if contracts.next().is_some()
        || !module
            .manifest
            .security_capabilities
            .iter()
            .any(|capability| capability.id == action.required_capability)
    {
        return ResourceObservationProviderLookup::ContractUnavailable;
    }

    let bindings = module
        .manifest
        .public_api_routes
        .iter()
        .map(|route| route.dependency_binding.as_str())
        .collect::<BTreeSet<_>>();
    let mut bindings = bindings.into_iter();
    let Some(dependency_binding) = bindings.next() else {
        return ResourceObservationProviderLookup::ContractUnavailable;
    };
    if bindings.next().is_some() {
        return ResourceObservationProviderLookup::ContractUnavailable;
    }

    ResourceObservationProviderLookup::Registered(ResourceObservationProviderRoute {
        owner: CorePrivateProviderOwner {
            installation_id: module.installation_id,
            module_instance_id: module.instance_id,
        },
        module_definition_id: module.manifest.definition_id.to_string(),
        dependency_binding: dependency_binding.to_owned(),
        functional_contract: contract.id.to_string(),
        contract_version: contract.version.to_string(),
        authorization_action: action.authorization_action.clone(),
        path: action.path.clone(),
        actor_capability: action.required_capability.to_string(),
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum CorePrivateProviderResult<T> {
    Response(T),
    Unavailable,
    Undisclosed,
}

/// Calls one exact, manifest-declared private module action as CoreGateway.
/// Module absence, health failure, transport failure, and incompatible wire
/// responses remain an explicit unavailable state; an unauthorized actor or
/// provider rejection remains undisclosed.
pub(crate) async fn call_private_provider<TRequest, TResponse>(
    state: &AppState,
    actor: &AuthenticatedRequest,
    request: CorePrivateProviderRequest<'_, TRequest>,
) -> ApiResult<CorePrivateProviderResult<TResponse>>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    let installed = installed_modules(&state.pool).await?;
    let mut candidates = installed.into_iter().filter(|module| {
        module.manifest.definition_id.as_str() == request.module_definition_id
            && request.expected_owner.is_none_or(|owner| {
                module.installation_id == owner.installation_id
                    && module.instance_id == owner.module_instance_id
            })
    });
    let Some(module) = candidates.next() else {
        return Ok(CorePrivateProviderResult::Unavailable);
    };
    if candidates.next().is_some()
        || !module.serving
        || !private_action_matches(&module.manifest, &request)
    {
        return Ok(CorePrivateProviderResult::Unavailable);
    }

    let declaration = module
        .manifest
        .provided_service_actions
        .iter()
        .find(|declaration| {
            declaration.functional_contract.as_str() == request.functional_contract
                && declaration.authorization_action == request.authorization_action
        })
        .expect("private action was checked before authorization");
    let mut bindings = capability_bindings(
        &state.pool,
        actor.account.account_id,
        request.actor_capability,
    )
    .await?;
    if actor
        .account
        .has_global_capability(request.actor_capability)
    {
        bindings.push(CapabilityScopeBindingV1 {
            capability: declaration.required_capability.clone(),
            organization_root_id: module.installation_id,
            authorized_organization_ids: Vec::new(),
        });
    }
    if bindings.is_empty() {
        return Ok(CorePrivateProviderResult::Undisclosed);
    }
    for binding in &mut bindings {
        binding.capability = declaration.required_capability.clone();
    }

    if request.correlation_id.is_nil() {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "Core private provider correlation identity must not be nil"
        )));
    }
    let correlation_id = request.correlation_id;
    let dependency_binding = DependencyBindingKey::new(request.dependency_binding)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let grant = match issue_module_authorization(
        state,
        actor,
        &module,
        correlation_id,
        AuthorizationRequest {
            action: request.authorization_action,
            dependency_binding: &dependency_binding,
            operation: AuthorizationGrantOperationV1::Read,
            required_capabilities_any_of: std::slice::from_ref(&declaration.required_capability),
            contract: &declaration.functional_contract,
        },
        bindings,
    )
    .await
    {
        Ok(grant) => grant,
        Err(ApiError::Forbidden(_)) => return Ok(CorePrivateProviderResult::Undisclosed),
        Err(ApiError::ServiceUnavailable(_)) => {
            return Ok(CorePrivateProviderResult::Unavailable);
        }
        Err(error) => return Err(error),
    };
    send_private_provider_request(
        &module,
        PrivateProviderTarget {
            dependency_binding: request.dependency_binding,
            authorization_action: request.authorization_action,
            path: request.path,
            media_type: request.media_type,
            correlation_id,
        },
        &grant,
        request.body,
    )
    .await
}

/// Calls one exact private provider action for a declared, non-browser Core
/// system job. The job receives only the provider action's manifest-declared
/// capability at installation scope; it does not inherit a manager account or
/// browser session.
pub(crate) async fn call_private_provider_for_system_job<TRequest, TResponse>(
    state: &AppState,
    request: CoreSystemJobProviderRequest<'_, TRequest>,
) -> ApiResult<CorePrivateProviderResult<TResponse>>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    validate_system_job_id(request.system_job_id)?;
    if request.correlation_id.is_nil() {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "Core system-job provider correlation identity must not be nil"
        )));
    }
    let installed = installed_modules(&state.pool).await?;
    let mut candidates = installed.into_iter().filter(|module| {
        module.manifest.definition_id.as_str() == request.module_definition_id
            && request.expected_owner.is_none_or(|owner| {
                module.installation_id == owner.installation_id
                    && module.instance_id == owner.module_instance_id
            })
    });
    let Some(module) = candidates.next() else {
        return Ok(CorePrivateProviderResult::Unavailable);
    };
    let action_request = CorePrivateProviderRequest {
        module_definition_id: request.module_definition_id,
        expected_owner: request.expected_owner,
        dependency_binding: request.dependency_binding,
        functional_contract: request.functional_contract,
        contract_version: request.contract_version,
        authorization_action: request.authorization_action,
        path: request.path,
        media_type: request.media_type,
        correlation_id: request.correlation_id,
        actor_capability: "",
        body: request.body,
    };
    if candidates.next().is_some()
        || !module.system_job_reachable
        || !private_action_matches(&module.manifest, &action_request)
    {
        return Ok(CorePrivateProviderResult::Unavailable);
    }
    let declaration = module
        .manifest
        .provided_service_actions
        .iter()
        .find(|declaration| {
            declaration.functional_contract.as_str() == request.functional_contract
                && declaration.authorization_action == request.authorization_action
        })
        .expect("private action was checked before system-job authorization");
    let target = PrivateProviderTarget {
        dependency_binding: request.dependency_binding,
        authorization_action: request.authorization_action,
        path: request.path,
        media_type: request.media_type,
        correlation_id: request.correlation_id,
    };
    let grant = match issue_system_job_authorization(
        state,
        &module,
        declaration,
        target,
        request.system_job_id,
    )
    .await
    {
        Ok(grant) => grant,
        Err(ApiError::Forbidden(_)) => return Ok(CorePrivateProviderResult::Undisclosed),
        Err(ApiError::ServiceUnavailable(_)) => {
            return Ok(CorePrivateProviderResult::Unavailable);
        }
        Err(error) => return Err(error),
    };
    send_private_provider_request(&module, target, &grant, request.body).await
}

async fn issue_system_job_authorization(
    state: &AppState,
    module: &InstalledModule,
    declaration: &tessara_module_contract::ProvidedServiceActionDeclaration,
    target: PrivateProviderTarget<'_>,
    system_job_id: &str,
) -> ApiResult<tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3>> {
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let authorization_revision: i64 = revisions.try_get("authorization_revision")?;
    let organization_revision: i64 = revisions.try_get("organization_revision")?;
    sync_control_projections(
        state,
        module.installation_id,
        module.instance_id,
        &module.manifest,
        authorization_revision,
        organization_revision,
    )
    .await?;
    let now = Utc::now();
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: module.installation_id,
            original_actor_id: core_system_job_principal_id(module.installation_id, system_job_id),
            correlation_id: target.correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: module.instance_id,
                module_definition_id: module.manifest.definition_id.clone(),
            },
            dependency_binding: DependencyBindingKey::new(target.dependency_binding)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: declaration.functional_contract.clone(),
            action: target.authorization_action.into(),
            operation: declaration.operation,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: declaration.required_capability.clone(),
                organization_root_id: module.installation_id,
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: authorization_revision as u64,
            organization_revision: organization_revision as u64,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .map_err(|error| ApiError::Internal(error.into()))
}

fn validate_system_job_id(system_job_id: &str) -> ApiResult<()> {
    let valid = !system_job_id.is_empty()
        && system_job_id.len() <= 128
        && system_job_id.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(&byte)
        });
    valid.then_some(()).ok_or_else(|| {
        ApiError::Internal(anyhow::anyhow!("Core system-job identity is not canonical"))
    })
}

fn core_system_job_principal_id(installation_id: Uuid, system_job_id: &str) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(b"tessara.core-system-job/v1\0");
    digest.update(installation_id.as_bytes());
    digest.update(b"\0");
    digest.update(system_job_id.as_bytes());
    let digest = digest.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

async fn send_private_provider_request<TRequest, TResponse>(
    module: &InstalledModule,
    target: PrivateProviderTarget<'_>,
    grant: &tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3>,
    request_body: &TRequest,
) -> ApiResult<CorePrivateProviderResult<TResponse>>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    let body =
        serde_json::to_vec(request_body).map_err(|error| ApiError::Internal(error.into()))?;
    let encoded_grant = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&grant).map_err(|error| ApiError::Internal(error.into()))?);
    let now = Utc::now();
    let core_request = protocol_signer(ProtocolSignaturePurposeV1::ModuleServiceRequest)?
        .sign(CoreServiceRequestV1 {
            schema_version: 1,
            installation_id: module.installation_id,
            method: "POST".into(),
            path: target.path.into(),
            canonical_body_digest: sha256_hex(&body),
            inbound_grant_digest: sha256_hex(encoded_grant.as_bytes()),
            correlation_id: target.correlation_id.to_string(),
            nonce: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .map_err(|error| ApiError::Internal(error.into()))?;
    let encoded_core_request = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&core_request).map_err(|error| ApiError::Internal(error.into()))?,
    );
    let endpoint = match service_endpoint(&module.manifest) {
        Ok(endpoint) => endpoint,
        Err(ApiError::ServiceUnavailable(_)) => {
            return Ok(CorePrivateProviderResult::Unavailable);
        }
        Err(error) => return Err(error),
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .map_err(|error| ApiError::Internal(error.into()))?;
    let response = match client
        .post(format!("{endpoint}{}", target.path))
        .header(reqwest::header::CONTENT_TYPE, target.media_type)
        .header(reqwest::header::ACCEPT, target.media_type)
        .header("x-tessara-authorization", encoded_grant)
        .header("x-tessara-core-service-request", encoded_core_request)
        .header(
            "x-tessara-correlation-id",
            target.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => return Ok(CorePrivateProviderResult::Unavailable),
    };
    if matches!(response.status().as_u16(), 401 | 403) {
        return Ok(CorePrivateProviderResult::Undisclosed);
    }
    if !response.status().is_success() {
        return Ok(CorePrivateProviderResult::Unavailable);
    }
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_owned);
    let content_length = response.content_length();
    let bytes = match bounded_private_provider_body(response).await {
        Ok(Some(bytes)) => bytes,
        Ok(None) | Err(_) => return Ok(CorePrivateProviderResult::Unavailable),
    };
    Ok(decode_private_provider_response(
        target.media_type,
        content_type.as_deref(),
        content_length,
        &bytes,
    ))
}

async fn bounded_private_provider_body(
    mut response: reqwest::Response,
) -> Result<Option<Vec<u8>>, reqwest::Error> {
    let mut body = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if body.len().saturating_add(chunk.len()) > CORE_PRIVATE_PROVIDER_RESPONSE_LIMIT_BYTES {
            return Ok(None);
        }
        body.extend_from_slice(&chunk);
    }
    Ok(Some(body))
}

fn decode_private_provider_response<T: DeserializeOwned>(
    expected_media_type: &str,
    content_type: Option<&str>,
    declared_length: Option<u64>,
    bytes: &[u8],
) -> CorePrivateProviderResult<T> {
    if content_type != Some(expected_media_type)
        || declared_length
            .is_some_and(|length| length > CORE_PRIVATE_PROVIDER_RESPONSE_LIMIT_BYTES as u64)
        || bytes.len() > CORE_PRIVATE_PROVIDER_RESPONSE_LIMIT_BYTES
    {
        return CorePrivateProviderResult::Unavailable;
    }
    serde_json::from_slice(bytes)
        .map(CorePrivateProviderResult::Response)
        .unwrap_or(CorePrivateProviderResult::Unavailable)
}

fn private_action_matches<T>(
    manifest: &ModuleManifest,
    request: &CorePrivateProviderRequest<'_, T>,
) -> bool {
    let Some(declaration) = manifest
        .provided_service_actions
        .iter()
        .find(|declaration| {
            declaration.functional_contract.as_str() == request.functional_contract
                && declaration.authorization_action == request.authorization_action
        })
    else {
        return false;
    };
    let contract_matches = manifest.provided_contracts.iter().any(|contract| {
        contract.id.as_str() == request.functional_contract
            && contract.version.to_string() == request.contract_version
    });
    declaration.method == ServiceActionMethod::Post
        && declaration.path == request.path
        && contract_matches
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
    let correlation_id = request_correlation_id_or_new(request.headers());
    let installed = installed_modules(&state.pool).await?;

    for module in installed {
        if matches!(method, Method::GET | Method::HEAD)
            && let Some(route) = module
                .manifest
                .browser_routes
                .iter()
                .enumerate()
                .filter(|(_, route)| {
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
                .max_by_key(|(index, route)| {
                    (
                        path_template_specificity(&route.path_template),
                        Reverse(*index),
                    )
                })
                .map(|(_, route)| route)
        {
            if !module.serving {
                return Ok(crate::module_unavailable_fallback_response());
            }
            let grant = module_authorization(
                state,
                actor,
                &module,
                correlation_id,
                AuthorizationRequest {
                    action: &route.authorization_action,
                    dependency_binding: &route.dependency_binding,
                    operation: AuthorizationGrantOperationV1::Read,
                    required_capabilities_any_of: std::slice::from_ref(&route.required_capability),
                    contract: &route.functional_contract,
                },
            )
            .await?;
            let navigation = crate::modules::load_context_navigation(
                state,
                &actor.account,
                module.installation_id,
            )
            .await?;
            let shell = shell_context(actor, &module, &path, correlation_id, navigation)?;
            return forward(
                &module,
                ForwardRequest {
                    method,
                    path: &path,
                    query: request.uri().query(),
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
            && let Some(route) = module
                .manifest
                .public_api_routes
                .iter()
                .enumerate()
                .filter(|(_, route)| {
                    api_method_matches(route.method, &method)
                        && path_template_matches(&route.path_template, &path)
                })
                .max_by_key(|(index, route)| {
                    (
                        path_template_specificity(&route.path_template),
                        Reverse(*index),
                    )
                })
                .map(|(_, route)| route)
        {
            let grant = module_authorization(
                state,
                actor,
                &module,
                correlation_id,
                AuthorizationRequest {
                    action: &route.authorization_action,
                    dependency_binding: &route.dependency_binding,
                    operation: route.operation,
                    required_capabilities_any_of: &route.required_capabilities_any_of,
                    contract: &route.functional_contract,
                },
            )
            .await?;
            let query = request.uri().query().map(str::to_owned);
            let (parts, body) = request.into_parts();
            let bytes = to_bytes(body, 2 * 1024 * 1024)
                .await
                .map_err(|_| ApiError::BadRequest("module request body is too large".into()))?;
            return forward(
                &module,
                ForwardRequest {
                    method,
                    path: &path,
                    query: query.as_deref(),
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
                            query: request.uri().query(),
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
         JOIN application_installations installations
           ON installations.id=instances.installation_id AND installations.singleton=true
         WHERE instances.identity_state='live' AND instances.installed
           AND releases.manifest IS NOT NULL
         ORDER BY instances.definition_id",
    )
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let deployed = row.try_get::<bool, _>("deployed")?;
            let configured = row.try_get::<bool, _>("configured")?;
            let enabled = row.try_get::<bool, _>("enabled")?;
            Ok(InstalledModule {
                instance_id: row.try_get("id")?,
                installation_id: row.try_get("installation_id")?,
                serving: deployed
                    && configured
                    && enabled
                    && row.try_get::<bool, _>("ready")?
                    && row.try_get::<bool, _>("healthy")?,
                system_job_reachable: deployed && configured && enabled,
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
    required_capabilities_any_of: &'a [SecurityCapabilityId],
    contract: &'a FunctionalContractId,
}

fn has_required_authorization_binding(
    required_capabilities_any_of: &[SecurityCapabilityId],
    bindings: &[CapabilityScopeBindingV1],
) -> bool {
    !required_capabilities_any_of.is_empty()
        && bindings.iter().any(|binding| {
            required_capabilities_any_of
                .iter()
                .any(|required| required == &binding.capability)
        })
}

fn filter_required_authorization_bindings(
    required_capabilities_any_of: &[SecurityCapabilityId],
    bindings: Vec<CapabilityScopeBindingV1>,
) -> Vec<CapabilityScopeBindingV1> {
    bindings
        .into_iter()
        .filter(|binding| required_capabilities_any_of.contains(&binding.capability))
        .collect()
}

async fn module_authorization(
    state: &AppState,
    actor: &AuthenticatedRequest,
    module: &InstalledModule,
    correlation_id: Uuid,
    request: AuthorizationRequest<'_>,
) -> ApiResult<tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3>> {
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
    issue_module_authorization(state, actor, module, correlation_id, request, bindings).await
}

async fn issue_module_authorization(
    state: &AppState,
    actor: &AuthenticatedRequest,
    module: &InstalledModule,
    correlation_id: Uuid,
    request: AuthorizationRequest<'_>,
    bindings: Vec<CapabilityScopeBindingV1>,
) -> ApiResult<tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3>> {
    let bindings =
        filter_required_authorization_bindings(request.required_capabilities_any_of, bindings);
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let authorization_revision: i64 = revisions.try_get("authorization_revision")?;
    let organization_revision: i64 = revisions.try_get("organization_revision")?;
    sync_control_projections(
        state,
        module.installation_id,
        module.instance_id,
        &module.manifest,
        authorization_revision,
        organization_revision,
    )
    .await?;

    if !has_required_authorization_binding(request.required_capabilities_any_of, &bindings) {
        tracing::warn!(
            actor_id = %actor.account.account_id,
            required_capabilities = ?request
                .required_capabilities_any_of
                .iter()
                .map(SecurityCapabilityId::as_str)
                .collect::<Vec<_>>(),
            binding_count = bindings.len(),
            "Generic module action has no authorized capability binding"
        );
        return Err(ApiError::Forbidden("module action unavailable".into()));
    }

    let delegation_basis =
        load_module_delegation_basis(&state.pool, actor.account.account_id, &bindings).await?;

    let now = Utc::now();
    let grant = AuthorizationGrantV3 {
        schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
        installation_id: module.installation_id,
        original_actor_id: actor.account.account_id,
        correlation_id,
        presenting_service: ModuleServicePrincipalV1::CoreGateway,
        audience: AuthorizationAudienceV1::ModuleInstance {
            module_instance_id: module.instance_id,
            module_definition_id: module.manifest.definition_id.clone(),
        },
        dependency_binding: request.dependency_binding.clone(),
        functional_contract: request.contract.clone(),
        action: request.action.into(),
        operation: request.operation,
        capability_scope_bindings: bindings,
        resource_assertion: None,
        delegation_basis,
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

async fn load_module_delegation_basis(
    pool: &sqlx::PgPool,
    original_actor_id: Uuid,
    bindings: &[CapabilityScopeBindingV1],
) -> ApiResult<Vec<DelegationBasisV1>> {
    let delegated_by_actor_ids = sqlx::query_scalar::<_, Uuid>(
        "SELECT delegate_account_id
         FROM account_delegations
         WHERE delegator_account_id=$1
         ORDER BY delegate_account_id",
    )
    .bind(original_actor_id)
    .fetch_all(pool)
    .await?;
    Ok(project_module_delegation_basis(
        original_actor_id,
        &delegated_by_actor_ids,
        bindings,
    ))
}

fn project_module_delegation_basis(
    original_actor_id: Uuid,
    delegated_by_actor_ids: &[Uuid],
    bindings: &[CapabilityScopeBindingV1],
) -> Vec<DelegationBasisV1> {
    delegated_by_actor_ids
        .iter()
        .flat_map(|delegated_by_actor_id| {
            bindings.iter().map(move |binding| DelegationBasisV1 {
                delegation_id: module_delegation_id(
                    original_actor_id,
                    *delegated_by_actor_id,
                    &binding.capability,
                    binding.organization_root_id,
                ),
                delegated_by_actor_id: *delegated_by_actor_id,
                capability: binding.capability.clone(),
                organization_root_id: binding.organization_root_id,
            })
        })
        .collect()
}

fn module_delegation_id(
    original_actor_id: Uuid,
    delegated_by_actor_id: Uuid,
    capability: &SecurityCapabilityId,
    organization_root_id: Uuid,
) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(b"tessara.module-delegation/v1\0");
    digest.update(original_actor_id.as_bytes());
    digest.update(delegated_by_actor_id.as_bytes());
    digest.update(capability.as_str().as_bytes());
    digest.update(organization_root_id.as_bytes());
    let digest = digest.finalize();
    let mut bytes = [0_u8; 16];
    bytes.copy_from_slice(&digest[..16]);
    bytes[6] = (bytes[6] & 0x0f) | 0x80;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    Uuid::from_bytes(bytes)
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn shell_context(
    actor: &AuthenticatedRequest,
    module: &InstalledModule,
    path: &str,
    correlation_id: Uuid,
    navigation: Vec<ShellNavigationGroupProjectionV2>,
) -> ApiResult<tessara_module_contract::SignedEnvelopeV1<ShellContextV2>> {
    let now = Utc::now();
    let context = ShellContextV2 {
        schema_version: SHELL_CONTEXT_SCHEMA_VERSION_V2,
        installation_id: module.installation_id,
        module_definition_id: module.manifest.definition_id.clone(),
        module_instance_id: module.instance_id,
        original_actor: OriginalActorProjectionV1 {
            actor_id: actor.account.account_id,
            display_name: actor.account.display_name.clone(),
            email: Some(actor.account.email.clone()),
        },
        theme: ShellThemeV1::System,
        navigation,
        return_destination: "/".into(),
        locale: "en-US".into(),
        time_zone: "UTC".into(),
        correlation_id,
        document_state: ShellDocumentStateV1::Active,
        issued_at: now,
        expires_at: now + Duration::seconds(60),
    };
    let _ = path;
    protocol_signer(ProtocolSignaturePurposeV1::ShellContext)?
        .sign(context)
        .map_err(|error| ApiError::Internal(error.into()))
}

pub(crate) async fn sync_control_projections(
    state: &AppState,
    installation_id: Uuid,
    module_instance_id: Uuid,
    manifest: &ModuleManifest,
    authorization_revision: i64,
    organization_revision: i64,
) -> ApiResult<()> {
    let endpoint = service_endpoint(manifest)?;
    let control_key = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let client = reqwest::Client::new();
    for projection in &manifest.control_projections {
        let payload = match projection.kind {
            tessara_module_contract::ControlProjectionKind::SecurityState => json!({
                "schema_version":1,
                "installation_id":installation_id,
                "module_instance_id":module_instance_id,
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
    query: Option<&'a str>,
    inbound_headers: &'a HeaderMap,
    body: Bytes,
    grant: Option<&'a tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3>>,
    shell: Option<&'a tessara_module_contract::SignedEnvelopeV1<ShellContextV2>>,
    idempotent: bool,
}

async fn forward(module: &InstalledModule, request: ForwardRequest<'_>) -> ApiResult<Response> {
    let endpoint = service_endpoint(&module.manifest)?;
    let client = reqwest::Client::new();
    let target = forwarded_target(&endpoint, request.path, request.query);
    let mut outbound = client.request(
        reqwest::Method::from_bytes(request.method.as_str().as_bytes())
            .map_err(|error| ApiError::Internal(error.into()))?,
        target,
    );
    if let Some(grant) = request.grant {
        outbound = outbound.header(
            "x-tessara-authorization",
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(grant).map_err(|error| ApiError::Internal(error.into()))?,
            ),
        );
        outbound = outbound.header(
            "x-tessara-correlation-id",
            grant.payload.correlation_id.to_string(),
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

fn forwarded_target(endpoint: &str, path: &str, query: Option<&str>) -> String {
    match query {
        Some(query) if !query.is_empty() => format!("{endpoint}{path}?{query}"),
        _ => format!("{endpoint}{path}"),
    }
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
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            serde_json::from_str::<BTreeMap<String, String>>(&value)
                .map_err(|_| module_unavailable())
                .and_then(|map| {
                    map.get(manifest.definition_id.as_str())
                        .filter(|endpoint| !endpoint.trim().is_empty())
                        .cloned()
                        .ok_or_else(|| {
                            ApiError::ServiceUnavailable(
                                "module service endpoint is unavailable".into(),
                            )
                        })
                })
        })
        .transpose()?;
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

fn path_template_specificity(template: &str) -> usize {
    template
        .trim_matches('/')
        .split('/')
        .filter(|segment| !(segment.starts_with('{') && segment.ends_with('}')))
        .count()
}

fn api_method_matches(declared: PublicApiMethod, actual: &Method) -> bool {
    matches!(
        (declared, actual),
        (PublicApiMethod::Get, &Method::GET)
            | (PublicApiMethod::Post, &Method::POST)
            | (PublicApiMethod::Put, &Method::PUT)
            | (PublicApiMethod::Patch, &Method::PATCH)
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
             AND (c.key=$2 OR c.key='admin:all'
                  OR ($2 LIKE '%:read' AND c.key=replace($2, ':read', ':manage')))
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
    use serde::Deserialize;

    use super::*;

    #[test]
    fn public_api_capability_alternatives_authorize_either_declared_binding() {
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-response-module/manifest.json"))
                .expect("Response manifest");
        let route = manifest
            .public_api_routes
            .iter()
            .find(|route| {
                route.method == PublicApiMethod::Delete
                    && route.path_template == "/api/responses/{response_id}"
            })
            .expect("Response delete route");
        assert_eq!(
            route
                .required_capabilities_any_of
                .iter()
                .map(SecurityCapabilityId::as_str)
                .collect::<Vec<_>>(),
            ["submissions:respond", "submissions:manage"]
        );
        let binding = |capability: &str| CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new(capability).unwrap(),
            organization_root_id: Uuid::from_u128(1),
            authorized_organization_ids: Vec::new(),
        };
        assert!(has_required_authorization_binding(
            &route.required_capabilities_any_of,
            &[binding("submissions:respond")]
        ));
        assert!(has_required_authorization_binding(
            &route.required_capabilities_any_of,
            &[binding("submissions:manage")]
        ));
        assert!(!has_required_authorization_binding(
            &route.required_capabilities_any_of,
            &[binding("submissions:read_own")]
        ));
        assert!(!has_required_authorization_binding(
            &[],
            &[binding("submissions:manage")]
        ));

        let start = manifest
            .public_api_routes
            .iter()
            .find(|route| {
                route.method == PublicApiMethod::Post && route.path_template == "/api/responses"
            })
            .expect("Response start route");
        assert_eq!(
            start
                .required_capabilities_any_of
                .iter()
                .map(SecurityCapabilityId::as_str)
                .collect::<Vec<_>>(),
            ["submissions:respond", "submissions:manage"]
        );
        assert!(has_required_authorization_binding(
            &start.required_capabilities_any_of,
            &[binding("submissions:manage")]
        ));
    }

    #[test]
    fn public_api_grants_exclude_undeclared_actor_bindings() {
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-response-module/manifest.json"))
                .expect("Response manifest");
        let binding = |capability: &str, organization_root_id: u128| CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new(capability).unwrap(),
            organization_root_id: Uuid::from_u128(organization_root_id),
            authorized_organization_ids: vec![Uuid::from_u128(organization_root_id + 100)],
        };
        let delete = manifest
            .public_api_routes
            .iter()
            .find(|route| route.authorization_action == "responses.delete")
            .expect("Response delete route");
        let delete_bindings = filter_required_authorization_bindings(
            &delete.required_capabilities_any_of,
            vec![
                binding("submissions:respond", 10),
                binding("submissions:manage", 20),
                binding("submissions:read_own", 30),
            ],
        );
        assert_eq!(
            delete_bindings
                .iter()
                .map(|binding| (binding.capability.as_str(), binding.organization_root_id))
                .collect::<Vec<_>>(),
            [
                ("submissions:respond", Uuid::from_u128(10)),
                ("submissions:manage", Uuid::from_u128(20)),
            ]
        );

        let detail = manifest
            .public_api_routes
            .iter()
            .find(|route| route.authorization_action == "responses.get")
            .expect("Response detail route");
        let detail_bindings = filter_required_authorization_bindings(
            &detail.required_capabilities_any_of,
            vec![
                binding("submissions:read_own", 40),
                binding("submissions:manage", 50),
            ],
        );
        assert_eq!(
            detail_bindings
                .iter()
                .map(|binding| (binding.capability.as_str(), binding.organization_root_id))
                .collect::<Vec<_>>(),
            [
                ("submissions:read_own", Uuid::from_u128(40)),
                ("submissions:manage", Uuid::from_u128(50)),
            ]
        );
        assert_eq!(
            detail_bindings[0].authorized_organization_ids,
            [Uuid::from_u128(140)]
        );
        assert_eq!(
            detail_bindings[1].authorized_organization_ids,
            [Uuid::from_u128(150)]
        );
    }

    #[test]
    fn public_api_delegations_are_capability_and_scope_bound() {
        let original_actor_id = Uuid::from_u128(1);
        let delegated_by_actor_ids = [Uuid::from_u128(2), Uuid::from_u128(3)];
        let bindings = [
            CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("submissions:respond").unwrap(),
                organization_root_id: Uuid::from_u128(10),
                authorized_organization_ids: vec![Uuid::from_u128(11)],
            },
            CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("submissions:manage").unwrap(),
                organization_root_id: Uuid::from_u128(20),
                authorized_organization_ids: Vec::new(),
            },
        ];

        let basis =
            project_module_delegation_basis(original_actor_id, &delegated_by_actor_ids, &bindings);
        assert_eq!(basis.len(), 4);
        assert_eq!(basis[0].delegated_by_actor_id, delegated_by_actor_ids[0]);
        assert_eq!(basis[0].capability.as_str(), "submissions:respond");
        assert_eq!(basis[0].organization_root_id, Uuid::from_u128(10));
        assert_eq!(basis[1].capability.as_str(), "submissions:manage");
        assert_eq!(basis[2].delegated_by_actor_id, delegated_by_actor_ids[1]);
        assert_eq!(
            basis,
            project_module_delegation_basis(original_actor_id, &delegated_by_actor_ids, &bindings,),
            "the relationship/capability/scope projection must be deterministic"
        );
        let unique_ids = basis
            .iter()
            .map(|item| item.delegation_id)
            .collect::<BTreeSet<_>>();
        assert_eq!(unique_ids.len(), basis.len());
    }

    #[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
    #[serde(deny_unknown_fields)]
    struct PrivateResponseFixture {
        state: String,
    }

    #[test]
    fn private_provider_action_matches_exact_manifest_path_action_contract_and_version() {
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-dataset-module/manifest.json"))
                .expect("Dataset manifest");
        let body = tessara_datasets_contract::DatasetSummaryRequest { schema_version: 1 };
        let request = CorePrivateProviderRequest {
            module_definition_id: tessara_datasets_contract::DATASET_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding: tessara_datasets_contract::DATASET_CORE_BINDING_KEY,
            functional_contract: tessara_datasets_contract::DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
            contract_version: tessara_datasets_contract::DATASET_REVERSE_CONTRACT_VERSION,
            authorization_action: tessara_datasets_contract::DATASET_SUMMARY_ACTION,
            path: tessara_datasets_contract::DATASET_SUMMARY_PATH,
            media_type: CORE_PRIVATE_PROVIDER_MEDIA_TYPE,
            correlation_id: Uuid::from_u128(1),
            actor_capability: "admin:all",
            body: &body,
        };
        assert!(private_action_matches(&manifest, &request));

        let wrong_path = CorePrivateProviderRequest {
            path: tessara_datasets_contract::DATASET_OPERATIONS_STATUS_PATH,
            ..request
        };
        assert!(!private_action_matches(&manifest, &wrong_path));

        let wrong_version = CorePrivateProviderRequest {
            contract_version: "1.0.1",
            ..wrong_path
        };
        assert!(!private_action_matches(&manifest, &wrong_version));
    }

    #[test]
    fn generic_resource_observation_route_is_exactly_manifest_and_owner_bound() {
        let installation_id = Uuid::from_u128(101);
        let module_instance_id = Uuid::from_u128(102);
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-dataset-module/manifest.json"))
                .expect("Dataset manifest");
        let installed = [InstalledModule {
            instance_id: module_instance_id,
            installation_id,
            manifest,
            serving: true,
            system_job_reachable: true,
        }];
        let reference = tessara_datasets_contract::DatasetRevisionReference::from_parts(
            installation_id,
            module_instance_id,
            Uuid::from_u128(103),
        )
        .expect("canonical reference");

        let ResourceObservationProviderLookup::Registered(route) =
            resource_observation_provider_from_installed(&installed, reference.reference())
        else {
            panic!("resource observation route was not registered");
        };
        let body = tessara_datasets_contract::DatasetResourceObservationRequest {
            schema_version: 1,
            reference: reference.reference().clone(),
        };
        let request = route.private_request(Uuid::from_u128(104), &body);

        assert_eq!(
            request.expected_owner,
            Some(CorePrivateProviderOwner {
                installation_id,
                module_instance_id,
            })
        );
        assert_eq!(
            request.module_definition_id,
            tessara_datasets_contract::DATASET_MODULE_DEFINITION_ID
        );
        assert_eq!(
            request.dependency_binding,
            tessara_datasets_contract::DATASET_CORE_BINDING_KEY
        );
        assert_eq!(
            request.functional_contract,
            tessara_datasets_contract::DATASET_RESOURCE_OBSERVATION_CONTRACT_ID
        );
        assert_eq!(
            request.contract_version,
            tessara_datasets_contract::DATASET_REVERSE_CONTRACT_VERSION
        );
        assert_eq!(
            request.authorization_action,
            tessara_datasets_contract::DATASET_RESOLVE_ACTION
        );
        assert_eq!(
            request.path,
            tessara_datasets_contract::DATASET_RESOLVE_PATH
        );
        assert_eq!(request.actor_capability, "datasets:read");
    }

    #[test]
    fn response_resource_observation_reuses_the_manifest_owned_core_binding() {
        let installation_id = Uuid::from_u128(105);
        let module_instance_id = Uuid::from_u128(106);
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-response-module/manifest.json"))
                .expect("Response manifest");
        let installed = [InstalledModule {
            instance_id: module_instance_id,
            installation_id,
            manifest,
            serving: true,
            system_job_reachable: true,
        }];
        let reference = tessara_responses_contract::ResponseReference::from_parts(
            installation_id,
            module_instance_id,
            Uuid::from_u128(107),
        )
        .expect("canonical reference");

        let ResourceObservationProviderLookup::Registered(route) =
            resource_observation_provider_from_installed(&installed, reference.reference())
        else {
            panic!("Response resource observation route was not registered");
        };
        let body = tessara_responses_contract::ResponseResourceObservationRequest {
            schema_version: 1,
            reference: reference.reference().clone(),
        };
        let request = route.private_request(Uuid::from_u128(108), &body);

        assert_eq!(
            request.dependency_binding,
            tessara_responses_contract::RESPONSE_RESOURCE_OBSERVATION_BINDING_KEY
        );
        assert_eq!(request.dependency_binding, "tessara.core.responses");
        assert_eq!(
            request.functional_contract,
            tessara_responses_contract::RESPONSE_RESOURCE_OBSERVATION_CONTRACT_ID
        );
        assert_eq!(
            request.authorization_action,
            tessara_responses_contract::RESPONSE_RESOLVE_ACTION
        );
        assert_eq!(
            request.path,
            tessara_responses_contract::RESPONSE_RESOLVE_PATH
        );
        assert_eq!(request.actor_capability, "submissions:read_own");
    }

    #[test]
    fn generic_resource_observation_rejects_wrong_owner_type_id_and_contract_before_dispatch() {
        let installation_id = Uuid::from_u128(111);
        let module_instance_id = Uuid::from_u128(112);
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../../tessara-dataset-module/manifest.json"))
                .expect("Dataset manifest");
        let installed = [InstalledModule {
            instance_id: module_instance_id,
            installation_id,
            manifest: manifest.clone(),
            serving: true,
            system_job_reachable: true,
        }];

        let wrong_instance = tessara_datasets_contract::DatasetReference::from_parts(
            installation_id,
            Uuid::from_u128(999),
            Uuid::from_u128(113),
        )
        .expect("canonical reference");
        assert_eq!(
            resource_observation_provider_from_installed(&installed, wrong_instance.reference()),
            ResourceObservationProviderLookup::OwnerUnavailable
        );

        let old_type = TypedResourceReference::new(
            installation_id,
            tessara_module_contract::ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            tessara_module_contract::ResourceTypeId::new("tessara.transition.dataset_revision")
                .expect("resource type"),
            Uuid::from_u128(114).to_string(),
        )
        .expect("structural reference");
        assert_eq!(
            resource_observation_provider_from_installed(&installed, &old_type),
            ResourceObservationProviderLookup::ReferenceUnsupported
        );

        let invalid_id = TypedResourceReference::new(
            installation_id,
            tessara_module_contract::ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            tessara_module_contract::ResourceTypeId::new(
                tessara_datasets_contract::DATASET_REVISION_RESOURCE_TYPE,
            )
            .expect("resource type"),
            "not-a-canonical-uuid".to_string(),
        )
        .expect("structural reference");
        assert_eq!(
            resource_observation_provider_from_installed(&installed, &invalid_id),
            ResourceObservationProviderLookup::ReferenceUnsupported
        );

        let mut incompatible_manifest = manifest;
        incompatible_manifest
            .provided_contracts
            .iter_mut()
            .find(|contract| {
                contract.id.as_str()
                    == tessara_datasets_contract::DATASET_RESOURCE_OBSERVATION_CONTRACT_ID
            })
            .expect("resource observation contract")
            .version = semver::Version::parse("1.0.1").expect("version");
        let incompatible = [InstalledModule {
            instance_id: module_instance_id,
            installation_id,
            manifest: incompatible_manifest,
            serving: true,
            system_job_reachable: true,
        }];
        let canonical = tessara_datasets_contract::DatasetReference::from_parts(
            installation_id,
            module_instance_id,
            Uuid::from_u128(115),
        )
        .expect("canonical reference");
        assert_eq!(
            resource_observation_provider_from_installed(&incompatible, canonical.reference()),
            ResourceObservationProviderLookup::ContractUnavailable
        );
    }

    #[test]
    fn private_provider_response_requires_exact_json_and_bounded_valid_wire() {
        let body = br#"{"state":"available"}"#;
        assert_eq!(
            decode_private_provider_response::<PrivateResponseFixture>(
                "application/json",
                Some("application/json"),
                Some(body.len() as u64),
                body,
            ),
            CorePrivateProviderResult::Response(PrivateResponseFixture {
                state: "available".into(),
            })
        );
        for content_type in [
            None,
            Some("application/json; charset=utf-8"),
            Some("application/vnd.tessara+json"),
            Some("text/json"),
        ] {
            assert_eq!(
                decode_private_provider_response::<PrivateResponseFixture>(
                    "application/json",
                    content_type,
                    Some(body.len() as u64),
                    body,
                ),
                CorePrivateProviderResult::Unavailable
            );
        }
    }

    #[test]
    fn private_provider_malformed_and_oversized_responses_are_unavailable() {
        assert_eq!(
            decode_private_provider_response::<PrivateResponseFixture>(
                "application/json",
                Some("application/json"),
                None,
                br#"{"state":}"
            ),
            CorePrivateProviderResult::Unavailable
        );
        let oversized = vec![b' '; CORE_PRIVATE_PROVIDER_RESPONSE_LIMIT_BYTES + 1];
        assert_eq!(
            decode_private_provider_response::<PrivateResponseFixture>(
                "application/json",
                Some("application/json"),
                None,
                &oversized,
            ),
            CorePrivateProviderResult::Unavailable
        );
        assert_eq!(
            decode_private_provider_response::<PrivateResponseFixture>(
                "application/json",
                Some("application/json"),
                Some((CORE_PRIVATE_PROVIDER_RESPONSE_LIMIT_BYTES + 1) as u64),
                br#"{"state":"available"}"#,
            ),
            CorePrivateProviderResult::Unavailable
        );
    }

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
    fn manifest_route_specificity_places_static_siblings_before_parameters() {
        assert!(
            path_template_specificity("/api/admin/components/datasets")
                > path_template_specificity("/api/admin/components/{component_id}")
        );
        assert!(
            path_template_specificity("/api/admin/components/validate")
                > path_template_specificity("/api/admin/components/{component_id}")
        );
        assert!(
            path_template_specificity("/datasets/new")
                > path_template_specificity("/datasets/{dataset_id}")
        );
        assert!(
            path_template_specificity("/api/admin/datasets/sql-preview")
                > path_template_specificity("/api/admin/datasets/{dataset_id}")
        );
        assert!(
            path_template_specificity("/api/admin/datasets/editor-options/forms")
                > path_template_specificity("/api/admin/datasets/{dataset_id}/refresh")
        );
        assert!(
            path_template_specificity("/api/admin/datasets/{dataset_id}/sql-preview")
                > path_template_specificity("/api/admin/datasets/{dataset_id}")
        );
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
    fn request_correlation_preserves_valid_identity_and_replaces_invalid_values() {
        let expected = Uuid::new_v4();
        let mut valid = HeaderMap::new();
        valid.insert(
            "x-tessara-correlation-id",
            HeaderValue::from_str(&expected.to_string()).unwrap(),
        );
        assert_eq!(request_correlation_id_or_new(&valid), expected);

        for value in [Uuid::nil().to_string(), "not-a-uuid".to_string()] {
            let mut invalid = HeaderMap::new();
            invalid.insert(
                "x-tessara-correlation-id",
                HeaderValue::from_str(&value).unwrap(),
            );
            assert!(!request_correlation_id_or_new(&invalid).is_nil());
        }
        assert!(!request_correlation_id_or_new(&HeaderMap::new()).is_nil());
    }

    #[test]
    fn module_gateway_preserves_the_exact_browser_query() {
        assert_eq!(
            forwarded_target(
                "http://dashboards:8091",
                "/api/dashboards/1/placements/2/render/table",
                Some("page_size=10&cursor=offset%3A10"),
            ),
            "http://dashboards:8091/api/dashboards/1/placements/2/render/table?page_size=10&cursor=offset%3A10"
        );
        assert_eq!(
            forwarded_target("http://components:8092", "/components", None),
            "http://components:8092/components"
        );
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

    #[test]
    fn generic_gateway_matches_patch_without_definition_specific_routing() {
        assert!(api_method_matches(PublicApiMethod::Patch, &Method::PATCH));
        assert!(!api_method_matches(PublicApiMethod::Patch, &Method::PUT));
    }
}
