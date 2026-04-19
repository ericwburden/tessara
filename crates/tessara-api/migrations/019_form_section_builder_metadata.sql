ALTER TABLE form_sections
    ADD COLUMN description text NOT NULL DEFAULT '',
    ADD COLUMN column_count integer NOT NULL DEFAULT 1,
    ADD CONSTRAINT form_sections_column_count_check
        CHECK (column_count IN (1, 2));
