//! Transition adapters for installation-bound typed resource references.

use semver::Version;
use sqlx::PgPool;
use tessara_module_contract::{
    ContractCompatibilityState, CoreInstallationOwnerState, FunctionalContractId,
    ModuleInstanceOwnerState, OwnerDataState, ProviderAvailabilityState, ProviderContractIdentity,
    ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationStrategy, ResourceObservationV1, ResourceOwner, ResourceOwnerState,
    ResourceResolutionV1, ResourceRevision, TypedResourceReference,
};
use uuid::Uuid;

use crate::auth::AccountContext;

use super::{
    dto::{
        CreateResourceReferenceRequestV1, MODULE_HTTP_SCHEMA_VERSION_V1,
        ResourceReferenceResponseV1,
    },
    error::{ModuleHttpError, ModuleHttpResult},
};

#[derive(Clone, Copy, Debug)]
enum ResourceKind {
    Form,
    FormVersion,
    Workflow,
    WorkflowVersion,
    Response,
    Dataset,
    DatasetRevision,
    DatasetMajorLine,
    ComponentVersion,
}

#[derive(Clone, Copy)]
struct ResourceSpec {
    kind: ResourceKind,
    capabilities_any_of: &'static [&'static str],
}

const FORMS: &[&str] = &["forms:read", "forms:manage"];
const WORKFLOWS: &[&str] = &["workflows:read", "workflows:manage"];
const RESPONSES: &[&str] = &[
    "submissions:read_own",
    "submissions:respond",
    "submissions:manage",
];
const DATASETS: &[&str] = &["datasets:read", "datasets:manage"];
const COMPONENTS: &[&str] = &["components:read", "components:manage"];

pub(crate) fn construct(
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
    if !matches!(
        &request.owner,
        ResourceOwner::CoreInstallation {
            installation_id: owner_installation_id
        } if *owner_installation_id == installation_id
    ) {
        return Err(ModuleHttpError::bad_request(
            "resource_reference_owner_mismatch",
            "Sprint 6A transition references must be owned by this Core installation.",
        ));
    }

    let Some(spec) = resource_spec(request.resource_type.as_str()) else {
        return Err(ModuleHttpError::bad_request(
            "resource_reference_type_unknown",
            "The resource type is not registered for a Sprint 6A transition adapter.",
        ));
    };
    if !has_any_capability(account, spec.capabilities_any_of) {
        return Err(ModuleHttpError::forbidden(
            "resource_reference_capability_required",
            "The current account lacks authority to construct this resource reference.",
        ));
    }
    if !valid_resource_id(spec.kind, &request.resource_id) {
        return Err(ModuleHttpError::bad_request(
            "resource_reference_id_invalid",
            "The resource identifier does not match the registered transition resource type.",
        ));
    }

    let reference = TypedResourceReference::new(
        request.installation_id,
        request.owner,
        request.resource_type,
        request.resource_id,
    )
    .map_err(|_| {
        ModuleHttpError::bad_request(
            "resource_reference_invalid",
            "The resource reference is structurally invalid.",
        )
    })?;

    Ok(ResourceReferenceResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        reference,
    })
}

