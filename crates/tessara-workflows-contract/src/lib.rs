//! Public contracts owned by the Workflow provider.
//!
//! Responses may start only from an authenticated, one-use context issued for
//! one active assignment. The context contains immutable identifiers and a
//! canonical digest so the Response owner never needs Workflow table access.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub const WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID: &str = "tessara.workflows.response-context";
pub const WORKFLOW_RESPONSE_CONTEXT_VERSION: &str = "1.0.0";
pub const WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION: u16 = 1;
pub const WORKFLOW_RESPONSE_CONTEXT_BINDING_KEY: &str = "tessara.responses.workflow-context";
pub const WORKFLOW_RESPONSE_CONTEXT_ACTION: &str = "workflows.issue_response_context";
pub const WORKFLOW_RESPONSE_CONTEXT_PATH: &str = "/api/private/workflows/response-context";
pub const WORKFLOW_RESPONSE_CONTEXT_MEDIA_TYPE: &str =
    "application/vnd.tessara.workflows.response-context+json;version=1";
pub const WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID: &str =
    "tessara.workflows.response-assignment-catalog";
pub const WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_VERSION: &str = "1.0.0";
pub const WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION: &str =
    "workflows.response_assignment_catalog";
pub const WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH: &str =
    "/api/private/workflows/response-assignment-catalog";
