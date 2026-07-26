use chrono::{DateTime, Duration, Utc};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use sqlx::{PgPool, Postgres, Row, Transaction};
use tessara_module_contract::{
    AdministratorEligibilityDecisionV1, AdministratorEnrollmentClaimKindV1,
    AdministratorEnrollmentClaimStateV1, EnrollmentRedemptionResultV1, ProtocolEnvelopeError,
    PurposeBoundVerifyingKeyV1, SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    ClaimAccessError, ClaimSecret, ClaimTransitionError, DEFAULT_CLAIM_LIFETIME_SECONDS,
    DEFAULT_RESERVATION_LIFETIME_SECONDS, EnrollmentClaimStatusV1, IssuedEnrollmentClaimV1,
    PersistedEnrollmentClaimV1,
};

#[derive(Clone)]
pub struct InstallationControlStore {
    pool: PgPool,
}

pub struct IssuedClaimOutputV1 {
    pub status: EnrollmentClaimStatusV1,
    pub secret: ClaimSecret,
}

impl std::fmt::Debug for IssuedClaimOutputV1 {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("IssuedClaimOutputV1")
            .field("status", &self.status)
            .field("secret", &"[REDACTED]")
            .finish()
    }
}

impl InstallationControlStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn migrate(&self) -> Result<(), InstallationControlError> {
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    pub async fn issue(
        &self,
        installation_id: Uuid,
        kind: AdministratorEnrollmentClaimKindV1,
        eligibility: SignedEnvelopeV1<AdministratorEligibilityDecisionV1>,
        eligibility_verifier: &PurposeBoundVerifyingKeyV1,
        recovery_verifier: Option<&PurposeBoundVerifyingKeyV1>,
        now: DateTime<Utc>,
    ) -> Result<IssuedClaimOutputV1, InstallationControlError> {
        eligibility_verifier.verify(&eligibility)?;
        eligibility
            .payload
            .validate_for(installation_id, kind, now, recovery_verifier)
            .map_err(|error| InstallationControlError::Ineligible(error.to_string()))?;

        let mut transaction = self.pool.begin().await?;
        installation_lock(&mut transaction, installation_id).await?;
        expire_active(&mut transaction, installation_id, now).await?;
        let active: Option<Uuid> = sqlx::query_scalar(
            "SELECT claim_id FROM administrator_enrollment_claims
             WHERE installation_id = $1 AND claim_state IN ('issued', 'reserved')
             LIMIT 1",
        )
        .bind(installation_id)
        .fetch_optional(&mut *transaction)
        .await?;
        if active.is_some() {
            return Err(InstallationControlError::ActiveClaimExists);
        }

        let generation: i32 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(generation), 0) + 1
             FROM administrator_enrollment_claims WHERE installation_id = $1",
        )
        .bind(installation_id)
        .fetch_one(&mut *transaction)
        .await?;
        let claim_id = Uuid::new_v4();
        let issued = IssuedEnrollmentClaimV1::issue(
            installation_id,
            claim_id,
            generation as u32,
            kind,
            now,
            now + Duration::seconds(DEFAULT_CLAIM_LIFETIME_SECONDS),
        )?;
        let eligibility_json = serde_json::to_value(&eligibility)?;
        let digest = json_digest(&eligibility_json)?;
        let recovery_json = eligibility
            .payload
            .recovery_authorization
            .as_ref()
            .map(serde_json::to_value)
            .transpose()?;

        sqlx::query(
            "INSERT INTO administrator_enrollment_claims
             (claim_id, installation_id, generation, claim_kind, claim_state,
              secret_verifier, eligibility_envelope, eligibility_digest,
              recovery_authorization, issued_at, expires_at)
             VALUES ($1,$2,$3,$4,'issued',$5,$6,$7,$8,$9,$10)",
        )
        .bind(claim_id)
        .bind(installation_id)
        .bind(generation)
        .bind(kind_text(kind))
        .bind(issued.verifier())
        .bind(eligibility_json)
        .bind(digest)
        .bind(recovery_json)
        .bind(now)
        .bind(issued.status.expires_at)
        .execute(&mut *transaction)
        .await?;
        append_event(
            &mut transaction,
            installation_id,
            claim_id,
            generation as u32,
            "issued",
            now,
            None,
            json!({"claim_kind": kind_text(kind), "eligibility_nonce": eligibility.payload.nonce}),
        )
        .await?;
        transaction.commit().await?;
        Ok(IssuedClaimOutputV1 {
            status: issued.status,
            secret: issued.secret,
        })
    }

    pub async fn status(
        &self,
        installation_id: Uuid,
    ) -> Result<Option<EnrollmentClaimStatusV1>, InstallationControlError> {
        let mut transaction = self.pool.begin().await?;
        installation_lock(&mut transaction, installation_id).await?;
        expire_active(&mut transaction, installation_id, Utc::now()).await?;
        let row = sqlx::query(
            "SELECT * FROM administrator_enrollment_claims
             WHERE installation_id = $1 ORDER BY generation DESC LIMIT 1",
        )
        .bind(installation_id)
        .fetch_optional(&mut *transaction)
        .await?;
        transaction.commit().await?;
        row.map(|row| status_from_row(&row)).transpose()
    }

    pub async fn reserve(
        &self,
        installation_id: Uuid,
        claim_id: Uuid,
        generation: u32,
        secret: &str,
        reservation_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<EnrollmentClaimStatusV1, ClaimAccessError> {
        self.reserve_inner(
            installation_id,
            claim_id,
            generation,
            secret,
            reservation_id,
            now,
        )
        .await
        .map_err(|_| ClaimAccessError::EnrollmentUnavailable)
    }

    async fn reserve_inner(
        &self,
        installation_id: Uuid,
        claim_id: Uuid,
        generation: u32,
        secret: &str,
        reservation_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<EnrollmentClaimStatusV1, InstallationControlError> {
        let mut transaction = self.pool.begin().await?;
        installation_lock(&mut transaction, installation_id).await?;
        let row = sqlx::query(
            "SELECT * FROM administrator_enrollment_claims
             WHERE installation_id=$1 AND claim_id=$2 AND generation=$3 FOR UPDATE",
        )
        .bind(installation_id)
        .bind(claim_id)
        .bind(generation as i32)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(InstallationControlError::Unavailable)?;
        let mut claim = claim_from_row(&row)?;
        claim
            .reserve(
                secret,
                reservation_id,
                now,
                now + Duration::seconds(DEFAULT_RESERVATION_LIFETIME_SECONDS),
            )
            .map_err(|_| InstallationControlError::Unavailable)?;
        persist_claim(&mut transaction, &claim).await?;
        append_event(
            &mut transaction,
            installation_id,
            claim_id,
            generation,
            "reserved",
            now,
            Some(reservation_id),
            json!({}),
        )
        .await?;
        transaction.commit().await?;
        Ok(claim.status().clone())
    }

    pub async fn consume(
        &self,
        result: EnrollmentRedemptionResultV1,
    ) -> Result<EnrollmentClaimStatusV1, ClaimAccessError> {
        self.consume_inner(result)
            .await
            .map_err(|_| ClaimAccessError::EnrollmentUnavailable)
    }

    pub async fn consume_signed(
        &self,
        result: SignedEnvelopeV1<EnrollmentRedemptionResultV1>,
        verifier: &PurposeBoundVerifyingKeyV1,
    ) -> Result<EnrollmentClaimStatusV1, ClaimAccessError> {
        verifier
            .verify(&result)
            .map_err(|_| ClaimAccessError::EnrollmentUnavailable)?;
        self.consume(result.payload).await
    }

    async fn consume_inner(
        &self,
        mut result: EnrollmentRedemptionResultV1,
    ) -> Result<EnrollmentClaimStatusV1, InstallationControlError> {
        result.completed_at =
            DateTime::from_timestamp_micros(result.completed_at.timestamp_micros())
                .ok_or(InstallationControlError::CorruptState)?;
        let mut transaction = self.pool.begin().await?;
        installation_lock(&mut transaction, result.installation_id).await?;
        let row = sqlx::query(
            "SELECT * FROM administrator_enrollment_claims
             WHERE installation_id=$1 AND claim_id=$2 AND generation=$3 FOR UPDATE",
        )
        .bind(result.installation_id)
        .bind(result.claim_id)
        .bind(result.generation as i32)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(InstallationControlError::Unavailable)?;
        let mut claim = claim_from_row(&row)?;
        claim
            .consume(result.clone())
            .map_err(|_| InstallationControlError::Unavailable)?;
        persist_claim(&mut transaction, &claim).await?;
        append_event(
            &mut transaction,
            result.installation_id,
            result.claim_id,
            result.generation,
            "consumed",
            result.completed_at,
            Some(result.reservation_id),
            json!({"account_id": result.account_id, "role_id": result.role_id}),
        )
        .await?;
        transaction.commit().await?;
        Ok(claim.status().clone())
    }

    pub async fn revoke(
        &self,
        installation_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<EnrollmentClaimStatusV1, InstallationControlError> {
        self.terminal_transition(installation_id, now, false).await
    }

    pub async fn replace(
        &self,
        installation_id: Uuid,
        now: DateTime<Utc>,
    ) -> Result<EnrollmentClaimStatusV1, InstallationControlError> {
        self.terminal_transition(installation_id, now, true).await
    }

    async fn terminal_transition(
        &self,
        installation_id: Uuid,
        now: DateTime<Utc>,
        replace: bool,
    ) -> Result<EnrollmentClaimStatusV1, InstallationControlError> {
        let mut transaction = self.pool.begin().await?;
        installation_lock(&mut transaction, installation_id).await?;
        let row = sqlx::query(
            "SELECT * FROM administrator_enrollment_claims
             WHERE installation_id=$1 AND claim_state IN ('issued','reserved')
             ORDER BY generation DESC LIMIT 1 FOR UPDATE",
        )
        .bind(installation_id)
        .fetch_optional(&mut *transaction)
        .await?
        .ok_or(InstallationControlError::NoActiveClaim)?;
        let mut claim = claim_from_row(&row)?;
        if replace {
            claim.replace(now)?;
        } else {
            claim.revoke(now)?;
        }
        persist_claim(&mut transaction, &claim).await?;
        let status = claim.status().clone();
        append_event(
            &mut transaction,
            installation_id,
            status.claim_id,
            status.generation,
            if replace { "replaced" } else { "revoked" },
            now,
            status.reservation_id,
            json!({}),
        )
        .await?;
        transaction.commit().await?;
        Ok(status)
    }

    pub async fn reconcile(
        &self,
        result: EnrollmentRedemptionResultV1,
    ) -> Result<EnrollmentClaimStatusV1, ClaimAccessError> {
        self.consume(result).await
    }
}

async fn installation_lock(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1::text, 6242))")
        .bind(installation_id)
        .execute(&mut **transaction)
        .await?;
    Ok(())
}

