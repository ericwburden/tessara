ALTER TABLE datasets
    ADD COLUMN composition_mode text NOT NULL DEFAULT 'union';

ALTER TABLE datasets
    ADD CONSTRAINT datasets_composition_mode_check
    CHECK (composition_mode IN ('union', 'join'));
