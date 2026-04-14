-- forms and workflows starter migration
CREATE TABLE field_definitions (
  field_definition_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  field_type TEXT NOT NULL,
  provider_kind TEXT NULL,
  provider_id UUID NULL,
  description TEXT NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