async fn expire_active(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    now: DateTime<Utc>,
) -> Result<(), InstallationControlError> {
    let rows = sqlx::query(
        "UPDATE administrator_enrollment_claims
         SET claim_state='expired', terminal_at=$2
         WHERE installation_id=$1 AND claim_state IN ('issued','reserved')
           AND (expires_at <= $2 OR (reservation_expires_at IS NOT NULL AND reservation_expires_at <= $2))
         RETURNING claim_id, generation, reservation_id",
    )
    .bind(installation_id)
    .bind(now)
    .fetch_all(&mut **transaction)
    .await?;
    for row in rows {
        append_event(
            transaction,
            installation_id,
            row.try_get("claim_id")?,
            row.try_get::<i32, _>("generation")? as u32,
            "expired",
            now,
            row.try_get("reservation_id")?,
            json!({}),
        )
        .await?;
    }
    Ok(())
}

async fn append_event(
    transaction: &mut Transaction<'_, Postgres>,
    installation_id: Uuid,
    claim_id: Uuid,
    generation: u32,
    event_kind: &str,
    occurred_at: DateTime<Utc>,
    reservation_id: Option<Uuid>,
    evidence: Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO administrator_enrollment_events
         (event_id, installation_id, claim_id, generation, event_kind, occurred_at,
          reservation_id, non_secret_evidence)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8)",
    )
    .bind(Uuid::new_v4())
    .bind(installation_id)
    .bind(claim_id)
    .bind(generation as i32)
    .bind(event_kind)
    .bind(occurred_at)
    .bind(reservation_id)
    .bind(evidence)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

