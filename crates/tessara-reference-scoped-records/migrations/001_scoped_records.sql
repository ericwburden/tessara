CREATE TABLE IF NOT EXISTS scoped_records (
    id UUID PRIMARY KEY,
    label TEXT NOT NULL CHECK (length(trim(label)) > 0),
    scope TEXT NOT NULL CHECK (length(trim(scope)) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

