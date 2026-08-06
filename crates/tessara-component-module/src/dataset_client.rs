use std::time::Duration;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use tessara_datasets_contract::{
    DATASET_BINDING_KEY, DATASET_CONTRACT_ID, DATASET_CONTRACT_VERSION,
};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AuthorizationAudienceV1,
    AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, AuthorizationValidationContextV3, DependencyBindingKey,
    FunctionalContractId, ModuleDefinitionId, ModuleServicePrincipalV1, ModuleServiceRequestV1,
    SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{ComponentModuleError, ComponentModuleState, load_security_state};

struct DownstreamAuthorization {
    encoded: String,
    installation_id: Uuid,
    caller_module_instance_id: Uuid,
    correlation_id: Uuid,
    original_actor_id: Uuid,
}

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
    let action = dataset_action(path, request)?;
    let reference_digest = request_reference_digest(request)?;
    let downstream = exchange_dataset_authorization(state, authorization, action).await?;
    let service_request = signed_service_request(state, &downstream, path, &body)?;
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
        .header("x-tessara-authorization", &downstream.encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            downstream.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
        .map_err(|error| {
            let (failure_code, result_code) = if error.is_timeout() {
                ("dataset.timeout", "dataset.request_timeout")
            } else {
                ("dataset.unavailable", "dataset.provider_unavailable")
            };
            observe(
                state,
                path,
                "unavailable",
                Some(failure_code),
                result_code,
                None,
            );
            tracing::warn!(
                correlation_id = %downstream.correlation_id,
                original_actor_id = %downstream.original_actor_id,
                actor_class = "user",
                caller_service_class = "module_instance",
                installation_id = %downstream.installation_id,
                module_instance_id = %downstream.caller_module_instance_id,
                module_definition_id = crate::MODULE_DEFINITION_ID,
                module_release = crate::MODULE_RELEASE_VERSION,
                dependency_binding = DATASET_BINDING_KEY,
                functional_contract = DATASET_CONTRACT_ID,
                contract_version = DATASET_CONTRACT_VERSION,
                provider_owner_kind = "core_installation",
                reference_digest,
                timeout_seconds,
                action,
                path,
                result_code,
                "Dataset dependency request failed"
            );
            ComponentModuleError::Unavailable(
                "Dataset provider is unavailable; retry the request".into(),
            )
        })?;
    if !response.status().is_success() {
        let response_status = response.status();
        let (failure_code, result_code) = if response_status.is_server_error() {
            ("dataset.unavailable", "dataset.provider_rejected")
        } else {
            ("dataset.restricted", "dataset.request_restricted")
        };
        observe(
            state,
            path,
            "rejected",
            Some(failure_code),
            result_code,
            None,
        );
        tracing::warn!(
            correlation_id = %downstream.correlation_id,
            original_actor_id = %downstream.original_actor_id,
            actor_class = "user",
            caller_service_class = "module_instance",
            installation_id = %downstream.installation_id,
            module_instance_id = %downstream.caller_module_instance_id,
            module_definition_id = crate::MODULE_DEFINITION_ID,
            module_release = crate::MODULE_RELEASE_VERSION,
            dependency_binding = DATASET_BINDING_KEY,
            functional_contract = DATASET_CONTRACT_ID,
            contract_version = DATASET_CONTRACT_VERSION,
            provider_owner_kind = "core_installation",
            reference_digest,
            timeout_seconds,
            action,
            path,
            response_status = %response_status,
            result_code,
            "Dataset dependency request was rejected"
        );
        return Err(if response_status.is_server_error() {
            ComponentModuleError::Unavailable(
                "Dataset provider is unavailable; retry the request".into(),
            )
        } else {
            ComponentModuleError::Forbidden
        });
    }
    let response_wire: serde_json::Value = response.json().await.map_err(|_| {
        observe(
            state,
            path,
            "invalid_response",
            Some("dataset.invalid_response"),
            "dataset.invalid_response",
            None,
        );
        tracing::warn!(
            correlation_id = %downstream.correlation_id,
            original_actor_id = %downstream.original_actor_id,
            actor_class = "user",
            caller_service_class = "module_instance",
            installation_id = %downstream.installation_id,
            module_instance_id = %downstream.caller_module_instance_id,
            module_definition_id = crate::MODULE_DEFINITION_ID,
            module_release = crate::MODULE_RELEASE_VERSION,
            dependency_binding = DATASET_BINDING_KEY,
            functional_contract = DATASET_CONTRACT_ID,
            contract_version = DATASET_CONTRACT_VERSION,
            provider_owner_kind = "core_installation",
            reference_digest,
            timeout_seconds,
            action,
            path,
            result_code = "dataset.invalid_response",
            "Dataset dependency returned an invalid contract envelope"
        );
        ComponentModuleError::Unavailable("Dataset provider response is invalid".into())
    })?;
    let compatible = (path == "/api/private/datasets/compatibility")
        .then(|| {
            response_wire
                .get("compatible")
                .and_then(serde_json::Value::as_bool)
        })
        .flatten();
    let result_code = if compatible == Some(false) {
        "dataset.compatibility_rejected"
    } else {
        "dataset.request_succeeded"
    };
    observe(state, path, "available", None, result_code, compatible);
    tracing::info!(
        correlation_id = %downstream.correlation_id,
        original_actor_id = %downstream.original_actor_id,
        actor_class = "user",
        caller_service_class = "module_instance",
        installation_id = %downstream.installation_id,
        module_instance_id = %downstream.caller_module_instance_id,
        module_definition_id = crate::MODULE_DEFINITION_ID,
        module_release = crate::MODULE_RELEASE_VERSION,
        dependency_binding = DATASET_BINDING_KEY,
        functional_contract = DATASET_CONTRACT_ID,
        contract_version = DATASET_CONTRACT_VERSION,
        provider_owner_kind = "core_installation",
        reference_digest,
        timeout_seconds,
        action,
        path,
        result_code,
        "Dataset dependency request completed"
    );
    serde_json::from_value(response_wire).map_err(|_| {
        observe(
            state,
            path,
            "invalid_response",
            Some("dataset.invalid_response"),
            "dataset.invalid_response",
            None,
        );
        ComponentModuleError::Unavailable("Dataset provider response is invalid".into())
    })
}

