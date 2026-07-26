use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use rand_core::{OsRng, RngCore};
use serde::Serialize;
use tessara_module_contract::{
    AdministratorEnrollmentClaimKindV1, AdministratorEnrollmentClaimStateV1,
    EnrollmentRedemptionResultV1,
};
use uuid::Uuid;

mod store;

pub use store::{InstallationControlError, InstallationControlStore, IssuedClaimOutputV1};

pub const DEFAULT_CLAIM_LIFETIME_SECONDS: i64 = 900;
pub const DEFAULT_RESERVATION_LIFETIME_SECONDS: i64 = 120;

pub struct ClaimSecret(String);

impl ClaimSecret {
    pub fn generate() -> Self {
        let mut bytes = [0_u8; 32];
        OsRng.fill_bytes(&mut bytes);
        Self(URL_SAFE_NO_PAD.encode(bytes))
    }

    pub fn expose_once(self) -> String {
        self.0
    }

    fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for ClaimSecret {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ClaimSecret([REDACTED])")
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EnrollmentClaimStatusV1 {
    pub schema_version: u16,
    pub installation_id: Uuid,
    pub claim_id: Uuid,
    pub generation: u32,
    pub kind: AdministratorEnrollmentClaimKindV1,
    pub state: AdministratorEnrollmentClaimStateV1,
    pub issued_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub reservation_id: Option<Uuid>,
    pub reservation_expires_at: Option<DateTime<Utc>>,
    pub terminal_at: Option<DateTime<Utc>>,
}

pub struct IssuedEnrollmentClaimV1 {
    pub status: EnrollmentClaimStatusV1,
    pub secret: ClaimSecret,
    verifier: String,
}

impl std::fmt::Debug for IssuedEnrollmentClaimV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("IssuedEnrollmentClaimV1")
            .field("status", &self.status)
            .field("secret", &"[REDACTED]")
            .field("verifier", &"[REDACTED]")
            .finish()
    }
}

impl IssuedEnrollmentClaimV1 {
    pub fn issue(
        installation_id: Uuid,
        claim_id: Uuid,
        generation: u32,
        kind: AdministratorEnrollmentClaimKindV1,
        issued_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<Self, ClaimTransitionError> {
        if generation == 0 || expires_at <= issued_at {
            return Err(ClaimTransitionError::InvalidIssue);
        }
        let secret = ClaimSecret::generate();
        let salt = SaltString::generate(&mut OsRng);
        let verifier = Argon2::default()
            .hash_password(secret.as_str().as_bytes(), &salt)
            .map_err(|_| ClaimTransitionError::VerifierFailure)?
            .to_string();
        Ok(Self {
            status: EnrollmentClaimStatusV1 {
                schema_version: 1,
                installation_id,
                claim_id,
                generation,
                kind,
                state: AdministratorEnrollmentClaimStateV1::Issued,
                issued_at,
                expires_at,
                reservation_id: None,
                reservation_expires_at: None,
                terminal_at: None,
            },
            secret,
            verifier,
        })
    }

    pub fn into_persisted(self) -> PersistedEnrollmentClaimV1 {
        PersistedEnrollmentClaimV1 {
            status: self.status,
            verifier: self.verifier,
            redemption_result: None,
        }
    }

    fn verifier(&self) -> &str {
        &self.verifier
    }
}

pub struct PersistedEnrollmentClaimV1 {
    status: EnrollmentClaimStatusV1,
    verifier: String,
    redemption_result: Option<EnrollmentRedemptionResultV1>,
}

impl std::fmt::Debug for PersistedEnrollmentClaimV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("PersistedEnrollmentClaimV1")
            .field("status", &self.status)
            .field("verifier", &"[REDACTED]")
            .field("redemption_result", &self.redemption_result)
            .finish()
    }
}

impl PersistedEnrollmentClaimV1 {
    fn from_parts(
        status: EnrollmentClaimStatusV1,
        verifier: String,
        redemption_result: Option<EnrollmentRedemptionResultV1>,
    ) -> Self {
        Self {
            status,
            verifier,
            redemption_result,
        }
    }

