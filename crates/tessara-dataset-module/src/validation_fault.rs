//! Sprint 8B validation-profile-only deterministic failure controls.
//!
//! The production/default feature set cannot arm these controls. The Sprint
//! 8B validation image opts into the feature explicitly and still requires an
//! exact disposable-scope contract projection before a fault can be armed.

use std::env;

#[cfg(feature = "sprint-8b-validation-faults")]
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[cfg(feature = "sprint-8b-validation-faults")]
use serde::Serialize;
#[cfg(any(feature = "sprint-8b-validation-faults", test))]
use uuid::Uuid;

use crate::DatasetModuleError;

#[cfg(any(feature = "sprint-8b-validation-faults", test))]
const CONTRACT: &str = "tessara.sprint-8b.failure-control/v1";
#[cfg(any(feature = "sprint-8b-validation-faults", test))]
const SCOPE: &str = "disposable-sprint-8b-only";
const DERIVED_REBUILD_FAULT: &str = "dataset.derived-rebuild";
#[cfg(feature = "sprint-8b-validation-faults")]
const DERIVED_RESOURCE_KEY: &str = "dataset.derived";

const CONTRACT_ENV: &str = "TESSARA_SPRINT_8B_FAULT_CONTRACT";
const SCOPE_ENV: &str = "TESSARA_SPRINT_8B_FAULT_SCOPE";
const KEY_ENV: &str = "TESSARA_SPRINT_8B_FAULT_KEY";
const CORRELATION_ENV: &str = "TESSARA_SPRINT_8B_FAULT_CORRELATION_ID";
const ATTEMPT_LIMIT_ENV: &str = "TESSARA_SPRINT_8B_FAULT_ATTEMPT_LIMIT";

#[derive(Clone, Debug, Default)]
pub struct DatasetValidationFaultControl {
    #[cfg(feature = "sprint-8b-validation-faults")]
    armed: Option<Arc<ArmedDerivedRebuildFault>>,
}

#[cfg(feature = "sprint-8b-validation-faults")]
#[derive(Debug)]
struct ArmedDerivedRebuildFault {
    correlation_id: Uuid,
    receipt_emitted: AtomicBool,
}

#[derive(Debug, thiserror::Error)]
pub enum DatasetValidationFaultConfigurationError {
    #[error("unknown Sprint 8B Dataset validation fault '{0}'")]
    UnknownFault(String),
    #[error("Sprint 8B Dataset validation fault controls are not compiled into this runtime")]
    ProfileUnavailable,
    #[error("Sprint 8B Dataset validation fault projection is invalid")]
    InvalidProjection,
}

impl DatasetValidationFaultControl {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn from_environment() -> Result<Self, DatasetValidationFaultConfigurationError> {
        Self::from_values(
            env::var(CONTRACT_ENV).unwrap_or_default(),
            env::var(SCOPE_ENV).unwrap_or_default(),
            env::var(KEY_ENV).unwrap_or_else(|_| "none".into()),
            env::var(CORRELATION_ENV).unwrap_or_default(),
            env::var(ATTEMPT_LIMIT_ENV).unwrap_or_else(|_| "0".into()),
        )
    }

    fn from_values(
        contract: String,
        scope: String,
        key: String,
        correlation_id: String,
        attempt_limit: String,
    ) -> Result<Self, DatasetValidationFaultConfigurationError> {
        if key == "none" {
            return Ok(Self::disabled());
        }
        if key != DERIVED_REBUILD_FAULT {
            return Err(DatasetValidationFaultConfigurationError::UnknownFault(key));
        }

        #[cfg(not(feature = "sprint-8b-validation-faults"))]
        {
            let _ = (contract, scope, correlation_id, attempt_limit);
            Err(DatasetValidationFaultConfigurationError::ProfileUnavailable)
        }

        #[cfg(feature = "sprint-8b-validation-faults")]
        {
            let correlation_id = Uuid::parse_str(&correlation_id)
                .ok()
                .filter(|value| !value.is_nil())
                .ok_or(DatasetValidationFaultConfigurationError::InvalidProjection)?;
            if contract != CONTRACT || scope != SCOPE || attempt_limit != "1" {
                return Err(DatasetValidationFaultConfigurationError::InvalidProjection);
            }
            Ok(Self {
                armed: Some(Arc::new(ArmedDerivedRebuildFault {
                    correlation_id,
                    receipt_emitted: AtomicBool::new(false),
                })),
            })
        }
    }

    #[cfg(feature = "sprint-8b-validation-faults")]
    #[doc(hidden)]
    pub fn deterministic_derived_rebuild_for_test(correlation_id: Uuid) -> Self {
        Self {
            armed: Some(Arc::new(ArmedDerivedRebuildFault {
                correlation_id,
                receipt_emitted: AtomicBool::new(false),
            })),
        }
    }