fn dataset_action<TRequest: Serialize>(
    path: &str,
    request: &TRequest,
) -> Result<&'static str, ComponentModuleError> {
    let (expected_wire_action, authorization_action) = match path {
        "/api/private/datasets/catalog" => ("catalog", "datasets.catalog"),
        "/api/private/datasets/schema" => ("resolve_schema", "datasets.schema"),
        "/api/private/datasets/distinct-values" => ("distinct_values", "datasets.distinct_values"),
        "/api/private/datasets/compatibility" => ("check_compatibility", "datasets.compatibility"),
        "/api/private/datasets/execute" => ("execute", "datasets.execute"),
        _ => return Err(ComponentModuleError::Forbidden),
    };
    let wire = serde_json::to_value(request)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    if wire.get("action").and_then(serde_json::Value::as_str) != Some(expected_wire_action) {
        return Err(ComponentModuleError::Forbidden);
    }
    Ok(authorization_action)
}

fn request_reference_digest<TRequest: Serialize>(
    request: &TRequest,
) -> Result<String, ComponentModuleError> {
    let wire = serde_json::to_value(request)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let encoded = serde_json::to_vec(wire.get("reference").unwrap_or(&serde_json::Value::Null))
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    Ok(format!("sha256:{}", sha256_hex(&encoded)))
}

fn observe(
    state: &ComponentModuleState,
    path: &str,
    status: &'static str,
    failure_code: Option<&'static str>,
    result_code: &'static str,
    compatible: Option<bool>,
) {
    if let Ok(mut observation) = state.dataset_health.write() {
        update_observation(
            &mut observation,
            path,
            status,
            failure_code,
            result_code,
            compatible,
            Utc::now(),
        );
    }
}

