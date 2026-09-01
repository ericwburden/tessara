//! Response-owned composition bootstrap and signed logical-key read-back.

use std::collections::{BTreeMap, BTreeSet};

use axum::{
    Json,
    body::to_bytes,
    extract::{Request, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tessara_composition::{
    BootstrapReceiptV1, OwnerBootstrapProviderActionV1, OwnerBootstrapRequestV1,
    OwnerBootstrapResponseV1,
};
use tessara_forms_contract::FormVersionSchemaResponse;
use tessara_module_contract::{AuthorizationAudienceV1, ModuleDefinitionId, ServiceActionMethod};
use tessara_module_runtime::SecurityStateProvider;
use tessara_responses_contract::{ResponseLifecycleState, ResponseReference};
use tessara_workflows_contract::WorkflowResponseStartContext;
use uuid::Uuid;

use crate::{
    CreateResponseCommand, IdempotentCommit, MODULE_DEFINITION_ID, RESPONSE_FORM_BINDING,
    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING, RESPONSE_WORKFLOW_CONTEXT_BINDING, ResponseAccess,
    ResponseMutationCommand, ResponseOwnerError, ResponseOwnerRepository, ResponseRuntime,
    ResponseValueInput, StartResponseClaimCommand, canonical_digest,
};

pub const RESPONSE_BOOTSTRAP_SCHEMA_VERSION: &str = "tessara.io/response-bootstrap/v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseBootstrapV1 {
    pub schema_version: String,
    pub responses: Vec<ResponseBootstrapDefinitionV1>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseBootstrapDefinitionV1 {
    pub resource_key: String,
    pub form: FormVersionSchemaResponse,
    pub workflow: WorkflowResponseStartContext,
    #[serde(default)]
    pub values: BTreeMap<String, Value>,
    pub lifecycle_state: ResponseBootstrapLifecycleStateV1,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseBootstrapLifecycleStateV1 {
    Draft,
    Submitted,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseBootstrapReadBackV1 {
    pub schema_version: u16,
    pub response: ResponseReference,
    pub lifecycle_state: ResponseLifecycleState,
    pub revision: u64,
    pub workflow_assignment_id: Uuid,
    pub workflow_instance_id: Uuid,
    pub workflow_step_instance_id: Uuid,
    pub workflow_event_sequence: u64,
}

pub(crate) async fn apply_bootstrap(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    request: Request,
) -> Response {
    bootstrap_response(apply(&runtime, request).await)
}

async fn apply(
    runtime: &ResponseRuntime,
    request: Request,
) -> Result<Json<OwnerBootstrapResponseV1>, BootstrapError> {
    require_control_key(request.headers())?;
    if request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
    {
        return Err(BootstrapError::BadRequest);
    }
    let bytes = to_bytes(request.into_body(), 1024 * 1024)
        .await
        .map_err(|_| BootstrapError::BadRequest)?;
    let request: OwnerBootstrapRequestV1<ResponseBootstrapV1> =
        serde_json::from_slice(&bytes).map_err(|_| BootstrapError::BadRequest)?;
    if request.input.schema_version != RESPONSE_BOOTSTRAP_SCHEMA_VERSION
        || request.apply_sequence == 0
        || request.dependency_validation.is_some()
        || request.idempotency_key.trim().is_empty()
        || !request
            .validate_input_digest()
            .map_err(|_| BootstrapError::BadRequest)?
    {
        return Err(BootstrapError::BadRequest);
    }
    let security = runtime
        .current_security_state()
        .await
        .map_err(|_| BootstrapError::Unavailable)?;
    let owner = AuthorizationAudienceV1::ModuleInstance {
        module_instance_id: security.module_instance_id,
        module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
            .map_err(|_| BootstrapError::Internal)?,
    };
    request
        .validate_authorization_for(
            &runtime.core_owner_bootstrap_verifier,
            &owner,
            MODULE_DEFINITION_ID,
            Utc::now(),
        )
        .map_err(|_| BootstrapError::Forbidden)?;
    validate_provider_compatibility_authority(
        &request.authorization.payload.provider_actions,
        security.installation_id,
    )?;
    validate_input(&request.input)?;

    let mut transaction = runtime.pool.begin().await?;
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(&request.idempotency_key)
        .execute(&mut *transaction)
        .await?;

    if let Some((locked, input, revision, sequence, stored)) =
        sqlx::query_as::<_, (String, String, i64, i64, Value)>(
            "SELECT locked_input_digest,input_digest,desired_revision,apply_sequence,receipt
             FROM response_bootstrap_receipts WHERE idempotency_key=$1 FOR UPDATE",
        )
        .bind(&request.idempotency_key)
        .fetch_optional(&mut *transaction)
        .await?
    {
        if locked != request.locked_input_digest.to_string()
            || input != request.input_digest.to_string()
            || revision != request.desired_revision as i64
            || sequence != request.apply_sequence as i64
        {
            return Err(BootstrapError::Conflict);
        }
        let mut response: OwnerBootstrapResponseV1 =
            serde_json::from_value(stored).map_err(|_| BootstrapError::Internal)?;
        response.receipt.changed = false;
        response.signed_receipt = runtime
            .bootstrap_receipt_signer
            .sign(response.receipt.clone())
            .map_err(|_| BootstrapError::Internal)?;
        record_provider_compatibility(&mut transaction).await?;
        transaction.commit().await?;
        return Ok(Json(response));
    }

    let mut resources = BTreeMap::new();
    for (index, definition) in request.input.responses.iter().enumerate() {
        let mut response = ResponseReference::from_parts(
            security.installation_id,
            security.module_instance_id,
            Uuid::new_v4(),
        )
        .map_err(|_| BootstrapError::Internal)?;
        let form_snapshot =
            serde_json::to_value(&definition.form).map_err(|_| BootstrapError::Internal)?;
        let workflow_context =
            serde_json::to_value(&definition.workflow).map_err(|_| BootstrapError::Internal)?;
        let identity_digest = canonical_digest(&(
            request.input_digest.to_string(),
            definition.resource_key.as_str(),
        ))?;
        let expires_at = DateTime::parse_from_rfc3339(&definition.workflow.expires_at)
            .map_err(|_| BootstrapError::Validation)?
            .with_timezone(&Utc);
        ResponseOwnerRepository::claim_start_in_transaction(
            &mut transaction,
            &StartResponseClaimCommand {
                workflow_assignment_id: definition.workflow.workflow_assignment_id,
                workflow_instance_id: definition.workflow.workflow_instance_id,
                workflow_step_instance_id: definition.workflow.workflow_step_instance_id,
                one_use_nonce: definition.workflow.one_use_nonce,
                actor_account_id: definition.workflow.started_by_account_id,
                idempotency_key_digest: identity_digest.clone(),
                request_digest: identity_digest.clone(),
                authorization_grant_jti: request.authorization.payload.jti,
                authorization_correlation_id: request.authorization.payload.correlation_id,
                expires_at,
            },
        )
        .await?;
        let created = ResponseOwnerRepository::create_in_transaction(
            &mut transaction,
            &CreateResponseCommand {
                response: response.clone(),
                form_id: definition.workflow.form_id,
                form_version_id: definition.workflow.form_version_id,
                node_id: definition.workflow.node_id,
                workflow_assignment_id: definition.workflow.workflow_assignment_id,
                workflow_version_id: definition.workflow.workflow_version_id,
                workflow_step_id: definition.workflow.workflow_step_id,
                workflow_instance_id: definition.workflow.workflow_instance_id,
                workflow_step_instance_id: definition.workflow.workflow_step_instance_id,
                workflow_start_nonce: definition.workflow.one_use_nonce,
                assignee_account_id: definition.workflow.assignee_account_id,
                started_by_account_id: definition.workflow.started_by_account_id,
                delegation_basis: definition.workflow.delegation_basis.clone(),
                form_snapshot_digest: canonical_digest(&form_snapshot)?,
                form_snapshot,
                workflow_context_digest: canonical_digest(&workflow_context)?,
                workflow_context,
                values: bootstrap_values(definition)?,
                idempotency_key_digest: identity_digest.clone(),
                request_digest: identity_digest.clone(),
            },
        )
        .await?;
        let snapshot = match created {
            IdempotentCommit::Applied(snapshot) | IdempotentCommit::Replayed(snapshot) => snapshot,
        };
        response = snapshot.response.clone();
        let mut lifecycle_state = snapshot.state;
        let mut revision = snapshot.revision;
        if definition.lifecycle_state == ResponseBootstrapLifecycleStateV1::Submitted
            && lifecycle_state == ResponseLifecycleState::Draft
        {
            let access = ResponseAccess {
                installation_id: security.installation_id,
                actor_account_id: definition.workflow.started_by_account_id,
                delegated_account_ids: BTreeSet::new(),
                respond_node_ids: BTreeSet::new(),
                respond_all: true,
                managed_node_ids: BTreeSet::new(),
                manage_all: false,
            };
            let submit_digest = canonical_digest(&(identity_digest.as_str(), "submit"))?;
            let submitted = match ResponseOwnerRepository::submit_in_transaction(
                &mut transaction,
                &access,
                &ResponseMutationCommand {
                    response_id: response.response_id(),
                    expected_revision: revision,
                    actor_account_id: definition.workflow.started_by_account_id,
                    idempotency_key_digest: submit_digest.clone(),
                    request_digest: submit_digest,
                    authorization_grant_jti: request.authorization.payload.jti,
                    authorization_correlation_id: request.authorization.payload.correlation_id,
                },
            )
            .await?
            {
                IdempotentCommit::Applied(result) | IdempotentCommit::Replayed(result) => result,
            };
            lifecycle_state = ResponseLifecycleState::Submitted;
            revision = submitted.revision;
        }
        let workflow_event_sequence: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sequence),0) FROM response_workflow_events WHERE response_id=$1",
        )
        .bind(response.response_id())
        .fetch_one(&mut *transaction)
        .await?;
        let read_back = ResponseBootstrapReadBackV1 {
            schema_version: 1,
            response: response.clone(),
            lifecycle_state,
            revision,
            workflow_assignment_id: definition.workflow.workflow_assignment_id,
            workflow_instance_id: definition.workflow.workflow_instance_id,
            workflow_step_instance_id: definition.workflow.workflow_step_instance_id,
            workflow_event_sequence: u64::try_from(workflow_event_sequence)
                .map_err(|_| BootstrapError::Internal)?,
        };
        resources.insert(
            definition.resource_key.clone(),
            serde_json::to_string(&response).map_err(|_| BootstrapError::Internal)?,
        );
        resources.insert(
            format!("{}.read_back", definition.resource_key),
            serde_json::to_string(&read_back).map_err(|_| BootstrapError::Internal)?,
        );
        runtime
            .validation_fault_control
            .fail_after_first_owner_write(index + 1)?;
    }
    let result_digest =
        tessara_composition::canonical_digest(&resources).map_err(|_| BootstrapError::Internal)?;
    record_provider_compatibility(&mut transaction).await?;
    let response = OwnerBootstrapResponseV1::signed(
        BootstrapReceiptV1 {
            owner: MODULE_DEFINITION_ID.into(),
            schema_version: RESPONSE_BOOTSTRAP_SCHEMA_VERSION.into(),
            input_digest: request.input_digest.clone(),
            result_digest,
            changed: true,
            resource_ids: resources,
        },
        &runtime.bootstrap_receipt_signer,
    )
    .map_err(|_| BootstrapError::Internal)?;
    sqlx::query(
        "INSERT INTO response_bootstrap_receipts
         (idempotency_key,locked_input_digest,input_digest,desired_revision,apply_sequence,authority_jti,receipt)
         VALUES($1,$2,$3,$4,$5,$6,$7)",
    )
    .bind(&request.idempotency_key)
    .bind(request.locked_input_digest.to_string())
    .bind(request.input_digest.to_string())
    .bind(request.desired_revision as i64)
    .bind(request.apply_sequence as i64)
    .bind(request.authorization.payload.jti)
    .bind(serde_json::to_value(&response).map_err(|_| BootstrapError::Internal)?)
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Json(response))
}

