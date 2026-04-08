CREATE TABLE datasets (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    name text NOT NULL,
    slug text NOT NULL UNIQUE,
    grain text NOT NULL,
    created_at timestamptz NOT NULL DEFAULT now(),
    CONSTRAINT datasets_grain_check CHECK (grain IN ('submission', 'node'))
);

CREATE TABLE dataset_sources (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    source_alias text NOT NULL,
    form_id uuid REFERENCES forms(id) ON DELETE CASCADE,
    compatibility_group_id uuid REFERENCES compatibility_groups(id) ON DELETE CASCADE,
    selection_rule text NOT NULL DEFAULT 'all',
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, source_alias),
    CONSTRAINT dataset_sources_selection_rule_check CHECK (selection_rule IN ('all', 'latest', 'earliest')),
    CONSTRAINT dataset_sources_one_source_check CHECK (
        (form_id IS NOT NULL AND compatibility_group_id IS NULL)
        OR (form_id IS NULL AND compatibility_group_id IS NOT NULL)
    )
);

CREATE TABLE dataset_fields (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    dataset_id uuid NOT NULL REFERENCES datasets(id) ON DELETE CASCADE,
    key text NOT NULL,
    label text NOT NULL,
    source_alias text NOT NULL,
    source_field_key text NOT NULL,
    field_type field_type NOT NULL,
    position integer NOT NULL DEFAULT 0,
    created_at timestamptz NOT NULL DEFAULT now(),
    UNIQUE (dataset_id, key),
    FOREIGN KEY (dataset_id, source_alias)
        REFERENCES dataset_sources(dataset_id, source_alias)
        ON DELETE CASCADE
);

CREATE INDEX dataset_sources_dataset_id_position_idx
    ON dataset_sources (dataset_id, position, source_alias);

CREATE INDEX dataset_fields_dataset_id_position_idx
    ON dataset_fields (dataset_id, position, key);
