-- Sprint 6B2 closeout baseline. This is the sole migration for a freshly seeded Core database.
-- Historical migrations 002-004 were intentionally squashed at closeout.

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
CREATE TYPE component_type AS ENUM ('table', 'bar', 'line', 'pie', 'donut', 'stat_card');
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
    form_version_id uuid NOT NULL,
    field_id uuid NOT NULL,
    value jsonb NOT NULL,
    PRIMARY KEY (submission_id, field_id),
    FOREIGN KEY (form_version_id, field_id) REFERENCES form_fields(form_version_id, field_id) ON DELETE RESTRICT
);

CREATE TABLE submission_value_multi (
    submission_id uuid NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    form_version_id uuid NOT NULL,
    field_id uuid NOT NULL,
    value text NOT NULL,
    PRIMARY KEY (submission_id, field_id, value),
    FOREIGN KEY (form_version_id, field_id) REFERENCES form_fields(form_version_id, field_id) ON DELETE RESTRICT
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
    created_at timestamptz NOT NULL DEFAULT now(),
    CHECK (grain IN ('submission', 'node'))
);

CREATE TABLE dataset_scope_nodes (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    node_id uuid NOT NULL REFERENCES nodes(id) ON DELETE CASCADE,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dataset_id, node_id)
);

CREATE INDEX dataset_scope_nodes_node_id_idx
    ON dataset_scope_nodes (node_id, dataset_id);

CREATE TABLE dataset_tags (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    tag text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dataset_id, tag),
    CHECK (btrim(tag) <> '')
);

CREATE INDEX dataset_tags_tag_idx
    ON dataset_tags (lower(tag), dataset_id);

CREATE TABLE dataset_revisions (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_number integer NOT NULL,
    version_label text NOT NULL,
    version_major integer,
    version_minor integer,
    version_patch integer,
    semantic_bump text,
    started_new_major_line boolean,
    force_new_major_version boolean NOT NULL DEFAULT false,
    revision_notes text NOT NULL DEFAULT '',
    status dataset_revision_status NOT NULL DEFAULT 'draft',
    initial_source jsonb,
    operations jsonb,
    restriction_policy jsonb,
    definition_metadata jsonb,
    compatibility_findings jsonb NOT NULL DEFAULT '[]'::jsonb,
    generated_sql text,
    output_fields jsonb,
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
CREATE UNIQUE INDEX dataset_revisions_one_draft_idx
    ON dataset_revisions (dataset_id)
    WHERE status = 'draft';
CREATE INDEX dataset_revisions_materialized_table_idx
    ON dataset_revisions (materialized_schema, materialized_table);
CREATE INDEX dataset_revisions_semantic_version_idx
    ON dataset_revisions (dataset_id, version_major, version_minor, version_patch);

CREATE TABLE dataset_major_materializations (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_major integer NOT NULL,
    materialized_schema text,
    materialized_table text,
    materialized_row_count bigint,
    materialized_at timestamptz,
    rebuild_status text NOT NULL DEFAULT 'pending',
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dataset_id, version_major)
);

CREATE INDEX dataset_major_materializations_table_idx
    ON dataset_major_materializations (materialized_schema, materialized_table);

CREATE TABLE dataset_sources (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    source_alias text NOT NULL,
    form_id uuid REFERENCES forms(id) ON DELETE CASCADE,
    form_version_id uuid REFERENCES form_versions(id) ON DELETE RESTRICT,
    source_dataset_id uuid REFERENCES datasets(id) ON DELETE RESTRICT,
    dataset_revision_id uuid REFERENCES dataset_revisions(id) ON DELETE RESTRICT,
    dataset_version_major integer,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, source_alias),
    CHECK (
        (
            form_id IS NOT NULL
            AND form_version_id IS NOT NULL
            AND source_dataset_id IS NULL
            AND dataset_revision_id IS NULL
            AND dataset_version_major IS NULL
        )
        OR (
            form_id IS NULL
            AND form_version_id IS NULL
            AND source_dataset_id IS NOT NULL
            AND dataset_revision_id IS NOT NULL
            AND dataset_version_major IS NULL
        )
        OR (
            form_id IS NULL
            AND form_version_id IS NULL
            AND source_dataset_id IS NOT NULL
            AND dataset_revision_id IS NULL
            AND dataset_version_major IS NOT NULL
        )
    )
);

