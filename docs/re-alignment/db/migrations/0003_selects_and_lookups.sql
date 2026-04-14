-- select options and lookup sources starter migration
CREATE TABLE option_sets (
  option_set_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  name TEXT NOT NULL,
  description TEXT NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE
);
