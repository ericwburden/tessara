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
use sha2::{Digest, Sha256};
use sqlx::Row;
use tessara_components_contract::{
    COMPONENT_BINDING_KEY, COMPONENT_CONTRACT_ID, COMPONENT_CONTRACT_SCHEMA_VERSION,
    COMPONENT_CONTRACT_VERSION, ComponentAction, ComponentCatalogResponse, ComponentChange,
    ComponentChangeCategory, ComponentLifecycleState, ComponentMetadata, ComponentPublicationState,
    ComponentRenderRequest, ComponentResolutionRequest, ComponentResolutionResponse,
    ComponentSuccessor, ComponentVersionReference,
};
use tessara_module_contract::{
    AUTHORIZATION_GRANT_SCHEMA_VERSION_V2, AuthorizationGrantOperationV1, AuthorizationGrantV2,
    AuthorizationValidationContextV2, CapabilityScopeBindingV1, ContractCompatibilityState,
    CoreInstallationOwnerState, DependencyBindingKey, FunctionalContractId, ModuleDefinitionId,
    ModuleInstanceOwnerState, ModuleServiceRequestV1, ModuleServiceRequestValidationContextV1,
    OwnerDataState, ProtocolSignaturePurposeV1, ProviderAvailabilityState,
    ProviderContractIdentity, PurposeBoundVerifyingKeyV1, ResourceAccessState,
    ResourceAuthorizationAssertionV2, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationStrategy, ResourceObservationV1, ResourceOwner, ResourceOwnerState,
    ResourceRevision, ResourceTypeId, SecurityCapabilityId, SignedEnvelopeV1,
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
const DASHBOARD_COMPOSITION_CONTRACT: &str = "tessara.dashboards.composition";
const COMPONENT_READ_CAPABILITY: &str = "components:read";

fn dashboard_contract_for_action(action: &str) -> &'static str {
    match action {
        "dashboards.load_composition"
        | "dashboards.reconcile_composition"
        | "dashboards.read_dependencies"
        | "dashboards.refresh_dependencies"
        | "dashboards.act_on_dependency" => DASHBOARD_COMPOSITION_CONTRACT,
        _ => DASHBOARD_CONTRACT,
    }
}

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
        .route(
            "/api/private/dashboard-components/render",
            post(render_component),
        )
}