CREATE TABLE dataset_fields (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    key text NOT NULL,
    label text NOT NULL,
    source_alias text NOT NULL,
    source_field_key text NOT NULL,
    source_field_id uuid,
    field_type field_type NOT NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, key)
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
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE RESTRICT,
    dataset_version_major integer NOT NULL,
    binding_mode text NOT NULL DEFAULT 'major_line' CHECK (binding_mode = 'major_line'),
    component_type component_type NOT NULL,
    version_number integer NOT NULL,
    version_label text NOT NULL,
    version_note text NOT NULL DEFAULT '',
    status component_version_status NOT NULL DEFAULT 'draft',
    config jsonb NOT NULL DEFAULT '{}'::jsonb,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (component_id, version_number),
    CONSTRAINT component_versions_component_type_supported_chk CHECK (
        component_type IN (
            'table'::component_type,
            'bar'::component_type,
            'line'::component_type,
            'pie'::component_type,
            'donut'::component_type,
            'stat_card'::component_type
        )
    )
);

CREATE UNIQUE INDEX component_versions_one_published_idx
    ON component_versions (component_id)
    WHERE status = 'published';
CREATE UNIQUE INDEX component_versions_one_draft_idx
    ON component_versions (component_id)
    WHERE status = 'draft';
CREATE INDEX component_versions_dataset_major_idx
    ON component_versions (dataset_id, dataset_version_major);

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
    form_version_id uuid NOT NULL,
    field_id uuid NOT NULL,
    field_key text NOT NULL,
    field_label text NOT NULL,
    field_type text NOT NULL,
    PRIMARY KEY (form_version_id, field_id)
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
    form_version_id uuid NOT NULL,
    field_id uuid NOT NULL,
    field_key text NOT NULL,
    value_text text,
    value_json jsonb NOT NULL,
    PRIMARY KEY (submission_id, field_id)
);

CREATE INDEX analytics_submission_fact_form_version_idx
    ON analytics.submission_fact (form_version_id);
CREATE INDEX analytics_submission_value_fact_form_version_field_idx
    ON analytics.submission_value_fact (form_version_id, field_id, submission_id);

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

-- Sprint 5A establishes a hard per-dashboard placement capacity before the
-- application begins relying on bounded grid fallback and viewer execution.

-- Close the preflight-to-trigger race with old application writers. This lock
-- conflicts with INSERT/UPDATE/DELETE and is held by the migration transaction
-- through trigger installation.
LOCK TABLE dashboard_components IN SHARE ROW EXCLUSIVE MODE;

-- Validity includes the current Component kind because kind-specific minimums
-- can turn previously valid placement geometry into fallback state. Prevent a
-- current-published update-in-place from changing that classifier between the
-- preflight snapshot and migration commit.
LOCK TABLE component_versions IN SHARE MODE;

DO $$
DECLARE
    violations text;
BEGIN
    SELECT string_agg(
        format('%s (%s placements)', dashboard_id, placement_count),
        ', '
        ORDER BY dashboard_id
    )
    INTO violations
    FROM (
        SELECT dashboard_id, COUNT(*) AS placement_count
        FROM dashboard_components
        GROUP BY dashboard_id
        HAVING COUNT(*) > 240
    ) AS over_capacity;

    IF violations IS NOT NULL THEN
        RAISE EXCEPTION USING
            ERRCODE = '23514',
            MESSAGE = 'dashboard placement capacity preflight failed: ' || violations,
            HINT = 'Back up/export affected dashboards, reduce each to at most 240 placements, and rerun the migration. See docs/sprints/sprint-5a-dashboard-capacity-runbook.md.';
    END IF;
