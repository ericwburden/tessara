//! Installation-local Supervisor ledger and apply state machine.

use std::{
    collections::BTreeMap,
    path::Path,
    sync::{Arc, Mutex},
};

use chrono::{DateTime, Utc};
use rusqlite::{Connection, OptionalExtension, params};
use semver::Version;
use serde::{Deserialize, Serialize};
use tessara_composition::{
    ApplyAuthorizationV1, ApplyOperationKindV1, BootstrapReceiptV1, CompositionFindingV1,
    CompositionOperationStateV1, CompositionOperationV1, InstallationReceiptV1,
    MaterializationActionV1, MaterializationPlanV1, OPERATION_API_V1, RECEIPT_API_V1,
    canonical_digest,
};
use tessara_module_contract::{
    ArtifactDigest, ProtocolSignaturePurposeV1, PurposeBoundVerifyingKeyV1, SignedEnvelopeV1,
};
use uuid::Uuid;

pub const SUPERVISOR_VERSION_V1: &str = "1.0.0";
pub const DEPLOYMENT_ADAPTER_VERSION_V1: &str = "1.0.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EmergencyOverrideV1 {
    pub override_id: Uuid,
    pub definition_id: String,
    pub reason: String,
    pub actor: serde_json::Value,
    pub issued_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub authorization_digest: ArtifactDigest,
    pub reconciled_at: Option<DateTime<Utc>>,
    pub expired: bool,
}

#[derive(Clone)]
pub struct SupervisorLedger {
    connection: Arc<Mutex<Connection>>,
}

