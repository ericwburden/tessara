-- Sprint 6A adds Core-owned discovery and policy state for the current
-- in-process transition catalog. Module Release and Module Instance
-- persistence deliberately begins in Sprint 6B, so neither table appears in
-- this migration.

ALTER TABLE capabilities
    ADD COLUMN scope_mode text NOT NULL DEFAULT 'scope_aware';

ALTER TABLE capabilities
    ADD CONSTRAINT capabilities_scope_mode_chk
    CHECK (scope_mode IN ('scope_aware', 'installation_global'));

INSERT INTO capabilities (key, description, scope_mode)
VALUES ('admin:all', 'Full administration access', 'installation_global')
ON CONFLICT (key) DO UPDATE SET
    scope_mode = EXCLUDED.scope_mode
WHERE capabilities.scope_mode IS DISTINCT FROM EXCLUDED.scope_mode;

CREATE TABLE application_installations (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    id uuid NOT NULL UNIQUE DEFAULT uuid_generate_v4(),
    created_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO application_installations (singleton)
VALUES (true)
ON CONFLICT (singleton) DO NOTHING;

CREATE TABLE core_runtime_observations (
    installation_id uuid PRIMARY KEY
        REFERENCES application_installations(id) ON DELETE RESTRICT,
    provenance text NOT NULL,
    observed_version text NOT NULL,
    finding_code text NOT NULL,
    observed_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT core_runtime_observations_provenance_chk
        CHECK (provenance = 'development_unresolved'),
    CONSTRAINT core_runtime_observations_finding_chk
        CHECK (finding_code = 'core_release_provenance_unresolved'),
    CONSTRAINT core_runtime_observations_version_chk
        CHECK (btrim(observed_version) <> '')
);

CREATE TABLE module_definition_reservations (
    definition_id text PRIMARY KEY,
    display_name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT module_definition_reservations_id_chk
        CHECK (
            definition_id ~ '^[a-z0-9]+([.:_-][a-z0-9]+)*$'
            AND definition_id !~ '[.:_-]{2}'
        ),
    CONSTRAINT module_definition_reservations_display_name_chk
        CHECK (btrim(display_name) <> '')
);

CREATE TABLE transition_descriptor_sources (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    definition_id text NOT NULL
        REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    schema_version integer NOT NULL,
    source_digest text NOT NULL,
    source_bytes bytea NOT NULL,
    content_type text NOT NULL DEFAULT 'application/json',
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (definition_id, source_digest),
    UNIQUE (id, definition_id),
    CONSTRAINT transition_descriptor_sources_schema_chk
        CHECK (schema_version = 1),
    CONSTRAINT transition_descriptor_sources_digest_chk
        CHECK (source_digest ~ '^sha256:[0-9a-f]{64}$'),
    CONSTRAINT transition_descriptor_sources_bytes_chk
        CHECK (octet_length(source_bytes) > 0),
    CONSTRAINT transition_descriptor_sources_content_type_chk
        CHECK (content_type = 'application/json')
);

CREATE TABLE transition_catalog_projections (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_id uuid NOT NULL UNIQUE
        REFERENCES transition_descriptor_sources(id) ON DELETE RESTRICT,
    installation_id uuid NOT NULL
        REFERENCES application_installations(id) ON DELETE RESTRICT,
    normalized_projection jsonb NOT NULL,
    provider_eligible boolean NOT NULL DEFAULT false CHECK (NOT provider_eligible),
    supervisor_materializable boolean NOT NULL DEFAULT false
        CHECK (NOT supervisor_materializable),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (id, source_id),
    CONSTRAINT transition_catalog_projections_shape_chk
        CHECK (
            jsonb_typeof(normalized_projection) = 'object'
            AND normalized_projection ->> 'kind' = 'transitional_in_process'
        )
);

CREATE TABLE transition_catalog_current (
    definition_id text PRIMARY KEY
        REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    source_id uuid NOT NULL UNIQUE,
    projection_id uuid NOT NULL UNIQUE,
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT transition_catalog_current_source_definition_fk
        FOREIGN KEY (source_id, definition_id)
        REFERENCES transition_descriptor_sources(id, definition_id) ON DELETE RESTRICT,
    CONSTRAINT transition_catalog_current_projection_source_fk
        FOREIGN KEY (projection_id, source_id)
        REFERENCES transition_catalog_projections(id, source_id) ON DELETE RESTRICT
);

CREATE TABLE module_catalog_findings (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    projection_id uuid NOT NULL
        REFERENCES transition_catalog_projections(id) ON DELETE RESTRICT,
    ordinal integer NOT NULL CHECK (ordinal >= 0),
    code text NOT NULL,
    path text NOT NULL,
    message text NOT NULL,
    UNIQUE (projection_id, ordinal),
    CONSTRAINT module_catalog_findings_code_chk CHECK (btrim(code) <> ''),
    CONSTRAINT module_catalog_findings_path_chk CHECK (btrim(path) <> ''),
    CONSTRAINT module_catalog_findings_message_chk CHECK (btrim(message) <> '')
);

CREATE TABLE capability_provenance (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    capability_id uuid NOT NULL
        REFERENCES capabilities(id) ON DELETE RESTRICT,
    source_kind text NOT NULL,
    source_key text NOT NULL,
    definition_id text
        REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    descriptor_source_id uuid,
    provider_state text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (capability_id, source_key),
    CONSTRAINT capability_provenance_source_kind_chk
        CHECK (source_kind IN ('core', 'transition_contribution')),
    CONSTRAINT capability_provenance_source_key_chk
        CHECK (btrim(source_key) <> ''),
    CONSTRAINT capability_provenance_provider_state_chk
        CHECK (provider_state IN ('core_authoritative', 'transitional_in_process')),
    CONSTRAINT capability_provenance_source_definition_fk
        FOREIGN KEY (descriptor_source_id, definition_id)
        REFERENCES transition_descriptor_sources(id, definition_id) ON DELETE RESTRICT,
    CONSTRAINT capability_provenance_shape_chk
        CHECK (
            (
                source_kind = 'core'
                AND source_key = 'core'
                AND definition_id IS NULL
                AND descriptor_source_id IS NULL
                AND provider_state = 'core_authoritative'
            )
            OR
            (
                source_kind = 'transition_contribution'
                AND definition_id IS NOT NULL
                AND source_key = definition_id
                AND descriptor_source_id IS NOT NULL
                AND provider_state = 'transitional_in_process'
            )
        )
);

CREATE TABLE module_navigation_contributions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    contribution_id text NOT NULL UNIQUE,
    definition_id text NOT NULL
        REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    descriptor_source_id uuid NOT NULL,
    destination text NOT NULL,
    label text NOT NULL,
    group_name text NOT NULL,
    reorder_band text NOT NULL,
    source_order_hint integer NOT NULL,
    default_policy_order integer NOT NULL CHECK (default_policy_order >= 0),
    required_capabilities_any_of jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT module_navigation_contributions_id_chk
        CHECK (
            contribution_id ~ '^[a-z0-9]+([.:_-][a-z0-9]+)*$'
            AND contribution_id !~ '[.:_-]{2}'
        ),
    CONSTRAINT module_navigation_contributions_destination_chk
        CHECK (
            destination ~ '^[a-z0-9]+([.:_-][a-z0-9]+)*$'
            AND destination !~ '[.:_-]{2}'
        ),
    CONSTRAINT module_navigation_contributions_label_chk CHECK (btrim(label) <> ''),
    CONSTRAINT module_navigation_contributions_group_chk
        CHECK (group_name IN ('Main', 'Admin')),
    CONSTRAINT module_navigation_contributions_band_chk
        CHECK (
            reorder_band IN (
                'main_between_organization_and_operations',
                'main_after_operations',
                'admin_between_administration_and_module_management'
            )
        ),
    CONSTRAINT module_navigation_contributions_capabilities_chk
        CHECK (
            jsonb_typeof(required_capabilities_any_of) = 'array'
            AND jsonb_array_length(required_capabilities_any_of) > 0
        ),
    CONSTRAINT module_navigation_contributions_source_definition_fk
        FOREIGN KEY (descriptor_source_id, definition_id)
        REFERENCES transition_descriptor_sources(id, definition_id) ON DELETE RESTRICT
);

