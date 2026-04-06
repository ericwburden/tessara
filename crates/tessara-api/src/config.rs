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
        })
    }
}
