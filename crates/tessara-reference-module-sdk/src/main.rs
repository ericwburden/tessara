use std::{env, net::SocketAddr, path::PathBuf, sync::Arc};

use anyhow::Result;
use tessara_module_runtime::{
    CoreVerifiers, initialize_tracing, serve, shutdown_signal, standard_http_router,
};
use tessara_reference_module_sdk::native::{ReferenceRuntime, router};

#[tokio::main]
async fn main() -> Result<()> {
    initialize_tracing();
    let runtime = Arc::new(
        ReferenceRuntime::open(
            PathBuf::from(
                env::var("TESSARA_REFERENCE_STATE_PATH")
                    .unwrap_or_else(|_| "/var/lib/tessara-reference/state.json".into()),
            ),
            CoreVerifiers::from_environment()?,
        )
        .await?,
    );
    let address: SocketAddr = env::var("TESSARA_REFERENCE_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8091".into())
        .parse()?;
    serve(
        address,
        standard_http_router(router(runtime)),
        shutdown_signal(),
    )
    .await
}
