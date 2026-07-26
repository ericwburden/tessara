//! Dashboard's temporary first-party ComponentVersion compatibility boundary.
//!
//! Sprint 6C stores references to the in-process Components provider without
//! pretending that provider is a Module Instance. The reference grants no
//! authority; callers must separately obtain a Core-issued downstream grant
//! for one of the declared actions before resolving it.

use serde::{Deserialize, Serialize};
use tessara_module_contract::{
    ContractCompatibilityState, ProviderAvailabilityState, ResourceAccessState,
    ResourceIdentityState, ResourceOwner, ResourceResolutionV1, TypedResourceReference,
};
use uuid::Uuid;

/// Manifest binding key for Dashboard's transition-only Components dependency.
pub const DASHBOARD_COMPONENT_BINDING_KEY: &str = "tessara.dashboards.component-version";

/// First-party Core Release contract used to resolve ComponentVersion data.
pub const DASHBOARD_COMPONENT_CONTRACT_ID: &str = "tessara.components.component-version";

/// Core-owned transition resource type accepted by Dashboard placements.
pub const DASHBOARD_COMPONENT_RESOURCE_TYPE: &str = "tessara.transition.component_version";

/// Explicit actions exposed by the temporary Components compatibility adapter.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DashboardComponentTransitionAction {
    /// Resolve the metadata needed by Dashboard authoring and placement state.
    ResolveMetadata,
    /// Render or execute the referenced ComponentVersion for a Dashboard view.
    Render,
}

/// Versioned request sent to the first-party Core Components adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardComponentResolutionRequestV1 {
    pub schema_version: u16,
    pub action: DashboardComponentTransitionAction,
    pub reference: DashboardComponentVersionReferenceV1,
}

impl DashboardComponentResolutionRequestV1 {
    pub fn new(
        action: DashboardComponentTransitionAction,
        reference: DashboardComponentVersionReferenceV1,
    ) -> Self {
        Self {
            schema_version: 1,
            action,
            reference,
        }
    }
}

/// Authorized metadata projected by the transition Components adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardComponentMetadataV1 {
    pub component_version_id: Uuid,
    pub component_id: Uuid,
    pub component_name: String,
    pub component_slug: String,
    pub component_type: String,
    pub version_number: i32,
    pub version_label: String,
    pub version_status: String,
    /// Provider-authoritative Organization scope used by Dashboard to prevent
    /// a placement from broadening visibility beyond its Component data.
    pub scope_node_ids: Vec<Uuid>,
}

/// Authorized placeable ComponentVersion metadata returned to the Dashboard
/// editor. The adapter filters this collection before any item reaches the
/// Dashboard process.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardComponentCatalogResponseV1 {
    pub schema_version: u16,
    pub components: Vec<DashboardComponentMetadataV1>,
}

/// Version-one action-bound resolution returned to Dashboard.
///
/// Metadata is present only for a fully authorized, resolved, compatible, and
/// available reference. All restricted outcomes therefore serialize to the
/// same metadata-free shape.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DashboardComponentResolutionResponseV1 {
    schema_version: u16,
    resolution: ResourceResolutionV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<DashboardComponentMetadataV1>,
}

impl DashboardComponentResolutionResponseV1 {
    pub fn new(
        resolution: ResourceResolutionV1,
        metadata: Option<DashboardComponentMetadataV1>,
    ) -> Result<Self, DashboardComponentResolutionValidationError> {
        let may_disclose_metadata = resolution.access_state() == ResourceAccessState::Authorized
            && resolution.resource_identity_state() == ResourceIdentityState::Resolved
            && resolution.compatibility_state() == ContractCompatibilityState::Compatible
            && resolution.availability_state() == ProviderAvailabilityState::Available;
        if metadata.is_some() != may_disclose_metadata {
            return Err(
                DashboardComponentResolutionValidationError::MetadataDoesNotMatchResolution,
            );
        }
        Ok(Self {
            schema_version: 1,
            resolution,
            metadata,
        })
    }

    pub const fn resolution(&self) -> &ResourceResolutionV1 {
        &self.resolution
    }

    pub const fn metadata(&self) -> Option<&DashboardComponentMetadataV1> {
        self.metadata.as_ref()
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DashboardComponentResolutionResponseV1Wire {
    schema_version: u16,
    resolution: ResourceResolutionV1,
    metadata: Option<DashboardComponentMetadataV1>,
}

impl<'de> Deserialize<'de> for DashboardComponentResolutionResponseV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DashboardComponentResolutionResponseV1Wire::deserialize(deserializer)?;
        if wire.schema_version != 1 {
            return Err(serde::de::Error::custom(
                DashboardComponentResolutionValidationError::UnsupportedSchemaVersion(
                    wire.schema_version,
                ),
            ));
        }
        Self::new(wire.resolution, wire.metadata).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DashboardComponentResolutionValidationError {
    #[error("Dashboard Component resolution schema version {0} is unsupported")]
    UnsupportedSchemaVersion(u16),
    #[error("Component metadata presence does not match the resource resolution")]
    MetadataDoesNotMatchResolution,
}

impl DashboardComponentTransitionAction {
    /// Stable action value bound into Core authorization grants.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ResolveMetadata => "resolve_metadata",
            Self::Render => "render",
        }
    }
}

