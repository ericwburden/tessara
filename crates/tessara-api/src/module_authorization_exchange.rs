//! Generic Core authorization exchange for one live module service acting on
//! behalf of an actor grant. Caller permissions come from its enrolled
//! Manifest and the exact applied lockfile; target operation and capability
//! come from the provider's declaration.

use axum::{Json, Router, body::Bytes, extract::State, http::HeaderMap, routing::post};
use chrono::{Duration, Utc};
use semver::Version;
use sqlx::Row;
use tessara_composition::{ApplicationLockfileV1, ResolvedModuleReleaseV1};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
    AuthorizationAudienceV1, AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2,
    AuthorizationGrantOperationV1, AuthorizationGrantV3, AuthorizationValidationContextV3,
    CapabilityScopeBindingV1, ModuleDefinitionId, ModuleManifest, ModuleServicePrincipalV1,
    ProtocolSignaturePurposeV1, ResourceAuthorizationAssertionV2,
};
use uuid::Uuid;

use crate::{
    core_security::{capability_bindings, protocol_signer},
    db::AppState,
    error::{ApiError, ApiResult},
};

const EXCHANGE_PATH: &str = "/api/private/module-authorization/exchange";

#[derive(Clone)]
struct LiveModule {
    instance_id: Uuid,
    definition_id: ModuleDefinitionId,
    version: Version,
    manifest_digest: String,
    manifest: ModuleManifest,
}

#[derive(Clone)]
struct ResolvedProviderAction {
    operation: AuthorizationGrantOperationV1,
    required_capability: String,
    contract_version: Version,
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(EXCHANGE_PATH, post(exchange))
}

async fn exchange(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Json<AuthorizationExchangeResponseV2>> {
    crate::module_service_requests::require_json_content_type(&headers)?;
    let inbound = crate::module_service_requests::verified_authorization(&headers)?;
    let installation_id = inbound.payload.installation_id;
    let lockfile = applied_lockfile(&state, installation_id).await?;
    let caller = audience_module(&state, installation_id, &inbound.payload.audience).await?;
    validate_inbound_grant(&state, &lockfile, &inbound.payload, &caller).await?;

    let caller_principal = ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id: caller.instance_id,
        module_definition_id: caller.definition_id.clone(),
    };
    crate::module_service_requests::validate_for_principal(
        &state,
        &headers,
        &inbound,
        crate::module_service_requests::ModuleServiceRequestExpectation {
            principal: &caller_principal,
            grant_consumption:
                crate::module_service_requests::AuthorizationGrantConsumption::ReusableExchange,
            method: "POST",
            path: EXCHANGE_PATH,
            body: &body,
        },
    )
    .await?;
    let request: AuthorizationExchangeRequestV2 =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    request.validate().map_err(|_| restricted())?;

    let provider_action = resolve_requested_provider_action(
        &state,
        installation_id,
        &lockfile,
        &request.target,
        request.functional_contract.as_str(),
        &request.action,
    )
    .await?;
    authorize_consumption(
        &lockfile,
        &caller,
        &request.target,
        request.dependency_binding.as_str(),
        request.functional_contract.as_str(),
        &request.action,
        &provider_action.contract_version,
    )?;

    let bindings = capability_bindings(
        &state.pool,
        inbound.payload.original_actor_id,
        &provider_action.required_capability,
    )
    .await?;
    if bindings.is_empty() {
        return Err(restricted());
    }
    if !resource_assertion_is_authorized(
        &bindings,
        &provider_action.required_capability,
        request.resource_assertion.as_ref(),
    ) {
        return Err(restricted());
    }
    let revisions = security_revisions(&state).await?;
    let now = Utc::now();
    let lifetime = match provider_action.operation {
        AuthorizationGrantOperationV1::Read => 60,
        AuthorizationGrantOperationV1::Mutation => 30,
    };
    let authorization = protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id,
            original_actor_id: inbound.payload.original_actor_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: caller_principal,
            audience: request.target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: request.action,
            operation: provider_action.operation,
            capability_scope_bindings: bindings,
            resource_assertion: request.resource_assertion,
            delegation_basis: inbound.payload.delegation_basis.clone(),
            authorization_revision: revisions.0,
            organization_revision: revisions.1,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(lifetime),
        })
        .map_err(|error| ApiError::Internal(error.into()))?;
    Ok(Json(AuthorizationExchangeResponseV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        authorization,
    }))
}

