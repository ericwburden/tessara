-- Sprint 6B2 secure module operation state. This migration is folded into the
-- baseline at sprint closeout after upgrade/rollback evidence is retained.

INSERT INTO capabilities (key, description, scope_mode)
VALUES (
    'core:admin',
    'Administer Core identity, roles, Organization, modules, and recovery.',
    'installation_global'
)
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    scope_mode = EXCLUDED.scope_mode;

INSERT INTO roles (name, description)
VALUES (
    'Core Administrator',
    'Installation-global role satisfying Core Administration Capability Floor v1.'
)
ON CONFLICT (name) DO UPDATE SET description = EXCLUDED.description;

INSERT INTO role_capabilities (role_id, capability_id)
SELECT roles.id, capabilities.id
FROM roles CROSS JOIN capabilities
WHERE roles.name = 'Core Administrator' AND capabilities.key = 'core:admin'
ON CONFLICT DO NOTHING;

CREATE TABLE core_administration_state (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    floor_version TEXT NOT NULL CHECK (floor_version = 'core-administration-v1'),
    designated_enrollment_role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    has_ever_had_viable_administrator BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO core_administration_state
    (singleton, floor_version, designated_enrollment_role_id)
SELECT true, 'core-administration-v1', id
FROM roles WHERE name = 'Core Administrator'
ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE core_security_revisions (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    authorization_revision BIGINT NOT NULL DEFAULT 1 CHECK (authorization_revision > 0),
    organization_revision BIGINT NOT NULL DEFAULT 1 CHECK (organization_revision > 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO core_security_revisions (singleton) VALUES (true)
ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE external_identity_bindings (
    issuer TEXT NOT NULL,
    external_subject TEXT NOT NULL,
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    is_usable BOOLEAN NOT NULL DEFAULT true,
    assertion_nonce UUID NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (issuer, external_subject),
    UNIQUE (account_id, issuer)
);

CREATE TABLE administrator_enrollment_redemptions (
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    claim_id UUID NOT NULL,
    generation INTEGER NOT NULL CHECK (generation > 0),
    reservation_id UUID NOT NULL UNIQUE,
    claim_kind TEXT NOT NULL CHECK (claim_kind IN ('initial', 'recovery')),
    identity_path TEXT NOT NULL CHECK (identity_path IN ('local', 'fixture_external')),
    account_id UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    completed_at TIMESTAMPTZ NOT NULL,
    result_envelope JSONB,
    PRIMARY KEY (installation_id, claim_id, generation)
);

CREATE TABLE consumed_authorization_mutations (
    module_instance_id UUID NOT NULL REFERENCES module_instances(id) ON DELETE CASCADE,
    jti UUID NOT NULL,
    action TEXT NOT NULL,
    payload_digest TEXT NOT NULL CHECK (payload_digest ~ '^sha256:[0-9a-f]{64}$'),
    idempotency_key TEXT NOT NULL,
    result JSONB NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (module_instance_id, jti),
    UNIQUE (module_instance_id, idempotency_key)
);

CREATE TABLE core_module_action_declarations (
    target_definition_id TEXT NOT NULL,
    dependency_binding TEXT NOT NULL,
    functional_contract TEXT NOT NULL,
    action TEXT NOT NULL,
    operation TEXT NOT NULL CHECK (operation IN ('read', 'mutation')),
    required_capability TEXT NOT NULL,
    PRIMARY KEY (target_definition_id, dependency_binding, functional_contract, action)
);

INSERT INTO core_module_action_declarations
    (target_definition_id, dependency_binding, functional_contract, action, operation, required_capability)
VALUES
    ('tessara.reference.scoped-records', 'tessara.core.scoped-records',
     'tessara.reference.scoped-records.record', 'records.list', 'read',
     'tessara.reference.scoped-records:read'),
    ('tessara.reference.scoped-records', 'tessara.core.scoped-records',
     'tessara.reference.scoped-records.record', 'records.get', 'read',
     'tessara.reference.scoped-records:read'),
    ('tessara.reference.scoped-records', 'tessara.core.scoped-records',
     'tessara.reference.scoped-records.record', 'records.create', 'mutation',
     'tessara.reference.scoped-records:manage'),
    ('tessara.reference.scoped-records', 'tessara.core.scoped-records',
     'tessara.reference.scoped-records.record', 'records.update', 'mutation',
     'tessara.reference.scoped-records:manage')
ON CONFLICT DO NOTHING;

CREATE TABLE core_security_events (
    event_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_kind TEXT NOT NULL,
    actor_account_id UUID REFERENCES accounts(id) ON DELETE SET NULL,
    subject_id UUID,
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    occurred_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION advance_authorization_revision()
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE revision BIGINT;
BEGIN
    UPDATE core_security_revisions
    SET authorization_revision = authorization_revision + 1, updated_at = now()
    WHERE singleton = true
    RETURNING authorization_revision INTO revision;
    RETURN revision;
END $$;

CREATE OR REPLACE FUNCTION advance_organization_revision()
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE revision BIGINT;
BEGIN
    UPDATE core_security_revisions
    SET organization_revision = organization_revision + 1, updated_at = now()
    WHERE singleton = true
    RETURNING organization_revision INTO revision;
    RETURN revision;
END $$;

CREATE OR REPLACE FUNCTION security_authorization_revision_trigger()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM advance_authorization_revision();
    RETURN NULL;
END $$;

CREATE OR REPLACE FUNCTION security_organization_revision_trigger()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    PERFORM advance_organization_revision();
    RETURN NULL;
END $$;

CREATE TRIGGER role_capabilities_security_revision
AFTER INSERT OR UPDATE OR DELETE ON role_capabilities
FOR EACH STATEMENT EXECUTE FUNCTION security_authorization_revision_trigger();
CREATE TRIGGER role_assignments_security_revision
AFTER INSERT OR UPDATE OR DELETE ON role_assignments
FOR EACH STATEMENT EXECUTE FUNCTION security_authorization_revision_trigger();
CREATE TRIGGER accounts_security_revision
AFTER INSERT OR UPDATE OR DELETE ON accounts
FOR EACH STATEMENT EXECUTE FUNCTION security_authorization_revision_trigger();
CREATE TRIGGER account_credentials_security_revision
AFTER INSERT OR UPDATE OR DELETE ON account_credentials
FOR EACH STATEMENT EXECUTE FUNCTION security_authorization_revision_trigger();
CREATE TRIGGER external_identities_security_revision
AFTER INSERT OR UPDATE OR DELETE ON external_identity_bindings
FOR EACH STATEMENT EXECUTE FUNCTION security_authorization_revision_trigger();
CREATE TRIGGER delegations_security_revision
AFTER INSERT OR UPDATE OR DELETE ON account_delegations
FOR EACH STATEMENT EXECUTE FUNCTION security_authorization_revision_trigger();
CREATE TRIGGER nodes_organization_revision
AFTER INSERT OR UPDATE OR DELETE ON nodes
FOR EACH STATEMENT EXECUTE FUNCTION security_organization_revision_trigger();
