//! Generic installation-bound typed resource reference construction and observation.

use sqlx::PgPool;
use tessara_datasets_contract::{
    DatasetResourceObservationRequest, DatasetResourceObservationResponse,
};
use tessara_module_contract::{
    ContractCompatibilityState, CoreInstallationOwnerState, ModuleInstanceOwnerState,
    OwnerDataState, ProviderAvailabilityState, ReferenceValidationError, ResourceAccessState,
    ResourceIdentityState, ResourceLifecycleState, ResourceObservationV1, ResourceOwner,
    ResourceOwnerState, ResourceResolutionV1, TypedResourceReference,
};
use uuid::Uuid;

use crate::{
    auth::{AccountContext, AuthenticatedRequest},
    db::AppState,
    module_gateway::{
        CorePrivateProviderResult, ResourceObservationProviderLookup, call_private_provider,
        resource_observation_provider,
    },
};

use super::{
    dto::{
        CreateResourceReferenceRequestV1, MODULE_HTTP_SCHEMA_VERSION_V1,
        ResourceReferenceResponseV1,
    },
    error::{ModuleHttpError, ModuleHttpResult},
};

#[derive(Clone, Copy, Debug)]
enum CoreResourceKind {
    Form,
    FormVersion,
    Workflow,
    WorkflowVersion,
    Response,
}

#[derive(Clone, Copy)]
struct CoreResourceSpec {
    kind: CoreResourceKind,
    capabilities_any_of: &'static [&'static str],
}

const FORMS: &[&str] = &["forms:read", "forms:manage"];
const WORKFLOWS: &[&str] = &["workflows:read", "workflows:manage"];
const RESPONSES: &[&str] = &[
    "submissions:read_own",
    "submissions:respond",
    "submissions:manage",
];

pub(crate) async fn construct(
    state: &AppState,
    request: CreateResourceReferenceRequestV1,
    installation_id: Uuid,
    account: &AccountContext,
) -> ModuleHttpResult<ResourceReferenceResponseV1> {
    if request.schema_version != MODULE_HTTP_SCHEMA_VERSION_V1 {
        return Err(ModuleHttpError::bad_request(
            "platform_schema_version_unsupported",
            "Only platform HTTP schema version 1 is supported.",
        ));
    }
    if request.installation_id != installation_id {
        return Err(ModuleHttpError::bad_request(
            "resource_reference_installation_mismatch",
            "The resource reference installation does not match this installation.",
        ));
    }

    let reference = TypedResourceReference::new(
        request.installation_id,
        request.owner,
        request.resource_type,
        request.resource_id,
    )
    .map_err(|error| {
        if error == ReferenceValidationError::InstallationMismatch {
            owner_mismatch()
        } else {
            ModuleHttpError::bad_request(
                "resource_reference_invalid",
                "The resource reference is structurally invalid.",
            )
        }
    })?;

    match reference.owner() {
        ResourceOwner::CoreInstallation {
            installation_id: owner_installation_id,
        } if *owner_installation_id == installation_id => {
            validate_core_reference_construction(&reference, account)?;
        }
        ResourceOwner::CoreInstallation { .. } => {
            return Err(owner_mismatch());
        }
        ResourceOwner::ModuleInstance {
            installation_id: owner_installation_id,
            ..
        } if *owner_installation_id == installation_id => {
            let route = load_resource_observation_provider(state, &reference).await?;
            let ResourceObservationProviderLookup::Registered(route) = route else {
                return Err(ModuleHttpError::bad_request(
                    "resource_reference_not_registered",
                    "The exact Module Instance does not register this typed resource reference.",
                ));
            };
            if !account.has_capability(route.actor_capability()) {
                return Err(ModuleHttpError::forbidden(
                    "resource_reference_capability_required",
                    "The current account lacks authority to construct this resource reference.",
                ));
            }
        }
        ResourceOwner::ModuleInstance { .. } => return Err(owner_mismatch()),
    }

    Ok(ResourceReferenceResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        reference,
    })
}

pub(crate) async fn resolve(
    state: &AppState,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    actor: &AuthenticatedRequest,
    correlation_id: Uuid,
) -> ModuleHttpResult<ResourceResolutionV1> {
    resolve_and_observe(state, reference, installation_id, actor, correlation_id)
        .await
        .map(|(resolution, _)| resolution)
}

pub(crate) async fn observe(
    state: &AppState,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    actor: &AuthenticatedRequest,
    correlation_id: Uuid,
) -> ModuleHttpResult<(ResourceResolutionV1, Option<ResourceObservationV1>)> {
    resolve_and_observe(state, reference, installation_id, actor, correlation_id).await
}