fn validate_provider_compatibility_authority(
    actions: &[OwnerBootstrapProviderActionV1],
    installation_id: Uuid,
) -> Result<(), BootstrapError> {
    let expected = [
        (
            RESPONSE_FORM_BINDING,
            tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_ID,
            tessara_forms_contract::RESPONSE_FORM_VERSION_SCHEMA_ACTION,
            tessara_forms_contract::RESPONSE_FORM_VERSION_SCHEMA_PATH,
        ),
        (
            RESPONSE_WORKFLOW_CONTEXT_BINDING,
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_ACTION,
            tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_PATH,
        ),
        (
            RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION,
            tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
        ),
    ];
    if actions.len() != expected.len()
        || expected.iter().any(|(binding, contract, action, path)| {
            actions
                .iter()
                .filter(|candidate| {
                    candidate.dependency_binding == *binding
                        && candidate.functional_contract == *contract
                        && candidate.action == *action
                        && candidate.method == ServiceActionMethod::Post
                        && candidate.path == *path
                        && matches!(
                            candidate.audience,
                            AuthorizationAudienceV1::CoreInstallation {
                                installation_id: candidate_installation
                            } if candidate_installation == installation_id
                        )
                })
                .count()
                != 1
        })
    {
        return Err(BootstrapError::Forbidden);
    }
    Ok(())
}

