use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AuthorizationAudienceV1,
    AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, AuthorizationValidationContextV3, DependencyBindingKey,
    FunctionalContractId, ModuleDefinitionId, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    SignedEnvelopeV1,
};
use tessara_module_runtime::SecurityStateProvider;
use uuid::Uuid;

use crate::{MODULE_DEFINITION_ID, ResponseRuntime};

#[derive(Clone, Copy)]
pub(crate) struct ProviderAction {
    pub binding: &'static str,
    pub contract: &'static str,
    pub action: &'static str,
    pub path: &'static str,
    pub media_type: &'static str,
}

pub(crate) async fn post<TRequest, TResponse>(
    runtime: &ResponseRuntime,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV3>,
    target: ProviderAction,
    request: &TRequest,
) -> Result<TResponse, ProviderClientError>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    let body = serde_json::to_vec(request).map_err(|_| ProviderClientError::Internal)?;
    let downstream = exchange(runtime, inbound, target).await?;
    let service_request = signed_service_request(
        runtime,
        &downstream.encoded,
        target.path,
        &body,
        downstream.installation_id,
        downstream.module_instance_id,
        downstream.correlation_id,
    )?;
    let provider_url = runtime
        .service_endpoints
        .provider_url(target.binding)
        .ok_or(ProviderClientError::Internal)?;
    let timeout_seconds: i16 = sqlx::query_scalar(
        "SELECT provider_request_timeout_seconds FROM response_module_configuration WHERE singleton=true",
    )
    .fetch_one(&runtime.pool)
    .await
    .map_err(|_| ProviderClientError::Unavailable)?;
    let timeout_seconds = u64::try_from(timeout_seconds)
        .ok()
        .filter(|seconds| (1..=30).contains(seconds))
        .ok_or(ProviderClientError::Internal)?;
    let response = runtime
        .provider_client
        .post(format!("{provider_url}{}", target.path))
        .timeout(Duration::from_secs(timeout_seconds))
        .header("content-type", target.media_type)
        .header("x-tessara-authorization", &downstream.encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            downstream.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
        .map_err(|_| ProviderClientError::Unavailable)?;
    if !response.status().is_success() {
        return Err(if response.status().is_server_error() {
            ProviderClientError::Unavailable
        } else {
            ProviderClientError::Restricted
        });
    }
    if response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(target.media_type)
    {
        return Err(ProviderClientError::Incompatible);
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|_| ProviderClientError::Unavailable)?;
    serde_json::from_slice(&bytes).map_err(|_| ProviderClientError::Incompatible)
}

#[derive(Clone)]
struct DownstreamAuthorization {
    encoded: String,
    installation_id: Uuid,
    module_instance_id: Uuid,
    correlation_id: Uuid,
}

async fn exchange(
    runtime: &ResponseRuntime,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV3>,
    target_action: ProviderAction,
) -> Result<DownstreamAuthorization, ProviderClientError> {
    let security = runtime
        .current_security_state()
        .await
        .map_err(|_| ProviderClientError::Unavailable)?;
    let definition =
        ModuleDefinitionId::new(MODULE_DEFINITION_ID).map_err(|_| ProviderClientError::Internal)?;
    let target = AuthorizationAudienceV1::CoreInstallation {
        installation_id: security.installation_id,
    };
    let request = AuthorizationExchangeRequestV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        target: target.clone(),
        dependency_binding: DependencyBindingKey::new(target_action.binding)
            .map_err(|_| ProviderClientError::Internal)?,
        functional_contract: FunctionalContractId::new(target_action.contract)
            .map_err(|_| ProviderClientError::Internal)?,
        action: target_action.action.into(),
        resource_assertion: None,
    };
    request
        .validate()
        .map_err(|_| ProviderClientError::Restricted)?;
    let inbound_encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(inbound).map_err(|_| ProviderClientError::Internal)?);
    let path = "/api/private/module-authorization/exchange";
    let body = serde_json::to_vec(&request).map_err(|_| ProviderClientError::Internal)?;
    let service_request = signed_service_request(
        runtime,
        &inbound_encoded,
        path,
        &body,
        security.installation_id,
        security.module_instance_id,
        inbound.payload.correlation_id,
    )?;
    let response = runtime
        .provider_client
        .post(format!(
            "{}{}",
            runtime.service_endpoints.core_authorization_url(),
            path
        ))
        .timeout(Duration::from_secs(5))
        .header("content-type", "application/json")
        .header("x-tessara-authorization", &inbound_encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            inbound.payload.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
        .map_err(|_| ProviderClientError::Unavailable)?;
    if !response.status().is_success() {
        return Err(if response.status().is_server_error() {
            ProviderClientError::Unavailable
        } else {
            ProviderClientError::Restricted
        });
    }
    let response: AuthorizationExchangeResponseV2 = response
        .json()
        .await
        .map_err(|_| ProviderClientError::Incompatible)?;
    if response.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2 {
        return Err(ProviderClientError::Incompatible);
    }
    runtime
        .verifiers
        .authorization
        .verify(&response.authorization)
        .map_err(|_| ProviderClientError::Restricted)?;
    response
        .authorization
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: definition,
            },
            audience: target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: target_action.action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision,
            organization_revision: security.organization_revision,
            now: Utc::now(),
        })
        .map_err(|_| ProviderClientError::Restricted)?;
    if response.authorization.payload.original_actor_id != inbound.payload.original_actor_id {
        return Err(ProviderClientError::Restricted);
    }
    Ok(DownstreamAuthorization {
        encoded: URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&response.authorization)
                .map_err(|_| ProviderClientError::Internal)?,
        ),
        installation_id: security.installation_id,
        module_instance_id: security.module_instance_id,
        correlation_id: response.authorization.payload.correlation_id,
    })
}

fn signed_service_request(
    runtime: &ResponseRuntime,
    authorization: &str,
    path: &str,
    body: &[u8],
    installation_id: Uuid,
    module_instance_id: Uuid,
    correlation_id: Uuid,
) -> Result<String, ProviderClientError> {
    let now = Utc::now();
    let request = ModuleServiceRequestV1 {
        schema_version: 1,
        installation_id,
        module_instance_id,
        module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
            .map_err(|_| ProviderClientError::Internal)?,
        method: "POST".into(),
        path: path.into(),
        canonical_body_digest: sha256_hex(body),
        inbound_grant_digest: sha256_hex(authorization.as_bytes()),
        correlation_id: correlation_id.to_string(),
        nonce: Uuid::new_v4(),
        issued_at: now,
        expires_at: now + ChronoDuration::seconds(30),
    };
    let envelope = runtime
        .service_request_signer
        .sign(request)
        .map_err(|_| ProviderClientError::Internal)?;
    Ok(URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&envelope).map_err(|_| ProviderClientError::Internal)?))
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub(crate) enum ProviderClientError {
    #[error("provider request is restricted")]
    Restricted,
    #[error("provider is unavailable")]
    Unavailable,
    #[error("provider contract is incompatible")]
    Incompatible,
    #[error("provider client failed")]
    Internal,
}
