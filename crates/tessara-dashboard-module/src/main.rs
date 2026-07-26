use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::postgres::PgPoolOptions;
use tessara_dashboard_module::{DashboardModuleState, router};
use tessara_module_contract::{ProtocolSignaturePurposeV1, PurposeBoundVerifyingKeyV1};
use tower_http::trace::TraceLayer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await?;
    if env::args().nth(1).as_deref() == Some("migrate") {
        sqlx::migrate!().run(&pool).await?;
        return Ok(());
    }

    let public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY")
                .context("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Core authorization public key must contain 32 bytes"))?;
    let authorization_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        "core-development-v1",
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        public_key,
    )?;
    let shell_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        "core-development-v1",
        ProtocolSignaturePurposeV1::ShellContext,
        public_key,
    )?;

    let address: SocketAddr = env::var("DASHBOARD_MODULE_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8091".into())
        .parse()?;
    let app = router(DashboardModuleState {
        pool,
        core_authorization_verifier: authorization_verifier,
        core_shell_verifier: shell_verifier,
    })
    .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Dashboard module listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}
