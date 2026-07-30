use std::{env, net::SocketAddr};

use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use tessara_module_runtime::{
    CoreVerifiers, initialize_tracing, serve, shutdown_signal, standard_http_router,
};
use tessara_reference_scoped_records::{ModuleState, router};

#[tokio::main]
async fn main() -> Result<()> {
    initialize_tracing();
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await?;
    if env::args().nth(1).as_deref() == Some("migrate") {
        sqlx::migrate!().run(&pool).await?;
        return Ok(());
    }
    let verifiers = CoreVerifiers::from_environment()?;
    let address: SocketAddr = env::var("SCOPED_RECORDS_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8090".into())
        .parse()?;
    let app = router(ModuleState {
        pool,
        core_authorization_verifier: verifiers.authorization,
        core_shell_verifier: verifiers.shell,
    });
    let app = standard_http_router(app);
    serve(address, app, shutdown_signal()).await
}
