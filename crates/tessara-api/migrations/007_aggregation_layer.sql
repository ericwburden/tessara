ALTER TABLE aggregations
    ADD COLUMN name text NOT NULL DEFAULT 'Aggregation',
    ADD COLUMN group_by_logical_key text,
    ADD COLUMN created_at timestamptz NOT NULL DEFAULT now();

ALTER TABLE aggregations
    ALTER COLUMN name DROP DEFAULT,
    ALTER COLUMN aggregation_kind DROP NOT NULL;

CREATE TABLE aggregation_metrics (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    aggregation_id uuid NOT NULL REFERENCES aggregations(id) ON DELETE CASCADE,
    metric_key text NOT NULL,
    source_logical_key text,
    metric_kind text NOT NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (aggregation_id, metric_key),
    CONSTRAINT aggregation_metrics_kind_check CHECK (metric_kind IN ('count', 'sum'))
);
