ALTER TABLE aggregation_metrics
    DROP CONSTRAINT aggregation_metrics_kind_check;

ALTER TABLE aggregation_metrics
    ADD CONSTRAINT aggregation_metrics_kind_check
    CHECK (metric_kind IN ('count', 'sum', 'avg', 'min', 'max'));
