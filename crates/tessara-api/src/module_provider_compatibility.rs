//! Provider-neutral, service-authenticated compatibility proof for exact
//! Core-owned actions consumed by independently deployed modules.

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode, header},
    response::Response,
    routing::post,
};
use chrono::{Duration, Utc};
use tessara_module_contract::{
    AuthorizationAudienceV1, MODULE_PROVIDER_COMPATIBILITY_MAX_LIFETIME_SECONDS,
    MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE, MODULE_PROVIDER_COMPATIBILITY_PATH,
    MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1, ModuleManifest,
    ModuleProviderCompatibilityExpectationV1, ModuleProviderCompatibilityRequestV1,
    ModuleProviderCompatibilityResponseV1, ProtocolSignaturePurposeV1, SignedEnvelopeV1,
};

use crate::{
    db::AppState,
    error::{ApiError, ApiResult},
};

const MODULE_PROVIDER_COMPATIBILITY_BODY_LIMIT_BYTES: usize = 64 * 1024;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            MODULE_PROVIDER_COMPATIBILITY_PATH,
            post(provider_compatibility),
        )
        .layer(DefaultBodyLimit::max(
            MODULE_PROVIDER_COMPATIBILITY_BODY_LIMIT_BYTES,
        ))
}

async fn provider_compatibility(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    require_exact_media_contract(&headers)?;
    if body.len() > MODULE_PROVIDER_COMPATIBILITY_BODY_LIMIT_BYTES {
        return Err(restricted());
    }
    let invocation =
        crate::module_service_requests::verify_module_provider_compatibility_invocation(
            &state, &headers, &body,
        )
        .await?;
    let request: ModuleProviderCompatibilityRequestV1 =
        serde_json::from_slice(&body).map_err(|_| restricted())?;
    request.validate().map_err(|_| restricted())?;
    if !request
        .expectations
        .iter()
        .all(|expectation| expectation_matches_current_core(&invocation.manifest, expectation))
    {
        return Err(restricted());
    }

    let issued_at = Utc::now();
    let response = ModuleProviderCompatibilityResponseV1 {
        schema_version: MODULE_PROVIDER_COMPATIBILITY_SCHEMA_VERSION_V1,
        provider: AuthorizationAudienceV1::CoreInstallation {
            installation_id: invocation.installation_id,
        },
        consumer: invocation.consumer,
        request_digest: request.canonical_digest().map_err(|_| restricted())?,
        expectations: request.expectations,
        correlation_id: invocation.correlation_id,
        issued_at,
        expires_at: issued_at
            + Duration::seconds(MODULE_PROVIDER_COMPATIBILITY_MAX_LIFETIME_SECONDS),
    };
    let envelope = crate::core_security::protocol_signer(
        ProtocolSignaturePurposeV1::ProviderCompatibilityResponse,
    )?
    .sign(response)
    .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    contract_response(&envelope)
}

fn expectation_matches_current_core(
    manifest: &ModuleManifest,
    expectation: &ModuleProviderCompatibilityExpectationV1,
) -> bool {
    let Some(action) = crate::core_service_providers::resolve_service_action(
        &expectation.functional_contract,
        &expectation.authorization_action,
    ) else {
        return false;
    };
    let Some(provider_version) =
        crate::core_service_providers::contract_version(&expectation.functional_contract)
    else {
        return false;
    };
    if expectation.contract_version != provider_version.to_string()
        || action.method != expectation.method
        || action.path != expectation.path
    {
        return false;
    }
    let declared_action = manifest.consumed_service_actions.iter().any(|declared| {
        declared.dependency_binding.as_str() == expectation.dependency_binding
            && declared.functional_contract.as_str() == expectation.functional_contract
            && declared.authorization_action == expectation.authorization_action
    });
    let declared_dependency = manifest.dependencies.iter().any(|dependency| {
        dependency.binding_key.as_str() == expectation.dependency_binding
            && dependency.contract_id.as_str() == expectation.functional_contract
            && dependency.version_requirement.matches(&provider_version)
    });
    declared_action && declared_dependency
}

fn require_exact_media_contract(headers: &HeaderMap) -> ApiResult<()> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some(MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE)
        || headers
            .get(header::ACCEPT)
            .and_then(|value| value.to_str().ok())
            != Some(MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE)
    {
        return Err(restricted());
    }
    Ok(())
}

fn contract_response(
    envelope: &SignedEnvelopeV1<ModuleProviderCompatibilityResponseV1>,
) -> ApiResult<Response> {
    let body = serde_json::to_vec(envelope)
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(
            header::CONTENT_TYPE,
            MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE,
        )
        .header(header::CACHE_CONTROL, "no-store")
        .body(Body::from(body))
        .map_err(|error| ApiError::Internal(anyhow::Error::from(error)))
}

fn restricted() -> ApiError {
    ApiError::NotFound("Module provider compatibility is unavailable".into())
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn consumer_manifest() -> ModuleManifest {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../tessara-dataset-module/manifest.json"
        )))
        .expect("current consumer manifest")
    }

    fn core_form_expectation() -> ModuleProviderCompatibilityExpectationV1 {
        ModuleProviderCompatibilityExpectationV1 {
            dependency_binding: tessara_forms_contract::FORM_VERSION_SCHEMA_BINDING_KEY.into(),
            functional_contract: tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_ID.into(),
            contract_version: tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_VERSION.into(),
            authorization_action: tessara_forms_contract::FORM_VERSION_CATALOG_ACTION.into(),
            method: tessara_module_contract::ServiceActionMethod::Post,
            path: tessara_forms_contract::FORM_VERSION_CATALOG_PATH.into(),
        }
    }

    #[test]
    fn compatibility_expectations_resolve_through_manifest_and_core_catalog_exactly() {
        let manifest = consumer_manifest();
        let expectation = core_form_expectation();
        assert!(expectation_matches_current_core(&manifest, &expectation));

        let mut changed = expectation.clone();
        changed.dependency_binding = "example.consumer.another-binding".into();
        assert!(!expectation_matches_current_core(&manifest, &changed));
        changed = expectation.clone();
        changed.contract_version = "1.0.1".into();
        assert!(!expectation_matches_current_core(&manifest, &changed));
        changed = expectation.clone();
        changed.authorization_action = tessara_forms_contract::FORM_VERSION_SCHEMA_ACTION.into();
        assert!(!expectation_matches_current_core(&manifest, &changed));
        changed = expectation.clone();
        changed.method = tessara_module_contract::ServiceActionMethod::Get;
        assert!(!expectation_matches_current_core(&manifest, &changed));
        changed = expectation;
        changed.path = tessara_forms_contract::FORM_VERSION_SCHEMA_PATH.into();
        assert!(!expectation_matches_current_core(&manifest, &changed));
    }

    #[test]
    fn compatibility_endpoint_requires_exact_request_and_response_media_type() {
        let mut headers = HeaderMap::new();
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static(MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE),
        );
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE),
        );
        require_exact_media_contract(&headers).unwrap();

        headers.insert(header::ACCEPT, HeaderValue::from_static("application/json"));
        assert!(require_exact_media_contract(&headers).is_err());
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(MODULE_PROVIDER_COMPATIBILITY_MEDIA_TYPE),
        );
        headers.insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );
        assert!(require_exact_media_contract(&headers).is_err());
    }
}