async fn record_provider_compatibility(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
) -> Result<(), BootstrapError> {
    let result = sqlx::query(
        "UPDATE response_provider_observations
         SET compatibility_state='compatible',last_observed_at=now(),last_compatible_at=now(),
             last_stable_finding=NULL",
    )
    .execute(&mut **transaction)
    .await?;
    if result.rows_affected() != 3 {
        return Err(BootstrapError::Internal);
    }
    Ok(())
}

fn validate_input(input: &ResponseBootstrapV1) -> Result<(), BootstrapError> {
    if input.responses.is_empty() {
        return Err(BootstrapError::Validation);
    }
    let mut keys = BTreeSet::new();
    for definition in &input.responses {
        if definition.resource_key.trim().is_empty()
            || definition.resource_key.ends_with(".read_back")
            || !keys.insert(definition.resource_key.as_str())
            || definition
                .form
                .validate_for(definition.workflow.form_version_id)
                .is_err()
            || definition
                .workflow
                .validate_for(definition.workflow.workflow_assignment_id)
                .is_err()
            || definition.form.form_id != definition.workflow.form_id
            || !definition
                .form
                .source_scope_node_ids
                .contains(&definition.workflow.node_id)
            || bootstrap_values(definition).is_err()
        {
            return Err(BootstrapError::Validation);
        }
    }
    Ok(())
}

