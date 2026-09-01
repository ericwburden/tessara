//! Dataset-owned authenticated clients for declared consumed providers.

use std::time::{Duration, Instant};

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration as ChronoDuration, Utc};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
use tessara_composition::{OwnerBootstrapAuthorizationV1, OwnerBootstrapProviderActionV1};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2, AuthorizationAudienceV1,
    AuthorizationExchangeRequestV2, AuthorizationExchangeResponseV2, AuthorizationGrantOperationV1,
    AuthorizationGrantV3, AuthorizationValidationContextV3, CapabilityScopeBindingV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ModuleServicePrincipalV1,
    ModuleServiceRequestV1, PurposeBoundSigningKeyV1, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{DatasetModuleError, DatasetModuleState, MODULE_DEFINITION_ID, load_security_state};

#[derive(Clone, Copy)]
pub(crate) struct ProviderAction {
    pub audience: ProviderAudience,
    pub binding: &'static str,
    pub contract: &'static str,
    pub action: &'static str,
    pub path: &'static str,
    pub media_type: &'static str,
    /// True only for authenticated, side-effect-free provider observations.
    /// Owner product writes must never opt into this retry path.
    pub retry_safe_observation: bool,
}

#[derive(Clone, Copy)]
pub(crate) enum ProviderAudience {
    Core,
    Module(&'static str),
}

#[derive(Clone, Copy)]
struct ProviderAttemptResult {
    code: &'static str,
    retry_classification: &'static str,
}

#[derive(Clone)]
pub(crate) struct BootstrapProviderAuthorization {
    pub(crate) authorization: SignedEnvelopeV1<OwnerBootstrapAuthorizationV1>,
}

pub(crate) trait ProviderAuthorization: Sync {
    fn is_bootstrap(&self) -> bool;
    fn correlation_id(&self) -> Uuid;
    fn original_actor_id(&self) -> Uuid;
    fn encoded_header(&self) -> Result<(String, &'static str), DatasetModuleError>;
    fn bootstrap_action(&self, target: ProviderAction) -> Option<&OwnerBootstrapProviderActionV1>;
    fn capability_scope_bindings(&self) -> &[CapabilityScopeBindingV1];
}

impl ProviderAuthorization for SignedEnvelopeV1<AuthorizationGrantV3> {
    fn is_bootstrap(&self) -> bool {
        false
    }

    fn correlation_id(&self) -> Uuid {
        self.payload.correlation_id
    }

    fn original_actor_id(&self) -> Uuid {
        self.payload.original_actor_id
    }

    fn encoded_header(&self) -> Result<(String, &'static str), DatasetModuleError> {
        Ok((
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(self)
                    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            ),
            "x-tessara-authorization",
        ))
    }

    fn bootstrap_action(&self, _target: ProviderAction) -> Option<&OwnerBootstrapProviderActionV1> {
        None
    }

    fn capability_scope_bindings(&self) -> &[CapabilityScopeBindingV1] {
        &self.payload.capability_scope_bindings
    }
}

impl ProviderAuthorization for BootstrapProviderAuthorization {
    fn is_bootstrap(&self) -> bool {
        true
    }

    fn correlation_id(&self) -> Uuid {
        self.authorization.payload.correlation_id
    }

    fn original_actor_id(&self) -> Uuid {
        self.authorization.payload.original_actor_id
    }

    fn encoded_header(&self) -> Result<(String, &'static str), DatasetModuleError> {
        Ok((
            URL_SAFE_NO_PAD.encode(
                serde_json::to_vec(&self.authorization)
                    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
            ),
            "x-tessara-owner-bootstrap-authorization",
        ))
    }

    fn bootstrap_action(&self, target: ProviderAction) -> Option<&OwnerBootstrapProviderActionV1> {
        self.authorization
            .payload
            .provider_actions
            .iter()
            .find(|action| {
                action.dependency_binding == target.binding
                    && action.functional_contract == target.contract
                    && action.action == target.action
                    && action.path == target.path
            })
    }