fn resource_assertion_is_authorized(
    bindings: &[CapabilityScopeBindingV1],
    required_capability: &str,
    assertion: Option<&ResourceAuthorizationAssertionV2>,
) -> bool {
    assertion.is_none_or(|assertion| {
        assertion
            .governing_organization_ids
            .iter()
            .any(|organization_id| {
                bindings.iter().any(|binding| {
                    binding.capability.as_str() == required_capability
                        && (binding.organization_root_id == *organization_id
                            || binding
                                .authorized_organization_ids
                                .contains(organization_id))
                })
            })
    })
}

async fn validate_inbound_grant(
    state: &AppState,
    lockfile: &ApplicationLockfileV1,
    grant: &AuthorizationGrantV3,
    audience: &LiveModule,
) -> ApiResult<()> {
    require_applied_module(lockfile, audience)?;
    let (operation, required_capability) = match &grant.presenting_service {
        ModuleServicePrincipalV1::CoreGateway => resolve_manifest_entry_action(
            &audience.manifest,
            grant.dependency_binding.as_str(),
            grant.functional_contract.as_str(),
            &grant.action,
        )?,
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => {
            let presenter = live_module(
                state,
                grant.installation_id,
                *module_instance_id,
                module_definition_id,
            )
            .await?;
            let action = resolve_module_provider_action(
                &audience.manifest,
                grant.functional_contract.as_str(),
                &grant.action,
            )?;
            authorize_consumption(
                lockfile,
                &presenter,
                &grant.audience,
                grant.dependency_binding.as_str(),
                grant.functional_contract.as_str(),
                &grant.action,
                &action.contract_version,
            )?;
            (action.operation, action.required_capability)
        }
    };
    if !grant
        .capability_scope_bindings
        .iter()
        .any(|binding| binding.capability.as_str() == required_capability)
    {
        return Err(restricted());
    }
    let revisions = security_revisions(state).await?;
    grant
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: grant.installation_id,
            correlation_id: grant.correlation_id,
            presenting_service: grant.presenting_service.clone(),
            audience: grant.audience.clone(),
            dependency_binding: grant.dependency_binding.clone(),
            functional_contract: grant.functional_contract.clone(),
            action: grant.action.clone(),
            operation,
            resource_assertion: grant.resource_assertion.clone(),
            authorization_revision: revisions.0,
            organization_revision: revisions.1,
            now: Utc::now(),
        })
        .map_err(|_| restricted())
}

fn resolve_manifest_entry_action(
    manifest: &ModuleManifest,
    dependency_binding: &str,
    functional_contract: &str,
    action: &str,
) -> ApiResult<(AuthorizationGrantOperationV1, String)> {
    if let Some(route) = manifest.public_api_routes.iter().find(|route| {
        route.dependency_binding.as_str() == dependency_binding
            && route.functional_contract.as_str() == functional_contract
            && route.authorization_action == action
    }) {
        return Ok((route.operation, route.required_capability.to_string()));
    }
    if let Some(route) = manifest.browser_routes.iter().find(|route| {
        route.dependency_binding.as_str() == dependency_binding
            && route.functional_contract.as_str() == functional_contract
            && route.authorization_action == action
    }) {
        return Ok((
            AuthorizationGrantOperationV1::Read,
            route.required_capability.to_string(),
        ));
    }
    Err(restricted())
}

async fn resolve_requested_provider_action(
    state: &AppState,
    installation_id: Uuid,
    lockfile: &ApplicationLockfileV1,
    target: &AuthorizationAudienceV1,
    functional_contract: &str,
    action: &str,
) -> ApiResult<ResolvedProviderAction> {
    match target {
        AuthorizationAudienceV1::CoreInstallation {
            installation_id: target_installation_id,
        } if *target_installation_id == installation_id => {
            let declaration =
                crate::core_service_providers::resolve_service_action(functional_contract, action)
                    .ok_or_else(restricted)?;
            let contract_version =
                crate::core_service_providers::contract_version(functional_contract)
                    .ok_or_else(restricted)?;
            Ok(ResolvedProviderAction {
                operation: declaration.operation,
                required_capability: declaration.required_capability.into(),
                contract_version,
            })
        }
        AuthorizationAudienceV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => {
            let target_module = live_module(
                state,
                installation_id,
                *module_instance_id,
                module_definition_id,
            )
            .await?;
            require_applied_module(lockfile, &target_module)?;
            resolve_module_provider_action(&target_module.manifest, functional_contract, action)
        }
        AuthorizationAudienceV1::CoreInstallation { .. } => Err(restricted()),
    }
}

