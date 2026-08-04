//! Exact-current Components V2 compatibility boundary.
//!
//! The provider owns Component publication, lifecycle, revision, change, and
//! successor meaning. Consumers own their findings and actions. A typed
//! reference or observation grants no authority; callers must separately carry
//! a fresh Core-issued downstream grant for the requested action.

use serde::{Deserialize, Serialize};
use tessara_module_contract::{
    ContractCompatibilityState, ProviderAvailabilityState, ResourceAccessState,
    ResourceIdentityState, ResourceObservationV1, ResourceOwner, ResourceResolutionV1,
    ResourceRevision, TypedResourceReference,
};
use uuid::Uuid;

pub const COMPONENT_CONTRACT_SCHEMA_VERSION: u16 = 2;
pub const COMPONENT_CONTRACT_VERSION: &str = "2.0.0";
pub const COMPONENT_BINDING_KEY: &str = "tessara.dashboards.component-version";
pub const COMPONENT_CONTRACT_ID: &str = "tessara.components.component-version";
pub const COMPONENT_RESOURCE_TYPE: &str = "tessara.transition.component_version";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentAction {
    ResolveMetadata,
    Render,
}

impl ComponentAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResolveMetadata => "resolve_metadata",
            Self::Render => "render",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentRenderKind {
    Table,
    Bar,
    Line,
    Pie,
    Donut,
    StatCard,
}

