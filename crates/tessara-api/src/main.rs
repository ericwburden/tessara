use std::net::SocketAddr;

use tessara_api::{config::Config, db};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "tessara_api=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = Config::from_env()?;
    let pool = db::connect_and_prepare(&config).await?;

    if std::env::args().any(|arg| arg == "seed-demo") {
        let summary = tessara_api::demo::seed_demo(&pool).await?;
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    let state = db::AppState { pool, config };
    let addr: SocketAddr = state.config.bind_addr.parse()?;
    let app = tessara_api::router(state);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "starting tessara api");
    axum::serve(listener, app).await?;

    Ok(())
}
