CREATE TABLE administrator_enrollment_claims (
    claim_id UUID NOT NULL,
    installation_id UUID NOT NULL,
    generation INTEGER NOT NULL CHECK (generation > 0),
    claim_kind TEXT NOT NULL CHECK (claim_kind IN ('initial', 'recovery')),
    claim_state TEXT NOT NULL CHECK (
        claim_state IN ('issued', 'reserved', 'consumed', 'expired', 'revoked', 'replaced')
    ),
    secret_verifier TEXT NOT NULL CHECK (length(secret_verifier) > 0),
    eligibility_envelope JSONB NOT NULL,
    eligibility_digest TEXT NOT NULL CHECK (eligibility_digest ~ '^sha256:[0-9a-f]{64}$'),
    recovery_authorization JSONB,
    reservation_id UUID,
    reservation_expires_at TIMESTAMPTZ,
    redemption_result JSONB,
    issued_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL CHECK (expires_at > issued_at),
    terminal_at TIMESTAMPTZ,
    PRIMARY KEY (claim_id, generation),
    CHECK (
        (claim_kind = 'recovery' AND recovery_authorization IS NOT NULL)
        OR (claim_kind = 'initial' AND recovery_authorization IS NULL)
    ),
    CHECK (
        (claim_state = 'reserved' AND reservation_id IS NOT NULL AND reservation_expires_at IS NOT NULL)
        OR claim_state <> 'reserved'
    ),
    CHECK (
        (claim_state = 'consumed' AND redemption_result IS NOT NULL AND terminal_at IS NOT NULL)
        OR claim_state <> 'consumed'
    ),
    CHECK (
        (claim_state IN ('expired', 'revoked', 'replaced') AND terminal_at IS NOT NULL)
        OR claim_state NOT IN ('expired', 'revoked', 'replaced')
    )
);

CREATE UNIQUE INDEX one_active_administrator_enrollment_claim
    ON administrator_enrollment_claims (installation_id)
    WHERE claim_state IN ('issued', 'reserved');

CREATE UNIQUE INDEX one_enrollment_reservation
    ON administrator_enrollment_claims (reservation_id)
    WHERE reservation_id IS NOT NULL;

CREATE TABLE administrator_enrollment_events (
    event_id UUID PRIMARY KEY,
    installation_id UUID NOT NULL,
    claim_id UUID NOT NULL,
    generation INTEGER NOT NULL,
    event_kind TEXT NOT NULL CHECK (
        event_kind IN ('issued', 'reserved', 'consumed', 'expired', 'revoked', 'replaced', 'reconciled')
    ),
    occurred_at TIMESTAMPTZ NOT NULL,
    reservation_id UUID,
    non_secret_evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    FOREIGN KEY (claim_id, generation)
        REFERENCES administrator_enrollment_claims (claim_id, generation)
);

CREATE INDEX administrator_enrollment_events_installation
    ON administrator_enrollment_events (installation_id, occurred_at, event_id);
