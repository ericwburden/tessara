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
    ApplicationLockfileV1, ApplyAuthorizationV1,
    BootstrapDependencyValidationAuthorizationIssueRequestV1,
    BootstrapDependencyValidationAuthorizationIssueResponseV1, BootstrapInputV1,
    BootstrapReceiptV1, CompositionFindingV1, CompositionOperationV1, FindingSeverityV1,
    InstallationReceiptV1, MaterializationActionV1, MaterializationPlanV1, OwnerBootstrapRequestV1,
    OwnerBootstrapResponseV1,
};
use tessara_module_contract::{ArtifactDigest, ProtocolSignaturePurposeV1, SignedEnvelopeV1};
use tessara_supervisor::{
    MaterializationAdapter, RecordingAdapter, SupervisorError, SupervisorLedger,
};
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    ledger: SupervisorLedger,
    client: reqwest::Client,
    core_url: String,
    module_urls: BTreeMap<String, String>,
    artifact_images: BTreeMap<String, Vec<String>>,
    deployment: Option<ComposeDeploymentV1>,
    projection_token: String,
    module_control_key: String,
    local_cas_root: PathBuf,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(untagged)]
enum ArtifactImageReferencesV1 {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ComposeDeploymentTargetV1 {
    image_environment: String,
    migration_service: String,
    runtime_service: String,
}

#[derive(Clone, Debug)]
struct ComposeDeploymentV1 {
    compose_file: PathBuf,
    project: String,
    targets: BTreeMap<String, ComposeDeploymentTargetV1>,
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
            client: build_health_client()?,
            core_url: env::var("TESSARA_CORE_INTERNAL_URL")
                .unwrap_or_else(|_| "http://core:8080".into()),
            module_urls: env::var("TESSARA_MODULE_CONTROL_ENDPOINTS")
                .ok()
                .map(|value| serde_json::from_str(&value))
                .transpose()?
                .unwrap_or_default(),
            artifact_images: parse_artifact_image_references()?,
            deployment: compose_deployment_from_environment()?,
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

fn parse_artifact_image_references() -> anyhow::Result<BTreeMap<String, Vec<String>>> {
    let Some(value) = env::var("TESSARA_ARTIFACT_IMAGE_REFERENCES")
        .ok()
        .filter(|value| !value.trim().is_empty())
    else {
        return Ok(BTreeMap::new());
    };
    let configured: BTreeMap<String, ArtifactImageReferencesV1> = serde_json::from_str(&value)?;
    configured
        .into_iter()
        .map(|(component, references)| {
            let references = match references {
                ArtifactImageReferencesV1::One(reference) => vec![reference],
                ArtifactImageReferencesV1::Many(references) => references,
            };
            anyhow::ensure!(
                !references.is_empty()
                    && references
                        .iter()
                        .all(|reference| !reference.trim().is_empty()),
                "artifact image references for {component} must be non-empty"
            );
            Ok((component, references))
        })
        .collect()
}

fn compose_deployment_from_environment() -> anyhow::Result<Option<ComposeDeploymentV1>> {
    let Some(compose_file) = env::var_os("TESSARA_DEPLOYMENT_COMPOSE_FILE") else {
        return Ok(None);
    };
    let project = env::var("TESSARA_DEPLOYMENT_COMPOSE_PROJECT")
        .context("TESSARA_DEPLOYMENT_COMPOSE_PROJECT is required with deployment Compose")?;
    let targets = env::var("TESSARA_DEPLOYMENT_TARGETS")
        .context("TESSARA_DEPLOYMENT_TARGETS is required with deployment Compose")?;
    let targets: BTreeMap<String, ComposeDeploymentTargetV1> = serde_json::from_str(&targets)?;
    anyhow::ensure!(
        !project.trim().is_empty(),
        "deployment Compose project is empty"
    );
    anyhow::ensure!(!targets.is_empty(), "deployment Compose targets are empty");
    Ok(Some(ComposeDeploymentV1 {
        compose_file: PathBuf::from(compose_file),
        project,
        targets,
    }))
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
    if let Ok(delay) = env::var("TESSARA_SUPERVISOR_APPLY_DELAY_MS")
        && let Ok(delay) = delay.parse::<u64>()
    {
        tokio::time::sleep(std::time::Duration::from_millis(delay)).await;
    }
    let lockfile_digest: ArtifactDigest = tessara_composition::canonical_digest(&request.lockfile)?;
    let mut adapter =
        match OwnerHttpAdapter::prepare(state, &request.lockfile, &request.authorization).await {
            Ok(adapter) => adapter,
            Err(error) => {
                state.ledger.fail_operation(
                    accepted.operation_id,
                    CompositionFindingV1 {
                        code: "owner_adapter_prepare_failed".into(),
                        severity: FindingSeverityV1::Error,
                        path: "/materialization".into(),
                        message: error.to_string(),
                    },
                    chrono::Utc::now(),
                )?;
                return Err(error);
            }
        };
    let receipt = state.ledger.execute(
        accepted.operation_id,
        lockfile_digest,
        &mut adapter,
        chrono::Utc::now(),
    )?;
    let operation = state
        .ledger
        .operation(accepted.operation_id)?
        .ok_or_else(|| anyhow::anyhow!("completed operation is missing"))?;
    if let Err(error) = project_result(
        state,
        request.lockfile.blueprint_revision,
        &request.lockfile,
        &operation,
        &receipt,
    )
    .await
    {
        let finding = CompositionFindingV1 {
            code: "core_projection_failed".into(),
            severity: FindingSeverityV1::Error,
            path: "/projection".into(),
            message: error.to_string(),
        };
        state.ledger.rollback_projection_failure(
            accepted.operation_id,
            finding,
            chrono::Utc::now(),
        )?;
        if let Some(failed) = state.ledger.operation(accepted.operation_id)? {
            let _ = project_operation(state, request.lockfile.blueprint_revision, &failed).await;
        }
        return Err(error);
    }
    Ok(ApplyResponseV1 { operation, receipt })
}

struct OwnerHttpAdapter {
    recording: RecordingAdapter,
    bootstrap_receipts: BTreeMap<String, BootstrapReceiptV1>,
    observed_artifacts: BTreeMap<String, ArtifactDigest>,
}

impl OwnerHttpAdapter {
    async fn prepare(
        state: &AppState,
        lockfile: &ApplicationLockfileV1,
        apply_authorization: &SignedEnvelopeV1<ApplyAuthorizationV1>,
    ) -> anyhow::Result<Self> {
        let mut available_bootstrap_receipts = state
            .ledger
            .current_receipt()?
            .map(|receipt| {
                receipt
                    .bootstrap_receipts
                    .into_iter()
                    .map(|receipt| (receipt.owner.clone(), receipt))
                    .collect::<BTreeMap<_, _>>()
            })
            .unwrap_or_default();
        let mut bootstrap_receipts = BTreeMap::new();
        let mut observed_artifacts = BTreeMap::new();
        let mut deployment_images = BTreeMap::new();
        for action in &lockfile.materialization_plan.actions {
            if let MaterializationActionV1::AcquireImage { component, digest } = action {
                let images = state.artifact_images.get(component).ok_or_else(|| {
                    anyhow::anyhow!("no runtime image reference is configured for {component}")
                })?;
                let mut matched = None;
                for image in images {
                    let output = std::process::Command::new("docker")
                        .args(["image", "inspect", "--format={{.Id}}", image])
                        .output()
                        .context("inspect runtime image through the Docker owner adapter")?;
                    if !output.status.success() {
                        continue;
                    }
                    let observed =
                        ArtifactDigest::new(String::from_utf8(output.stdout)?.trim().to_string())?;
                    if &observed == digest {
                        matched = Some((image.clone(), observed));
                        break;
                    }
                }
                let (image, observed) = matched.ok_or_else(|| {
                    anyhow::anyhow!(
                        "no configured runtime image for {component} matches locked digest {digest}"
                    )
                })?;
                deployment_images.insert(component.clone(), image);
                observed_artifacts.insert(component.clone(), observed);
            }
        }
        // Project module security state before owner bootstrap. Component and
        // Dashboard bootstrap validate their installation/instance boundary
        // against this state, while the public gateway remains offline until
        // the complete materialization and health gates succeed.
        for action in &lockfile.materialization_plan.actions {
            if let MaterializationActionV1::SetEnablement {
                definition_id,
                enabled,
            } = action
            {
                apply_module_enablement(state, lockfile, definition_id, *enabled).await?;
            }
        }

        for action in &lockfile.materialization_plan.actions {
            match action {
                MaterializationActionV1::Migrate { owner, .. } => {
                    if let Some(image) = deployment_images.get(owner) {
                        run_compose_migration(state, owner, image)?;
                    }
                }
                MaterializationActionV1::Bootstrap { owner, .. } => {
                    let input = bootstrap_input(lockfile, owner).ok_or_else(|| {
                        anyhow::anyhow!(
                            "materialization plan bootstraps {owner} without locked input"
                        )
                    })?;
                    let receipt = invoke_bootstrap(
                        state,
                        lockfile,
                        apply_authorization,
                        owner,
                        input,
                        &available_bootstrap_receipts,
                    )
                    .await?;
                    available_bootstrap_receipts.insert(owner.clone(), receipt.clone());
                    bootstrap_receipts.insert(owner.clone(), receipt);
                }
                MaterializationActionV1::HealthGate { owner } => {
                    if let Some(image) = deployment_images.get(owner) {
                        run_compose_runtime_switch(state, owner, image)?;
                    }
                    verify_owner_health(state, owner).await?;
                }
                _ => {}
            }
        }
        Ok(Self {
            recording: RecordingAdapter::default(),
            bootstrap_receipts,
            observed_artifacts,
        })
    }
}

fn bootstrap_input<'a>(
    lockfile: &'a ApplicationLockfileV1,
    owner: &str,
) -> Option<&'a BootstrapInputV1> {
    if owner == "core" {
        return lockfile.core.bootstrap.as_ref();
    }
    lockfile
        .modules
        .iter()
        .find(|module| module.definition_id == owner)
        .and_then(|module| module.bootstrap.as_ref())
}

