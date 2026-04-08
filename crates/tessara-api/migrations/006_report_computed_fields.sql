ALTER TABLE report_field_bindings
    ADD COLUMN computed_expression text;

ALTER TABLE report_field_bindings
    ALTER COLUMN source_field_key DROP NOT NULL;

ALTER TABLE report_field_bindings
    ADD CONSTRAINT report_field_bindings_source_or_computed_check
    CHECK (
        (source_field_key IS NOT NULL AND computed_expression IS NULL)
        OR (source_field_key IS NULL AND computed_expression IS NOT NULL)
    );
