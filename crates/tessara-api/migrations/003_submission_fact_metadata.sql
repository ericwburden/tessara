ALTER TABLE analytics.submission_fact
    ADD COLUMN IF NOT EXISTS created_at timestamptz,
    ADD COLUMN IF NOT EXISTS last_modified_at timestamptz,
    ADD COLUMN IF NOT EXISTS last_modified_by_user_name text;
