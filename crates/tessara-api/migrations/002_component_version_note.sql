ALTER TABLE component_versions
    ADD COLUMN IF NOT EXISTS version_note text NOT NULL DEFAULT '';