fn compose_target<'a>(
    state: &'a AppState,
    owner: &str,
) -> anyhow::Result<(&'a ComposeDeploymentV1, &'a ComposeDeploymentTargetV1)> {
    let deployment = state.deployment.as_ref().ok_or_else(|| {
        anyhow::anyhow!("no deployment adapter is configured for changed owner {owner}")
    })?;
    let target = deployment.targets.get(owner).ok_or_else(|| {
        anyhow::anyhow!("no Compose deployment target is configured for changed owner {owner}")
    })?;
    Ok((deployment, target))
}

fn run_compose_migration(state: &AppState, owner: &str, image: &str) -> anyhow::Result<()> {
    let (deployment, target) = compose_target(state, owner)?;
    run_compose(
        deployment,
        target,
        image,
        &[
            "run",
            "--rm",
            "--no-deps",
            target.migration_service.as_str(),
        ],
        "migration",
    )
}

fn run_compose_runtime_switch(state: &AppState, owner: &str, image: &str) -> anyhow::Result<()> {
    let (deployment, target) = compose_target(state, owner)?;
    run_compose(
        deployment,
        target,
        image,
        &[
            "up",
            "-d",
            "--no-deps",
            "--no-build",
            target.runtime_service.as_str(),
        ],
        "runtime switch",
    )
}