async fn resolve_and_observe(
    state: &AppState,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    actor: &AuthenticatedRequest,
    correlation_id: Uuid,
) -> ModuleHttpResult<(ResourceResolutionV1, Option<ResourceObservationV1>)> {
    match reference.owner() {
        ResourceOwner::CoreInstallation { .. } => {
            resolve_core_reference(&state.pool, reference, installation_id, &actor.account)
                .await
                .map(|resolution| (resolution, None))
        }
        ResourceOwner::ModuleInstance { .. } => {
            resolve_module_reference(state, reference, installation_id, actor, correlation_id).await
        }
    }
}

async fn resolve_module_reference(
    state: &AppState,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    actor: &AuthenticatedRequest,
    correlation_id: Uuid,
) -> ModuleHttpResult<(ResourceResolutionV1, Option<ResourceObservationV1>)> {
    if reference.installation_id() != installation_id
        || !matches!(
            reference.owner(),
            ResourceOwner::ModuleInstance {
                installation_id: owner_installation_id,
                ..
            } if *owner_installation_id == installation_id
        )
    {
        return restricted_observation(ResourceAccessState::NotEvaluated);
    }

    let route = load_resource_observation_provider(state, reference).await?;
    let ResourceObservationProviderLookup::Registered(route) = route else {
        return restricted_observation(ResourceAccessState::NotEvaluated);
    };
    if !actor.account.has_capability(route.actor_capability()) {
        return restricted_observation(ResourceAccessState::Unauthorized);
    }

    let request = DatasetResourceObservationRequest {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        reference: reference.clone(),
    };
    match call_private_provider::<_, DatasetResourceObservationResponse>(
        state,
        actor,
        route.private_request(correlation_id, &request),
    )
    .await
    .map_err(|error| {
        tracing::error!(error = ?error, "Generic resource-observation dispatch failed");
        ModuleHttpError::Internal("generic resource-observation dispatch failed")
    })? {
        CorePrivateProviderResult::Response(response) => {
            if let Err(error) = response.validate_for(reference) {
                tracing::warn!(error = ?error, "Module resource-observation response was invalid");
                return module_provider_unavailable();
            }
            Ok((response.resolution, response.observation))
        }
        CorePrivateProviderResult::Unavailable => module_provider_unavailable(),
        CorePrivateProviderResult::Undisclosed => {
            restricted_observation(ResourceAccessState::Unauthorized)
        }
    }
}

async fn load_resource_observation_provider(
    state: &AppState,
    reference: &TypedResourceReference,
) -> ModuleHttpResult<ResourceObservationProviderLookup> {
    resource_observation_provider(&state.pool, reference)
        .await
        .map_err(|error| {
            tracing::error!(error = ?error, "Generic resource-observation registration failed");
            ModuleHttpError::Internal("generic resource-observation registration failed")
        })
}

fn validate_core_reference_construction(
    reference: &TypedResourceReference,
    account: &AccountContext,
) -> ModuleHttpResult<()> {
    let Some(spec) = core_resource_spec(reference.resource_type().as_str()) else {
        return Err(ModuleHttpError::bad_request(
            "resource_reference_type_unknown",
            "The Core-owned resource type is not registered.",
        ));
    };
    if !has_any_capability(account, spec.capabilities_any_of) {
        return Err(ModuleHttpError::forbidden(
            "resource_reference_capability_required",
            "The current account lacks authority to construct this resource reference.",
        ));
    }
    if parse_canonical_uuid(reference.resource_id()).is_none() {
        return Err(ModuleHttpError::bad_request(
            "resource_reference_id_invalid",
            "The resource identifier does not match the registered Core resource type.",
        ));
    }
    Ok(())
}

/// Resolves Core-owned transition resources without changing the established
/// Form, Workflow, or Response behavior.
async fn resolve_core_reference(
    pool: &PgPool,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    account: &AccountContext,
) -> ModuleHttpResult<ResourceResolutionV1> {
    let Some(spec) = core_resource_spec(reference.resource_type().as_str()) else {
        return restricted(ResourceAccessState::NotEvaluated);
    };
    if !has_any_capability(account, spec.capabilities_any_of) {
        return restricted(ResourceAccessState::Unauthorized);
    }
    if matches!(spec.kind, CoreResourceKind::Response)
        && !account.has_global_capability("submissions:manage")
    {
        return resolve_ownership_bound_response(pool, reference, installation_id, account).await;
    }
    if !has_any_global_capability(account, spec.capabilities_any_of) {
        return restricted(ResourceAccessState::NotEvaluated);
    }

    let owner_state = match reference.owner() {
        ResourceOwner::CoreInstallation {
            installation_id: owner_installation_id,
        } if *owner_installation_id == installation_id
            && reference.installation_id() == installation_id =>
        {
            ResourceOwnerState::CoreInstallation {
                state: CoreInstallationOwnerState::Live,
            }
        }
        ResourceOwner::CoreInstallation { .. } => {
            return authorized(
                ResourceOwnerState::CoreInstallation {
                    state: CoreInstallationOwnerState::InstallationMismatch,
                },
                ResourceIdentityState::NotEvaluated,
                ResourceLifecycleState::NotEvaluated,
                ProviderAvailabilityState::Available,
            );
        }
        ResourceOwner::ModuleInstance { .. } => {
            return restricted(ResourceAccessState::NotEvaluated);
        }
    };

    if parse_canonical_uuid(reference.resource_id()).is_none() {
        return authorized(
            owner_state,
            ResourceIdentityState::UnknownResource,
            ResourceLifecycleState::NotEvaluated,
            ProviderAvailabilityState::Available,
        );
    }

    match load_core_lifecycle(pool, spec.kind, reference.resource_id()).await? {
        Some(state) => authorized(
            owner_state,
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined { state },
            ProviderAvailabilityState::Available,
        ),
        None => authorized(
            owner_state,
            ResourceIdentityState::UnknownResource,
            ResourceLifecycleState::NotEvaluated,
            ProviderAvailabilityState::Available,
        ),
    }
}

