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
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE dashboard_scope_nodes (
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    node_id UUID NOT NULL REFERENCES dashboard_organization_nodes(node_id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (dashboard_id, node_id)
);

CREATE INDEX dashboard_scope_nodes_node_id_idx
    ON dashboard_scope_nodes (node_id, dashboard_id);

CREATE TABLE dashboard_placements (
    id UUID PRIMARY KEY,
    dashboard_id UUID NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
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
