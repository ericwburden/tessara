CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TYPE field_type AS ENUM ('text', 'number', 'boolean', 'date', 'single_choice', 'multi_choice');
CREATE TYPE form_version_status AS ENUM ('draft', 'published', 'superseded');
CREATE TYPE submission_status AS ENUM ('draft', 'submitted');
CREATE TYPE missing_data_policy AS ENUM ('null', 'exclude_row', 'bucket_unknown');

CREATE TABLE accounts (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    email text NOT NULL UNIQUE,
    display_name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE roles (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL UNIQUE,
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

CREATE TABLE account_role_assignments (
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    role_id uuid NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    PRIMARY KEY (account_id, role_id)
);

CREATE TABLE permission_grants (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    capability_id uuid NOT NULL REFERENCES capabilities(id) ON DELETE CASCADE,
    scope_kind text NOT NULL DEFAULT 'global',
    scope_id uuid,
    is_allowed boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE auth_sessions (
    token uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    account_id uuid NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE node_types (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
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

CREATE TABLE compatibility_groups (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_id uuid NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    name text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE form_versions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_id uuid NOT NULL REFERENCES forms(id) ON DELETE CASCADE,
    compatibility_group_id uuid REFERENCES compatibility_groups(id) ON DELETE SET NULL,
    version_label text NOT NULL,
    status form_version_status NOT NULL DEFAULT 'draft',
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (form_id, version_label)
);

CREATE TABLE form_sections (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE CASCADE,
    title text NOT NULL,
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
    UNIQUE (form_version_id, key)
);

CREATE TABLE choice_lists (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE CASCADE,
    name text NOT NULL
);

CREATE TABLE choice_list_items (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    choice_list_id uuid NOT NULL REFERENCES choice_lists(id) ON DELETE CASCADE,
    value text NOT NULL,
    label text NOT NULL,
    position integer NOT NULL DEFAULT 0
);

CREATE TABLE form_assignments (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    account_id uuid REFERENCES accounts(id) ON DELETE SET NULL,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE submissions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    assignment_id uuid NOT NULL REFERENCES form_assignments(id) ON DELETE RESTRICT,
    form_version_id uuid NOT NULL REFERENCES form_versions(id) ON DELETE RESTRICT,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE RESTRICT,
    status submission_status NOT NULL DEFAULT 'draft',
    submitted_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now()
);

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

CREATE TABLE reports (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    form_id uuid REFERENCES forms(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE report_field_bindings (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id uuid NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    logical_key text NOT NULL,
    source_field_key text NOT NULL,
    missing_policy missing_data_policy NOT NULL DEFAULT 'null',
    position integer NOT NULL DEFAULT 0
);

CREATE TABLE aggregations (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    report_id uuid NOT NULL REFERENCES reports(id) ON DELETE CASCADE,
    aggregation_kind text NOT NULL,
    field_logical_key text
);

CREATE TABLE aggregation_groupings (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregation_id uuid NOT NULL REFERENCES aggregations(id) ON DELETE CASCADE,
    field_logical_key text NOT NULL
);

CREATE TABLE charts (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    report_id uuid REFERENCES reports(id) ON DELETE SET NULL,
    chart_type text NOT NULL DEFAULT 'table'
);

CREATE TABLE dashboards (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL
);

CREATE TABLE dashboard_components (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dashboard_id uuid NOT NULL REFERENCES dashboards(id) ON DELETE CASCADE,
    chart_id uuid NOT NULL REFERENCES charts(id) ON DELETE CASCADE,
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
    submitted_at timestamptz
);

CREATE TABLE analytics.submission_value_fact (
    submission_id uuid NOT NULL,
    field_id uuid NOT NULL,
    field_key text NOT NULL,
    value_text text,
    value_json jsonb NOT NULL,
    PRIMARY KEY (submission_id, field_id)
);
