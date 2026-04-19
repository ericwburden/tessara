ALTER TABLE account_credentials
    RENAME COLUMN password TO legacy_password;

ALTER TABLE account_credentials
    ALTER COLUMN legacy_password DROP NOT NULL;

ALTER TABLE account_credentials
    ADD COLUMN password_hash text,
    ADD COLUMN password_scheme text,
    ADD COLUMN password_updated_at timestamptz NOT NULL DEFAULT now();

CREATE INDEX IF NOT EXISTS idx_account_credentials_password_scheme
    ON account_credentials (password_scheme);

ALTER TABLE auth_sessions
    ADD COLUMN expires_at timestamptz NOT NULL DEFAULT (now() + interval '12 hours'),
    ADD COLUMN last_seen_at timestamptz NOT NULL DEFAULT now(),
    ADD COLUMN revoked_at timestamptz;

CREATE INDEX IF NOT EXISTS idx_auth_sessions_account_id
    ON auth_sessions (account_id);

CREATE INDEX IF NOT EXISTS idx_auth_sessions_active_lookup
    ON auth_sessions (token, revoked_at, expires_at);
