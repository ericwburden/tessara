CREATE TYPE response_status AS ENUM ('draft', 'submitted', 'deleted');

CREATE TABLE response_module_configuration (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    display_label TEXT NOT NULL,
    provider_request_timeout_seconds SMALLINT NOT NULL CHECK (provider_request_timeout_seconds BETWEEN 1 AND 30),
    workflow_event_page_size SMALLINT NOT NULL CHECK (workflow_event_page_size BETWEEN 1 AND 1000),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO response_module_configuration (
    singleton, schema_version, display_label,
    provider_request_timeout_seconds, workflow_event_page_size
) VALUES (TRUE, 1, 'Responses', 5, 250);

CREATE TABLE response_module_security_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    schema_version SMALLINT NOT NULL CHECK (schema_version = 1),
    installation_id UUID NOT NULL,
    module_instance_id UUID NOT NULL,
    authorization_revision BIGINT NOT NULL CHECK (authorization_revision >= 0),
    organization_revision BIGINT NOT NULL CHECK (organization_revision >= 0),
    enabled BOOLEAN NOT NULL,
    document_state TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE response_consumed_service_nonces (
    module_instance_id UUID NOT NULL,
    nonce UUID NOT NULL,
    authorization_jti UUID NOT NULL UNIQUE,
    correlation_id TEXT NOT NULL CHECK (btrim(correlation_id) <> ''),
    issued_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (module_instance_id, nonce)
);

CREATE TABLE response_consumed_core_service_nonces (
    installation_id UUID NOT NULL,
    nonce UUID NOT NULL,
    authorization_jti UUID NOT NULL UNIQUE,
    correlation_id TEXT NOT NULL CHECK (btrim(correlation_id) <> ''),
    issued_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, nonce)
);

CREATE TABLE responses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    form_id UUID NOT NULL,
    form_version_id UUID NOT NULL,
    node_id UUID NOT NULL,
    workflow_assignment_id UUID NOT NULL,
    workflow_version_id UUID NOT NULL,
    workflow_step_id UUID NOT NULL,
    workflow_instance_id UUID NOT NULL,
    workflow_step_instance_id UUID NOT NULL,
    assignee_account_id UUID NOT NULL,
    started_by_account_id UUID NOT NULL,
    delegation_basis TEXT,
    status response_status NOT NULL DEFAULT 'draft',
    revision BIGINT NOT NULL DEFAULT 1 CHECK (revision > 0),
    form_snapshot JSONB NOT NULL,
    form_snapshot_digest TEXT NOT NULL CHECK (form_snapshot_digest ~ '^sha256:[0-9a-f]{64}$'),
    workflow_context JSONB NOT NULL,
    workflow_context_digest TEXT NOT NULL CHECK (workflow_context_digest ~ '^sha256:[0-9a-f]{64}$'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    submitted_at TIMESTAMPTZ,
    deleted_at TIMESTAMPTZ,
    CHECK (status <> 'submitted' OR submitted_at IS NOT NULL),
    CHECK (status <> 'draft' OR (submitted_at IS NULL AND deleted_at IS NULL)),
    CHECK (status <> 'deleted' OR deleted_at IS NOT NULL)
);

CREATE INDEX responses_scope_status_idx ON responses (node_id, status, updated_at DESC, id);
CREATE UNIQUE INDEX responses_active_assignment_idx
    ON responses (workflow_assignment_id)
    WHERE status IN ('draft', 'submitted');

CREATE TABLE response_values (
    response_id UUID NOT NULL REFERENCES responses(id) ON DELETE CASCADE,
    field_id UUID NOT NULL,
    field_key TEXT NOT NULL,
    value JSONB NOT NULL,
    value_text TEXT,
    PRIMARY KEY (response_id, field_id),
    UNIQUE (response_id, field_key)
);

CREATE TABLE response_audit_events (
    id BIGSERIAL PRIMARY KEY,
    response_id UUID NOT NULL REFERENCES responses(id) ON DELETE CASCADE,
    response_revision BIGINT NOT NULL CHECK (response_revision > 0),
    action TEXT NOT NULL,
    actor_account_id UUID NOT NULL,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    detail JSONB NOT NULL DEFAULT '{}'::jsonb,
    UNIQUE (response_id, response_revision, action)
);

CREATE TABLE response_idempotency_receipts (
    actor_account_id UUID NOT NULL,
    action TEXT NOT NULL,
    idempotency_key_digest TEXT NOT NULL CHECK (idempotency_key_digest ~ '^sha256:[0-9a-f]{64}$'),
    request_digest TEXT NOT NULL CHECK (request_digest ~ '^sha256:[0-9a-f]{64}$'),
    response_status SMALLINT NOT NULL,
    response_body JSONB NOT NULL,
    committed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (actor_account_id, action, idempotency_key_digest)
);

CREATE TABLE response_workflow_event_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    provider_epoch UUID NOT NULL DEFAULT gen_random_uuid()
);
INSERT INTO response_workflow_event_state (singleton) VALUES (TRUE);

CREATE TABLE response_workflow_events (
    sequence BIGSERIAL PRIMARY KEY,
    event_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    response_id UUID NOT NULL REFERENCES responses(id) ON DELETE CASCADE,
    response_revision BIGINT NOT NULL CHECK (response_revision > 0),
    event_kind TEXT NOT NULL CHECK (event_kind IN ('started', 'draft_saved', 'submitted', 'deleted')),
    payload JSONB NOT NULL,
    content_digest TEXT NOT NULL CHECK (content_digest ~ '^sha256:[0-9a-f]{64}$'),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (response_id, response_revision, event_kind)
);

CREATE TABLE response_export_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    provider_epoch UUID NOT NULL DEFAULT gen_random_uuid()
);
INSERT INTO response_export_state (singleton) VALUES (TRUE);

CREATE TABLE response_export_changes (
    sequence BIGSERIAL PRIMARY KEY,
    response_id UUID NOT NULL,
    form_version_id UUID NOT NULL,
    node_id UUID NOT NULL,
    change_kind TEXT NOT NULL CHECK (change_kind IN ('upsert', 'tombstone')),
    payload JSONB NOT NULL,
    content_digest TEXT NOT NULL CHECK (content_digest ~ '^sha256:[0-9a-f]{64}$'),
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
CREATE INDEX response_export_changes_response_idx
    ON response_export_changes (response_id, sequence DESC);
CREATE INDEX response_export_changes_partition_idx
    ON response_export_changes (form_version_id, node_id, sequence);

CREATE TABLE response_bootstrap_receipts (
    logical_key TEXT PRIMARY KEY,
    request_digest TEXT NOT NULL CHECK (request_digest ~ '^sha256:[0-9a-f]{64}$'),
    response_id UUID NOT NULL REFERENCES responses(id),
    signed_receipt JSONB NOT NULL,
    committed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
