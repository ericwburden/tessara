ALTER TYPE component_type ADD VALUE IF NOT EXISTS 'table';

ALTER TABLE component_versions
    DROP CONSTRAINT IF EXISTS component_versions_component_type_table_chk;

ALTER TABLE component_versions
    ADD CONSTRAINT component_versions_component_type_table_chk
    CHECK (component_type::text = 'table');
