use std::{collections::BTreeMap, env, net::SocketAddr, path::PathBuf};

use anyhow::Context;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use tessara_composition::{
    ApplicationLockfileV1, ApplyAuthorizationV1, ApplyOperationKindV1, BootstrapInputV1,
    BootstrapReceiptV1, CompositionOperationV1, InstallationReceiptV1, MaterializationActionV1,
    MaterializationPlanV1, OwnerBootstrapRequestV1, OwnerBootstrapResponseV1,
};
use tessara_module_contract::{ArtifactDigest, ProtocolSignaturePurposeV1, SignedEnvelopeV1};
use tessara_supervisor::{
    EmergencyOverrideV1, MaterializationAdapter, RecordingAdapter, SupervisorError,
    SupervisorLedger,
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    ledger: SupervisorLedger,
    client: reqwest::Client,
    core_url: String,
    module_urls: BTreeMap<String, String>,
    artifact_images: BTreeMap<String, String>,
    projection_token: String,
    module_control_key: String,
    local_cas_root: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
    let ledger_path = env::var_os("TESSARA_SUPERVISOR_LEDGER")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/sprint-6f-supervisor/ledger.sqlite3"));
    if let Some(parent) = ledger_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let ledger = SupervisorLedger::open(ledger_path)?;
    if let Ok(value) = env::var("TESSARA_INSTALLATION_ID") {
        ledger.initialize_installation(value.parse()?, chrono::Utc::now())?;
    }
    register_environment_trust_anchors(&ledger)?;
    let address: SocketAddr = env::var("TESSARA_SUPERVISOR_ADDR")
        .unwrap_or_else(|_| "127.0.0.1:8090".into())
        .parse()?;
    let listener = tokio::net::TcpListener::bind(address).await?;
    let app = Router::new()
        .route("/health/live", get(|| async { StatusCode::NO_CONTENT }))
        .route("/health/ready", get(|| async { StatusCode::NO_CONTENT }))
        .route("/v1/apply", post(apply))
        .route("/v1/operations/{operation_id}", get(operation))
        .route("/v1/receipts/current", get(current_receipt))
        .route("/v1/emergency-overrides", get(emergency_overrides))
        .with_state(AppState {
            ledger,
            client: reqwest::Client::new(),
            core_url: env::var("TESSARA_CORE_INTERNAL_URL")
                .unwrap_or_else(|_| "http://core:8080".into()),
            module_urls: env::var("TESSARA_MODULE_CONTROL_ENDPOINTS")
                .ok()
                .map(|value| serde_json::from_str(&value))
                .transpose()?
                .unwrap_or_default(),
            artifact_images: env::var("TESSARA_ARTIFACT_IMAGE_REFERENCES")
                .ok()
                .map(|value| serde_json::from_str(&value))
                .transpose()?
                .unwrap_or_default(),
            projection_token: env::var("TESSARA_SUPERVISOR_PROJECTION_TOKEN")
                .unwrap_or_else(|_| "local-supervisor-projection-token".into()),
            module_control_key: env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
                .unwrap_or_else(|_| "development-module-control-only".into()),
            local_cas_root: env::var_os("TESSARA_LOCAL_CAS_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|| PathBuf::from("/var/lib/tessara-supervisor/cas")),
        });
    axum::serve(listener, app).await.context("serve Supervisor")
}

fn register_environment_trust_anchors(ledger: &SupervisorLedger) -> anyhow::Result<()> {
    let Some(public_key) = env::var("TESSARA_SUPERVISOR_OPERATOR_PUBLIC_KEY_HEX").ok() else {
        return Ok(());
    };
    let public_key = decode_hex_32(&public_key)?;
    let issuer = env::var("TESSARA_SUPERVISOR_OPERATOR_ISSUER")
        .unwrap_or_else(|_| "tessara.local.sprint-6f".into());
    let key_id =
        env::var("TESSARA_SUPERVISOR_OPERATOR_KEY_ID").unwrap_or_else(|_| "apply-dev-v1".into());
    ledger.register_trust_anchor(
        &issuer,
        &key_id,
        "apply_authorization",
        &public_key,
        chrono::Utc::now(),
    )?;
    Ok(())
}

fn decode_hex_32(value: &str) -> anyhow::Result<[u8; 32]> {
    anyhow::ensure!(
        value.len() == 64,
        "public key must be 64 hexadecimal characters"
    );
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        *byte = u8::from_str_radix(&value[index * 2..index * 2 + 2], 16)?;
    }
    Ok(bytes)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ApplyRequestV1 {
    lockfile: ApplicationLockfileV1,
    authorization: SignedEnvelopeV1<ApplyAuthorizationV1>,
}

#[derive(Serialize)]
struct ApplyResponseV1 {
    operation: CompositionOperationV1,
    receipt: InstallationReceiptV1,
}

async fn apply(
    State(state): State<AppState>,
    Json(request): Json<ApplyRequestV1>,
) -> axum::response::Response {
    match apply_inner(&state, request).await {
        Ok(response) => (StatusCode::ACCEPTED, Json(response)).into_response(),
        Err(error) => error_response(error.to_string()),
    }
}

async fn apply_inner(state: &AppState, request: ApplyRequestV1) -> anyhow::Result<ApplyResponseV1> {
    let verifier = state.ledger.verifier_for(
        &request.authorization.issuer,
        &request.authorization.key_id,
        ProtocolSignaturePurposeV1::ApplyAuthorization,
    )?;
    let plan: &MaterializationPlanV1 = &request.lockfile.materialization_plan;
    let accepted =
        state
            .ledger
            .accept_apply(plan, &request.authorization, &verifier, chrono::Utc::now())?;
    if accepted.receipt_digest.is_some() {
        let receipt = state
            .ledger
            .current_receipt()?
            .ok_or_else(|| anyhow::anyhow!("idempotent operation receipt is missing"))?;
        project_result(
            state,
            request.lockfile.blueprint_revision,
            &request.lockfile,
            &accepted,
            &receipt,
        )
        .await?;
        return Ok(ApplyResponseV1 {
            operation: accepted,
            receipt,
        });
    }
    if let Ok(delay) = env::var("TESSARA_SUPERVISOR_APPLY_DELAY_MS") {
        if let Ok(delay) = delay.parse::<u64>() {
            tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
        }
    }
    let lockfile_digest: ArtifactDigest = tessara_composition::canonical_digest(&request.lockfile)?;
    let mut adapter = OwnerHttpAdapter::prepare(state, &request.lockfile).await?;
    let receipt = state.ledger.execute(
        accepted.operation_id,
        lockfile_digest,
        &mut adapter,
        chrono::Utc::now(),
    )?;
    if request.authorization.payload.operation == ApplyOperationKindV1::EmergencyDisable {
        let definition_id = plan
            .actions
            .iter()
            .find_map(|action| match action {
                MaterializationActionV1::SetEnablement {
                    definition_id,
                    enabled: false,
                } => Some(definition_id.clone()),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("emergency authorization has no disable target"))?;
        state
            .ledger
            .record_emergency_override(&EmergencyOverrideV1 {
                override_id: Uuid::new_v4(),
                definition_id,
                reason: request
                    .authorization
                    .payload
                    .reason
                    .clone()
                    .unwrap_or_else(|| "Emergency disable".into()),
                actor: serde_json::to_value(&request.authorization.payload.initiator)?,
                issued_at: request.authorization.payload.issued_at,
                expires_at: Some(request.authorization.payload.expires_at),
                authorization_digest: tessara_composition::canonical_digest(
                    &request.authorization,
                )?,
                reconciled_at: None,
                expired: false,
            })?;
    }
    let operation = state
        .ledger
        .operation(accepted.operation_id)?
        .ok_or_else(|| anyhow::anyhow!("completed operation is missing"))?;
    project_result(
        state,
        request.lockfile.blueprint_revision,
        &request.lockfile,
        &operation,
        &receipt,
    )
    .await?;
    Ok(ApplyResponseV1 { operation, receipt })
}

struct OwnerHttpAdapter {
    recording: RecordingAdapter,
    bootstrap_receipts: BTreeMap<String, BootstrapReceiptV1>,
    observed_artifacts: BTreeMap<String, ArtifactDigest>,
}

impl OwnerHttpAdapter {
    async fn prepare(state: &AppState, lockfile: &ApplicationLockfileV1) -> anyhow::Result<Self> {
        let mut bootstrap_receipts = BTreeMap::new();
        let mut observed_artifacts = BTreeMap::new();
        for action in &lockfile.materialization_plan.actions {
            if let MaterializationActionV1::AcquireImage { component, digest } = action {
                let image = state.artifact_images.get(component).ok_or_else(|| {
                    anyhow::anyhow!("no runtime image reference is configured for {component}")
                })?;
                let output = std::process::Command::new("docker")
                    .args(["image", "inspect", "--format={{.Id}}", image])
                    .output()
                    .context("inspect runtime image through the Docker owner adapter")?;
                anyhow::ensure!(
                    output.status.success(),
                    "Docker could not inspect runtime image {image}: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
                let observed =
                    ArtifactDigest::new(String::from_utf8(output.stdout)?.trim().to_string())?;
                anyhow::ensure!(
                    &observed == digest,
                    "observed runtime image for {component} is {observed}, not locked digest {digest}"
                );
                observed_artifacts.insert(component.clone(), observed);
            }
        }
        if let Some(input) = &lockfile.core.bootstrap {
            let receipt = invoke_bootstrap(state, lockfile, "core", input).await?;
            bootstrap_receipts.insert("core".into(), receipt);
        }
        for module in &lockfile.modules {
            if let Some(input) = &module.bootstrap {
                let receipt =
                    invoke_bootstrap(state, lockfile, &module.definition_id, input).await?;
                bootstrap_receipts.insert(module.definition_id.clone(), receipt);
            }
        }
        for action in &lockfile.materialization_plan.actions {
            if let MaterializationActionV1::SetEnablement {
                definition_id,
                enabled,
            } = action
            {
                apply_module_enablement(state, lockfile, definition_id, *enabled).await?;
            }
        }
        for owner in lockfile
            .materialization_plan
            .actions
            .iter()
            .filter_map(|action| {
                if let MaterializationActionV1::HealthGate { owner } = action {
                    Some(owner.as_str())
                } else {
                    None
                }
            })
        {
            verify_owner_health(state, owner).await?;
        }
        Ok(Self {
            recording: RecordingAdapter::default(),
            bootstrap_receipts,
            observed_artifacts,
        })
    }
}

async fn apply_module_enablement(
    state: &AppState,
    lockfile: &ApplicationLockfileV1,
    owner: &str,
    enabled: bool,
) -> anyhow::Result<()> {
    let base = state
        .module_urls
        .get(owner)
        .ok_or_else(|| anyhow::anyhow!("no owner endpoint is configured for {owner}"))?;
    let module_instance_id =
        tessara_composition::module_instance_id(lockfile.installation_id, owner);
    let status = state
        .client
        .put(format!(
            "{}/api/private/security-state",
            base.trim_end_matches('/')
        ))
        .header("x-tessara-module-control-key", &state.module_control_key)
        .json(&serde_json::json!({
            "schema_version": 1,
            "installation_id": lockfile.installation_id,
            "module_instance_id": module_instance_id,
            "authorization_revision": lockfile.blueprint_revision,
            "organization_revision": lockfile.blueprint_revision,
            "enabled": enabled,
            "document_state": if enabled { "enabled" } else { "disabled" }
        }))
        .send()
        .await?
        .status();
    anyhow::ensure!(
        status.is_success(),
        "{owner} enablement failed with HTTP {status}"
    );
    Ok(())
}

impl MaterializationAdapter for OwnerHttpAdapter {
    fn execute(
        &mut self,
        action: &MaterializationActionV1,
    ) -> Result<Option<BootstrapReceiptV1>, SupervisorError> {
        self.recording.execute(action)?;
        if let MaterializationActionV1::Bootstrap { owner, .. } = action {
            Ok(self.bootstrap_receipts.get(owner).cloned())
        } else {
            Ok(None)
        }
    }

    fn observed_artifacts(&self) -> BTreeMap<String, ArtifactDigest> {
        self.observed_artifacts.clone()
    }

    fn configuration_digests(&self) -> BTreeMap<String, ArtifactDigest> {
        self.recording.configuration_digests()
    }
}

async fn invoke_bootstrap(
    state: &AppState,
    lockfile: &ApplicationLockfileV1,
    owner: &str,
    input: &BootstrapInputV1,
) -> anyhow::Result<BootstrapReceiptV1> {
    let input_bytes = tessara_composition::acquire_bootstrap_input(input, &state.local_cas_root)?;
    let input_value: serde_json::Value = serde_json::from_slice(&input_bytes)?;
    let input_digest = tessara_composition::canonical_digest(&input_value)?;
    let expected_digest = match input {
        BootstrapInputV1::Inline { .. } => input_digest.clone(),
        BootstrapInputV1::LocalCas { digest, .. } => digest.clone(),
    };
    anyhow::ensure!(
        input_digest == expected_digest,
        "bootstrap input digest does not match its locked identity"
    );
    let request = OwnerBootstrapRequestV1 {
        installation_id: lockfile.installation_id,
        desired_revision: lockfile.blueprint_revision,
        idempotency_key: format!(
            "composition:{owner}:r{}:{input_digest}",
            lockfile.blueprint_revision
        ),
        input_digest,
        input: input_value,
    };
    let (url, header_name, header_value) = if owner == "core" {
        (
            format!(
                "{}/api/internal/composition/bootstrap/core",
                state.core_url.trim_end_matches('/')
            ),
            "x-tessara-supervisor-token",
            state.projection_token.as_str(),
        )
    } else {
        let base = state
            .module_urls
            .get(owner)
            .ok_or_else(|| anyhow::anyhow!("no owner endpoint is configured for {owner}"))?;
        (
            format!("{}/api/private/bootstrap", base.trim_end_matches('/')),
            "x-tessara-module-control-key",
            state.module_control_key.as_str(),
        )
    };
    let response = state
        .client
        .post(url)
        .header(header_name, header_value)
        .json(&request)
        .send()
        .await?;
    let status = response.status();
    let body = response.bytes().await?;
    anyhow::ensure!(
        status.is_success(),
        "{owner} bootstrap failed with HTTP {status}: {}",
        String::from_utf8_lossy(&body)
    );
    let response: OwnerBootstrapResponseV1 = serde_json::from_slice(&body)?;
    anyhow::ensure!(
        response.receipt.owner == owner,
        "bootstrap receipt owner mismatch"
    );
    anyhow::ensure!(
        response.receipt.input_digest == request.input_digest,
        "bootstrap receipt input digest mismatch"
    );
    Ok(response.receipt)
}

async fn verify_owner_health(state: &AppState, owner: &str) -> anyhow::Result<()> {
    let base = if owner == "core" {
        state.core_url.as_str()
    } else {
        state
            .module_urls
            .get(owner)
            .ok_or_else(|| anyhow::anyhow!("no health endpoint is configured for {owner}"))?
    };
    let status = state
        .client
        .get(format!("{}/health/ready", base.trim_end_matches('/')))
        .send()
        .await?
        .status();
    anyhow::ensure!(
        status.is_success(),
        "{owner} health gate failed with HTTP {status}"
    );
    Ok(())
}

async fn project_result(
    state: &AppState,
    blueprint_revision: u64,
    lockfile: &ApplicationLockfileV1,
    operation: &CompositionOperationV1,
    receipt: &InstallationReceiptV1,
) -> anyhow::Result<()> {
    for (path, body) in [
        (
            "/api/internal/composition/operations",
            serde_json::json!({"blueprint_revision": blueprint_revision, "operation": operation}),
        ),
        (
            "/api/internal/composition/receipts",
            serde_json::json!({"lockfile": lockfile, "receipt": receipt}),
        ),
    ] {
        let status = state
            .client
            .post(format!("{}{}", state.core_url.trim_end_matches('/'), path))
            .header("x-tessara-supervisor-token", &state.projection_token)
            .json(&body)
            .send()
            .await?
            .status();
        anyhow::ensure!(
            status.is_success(),
            "Core composition projection failed with HTTP {status}"
        );
    }
    Ok(())
}

async fn current_receipt(State(state): State<AppState>) -> axum::response::Response {
    match state.ledger.current_receipt() {
        Ok(Some(receipt)) => Json(receipt).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => error_response(error.to_string()),
    }
}

async fn emergency_overrides(State(state): State<AppState>) -> axum::response::Response {
    match state.ledger.emergency_overrides() {
        Ok(overrides) => Json(overrides).into_response(),
        Err(error) => error_response(error.to_string()),
    }
}

async fn operation(
    State(state): State<AppState>,
    Path(operation_id): Path<Uuid>,
) -> impl IntoResponse {
    match state.ledger.operation(operation_id) {
        Ok(Some(operation)) => Json(operation).into_response(),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => error_response(error.to_string()),
    }
}

fn error_response(message: String) -> axum::response::Response {
    #[derive(Serialize)]
    struct ErrorBody {
        code: &'static str,
        message: String,
    }
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            code: "supervisor_unavailable",
            message,
        }),
    )
        .into_response()
}
