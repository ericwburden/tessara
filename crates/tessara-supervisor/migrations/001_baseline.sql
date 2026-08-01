PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS installation_root (
    singleton INTEGER PRIMARY KEY CHECK (singleton = 1),
    installation_id TEXT NOT NULL UNIQUE,
    created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS trust_anchors (
    issuer TEXT NOT NULL,
    key_id TEXT NOT NULL,
    purpose TEXT NOT NULL,
    public_key BLOB NOT NULL,
    created_at TEXT NOT NULL,
    PRIMARY KEY (issuer, key_id, purpose)
);

CREATE TABLE IF NOT EXISTS accepted_nonces (
    nonce TEXT PRIMARY KEY,
    accepted_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS operations (
    operation_id TEXT PRIMARY KEY,
    installation_id TEXT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    apply_sequence INTEGER NOT NULL CHECK (apply_sequence > 0),
    plan_digest TEXT NOT NULL,
    authorization_digest TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('accepted','acquiring','provisioning','migrating','configuring','bootstrapping','health_checking','switching','verifying','succeeded','failed','rolled_back')),
    accepted_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    plan_json TEXT NOT NULL,
    authorization_json TEXT NOT NULL,
    finding_json TEXT,
    receipt_digest TEXT
);

CREATE UNIQUE INDEX IF NOT EXISTS operations_active_one
ON operations ((1))
WHERE state NOT IN ('succeeded','failed','rolled_back');

CREATE TABLE IF NOT EXISTS receipts (
    revision INTEGER PRIMARY KEY CHECK (revision > 0),
    receipt_digest TEXT NOT NULL UNIQUE,
    receipt_json TEXT NOT NULL,
    applied_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS emergency_overrides (
    override_id TEXT PRIMARY KEY,
    definition_id TEXT NOT NULL,
    reason TEXT NOT NULL,
    actor_json TEXT NOT NULL,
    issued_at TEXT NOT NULL,
    expires_at TEXT,
    authorization_digest TEXT NOT NULL UNIQUE,
    reconciled_at TEXT
);

CREATE TABLE IF NOT EXISTS enrollment_claims (
    claim_id TEXT NOT NULL,
    generation INTEGER NOT NULL,
    state TEXT NOT NULL,
    verifier BLOB NOT NULL,
    non_secret_metadata_json TEXT NOT NULL,
    PRIMARY KEY (claim_id, generation)
);

CREATE TABLE IF NOT EXISTS enrollment_events (
    event_id TEXT PRIMARY KEY,
    claim_id TEXT NOT NULL,
    generation INTEGER NOT NULL,
    event_kind TEXT NOT NULL,
    occurred_at TEXT NOT NULL,
    evidence_json TEXT NOT NULL
);
