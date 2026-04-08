ALTER TABLE charts
    ADD COLUMN aggregation_id uuid REFERENCES aggregations(id) ON DELETE SET NULL;

ALTER TABLE charts
    ADD CONSTRAINT charts_single_data_source_check
    CHECK (NOT (report_id IS NOT NULL AND aggregation_id IS NOT NULL));

CREATE INDEX charts_aggregation_id_idx
    ON charts(aggregation_id)
    WHERE aggregation_id IS NOT NULL;