    fn capability_scope_bindings(&self) -> &[CapabilityScopeBindingV1] {
        &self.authorization.payload.capability_scope_bindings
    }
}

pub(crate) async fn post<TRequest, TResponse, TAuthorization>(
    state: &DatasetModuleState,
    inbound: &TAuthorization,
    target: ProviderAction,
    request: &TRequest,
) -> Result<TResponse, DatasetModuleError>
where
    TRequest: Serialize,
    TResponse: DeserializeOwned,
    TAuthorization: ProviderAuthorization + ?Sized,
{
    let body = serde_json::to_vec(request)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let bootstrap_downstream = if inbound.bootstrap_action(target).is_some() {
        let (encoded, header_name) = inbound.encoded_header()?;
        let security = load_security_state(&state.pool).await?.ok_or_else(|| {
            DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
        })?;
        Some(DownstreamAuthorization {
            encoded,
            header_name,
            installation_id: security.installation_id,
            module_instance_id: security.module_instance_id,
            correlation_id: inbound.correlation_id(),
        })
    } else if inbound.is_bootstrap() {
        return Err(DatasetModuleError::Forbidden);
    } else {
        None
    };
    let (timeout_seconds, retry_limit): (i32, i32) = sqlx::query_as(
        "SELECT provider_request_timeout_seconds,provider_retry_limit \
         FROM dataset_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let timeout_seconds = u64::try_from(timeout_seconds)
        .ok()
        .filter(|value| (1..=30).contains(value))
        .ok_or_else(|| {
            DatasetModuleError::Internal(
                "Dataset provider request timeout configuration is invalid".into(),
            )
        })?;
    let retry_limit = u16::try_from(retry_limit)
        .ok()
        .filter(|value| *value <= 3)
        .ok_or_else(|| {
            DatasetModuleError::Internal("Dataset provider retry configuration is invalid".into())
        })?;
    let retry_limit = if target.retry_safe_observation {
        retry_limit
    } else {
        0
    };
    let provider_url = state
        .service_endpoints
        .provider_url(target.binding)
        .ok_or_else(|| {
            DatasetModuleError::Internal(format!(
                "Dataset provider binding '{}' is not configured",
                target.binding
            ))
        })?;
    for attempt in 0..=retry_limit {
        let started = Instant::now();
        let attempt_number = attempt + 1;
        let attempt_limit = retry_limit + 1;
        // Core provider grants are one-use capabilities. A provider may have
        // consumed the grant before returning a retriable server error, so a
        // retry must exchange the inbound actor grant again rather than only
        // refreshing the service-request nonce.
        let downstream = match &bootstrap_downstream {
            Some(downstream) => downstream.clone(),
            None => match exchange(state, inbound, target).await {
                Ok(downstream) => downstream,
                Err(error) => {
                    emit_provider_attempt(
                        inbound.correlation_id(),
                        None,
                        target,
                        attempt_number,
                        attempt_limit,
                        started,
                        ProviderAttemptResult {
                            code: "authorization_exchange_failed",
                            retry_classification: "not_retryable",
                        },
                    );
                    return Err(error);
                }
            },
        };
        // The service-request envelope is intentionally created inside the
        // attempt loop: every retry has a fresh nonce and one-use signature.
        let service_request = match signed_service_request(
            &state.service_request_signer,
            &downstream.encoded,
            target.path,
            &body,
            downstream.installation_id,
            downstream.module_instance_id,
            downstream.correlation_id,
        ) {
            Ok(request) => request,
            Err(error) => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "service_request_signing_failed",
                        retry_classification: "not_retryable",
                    },
                );
                return Err(error);
            }
        };
        let response = state
            .provider_client
            .post(format!("{provider_url}{}", target.path))
            .timeout(Duration::from_secs(timeout_seconds))
            .header("content-type", target.media_type)
            .header(downstream.header_name, &downstream.encoded)
            .header("x-tessara-module-service-request", service_request)
            .header(
                "x-tessara-correlation-id",
                downstream.correlation_id.to_string(),
            )
            .body(body.clone())
            .send()
            .await;
        let response = match response {
            Ok(response) => response,
            Err(_) if attempt < retry_limit => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "transport_unavailable",
                        retry_classification: "retrying",
                    },
                );
                continue;
            }
            Err(_) => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "transport_unavailable",
                        retry_classification: "exhausted",
                    },
                );
                return Err(DatasetModuleError::Unavailable(
                    "Dataset authoring provider is unavailable".into(),
                ));
            }
        };
        if response.status().is_server_error() {
            if attempt < retry_limit {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "provider_unavailable",
                        retry_classification: "retrying",
                    },
                );
                continue;
            }
            emit_provider_attempt(
                downstream.correlation_id,
                Some(downstream.module_instance_id),
                target,
                attempt_number,
                attempt_limit,
                started,
                ProviderAttemptResult {
                    code: "provider_unavailable",
                    retry_classification: "exhausted",
                },
            );
            return Err(DatasetModuleError::Unavailable(
                "Dataset authoring provider is unavailable".into(),
            ));
        }
        if !response.status().is_success() {
            // A provider-owned semantic rejection is authoritative and is
            // never retried, even for an otherwise safe observation.
            emit_provider_attempt(
                downstream.correlation_id,
                Some(downstream.module_instance_id),
                target,
                attempt_number,
                attempt_limit,
                started,
                ProviderAttemptResult {
                    code: "provider_rejected",
                    retry_classification: "not_retryable",
                },
            );
            return Err(DatasetModuleError::Forbidden);
        }
        let response_media_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok());
        if response_media_type != Some(target.media_type) {
            tracing::warn!(
                target: "tessara_dataset_module::provider_client",
                event = "dataset.provider.response_media_incompatible",
                correlation_id = %downstream.correlation_id,
                module_instance_id = %downstream.module_instance_id,
                operation = target.action,
                dependency = target.binding,
                functional_contract = target.contract,
                expected_media_type = target.media_type,
                actual_media_type = response_media_type.unwrap_or("absent-or-invalid"),
                "Dataset provider response media type was incompatible"
            );
            emit_provider_attempt(
                downstream.correlation_id,
                Some(downstream.module_instance_id),
                target,
                attempt_number,
                attempt_limit,
                started,
                ProviderAttemptResult {
                    code: "response_media_incompatible",
                    retry_classification: "not_retryable",
                },
            );
            return Err(DatasetModuleError::DependencyIncompatible(
                "Dataset authoring provider returned an incompatible media type".into(),
            ));
        }
        let bytes = match response.bytes().await {
            Ok(bytes) => bytes,
            Err(_) if attempt < retry_limit => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "response_transport_unavailable",
                        retry_classification: "retrying",
                    },
                );
                continue;
            }
            Err(_) => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "response_transport_unavailable",
                        retry_classification: "exhausted",
                    },
                );
                return Err(DatasetModuleError::Unavailable(
                    "Dataset authoring provider response is invalid".into(),
                ));
            }
        };
        // A successfully transported but malformed response is a semantic
        // incompatibility. Retrying it would hide a provider contract defect.
        return match decode_provider_response(&bytes) {
            Ok(response) => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "success",
                        retry_classification: "not_required",
                    },
                );
                Ok(response)
            }
            Err(error) => {
                emit_provider_attempt(
                    downstream.correlation_id,
                    Some(downstream.module_instance_id),
                    target,
                    attempt_number,
                    attempt_limit,
                    started,
                    ProviderAttemptResult {
                        code: "response_incompatible",
                        retry_classification: "not_retryable",
                    },
                );
                Err(error)
            }
        };
    }
    unreachable!("the bounded provider attempt range always executes")
}

