use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use tessara_module_contract::{AuthorizationGrantV2, ModuleServiceRequestV1, SignedEnvelopeV1};
use uuid::Uuid;

use crate::{ComponentModuleError, ComponentModuleState};

pub(super) async fn post<TRequest, TResponse>(
    state: &ComponentModuleState,
    authorization: &str,
    path: &'static str,
    request: &TRequest,
) -> Result<TResponse, ComponentModuleError>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
{
    let body = serde_json::to_vec(request)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let service_request = signed_service_request(state, authorization, path, &body)?;
    let timeout_seconds: i32 = sqlx::query_scalar(
        "SELECT dataset_request_timeout_seconds FROM component_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let response = state
        .dataset_client
        .post(format!("{}{}", state.core_internal_url, path))
        .timeout(Duration::from_secs(timeout_seconds as u64))
        .header("content-type", "application/json")
        .header("x-tessara-authorization", authorization)
        .header("x-tessara-module-service-request", service_request)
        .body(body)
        .send()
        .await
        .map_err(|error| {
            observe(
                state,
                "unavailable",
                Some(if error.is_timeout() {
                    "dataset.timeout"
                } else {
                    "dataset.unavailable"
                }),
            );
            tracing::warn!(%error, path, "Dataset provider request failed");
            ComponentModuleError::Unavailable(
                "Dataset provider is unavailable; retry the request".into(),
            )
        })?;
    if !response.status().is_success() {
        observe(
            state,
            "rejected",
            Some(if response.status().is_server_error() {
                "dataset.unavailable"
            } else {
                "dataset.restricted"
            }),
        );
        tracing::warn!(status = %response.status(), path, "Dataset provider rejected request");
        return Err(if response.status().is_server_error() {
            ComponentModuleError::Unavailable(
                "Dataset provider is unavailable; retry the request".into(),
            )
        } else {
            ComponentModuleError::Forbidden
        });
    }
    let decoded = response.json().await.map_err(|error| {
        observe(state, "invalid_response", Some("dataset.invalid_response"));
        tracing::warn!(%error, path, "Dataset provider returned an invalid contract envelope");
        ComponentModuleError::Unavailable("Dataset provider response is invalid".into())
    })?;
    observe(state, "available", None);
    Ok(decoded)
}

fn observe(state: &ComponentModuleState, status: &'static str, failure_code: Option<&'static str>) {
    if let Ok(mut observation) = state.dataset_health.write() {
        observation.status = status;
        observation.failure_code = failure_code;
        observation.observed_at = Some(Utc::now());
    }
}

pub(super) async fn exchange_authorization(
    state: &ComponentModuleState,
    inbound_authorization: &str,
    request: &tessara_module_contract::AuthorizationExchangeRequestV1,
    caller_module_instance_id: Uuid,
) -> Result<tessara_module_contract::AuthorizationExchangeResponseV1, ComponentModuleError> {
    let path = "/api/private/module-authorization/exchange";
    let body = serde_json::to_vec(request)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let service_request = signed_service_request_for_instance(
        state,
        inbound_authorization,
        path,
        &body,
        caller_module_instance_id,
    )?;
    let response = state
        .dataset_client
        .post(format!("{}{}", state.core_internal_url, path))
        .timeout(Duration::from_secs(5))
        .header("content-type", "application/json")
        .header("x-tessara-authorization", inbound_authorization)
        .header("x-tessara-module-service-request", service_request)
        .body(body)
        .send()
        .await
        .map_err(|_| {
            ComponentModuleError::Unavailable("Core authorization exchange is unavailable".into())
        })?;
    if !response.status().is_success() {
        return Err(ComponentModuleError::Forbidden);
    }
    response.json().await.map_err(|_| {
        ComponentModuleError::Unavailable("Core authorization exchange response is invalid".into())
    })
}

fn signed_service_request(
    state: &ComponentModuleState,
    authorization: &str,
    path: &str,
    body: &[u8],
) -> Result<String, ComponentModuleError> {
    let grant: SignedEnvelopeV1<AuthorizationGrantV2> = decode(authorization)?;
    signed_service_request_for_instance(
        state,
        authorization,
        path,
        body,
        grant.payload.audience_module_instance_id,
    )
}

fn signed_service_request_for_instance(
    state: &ComponentModuleState,
    authorization: &str,
    path: &str,
    body: &[u8],
    caller_module_instance_id: Uuid,
) -> Result<String, ComponentModuleError> {
    let grant: SignedEnvelopeV1<AuthorizationGrantV2> = decode(authorization)?;
    let now = Utc::now();
    let request = ModuleServiceRequestV1 {
        schema_version: 1,
        installation_id: grant.payload.installation_id,
        module_instance_id: caller_module_instance_id,
        module_definition_id: crate::MODULE_DEFINITION_ID.parse().map_err(|_| {
            ComponentModuleError::Internal("Component service identity is invalid".into())
        })?,
        method: "POST".into(),
        path: path.into(),
        canonical_body_digest: sha256_hex(body),
        inbound_grant_digest: sha256_hex(authorization.as_bytes()),
        correlation_id: Uuid::new_v4().to_string(),
        nonce: Uuid::new_v4(),
        issued_at: now,
        expires_at: now + ChronoDuration::seconds(30),
    };
    let envelope = state.service_request_signer.sign(request).map_err(|_| {
        ComponentModuleError::Internal("Component service request signing failed".into())
    })?;
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn decode<T: DeserializeOwned>(value: &str) -> Result<T, ComponentModuleError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    serde_json::from_slice(&bytes).map_err(|_| ComponentModuleError::Forbidden)
}

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
