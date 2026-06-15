CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE SCHEMA IF NOT EXISTS dataset_materialized;

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
CREATE TYPE submission_status AS ENUM ('draft', 'submitted');
CREATE TYPE dataset_revision_status AS ENUM ('draft', 'published', 'superseded');
CREATE TYPE component_type AS ENUM (
    'detail_table',
    'aggregate_table',
    'bar',
    'line',
    'pie',
    'donut',
    'stat_card'
);
CREATE TYPE component_version_status AS ENUM ('draft', 'published', 'superseded');

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
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
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

CREATE TABLE submissions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    workflow_assignment_id uuid NOT NULL REFERENCES workflow_assignments(id) ON DELETE RESTRICT,
    workflow_instance_id uuid REFERENCES workflow_instances(id) ON DELETE SET NULL,
    status submission_status NOT NULL DEFAULT 'draft',
    submitted_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE workflow_step_instances (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_instance_id uuid NOT NULL REFERENCES workflow_instances(id) ON DELETE CASCADE,
    workflow_step_id uuid NOT NULL REFERENCES workflow_steps(id) ON DELETE RESTRICT,
    submission_id uuid UNIQUE REFERENCES submissions(id) ON DELETE SET NULL,
    status text NOT NULL DEFAULT 'in_progress',
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    CHECK (status IN ('in_progress', 'completed'))
);

ALTER TABLE submissions
    ADD COLUMN workflow_step_instance_id uuid REFERENCES workflow_step_instances(id) ON DELETE SET NULL;

CREATE TABLE submission_values (
    submission_id uuid NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    field_id uuid NOT NULL REFERENCES form_fields(id) ON DELETE RESTRICT,
    value jsonb NOT NULL,
    PRIMARY KEY (submission_id, field_id)
);

CREATE TABLE submission_value_multi (
    submission_id uuid NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    field_id uuid NOT NULL REFERENCES form_fields(id) ON DELETE RESTRICT,
    value text NOT NULL,
    PRIMARY KEY (submission_id, field_id, value)
);

CREATE TABLE submission_audit_events (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    submission_id uuid NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    event_type text NOT NULL,
    account_id uuid REFERENCES accounts(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE datasets (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    grain text NOT NULL,
    composition_mode text NOT NULL DEFAULT 'union',
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (grain IN ('submission', 'node')),
    CHECK (composition_mode IN ('union', 'union_all', 'left_join', 'inner_join', 'outer_join'))
);

CREATE TABLE dataset_scope_nodes (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dataset_id, node_id)
);

CREATE INDEX dataset_scope_nodes_node_id_idx
    ON dataset_scope_nodes (node_id, dataset_id);

CREATE TABLE dataset_revisions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_number integer NOT NULL,
    version_label text NOT NULL,
    status dataset_revision_status NOT NULL DEFAULT 'draft',
    definition_ast jsonb,
    aggregation jsonb,
    generated_sql text,
    materialized_schema text,
    materialized_table text,
    materialized_row_count bigint,
    materialized_at timestamptz,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, version_number)
);

CREATE UNIQUE INDEX dataset_revisions_one_published_idx
    ON dataset_revisions (dataset_id)
    WHERE status = 'published';
CREATE INDEX dataset_revisions_materialized_table_idx
    ON dataset_revisions (materialized_schema, materialized_table);

CREATE TABLE dataset_sources (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    source_alias text NOT NULL,
    form_id uuid REFERENCES forms(id) ON DELETE CASCADE,
    dataset_revision_id uuid REFERENCES dataset_revisions(id) ON DELETE RESTRICT,
    form_version_major integer,
    selection_rule text NOT NULL DEFAULT 'all',
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, source_alias),
    CHECK (selection_rule IN ('all', 'latest', 'earliest')),
    CHECK (
        ((form_id IS NOT NULL)::integer
        + (dataset_revision_id IS NOT NULL)::integer) = 1
    )
);

CREATE TABLE dataset_fields (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    key text NOT NULL,
    label text NOT NULL,
    source_alias text NOT NULL,
    source_field_key text NOT NULL,
    field_type field_type NOT NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, key),
    FOREIGN KEY (dataset_id, source_alias)
        REFERENCES dataset_sources(dataset_id, source_alias)
        ON DELETE CASCADE
);

CREATE INDEX dataset_sources_dataset_id_position_idx
    ON dataset_sources (dataset_id, position, source_alias);
CREATE INDEX dataset_fields_dataset_id_position_idx
    ON dataset_fields (dataset_id, position, key);

CREATE TABLE components (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    description text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE component_versions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    component_id uuid NOT NULL REFERENCES components(id) ON DELETE CASCADE,
    dataset_revision_id uuid NOT NULL REFERENCES dataset_revisions(id) ON DELETE RESTRICT,
    component_type component_type NOT NULL,
    version_number integer NOT NULL,
    version_label text NOT NULL,
    status component_version_status NOT NULL DEFAULT 'draft',
    config jsonb NOT NULL DEFAULT '{}'::jsonb,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (component_id, version_number)
);

CREATE UNIQUE INDEX component_versions_one_published_idx
    ON component_versions (component_id)
    WHERE status = 'published';

CREATE TABLE dashboards (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    description text,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE dashboard_scope_nodes (
    dashboard_id uuid NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dashboard_id, node_id)
);

CREATE INDEX dashboard_scope_nodes_node_id_idx
    ON dashboard_scope_nodes (node_id, dashboard_id);

CREATE TABLE dashboard_components (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dashboard_id uuid NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    component_version_id uuid NOT NULL REFERENCES component_versions(id) ON DELETE RESTRICT,
    position integer NOT NULL DEFAULT 0,
    config jsonb NOT NULL DEFAULT '{}'::jsonb
);

CREATE SCHEMA IF NOT EXISTS analytics;

CREATE TABLE analytics.node_dim (
    node_id uuid PRIMARY KEY,
    node_name text NOT NULL,
    node_type_id uuid NOT NULL,
    parent_node_id uuid
);

CREATE TABLE analytics.form_dim (
    form_id uuid PRIMARY KEY,
    form_name text NOT NULL,
    form_slug text NOT NULL
);

CREATE TABLE analytics.form_version_dim (
    form_version_id uuid PRIMARY KEY,
    form_id uuid NOT NULL,
    version_label text NOT NULL,
    compatibility_group_id uuid
);

CREATE TABLE analytics.field_dim (
    field_id uuid PRIMARY KEY,
    form_version_id uuid NOT NULL,
    field_key text NOT NULL,
    field_label text NOT NULL,
    field_type text NOT NULL
);

CREATE TABLE analytics.compatibility_group_dim (
    compatibility_group_id uuid PRIMARY KEY,
    form_id uuid NOT NULL,
    name text NOT NULL
);

CREATE TABLE analytics.submission_fact (
    submission_id uuid PRIMARY KEY,
    form_version_id uuid NOT NULL,
    node_id uuid NOT NULL,
    status text NOT NULL,
    submitted_at timestamptz,
    created_at timestamptz,
    last_modified_at timestamptz,
    last_modified_by_user_name text
);

CREATE TABLE analytics.submission_value_fact (
    submission_id uuid NOT NULL,
    field_id uuid NOT NULL,
    field_key text NOT NULL,
    value_text text,
    value_json jsonb NOT NULL,
    PRIMARY KEY (submission_id, field_id)
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
