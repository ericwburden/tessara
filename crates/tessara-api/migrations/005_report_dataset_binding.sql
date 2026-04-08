ALTER TABLE reports
    ADD COLUMN dataset_id uuid REFERENCES datasets(id) ON DELETE SET NULL;

CREATE INDEX reports_dataset_id_idx
    ON reports (dataset_id)
    WHERE dataset_id IS NOT NULL;
