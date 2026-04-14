-- access and organization starter migration
CREATE TABLE organization_nodes (
  organization_node_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  parent_organization_node_id UUID NULL REFERENCES organization_nodes(organization_node_id),
  label TEXT NOT NULL,
  node_type TEXT NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE users (
  user_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  email TEXT NOT NULL UNIQUE,
  display_name TEXT NOT NULL,
  is_active BOOLEAN NOT NULL DEFAULT TRUE,
  created_at TIMESTAMPTZ NOT NULL,
  updated_at TIMESTAMPTZ NOT NULL
);
