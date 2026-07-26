use std::{
    env,
    net::SocketAddr,
    process::{Command, ExitCode},
};

use anyhow::{Context, Result, bail};
use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use tessara_installation_control::InstallationControlStore;
use tessara_module_contract::{
    AdministratorEligibilityDecisionV1, AdministratorEnrollmentClaimKindV1,
    AdministratorEnrollmentClaimStateV1, EnrollmentRedemptionResultV1, EnrollmentReservationV1,
    LocalOperatorAuthorizationV1, ProtocolSignaturePurposeV1, PurposeBoundSigningKeyV1,
    PurposeBoundVerifyingKeyV1, SignedEnvelopeV1,
};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error:#}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let command = env::args().nth(1).unwrap_or_else(|| "serve".into());
    let database_url = env::var("DATABASE_URL").context("DATABASE_URL is required")?;
    let pool = PgPoolOptions::new()
        .max_connections(8)
        .connect(&database_url)
        .await?;
    let store = InstallationControlStore::new(pool);
    match command.as_str() {
        "enrollment" => {
            let action = required_arg(2, "enrollment action")?;
            let open = env::args().any(|argument| argument == "--open");
            let (kind, operator_identity, reason) = match action.as_str() {
                "issue" => (AdministratorEnrollmentClaimKindV1::Initial, None, None),
                "recover" => {
                    let reason = option_value("--reason")
                        .or_else(|| env::var("TESSARA_RECOVERY_REASON").ok())
                        .filter(|value| !value.trim().is_empty())
                        .context("--reason is required for administrator recovery")?;
                    let operator_identity = option_value("--operator")
                        .or_else(|| env::var("TESSARA_OPERATOR_IDENTITY").ok())
                        .filter(|value| !value.trim().is_empty())
                        .context(
                            "--operator or TESSARA_OPERATOR_IDENTITY is required for recovery",
                        )?;
                    (
                        AdministratorEnrollmentClaimKindV1::Recovery,
                        Some(operator_identity),
                        Some(reason),
                    )
                }
                other => bail!("unsupported enrollment action '{other}'"),
            };
            let output = guided_enrollment(
                &store,
                kind,
                operator_identity.as_deref(),
                reason.as_deref(),
            )
            .await?;
            if open {
                let _ = open_browser(&output.enrollment_url);
            }
            print_json(&output)?;
        }
        "migrate" => store.migrate().await?,
        "serve" => serve(store).await?,
        "status" => {
            let installation_id = required_uuid_arg(2, "installation ID")?;
            print_json(&store.status(installation_id).await?)?;
        }
        "revoke" => {
            let installation_id = required_uuid_arg(2, "installation ID")?;
            print_json(&store.revoke(installation_id, Utc::now()).await?)?;
        }
        "replace" => {
            let installation_id = required_uuid_arg(2, "installation ID")?;
            print_json(&store.replace(installation_id, Utc::now()).await?)?;
        }
        "issue" => {
            let installation_id = required_uuid_arg(2, "installation ID")?;
            let kind = match required_arg(3, "claim kind")?.as_str() {
                "initial" => AdministratorEnrollmentClaimKindV1::Initial,
                "recovery" => AdministratorEnrollmentClaimKindV1::Recovery,
                other => bail!("unsupported claim kind '{other}'"),
            };
            let eligibility: SignedEnvelopeV1<AdministratorEligibilityDecisionV1> =
                serde_json::from_slice(&std::fs::read(required_arg(4, "eligibility JSON")?)?)?;
            let eligibility_verifier = verifier_from_env(
                "TESSARA_CORE_ELIGIBILITY_PUBLIC_KEY",
                ProtocolSignaturePurposeV1::EnrollmentEligibility,
                "tessara.core",
                "core-development-v1",
            )?;
            let recovery_verifier = (kind == AdministratorEnrollmentClaimKindV1::Recovery)
                .then(|| {
                    verifier_from_env(
                        "TESSARA_RECOVERY_OPERATOR_PUBLIC_KEY",
                        ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization,
                        "tessara.installation-control",
                        "recovery-operator-v1",
                    )
                })
                .transpose()?;
            let issued = store
                .issue(
                    installation_id,
                    kind,
                    eligibility,
                    &eligibility_verifier,
                    recovery_verifier.as_ref(),
                    Utc::now(),
                )
                .await?;
            print_json(&json!({
                "status": issued.status,
                "claim_secret": issued.secret.expose_once(),
                "warning": "This secret is shown once. Store it securely now."
            }))?;
        }
        "authorize-recovery" => {
            let installation_id = required_uuid_arg(2, "installation ID")?;
            let operator_identity = required_arg(3, "operator identity")?;
            let reason = required_arg(4, "reason")?;
            if operator_identity.trim().is_empty() || reason.trim().is_empty() {
                bail!("operator identity and reason are required");
            }
            let secret: [u8; 32] = URL_SAFE_NO_PAD
                .decode(
                    env::var("TESSARA_RECOVERY_OPERATOR_SIGNING_KEY")
                        .context("TESSARA_RECOVERY_OPERATOR_SIGNING_KEY is required")?,
                )?
                .try_into()
                .map_err(|_| {
                    anyhow::anyhow!(
                        "TESSARA_RECOVERY_OPERATOR_SIGNING_KEY must contain 32 base64url bytes"
                    )
                })?;
            let signer = PurposeBoundSigningKeyV1::from_secret_bytes(
                "tessara.installation-control",
                "recovery-operator-v1",
                ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization,
                secret,
            )?;
            let now = Utc::now();
            print_json(&signer.sign(LocalOperatorAuthorizationV1 {
                schema_version: 1,
                operator_identity,
                reason,
                installation_id,
                nonce: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + chrono::Duration::seconds(300),
            })?)?;
        }
        other => bail!("unknown command '{other}'"),
    }
    Ok(())
}