impl SupervisorLedger {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, SupervisorError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "foreign_keys", "ON")?;
        connection.execute_batch(include_str!("../migrations/001_baseline.sql"))?;
        Ok(Self {
            connection: Arc::new(Mutex::new(connection)),
        })
    }

    pub fn initialize_installation(
        &self,
        installation_id: Uuid,
        created_at: DateTime<Utc>,
    ) -> Result<(), SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let existing: Option<String> = connection
            .query_row(
                "SELECT installation_id FROM installation_root WHERE singleton=1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        match existing {
            Some(existing) if existing != installation_id.to_string() => {
                Err(SupervisorError::InstallationMismatch)
            }
            Some(_) => Ok(()),
            None => {
                connection.execute("INSERT INTO installation_root(singleton,installation_id,created_at) VALUES(1,?1,?2)", params![installation_id.to_string(), created_at.to_rfc3339()])?;
                Ok(())
            }
        }
    }

    pub fn register_trust_anchor(
        &self,
        issuer: &str,
        key_id: &str,
        purpose: &str,
        public_key: &[u8],
        now: DateTime<Utc>,
    ) -> Result<(), SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        connection.execute(
            "INSERT INTO trust_anchors(issuer,key_id,purpose,public_key,created_at) VALUES(?1,?2,?3,?4,?5)
             ON CONFLICT(issuer,key_id,purpose) DO UPDATE SET public_key=excluded.public_key",
            params![issuer, key_id, purpose, public_key, now.to_rfc3339()],
        )?;
        Ok(())
    }

    pub fn verifier_for(
        &self,
        issuer: &str,
        key_id: &str,
        purpose: ProtocolSignaturePurposeV1,
    ) -> Result<PurposeBoundVerifyingKeyV1, SupervisorError> {
        let purpose_name = signature_purpose_name(purpose);
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let public_key: Vec<u8> = connection
            .query_row(
                "SELECT public_key FROM trust_anchors WHERE issuer=?1 AND key_id=?2 AND purpose=?3",
                params![issuer, key_id, purpose_name],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(SupervisorError::TrustAnchorMissing)?;
        let public_key: [u8; 32] = public_key
            .try_into()
            .map_err(|_| SupervisorError::CorruptLedger)?;
        PurposeBoundVerifyingKeyV1::from_public_bytes(issuer, key_id, purpose, public_key)
            .map_err(SupervisorError::Signature)
    }

    pub fn accept_apply(
        &self,
        plan: &MaterializationPlanV1,
        signed_authorization: &SignedEnvelopeV1<ApplyAuthorizationV1>,
        verifier: &PurposeBoundVerifyingKeyV1,
        now: DateTime<Utc>,
    ) -> Result<CompositionOperationV1, SupervisorError> {
        verifier
            .verify(signed_authorization)
            .map_err(SupervisorError::Signature)?;
        let plan_digest = canonical_digest(plan)?;
        let authorization_digest = canonical_digest(signed_authorization)?;
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let transaction = connection.transaction()?;
        let installation: String = transaction
            .query_row(
                "SELECT installation_id FROM installation_root WHERE singleton=1",
                [],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(SupervisorError::NotInitialized)?;
        if installation != plan.installation_id.to_string() {
            return Err(SupervisorError::InstallationMismatch);
        }
        let current_receipt: Option<String> = transaction
            .query_row(
                "SELECT receipt_digest FROM receipts ORDER BY revision DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        let current_receipt = current_receipt.map(parse_digest).transpose()?;
        signed_authorization
            .payload
            .validate_for(plan, &plan_digest, current_receipt.as_ref(), now)
            .map_err(SupervisorError::Authorization)?;

        let existing: Option<(String, String, String)> = transaction.query_row(
            "SELECT operation_id,authorization_digest,state FROM operations WHERE idempotency_key=?1",
            [&signed_authorization.payload.idempotency_key],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).optional()?;
        if let Some((operation_id, stored_digest, _)) = existing {
            if stored_digest != authorization_digest.to_string() {
                return Err(SupervisorError::IdempotencyConflict);
            }
            transaction.commit()?;
            drop(connection);
            return self
                .operation(
                    Uuid::parse_str(&operation_id).map_err(|_| SupervisorError::CorruptLedger)?,
                )
                .and_then(|operation| operation.ok_or(SupervisorError::CorruptLedger));
        }
        let replay: bool = transaction.query_row(
            "SELECT EXISTS(SELECT 1 FROM accepted_nonces WHERE nonce=?1)",
            [signed_authorization.payload.nonce.to_string()],
            |row| row.get(0),
        )?;
        if replay {
            return Err(SupervisorError::Replay);
        }
        let latest_sequence: Option<u64> =
            transaction.query_row("SELECT MAX(apply_sequence) FROM operations", [], |row| {
                row.get(0)
            })?;
        if latest_sequence
            .is_some_and(|latest| signed_authorization.payload.apply_sequence <= latest)
        {
            return Err(SupervisorError::NonMonotonicApplySequence);
        }
        let running: bool = transaction.query_row("SELECT EXISTS(SELECT 1 FROM operations WHERE state NOT IN ('succeeded','failed','rolled_back'))", [], |row| row.get(0))?;
        if running {
            return Err(SupervisorError::ConcurrentApply);
        }

        let operation_id = Uuid::new_v4();
        transaction.execute(
            "INSERT INTO accepted_nonces(nonce,accepted_at) VALUES(?1,?2)",
            params![
                signed_authorization.payload.nonce.to_string(),
                now.to_rfc3339()
            ],
        )?;
        transaction.execute(
            "INSERT INTO operations(operation_id,installation_id,idempotency_key,apply_sequence,plan_digest,authorization_digest,state,accepted_at,updated_at,plan_json,authorization_json)
             VALUES(?1,?2,?3,?4,?5,?6,'accepted',?7,?7,?8,?9)",
            params![operation_id.to_string(), installation, signed_authorization.payload.idempotency_key, signed_authorization.payload.apply_sequence, plan_digest.to_string(), authorization_digest.to_string(), now.to_rfc3339(), serde_json::to_string(plan)?, serde_json::to_string(signed_authorization)?],
        )?;
        transaction.commit()?;
        Ok(CompositionOperationV1 {
            api_version: OPERATION_API_V1.into(),
            operation_id,
            installation_id: plan.installation_id,
            idempotency_key: signed_authorization.payload.idempotency_key.clone(),
            plan_digest,
            authorization_digest,
            state: CompositionOperationStateV1::Accepted,
            accepted_at: now,
            updated_at: now,
            finding: None,
            receipt_digest: None,
        })
    }

    pub fn execute<A: MaterializationAdapter>(
        &self,
        operation_id: Uuid,
        lockfile_digest: ArtifactDigest,
        adapter: &mut A,
        now: DateTime<Utc>,
    ) -> Result<InstallationReceiptV1, SupervisorError> {
        let operation = self
            .operation(operation_id)?
            .ok_or(SupervisorError::OperationMissing)?;
        let plan = self.plan(operation_id)?;
        let mut bootstrap_receipts = Vec::new();
        for action in &plan.actions {
            let state = state_for_action(action);
            self.transition(operation_id, state, Utc::now(), None)?;
            if let Some(receipt) = adapter.execute(action)? {
                bootstrap_receipts.push(receipt);
            }
        }
        let previous = self.current_receipt()?;
        let authorization = self.authorization(operation_id)?;
        let emergency = authorization.payload.operation == ApplyOperationKindV1::EmergencyDisable;
        let no_op = previous
            .as_ref()
            .is_some_and(|receipt| receipt.plan_digest == operation.plan_digest);
        let mut desired_enablement = previous
            .as_ref()
            .map(|receipt| receipt.desired_enablement.clone())
            .unwrap_or_default();
        let action_enablement = plan
            .actions
            .iter()
            .filter_map(|action| match action {
                MaterializationActionV1::SetEnablement {
                    definition_id,
                    enabled,
                } => Some((definition_id.clone(), *enabled)),
                _ => None,
            })
            .collect::<BTreeMap<_, _>>();
        if !emergency {
            desired_enablement.extend(action_enablement.clone());
        }
        let mut observed_enablement = previous
            .as_ref()
            .map(|receipt| receipt.observed_enablement.clone())
            .unwrap_or_default();
        observed_enablement.extend(action_enablement.clone());
        let mut observed_artifacts = previous
            .as_ref()
            .map(|receipt| receipt.observed_artifacts.clone())
            .unwrap_or_default();
        observed_artifacts.extend(adapter.observed_artifacts());
        let mut configuration_digests = previous
            .as_ref()
            .map(|receipt| receipt.configuration_digests.clone())
            .unwrap_or_default();
        configuration_digests.extend(adapter.configuration_digests());
        if emergency {
            bootstrap_receipts = previous
                .as_ref()
                .map(|receipt| receipt.bootstrap_receipts.clone())
                .unwrap_or_default();
        }
        let receipt = InstallationReceiptV1 {
            api_version: RECEIPT_API_V1.into(),
            installation_id: operation.installation_id,
            revision: previous.as_ref().map_or(1, |receipt| receipt.revision + 1),
            lockfile_digest,
            plan_digest: operation.plan_digest.clone(),
            authorization_digest: operation.authorization_digest.clone(),
            composition_engine_version: Version::new(1, 0, 0),
            supervisor_version: Version::parse(SUPERVISOR_VERSION_V1).unwrap(),
            deployment_adapter_version: Version::parse(DEPLOYMENT_ADAPTER_VERSION_V1).unwrap(),
            desired_enablement: desired_enablement.clone(),
            observed_enablement,
            observed_artifacts,
            configuration_digests,
            bootstrap_receipts,
            applied_at: now,
            previous_receipt_digest: previous.as_ref().map(canonical_digest).transpose()?,
            no_op,
        };
        let receipt_digest = canonical_digest(&receipt)?;
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        connection.execute("INSERT INTO receipts(revision,receipt_digest,receipt_json,applied_at) VALUES(?1,?2,?3,?4)", params![receipt.revision, receipt_digest.to_string(), serde_json::to_string(&receipt)?, now.to_rfc3339()])?;
        connection.execute("UPDATE operations SET state='succeeded',updated_at=?2,receipt_digest=?3 WHERE operation_id=?1", params![operation_id.to_string(), now.to_rfc3339(), receipt_digest.to_string()])?;
        if !emergency {
            for (definition_id, enabled) in action_enablement {
                if enabled {
                    connection.execute("UPDATE emergency_overrides SET reconciled_at=?2 WHERE definition_id=?1 AND reconciled_at IS NULL", params![definition_id, now.to_rfc3339()])?;
                }
            }
        }
        Ok(receipt)
    }

    pub fn operation(
        &self,
        operation_id: Uuid,
    ) -> Result<Option<CompositionOperationV1>, SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        connection.query_row(
            "SELECT installation_id,idempotency_key,plan_digest,authorization_digest,state,accepted_at,updated_at,finding_json,receipt_digest FROM operations WHERE operation_id=?1",
            [operation_id.to_string()],
            |row| {
                let finding: Option<String> = row.get(7)?;
                Ok(CompositionOperationV1 { api_version: OPERATION_API_V1.into(), operation_id, installation_id: parse_uuid(row.get::<_,String>(0)?)?, idempotency_key: row.get(1)?, plan_digest: parse_digest_sql(row.get(2)?)?, authorization_digest: parse_digest_sql(row.get(3)?)?, state: parse_state(row.get::<_,String>(4)?)?, accepted_at: parse_time(row.get::<_,String>(5)?)?, updated_at: parse_time(row.get::<_,String>(6)?)?, finding: finding.map(|value| serde_json::from_str(&value).map_err(to_sql_error)).transpose()?, receipt_digest: row.get::<_,Option<String>>(8)?.map(parse_digest_sql).transpose()? })
            },
        ).optional().map_err(SupervisorError::Sqlite)
    }

    pub fn current_receipt(&self) -> Result<Option<InstallationReceiptV1>, SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let json: Option<String> = connection
            .query_row(
                "SELECT receipt_json FROM receipts ORDER BY revision DESC LIMIT 1",
                [],
                |row| row.get(0),
            )
            .optional()?;
        json.map(|value| serde_json::from_str(&value).map_err(SupervisorError::Json))
            .transpose()
    }

    pub fn record_emergency_override(
        &self,
        override_record: &EmergencyOverrideV1,
    ) -> Result<(), SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        connection.execute(
            "INSERT INTO emergency_overrides(override_id,definition_id,reason,actor_json,issued_at,expires_at,authorization_digest,reconciled_at) VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![override_record.override_id.to_string(), override_record.definition_id,
                override_record.reason, serde_json::to_string(&override_record.actor)?,
                override_record.issued_at.to_rfc3339(), override_record.expires_at.map(|value| value.to_rfc3339()),
                override_record.authorization_digest.to_string(), override_record.reconciled_at.map(|value| value.to_rfc3339())],
        )?;
        Ok(())
    }

    pub fn emergency_overrides(&self) -> Result<Vec<EmergencyOverrideV1>, SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let mut statement = connection.prepare("SELECT override_id,definition_id,reason,actor_json,issued_at,expires_at,authorization_digest,reconciled_at FROM emergency_overrides ORDER BY issued_at DESC")?;
        let rows = statement.query_map([], |row| {
            Ok(EmergencyOverrideV1 {
                override_id: parse_uuid(row.get::<_, String>(0)?)?,
                definition_id: row.get(1)?,
                reason: row.get(2)?,
                actor: serde_json::from_str(&row.get::<_, String>(3)?).map_err(to_sql_error)?,
                issued_at: parse_time(row.get::<_, String>(4)?)?,
                expires_at: row
                    .get::<_, Option<String>>(5)?
                    .map(parse_time)
                    .transpose()?,
                authorization_digest: parse_digest_sql(row.get(6)?)?,
                reconciled_at: row
                    .get::<_, Option<String>>(7)?
                    .map(parse_time)
                    .transpose()?,
                expired: row
                    .get::<_, Option<String>>(5)?
                    .map(parse_time)
                    .transpose()?
                    .is_some_and(|expires_at| expires_at <= Utc::now()),
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(SupervisorError::Sqlite)
    }

    fn plan(&self, operation_id: Uuid) -> Result<MaterializationPlanV1, SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let json: String = connection.query_row(
            "SELECT plan_json FROM operations WHERE operation_id=?1",
            [operation_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(serde_json::from_str(&json)?)
    }

    fn authorization(
        &self,
        operation_id: Uuid,
    ) -> Result<SignedEnvelopeV1<ApplyAuthorizationV1>, SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        let json: String = connection.query_row(
            "SELECT authorization_json FROM operations WHERE operation_id=?1",
            [operation_id.to_string()],
            |row| row.get(0),
        )?;
        Ok(serde_json::from_str(&json)?)
    }

    fn transition(
        &self,
        operation_id: Uuid,
        state: CompositionOperationStateV1,
        now: DateTime<Utc>,
        finding: Option<&CompositionFindingV1>,
    ) -> Result<(), SupervisorError> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| SupervisorError::LedgerPoisoned)?;
        connection.execute(
            "UPDATE operations SET state=?2,updated_at=?3,finding_json=?4 WHERE operation_id=?1",
            params![
                operation_id.to_string(),
                state.as_str(),
                now.to_rfc3339(),
                finding.map(serde_json::to_string).transpose()?
            ],
        )?;
        Ok(())
    }
}

