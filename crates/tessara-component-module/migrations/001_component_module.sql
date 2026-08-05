-- Sprint 8A Component Module fresh baseline.
--
-- This schema is owned by one Component Module Instance. Dataset relationships
-- are typed Core contract references; no Dataset, Dashboard, or Core foreign
-- key, view, credential, or writable schema is present.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE component_type AS ENUM ('table', 'bar', 'line', 'pie', 'donut', 'stat_card');
CREATE TYPE component_version_status AS ENUM ('draft', 'published', 'superseded');
CREATE TYPE component_lifecycle_state AS ENUM ('active', 'inactive', 'archived', 'tombstoned');
CREATE TYPE component_change_category AS ENUM ('publication', 'lifecycle', 'payload', 'successor');

CREATE TABLE component_configuration (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    display_label TEXT NOT NULL
        CHECK (btrim(display_label) <> '' AND char_length(display_label) <= 80),
    dataset_request_timeout_seconds INTEGER NOT NULL
        CHECK (dataset_request_timeout_seconds BETWEEN 1 AND 30),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO component_configuration
    (singleton, schema_version, display_label, dataset_request_timeout_seconds)
VALUES (true, 1, 'Components', 5);

CREATE TABLE component_security_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    installation_id UUID NOT NULL,
    module_instance_id UUID NOT NULL,
    authorization_revision BIGINT NOT NULL CHECK (authorization_revision > 0),
    organization_revision BIGINT NOT NULL CHECK (organization_revision > 0),
    enabled BOOLEAN NOT NULL DEFAULT false,
    document_state TEXT NOT NULL DEFAULT 'disabled'
        CHECK (document_state IN ('enabled', 'disabled', 'degraded', 'recovery')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE components (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    external_key TEXT UNIQUE,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    slug TEXT NOT NULL UNIQUE CHECK (btrim(slug) <> ''),
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE component_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    component_id UUID NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    dataset_reference JSONB NOT NULL CHECK (jsonb_typeof(dataset_reference) = 'object'),
    dataset_scope_node_ids UUID[] NOT NULL CHECK (cardinality(dataset_scope_node_ids) > 0),
    component_type component_type NOT NULL,
    status component_version_status NOT NULL DEFAULT 'draft',
    lifecycle_state component_lifecycle_state NOT NULL DEFAULT 'active',
    resource_revision BIGINT NOT NULL DEFAULT 1 CHECK (resource_revision > 0),
    authority_revision BIGINT NOT NULL DEFAULT 1 CHECK (authority_revision > 0),
    successor_version_id UUID REFERENCES component_versions(id) ON DELETE RESTRICT,
    version_number INTEGER NOT NULL CHECK (version_number > 0),
    version_label TEXT NOT NULL CHECK (btrim(version_label) <> ''),
    version_note TEXT NOT NULL DEFAULT '',
    config JSONB NOT NULL CHECK (jsonb_typeof(config) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (component_id, version_number),
    CHECK (dataset_reference #>> '{reference,resource_type}' = 'tessara.transition.dataset_major_line'),
    CHECK (dataset_reference #>> '{reference,owner,kind}' = 'core_installation'),
    CHECK (dataset_reference #>> '{reference,installation_id}' =
           dataset_reference #>> '{reference,owner,installation_id}'),
    CHECK (successor_version_id IS NULL OR successor_version_id <> id)
);

CREATE INDEX component_versions_component_status_idx
    ON component_versions (component_id, status, lifecycle_state, version_number DESC);

CREATE TABLE component_version_change_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    component_version_id UUID NOT NULL REFERENCES component_versions(id) ON DELETE RESTRICT,
    resource_revision BIGINT NOT NULL CHECK (resource_revision > 0),
    category component_change_category NOT NULL,
    from_publication_state component_version_status,
    to_publication_state component_version_status,
    from_lifecycle_state component_lifecycle_state,
    to_lifecycle_state component_lifecycle_state,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (component_version_id, resource_revision, category),
    CONSTRAINT component_version_change_events_publication_chk CHECK (
        category <> 'publication'
        OR from_publication_state IS DISTINCT FROM to_publication_state
    ),
    CONSTRAINT component_version_change_events_lifecycle_chk CHECK (
        category <> 'lifecycle'
        OR from_lifecycle_state IS DISTINCT FROM to_lifecycle_state
    )
);

CREATE INDEX component_version_change_events_version_revision_idx
    ON component_version_change_events
       (component_version_id, resource_revision, created_at, id);

CREATE OR REPLACE FUNCTION reject_component_version_change_event_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'component version change history is immutable';
END;
$$;

CREATE TRIGGER component_version_change_events_immutable
BEFORE UPDATE OR DELETE ON component_version_change_events
FOR EACH ROW EXECUTE FUNCTION reject_component_version_change_event_mutation();

CREATE TABLE component_mutation_replays (
    jti UUID PRIMARY KEY,
    original_actor_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (btrim(action) <> ''),
    payload_digest TEXT NOT NULL CHECK (payload_digest ~ '^sha256:[0-9a-f]{64}$'),
    idempotency_key TEXT NOT NULL UNIQUE CHECK (btrim(idempotency_key) <> ''),
    result JSONB NOT NULL CHECK (jsonb_typeof(result) = 'object'),
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE component_consumed_service_nonces (
    module_instance_id UUID NOT NULL,
    nonce UUID NOT NULL,
    correlation_id TEXT NOT NULL CHECK (btrim(correlation_id) <> ''),
    issued_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (module_instance_id, nonce)
);

CREATE TABLE component_bootstrap_receipts (
    idempotency_key TEXT PRIMARY KEY CHECK (btrim(idempotency_key) <> ''),
    input_digest TEXT NOT NULL CHECK (input_digest ~ '^sha256:[0-9a-f]{64}$'),
    desired_revision BIGINT NOT NULL CHECK (desired_revision > 0),
    receipt JSONB NOT NULL CHECK (jsonb_typeof(receipt) = 'object'),
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
