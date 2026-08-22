use std::{collections::BTreeMap, env, net::SocketAddr, sync::Arc};

use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use sqlx::postgres::PgPoolOptions;
use tessara_component_module::{
    ComponentModuleInit, ComponentModuleState, ComponentServiceEndpoints, router,
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
    let core_authorization_key_id = env::var("TESSARA_CORE_AUTHORIZATION_KEY_ID")
        .unwrap_or_else(|_| "core-development-v1".into());
    let authorization_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_authorization_key_id,
        ProtocolSignaturePurposeV1::AuthorizationGrant,
        core_public_key,
    )?;
    let bootstrap_validation_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_authorization_key_id,
        ProtocolSignaturePurposeV1::BootstrapValidationAuthorization,
        core_public_key,
    )?;
    let owner_bootstrap_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        &core_authorization_key_id,
        ProtocolSignaturePurposeV1::OwnerBootstrapAuthorization,
        core_public_key,
    )?;
    let shell_verifier = PurposeBoundVerifyingKeyV1::from_public_bytes(
        "tessara.core",
        core_authorization_key_id,
        ProtocolSignaturePurposeV1::ShellContext,
        core_public_key,
    )?;
    let service_secret: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_COMPONENT_SERVICE_SIGNING_KEY")
                .context("TESSARA_COMPONENT_SERVICE_SIGNING_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| anyhow::anyhow!("Component service signing key must contain 32 bytes"))?;
    let service_request_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.components",
        env::var("TESSARA_COMPONENT_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "component-development-v1".into()),
        ProtocolSignaturePurposeV1::ModuleServiceRequest,
        service_secret,
    )?);
    let bootstrap_receipt_signer = Arc::new(PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.components",
        env::var("TESSARA_COMPONENT_SERVICE_SIGNING_KEY_ID")
            .unwrap_or_else(|_| "component-development-v1".into()),
        ProtocolSignaturePurposeV1::OwnerBootstrapReceipt,
        service_secret,
    )?);
    let service_identity_registry = ModuleServiceIdentityRegistryV1::from_json(
        &env::var(MODULE_SERVICE_IDENTITIES_ENVIRONMENT)
            .with_context(|| format!("{MODULE_SERVICE_IDENTITIES_ENVIRONMENT} is required"))?,
    )
    .context("module service identity registry is invalid")?;
    let dataset_provider_url = dataset_provider_url(
        &env::var("TESSARA_MODULE_SERVICE_ENDPOINTS")
            .context("TESSARA_MODULE_SERVICE_ENDPOINTS is required")?,
    )?;

    let address: SocketAddr = env::var("COMPONENT_MODULE_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8092".into())
        .parse()?;
    let app = router(ComponentModuleState::new(ComponentModuleInit {
        pool,
        core_authorization_verifier: authorization_verifier,
        core_owner_bootstrap_verifier: owner_bootstrap_verifier,
        core_bootstrap_validation_verifier: bootstrap_validation_verifier,
        core_shell_verifier: shell_verifier,
        service_identity_registry,
        service_request_signer,
        bootstrap_receipt_signer,
        service_endpoints: ComponentServiceEndpoints::new(
            env::var("TESSARA_CORE_INTERNAL_URL").unwrap_or_else(|_| "http://core:8080".into()),
            dataset_provider_url,
        ),
    })?)
    .layer(TraceLayer::new_for_http());
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "Component module listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown())
        .await?;
    Ok(())
}

fn dataset_provider_url(configured_endpoints: &str) -> Result<String> {
    let module_service_endpoints: BTreeMap<String, String> =
        serde_json::from_str(configured_endpoints)
            .context("TESSARA_MODULE_SERVICE_ENDPOINTS must be a definition-keyed JSON object")?;
    let endpoint = module_service_endpoints
        .get(tessara_datasets_contract::DATASET_MODULE_DEFINITION_ID)
        .filter(|endpoint| !endpoint.trim().is_empty())
        .map(|endpoint| endpoint.trim())
        .context("the selected Dataset provider endpoint is required")?;
    let parsed = reqwest::Url::parse(endpoint)
        .context("the selected Dataset provider endpoint must be an absolute URL")?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
        anyhow::bail!("the selected Dataset provider endpoint must use HTTP or HTTPS");
    }
    Ok(endpoint.to_string())
}

async fn shutdown() {
    let _ = tokio::signal::ctrl_c().await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dataset_provider_endpoint_is_required_and_definition_keyed() {
        assert_eq!(
            dataset_provider_url(
                r#"{"tessara.datasets":"http://datasets:8093","tessara.components":"http://components:8092"}"#,
            )
            .expect("Dataset provider endpoint"),
            "http://datasets:8093"
        );
        assert!(
            dataset_provider_url(r#"{"tessara.components":"http://components:8092"}"#).is_err()
        );
        assert!(dataset_provider_url(r#"{"tessara.datasets":"   "}"#).is_err());
        assert!(dataset_provider_url(r#"{"tessara.datasets":"datasets:8093"}"#).is_err());
        assert!(dataset_provider_url(r#"{"tessara.datasets":"file:///datasets"}"#).is_err());
        assert!(dataset_provider_url("not-json").is_err());
    }
}
