//! Exact-current Components V3 module boundary.
//!
//! The provider owns Component publication, lifecycle, revision, change, and
//! successor meaning. Consumers own their findings and actions. A typed
//! reference or observation grants no authority; callers must separately carry
//! a fresh Core-issued downstream grant for the requested action.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tessara_module_contract::{
    ContractCompatibilityState, ProviderAvailabilityState, ResourceAccessState,
    ResourceIdentityState, ResourceObservationV1, ResourceOwner, ResourceResolutionV1,
    ResourceRevision, TypedResourceReference,
};
use uuid::Uuid;

pub const COMPONENT_CONTRACT_SCHEMA_VERSION: u16 = 3;
pub const COMPONENT_CONTRACT_VERSION: &str = "3.0.0";
pub const COMPONENT_RENDER_RESPONSE_SCHEMA_VERSION: u16 = 1;
pub const COMPONENT_BINDING_KEY: &str = "tessara.dashboards.component-version";
pub const COMPONENT_MODULE_DEFINITION_ID: &str = "tessara.components";
pub const COMPONENT_CONTRACT_ID: &str = "tessara.components.component-version";
pub const COMPONENT_RESOURCE_TYPE: &str = "tessara.components.component_version";

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
    /// Parses the canonical snake-case API value for an executable Component kind.
    pub fn from_api_kind(value: &str) -> Option<Self> {
        match value {
            "table" => Some(Self::Table),
            "bar" => Some(Self::Bar),
            "line" => Some(Self::Line),
            "pie" => Some(Self::Pie),
            "donut" => Some(Self::Donut),
            "stat_card" => Some(Self::StatCard),
            _ => None,
        }
    }

    /// Returns the canonical snake-case API value for this kind.
    pub const fn as_api_value(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Bar => "bar",
            Self::Line => "line",
            Self::Pie => "pie",
            Self::Donut => "donut",
            Self::StatCard => "stat_card",
        }
    }

    /// Returns the canonical product execution-route segment for this kind.
    pub const fn endpoint_segment(self) -> &'static str {
        match self {
            Self::StatCard => "stat-card",
            other => other.as_api_value(),
        }
    }

    /// Returns the stable user-facing label for this kind.
    pub const fn label(self) -> &'static str {
        match self {
            Self::Table => "Table",
            Self::Bar => "Bar",
            Self::Line => "Line",
            Self::Pie => "Pie",
            Self::Donut => "Donut",
            Self::StatCard => "Stat Card",
        }
    }

    pub const fn component_type(self) -> &'static str {
        self.as_api_value()
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
        if !matches!(reference.owner(), ResourceOwner::ModuleInstance { .. }) {
            return Err(ComponentVersionReferenceValidationError::ExpectedModuleInstanceOwner);
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
    #[error("ComponentVersion references must be Module Instance owned")]
    ExpectedModuleInstanceOwner,
    #[error(
        "ComponentVersion reference has resource type '{actual}', expected \
         'tessara.components.component_version'"
    )]
    UnexpectedResourceType { actual: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentResolutionRequest {
    #[serde(deserialize_with = "deserialize_schema_version_v3")]
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
    #[serde(deserialize_with = "deserialize_schema_version_v3")]
    pub schema_version: u16,
    pub action: ComponentAction,
    pub reference: ComponentVersionReference,
    pub kind: ComponentRenderKind,
    pub resource_authority_revision: u64,
    pub query: String,
    pub dashboard_scope_node_ids: Vec<Uuid>,
}

/// One exact Components-owned execution response.
///
/// The wire stays untagged so existing same-origin product routes retain their
/// established JSON bodies. Exact schema, kind, and payload-branch validation
/// is performed by each variant during deserialization.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ComponentRenderResponse {
    Table(Box<ComponentTableResponse>),
    Visual(Box<ComponentVisualResponse>),
}

impl ComponentRenderResponse {
    pub fn table(
        response: ComponentTableResponse,
    ) -> Result<Self, ComponentRenderResponseValidationError> {
        response.validate()?;
        Ok(Self::Table(Box::new(response)))
    }

    pub fn visual(
        response: ComponentVisualResponse,
    ) -> Result<Self, ComponentRenderResponseValidationError> {
        response.validate()?;
        Ok(Self::Visual(Box::new(response)))
    }

