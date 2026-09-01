//! Current independent Response Module boundary contracts.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tessara_module_contract::{
    ContractCompatibilityState, ModuleInstanceOwnerState, OwnerDataState,
    ProviderAvailabilityState, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationV1, ResourceOwner, ResourceOwnerState, ResourceResolutionV1,
    TypedResourceReference,
};
use uuid::Uuid;

pub const RESPONSE_MODULE_DEFINITION_ID: &str = "tessara.responses";
pub const RESPONSE_CONTRACT_SCHEMA_VERSION: u16 = 2;
pub const RESPONSE_CONTRACT_VERSION: &str = "2.0.0";
pub const RESPONSE_RESOURCE_CONTRACT_ID: &str = "tessara.responses.response";
pub const RESPONSE_LIFECYCLE_CONTRACT_ID: &str = "tessara.responses.response-lifecycle";
pub const RESPONSE_RESOURCE_TYPE: &str = "tessara.responses.response";

pub const RESPONSE_EVENT_CONTRACT_ID: &str = "tessara.responses.response-events";
pub const RESPONSE_EVENT_CONTRACT_VERSION: &str = "1.0.0";
pub const RESPONSE_EVENT_SCHEMA_VERSION: u16 = 1;
pub const RESPONSE_EVENT_BINDING_KEY: &str = "tessara.workflows.response-events";
pub const RESPONSE_EVENT_CHECKPOINT_ACTION: &str = "responses.events_checkpoint";
pub const RESPONSE_EVENT_START_ACTION: &str = "responses.events_start";
pub const RESPONSE_EVENT_PAGE_ACTION: &str = "responses.events_page";
pub const RESPONSE_EVENT_CHECKPOINT_PATH: &str = "/api/private/responses/events/checkpoint";
pub const RESPONSE_EVENT_START_PATH: &str = "/api/private/responses/events/start";
pub const RESPONSE_EVENT_PAGE_PATH: &str = "/api/private/responses/events/page";
pub const RESPONSE_EVENT_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.response-events+json;version=1";
pub const MAX_RESPONSE_EVENT_PAGE_SIZE: u16 = 1_000;

pub const RESPONSE_START_RECONCILIATION_CONTRACT_ID: &str =
    "tessara.responses.start-reconciliation";
pub const RESPONSE_START_RECONCILIATION_CONTRACT_VERSION: &str = "1.0.0";
pub const RESPONSE_START_RECONCILIATION_SCHEMA_VERSION: u16 = 1;
pub const RESPONSE_START_RECONCILIATION_BINDING_KEY: &str =
    "tessara.workflows.response-reconciliation";
pub const RESPONSE_START_RECONCILIATION_ACTION: &str = "responses.reconcile_start";
pub const RESPONSE_START_RECONCILIATION_PATH: &str = "/api/private/responses/start-reconciliation";
pub const RESPONSE_START_RECONCILIATION_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.start-reconciliation+json;version=1";

pub const RESPONSE_FORM_VERSION_USAGE_CONTRACT_ID: &str = "tessara.responses.form-version-usage";
pub const RESPONSE_SUMMARY_CONTRACT_ID: &str = "tessara.responses.summary";
pub const RESPONSE_OPERATIONAL_STATUS_CONTRACT_ID: &str = "tessara.responses.operational-status";
pub const RESPONSE_RESOURCE_OBSERVATION_CONTRACT_ID: &str =
    "tessara.responses.resource-observation";
