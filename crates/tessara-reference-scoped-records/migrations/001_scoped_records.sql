-- Sprint 6B2 closeout baseline. This is the sole migration for a freshly
-- provisioned Scoped Records Module Instance database.

CREATE TABLE IF NOT EXISTS scoped_records (
    id UUID PRIMARY KEY,
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    scope TEXT NOT NULL CHECK (length(trim(scope)) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

ALTER TABLE scoped_records ADD COLUMN organization_owner_id UUID;
UPDATE scoped_records
SET organization_owner_id = md5(scope)::uuid
WHERE organization_owner_id IS NULL;

CREATE OR REPLACE FUNCTION scoped_records_legacy_owner_compatibility()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF NEW.organization_owner_id IS NULL THEN
        NEW.organization_owner_id := md5(NEW.scope)::uuid;
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER scoped_records_legacy_owner_compatibility
BEFORE INSERT OR UPDATE ON scoped_records
FOR EACH ROW EXECUTE FUNCTION scoped_records_legacy_owner_compatibility();

ALTER TABLE scoped_records ALTER COLUMN organization_owner_id SET NOT NULL;
ALTER TABLE scoped_records ADD COLUMN updated_at TIMESTAMPTZ NOT NULL DEFAULT now();
CREATE INDEX scoped_records_organization_owner_idx
    ON scoped_records (organization_owner_id, updated_at DESC, id);

CREATE TABLE scoped_records_configuration (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    display_label TEXT NOT NULL CHECK (btrim(display_label) <> ''),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
INSERT INTO scoped_records_configuration (singleton, schema_version, display_label)
VALUES (true, 1, 'Scoped Records') ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE scoped_records_security_state (
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

CREATE TABLE scoped_records_mutation_replays (
    jti UUID PRIMARY KEY,
    original_actor_id UUID NOT NULL,
    action TEXT NOT NULL,
    payload_digest TEXT NOT NULL CHECK (payload_digest ~ '^sha256:[0-9a-f]{64}$'),
    idempotency_key TEXT NOT NULL UNIQUE,
    result JSONB NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE scoped_records_bootstrap_receipts (
    idempotency_key TEXT PRIMARY KEY CHECK (btrim(idempotency_key) <> ''),
    input_digest TEXT NOT NULL CHECK (input_digest ~ '^sha256:[0-9a-f]{64}$'),
    desired_revision BIGINT NOT NULL CHECK (desired_revision > 0),
    receipt JSONB NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