END
$$;

-- The runtime gives every legacy, malformed, or future-schema placement a
-- full-width fallback row which must avoid every row touched by valid V1
-- geometry. Count alone is therefore insufficient: a mixed dashboard can be
-- below 240 stored rows while having no legal display fallback. Decode only
-- bounded JSON integers so malformed payloads remain data, not migration
-- errors.
CREATE FUNCTION pg_temp.dashboard_json_i32(value jsonb)
RETURNS integer
LANGUAGE plpgsql
IMMUTABLE
STRICT
AS $$
DECLARE
    decoded text;
    numeric_value numeric;
BEGIN
    IF jsonb_typeof(value) <> 'number' THEN
        RETURN NULL;
    END IF;

    decoded := value #>> '{}';
    IF decoded !~ '^-?[0-9]+$' THEN
        RETURN NULL;
    END IF;

    numeric_value := decoded::numeric;
    IF numeric_value < -2147483648 OR numeric_value > 2147483647 THEN
        RETURN NULL;
    END IF;
    RETURN numeric_value::integer;
END
$$;

CREATE TEMPORARY TABLE dashboard_placement_layout_preflight
ON COMMIT DROP
AS
WITH decoded AS (
    SELECT dashboard_components.dashboard_id,
           dashboard_components.id AS placement_id,
           dashboard_components.config,
           component_versions.component_type::text AS component_type,
           pg_temp.dashboard_json_i32(
               dashboard_components.config -> 'schema_version'
           ) AS schema_version,
           pg_temp.dashboard_json_i32(
               dashboard_components.config -> 'grid_row'
           ) AS grid_row,
           pg_temp.dashboard_json_i32(
               dashboard_components.config -> 'grid_column'
           ) AS grid_column,
           pg_temp.dashboard_json_i32(
               dashboard_components.config -> 'grid_width'
           ) AS grid_width,
           pg_temp.dashboard_json_i32(
               dashboard_components.config -> 'grid_height'
           ) AS grid_height
    FROM dashboard_components
    JOIN component_versions
      ON component_versions.id = dashboard_components.component_version_id
)
SELECT dashboard_id,
       placement_id,
       grid_row,
       grid_column,
       grid_width,
       grid_height,
       (
           jsonb_typeof(config) = 'object'
           AND schema_version = 1
           AND (
               NOT config ? 'title'
               OR config -> 'title' = 'null'::jsonb
               OR jsonb_typeof(config -> 'title') = 'string'
           )
           AND grid_row >= 1
           AND grid_column >= 1
           AND grid_width >= CASE WHEN component_type = 'table' THEN 6 ELSE 1 END
           AND grid_height >= CASE WHEN component_type = 'table' THEN 4 ELSE 1 END
           AND grid_column::bigint + grid_width::bigint - 1 <= 12
           AND grid_row::bigint + grid_height::bigint - 1 <= 240
       ) IS TRUE AS valid_v1
FROM decoded;

DO $$
DECLARE
    violations text;
BEGIN
    SELECT string_agg(DISTINCT left_placement.dashboard_id::text, ', ')
    INTO violations
    FROM dashboard_placement_layout_preflight AS left_placement
    JOIN dashboard_placement_layout_preflight AS right_placement
      ON right_placement.dashboard_id = left_placement.dashboard_id
     AND right_placement.placement_id > left_placement.placement_id
     AND right_placement.valid_v1
     AND left_placement.valid_v1
     AND left_placement.grid_column
             <= right_placement.grid_column + right_placement.grid_width - 1
     AND right_placement.grid_column
             <= left_placement.grid_column + left_placement.grid_width - 1
     AND left_placement.grid_row
             <= right_placement.grid_row + right_placement.grid_height - 1
     AND right_placement.grid_row
             <= left_placement.grid_row + left_placement.grid_height - 1;

    IF violations IS NOT NULL THEN
        RAISE EXCEPTION USING
            ERRCODE = '23514',
            MESSAGE = 'dashboard placement layout preflight found overlapping valid V1 geometry: ' || violations,
            HINT = 'Repair or remove the overlapping placements, then rerun the migration. See docs/sprints/sprint-5a-dashboard-capacity-runbook.md.';
    END IF;