pub const RESPONSE_REVERSE_CONTRACT_VERSION: &str = "1.0.0";
pub const RESPONSE_REVERSE_SCHEMA_VERSION: u16 = 1;
pub const RESPONSE_FORM_VERSION_USAGE_BINDING_KEY: &str = "tessara.forms.response-usage";
pub const RESPONSE_SUMMARY_BINDING_KEY: &str = "tessara.core.response-summary";
pub const RESPONSE_OPERATIONS_STATUS_BINDING_KEY: &str = "tessara.core.response-operations";
pub const RESPONSE_RESOURCE_OBSERVATION_BINDING_KEY: &str = "tessara.core.responses";
pub const RESPONSE_FORM_VERSION_USAGE_ACTION: &str = "responses.form_version_usage";
pub const RESPONSE_SUMMARY_ACTION: &str = "responses.summary";
pub const RESPONSE_OPERATIONS_STATUS_ACTION: &str = "responses.operations_status";
pub const RESPONSE_RESOLVE_ACTION: &str = "responses.resolve";
pub const RESPONSE_FORM_VERSION_USAGE_PATH: &str = "/api/private/responses/form-version-usage";
pub const RESPONSE_SUMMARY_PATH: &str = "/api/private/responses/summary";
pub const RESPONSE_OPERATIONS_STATUS_PATH: &str = "/api/private/responses/operations-status";
pub const RESPONSE_RESOLVE_PATH: &str = "/api/private/responses/resolve";
pub const RESPONSE_FORM_VERSION_USAGE_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.form-version-usage+json;version=1";
pub const RESPONSE_SUMMARY_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.summary+json;version=1";
pub const RESPONSE_OPERATIONS_STATUS_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.operational-status+json;version=1";
pub const RESPONSE_RESOURCE_OBSERVATION_MEDIA_TYPE: &str = "application/json";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ResponseReference {
    reference: TypedResourceReference,
}

impl ResponseReference {
    pub fn new(reference: TypedResourceReference) -> Result<Self, ResponseReferenceError> {
        if !matches!(reference.owner(), ResourceOwner::ModuleInstance { .. }) {
            return Err(ResponseReferenceError::ExpectedModuleInstanceOwner);
        }
        if reference.resource_type().as_str() != RESPONSE_RESOURCE_TYPE {
            return Err(ResponseReferenceError::UnexpectedResourceType);
        }
        let id = Uuid::parse_str(reference.resource_id())
            .map_err(|_| ResponseReferenceError::InvalidResourceId)?;
        if id.is_nil() || id.to_string() != reference.resource_id() {
            return Err(ResponseReferenceError::InvalidResourceId);
        }
        Ok(Self { reference })
    }

    pub fn from_parts(
        installation_id: Uuid,
        module_instance_id: Uuid,
        response_id: Uuid,
    ) -> Result<Self, ResponseReferenceError> {
        let reference = TypedResourceReference::new(
            installation_id,
            ResourceOwner::ModuleInstance {
                installation_id,
                module_instance_id,
            },
            RESPONSE_RESOURCE_TYPE
                .parse()
                .expect("Response resource type constant is valid"),
            response_id.to_string(),
        )
        .map_err(|_| ResponseReferenceError::InvalidReferenceEnvelope)?;
        Self::new(reference)
    }

    pub const fn reference(&self) -> &TypedResourceReference {
        &self.reference
    }

