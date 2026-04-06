CREATE UNIQUE INDEX IF NOT EXISTS compatibility_groups_form_id_name_idx
    ON compatibility_groups (form_id, name);
