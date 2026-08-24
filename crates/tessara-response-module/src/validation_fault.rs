//! Sprint 8C validation-profile-only deterministic Response failure control.
//!
//! Normal Response builds cannot arm this control. The disposable Sprint 8C
//! validation image opts into the feature and must still project the exact
//! contract, scope, key, correlation identity, and one-attempt limit.

use std::env;

#[cfg(feature = "sprint-8c-validation-faults")]
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

#[cfg(feature = "sprint-8c-validation-faults")]
use serde::Serialize;
#[cfg(any(feature = "sprint-8c-validation-faults", test))]
use uuid::Uuid;

use crate::ResponseOwnerError;

#[cfg(any(feature = "sprint-8c-validation-faults", test))]
const CONTRACT: &str = "tessara.sprint-8c.failure-control/v1";
#[cfg(any(feature = "sprint-8c-validation-faults", test))]
const SCOPE: &str = "disposable-sprint-8c-only";
const MID_APPLY_FAULT: &str = "response.bootstrap.mid-apply";

const CONTRACT_ENV: &str = "TESSARA_SPRINT_8C_FAULT_CONTRACT";
const SCOPE_ENV: &str = "TESSARA_SPRINT_8C_FAULT_SCOPE";
const KEY_ENV: &str = "TESSARA_SPRINT_8C_FAULT_KEY";
const CORRELATION_ENV: &str = "TESSARA_SPRINT_8C_FAULT_CORRELATION_ID";
const ATTEMPT_LIMIT_ENV: &str = "TESSARA_SPRINT_8C_FAULT_ATTEMPT_LIMIT";

#[derive(Clone, Debug, Default)]
pub struct ResponseValidationFaultControl {
    #[cfg(feature = "sprint-8c-validation-faults")]
    armed: Option<Arc<ArmedMidApplyFault>>,
}

#[cfg(feature = "sprint-8c-validation-faults")]
#[derive(Debug)]
struct ArmedMidApplyFault {
    correlation_id: Uuid,
    attempt_consumed: AtomicBool,
}

#[derive(Debug, thiserror::Error)]
pub enum ResponseValidationFaultConfigurationError {
    #[error("unknown Sprint 8C Response validation fault '{0}'")]
    UnknownFault(String),
    #[error("Sprint 8C Response validation fault controls are not compiled into this runtime")]
    ProfileUnavailable,
    #[error("Sprint 8C Response validation fault projection is invalid")]
    InvalidProjection,
}

impl ResponseValidationFaultControl {
    pub fn disabled() -> Self {
        Self::default()
    }

    pub fn from_environment() -> Result<Self, ResponseValidationFaultConfigurationError> {
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
    ) -> Result<Self, ResponseValidationFaultConfigurationError> {
        if key == "none" {
            return Ok(Self::disabled());
        }
        if key != MID_APPLY_FAULT {
            return Err(ResponseValidationFaultConfigurationError::UnknownFault(key));
        }

        #[cfg(not(feature = "sprint-8c-validation-faults"))]
        {
            let _ = (contract, scope, correlation_id, attempt_limit);
            Err(ResponseValidationFaultConfigurationError::ProfileUnavailable)
        }

        #[cfg(feature = "sprint-8c-validation-faults")]
        {
            let correlation_id = Uuid::parse_str(&correlation_id)
                .ok()
                .filter(|value| !value.is_nil())
                .ok_or(ResponseValidationFaultConfigurationError::InvalidProjection)?;
            if contract != CONTRACT || scope != SCOPE || attempt_limit != "1" {
                return Err(ResponseValidationFaultConfigurationError::InvalidProjection);
            }
            Ok(Self {
                armed: Some(Arc::new(ArmedMidApplyFault {
                    correlation_id,
                    attempt_consumed: AtomicBool::new(false),
                })),
            })
        }
    }

    #[cfg(feature = "sprint-8c-validation-faults")]
    #[doc(hidden)]
    pub fn deterministic_mid_apply_for_test(correlation_id: Uuid) -> Self {
        Self {
            armed: Some(Arc::new(ArmedMidApplyFault {
                correlation_id,
                attempt_consumed: AtomicBool::new(false),
            })),
        }
    }

