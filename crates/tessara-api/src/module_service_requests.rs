//! Generic verification and materialization-time enrollment for requests
//! signed by independently deployed module services.

use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_composition::{
    ApplicationLockfileV1, MaterializationActionV1, OwnerBootstrapAuthorizationContextV1,
    OwnerBootstrapAuthorizationV1,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantV3, AuthorizationValidationContextV3,
    CapabilityScopeBindingV1, MODULE_SERVICE_IDENTITIES_ENVIRONMENT, ModuleManifest,
    ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, ProtocolSignaturePurposeV1,
    PurposeBoundVerifyingKeyV1, ServiceActionMethod, SignedEnvelopeV1,
};

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuthorizationGrantConsumption {
    ReusableExchange,
    OneTimeProviderAudience(uuid::Uuid),
}

#[derive(Clone, Debug)]
pub(crate) struct CoreProviderAuthorizationV1 {
    pub(crate) payload: CoreProviderAuthorizationPayloadV1,
}

#[derive(Clone, Debug)]
pub(crate) struct CoreProviderAuthorizationPayloadV1 {
    pub(crate) installation_id: uuid::Uuid,
    pub(crate) original_actor_id: uuid::Uuid,
    pub(crate) presenting_service: ModuleServicePrincipalV1,
    pub(crate) capability_scope_bindings: Vec<CapabilityScopeBindingV1>,
    pub(crate) delegation_basis: Vec<tessara_module_contract::DelegationBasisV1>,
}

impl AuthorizationGrantConsumption {
    fn authorization_jti(self) -> Option<uuid::Uuid> {
        match self {
            Self::ReusableExchange => None,
            Self::OneTimeProviderAudience(jti) => Some(jti),
        }
    }
}

/// Validates one exact Core-owned private provider invocation. Product
/// providers still own their media type, typed body, scope, and nondisclosure
/// rules; this function owns the shared grant/service-request boundary.
pub(crate) async fn authorize_core_provider(
    state: &AppState,
    headers: &HeaderMap,
    functional_contract: &str,
    action: &str,
    path: &str,
    body: &[u8],
    restricted_message: &'static str,
) -> ApiResult<CoreProviderAuthorizationV1> {
    let restricted = || ApiError::NotFound(restricted_message.into());
    let declaration =
        crate::core_service_providers::resolve_service_action(functional_contract, action)
            .filter(|declaration| {
                declaration.method == ServiceActionMethod::Post && declaration.path == path
            })
            .ok_or_else(&restricted)?;
    if headers.contains_key("x-tessara-owner-bootstrap-authorization") {
        return authorize_bootstrap_core_provider(
            state,
            headers,
            BootstrapCoreProviderExpectation {
                functional_contract,
                action,
                path,
                body,
                required_capability: declaration.required_capability,
                restricted_message,
            },
        )
        .await;
    }
    let inbound = verified_authorization(headers)?;
    let installation_exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM application_installations WHERE id=$1)")
            .bind(inbound.payload.installation_id)
            .fetch_one(&state.pool)
            .await?;
    if !installation_exists
        || inbound.payload.audience
            != (AuthorizationAudienceV1::CoreInstallation {
                installation_id: inbound.payload.installation_id,
            })
        || inbound.payload.functional_contract.as_str() != declaration.functional_contract
        || inbound.payload.action != declaration.authorization_action
        || inbound.payload.operation != declaration.operation
        || !inbound
            .payload
            .capability_scope_bindings
            .iter()
            .any(|binding| binding.capability.as_str() == declaration.required_capability)
    {
        return Err(restricted());
    }
    let revisions = sqlx::query_as::<_, (i64, i64)>(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    inbound
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: inbound.payload.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: inbound.payload.presenting_service.clone(),
            audience: AuthorizationAudienceV1::CoreInstallation {
                installation_id: inbound.payload.installation_id,
            },
            dependency_binding: inbound.payload.dependency_binding.clone(),
            functional_contract: inbound.payload.functional_contract.clone(),
            action: declaration.authorization_action.into(),
            operation: declaration.operation,
            resource_assertion: inbound.payload.resource_assertion.clone(),
            authorization_revision: revisions.0 as u64,
            organization_revision: revisions.1 as u64,
            now: Utc::now(),
        })
        .map_err(|_| restricted())?;
    let presenter = match &inbound.payload.presenting_service {
        ModuleServicePrincipalV1::ModuleInstance { .. } => {
            inbound.payload.presenting_service.clone()
        }
        ModuleServicePrincipalV1::CoreGateway => return Err(restricted()),
    };
    validate_for_principal(
        state,
        headers,
        &inbound,
        ModuleServiceRequestExpectation {
            principal: &presenter,
            grant_consumption: AuthorizationGrantConsumption::OneTimeProviderAudience(
                inbound.payload.jti,
            ),
            method: "POST",
            path,
            body,
        },
    )
    .await?;
    Ok(CoreProviderAuthorizationV1 {
        payload: CoreProviderAuthorizationPayloadV1 {
            installation_id: inbound.payload.installation_id,
            original_actor_id: inbound.payload.original_actor_id,
            presenting_service: inbound.payload.presenting_service,
            capability_scope_bindings: inbound.payload.capability_scope_bindings,
            delegation_basis: inbound.payload.delegation_basis,
        },
    })
}