/// Validated Dashboard placement reference to a transition ComponentVersion.
///
/// The inner platform reference remains installation-scoped and
/// non-authoritative. This wrapper narrows it to the exact Core-owned resource
/// type allowed during Sprint 6C and prevents accidental persistence of a
/// Module Instance owner or another resource kind.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct DashboardComponentVersionReferenceV1 {
    reference: TypedResourceReference,
}

impl DashboardComponentVersionReferenceV1 {
    /// Validates and wraps one transition ComponentVersion reference.
    pub fn new(
        reference: TypedResourceReference,
    ) -> Result<Self, DashboardComponentVersionReferenceValidationError> {
        reference.validate()?;

        if !matches!(reference.owner(), ResourceOwner::CoreInstallation { .. }) {
            return Err(
                DashboardComponentVersionReferenceValidationError::ExpectedCoreInstallationOwner,
            );
        }
        if reference.resource_type().as_str() != DASHBOARD_COMPONENT_RESOURCE_TYPE {
            return Err(
                DashboardComponentVersionReferenceValidationError::UnexpectedResourceType {
                    actual: reference.resource_type().as_str().to_string(),
                },
            );
        }

        Ok(Self { reference })
    }

    /// Returns the installation-scoped platform reference.
    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct DashboardComponentVersionReferenceV1Wire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for DashboardComponentVersionReferenceV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = DashboardComponentVersionReferenceV1Wire::deserialize(deserializer)?;
        Self::new(wire.reference).map_err(serde::de::Error::custom)
    }
}