fn emit_provider_attempt(
    correlation_id: Uuid,
    module_instance_id: Option<Uuid>,
    target: ProviderAction,
    attempt: u16,
    attempt_limit: u16,
    started: Instant,
    result: ProviderAttemptResult,
) {
    let module_instance_id = module_instance_id
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unavailable".into());
    let duration_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    tracing::info!(
        target: "tessara_dataset_module::provider_client",
        event = "dataset.provider.attempt",
        correlation_id = %correlation_id,
        module_instance_id = %module_instance_id,
        operation = target.action,
        dependency = target.binding,
        functional_contract = target.contract,
        attempt,
        attempt_limit,
        duration_ms,
        result_code = result.code,
        retry_classification = result.retry_classification,
        "Dataset provider attempt completed"
    );
}

fn decode_provider_response<TResponse>(bytes: &[u8]) -> Result<TResponse, DatasetModuleError>
where
    TResponse: DeserializeOwned,
{
    serde_json::from_slice(bytes).map_err(|_| {
        DatasetModuleError::DependencyIncompatible(
            "Dataset authoring provider response is incompatible".into(),
        )
    })
}

#[derive(Clone)]
struct DownstreamAuthorization {
    encoded: String,
    header_name: &'static str,
    installation_id: Uuid,
    module_instance_id: Uuid,
    correlation_id: Uuid,
}

