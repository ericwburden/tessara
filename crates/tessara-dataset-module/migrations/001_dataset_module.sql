CREATE SCHEMA IF NOT EXISTS dataset_materialized;
DO $$
BEGIN
    IF (SELECT nspowner FROM pg_namespace WHERE nspname = 'dataset_materialized') <>
       (SELECT oid FROM pg_roles WHERE rolname = current_user) THEN
        RAISE EXCEPTION 'dataset_materialized must be owned by the Dataset migration owner';
    END IF;
END
$$;

CREATE TABLE dataset_security_state (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    installation_id uuid NOT NULL,
    module_instance_id uuid NOT NULL,
    authorization_revision bigint NOT NULL CHECK (authorization_revision >= 0),
    organization_revision bigint NOT NULL CHECK (organization_revision >= 0),
    enabled boolean NOT NULL,
    document_state text NOT NULL CHECK (document_state IN ('enabled', 'disabled', 'degraded', 'recovery')),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE dataset_configuration (
    singleton boolean PRIMARY KEY DEFAULT true CHECK (singleton),
    schema_version integer NOT NULL CHECK (schema_version = 1),
    display_label text NOT NULL CHECK (length(btrim(display_label)) BETWEEN 1 AND 80),
    provider_request_timeout_seconds integer NOT NULL
        CHECK (provider_request_timeout_seconds BETWEEN 1 AND 30),
    provider_retry_limit integer NOT NULL CHECK (provider_retry_limit BETWEEN 0 AND 3),
    response_export_page_size integer NOT NULL
        CHECK (response_export_page_size BETWEEN 1 AND 1000),
    updated_at timestamptz NOT NULL DEFAULT now()
);

INSERT INTO dataset_configuration
    (singleton, schema_version, display_label, provider_request_timeout_seconds,
     provider_retry_limit, response_export_page_size)
VALUES (true, 1, 'Datasets', 5, 1, 250);

-- Canonical union for the hidden per-row governing scope carried through
-- joins, unions, and aggregations. The aggregate state is always sorted and
-- deduplicated so equivalent source orderings materialize identically.
CREATE FUNCTION dataset_scope_union_state(current_scope uuid[], next_scope uuid[])
RETURNS uuid[] LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $$
    SELECT ARRAY(
        SELECT DISTINCT node_id
        FROM unnest(COALESCE(current_scope, '{}'::uuid[]) || COALESCE(next_scope, '{}'::uuid[]))
             AS node_id
        ORDER BY node_id
    )
$$;

CREATE AGGREGATE dataset_scope_union(uuid[]) (
    SFUNC = dataset_scope_union_state,
    STYPE = uuid[],
    INITCOND = '{}'
);

CREATE TABLE datasets (
    id uuid PRIMARY KEY,
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    grain text NOT NULL CHECK (grain IN ('submission', 'node')),
    description text,
    lifecycle_state text NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active', 'inactive', 'archived', 'tombstoned')),
    resource_revision bigint NOT NULL DEFAULT 1 CHECK (resource_revision > 0),
    authority_revision bigint NOT NULL DEFAULT 1 CHECK (authority_revision > 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now()
);

CREATE TABLE dataset_scope_nodes (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    node_id uuid NOT NULL,
    node_name text NOT NULL,
    node_type_name text NOT NULL,
    parent_node_id uuid,
    node_path text NOT NULL,
    requested_set_revision text NOT NULL,
    requested_set_digest text NOT NULL CHECK (requested_set_digest ~ '^sha256:[0-9a-f]{64}$'),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dataset_id, node_id)
);

CREATE INDEX dataset_scope_nodes_node_id_idx
    ON dataset_scope_nodes(node_id, dataset_id);

CREATE TABLE dataset_tags (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    tag text NOT NULL CHECK (btrim(tag) <> ''),
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (dataset_id, tag)
);

CREATE TABLE dataset_revisions (
    id uuid PRIMARY KEY,
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_number integer NOT NULL CHECK (version_number > 0),
    version_label text NOT NULL,
    version_major integer CHECK (version_major > 0),
    version_minor integer CHECK (version_minor >= 0),
    version_patch integer CHECK (version_patch >= 0),
    semantic_bump text CHECK (semantic_bump IN ('initial', 'major', 'minor', 'patch')),
    started_new_major_line boolean,
    force_new_major_version boolean NOT NULL DEFAULT false,
    revision_notes text NOT NULL DEFAULT '',
    status text NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'published', 'superseded')),
    lifecycle_state text NOT NULL DEFAULT 'active'
        CHECK (lifecycle_state IN ('active', 'inactive', 'archived', 'tombstoned')),
    resource_revision bigint NOT NULL DEFAULT 1 CHECK (resource_revision > 0),
    initial_source jsonb,
    operations jsonb NOT NULL DEFAULT '[]'::jsonb,
    restriction_policy jsonb,
    definition_metadata jsonb,
    compatibility_findings jsonb NOT NULL DEFAULT '[]'::jsonb,
    generated_sql text,
    output_fields jsonb NOT NULL DEFAULT '[]'::jsonb,
    materialized_schema text,
    materialized_table text,
    materialized_row_count bigint,
    materialized_at timestamptz,
    published_at timestamptz,
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(dataset_id, version_number)
);