pub const WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE: &str =
    "application/vnd.tessara.workflows.response-assignment-catalog+json;version=1";

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseContextRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub workflow_assignment_id: Uuid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowResponseContextState {
    Available,
    Undisclosed,
    Unavailable,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseStepSnapshot {
    pub workflow_step_id: Uuid,
    pub title: String,
    pub form_name: String,
    pub status: String,
    pub position: i32,
    pub completed_at: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseStartContext {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub workflow_assignment_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_description: String,
    pub workflow_version_id: Uuid,
    pub workflow_version_label: Option<String>,
    pub workflow_step_id: Uuid,
    pub workflow_step_title: String,
    pub workflow_step_position: i32,
    pub workflow_step_count: i64,
    pub next_workflow_step_title: Option<String>,
    pub next_workflow_step_form_name: Option<String>,
    pub history: Vec<WorkflowResponseStepSnapshot>,
    pub workflow_instance_id: Uuid,
    pub workflow_step_instance_id: Uuid,
    pub form_id: Uuid,
    pub form_version_id: Uuid,
    pub node_id: Uuid,
    pub node_name: String,
    pub assignee_account_id: Uuid,
    pub assignee_display_name: String,
    pub started_by_account_id: Uuid,
    pub delegation_basis: Option<String>,
    pub one_use_nonce: Uuid,
    pub issued_at: String,
    pub expires_at: String,
    pub context_digest: String,
}

impl WorkflowResponseStartContext {
    pub fn with_recomputed_digest(mut self) -> Result<Self, WorkflowResponseContextError> {
        self.validate_shape()?;
        self.context_digest = self.expected_digest()?;
        Ok(self)
    }

    pub fn validate_for(
        &self,
        requested_assignment_id: Uuid,
    ) -> Result<(), WorkflowResponseContextError> {
        self.validate_shape()?;
        if self.workflow_assignment_id != requested_assignment_id {
            return Err(WorkflowResponseContextError::AssignmentMismatch);
        }
        if self.context_digest != self.expected_digest()? {
            return Err(WorkflowResponseContextError::DigestMismatch);
        }
        Ok(())
    }

    fn validate_shape(&self) -> Result<(), WorkflowResponseContextError> {
        if self.schema_version != WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION {
            return Err(WorkflowResponseContextError::SchemaVersion);
        }
        let ids = [
            self.workflow_assignment_id,
            self.workflow_id,
            self.workflow_version_id,
            self.workflow_step_id,
            self.workflow_instance_id,
            self.workflow_step_instance_id,
            self.form_id,
            self.form_version_id,
            self.node_id,
            self.assignee_account_id,
            self.started_by_account_id,
            self.one_use_nonce,
        ];
        if ids.iter().any(Uuid::is_nil) {
            return Err(WorkflowResponseContextError::Identity);
        }
        if self.issued_at.trim().is_empty()
            || self.expires_at.trim().is_empty()
            || self.workflow_name.trim().is_empty()
            || self.workflow_step_title.trim().is_empty()
            || self.workflow_step_position < 0
            || self.workflow_step_count <= 0
            || self.node_name.trim().is_empty()
            || self.assignee_display_name.trim().is_empty()
            || self.delegation_basis.as_deref().is_some_and(str::is_empty)
        {
            return Err(WorkflowResponseContextError::AuthorityWindow);
        }
        let mut previous_position = None;
        for step in &self.history {
            if step.workflow_step_id.is_nil()
                || step.title.trim().is_empty()
                || step.form_name.trim().is_empty()
                || step.status.trim().is_empty()
                || step.position < 0
                || previous_position.is_some_and(|position| position >= step.position)
            {
                return Err(WorkflowResponseContextError::Identity);
            }
            previous_position = Some(step.position);
        }
        Ok(())
    }

    fn expected_digest(&self) -> Result<String, WorkflowResponseContextError> {
        #[derive(Serialize)]
        struct DigestInput<'a> {
            schema_version: u16,
            workflow_assignment_id: &'a Uuid,
            workflow_id: &'a Uuid,
            workflow_name: &'a str,
            workflow_description: &'a str,
            workflow_version_id: &'a Uuid,
            workflow_version_label: &'a Option<String>,
            workflow_step_id: &'a Uuid,
            workflow_step_title: &'a str,
            workflow_step_position: i32,
            workflow_step_count: i64,
            next_workflow_step_title: &'a Option<String>,
            next_workflow_step_form_name: &'a Option<String>,
            history: &'a [WorkflowResponseStepSnapshot],
            workflow_instance_id: &'a Uuid,
            workflow_step_instance_id: &'a Uuid,
            form_id: &'a Uuid,
            form_version_id: &'a Uuid,
            node_id: &'a Uuid,
            node_name: &'a str,
            assignee_account_id: &'a Uuid,
            assignee_display_name: &'a str,
            started_by_account_id: &'a Uuid,
            delegation_basis: &'a Option<String>,
            one_use_nonce: &'a Uuid,
            issued_at: &'a str,
            expires_at: &'a str,
        }
        let bytes = serde_jcs::to_vec(&DigestInput {
            schema_version: self.schema_version,
            workflow_assignment_id: &self.workflow_assignment_id,
            workflow_id: &self.workflow_id,
            workflow_name: &self.workflow_name,
            workflow_description: &self.workflow_description,
            workflow_version_id: &self.workflow_version_id,
            workflow_version_label: &self.workflow_version_label,
            workflow_step_id: &self.workflow_step_id,
            workflow_step_title: &self.workflow_step_title,
            workflow_step_position: self.workflow_step_position,
            workflow_step_count: self.workflow_step_count,
            next_workflow_step_title: &self.next_workflow_step_title,
            next_workflow_step_form_name: &self.next_workflow_step_form_name,
            history: &self.history,
            workflow_instance_id: &self.workflow_instance_id,
            workflow_step_instance_id: &self.workflow_step_instance_id,
            form_id: &self.form_id,
            form_version_id: &self.form_version_id,
            node_id: &self.node_id,
            node_name: &self.node_name,
            assignee_account_id: &self.assignee_account_id,
            assignee_display_name: &self.assignee_display_name,
            started_by_account_id: &self.started_by_account_id,
            delegation_basis: &self.delegation_basis,
            one_use_nonce: &self.one_use_nonce,
            issued_at: &self.issued_at,
            expires_at: &self.expires_at,
        })
        .map_err(|_| WorkflowResponseContextError::Canonicalization)?;
        Ok(format!("sha256:{:x}", Sha256::digest(bytes)))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseAssignmentCatalogRequest {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub assignee_account_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseAssignmentCatalogItem {
    pub workflow_assignment_id: Uuid,
    pub workflow_id: Uuid,
    pub workflow_name: String,
    pub workflow_description: String,
    pub workflow_version_id: Uuid,
    pub workflow_version_label: Option<String>,
    pub workflow_step_id: Uuid,
    pub workflow_step_title: String,
    pub workflow_step_position: i32,
    pub workflow_step_count: i64,
    pub next_workflow_step_title: Option<String>,
    pub next_workflow_step_form_name: Option<String>,
    pub form_id: Uuid,
    pub form_name: String,
    pub form_version_id: Uuid,
    pub form_version_label: Option<String>,
    pub node_id: Uuid,
    pub node_name: String,
    pub assignee_account_id: Uuid,
    pub assignee_display_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseAssignmentCatalogResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub state: WorkflowResponseContextState,
    pub assignments: Vec<WorkflowResponseAssignmentCatalogItem>,
}

impl WorkflowResponseAssignmentCatalogResponse {
    pub fn validate_for(
        &self,
        assignee_account_id: Uuid,
    ) -> Result<(), WorkflowResponseContextError> {
        if self.schema_version != WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION
            || assignee_account_id.is_nil()
        {
            return Err(WorkflowResponseContextError::SchemaVersion);
        }
        match self.state {
            WorkflowResponseContextState::Available => {}
            WorkflowResponseContextState::Undisclosed
            | WorkflowResponseContextState::Unavailable
            | WorkflowResponseContextState::Incompatible => {
                return if self.assignments.is_empty() {
                    Ok(())
                } else {
                    Err(WorkflowResponseContextError::RestrictedCarriesContext)
                };
            }
        }
        let mut previous = None;
        for item in &self.assignments {
            if item.assignee_account_id != assignee_account_id
                || [
                    item.workflow_assignment_id,
                    item.workflow_id,
                    item.workflow_version_id,
                    item.workflow_step_id,
                    item.form_id,
                    item.form_version_id,
                    item.node_id,
                    item.assignee_account_id,
                ]
                .iter()
                .any(Uuid::is_nil)
                || item.workflow_name.trim().is_empty()
                || item.workflow_step_title.trim().is_empty()
                || item.form_name.trim().is_empty()
                || item.node_name.trim().is_empty()
                || item.assignee_display_name.trim().is_empty()
                || item.workflow_step_position < 0
                || item.workflow_step_count <= 0
                || previous.is_some_and(|id| id >= item.workflow_assignment_id)
            {
                return Err(WorkflowResponseContextError::Identity);
            }
            previous = Some(item.workflow_assignment_id);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WorkflowResponseContextResponse {
    #[serde(deserialize_with = "deserialize_schema_version")]
    pub schema_version: u16,
    pub state: WorkflowResponseContextState,
    pub context: Option<WorkflowResponseStartContext>,
}

impl WorkflowResponseContextResponse {
    pub fn validate_for(
        &self,
        requested_assignment_id: Uuid,
    ) -> Result<(), WorkflowResponseContextError> {
        if self.schema_version != WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION {
            return Err(WorkflowResponseContextError::SchemaVersion);
        }
        match (self.state, self.context.as_ref()) {
            (WorkflowResponseContextState::Available, Some(context)) => {
                context.validate_for(requested_assignment_id)
            }
            (WorkflowResponseContextState::Available, None) => {
                Err(WorkflowResponseContextError::AvailableWithoutContext)
            }
            (WorkflowResponseContextState::Undisclosed, None)
            | (WorkflowResponseContextState::Unavailable, None)
            | (WorkflowResponseContextState::Incompatible, None) => Ok(()),
            _ => Err(WorkflowResponseContextError::RestrictedCarriesContext),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, thiserror::Error)]
pub enum WorkflowResponseContextError {
    #[error("Workflow response-context schema version is unsupported")]
    SchemaVersion,
    #[error("Workflow response-context identity is invalid")]
    Identity,
    #[error("Workflow response-context authority window is invalid")]
    AuthorityWindow,
    #[error("Workflow response-context assignment does not match the request")]
    AssignmentMismatch,
    #[error("Workflow response-context digest does not match its immutable facts")]
    DigestMismatch,
    #[error("Workflow response-context could not be canonicalized")]
    Canonicalization,
    #[error("an available Workflow result requires a start context")]
    AvailableWithoutContext,
    #[error("an unavailable, incompatible, or undisclosed result cannot carry a context")]
    RestrictedCarriesContext,
}

fn deserialize_schema_version<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let version = u16::deserialize(deserializer)?;
    if version != WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION {
        return Err(serde::de::Error::custom(format!(
            "Workflow response-context schema version {version} is unsupported; expected {WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION}"
        )));
    }
    Ok(version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn context() -> WorkflowResponseStartContext {
        WorkflowResponseStartContext {
            schema_version: 1,
            workflow_assignment_id: Uuid::from_u128(1),
            workflow_id: Uuid::from_u128(10),
            workflow_name: "Enrollment".into(),
            workflow_description: "Enrollment workflow".into(),
            workflow_version_id: Uuid::from_u128(2),
            workflow_version_label: Some("Published".into()),
            workflow_step_id: Uuid::from_u128(3),
            workflow_step_title: "Applicant".into(),
            workflow_step_position: 0,
            workflow_step_count: 1,
            next_workflow_step_title: None,
            next_workflow_step_form_name: None,
            history: Vec::new(),
            workflow_instance_id: Uuid::from_u128(4),
            workflow_step_instance_id: Uuid::from_u128(5),
            form_id: Uuid::from_u128(11),
            form_version_id: Uuid::from_u128(6),
            node_id: Uuid::from_u128(7),
            node_name: "North".into(),
            assignee_account_id: Uuid::from_u128(8),
            assignee_display_name: "Ada".into(),
            started_by_account_id: Uuid::from_u128(8),
            delegation_basis: None,
            one_use_nonce: Uuid::from_u128(9),
            issued_at: "2026-08-23T12:00:00Z".into(),
            expires_at: "2026-08-23T12:05:00Z".into(),
            context_digest: String::new(),
        }
        .with_recomputed_digest()
        .unwrap()
    }

    #[test]
    fn context_is_assignment_and_content_bound() {
        let context = context();
        assert!(context.validate_for(Uuid::from_u128(1)).is_ok());
        assert_eq!(
            context.validate_for(Uuid::from_u128(99)),
            Err(WorkflowResponseContextError::AssignmentMismatch)
        );
        let mut tampered = context;
        tampered.form_version_id = Uuid::from_u128(60);
        assert_eq!(
            tampered.validate_for(Uuid::from_u128(1)),
            Err(WorkflowResponseContextError::DigestMismatch)
        );
    }

    #[test]
    fn nondisclosing_results_carry_no_context() {
        let response = WorkflowResponseContextResponse {
            schema_version: 1,
            state: WorkflowResponseContextState::Undisclosed,
            context: None,
        };
        assert!(response.validate_for(Uuid::from_u128(1)).is_ok());
        let leaking = WorkflowResponseContextResponse {
            context: Some(context()),
            ..response
        };
        assert_eq!(
            leaking.validate_for(Uuid::from_u128(1)),
            Err(WorkflowResponseContextError::RestrictedCarriesContext)
        );
    }

    #[test]
    fn request_rejects_unknown_fields_and_old_versions() {
        let request = json!({"schema_version":1,"workflow_assignment_id":Uuid::from_u128(1)});
        assert!(serde_json::from_value::<WorkflowResponseContextRequest>(request.clone()).is_ok());
        let mut old = request.clone();
        old["schema_version"] = json!(0);
        assert!(serde_json::from_value::<WorkflowResponseContextRequest>(old).is_err());
        let mut unknown = request;
        unknown["allow_direct_start"] = json!(true);
        assert!(serde_json::from_value::<WorkflowResponseContextRequest>(unknown).is_err());
    }
}