async fn render_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ComponentRenderRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let inbound = validate_inbound_dashboard_grant(&state, &headers).await?;
    let request_body =
        serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    validate_module_service_request(
        &state,
        &headers,
        &inbound,
        "POST",
        "/api/private/dashboard-components/render",
        &request_body,
    )
    .await?;
    if request.schema_version != COMPONENT_CONTRACT_SCHEMA_VERSION
        || request.action != ComponentAction::Render
        || request.dashboard_scope_node_ids.is_empty()
        || request
            .dashboard_scope_node_ids
            .windows(2)
            .any(|pair| pair[0] >= pair[1])
    {
        return Err(restricted_authorization());
    }
    let reference = request.reference.reference();
    if reference.installation_id() != inbound.payload.installation_id {
        return Err(restricted_authorization());
    }
    let version_id =
        parse_canonical_uuid(reference.resource_id()).ok_or_else(restricted_authorization)?;
    let version = sqlx::query(
        "SELECT dataset_id,authority_revision FROM component_versions
         WHERE id=$1 AND status IN ('published','superseded')
           AND lifecycle_state='active'::component_lifecycle_state",
    )
    .bind(version_id)
    .fetch_optional(&state.pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let dataset_id: Uuid = version.try_get("dataset_id")?;
    let authority_revision: i64 = version.try_get("authority_revision")?;
    if request.resource_authority_revision != authority_revision as u64 {
        return Err(restricted_authorization());
    }
    let dataset_scope: Vec<Uuid> = sqlx::query_scalar(
        "SELECT node_id FROM dataset_scope_nodes WHERE dataset_id=$1 ORDER BY node_id",
    )
    .bind(dataset_id)
    .fetch_all(&state.pool)
    .await?;
    let component_bindings =
        component_capability_bindings(&state, inbound.payload.original_actor_id).await?;
    let resource_assertion = ResourceAuthorizationAssertionV2 {
        resource_type: ResourceTypeId::new(tessara_components_contract::COMPONENT_RESOURCE_TYPE)
            .map_err(|error| ApiError::Internal(error.into()))?,
        resource_id: version_id.to_string(),
        authority_revision: authority_revision as u64,
        governing_organization_ids: dataset_scope.clone(),
    };
    let downstream = issue_downstream_grant(
        &state,
        &inbound,
        ComponentAction::Render,
        component_bindings,
        Some(resource_assertion.clone()),
    )
    .await?;
    validate_downstream_grant(
        &state,
        &downstream,
        ComponentAction::Render,
        &inbound,
        Some(resource_assertion),
    )
    .await?;
    let dashboard_read = SecurityCapabilityId::new("dashboards:read")
        .map_err(|error| ApiError::Internal(error.into()))?;
    let component_read = SecurityCapabilityId::new(COMPONENT_READ_CAPABILITY)
        .map_err(|error| ApiError::Internal(error.into()))?;
    let same_governing_node = dataset_scope.iter().any(|node_id| {
        request.dashboard_scope_node_ids.contains(node_id)
            && inbound.payload.authorizes(&dashboard_read, *node_id)
            && downstream.payload.authorizes(&component_read, *node_id)
    });
    if !same_governing_node {
        return Err(restricted_authorization());
    }
    let account =
        crate::auth::account_context_for_actor(&state.pool, inbound.payload.original_actor_id)
            .await?;
    crate::components::render_version_for_account(
        &state.pool,
        &account,
        version_id,
        request.kind.component_type(),
        &request.query,
    )
    .await
    .map(Json)
}