CREATE TABLE navigation_policies (
    installation_id uuid PRIMARY KEY
        REFERENCES application_installations(id) ON DELETE RESTRICT,
    revision bigint NOT NULL DEFAULT 0 CHECK (revision >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE navigation_policy_entries (
    installation_id uuid NOT NULL
        REFERENCES navigation_policies(installation_id) ON DELETE RESTRICT,
    contribution_id text NOT NULL
        REFERENCES module_navigation_contributions(contribution_id) ON DELETE RESTRICT,
    visible boolean NOT NULL DEFAULT true,
    policy_order integer NOT NULL CHECK (policy_order >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, contribution_id)
);

CREATE TABLE core_control_plane_audit_events (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    installation_id uuid
        REFERENCES application_installations(id) ON DELETE RESTRICT,
    event_type text NOT NULL,
    actor_kind text NOT NULL,
    actor_account_id uuid REFERENCES accounts(id) ON DELETE RESTRICT,
    correlation_id uuid NOT NULL,
    payload jsonb NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT core_control_plane_audit_events_type_chk CHECK (btrim(event_type) <> ''),
    CONSTRAINT core_control_plane_audit_events_actor_kind_chk
        CHECK (actor_kind IN ('system', 'account')),
    CONSTRAINT core_control_plane_audit_events_actor_chk
        CHECK (
            (actor_kind = 'system' AND actor_account_id IS NULL)
            OR (actor_kind = 'account' AND actor_account_id IS NOT NULL)
        ),
    CONSTRAINT core_control_plane_audit_events_payload_chk
        CHECK (jsonb_typeof(payload) = 'object')
);

CREATE INDEX transition_descriptor_sources_definition_created_idx
    ON transition_descriptor_sources (definition_id, created_at, id);
CREATE INDEX module_catalog_findings_projection_ordinal_idx
    ON module_catalog_findings (projection_id, ordinal);
CREATE INDEX capability_provenance_definition_idx
    ON capability_provenance (definition_id, capability_id);
CREATE INDEX module_navigation_contributions_group_band_idx
    ON module_navigation_contributions (group_name, reorder_band, default_policy_order, contribution_id);
CREATE INDEX core_control_plane_audit_events_type_created_idx
    ON core_control_plane_audit_events (event_type, created_at, id);