async fn persist_claim(
    transaction: &mut Transaction<'_, Postgres>,
    claim: &PersistedEnrollmentClaimV1,
) -> Result<(), InstallationControlError> {
    let status = claim.status();
    let redemption = claim
        .redemption_result
        .as_ref()
        .map(serde_json::to_value)
        .transpose()?;
    sqlx::query(
        "UPDATE administrator_enrollment_claims SET
         claim_state=$4, reservation_id=$5, reservation_expires_at=$6,
         redemption_result=$7, terminal_at=$8
         WHERE installation_id=$1 AND claim_id=$2 AND generation=$3",
    )
    .bind(status.installation_id)
    .bind(status.claim_id)
    .bind(status.generation as i32)
    .bind(state_text(status.state))
    .bind(status.reservation_id)
    .bind(status.reservation_expires_at)
    .bind(redemption)
    .bind(status.terminal_at)
    .execute(&mut **transaction)
    .await?;
    Ok(())
}

fn claim_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<PersistedEnrollmentClaimV1, InstallationControlError> {
    let status = status_from_row(row)?;
    let redemption_result = row
        .try_get::<Option<Value>, _>("redemption_result")?
        .map(serde_json::from_value)
        .transpose()?;
    Ok(PersistedEnrollmentClaimV1::from_parts(
        status,
        row.try_get("secret_verifier")?,
        redemption_result,
    ))
}

