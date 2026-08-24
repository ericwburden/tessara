use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::postgres::PgPoolOptions;
use std::{env, net::SocketAddr, sync::Arc};
use tessara_module_contract::{
    MODULE_SERVICE_IDENTITIES_ENVIRONMENT, ModuleServiceIdentityRegistryV1,
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
};
use tessara_module_runtime::{
    CoreVerifiers, initialize_tracing, serve, shutdown_signal, standard_http_router,
};
use tessara_response_module::{
    RESPONSE_PROVIDER_ENDPOINTS_ENVIRONMENT, ResponseRuntime, ResponseServiceEndpoints, router,
};

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
    let service_secret: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_RESPONSE_SERVICE_SIGNING_KEY")
                .context("TESSARA_RESPONSE_SERVICE_SIGNING_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Response service signing key must contain 32 bytes"))?;
    let service_request_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.responses",
        env::var("TESSARA_RESPONSE_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "response-development-v1".into()),
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        service_secret,
    )?);
    let endpoints = ResponseServiceEndpoints::from_json(
        &env::var("TESSARA_CORE_INTERNAL_URL").context("TESSARA_CORE_INTERNAL_URL is required")?,
        &env::var(RESPONSE_PROVIDER_ENDPOINTS_ENVIRONMENT)
            .with_context(|| format!("{RESPONSE_PROVIDER_ENDPOINTS_ENVIRONMENT} is required"))?,
    )?;
    let service_identity_registry = ModuleServiceIdentityRegistryV1::from_json(
        &env::var(MODULE_SERVICE_IDENTITIES_ENVIRONMENT)
            .with_context(|| format!("{MODULE_SERVICE_IDENTITIES_ENVIRONMENT} is required"))?,
    )
    .context("module service identity registry is invalid")?;
    let core_public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY")
                .context("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Core authorization public key must contain 32 bytes"))?;
    let core_service_request_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        env::var("TESSARA_CORE_AUTHORIZATION_KEY_ID")
            .unwrap_or_else(|_| "core-development-v1".into()),
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        core_public_key,
    )?;
    let core_owner_bootstrap_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        env::var("TESSARA_CORE_AUTHORIZATION_KEY_ID")
            .unwrap_or_else(|_| "core-development-v1".into()),
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        core_public_key,
    )?;
    let bootstrap_receipt_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.responses",
        env::var("TESSARA_RESPONSE_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "response-development-v1".into()),
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        service_secret,
    )?);
    let runtime = Arc::new(ResponseRuntime::new(
        pool,
        CoreVerifiers::from_environment()?,
        service_request_signer,
        core_service_request_verifier,
        service_identity_registry,
        endpoints,
        core_owner_bootstrap_verifier,
        bootstrap_receipt_signer,
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