#[derive(Deserialize)]
struct InstallationContext {
    installation_id: Uuid,
}

#[derive(Deserialize)]
struct EnrollmentHandoff {
    handoff_token: String,
}

#[derive(Serialize)]
struct GuidedEnrollmentOutput {
    status: tessara_installation_control::EnrollmentClaimStatusV1,
    claim_secret: String,
    enrollment_url: String,
    warning: &'static str,
}

async fn guided_enrollment(
    store: &InstallationControlStore,
    kind: AdministratorEnrollmentClaimKindV1,
    operator_identity: Option<&str>,
    reason: Option<&str>,
) -> Result<GuidedEnrollmentOutput> {
    let client = reqwest::Client::new();
    let core_url = env::var("TESSARA_CORE_PRIVATE_URL")
        .unwrap_or_else(|_| "http://core:8080".into())
        .trim_end_matches('/')
        .to_string();
    let shared_key = env::var("TESSARA_INSTALLATION_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-installation-control-only".into());
    let context: InstallationContext = client
        .get(format!(
            "{core_url}/api/private/installation-control/context"
        ))
        .header("x-tessara-installation-control-key", &shared_key)
        .send()
        .await
        .context("failed to reach Core installation context")?
        .error_for_status()
        .context("Core installation context is unavailable")?
        .json()
        .await
        .context("Core returned an invalid installation context")?;

    let recovery_authorization = if kind == AdministratorEnrollmentClaimKindV1::Recovery {
        let now = Utc::now();
        Some(
            recovery_operator_signer()?.sign(LocalOperatorAuthorizationV1 {
                schema_version: 1,
                operator_identity: operator_identity.unwrap_or_default().trim().into(),
                reason: reason.unwrap_or_default().trim().into(),
                installation_id: context.installation_id,
                nonce: Uuid::new_v4(),
                issued_at: now,
                expires_at: now + chrono::Duration::seconds(300),
            })?,
        )
    } else {
        None
    };
    let eligibility: SignedEnvelopeV1<AdministratorEligibilityDecisionV1> = client
        .post(format!(
            "{core_url}/api/private/installation-control/administrator-eligibility"
        ))
        .header("x-tessara-installation-control-key", &shared_key)
        .json(&json!({
            "schema_version": 1,
            "installation_id": context.installation_id,
            "kind": kind,
            "recovery_authorization": recovery_authorization,
        }))
        .send()
        .await
        .context("failed to request administrator eligibility from Core")?
        .error_for_status()
        .context("administrator enrollment is not currently eligible")?
        .json()
        .await
        .context("Core returned an invalid eligibility decision")?;

    let eligibility_verifier = verifier_from_env(
        "TESSARA_CORE_ELIGIBILITY_PUBLIC_KEY",
        ProtocolSignaturePurposeV1::EnrollmentEligibility,
        "tessara.core",
        "core-development-v1",
    )?;
    let recovery_verifier = (kind == AdministratorEnrollmentClaimKindV1::Recovery)
        .then(|| {
            verifier_from_env(
                "TESSARA_RECOVERY_OPERATOR_PUBLIC_KEY",
                ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization,
                "tessara.installation-control",
                "recovery-operator-v1",
            )
        })
        .transpose()?;
    eligibility_verifier.verify(&eligibility)?;
    eligibility.payload.validate_for(
        context.installation_id,
        kind,
        Utc::now(),
        recovery_verifier.as_ref(),
    )?;
    if store
        .status(context.installation_id)
        .await?
        .is_some_and(|status| {
            matches!(
                status.state,
                AdministratorEnrollmentClaimStateV1::Issued
                    | AdministratorEnrollmentClaimStateV1::Reserved
            )
        })
    {
        store.replace(context.installation_id, Utc::now()).await?;
    }
    let issued = store
        .issue(
            context.installation_id,
            kind,
            eligibility,
            &eligibility_verifier,
            recovery_verifier.as_ref(),
            Utc::now(),
        )
        .await?;
    let handoff: EnrollmentHandoff = client
        .post(format!(
            "{core_url}/api/private/installation-control/enrollment-handoffs"
        ))
        .header("x-tessara-installation-control-key", &shared_key)
        .json(&json!({
            "schema_version": 1,
            "installation_id": issued.status.installation_id,
            "claim_id": issued.status.claim_id,
            "generation": issued.status.generation,
            "claim_kind": issued.status.kind,
        }))
        .send()
        .await
        .context("failed to create the enrollment browser handoff")?
        .error_for_status()
        .context("Core rejected the enrollment browser handoff")?
        .json()
        .await
        .context("Core returned an invalid enrollment browser handoff")?;
    let public_url = env::var("TESSARA_PUBLIC_URL")
        .unwrap_or_else(|_| "http://127.0.0.1:8080".into())
        .trim_end_matches('/')
        .to_string();
    Ok(GuidedEnrollmentOutput {
        status: issued.status,
        claim_secret: issued.secret.expose_once(),
        enrollment_url: format!("{public_url}/enrollment?handoff={}", handoff.handoff_token),
        warning: "This secret is shown once. Enter it in the opened enrollment page; it cannot be retrieved later.",
    })
}