fn run_compose(
    deployment: &ComposeDeploymentV1,
    target: &ComposeDeploymentTargetV1,
    image: &str,
    action: &[&str],
    description: &str,
) -> anyhow::Result<()> {
    let compose_file = deployment
        .compose_file
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("deployment Compose path is not UTF-8"))?;
    let mut command = std::process::Command::new("docker");
    command.args([
        "compose",
        "--ansi",
        "never",
        "--project-name",
        deployment.project.as_str(),
        "--file",
        compose_file,
        "--profile",
        "reference",
    ]);
    command.args(action);
    command.env(&target.image_environment, image);
    let output = command
        .output()
        .with_context(|| format!("run Compose {description}"))?;
    anyhow::ensure!(
        output.status.success(),
        "Compose {description} failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    Ok(())
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
    apply_authorization: &SignedEnvelopeV1<ApplyAuthorizationV1>,
    owner: &str,
    input: &BootstrapInputV1,
    prior_receipts: &BTreeMap<String, BootstrapReceiptV1>,
) -> anyhow::Result<BootstrapReceiptV1> {
    let mut request = prepare_bootstrap_request(
        BootstrapRequestContext {
            installation_id: lockfile.installation_id,
            desired_revision: lockfile.blueprint_revision,
            apply_sequence: apply_authorization.payload.apply_sequence,
            target_plan_digest: apply_authorization.payload.target_plan_digest.clone(),
        },
        owner,
        input,
        prior_receipts,
        &state.local_cas_root,
    )?;
    if owner != "core" {
        request.dependency_validation = issue_bootstrap_dependency_authorization(
            state,
            owner,
            &request.input_digest,
            request.desired_revision,
            request.apply_sequence,
            apply_authorization,
        )
        .await?;
    }
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

async fn issue_bootstrap_dependency_authorization(
    state: &AppState,
    owner: &str,
    input_digest: &ArtifactDigest,
    desired_revision: u64,
    apply_sequence: u64,
    apply_authorization: &SignedEnvelopeV1<ApplyAuthorizationV1>,
) -> anyhow::Result<Option<tessara_composition::BootstrapDependencyValidationInvocationV1>> {
    let response = state
        .client
        .post(format!(
            "{}/api/internal/composition/bootstrap/dependency-authorization",
            state.core_url.trim_end_matches('/')
        ))
        .header("x-tessara-supervisor-token", &state.projection_token)
        .json(&BootstrapDependencyValidationAuthorizationIssueRequestV1 {
            installation_id: apply_authorization.payload.installation_id,
            owner_definition_id: owner.to_string(),
            input_digest: input_digest.clone(),
            desired_revision,
            apply_sequence,
            apply_authorization: apply_authorization.clone(),
        })
        .send()
        .await?;
    let status = response.status();
    let body = response.bytes().await?;
    anyhow::ensure!(
        status.is_success(),
        "{owner} bootstrap dependency authorization failed with HTTP {status}: {}",
        String::from_utf8_lossy(&body)
    );
    Ok(
        serde_json::from_slice::<BootstrapDependencyValidationAuthorizationIssueResponseV1>(&body)?
            .validation,
    )
}

struct BootstrapRequestContext {
    installation_id: Uuid,
    desired_revision: u64,
    apply_sequence: u64,
    target_plan_digest: ArtifactDigest,
}

fn prepare_bootstrap_request(
    context: BootstrapRequestContext,
    owner: &str,
    input: &BootstrapInputV1,
    prior_receipts: &BTreeMap<String, BootstrapReceiptV1>,
    local_cas_root: &std::path::Path,
) -> anyhow::Result<OwnerBootstrapRequestV1<serde_json::Value>> {
    let input_bytes = tessara_composition::acquire_bootstrap_input(input, local_cas_root)?;
    let acquired_value: serde_json::Value = serde_json::from_slice(&input_bytes)?;
    if let BootstrapInputV1::LocalCas { digest, .. } = input {
        let acquired_digest = tessara_composition::canonical_digest(&acquired_value)?;
        anyhow::ensure!(
            &acquired_digest == digest,
            "bootstrap input digest does not match its locked identity"
        );
    }
    let input_value = tessara_composition::resolve_bootstrap_receipt_bindings(
        acquired_value,
        input.receipt_bindings(),
        prior_receipts,
    )?;
    let input_digest = tessara_composition::canonical_digest(&input_value)?;
    Ok(OwnerBootstrapRequestV1 {
        installation_id: context.installation_id,
        desired_revision: context.desired_revision,
        apply_sequence: context.apply_sequence,
        target_plan_digest: context.target_plan_digest,
        idempotency_key: format!(
            "composition:{}:{owner}:r{}:{input_digest}",
            context.installation_id, context.desired_revision
        ),
        input_digest,
        dependency_validation: None,
        input: input_value,
    })
}

fn build_health_client() -> anyhow::Result<reqwest::Client> {
    reqwest::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .context("build exact no-redirect Supervisor health client")
}

fn owner_health_path(owner: &str) -> &'static str {
    if owner == "core" {
        "/health"
    } else {
        "/health/ready"
    }
}

