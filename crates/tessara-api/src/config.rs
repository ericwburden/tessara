//! Runtime configuration loaded from environment variables.

/// Configuration required to start the Tessara API locally or in a container.
///
/// Defaults are intentionally developer-friendly so `docker compose` and local
/// `cargo run` invocations work without a separate configuration framework.
#[derive(Clone)]
pub struct Config {
    /// PostgreSQL connection string used for migrations and runtime queries.
    pub database_url: String,
    /// Socket address the API binds to, for example `0.0.0.0:8080`.
    pub bind_addr: String,
    /// Development administrator email seeded at startup.
    pub dev_admin_email: String,
    /// Development administrator password accepted by the temporary login flow.
    pub dev_admin_password: String,
    /// Cookie name used for browser `/app` sessions.
    pub auth_cookie_name: String,
    /// Whether auth cookies should be marked `Secure`.
    pub auth_cookie_secure: bool,
    /// Browser/API session lifetime in hours.
    pub auth_session_ttl_hours: i64,
}

impl Config {
    /// Loads configuration from environment variables, falling back to local
    /// development defaults when variables are absent.
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgres://tessara:tessara@localhost:5432/tessara".into()),
            bind_addr: std::env::var("TESSARA_BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".into()),
            dev_admin_email: std::env::var("TESSARA_DEV_ADMIN_EMAIL")
                .unwrap_or_else(|_| "admin@tessara.local".into()),
            dev_admin_password: std::env::var("TESSARA_DEV_ADMIN_PASSWORD")
                .unwrap_or_else(|_| "tessara-dev-admin".into()),
            auth_cookie_name: std::env::var("TESSARA_AUTH_COOKIE_NAME")
                .unwrap_or_else(|_| "tessara_session".into()),
            auth_cookie_secure: std::env::var("TESSARA_AUTH_COOKIE_SECURE")
                .ok()
                .as_deref()
                .is_some_and(|value| matches!(value, "1" | "true" | "TRUE" | "True")),
            auth_session_ttl_hours: std::env::var("TESSARA_AUTH_SESSION_TTL_HOURS")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(12),
        })
    }
}