fn resolve_module_provider_action(
    manifest: &ModuleManifest,
    functional_contract: &str,
    action: &str,
) -> ApiResult<ResolvedProviderAction> {
    let declaration = manifest
        .provided_service_actions
        .iter()
        .find(|declaration| {
            declaration.functional_contract.as_str() == functional_contract
                && declaration.authorization_action == action
        })
        .ok_or_else(restricted)?;
    let contract_version = manifest
        .provided_contracts
        .iter()
        .find(|contract| contract.id.as_str() == functional_contract)
        .map(|contract| contract.version.clone())
        .ok_or_else(restricted)?;
    Ok(ResolvedProviderAction {
        operation: declaration.operation,
        required_capability: declaration.required_capability.to_string(),
        contract_version,
    })
}

fn authorize_consumption(
    lockfile: &ApplicationLockfileV1,
    caller: &LiveModule,
    target: &AuthorizationAudienceV1,
    dependency_binding: &str,
    functional_contract: &str,
    action: &str,
    provider_version: &Version,
) -> ApiResult<()> {
    let caller_selection = require_applied_module(lockfile, caller)?;
    let dependency = caller
        .manifest
        .dependencies
        .iter()
        .find(|dependency| {
            dependency.binding_key.as_str() == dependency_binding
                && dependency.contract_id.as_str() == functional_contract
        })
        .ok_or_else(restricted)?;
    if !dependency.version_requirement.matches(provider_version)
        || !caller
            .manifest
            .consumed_service_actions
            .iter()
            .any(|declaration| {
                declaration.dependency_binding.as_str() == dependency_binding
                    && declaration.functional_contract.as_str() == functional_contract
                    && declaration.authorization_action == action
            })
    {
        return Err(restricted());
    }
    let binding = caller_selection
        .dependency_bindings
        .get(dependency_binding)
        .filter(|binding| {
            binding.contract_id == functional_contract
                && binding.contract_version == *provider_version
        })
        .ok_or_else(restricted)?;
    let expected_provider = match target {
        AuthorizationAudienceV1::CoreInstallation { installation_id }
            if *installation_id == lockfile.installation_id =>
        {
            "core"
        }
        AuthorizationAudienceV1::ModuleInstance {
            module_definition_id,
            ..
        } => module_definition_id.as_str(),
        AuthorizationAudienceV1::CoreInstallation { .. } => return Err(restricted()),
    };
    if binding.provider != expected_provider {
        return Err(restricted());
    }
    Ok(())
}

fn require_applied_module<'a>(
    lockfile: &'a ApplicationLockfileV1,
    module: &LiveModule,
) -> ApiResult<&'a ResolvedModuleReleaseV1> {
    lockfile
        .modules
        .iter()
        .find(|selection| {
            selection.enabled
                && selection.definition_id == module.definition_id.as_str()
                && selection.version == module.version
                && selection.manifest_digest.to_string() == module.manifest_digest
        })
        .ok_or_else(restricted)
}

async fn audience_module(
    state: &AppState,
    installation_id: Uuid,
    audience: &AuthorizationAudienceV1,
) -> ApiResult<LiveModule> {
    let AuthorizationAudienceV1::ModuleInstance {
        module_instance_id,
        module_definition_id,
    } = audience
    else {
        return Err(restricted());
    };
    live_module(
        state,
        installation_id,
        *module_instance_id,
        module_definition_id,
    )
    .await
}