pub fn signature_purpose_name(purpose: ProtocolSignaturePurposeV1) -> &'static str {
    match purpose {
        ProtocolSignaturePurposeV1::ReleaseCatalog => "release_catalog",
        ProtocolSignaturePurposeV1::ResolvedComposition => "resolved_composition",
        ProtocolSignaturePurposeV1::ApplyAuthorization => "apply_authorization",
        ProtocolSignaturePurposeV1::SupervisorRequest => "supervisor_request",
        ProtocolSignaturePurposeV1::SupervisorResponse => "supervisor_response",
        ProtocolSignaturePurposeV1::InstallationReceipt => "installation_receipt",
        ProtocolSignaturePurposeV1::EnrollmentEligibility => "enrollment_eligibility",
        ProtocolSignaturePurposeV1::EnrollmentRedemption => "enrollment_redemption",
        ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization => {
            "recovery_operator_authorization"
        }
        ProtocolSignaturePurposeV1::FixtureExternalIdentity => "fixture_external_identity",
        ProtocolSignaturePurposeV1::ShellContext => "shell_context",
        ProtocolSignaturePurposeV1::AuthorizationGrant => "authorization_grant",
    }
}

pub trait MaterializationAdapter {
    fn execute(
        &mut self,
        action: &MaterializationActionV1,
    ) -> Result<Option<BootstrapReceiptV1>, SupervisorError>;
    fn observed_artifacts(&self) -> BTreeMap<String, ArtifactDigest>;
    fn configuration_digests(&self) -> BTreeMap<String, ArtifactDigest>;
}