/// Resolves an adapter reference without consulting product data until the
/// actor has installation-global product authority. Scoped authority is
/// deliberately projected as `not_evaluated`: the adapter cannot infer the
/// provider's row-level scope decision without disclosing resource existence.
pub(crate) async fn resolve(
    pool: &PgPool,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    account: &AccountContext,
) -> ModuleHttpResult<ResourceResolutionV1> {
    let Some(spec) = resource_spec(reference.resource_type().as_str()) else {
        return restricted(ResourceAccessState::NotEvaluated);
    };

    if !has_any_capability(account, spec.capabilities_any_of) {
        return restricted(ResourceAccessState::Unauthorized);
    }
    if matches!(spec.kind, ResourceKind::Response)
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
            return authorized(
                ResourceOwnerState::ModuleInstance {
                    instance_state: ModuleInstanceOwnerState::UnknownModuleInstance,
                    data_state: OwnerDataState::NotEvaluated,
                },
                ResourceIdentityState::NotEvaluated,
                ResourceLifecycleState::NotEvaluated,
                ProviderAvailabilityState::Unavailable,
            );
        }
    };

    if !valid_resource_id(spec.kind, reference.resource_id()) {
        return authorized(
            owner_state,
            ResourceIdentityState::UnknownResource,
            ResourceLifecycleState::NotEvaluated,
            ProviderAvailabilityState::Available,
        );
    }

    let lifecycle = load_lifecycle(pool, spec.kind, reference.resource_id()).await?;
    match lifecycle {
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

/// Produces an exact live observation only after the same authorization-first
/// resolution used by the transition reference adapter. Restricted and unknown
/// resources never carry an observation envelope.
pub(crate) async fn observe(
    pool: &PgPool,
    reference: &TypedResourceReference,
    installation_id: Uuid,
    account: &AccountContext,
) -> ModuleHttpResult<(ResourceResolutionV1, Option<ResourceObservationV1>)> {
    let resolution = resolve(pool, reference, installation_id, account).await?;
    if resolution.access_state() != ResourceAccessState::Authorized {
        return Ok((resolution, None));
    }
    let Some(spec) = resource_spec(reference.resource_type().as_str()) else {
        return Ok((resolution, None));
    };
    let Some((contract_id, contract_version)) = observation_contract(spec.kind) else {
        return Ok((resolution, None));
    };
    let revision = match spec.kind {
        ResourceKind::DatasetRevision => {
            let Some(id) = parse_canonical_uuid(reference.resource_id()) else {
                return Ok((resolution, None));
            };
            let revision = sqlx::query_scalar::<_, i64>(
                "SELECT resource_revision FROM dataset_revisions WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?;
            revision
        }
        ResourceKind::ComponentVersion => {
            let Some(id) = parse_canonical_uuid(reference.resource_id()) else {
                return Ok((resolution, None));
            };
            let revision = sqlx::query_scalar::<_, i64>(
                "SELECT resource_revision FROM component_versions WHERE id = $1",
            )
            .bind(id)
            .fetch_optional(pool)
            .await?;
            revision
        }
        _ => return Ok((resolution, None)),
    };
    let Some(revision) = revision else {
        return Ok((resolution, None));
    };
    let revision = u64::try_from(revision)
        .ok()
        .and_then(|value| ResourceRevision::new(value).ok())
        .ok_or(ModuleHttpError::Internal(
            "provider resource revision was invalid",
        ))?;
    let contract_id = FunctionalContractId::new(contract_id)
        .map_err(|_| ModuleHttpError::Internal("provider contract identity was invalid"))?;
    let contract_version = Version::parse(contract_version)
        .map_err(|_| ModuleHttpError::Internal("provider contract version was invalid"))?;
    Ok((
        resolution,
        Some(ResourceObservationV1::new(
            reference.clone(),
            ProviderContractIdentity::new(contract_id, contract_version),
            ResourceObservationStrategy::LiveResolutionWithRevision,
            revision,
        )),
    ))
}

fn observation_contract(kind: ResourceKind) -> Option<(&'static str, &'static str)> {
    match kind {
        ResourceKind::DatasetRevision => Some(("tessara.datasets.dataset-revision", "1.0.0")),
        ResourceKind::ComponentVersion => Some((
            tessara_components_contract::COMPONENT_CONTRACT_ID,
            tessara_components_contract::COMPONENT_CONTRACT_VERSION,
        )),
        _ => None,
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

fn resource_spec(resource_type: &str) -> Option<ResourceSpec> {
    let (kind, capabilities_any_of) = match resource_type {
        "tessara.transition.form" => (ResourceKind::Form, FORMS),
        "tessara.transition.form_version" => (ResourceKind::FormVersion, FORMS),
        "tessara.transition.workflow" => (ResourceKind::Workflow, WORKFLOWS),
        "tessara.transition.workflow_version" => (ResourceKind::WorkflowVersion, WORKFLOWS),
        "tessara.transition.response" => (ResourceKind::Response, RESPONSES),
        "tessara.transition.dataset" => (ResourceKind::Dataset, DATASETS),
        "tessara.transition.dataset_revision" => (ResourceKind::DatasetRevision, DATASETS),
        "tessara.transition.dataset_major_line" => (ResourceKind::DatasetMajorLine, DATASETS),
        "tessara.transition.component_version" => (ResourceKind::ComponentVersion, COMPONENTS),
        _ => return None,
    };
    Some(ResourceSpec {
        kind,
        capabilities_any_of,
    })
}

fn valid_resource_id(kind: ResourceKind, resource_id: &str) -> bool {
    match kind {
        ResourceKind::DatasetMajorLine => parse_dataset_major_line(resource_id).is_some(),
        _ => parse_canonical_uuid(resource_id).is_some(),
    }
}

fn parse_canonical_uuid(resource_id: &str) -> Option<Uuid> {
    let parsed = Uuid::parse_str(resource_id).ok()?;
    (parsed.hyphenated().to_string() == resource_id).then_some(parsed)
}

fn parse_dataset_major_line(resource_id: &str) -> Option<(Uuid, i32)> {
    let (dataset_id, major) = resource_id.split_once(':')?;
    let dataset_id = parse_canonical_uuid(dataset_id)?;
    let major = major.parse::<i32>().ok()?;
    if major < 0 {
        return None;
    }
    // Reject alternate integer spellings such as +1 and 01.
    let (_, source_major) = resource_id.split_once(':')?;
    (source_major == major.to_string()).then_some((dataset_id, major))
}

async fn load_lifecycle(
    pool: &PgPool,
    kind: ResourceKind,
    resource_id: &str,
) -> Result<Option<String>, sqlx::Error> {
    let uuid = || parse_canonical_uuid(resource_id).expect("validated UUID resource id");
    match kind {
        ResourceKind::Form => {
            sqlx::query_scalar("SELECT 'active'::text FROM forms WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::FormVersion => {
            sqlx::query_scalar("SELECT status::text FROM form_versions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::Workflow => {
            sqlx::query_scalar("SELECT 'active'::text FROM workflows WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::WorkflowVersion => {
            sqlx::query_scalar("SELECT status::text FROM workflow_versions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::Response => {
            sqlx::query_scalar("SELECT status::text FROM submissions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::Dataset => {
            sqlx::query_scalar("SELECT 'active'::text FROM datasets WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::DatasetRevision => {
            sqlx::query_scalar("SELECT status::text FROM dataset_revisions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
        ResourceKind::DatasetMajorLine => {
            let (dataset_id, major) =
                parse_dataset_major_line(resource_id).expect("validated major-line id");
            sqlx::query_scalar(
                "SELECT rebuild_status FROM dataset_major_materializations \
                 WHERE dataset_id = $1 AND version_major = $2",
            )
            .bind(dataset_id)
            .bind(major)
            .fetch_optional(pool)
            .await
        }
        ResourceKind::ComponentVersion => {
            sqlx::query_scalar("SELECT status::text FROM component_versions WHERE id = $1")
                .bind(uuid())
                .fetch_optional(pool)
                .await
        }
    }
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use tessara_module_contract::{
        FunctionalContractId, ProviderContractIdentity, ResourceAccessState,
        ResourceObservationStrategy, ResourceObservationV1, ResourceOwner, ResourceRevision,
        ResourceTypeId, TypedResourceReference,
    };
    use uuid::Uuid;

    use crate::auth::{AccountContext, CapabilityScope};

    use super::{
        CreateResourceReferenceRequestV1, ResourceKind, construct, observation_contract,
        parse_dataset_major_line, resource_spec,
    };

    #[test]
    fn registry_contains_every_transition_resource_type_and_no_release_instance_type() {
        for resource_type in [
            "tessara.transition.form",
            "tessara.transition.form_version",
            "tessara.transition.workflow",
            "tessara.transition.workflow_version",
            "tessara.transition.response",
            "tessara.transition.dataset",
            "tessara.transition.dataset_revision",
            "tessara.transition.dataset_major_line",
            "tessara.transition.component_version",
        ] {
            assert!(resource_spec(resource_type).is_some(), "{resource_type}");
        }
        assert!(resource_spec("tessara.module.release").is_none());
        assert!(resource_spec("tessara.module.instance").is_none());
    }

    #[test]
    fn construction_rejects_foreign_owner_before_returning_a_reference() {
        let installation_id = Uuid::new_v4();
        let error = construct(
            CreateResourceReferenceRequestV1 {
                schema_version: 1,
                installation_id,
                owner: ResourceOwner::CoreInstallation {
                    installation_id: Uuid::new_v4(),
                },
                resource_type: ResourceTypeId::new("tessara.transition.form_version")
                    .expect("resource type"),
                resource_id: Uuid::new_v4().to_string(),
            },
            installation_id,
            &account("forms:read", true),
        )
        .expect_err("foreign owner fails");

        assert_eq!(error.code(), "resource_reference_owner_mismatch");
    }

    #[test]
    fn dataset_major_line_identifier_is_canonical_and_unambiguous() {
        let dataset_id = Uuid::new_v4();
        assert_eq!(
            parse_dataset_major_line(&format!("{dataset_id}:12")),
            Some((dataset_id, 12))
        );
        assert_eq!(parse_dataset_major_line(&format!("{dataset_id}:01")), None);
        assert_eq!(parse_dataset_major_line(&format!("{dataset_id}:-1")), None);
        assert_eq!(parse_dataset_major_line(&format!("{dataset_id}:1:2")), None);
    }

    #[test]
    fn contract_can_build_restricted_shape_for_known_and_random_identifiers() {
        let installation_id = Uuid::new_v4();
        for resource_id in [Uuid::nil().to_string(), Uuid::new_v4().to_string()] {
            let reference = TypedResourceReference::new(
                installation_id,
                ResourceOwner::CoreInstallation { installation_id },
                ResourceTypeId::new("tessara.transition.dashboard").expect("type"),
                resource_id,
            )
            .expect("reference");
            assert_eq!(reference.installation_id(), installation_id);
        }

        let resolution = tessara_module_contract::ResourceResolutionV1::restricted(
            ResourceAccessState::Unauthorized,
        )
        .expect("restricted projection");
        let wire = serde_json::to_value(resolution).expect("serialize");
        assert_eq!(wire["access_state"], "unauthorized");
        assert_eq!(wire["owner_state"]["kind"], "undisclosed");
    }

    #[tokio::test]
    async fn unauthorized_and_scoped_only_resolution_never_touch_data_or_disclose_identity() {
        let pool = sqlx::postgres::PgPoolOptions::new()
            .connect_lazy("postgres://invalid:invalid@127.0.0.1:1/never_connected")
            .expect("lazy pool");
        let installation_id = Uuid::new_v4();
        let known_shape_id = Uuid::nil().to_string();
        let random_id = Uuid::new_v4().to_string();

        for actor in [account("datasets:read", false), account("forms:read", true)] {
            let mut wires = Vec::new();
            for resource_id in [&known_shape_id, &random_id] {
                let reference = TypedResourceReference::new(
                    installation_id,
                    ResourceOwner::CoreInstallation { installation_id },
                    ResourceTypeId::new("tessara.transition.dataset_revision").expect("type"),
                    resource_id,
                )
                .expect("reference");
                let resolution = super::resolve(&pool, &reference, installation_id, &actor)
                    .await
                    .expect("restricted resolution");
                wires.push(serde_json::to_value(resolution).expect("serialize"));
                let (resolution, observation) =
                    super::observe(&pool, &reference, installation_id, &actor)
                        .await
                        .expect("restricted observation");
                assert!(observation.is_none());
                assert_eq!(
                    serde_json::to_value(resolution).expect("serialize"),
                    *wires.last().expect("resolution wire")
                );
            }
            assert_eq!(wires[0], wires[1]);
            assert_eq!(wires[0]["owner_state"]["kind"], "undisclosed");
            assert_eq!(wires[0]["resource_identity_state"], "undisclosed");
        }
    }

    #[test]
    fn dataset_and_component_adapters_share_observation_semantics_but_not_provider_identity() {
        let installation_id = Uuid::new_v4();
        let owner = ResourceOwner::CoreInstallation { installation_id };
        let resource_id = Uuid::new_v4().to_string();
        let make = |kind: ResourceKind, resource_type: &str, revision: u64| {
            let (contract_id, contract_version) =
                observation_contract(kind).expect("observable transition kind");
            ResourceObservationV1::new(
                TypedResourceReference::new(
                    installation_id,
                    owner.clone(),
                    ResourceTypeId::new(resource_type).expect("resource type"),
                    resource_id.clone(),
                )
                .expect("reference"),
                ProviderContractIdentity::new(
                    FunctionalContractId::new(contract_id).expect("contract id"),
                    Version::parse(contract_version).expect("contract version"),
                ),
                ResourceObservationStrategy::LiveResolutionWithRevision,
                ResourceRevision::new(revision).expect("revision"),
            )
        };
        let dataset = make(
            ResourceKind::DatasetRevision,
            "tessara.transition.dataset_revision",
            4,
        );
        let component = make(
            ResourceKind::ComponentVersion,
            "tessara.transition.component_version",
            4,
        );
        assert_eq!(dataset.strategy(), component.strategy());
        assert_eq!(dataset.resource_revision(), component.resource_revision());
        assert_eq!(dataset.reference().owner(), component.reference().owner());
        assert_eq!(
            dataset.reference().resource_id(),
            component.reference().resource_id()
        );
        assert_ne!(
            dataset.provider_contract().contract_id(),
            component.provider_contract().contract_id()
        );
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
