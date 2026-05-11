ALTER TABLE form_sections
    DROP CONSTRAINT form_sections_column_count_check,
    ADD CONSTRAINT form_sections_column_count_check
        CHECK (column_count BETWEEN 1 AND 12);

ALTER TABLE form_fields
    ADD COLUMN grid_row integer NOT NULL DEFAULT 1,
    ADD COLUMN grid_column integer NOT NULL DEFAULT 1,
    ADD COLUMN grid_width integer NOT NULL DEFAULT 1,
    ADD COLUMN grid_height integer NOT NULL DEFAULT 1,
    ADD CONSTRAINT form_fields_grid_row_check CHECK (grid_row >= 1),
    ADD CONSTRAINT form_fields_grid_column_check CHECK (grid_column >= 1),
    ADD CONSTRAINT form_fields_grid_width_check CHECK (grid_width >= 1),
    ADD CONSTRAINT form_fields_grid_height_check CHECK (grid_height >= 1);