fn recovery_operator_signer() -> Result<PurposeBoundSigningKeyV1> {
    let secret: [u8; 32] = URL_SAFE_NO_PAD
        .decode(
            env::var("TESSARA_RECOVERY_OPERATOR_SIGNING_KEY")
                .context("TESSARA_RECOVERY_OPERATOR_SIGNING_KEY is required")?,
        )?
        .try_into()
        .map_err(|_| {
            anyhow::anyhow!("TESSARA_RECOVERY_OPERATOR_SIGNING_KEY must contain 32 base64url bytes")
        })?;
    PurposeBoundSigningKeyV1::from_secret_bytes(
        "tessara.installation-control",
        "recovery-operator-v1",
        ProtocolSignaturePurposeV1::RecoveryOperatorAuthorization,
        secret,
    )
    .map_err(Into::into)
}

fn open_browser(url: &str) -> Result<()> {
    let status = if cfg!(target_os = "windows") {
        Command::new("cmd").args(["/C", "start", "", url]).status()
    } else if cfg!(target_os = "macos") {
        Command::new("open").arg(url).status()
    } else {
        Command::new("xdg-open").arg(url).status()
    }
    .context("failed to start the system browser")?;
    if !status.success() {
        bail!("the system browser did not accept the enrollment URL");
    }
    Ok(())
}

#[derive(Clone)]
struct ServerState {
    store: InstallationControlStore,
    redemption_verifier: PurposeBoundVerifyingKeyV1,
}