    pub fn status(&self) -> &EnrollmentClaimStatusV1 {
        &self.status
    }

    pub fn reserve(
        &mut self,
        presented_secret: &str,
        reservation_id: Uuid,
        now: DateTime<Utc>,
        reservation_expires_at: DateTime<Utc>,
    ) -> Result<(), ClaimAccessError> {
        self.expire_if_needed(now);
        if self.status.state == AdministratorEnrollmentClaimStateV1::Reserved
            && self.status.reservation_id == Some(reservation_id)
            && self
                .status
                .reservation_expires_at
                .is_some_and(|expiry| expiry > now)
        {
            return Ok(());
        }
        if self.status.state != AdministratorEnrollmentClaimStateV1::Issued
            || reservation_id.is_nil()
            || reservation_expires_at <= now
            || reservation_expires_at > self.status.expires_at
            || !self.secret_matches(presented_secret)
        {
            return Err(ClaimAccessError::EnrollmentUnavailable);
        }
        self.status.state = AdministratorEnrollmentClaimStateV1::Reserved;
        self.status.reservation_id = Some(reservation_id);
        self.status.reservation_expires_at = Some(reservation_expires_at);
        Ok(())
    }

    pub fn consume(
        &mut self,
        result: EnrollmentRedemptionResultV1,
    ) -> Result<(), ClaimAccessError> {
        if self.status.state == AdministratorEnrollmentClaimStateV1::Consumed
            && self.redemption_result.as_ref() == Some(&result)
        {
            return Ok(());
        }
        if self.status.state != AdministratorEnrollmentClaimStateV1::Reserved
            || self.status.installation_id != result.installation_id
            || self.status.claim_id != result.claim_id
            || self.status.generation != result.generation
            || self.status.reservation_id != Some(result.reservation_id)
            || self
                .status
                .reservation_expires_at
                .is_none_or(|expiry| expiry <= result.completed_at)
        {
            return Err(ClaimAccessError::EnrollmentUnavailable);
        }
        self.status.state = AdministratorEnrollmentClaimStateV1::Consumed;
        self.status.terminal_at = Some(result.completed_at);
        self.redemption_result = Some(result);
        Ok(())
    }

    pub fn revoke(&mut self, now: DateTime<Utc>) -> Result<(), ClaimTransitionError> {
        if !matches!(
            self.status.state,
            AdministratorEnrollmentClaimStateV1::Issued
                | AdministratorEnrollmentClaimStateV1::Reserved
        ) {
            return Err(ClaimTransitionError::AlreadyTerminal);
        }
        self.status.state = AdministratorEnrollmentClaimStateV1::Revoked;
        self.status.terminal_at = Some(now);
        Ok(())
    }

    pub fn replace(&mut self, now: DateTime<Utc>) -> Result<(), ClaimTransitionError> {
        if !matches!(
            self.status.state,
            AdministratorEnrollmentClaimStateV1::Issued
                | AdministratorEnrollmentClaimStateV1::Reserved
        ) {
            return Err(ClaimTransitionError::AlreadyTerminal);
        }
        self.status.state = AdministratorEnrollmentClaimStateV1::Replaced;
        self.status.terminal_at = Some(now);
        Ok(())
    }

    fn expire_if_needed(&mut self, now: DateTime<Utc>) {
        if self.status.expires_at <= now
            || (self.status.state == AdministratorEnrollmentClaimStateV1::Reserved
                && self
                    .status
                    .reservation_expires_at
                    .is_some_and(|expiry| expiry <= now))
        {
            self.status.state = AdministratorEnrollmentClaimStateV1::Expired;
            self.status.terminal_at = Some(now);
        }
    }