fn bootstrap_values(
    definition: &ResponseBootstrapDefinitionV1,
) -> Result<Vec<ResponseValueInput>, BootstrapError> {
    let fields = definition
        .form
        .fields
        .iter()
        .map(|field| (field.key.as_str(), field))
        .collect::<BTreeMap<_, _>>();
    definition
        .values
        .iter()
        .map(|(key, value)| {
            let field = fields.get(key.as_str()).ok_or(BootstrapError::Validation)?;
            crate::product_store::validate_field_value(&field.field_type, &field.options, value)
                .map_err(|_| BootstrapError::Validation)?;
            Ok(ResponseValueInput {
                field_id: field.field_id,
                field_key: field.key.clone(),
                value: value.clone(),
                value_text: crate::product_store::value_text(value),
            })
        })
        .collect()
}

fn require_control_key(headers: &axum::http::HeaderMap) -> Result<(), BootstrapError> {
    let expected = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    let actual = headers
        .get("x-tessara-module-control-key")
        .and_then(|value| value.to_str().ok());
    (actual == Some(expected.as_str()))
        .then_some(())
        .ok_or(BootstrapError::Forbidden)
}

#[derive(Debug, thiserror::Error)]
enum BootstrapError {
    #[error("forbidden")]
    Forbidden,
    #[error("invalid Response bootstrap request")]
    BadRequest,
    #[error("invalid Response bootstrap fixture")]
    Validation,
    #[error("Response bootstrap idempotency conflict")]
    Conflict,
    #[error("Response bootstrap is unavailable")]
    Unavailable,
    #[error("Response bootstrap failed")]
    Internal,
    #[error(transparent)]
    Owner(#[from] ResponseOwnerError),
    #[error(transparent)]
    Persistence(#[from] sqlx::Error),
}

fn bootstrap_response(result: Result<Json<OwnerBootstrapResponseV1>, BootstrapError>) -> Response {
    match result {
        Ok(response) => response.into_response(),
        Err(error) => {
            let status = match error {
                BootstrapError::Forbidden => StatusCode::FORBIDDEN,
                BootstrapError::BadRequest | BootstrapError::Validation => StatusCode::BAD_REQUEST,
                BootstrapError::Conflict
                | BootstrapError::Owner(ResponseOwnerError::IdempotencyConflict) => {
                    StatusCode::CONFLICT
                }
                BootstrapError::Owner(ResponseOwnerError::StartLeaseExpired) => StatusCode::GONE,
                BootstrapError::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
                BootstrapError::Internal
                | BootstrapError::Owner(_)
                | BootstrapError::Persistence(_) => StatusCode::INTERNAL_SERVER_ERROR,
            };
            (
                status,
                Json(serde_json::json!({ "error": error.to_string() })),
            )
                .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bootstrap_schema_is_owner_specific_and_rejects_empty_input() {
        assert_eq!(
            RESPONSE_BOOTSTRAP_SCHEMA_VERSION,
            "tessara.io/response-bootstrap/v1"
        );
        assert!(
            validate_input(&ResponseBootstrapV1 {
                schema_version: RESPONSE_BOOTSTRAP_SCHEMA_VERSION.into(),
                responses: Vec::new(),
            })
            .is_err()
        );

        let installation_id = Uuid::from_u128(1);
        let mut actions = [
            (
                RESPONSE_FORM_BINDING,
                tessara_forms_contract::FORM_VERSION_SCHEMA_CONTRACT_ID,
                tessara_forms_contract::RESPONSE_FORM_VERSION_SCHEMA_ACTION,
                tessara_forms_contract::RESPONSE_FORM_VERSION_SCHEMA_PATH,
            ),
            (
                RESPONSE_WORKFLOW_CONTEXT_BINDING,
                tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
                tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_ACTION,
                tessara_workflows_contract::WORKFLOW_RESPONSE_CONTEXT_PATH,
            ),
            (
                RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
                tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION,
                tessara_workflows_contract::WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
            ),
        ]
        .into_iter()
        .map(
            |(binding, contract, action, path)| OwnerBootstrapProviderActionV1 {
                dependency_binding: binding.into(),
                functional_contract: contract.into(),
                action: action.into(),
                method: ServiceActionMethod::Post,
                path: path.into(),
                audience: AuthorizationAudienceV1::CoreInstallation { installation_id },
            },
        )
        .collect::<Vec<_>>();
        validate_provider_compatibility_authority(&actions, installation_id)
            .expect("signed bootstrap resolves every exact Core provider action");
        actions[0].audience = AuthorizationAudienceV1::CoreInstallation {
            installation_id: Uuid::from_u128(2),
        };
        assert!(
            validate_provider_compatibility_authority(&actions, installation_id).is_err(),
            "a healthy wrong-owner provider authority must be rejected"
        );
    }
}
