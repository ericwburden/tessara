//! Independently deployed Dashboard Module boundary.
//!
//! Sprint 6C moves Dashboard persistence and product transport into this
//! process. Core supplies signed shell and authorization projections; the
//! module never receives Core browser state or reusable Core authority.

use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};

use axum::{
    Json, Router,
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::{get, post, put},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Row};
use tessara_dashboards::{
    DashboardPlacementConfigV1, GridPlacement, GridRect, encode_dashboard_placement_config,
    validate_dashboard_layout,
};
use tessara_module_contract::{
    ModuleDefinitionId, ModuleManifest, PurposeBoundSigningKeyV1, PurposeBoundVerifyingKeyV1,
    ResourceOwner, ShellContextV2, ShellContextValidationContextV2, SignedEnvelopeV1,
    derive_row_major_positions,
};
use uuid::Uuid;

mod composition;
mod dependencies;
mod documents;
mod product;

pub const MODULE_DEFINITION_ID: &str = "tessara.dashboards";
pub const READ_CAPABILITY: &str = "dashboards:read";
pub const MANAGE_CAPABILITY: &str = "dashboards:manage";
pub const COMPONENT_BINDING_KEY: &str = "tessara.dashboards.component-version";
pub const COMPONENT_CONTRACT_ID: &str = "tessara.components.component-version";
pub const MODULE_RELEASE_VERSION: &str = "3.0.2";

const COMPONENT_PROVIDER_REQUEST_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct DashboardModuleState {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_owner_bootstrap_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
    pub service_request_signer: Arc<PurposeBoundSigningKeyV1>,
    pub bootstrap_receipt_signer: Arc<PurposeBoundSigningKeyV1>,
    pub(crate) service_client: reqwest::Client,
    pub(crate) core_internal_url: String,
    pub(crate) component_provider_url: String,
}

pub struct DashboardModuleInit {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_owner_bootstrap_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
    pub service_request_signer: Arc<PurposeBoundSigningKeyV1>,
    pub bootstrap_receipt_signer: Arc<PurposeBoundSigningKeyV1>,
    pub core_internal_url: String,
    pub component_provider_url: String,
}

impl DashboardModuleState {
    pub fn new(init: DashboardModuleInit) -> Result<Self, reqwest::Error> {
        let DashboardModuleInit {
            pool,
            core_authorization_verifier,
            core_owner_bootstrap_verifier,
            core_shell_verifier,
            service_request_signer,
            bootstrap_receipt_signer,
            core_internal_url,
            component_provider_url,
        } = init;
        Ok(Self {
            pool,
            core_authorization_verifier,
            core_owner_bootstrap_verifier,
            core_shell_verifier,
            service_request_signer,
            bootstrap_receipt_signer,
            service_client: module_service_client_with_timeout(COMPONENT_PROVIDER_REQUEST_TIMEOUT)?,
            core_internal_url: core_internal_url.trim_end_matches('/').to_string(),
            component_provider_url: component_provider_url.trim_end_matches('/').to_string(),
        })
    }
}

