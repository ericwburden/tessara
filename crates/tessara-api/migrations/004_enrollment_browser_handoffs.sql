-- Short-lived, non-secret browser handoffs created by installation control.
-- Tokens are stored only as digests and may be consumed once.

CREATE TABLE administrator_enrollment_handoffs (
    token_digest TEXT PRIMARY KEY CHECK (token_digest ~ '^sha256:[0-9a-f]{64}$'),
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE CASCADE,
    claim_id UUID NOT NULL,
    generation INTEGER NOT NULL CHECK (generation > 0),
    claim_kind TEXT NOT NULL CHECK (claim_kind IN ('initial', 'recovery')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ,
    CHECK (expires_at > created_at),
    CHECK (consumed_at IS NULL OR consumed_at >= created_at)
);

CREATE INDEX administrator_enrollment_handoffs_expiry
    ON administrator_enrollment_handoffs (expires_at)
    WHERE consumed_at IS NULL;
