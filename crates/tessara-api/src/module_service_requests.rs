//! Generic verification and materialization-time enrollment for requests
//! signed by independently deployed module services.

use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::{Postgres, Row, Transaction};
use tessara_module_contract::{
    AuthorizationGrantV3, MODULE_SERVICE_IDENTITIES_ENVIRONMENT, ModuleManifest,
    ModuleServiceIdentityRegistryV1, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, ProtocolSignaturePurposeV1,
    PurposeBoundVerifyingKeyV1, SignedEnvelopeV1,
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

impl AuthorizationGrantConsumption {
    fn authorization_jti(self) -> Option<uuid::Uuid> {
        match self {
            Self::ReusableExchange => None,
            Self::OneTimeProviderAudience(jti) => Some(jti),
        }
    }
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
    let ModuleServicePrincipalV1::ModuleInstance {
        module_instance_id,
        module_definition_id,
    } = expectation.principal
    else {
        return Err(restricted_authorization());
    };
    let correlation_id = verified_correlation_header(headers, inbound.payload.correlation_id)?;
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
           AND instance.deployed=true AND instance.configured=true
           AND instance.ready=true AND instance.enabled=true AND instance.healthy=true",
    )
    .bind(module_instance_id)
    .bind(module_definition_id.as_str())
    .bind(inbound.payload.installation_id)
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
            installation_id: inbound.payload.installation_id,
            module_instance_id: *module_instance_id,
            module_definition_id: module_definition_id.clone(),
            method: expectation.method.into(),
            path: expectation.path.into(),
            canonical_body_digest: sha256_hex(expectation.body),
            inbound_grant_digest: sha256_hex(authorization_header(headers)?.as_bytes()),
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

    use super::{AuthorizationGrantConsumption, verified_correlation_header};

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
}