    pub fn kind(&self) -> ComponentRenderKind {
        match self {
            Self::Table(response) => response.component_type,
            Self::Visual(response) => response.component_type,
        }
    }

    pub fn component_id(&self) -> Uuid {
        match self {
            Self::Table(response) => response.component_id,
            Self::Visual(response) => response.component_id,
        }
    }

    pub fn component_version_id(&self) -> Uuid {
        match self {
            Self::Table(response) => response.component_version_id,
            Self::Visual(response) => response.component_version_id,
        }
    }

    pub fn as_table(&self) -> Option<&ComponentTableResponse> {
        match self {
            Self::Table(response) => Some(response.as_ref()),
            Self::Visual(_) => None,
        }
    }

    pub fn as_visual(&self) -> Option<&ComponentVisualResponse> {
        match self {
            Self::Table(_) => None,
            Self::Visual(response) => Some(response.as_ref()),
        }
    }

    pub fn validate(&self) -> Result<(), ComponentRenderResponseValidationError> {
        match self {
            Self::Table(response) => response.validate(),
            Self::Visual(response) => response.validate(),
        }
    }

    pub fn validate_for(
        &self,
        expected_kind: ComponentRenderKind,
        expected_component_id: Uuid,
        expected_component_version_id: Uuid,
    ) -> Result<(), ComponentRenderResponseValidationError> {
        self.validate()?;
        if expected_component_id.is_nil()
            || expected_component_version_id.is_nil()
            || self.component_id().is_nil()
            || self.component_version_id().is_nil()
        {
            return Err(ComponentRenderResponseValidationError::PersistentIdentityRequired);
        }
        if self.kind() != expected_kind {
            return Err(ComponentRenderResponseValidationError::KindMismatch {
                expected: expected_kind,
                actual: self.kind(),
            });
        }
        if self.component_id() != expected_component_id {
            return Err(ComponentRenderResponseValidationError::ComponentIdentityMismatch);
        }
        if self.component_version_id() != expected_component_version_id {
            return Err(ComponentRenderResponseValidationError::ComponentVersionIdentityMismatch);
        }
        Ok(())
    }

