//! Core authorization exchange for a verified module service acting on behalf
//! of an actor grant addressed to another module. The exchange is exact,
//! least-privilege, and never accepts a caller-selected capability outside the
//! declared provider action.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use chrono::{Duration, Utc};
use tessara_module_contract::{
    AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V1, AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
    AuthorizationExchangeRequestV1, AuthorizationExchangeResponseV1, AuthorizationGrantOperationV1,
    AuthorizationGrantV2, ProtocolSignaturePurposeV1,
};

use crate::{
    core_security::{capability_bindings, protocol_signer},
    db::AppState,
    error::{ApiError, ApiResult},
};

const COMPONENT_DEFINITION_ID: &str = "tessara.components";
const CORE_COMPONENT_BINDING: &str = "tessara.core.components";
const COMPONENT_CONTRACT: &str = "tessara.components.component-version";
const COMPONENT_READ_CAPABILITY: &str = "components:read";
const EXCHANGE_PATH: &str = "/api/private/module-authorization/exchange";
const DASHBOARD_DEFINITION_ID: &str = "tessara.dashboards";
const CORE_DASHBOARD_BINDING: &str = "tessara.core.dashboards";
const DASHBOARD_CONTRACT: &str = "tessara.dashboards.dashboard";
const DASHBOARD_COMPOSITION_CONTRACT: &str = "tessara.dashboards.composition";

fn dashboard_action_contract(
    action: &str,
) -> Option<(&'static str, AuthorizationGrantOperationV1)> {
    let operation = match action {
        "dashboards.list"
        | "dashboards.list_manageable"
        | "dashboards.get"
        | "dashboards.load_composition"
        | "dashboards.read_dependencies"
        | "dashboards.render_placement" => AuthorizationGrantOperationV1::Read,
        "dashboards.create"
        | "dashboards.update"
        | "dashboards.delete"
        | "dashboards.reconcile_composition"
        | "dashboards.refresh_dependencies"
        | "dashboards.act_on_dependency" => AuthorizationGrantOperationV1::Mutation,
        _ => return None,
    };
    let contract = if matches!(
        action,
        "dashboards.load_composition"
            | "dashboards.reconcile_composition"
            | "dashboards.read_dependencies"
            | "dashboards.refresh_dependencies"
            | "dashboards.act_on_dependency"
    ) {
        DASHBOARD_COMPOSITION_CONTRACT
    } else {
        DASHBOARD_CONTRACT
    };
    Some((contract, operation))
}

pub(crate) fn routes() -> Router<AppState> {
    Router::new().route(EXCHANGE_PATH, post(exchange))
}

async fn exchange(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<AuthorizationExchangeRequestV1>,
) -> ApiResult<Json<AuthorizationExchangeResponseV1>> {
    request.validate().map_err(|_| restricted())?;
    let inbound = crate::module_service_requests::validate_inbound_grant(
        &state,
        &headers,
        DASHBOARD_DEFINITION_ID,
        CORE_DASHBOARD_BINDING,
        dashboard_action_contract,
    )
    .await?;
    let body = serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    crate::module_service_requests::validate_for_instance(
        &state,
        &headers,
        &inbound,
        request.target_module_instance_id,
        COMPONENT_DEFINITION_ID,
        "TESSARA_COMPONENT_SERVICE_PUBLIC_KEY",
        "TESSARA_COMPONENT_SERVICE_SIGNING_KEY_ID",
        "component-development-v1",
        "POST",
        EXCHANGE_PATH,
        &body,
    )
    .await?;
    if request.target_module_definition_id.as_str() != COMPONENT_DEFINITION_ID
        || request.dependency_binding.as_str() != CORE_COMPONENT_BINDING
        || request.functional_contract.as_str() != COMPONENT_CONTRACT
        || request.required_capability.as_str() != COMPONENT_READ_CAPABILITY
        || request.operation != AuthorizationGrantOperationV1::Read
        || !matches!(
            request.action.as_str(),
            "components.resolve" | "components.execute"
        )
    {
        return Err(restricted());
    }
    let target_live: bool = sqlx::query_scalar(
        "SELECT EXISTS(SELECT 1 FROM module_instances
         WHERE id=$1 AND installation_id=$2 AND definition_id=$3
           AND identity_state='live' AND installed=true AND deployed=true
           AND configured=true AND ready=true AND enabled=true AND healthy=true)",
    )
    .bind(request.target_module_instance_id)
    .bind(inbound.payload.installation_id)
    .bind(COMPONENT_DEFINITION_ID)
    .fetch_one(&state.pool)
    .await?;
    if !target_live {
        return Err(restricted());
    }
    let bindings = capability_bindings(
        &state.pool,
        inbound.payload.original_actor_id,
        COMPONENT_READ_CAPABILITY,
    )
    .await?;
    if bindings.is_empty() {
        return Err(restricted());
    }
    let revisions = sqlx::query_as::<_, (i64, i64)>(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let now = Utc::now();
    let authorization = protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(AuthorizationGrantV2 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
            installation_id: inbound.payload.installation_id,
            original_actor_id: inbound.payload.original_actor_id,
            presenting_service: tessara_module_contract::ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: request.target_module_instance_id,
            dependency_binding: request.dependency_binding,
            functional_contract: request.functional_contract,
            action: request.action,
            operation: request.operation,
            capability_scope_bindings: bindings,
            resource_assertion: request.resource_assertion,
            delegation_basis: inbound.payload.delegation_basis.clone(),
            authorization_revision: revisions.0 as u64,
            organization_revision: revisions.1 as u64,
            jti: uuid::Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(60),
        })
        .map_err(|error| ApiError::Internal(error.into()))?;
    Ok(Json(AuthorizationExchangeResponseV1 {
        schema_version: AUTHORIZATION_EXCHANGE_SCHEMA_VERSION_V1,
        authorization,
    }))
}

fn restricted() -> ApiError {
    ApiError::Forbidden("module authorization exchange unavailable".into())
}
