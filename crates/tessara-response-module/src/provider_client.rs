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

const PROVIDER_RESPONSE_LIMIT_BYTES: usize = 1024 * 1024;

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
    let response = match runtime
        .provider_client
        .post(format!("{provider_url}{}", target.path))
        .timeout(Duration::from_secs(timeout_seconds))
        .header("content-type", target.media_type)
        .header("accept", target.media_type)
        .header("x-tessara-authorization", &downstream.encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            downstream.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
    {
        Ok(response) => response,
        Err(_) => {
            record_provider_observation(runtime, target.binding, ProviderClientError::Unavailable)
                .await;
            return Err(ProviderClientError::Unavailable);
        }
    };
    if !response.status().is_success() {
        let error = if response.status().is_server_error() {
            ProviderClientError::Unavailable
        } else {
            ProviderClientError::Restricted
        };
        record_provider_observation(runtime, target.binding, error).await;
        return Err(error);
    }
    if response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(target.media_type)
    {
        record_provider_observation(runtime, target.binding, ProviderClientError::Incompatible)
            .await;
        return Err(ProviderClientError::Incompatible);
    }
    let bytes = match bounded_response_body(response).await {
        Ok(bytes) => bytes,
        Err(error) => {
            record_provider_observation(runtime, target.binding, error).await;
            return Err(error);
        }
    };
    let decoded = serde_json::from_slice(&bytes).map_err(|_| ProviderClientError::Incompatible);
    match decoded {
        Ok(value) => Ok(value),
        Err(error) => {
            record_provider_observation(runtime, target.binding, error).await;
            Err(error)
        }
    }
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
        .header("accept", "application/json")
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
    if response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return Err(ProviderClientError::Incompatible);
    }
    let response: AuthorizationExchangeResponseV2 =
        serde_json::from_slice(&bounded_response_body(response).await?)
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

pub(crate) async fn bounded_response_body(
    mut response: reqwest::Response,
) -> Result<Vec<u8>, ProviderClientError> {
    if !declared_response_length_is_bounded(response.content_length()) {
        return Err(ProviderClientError::Incompatible);
    }
    let mut body = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|_| ProviderClientError::Unavailable)?
    {
        append_bounded_response_chunk(&mut body, &chunk)?;
    }
    Ok(body)
}

fn declared_response_length_is_bounded(length: Option<u64>) -> bool {
    length.is_none_or(|length| length <= PROVIDER_RESPONSE_LIMIT_BYTES as u64)
}

fn append_bounded_response_chunk(
    body: &mut Vec<u8>,
    chunk: &[u8],
) -> Result<(), ProviderClientError> {
    if body.len().saturating_add(chunk.len()) > PROVIDER_RESPONSE_LIMIT_BYTES {
        return Err(ProviderClientError::Incompatible);
    }
    body.extend_from_slice(chunk);
    Ok(())
}

pub(crate) async fn record_provider_compatible(runtime: &ResponseRuntime, binding: &str) {
    let _ = sqlx::query(
        "UPDATE response_provider_observations
         SET compatibility_state='compatible',last_observed_at=now(),last_compatible_at=now(),
             last_stable_finding=NULL
         WHERE binding_key=$1",
    )
    .bind(binding)
    .execute(&runtime.pool)
    .await;
}

pub(crate) async fn record_provider_observation(
    runtime: &ResponseRuntime,
    binding: &str,
    error: ProviderClientError,
) {
    let Some((state, finding)) = provider_observation(binding, error) else {
        return;
    };
    let _ = sqlx::query(
        "UPDATE response_provider_observations
         SET compatibility_state=$2,last_observed_at=now(),last_stable_finding=$3
         WHERE binding_key=$1",
    )
    .bind(binding)
    .bind(state)
    .bind(finding)
    .execute(&runtime.pool)
    .await;
}

fn provider_observation(
    binding: &str,
    error: ProviderClientError,
) -> Option<(&'static str, &'static str)> {
    match (binding, error) {
        (_, ProviderClientError::Restricted | ProviderClientError::Internal) => None,
        (crate::RESPONSE_FORM_BINDING, ProviderClientError::Unavailable) => {
            Some(("unavailable", "response.provider.forms.unavailable"))
        }
        (crate::RESPONSE_FORM_BINDING, ProviderClientError::Incompatible) => {
            Some(("incompatible", "response.provider.forms.incompatible"))
        }
        (
            crate::RESPONSE_WORKFLOW_CONTEXT_BINDING | crate::RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
            ProviderClientError::Unavailable,
        ) => Some(("unavailable", "response.provider.workflow.unavailable")),
        (
            crate::RESPONSE_WORKFLOW_CONTEXT_BINDING | crate::RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
            ProviderClientError::Incompatible,
        ) => Some(("incompatible", "response.provider.workflow.incompatible")),
        _ => None,
    }
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
    signed_service_request_with_context(
        runtime,
        authorization.as_bytes(),
        path,
        body,
        installation_id,
        module_instance_id,
        correlation_id,
    )
}

pub(crate) fn signed_operational_request(
    runtime: &ResponseRuntime,
    path: &str,
    body: &[u8],
    installation_id: Uuid,
    module_instance_id: Uuid,
    correlation_id: Uuid,
) -> Result<String, ProviderClientError> {
    signed_service_request_with_context(
        runtime,
        tessara_module_contract::MODULE_PROVIDER_COMPATIBILITY_SERVICE_CONTEXT.as_bytes(),
        path,
        body,
        installation_id,
        module_instance_id,
        correlation_id,
    )
}

fn signed_service_request_with_context(
    runtime: &ResponseRuntime,
    inbound_context: &[u8],
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
        inbound_grant_digest: sha256_hex(inbound_context),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_response_bound_rejects_declared_and_streamed_overflow() {
        assert!(declared_response_length_is_bounded(None));
        assert!(declared_response_length_is_bounded(Some(
            PROVIDER_RESPONSE_LIMIT_BYTES as u64
        )));
        assert!(!declared_response_length_is_bounded(Some(
            PROVIDER_RESPONSE_LIMIT_BYTES as u64 + 1
        )));

        let mut body = vec![0; PROVIDER_RESPONSE_LIMIT_BYTES - 1];
        append_bounded_response_chunk(&mut body, &[1]).expect("exact response limit");
        assert_eq!(body.len(), PROVIDER_RESPONSE_LIMIT_BYTES);
        assert_eq!(
            append_bounded_response_chunk(&mut body, &[2]),
            Err(ProviderClientError::Incompatible)
        );
    }

    #[test]
    fn provider_observation_is_sanitized_and_binding_specific() {
        assert_eq!(
            provider_observation(
                crate::RESPONSE_FORM_BINDING,
                ProviderClientError::Incompatible,
            ),
            Some(("incompatible", "response.provider.forms.incompatible"))
        );
        assert_eq!(
            provider_observation(
                crate::RESPONSE_WORKFLOW_CONTEXT_BINDING,
                ProviderClientError::Unavailable,
            ),
            Some(("unavailable", "response.provider.workflow.unavailable"))
        );
        assert_eq!(
            provider_observation(
                crate::RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                ProviderClientError::Restricted,
            ),
            None
        );
        assert_eq!(
            provider_observation(
                "tessara.responses.unknown",
                ProviderClientError::Unavailable
            ),
            None
        );
    }
}
