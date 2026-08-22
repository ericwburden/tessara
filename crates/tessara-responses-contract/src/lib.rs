//! Canonical Responses-owned submitted-response export contract.
//!
//! The provider owns ordering, cursor meaning, snapshot bounds, scope filtering,
//! and tombstone production. Consumers must treat every cursor as opaque and
//! may publish imported state only after a complete fixed-bound export.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use tessara_module_contract::ModuleServicePrincipalV1;
use uuid::Uuid;

pub const RESPONSE_EXPORT_CONTRACT_ID: &str = "tessara.responses.submitted-response-export";
pub const RESPONSE_EXPORT_CONTRACT_VERSION: &str = "1.0.0";
pub const RESPONSE_EXPORT_SCHEMA_VERSION: u16 = 1;
pub const RESPONSE_EXPORT_BINDING_KEY: &str = "tessara.datasets.response-export";
pub const RESPONSE_EXPORT_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.submitted-response-export+json;version=1";
pub const RESPONSE_EXPORT_CHECKPOINT_ACTION: &str = "responses.export_checkpoint";
pub const RESPONSE_EXPORT_START_ACTION: &str = "responses.export_start";
pub const RESPONSE_EXPORT_PAGE_ACTION: &str = "responses.export_page";
pub const RESPONSE_EXPORT_CHECKPOINT_PATH: &str =
    "/api/private/responses/submitted-export/checkpoint";
pub const RESPONSE_EXPORT_START_PATH: &str = "/api/private/responses/submitted-export/start";
pub const RESPONSE_EXPORT_PAGE_PATH: &str = "/api/private/responses/submitted-export/page";
pub const MAX_EXPORT_PAGE_SIZE: u16 = 1_000;

/// Core-owned administrative lifecycle surface used by owner-controlled
/// materialization and recovery fixtures. It is intentionally a product
/// mutation contract rather than a database-fixture backdoor.
pub const RESPONSE_OWNER_ACTION_PATH: &str = "/api/admin/responses/owner-actions";
pub const RESPONSE_OWNER_ACTION_MEDIA_TYPE: &str =
    "application/vnd.tessara.responses.owner-action+json;version=1";
pub const RESPONSE_OWNER_ACTION_IDEMPOTENCY_HEADER: &str = "x-idempotency-key";
pub const RESPONSE_OWNER_ACTION_SCHEMA_VERSION: u16 = 1;
pub const RESPONSE_OWNER_RECEIPT_SIGNING_ISSUER: &str = "tessara.core";
pub const RESPONSE_OWNER_RECEIPT_SIGNING_KEY_ID: &str = "core-development-v1";
pub const RESPONSE_OWNER_RECEIPT_VERIFICATION_KEY_ENV: &str =
    "TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseOwnerAction {
    Create,
    Correct,
    StatusOut,
    StatusIn,
    Redact,
    Delete,
}

impl ResponseOwnerAction {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Create => "create",
            Self::Correct => "correct",
            Self::StatusOut => "status_out",
            Self::StatusIn => "status_in",
            Self::Redact => "redact",
            Self::Delete => "delete",
        }
    }
}

/// Strict, action-tagged request. `values` is a complete FormVersion value set:
/// callers must include every field key, using JSON null for an empty optional
/// value. Corrections therefore cannot accidentally leave an older value in
/// the immutable aggregate envelope.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResponseOwnerActionRequest {
    Create {
        #[serde(deserialize_with = "deserialize_owner_action_schema_version")]
        schema_version: u16,
        logical_key: String,
        form_version_id: Uuid,
        node_id: Uuid,
        values: BTreeMap<String, Value>,
    },
    Correct {
        #[serde(deserialize_with = "deserialize_owner_action_schema_version")]
        schema_version: u16,
        logical_key: String,
        response_id: Uuid,
        values: BTreeMap<String, Value>,
    },
    StatusOut {
        #[serde(deserialize_with = "deserialize_owner_action_schema_version")]
        schema_version: u16,
        logical_key: String,
        response_id: Uuid,
    },
    StatusIn {
        #[serde(deserialize_with = "deserialize_owner_action_schema_version")]
        schema_version: u16,
        logical_key: String,
        response_id: Uuid,
    },
    Redact {
        #[serde(deserialize_with = "deserialize_owner_action_schema_version")]
        schema_version: u16,
        logical_key: String,
        response_id: Uuid,
    },
    Delete {
        #[serde(deserialize_with = "deserialize_owner_action_schema_version")]
        schema_version: u16,
        logical_key: String,
        response_id: Uuid,
    },
}