fn is_utf8_plain_text(content_type: Option<&str>) -> bool {
    let Some(content_type) = content_type else {
        return false;
    };
    let mut parts = content_type.split(';').map(str::trim);
    if !parts
        .next()
        .is_some_and(|media_type| media_type.eq_ignore_ascii_case("text/plain"))
    {
        return false;
    }
    parts.all(|parameter| {
        let Some((name, value)) = parameter.split_once('=') else {
            return false;
        };
        name.trim().eq_ignore_ascii_case("charset")
            && value.trim().trim_matches('"').eq_ignore_ascii_case("utf-8")
    })
}

fn validate_owner_health_response(
    owner: &str,
    status: StatusCode,
    content_type: Option<&str>,
    body: &[u8],
) -> Result<(), String> {
    if owner == "core" {
        if status != StatusCode::OK {
            return Err(format!("Core /health returned HTTP {status}"));
        }
        if !is_utf8_plain_text(content_type) {
            return Err("Core /health did not return text/plain UTF-8 content".into());
        }
        if body != b"ok" {
            return Err(format!(
                "Core /health did not return the exact two-byte body (observed {} bytes)",
                body.len()
            ));
        }
        return Ok(());
    }
    if status != StatusCode::OK && status != StatusCode::NO_CONTENT {
        return Err(format!("{owner} /health/ready returned HTTP {status}"));
    }
    if status == StatusCode::NO_CONTENT && !body.is_empty() {
        return Err(format!(
            "{owner} /health/ready did not return the exact empty body (observed {} bytes)",
            body.len()
        ));
    }
    Ok(())
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
    let url = format!("{}{}", base.trim_end_matches('/'), owner_health_path(owner));
    let mut last_status = None;
    for attempt in 1..=60 {
        match state.client.get(&url).send().await {
            Ok(response) => {
                let status = response.status();
                let content_type = response
                    .headers()
                    .get(reqwest::header::CONTENT_TYPE)
                    .and_then(|value| value.to_str().ok())
                    .map(str::to_owned);
                match response.bytes().await {
                    Ok(body) => match validate_owner_health_response(
                        owner,
                        status,
                        content_type.as_deref(),
                        &body,
                    ) {
                        Ok(()) => return Ok(()),
                        Err(error) => last_status = Some(error),
                    },
                    Err(error) => last_status = Some(error.to_string()),
                }
            }
            Err(error) => last_status = Some(error.to_string()),
        }
        if attempt < 60 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
    }
    anyhow::bail!(
        "{owner} health gate did not pass within 60 seconds ({})",
        last_status.unwrap_or_else(|| "no response".into())
    )
}