    /// Validate the explicit unsaved-preview identity.
    ///
    /// Preview output is never a persisted Component resource. Both identity
    /// fields therefore use nil UUIDs only on the authoring preview route;
    /// persisted provider consumers must use [`Self::validate_for`].
    pub fn validate_for_preview(&self) -> Result<(), ComponentRenderResponseValidationError> {
        self.validate()?;
        if !self.component_id().is_nil() || !self.component_version_id().is_nil() {
            return Err(ComponentRenderResponseValidationError::PreviewIdentityRequired);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ComponentTableResponse {
    pub schema_version: u16,
    pub component_version_id: Uuid,
    pub component_id: Uuid,
    pub component_type: ComponentRenderKind,
    pub materialization_state: String,
    pub columns: Vec<ComponentTableColumn>,
    pub rows: Vec<ComponentTableRow>,
    pub pagination: ComponentTablePagination,
}

impl ComponentTableResponse {
    pub fn validate(&self) -> Result<(), ComponentRenderResponseValidationError> {
        validate_render_response_schema_version(self.schema_version)?;
        if self.component_type != ComponentRenderKind::Table {
            return Err(ComponentRenderResponseValidationError::TableKindRequired {
                actual: self.component_type,
            });
        }
        validate_materialization_state(&self.materialization_state)?;
        if self.pagination.page_size == 0
            || self.pagination.has_more != self.pagination.next_cursor.is_some()
        {
            return Err(ComponentRenderResponseValidationError::InvalidPagination);
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentTableResponseWire {
    #[serde(deserialize_with = "deserialize_render_response_schema_version")]
    schema_version: u16,
    component_version_id: Uuid,
    component_id: Uuid,
    component_type: ComponentRenderKind,
    materialization_state: String,
    columns: Vec<ComponentTableColumn>,
    rows: Vec<ComponentTableRow>,
    pagination: ComponentTablePagination,
}

impl<'de> Deserialize<'de> for ComponentTableResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ComponentTableResponseWire::deserialize(deserializer)?;
        let response = Self {
            schema_version: wire.schema_version,
            component_version_id: wire.component_version_id,
            component_id: wire.component_id,
            component_type: wire.component_type,
            materialization_state: wire.materialization_state,
            columns: wire.columns,
            rows: wire.rows,
            pagination: wire.pagination,
        };
        response.validate().map_err(serde::de::Error::custom)?;
        Ok(response)
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentTablePagination {
    pub page_size: u32,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentTableColumn {
    pub key: String,
    pub label: String,
    pub field_type: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentTableRow {
    pub row_id: String,
    pub values: BTreeMap<String, Option<String>>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ComponentVisualResponse {
    pub schema_version: u16,
    pub component_version_id: Uuid,
    pub component_id: Uuid,
    pub component_type: ComponentRenderKind,
    pub materialization_state: String,
    pub value_format: String,
    pub legend_title: Option<String>,
    pub bar_orientation: Option<String>,
    pub bar_comparison_layout: Option<String>,
    pub x_axis_label: Option<String>,
    pub y_axis_label: Option<String>,
    pub line_smoothing: Option<bool>,
    pub stat: Option<ComponentStatValue>,
    pub points: Vec<ComponentVisualPoint>,
    pub slices: Vec<ComponentVisualSlice>,
}

impl ComponentVisualResponse {
    pub fn validate(&self) -> Result<(), ComponentRenderResponseValidationError> {
        validate_render_response_schema_version(self.schema_version)?;
        validate_materialization_state(&self.materialization_state)?;
        let valid_payload = match self.component_type {
            ComponentRenderKind::Table => {
                return Err(ComponentRenderResponseValidationError::VisualKindRequired);
            }
            ComponentRenderKind::Bar => {
                self.stat.is_none() && self.slices.is_empty() && self.line_smoothing.is_none()
            }
            ComponentRenderKind::Line => {
                self.legend_title.is_none()
                    && self.bar_orientation.is_none()
                    && self.bar_comparison_layout.is_none()
                    && self.stat.is_none()
                    && self.slices.is_empty()
                    && self.line_smoothing.is_some()
            }
            ComponentRenderKind::Pie | ComponentRenderKind::Donut => {
                self.bar_orientation.is_none()
                    && self.bar_comparison_layout.is_none()
                    && self.x_axis_label.is_none()
                    && self.y_axis_label.is_none()
                    && self.stat.is_none()
                    && self.points.is_empty()
                    && self.line_smoothing.is_none()
            }
            ComponentRenderKind::StatCard => {
                self.legend_title.is_none()
                    && self.bar_orientation.is_none()
                    && self.bar_comparison_layout.is_none()
                    && self.x_axis_label.is_none()
                    && self.y_axis_label.is_none()
                    && self.line_smoothing.is_none()
                    && self.stat.is_some()
                    && self.points.is_empty()
                    && self.slices.is_empty()
            }
        };
        if !valid_payload {
            return Err(
                ComponentRenderResponseValidationError::InvalidVisualPayload {
                    kind: self.component_type,
                },
            );
        }
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ComponentVisualResponseWire {
    #[serde(deserialize_with = "deserialize_render_response_schema_version")]
    schema_version: u16,
    component_version_id: Uuid,
    component_id: Uuid,
    component_type: ComponentRenderKind,
    materialization_state: String,
    value_format: String,
    legend_title: Option<String>,
    bar_orientation: Option<String>,
    bar_comparison_layout: Option<String>,
    x_axis_label: Option<String>,
    y_axis_label: Option<String>,
    line_smoothing: Option<bool>,
    stat: Option<ComponentStatValue>,
    points: Vec<ComponentVisualPoint>,
    slices: Vec<ComponentVisualSlice>,
}

impl<'de> Deserialize<'de> for ComponentVisualResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = ComponentVisualResponseWire::deserialize(deserializer)?;
        let response = Self {
            schema_version: wire.schema_version,
            component_version_id: wire.component_version_id,
            component_id: wire.component_id,
            component_type: wire.component_type,
            materialization_state: wire.materialization_state,
            value_format: wire.value_format,
            legend_title: wire.legend_title,
            bar_orientation: wire.bar_orientation,
            bar_comparison_layout: wire.bar_comparison_layout,
            x_axis_label: wire.x_axis_label,
            y_axis_label: wire.y_axis_label,
            line_smoothing: wire.line_smoothing,
            stat: wire.stat,
            points: wire.points,
            slices: wire.slices,
        };
        response.validate().map_err(serde::de::Error::custom)?;
        Ok(response)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentStatValue {
    pub label: String,
    pub value: Option<f64>,
    pub display_value: Option<String>,
    pub supporting_text: Option<String>,
    pub panel_style: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentVisualPoint {
    pub x: String,
    pub value: f64,
    pub display_value: String,
    pub color: Option<String>,
    pub comparison: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentVisualSlice {
    pub category: String,
    pub value: f64,
    pub display_value: String,
    pub color: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ComponentRenderResponseValidationError {
    #[error("Component render response schema version {0} is unsupported; expected 1")]
    UnsupportedSchemaVersion(u16),
    #[error("Component render response materialization state must not be blank")]
    EmptyMaterializationState,
    #[error("Component table response requires kind 'table', found '{actual:?}'")]
    TableKindRequired { actual: ComponentRenderKind },
    #[error("Component visual response cannot use kind 'table'")]
    VisualKindRequired,
    #[error("Component table pagination is inconsistent")]
    InvalidPagination,
    #[error("Component visual response payload is invalid for kind '{kind:?}'")]
    InvalidVisualPayload { kind: ComponentRenderKind },
    #[error("Component render response kind mismatch: expected '{expected:?}', found '{actual:?}'")]
    KindMismatch {
        expected: ComponentRenderKind,
        actual: ComponentRenderKind,
    },
    #[error("Component render response names a different Component")]
    ComponentIdentityMismatch,
    #[error("Component render response names a different ComponentVersion")]
    ComponentVersionIdentityMismatch,
    #[error("persisted Component render responses require non-nil Component identities")]
    PersistentIdentityRequired,
    #[error("Component preview render responses require the explicit nil preview identity")]
    PreviewIdentityRequired,
}

fn validate_render_response_schema_version(
    version: u16,
) -> Result<(), ComponentRenderResponseValidationError> {
    if version != COMPONENT_RENDER_RESPONSE_SCHEMA_VERSION {
        return Err(ComponentRenderResponseValidationError::UnsupportedSchemaVersion(version));
    }
    Ok(())
}

fn validate_materialization_state(
    state: &str,
) -> Result<(), ComponentRenderResponseValidationError> {
    if state.trim().is_empty() {
        return Err(ComponentRenderResponseValidationError::EmptyMaterializationState);
    }
    Ok(())
}

fn deserialize_render_response_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    validate_render_response_schema_version(version).map_err(serde::de::Error::custom)?;
    Ok(version)
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentMetadata {
    pub reference: ComponentVersionReference,
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
    #[serde(deserialize_with = "deserialize_schema_version_v3")]
    pub schema_version: u16,
    pub components: Vec<ComponentMetadata>,
}

/// Authorized Components V3 resolution.
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

        if let Some(metadata) = &self.metadata
            && (metadata.reference.reference() != observation.reference()
                || metadata.reference.reference().resource_id()
                    != metadata.component_version_id.to_string())
        {
            return Err(ComponentResolutionValidationError::MetadataReferenceMismatch);
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
    #[error("Components contract schema version {0} is unsupported; expected 3")]
    UnsupportedSchemaVersion(u16),
    #[error("restricted or unresolved Component resolution discloses provider detail")]
    RestrictedOrUnresolvedDisclosure,
    #[error("resolved Component response is missing its typed observation")]
    MissingObservation,
    #[error("resolved Component response names an unexpected provider contract")]
    UnexpectedProviderContract,
    #[error("resolved Component metadata reference does not match its observation or identifier")]
    MetadataReferenceMismatch,
    #[error("resolved non-tombstoned Component response is missing metadata")]
    MissingMetadata,
    #[error("tombstoned Component response discloses metadata or successor detail")]
    TombstoneDisclosesMetadata,
    #[error("Component change has no provider-authored category")]
    EmptyChangeCategories,
    #[error("Component changes are not strictly increasing within the observed revision")]
    InvalidChangeOrder,
}

fn deserialize_schema_version_v3<'de, D>(deserializer: D) -> Result<u16, D::Error>
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
        FunctionalContractId, ModuleInstanceOwnerState, OwnerDataState, ProviderContractIdentity,
        ResourceLifecycleState, ResourceObservationStrategy, ResourceOwnerState,
    };

    use super::*;

    const INSTALLATION_ID: Uuid = Uuid::from_u128(1);

    fn reference() -> ComponentVersionReference {
        ComponentVersionReference::new(
            TypedResourceReference::new(
                INSTALLATION_ID,
                ResourceOwner::ModuleInstance {
                    installation_id: INSTALLATION_ID,
                    module_instance_id: Uuid::from_u128(10),
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
            ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::Live,
                data_state: OwnerDataState::Retained,
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
            reference: reference(),
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

    fn table_response() -> ComponentTableResponse {
        ComponentTableResponse {
            schema_version: COMPONENT_RENDER_RESPONSE_SCHEMA_VERSION,
            component_version_id: Uuid::from_u128(2),
            component_id: Uuid::from_u128(3),
            component_type: ComponentRenderKind::Table,
            materialization_state: "ready".into(),
            columns: vec![ComponentTableColumn {
                key: "amount".into(),
                label: "Amount".into(),
                field_type: "number".into(),
            }],
            rows: vec![ComponentTableRow {
                row_id: "row-1".into(),
                values: BTreeMap::from([("amount".into(), Some("42".into()))]),
            }],
            pagination: ComponentTablePagination {
                page_size: 25,
                next_cursor: Some("next".into()),
                has_more: true,
            },
        }
    }

    fn line_response() -> ComponentVisualResponse {
        ComponentVisualResponse {
            schema_version: COMPONENT_RENDER_RESPONSE_SCHEMA_VERSION,
            component_version_id: Uuid::from_u128(2),
            component_id: Uuid::from_u128(3),
            component_type: ComponentRenderKind::Line,
            materialization_state: "ready".into(),
            value_format: "integer".into(),
            legend_title: None,
            bar_orientation: None,
            bar_comparison_layout: None,
            x_axis_label: Some("Month".into()),
            y_axis_label: Some("Total".into()),
            line_smoothing: Some(true),
            stat: None,
            points: vec![ComponentVisualPoint {
                x: "January".into(),
                value: 42.0,
                display_value: "42".into(),
                color: Some("#3568d4".into()),
                comparison: None,
            }],
            slices: Vec::new(),
        }
    }

    fn visual_response(kind: ComponentRenderKind) -> ComponentVisualResponse {
        let mut response = line_response();
        response.component_type = kind;
        match kind {
            ComponentRenderKind::Table => panic!("table is not a visual response kind"),
            ComponentRenderKind::Bar => {
                response.legend_title = Some("Program".into());
                response.bar_orientation = Some("vertical".into());
                response.bar_comparison_layout = Some("grouped".into());
                response.line_smoothing = None;
            }
            ComponentRenderKind::Line => {}
            ComponentRenderKind::Pie | ComponentRenderKind::Donut => {
                response.legend_title = Some("Program".into());
                response.x_axis_label = None;
                response.y_axis_label = None;
                response.line_smoothing = None;
                response.points.clear();
                response.slices.push(ComponentVisualSlice {
                    category: "Outreach".into(),
                    value: 42.0,
                    display_value: "42".into(),
                    color: Some("#3568d4".into()),
                });
            }
            ComponentRenderKind::StatCard => {
                response.x_axis_label = None;
                response.y_axis_label = None;
                response.line_smoothing = None;
                response.points.clear();
                response.stat = Some(ComponentStatValue {
                    label: "Total".into(),
                    value: Some(42.0),
                    display_value: Some("42".into()),
                    supporting_text: Some("Active programs".into()),
                    panel_style: "default".into(),
                });
            }
        }
        response
    }

    #[test]
    fn render_kind_owns_api_route_and_label_vocabulary() {
        let expected = [
            (ComponentRenderKind::Table, "table", "table", "Table"),
            (ComponentRenderKind::Bar, "bar", "bar", "Bar"),
            (ComponentRenderKind::Line, "line", "line", "Line"),
            (ComponentRenderKind::Pie, "pie", "pie", "Pie"),
            (ComponentRenderKind::Donut, "donut", "donut", "Donut"),
            (
                ComponentRenderKind::StatCard,
                "stat_card",
                "stat-card",
                "Stat Card",
            ),
        ];
        for (kind, api, endpoint, label) in expected {
            assert_eq!(ComponentRenderKind::from_api_kind(api), Some(kind));
            assert_eq!(kind.as_api_value(), api);
            assert_eq!(kind.component_type(), api);
            assert_eq!(kind.endpoint_segment(), endpoint);
            assert_eq!(kind.label(), label);
        }
        assert_eq!(ComponentRenderKind::from_api_kind("stat-card"), None);
        assert_eq!(ComponentRenderKind::from_api_kind("report"), None);
    }

    #[test]
    fn table_render_response_round_trips_and_rejects_inexact_wire() {
        let response = ComponentRenderResponse::table(table_response()).expect("table response");
        response
            .validate_for(
                ComponentRenderKind::Table,
                Uuid::from_u128(3),
                Uuid::from_u128(2),
            )
            .expect("matching response identity");
        let wire = serde_json::to_value(&response).expect("table response wire");
        let decoded: ComponentRenderResponse =
            serde_json::from_value(wire.clone()).expect("typed table response");
        assert_eq!(decoded, response);

        let mut wrong_schema = wire.clone();
        wrong_schema["schema_version"] = json!(2);
        assert!(serde_json::from_value::<ComponentRenderResponse>(wrong_schema).is_err());

        let mut wrong_kind = wire.clone();
        wrong_kind["component_type"] = json!("bar");
        assert!(serde_json::from_value::<ComponentRenderResponse>(wrong_kind).is_err());

        let mut unknown = wire.clone();
        unknown
            .as_object_mut()
            .expect("object")
            .insert("legacy_rows".into(), json!([]));
        assert!(serde_json::from_value::<ComponentRenderResponse>(unknown).is_err());

        let mut leaked_provider_reference = wire.clone();
        leaked_provider_reference
            .as_object_mut()
            .expect("object")
            .insert("dataset_reference".into(), json!({"reference": {}}));
        assert!(
            serde_json::from_value::<ComponentRenderResponse>(leaked_provider_reference).is_err()
        );

        let mut inconsistent_pagination = wire;
        inconsistent_pagination["pagination"]["has_more"] = json!(false);
        assert!(
            serde_json::from_value::<ComponentRenderResponse>(inconsistent_pagination).is_err()
        );
    }

    #[test]
    fn visual_render_response_validates_kind_shape_and_identity() {
        let response =
            ComponentRenderResponse::visual(line_response()).expect("line response shape");
        assert!(response.as_visual().is_some());
        assert!(response.as_table().is_none());
        assert_eq!(
            response.validate_for(
                ComponentRenderKind::Bar,
                Uuid::from_u128(3),
                Uuid::from_u128(2),
            ),
            Err(ComponentRenderResponseValidationError::KindMismatch {
                expected: ComponentRenderKind::Bar,
                actual: ComponentRenderKind::Line,
            })
        );
        assert_eq!(
            response.validate_for_preview(),
            Err(ComponentRenderResponseValidationError::PreviewIdentityRequired)
        );

        let preview = ComponentRenderResponse::visual(ComponentVisualResponse {
            component_version_id: Uuid::nil(),
            component_id: Uuid::nil(),
            ..line_response()
        })
        .expect("preview response shape");
        preview
            .validate_for_preview()
            .expect("explicit preview identity");
        assert_eq!(
            preview.validate_for(ComponentRenderKind::Line, Uuid::nil(), Uuid::nil()),
            Err(ComponentRenderResponseValidationError::PersistentIdentityRequired)
        );

        let partial_preview = ComponentRenderResponse::visual(ComponentVisualResponse {
            component_version_id: Uuid::nil(),
            ..line_response()
        })
        .expect("partial preview response shape");
        assert_eq!(
            partial_preview.validate_for_preview(),
            Err(ComponentRenderResponseValidationError::PreviewIdentityRequired)
        );

        let mut invalid_stat = serde_json::to_value(response).expect("line response wire");
        invalid_stat["component_type"] = json!("stat_card");
        invalid_stat["line_smoothing"] = serde_json::Value::Null;
        assert!(serde_json::from_value::<ComponentRenderResponse>(invalid_stat).is_err());

        let invalid_line = ComponentVisualResponse {
            line_smoothing: None,
            ..line_response()
        };
        assert_eq!(
            invalid_line.validate(),
            Err(
                ComponentRenderResponseValidationError::InvalidVisualPayload {
                    kind: ComponentRenderKind::Line,
                }
            )
        );
    }

    #[test]
    fn visual_render_response_rejects_every_cross_kind_field() {
        for kind in [
            ComponentRenderKind::Bar,
            ComponentRenderKind::Line,
            ComponentRenderKind::Pie,
            ComponentRenderKind::Donut,
            ComponentRenderKind::StatCard,
        ] {
            ComponentRenderResponse::visual(visual_response(kind))
                .unwrap_or_else(|error| panic!("valid {kind:?} response was rejected: {error}"));
        }

        let stat = json!({
            "label": "Total",
            "value": 42.0,
            "display_value": "42",
            "supporting_text": null,
            "panel_style": "default"
        });
        let point = json!({
            "x": "January",
            "value": 42.0,
            "display_value": "42",
            "color": null,
            "comparison": null
        });
        let slice = json!({
            "category": "Outreach",
            "value": 42.0,
            "display_value": "42",
            "color": null
        });
        let invalid_fields = vec![
            (ComponentRenderKind::Bar, "line_smoothing", json!(true)),
            (ComponentRenderKind::Bar, "stat", stat.clone()),
            (ComponentRenderKind::Bar, "slices", json!([slice.clone()])),
            (ComponentRenderKind::Line, "legend_title", json!("Program")),
            (
                ComponentRenderKind::Line,
                "bar_orientation",
                json!("vertical"),
            ),
            (
                ComponentRenderKind::Line,
                "bar_comparison_layout",
                json!("grouped"),
            ),
            (ComponentRenderKind::Line, "line_smoothing", json!(null)),
            (ComponentRenderKind::Line, "stat", stat.clone()),
            (ComponentRenderKind::Line, "slices", json!([slice.clone()])),
            (
                ComponentRenderKind::Pie,
                "bar_orientation",
                json!("vertical"),
            ),
            (
                ComponentRenderKind::Pie,
                "bar_comparison_layout",
                json!("grouped"),
            ),
            (ComponentRenderKind::Pie, "x_axis_label", json!("Month")),
            (ComponentRenderKind::Pie, "y_axis_label", json!("Total")),
            (ComponentRenderKind::Pie, "line_smoothing", json!(true)),
            (ComponentRenderKind::Pie, "stat", stat.clone()),
            (ComponentRenderKind::Pie, "points", json!([point.clone()])),
            (
                ComponentRenderKind::Donut,
                "bar_orientation",
                json!("vertical"),
            ),
            (
                ComponentRenderKind::Donut,
                "bar_comparison_layout",
                json!("grouped"),
            ),
            (ComponentRenderKind::Donut, "x_axis_label", json!("Month")),
            (ComponentRenderKind::Donut, "y_axis_label", json!("Total")),
            (ComponentRenderKind::Donut, "line_smoothing", json!(true)),
            (ComponentRenderKind::Donut, "stat", stat.clone()),
            (ComponentRenderKind::Donut, "points", json!([point.clone()])),
            (
                ComponentRenderKind::StatCard,
                "legend_title",
                json!("Program"),
            ),
            (
                ComponentRenderKind::StatCard,
                "bar_orientation",
                json!("vertical"),
            ),
            (
                ComponentRenderKind::StatCard,
                "bar_comparison_layout",
                json!("grouped"),
            ),
            (
                ComponentRenderKind::StatCard,
                "x_axis_label",
                json!("Month"),
            ),
            (
                ComponentRenderKind::StatCard,
                "y_axis_label",
                json!("Total"),
            ),
            (ComponentRenderKind::StatCard, "line_smoothing", json!(true)),
            (ComponentRenderKind::StatCard, "stat", json!(null)),
            (ComponentRenderKind::StatCard, "points", json!([point])),
            (ComponentRenderKind::StatCard, "slices", json!([slice])),
        ];

        for (kind, field, value) in invalid_fields {
            let response = ComponentRenderResponse::visual(visual_response(kind))
                .expect("valid visual response fixture");
            let mut wire = serde_json::to_value(response).expect("visual response wire");
            wire[field] = value;
            assert!(
                serde_json::from_value::<ComponentRenderResponse>(wire).is_err(),
                "{kind:?} accepted cross-kind field '{field}'"
            );
        }
    }

    #[test]
    fn exact_v3_request_rejects_v2_and_unknown_fields() {
        let request = ComponentResolutionRequest::new(
            ComponentAction::ResolveMetadata,
            reference(),
            Some(ResourceRevision::new(4).expect("revision")),
        );
        let mut wire = serde_json::to_value(request).expect("wire");
        assert_eq!(wire["schema_version"], 3);
        wire["schema_version"] = json!(2);
        assert!(serde_json::from_value::<ComponentResolutionRequest>(wire.clone()).is_err());
        wire["schema_version"] = json!(3);
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