fn update_observation(
    observation: &mut crate::DatasetHealthObservation,
    path: &str,
    status: &'static str,
    failure_code: Option<&'static str>,
    result_code: &'static str,
    compatible: Option<bool>,
    observed_at: chrono::DateTime<Utc>,
) {
    observation.status = status;
    observation.failure_code = failure_code;
    observation.result_code = result_code;
    observation.observed_at = Some(observed_at);
    if path == "/api/private/datasets/compatibility" {
        observation.compatibility.status = match compatible {
            Some(true) => "compatible",
            Some(false) => "incompatible",
            None => status,
        };
        observation.compatibility.failure_code = if compatible == Some(false) {
            Some("dataset.incompatible")
        } else {
            failure_code
        };
        observation.compatibility.compatible = compatible;
        observation.compatibility.observed_at = Some(observed_at);
    }
}

async fn exchange_dataset_authorization(
    state: &ComponentModuleState,
    inbound_authorization: &str,
    action: &str,
) -> Result<DownstreamAuthorization, ComponentModuleError> {
    let inbound: SignedEnvelopeV1<AuthorizationGrantV3> = decode(inbound_authorization)?;
    state
        .core_authorization_verifier
        .verify(&inbound)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        ComponentModuleError::Unavailable("Component security state is unavailable".into())
    })?;
    let component_definition = ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    inbound
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: inbound.payload.presenting_service.clone(),
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: component_definition.clone(),
            },
            dependency_binding: inbound.payload.dependency_binding.clone(),
            functional_contract: inbound.payload.functional_contract.clone(),
            action: inbound.payload.action.clone(),
            operation: inbound.payload.operation,
            resource_assertion: inbound.payload.resource_assertion.clone(),
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let target = AuthorizationAudienceV1::CoreInstallation {
        installation_id: security.installation_id,
    };
    let request = AuthorizationExchangeRequestV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        target: target.clone(),
        dependency_binding: DependencyBindingKey::new(DATASET_BINDING_KEY)
            .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
        functional_contract: FunctionalContractId::new(DATASET_CONTRACT_ID)
            .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
        action: action.into(),
        resource_assertion: None,
    };
    request
        .validate()
        .map_err(|_| ComponentModuleError::Forbidden)?;
    let path = "/api/private/module-authorization/exchange";
    let body = serde_json::to_vec(&request)
        .map_err(|error| ComponentModuleError::Internal(error.to_string()))?;
    let service_request = signed_service_request_for_instance(
        state,
        inbound_authorization,
        path,
        &body,
        security.installation_id,
        security.module_instance_id,
        inbound.payload.correlation_id,
    )?;
    let response = state
        .dataset_client
        .post(format!("{}{}", state.core_internal_url, path))
        .timeout(Duration::from_secs(5))
        .header("content-type", "application/json")
        .header("x-tessara-authorization", inbound_authorization)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            inbound.payload.correlation_id.to_string(),
        )
        .body(body)
        .send()
        .await
        .map_err(|_| {
            ComponentModuleError::Unavailable("Core authorization exchange is unavailable".into())
        })?;
    if !response.status().is_success() {
        return if response.status().is_server_error() {
            Err(ComponentModuleError::Unavailable(
                "Core authorization exchange unavailable".into(),
            ))
        } else {
            Err(ComponentModuleError::Forbidden)
        };
    }
    let response: AuthorizationExchangeResponseV2 = response.json().await.map_err(|_| {
        ComponentModuleError::Unavailable("Core authorization exchange response is invalid".into())
    })?;
    if response.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2 {
        return Err(ComponentModuleError::Forbidden);
    }
    state
        .core_authorization_verifier
        .verify(&response.authorization)
        .map_err(|_| ComponentModuleError::Forbidden)?;
    response
        .authorization
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id: inbound.payload.correlation_id,
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: component_definition,
            },
            audience: target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| ComponentModuleError::Forbidden)?;
    if response.authorization.payload.original_actor_id != inbound.payload.original_actor_id {
        return Err(ComponentModuleError::Forbidden);
    }
    let encoded = URL_SAFE_NO_PAD.encode(
        serde_json::to_vec(&response.authorization)
            .map_err(|error| ComponentModuleError::Internal(error.to_string()))?,
    );
    Ok(DownstreamAuthorization {
        encoded,
        installation_id: security.installation_id,
        caller_module_instance_id: security.module_instance_id,
        correlation_id: response.authorization.payload.correlation_id,
        original_actor_id: response.authorization.payload.original_actor_id,
    })
}