async fn resolve_ownership_bound_response(
    pool: &PgPool,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    account: &AccountContext,
) -> ModuleHttpResult<ResourceResolutionV1> {
    let current_installation_owner = matches!(
        reference.owner(),
        ResourceOwner::CoreInstallation {
            installation_id: owner_installation_id
        } if *owner_installation_id == installation_id
            && reference.installation_id() == installation_id
    );
    let Some(submission_id) = current_installation_owner
        .then(|| parse_canonical_uuid(reference.resource_id()))
        .flatten()
    else {
        return restricted(ResourceAccessState::NotEvaluated);
    };

    let lifecycle =
        crate::submissions::reference_lifecycle_if_accessible(pool, account, submission_id)
            .await
            .map_err(|error| {
                tracing::error!(error = ?error, "Response reference access evaluation failed");
                ModuleHttpError::Internal("Response reference access evaluation failed")
            })?;
    let Some(lifecycle) = lifecycle else {
        return restricted(ResourceAccessState::NotEvaluated);
    };

    authorized(
        ResourceOwnerState::CoreInstallation {
            state: CoreInstallationOwnerState::Live,
        },
        ResourceIdentityState::Resolved,
        ResourceLifecycleState::ProviderDefined { state: lifecycle },
        ProviderAvailabilityState::Available,
    )
}

fn restricted(access_state: ResourceAccessState) -> ModuleHttpResult<ResourceResolutionV1> {
    ResourceResolutionV1::restricted(access_state)
        .map_err(|_| ModuleHttpError::Internal("restricted resource projection was invalid"))
}

fn restricted_observation(
    access_state: ResourceAccessState,
) -> ModuleHttpResult<(ResourceResolutionV1, Option<ResourceObservationV1>)> {
    restricted(access_state).map(|resolution| (resolution, None))
}

fn authorized(
    owner_state: ResourceOwnerState,
    identity_state: ResourceIdentityState,
    lifecycle_state: ResourceLifecycleState,
    availability_state: ProviderAvailabilityState,
) -> ModuleHttpResult<ResourceResolutionV1> {
    ResourceResolutionV1::authorized(
        owner_state,
        identity_state,
        lifecycle_state,
        ContractCompatibilityState::Compatible,
        availability_state,
    )
    .map_err(|_| ModuleHttpError::Internal("authorized resource projection was invalid"))
}

fn module_provider_unavailable()
-> ModuleHttpResult<(ResourceResolutionV1, Option<ResourceObservationV1>)> {
    authorized(
        ResourceOwnerState::ModuleInstance {
            instance_state: ModuleInstanceOwnerState::Live,
            data_state: OwnerDataState::Retained,
        },
        ResourceIdentityState::NotEvaluated,
        ResourceLifecycleState::NotEvaluated,
        ProviderAvailabilityState::Unavailable,
    )
    .map(|resolution| (resolution, None))
}

fn owner_mismatch() -> ModuleHttpError {
    ModuleHttpError::bad_request(
        "resource_reference_owner_mismatch",
        "The resource owner does not belong to this installation.",
    )
}

fn has_any_capability(account: &AccountContext, capabilities: &[&str]) -> bool {
    capabilities
        .iter()
        .any(|capability| account.has_capability(capability))
}

fn has_any_global_capability(account: &AccountContext, capabilities: &[&str]) -> bool {
    capabilities
        .iter()
        .any(|capability| account.has_global_capability(capability))
}