async fn serve(store: InstallationControlStore) -> Result<()> {
    let redemption_verifier = verifier_from_env(
        "TESSARA_CORE_REDEMPTION_PUBLIC_KEY",
        ProtocolSignaturePurposeV1::EnrollmentRedemption,
        "tessara.core",
        "core-development-v1",
    )?;
    let state = ServerState {
        store,
        redemption_verifier,
    };
    let app = Router::new()
        .route("/v1/status/{installation_id}", get(status))
        .route("/v1/reservations", post(reserve))
        .route("/v1/redemptions", post(consume))
        .route("/health/live", get(|| async { StatusCode::NO_CONTENT }))
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let address: SocketAddr = env::var("INSTALLATION_CONTROL_BIND_ADDR")
        .unwrap_or_else(|_| "0.0.0.0:8075".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(%address, "installation control listening on the private network");
    axum::serve(listener, app).await?;
    Ok(())
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReserveInput {
    schema_version: u16,
    installation_id: Uuid,
    claim_id: Uuid,
    generation: u32,
    claim_secret: String,
    reservation_id: Uuid,
}

async fn reserve(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(input): Json<ReserveInput>,
) -> Result<Json<EnrollmentReservationV1>, PublicError> {
    require_core(&headers)?;
    if input.schema_version != 1 {
        return Err(PublicError);
    }
    let now = Utc::now();
    let status = state
        .store
        .reserve(
            input.installation_id,
            input.claim_id,
            input.generation,
            &input.claim_secret,
            input.reservation_id,
            now,
        )
        .await
        .map_err(|_| PublicError)?;
    Ok(Json(EnrollmentReservationV1 {
        schema_version: 1,
        installation_id: status.installation_id,
        claim_id: status.claim_id,
        generation: status.generation,
        reservation_id: status.reservation_id.ok_or(PublicError)?,
        reserved_at: now,
        expires_at: status.reservation_expires_at.ok_or(PublicError)?,
    }))
}

async fn consume(
    State(state): State<ServerState>,
    headers: HeaderMap,
    Json(result): Json<SignedEnvelopeV1<EnrollmentRedemptionResultV1>>,
) -> Result<Json<tessara_installation_control::EnrollmentClaimStatusV1>, PublicError> {
    require_core(&headers)?;
    state
        .store
        .consume_signed(result, &state.redemption_verifier)
        .await
        .map(Json)
        .map_err(|_| PublicError)
}

async fn status(
    State(state): State<ServerState>,
    headers: HeaderMap,
    axum::extract::Path(installation_id): axum::extract::Path<Uuid>,
) -> Result<Json<serde_json::Value>, PublicError> {
    require_core(&headers)?;
    Ok(Json(
        json!({"status": state.store.status(installation_id).await.map_err(|_| PublicError)?}),
    ))
}

fn require_core(headers: &HeaderMap) -> Result<(), PublicError> {
    let expected = env::var("TESSARA_INSTALLATION_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-installation-control-only".into());
    if headers
        .get("x-tessara-installation-control-key")
        .and_then(|value| value.to_str().ok())
        == Some(expected.as_str())
    {
        Ok(())
    } else {
        Err(PublicError)
    }
}

struct PublicError;

impl IntoResponse for PublicError {
    fn into_response(self) -> axum::response::Response {
        (
            StatusCode::NOT_FOUND,
            Json(json!({
                "code": "administrator_enrollment_unavailable",
                "message": "Administrator enrollment is unavailable."
            })),
        )
            .into_response()
    }
}

fn verifier_from_env(
    variable: &str,
    purpose: ProtocolSignaturePurposeV1,
    issuer: &str,
    key_id: &str,
) -> Result<PurposeBoundVerifyingKeyV1> {
    let bytes: [u8; 32] = URL_SAFE_NO_PAD
        .decode(env::var(variable).with_context(|| format!("{variable} is required"))?)?
        .try_into()
        .map_err(|_| anyhow::anyhow!("{variable} must contain 32 base64url bytes"))?;
    Ok(PurposeBoundVerifyingKeyV1::from_public_bytes(
        issuer, key_id, purpose, bytes,
    )?)
}

fn required_arg(index: usize, name: &str) -> Result<String> {
    env::args()
        .nth(index)
        .with_context(|| format!("missing {name}"))
}

fn required_uuid_arg(index: usize, name: &str) -> Result<Uuid> {
    Ok(required_arg(index, name)?.parse()?)
}

fn option_value(name: &str) -> Option<String> {
    let arguments = env::args().collect::<Vec<_>>();
    arguments
        .iter()
        .position(|argument| argument == name)
        .and_then(|index| arguments.get(index + 1))
        .cloned()
}

fn print_json(value: &impl serde::Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}