fn signed_service_request(
    state: &ComponentModuleState,
    authorization: &DownstreamAuthorization,
    path: &str,
    body: &[u8],
) -> Result<String, ComponentModuleError> {
    signed_service_request_for_instance(
        state,
        &authorization.encoded,
        path,
        body,
        authorization.installation_id,
        authorization.caller_module_instance_id,
        authorization.correlation_id,
    )
}

fn signed_service_request_for_instance(
    state: &ComponentModuleState,
    authorization: &str,
    path: &str,
    body: &[u8],
    installation_id: Uuid,
    caller_module_instance_id: Uuid,
    correlation_id: Uuid,
) -> Result<String, ComponentModuleError> {
    let now = Utc::now();
    let request = ModuleServiceRequestV1 {
        schema_version: 1,
        installation_id,
        module_instance_id: caller_module_instance_id,
        module_definition_id: crate::MODULE_DEFINITION_ID.parse().map_err(|_| {
            ComponentModuleError::Internal("Component service identity is invalid".into())
        })?,
        method: "POST".into(),
        path: path.into(),
        canonical_body_digest: sha256_hex(body),
        inbound_grant_digest: sha256_hex(authorization.as_bytes()),
        correlation_id: correlation_id.to_string(),
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn dependency_reference_observability_is_digest_only_and_stable() {
        let request = json!({
            "schema_version": 1,
            "action": "resolve_schema",
            "reference": {
                "reference": {
                    "installation_id": "01980000-0000-7000-8000-00000000008a",
                    "resource_type": "tessara.transition.dataset_major_line",
                    "resource_id": "01980000-0002-7000-8000-000000000003@1"
                }
            }
        });
        let first = request_reference_digest(&request).unwrap();
        let second = request_reference_digest(&request).unwrap();
        assert_eq!(first, second);
        assert!(first.starts_with("sha256:"));
        assert_eq!(first.len(), 71);
        assert!(!first.contains("01980000"));
        assert_eq!(
            dataset_action("/api/private/datasets/schema", &request).unwrap(),
            "datasets.schema"
        );
    }

    #[test]
    fn compatibility_observation_retains_exact_health_and_result_state() {
        let mut observation = crate::DatasetHealthObservation::default();
        let first = Utc::now();
        update_observation(
            &mut observation,
            "/api/private/datasets/compatibility",
            "available",
            None,
            "dataset.compatibility_rejected",
            Some(false),
            first,
        );
        assert_eq!(observation.status, "available");
        assert_eq!(observation.result_code, "dataset.compatibility_rejected");
        assert_eq!(observation.compatibility.status, "incompatible");
        assert_eq!(
            observation.compatibility.failure_code,
            Some("dataset.incompatible")
        );
        assert_eq!(observation.compatibility.compatible, Some(false));

        let second = first + chrono::Duration::seconds(1);
        update_observation(
            &mut observation,
            "/api/private/datasets/compatibility",
            "available",
            None,
            "dataset.request_succeeded",
            Some(true),
            second,
        );
        assert_eq!(observation.compatibility.status, "compatible");
        assert_eq!(observation.compatibility.failure_code, None);
        assert_eq!(observation.compatibility.observed_at, Some(second));
    }
}