    pub fn response_id(&self) -> Uuid {
        Uuid::parse_str(self.reference.resource_id()).expect("validated Response reference")
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ResponseReferenceWire {
    reference: TypedResourceReference,
}

impl<'de> Deserialize<'de> for ResponseReference {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Self::new(ResponseReferenceWire::deserialize(deserializer)?.reference)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResponseReferenceError {
    #[error("Response reference envelope is invalid")]
    InvalidReferenceEnvelope,
    #[error("Response reference must be owned by a Module Instance")]
    ExpectedModuleInstanceOwner,
    #[error("Response reference has the wrong resource type")]
    UnexpectedResourceType,
    #[error("Response resource identity must be one canonical non-nil UUID")]
    InvalidResourceId,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseLifecycleState {
    Draft,
    Submitted,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseLifecycleSnapshot {
    #[serde(deserialize_with = "deserialize_response_schema_version")]
    pub schema_version: u16,
    pub response: ResponseReference,
    pub state: ResponseLifecycleState,
    pub revision: u64,
    pub form_version_id: Uuid,
    pub workflow_assignment_id: Uuid,
    pub workflow_instance_id: Uuid,
    pub workflow_step_instance_id: Uuid,
    pub node_id: Uuid,
    pub created_at: String,
    pub modified_at: String,
    pub submitted_at: Option<String>,
    pub form_snapshot_digest: String,
    pub workflow_context_digest: String,
}

impl ResponseLifecycleSnapshot {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        if self.revision == 0
            || [
                self.form_version_id,
                self.workflow_assignment_id,
                self.workflow_instance_id,
                self.workflow_step_instance_id,
                self.node_id,
            ]
            .iter()
            .any(Uuid::is_nil)
            || self.created_at.trim().is_empty()
            || self.modified_at.trim().is_empty()
            || !is_sha256_digest(&self.form_snapshot_digest)
            || !is_sha256_digest(&self.workflow_context_digest)
        {
            return Err(ResponseBoundaryError::InvalidLifecycleSnapshot);
        }
        match (self.state, self.submitted_at.as_deref()) {
            (ResponseLifecycleState::Submitted, Some(value)) if !value.trim().is_empty() => Ok(()),
            (ResponseLifecycleState::Draft | ResponseLifecycleState::Deleted, None) => Ok(()),
            _ => Err(ResponseBoundaryError::InvalidLifecycleSnapshot),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseEventKind {
    Started,
    DraftSaved,
    Submitted,
    Deleted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseWorkflowEvent {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub sequence: u64,
    pub event_id: Uuid,
    pub kind: ResponseEventKind,
    pub response: ResponseReference,
    pub response_revision: u64,
    pub workflow_assignment_id: Uuid,
    pub workflow_instance_id: Uuid,
    pub workflow_step_instance_id: Uuid,
    pub occurred_at: String,
    pub content_digest: String,
}

impl ResponseWorkflowEvent {
    pub fn with_recomputed_digest(mut self) -> Result<Self, ResponseBoundaryError> {
        self.validate_shape()?;
        self.content_digest = self.expected_digest()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        self.validate_shape()?;
        if self.content_digest != self.expected_digest()? {
            return Err(ResponseBoundaryError::DigestMismatch);
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), ResponseBoundaryError> {
        if self.sequence == 0
            || self.response_revision == 0
            || self.event_id.is_nil()
            || self.workflow_assignment_id.is_nil()
            || self.workflow_instance_id.is_nil()
            || self.workflow_step_instance_id.is_nil()
            || self.occurred_at.trim().is_empty()
        {
            return Err(ResponseBoundaryError::InvalidEvent);
        }
        Ok(())
    }

    fn expected_digest(&self) -> Result<String, ResponseBoundaryError> {
        #[derive(Serialize)]
        struct DigestInput<'a> {
            schema_version: u16,
            sequence: u64,
            event_id: &'a Uuid,
            kind: ResponseEventKind,
            response: &'a ResponseReference,
            response_revision: u64,
            workflow_assignment_id: &'a Uuid,
            workflow_instance_id: &'a Uuid,
            workflow_step_instance_id: &'a Uuid,
            occurred_at: &'a str,
        }
        canonical_digest(&DigestInput {
            schema_version: self.schema_version,
            sequence: self.sequence,
            event_id: &self.event_id,
            kind: self.kind,
            response: &self.response,
            response_revision: self.response_revision,
            workflow_assignment_id: &self.workflow_assignment_id,
            workflow_instance_id: &self.workflow_instance_id,
            workflow_step_instance_id: &self.workflow_step_instance_id,
            occurred_at: &self.occurred_at,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEventCheckpointRequest {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub committed_sequence: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEventCheckpointResponse {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub authenticated_head: u64,
    pub committed_sequence_valid: bool,
    pub changed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEventStartRequest {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub committed_sequence: u64,
    pub authenticated_head: u64,
    pub page_size: u16,
}

impl ResponseEventStartRequest {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        if self.provider_epoch.is_nil()
            || self.committed_sequence > self.authenticated_head
            || self.page_size == 0
            || self.page_size > MAX_RESPONSE_EVENT_PAGE_SIZE
        {
            return Err(ResponseBoundaryError::InvalidEventWindow);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEventStartResponse {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub start_after_sequence: u64,
    pub snapshot_upper_bound: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEventPageRequest {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub snapshot_upper_bound: u64,
    pub after_sequence: u64,
    pub page_size: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseEventPageResponse {
    #[serde(deserialize_with = "deserialize_event_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub snapshot_upper_bound: u64,
    pub entries: Vec<ResponseWorkflowEvent>,
    pub next_after_sequence: u64,
    pub complete: bool,
    pub page_digest: String,
}

impl ResponseEventPageResponse {
    pub fn with_recomputed_digest(mut self) -> Result<Self, ResponseBoundaryError> {
        self.validate_shape()?;
        self.page_digest = self.expected_digest()?;
        Ok(self)
    }

    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        self.validate_shape()?;
        if self.page_digest != self.expected_digest()? {
            return Err(ResponseBoundaryError::DigestMismatch);
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), ResponseBoundaryError> {
        if self.provider_epoch.is_nil() || self.next_after_sequence > self.snapshot_upper_bound {
            return Err(ResponseBoundaryError::InvalidEventWindow);
        }
        let mut prior = None;
        for event in &self.entries {
            event.validate()?;
            if event.sequence > self.snapshot_upper_bound
                || prior.is_some_and(|sequence| sequence >= event.sequence)
            {
                return Err(ResponseBoundaryError::InvalidEventOrdering);
            }
            prior = Some(event.sequence);
        }
        if self.entries.last().map(|event| event.sequence) != Some(self.next_after_sequence)
            && !self.entries.is_empty()
        {
            return Err(ResponseBoundaryError::InvalidEventWindow);
        }
        if self.complete && self.next_after_sequence != self.snapshot_upper_bound {
            return Err(ResponseBoundaryError::InvalidEventWindow);
        }
        Ok(())
    }

    fn expected_digest(&self) -> Result<String, ResponseBoundaryError> {
        #[derive(Serialize)]
        struct DigestInput<'a> {
            schema_version: u16,
            provider_epoch: &'a Uuid,
            snapshot_upper_bound: u64,
            entries: &'a [ResponseWorkflowEvent],
            next_after_sequence: u64,
            complete: bool,
        }
        canonical_digest(&DigestInput {
            schema_version: self.schema_version,
            provider_epoch: &self.provider_epoch,
            snapshot_upper_bound: self.snapshot_upper_bound,
            entries: &self.entries,
            next_after_sequence: self.next_after_sequence,
            complete: self.complete,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseStartReconciliationRequest {
    #[serde(deserialize_with = "deserialize_reconciliation_schema_version")]
    pub schema_version: u16,
    pub workflow_assignment_id: Uuid,
    pub workflow_instance_id: Uuid,
    pub workflow_step_instance_id: Uuid,
    pub one_use_nonce: Uuid,
}

impl ResponseStartReconciliationRequest {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        if [
            self.workflow_assignment_id,
            self.workflow_instance_id,
            self.workflow_step_instance_id,
            self.one_use_nonce,
        ]
        .iter()
        .any(Uuid::is_nil)
        {
            return Err(ResponseBoundaryError::InvalidStartReconciliation);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStartReconciliationState {
    Pending,
    Committed,
    Absent,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseStartReconciliationCommit {
    pub response: ResponseReference,
    pub revision: u64,
    pub lifecycle_state: ResponseLifecycleState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseStartReconciliationResponse {
    #[serde(deserialize_with = "deserialize_reconciliation_schema_version")]
    pub schema_version: u16,
    pub state: ResponseStartReconciliationState,
    pub commit: Option<ResponseStartReconciliationCommit>,
}

impl ResponseStartReconciliationResponse {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        match (self.state, self.commit.as_ref()) {
            (ResponseStartReconciliationState::Pending, None)
            | (ResponseStartReconciliationState::Absent, None) => Ok(()),
            (ResponseStartReconciliationState::Committed, Some(commit)) if commit.revision > 0 => {
                Ok(())
            }
            _ => Err(ResponseBoundaryError::InvalidStartReconciliation),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseProviderResultState {
    Available,
    Empty,
    Unavailable,
    Undisclosed,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFormVersionUsageRequest {
    pub schema_version: u16,
    pub form_version_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFormVersionUsageItem {
    pub response: ResponseReference,
    pub lifecycle_state: ResponseLifecycleState,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseFormVersionUsageResponse {
    pub schema_version: u16,
    pub state: ResponseProviderResultState,
    pub items: Vec<ResponseFormVersionUsageItem>,
}

impl ResponseFormVersionUsageResponse {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        validate_reverse_schema(self.schema_version)?;
        validate_result_items(self.state, self.items.len())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseSummaryRequest {
    pub schema_version: u16,
    pub requested_scope_node_ids: Vec<Uuid>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseSummaryResponse {
    pub schema_version: u16,
    pub state: ResponseProviderResultState,
    pub draft_count: Option<u64>,
    pub submitted_count: Option<u64>,
}

impl ResponseSummaryResponse {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        validate_reverse_schema(self.schema_version)?;
        match self.state {
            ResponseProviderResultState::Available
                if self.draft_count.is_none() || self.submitted_count.is_none() =>
            {
                Err(ResponseBoundaryError::MissingCounts)
            }
            ResponseProviderResultState::Empty
                if self.draft_count != Some(0) || self.submitted_count != Some(0) =>
            {
                Err(ResponseBoundaryError::InvalidCounts)
            }
            ResponseProviderResultState::Unavailable | ResponseProviderResultState::Undisclosed
                if self.draft_count.is_some() || self.submitted_count.is_some() =>
            {
                Err(ResponseBoundaryError::RestrictedCarriesData)
            }
            _ => Ok(()),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOperationsStatusRequest {
    pub schema_version: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOperationsStatus {
    pub ready: bool,
    pub forms_binding_compatible: bool,
    pub workflow_binding_compatible: bool,
    pub pending_workflow_event_count: u64,
    pub export_head_sequence: u64,
    pub sanitized_findings: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOperationsStatusResponse {
    pub schema_version: u16,
    pub state: ResponseProviderResultState,
    pub status: Option<ResponseOperationsStatus>,
}

impl ResponseOperationsStatusResponse {
    pub fn validate(&self) -> Result<(), ResponseBoundaryError> {
        validate_reverse_schema(self.schema_version)?;
        match (self.state, self.status.as_ref()) {
            (ResponseProviderResultState::Available, Some(_)) => Ok(()),
            (ResponseProviderResultState::Available, None) => {
                Err(ResponseBoundaryError::AvailableWithoutData)
            }
            (ResponseProviderResultState::Empty, _) => Err(ResponseBoundaryError::InvalidState),
            (_, None) => Ok(()),
            (_, Some(_)) => Err(ResponseBoundaryError::RestrictedCarriesData),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseResourceObservationRequest {
    pub schema_version: u16,
    pub reference: TypedResourceReference,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseResourceObservationResponse {
    pub schema_version: u16,
    pub resolution: ResourceResolutionV1,
    pub observation: Option<ResourceObservationV1>,
}

impl ResponseResourceObservationResponse {
    pub fn validate_for(
        &self,
        requested_reference: &TypedResourceReference,
    ) -> Result<(), ResponseBoundaryError> {
        validate_reverse_schema(self.schema_version)?;
        let disclosed = self.resolution.access_state() == ResourceAccessState::Authorized
            && self.resolution.resource_identity_state() == ResourceIdentityState::Resolved
            && self.resolution.compatibility_state() == ContractCompatibilityState::Compatible
            && self.resolution.availability_state() == ProviderAvailabilityState::Available;
        if !disclosed {
            return if self.observation.is_none() {
                Ok(())
            } else {
                Err(ResponseBoundaryError::RestrictedCarriesData)
            };
        }
        if self.resolution.owner_state()
            != (ResourceOwnerState::ModuleInstance {
                instance_state: ModuleInstanceOwnerState::Live,
                data_state: OwnerDataState::Retained,
            })
            || !matches!(
                self.resolution.resource_lifecycle_state(),
                ResourceLifecycleState::ProviderDefined { .. }
            )
        {
            return Err(ResponseBoundaryError::ObservationOwnerMismatch);
        }
        ResponseReference::new(requested_reference.clone())?;
        let observation = self
            .observation
            .as_ref()
            .ok_or(ResponseBoundaryError::MissingObservation)?;
        if observation.reference() != requested_reference {
            return Err(ResponseBoundaryError::ObservationReferenceMismatch);
        }
        if observation.provider_contract().contract_id().as_str() != RESPONSE_RESOURCE_CONTRACT_ID
            || observation
                .provider_contract()
                .contract_version()
                .to_string()
                != RESPONSE_CONTRACT_VERSION
        {
            return Err(ResponseBoundaryError::ObservationContractMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResponseBoundaryError {
    #[error("Response boundary schema version is unsupported")]
    SchemaVersion,
    #[error("Response lifecycle snapshot is invalid")]
    InvalidLifecycleSnapshot,
    #[error("Response event is invalid")]
    InvalidEvent,
    #[error("Response event window is invalid")]
    InvalidEventWindow,
    #[error("Response events are not strictly ordered")]
    InvalidEventOrdering,
    #[error("Response start reconciliation is invalid")]
    InvalidStartReconciliation,
    #[error("canonical digest does not match")]
    DigestMismatch,
    #[error("boundary data could not be canonicalized")]
    Canonicalization,
    #[error("available result requires data")]
    AvailableWithoutData,
    #[error("available summary requires explicit counts")]
    MissingCounts,
    #[error("summary counts do not match state")]
    InvalidCounts,
    #[error("provider state is invalid for this contract")]
    InvalidState,
    #[error("unavailable, empty, or undisclosed result carries product data")]
    RestrictedCarriesData,
    #[error("resolved resource owner is not the live retained Response instance")]
    ObservationOwnerMismatch,
    #[error("resolved resource response is missing its observation")]
    MissingObservation,
    #[error("resource observation does not echo the requested reference")]
    ObservationReferenceMismatch,
    #[error("resource observation contract is not Response v2")]
    ObservationContractMismatch,
    #[error(transparent)]
    Reference(#[from] ResponseReferenceError),
}

fn validate_result_items(
    state: ResponseProviderResultState,
    count: usize,
) -> Result<(), ResponseBoundaryError> {
    match state {
        ResponseProviderResultState::Available if count == 0 => {
            Err(ResponseBoundaryError::AvailableWithoutData)
        }
        ResponseProviderResultState::Empty
        | ResponseProviderResultState::Unavailable
        | ResponseProviderResultState::Undisclosed
            if count != 0 =>
        {
            Err(ResponseBoundaryError::RestrictedCarriesData)
        }
        _ => Ok(()),
    }
}

fn validate_reverse_schema(version: u16) -> Result<(), ResponseBoundaryError> {
    if version != RESPONSE_REVERSE_SCHEMA_VERSION {
        return Err(ResponseBoundaryError::SchemaVersion);
    }
    Ok(())
}

fn canonical_digest<T: Serialize + ?Sized>(value: &T) -> Result<String, ResponseBoundaryError> {
    let bytes = serde_jcs::to_vec(value).map_err(|_| ResponseBoundaryError::Canonicalization)?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

fn is_sha256_digest(value: &str) -> bool {
    value.strip_prefix("sha256:").is_some_and(|digest| {
        digest.len() == 64
            && digest
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn deserialize_response_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != RESPONSE_CONTRACT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported Response contract schema version",
        ));
    }
    Ok(version)
}

fn deserialize_event_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != RESPONSE_EVENT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported Response event schema version",
        ));
    }
    Ok(version)
}

fn deserialize_reconciliation_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != RESPONSE_START_RECONCILIATION_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(
            "unsupported Response start reconciliation schema version",
        ));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn reference() -> ResponseReference {
        ResponseReference::from_parts(Uuid::from_u128(1), Uuid::from_u128(2), Uuid::from_u128(3))
            .unwrap()
    }

    fn event(sequence: u64) -> ResponseWorkflowEvent {
        ResponseWorkflowEvent {
            schema_version: 1,
            sequence,
            event_id: Uuid::from_u128(10 + u128::from(sequence)),
            kind: ResponseEventKind::Submitted,
            response: reference(),
            response_revision: 2,
            workflow_assignment_id: Uuid::from_u128(4),
            workflow_instance_id: Uuid::from_u128(5),
            workflow_step_instance_id: Uuid::from_u128(6),
            occurred_at: "2026-08-23T12:00:00Z".into(),
            content_digest: String::new(),
        }
        .with_recomputed_digest()
        .unwrap()
    }

    #[test]
    fn response_reference_rejects_core_and_transition_ownership() {
        let installation_id = Uuid::from_u128(1);
        let core = TypedResourceReference::new(
            installation_id,
            ResourceOwner::CoreInstallation { installation_id },
            "tessara.transition.response".parse().unwrap(),
            Uuid::from_u128(3).to_string(),
        )
        .unwrap();
        assert_eq!(
            ResponseReference::new(core),
            Err(ResponseReferenceError::ExpectedModuleInstanceOwner)
        );
    }

    #[test]
    fn workflow_event_page_is_fixed_bound_ordered_and_content_bound() {
        let page = ResponseEventPageResponse {
            schema_version: 1,
            provider_epoch: Uuid::from_u128(9),
            snapshot_upper_bound: 2,
            entries: vec![event(1), event(2)],
            next_after_sequence: 2,
            complete: true,
            page_digest: String::new(),
        }
        .with_recomputed_digest()
        .unwrap();
        assert!(page.validate().is_ok());
        let mut reordered = page.clone();
        reordered.entries.reverse();
        assert_eq!(
            reordered.validate(),
            Err(ResponseBoundaryError::InvalidEventOrdering)
        );
        let mut tampered = page;
        tampered.entries[0].response_revision = 99;
        assert_eq!(
            tampered.validate(),
            Err(ResponseBoundaryError::DigestMismatch)
        );
    }

    #[test]
    fn unavailable_summary_never_impersonates_zero() {
        let unavailable = ResponseSummaryResponse {
            schema_version: 1,
            state: ResponseProviderResultState::Unavailable,
            draft_count: None,
            submitted_count: None,
        };
        assert!(unavailable.validate().is_ok());
        let leaking = ResponseSummaryResponse {
            draft_count: Some(0),
            submitted_count: Some(0),
            ..unavailable
        };
        assert_eq!(
            leaking.validate(),
            Err(ResponseBoundaryError::RestrictedCarriesData)
        );
    }

    #[test]
    fn old_event_shapes_and_versions_are_rejected() {
        let old = json!({"schema_version":0,"committed_sequence":0});
        assert!(serde_json::from_value::<ResponseEventCheckpointRequest>(old).is_err());
        let mirrored = json!({
            "schema_version":1,
            "committed_sequence":0,
            "submission_id":Uuid::from_u128(3)
        });
        assert!(serde_json::from_value::<ResponseEventCheckpointRequest>(mirrored).is_err());
    }

    #[test]
    fn start_reconciliation_states_have_one_exact_payload_shape() {
        for state in [
            ResponseStartReconciliationState::Pending,
            ResponseStartReconciliationState::Absent,
        ] {
            assert!(
                ResponseStartReconciliationResponse {
                    schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
                    state,
                    commit: None,
                }
                .validate()
                .is_ok()
            );
        }
        let committed = ResponseStartReconciliationResponse {
            schema_version: RESPONSE_START_RECONCILIATION_SCHEMA_VERSION,
            state: ResponseStartReconciliationState::Committed,
            commit: Some(ResponseStartReconciliationCommit {
                response: reference(),
                revision: 1,
                lifecycle_state: ResponseLifecycleState::Draft,
            }),
        };
        assert!(committed.validate().is_ok());
        assert_eq!(
            ResponseStartReconciliationResponse {
                state: ResponseStartReconciliationState::Absent,
                ..committed
            }
            .validate(),
            Err(ResponseBoundaryError::InvalidStartReconciliation)
        );
    }
}
