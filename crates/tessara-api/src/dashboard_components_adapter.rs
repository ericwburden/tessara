//! First-party transition adapter between the independently deployed
//! Dashboard module and Core's temporary in-process Components provider.
//!
//! A Dashboard reference is never authority. Core validates the inbound
//! Dashboard action grant, evaluates Components capability independently,
//! binds and verifies a downstream grant for the exact adapter action, and
//! only then reads Component product state.

use axum::{Json, Router, extract::State, http::HeaderMap, routing::post};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use sqlx::Row;
use tessara_dashboards::{
    DASHBOARD_COMPONENT_BINDING_KEY, DASHBOARD_COMPONENT_CONTRACT_ID,
    DashboardComponentCatalogResponseV1, DashboardComponentMetadataV1,
    DashboardComponentResolutionRequestV1, DashboardComponentResolutionResponseV1,
    DashboardComponentTransitionAction,
};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV1, AuthorizationValidationContextV1,
    CapabilityScopeBindingV1, ContractCompatibilityState, CoreInstallationOwnerState,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ProtocolSignaturePurposeV1,
    ProviderAvailabilityState, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceOwner, ResourceOwnerState, SecurityCapabilityId, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    core_security::protocol_signer,
    db::AppState,
    error::{ApiError, ApiResult},
};

const DASHBOARD_DEFINITION_ID: &str = "tessara.dashboards";
const CORE_DASHBOARD_BINDING: &str = "tessara.core.dashboards";
const DASHBOARD_CONTRACT: &str = "tessara.dashboards.dashboard";
const COMPONENT_READ_CAPABILITY: &str = "components:read";

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/api/private/dashboard-components/resolve",
            post(resolve_component),
        )
        .route(
            "/api/private/dashboard-components/catalog",
            post(component_catalog),
        )
}