    fn secret_matches(&self, presented_secret: &str) -> bool {
        PasswordHash::new(&self.verifier)
            .ok()
            .is_some_and(|verifier| {
                Argon2::default()
                    .verify_password(presented_secret.as_bytes(), &verifier)
                    .is_ok()
            })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ClaimAccessError {
    #[error("administrator enrollment is unavailable")]
    EnrollmentUnavailable,
}

#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum ClaimTransitionError {
    #[error("claim issuance values are invalid")]
    InvalidIssue,
    #[error("claim verifier creation failed")]
    VerifierFailure,
    #[error("claim generation is already terminal")]
    AlreadyTerminal,
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use super::*;

    fn now() -> DateTime<Utc> {
        DateTime::parse_from_rfc3339("2026-07-23T18:00:00Z")
            .unwrap()
            .with_timezone(&Utc)
    }

    fn issued() -> IssuedEnrollmentClaimV1 {
        IssuedEnrollmentClaimV1::issue(
            Uuid::from_u128(1),
            Uuid::from_u128(2),
            1,
            AdministratorEnrollmentClaimKindV1::Initial,
            now(),
            now() + Duration::seconds(DEFAULT_CLAIM_LIFETIME_SECONDS),
        )
        .unwrap()
    }

    #[test]
    fn secret_is_once_exposed_and_debug_redacted() {
        let claim = issued();
        let debug = format!("{claim:?}");
        assert!(debug.contains("[REDACTED]"));
        assert!(!debug.contains(&claim.secret.0));
        let secret = claim.secret.expose_once();
        assert!(!secret.is_empty());
    }

    #[test]
    fn reserve_is_secret_bound_and_same_reservation_is_idempotent() {
        let claim = issued();
        let secret = claim.secret.0.clone();
        let mut persisted = claim.into_persisted();
        let reservation = Uuid::from_u128(3);
        assert_eq!(
            persisted.reserve(
                "wrong",
                reservation,
                now() + Duration::seconds(1),
                now() + Duration::seconds(121),
            ),
            Err(ClaimAccessError::EnrollmentUnavailable)
        );
        persisted
            .reserve(
                &secret,
                reservation,
                now() + Duration::seconds(1),
                now() + Duration::seconds(121),
            )
            .unwrap();
        persisted
            .reserve(
                "not-needed-on-resume",
                reservation,
                now() + Duration::seconds(2),
                now() + Duration::seconds(121),
            )
            .unwrap();
    }

    #[test]
    fn consume_requires_the_exact_live_reservation_and_is_idempotent() {
        let claim = issued();
        let secret = claim.secret.0.clone();
        let mut persisted = claim.into_persisted();
        let reservation = Uuid::from_u128(3);
        persisted
            .reserve(
                &secret,
                reservation,
                now() + Duration::seconds(1),
                now() + Duration::seconds(121),
            )
            .unwrap();
        let result = EnrollmentRedemptionResultV1 {
            schema_version: 1,
            installation_id: Uuid::from_u128(1),
            claim_id: Uuid::from_u128(2),
            generation: 1,
            reservation_id: reservation,
            account_id: Uuid::from_u128(4),
            role_id: Uuid::from_u128(5),
            completed_at: now() + Duration::seconds(10),
        };
        persisted.consume(result.clone()).unwrap();
        persisted.consume(result).unwrap();
        assert_eq!(
            persisted.status().state,
            AdministratorEnrollmentClaimStateV1::Consumed
        );
    }

    #[test]
    fn expired_revoked_replaced_and_consumed_are_one_caller_error() {
        let claim = issued();
        let secret = claim.secret.0.clone();
        let mut persisted = claim.into_persisted();
        assert_eq!(
            persisted.reserve(
                &secret,
                Uuid::from_u128(3),
                now() + Duration::seconds(DEFAULT_CLAIM_LIFETIME_SECONDS),
                now() + Duration::seconds(DEFAULT_CLAIM_LIFETIME_SECONDS + 1),
            ),
            Err(ClaimAccessError::EnrollmentUnavailable)
        );
        assert_eq!(
            persisted.reserve(
                &secret,
                Uuid::from_u128(4),
                now() + Duration::seconds(DEFAULT_CLAIM_LIFETIME_SECONDS + 1),
                now() + Duration::seconds(DEFAULT_CLAIM_LIFETIME_SECONDS + 2),
            ),
            Err(ClaimAccessError::EnrollmentUnavailable)
        );
    }
}