CREATE UNIQUE INDEX dataset_revisions_one_published_idx
    ON dataset_revisions(dataset_id) WHERE status = 'published';
CREATE UNIQUE INDEX dataset_revisions_one_draft_idx
    ON dataset_revisions(dataset_id) WHERE status = 'draft';
CREATE INDEX dataset_revisions_semantic_version_idx
    ON dataset_revisions(dataset_id, version_major, version_minor, version_patch);

CREATE TABLE dataset_revision_scope_nodes (
    revision_id uuid NOT NULL REFERENCES dataset_revisions(id) ON DELETE CASCADE,
    node_id uuid NOT NULL,
    node_name text NOT NULL,
    node_type_name text NOT NULL,
    parent_node_id uuid,
    node_path text NOT NULL,
    requested_set_revision text NOT NULL,
    requested_set_digest text NOT NULL CHECK (requested_set_digest ~ '^sha256:[0-9a-f]{64}$'),
    PRIMARY KEY (revision_id,node_id)
);

CREATE TABLE dataset_revision_sources (
    revision_id uuid NOT NULL REFERENCES dataset_revisions(id) ON DELETE CASCADE,
    source_alias text NOT NULL,
    source_kind text NOT NULL CHECK (source_kind IN ('form_version','dataset_revision','dataset_major_line')),
    source_reference jsonb NOT NULL,
    source_name text NOT NULL,
    source_slug text,
    source_version_label text,
    source_scope_node_ids uuid[] NOT NULL DEFAULT '{}',
    source_scope_revision text NOT NULL,
    source_scope_digest text NOT NULL CHECK (source_scope_digest ~ '^sha256:[0-9a-f]{64}$'),
    source_content_revision text NOT NULL,
    source_content_digest text NOT NULL CHECK (source_content_digest ~ '^sha256:[0-9a-f]{64}$'),
    position integer NOT NULL CHECK (position >= 0),
    PRIMARY KEY (revision_id,source_alias)
);

CREATE TABLE dataset_major_materializations (
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    version_major integer NOT NULL CHECK (version_major > 0),
    materialized_schema text,
    materialized_table text,
    materialized_row_count bigint,
    materialized_at timestamptz,
    rebuild_status text NOT NULL DEFAULT 'pending'
        CHECK (rebuild_status IN ('pending', 'ready', 'failed')),
    created_at timestamptz NOT NULL DEFAULT now(),
    updated_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY(dataset_id, version_major)
);