    pub(crate) fn fail_after_first_owner_write(
        &self,
        completed_response_count: usize,
    ) -> Result<(), ResponseOwnerError> {
        #[cfg(not(feature = "sprint-8c-validation-faults"))]
        {
            let _ = completed_response_count;
            Ok(())
        }

        #[cfg(feature = "sprint-8c-validation-faults")]
        {
            let Some(armed) = self.armed.as_ref() else {
                return Ok(());
            };
            if completed_response_count == 0 {
                return Ok(());
            }
            if armed.attempt_consumed.swap(true, Ordering::SeqCst) {
                return Ok(());
            }
            let receipt = ResponseValidationFaultReceipt {
                schema_version: 1,
                contract: CONTRACT,
                fault_key: MID_APPLY_FAULT,
                correlation_id: armed.correlation_id,
                target_service: "responses",
                phase: "response_bootstrap_transaction",
                attempt: 1,
                attempt_limit: 1,
                outcome: "rolled_back",
                failure_code: "response.bootstrap.injected_failure",
                response_transaction: "rolled_back",
                unauthorized_state: "none",
                cross_owner_write: "none",
            };
            let receipt = serde_json::to_string(&receipt).map_err(|_| {
                ResponseOwnerError::InvariantViolation(
                    "Response validation fault receipt serialization failed".into(),
                )
            })?;
            tracing::error!(fault_receipt = %receipt, "Sprint 8C Response validation fault");
            Err(ResponseOwnerError::BootstrapFaultInjected)
        }
    }
}

#[cfg(feature = "sprint-8c-validation-faults")]
#[derive(Serialize)]
struct ResponseValidationFaultReceipt<'a> {
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
    response_transaction: &'a str,
    unauthorized_state: &'a str,
    cross_owner_write: &'a str,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disabled_and_unknown_faults_are_exact() {
        assert!(
            ResponseValidationFaultControl::from_values(
                String::new(),
                String::new(),
                "none".into(),
                String::new(),
                "0".into(),
            )
            .is_ok()
        );
        assert!(matches!(
            ResponseValidationFaultControl::from_values(
                CONTRACT.into(),
                SCOPE.into(),
                "response.bootstrap.unknown".into(),
                Uuid::new_v4().to_string(),
                "1".into(),
            ),
            Err(ResponseValidationFaultConfigurationError::UnknownFault(_))
        ));
    }

    #[cfg(feature = "sprint-8c-validation-faults")]
    #[test]
    fn mid_apply_fault_requires_exact_projection_and_owner_write() {
        let correlation_id = Uuid::new_v4();
        for (contract, scope, attempt_limit) in [
            ("wrong", SCOPE, "1"),
            (CONTRACT, "wrong", "1"),
            (CONTRACT, SCOPE, "2"),
        ] {
            assert!(matches!(
                ResponseValidationFaultControl::from_values(
                    contract.into(),
                    scope.into(),
                    MID_APPLY_FAULT.into(),
                    correlation_id.to_string(),
                    attempt_limit.into(),
                ),
                Err(ResponseValidationFaultConfigurationError::InvalidProjection)
            ));
        }
        let control = ResponseValidationFaultControl::from_values(
            CONTRACT.into(),
            SCOPE.into(),
            MID_APPLY_FAULT.into(),
            correlation_id.to_string(),
            "1".into(),
        )
        .expect("exact validation fault projection");
        control
            .fail_after_first_owner_write(0)
            .expect("pre-write boundary");
        assert!(matches!(
            control.fail_after_first_owner_write(1),
            Err(ResponseOwnerError::BootstrapFaultInjected)
        ));
        control
            .fail_after_first_owner_write(2)
            .expect("the one permitted validation fault attempt must disarm");
    }

    #[cfg(not(feature = "sprint-8c-validation-faults"))]
    #[test]
    fn active_fault_is_unavailable_outside_validation_profile() {
        assert!(matches!(
            ResponseValidationFaultControl::from_values(
                CONTRACT.into(),
                SCOPE.into(),
                MID_APPLY_FAULT.into(),
                Uuid::new_v4().to_string(),
                "1".into(),
            ),
            Err(ResponseValidationFaultConfigurationError::ProfileUnavailable)
        ));
    }
}
