use std::{env, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::postgres::PgPoolOptions;
use tessara_module_contract::{
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
};
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
    let core_public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY")
                .context("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Core authorization public key must contain 32 bytes"))?;
    let owner_bootstrap_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        env::var("TESSARA_CORE_AUTHORIZATION_KEY_ID")
            .unwrap_or_else(|_| "core-development-v1".into()),
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        core_public_key,
    )?;
    let service_secret: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_SCOPED_RECORDS_SERVICE_SIGNING_KEY")
                .context("TESSARA_SCOPED_RECORDS_SERVICE_SIGNING_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Scoped Records service signing key must contain 32 bytes"))?;
    let bootstrap_receipt_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.reference.scoped-records",
        env::var("TESSARA_SCOPED_RECORDS_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "scoped-records-development-v1".into()),
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        service_secret,
    )?);
    let address: SocketAddr = env::var("SCOPED_RECORDS_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8090".into())
        .parse()?;
    let app = router(ModuleState {
        pool,
        core_authorization_verifier: verifiers.authorization,
        core_owner_bootstrap_verifier: owner_bootstrap_verifier,
        core_shell_verifier: verifiers.shell,
        bootstrap_receipt_signer,
    });
    let app = standard_http_router(app);
    serve(address, app, shutdown_signal()).await
}