async fn project_result(
    state: &AppState,
    blueprint_revision: u64,
    lockfile: &ApplicationLockfileV1,
    operation: &CompositionOperationV1,
    receipt: &InstallationReceiptV1,
) -> anyhow::Result<()> {
    project_operation(state, blueprint_revision, operation).await?;
    let status = state
        .client
        .post(format!(
            "{}/api/internal/composition/receipts",
            state.core_url.trim_end_matches('/')
        ))
        .header("x-tessara-supervisor-token", &state.projection_token)
        .json(&serde_json::json!({"lockfile": lockfile, "receipt": receipt}))
        .send()
        .await?
        .status();
    anyhow::ensure!(
        status.is_success(),
        "Core composition receipt projection failed with HTTP {status}"
    );
    Ok(())
}

async fn project_operation(
    state: &AppState,
    blueprint_revision: u64,
    operation: &CompositionOperationV1,
) -> anyhow::Result<()> {
    let status = state
        .client
        .post(format!(
            "{}/api/internal/composition/operations",
            state.core_url.trim_end_matches('/')
        ))
        .header("x-tessara-supervisor-token", &state.projection_token)
        .json(&serde_json::json!({
            "blueprint_revision": blueprint_revision,
            "operation": operation
        }))
        .send()
        .await?
        .status();
    anyhow::ensure!(
        status.is_success(),
        "Core composition operation projection failed with HTTP {status}"
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> ArtifactDigest {
        ArtifactDigest::new(format!("sha256:{}", byte.to_string().repeat(64))).unwrap()
    }

    fn component_receipt(changed: bool) -> BootstrapReceiptV1 {
        BootstrapReceiptV1 {
            owner: "tessara.components".into(),
            schema_version: "tessara.io/component-bootstrap/v1".into(),
            input_digest: digest('1'),
            result_digest: digest('2'),
            changed,
            resource_ids: BTreeMap::from([(
                "row-count".into(),
                serde_json::json!({
                    "reference": {
                        "installation_id": "01980000-0000-7000-8000-00000000008a",
                        "owner": {
                            "kind": "module_instance",
                            "installation_id": "01980000-0000-7000-8000-00000000008a",
                            "module_instance_id": "142a1ece-f74b-85f6-8ca0-92f4a02e9409"
                        },
                        "resource_type": "tessara.components.component_version",
                        "resource_id": "01980000-0001-7000-8000-000000000001"
                    }
                })
                .to_string(),
            )]),
        }
    }

    #[test]
    fn owner_health_contract_uses_core_health_and_module_readiness_paths() {
        assert_eq!(owner_health_path("core"), "/health");
        assert_eq!(owner_health_path("tessara.components"), "/health/ready");
    }

    #[test]
    fn owner_health_contract_requires_exact_core_semantics() {
        assert!(
            validate_owner_health_response(
                "core",
                StatusCode::OK,
                Some("text/plain; charset=utf-8"),
                b"ok"
            )
            .is_ok()
        );
        for (status, content_type, body) in [
            (StatusCode::NO_CONTENT, Some("text/plain"), b"ok".as_slice()),
            (StatusCode::OK, Some("text/html; charset=utf-8"), b"ok"),
            (StatusCode::OK, Some("text/plain"), b"<html>login</html>"),
        ] {
            assert!(validate_owner_health_response("core", status, content_type, body).is_err());
        }
    }

    #[test]
    fn owner_health_contract_accepts_canonical_module_readiness_responses() {
        assert!(
            validate_owner_health_response("tessara.components", StatusCode::NO_CONTENT, None, b"")
                .is_ok()
        );
        assert!(
            validate_owner_health_response(
                "tessara.components",
                StatusCode::OK,
                Some("application/json"),
                br#"{"status":"ready"}"#
            )
            .is_ok()
        );
        assert!(
            validate_owner_health_response(
                "tessara.components",
                StatusCode::SEE_OTHER,
                Some("text/html"),
                b""
            )
            .is_err()
        );
        assert!(
            validate_owner_health_response("tessara.components", StatusCode::CREATED, None, b"")
                .is_err()
        );
        assert!(
            validate_owner_health_response(
                "tessara.components",
                StatusCode::NO_CONTENT,
                None,
                b"unexpected"
            )
            .is_err()
        );
    }

    #[tokio::test]
    async fn health_client_does_not_follow_login_redirects() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new()
                    .route(
                        "/health",
                        get(|| async { axum::response::Redirect::to("/login") }),
                    )
                    .route("/login", get(|| async { "ok" })),
            )
            .await
            .unwrap();
        });
        let response = build_health_client()
            .unwrap()
            .get(format!("http://{address}/health"))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::SEE_OTHER);
        assert_eq!(response.url().as_str(), format!("http://{address}/health"));
        server.abort();
    }

    #[test]
    fn unchanged_provider_receipt_produces_the_same_bound_consumer_request() {
        let input = BootstrapInputV1::Inline {
            schema_version: "tessara.io/dashboard-bootstrap/v2".into(),
            value: serde_json::json!({"placements":[{"component_reference":null}]}),
            receipt_bindings: vec![tessara_composition::BootstrapReceiptBindingV1 {
                target_pointer: "/placements/0/component_reference".into(),
                source_owner: "tessara.components".into(),
                resource_key: "row-count".into(),
                value_encoding: tessara_composition::BootstrapReceiptValueEncodingV1::Json,
            }],
        };
        let installation_id = Uuid::parse_str("01980000-0000-7000-8000-00000000008a").unwrap();
        let first = prepare_bootstrap_request(
            BootstrapRequestContext {
                installation_id,
                desired_revision: 1,
                apply_sequence: 1,
                target_plan_digest: digest('9'),
            },
            "tessara.dashboards",
            &input,
            &BTreeMap::from([("tessara.components".into(), component_receipt(true))]),
            std::path::Path::new("."),
        )
        .unwrap();
        let replay = prepare_bootstrap_request(
            BootstrapRequestContext {
                installation_id,
                desired_revision: 1,
                apply_sequence: 1,
                target_plan_digest: digest('9'),
            },
            "tessara.dashboards",
            &input,
            &BTreeMap::from([("tessara.components".into(), component_receipt(false))]),
            std::path::Path::new("."),
        )
        .unwrap();

        assert_eq!(first.input, replay.input);
        assert_eq!(first.input_digest, replay.input_digest);
        assert_eq!(first.idempotency_key, replay.idempotency_key);
        let other_installation = prepare_bootstrap_request(
            BootstrapRequestContext {
                installation_id: Uuid::new_v4(),
                desired_revision: 1,
                apply_sequence: 1,
                target_plan_digest: digest('9'),
            },
            "tessara.dashboards",
            &input,
            &BTreeMap::from([("tessara.components".into(), component_receipt(false))]),
            std::path::Path::new("."),
        )
        .unwrap();
        assert_ne!(first.idempotency_key, other_installation.idempotency_key);
        assert!(first.idempotency_key.contains(&installation_id.to_string()));
        assert_eq!(
            first
                .input
                .pointer("/placements/0/component_reference/reference/resource_type"),
            Some(&serde_json::json!("tessara.components.component_version"))
        );
    }
}