CREATE TABLE dataset_sources (
    id uuid PRIMARY KEY,
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    source_binding_id uuid NOT NULL UNIQUE,
    source_alias text NOT NULL,
    source_kind text NOT NULL CHECK (source_kind IN ('form_version', 'dataset_revision', 'dataset_major_line')),
    source_reference jsonb NOT NULL,
    source_name text NOT NULL,
    source_slug text,
    source_version_label text,
    source_scope_node_ids uuid[] NOT NULL DEFAULT '{}',
    source_scope_revision text NOT NULL,
    source_scope_digest text NOT NULL CHECK (source_scope_digest ~ '^sha256:[0-9a-f]{64}$'),
    source_content_revision text NOT NULL,
    source_content_digest text NOT NULL CHECK (source_content_digest ~ '^sha256:[0-9a-f]{64}$'),
    position integer NOT NULL DEFAULT 0 CHECK (position >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(dataset_id, source_alias)
);

CREATE TABLE dataset_fields (
    id uuid PRIMARY KEY,
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    key text NOT NULL,
    label text NOT NULL,
    source_alias text NOT NULL,
    source_field_key text NOT NULL,
    source_field_id uuid,
    field_type text NOT NULL CHECK (field_type IN ('text', 'number', 'boolean', 'date', 'single_choice', 'multi_choice', 'static_text')),
    position integer NOT NULL DEFAULT 0 CHECK (position >= 0),
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE(dataset_id, key)
);

CREATE INDEX dataset_sources_dataset_id_position_idx
    ON dataset_sources(dataset_id, position, source_alias);
CREATE INDEX dataset_fields_dataset_id_position_idx
    ON dataset_fields(dataset_id, position, key);

CREATE OR REPLACE FUNCTION advance_dataset_authority_revision()
RETURNS trigger LANGUAGE plpgsql AS $$
DECLARE
    target_dataset_id uuid;
BEGIN
    target_dataset_id := CASE WHEN TG_OP = 'DELETE' THEN OLD.dataset_id ELSE NEW.dataset_id END;
    UPDATE datasets
    SET authority_revision = authority_revision + 1, updated_at = now()
    WHERE id = target_dataset_id;
    RETURN CASE WHEN TG_OP = 'DELETE' THEN OLD ELSE NEW END;
END;
$$;

CREATE TRIGGER dataset_scope_nodes_authority_revision
AFTER INSERT OR UPDATE OR DELETE ON dataset_scope_nodes
FOR EACH ROW EXECUTE FUNCTION advance_dataset_authority_revision();

CREATE TRIGGER dataset_revisions_authority_revision
AFTER INSERT OR UPDATE OR DELETE ON dataset_revisions
FOR EACH ROW EXECUTE FUNCTION advance_dataset_authority_revision();

CREATE TABLE dataset_source_bindings (
    id uuid PRIMARY KEY,
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    binding_key text NOT NULL,
    source_kind text NOT NULL CHECK (source_kind IN ('response_export', 'dataset_major_line')),
    source_identity jsonb NOT NULL,
    source_identity_digest text NOT NULL,
    UNIQUE (dataset_id, binding_key)
);

-- Provider-bound authorization grants and signed service requests are
-- one-time capabilities. Dataset owns their replay state because it is the
-- authoritative provider that consumes them.
CREATE TABLE dataset_consumed_service_nonces (
    module_instance_id uuid NOT NULL,
    nonce uuid NOT NULL,
    authorization_jti uuid NOT NULL UNIQUE,
    correlation_id text NOT NULL CHECK (btrim(correlation_id) <> ''),
    issued_at timestamptz NOT NULL,
    consumed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (module_instance_id, nonce)
);

-- CoreGateway is not a Module Instance and therefore consumes an independent
-- one-use request namespace. Keeping this table distinct prevents a Core call
-- from ever being accepted as a module-to-module service request.
CREATE TABLE dataset_consumed_core_service_nonces (
    installation_id uuid NOT NULL,
    nonce uuid NOT NULL,
    authorization_jti uuid NOT NULL UNIQUE,
    correlation_id text NOT NULL CHECK (btrim(correlation_id) <> ''),
    issued_at timestamptz NOT NULL,
    consumed_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (installation_id, nonce)
);

ALTER TABLE dataset_sources
    ADD CONSTRAINT dataset_sources_source_binding_fk
    FOREIGN KEY (source_binding_id) REFERENCES dataset_source_bindings(id) ON DELETE CASCADE;

-- Published state is the only cursor visible to materialization and readers.
CREATE TABLE dataset_sync_partitions (
    source_binding_id uuid PRIMARY KEY REFERENCES dataset_source_bindings(id) ON DELETE CASCADE,
    provider_epoch uuid,
    committed_cursor text,
    committed_snapshot_upper_bound text,
    generation bigint NOT NULL DEFAULT 0 CHECK (generation >= 0),
    last_checked_at timestamptz,
    last_succeeded_at timestamptz,
    freshness_state text NOT NULL DEFAULT 'never_materialized'
        CHECK (freshness_state IN ('current', 'stale', 'degraded', 'refreshing', 'failed', 'never_materialized')),
    sanitized_failure_code text,
    updated_at timestamptz NOT NULL DEFAULT now()
);

-- Attempt/page checkpoints are private staging state and never become a
-- committed cursor independently of projection/materialization promotion.
CREATE TABLE dataset_sync_attempts (
    id uuid PRIMARY KEY,
    source_binding_id uuid NOT NULL REFERENCES dataset_sync_partitions(source_binding_id) ON DELETE CASCADE,
    provider_epoch uuid NOT NULL,
    start_generation bigint NOT NULL CHECK (start_generation >= 0),
    start_cursor text,
    snapshot_upper_bound text NOT NULL,
    last_staged_cursor text,
    full_snapshot_rebase boolean NOT NULL,
    state text NOT NULL CHECK (state IN ('active', 'complete', 'promoted', 'failed', 'superseded')),
    started_at timestamptz NOT NULL DEFAULT now(),
    completed_at timestamptz,
    UNIQUE (source_binding_id, id)
);

CREATE UNIQUE INDEX dataset_sync_attempt_one_active_per_partition
    ON dataset_sync_attempts(source_binding_id)
    WHERE state = 'active';

CREATE TABLE dataset_sync_staged_changes (
    attempt_id uuid NOT NULL REFERENCES dataset_sync_attempts(id) ON DELETE CASCADE,
    ordinal bigint NOT NULL CHECK (ordinal >= 0),
    cursor text NOT NULL,
    change_kind text NOT NULL CHECK (change_kind IN ('upsert', 'tombstone')),
    response_id uuid NOT NULL,
    payload jsonb NOT NULL,
    content_digest text NOT NULL,
    page_digest text NOT NULL,
    PRIMARY KEY (attempt_id, ordinal),
    UNIQUE (attempt_id, cursor)
);

CREATE TABLE dataset_imported_responses (
    source_binding_id uuid NOT NULL REFERENCES dataset_source_bindings(id) ON DELETE CASCADE,
    response_id uuid NOT NULL,
    form_id uuid,
    form_version_id uuid,
    node_id uuid,
    node_name text,
    status text,
    submitted_at timestamptz,
    created_at timestamptz,
    last_modified_at timestamptz,
    last_modified_by_user_name text,
    restriction_tier text CHECK (restriction_tier IN ('public', 'internal', 'restricted', 'confidential')),
    scope_node_ids uuid[] NOT NULL DEFAULT '{}',
    content_digest text NOT NULL CHECK (content_digest ~ '^sha256:[0-9a-f]{64}$'),
    tombstoned boolean NOT NULL,
    tombstone_reason text CHECK (tombstone_reason IN ('deleted', 'redacted', 'status_excluded', 'scope_excluded')),
    promoted_generation bigint NOT NULL CHECK (promoted_generation > 0),
    PRIMARY KEY (source_binding_id, response_id),
    CHECK (
        (
            tombstoned
            AND form_id IS NULL
            AND form_version_id IS NULL
            AND node_id IS NULL
            AND node_name IS NULL
            AND status IS NULL
            AND submitted_at IS NULL
            AND created_at IS NULL
            AND last_modified_at IS NULL
            AND last_modified_by_user_name IS NULL
            AND restriction_tier IS NULL
            AND cardinality(scope_node_ids) = 0
            AND tombstone_reason IS NOT NULL
        )
        OR
        (
            NOT tombstoned
            AND form_id IS NOT NULL
            AND form_version_id IS NOT NULL
            AND node_id IS NOT NULL
            AND node_name IS NOT NULL
            AND status = 'submitted'
            AND submitted_at IS NOT NULL
            AND created_at IS NOT NULL
            AND last_modified_at IS NOT NULL
            AND restriction_tier IS NOT NULL
            AND cardinality(scope_node_ids) > 0
            AND node_id = ANY(scope_node_ids)
            AND tombstone_reason IS NULL
        )
    )
);

CREATE INDEX dataset_imported_responses_source_form_version_idx
    ON dataset_imported_responses(source_binding_id, form_version_id, response_id)
    WHERE NOT tombstoned;

CREATE TABLE dataset_imported_response_values (
    source_binding_id uuid NOT NULL,
    response_id uuid NOT NULL,
    form_version_id uuid NOT NULL,
    field_id uuid NOT NULL,
    field_key text NOT NULL CHECK (btrim(field_key) <> ''),
    value_json jsonb NOT NULL,
    value_text text,
    promoted_generation bigint NOT NULL CHECK (promoted_generation > 0),
    PRIMARY KEY (source_binding_id, response_id, field_id),
    UNIQUE (source_binding_id, response_id, field_key),
    FOREIGN KEY (source_binding_id, response_id)
        REFERENCES dataset_imported_responses(source_binding_id, response_id)
        ON DELETE CASCADE
);

CREATE INDEX dataset_imported_response_values_pivot_idx
    ON dataset_imported_response_values(source_binding_id, form_version_id, field_id, response_id);

CREATE TABLE dataset_materialization_receipts (
    id uuid PRIMARY KEY,
    source_binding_id uuid NOT NULL REFERENCES dataset_source_bindings(id) ON DELETE CASCADE,
    attempt_id uuid NOT NULL UNIQUE REFERENCES dataset_sync_attempts(id),
    generation bigint NOT NULL CHECK (generation > 0),
    committed_cursor text NOT NULL,
    snapshot_upper_bound text NOT NULL,
    input_digest text NOT NULL,
    result_digest text NOT NULL,
    promoted_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (source_binding_id, generation)
);

CREATE TABLE dataset_idempotency_receipts (
    actor_id uuid NOT NULL,
    action text NOT NULL,
    idempotency_key text NOT NULL,
    request_digest text NOT NULL,
    response_status integer NOT NULL,
    response_media_type text NOT NULL,
    response_body bytea NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (actor_id, idempotency_key),
    UNIQUE (idempotency_key)
);

CREATE TABLE dataset_bootstrap_receipts (
    idempotency_key text PRIMARY KEY CHECK (btrim(idempotency_key) <> ''),
    locked_input_digest text NOT NULL CHECK (locked_input_digest ~ '^sha256:[0-9a-f]{64}$'),
    input_digest text NOT NULL CHECK (input_digest ~ '^sha256:[0-9a-f]{64}$'),
    desired_revision bigint NOT NULL CHECK (desired_revision > 0),
    apply_sequence bigint NOT NULL CHECK (apply_sequence > 0),
    authority_jti uuid NOT NULL UNIQUE,
    receipt jsonb NOT NULL,
    applied_at timestamptz NOT NULL DEFAULT now()
);
