CREATE TABLE module_releases (
    id UUID PRIMARY KEY,
    definition_id TEXT NOT NULL REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    version TEXT NOT NULL,
    manifest_digest TEXT NOT NULL,
    runtime_image_digest TEXT NOT NULL,
    publisher TEXT NOT NULL,
    trust_state TEXT NOT NULL CHECK (trust_state IN ('unknown', 'trusted', 'rejected')),
    compatibility_state TEXT NOT NULL CHECK (compatibility_state IN ('not_evaluated', 'compatible', 'incompatible')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (definition_id, manifest_digest)
);

CREATE TABLE module_instances (
    id UUID PRIMARY KEY,
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    definition_id TEXT NOT NULL REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    release_id UUID NOT NULL REFERENCES module_releases(id) ON DELETE RESTRICT,
    identity_state TEXT NOT NULL CHECK (identity_state IN ('live', 'tombstoned')),
    data_state TEXT NOT NULL CHECK (data_state IN ('retained', 'destroyed')),
    database_name TEXT NOT NULL,
    installed BOOLEAN NOT NULL,
    deployed BOOLEAN NOT NULL,
    configured BOOLEAN NOT NULL,
    ready BOOLEAN NOT NULL,
    enabled BOOLEAN NOT NULL,
    healthy BOOLEAN NOT NULL,
    last_observed_at TIMESTAMPTZ NOT NULL,
    UNIQUE (installation_id, definition_id)
);

CREATE TABLE deployment_receipts (
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    revision BIGINT NOT NULL CHECK (revision > 0),
    plan_digest TEXT NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL,
    operator_name TEXT NOT NULL,
    idempotency_key TEXT NOT NULL UNIQUE,
    previous_revision BIGINT,
    receipt JSONB NOT NULL,
    PRIMARY KEY (installation_id, revision),
    CONSTRAINT deployment_receipts_previous_revision_chk CHECK (previous_revision IS NULL OR previous_revision <> revision)
);

CREATE INDEX deployment_receipts_current_idx
    ON deployment_receipts (installation_id, revision DESC);
