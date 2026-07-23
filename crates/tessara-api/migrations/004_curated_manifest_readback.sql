ALTER TABLE module_releases
    ADD COLUMN manifest JSONB;

ALTER TABLE module_instances
    ADD COLUMN configuration JSONB NOT NULL DEFAULT '{}'::JSONB,
    ADD COLUMN route_prefix TEXT;
