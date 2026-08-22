//! Dataset-owned operational status, readiness, and sanitized diagnostics.

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;

use crate::{
    DatasetConfigurationV1, DatasetModuleState, MODULE_DEFINITION_ID, MODULE_RELEASE_VERSION,
    REQUIRED_DATASET_PROVIDER_BINDINGS, SecurityState, validate_configuration,
};

const DATABASE_FAILURE: &str = "dataset.database.unavailable";
const CONFIGURATION_MISSING_FAILURE: &str = "dataset.configuration.missing";
const CONFIGURATION_INVALID_FAILURE: &str = "dataset.configuration.invalid";
const SECURITY_MISSING_FAILURE: &str = "dataset.security_state.missing";
const SECURITY_DISABLED_FAILURE: &str = "dataset.security_state.disabled";
const SECURITY_RECOVERY_FAILURE: &str = "dataset.security_state.recovery";
const SECURITY_DEGRADED_FAILURE: &str = "dataset.security_state.degraded";
const BINDING_INCOMPATIBLE_FAILURE: &str = "dataset.binding.incompatible";
const LAST_GOOD_MISSING_FAILURE: &str = "dataset.product.last_good_missing";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ReadinessResponse {
    pub schema_version: u16,
    pub module_definition_id: &'static str,
    pub module_release_version: &'static str,
    pub status: &'static str,
    pub database: &'static str,
    pub security_state: &'static str,
    pub configuration: &'static str,
    pub required_bindings: &'static str,
    pub product_state: &'static str,
    pub freshness: &'static str,
    pub failures: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DiagnosticsResponse {
    pub schema_version: u16,
    pub module: &'static str,
    pub release: &'static str,
    pub manifest_schema: u16,
    pub contracts: ContractDiagnostics,
    pub configuration: Option<DatasetConfigurationV1>,
    pub database: DatabaseDiagnostics,
    pub authorization: AuthorizationDiagnostics,
    pub dependencies: Vec<BindingDiagnostics>,
    pub product: ProductDiagnostics,
    pub probes: ProbeDiagnostics,
    pub failures: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ContractDiagnostics {
    pub dataset: &'static str,
    pub response_export: &'static str,
    pub form_version_schema: &'static str,
    pub scope_catalog: &'static str,
    pub principal_display: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct DatabaseDiagnostics {
    pub status: &'static str,
    pub binding: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct AuthorizationDiagnostics {
    pub projection: &'static str,
    pub enabled: Option<bool>,
    pub document_state: Option<String>,
    pub authorization_revision: Option<i64>,
    pub organization_revision: Option<i64>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct BindingDiagnostics {
    pub binding_key: &'static str,
    pub contract_id: &'static str,
    pub contract_version: &'static str,
    pub status: &'static str,
    pub last_observation_at: Option<DateTime<Utc>>,
    pub last_success_at: Option<DateTime<Utc>>,
    pub freshness: &'static str,
    pub failure_code: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProductDiagnostics {
    pub state: &'static str,
    pub last_materialization_at: Option<DateTime<Utc>>,
    pub freshness: &'static str,
}

#[derive(Clone, Debug, Serialize)]
pub(crate) struct ProbeDiagnostics {
    pub liveness: &'static str,
    pub readiness: &'static str,
}

#[derive(Clone, Debug)]
pub(crate) struct OperationalSnapshot {
    database: DatabaseState,
    configuration_state: ConfigurationState,
    configuration: Option<DatasetConfigurationV1>,
    security_state: SecurityProjectionState,
    security: Option<SecurityState>,
    bindings_compatible: bool,
    product_state: ProductState,
    last_materialization_at: Option<DateTime<Utc>>,
    freshness: FreshnessObservation,
    failures: Vec<String>,
}

impl OperationalSnapshot {
    pub(crate) fn readiness(&self) -> ReadinessResponse {
        ReadinessResponse {
            schema_version: 1,
            module_definition_id: MODULE_DEFINITION_ID,
            module_release_version: MODULE_RELEASE_VERSION,
            status: self.overall_status(),
            database: self.database.as_str(),
            security_state: self.security_state.as_str(),
            configuration: self.configuration_state.as_str(),
            required_bindings: if self.bindings_compatible {
                "compatible"
            } else {
                "incompatible"
            },
            product_state: self.product_state.as_str(),
            freshness: self.freshness.state.as_str(),
            failures: self.failures.clone(),
        }
    }

    pub(crate) fn diagnostics(&self) -> DiagnosticsResponse {
        let response_observation = BindingObservation {
            last_observation_at: self.freshness.last_observation_at,
            last_success_at: self.freshness.last_success_at,
            freshness: self.freshness.state,
            failure_code: self.freshness.failure_code.clone(),
        };
        DiagnosticsResponse {
            schema_version: 1,
            module: MODULE_DEFINITION_ID,
            release: MODULE_RELEASE_VERSION,
            manifest_schema: 3,
            contracts: ContractDiagnostics {
                dataset: tessara_datasets_contract::DATASET_CONTRACT_VERSION,
                response_export: tessara_responses_contract::RESPONSE_EXPORT_CONTRACT_VERSION,
                form_version_schema: tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_VERSION,
                scope_catalog: tessara_control_plane_contract::SCOPE_CATALOG_CONTRACT_VERSION,
                principal_display:
                    tessara_control_plane_contract::PRINCIPAL_DISPLAY_CONTRACT_VERSION,
            },
            configuration: self.configuration.clone(),
            database: DatabaseDiagnostics {
                status: self.database.as_str(),
                binding: "dataset_module_instance",
            },
            authorization: AuthorizationDiagnostics {
                projection: self.security_state.as_str(),
                enabled: self.security.as_ref().map(|value| value.enabled),
                document_state: self
                    .security
                    .as_ref()
                    .map(|value| value.document_state.clone()),
                authorization_revision: self
                    .security
                    .as_ref()
                    .map(|value| value.authorization_revision),
                organization_revision: self
                    .security
                    .as_ref()
                    .map(|value| value.organization_revision),
                updated_at: self.security.as_ref().map(|value| value.updated_at),
            },
            dependencies: required_binding_diagnostics(
                self.bindings_compatible,
                &response_observation,
            ),
            product: ProductDiagnostics {
                state: self.product_state.as_str(),
                last_materialization_at: self.last_materialization_at,
                freshness: self.freshness.state.as_str(),
            },
            probes: ProbeDiagnostics {
                liveness: crate::LIVENESS_PATH,
                readiness: crate::READINESS_PATH,
            },
            failures: self.failures.clone(),
        }
    }

    pub(crate) fn status_code(&self) -> axum::http::StatusCode {
        if self.overall_status() == "not_ready" {
            axum::http::StatusCode::SERVICE_UNAVAILABLE
        } else {
            axum::http::StatusCode::OK
        }
    }

    pub(crate) fn module_instance_id(&self) -> Option<uuid::Uuid> {
        self.security.as_ref().map(|value| value.module_instance_id)
    }

    fn overall_status(&self) -> &'static str {
        let not_ready = self.database != DatabaseState::Ready
            || self.configuration_state != ConfigurationState::Valid
            || !self.bindings_compatible
            || !self.security_state.can_serve()
            || self.product_state == ProductState::LastGoodMissing;
        if not_ready {
            "not_ready"
        } else if self.security_state == SecurityProjectionState::Degraded
            || !matches!(
                self.freshness.state,
                FreshnessState::Current | FreshnessState::NotApplicable
            )
        {
            "degraded"
        } else {
            "ready"
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DatabaseState {
    Ready,
    Unavailable,
}

impl DatabaseState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigurationState {
    Valid,
    Missing,
    Invalid,
}

impl ConfigurationState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Valid => "valid",
            Self::Missing => "missing",
            Self::Invalid => "invalid",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SecurityProjectionState {
    Installed,
    Degraded,
    Missing,
    Disabled,
    Recovery,
}

impl SecurityProjectionState {
    fn from_security(security: Option<&SecurityState>) -> Self {
        let Some(security) = security else {
            return Self::Missing;
        };
        if !security.enabled || security.document_state == "disabled" {
            Self::Disabled
        } else {
            match security.document_state.as_str() {
                "enabled" => Self::Installed,
                "degraded" => Self::Degraded,
                _ => Self::Recovery,
            }
        }
    }

    fn can_serve(self) -> bool {
        matches!(self, Self::Installed | Self::Degraded)
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Installed => "installed",
            Self::Degraded => "degraded",
            Self::Missing => "missing",
            Self::Disabled => "disabled",
            Self::Recovery => "recovery",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ProductState {
    Empty,
    LastGoodAvailable,
    LastGoodMissing,
}

impl ProductState {
    fn as_str(self) -> &'static str {
        match self {
            Self::Empty => "empty",
            Self::LastGoodAvailable => "last_good_available",
            Self::LastGoodMissing => "last_good_missing",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FreshnessState {
    NotApplicable,
    Current,
    Stale,
    Refreshing,
    Degraded,
    Failed,
    NeverMaterialized,
}

impl FreshnessState {
    fn as_str(self) -> &'static str {
        match self {
            Self::NotApplicable => "not_applicable",
            Self::Current => "current",
            Self::Stale => "stale",
            Self::Refreshing => "refreshing",
            Self::Degraded => "degraded",
            Self::Failed => "failed",
            Self::NeverMaterialized => "never_materialized",
        }
    }

    fn failure_code(self) -> Option<&'static str> {
        match self {
            Self::NotApplicable | Self::Current => None,
            Self::Stale => Some("dataset.freshness.stale"),
            Self::Refreshing => Some("dataset.freshness.refreshing"),
            Self::Degraded => Some("dataset.freshness.degraded"),
            Self::Failed => Some("dataset.freshness.failed"),
            Self::NeverMaterialized => Some("dataset.freshness.never_materialized"),
        }
    }
}

#[derive(Clone, Debug)]
struct FreshnessObservation {
    state: FreshnessState,
    last_observation_at: Option<DateTime<Utc>>,
    last_success_at: Option<DateTime<Utc>>,
    failure_code: Option<String>,
}

#[derive(Clone, Debug)]
struct BindingObservation {
    last_observation_at: Option<DateTime<Utc>>,
    last_success_at: Option<DateTime<Utc>>,
    freshness: FreshnessState,
    failure_code: Option<String>,
}

type ConfigurationRow = (i32, String, i32, i32, i32);
type ProductRow = (i64, i64, bool, Option<DateTime<Utc>>);
type FreshnessRow = (
    Option<String>,
    Option<DateTime<Utc>>,
    Option<DateTime<Utc>>,
    Option<String>,
);

pub(crate) async fn operational_snapshot(state: &DatasetModuleState) -> OperationalSnapshot {
    if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(&state.pool)
        .await
        .is_err()
    {
        return database_unavailable_snapshot();
    }

    let configuration = load_configuration(&state.pool).await;
    let security = crate::load_security_state(&state.pool).await;
    let product = load_product_state(&state.pool).await;
    let freshness = load_freshness(&state.pool).await;
    let (Ok(configuration), Ok(security), Ok(product), Ok(freshness)) =
        (configuration, security, product, freshness)
    else {
        return database_unavailable_snapshot();
    };

    let bindings_compatible = state.service_endpoints.provider_urls.len()
        == REQUIRED_DATASET_PROVIDER_BINDINGS.len()
        && REQUIRED_DATASET_PROVIDER_BINDINGS
            .iter()
            .all(|binding| state.service_endpoints.provider_urls.contains_key(*binding));
    compose_snapshot(
        DatabaseState::Ready,
        configuration,
        security,
        bindings_compatible,
        product,
        freshness,
    )
}

async fn load_configuration(pool: &PgPool) -> Result<Option<DatasetConfigurationV1>, sqlx::Error> {
    let row = sqlx::query_as::<_, ConfigurationRow>(
        "SELECT schema_version,display_label,provider_request_timeout_seconds,provider_retry_limit,\
         response_export_page_size FROM dataset_configuration WHERE singleton=true",
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.and_then(configuration_from_row))
}

fn configuration_from_row(row: ConfigurationRow) -> Option<DatasetConfigurationV1> {
    Some(DatasetConfigurationV1 {
        schema_version: u16::try_from(row.0).ok()?,
        display_label: row.1,
        provider_request_timeout_seconds: u16::try_from(row.2).ok()?,
        provider_retry_limit: u16::try_from(row.3).ok()?,
        response_export_page_size: u16::try_from(row.4).ok()?,
    })
}

async fn load_product_state(pool: &PgPool) -> Result<ProductRow, sqlx::Error> {
    sqlx::query_as(
        r#"
        WITH active_datasets AS (
          SELECT id FROM datasets WHERE lifecycle_state <> 'tombstoned'
        ),
        required_major_lines AS (
          SELECT d.id AS dataset_id,r.version_major
          FROM active_datasets d
          JOIN dataset_revisions r ON r.dataset_id=d.id
          WHERE r.lifecycle_state <> 'tombstoned'
            AND r.status IN ('published','superseded')
          GROUP BY d.id,r.version_major
        ),
        observed AS (
          SELECT required.dataset_id,required.version_major,m.materialized_at,
                 COALESCE(m.rebuild_status='ready'
                   AND m.materialized_schema='dataset_materialized'
                   AND m.materialized_table IS NOT NULL
                   AND m.materialized_row_count IS NOT NULL
                   AND m.materialized_row_count >= 0
                   AND m.materialized_at IS NOT NULL
                   AND to_regclass(format('%I.%I',m.materialized_schema,m.materialized_table))
                       IS NOT NULL,false) AS valid
          FROM required_major_lines required
          LEFT JOIN dataset_major_materializations m
            ON m.dataset_id=required.dataset_id AND m.version_major=required.version_major
        )
        SELECT (SELECT COUNT(*) FROM active_datasets)::bigint,
               (SELECT COUNT(*) FROM required_major_lines)::bigint,
               COALESCE((SELECT bool_and(valid) FROM observed),false),
               (SELECT MAX(materialized_at) FROM observed WHERE valid)
        "#,
    )
    .fetch_one(pool)
    .await
}

async fn load_freshness(pool: &PgPool) -> Result<Vec<FreshnessRow>, sqlx::Error> {
    sqlx::query_as(
        "SELECT p.freshness_state,p.last_checked_at,p.last_succeeded_at,\
                p.sanitized_failure_code \
         FROM dataset_source_bindings b \
         JOIN datasets d ON d.id=b.dataset_id AND d.lifecycle_state <> 'tombstoned' \
         LEFT JOIN dataset_sync_partitions p ON p.source_binding_id=b.id \
         ORDER BY b.id",
    )
    .fetch_all(pool)
    .await
}

fn compose_snapshot(
    database: DatabaseState,
    configuration: Option<DatasetConfigurationV1>,
    security: Option<SecurityState>,
    bindings_compatible: bool,
    product: ProductRow,
    freshness_rows: Vec<FreshnessRow>,
) -> OperationalSnapshot {
    let configuration_state = match configuration.as_ref() {
        None => ConfigurationState::Missing,
        Some(value) if validate_configuration(value).valid => ConfigurationState::Valid,
        Some(_) => ConfigurationState::Invalid,
    };
    let security_state = SecurityProjectionState::from_security(security.as_ref());
    let (active_datasets, required_major_lines, all_valid, last_materialization_at) = product;
    let product_state = if active_datasets == 0 {
        ProductState::Empty
    } else if required_major_lines > 0 && all_valid {
        ProductState::LastGoodAvailable
    } else {
        ProductState::LastGoodMissing
    };
    let freshness = compose_freshness(product_state, freshness_rows);
    let mut failures = BTreeSet::new();
    match configuration_state {
        ConfigurationState::Valid => {}
        ConfigurationState::Missing => {
            failures.insert(CONFIGURATION_MISSING_FAILURE.to_string());
        }
        ConfigurationState::Invalid => {
            failures.insert(CONFIGURATION_INVALID_FAILURE.to_string());
        }
    }
    match security_state {
        SecurityProjectionState::Installed => {}
        SecurityProjectionState::Degraded => {
            failures.insert(SECURITY_DEGRADED_FAILURE.to_string());
        }
        SecurityProjectionState::Missing => {
            failures.insert(SECURITY_MISSING_FAILURE.to_string());
        }
        SecurityProjectionState::Disabled => {
            failures.insert(SECURITY_DISABLED_FAILURE.to_string());
        }
        SecurityProjectionState::Recovery => {
            failures.insert(SECURITY_RECOVERY_FAILURE.to_string());
        }
    }
    if !bindings_compatible {
        failures.insert(BINDING_INCOMPATIBLE_FAILURE.to_string());
    }
    if product_state == ProductState::LastGoodMissing {
        failures.insert(LAST_GOOD_MISSING_FAILURE.to_string());
    }
    if let Some(code) = freshness
        .failure_code
        .clone()
        .or_else(|| freshness.state.failure_code().map(str::to_owned))
    {
        failures.insert(code);
    }
    OperationalSnapshot {
        database,
        configuration_state,
        configuration,
        security_state,
        security,
        bindings_compatible,
        product_state,
        last_materialization_at,
        freshness,
        failures: failures.into_iter().collect(),
    }
}

fn compose_freshness(product_state: ProductState, rows: Vec<FreshnessRow>) -> FreshnessObservation {
    if product_state == ProductState::Empty {
        return FreshnessObservation {
            state: FreshnessState::NotApplicable,
            last_observation_at: None,
            last_success_at: None,
            failure_code: None,
        };
    }
    let mut states = BTreeSet::new();
    let mut last_observation_at = None;
    let mut last_success_at = None;
    let mut failure_codes = BTreeSet::new();
    for (state, observed_at, succeeded_at, failure_code) in rows {
        states.insert(state.unwrap_or_else(|| "never_materialized".into()));
        if observed_at > last_observation_at {
            last_observation_at = observed_at;
        }
        if succeeded_at > last_success_at {
            last_success_at = succeeded_at;
        }
        if let Some(code) = failure_code.and_then(sanitize_failure_code) {
            failure_codes.insert(code);
        }
    }
    let state = if states.contains("failed") {
        FreshnessState::Failed
    } else if states.contains("degraded") {
        FreshnessState::Degraded
    } else if states.contains("stale") {
        FreshnessState::Stale
    } else if states.contains("refreshing") {
        FreshnessState::Refreshing
    } else if states.is_empty() || states.contains("never_materialized") {
        FreshnessState::NeverMaterialized
    } else {
        FreshnessState::Current
    };
    FreshnessObservation {
        state,
        last_observation_at,
        last_success_at,
        failure_code: failure_codes.into_iter().next(),
    }
}

fn sanitize_failure_code(code: String) -> Option<String> {
    let code = code.trim();
    (code.starts_with("dataset.")
        && code.len() <= 120
        && code.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        }))
    .then(|| code.to_owned())
}

fn database_unavailable_snapshot() -> OperationalSnapshot {
    OperationalSnapshot {
        database: DatabaseState::Unavailable,
        configuration_state: ConfigurationState::Missing,
        configuration: None,
        security_state: SecurityProjectionState::Missing,
        security: None,
        bindings_compatible: true,
        product_state: ProductState::LastGoodMissing,
        last_materialization_at: None,
        freshness: FreshnessObservation {
            state: FreshnessState::NeverMaterialized,
            last_observation_at: None,
            last_success_at: None,
            failure_code: None,
        },
        failures: vec![DATABASE_FAILURE.to_string()],
    }
}

fn required_binding_diagnostics(
    compatible: bool,
    response_observation: &BindingObservation,
) -> Vec<BindingDiagnostics> {
    let status = if compatible {
        "compatible"
    } else {
        "incompatible"
    };
    vec![
        BindingDiagnostics {
            binding_key: tessara_responses_contract::RESPONSE_EXPORT_BINDING_KEY,
            contract_id: tessara_responses_contract::RESPONSE_EXPORT_CONTRACT_ID,
            contract_version: tessara_responses_contract::RESPONSE_EXPORT_CONTRACT_VERSION,
            status,
            last_observation_at: response_observation.last_observation_at,
            last_success_at: response_observation.last_success_at,
            freshness: response_observation.freshness.as_str(),
            failure_code: response_observation.failure_code.clone(),
        },
        unobserved_binding(
            tessara_forms_contract::FORM_VERSION_SCHEMA_BINDING_KEY,
            tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_ID,
            tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_VERSION,
            status,
        ),
        unobserved_binding(
            tessara_control_plane_contract::SCOPE_CATALOG_BINDING_KEY,
            tessara_control_plane_contract::SCOPE_CATALOG_CONTRACT_ID,
            tessara_control_plane_contract::SCOPE_CATALOG_CONTRACT_VERSION,
            status,
        ),
        unobserved_binding(
            tessara_control_plane_contract::PRINCIPAL_DISPLAY_BINDING_KEY,
            tessara_control_plane_contract::PRINCIPAL_DISPLAY_CONTRACT_ID,
            tessara_control_plane_contract::PRINCIPAL_DISPLAY_CONTRACT_VERSION,
            status,
        ),
    ]
}

fn unobserved_binding(
    binding_key: &'static str,
    contract_id: &'static str,
    contract_version: &'static str,
    status: &'static str,
) -> BindingDiagnostics {
    BindingDiagnostics {
        binding_key,
        contract_id,
        contract_version,
        status,
        last_observation_at: None,
        last_success_at: None,
        freshness: "not_observed",
        failure_code: None,
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    fn valid_configuration() -> DatasetConfigurationV1 {
        DatasetConfigurationV1::default()
    }

    fn serving_security(document_state: &str) -> SecurityState {
        SecurityState {
            installation_id: Uuid::from_u128(1),
            module_instance_id: Uuid::from_u128(2),
            authorization_revision: 3,
            organization_revision: 4,
            enabled: true,
            document_state: document_state.into(),
            updated_at: Utc::now(),
        }
    }

    #[test]
    fn controlled_empty_bootstrap_is_ready() {
        let snapshot = compose_snapshot(
            DatabaseState::Ready,
            Some(valid_configuration()),
            Some(serving_security("enabled")),
            true,
            (0, 0, false, None),
            Vec::new(),
        );
        assert_eq!(snapshot.readiness().status, "ready");
        assert_eq!(snapshot.readiness().product_state, "empty");
        assert_eq!(snapshot.readiness().freshness, "not_applicable");
        assert!(snapshot.readiness().failures.is_empty());
    }

    #[test]
    fn missing_binding_and_last_good_fail_closed_with_stable_codes() {
        let snapshot = compose_snapshot(
            DatabaseState::Ready,
            Some(valid_configuration()),
            Some(serving_security("enabled")),
            false,
            (1, 1, false, None),
            vec![(Some("failed".into()), None, None, None)],
        );
        let readiness = snapshot.readiness();
        assert_eq!(readiness.status, "not_ready");
        assert_eq!(readiness.required_bindings, "incompatible");
        assert_eq!(readiness.product_state, "last_good_missing");
        assert_eq!(readiness.freshness, "failed");
        assert!(
            readiness
                .failures
                .contains(&BINDING_INCOMPATIBLE_FAILURE.to_string())
        );
        assert!(
            readiness
                .failures
                .contains(&LAST_GOOD_MISSING_FAILURE.to_string())
        );
        assert!(
            readiness
                .failures
                .contains(&"dataset.freshness.failed".to_string())
        );
    }

    #[test]
    fn valid_last_good_turns_failed_source_observation_into_degraded_readiness() {
        let observed = Utc::now();
        let snapshot = compose_snapshot(
            DatabaseState::Ready,
            Some(valid_configuration()),
            Some(serving_security("enabled")),
            true,
            (1, 1, true, Some(observed)),
            vec![(
                Some("failed".into()),
                Some(observed),
                Some(observed),
                Some("dataset.dependency_unavailable".into()),
            )],
        );
        let readiness = snapshot.readiness();
        assert_eq!(readiness.status, "degraded");
        assert_eq!(readiness.product_state, "last_good_available");
        assert_eq!(readiness.freshness, "failed");
        assert_eq!(
            readiness.failures,
            vec!["dataset.dependency_unavailable".to_string()]
        );
    }

    #[test]
    fn unsafe_persisted_failure_detail_is_never_reported() {
        assert_eq!(
            sanitize_failure_code("dataset.dependency_unavailable".into()),
            Some("dataset.dependency_unavailable".into())
        );
        assert_eq!(
            sanitize_failure_code("https://provider.internal/secret".into()),
            None
        );
        assert_eq!(
            sanitize_failure_code("dataset.failure: detail".into()),
            None
        );
    }
}
