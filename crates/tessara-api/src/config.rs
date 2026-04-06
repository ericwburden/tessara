#[derive(Clone)]
pub struct Config {
    pub database_url: String,
    pub bind_addr: String,
    pub dev_admin_email: String,
    pub dev_admin_password: String,
}

impl Config {
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