struct BootstrapCoreProviderExpectation<'a> {
    functional_contract: &'a str,
    action: &'a str,
    path: &'a str,
    body: &'a [u8],
    required_capability: &'a str,
    restricted_message: &'static str,
}

async fn authorize_bootstrap_core_provider(
    state: &AppState,
    headers: &HeaderMap,
    expectation: BootstrapCoreProviderExpectation<'_>,
) -> ApiResult<CoreProviderAuthorizationV1> {
    let restricted = || ApiError::NotFound(expectation.restricted_message.into());
    let encoded_authorization = headers
        .get("x-tessara-owner-bootstrap-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(&restricted)?;
    let authorization_bytes = URL_SAFE_NO_PAD
        .decode(encoded_authorization)
        .map_err(|_| restricted())?;
    let authorization: SignedEnvelopeV1<OwnerBootstrapAuthorizationV1> =
        serde_json::from_slice(&authorization_bytes).map_err(|_| restricted())?;
    crate::core_security::protocol_signer(ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization)?
        .verifier()
        .verify(&authorization)
        .map_err(|_| restricted())?;
    let AuthorizationAudienceV1::ModuleInstance {
        module_instance_id,
        module_definition_id,
    } = &authorization.payload.owner
    else {
        return Err(restricted());
    };
    if authorization.payload.owner_definition_id != module_definition_id.as_str()
        || *module_instance_id
            != tessara_composition::module_instance_id(
                authorization.payload.installation_id,
                module_definition_id.as_str(),
            )
        || !authorization
            .payload
            .provider_actions
            .iter()
            .any(|provider_action| {
                provider_action.functional_contract == expectation.functional_contract
                    && provider_action.action == expectation.action
                    && provider_action.method == ServiceActionMethod::Post
                    && provider_action.path == expectation.path
                    && provider_action.audience
                        == (AuthorizationAudienceV1::CoreInstallation {
                            installation_id: authorization.payload.installation_id,
                        })
            })
        || !authorization
            .payload
            .capability_scope_bindings
            .iter()
            .any(|binding| binding.capability.as_str() == expectation.required_capability)
    {
        return Err(restricted());
    }
    authorization
        .payload
        .validate_for(&OwnerBootstrapAuthorizationContextV1 {
            installation_id: authorization.payload.installation_id,
            owner: &authorization.payload.owner,
            owner_definition_id: module_definition_id.as_str(),
            locked_input_digest: &authorization.payload.locked_input_digest,
            input_digest: &authorization.payload.input_digest,
            desired_revision: authorization.payload.desired_revision,
            apply_sequence: authorization.payload.apply_sequence,
            target_plan_digest: &authorization.payload.target_plan_digest,
            idempotency_key: &authorization.payload.idempotency_key,
            now: Utc::now(),
        })
        .map_err(|_| restricted())?;
    let lockfile_value: serde_json::Value = sqlx::query_scalar(
        "SELECT document FROM composition_lockfiles
         WHERE installation_id=$1 AND blueprint_revision=$2",
    )
    .bind(authorization.payload.installation_id)
    .bind(authorization.payload.desired_revision as i64)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(&restricted)?;
    let lockfile: ApplicationLockfileV1 =
        serde_json::from_value(lockfile_value).map_err(|_| restricted())?;
    let selected = lockfile
        .modules
        .iter()
        .find(|module| module.enabled && module.definition_id == module_definition_id.as_str())
        .ok_or_else(&restricted)?;
    let provider_action = authorization
        .payload
        .provider_actions
        .iter()
        .find(|provider_action| {
            provider_action.functional_contract == expectation.functional_contract
                && provider_action.action == expectation.action
                && provider_action.path == expectation.path
        })
        .ok_or_else(&restricted)?;
    let binding = selected
        .dependency_bindings
        .get(&provider_action.dependency_binding)
        .filter(|binding| {
            binding.provider == "core" && binding.contract_id == expectation.functional_contract
        })
        .ok_or_else(&restricted)?;
    let declared_version =
        crate::core_service_providers::contract_version(expectation.functional_contract)
            .ok_or_else(&restricted)?;
    let revisions = sqlx::query_as::<_, (i64, i64)>(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    if lockfile.materialization_plan_digest != authorization.payload.target_plan_digest
        || binding.contract_version != declared_version
        || revisions.0 as u64 != authorization.payload.authorization_revision
        || revisions.1 as u64 != authorization.payload.organization_revision
        || !lockfile.materialization_plan.actions.iter().any(|planned| {
            matches!(planned, MaterializationActionV1::Bootstrap { owner, input_digest }
                if owner == module_definition_id.as_str()
                    && input_digest == &authorization.payload.locked_input_digest)
        })
    {
        return Err(restricted());
    }
    let correlation_id = verified_correlation_header(headers, authorization.payload.correlation_id)
        .map_err(|_| restricted())?;
    let encoded_request = headers
        .get("x-tessara-module-service-request")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(&restricted)?;
    let request_bytes = URL_SAFE_NO_PAD
        .decode(encoded_request)
        .map_err(|_| restricted())?;
    let service_request: SignedEnvelopeV1<ModuleServiceRequestV1> =
        serde_json::from_slice(&request_bytes).map_err(|_| restricted())?;
    let registry = configured_registry()
        .map_err(ApiError::Internal)?
        .ok_or_else(&restricted)?;
    registry
        .module_service_verifier(module_definition_id)
        .map_err(|_| restricted())?
        .verify(&service_request)
        .map_err(|_| restricted())?;
    service_request
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id: authorization.payload.installation_id,
            module_instance_id: *module_instance_id,
            module_definition_id: module_definition_id.clone(),
            method: "POST".into(),
            path: expectation.path.into(),
            canonical_body_digest: sha256_hex(expectation.body),
            inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| restricted())?;
    let consumed = sqlx::query(
        "INSERT INTO consumed_module_service_nonces
           (module_instance_id,nonce,authorization_jti,correlation_id,issued_at)
         VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(service_request.payload.module_instance_id)
    .bind(service_request.payload.nonce)
    .bind(Option::<uuid::Uuid>::None)
    .bind(&service_request.payload.correlation_id)
    .bind(service_request.payload.issued_at)
    .execute(&state.pool)
    .await?;
    if consumed.rows_affected() != 1 {
        return Err(restricted());
    }
    Ok(CoreProviderAuthorizationV1 {
        payload: CoreProviderAuthorizationPayloadV1 {
            installation_id: authorization.payload.installation_id,
            original_actor_id: authorization.payload.original_actor_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: *module_instance_id,
                module_definition_id: module_definition_id.clone(),
            },
            capability_scope_bindings: authorization.payload.capability_scope_bindings,
            delegation_basis: Vec::new(),
        },
    })
}

pub(crate) struct ModuleServiceRequestExpectation<'a> {
    pub(crate) principal: &'a ModuleServicePrincipalV1,
    pub(crate) grant_consumption: AuthorizationGrantConsumption,
    pub(crate) method: &'a str,
    pub(crate) path: &'a str,
    pub(crate) body: &'a [u8],
}

