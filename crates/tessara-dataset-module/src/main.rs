use std::{env, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::postgres::PgPoolOptions;
use tessara_dataset_module::{
    DATASET_PROVIDER_ENDPOINTS_ENVIRONMENT, DatasetCoreVerifiers, DatasetModuleState,
    DatasetServiceEndpoints, DatasetValidationFaultControl, router,
};
use tessara_module_contract::{
    MODULE_SERVICE_IDENTITIES_ENVIRONMENT, ModuleServiceIdentityRegistryV1,
    ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
};
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
    let core_public_key: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY")
                .context("TESSARA_CORE_AUTHORIZATION_PUBLIC_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Core authorization public key must contain 32 bytes"))?;
    let core_key_id = env::var("TESSARA_CORE_AUTHORIZATION_KEY_ID")
        .unwrap_or_else(|_| "core-development-v1".into());
    let authorization_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_key_id,
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        core_public_key,
    )?;
    let core_service_request_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_key_id,
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        core_public_key,
    )?;
    let owner_bootstrap_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_key_id,
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        core_public_key,
    )?;
    let bootstrap_validation_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_key_id,
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        core_public_key,
    )?;
    let shell_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_key_id,
        ProtocolSignaturePurposeV1::ShellContext,
        core_public_key,
    )?;
    let service_secret: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_DATASET_SERVICE_SIGNING_KEY")
                .context("TESSARA_DATASET_SERVICE_SIGNING_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Dataset service signing key must contain 32 bytes"))?;
    let service_request_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.datasets",
        env::var("TESSARA_DATASET_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "dataset-development-v1".into()),
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        service_secret,
    )?);
    let bootstrap_receipt_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.datasets",
        env::var("TESSARA_DATASET_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "dataset-development-v1".into()),
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        service_secret,
    )?);
    let service_identity_registry = ModuleServiceIdentityRegistryV1::from_json(
        &env::var(MODULE_SERVICE_IDENTITIES_ENVIRONMENT)
            .with_context(|| format!("{MODULE_SERVICE_IDENTITIES_ENVIRONMENT} is required"))?,
    )
    .context("module service identity registry is invalid")?;
    let service_endpoints = DatasetServiceEndpoints::from_json(
        &env::var("TESSARA_CORE_INTERNAL_URL").context("TESSARA_CORE_INTERNAL_URL is required")?,
        &env::var(DATASET_PROVIDER_ENDPOINTS_ENVIRONMENT)
            .with_context(|| format!("{DATASET_PROVIDER_ENDPOINTS_ENVIRONMENT} is required"))?,
    )
    .context("Dataset service endpoint configuration is invalid")?;
    let address: SocketAddr = env::var("DATASET_MODULE_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8093".into())
        .parse()?;
    let validation_fault_control = DatasetValidationFaultControl::from_environment()
        .context("Dataset validation-profile failure control is invalid")?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Dataset module listening");
    axum::serve(
        listener,
        router(DatasetModuleState::new(
            pool,
            DatasetCoreVerifiers {
                authorization: authorization_verifier,
                owner_bootstrap: owner_bootstrap_verifier,
                service_request: core_service_request_verifier,
                bootstrap_validation: bootstrap_validation_verifier,
                shell: shell_verifier,
            },
            service_identity_registry,
            service_request_signer,
            bootstrap_receipt_signer,
            service_endpoints,
            validation_fault_control,
        )?)
        .layer(TraceLayer::new_for_http()),
    )
    .with_graceful_shutdown(shutdown())
    .await?;
    Ok(())
}

async fn shutdown() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        let mut terminate = signal(SignalKind::terminate())
            .expect("SIGTERM handler must be available for the Dataset runtime");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = terminate.recv() => {}
        }
    }

    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}
