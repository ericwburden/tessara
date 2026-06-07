CREATE SCHEMA IF NOT EXISTS dataset_materialized;

ALTER TABLE datasets
    DROP CONSTRAINT IF EXISTS datasets_composition_mode_check;

ALTER TABLE datasets
    ADD CONSTRAINT datasets_composition_mode_check
    CHECK (composition_mode IN ('union', 'union_all', 'left_join', 'inner_join', 'outer_join', 'join'));

ALTER TABLE dataset_revisions
    ADD COLUMN IF NOT EXISTS definition_ast jsonb,
    ADD COLUMN IF NOT EXISTS generated_sql text,
    ADD COLUMN IF NOT EXISTS materialized_schema text,
    ADD COLUMN IF NOT EXISTS materialized_table text,
    ADD COLUMN IF NOT EXISTS materialized_row_count bigint,
    ADD COLUMN IF NOT EXISTS materialized_at timestamptz;

CREATE INDEX IF NOT EXISTS dataset_revisions_materialized_table_idx
    ON dataset_revisions (materialized_schema, materialized_table);

ALTER TABLE dataset_sources
    ADD COLUMN IF NOT EXISTS dataset_revision_id uuid REFERENCES dataset_revisions(id) ON DELETE RESTRICT;

ALTER TABLE dataset_sources
    DROP CONSTRAINT IF EXISTS dataset_sources_check;

ALTER TABLE dataset_sources
    ADD CONSTRAINT dataset_sources_one_source_check
    CHECK (
        ((form_id IS NOT NULL)::integer
        + (compatibility_group_id IS NOT NULL)::integer
        + (dataset_revision_id IS NOT NULL)::integer) = 1
    );