pub(crate) fn configured_registry() -> anyhow::Result<Option<ModuleServiceIdentityRegistryV1>> {
    let Some(value) = std::env::var(MODULE_SERVICE_IDENTITIES_ENVIRONMENT)
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(None);
    };
    ModuleServiceIdentityRegistryV1::from_json(&value)
        .map(Some)
        .map_err(anyhow::Error::from)
}

pub(crate) fn require_json_content_type(headers: &HeaderMap) -> ApiResult<()> {
    let is_json = headers
        .get(axum::http::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .map(str::trim)
        .is_some_and(|media_type| {
            let media_type = media_type.to_ascii_lowercase();
            media_type == "application/json"
                || (media_type.starts_with("application/") && media_type.ends_with("+json"))
        });
    if !is_json {
        return Err(restricted_authorization());
    }
    Ok(())
}

/// Projects the source-exact key for one selected Module Instance. A module
/// that declares downstream actions must have a configured signing identity;
/// request-time traffic never creates or updates this row.
pub(crate) async fn project_service_identity(
    transaction: &mut Transaction<'_, Postgres>,
    registry: Option<&ModuleServiceIdentityRegistryV1>,
    module_instance_id: uuid::Uuid,
    manifest: &ModuleManifest,
) -> anyhow::Result<()> {
    let identity = registry.and_then(|registry| registry.identity(&manifest.definition_id));
    if identity.is_none() && !manifest.consumed_service_actions.is_empty() {
        anyhow::bail!(
            "selected module '{}' declares consumed service actions but has no materialization service identity",
            manifest.definition_id
        );
    }
    let Some(identity) = identity else {
        sqlx::query("DELETE FROM module_service_identities WHERE module_instance_id=$1")
            .bind(module_instance_id)
            .execute(&mut **transaction)
            .await?;
        return Ok(());
    };
    identity.public_key_bytes()?;
    let fingerprint = identity.public_key_fingerprint()?;
    sqlx::query(
        "INSERT INTO module_service_identities
           (module_instance_id,module_definition_id,key_id,public_key,public_key_fingerprint)
         VALUES ($1,$2,$3,$4,$5)
         ON CONFLICT (module_instance_id) DO UPDATE SET
           module_definition_id=EXCLUDED.module_definition_id,
           key_id=EXCLUDED.key_id,
           public_key=EXCLUDED.public_key,
           public_key_fingerprint=EXCLUDED.public_key_fingerprint,
           registered_at=now()",
    )
    .bind(module_instance_id)
    .bind(manifest.definition_id.as_str())
    .bind(&identity.key_id)
    .bind(&identity.public_key)
    .bind(&fingerprint)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

pub(crate) fn verified_authorization(
    headers: &HeaderMap,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV3>> {
    let encoded = authorization_header(headers)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| restricted_authorization())?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV3> =
        serde_json::from_slice(&bytes).map_err(|_| restricted_authorization())?;
    crate::core_security::protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .verifier()
        .verify(&envelope)
        .map_err(|_| restricted_authorization())?;
    Ok(envelope)
}

pub(crate) async fn validate_for_principal(
    state: &AppState,
    headers: &HeaderMap,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV3>,
    expectation: ModuleServiceRequestExpectation<'_>,
) -> ApiResult<()> {
    validate_for_principal_with_authorization(
        state,
        headers,
        inbound.payload.installation_id,
        inbound.payload.correlation_id,
        authorization_header(headers)?,
        expectation,
    )
    .await
}

pub(crate) async fn validate_for_principal_with_authorization(
    state: &AppState,
    headers: &HeaderMap,
    installation_id: uuid::Uuid,
    correlation_id: uuid::Uuid,
    encoded_authorization: &str,
    expectation: ModuleServiceRequestExpectation<'_>,
) -> ApiResult<()> {
    let ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id,
        module_definition_id,
    } = expectation.principal
    else {
        return Err(restricted_authorization());
    };
    let correlation_id = verified_correlation_header(headers, correlation_id)?;
    let encoded = headers
        .get("x-tessara-module-service-request")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted_authorization)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| restricted_authorization())?;
    let envelope: SignedEnvelopeV1<ModuleServiceRequestV1> =
        serde_json::from_slice(&bytes).map_err(|_| restricted_authorization())?;

    let identity = sqlx::query(
        "SELECT identity.key_id,identity.public_key,identity.public_key_fingerprint
         FROM module_service_identities identity
         JOIN module_instances instance ON instance.id=identity.module_instance_id
         WHERE identity.module_instance_id=$1
           AND identity.module_definition_id=$2
           AND instance.installation_id=$3
           AND instance.definition_id=identity.module_definition_id
           AND instance.identity_state='live' AND instance.installed=true
           AND instance.deployed=true AND instance.configured=true AND instance.enabled=true
           AND instance.ready=true AND instance.healthy=true",
    )
    .bind(module_instance_id)
    .bind(module_definition_id.as_str())
    .bind(installation_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let key_id: String = identity.try_get("key_id")?;
    let public_key_encoded: String = identity.try_get("public_key")?;
    let recorded_fingerprint: String = identity.try_get("public_key_fingerprint")?;
    let public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(&public_key_encoded)
        .map_err(|_| restricted_authorization())?
        .try_into()
        .map_err(|_| restricted_authorization())?;
    if recorded_fingerprint != format!("sha256:{}", sha256_hex(&public_key)) {
        return Err(restricted_authorization());
    }
    let verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        module_definition_id.as_str(),
        key_id,
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        public_key,
    )
    .map_err(|_| restricted_authorization())?;
    verifier
        .verify(&envelope)
        .map_err(|_| restricted_authorization())?;

    envelope
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id,
            module_instance_id: *module_instance_id,
            module_definition_id: module_definition_id.clone(),
            method: expectation.method.into(),
            path: expectation.path.into(),
            canonical_body_digest: sha256_hex(expectation.body),
            inbound_grant_digest: sha256_hex(encoded_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| restricted_authorization())?;

    let mut transaction = state.pool.begin().await?;
    let consumed = sqlx::query(
        "INSERT INTO consumed_module_service_nonces
           (module_instance_id,nonce,authorization_jti,correlation_id,issued_at)
         VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(envelope.payload.module_instance_id)
    .bind(envelope.payload.nonce)
    .bind(expectation.grant_consumption.authorization_jti())
    .bind(&envelope.payload.correlation_id)
    .bind(envelope.payload.issued_at)
    .execute(&mut *transaction)
    .await?;
    if consumed.rows_affected() != 1 {
        return Err(restricted_authorization());
    }
    transaction.commit().await?;
    Ok(())
}