impl ComponentRenderKind {
    pub const fn component_type(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Pie => "pie",
            Self::Donut => "donut",
            Self::StatCard => "stat_card",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentPublicationState {
    Draft,
    Published,
    Superseded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentLifecycleState {
    Active,
    Inactive,
    Archived,
    Tombstoned,
}

impl ComponentLifecycleState {
    pub const fn metadata_visible(self) -> bool {
        !matches!(self, Self::Tombstoned)
    }

    pub const fn renderable(self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentChangeCategory {
    Publication,
    Lifecycle,
    Payload,
    Successor,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentChange {
    pub resource_revision: ResourceRevision,
    pub categories: Vec<ComponentChangeCategory>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ComponentVersionReference {
    reference: TypedResourceReference,
}

impl ComponentVersionReference {
    pub fn new(
        reference: TypedResourceReference,
    ) -> Result<Self, ComponentVersionReferenceValidationError> {
        reference.validate()?;
        if !matches!(reference.owner(), ResourceOwner::CoreInstallation { .. }) {
            return Err(ComponentVersionReferenceValidationError::ExpectedCoreInstallationOwner);
        }
        if reference.resource_type().as_str() != COMPONENT_RESOURCE_TYPE {
            return Err(
                ComponentVersionReferenceValidationError::UnexpectedResourceType {
                    actual: reference.resource_type().as_str().to_string(),
                },
            );
        }
        Ok(Self { reference })
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentVersionReferenceWire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for ComponentVersionReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ComponentVersionReferenceWire::deserialize(deserializer)?;
        Self::new(wire.reference).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ComponentVersionReferenceValidationError {
    #[error(transparent)]
    InvalidReference(#[from] tessara_module_contract::ReferenceValidationError),
    #[error("ComponentVersion references must be Core installation owned")]
    ExpectedCoreInstallationOwner,
    #[error(
        "ComponentVersion reference has resource type '{actual}', expected \
         'tessara.transition.component_version'"
    )]
    UnexpectedResourceType { actual: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentResolutionRequest {
    #[serde(deserialize_with = "deserialize_schema_version_v2")]
    pub schema_version: u16,
    pub action: ComponentAction,
    pub reference: ComponentVersionReference,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changes_since_revision: Option<ResourceRevision>,
}

impl ComponentResolutionRequest {
    pub fn new(
        action: ComponentAction,
        reference: ComponentVersionReference,
        changes_since_revision: Option<ResourceRevision>,
    ) -> Self {
        Self {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            action,
            reference,
            changes_since_revision,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentRenderRequest {
    #[serde(deserialize_with = "deserialize_schema_version_v2")]
    pub schema_version: u16,
    pub action: ComponentAction,
    pub reference: ComponentVersionReference,
    pub kind: ComponentRenderKind,
    pub resource_authority_revision: u64,
    pub query: String,
    pub dashboard_scope_node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentMetadata {
    pub component_version_id: Uuid,
    pub component_id: Uuid,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub publication_state: ComponentPublicationState,
    pub lifecycle_state: ComponentLifecycleState,
    pub authority_revision: u64,
    pub scope_node_ids: Vec<Uuid>,
}

impl ComponentMetadata {
    pub const fn renderable(&self) -> bool {
        matches!(
            self.publication_state,
            ComponentPublicationState::Published | ComponentPublicationState::Superseded
        ) && self.lifecycle_state.renderable()
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSuccessor {
    pub reference: ComponentVersionReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version_v2")]
    pub schema_version: u16,
    pub components: Vec<ComponentMetadata>,
}

/// Authorized Components V2 resolution.
///
/// Restricted and unresolved results carry no observation, metadata, change,
/// or successor detail. An authorized tombstone carries the typed observation
/// but suppresses all metadata and successor detail.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ComponentResolutionResponse {
    schema_version: u16,
    resolution: ResourceResolutionV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    observation: Option<ResourceObservationV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<ComponentMetadata>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    changes: Vec<ComponentChange>,
    #[serde(skip_serializing_if = "Option::is_none")]
    successor: Option<ComponentSuccessor>,
}

impl ComponentResolutionResponse {
    pub fn new(
        resolution: ResourceResolutionV1,
        observation: Option<ResourceObservationV1>,
        metadata: Option<ComponentMetadata>,
        changes: Vec<ComponentChange>,
        successor: Option<ComponentSuccessor>,
    ) -> Result<Self, ComponentResolutionValidationError> {
        let response = Self {
            schema_version: COMPONENT_CONTRACT_SCHEMA_VERSION,
            resolution,
            observation,
            metadata,
            changes,
            successor,
        };
        response.validate()?;
        Ok(response)
    }

    pub const fn resolution(&self) -> &ResourceResolutionV1 {
        &self.resolution
    }

    pub const fn observation(&self) -> Option<&ResourceObservationV1> {
        self.observation.as_ref()
    }

    pub const fn metadata(&self) -> Option<&ComponentMetadata> {
        self.metadata.as_ref()
    }

    pub fn changes(&self) -> &[ComponentChange] {
        &self.changes
    }

    pub const fn successor(&self) -> Option<&ComponentSuccessor> {
        self.successor.as_ref()
    }

    fn validate(&self) -> Result<(), ComponentResolutionValidationError> {
        let disclosed_resolution = self.resolution.access_state()
            == ResourceAccessState::Authorized
            && self.resolution.resource_identity_state() == ResourceIdentityState::Resolved
            && self.resolution.compatibility_state() == ContractCompatibilityState::Compatible
            && self.resolution.availability_state() == ProviderAvailabilityState::Available;
        if !disclosed_resolution {
            if self.observation.is_some()
                || self.metadata.is_some()
                || !self.changes.is_empty()
                || self.successor.is_some()
            {
                return Err(ComponentResolutionValidationError::RestrictedOrUnresolvedDisclosure);
            }
            return Ok(());
        }

        let observation = self
            .observation
            .as_ref()
            .ok_or(ComponentResolutionValidationError::MissingObservation)?;
        if observation.provider_contract().contract_id().as_str() != COMPONENT_CONTRACT_ID
            || observation
                .provider_contract()
                .contract_version()
                .to_string()
                != COMPONENT_CONTRACT_VERSION
        {
            return Err(ComponentResolutionValidationError::UnexpectedProviderContract);
        }

        let tombstoned = matches!(
            self.resolution.resource_lifecycle_state(),
            tessara_module_contract::ResourceLifecycleState::ProviderDefined { state }
                if state == "tombstoned"
        );
        if tombstoned {
            if self.metadata.is_some() || self.successor.is_some() {
                return Err(ComponentResolutionValidationError::TombstoneDisclosesMetadata);
            }
        } else if self.metadata.is_none() {
            return Err(ComponentResolutionValidationError::MissingMetadata);
        }

        let current_revision = observation.resource_revision();
        let mut previous = None;
        for change in &self.changes {
            if change.categories.is_empty() {
                return Err(ComponentResolutionValidationError::EmptyChangeCategories);
            }
            if change.resource_revision > current_revision
                || previous.is_some_and(|prior| change.resource_revision <= prior)
            {
                return Err(ComponentResolutionValidationError::InvalidChangeOrder);
            }
            previous = Some(change.resource_revision);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentResolutionResponseWire {
    schema_version: u16,
    resolution: ResourceResolutionV1,
    observation: Option<ResourceObservationV1>,
    metadata: Option<ComponentMetadata>,
    #[serde(default)]
    changes: Vec<ComponentChange>,
    successor: Option<ComponentSuccessor>,
}

impl<'de> Deserialize<'de> for ComponentResolutionResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ComponentResolutionResponseWire::deserialize(deserializer)?;
        if wire.schema_version != COMPONENT_CONTRACT_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(
                ComponentResolutionValidationError::UnsupportedSchemaVersion(wire.schema_version),
            ));
        }
        Self::new(
            wire.resolution,
            wire.observation,
            wire.metadata,
            wire.changes,
            wire.successor,
        )
        .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ComponentResolutionValidationError {
    #[error("Components contract schema version {0} is unsupported; expected 2")]
    UnsupportedSchemaVersion(u16),
    #[error("restricted or unresolved Component resolution discloses provider detail")]
    RestrictedOrUnresolvedDisclosure,
    #[error("resolved Component response is missing its typed observation")]
    MissingObservation,
    #[error("resolved Component response names an unexpected provider contract")]
    UnexpectedProviderContract,
    #[error("resolved non-tombstoned Component response is missing metadata")]
    MissingMetadata,
    #[error("tombstoned Component response discloses metadata or successor detail")]
    TombstoneDisclosesMetadata,
    #[error("Component change has no provider-authored category")]
    EmptyChangeCategories,
    #[error("Component changes are not strictly increasing within the observed revision")]
    InvalidChangeOrder,
}

fn deserialize_schema_version_v2<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != COMPONENT_CONTRACT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Components contract schema version {version} is unsupported; expected {COMPONENT_CONTRACT_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use semver::Version;
    use serde_json::json;
    use tessara_module_contract::{
        CoreInstallationOwnerState, FunctionalContractId, ProviderContractIdentity,
        ResourceLifecycleState, ResourceObservationStrategy, ResourceOwnerState,
    };

    use super::*;

    const INSTALLATION_ID: Uuid = Uuid::from_u128(1);

    fn reference() -> ComponentVersionReference {
        ComponentVersionReference::new(
            TypedResourceReference::new(
                INSTALLATION_ID,
                ResourceOwner::CoreInstallation {
                    installation_id: INSTALLATION_ID,
                },
                COMPONENT_RESOURCE_TYPE.parse().expect("resource type"),
                Uuid::from_u128(2).to_string(),
            )
            .expect("reference"),
        )
        .expect("component reference")
    }

    fn observation(revision: u64) -> ResourceObservationV1 {
        ResourceObservationV1::new(
            reference().reference().clone(),
            ProviderContractIdentity::new(
                FunctionalContractId::new(COMPONENT_CONTRACT_ID).expect("contract"),
                Version::parse(COMPONENT_CONTRACT_VERSION).expect("version"),
            ),
            ResourceObservationStrategy::LiveResolutionWithRevision,
            ResourceRevision::new(revision).expect("revision"),
        )
    }

    fn resolution(lifecycle: &str) -> ResourceResolutionV1 {
        ResourceResolutionV1::authorized(
            ResourceOwnerState::CoreInstallation {
                state: CoreInstallationOwnerState::Live,
            },
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: lifecycle.into(),
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        )
        .expect("resolution")
    }

    fn metadata(lifecycle_state: ComponentLifecycleState) -> ComponentMetadata {
        ComponentMetadata {
            component_version_id: Uuid::from_u128(2),
            component_id: Uuid::from_u128(3),
            component_name: "Program Snapshot".into(),
            component_slug: "program-snapshot".into(),
            component_type: "table".into(),
            version_number: 1,
            version_label: "v1".into(),
            publication_state: ComponentPublicationState::Published,
            lifecycle_state,
            authority_revision: 9,
            scope_node_ids: vec![Uuid::from_u128(4)],
        }
    }

    #[test]
    fn exact_v2_request_rejects_v1_and_unknown_fields() {
        let request = ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            reference(),
            Some(ResourceRevision::new(4).expect("revision")),
        );
        let mut wire = serde_json::to_value(request).expect("wire");
        assert_eq!(wire["schema_version"], 2);
        wire["schema_version"] = json!(1);
        assert!(serde_json::from_value::<ComponentResolutionRequest>(wire.clone()).is_err());
        wire["schema_version"] = json!(2);
        wire.as_object_mut()
            .expect("object")
            .insert("fallback_version".into(), json!(1));
        assert!(serde_json::from_value::<ComponentResolutionRequest>(wire).is_err());
    }

    #[test]
    fn lifecycle_and_publication_are_distinct_and_drive_renderability() {
        let mut value = metadata(ComponentLifecycleState::Active);
        assert!(value.renderable());
        value.lifecycle_state = ComponentLifecycleState::Inactive;
        assert!(!value.renderable());
        value.lifecycle_state = ComponentLifecycleState::Active;
        value.publication_state = ComponentPublicationState::Draft;
        assert!(!value.renderable());
    }

    #[test]
    fn restricted_and_tombstoned_shapes_do_not_disclose_metadata() {
        let restricted = ResourceResolutionV1::restricted(ResourceAccessState::Unauthorized)
            .expect("restricted");
        assert!(
            ComponentResolutionResponse::new(
                restricted,
                Some(observation(1)),
                None,
                Vec::new(),
                None,
            )
            .is_err()
        );

        let tombstone = ComponentResolutionResponse::new(
            resolution("tombstoned"),
            Some(observation(5)),
            None,
            vec![ComponentChange {
                resource_revision: ResourceRevision::new(5).expect("revision"),
                categories: vec![ComponentChangeCategory::Lifecycle],
            }],
            None,
        )
        .expect("tombstone");
        assert!(tombstone.metadata().is_none());
        assert!(tombstone.successor().is_none());
    }

    #[test]
    fn change_markers_are_nonempty_ordered_and_bounded_by_observation() {
        let valid = ComponentResolutionResponse::new(
            resolution("active"),
            Some(observation(5)),
            Some(metadata(ComponentLifecycleState::Active)),
            vec![
                ComponentChange {
                    resource_revision: ResourceRevision::new(3).expect("revision"),
                    categories: vec![ComponentChangeCategory::Payload],
                },
                ComponentChange {
                    resource_revision: ResourceRevision::new(5).expect("revision"),
                    categories: vec![ComponentChangeCategory::Successor],
                },
            ],
            None,
        );
        assert!(valid.is_ok());

        let invalid = ComponentResolutionResponse::new(
            resolution("active"),
            Some(observation(5)),
            Some(metadata(ComponentLifecycleState::Active)),
            vec![ComponentChange {
                resource_revision: ResourceRevision::new(6).expect("revision"),
                categories: vec![ComponentChangeCategory::Payload],
            }],
            None,
        );
        assert!(invalid.is_err());
    }
}