async fn exchange<TAuthorization: ProviderAuthorization + ?Sized>(
    state: &DatasetModuleState,
    inbound: &TAuthorization,
    provider: ProviderAction,
) -> Result<DownstreamAuthorization, DatasetModuleError> {
    let security = load_security_state(&state.pool).await?.ok_or_else(|| {
        DatasetModuleError::Unavailable("Dataset security state is unavailable".into())
    })?;
    let definition = ModuleDefinitionId::new(MODULE_DEFINITION_ID)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let target = provider_audience(provider.audience, security.installation_id)?;
    let request = AuthorizationExchangeRequestV2 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2,
        target: target.clone(),
        dependency_binding: DependencyBindingKey::new(provider.binding)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        functional_contract: FunctionalContractId::new(provider.contract)
            .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        action: provider.action.into(),
        resource_assertion: None,
    };
    request
        .validate()
        .map_err(|_| DatasetModuleError::Forbidden)?;
    let (inbound_encoded, inbound_header_name) = inbound.encoded_header()?;
    if inbound_header_name != "x-tessara-authorization" {
        return Err(DatasetModuleError::Forbidden);
    }
    let path = "/api/private/module-authorization/exchange";
    let body = serde_json::to_vec(&request)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    let service_request = signed_service_request(
        &state.service_request_signer,
        &inbound_encoded,
        path,
        &body,
        security.installation_id,
        security.module_instance_id,
        inbound.correlation_id(),
    )?;
    let response = state
        .provider_client
        .post(format!(
            "{}{}",
            state.service_endpoints.core_authorization_url(),
            path
        ))
        .timeout(Duration::from_secs(5))
        .header("content-type", "application/json")
        .header("x-tessara-authorization", &inbound_encoded)
        .header("x-tessara-module-service-request", service_request)
        .header(
            "x-tessara-correlation-id",
            inbound.correlation_id().to_string(),
        )
        .body(body)
        .send()
        .await
        .map_err(|_| {
            DatasetModuleError::Unavailable("Core authorization exchange is unavailable".into())
        })?;
    if !response.status().is_success() {
        return Err(if response.status().is_server_error() {
            DatasetModuleError::Unavailable("Core authorization exchange is unavailable".into())
        } else {
            DatasetModuleError::Forbidden
        });
    }
    let response: AuthorizationExchangeResponseV2 = response.json().await.map_err(|_| {
        DatasetModuleError::Unavailable("Core authorization exchange response is invalid".into())
    })?;
    if response.schema_version != AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V2 {
        return Err(DatasetModuleError::Forbidden);
    }
    state
        .core_authorization_verifier
        .verify(&response.authorization)
        .map_err(|_| DatasetModuleError::Forbidden)?;
    response
        .authorization
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id: inbound.correlation_id(),
            presenting_service: ModuleServicePrincipalV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: definition,
            },
            audience: target,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: provider.action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision as u64,
            organization_revision: security.organization_revision as u64,
            now: Utc::now(),
        })
        .map_err(|_| DatasetModuleError::Forbidden)?;
    if response.authorization.payload.original_actor_id != inbound.original_actor_id() {
        return Err(DatasetModuleError::Forbidden);
    }
    Ok(DownstreamAuthorization {
        encoded: URL_SAFE_NO_PAD.encode(
            serde_json::to_vec(&response.authorization)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        ),
        header_name: "x-tessara-authorization",
        installation_id: security.installation_id,
        module_instance_id: security.module_instance_id,
        correlation_id: response.authorization.payload.correlation_id,
    })
}

fn provider_audience(
    audience: ProviderAudience,
    installation_id: Uuid,
) -> Result<AuthorizationAudienceV1, DatasetModuleError> {
    match audience {
        ProviderAudience::Core => Ok(AuthorizationAudienceV1::CoreInstallation { installation_id }),
        ProviderAudience::Module(definition_id) => Ok(AuthorizationAudienceV1::ModuleInstance {
            module_instance_id: tessara_composition::module_instance_id(
                installation_id,
                definition_id,
            ),
            module_definition_id: ModuleDefinitionId::new(definition_id)
                .map_err(|error| DatasetModuleError::Internal(error.to_string()))?,
        }),
    }
}