impl ResponseOwnerActionRequest {
    pub const fn action(&self) -> ResponseOwnerAction {
        match self {
            Self::Create { .. } => ResponseOwnerAction::Create,
            Self::Correct { .. } => ResponseOwnerAction::Correct,
            Self::StatusOut { .. } => ResponseOwnerAction::StatusOut,
            Self::StatusIn { .. } => ResponseOwnerAction::StatusIn,
            Self::Redact { .. } => ResponseOwnerAction::Redact,
            Self::Delete { .. } => ResponseOwnerAction::Delete,
        }
    }

    pub fn logical_key(&self) -> &str {
        match self {
            Self::Create { logical_key, .. }
            | Self::Correct { logical_key, .. }
            | Self::StatusOut { logical_key, .. }
            | Self::StatusIn { logical_key, .. }
            | Self::Redact { logical_key, .. }
            | Self::Delete { logical_key, .. } => logical_key,
        }
    }

    pub const fn target_response_id(&self) -> Option<Uuid> {
        match self {
            Self::Create { .. } => None,
            Self::Correct { response_id, .. }
            | Self::StatusOut { response_id, .. }
            | Self::StatusIn { response_id, .. }
            | Self::Redact { response_id, .. }
            | Self::Delete { response_id, .. } => Some(*response_id),
        }
    }

