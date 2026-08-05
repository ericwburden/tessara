//! Generic verification for requests signed by an independently deployed
//! module service. Product adapters supply only the expected module identity,
//! key environment, method, path, and body; no module-specific branch lives in
//! this verifier.

use axum::http::HeaderMap;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV2, AuthorizationValidationContextV2,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ModuleServiceRequestV1,
    ModuleServiceRequestValidationContextV1, ProtocolSignaturePurposeV1,
    PurposeBoundVerifyingKeyV1, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

#[allow(clippy::too_many_arguments)]
pub(crate) async fn validate(
    state: &AppState,
    headers: &HeaderMap,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV2>,
    module_definition_id: &str,
    public_key_environment: &str,
    key_id_environment: &str,
    default_key_id: &str,
    method: &str,
    path: &str,
    body: &[u8],
) -> ApiResult<()> {
    validate_for_instance(
        state,
        headers,
        inbound,
        inbound.payload.audience_module_instance_id,
        module_definition_id,
        public_key_environment,
        key_id_environment,
        default_key_id,
        method,
        path,
        body,
    )
    .await
}

#[allow(clippy::too_many_arguments)]
pub(crate) async fn validate_for_instance(
    state: &AppState,
    headers: &HeaderMap,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV2>,
    caller_module_instance_id: Uuid,
    module_definition_id: &str,
    public_key_environment: &str,
    key_id_environment: &str,
    default_key_id: &str,
    method: &str,
    path: &str,
    body: &[u8],
) -> ApiResult<()> {
    let encoded = headers
        .get("x-tessara-module-service-request")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted_authorization)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| restricted_authorization())?;
    let envelope: SignedEnvelopeV1<ModuleServiceRequestV1> =
        serde_json::from_slice(&bytes).map_err(|_| restricted_authorization())?;

    let public_key_encoded =
        std::env::var(public_key_environment).map_err(|_| restricted_authorization())?;
    let public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(&public_key_encoded)
        .map_err(|_| restricted_authorization())?
        .try_into()
        .map_err(|_| restricted_authorization())?;
    let key_id = std::env::var(key_id_environment).unwrap_or_else(|_| default_key_id.into());
    let verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        module_definition_id,
        key_id.clone(),
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        public_key,
    )
    .map_err(|_| restricted_authorization())?;
    verifier
        .verify(&envelope)
        .map_err(|_| restricted_authorization())?;

    let authorization = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted_authorization)?;
    envelope
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id: inbound.payload.installation_id,
            module_instance_id: caller_module_instance_id,
            module_definition_id: ModuleDefinitionId::new(module_definition_id)
                .map_err(|error| ApiError::Internal(error.into()))?,
            method: method.into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(authorization.as_bytes()),
            now: Utc::now(),
        })
        .map_err(|_| restricted_authorization())?;

    let instance_matches: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM module_instances
         WHERE id=$1 AND installation_id=$2 AND definition_id=$3
           AND identity_state='live' AND installed=true AND deployed=true
           AND configured=true AND ready=true AND enabled=true AND healthy=true)",
    )
    .bind(envelope.payload.module_instance_id)
    .bind(envelope.payload.installation_id)
    .bind(module_definition_id)
    .fetch_one(&state.pool)
    .await?;
    if !instance_matches {
        return Err(restricted_authorization());
    }

    let fingerprint = format!("sha256:{}", sha256_hex(&public_key));
    let mut transaction = state.pool.begin().await?;
    sqlx::query(
        "INSERT INTO module_service_identities
           (module_instance_id,module_definition_id,key_id,public_key,public_key_fingerprint)
         VALUES ($1,$2,$3,$4,$5) ON CONFLICT (module_instance_id) DO NOTHING",
    )
    .bind(envelope.payload.module_instance_id)
    .bind(module_definition_id)
    .bind(&key_id)
    .bind(&public_key_encoded)
    .bind(&fingerprint)
    .execute(&mut *transaction)
    .await?;
    let identity_matches: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM module_service_identities
         WHERE module_instance_id=$1 AND module_definition_id=$2 AND key_id=$3
           AND public_key=$4 AND public_key_fingerprint=$5)",
    )
    .bind(envelope.payload.module_instance_id)
    .bind(module_definition_id)
    .bind(&key_id)
    .bind(&public_key_encoded)
    .bind(&fingerprint)
    .fetch_one(&mut *transaction)
    .await?;
    if !identity_matches {
        return Err(restricted_authorization());
    }
    let consumed = sqlx::query(
        "INSERT INTO consumed_module_service_nonces
           (module_instance_id,nonce,correlation_id,issued_at)
         VALUES ($1,$2,$3,$4) ON CONFLICT DO NOTHING",
    )
    .bind(envelope.payload.module_instance_id)
    .bind(envelope.payload.nonce)
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

#[allow(clippy::too_many_arguments)]
pub(crate) async fn validate_inbound_grant(
    state: &AppState,
    headers: &HeaderMap,
    audience_module_definition_id: &str,
    dependency_binding: &str,
    action_contract: fn(&str) -> Option<(&'static str, AuthorizationGrantOperationV1)>,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV2>> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted_authorization)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| restricted_authorization())?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV2> =
        serde_json::from_slice(&bytes).map_err(|_| restricted_authorization())?;
    let (functional_contract, operation) =
        action_contract(&envelope.payload.action).ok_or_else(restricted_authorization)?;
    crate::core_security::protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .verifier()
        .verify(&envelope)
        .map_err(|_| restricted_authorization())?;
    let instance = sqlx::query(
        "SELECT installation_id FROM module_instances
         WHERE id=$1 AND definition_id=$2 AND identity_state='live'
           AND installed=true AND deployed=true AND configured=true
           AND ready=true AND enabled=true AND healthy=true",
    )
    .bind(envelope.payload.audience_module_instance_id)
    .bind(audience_module_definition_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let installation_id: Uuid = instance.try_get("installation_id")?;
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV2 {
            installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: envelope.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(dependency_binding)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(functional_contract)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: envelope.payload.action.clone(),
            operation,
            resource_assertion: None,
            authorization_revision: revisions.try_get::<i64, _>("authorization_revision")? as u64,
            organization_revision: revisions.try_get::<i64, _>("organization_revision")? as u64,
            now: Utc::now(),
        })
        .map_err(|_| restricted_authorization())?;
    Ok(envelope)
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