#[cfg(test)]
fn materializing_instance_is_selected(
    installation_id: uuid::Uuid,
    module_definition_id: &tessara_module_contract::ModuleDefinitionId,
    module_instance_id: uuid::Uuid,
) -> bool {
    module_instance_id
        == tessara_composition::module_instance_id(installation_id, module_definition_id.as_str())
}

fn verified_correlation_header(
    headers: &HeaderMap,
    grant_correlation_id: uuid::Uuid,
) -> ApiResult<uuid::Uuid> {
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| uuid::Uuid::parse_str(value).ok())
        .filter(|value| !value.is_nil() && *value == grant_correlation_id)
        .ok_or_else(restricted_authorization)?;
    Ok(correlation_id)
}

fn authorization_header(headers: &HeaderMap) -> ApiResult<&str> {
    headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted_authorization)
}

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn restricted_authorization() -> ApiError {
    ApiError::Forbidden("module action unavailable".into())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use axum::http::{HeaderMap, HeaderValue};

    use super::{
        AuthorizationGrantConsumption, materializing_instance_is_selected,
        require_json_content_type, verified_correlation_header,
    };

    #[test]
    fn exact_body_routes_retain_json_content_type_enforcement() {
        for content_type in [
            "application/json",
            "application/json; charset=utf-8",
            "application/vnd.tessara+json",
        ] {
            let mut headers = HeaderMap::new();
            headers.insert(
                axum::http::header::CONTENT_TYPE,
                HeaderValue::from_static(content_type),
            );
            require_json_content_type(&headers).unwrap_or_else(|_| {
                panic!("valid JSON content type '{content_type}' was rejected")
            });
        }

        for content_type in [None, Some("text/json"), Some("application/octet-stream")] {
            let mut headers = HeaderMap::new();
            if let Some(content_type) = content_type {
                headers.insert(
                    axum::http::header::CONTENT_TYPE,
                    HeaderValue::from_static(content_type),
                );
            }
            assert!(require_json_content_type(&headers).is_err());
        }
    }

    #[test]
    fn correlation_header_must_match_the_inbound_grant_exactly() {
        let correlation_id = uuid::Uuid::new_v4();
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-tessara-correlation-id",
            HeaderValue::from_str(&correlation_id.to_string()).expect("correlation header"),
        );
        assert_eq!(
            verified_correlation_header(&headers, correlation_id).expect("matching correlation"),
            correlation_id
        );

        assert!(
            verified_correlation_header(&headers, uuid::Uuid::new_v4()).is_err(),
            "a caller cannot relabel the signed grant with another correlation"
        );
        headers.insert(
            "x-tessara-correlation-id",
            HeaderValue::from_static("00000000-0000-0000-0000-000000000000"),
        );
        assert!(verified_correlation_header(&headers, uuid::Uuid::nil()).is_err());
    }

    #[test]
    fn exchange_grants_are_reusable_but_provider_grants_have_one_time_replay_keys() {
        let grant_jti = uuid::Uuid::new_v4();
        let mut nonces = HashSet::new();
        let mut provider_grant_jtis = HashSet::new();
        let mut insert = |nonce, consumption: AuthorizationGrantConsumption| {
            nonces.insert(nonce)
                && consumption
                    .authorization_jti()
                    .is_none_or(|jti| provider_grant_jtis.insert(jti))
        };

        assert!(insert(
            uuid::Uuid::new_v4(),
            AuthorizationGrantConsumption::ReusableExchange
        ));
        assert!(
            insert(
                uuid::Uuid::new_v4(),
                AuthorizationGrantConsumption::ReusableExchange
            ),
            "the same route grant remains reusable for a fresh exchange service nonce"
        );
        assert!(insert(
            uuid::Uuid::new_v4(),
            AuthorizationGrantConsumption::OneTimeProviderAudience(grant_jti)
        ));
        assert!(
            !insert(
                uuid::Uuid::new_v4(),
                AuthorizationGrantConsumption::OneTimeProviderAudience(grant_jti)
            ),
            "a fresh service nonce cannot replay a consumed provider-audience grant"
        );
    }

    #[test]
    fn materializing_service_identity_is_bound_to_the_selected_instance() {
        let installation_id = uuid::Uuid::new_v4();
        let definition = tessara_module_contract::ModuleDefinitionId::new("example.materializer")
            .expect("materializing module definition");
        let selected =
            tessara_composition::module_instance_id(installation_id, definition.as_str());
        assert!(materializing_instance_is_selected(
            installation_id,
            &definition,
            selected
        ));
        assert!(!materializing_instance_is_selected(
            installation_id,
            &definition,
            uuid::Uuid::new_v4()
        ));
    }
}