#[derive(Default)]
pub struct RecordingAdapter {
    pub actions: Vec<MaterializationActionV1>,
    artifacts: BTreeMap<String, ArtifactDigest>,
    configurations: BTreeMap<String, ArtifactDigest>,
}

impl MaterializationAdapter for RecordingAdapter {
    fn execute(
        &mut self,
        action: &MaterializationActionV1,
    ) -> Result<Option<BootstrapReceiptV1>, SupervisorError> {
        self.actions.push(action.clone());
        match action {
            MaterializationActionV1::AcquireImage { component, digest } => {
                self.artifacts.insert(component.clone(), digest.clone());
            }
            MaterializationActionV1::Configure { owner, digest } => {
                self.configurations.insert(owner.clone(), digest.clone());
            }
            _ => {}
        }
        Ok(None)
    }
    fn observed_artifacts(&self) -> BTreeMap<String, ArtifactDigest> {
        self.artifacts.clone()
    }
    fn configuration_digests(&self) -> BTreeMap<String, ArtifactDigest> {
        self.configurations.clone()
    }
}

fn state_for_action(action: &MaterializationActionV1) -> CompositionOperationStateV1 {
    match action {
        MaterializationActionV1::AcquireImage { .. } => CompositionOperationStateV1::Acquiring,
        MaterializationActionV1::ProvisionDatabase { .. } => {
            CompositionOperationStateV1::Provisioning
        }
        MaterializationActionV1::Migrate { .. } => CompositionOperationStateV1::Migrating,
        MaterializationActionV1::Configure { .. }
        | MaterializationActionV1::SetEnablement { .. } => CompositionOperationStateV1::Configuring,
        MaterializationActionV1::Bootstrap { .. } => CompositionOperationStateV1::Bootstrapping,
        MaterializationActionV1::HealthGate { .. } => CompositionOperationStateV1::HealthChecking,
        MaterializationActionV1::SwitchTraffic { .. } => CompositionOperationStateV1::Switching,
        MaterializationActionV1::VerifyReadBack => CompositionOperationStateV1::Verifying,
    }
}