/// Invalid Dashboard transition ComponentVersion reference.
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DashboardComponentVersionReferenceValidationError {
    #[error(transparent)]
    InvalidReference(#[from] tessara_module_contract::ReferenceValidationError),
    #[error("Dashboard transition ComponentVersion references must be Core installation owned")]
    ExpectedCoreInstallationOwner,
    #[error(
        "Dashboard transition ComponentVersion reference has resource type '{actual}', expected \
         'tessara.transition.component_version'"
    )]
    UnexpectedResourceType { actual: String },
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tessara_module_contract::{
        ContractCompatibilityState, CoreInstallationOwnerState, ProviderAvailabilityState,
        ResourceAccessState, ResourceIdentityState, ResourceLifecycleState, ResourceOwnerState,
        ResourceResolutionV1,
    };
    use uuid::Uuid;

    use super::{
        DASHBOARD_COMPONENT_BINDING_KEY, DASHBOARD_COMPONENT_CONTRACT_ID,
        DASHBOARD_COMPONENT_RESOURCE_TYPE, DashboardComponentMetadataV1,
        DashboardComponentResolutionRequestV1, DashboardComponentResolutionResponseV1,
        DashboardComponentTransitionAction, DashboardComponentVersionReferenceV1,
    };

    const INSTALLATION_ID: &str = "11111111-1111-4111-8111-111111111111";
    const INSTANCE_ID: &str = "22222222-2222-4222-8222-222222222222";

    fn reference(owner: serde_json::Value, resource_type: &str) -> serde_json::Value {
        json!({
            "reference": {
                "installation_id": INSTALLATION_ID,
                "owner": owner,
                "resource_type": resource_type,
                "resource_id": "33333333-3333-4333-8333-333333333333"
            }
        })
    }

    #[test]
    fn transition_vocabulary_matches_the_existing_manifest_identity() {
        assert_eq!(
            DASHBOARD_COMPONENT_BINDING_KEY,
            "tessara.dashboards.component-version"
        );
        assert_eq!(
            DASHBOARD_COMPONENT_CONTRACT_ID,
            "tessara.components.component-version"
        );
        assert_eq!(
            DASHBOARD_COMPONENT_RESOURCE_TYPE,
            "tessara.transition.component_version"
        );
        assert_eq!(
            DashboardComponentTransitionAction::ResolveMetadata.as_str(),
            "resolve_metadata"
        );
        assert_eq!(
            DashboardComponentTransitionAction::Render.as_str(),
            "render"
        );
    }

    #[test]
    fn accepts_only_core_owned_transition_component_versions() {
        let core_owned = reference(
            json!({
                "kind": "core_installation",
                "installation_id": INSTALLATION_ID
            }),
            DASHBOARD_COMPONENT_RESOURCE_TYPE,
        );
        let parsed: DashboardComponentVersionReferenceV1 =
            serde_json::from_value(core_owned.clone()).expect("valid transition reference");
        assert_eq!(
            serde_json::to_value(parsed).expect("serialize reference"),
            core_owned
        );

        let module_owned = reference(
            json!({
                "kind": "module_instance",
                "installation_id": INSTALLATION_ID,
                "module_instance_id": INSTANCE_ID
            }),
            DASHBOARD_COMPONENT_RESOURCE_TYPE,
        );
        assert!(
            serde_json::from_value::<DashboardComponentVersionReferenceV1>(module_owned).is_err()
        );

        let wrong_type = reference(
            json!({
                "kind": "core_installation",
                "installation_id": INSTALLATION_ID
            }),
            "tessara.transition.dashboard",
        );
        assert!(
            serde_json::from_value::<DashboardComponentVersionReferenceV1>(wrong_type).is_err()
        );
    }

    #[test]
    fn rejects_cross_installation_and_unknown_fields() {
        let cross_installation = reference(
            json!({
                "kind": "core_installation",
                "installation_id": "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"
            }),
            DASHBOARD_COMPONENT_RESOURCE_TYPE,
        );
        assert!(
            serde_json::from_value::<DashboardComponentVersionReferenceV1>(cross_installation)
                .is_err()
        );

        let mut extra = reference(
            json!({
                "kind": "core_installation",
                "installation_id": INSTALLATION_ID
            }),
            DASHBOARD_COMPONENT_RESOURCE_TYPE,
        );
        extra
            .as_object_mut()
            .expect("wrapper object")
            .insert("authority".into(), json!("implied"));
        assert!(serde_json::from_value::<DashboardComponentVersionReferenceV1>(extra).is_err());
    }

    #[test]
    fn action_request_is_versioned_and_rejects_unknown_fields() {
        let wrapped: DashboardComponentVersionReferenceV1 = serde_json::from_value(reference(
            json!({
                "kind": "core_installation",
                "installation_id": INSTALLATION_ID
            }),
            DASHBOARD_COMPONENT_RESOURCE_TYPE,
        ))
        .expect("reference");
        let request = DashboardComponentResolutionRequestV1::new(
            DashboardComponentTransitionAction::ResolveMetadata,
            wrapped,
        );
        let mut wire = serde_json::to_value(request).expect("wire");
        assert_eq!(wire["schema_version"], 1);
        assert_eq!(wire["action"], "resolve_metadata");
        wire.as_object_mut()
            .expect("object")
            .insert("provider_hint".into(), json!("components"));
        assert!(serde_json::from_value::<DashboardComponentResolutionRequestV1>(wire).is_err());
    }

    #[test]
    fn restricted_resolution_can_never_carry_metadata() {
        let restricted =
            ResourceResolutionV1::restricted(ResourceAccessState::Unauthorized).expect("shape");
        assert!(
            DashboardComponentResolutionResponseV1::new(restricted.clone(), Some(metadata()))
                .is_err()
        );
        let response =
            DashboardComponentResolutionResponseV1::new(restricted, None).expect("restricted");
        let wire = serde_json::to_value(response).expect("wire");
        assert!(wire.get("metadata").is_none());
        assert_eq!(wire["resolution"]["access_state"], "unauthorized");
    }

    #[test]
    fn metadata_requires_fully_resolved_compatible_available_state() {
        let owner_state = ResourceOwnerState::CoreInstallation {
            state: CoreInstallationOwnerState::Live,
        };
        let resolved = ResourceResolutionV1::authorized(
            owner_state,
            ResourceIdentityState::Resolved,
            ResourceLifecycleState::ProviderDefined {
                state: "published".into(),
            },
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Available,
        )
        .expect("resolved");
        assert!(DashboardComponentResolutionResponseV1::new(resolved, Some(metadata())).is_ok());

        let unavailable = ResourceResolutionV1::authorized(
            owner_state,
            ResourceIdentityState::NotEvaluated,
            ResourceLifecycleState::NotEvaluated,
            ContractCompatibilityState::Compatible,
            ProviderAvailabilityState::Unavailable,
        )
        .expect("unavailable");
        assert!(
            DashboardComponentResolutionResponseV1::new(unavailable, Some(metadata())).is_err()
        );
    }

    fn metadata() -> DashboardComponentMetadataV1 {
        DashboardComponentMetadataV1 {
            component_version_id: Uuid::from_u128(1),
            component_id: Uuid::from_u128(2),
            component_name: "Program Snapshot".into(),
            component_slug: "program-snapshot".into(),
            component_type: "table".into(),
            version_number: 1,
            version_label: "v1".into(),
            version_status: "published".into(),
            scope_node_ids: vec![Uuid::from_u128(3)],
        }
    }
}