END
$$;

DO $$
DECLARE
    violations text;
BEGIN
    WITH occupied_rows AS (
        SELECT DISTINCT placement.dashboard_id,
               generate_series(
                   placement.grid_row,
                   placement.grid_row + placement.grid_height - 1
               ) AS grid_row
        FROM dashboard_placement_layout_preflight AS placement
        WHERE placement.valid_v1
    ),
    occupied_counts AS (
        SELECT dashboard_id, COUNT(*) AS occupied_row_count
        FROM occupied_rows
        GROUP BY dashboard_id
    ),
    requirements AS (
        SELECT placement.dashboard_id,
               COUNT(*) FILTER (WHERE NOT placement.valid_v1) AS fallback_row_count,
               COALESCE(occupied_counts.occupied_row_count, 0) AS occupied_row_count
        FROM dashboard_placement_layout_preflight AS placement
        LEFT JOIN occupied_counts
          ON occupied_counts.dashboard_id = placement.dashboard_id
        GROUP BY placement.dashboard_id, occupied_counts.occupied_row_count
    )
    SELECT string_agg(
        format(
            '%s (%s occupied rows + %s fallback rows)',
            dashboard_id,
            occupied_row_count,
            fallback_row_count
        ),
        ', '
        ORDER BY dashboard_id
    )
    INTO violations
    FROM requirements
    WHERE occupied_row_count + fallback_row_count > 240;

    IF violations IS NOT NULL THEN
        RAISE EXCEPTION USING
            ERRCODE = '23514',
            MESSAGE = 'dashboard placement display-layout preflight failed: ' || violations,
            HINT = 'Repair or remove placements until every fallback row fits outside valid V1 geometry, then rerun the migration. See docs/sprints/sprint-5a-dashboard-capacity-runbook.md.';
    END IF;
END
$$;

DROP FUNCTION pg_temp.dashboard_json_i32(jsonb);

CREATE INDEX dashboard_components_dashboard_position_idx
    ON dashboard_components (dashboard_id, position, id);

CREATE OR REPLACE FUNCTION enforce_dashboard_component_capacity()
RETURNS trigger
LANGUAGE plpgsql
AS $$
DECLARE
    existing_count bigint;
BEGIN
    IF TG_OP = 'UPDATE' AND NEW.dashboard_id = OLD.dashboard_id THEN
        RETURN NEW;
    END IF;

    -- Serialize inserts/moves for one dashboard so concurrent requests cannot
    -- both observe the same final capacity.
    PERFORM 1
    FROM dashboards
    WHERE id = NEW.dashboard_id
    FOR UPDATE;

    SELECT COUNT(*)
    INTO existing_count
    FROM dashboard_components
    WHERE dashboard_id = NEW.dashboard_id
      AND (TG_OP <> 'UPDATE' OR id <> OLD.id);

    IF existing_count >= 240 THEN
        RAISE EXCEPTION USING
            ERRCODE = '23514',
            MESSAGE = format(
                'dashboard %s already has the maximum 240 placements',
                NEW.dashboard_id
            ),
            CONSTRAINT = 'dashboard_components_capacity_chk';
    END IF;

    RETURN NEW;
END
$$;

CREATE TRIGGER dashboard_components_capacity_trigger
BEFORE INSERT OR UPDATE OF dashboard_id ON dashboard_components
FOR EACH ROW
EXECUTE FUNCTION enforce_dashboard_component_capacity();

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
    SELECT entries.installation_id, entries.contribution_id, 'core.main', entries.visible, 6
    FROM navigation_policy_entries AS entries
    WHERE entries.contribution_id = 'tessara.datasets.navigation'
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
    database_name TEXT NOT NULL,
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