fn status_from_row(
    row: &sqlx::postgres::PgRow,
) -> Result<EnrollmentClaimStatusV1, InstallationControlError> {
    Ok(EnrollmentClaimStatusV1 {
        schema_version: 1,
        installation_id: row.try_get("installation_id")?,
        claim_id: row.try_get("claim_id")?,
        generation: row.try_get::<i32, _>("generation")? as u32,
        kind: parse_kind(row.try_get("claim_kind")?)?,
        state: parse_state(row.try_get("claim_state")?)?,
        issued_at: row.try_get("issued_at")?,
        expires_at: row.try_get("expires_at")?,
        reservation_id: row.try_get("reservation_id")?,
        reservation_expires_at: row.try_get("reservation_expires_at")?,
        terminal_at: row.try_get("terminal_at")?,
    })
}

fn kind_text(kind: AdministratorEnrollmentClaimKindV1) -> &'static str {
    match kind {
        AdministratorEnrollmentClaimKindV1::Initial => "initial",
        AdministratorEnrollmentClaimKindV1::Recovery => "recovery",
    }
}

fn state_text(state: AdministratorEnrollmentClaimStateV1) -> &'static str {
    match state {
        AdministratorEnrollmentClaimStateV1::Issued => "issued",
        AdministratorEnrollmentClaimStateV1::Reserved => "reserved",
        AdministratorEnrollmentClaimStateV1::Consumed => "consumed",
        AdministratorEnrollmentClaimStateV1::Expired => "expired",
        AdministratorEnrollmentClaimStateV1::Revoked => "revoked",
        AdministratorEnrollmentClaimStateV1::Replaced => "replaced",
    }
}

fn parse_kind(value: &str) -> Result<AdministratorEnrollmentClaimKindV1, InstallationControlError> {
    match value {
        "initial" => Ok(AdministratorEnrollmentClaimKindV1::Initial),
        "recovery" => Ok(AdministratorEnrollmentClaimKindV1::Recovery),
        _ => Err(InstallationControlError::CorruptState),
    }
}

fn parse_state(
    value: &str,
) -> Result<AdministratorEnrollmentClaimStateV1, InstallationControlError> {
    match value {
        "issued" => Ok(AdministratorEnrollmentClaimStateV1::Issued),
        "reserved" => Ok(AdministratorEnrollmentClaimStateV1::Reserved),
        "consumed" => Ok(AdministratorEnrollmentClaimStateV1::Consumed),
        "expired" => Ok(AdministratorEnrollmentClaimStateV1::Expired),
        "revoked" => Ok(AdministratorEnrollmentClaimStateV1::Revoked),
        "replaced" => Ok(AdministratorEnrollmentClaimStateV1::Replaced),
        _ => Err(InstallationControlError::CorruptState),
    }
}

fn json_digest(value: &Value) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
}

#[derive(Debug, thiserror::Error)]
pub enum InstallationControlError {
    #[error("installation-control database operation failed")]
    Database(#[from] sqlx::Error),
    #[error("installation-control migration failed")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("signed eligibility verification failed: {0}")]
    Signature(#[from] ProtocolEnvelopeError),
    #[error("administrator eligibility was denied: {0}")]
    Ineligible(String),
    #[error("an active enrollment claim already exists")]
    ActiveClaimExists,
    #[error("no active enrollment claim exists")]
    NoActiveClaim,
    #[error("administrator enrollment is unavailable")]
    Unavailable,
    #[error("persisted installation-control state is invalid")]
    CorruptState,
    #[error("claim transition failed: {0}")]
    Transition(#[from] ClaimTransitionError),
    #[error("installation-control JSON operation failed")]
    Json(#[from] serde_json::Error),
}