    pub(crate) fn fail_derived_rebuild(
        &self,
        resource_key: &str,
    ) -> Result<(), DatasetModuleError> {
        #[cfg(not(feature = "sprint-8b-validation-faults"))]
        {
            let _ = resource_key;
            Ok(())
        }

        #[cfg(feature = "sprint-8b-validation-faults")]
        {
            let Some(armed) = self.armed.as_ref() else {
                return Ok(());
            };
            if resource_key != DERIVED_RESOURCE_KEY {
                return Ok(());
            }
            if !armed.receipt_emitted.swap(true, Ordering::SeqCst) {
                let receipt = DatasetValidationFaultReceipt {
                    schema_version: 1,
                    contract: CONTRACT,
                    fault_key: DERIVED_REBUILD_FAULT,
                    correlation_id: armed.correlation_id,
                    target_service: "datasets",
                    phase: "dataset_bootstrap_transaction",
                    attempt: 1,
                    attempt_limit: 1,
                    outcome: "rolled_back",
                    failure_code: "dataset.dependency_unavailable",
                    dataset_transaction: "rolled_back",
                    unauthorized_state: "none",
                    cross_owner_write: "none",
                };
                let receipt = serde_json::to_string(&receipt)
                    .map_err(|error| DatasetModuleError::Internal(error.to_string()))?;
                tracing::error!(fault_receipt = %receipt, "Sprint 8B Dataset validation fault");
            }
            Err(DatasetModuleError::Unavailable(
                "Dataset validation-profile derived rebuild failed".into(),
            ))
        }
    }
}

#[cfg(feature = "sprint-8b-validation-faults")]
#[derive(Serialize)]
struct DatasetValidationFaultReceipt<'a> {
    schema_version: u16,
    contract: &'a str,
    fault_key: &'a str,
    correlation_id: Uuid,
    target_service: &'a str,
    phase: &'a str,
    attempt: u16,
    attempt_limit: u16,
    outcome: &'a str,
    failure_code: &'a str,
    dataset_transaction: &'a str,
    unauthorized_state: &'a str,
    cross_owner_write: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_or_unknown_faults_are_exact() {
        assert!(
            DatasetValidationFaultControl::from_values(
                String::new(),
                String::new(),
                "none".into(),
                String::new(),
                "0".into(),
            )
            .is_ok()
        );
        assert!(matches!(
            DatasetValidationFaultControl::from_values(
                CONTRACT.into(),
                SCOPE.into(),
                "dataset.some-other-fault".into(),
                Uuid::new_v4().to_string(),
                "1".into(),
            ),
            Err(DatasetValidationFaultConfigurationError::UnknownFault(_))
        ));
    }

    #[cfg(feature = "sprint-8b-validation-faults")]
    #[test]
    fn derived_fault_requires_exact_projection_and_cannot_be_bypassed() {
        let correlation_id = Uuid::new_v4();
        for (contract, scope, attempt_limit) in [
            ("wrong", SCOPE, "1"),
            (CONTRACT, "wrong", "1"),
            (CONTRACT, SCOPE, "2"),
        ] {
            assert!(matches!(
                DatasetValidationFaultControl::from_values(
                    contract.into(),
                    scope.into(),
                    DERIVED_REBUILD_FAULT.into(),
                    correlation_id.to_string(),
                    attempt_limit.into(),
                ),
                Err(DatasetValidationFaultConfigurationError::InvalidProjection)
            ));
        }
        let control = DatasetValidationFaultControl::from_values(
            CONTRACT.into(),
            SCOPE.into(),
            DERIVED_REBUILD_FAULT.into(),
            correlation_id.to_string(),
            "1".into(),
        )
        .expect("exact validation fault projection");
        control
            .fail_derived_rebuild("dataset.base")
            .expect("unselected Dataset resource");
        assert!(control.fail_derived_rebuild(DERIVED_RESOURCE_KEY).is_err());
        assert!(control.fail_derived_rebuild(DERIVED_RESOURCE_KEY).is_err());
    }

    #[cfg(not(feature = "sprint-8b-validation-faults"))]
    #[test]
    fn active_fault_is_unavailable_outside_the_validation_profile() {
        assert!(matches!(
            DatasetValidationFaultControl::from_values(
                CONTRACT.into(),
                SCOPE.into(),
                DERIVED_REBUILD_FAULT.into(),
                Uuid::new_v4().to_string(),
                "1".into(),
            ),
            Err(DatasetValidationFaultConfigurationError::ProfileUnavailable)
        ));
    }
}
