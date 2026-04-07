CREATE UNIQUE INDEX IF NOT EXISTS choice_lists_form_version_id_name_idx
    ON choice_lists (form_version_id, name);

CREATE UNIQUE INDEX IF NOT EXISTS choice_list_items_choice_list_id_value_idx
    ON choice_list_items (choice_list_id, value);