fn parse_digest(value: String) -> Result<ArtifactDigest, SupervisorError> {
    ArtifactDigest::new(value).map_err(|_| SupervisorError::CorruptLedger)
}
fn parse_digest_sql(value: String) -> rusqlite::Result<ArtifactDigest> {
    ArtifactDigest::new(value).map_err(|_| to_sql_error(std::io::Error::other("invalid digest")))
}
fn parse_uuid(value: String) -> rusqlite::Result<Uuid> {
    Uuid::parse_str(&value).map_err(to_sql_error)
}
fn parse_time(value: String) -> rusqlite::Result<DateTime<Utc>> {
    value.parse().map_err(to_sql_error)
}
fn parse_state(value: String) -> rusqlite::Result<CompositionOperationStateV1> {
    match value.as_str() {
        "accepted" => Ok(CompositionOperationStateV1::Accepted),
        "acquiring" => Ok(CompositionOperationStateV1::Acquiring),
        "provisioning" => Ok(CompositionOperationStateV1::Provisioning),
        "migrating" => Ok(CompositionOperationStateV1::Migrating),
        "configuring" => Ok(CompositionOperationStateV1::Configuring),
        "bootstrapping" => Ok(CompositionOperationStateV1::Bootstrapping),
        "health_checking" => Ok(CompositionOperationStateV1::HealthChecking),
        "switching" => Ok(CompositionOperationStateV1::Switching),
        "verifying" => Ok(CompositionOperationStateV1::Verifying),
        "succeeded" => Ok(CompositionOperationStateV1::Succeeded),
        "failed" => Ok(CompositionOperationStateV1::Failed),
        "rolled_back" => Ok(CompositionOperationStateV1::RolledBack),
        _ => Err(to_sql_error(std::io::Error::other("invalid state"))),
    }
}
fn to_sql_error(error: impl std::error::Error + Send + Sync + 'static) -> rusqlite::Error {
    rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(error))
}

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("Supervisor SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("Supervisor JSON operation failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("signed Supervisor request failed verification: {0}")]
    Signature(tessara_module_contract::ProtocolEnvelopeError),
    #[error("apply authorization failed: {0:?}")]
    Authorization(CompositionFindingV1),
    #[error("Supervisor ledger is not initialized")]
    NotInitialized,
    #[error("required Supervisor trust anchor is not registered")]
    TrustAnchorMissing,
    #[error("request belongs to another installation")]
    InstallationMismatch,
    #[error("authorization nonce was already accepted")]
    Replay,
    #[error("another apply is already active")]
    ConcurrentApply,
    #[error("idempotency key was reused with different input")]
    IdempotencyConflict,
    #[error("apply sequence must be greater than every previously accepted sequence")]
    NonMonotonicApplySequence,
    #[error("composition operation was not found")]
    OperationMissing,
    #[error("Supervisor ledger contains corrupt state")]
    CorruptLedger,
    #[error("Supervisor ledger lock was poisoned")]
    LedgerPoisoned,
    #[error("materialization failed: {0}")]
    Materialization(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use tessara_composition::{
        AUTHORIZATION_API_V1, ActorEvidenceV1, ApplyOperationKindV1, PLAN_API_V1, required_effects,
    };
    use tessara_module_contract::{ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1};

    fn digest(byte: char) -> ArtifactDigest {
        ArtifactDigest::new(format!("sha256:{}", byte.to_string().repeat(64))).unwrap()
    }
    fn plan(installation_id: Uuid) -> MaterializationPlanV1 {
        MaterializationPlanV1 {
            api_version: PLAN_API_V1.into(),
            installation_id,
            desired_revision: 1,
            actions: vec![MaterializationActionV1::VerifyReadBack],
        }
    }
    fn authorization(
        installation_id: Uuid,
        plan: &MaterializationPlanV1,
        now: DateTime<Utc>,
        key: &str,
    ) -> ApplyAuthorizationV1 {
        ApplyAuthorizationV1 {
            api_version: AUTHORIZATION_API_V1.into(),
            operation: ApplyOperationKindV1::Materialize,
            installation_id,
            base_receipt_digest: None,
            target_plan_digest: canonical_digest(plan).unwrap(),
            desired_revision: 1,
            apply_sequence: 1,
            nonce: Uuid::new_v4(),
            idempotency_key: key.into(),
            initiator: ActorEvidenceV1 {
                actor_id: "planner".into(),
                actor_kind: "account".into(),
                authority: "composition:plan".into(),
            },
            approver: ActorEvidenceV1 {
                actor_id: "admin".into(),
                actor_kind: "account".into(),
                authority: "composition:approve".into(),
            },
            issued_at: now,
            expires_at: now + Duration::minutes(5),
            approved_effects: required_effects(plan),
            reason: None,
        }
    }
    fn signer() -> PurposeBoundSigningKeyV1 {
        PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.core",
            "apply-1",
            ProtocolSignaturePurposeV1::ApplyAuthorization,
            [9; 32],
        )
        .unwrap()
    }

    #[test]
    fn ledger_rejects_replay_and_returns_same_idempotent_operation() {
        let ledger = SupervisorLedger::open(":memory:").unwrap();
        let installation = Uuid::new_v4();
        let now = Utc::now();
        ledger.initialize_installation(installation, now).unwrap();
        let plan = plan(installation);
        let signer = signer();
        let auth = signer
            .sign(authorization(installation, &plan, now, "apply-1"))
            .unwrap();
        let first = ledger
            .accept_apply(&plan, &auth, &signer.verifier(), now)
            .unwrap();
        let second = ledger
            .accept_apply(&plan, &auth, &signer.verifier(), now)
            .unwrap();
        assert_eq!(first.operation_id, second.operation_id);
        let mut replay_payload = authorization(installation, &plan, now, "apply-2");
        replay_payload.nonce = auth.payload.nonce;
        let replay = signer.sign(replay_payload).unwrap();
        assert!(matches!(
            ledger.accept_apply(&plan, &replay, &signer.verifier(), now),
            Err(SupervisorError::Replay)
        ));
    }

    #[test]
    fn execution_checkpoints_and_emits_source_exact_receipt() {
        let ledger = SupervisorLedger::open(":memory:").unwrap();
        let installation = Uuid::new_v4();
        let now = Utc::now();
        ledger.initialize_installation(installation, now).unwrap();
        let mut plan = plan(installation);
        plan.actions.insert(
            0,
            MaterializationActionV1::AcquireImage {
                component: "core".into(),
                digest: digest('a'),
            },
        );
        let signer = signer();
        let auth = signer
            .sign(authorization(installation, &plan, now, "apply-1"))
            .unwrap();
        let operation = ledger
            .accept_apply(&plan, &auth, &signer.verifier(), now)
            .unwrap();
        let mut adapter = RecordingAdapter::default();
        let receipt = ledger
            .execute(operation.operation_id, digest('f'), &mut adapter, now)
            .unwrap();
        assert_eq!(receipt.observed_artifacts["core"], digest('a'));
        assert_eq!(
            ledger
                .operation(operation.operation_id)
                .unwrap()
                .unwrap()
                .state,
            CompositionOperationStateV1::Succeeded
        );
    }

    #[test]
    fn emergency_disable_preserves_desired_enablement_as_visible_drift() {
        let ledger = SupervisorLedger::open(":memory:").unwrap();
        let installation = Uuid::new_v4();
        let now = Utc::now();
        ledger.initialize_installation(installation, now).unwrap();
        let signer = signer();
        let mut initial = plan(installation);
        initial.actions.insert(
            0,
            MaterializationActionV1::SetEnablement {
                definition_id: "example.module".into(),
                enabled: true,
            },
        );
        let signed_initial = signer
            .sign(authorization(installation, &initial, now, "initial"))
            .unwrap();
        let initial_operation = ledger
            .accept_apply(&initial, &signed_initial, &signer.verifier(), now)
            .unwrap();
        let initial_receipt = ledger
            .execute(
                initial_operation.operation_id,
                digest('a'),
                &mut RecordingAdapter::default(),
                now,
            )
            .unwrap();

        let emergency_plan = MaterializationPlanV1 {
            api_version: PLAN_API_V1.into(),
            installation_id: installation,
            desired_revision: 1,
            actions: vec![
                MaterializationActionV1::SetEnablement {
                    definition_id: "example.module".into(),
                    enabled: false,
                },
                MaterializationActionV1::VerifyReadBack,
            ],
        };
        let mut emergency = authorization(installation, &emergency_plan, now, "emergency");
        emergency.operation = ApplyOperationKindV1::EmergencyDisable;
        emergency.base_receipt_digest = Some(canonical_digest(&initial_receipt).unwrap());
        emergency.apply_sequence = 2;
        emergency.approved_effects =
            std::collections::BTreeSet::from([tessara_composition::ApprovedEffectV1::Disable]);
        emergency.reason = Some("Contain unsafe behavior".into());
        let signed_emergency = signer.sign(emergency).unwrap();
        let operation = ledger
            .accept_apply(&emergency_plan, &signed_emergency, &signer.verifier(), now)
            .unwrap();
        let receipt = ledger
            .execute(
                operation.operation_id,
                digest('b'),
                &mut RecordingAdapter::default(),
                now,
            )
            .unwrap();
        assert_eq!(receipt.desired_enablement["example.module"], true);
        assert_eq!(receipt.observed_enablement["example.module"], false);
    }
}