async fn resolve_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<DashboardComponentResolutionRequestV1>,
) -> ApiResult<Json<DashboardComponentResolutionResponseV1>> {
    let inbound = validate_inbound_dashboard_grant(&state, &headers).await?;
    let reference = request.reference.reference();
    let installation_id = inbound.payload.installation_id;
    if reference.installation_id() != installation_id
        || !matches!(
            reference.owner(),
            ResourceOwner::CoreInstallation {
                installation_id: owner_installation_id
            } if *owner_installation_id == installation_id
        )
    {
        return Ok(Json(restricted(ResourceAccessState::NotEvaluated)?));
    }

    let component_bindings =
        component_capability_bindings(&state, inbound.payload.original_actor_id).await?;
    if component_bindings.is_empty() {
        return Ok(Json(restricted(ResourceAccessState::Unauthorized)?));
    }
    let globally_authorized =
        has_global_component_authority(&state, inbound.payload.original_actor_id).await?;

    let downstream =
        issue_downstream_grant(&state, &inbound, request.action, component_bindings).await?;
    validate_downstream_grant(&state, &downstream, request.action, &inbound).await?;

    let provider_state =
        std::env::var("TESSARA_COMPONENTS_PROVIDER_STATE").unwrap_or_else(|_| "available".into());
    if provider_state == "unavailable" {
        if !globally_authorized {
            return Ok(Json(restricted(ResourceAccessState::NotEvaluated)?));
        }
        return Ok(Json(authorized_without_metadata(
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Unavailable,
        )?));
    }
    if provider_state == "incompatible" {
        if !globally_authorized {
            return Ok(Json(restricted(ResourceAccessState::NotEvaluated)?));
        }
        return Ok(Json(authorized_without_metadata(
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Incompatible,
            ProviderAvailabilityState::Available,
        )?));
    }

    let Some(component_version_id) = parse_canonical_uuid(reference.resource_id()) else {
        return if globally_authorized {
            Ok(Json(authorized_without_metadata(
                ResourceIdentityState::UnknownResource,
                ResourceLifecycleState::NotEvaluated,
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Available,
            )?))
        } else {
            Ok(Json(restricted(ResourceAccessState::NotEvaluated)?))
        };
    };
    let row = sqlx::query(
        "SELECT cv.id,cv.component_id,c.name AS component_name,c.slug AS component_slug,
                cv.component_type::text AS component_type,cv.version_number,
                cv.version_label,cv.status::text AS version_status,cv.dataset_id
         FROM component_versions cv
         JOIN components c ON c.id=cv.component_id
         WHERE cv.id=$1",
    )
    .bind(component_version_id)
    .fetch_optional(&state.pool)
    .await?;
    let Some(row) = row else {
        return if globally_authorized {
            Ok(Json(authorized_without_metadata(
                ResourceIdentityState::UnknownResource,
                ResourceLifecycleState::NotEvaluated,
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Available,
            )?))
        } else {
            Ok(Json(restricted(ResourceAccessState::NotEvaluated)?))
        };
    };

    let dataset_id: Uuid = row.try_get("dataset_id")?;
    let dataset_scope: Vec<Uuid> = sqlx::query_scalar(
        "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_id",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let capability = SecurityCapabilityId::new(COMPONENT_READ_CAPABILITY)
        .map_err(|error| ApiError::Internal(error.into()))?;
    if dataset_scope.is_empty()
        || !dataset_scope
            .iter()
            .any(|organization_id| downstream.payload.authorizes(&capability, *organization_id))
    {
        return Ok(Json(restricted(ResourceAccessState::NotEvaluated)?));
    }

    let version_status: String = row.try_get("version_status")?;
    let metadata = DashboardComponentMetadataV1 {
        component_version_id: row.try_get("id")?,
        component_id: row.try_get("component_id")?,
        component_name: row.try_get("component_name")?,
        component_slug: row.try_get("component_slug")?,
        component_type: row.try_get("component_type")?,
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        version_status: version_status.clone(),
        scope_node_ids: dataset_scope,
    };
    let resolution = tessara_module_contract::ResourceResolutionV1::authorized(
        ResourceOwnerState::CoreInstallation {
            state: CoreInstallationOwnerState::Live,
        },
        ResourceIdentityState::Resolved,
        ResourceLifecycleState::ProviderDefined {
            state: version_status,
        },
        ContractCompatibilityState::Compatible,
        ProviderAvailabilityState::Available,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    Ok(Json(
        DashboardComponentResolutionResponseV1::new(resolution, Some(metadata))
            .map_err(|error| ApiError::Internal(error.into()))?,
    ))
}

async fn component_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<DashboardComponentCatalogResponseV1>> {
    let inbound = validate_inbound_dashboard_grant(&state, &headers).await?;
    let bindings = component_capability_bindings(&state, inbound.payload.original_actor_id).await?;
    if bindings.is_empty() {
        return Ok(Json(DashboardComponentCatalogResponseV1 {
            schema_version: 1,
            components: Vec::new(),
        }));
    }
    let downstream = issue_downstream_grant(
        &state,
        &inbound,
        DashboardComponentTransitionAction::ResolveMetadata,
        bindings,
    )
    .await?;
    validate_downstream_grant(
        &state,
        &downstream,
        DashboardComponentTransitionAction::ResolveMetadata,
        &inbound,
    )
    .await?;
    if std::env::var("TESSARA_COMPONENTS_PROVIDER_STATE").is_ok_and(|state| state != "available") {
        return Ok(Json(DashboardComponentCatalogResponseV1 {
            schema_version: 1,
            components: Vec::new(),
        }));
    }
    let rows = sqlx::query(
        "SELECT cv.id,cv.component_id,c.name AS component_name,c.slug AS component_slug,
                cv.component_type::text AS component_type,cv.version_number,
                cv.version_label,cv.status::text AS version_status,cv.dataset_id
         FROM component_versions cv
         JOIN components c ON c.id=cv.component_id
         WHERE cv.status IN ('published','superseded')
         ORDER BY c.name,cv.version_number,cv.id",
    )
    .fetch_all(&state.pool)
    .await?;
    let capability = SecurityCapabilityId::new(COMPONENT_READ_CAPABILITY)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let mut components = Vec::new();
    for row in rows {
        let dataset_id: Uuid = row.try_get("dataset_id")?;
        let nodes: Vec<Uuid> = sqlx::query_scalar(
            "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_id",
        )
        .bind(dataset_id)
        .fetch_all(&state.pool)
        .await?;
        if nodes.is_empty()
            || !nodes
                .iter()
                .any(|node_id| downstream.payload.authorizes(&capability, *node_id))
        {
            continue;
        }
        components.push(DashboardComponentMetadataV1 {
            component_version_id: row.try_get("id")?,
            component_id: row.try_get("component_id")?,
            component_name: row.try_get("component_name")?,
            component_slug: row.try_get("component_slug")?,
            component_type: row.try_get("component_type")?,
            version_number: row.try_get("version_number")?,
            version_label: row.try_get("version_label")?,
            version_status: row.try_get("version_status")?,
            scope_node_ids: nodes,
        });
    }
    Ok(Json(DashboardComponentCatalogResponseV1 {
        schema_version: 1,
        components,
    }))
}

async fn validate_inbound_dashboard_grant(
    state: &AppState,
    headers: &HeaderMap,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV1>> {
    let encoded = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or_else(restricted_authorization)?;
    let bytes = URL_SAFE_NO_PAD
        .decode(encoded)
        .map_err(|_| restricted_authorization())?;
    let envelope: SignedEnvelopeV1<AuthorizationGrantV1> =
        serde_json::from_slice(&bytes).map_err(|_| restricted_authorization())?;
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
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
    .bind(DASHBOARD_DEFINITION_ID)
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
    let expected_operation = match envelope.payload.action.as_str() {
        "dashboards.list"
        | "dashboards.list_manageable"
        | "dashboards.get"
        | "dashboards.load_composition" => AuthorizationGrantOperationV1::Read,
        "dashboards.create"
        | "dashboards.update"
        | "dashboards.delete"
        | "dashboards.reconcile_composition" => AuthorizationGrantOperationV1::Mutation,
        _ => return Err(restricted_authorization()),
    };
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV1 {
            installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: envelope.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(CORE_DASHBOARD_BINDING)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(DASHBOARD_CONTRACT)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: envelope.payload.action.clone(),
            operation: expected_operation,
            authorization_revision: revisions.try_get::<i64, _>("authorization_revision")? as u64,
            organization_revision: revisions.try_get::<i64, _>("organization_revision")? as u64,
            now: Utc::now(),
        })
        .map_err(|_| restricted_authorization())?;
    Ok(envelope)
}

async fn issue_downstream_grant(
    state: &AppState,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV1>,
    action: DashboardComponentTransitionAction,
    capability_scope_bindings: Vec<CapabilityScopeBindingV1>,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV1>> {
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let now = Utc::now();
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(AuthorizationGrantV1 {
            schema_version: 1,
            installation_id: inbound.payload.installation_id,
            original_actor_id: inbound.payload.original_actor_id,
            presenting_service: ModuleDefinitionId::new(DASHBOARD_DEFINITION_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: inbound.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(DASHBOARD_COMPONENT_BINDING_KEY)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(DASHBOARD_COMPONENT_CONTRACT_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: action.as_str().into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings,
            resource_assertion: None,
            delegation_basis: inbound.payload.delegation_basis.clone(),
            authorization_revision: revisions.try_get::<i64, _>("authorization_revision")? as u64,
            organization_revision: revisions.try_get::<i64, _>("organization_revision")? as u64,
            jti: Uuid::new_v4(),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        })
        .map_err(|error| ApiError::Internal(error.into()))
}

async fn validate_downstream_grant(
    state: &AppState,
    grant: &SignedEnvelopeV1<AuthorizationGrantV1>,
    action: DashboardComponentTransitionAction,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV1>,
) -> ApiResult<()> {
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .verifier()
        .verify(grant)
        .map_err(|_| restricted_authorization())?;
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    grant
        .payload
        .validate_for(&AuthorizationValidationContextV1 {
            installation_id: inbound.payload.installation_id,
            presenting_service: ModuleDefinitionId::new(DASHBOARD_DEFINITION_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: inbound.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(DASHBOARD_COMPONENT_BINDING_KEY)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(DASHBOARD_COMPONENT_CONTRACT_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: action.as_str().into(),
            operation: AuthorizationGrantOperationV1::Read,
            authorization_revision: revisions.try_get::<i64, _>("authorization_revision")? as u64,
            organization_revision: revisions.try_get::<i64, _>("organization_revision")? as u64,
            now: Utc::now(),
        })
        .map_err(|_| restricted_authorization())
}

async fn component_capability_bindings(
    state: &AppState,
    actor_id: Uuid,
) -> ApiResult<Vec<CapabilityScopeBindingV1>> {
    let global = has_global_component_authority(state, actor_id).await?;
    let roots: Vec<Uuid> = if global {
        sqlx::query_scalar("SELECT id FROM nodes WHERE parent_node_id IS NULL ORDER BY id")
            .fetch_all(&state.pool)
            .await?
    } else {
        sqlx::query_scalar(
            "SELECT DISTINCT ra.node_id FROM role_assignments ra
             JOIN role_capabilities rc ON rc.role_id=ra.role_id
             JOIN capabilities c ON c.id=rc.capability_id
             WHERE ra.account_id=$1 AND ra.node_id IS NOT NULL
               AND (c.key=$2 OR c.key='admin:all')
             ORDER BY ra.node_id",
        )
        .bind(actor_id)
        .bind(COMPONENT_READ_CAPABILITY)
        .fetch_all(&state.pool)
        .await?
    };
    let capability = SecurityCapabilityId::new(COMPONENT_READ_CAPABILITY)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let mut bindings = Vec::new();
    for root in roots {
        let descendants: Vec<Uuid> = sqlx::query_scalar(
            "WITH RECURSIVE subtree AS (
               SELECT id FROM nodes WHERE id=$1
               UNION ALL
               SELECT child.id FROM nodes child JOIN subtree parent
                 ON child.parent_node_id=parent.id
             )
             SELECT id FROM subtree WHERE id<>$1 ORDER BY id",
        )
        .bind(root)
        .fetch_all(&state.pool)
        .await?;
        bindings.push(CapabilityScopeBindingV1 {
            capability: capability.clone(),
            organization_root_id: root,
            authorized_organization_ids: descendants,
        });
    }
    Ok(bindings)
}

async fn has_global_component_authority(state: &AppState, actor_id: Uuid) -> ApiResult<bool> {
    Ok(sqlx::query_scalar(
        "SELECT EXISTS(
           SELECT 1 FROM role_assignments ra
           JOIN role_capabilities rc ON rc.role_id=ra.role_id
           JOIN capabilities c ON c.id=rc.capability_id
           WHERE ra.account_id=$1 AND ra.node_id IS NULL
             AND (c.key=$2 OR c.key='admin:all')
         )",
    )
    .bind(actor_id)
    .bind(COMPONENT_READ_CAPABILITY)
    .fetch_one(&state.pool)
    .await?)
}

fn restricted(access: ResourceAccessState) -> ApiResult<DashboardComponentResolutionResponseV1> {
    let resolution = tessara_module_contract::ResourceResolutionV1::restricted(access)
        .map_err(|error| ApiError::Internal(error.into()))?;
    DashboardComponentResolutionResponseV1::new(resolution, None)
        .map_err(|error| ApiError::Internal(error.into()))
}

fn authorized_without_metadata(
    identity: ResourceIdentityState,
    lifecycle: ResourceLifecycleState,
    compatibility: ContractCompatibilityState,
    availability: ProviderAvailabilityState,
) -> ApiResult<DashboardComponentResolutionResponseV1> {
    let resolution = tessara_module_contract::ResourceResolutionV1::authorized(
        ResourceOwnerState::CoreInstallation {
            state: CoreInstallationOwnerState::Live,
        },
        identity,
        lifecycle,
        compatibility,
        availability,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    DashboardComponentResolutionResponseV1::new(resolution, None)
        .map_err(|error| ApiError::Internal(error.into()))
}

fn parse_canonical_uuid(value: &str) -> Option<Uuid> {
    let parsed = Uuid::parse_str(value).ok()?;
    (parsed.to_string() == value).then_some(parsed)
}

fn restricted_authorization() -> ApiError {
    ApiError::Forbidden("Dashboard Component action unavailable".into())
}

#[cfg(test)]
mod tests {
    use super::parse_canonical_uuid;

    #[test]
    fn component_resource_ids_are_canonical_uuids() {
        assert!(parse_canonical_uuid("11111111-1111-4111-8111-111111111111").is_some());
        assert!(parse_canonical_uuid("11111111111141118111111111111111").is_none());
        assert!(parse_canonical_uuid("not-a-uuid").is_none());
    }
}