fn core_resource_spec(resource_type: &str) -> Option<CoreResourceSpec> {
    let (kind, capabilities_any_of) = match resource_type {
        "tessara.transition.form" => (CoreResourceKind::Form, FORMS),
        "tessara.transition.form_version" => (CoreResourceKind::FormVersion, FORMS),
        "tessara.transition.workflow" => (CoreResourceKind::Workflow, WORKFLOWS),
        "tessara.transition.workflow_version" => (CoreResourceKind::WorkflowVersion, WORKFLOWS),
        "tessara.transition.response" => (CoreResourceKind::Response, RESPONSES),
        _ => return None,
    };
    Some(CoreResourceSpec {
        kind,
        capabilities_any_of,
    })
}

fn parse_canonical_uuid(resource_id: &str) -> Option<Uuid> {
    let parsed = Uuid::parse_str(resource_id).ok()?;
    (parsed.hyphenated().to_string() == resource_id).then_some(parsed)
}

async fn load_core_lifecycle(
    pool: &PgPool,
    kind: CoreResourceKind,
    resource_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let uuid = || parse_canonical_uuid(resource_id).expect("validated UUID resource id");
    match kind {
        CoreResourceKind::Form => {
            sqlx::query_scalar("SELECT 'active'::text FROM forms WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        CoreResourceKind::FormVersion => {
            sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        CoreResourceKind::Workflow => {
            sqlx::query_scalar("SELECT 'active'::text FROM workflows WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        CoreResourceKind::WorkflowVersion => {
            sqlx::query_scalar("SELECT status::text FROM workflow_versions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        CoreResourceKind::Response => {
            sqlx::query_scalar("SELECT status::text FROM submissions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use tessara_module_contract::{ResourceAccessState, ResourceOwner, ResourceTypeId};
    use uuid::Uuid;

    use crate::auth::{AccountContext, CapabilityScope};

    use super::{core_resource_spec, module_provider_unavailable, resolve_core_reference};

    #[test]
    fn core_registry_retains_only_core_owned_transition_resource_types() {
        for resource_type in [
            "tessara.transition.form",
            "tessara.transition.form_version",
            "tessara.transition.workflow",
            "tessara.transition.workflow_version",
            "tessara.transition.response",
        ] {
            assert!(
                core_resource_spec(resource_type).is_some(),
                "{resource_type}"
            );
        }
        for removed in [
            "tessara.transition.dataset",
            "tessara.transition.dataset_revision",
            "tessara.transition.dataset_major_line",
            "tessara.module.release",
            "tessara.module.instance",
        ] {
            assert!(core_resource_spec(removed).is_none(), "{removed}");
        }
    }

    #[test]
    fn module_provider_outage_is_distinct_from_nondisclosure() {
        let (resolution, observation) = module_provider_unavailable().expect("valid projection");
        assert!(observation.is_none());
        assert_eq!(
            resolution.access_state(),
            tessara_module_contract::ResourceAccessState::Authorized
        );
        assert_eq!(
            resolution.availability_state(),
            tessara_module_contract::ProviderAvailabilityState::Unavailable
        );

        let restricted = tessara_module_contract::ResourceResolutionV1::restricted(
            ResourceAccessState::Unauthorized,
        )
        .expect("restricted projection");
        assert_ne!(resolution, restricted);
    }

    #[tokio::test]
    async fn unauthorized_core_resolution_remains_non_disclosing_without_database_access() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://invalid:invalid@127.0.0.1:1/never_connected")
            .expect("lazy pool");
        let installation_id = Uuid::new_v4();

        let mut wires = Vec::new();
        for resource_id in [Uuid::nil(), Uuid::new_v4()] {
            let reference = tessara_module_contract::TypedResourceReference::new(
                installation_id,
                ResourceOwner::CoreInstallation { installation_id },
                ResourceTypeId::new("tessara.transition.form_version").expect("type"),
                resource_id.to_string(),
            )
            .expect("reference");
            let resolution = resolve_core_reference(
                &pool,
                &reference,
                installation_id,
                &account("workflows:read", true),
            )
            .await
            .expect("restricted resolution");
            wires.push(serde_json::to_value(resolution).expect("serialize"));
        }

        assert_eq!(wires[0], wires[1]);
        assert_eq!(wires[0]["owner_state"]["kind"], "undisclosed");
        assert_eq!(wires[0]["resource_identity_state"], "undisclosed");
    }

    fn account(capability: &str, global: bool) -> AccountContext {
        AccountContext {
            account_id: Uuid::nil(),
            email: "reference@example.test".to_string(),
            display_name: "Reference".to_string(),
            is_active: true,
            roles: Vec::new(),
            capabilities: vec![capability.to_string()],
            capability_scopes: vec![CapabilityScope {
                capability: capability.to_string(),
                global,
                node_ids: Vec::new(),
            }],
            scope_nodes: Vec::new(),
            delegations: Vec::new(),
        }
    }
}
