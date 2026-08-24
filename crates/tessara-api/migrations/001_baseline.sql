-- Sprint 6B2 closeout baseline. This is the sole migration for a freshly seeded Core database.
-- Historical migrations 002-004 were intentionally squashed at closeout.

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";


CREATE TYPE field_type AS ENUM (
    'text',
    'number',
    'boolean',
    'date',
    'single_choice',
    'multi_choice',
    'static_text'
);
CREATE TYPE form_version_status AS ENUM ('draft', 'published', 'superseded');

CREATE TABLE accounts (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    email text NOT NULL UNIQUE,
    display_name text NOT NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE account_credentials (
    account_id uuid PRIMARY KEY REFERENCES accounts(id) ON DELETE CASCADE,
    password_hash text NOT NULL,
    password_scheme text NOT NULL,
    password_updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE roles (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL UNIQUE,
    description text NOT NULL DEFAULT '',
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE capabilities (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    key text NOT NULL UNIQUE,
    description text NOT NULL DEFAULT ''
);

CREATE TABLE role_capabilities (
    role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    capability_id uuid NOT NULL REFERENCES capabilities(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, capability_id)
);

CREATE TABLE auth_sessions (
    token uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    expires_at timestamptz NOT NULL DEFAULT (now() + interval '12 hours'),
    last_seen_at timestamptz NOT NULL DEFAULT now(),
    revoked_at timestamptz
);

CREATE INDEX auth_sessions_account_id_idx ON auth_sessions (account_id);
CREATE INDEX auth_sessions_active_lookup_idx ON auth_sessions (token, revoked_at, expires_at);

CREATE TABLE node_types (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    plural_label text,
    description text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE node_type_relationships (
    parent_node_type_id uuid NOT NULL REFERENCES node_types(id) ON DELETE CASCADE,
    child_node_type_id uuid NOT NULL REFERENCES node_types(id) ON DELETE CASCADE,
    PRIMARY KEY (parent_node_type_id, child_node_type_id)
);

CREATE TABLE node_metadata_field_definitions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_type_id uuid NOT NULL REFERENCES node_types(id) ON DELETE CASCADE,
    key text NOT NULL,
    label text NOT NULL,
    field_type field_type NOT NULL,
    required boolean NOT NULL DEFAULT false,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (node_type_id, key)
);

CREATE TABLE nodes (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    node_type_id uuid NOT NULL REFERENCES node_types(id) ON DELETE RESTRICT,
    parent_node_id uuid REFERENCES nodes(id) ON DELETE RESTRICT,
    name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE role_assignments (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    node_id uuid REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX role_assignments_global_unique
    ON role_assignments (account_id, role_id)
    WHERE node_id IS NULL;
CREATE UNIQUE INDEX role_assignments_scoped_unique
    ON role_assignments (account_id, role_id, node_id)
    WHERE node_id IS NOT NULL;

CREATE TABLE account_delegations (
    delegator_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    delegate_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (delegator_account_id, delegate_account_id),
    CHECK (delegator_account_id <> delegate_account_id)
);

CREATE TABLE node_metadata_values (
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    field_definition_id uuid NOT NULL REFERENCES node_metadata_field_definitions(id) ON DELETE CASCADE,
    value jsonb NOT NULL,
    PRIMARY KEY (node_id, field_definition_id)
);

CREATE TABLE forms (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    scope_node_type_id uuid REFERENCES node_types(id) ON DELETE RESTRICT,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE form_scope_nodes (
    form_id uuid NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (form_id, node_id)
);

CREATE INDEX form_scope_nodes_node_id_idx
    ON form_scope_nodes (node_id, form_id);

CREATE TABLE compatibility_groups (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_id uuid NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (form_id, name)
);

CREATE TABLE form_versions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_id uuid NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    compatibility_group_id uuid REFERENCES compatibility_groups(id) ON DELETE SET NULL,
    version_label text,
    status form_version_status NOT NULL DEFAULT 'draft',
    version_major integer,
    version_minor integer,
    version_patch integer,
    semantic_bump text,
    started_new_major_line boolean,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (form_id, version_label)
);

CREATE TABLE form_sections (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE CASCADE,
    title text NOT NULL,
    description text NOT NULL DEFAULT '',
    position integer NOT NULL DEFAULT 0
);

CREATE TABLE form_fields (
    field_id uuid NOT NULL DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE CASCADE,
    section_id uuid NOT NULL REFERENCES form_sections(id) ON DELETE CASCADE,
    key text NOT NULL,
    label text NOT NULL,
    field_type field_type NOT NULL,
    required boolean NOT NULL DEFAULT false,
    position integer NOT NULL DEFAULT 0,
    grid_row integer NOT NULL DEFAULT 1,
    grid_column integer NOT NULL DEFAULT 1,
    grid_width integer NOT NULL DEFAULT 1,
    grid_height integer NOT NULL DEFAULT 1,
    PRIMARY KEY (form_version_id, field_id),
    UNIQUE (form_version_id, key)
);

CREATE TABLE choice_lists (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE CASCADE,
    name text NOT NULL,
    import_key text,
    UNIQUE (form_version_id, name)
);

CREATE TABLE choice_list_items (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    choice_list_id uuid NOT NULL REFERENCES choice_lists(id) ON DELETE CASCADE,
    value text NOT NULL,
    label text NOT NULL,
    import_key text,
    position integer NOT NULL DEFAULT 0
);

CREATE TABLE workflows (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_node_type_id uuid NOT NULL REFERENCES node_types(id) ON DELETE RESTRICT,
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    description text NOT NULL DEFAULT '',
    source text NOT NULL DEFAULT 'authored',
    source_form_id uuid REFERENCES forms(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (source IN ('authored', 'generated_form'))
);

CREATE UNIQUE INDEX workflows_generated_form_source_idx
    ON workflows (source_form_id)
    WHERE source = 'generated_form' AND source_form_id IS NOT NULL;

CREATE TABLE workflow_available_nodes (
    workflow_id uuid NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (workflow_id, node_id)
);

CREATE TABLE workflow_versions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id uuid NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    version_label text,
    status form_version_status NOT NULL DEFAULT 'draft',
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE workflow_steps (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_version_id uuid NOT NULL REFERENCES workflow_versions(id) ON DELETE CASCADE,
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE RESTRICT,
    title text NOT NULL,
    position integer NOT NULL DEFAULT 0,
    UNIQUE (workflow_version_id, position)
);

CREATE TABLE workflow_assignments (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_version_id uuid NOT NULL REFERENCES workflow_versions(id) ON DELETE RESTRICT,
    workflow_step_id uuid NOT NULL REFERENCES workflow_steps(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    assigned_by_account_id uuid REFERENCES accounts(id) ON DELETE SET NULL,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (workflow_step_id, node_id, account_id)
);

CREATE TABLE workflow_instances (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_assignment_id uuid NOT NULL REFERENCES workflow_assignments(id) ON DELETE RESTRICT,
    workflow_version_id uuid NOT NULL REFERENCES workflow_versions(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    assignee_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    started_by_account_id uuid REFERENCES accounts(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'in_progress',
    created_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    CHECK (status IN ('in_progress', 'completed'))
);


CREATE TABLE workflow_step_instances (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_instance_id uuid NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
    workflow_step_id uuid NOT NULL REFERENCES workflow_steps(id) ON DELETE RESTRICT,
    status text NOT NULL DEFAULT 'in_progress',
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    CHECK (status IN ('in_progress', 'completed'))
);

CREATE TABLE workflow_response_reservations (
    workflow_assignment_id uuid NOT NULL REFERENCES workflow_assignments(id) ON DELETE CASCADE,
    workflow_instance_id uuid NOT NULL UNIQUE REFERENCES workflow_instances(id) ON DELETE CASCADE,
    workflow_step_instance_id uuid NOT NULL UNIQUE REFERENCES workflow_step_instances(id) ON DELETE CASCADE,
    started_by_account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    one_use_nonce uuid PRIMARY KEY,
    context_payload jsonb NOT NULL,
    context_digest text NOT NULL CHECK (context_digest ~ '^sha256:[0-9a-f]{64}$'),
    expires_at timestamptz NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    consumed_response_id uuid,
    consumed_at timestamptz,
    CHECK ((consumed_response_id IS NULL) = (consumed_at IS NULL))
);
CREATE UNIQUE INDEX workflow_response_reservations_unconsumed_assignment_idx
    ON workflow_response_reservations (workflow_assignment_id)
    WHERE consumed_at IS NULL;

CREATE TABLE workflow_response_projection (
    workflow_assignment_id uuid NOT NULL REFERENCES workflow_assignments(id) ON DELETE CASCADE,
    workflow_instance_id uuid NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
    workflow_step_instance_id uuid NOT NULL REFERENCES workflow_step_instances(id) ON DELETE CASCADE,
    response_installation_id uuid NOT NULL,
    response_module_instance_id uuid NOT NULL,
    response_id uuid PRIMARY KEY,
    response_revision bigint NOT NULL CHECK (response_revision > 0),
    response_state text NOT NULL CHECK (response_state IN ('draft', 'submitted', 'deleted')),
    last_event_sequence bigint NOT NULL CHECK (last_event_sequence >= 0),
    updated_at timestamptz NOT NULL,
    UNIQUE (response_installation_id, response_module_instance_id, response_id)
);
CREATE UNIQUE INDEX workflow_response_projection_active_assignment_idx
    ON workflow_response_projection (workflow_assignment_id)
    WHERE response_state IN ('draft', 'submitted');

CREATE TABLE workflow_response_event_consumer_state (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    provider_epoch uuid,
    committed_sequence bigint NOT NULL DEFAULT 0 CHECK (committed_sequence >= 0),
    synchronized_at timestamptz,
    last_error_at timestamptz
);
INSERT INTO workflow_response_event_consumer_state (singleton) VALUES (true);

CREATE TABLE workflow_response_consumed_events (
    event_id uuid PRIMARY KEY,
    sequence bigint NOT NULL UNIQUE CHECK (sequence > 0),
    content_digest text NOT NULL CHECK (content_digest ~ '^sha256:[0-9a-f]{64}$'),
    consumed_at timestamptz NOT NULL DEFAULT now()
);



CREATE INDEX workflow_versions_workflow_idx
    ON workflow_versions (workflow_id, status, created_at);
CREATE INDEX workflow_assignments_account_idx
    ON workflow_assignments (account_id, is_active, created_at);
CREATE INDEX workflow_assignments_workflow_idx
    ON workflow_assignments (workflow_version_id, is_active, created_at);
CREATE INDEX workflow_instances_assignment_idx
    ON workflow_instances (workflow_assignment_id, created_at);
CREATE INDEX workflow_instances_status_idx
    ON workflow_instances (status, created_at);
CREATE INDEX workflow_step_instances_instance_status_idx
    ON workflow_step_instances (workflow_instance_id, status);


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

-- Sprint 6A-UI: replace the effective reorder-band policy with generic groups
-- and one complete placement collection. Descriptor group/band columns remain
-- immutable catalog provenance and the legacy policy rows remain rollback data;
-- neither is an effective navigation source after this migration.

CREATE TABLE navigation_groups (
    installation_id uuid NOT NULL
        REFERENCES navigation_policies(installation_id) ON DELETE RESTRICT,
    group_id text NOT NULL,
    label text NOT NULL,
    display_order integer NOT NULL CHECK (display_order >= 0),
    owner text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, group_id),
    UNIQUE (installation_id, display_order),
    CONSTRAINT navigation_groups_owner_chk CHECK (owner IN ('core', 'custom')),
    CONSTRAINT navigation_groups_identity_chk CHECK (
        (owner = 'core' AND group_id IN ('core.main', 'core.admin'))
        OR
        (
            owner = 'custom'
            AND group_id ~ '^custom\.[0-9a-f]{8}-[0-9a-f]{4}-4[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$'
        )
    ),
    CONSTRAINT navigation_groups_label_chk CHECK (
        label = btrim(label)
        AND char_length(label) BETWEEN 1 AND 64
        AND label !~ '[[:cntrl:]]'
    )
);

CREATE UNIQUE INDEX navigation_groups_label_unique
    ON navigation_groups (installation_id, lower(label));

CREATE TABLE navigation_destination_placements (
    installation_id uuid NOT NULL
        REFERENCES navigation_policies(installation_id) ON DELETE RESTRICT,
    destination_id text NOT NULL,
    group_id text NOT NULL,
    visible boolean NOT NULL,
    display_order integer NOT NULL CHECK (display_order >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, destination_id),
    UNIQUE (installation_id, group_id, display_order),
    CONSTRAINT navigation_destination_placements_group_fk
        FOREIGN KEY (installation_id, group_id)
        REFERENCES navigation_groups(installation_id, group_id) ON DELETE RESTRICT,
    CONSTRAINT navigation_destination_placements_identity_chk CHECK (
        destination_id ~ '^[a-z0-9]+([.:_-][a-z0-9]+)*$'
        AND destination_id !~ '[.:_-]{2}'
    )
);

-- A fresh database has no installation at migration time; startup catalog
-- reconciliation seeds the same collection. A populated Sprint 6A database
-- receives the approved deterministic two-group layout atomically here.
INSERT INTO navigation_groups (installation_id, group_id, label, display_order, owner)
SELECT installation_id, 'core.main', 'Main', 0, 'core'
FROM navigation_policies
UNION ALL
SELECT installation_id, 'core.admin', 'Admin', 1, 'core'
FROM navigation_policies;

INSERT INTO navigation_destination_placements (
    installation_id,
    destination_id,
    group_id,
    visible,
    display_order
)
SELECT installation_id, destination_id, group_id, visible, display_order
FROM (
    SELECT installation_id, 'core.home'::text AS destination_id,
           'core.main'::text AS group_id, true AS visible, 0 AS display_order
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.organization', 'core.main', true, 1
    FROM navigation_policies
    UNION ALL
    SELECT entries.installation_id, entries.contribution_id, 'core.main', entries.visible,
           2 + entries.policy_order
    FROM navigation_policy_entries AS entries
    JOIN module_navigation_contributions AS contributions
      ON contributions.contribution_id = entries.contribution_id
    WHERE contributions.reorder_band = 'main_between_organization_and_operations'
    UNION ALL
    SELECT installation_id, 'core.operations', 'core.main', true, 5
    FROM navigation_policies
    UNION ALL
    SELECT entries.installation_id, entries.contribution_id, 'core.main', entries.visible,
           7 + entries.policy_order
    FROM navigation_policy_entries AS entries
    JOIN module_navigation_contributions AS contributions
      ON contributions.contribution_id = entries.contribution_id
    WHERE contributions.reorder_band = 'main_after_operations'
    UNION ALL
    SELECT installation_id, 'core.admin.users', 'core.admin', true, 0
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.admin.roles', 'core.admin', true, 1
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.admin.node_types', 'core.admin', true, 2
    FROM navigation_policies
    UNION ALL
    SELECT installation_id, 'core.admin.modules', 'core.admin', true, 3
    FROM navigation_policies
) AS approved_layout;

INSERT INTO core_control_plane_audit_events (
    installation_id,
    event_type,
    actor_kind,
    actor_account_id,
    correlation_id,
    payload
)
SELECT
    policies.installation_id,
    'navigation_policy_schema_migrated',
    'system',
    NULL,
    uuid_generate_v4(),
    jsonb_build_object(
        'from_schema_version', 1,
        'to_schema_version', 2,
        'old_policy_fingerprint', md5(COALESCE((
            SELECT string_agg(
                concat_ws(':', contribution_id, visible, policy_order),
                '|' ORDER BY contribution_id
            )
            FROM navigation_policy_entries
            WHERE installation_id = policies.installation_id
        ), '')),
        'new_policy_fingerprint', md5(COALESCE((
            SELECT string_agg(
                concat_ws(':', destination_id, group_id, visible, display_order),
                '|' ORDER BY destination_id
            )
            FROM navigation_destination_placements
            WHERE installation_id = policies.installation_id
        ), ''))
    )
FROM navigation_policies AS policies;

CREATE TABLE module_releases (
    id UUID PRIMARY KEY,
    definition_id TEXT NOT NULL REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    version TEXT NOT NULL,
    manifest_digest TEXT NOT NULL,
    manifest JSONB,
    runtime_image_digest TEXT NOT NULL,
    publisher TEXT NOT NULL,
    trust_state TEXT NOT NULL CHECK (trust_state IN ('unknown', 'curated', 'trusted', 'rejected')),
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
    database_name TEXT,
    configuration JSONB NOT NULL DEFAULT '{}'::JSONB,
    route_prefix TEXT,
    installed BOOLEAN NOT NULL,
    deployed BOOLEAN NOT NULL,
    configured BOOLEAN NOT NULL,
    ready BOOLEAN NOT NULL,
    enabled BOOLEAN NOT NULL,
    healthy BOOLEAN NOT NULL,
    last_observed_at TIMESTAMPTZ NOT NULL,
    UNIQUE (installation_id, definition_id)
);

CREATE TABLE module_service_identities (
    module_instance_id UUID PRIMARY KEY REFERENCES module_instances(id) ON DELETE CASCADE,
    module_definition_id TEXT NOT NULL REFERENCES module_definition_reservations(definition_id) ON DELETE RESTRICT,
    key_id TEXT NOT NULL,
    public_key TEXT NOT NULL,
    public_key_fingerprint TEXT NOT NULL CHECK (public_key_fingerprint ~ '^sha256:[0-9a-f]{64}$'),
    registered_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (module_instance_id, key_id)
);

CREATE TABLE consumed_module_service_nonces (
    module_instance_id UUID NOT NULL REFERENCES module_instances(id) ON DELETE CASCADE,
    nonce UUID NOT NULL,
    authorization_jti UUID UNIQUE,
    correlation_id TEXT NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (module_instance_id, nonce)
);

-- Materialization-time service requests precede final Module Instance
-- enrollment. Their apply-bound authorization is replay protected without
-- creating provisional release or instance rows.
CREATE TABLE consumed_bootstrap_validation_authorizations (
    authorization_jti UUID PRIMARY KEY,
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE CASCADE,
    module_instance_id UUID NOT NULL,
    service_nonce UUID NOT NULL,
    correlation_id UUID NOT NULL,
    issued_at TIMESTAMPTZ NOT NULL,
    consumed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (module_instance_id, service_nonce)
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

-- Sprint 6B2 secure module operation state.

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

-- Every capability declared by an accepted independent-module manifest must
-- be available to Core's role editor.

INSERT INTO capabilities (key, description, scope_mode)
SELECT DISTINCT ON (declaration ->> 'id')
    declaration ->> 'id',
    declaration ->> 'description',
    'scope_aware'
FROM module_releases
CROSS JOIN LATERAL jsonb_array_elements(
    COALESCE(manifest -> 'security_capabilities', '[]'::jsonb)
) AS declaration
WHERE manifest IS NOT NULL
  AND btrim(declaration ->> 'id') <> ''
  AND btrim(declaration ->> 'description') <> ''
ORDER BY declaration ->> 'id', module_releases.created_at DESC
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    scope_mode = EXCLUDED.scope_mode;

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

-- Sprint 6F Core projection of desired and observed application composition.
-- The Supervisor remains authoritative for execution and receipts; these
-- tables support planning, explicit approval, UI read-back, and drift review.

INSERT INTO capabilities (key, description, scope_mode)
VALUES
    ('composition:read', 'Inspect application composition and receipts', 'installation_global'),
    ('composition:plan', 'Create and resolve application Blueprint revisions', 'installation_global'),
    ('composition:approve', 'Approve and apply composition plans or emergency disables', 'installation_global')
ON CONFLICT (key) DO UPDATE SET
    description = EXCLUDED.description,
    scope_mode = EXCLUDED.scope_mode;

CREATE TABLE composition_blueprints (
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    revision BIGINT NOT NULL CHECK (revision > 0),
    digest TEXT NOT NULL CHECK (digest ~ '^sha256:[0-9a-f]{64}$'),
    document JSONB NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('draft', 'resolved', 'approved', 'superseded')),
    created_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, revision),
    UNIQUE (installation_id, digest)
);

CREATE TABLE composition_lockfiles (
    installation_id UUID NOT NULL,
    blueprint_revision BIGINT NOT NULL,
    lockfile_digest TEXT NOT NULL CHECK (lockfile_digest ~ '^sha256:[0-9a-f]{64}$'),
    plan_digest TEXT NOT NULL CHECK (plan_digest ~ '^sha256:[0-9a-f]{64}$'),
    catalog_digest TEXT NOT NULL CHECK (catalog_digest ~ '^sha256:[0-9a-f]{64}$'),
    document JSONB NOT NULL,
    resolved_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    resolved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, blueprint_revision),
    UNIQUE (installation_id, lockfile_digest),
    FOREIGN KEY (installation_id, blueprint_revision)
        REFERENCES composition_blueprints(installation_id, revision) ON DELETE RESTRICT
);

CREATE TABLE composition_approvals (
    installation_id UUID NOT NULL,
    blueprint_revision BIGINT NOT NULL,
    lockfile_digest TEXT NOT NULL,
    plan_digest TEXT NOT NULL,
    approved_effects JSONB NOT NULL,
    reason TEXT,
    approved_by UUID NOT NULL REFERENCES accounts(id) ON DELETE RESTRICT,
    approved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, blueprint_revision),
    FOREIGN KEY (installation_id, blueprint_revision)
        REFERENCES composition_lockfiles(installation_id, blueprint_revision) ON DELETE RESTRICT
);

CREATE TABLE composition_operation_projections (
    operation_id UUID PRIMARY KEY,
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    blueprint_revision BIGINT NOT NULL,
    state TEXT NOT NULL,
    operation JSONB NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE composition_receipt_projections (
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    revision BIGINT NOT NULL CHECK (revision > 0),
    digest TEXT NOT NULL CHECK (digest ~ '^sha256:[0-9a-f]{64}$'),
    lockfile JSONB NOT NULL,
    receipt JSONB NOT NULL,
    observed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, revision),
    UNIQUE (installation_id, digest)
);

CREATE TABLE composition_drift_findings (
    finding_id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    installation_id UUID NOT NULL REFERENCES application_installations(id) ON DELETE RESTRICT,
    code TEXT NOT NULL,
    path TEXT NOT NULL,
    desired JSONB,
    observed JSONB,
    disposition TEXT NOT NULL DEFAULT 'open' CHECK (disposition IN ('open', 'adopted', 'reconciled')),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    resolved_at TIMESTAMPTZ
);

CREATE TABLE core_bootstrap_receipts (
    idempotency_key TEXT PRIMARY KEY CHECK (btrim(idempotency_key) <> ''),
    locked_input_digest TEXT NOT NULL CHECK (locked_input_digest ~ '^sha256:[0-9a-f]{64}$'),
    input_digest TEXT NOT NULL CHECK (input_digest ~ '^sha256:[0-9a-f]{64}$'),
    desired_revision BIGINT NOT NULL CHECK (desired_revision > 0),
    apply_sequence BIGINT NOT NULL CHECK (apply_sequence > 0),
    authority_jti UUID NOT NULL UNIQUE,
    receipt JSONB NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT now()
);
