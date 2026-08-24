use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use tessara_module_runtime::{
    CoreVerifiers, initialize_tracing, serve, shutdown_signal, standard_http_router,
};
use tessara_response_module::{ResponseRuntime, router};

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
    let runtime = Arc::new(ResponseRuntime::new(
        pool,
        CoreVerifiers::from_environment()?,
    ));
    let address: SocketAddr = env::var("RESPONSE_MODULE_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8094".into())
        .parse()?;
    serve(
        address,
        standard_http_router(router(runtime)),
        shutdown_signal(),
    )
    .await
}
