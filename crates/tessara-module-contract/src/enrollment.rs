use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    CONTRACT_SCHEMA_VERSION_V1, ProtocolEnvelopeError, PurposeBoundVerifyingKeyV1,
    SignedEnvelopeV1, SignedWindowError, protocol::validate_window,
};

pub const CORE_ELIGIBILITY_MAX_LIFETIME_SECONDS: i64 = 30;
pub const RECOVERY_OPERATOR_MAX_LIFETIME_SECONDS: i64 = 300;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdministratorEnrollmentClaimKindV1 {
    Initial,
    Recovery,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AdministratorEnrollmentClaimStateV1 {
    Issued,
    Reserved,
    Consumed,
    Expired,
    Revoked,
    Replaced,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LocalOperatorAuthorizationV1 {
    pub schema_version: u16,
    pub operator_identity: String,
    pub reason: String,
    pub installation_id: Uuid,
    pub nonce: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdministratorEligibilityDecisionV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub viable_administrator_exists: bool,
    pub has_ever_had_viable_administrator: bool,
    pub recovery_authorization: Option<SignedEnvelopeV1<LocalOperatorAuthorizationV1>>,
    pub nonce: Uuid,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

impl AdministratorEligibilityDecisionV1 {
    pub fn validate_for(
        &self,
        installation_id: Uuid,
        kind: AdministratorEnrollmentClaimKindV1,
        now: DateTime<Utc>,
        recovery_verifier: Option<&PurposeBoundVerifyingKeyV1>,
    ) -> Result<(), AdministratorEligibilityError> {
        if self.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
            return Err(AdministratorEligibilityError::UnsupportedSchemaVersion);
        }
        if self.installation_id != installation_id {
            return Err(AdministratorEligibilityError::WrongInstallation);
        }
        if self.viable_administrator_exists {
            return Err(AdministratorEligibilityError::ViableAdministratorExists);
        }
        validate_window(
            self.issued_at,
            self.expires_at,
            now,
            CORE_ELIGIBILITY_MAX_LIFETIME_SECONDS,
        )
        .map_err(AdministratorEligibilityError::Window)?;
        match kind {
            AdministratorEnrollmentClaimKindV1::Initial => {
                if self.has_ever_had_viable_administrator {
                    return Err(AdministratorEligibilityError::InitialEnrollmentClosed);
                }
                if self.recovery_authorization.is_some() {
                    return Err(AdministratorEligibilityError::UnexpectedRecoveryAuthorization);
                }
            }
            AdministratorEnrollmentClaimKindV1::Recovery => {
                let authorization = self
                    .recovery_authorization
                    .as_ref()
                    .ok_or(AdministratorEligibilityError::MissingRecoveryAuthorization)?;
                let verifier =
                    recovery_verifier.ok_or(AdministratorEligibilityError::MissingRecoveryTrust)?;
                verifier
                    .verify(authorization)
                    .map_err(AdministratorEligibilityError::RecoverySignature)?;
                let payload = &authorization.payload;
                if payload.schema_version != CONTRACT_SCHEMA_VERSION_V1 {
                    return Err(AdministratorEligibilityError::UnsupportedSchemaVersion);
                }
                if payload.installation_id != installation_id {
                    return Err(AdministratorEligibilityError::WrongInstallation);
                }
                if payload.operator_identity.trim().is_empty() || payload.reason.trim().is_empty() {
                    return Err(AdministratorEligibilityError::IncompleteRecoveryAuthorization);
                }
                validate_window(
                    payload.issued_at,
                    payload.expires_at,
                    now,
                    RECOVERY_OPERATOR_MAX_LIFETIME_SECONDS,
                )
                .map_err(AdministratorEligibilityError::RecoveryWindow)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum AdministratorEligibilityError {
    #[error("administrator eligibility schema version is unsupported")]
    UnsupportedSchemaVersion,
    #[error("administrator eligibility belongs to another installation")]
    WrongInstallation,
    #[error("a viable administrator already exists")]
    ViableAdministratorExists,
    #[error("initial administrator enrollment has permanently closed")]
    InitialEnrollmentClosed,
    #[error("initial eligibility unexpectedly contains recovery authorization")]
    UnexpectedRecoveryAuthorization,
    #[error("recovery eligibility requires local operator authorization")]
    MissingRecoveryAuthorization,
    #[error("recovery eligibility verifier is unavailable")]
    MissingRecoveryTrust,
    #[error("recovery operator authorization is incomplete")]
    IncompleteRecoveryAuthorization,
    #[error("recovery operator signature failed: {0}")]
    RecoverySignature(ProtocolEnvelopeError),
    #[error("administrator eligibility window failed: {0}")]
    Window(SignedWindowError),
    #[error("recovery operator authorization window failed: {0}")]
    RecoveryWindow(SignedWindowError),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentReservationV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub claim_id: Uuid,
    pub generation: u32,
    pub reservation_id: Uuid,
    pub reserved_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentRedemptionResultV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub claim_id: Uuid,
    pub generation: u32,
    pub reservation_id: Uuid,
    pub account_id: Uuid,
    pub role_id: Uuid,
    pub completed_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;
    use crate::{ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1};

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-23T17:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    #[test]
    fn initial_and_recovery_eligibility_are_distinct() {
        let installation_id = Uuid::from_u128(1);
        let decision = AdministratorEligibilityDecisionV1 {
            schema_version: 1,
            installation_id,
            viable_administrator_exists: false,
            has_ever_had_viable_administrator: false,
            recovery_authorization: None,
            nonce: Uuid::from_u128(2),
            issued_at: now(),
            expires_at: now() + Duration::seconds(30),
        };
        decision
            .validate_for(
                installation_id,
                AdministratorEnrollmentClaimKindV1::Initial,
                now() + Duration::seconds(1),
                None,
            )
            .unwrap();
        assert_eq!(
            decision.validate_for(
                installation_id,
                AdministratorEnrollmentClaimKindV1::Recovery,
                now() + Duration::seconds(1),
                None,
            ),
            Err(AdministratorEligibilityError::MissingRecoveryAuthorization)
        );
    }

    #[test]
    fn recovery_requires_a_valid_reasoned_operator_authorization() {
        let installation_id = Uuid::from_u128(1);
        let operator_signer = PurposeBoundSigningKeyV1::from_secret_bytes(
            "tessara.installation-control",
            "recovery-operator-dev-1",
            ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization,
            [10; 32],
        )
        .unwrap();
        let authorization = operator_signer
            .sign(LocalOperatorAuthorizationV1 {
                schema_version: 1,
                operator_identity: "local:operator".into(),
                reason: "Restore administrative access".into(),
                installation_id,
                nonce: Uuid::from_u128(3),
                issued_at: now(),
                expires_at: now() + Duration::seconds(300),
            })
            .unwrap();
        let decision = AdministratorEligibilityDecisionV1 {
            schema_version: 1,
            installation_id,
            viable_administrator_exists: false,
            has_ever_had_viable_administrator: true,
            recovery_authorization: Some(authorization),
            nonce: Uuid::from_u128(4),
            issued_at: now(),
            expires_at: now() + Duration::seconds(30),
        };
        decision
            .validate_for(
                installation_id,
                AdministratorEnrollmentClaimKindV1::Recovery,
                now() + Duration::seconds(1),
                Some(&operator_signer.verifier()),
            )
            .unwrap();
        assert_eq!(
            decision.validate_for(
                installation_id,
                AdministratorEnrollmentClaimKindV1::Initial,
                now() + Duration::seconds(1),
                None,
            ),
            Err(AdministratorEligibilityError::InitialEnrollmentClosed)
        );
    }
}