fn module_service_client_with_timeout(
    timeout: Duration,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().timeout(timeout).build()
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardConfigurationV1 {
    pub schema_version: u16,
    pub display_label: String,
    pub default_page_size: u16,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigurationFindingV1 {
    pub code: &'static str,
    pub field: &'static str,
    pub message: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ConfigurationValidationV1 {
    pub schema_version: u16,
    pub valid: bool,
    pub normalized: Option<DashboardConfigurationV1>,
    pub findings: Vec<ConfigurationFindingV1>,
}

pub fn validate_configuration(input: &DashboardConfigurationV1) -> ConfigurationValidationV1 {
    let label = input.display_label.trim();
    let mut findings = Vec::new();
    if input.schema_version != 1 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.schema_version.unsupported",
            field: "schema_version",
            message: "Only Dashboard configuration schema v1 is supported.",
        });
    }
    if label.is_empty() {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.display_label.required",
            field: "display_label",
            message: "Display label is required.",
        });
    } else if label.chars().count() > 80 {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.display_label.too_long",
            field: "display_label",
            message: "Display label must contain at most 80 characters.",
        });
    }
    if !(10..=100).contains(&input.default_page_size) {
        findings.push(ConfigurationFindingV1 {
            code: "configuration.default_page_size.out_of_range",
            field: "default_page_size",
            message: "Default page size must be between 10 and 100.",
        });
    }
    ConfigurationValidationV1 {
        schema_version: 1,
        valid: findings.is_empty(),
        normalized: findings.is_empty().then(|| DashboardConfigurationV1 {
            schema_version: 1,
            display_label: label.to_string(),
            default_page_size: input.default_page_size,
        }),
        findings,
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SecurityStateInput {
    schema_version: u16,
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: u64,
    organization_revision: u64,
    enabled: bool,
    document_state: String,
}

#[derive(FromRow)]
struct SecurityState {
    installation_id: Uuid,
    module_instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
    enabled: bool,
    document_state: String,
    updated_at: DateTime<Utc>,
}

pub fn router(state: DashboardModuleState) -> Router {
    Router::new()
        .route(
            "/api/configuration/validate",
            post(validate_configuration_api),
        )
        .merge(documents::routes())
        .route(
            "/api/configuration",
            get(get_configuration).put(put_configuration),
        )
        .route("/api/private/security-state", put(update_security_state))
        .route("/api/private/bootstrap", post(apply_bootstrap))
        .route("/api/manifest", get(get_manifest))
        .route(
            "/_tessara/modules/tessara.dashboards/{release}/{digest}/{asset}",
            get(dashboard_asset),
        )
        .merge(product::routes())
        .merge(composition::routes())
        .merge(dependencies::routes())
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/api/diagnostics", get(diagnostics))
        .with_state(state)
}

pub fn manifest() -> ModuleManifest {
    serde_json::from_str(include_str!("../manifest.json"))
        .expect("Dashboard manifest must remain valid")
}

async fn get_manifest(headers: HeaderMap) -> Result<Json<ModuleManifest>, DashboardModuleError> {
    require_private_key(&headers)?;
    Ok(Json(manifest()))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardBootstrapV2 {
    pub schema_version: String,
    pub external_key: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub scope_node_id: Uuid,
    pub placements: Vec<DashboardBootstrapPlacementV2>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DashboardBootstrapPlacementV2 {
    pub placement_key: String,
    pub component_reference: tessara_components_contract::ComponentVersionReference,
    pub column: u16,
    pub row: u16,
    pub width: u16,
    pub height: u16,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PreparedDashboardBootstrapPlacement {
    placement_id: Uuid,
    position: i32,
    config: Value,
}

async fn apply_bootstrap(
    State(state): State<DashboardModuleState>,
    request: Request,
) -> Result<Json<tessara_composition::OwnerBootstrapResponseV1>, DashboardModuleError> {
    require_private_key(request.headers())?;
    require_exact_json(request.headers())?;
    let body = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| {
            DashboardModuleError::BadRequest("Dashboard bootstrap payload is too large".into())
        })?;
    let request: tessara_composition::OwnerBootstrapRequestV1<DashboardBootstrapV2> =
        serde_json::from_slice(&body).map_err(|_| {
            DashboardModuleError::BadRequest("Dashboard bootstrap payload is invalid".into())
        })?;
    if request.input.schema_version != "tessara.io/dashboard-bootstrap/v2"
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?
    {
        return Err(DashboardModuleError::BadRequest(
            "Dashboard bootstrap contract or digest is invalid".into(),
        ));
    }
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DashboardModuleError::Unavailable("security state unavailable".into()))?;
    if security.installation_id != request.installation_id {
        return Err(DashboardModuleError::BadRequest(
            "Dashboard bootstrap belongs to another installation".into(),
        ));
    }
    let owner = tessara_module_contract::AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
            .map_err(|error| DashboardModuleError::Internal(error.to_string()))?,
    };
    request
        .validate_authorization_for(
            &state.core_owner_bootstrap_verifier,
            &owner,
            MODULE_DEFINITION_ID,
            Utc::now(),
        )
        .map_err(|_| DashboardModuleError::Forbidden)?;
    let prepared_placements =
        validate_dashboard_bootstrap_input(request.installation_id, &request.input)?;
    let dashboard_id = tessara_composition::owner_resource_id(
        request.installation_id,
        MODULE_DEFINITION_ID,
        "dashboard",
        &request.input.external_key,
    );
    if let Some((digest, receipt)) = sqlx::query_as::<_, (String, Value)>(
        "SELECT input_digest,receipt FROM dashboard_bootstrap_receipts WHERE idempotency_key=$1",
    )
    .bind(&request.idempotency_key)
    .fetch_optional(&state.pool)
    .await?
    {
        if digest != request.input_digest.to_string() {
            return Err(DashboardModuleError::Conflict(
                "Bootstrap idempotency key was reused with different input".into(),
            ));
        }
        let mut response: tessara_composition::OwnerBootstrapResponseV1 =
            serde_json::from_value(receipt)
                .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
        response.receipt.changed = false;
        response.signed_receipt = state
            .bootstrap_receipt_signer
            .sign(response.receipt.clone())
            .map_err(|error| DashboardModuleError::Internal(error.to_string()))?;
        return Ok(Json(response));
    }
    let mut transaction = state.pool.begin().await?;
    // The composition bootstrap may introduce the Core scope node in the same
    // apply operation. Seed the module-owned projection before linking the
    // dashboard; a later Core organization projection enriches this row.
    sqlx::query("INSERT INTO dashboard_organization_nodes(node_id,node_name,node_type_name,parent_node_id,node_path,active,projection_revision) VALUES($1,$2,'Organization',NULL,$3,true,$4) ON CONFLICT(node_id) DO NOTHING")
        .bind(request.input.scope_node_id)
        .bind("Composition bootstrap scope")
        .bind(format!("/{}", request.input.scope_node_id))
        .bind(request.desired_revision as i64)
        .execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO dashboards(id,name,description,authority_revision) VALUES($1,$2,$3,2) ON CONFLICT(id) DO UPDATE SET name=EXCLUDED.name,description=EXCLUDED.description,authority_revision=GREATEST(dashboards.authority_revision,2),updated_at=now()")
        .bind(dashboard_id).bind(request.input.name.trim()).bind(&request.input.description)
        .execute(&mut *transaction).await?;
    sqlx::query("INSERT INTO dashboard_scope_nodes(dashboard_id,node_id) VALUES($1,$2) ON CONFLICT DO NOTHING")
        .bind(dashboard_id).bind(request.input.scope_node_id)
        .execute(&mut *transaction).await?;
    for (placement, prepared) in request.input.placements.iter().zip(&prepared_placements) {
        let reference = placement.component_reference.reference().clone();
        sqlx::query("INSERT INTO dashboard_placements(id,dashboard_id,component_reference,position,config) VALUES($1,$2,$3,$4,$5) ON CONFLICT(id) DO UPDATE SET component_reference=EXCLUDED.component_reference,position=EXCLUDED.position,config=EXCLUDED.config,updated_at=now()")
            .bind(prepared.placement_id).bind(dashboard_id)
            .bind(serde_json::to_value(reference).map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?)
            .bind(prepared.position).bind(&prepared.config).execute(&mut *transaction).await?;
    }
    let resource_ids =
        dashboard_bootstrap_resource_ids(&request.input, dashboard_id, &prepared_placements);
    let result_digest = tessara_composition::canonical_digest(&resource_ids)
        .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
    let response = tessara_composition::OwnerBootstrapResponseV1::signed(
        tessara_composition::BootstrapReceiptV1 {
            owner: MODULE_DEFINITION_ID.into(),
            schema_version: request.input.schema_version.clone(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids,
        },
        &state.bootstrap_receipt_signer,
    )
    .map_err(|error| DashboardModuleError::Internal(error.to_string()))?;
    let receipt = serde_json::to_value(&response)
        .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?;
    sqlx::query("INSERT INTO dashboard_bootstrap_receipts(idempotency_key,input_digest,desired_revision,receipt) VALUES($1,$2,$3,$4)")
        .bind(&request.idempotency_key).bind(request.input_digest.to_string())
        .bind(request.desired_revision as i64).bind(receipt).execute(&mut *transaction).await?;
    transaction.commit().await?;
    Ok(Json(response))
}

fn validate_dashboard_bootstrap_input(
    installation_id: Uuid,
    input: &DashboardBootstrapV2,
) -> Result<Vec<PreparedDashboardBootstrapPlacement>, DashboardModuleError> {
    if input.external_key.trim().is_empty()
        || input.name.trim().is_empty()
        || input.scope_node_id.is_nil()
    {
        return Err(DashboardModuleError::BadRequest(
            "Dashboard bootstrap logical key, name, and resolved scope are required".into(),
        ));
    }
    let expected_component_instance = tessara_composition::module_instance_id(
        installation_id,
        tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID,
    );
    if input.placements.iter().any(|placement| {
        let reference = placement.component_reference.reference();
        reference.installation_id() != installation_id
            || !matches!(
                reference.owner(),
                ResourceOwner::ModuleInstance {
                    installation_id: owner_installation_id,
                    module_instance_id,
                } if *owner_installation_id == installation_id
                    && *module_instance_id == expected_component_instance
            )
    }) {
        return Err(DashboardModuleError::BadRequest(
            "Dashboard bootstrap Component references must belong to the selected Component Module Instance"
                .into(),
        ));
    }

    let mut placement_keys = BTreeSet::new();
    let mut layout = Vec::with_capacity(input.placements.len());
    let mut configs = BTreeMap::new();
    for placement in &input.placements {
        let placement_key = placement.placement_key.trim();
        if placement_key.is_empty() || !placement_keys.insert(placement_key.to_string()) {
            return Err(DashboardModuleError::BadRequest(
                "Dashboard bootstrap placement keys must be non-empty and unique".into(),
            ));
        }

        // Blueprint coordinates are zero-based. Product-owned Dashboard config
        // is the canonical, one-based V1 grid contract.
        let rect = GridRect::new(
            i32::from(placement.row) + 1,
            i32::from(placement.column) + 1,
            i32::from(placement.width),
            i32::from(placement.height),
        );
        let config = DashboardPlacementConfigV1::new(None, rect)
            .and_then(|config| encode_dashboard_placement_config(&config))
            .map_err(|error| {
                DashboardModuleError::BadRequest(format!(
                    "Dashboard bootstrap layout is invalid: {error}"
                ))
            })?;
        let placement_id = tessara_composition::owner_resource_id(
            installation_id,
            MODULE_DEFINITION_ID,
            "dashboard-placement",
            placement_key,
        );
        layout.push(GridPlacement::new(placement_id, rect));
        configs.insert(placement_id, config);
    }
    validate_dashboard_layout(&layout).map_err(|error| {
        DashboardModuleError::BadRequest(format!("Dashboard bootstrap layout is invalid: {error}"))
    })?;

    let positions = derive_row_major_positions(&layout)
        .into_iter()
        .map(|(placement_id, position)| {
            i32::try_from(position)
                .map(|position| (placement_id, position))
                .map_err(|_| {
                    DashboardModuleError::BadRequest(
                        "Dashboard bootstrap placement position is out of range".into(),
                    )
                })
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;

    input
        .placements
        .iter()
        .map(|placement| {
            let placement_id = tessara_composition::owner_resource_id(
                installation_id,
                MODULE_DEFINITION_ID,
                "dashboard-placement",
                placement.placement_key.trim(),
            );
            Ok(PreparedDashboardBootstrapPlacement {
                placement_id,
                position: positions[&placement_id],
                config: configs[&placement_id].clone(),
            })
        })
        .collect()
}

fn dashboard_bootstrap_resource_ids(
    input: &DashboardBootstrapV2,
    dashboard_id: Uuid,
    prepared_placements: &[PreparedDashboardBootstrapPlacement],
) -> BTreeMap<String, String> {
    let mut resource_ids = BTreeMap::from([
        ("dashboard".into(), dashboard_id.to_string()),
        ("external_key".into(), input.external_key.trim().to_string()),
    ]);
    for (placement, prepared) in input.placements.iter().zip(prepared_placements) {
        resource_ids.insert(
            format!("placement.{}", placement.placement_key.trim()),
            prepared.placement_id.to_string(),
        );
    }
    resource_ids
}

pub(crate) async fn verified_shell_context(
    state: &DashboardModuleState,
    headers: &HeaderMap,
) -> Result<ShellContextV2, DashboardModuleError> {
    let encoded = headers
        .get("x-tessara-shell-context")
        .and_then(|value| value.to_str().ok())
        .ok_or(DashboardModuleError::Forbidden)?;
    let correlation_id = headers
        .get("x-tessara-correlation-id")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| Uuid::parse_str(value).ok())
        .ok_or(DashboardModuleError::Forbidden)?;
    let envelope: SignedEnvelopeV1<ShellContextV2> = serde_json::from_slice(
        &URL_SAFE_NO_PAD
            .decode(encoded)
            .map_err(|_| DashboardModuleError::Forbidden)?,
    )
    .map_err(|_| DashboardModuleError::Forbidden)?;
    let security = load_security_state(&state.pool)
        .await?
        .ok_or_else(|| DashboardModuleError::Unavailable("security state unavailable".into()))?;
    state
        .core_shell_verifier
        .verify(&envelope)
        .map_err(|_| DashboardModuleError::Forbidden)?;
    envelope
        .payload
        .validate_for(&ShellContextValidationContextV2 {
            installation_id: security.installation_id,
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                .map_err(|_| DashboardModuleError::Forbidden)?,
            module_instance_id: security.module_instance_id,
            correlation_id,
            now: Utc::now(),
        })
        .map_err(|_| DashboardModuleError::Forbidden)?;
    Ok(envelope.payload)
}

async fn dashboard_asset(
    axum::extract::Path((release, digest, asset)): axum::extract::Path<(String, String, String)>,
) -> Response {
    if release != MODULE_RELEASE_VERSION {
        return StatusCode::NOT_FOUND.into_response();
    }
    let (expected_digest, content_type, bytes): (&str, &str, &'static [u8]) = match asset.as_str() {
        "module-ui.css" => (
            tessara_module_ui::MODULE_UI_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_module_ui::MODULE_UI_CSS.as_bytes(),
        ),
        "dashboard.css" => (
            tessara_dashboard_ui::DASHBOARD_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_dashboard_ui::DASHBOARD_CSS.as_bytes(),
        ),
        "dashboard-lifecycle.css" => (
            tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS_SHA256,
            "text/css; charset=utf-8",
            tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS.as_bytes(),
        ),
        "dashboard.js" => (
            tessara_dashboard_ui::DASHBOARD_JS_SHA256,
            "text/javascript; charset=utf-8",
            tessara_dashboard_ui::DASHBOARD_JS.as_bytes(),
        ),
        "dashboard-bindings.js" => (
            tessara_dashboard_ui::DASHBOARD_BINDINGS_JS_SHA256,
            "text/javascript; charset=utf-8",
            tessara_dashboard_ui::DASHBOARD_BINDINGS_JS.as_bytes(),
        ),
        "dashboard.wasm" => (
            tessara_dashboard_ui::DASHBOARD_WASM_SHA256,
            "application/wasm",
            tessara_dashboard_ui::DASHBOARD_WASM,
        ),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    if digest != format!("sha256:{expected_digest}") {
        return StatusCode::NOT_FOUND.into_response();
    }
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, content_type),
            (header::CACHE_CONTROL, "public, max-age=31536000, immutable"),
        ],
        Body::from(bytes),
    )
        .into_response()
}

async fn validate_configuration_api(
    Json(input): Json<DashboardConfigurationV1>,
) -> Json<ConfigurationValidationV1> {
    Json(validate_configuration(&input))
}

async fn get_configuration(
    State(state): State<DashboardModuleState>,
) -> Result<Json<Value>, DashboardModuleError> {
    let row = sqlx::query(
        "SELECT schema_version, display_label, default_page_size, updated_at
         FROM dashboard_configuration WHERE singleton=true",
    )
    .fetch_one(&state.pool)
    .await?;
    Ok(Json(json!({
        "schema_version": row.try_get::<i32,_>("schema_version")?,
        "display_label": row.try_get::<String,_>("display_label")?,
        "default_page_size": row.try_get::<i32,_>("default_page_size")?,
        "updated_at": row.try_get::<DateTime<Utc>,_>("updated_at")?,
    })))
}

async fn put_configuration(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Json(input): Json<DashboardConfigurationV1>,
) -> Result<Json<ConfigurationValidationV1>, DashboardModuleError> {
    require_private_key(&headers)?;
    let validation = validate_configuration(&input);
    let Some(normalized) = &validation.normalized else {
        return Ok(Json(validation));
    };
    sqlx::query(
        "UPDATE dashboard_configuration
         SET display_label=$1, default_page_size=$2, updated_at=now()
         WHERE singleton=true",
    )
    .bind(&normalized.display_label)
    .bind(i32::from(normalized.default_page_size))
    .execute(&state.pool)
    .await?;
    Ok(Json(validation))
}

async fn update_security_state(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Json(input): Json<SecurityStateInput>,
) -> Result<StatusCode, DashboardModuleError> {
    require_private_key(&headers)?;
    if input.schema_version != 1
        || !matches!(
            input.document_state.as_str(),
            "enabled" | "disabled" | "degraded" | "recovery"
        )
        || input.authorization_revision == 0
        || input.organization_revision == 0
    {
        return Err(DashboardModuleError::BadRequest(
            "invalid security state".into(),
        ));
    }
    let updated = sqlx::query(
        "INSERT INTO dashboard_security_state
         (singleton, installation_id, module_instance_id, authorization_revision,
          organization_revision, enabled, document_state)
         VALUES (true,$1,$2,$3,$4,$5,$6)
         ON CONFLICT (singleton) DO UPDATE SET
           authorization_revision=GREATEST(
             dashboard_security_state.authorization_revision,
             EXCLUDED.authorization_revision
           ),
           organization_revision=GREATEST(
             dashboard_security_state.organization_revision,
             EXCLUDED.organization_revision
           ),
           enabled=EXCLUDED.enabled,
           document_state=EXCLUDED.document_state,
           updated_at=now()
         WHERE dashboard_security_state.installation_id=EXCLUDED.installation_id
           AND dashboard_security_state.module_instance_id=EXCLUDED.module_instance_id",
    )
    .bind(input.installation_id)
    .bind(input.module_instance_id)
    .bind(input.authorization_revision as i64)
    .bind(input.organization_revision as i64)
    .bind(input.enabled)
    .bind(input.document_state)
    .execute(&state.pool)
    .await?;
    if updated.rows_affected() == 0 {
        return Err(DashboardModuleError::Conflict(
            "security state identity cannot change".into(),
        ));
    }
    Ok(StatusCode::NO_CONTENT)
}

async fn live() -> Json<Value> {
    Json(json!({
        "status": "passing",
        "module": MODULE_DEFINITION_ID,
        "schema_version": 1,
    }))
}

async fn ready(
    State(state): State<DashboardModuleState>,
) -> Result<Json<Value>, DashboardModuleError> {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await?;
    let security = load_security_state(&state.pool).await?;
    let ready = security
        .as_ref()
        .is_some_and(|value| value.enabled && value.document_state == "enabled");
    let status = if ready { "ready" } else { "not_ready" };
    Ok(Json(json!({
        "status": status,
        "module": MODULE_DEFINITION_ID,
        "database": "connected",
        "security_state": security.map(|value| value.document_state),
    })))
}

async fn diagnostics(
    State(state): State<DashboardModuleState>,
) -> Result<Json<Value>, DashboardModuleError> {
    let configuration = get_configuration(State(state.clone())).await?.0;
    let security = load_security_state(&state.pool).await?;
    Ok(Json(json!({
        "schema_version": 1,
        "module": MODULE_DEFINITION_ID,
        "release": MODULE_RELEASE_VERSION,
        "assets": {
            "dashboard_css": format!("sha256:{}", tessara_dashboard_ui::DASHBOARD_CSS_SHA256),
            "dashboard_js": format!("sha256:{}", tessara_dashboard_ui::DASHBOARD_JS_SHA256),
            "dashboard_bindings_js": format!("sha256:{}", tessara_dashboard_ui::DASHBOARD_BINDINGS_JS_SHA256),
            "dashboard_wasm": format!("sha256:{}", tessara_dashboard_ui::DASHBOARD_WASM_SHA256),
        },
        "configuration": configuration,
        "database": {"status": "connected", "binding": "dashboard_module_instance"},
        "authorization": security.as_ref().map(|value| json!({
            "installation_id": value.installation_id,
            "module_instance_id": value.module_instance_id,
            "authorization_revision": value.authorization_revision,
            "organization_revision": value.organization_revision,
            "enabled": value.enabled,
            "document_state": value.document_state,
            "updated_at": value.updated_at,
        })),
        "components_dependency": {
            "binding_key": COMPONENT_BINDING_KEY,
            "contract_id": COMPONENT_CONTRACT_ID,
            "provider": "selected_component_module_instance",
            "contract_version": tessara_components_contract::COMPONENT_CONTRACT_VERSION,
            "actions": ["resolve_metadata", "render"],
            "external_blueprints_allowed": false,
        },
        "findings": [],
    })))
}

async fn load_security_state(pool: &PgPool) -> Result<Option<SecurityState>, DashboardModuleError> {
    Ok(sqlx::query_as::<_, SecurityState>(
        "SELECT installation_id, module_instance_id, authorization_revision,
                organization_revision, enabled, document_state, updated_at
         FROM dashboard_security_state WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await?)
}

fn require_private_key(headers: &HeaderMap) -> Result<(), DashboardModuleError> {
    let expected = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let supplied = headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok());
    if supplied == Some(expected.as_str()) {
        Ok(())
    } else {
        Err(DashboardModuleError::Forbidden)
    }
}

fn require_exact_json(headers: &HeaderMap) -> Result<(), DashboardModuleError> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return Err(DashboardModuleError::BadRequest(
            "Content-Type must be application/json".into(),
        ));
    }
    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardModuleError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("forbidden")]
    Forbidden,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("unavailable: {0}")]
    Unavailable(String),
    #[error("internal error: {0}")]
    Internal(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl axum::response::IntoResponse for DashboardModuleError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
            Self::NotFound(message) => (StatusCode::NOT_FOUND, message),
            Self::Conflict(message) => (StatusCode::CONFLICT, message),
            Self::Unavailable(message) => (StatusCode::SERVICE_UNAVAILABLE, message),
            Self::Internal(error) => {
                tracing::error!(%error, "Dashboard module internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
            Self::Database(error) => {
                tracing::error!(%error, "Dashboard module database error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal server error".into(),
                )
            }
        };
        (status, Json(json!({"error": message}))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use axum::{Router, routing::get};
    use sha2::{Digest, Sha256};
    use tessara_dashboards::DashboardPlacementConfigV1;
    use tessara_module_contract::{ModuleManifest, ResourceOwner, TypedResourceReference};
    use uuid::Uuid;

    use super::{
        DashboardBootstrapPlacementV2, DashboardBootstrapV2, DashboardConfigurationV1,
        MODULE_DEFINITION_ID, dashboard_bootstrap_resource_ids, manifest,
        module_service_client_with_timeout, validate_configuration,
        validate_dashboard_bootstrap_input,
    };

    fn sha256(bytes: &[u8]) -> String {
        format!("{:x}", Sha256::digest(bytes))
    }

    #[test]
    fn browser_asset_identities_are_source_exact_and_match_the_manifest() {
        let assets = [
            (
                "/dashboard.css",
                sha256(tessara_dashboard_ui::DASHBOARD_CSS.as_bytes()),
                tessara_dashboard_ui::DASHBOARD_CSS_SHA256,
            ),
            (
                "/dashboard-lifecycle.css",
                sha256(tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS.as_bytes()),
                tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS_SHA256,
            ),
            (
                "/dashboard.js",
                sha256(tessara_dashboard_ui::DASHBOARD_JS.as_bytes()),
                tessara_dashboard_ui::DASHBOARD_JS_SHA256,
            ),
            (
                "/dashboard-bindings.js",
                sha256(tessara_dashboard_ui::DASHBOARD_BINDINGS_JS.as_bytes()),
                tessara_dashboard_ui::DASHBOARD_BINDINGS_JS_SHA256,
            ),
            (
                "/dashboard.wasm",
                sha256(tessara_dashboard_ui::DASHBOARD_WASM),
                tessara_dashboard_ui::DASHBOARD_WASM_SHA256,
            ),
        ];
        let manifest = manifest();

        for (path, source_digest, declared_digest) in assets {
            assert_eq!(
                source_digest, declared_digest,
                "stale digest constant for {path}"
            );
            let manifest_asset = manifest
                .assets
                .iter()
                .find(|asset| asset.path == path)
                .unwrap_or_else(|| panic!("manifest should declare {path}"));
            assert_eq!(
                manifest_asset.digest.to_string(),
                format!("sha256:{source_digest}"),
                "stale manifest digest for {path}"
            );
        }
    }

    #[test]
    fn sprint_8a_bootstrap_owns_seven_exact_receipt_bound_component_placements() {
        let blueprint: tessara_composition::ApplicationBlueprintV1 =
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../deploy/sprint-8a/blueprints/reference.json"
            )))
            .expect("valid Sprint 8A Blueprint");
        let component_module = blueprint
            .modules
            .iter()
            .find(|module| {
                module.definition_id == tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID
            })
            .expect("Component selection");
        let tessara_composition::BootstrapInputV1::Inline {
            value: component_input,
            ..
        } = component_module
            .bootstrap
            .as_ref()
            .expect("Component bootstrap")
        else {
            panic!("Sprint 8A Component bootstrap must be inline");
        };
        let component_instance_id = tessara_composition::module_instance_id(
            blueprint.installation_id,
            tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID,
        );
        let resource_ids = component_input["components"]
            .as_array()
            .expect("Component bootstrap inventory")
            .iter()
            .flat_map(|component| {
                component["versions"]
                    .as_array()
                    .expect("Component bootstrap versions")
                    .iter()
            })
            .map(|version| {
                let resource_key = version["resource_key"]
                    .as_str()
                    .expect("Component receipt resource key");
                let version_id = tessara_composition::owner_resource_id(
                    blueprint.installation_id,
                    tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID,
                    "component-version",
                    resource_key,
                )
                .to_string();
                let reference = tessara_components_contract::ComponentVersionReference::new(
                    TypedResourceReference::new(
                        blueprint.installation_id,
                        ResourceOwner::ModuleInstance {
                            installation_id: blueprint.installation_id,
                            module_instance_id: component_instance_id,
                        },
                        tessara_components_contract::COMPONENT_RESOURCE_TYPE
                            .parse()
                            .expect("Component resource type"),
                        &version_id,
                    )
                    .expect("typed resource reference"),
                )
                .expect("Component version reference");
                (
                    resource_key.to_string(),
                    serde_json::to_string(&reference).expect("serialized Component reference"),
                )
            })
            .collect::<std::collections::BTreeMap<_, _>>();
        let receipt = tessara_composition::BootstrapReceiptV1 {
            owner: tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID.into(),
            schema_version: "tessara.io/component-bootstrap/v1".into(),
            input_digest: tessara_composition::canonical_digest(component_input)
                .expect("Component input digest"),
            result_digest: tessara_composition::canonical_digest(&resource_ids)
                .expect("Component result digest"),
            changed: true,
            resource_ids,
        };
        let module = blueprint
            .modules
            .iter()
            .find(|module| module.definition_id == "tessara.dashboards")
            .expect("Dashboard selection");
        let tessara_composition::BootstrapInputV1::Inline {
            value,
            receipt_bindings,
            ..
        } = module.bootstrap.as_ref().expect("Dashboard bootstrap")
        else {
            panic!("Sprint 8A Dashboard bootstrap must be inline");
        };
        assert!(
            value["placements"]
                .as_array()
                .expect("Dashboard placements")
                .iter()
                .all(|placement| placement["component_reference"].is_null()),
            "the locked Dashboard seed must not contain hardcoded Component references"
        );
        assert_eq!(receipt_bindings.len(), 7);
        let mut resolved = tessara_composition::resolve_bootstrap_receipt_bindings(
            value.clone(),
            receipt_bindings,
            &std::collections::BTreeMap::from([(
                tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID.into(),
                receipt,
            )]),
        )
        .expect("Dashboard receipt bindings resolve");
        resolved
            .as_object_mut()
            .expect("Dashboard bootstrap object")
            .remove("dashboard_id");
        for placement in resolved["placements"]
            .as_array_mut()
            .expect("Dashboard placements")
        {
            placement
                .as_object_mut()
                .expect("Dashboard placement object")
                .remove("placement_id");
        }
        let bootstrap: DashboardBootstrapV2 =
            serde_json::from_value(resolved).expect("typed Dashboard bootstrap");
        let prepared_placements =
            validate_dashboard_bootstrap_input(blueprint.installation_id, &bootstrap)
                .expect("resolved Dashboard bootstrap identity and layout");
        assert_eq!(
            prepared_placements
                .iter()
                .map(|placement| placement.placement_id)
                .collect::<Vec<_>>(),
            bootstrap
                .placements
                .iter()
                .map(|placement| tessara_composition::owner_resource_id(
                    blueprint.installation_id,
                    MODULE_DEFINITION_ID,
                    "dashboard-placement",
                    &placement.placement_key,
                ))
                .collect::<Vec<_>>()
        );
        let expected_layout = [
            (0, 1, 1, 4, 2),
            (1, 3, 1, 12, 6),
            (2, 9, 1, 6, 4),
            (3, 9, 7, 6, 4),
            (4, 13, 1, 4, 2),
            (5, 13, 5, 4, 2),
            (6, 13, 9, 4, 2),
        ];
        for (prepared, (position, row, column, width, height)) in
            prepared_placements.iter().zip(expected_layout)
        {
            assert_eq!(prepared.position, position);
            assert!(
                prepared.config.get("placement_key").is_none(),
                "bootstrap-only placement keys must not enter product config"
            );
            let config: DashboardPlacementConfigV1 =
                serde_json::from_value(prepared.config.clone()).expect("canonical V1 config");
            assert_eq!(config.schema_version, 1);
            assert_eq!(config.title, None);
            assert_eq!(config.grid_row, row);
            assert_eq!(config.grid_column, column);
            assert_eq!(config.grid_width, width);
            assert_eq!(config.grid_height, height);
        }
        for placement in &bootstrap.placements {
            let reference = placement.component_reference.reference();
            assert_eq!(reference.installation_id(), blueprint.installation_id);
            assert_eq!(
                reference.owner(),
                &ResourceOwner::ModuleInstance {
                    installation_id: blueprint.installation_id,
                    module_instance_id: component_instance_id,
                }
            );
            assert_eq!(
                reference.resource_type().as_str(),
                "tessara.components.component_version"
            );
            uuid::Uuid::parse_str(reference.resource_id())
                .expect("Component receipt must carry an owner-derived version UUID");
        }
    }

    fn bootstrap_placement_wire(
        installation_id: uuid::Uuid,
        owner: serde_json::Value,
        resource_type: &str,
    ) -> serde_json::Value {
        serde_json::json!({
            "placement_key": "row-count",
            "component_reference": {
                "reference": {
                    "installation_id": installation_id,
                    "owner": owner,
                    "resource_type": resource_type,
                    "resource_id": "01980000-0001-7000-8000-000000000001"
                }
            },
            "column": 0,
            "row": 0,
            "width": 4,
            "height": 2
        })
    }

    fn valid_bootstrap_placement(
        installation_id: uuid::Uuid,
        placement_key: &str,
        row: u16,
        column: u16,
        width: u16,
        height: u16,
    ) -> DashboardBootstrapPlacementV2 {
        let component_instance_id = tessara_composition::module_instance_id(
            installation_id,
            tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID,
        );
        DashboardBootstrapPlacementV2 {
            placement_key: placement_key.into(),
            component_reference: tessara_components_contract::ComponentVersionReference::new(
                TypedResourceReference::new(
                    installation_id,
                    ResourceOwner::ModuleInstance {
                        installation_id,
                        module_instance_id: component_instance_id,
                    },
                    tessara_components_contract::COMPONENT_RESOURCE_TYPE
                        .parse()
                        .expect("Component resource type"),
                    uuid::Uuid::new_v4().to_string(),
                )
                .expect("typed Component resource reference"),
            )
            .expect("Component version reference"),
            column,
            row,
            width,
            height,
        }
    }

    fn valid_dashboard_bootstrap(
        placements: Vec<DashboardBootstrapPlacementV2>,
    ) -> DashboardBootstrapV2 {
        DashboardBootstrapV2 {
            schema_version: "tessara.io/dashboard-bootstrap/v2".into(),
            external_key: "validation-fixture".into(),
            name: "Validation fixture".into(),
            description: None,
            scope_node_id: uuid::Uuid::new_v4(),
            placements,
        }
    }

    #[test]
    fn dashboard_bootstrap_derives_dense_positions_from_zero_based_blueprint_geometry() {
        let installation_id = uuid::Uuid::new_v4();
        let first = valid_bootstrap_placement(installation_id, "first", 0, 0, 4, 2);
        let second = valid_bootstrap_placement(installation_id, "second", 2, 0, 4, 2);
        let bootstrap = valid_dashboard_bootstrap(vec![second, first]);

        let prepared = validate_dashboard_bootstrap_input(installation_id, &bootstrap)
            .expect("valid complete bootstrap layout");

        assert_eq!(
            prepared
                .iter()
                .map(|placement| placement.position)
                .collect::<Vec<_>>(),
            vec![1, 0],
            "positions are dense row-major ranks, not sparse cell offsets"
        );
        let second_config: DashboardPlacementConfigV1 =
            serde_json::from_value(prepared[0].config.clone()).expect("canonical second config");
        let first_config: DashboardPlacementConfigV1 =
            serde_json::from_value(prepared[1].config.clone()).expect("canonical first config");
        assert_eq!(
            second_config.rect(),
            tessara_dashboards::GridRect::new(3, 1, 4, 2)
        );
        assert_eq!(
            first_config.rect(),
            tessara_dashboards::GridRect::new(1, 1, 4, 2)
        );
    }

    #[test]
    fn dashboard_bootstrap_receipt_exposes_exact_logical_dashboard_and_placement_identities() {
        let installation_id = Uuid::from_u128(0x8b);
        let bootstrap = valid_dashboard_bootstrap(vec![
            valid_bootstrap_placement(installation_id, "dataset-stat", 0, 0, 4, 2),
            valid_bootstrap_placement(installation_id, "dataset-table", 2, 0, 12, 6),
            valid_bootstrap_placement(installation_id, "dataset-chart", 8, 0, 8, 4),
            valid_bootstrap_placement(installation_id, "dataset-disjoint", 8, 8, 4, 4),
        ]);
        let prepared = validate_dashboard_bootstrap_input(installation_id, &bootstrap)
            .expect("non-overlapping Sprint 8B Dashboard fixture");
        let dashboard_id = tessara_composition::owner_resource_id(
            installation_id,
            MODULE_DEFINITION_ID,
            "dashboard",
            &bootstrap.external_key,
        );
        let resource_ids = dashboard_bootstrap_resource_ids(&bootstrap, dashboard_id, &prepared);

        assert_eq!(
            resource_ids.keys().map(String::as_str).collect::<Vec<_>>(),
            vec![
                "dashboard",
                "external_key",
                "placement.dataset-chart",
                "placement.dataset-disjoint",
                "placement.dataset-stat",
                "placement.dataset-table",
            ]
        );
        assert_eq!(resource_ids["dashboard"], dashboard_id.to_string());
        assert_eq!(resource_ids["external_key"], bootstrap.external_key);
        for placement in &bootstrap.placements {
            assert_eq!(
                resource_ids[&format!("placement.{}", placement.placement_key)],
                tessara_composition::owner_resource_id(
                    installation_id,
                    MODULE_DEFINITION_ID,
                    "dashboard-placement",
                    &placement.placement_key,
                )
                .to_string()
            );
        }
    }

    #[test]
    fn dashboard_bootstrap_rejects_duplicate_normalized_logical_keys() {
        let installation_id = uuid::Uuid::new_v4();
        let first = valid_bootstrap_placement(installation_id, "duplicate", 0, 0, 4, 2);
        let mut duplicate = valid_bootstrap_placement(installation_id, "second", 2, 0, 4, 2);
        duplicate.placement_key = " duplicate ".into();
        let duplicate_keys = valid_dashboard_bootstrap(vec![first, duplicate]);
        assert!(validate_dashboard_bootstrap_input(installation_id, &duplicate_keys).is_err());
    }

    #[test]
    fn dashboard_bootstrap_rejects_overlapping_and_out_of_bounds_complete_layouts() {
        let installation_id = uuid::Uuid::new_v4();
        let first = valid_bootstrap_placement(installation_id, "first", 0, 0, 4, 2);
        let mut second = valid_bootstrap_placement(installation_id, "second", 1, 0, 4, 2);
        let overlap = valid_dashboard_bootstrap(vec![first.clone(), second.clone()]);
        assert!(validate_dashboard_bootstrap_input(installation_id, &overlap).is_err());

        second.row = 239;
        let beyond_bottom = valid_dashboard_bootstrap(vec![first.clone(), second.clone()]);
        assert!(validate_dashboard_bootstrap_input(installation_id, &beyond_bottom).is_err());

        second.row = 2;
        second.column = 11;
        second.width = 2;
        let beyond_right = valid_dashboard_bootstrap(vec![first, second]);
        assert!(validate_dashboard_bootstrap_input(installation_id, &beyond_right).is_err());
    }

    #[test]
    fn dashboard_bootstrap_rejects_wrong_component_owner_instance_and_type() {
        let installation_id =
            uuid::Uuid::parse_str("01980000-0000-7000-8000-00000000008a").unwrap();
        let core_owned = bootstrap_placement_wire(
            installation_id,
            serde_json::json!({
                "kind": "core_installation",
                "installation_id": installation_id
            }),
            tessara_components_contract::COMPONENT_RESOURCE_TYPE,
        );
        assert!(
            serde_json::from_value::<DashboardBootstrapPlacementV2>(core_owned).is_err(),
            "Component v3 references reject a Core owner"
        );

        let wrong_type = bootstrap_placement_wire(
            installation_id,
            serde_json::json!({
                "kind": "module_instance",
                "installation_id": installation_id,
                "module_instance_id": tessara_composition::module_instance_id(
                    installation_id,
                    tessara_components_contract::COMPONENT_MODULE_DEFINITION_ID,
                )
            }),
            "tessara.transition.component_version",
        );
        assert!(
            serde_json::from_value::<DashboardBootstrapPlacementV2>(wrong_type).is_err(),
            "Component v3 references reject an old transition resource type"
        );

        let wrong_instance = bootstrap_placement_wire(
            installation_id,
            serde_json::json!({
                "kind": "module_instance",
                "installation_id": installation_id,
                "module_instance_id": "01980000-0000-7000-8000-000000000099"
            }),
            tessara_components_contract::COMPONENT_RESOURCE_TYPE,
        );
        let placement: DashboardBootstrapPlacementV2 =
            serde_json::from_value(wrong_instance).expect("well-typed but wrong provider instance");
        let bootstrap = DashboardBootstrapV2 {
            schema_version: "tessara.io/dashboard-bootstrap/v2".into(),
            external_key: "wrong-instance".into(),
            name: "Wrong instance".into(),
            description: None,
            scope_node_id: uuid::Uuid::new_v4(),
            placements: vec![placement],
        };
        assert!(validate_dashboard_bootstrap_input(installation_id, &bootstrap).is_err());
    }

    #[tokio::test]
    async fn module_service_client_enforces_the_complete_request_deadline() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind delayed provider");
        let address = listener.local_addr().expect("delayed provider address");
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route(
                    "/",
                    get(|| async {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        "late provider response"
                    }),
                ),
            )
            .await
            .expect("serve delayed provider");
        });
        let client = module_service_client_with_timeout(Duration::from_millis(50))
            .expect("bounded provider client");
        let started = Instant::now();

        let error = client
            .get(format!("http://{address}/"))
            .send()
            .await
            .expect_err("delayed provider must time out");

        assert!(error.is_timeout());
        assert!(started.elapsed() < Duration::from_secs(1));
        server.abort();
    }

    #[test]
    fn configuration_validation_normalizes_and_bounds_values() {
        let validation = validate_configuration(&DashboardConfigurationV1 {
            schema_version: 1,
            display_label: "  Dashboards  ".into(),
            default_page_size: 25,
        });
        assert!(validation.valid);
        assert_eq!(
            validation.normalized.expect("normalized").display_label,
            "Dashboards"
        );

        let invalid = validate_configuration(&DashboardConfigurationV1 {
            schema_version: 2,
            display_label: " ".into(),
            default_page_size: 9,
        });
        assert!(!invalid.valid);
        assert_eq!(invalid.findings.len(), 3);
    }

    #[test]
    fn authoritative_manifest_declares_the_independent_dashboard_boundary() {
        let manifest: ModuleManifest =
            serde_json::from_str(include_str!("../manifest.json")).expect("valid manifest");
        assert_eq!(manifest.definition_id.as_str(), "tessara.dashboards");
        assert_eq!(manifest.release_version.to_string(), "3.0.2");
        let lifecycle = manifest
            .browser_lifecycle
            .as_ref()
            .expect("Dashboard declares lifecycle v1");
        assert_eq!(lifecycle.lifecycle_abi.to_string(), "1.0.0");
        assert_eq!(lifecycle.entry_asset, "/dashboard.js");
        assert!(lifecycle.complete_document_fallback);
        let tessara_module_contract::DeploymentProfile::TessaraOciV1(deployment) =
            &manifest.deployment;
        assert_eq!(deployment.listen.port, 8091);
        assert_eq!(
            deployment
                .runtime_image
                .command
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["/usr/local/bin/dashboard-module", "serve"]
        );
        assert_eq!(
            deployment
                .migration_image
                .as_ref()
                .expect("Dashboard declares a migration image")
                .command
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
            ["/usr/local/bin/dashboard-module", "migrate"]
        );
        assert_eq!(manifest.dependencies.len(), 1);
        assert_eq!(
            manifest.dependencies[0].binding_key.as_str(),
            "tessara.dashboards.component-version"
        );
        assert!(
            manifest
                .security_capabilities
                .iter()
                .any(|capability| capability.id.as_str() == "dashboards:read")
        );

        for (path, expected_digest) in [
            ("/dashboard.css", tessara_dashboard_ui::DASHBOARD_CSS_SHA256),
            (
                "/dashboard-lifecycle.css",
                tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS_SHA256,
            ),
            ("/dashboard.js", tessara_dashboard_ui::DASHBOARD_JS_SHA256),
            (
                "/dashboard-bindings.js",
                tessara_dashboard_ui::DASHBOARD_BINDINGS_JS_SHA256,
            ),
            (
                "/dashboard.wasm",
                tessara_dashboard_ui::DASHBOARD_WASM_SHA256,
            ),
        ] {
            let declared = manifest
                .assets
                .iter()
                .find(|asset| asset.path == path)
                .unwrap_or_else(|| panic!("manifest should declare {path}"));
            assert_eq!(
                declared.digest.to_string(),
                format!("sha256:{expected_digest}")
            );
        }
    }

    #[test]
    fn dashboard_module_v3_fresh_baseline_is_pinned() {
        let baseline = include_bytes!("../migrations/001_dashboard_module.sql");
        assert_eq!(
            format!("{:x}", Sha256::digest(baseline)),
            "da0b935b9f19f960d2d15422ca8f423d975f933ffd764353d9ad1f812df83d45"
        );
        let baseline = std::str::from_utf8(baseline).expect("baseline is UTF-8");
        assert!(baseline.contains("CREATE TABLE dashboard_dependency_observations"));
        assert!(baseline.contains("CREATE TABLE dashboard_dependency_findings"));
        assert!(baseline.contains("CREATE TABLE dashboard_dependency_action_receipts"));
        assert!(baseline.contains("authorization_context_digest TEXT NOT NULL"));
        assert!(baseline.contains("authorization_expires_at TIMESTAMPTZ NOT NULL"));
        assert!(baseline.contains("resolution_origin TEXT NOT NULL"));
        assert!(baseline.contains("tessara.components.component_version"));
        assert!(baseline.contains("module_instance"));
    }
}
