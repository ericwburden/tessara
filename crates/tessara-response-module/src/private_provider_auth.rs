use axum::http::HeaderMap;
use chrono::Utc;
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, CoreServiceRequestV1, CoreServiceRequestValidationContextV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ModuleServicePrincipalV1,
    ModuleServiceRequestV1, ModuleServiceRequestValidationContextV1, SecurityCapabilityId,
    SignedEnvelopeV1,
};
use tessara_module_runtime::{
    SecurityStateProvider, decode_signed_envelope_header, request_correlation_id,
};

use crate::{MODULE_DEFINITION_ID, ResponseRuntime};

pub(crate) struct PrivateProviderContract {
    pub(crate) path: &'static str,
    pub(crate) binding: &'static str,
    pub(crate) contract: &'static str,
    pub(crate) action: &'static str,
    pub(crate) capability: &'static str,
}

pub(crate) async fn authorize(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    body: &[u8],
    boundary: PrivateProviderContract,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, ()> {
    let encoded_authorization = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(())?;
    let grant: SignedEnvelopeV1<AuthorizationGrantV3> =
        decode_signed_envelope_header(headers, "x-tessara-authorization").map_err(|_| ())?;
    runtime
        .verifiers
        .authorization
        .verify(&grant)
        .map_err(|_| ())?;
    let security = runtime.current_security_state().await.map_err(|_| ())?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(());
    }
    let correlation_id = request_correlation_id(headers).map_err(|_| ())?;
    let presenting_service = match &grant.payload.presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: *module_instance_id,
            module_definition_id: module_definition_id.clone(),
        },
        ModuleServicePrincipalV1::CoreGateway => ModuleServicePrincipalV1::CoreGateway,
    };
    grant
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: presenting_service.clone(),
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                    .map_err(|_| ())?,
            },
            dependency_binding: DependencyBindingKey::new(boundary.binding).map_err(|_| ())?,
            functional_contract: FunctionalContractId::new(boundary.contract).map_err(|_| ())?,
            action: boundary.action.into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion: None,
            authorization_revision: security.authorization_revision,
            organization_revision: security.organization_revision,
            now: Utc::now(),
        })
        .map_err(|_| ())?;
    let capability = SecurityCapabilityId::new(boundary.capability).map_err(|_| ())?;
    if !grant
        .payload
        .capability_scope_bindings
        .iter()
        .any(|binding| binding.capability == capability)
    {
        return Err(());
    }
    match &presenting_service {
        ModuleServicePrincipalV1::ModuleInstance { .. } => {
            validate_and_consume_service_request(
                runtime,
                headers,
                boundary.path,
                body,
                encoded_authorization,
                &presenting_service,
                grant.payload.jti,
                security.installation_id,
                correlation_id,
            )
            .await?;
        }
        ModuleServicePrincipalV1::CoreGateway => {
            validate_and_consume_core_service_request(
                runtime,
                headers,
                boundary.path,
                body,
                encoded_authorization,
                grant.payload.jti,
                security.installation_id,
                correlation_id,
            )
            .await?;
        }
    }
    Ok(grant)
}

#[allow(clippy::too_many_arguments)]
async fn validate_and_consume_core_service_request(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    path: &str,
    body: &[u8],
    inbound_authorization: &str,
    authorization_jti: uuid::Uuid,
    installation_id: uuid::Uuid,
    correlation_id: uuid::Uuid,
) -> Result<(), ()> {
    let service: SignedEnvelopeV1<CoreServiceRequestV1> =
        decode_signed_envelope_header(headers, "x-tessara-core-service-request").map_err(|_| ())?;
    runtime
        .core_service_request_verifier
        .verify(&service)
        .map_err(|_| ())?;
    service
        .payload
        .validate_for(&CoreServiceRequestValidationContextV1 {
            installation_id,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(inbound_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| ())?;
    let consumed = sqlx::query(
        "INSERT INTO response_consumed_core_service_nonces
         (installation_id,nonce,authorization_jti,correlation_id,issued_at)
         VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(service.payload.installation_id)
    .bind(service.payload.nonce)
    .bind(authorization_jti)
    .bind(&service.payload.correlation_id)
    .bind(service.payload.issued_at)
    .execute(&runtime.pool)
    .await
    .map_err(|_| ())?;
    (consumed.rows_affected() == 1).then_some(()).ok_or(())
}

#[allow(clippy::too_many_arguments)]
async fn validate_and_consume_service_request(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    path: &str,
    body: &[u8],
    inbound_authorization: &str,
    presenting_service: &ModuleServicePrincipalV1,
    authorization_jti: uuid::Uuid,
    installation_id: uuid::Uuid,
    correlation_id: uuid::Uuid,
) -> Result<(), ()> {
    let (module_instance_id, module_definition_id) = match presenting_service {
        ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } => (*module_instance_id, module_definition_id.clone()),
        ModuleServicePrincipalV1::CoreGateway => return Err(()),
    };
    let service: SignedEnvelopeV1<ModuleServiceRequestV1> =
        decode_signed_envelope_header(headers, "x-tessara-module-service-request")
            .map_err(|_| ())?;
    runtime
        .service_identity_registry
        .module_service_verifier(&module_definition_id)
        .map_err(|_| ())?
        .verify(&service)
        .map_err(|_| ())?;
    service
        .payload
        .validate_for(&ModuleServiceRequestValidationContextV1 {
            installation_id,
            module_instance_id,
            module_definition_id,
            method: "POST".into(),
            path: path.into(),
            canonical_body_digest: sha256_hex(body),
            inbound_grant_digest: sha256_hex(inbound_authorization.as_bytes()),
            correlation_id: correlation_id.to_string(),
            now: Utc::now(),
        })
        .map_err(|_| ())?;
    let consumed = sqlx::query(
        "INSERT INTO response_consumed_service_nonces
         (module_instance_id,nonce,authorization_jti,correlation_id,issued_at)
         VALUES($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING",
    )
    .bind(service.payload.module_instance_id)
    .bind(service.payload.nonce)
    .bind(authorization_jti)
    .bind(&service.payload.correlation_id)
    .bind(service.payload.issued_at)
    .execute(&runtime.pool)
    .await
    .map_err(|_| ())?;
    (consumed.rows_affected() == 1).then_some(()).ok_or(())
}

fn sha256_hex(value: &[u8]) -> String {
    format!("{:x}", Sha256::digest(value))
}