async fn live_module(
    state: &AppState,
    installation_id: Uuid,
    module_instance_id: Uuid,
    module_definition_id: &ModuleDefinitionId,
) -> ApiResult<LiveModule> {
    let row = sqlx::query(
        "SELECT instance.id,instance.definition_id,release.version,
                release.manifest_digest,release.manifest
         FROM module_instances instance
         JOIN module_releases release ON release.id=instance.release_id
         WHERE instance.id=$1 AND instance.installation_id=$2
           AND instance.definition_id=$3 AND release.definition_id=instance.definition_id
           AND instance.identity_state='live' AND instance.installed=true
           AND instance.deployed=true AND instance.configured=true
           AND instance.ready=true AND instance.enabled=true AND instance.healthy=true",
    )
    .bind(module_instance_id)
    .bind(installation_id)
    .bind(module_definition_id.as_str())
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted)?;
    let definition_id = ModuleDefinitionId::new(row.try_get::<String, _>("definition_id")?)
        .map_err(|_| restricted())?;
    let version =
        Version::parse(&row.try_get::<String, _>("version")?).map_err(|_| restricted())?;
    let manifest: ModuleManifest =
        serde_json::from_value(row.try_get("manifest")?).map_err(|_| restricted())?;
    let manifest_digest: String = row.try_get("manifest_digest")?;
    let observed_digest = tessara_composition::canonical_digest(&manifest)
        .map_err(|error| ApiError::Internal(error.into()))?
        .to_string();
    if manifest.definition_id != definition_id
        || manifest.release_version != version
        || observed_digest != manifest_digest
    {
        return Err(restricted());
    }
    Ok(LiveModule {
        instance_id: module_instance_id,
        definition_id,
        version,
        manifest_digest,
        manifest,
    })
}

async fn applied_lockfile(
    state: &AppState,
    installation_id: Uuid,
) -> ApiResult<ApplicationLockfileV1> {
    let document = sqlx::query_scalar(
        "SELECT lockfile.document
         FROM composition_receipt_projections receipt
         JOIN composition_lockfiles lockfile
           ON lockfile.installation_id=receipt.installation_id
          AND lockfile.lockfile_digest=receipt.receipt->>'lockfile_digest'
         WHERE receipt.installation_id=$1
         ORDER BY receipt.revision DESC LIMIT 1",
    )
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted)?;
    let lockfile: ApplicationLockfileV1 =
        serde_json::from_value(document).map_err(|_| restricted())?;
    if lockfile.installation_id != installation_id {
        return Err(restricted());
    }
    Ok(lockfile)
}

async fn security_revisions(state: &AppState) -> ApiResult<(u64, u64)> {
    let revisions = sqlx::query_as::<_, (i64, i64)>(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok((revisions.0 as u64, revisions.1 as u64))
}

fn restricted() -> ApiError {
    ApiError::Forbidden("module authorization exchange unavailable".into())
}

#[cfg(test)]
mod tests {
    use tessara_module_contract::{ResourceTypeId, SecurityCapabilityId};

    use super::*;

    fn id(value: u128) -> Uuid {
        Uuid::from_u128(value)
    }

    fn binding(capability: &str, root: u128, descendants: &[u128]) -> CapabilityScopeBindingV1 {
        CapabilityScopeBindingV1 {
            capability: SecurityCapabilityId::new(capability).expect("valid capability fixture"),
            organization_root_id: id(root),
            authorized_organization_ids: descendants.iter().copied().map(id).collect(),
        }
    }

    fn assertion(organizations: &[u128]) -> ResourceAuthorizationAssertionV2 {
        ResourceAuthorizationAssertionV2 {
            resource_type: ResourceTypeId::new("tessara.example.resource")
                .expect("valid resource type fixture"),
            resource_id: "resource-1".to_string(),
            authority_revision: 1,
            governing_organization_ids: organizations.iter().copied().map(id).collect(),
        }
    }

    #[test]
    fn downstream_exchange_accepts_only_scope_authorized_resource_assertions() {
        let bindings = [
            binding("components:read", 10, &[11]),
            binding("components:manage", 20, &[21]),
        ];

        assert!(resource_assertion_is_authorized(
            &bindings,
            "components:read",
            None
        ));
        assert!(resource_assertion_is_authorized(
            &bindings,
            "components:read",
            Some(&assertion(&[10]))
        ));
        assert!(resource_assertion_is_authorized(
            &bindings,
            "components:read",
            Some(&assertion(&[11]))
        ));
        assert!(resource_assertion_is_authorized(
            &bindings,
            "components:read",
            Some(&assertion(&[99, 11]))
        ));
        assert!(!resource_assertion_is_authorized(
            &bindings,
            "components:read",
            Some(&assertion(&[99]))
        ));
        assert!(!resource_assertion_is_authorized(
            &bindings,
            "components:read",
            Some(&assertion(&[20, 21]))
        ));
    }
}
