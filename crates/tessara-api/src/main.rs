use std::{fs, net::SocketAddr, path::PathBuf};

use anyhow::Context;
use tessara_api::{config::Config, db};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "tessara_api=debug".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = std::env::args().skip(1).collect::<Vec<_>>();

    if let Some(path) = command_path(
        &args,
        "validate-legacy-fixture",
        "usage: tessara-api validate-legacy-fixture <path>",
    )? {
        let fixture = fs::read_to_string(&path)
            .with_context(|| format!("failed to read legacy fixture {}", path.display()))?;
        let report = tessara_api::legacy_import::validate_legacy_fixture_str(&fixture)?;
        let is_clean = report.is_clean();
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !is_clean {
            std::process::exit(2);
        }
        return Ok(());
    }

    if let Some(path) = command_path(
        &args,
        "dry-run-legacy-fixture",
        "usage: tessara-api dry-run-legacy-fixture <path>",
    )? {
        let fixture = fs::read_to_string(&path)
            .with_context(|| format!("failed to read legacy fixture {}", path.display()))?;
        let report = tessara_api::legacy_import::dry_run_legacy_fixture_str(&fixture)?;
        let would_import = report.would_import;
        println!("{}", serde_json::to_string_pretty(&report)?);
        if !would_import {
            std::process::exit(2);
        }
        return Ok(());
    }

    let config = Config::from_env()?;
    let pool = db::connect_and_prepare(&config).await?;

    if args.iter().any(|arg| arg == "seed-demo") {
        let summary = tessara_api::demo::seed_demo(&pool).await?;
        println!("{}", serde_json::to_string_pretty(&summary)?);
        return Ok(());
    }

    if let Some(path) = command_path(
        &args,
        "import-legacy-fixture",
        "usage: tessara-api import-legacy-fixture <path>",
    )? {
        let summary = tessara_api::legacy_import::import_legacy_fixture_file(&pool, &path).await?;
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

fn command_path(args: &[String], command: &str, usage: &str) -> anyhow::Result<Option<PathBuf>> {
    let Some(position) = args.iter().position(|arg| arg == command) else {
        return Ok(None);
    };
    let path = args
        .get(position + 1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!(usage.to_string()))?;

    Ok(Some(path))
}