fn signed_service_request(
    signer: &PurposeBoundSigningKeyV1,
    authorization: &str,
    path: &str,
    body: &[u8],
    installation_id: Uuid,
    module_instance_id: Uuid,
    correlation_id: Uuid,
) -> Result<String, DatasetModuleError> {
    let now = Utc::now();
    let request = ModuleServiceRequestV1 {
        schema_version: 1,
        installation_id,
        module_instance_id,
        module_definition_id: MODULE_DEFINITION_ID.parse().map_err(|_| {
            DatasetModuleError::Internal("Dataset service identity is invalid".into())
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
    let envelope = signer.sign(request).map_err(|_| {
        DatasetModuleError::Internal("Dataset service request signing failed".into())
    })?;
    let bytes = serde_json::to_vec(&envelope)
        .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use tracing_subscriber::fmt::MakeWriter;

    use super::*;

    #[derive(Clone, Default)]
    struct CapturedWriter(Arc<Mutex<Vec<u8>>>);

    struct CapturedGuard(Arc<Mutex<Vec<u8>>>);

    impl Write for CapturedGuard {
        fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
            self.0
                .lock()
                .expect("capture lock")
                .extend_from_slice(buffer);
            Ok(buffer.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    impl<'writer> MakeWriter<'writer> for CapturedWriter {
        type Writer = CapturedGuard;

        fn make_writer(&'writer self) -> Self::Writer {
            CapturedGuard(Arc::clone(&self.0))
        }
    }

    #[test]
    fn invalid_success_response_json_is_dependency_incompatible() {
        assert!(matches!(
            decode_provider_response::<serde_json::Value>(b"{"),
            Err(DatasetModuleError::DependencyIncompatible(_))
        ));
    }

    #[test]
    fn provider_audience_preserves_core_and_independent_module_ownership() {
        let installation_id = Uuid::from_u128(1);
        assert_eq!(
            provider_audience(ProviderAudience::Core, installation_id).unwrap(),
            AuthorizationAudienceV1::CoreInstallation { installation_id }
        );

        let definition_id = "tessara.responses";
        assert_eq!(
            provider_audience(ProviderAudience::Module(definition_id), installation_id).unwrap(),
            AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: tessara_composition::module_instance_id(
                    installation_id,
                    definition_id,
                ),
                module_definition_id: ModuleDefinitionId::new(definition_id).unwrap(),
            }
        );
    }

    #[test]
    fn provider_attempt_event_is_structured_and_contains_no_request_detail() {
        let captured = CapturedWriter::default();
        let subscriber = tracing_subscriber::fmt()
            .without_time()
            .with_ansi(false)
            .with_target(false)
            .with_level(false)
            .with_writer(captured.clone())
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            emit_provider_attempt(
                Uuid::from_u128(1),
                Some(Uuid::from_u128(2)),
                ProviderAction {
                    audience: ProviderAudience::Module("tessara.responses"),
                    binding: "tessara.datasets.response-export",
                    contract: "tessara.responses.submitted-response-export",
                    action: "responses.export_checkpoint",
                    path: "/must-not-be-logged/secret",
                    media_type: "application/secret",
                    retry_safe_observation: true,
                },
                2,
                3,
                Instant::now(),
                ProviderAttemptResult {
                    code: "transport_unavailable",
                    retry_classification: "retrying",
                },
            );
        });
        let bytes = captured.0.lock().expect("capture lock").clone();
        let event = String::from_utf8(bytes).expect("UTF-8 trace event");
        for expected in [
            "dataset.provider.attempt",
            "correlation_id",
            "module_instance_id",
            "responses.export_checkpoint",
            "tessara.datasets.response-export",
            "tessara.responses.submitted-response-export",
            "attempt=2",
            "attempt_limit=3",
            "duration_ms",
            "transport_unavailable",
            "retrying",
        ] {
            assert!(event.contains(expected), "missing trace field: {expected}");
        }
        for forbidden in [
            "/must-not-be-logged/secret",
            "application/secret",
            "authorization",
            "request_body",
            "response_body",
            "source_reference",
        ] {
            assert!(
                !event.contains(forbidden),
                "leaked trace field: {forbidden}"
            );
        }
    }
}
