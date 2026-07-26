//! Independently deployed Dashboard Module boundary.
//!
//! Sprint 6C moves Dashboard persistence and product transport into this
//! process. Core supplies signed shell and authorization projections; the
//! module never receives Core browser state or reusable Core authority.

use axum::{
    Json, Router,
    extract::State,
    http::{HeaderMap, StatusCode},
    routing::{get, post, put},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::{FromRow, PgPool, Row};
use tessara_module_contract::PurposeBoundVerifyingKeyV1;
use uuid::Uuid;

pub const MODULE_DEFINITION_ID: &str = "tessara.dashboards";
pub const READ_CAPABILITY: &str = "dashboards:read";
pub const MANAGE_CAPABILITY: &str = "dashboards:manage";
pub const COMPONENT_BINDING_KEY: &str = "tessara.dashboards.component-version";
pub const COMPONENT_CONTRACT_ID: &str = "tessara.components.component-version";

#[derive(Clone)]
pub struct DashboardModuleState {
    pub pool: PgPool,
    pub core_authorization_verifier: PurposeBoundVerifyingKeyV1,
    pub core_shell_verifier: PurposeBoundVerifyingKeyV1,
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
        .route(
            "/api/configuration",
            get(get_configuration).put(put_configuration),
        )
        .route("/api/private/security-state", put(update_security_state))
        .route("/health/live", get(live))
        .route("/health/ready", get(ready))
        .route("/api/diagnostics", get(diagnostics))
        .with_state(state)
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
    sqlx::query(
        "INSERT INTO dashboard_security_state
         (singleton, installation_id, module_instance_id, authorization_revision,
          organization_revision, enabled, document_state)
         VALUES (true,$1,$2,$3,$4,$5,$6)
         ON CONFLICT (singleton) DO UPDATE SET
           installation_id=EXCLUDED.installation_id,
           module_instance_id=EXCLUDED.module_instance_id,
           authorization_revision=EXCLUDED.authorization_revision,
           organization_revision=EXCLUDED.organization_revision,
           enabled=EXCLUDED.enabled,
           document_state=EXCLUDED.document_state,
           updated_at=now()",
    )
    .bind(input.installation_id)
    .bind(input.module_instance_id)
    .bind(input.authorization_revision as i64)
    .bind(input.organization_revision as i64)
    .bind(input.enabled)
    .bind(input.document_state)
    .execute(&state.pool)
    .await?;
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
            "provider": "core_installation",
            "actions": ["resolve_metadata", "render"],
            "transition_only": true,
            "external_blueprints_allowed": false,
            "migration_target": "Sprint 8A",
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

#[derive(Debug, thiserror::Error)]
pub enum DashboardModuleError {
    #[error("bad request: {0}")]
    BadRequest(String),
    #[error("forbidden")]
    Forbidden,
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}

impl axum::response::IntoResponse for DashboardModuleError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            Self::BadRequest(message) => (StatusCode::BAD_REQUEST, message),
            Self::Forbidden => (StatusCode::FORBIDDEN, "forbidden".into()),
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
    use super::{DashboardConfigurationV1, validate_configuration};

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
}