async fn resolve_component(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(request): Json<ComponentResolutionRequest>,
) -> ApiResult<Json<ComponentResolutionResponse>> {
    let inbound = validate_inbound_dashboard_grant(&state, &headers).await?;
    let request_body =
        serde_json::to_vec(&request).map_err(|error| ApiError::Internal(error.into()))?;
    validate_module_service_request(
        &state,
        &headers,
        &inbound,
        "POST",
        "/api/private/dashboard-components/resolve",
        &request_body,
    )
    .await?;
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

    let downstream = issue_downstream_grant(
        &state,
        &inbound,
        request.action,
        component_bindings.clone(),
        None,
    )
    .await?;
    validate_downstream_grant(&state, &downstream, request.action, &inbound, None).await?;

    let provider_state =
        std::env::var("TESSARA_COMPONENTS_PROVIDER_STATE").unwrap_or_else(|_| "available".into());
    if let Some(fixture) = provider_fixture(&provider_state) {
        if !globally_authorized {
            return Ok(Json(restricted(ResourceAccessState::NotEvaluated)?));
        }
        return Ok(Json(fixture?));
    }

    let Some(component_version_id) = parse_canonical_uuid(reference.resource_id()) else {
        return if globally_authorized {
            Ok(Json(authorized_without_metadata(
                ResourceOwnerState::CoreInstallation {
                    state: CoreInstallationOwnerState::Live,
                },
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
                cv.version_label,cv.status::text AS version_status,
                cv.lifecycle_state::text AS lifecycle_state,cv.dataset_id,
                cv.authority_revision,cv.resource_revision,cv.successor_version_id
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
                ResourceOwnerState::CoreInstallation {
                    state: CoreInstallationOwnerState::Live,
                },
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
    let resource_assertion = ResourceAuthorizationAssertionV2 {
        resource_type: ResourceTypeId::new(tessara_components_contract::COMPONENT_RESOURCE_TYPE)
            .map_err(|error| ApiError::Internal(error.into()))?,
        resource_id: component_version_id.to_string(),
        authority_revision: row.try_get::<i64, _>("authority_revision")? as u64,
        governing_organization_ids: dataset_scope.clone(),
    };
    let downstream = issue_downstream_grant(
        &state,
        &inbound,
        request.action,
        component_bindings,
        Some(resource_assertion.clone()),
    )
    .await?;
    validate_downstream_grant(
        &state,
        &downstream,
        request.action,
        &inbound,
        Some(resource_assertion),
    )
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
    let publication_state = publication_state(&version_status)?;
    let lifecycle_state_value: String = row.try_get("lifecycle_state")?;
    let lifecycle_state = lifecycle_state(&lifecycle_state_value)?;
    let authority_revision = row.try_get::<i64, _>("authority_revision")? as u64;
    let resource_revision =
        ResourceRevision::new(row.try_get::<i64, _>("resource_revision")? as u64)
            .map_err(|error| ApiError::Internal(error.into()))?;
    let metadata = ComponentMetadata {
        component_version_id: row.try_get("id")?,
        component_id: row.try_get("component_id")?,
        component_name: row.try_get("component_name")?,
        component_slug: row.try_get("component_slug")?,
        component_type: row.try_get("component_type")?,
        version_number: row.try_get("version_number")?,
        version_label: row.try_get("version_label")?,
        publication_state,
        lifecycle_state,
        authority_revision,
        scope_node_ids: dataset_scope,
    };
    let resolution = tessara_module_contract::ResourceResolutionV1::authorized(
        ResourceOwnerState::CoreInstallation {
            state: CoreInstallationOwnerState::Live,
        },
        ResourceIdentityState::Resolved,
        ResourceLifecycleState::ProviderDefined {
            state: lifecycle_state_value,
        },
        ContractCompatibilityState::Compatible,
        ProviderAvailabilityState::Available,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    let observation = ResourceObservationV1::new(
        reference.clone(),
        ProviderContractIdentity::new(
            FunctionalContractId::new(COMPONENT_CONTRACT_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            COMPONENT_CONTRACT_VERSION
                .parse()
                .map_err(|error| ApiError::Internal(anyhow::Error::new(error)))?,
        ),
        ResourceObservationStrategy::LiveResolutionWithRevision,
        resource_revision,
    );
    let changes = component_changes_since(
        &state.pool,
        component_version_id,
        request.changes_since_revision,
        resource_revision,
    )
    .await?;
    let successor = if lifecycle_state.metadata_visible() {
        component_successor(
            installation_id,
            &state.pool,
            row.try_get("component_id")?,
            row.try_get("successor_version_id")?,
        )
        .await?
    } else {
        None
    };
    let metadata = lifecycle_state.metadata_visible().then_some(metadata);
    Ok(Json(
        ComponentResolutionResponse::new(
            resolution,
            Some(observation),
            metadata,
            changes,
            successor,
        )
        .map_err(|error| ApiError::Internal(error.into()))?,
    ))
}

async fn component_catalog(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> ApiResult<Json<ComponentCatalogResponse>> {
    let inbound = validate_inbound_dashboard_grant(&state, &headers).await?;
    validate_module_service_request(
        &state,
        &headers,
        &inbound,
        "POST",
        "/api/private/dashboard-components/catalog",
        &[],
    )
    .await?;
    let bindings = component_capability_bindings(&state, inbound.payload.original_actor_id).await?;
    if bindings.is_empty() {
        return Ok(Json(ComponentCatalogResponse {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            components: Vec::new(),
        }));
    }
    let downstream = issue_downstream_grant(
        &state,
        &inbound,
        ComponentAction::ResolveMetadata,
        bindings,
        None,
    )
    .await?;
    validate_downstream_grant(
        &state,
        &downstream,
        ComponentAction::ResolveMetadata,
        &inbound,
        None,
    )
    .await?;
    if std::env::var("TESSARA_COMPONENTS_PROVIDER_STATE").is_ok_and(|state| state != "available") {
        return Ok(Json(ComponentCatalogResponse {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            components: Vec::new(),
        }));
    }
    let rows = sqlx::query(
        "SELECT cv.id,cv.component_id,c.name AS component_name,c.slug AS component_slug,
                cv.component_type::text AS component_type,cv.version_number,
                cv.version_label,cv.status::text AS version_status,
                cv.lifecycle_state::text AS lifecycle_state,cv.dataset_id,
                cv.authority_revision
         FROM component_versions cv
         JOIN components c ON c.id=cv.component_id
         WHERE cv.status IN ('published','superseded')
           AND cv.lifecycle_state='active'::component_lifecycle_state
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
        let version_status: String = row.try_get("version_status")?;
        components.push(ComponentMetadata {
            component_version_id: row.try_get("id")?,
            component_id: row.try_get("component_id")?,
            component_name: row.try_get("component_name")?,
            component_slug: row.try_get("component_slug")?,
            component_type: row.try_get("component_type")?,
            version_number: row.try_get("version_number")?,
            version_label: row.try_get("version_label")?,
            publication_state: publication_state(&version_status)?,
            lifecycle_state: lifecycle_state(&row.try_get::<String, _>("lifecycle_state")?)?,
            authority_revision: row.try_get::<i64, _>("authority_revision")? as u64,
            scope_node_ids: nodes,
        });
    }
    Ok(Json(ComponentCatalogResponse {
        schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
        components,
    }))
}

async fn validate_module_service_request(
    state: &AppState,
    headers: &HeaderMap,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV2>,
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

    let public_key_encoded = std::env::var("TESSARA_DASHBOARD_SERVICE_PUBLIC_KEY")
        .map_err(|_| restricted_authorization())?;
    let public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(&public_key_encoded)
        .map_err(|_| restricted_authorization())?
        .try_into()
        .map_err(|_| restricted_authorization())?;
    let key_id = std::env::var("TESSARA_DASHBOARD_SERVICE_SIGNING_KEY_ID")
        .unwrap_or_else(|_| "dashboard-development-v1".into());
    let verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        DASHBOARD_DEFINITION_ID,
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
            module_instance_id: inbound.payload.audience_module_instance_id,
            module_definition_id: ModuleDefinitionId::new(DASHBOARD_DEFINITION_ID)
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
    .bind(DASHBOARD_DEFINITION_ID)
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
    .bind(DASHBOARD_DEFINITION_ID)
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
    .bind(DASHBOARD_DEFINITION_ID)
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

fn sha256_hex(value: &[u8]) -> String {
    Sha256::digest(value)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

async fn validate_inbound_dashboard_grant(
    state: &AppState,
    headers: &HeaderMap,
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
        | "dashboards.load_composition"
        | "dashboards.read_dependencies"
        | "dashboards.render_placement" => AuthorizationGrantOperationV1::Read,
        "dashboards.create"
        | "dashboards.update"
        | "dashboards.delete"
        | "dashboards.reconcile_composition"
        | "dashboards.refresh_dependencies"
        | "dashboards.act_on_dependency" => AuthorizationGrantOperationV1::Mutation,
        _ => return Err(restricted_authorization()),
    };
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV2 {
            installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: envelope.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(CORE_DASHBOARD_BINDING)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(dashboard_contract_for_action(
                &envelope.payload.action,
            ))
            .map_err(|error| ApiError::Internal(error.into()))?,
            action: envelope.payload.action.clone(),
            operation: expected_operation,
            resource_assertion: None,
            authorization_revision: revisions.try_get::<i64, _>("authorization_revision")? as u64,
            organization_revision: revisions.try_get::<i64, _>("organization_revision")? as u64,
            now: Utc::now(),
        })
        .map_err(|_| restricted_authorization())?;
    Ok(envelope)
}

async fn issue_downstream_grant(
    state: &AppState,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV2>,
    action: ComponentAction,
    capability_scope_bindings: Vec<CapabilityScopeBindingV1>,
    resource_assertion: Option<ResourceAuthorizationAssertionV2>,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV2>> {
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    let now = Utc::now();
    protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(AuthorizationGrantV2 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V2,
            installation_id: inbound.payload.installation_id,
            original_actor_id: inbound.payload.original_actor_id,
            presenting_service: ModuleDefinitionId::new(DASHBOARD_DEFINITION_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: inbound.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(COMPONENT_BINDING_KEY)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(COMPONENT_CONTRACT_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: action.as_str().into(),
            operation: AuthorizationGrantOperationV1::Read,
            capability_scope_bindings,
            resource_assertion,
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
    grant: &SignedEnvelopeV1<AuthorizationGrantV2>,
    action: ComponentAction,
    inbound: &SignedEnvelopeV1<AuthorizationGrantV2>,
    resource_assertion: Option<ResourceAuthorizationAssertionV2>,
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
        .validate_for(&AuthorizationValidationContextV2 {
            installation_id: inbound.payload.installation_id,
            presenting_service: ModuleDefinitionId::new(DASHBOARD_DEFINITION_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: inbound.payload.audience_module_instance_id,
            dependency_binding: DependencyBindingKey::new(COMPONENT_BINDING_KEY)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(COMPONENT_CONTRACT_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: action.as_str().into(),
            operation: AuthorizationGrantOperationV1::Read,
            resource_assertion,
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

fn restricted(access: ResourceAccessState) -> ApiResult<ComponentResolutionResponse> {
    let resolution = tessara_module_contract::ResourceResolutionV1::restricted(access)
        .map_err(|error| ApiError::Internal(error.into()))?;
    ComponentResolutionResponse::new(resolution, None, None, Vec::new(), None)
        .map_err(|error| ApiError::Internal(error.into()))
}

fn authorized_without_metadata(
    owner: ResourceOwnerState,
    identity: ResourceIdentityState,
    lifecycle: ResourceLifecycleState,
    compatibility: ContractCompatibilityState,
    availability: ProviderAvailabilityState,
) -> ApiResult<ComponentResolutionResponse> {
    let resolution = tessara_module_contract::ResourceResolutionV1::authorized(
        owner,
        identity,
        lifecycle,
        compatibility,
        availability,
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    ComponentResolutionResponse::new(resolution, None, None, Vec::new(), None)
        .map_err(|error| ApiError::Internal(error.into()))
}

fn publication_state(value: &str) -> ApiResult<ComponentPublicationState> {
    match value {
        "draft" => Ok(ComponentPublicationState::Draft),
        "published" => Ok(ComponentPublicationState::Published),
        "superseded" => Ok(ComponentPublicationState::Superseded),
        _ => Err(ApiError::Internal(anyhow::anyhow!(
            "stored Component publication state is invalid"
        ))),
    }
}

fn lifecycle_state(value: &str) -> ApiResult<ComponentLifecycleState> {
    match value {
        "active" => Ok(ComponentLifecycleState::Active),
        "inactive" => Ok(ComponentLifecycleState::Inactive),
        "archived" => Ok(ComponentLifecycleState::Archived),
        "tombstoned" => Ok(ComponentLifecycleState::Tombstoned),
        _ => Err(ApiError::Internal(anyhow::anyhow!(
            "stored Component lifecycle state is invalid"
        ))),
    }
}

async fn component_changes_since(
    pool: &sqlx::PgPool,
    version_id: Uuid,
    prior: Option<ResourceRevision>,
    current: ResourceRevision,
) -> ApiResult<Vec<ComponentChange>> {
    let Some(prior) = prior else {
        return Ok(Vec::new());
    };
    if prior > current {
        return Err(ApiError::BadRequest(
            "changes_since_revision is later than the current resource revision".into(),
        ));
    }
    let rows = sqlx::query(
        "SELECT resource_revision,
                array_agg(category::text ORDER BY category::text) AS categories
         FROM component_version_change_events
         WHERE component_version_id=$1 AND resource_revision>$2 AND resource_revision<=$3
         GROUP BY resource_revision
         ORDER BY resource_revision",
    )
    .bind(version_id)
    .bind(prior.get() as i64)
    .bind(current.get() as i64)
    .fetch_all(pool)
    .await?;
    rows.into_iter()
        .map(|row| {
            let revision =
                ResourceRevision::new(row.try_get::<i64, _>("resource_revision")? as u64)
                    .map_err(|error| ApiError::Internal(error.into()))?;
            let categories = row
                .try_get::<Vec<String>, _>("categories")?
                .into_iter()
                .map(|category| match category.as_str() {
                    "publication" => Ok(ComponentChangeCategory::Publication),
                    "lifecycle" => Ok(ComponentChangeCategory::Lifecycle),
                    "payload" => Ok(ComponentChangeCategory::Payload),
                    "successor" => Ok(ComponentChangeCategory::Successor),
                    _ => Err(ApiError::Internal(anyhow::anyhow!(
                        "stored Component change category is invalid"
                    ))),
                })
                .collect::<ApiResult<Vec<_>>>()?;
            Ok(ComponentChange {
                resource_revision: revision,
                categories,
            })
        })
        .collect()
}

async fn component_successor(
    installation_id: Uuid,
    pool: &sqlx::PgPool,
    component_id: Uuid,
    successor_version_id: Option<Uuid>,
) -> ApiResult<Option<ComponentSuccessor>> {
    let Some(successor_version_id) = successor_version_id else {
        return Ok(None);
    };
    let valid: bool = sqlx::query_scalar(
        "SELECT EXISTS(
           SELECT 1 FROM component_versions
           WHERE id=$1 AND component_id=$2
             AND status='published'::component_version_status
             AND lifecycle_state='active'::component_lifecycle_state
         )",
    )
    .bind(successor_version_id)
    .bind(component_id)
    .fetch_one(pool)
    .await?;
    if !valid {
        return Err(ApiError::Internal(anyhow::anyhow!(
            "stored Component successor violates provider invariants"
        )));
    }
    let reference = tessara_module_contract::TypedResourceReference::new(
        installation_id,
        ResourceOwner::CoreInstallation { installation_id },
        ResourceTypeId::new(tessara_components_contract::COMPONENT_RESOURCE_TYPE)
            .map_err(|error| ApiError::Internal(error.into()))?,
        successor_version_id.to_string(),
    )
    .map_err(|error| ApiError::Internal(error.into()))?;
    Ok(Some(ComponentSuccessor {
        reference: ComponentVersionReference::new(reference)
            .map_err(|error| ApiError::Internal(error.into()))?,
    }))
}

fn provider_fixture(state: &str) -> Option<ApiResult<ComponentResolutionResponse>> {
    let core_owner = || ResourceOwnerState::CoreInstallation {
        state: CoreInstallationOwnerState::Live,
    };
    let fixture = match state {
        "available" => return None,
        "unavailable" => authorized_without_metadata(
            core_owner(),
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Unavailable,
        ),
        "incompatible" => authorized_without_metadata(
            core_owner(),
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Incompatible,
            ProviderAvailabilityState::Available,
        ),
        "inactive" | "superseded" | "tombstoned" => authorized_without_metadata(
            core_owner(),
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::ProviderDefined {
                state: state.to_string(),
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        ),
        "owner_tombstoned" => authorized_without_metadata(
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::OwnerModuleInstanceTombstoned,
                data_state: OwnerDataState::Retained,
            },
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        ),
        "owner_data_destroyed" => authorized_without_metadata(
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::OwnerModuleInstanceTombstoned,
                data_state: OwnerDataState::OwnerDataDestroyed,
            },
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        ),
        "missing" => authorized_without_metadata(
            core_owner(),
            ResourceIdentityState::UnknownResource,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        ),
        "not_evaluated" => authorized_without_metadata(
            ResourceOwnerState::NotEvaluated,
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::NotEvaluated,
            ProviderAvailabilityState::NotEvaluated,
        ),
        _ => Err(ApiError::ServiceUnavailable(
            "Components provider state fixture is invalid".into(),
        )),
    };
    Some(fixture)
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
    use tessara_module_contract::{
        ContractCompatibilityState, ModuleInstanceOwnerState, OwnerDataState,
        ProviderAvailabilityState, ResourceIdentityState, ResourceLifecycleState,
        ResourceOwnerState,
    };

    use super::{
        DASHBOARD_COMPOSITION_CONTRACT, DASHBOARD_CONTRACT, dashboard_contract_for_action,
        parse_canonical_uuid, provider_fixture,
    };

    #[test]
    fn inbound_dashboard_contract_matches_the_action_family() {
        assert_eq!(
            dashboard_contract_for_action("dashboards.load_composition"),
            DASHBOARD_COMPOSITION_CONTRACT
        );
        assert_eq!(
            dashboard_contract_for_action("dashboards.reconcile_composition"),
            DASHBOARD_COMPOSITION_CONTRACT
        );
        assert_eq!(
            dashboard_contract_for_action("dashboards.get"),
            DASHBOARD_CONTRACT
        );
    }

    #[test]
    fn component_resource_ids_are_canonical_uuids() {
        assert!(parse_canonical_uuid("11111111-1111-4111-8111-111111111111").is_some());
        assert!(parse_canonical_uuid("11111111111141118111111111111111").is_none());
        assert!(parse_canonical_uuid("not-a-uuid").is_none());
    }

    #[test]
    fn transition_failure_matrix_is_deterministic_and_distinct() {
        assert!(provider_fixture("available").is_none());
        let unavailable = provider_fixture("unavailable")
            .expect("fixture")
            .expect("valid fixture");
        assert_eq!(
            unavailable.resolution().availability_state(),
            ProviderAvailabilityState::Unavailable
        );
        let incompatible = provider_fixture("incompatible")
            .expect("fixture")
            .expect("valid fixture");
        assert_eq!(
            incompatible.resolution().compatibility_state(),
            ContractCompatibilityState::Incompatible
        );
        for lifecycle in ["inactive", "superseded", "tombstoned"] {
            assert_eq!(
                provider_fixture(lifecycle)
                    .expect("fixture")
                    .expect("valid fixture")
                    .resolution()
                    .resource_lifecycle_state(),
                &ResourceLifecycleState::ProviderDefined {
                    state: lifecycle.into()
                }
            );
        }
        let owner_tombstoned = provider_fixture("owner_tombstoned")
            .expect("fixture")
            .expect("valid fixture");
        assert!(matches!(
            owner_tombstoned.resolution().owner_state(),
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::OwnerModuleInstanceTombstoned,
                data_state: OwnerDataState::Retained,
            }
        ));
        let destroyed = provider_fixture("owner_data_destroyed")
            .expect("fixture")
            .expect("valid fixture");
        assert!(matches!(
            destroyed.resolution().owner_state(),
            ResourceOwnerState::ModuleInstance {
                data_state: OwnerDataState::OwnerDataDestroyed,
                ..
            }
        ));
        assert_eq!(
            provider_fixture("missing")
                .expect("fixture")
                .expect("valid fixture")
                .resolution()
                .resource_identity_state(),
            ResourceIdentityState::UnknownResource
        );
        assert_eq!(
            provider_fixture("not_evaluated")
                .expect("fixture")
                .expect("valid fixture")
                .resolution()
                .availability_state(),
            ProviderAvailabilityState::NotEvaluated
        );
        assert!(provider_fixture("invalid").expect("fixture").is_err());
    }
}
