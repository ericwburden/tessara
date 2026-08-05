-- Sprint 6C Dashboard Module baseline.
--
-- This schema is owned by one Dashboard Module Instance. It intentionally has
-- no foreign key, view, or runtime dependency on a Core or Components table.

CREATE TABLE dashboard_configuration (
    singleton BOOLEAN PRIMARY KEY DEFAULT true CHECK (singleton),
    schema_version INTEGER NOT NULL CHECK (schema_version = 1),
    display_label TEXT NOT NULL CHECK (btrim(display_label) <> ''),
    default_page_size INTEGER NOT NULL CHECK (default_page_size BETWEEN 10 AND 100),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

INSERT INTO dashboard_configuration
    (singleton, schema_version, display_label, default_page_size)
VALUES (true, 1, 'Dashboards', 25)
ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE dashboard_security_state (
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

CREATE TABLE dashboard_organization_nodes (
    node_id UUID PRIMARY KEY,
    node_name TEXT NOT NULL CHECK (btrim(node_name) <> ''),
    node_type_name TEXT NOT NULL CHECK (btrim(node_type_name) <> ''),
    parent_node_id UUID,
    node_path TEXT NOT NULL CHECK (btrim(node_path) <> ''),
    active BOOLEAN NOT NULL DEFAULT true,
    projection_revision BIGINT NOT NULL CHECK (projection_revision > 0),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE dashboards (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL CHECK (btrim(name) <> ''),
    description TEXT,
    authority_revision BIGINT NOT NULL DEFAULT 1 CHECK (authority_revision > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE dashboard_scope_nodes (
    dashboard_id UUID NOT NULL,
    node_id UUID NOT NULL REFERENCES dashboard_organization_nodes(node_id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (dashboard_id, node_id)
);

CREATE INDEX dashboard_scope_nodes_node_id_idx
    ON dashboard_scope_nodes (node_id, dashboard_id);

CREATE TABLE dashboard_placements (
    id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    component_reference JSONB NOT NULL,
    position INTEGER NOT NULL CHECK (position >= 0 AND position < 240),
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (dashboard_id, position),
    CHECK (jsonb_typeof(component_reference) = 'object'),
    CHECK (component_reference ->> 'resource_type' = 'tessara.transition.component_version'),
    CHECK (component_reference #>> '{owner,kind}' = 'core_installation'),
    CHECK (component_reference ->> 'installation_id' =
           component_reference #>> '{owner,installation_id}')
);

CREATE INDEX dashboard_placements_dashboard_position_idx
    ON dashboard_placements (dashboard_id, position, id);

CREATE OR REPLACE FUNCTION advance_dashboard_authority_revision()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    target_dashboard_id UUID;
BEGIN
    target_dashboard_id := CASE WHEN TG_OP = 'DELETE' THEN OLD.dashboard_id ELSE NEW.dashboard_id END;
    UPDATE dashboards
    SET authority_revision = authority_revision + 1, updated_at = now()
    WHERE id = target_dashboard_id;
    RETURN CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;
END;
$$;

CREATE TRIGGER dashboard_scope_nodes_authority_revision
AFTER INSERT OR UPDATE OR DELETE ON dashboard_scope_nodes
FOR EACH ROW EXECUTE FUNCTION advance_dashboard_authority_revision();

CREATE TRIGGER dashboard_placements_authority_revision
AFTER INSERT OR UPDATE OR DELETE ON dashboard_placements
FOR EACH ROW EXECUTE FUNCTION advance_dashboard_authority_revision();

CREATE OR REPLACE FUNCTION advance_dashboard_metadata_authority_revision()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    IF ROW(NEW.name, NEW.description) IS DISTINCT FROM ROW(OLD.name, OLD.description) THEN
        NEW.authority_revision := OLD.authority_revision + 1;
    END IF;
    RETURN NEW;
END;
$$;

CREATE TRIGGER dashboards_metadata_authority_revision
BEFORE UPDATE ON dashboards
FOR EACH ROW EXECUTE FUNCTION advance_dashboard_metadata_authority_revision();

CREATE OR REPLACE FUNCTION enforce_dashboard_placement_capacity()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    placement_count INTEGER;
BEGIN
    IF TG_OP = 'UPDATE' AND NEW.dashboard_id = OLD.dashboard_id THEN
        RETURN NEW;
    END IF;

    PERFORM 1 FROM dashboards WHERE id = NEW.dashboard_id FOR UPDATE;
    SELECT COUNT(*) INTO placement_count
    FROM dashboard_placements
    WHERE dashboard_id = NEW.dashboard_id;

    IF placement_count >= 240 THEN
        RAISE EXCEPTION 'Dashboard % cannot exceed 240 placements', NEW.dashboard_id
            USING ERRCODE = '23514',
                  CONSTRAINT = 'dashboard_placements_capacity_chk';
    END IF;
    RETURN NEW;
END $$;

CREATE TRIGGER dashboard_placements_capacity_trigger
BEFORE INSERT OR UPDATE OF dashboard_id ON dashboard_placements
FOR EACH ROW EXECUTE FUNCTION enforce_dashboard_placement_capacity();

CREATE TABLE dashboard_mutation_replays (
    jti UUID PRIMARY KEY,
    original_actor_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (btrim(action) <> ''),
    payload_digest TEXT NOT NULL CHECK (payload_digest ~ '^sha256:[0-9a-f]{64}$'),
    idempotency_key TEXT NOT NULL UNIQUE CHECK (btrim(idempotency_key) <> ''),
    result JSONB NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE dashboard_dependency_observations (
    id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    placement_id UUID NOT NULL,
    saved_reference JSONB NOT NULL CHECK (jsonb_typeof(saved_reference) = 'object'),
    reference_digest TEXT NOT NULL CHECK (reference_digest ~ '^sha256:[0-9a-f]{64}$'),
    provider_contract_id TEXT NOT NULL CHECK (btrim(provider_contract_id) <> ''),
    provider_contract_version TEXT NOT NULL CHECK (btrim(provider_contract_version) <> ''),
    resource_revision BIGINT CHECK (resource_revision > 0),
    observation_fingerprint TEXT NOT NULL CHECK (observation_fingerprint ~ '^sha256:[0-9a-f]{64}$'),
    resolution JSONB NOT NULL CHECK (jsonb_typeof(resolution) = 'object'),
    provider_detail JSONB CHECK (provider_detail IS NULL OR jsonb_typeof(provider_detail) = 'object'),
    observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (placement_id, observation_fingerprint)
);

CREATE INDEX dashboard_dependency_observations_dashboard_observed_idx
    ON dashboard_dependency_observations (dashboard_id, observed_at DESC, id);

CREATE TABLE dashboard_dependency_findings (
    id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL,
    placement_id UUID NOT NULL,
    observation_id UUID NOT NULL
        REFERENCES dashboard_dependency_observations(id) ON DELETE RESTRICT,
    saved_reference JSONB NOT NULL CHECK (jsonb_typeof(saved_reference) = 'object'),
    reference_digest TEXT NOT NULL CHECK (reference_digest ~ '^sha256:[0-9a-f]{64}$'),
    observed_resource_revision BIGINT NOT NULL CHECK (observed_resource_revision >= 0),
    finding_code TEXT NOT NULL CHECK (btrim(finding_code) <> ''),
    impact JSONB NOT NULL CHECK (jsonb_typeof(impact) = 'object'),
    disposition TEXT NOT NULL DEFAULT 'open'
        CHECK (disposition IN ('open', 'deferred', 'resolved')),
    finding_revision BIGINT NOT NULL DEFAULT 1 CHECK (finding_revision > 0),
    deferred_by UUID,
    deferred_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (placement_id, reference_digest, observed_resource_revision, finding_code),
    CHECK (
        (disposition = 'open' AND deferred_by IS NULL AND deferred_at IS NULL AND resolved_at IS NULL)
        OR (disposition = 'deferred' AND deferred_by IS NOT NULL AND deferred_at IS NOT NULL AND resolved_at IS NULL)
        OR (disposition = 'resolved' AND resolved_at IS NOT NULL)
    )
);

CREATE INDEX dashboard_dependency_findings_dashboard_disposition_idx
    ON dashboard_dependency_findings (dashboard_id, disposition, updated_at DESC, id);

CREATE TABLE dashboard_dependency_action_receipts (
    idempotency_key TEXT PRIMARY KEY CHECK (btrim(idempotency_key) <> ''),
    request_digest TEXT NOT NULL CHECK (request_digest ~ '^sha256:[0-9a-f]{64}$'),
    dashboard_id UUID NOT NULL,
    placement_id UUID NOT NULL,
    finding_id UUID NOT NULL REFERENCES dashboard_dependency_findings(id) ON DELETE RESTRICT,
    actor_id UUID NOT NULL,
    action TEXT NOT NULL CHECK (action IN ('defer','upgrade','replace','remove')),
    expected_finding_revision BIGINT NOT NULL CHECK (expected_finding_revision > 0),
    result JSONB NOT NULL CHECK (jsonb_typeof(result) = 'object'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE OR REPLACE FUNCTION reject_dashboard_dependency_history_mutation()
RETURNS trigger LANGUAGE plpgsql AS $$
BEGIN
    RAISE EXCEPTION 'dashboard dependency history is immutable';
END;
$$;

CREATE TRIGGER dashboard_dependency_observations_immutable
BEFORE UPDATE OR DELETE ON dashboard_dependency_observations
FOR EACH ROW EXECUTE FUNCTION reject_dashboard_dependency_history_mutation();

CREATE TRIGGER dashboard_dependency_action_receipts_immutable
BEFORE UPDATE OR DELETE ON dashboard_dependency_action_receipts
FOR EACH ROW EXECUTE FUNCTION reject_dashboard_dependency_history_mutation();

CREATE TABLE dashboard_bootstrap_receipts (
    idempotency_key TEXT PRIMARY KEY CHECK (btrim(idempotency_key) <> ''),
    input_digest TEXT NOT NULL CHECK (input_digest ~ '^sha256:[0-9a-f]{64}$'),
    desired_revision BIGINT NOT NULL CHECK (desired_revision > 0),
    receipt JSONB NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