    pub fn validate_logical_key(&self) -> Result<(), ResponseOwnerActionValidationError> {
        let value = self.logical_key();
        let bytes = value.as_bytes();
        if bytes.is_empty()
            || bytes.len() > 128
            || !bytes[0].is_ascii_lowercase()
            || bytes.iter().any(|byte| {
                !(byte.is_ascii_lowercase()
                    || byte.is_ascii_digit()
                    || matches!(byte, b'.' | b'-' | b'_' | b'/' | b':'))
            })
        {
            return Err(ResponseOwnerActionValidationError::InvalidLogicalKey);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResponseOwnerActionValidationError {
    #[error("Response logical key is invalid")]
    InvalidLogicalKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseOwnerExportKind {
    Upsert,
    Tombstone,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOwnerExportReceipt {
    pub change_sequence: u64,
    pub change_kind: ResponseOwnerExportKind,
    pub tombstone_reason: Option<ResponseTombstoneReason>,
    pub content_digest: String,
}

/// Purpose-bound payload signed by Core after the owner transaction has
/// mutated the aggregate and appended its one final export change.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOwnerActionReceipt {
    pub schema_version: u16,
    pub logical_key: String,
    pub action: ResponseOwnerAction,
    pub actor_account_id: Uuid,
    pub response_id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub node_id: Uuid,
    pub method: String,
    pub path: String,
    pub raw_body_digest: String,
    pub idempotency_key_digest: String,
    pub committed_at: String,
    pub export: ResponseOwnerExportReceipt,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseOwnerActionResponse {
    pub schema_version: u16,
    pub replayed: bool,
    pub signed_receipt: tessara_module_contract::SignedEnvelopeV1<ResponseOwnerActionReceipt>,
}

/// Provider-issued position. Its contents have no consumer-visible semantics.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct ResponseExportCursor(String);

impl ResponseExportCursor {
    pub fn parse(value: impl Into<String>) -> Result<Self, CursorValidationError> {
        let value = value.into();
        if value.is_empty() || value.len() > 1024 || value.chars().any(char::is_whitespace) {
            return Err(CursorValidationError);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl<'de> Deserialize<'de> for ResponseExportCursor {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::parse(value).map_err(serde::de::Error::custom)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
#[error("Response export cursor must be 1..=1024 non-whitespace characters")]
pub struct CursorValidationError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportPartition {
    pub source_binding_id: Uuid,
    pub form_version_ids: Vec<Uuid>,
    pub authorized_scope_node_ids: Vec<Uuid>,
    /// Digest of the authenticated audience, action, and requested scope.
    pub authorization_digest: String,
}

impl ResponseExportPartition {
    pub fn with_recomputed_authorization_digest(
        mut self,
        presenting_service: &ModuleServicePrincipalV1,
        installation_id: Uuid,
    ) -> Result<Self, ResponseExportPartitionDigestError> {
        self.authorization_digest =
            self.expected_authorization_digest(presenting_service, installation_id)?;
        Ok(self)
    }

    pub fn recompute_authorization_digest(
        &mut self,
        presenting_service: &ModuleServicePrincipalV1,
        installation_id: Uuid,
    ) -> Result<(), ResponseExportPartitionDigestError> {
        self.authorization_digest =
            self.expected_authorization_digest(presenting_service, installation_id)?;
        Ok(())
    }

    pub fn validate_authorization_digest(
        &self,
        presenting_service: &ModuleServicePrincipalV1,
        installation_id: Uuid,
    ) -> Result<String, ResponseExportPartitionDigestError> {
        let expected = self.expected_authorization_digest(presenting_service, installation_id)?;
        if self.authorization_digest == expected {
            Ok(expected)
        } else {
            Err(ResponseExportPartitionDigestError::Mismatch)
        }
    }

    fn expected_authorization_digest(
        &self,
        presenting_service: &ModuleServicePrincipalV1,
        installation_id: Uuid,
    ) -> Result<String, ResponseExportPartitionDigestError> {
        #[derive(Serialize)]
        struct DigestInput<'a> {
            source_binding_id: &'a Uuid,
            form_version_ids: &'a [Uuid],
            authorized_scope_node_ids: &'a [Uuid],
            presenting_service: &'a ModuleServicePrincipalV1,
            installation_id: &'a Uuid,
        }

        let mut form_version_ids = self.form_version_ids.clone();
        form_version_ids.sort_unstable();
        form_version_ids.dedup();
        let mut authorized_scope_node_ids = self.authorized_scope_node_ids.clone();
        authorized_scope_node_ids.sort_unstable();
        authorized_scope_node_ids.dedup();
        let input = DigestInput {
            source_binding_id: &self.source_binding_id,
            form_version_ids: &form_version_ids,
            authorized_scope_node_ids: &authorized_scope_node_ids,
            presenting_service,
            installation_id: &installation_id,
        };
        let bytes = serde_jcs::to_vec(&input)
            .map_err(|_| ResponseExportPartitionDigestError::Canonicalization)?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResponseExportPartitionDigestError {
    #[error("the Response export partition authorization binding could not be canonicalized")]
    Canonicalization,
    #[error("the Response export partition authorization digest does not match its binding")]
    Mismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseExportAction {
    Checkpoint,
    Start,
    Page,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportCheckpointRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: ResponseExportAction,
    pub partition: ResponseExportPartition,
    pub committed_cursor: Option<ResponseExportCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportCheckpointResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub authenticated_head: ResponseExportCursor,
    pub committed_cursor_valid: bool,
    pub changed: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportStartRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: ResponseExportAction,
    pub partition: ResponseExportPartition,
    pub provider_epoch: Uuid,
    pub committed_cursor: Option<ResponseExportCursor>,
    pub authenticated_head: ResponseExportCursor,
    pub full_snapshot_rebase: bool,
    pub page_size: u16,
}

impl ResponseExportStartRequest {
    pub fn validate(&self) -> Result<(), ResponseExportValidationError> {
        if self.action != ResponseExportAction::Start {
            return Err(ResponseExportValidationError::WrongAction);
        }
        if self.page_size == 0 || self.page_size > MAX_EXPORT_PAGE_SIZE {
            return Err(ResponseExportValidationError::InvalidPageSize);
        }
        if self.full_snapshot_rebase && self.committed_cursor.is_some() {
            return Err(ResponseExportValidationError::RebaseWithCommittedCursor);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportStartResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub start_after_cursor: Option<ResponseExportCursor>,
    pub snapshot_upper_bound: ResponseExportCursor,
    pub full_snapshot_rebase: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportPageRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub action: ResponseExportAction,
    pub partition: ResponseExportPartition,
    pub provider_epoch: Uuid,
    pub snapshot_upper_bound: ResponseExportCursor,
    pub after_cursor: Option<ResponseExportCursor>,
    pub page_size: u16,
}

/// One immutable, self-describing Response field value. The stable field ID
/// binds the value to the FormVersion schema while `value_text` preserves the
/// canonical projection semantics used by Dataset filtering, joins, and
/// aggregation without requiring a read back into Response-owned storage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmittedResponseValue {
    pub field_id: Uuid,
    pub value: Value,
    pub value_text: Option<String>,
}

/// Response-owner classification that becomes the Dataset row's upstream
/// restriction floor. The current Response producer emits `public`, while the
/// complete vocabulary keeps the envelope self-describing when an owner later
/// captures a more restrictive source fact.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmittedResponseRestrictionTier {
    Public,
    Internal,
    Restricted,
    Confidential,
}

impl SubmittedResponseRestrictionTier {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
            Self::Restricted => "restricted",
            Self::Confidential => "confidential",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmittedResponseUpsert {
    pub response_id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub node_id: Uuid,
    pub node_name: String,
    pub submitted_at: String,
    pub created_at: String,
    pub last_modified_at: String,
    pub last_modified_by_user_name: Option<String>,
    pub status: String,
    pub restriction_tier: SubmittedResponseRestrictionTier,
    pub scope_node_ids: Vec<Uuid>,
    pub values: BTreeMap<String, SubmittedResponseValue>,
    pub content_digest: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "change", rename_all = "snake_case", deny_unknown_fields)]
pub enum SubmittedResponseChange {
    Upsert(Box<SubmittedResponseUpsert>),
    Tombstone {
        response_id: Uuid,
        reason: ResponseTombstoneReason,
        content_digest: String,
    },
}

impl SubmittedResponseChange {
    pub fn with_recomputed_content_digest(mut self) -> Result<Self, DigestValidationError> {
        let expected = self.expected_content_digest()?;
        match &mut self {
            Self::Upsert(upsert) => upsert.content_digest = expected,
            Self::Tombstone { content_digest, .. } => *content_digest = expected,
        }
        Ok(self)
    }

    pub fn validate_content_digest(&self) -> Result<(), DigestValidationError> {
        let actual = match self {
            Self::Upsert(upsert) => &upsert.content_digest,
            Self::Tombstone { content_digest, .. } => content_digest,
        };
        if actual == &self.expected_content_digest()? {
            Ok(())
        } else {
            Err(DigestValidationError::ContentDigestMismatch)
        }
    }

    fn expected_content_digest(&self) -> Result<String, DigestValidationError> {
        #[derive(Serialize)]
        #[serde(tag = "change", rename_all = "snake_case")]
        enum DigestInput<'a> {
            Upsert {
                response_id: &'a Uuid,
                form_id: &'a Uuid,
                form_version_id: &'a Uuid,
                node_id: &'a Uuid,
                node_name: &'a str,
                submitted_at: &'a str,
                created_at: &'a str,
                last_modified_at: &'a str,
                last_modified_by_user_name: &'a Option<String>,
                status: &'a str,
                restriction_tier: &'a SubmittedResponseRestrictionTier,
                scope_node_ids: &'a [Uuid],
                values: &'a BTreeMap<String, SubmittedResponseValue>,
            },
            Tombstone {
                response_id: &'a Uuid,
                reason: &'a ResponseTombstoneReason,
            },
        }
        let input = match self {
            Self::Upsert(upsert) => DigestInput::Upsert {
                response_id: &upsert.response_id,
                form_id: &upsert.form_id,
                form_version_id: &upsert.form_version_id,
                node_id: &upsert.node_id,
                node_name: &upsert.node_name,
                submitted_at: &upsert.submitted_at,
                created_at: &upsert.created_at,
                last_modified_at: &upsert.last_modified_at,
                last_modified_by_user_name: &upsert.last_modified_by_user_name,
                status: &upsert.status,
                restriction_tier: &upsert.restriction_tier,
                scope_node_ids: &upsert.scope_node_ids,
                values: &upsert.values,
            },
            Self::Tombstone {
                response_id,
                reason,
                ..
            } => DigestInput::Tombstone {
                response_id,
                reason,
            },
        };
        let bytes =
            serde_jcs::to_vec(&input).map_err(|_| DigestValidationError::Canonicalization)?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseTombstoneReason {
    Deleted,
    Redacted,
    StatusExcluded,
    ScopeExcluded,
}

impl ResponseTombstoneReason {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Deleted => "deleted",
            Self::Redacted => "redacted",
            Self::StatusExcluded => "status_excluded",
            Self::ScopeExcluded => "scope_excluded",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportEntry {
    pub cursor: ResponseExportCursor,
    pub change: SubmittedResponseChange,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseExportPageResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub provider_epoch: Uuid,
    pub snapshot_upper_bound: ResponseExportCursor,
    pub entries: Vec<ResponseExportEntry>,
    pub next_after_cursor: Option<ResponseExportCursor>,
    pub complete: bool,
    pub page_digest: String,
}

impl ResponseExportPageResponse {
    pub fn canonical_page_digest(
        entries: &[ResponseExportEntry],
    ) -> Result<String, DigestValidationError> {
        let bytes =
            serde_jcs::to_vec(entries).map_err(|_| DigestValidationError::Canonicalization)?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }

    pub fn validate(&self) -> Result<(), ResponseExportValidationError> {
        if !self.complete && self.next_after_cursor.is_none() {
            return Err(ResponseExportValidationError::MissingContinuationCursor);
        }
        if self.complete && self.next_after_cursor.is_some() {
            return Err(ResponseExportValidationError::UnexpectedContinuationCursor);
        }
        if self
            .entries
            .windows(2)
            .any(|pair| pair[0].cursor >= pair[1].cursor)
        {
            return Err(ResponseExportValidationError::NonMonotonicPage);
        }
        if self
            .entries
            .iter()
            .any(|entry| entry.change.validate_content_digest().is_err())
        {
            return Err(ResponseExportValidationError::ContentDigestMismatch);
        }
        let expected_page_digest = Self::canonical_page_digest(&self.entries)
            .map_err(|_| ResponseExportValidationError::Canonicalization)?;
        if self.page_digest != expected_page_digest {
            return Err(ResponseExportValidationError::PageDigestMismatch);
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum DigestValidationError {
    #[error("a Response change could not be canonicalized")]
    Canonicalization,
    #[error("Response content digest does not match the canonical change")]
    ContentDigestMismatch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ResponseExportValidationError {
    #[error("request action does not match the endpoint")]
    WrongAction,
    #[error("page size must be within the contract bounds")]
    InvalidPageSize,
    #[error("a full snapshot rebase cannot carry a committed cursor")]
    RebaseWithCommittedCursor,
    #[error("an incomplete page requires a continuation cursor")]
    MissingContinuationCursor,
    #[error("a complete page cannot carry a continuation cursor")]
    UnexpectedContinuationCursor,
    #[error("page entry cursors must be strictly increasing")]
    NonMonotonicPage,
    #[error("a page entry content digest is invalid")]
    ContentDigestMismatch,
    #[error("the page digest is invalid")]
    PageDigestMismatch,
    #[error("the page could not be canonicalized")]
    Canonicalization,
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != RESPONSE_EXPORT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Response export schema version {version} is unsupported; expected {RESPONSE_EXPORT_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

fn deserialize_owner_action_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != RESPONSE_OWNER_ACTION_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Response owner action schema version {version} is unsupported; expected {RESPONSE_OWNER_ACTION_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use tessara_module_contract::ModuleDefinitionId;

    use super::*;

    fn submitted_upsert() -> SubmittedResponseChange {
        SubmittedResponseChange::Upsert(Box::new(SubmittedResponseUpsert {
            response_id: Uuid::from_u128(10),
            form_id: Uuid::from_u128(11),
            form_version_id: Uuid::from_u128(12),
            node_id: Uuid::from_u128(13),
            node_name: "North Division".into(),
            submitted_at: "2026-08-12T00:01:00Z".into(),
            created_at: "2026-08-12T00:00:00Z".into(),
            last_modified_at: "2026-08-12T00:02:00Z".into(),
            last_modified_by_user_name: Some("Riley Reviewer".into()),
            status: "submitted".into(),
            restriction_tier: SubmittedResponseRestrictionTier::Restricted,
            scope_node_ids: vec![Uuid::from_u128(13)],
            values: BTreeMap::from([(
                "answer".into(),
                SubmittedResponseValue {
                    field_id: Uuid::from_u128(14),
                    value: json!("current"),
                    value_text: Some("current".into()),
                },
            )]),
            content_digest: String::new(),
        }))
        .with_recomputed_content_digest()
        .unwrap()
    }

    #[test]
    fn cursor_is_opaque_and_strict() {
        let cursor = ResponseExportCursor::parse("epoch1:0000000000000042").unwrap();
        assert_eq!(
            serde_json::to_value(&cursor).unwrap(),
            json!(cursor.as_str())
        );
        assert!(ResponseExportCursor::parse("").is_err());
        assert!(ResponseExportCursor::parse("contains whitespace").is_err());
    }

    #[test]
    fn partition_authorization_digest_is_order_and_duplicate_independent() {
        let presenter = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: Uuid::from_u128(20),
            module_definition_id: ModuleDefinitionId::new("tessara.datasets").unwrap(),
        };
        let installation_id = Uuid::from_u128(21);
        let original = ResponseExportPartition {
            source_binding_id: Uuid::from_u128(1),
            form_version_ids: vec![Uuid::from_u128(3), Uuid::from_u128(2), Uuid::from_u128(3)],
            authorized_scope_node_ids: vec![
                Uuid::from_u128(5),
                Uuid::from_u128(4),
                Uuid::from_u128(5),
            ],
            authorization_digest: String::new(),
        }
        .with_recomputed_authorization_digest(&presenter, installation_id)
        .unwrap();
        let reordered = ResponseExportPartition {
            source_binding_id: original.source_binding_id,
            form_version_ids: vec![Uuid::from_u128(2), Uuid::from_u128(3)],
            authorized_scope_node_ids: vec![Uuid::from_u128(4), Uuid::from_u128(5)],
            authorization_digest: original.authorization_digest.clone(),
        };
        assert_eq!(
            reordered
                .validate_authorization_digest(&presenter, installation_id)
                .unwrap(),
            original.authorization_digest
        );
    }

    #[test]
    fn partition_authorization_digest_rejects_every_bound_identity_tamper() {
        let presenter = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: Uuid::from_u128(20),
            module_definition_id: ModuleDefinitionId::new("tessara.datasets").unwrap(),
        };
        let installation_id = Uuid::from_u128(21);
        let partition = ResponseExportPartition {
            source_binding_id: Uuid::from_u128(1),
            form_version_ids: vec![Uuid::from_u128(2)],
            authorized_scope_node_ids: vec![Uuid::from_u128(3)],
            authorization_digest: String::new(),
        }
        .with_recomputed_authorization_digest(&presenter, installation_id)
        .unwrap();

        let mut tampered = partition.clone();
        tampered.source_binding_id = Uuid::from_u128(9);
        assert_eq!(
            tampered.validate_authorization_digest(&presenter, installation_id),
            Err(ResponseExportPartitionDigestError::Mismatch)
        );
        let mut tampered = partition.clone();
        tampered.form_version_ids.push(Uuid::from_u128(9));
        assert_eq!(
            tampered.validate_authorization_digest(&presenter, installation_id),
            Err(ResponseExportPartitionDigestError::Mismatch)
        );
        let mut tampered = partition.clone();
        tampered.authorized_scope_node_ids.push(Uuid::from_u128(9));
        assert_eq!(
            tampered.validate_authorization_digest(&presenter, installation_id),
            Err(ResponseExportPartitionDigestError::Mismatch)
        );
        let other_presenter = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: Uuid::from_u128(22),
            module_definition_id: ModuleDefinitionId::new("tessara.datasets").unwrap(),
        };
        assert_eq!(
            partition.validate_authorization_digest(&other_presenter, installation_id),
            Err(ResponseExportPartitionDigestError::Mismatch)
        );
        assert_eq!(
            partition.validate_authorization_digest(&presenter, Uuid::from_u128(23)),
            Err(ResponseExportPartitionDigestError::Mismatch)
        );
    }

    #[test]
    fn start_requires_bounded_pages_and_clean_rebase() {
        let partition = ResponseExportPartition {
            source_binding_id: Uuid::from_u128(1),
            form_version_ids: vec![Uuid::from_u128(2)],
            authorized_scope_node_ids: vec![Uuid::from_u128(3)],
            authorization_digest: "sha256:scope".into(),
        };
        let mut request = ResponseExportStartRequest {
            schema_version: 1,
            action: ResponseExportAction::Start,
            partition,
            provider_epoch: Uuid::from_u128(4),
            committed_cursor: None,
            authenticated_head: ResponseExportCursor::parse("head:4").unwrap(),
            full_snapshot_rebase: true,
            page_size: 100,
        };
        assert!(request.validate().is_ok());
        request.page_size = 0;
        assert_eq!(
            request.validate(),
            Err(ResponseExportValidationError::InvalidPageSize)
        );
    }

    #[test]
    fn page_requires_strict_order_and_exact_completion_shape() {
        let entries = Vec::new();
        let mut response = ResponseExportPageResponse {
            schema_version: 1,
            provider_epoch: Uuid::from_u128(1),
            snapshot_upper_bound: ResponseExportCursor::parse("head:9").unwrap(),
            page_digest: ResponseExportPageResponse::canonical_page_digest(&entries).unwrap(),
            entries,
            next_after_cursor: None,
            complete: true,
        };
        assert!(response.validate().is_ok());
        response.complete = false;
        assert_eq!(
            response.validate(),
            Err(ResponseExportValidationError::MissingContinuationCursor)
        );
    }

    #[test]
    fn content_and_page_digests_reject_semantic_tampering() {
        let change = SubmittedResponseChange::Tombstone {
            response_id: Uuid::from_u128(2),
            reason: ResponseTombstoneReason::Deleted,
            content_digest: String::new(),
        }
        .with_recomputed_content_digest()
        .unwrap();
        let entries = vec![ResponseExportEntry {
            cursor: ResponseExportCursor::parse("head:2").unwrap(),
            change,
        }];
        let mut response = ResponseExportPageResponse {
            schema_version: 1,
            provider_epoch: Uuid::from_u128(1),
            snapshot_upper_bound: ResponseExportCursor::parse("head:2").unwrap(),
            page_digest: ResponseExportPageResponse::canonical_page_digest(&entries).unwrap(),
            entries,
            next_after_cursor: None,
            complete: true,
        };
        assert!(response.validate().is_ok());
        response.page_digest = "sha256:tampered".into();
        assert_eq!(
            response.validate(),
            Err(ResponseExportValidationError::PageDigestMismatch)
        );
        response.page_digest =
            ResponseExportPageResponse::canonical_page_digest(&response.entries).unwrap();
        let SubmittedResponseChange::Tombstone { reason, .. } = &mut response.entries[0].change
        else {
            unreachable!()
        };
        *reason = ResponseTombstoneReason::Redacted;
        response.page_digest =
            ResponseExportPageResponse::canonical_page_digest(&response.entries).unwrap();
        assert_eq!(
            response.validate(),
            Err(ResponseExportValidationError::ContentDigestMismatch)
        );
    }

    #[test]
    fn upsert_digest_covers_restriction_and_canonical_value_projection() {
        let mut change = submitted_upsert();
        assert!(change.validate_content_digest().is_ok());
        let SubmittedResponseChange::Upsert(upsert) = &mut change else {
            unreachable!()
        };
        upsert.restriction_tier = SubmittedResponseRestrictionTier::Confidential;
        assert_eq!(
            change.validate_content_digest(),
            Err(DigestValidationError::ContentDigestMismatch)
        );

        let mut change = submitted_upsert();
        let SubmittedResponseChange::Upsert(upsert) = &mut change else {
            unreachable!()
        };
        upsert.values.get_mut("answer").unwrap().value_text = Some("tampered".into());
        assert_eq!(
            change.validate_content_digest(),
            Err(DigestValidationError::ContentDigestMismatch)
        );
    }

    #[test]
    fn upsert_wire_requires_explicit_restriction_tier() {
        let mut wire = serde_json::to_value(submitted_upsert()).unwrap();
        wire.as_object_mut().unwrap().remove("restriction_tier");
        assert!(serde_json::from_value::<SubmittedResponseChange>(wire).is_err());
    }

    #[test]
    fn old_schema_and_unknown_fields_fail_closed() {
        let wire = json!({
            "schema_version": 0,
            "action": "checkpoint",
            "partition": {
                "source_binding_id": Uuid::from_u128(1),
                "form_version_ids": [],
                "authorized_scope_node_ids": [],
                "authorization_digest": "sha256:scope"
            },
            "committed_cursor": null
        });
        assert!(serde_json::from_value::<ResponseExportCheckpointRequest>(wire).is_err());
    }

    #[test]
    fn owner_actions_are_strict_complete_and_logically_named() {
        let request: ResponseOwnerActionRequest = serde_json::from_value(json!({
            "schema_version": 1,
            "action": "correct",
            "logical_key": "response.corrected",
            "response_id": Uuid::from_u128(1),
            "values": {"answer": "corrected", "optional": null}
        }))
        .unwrap();
        assert_eq!(request.action(), ResponseOwnerAction::Correct);
        assert_eq!(request.logical_key(), "response.corrected");
        assert_eq!(request.target_response_id(), Some(Uuid::from_u128(1)));
        assert!(request.validate_logical_key().is_ok());

        let mut unknown = serde_json::to_value(&request).unwrap();
        unknown["unexpected"] = json!(true);
        assert!(serde_json::from_value::<ResponseOwnerActionRequest>(unknown).is_err());

        let mut old = serde_json::to_value(&request).unwrap();
        old["schema_version"] = json!(0);
        assert!(serde_json::from_value::<ResponseOwnerActionRequest>(old).is_err());

        let mut invalid_key = request;
        let ResponseOwnerActionRequest::Correct { logical_key, .. } = &mut invalid_key else {
            unreachable!()
        };
        *logical_key = "Response has spaces".into();
        assert_eq!(
            invalid_key.validate_logical_key(),
            Err(ResponseOwnerActionValidationError::InvalidLogicalKey)
        );
    }
}
