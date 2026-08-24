use std::collections::BTreeSet;

use axum::{
    Json,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tessara_forms_contract::{
    FORM_VERSION_SCHEMA_CONTRACT_ID, FORM_VERSION_SCHEMA_MEDIA_TYPE, FORM_VERSION_SCHEMA_VERSION,
    FormVersionSchemaAction, FormVersionSchemaRequest, FormVersionSchemaResponse,
    RESPONSE_FORM_VERSION_SCHEMA_ACTION, RESPONSE_FORM_VERSION_SCHEMA_PATH,
};
use tessara_module_contract::{
    AuthorizationAudienceV1, AuthorizationGrantOperationV1, AuthorizationGrantV3,
    AuthorizationValidationContextV3, DependencyBindingKey, FunctionalContractId,
    ModuleDefinitionId, ModuleServicePrincipalV1, SecurityCapabilityId, SignedEnvelopeV1,
};
use tessara_module_runtime::{
    SecurityStateProvider, decode_signed_envelope_header, request_correlation_id,
};
use tessara_responses_contract::{
    RESPONSE_IDEMPOTENCY_HEADER, RESPONSE_LIFECYCLE_CONTRACT_ID, RESPONSE_RESOURCE_CONTRACT_ID,
    ResponseMutationResult, ResponseReference, ResponseRevisionRequest, ResponseStartOption,
    ResponseStartOptions, SaveResponseValuesRequest, StartResponseRequest,
};
use tessara_workflows_contract::{
    WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
    WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE, WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
    WORKFLOW_RESPONSE_CONTEXT_ACTION, WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
    WORKFLOW_RESPONSE_CONTEXT_MEDIA_TYPE, WORKFLOW_RESPONSE_CONTEXT_PATH,
    WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION, WorkflowResponseAssignmentCatalogRequest,
    WorkflowResponseAssignmentCatalogResponse, WorkflowResponseContextRequest,
    WorkflowResponseContextResponse, WorkflowResponseContextState,
};
use uuid::Uuid;

use crate::provider_client::{self, ProviderAction, ProviderClientError};
use crate::{
    CreateResponseCommand, IdempotentCommit, MODULE_DEFINITION_ID, RESPONSE_FORM_BINDING,
    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING, RESPONSE_WORKFLOW_CONTEXT_BINDING, ResponseAccess,
    ResponseListFilter, ResponseMutationCommand, ResponseOwnerError, ResponseOwnerRepository,
    ResponseRuntime, SaveResponseCommand, StartResponseClaimCommand, canonical_digest,
};

const CORE_RESPONSE_BINDING: &str = "tessara.core.responses";

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ListResponsesQuery {
    status: Option<String>,
    form_id: Option<Uuid>,
    node_id: Option<Uuid>,
    delegate_account_id: Option<Uuid>,
    q: Option<String>,
}

pub(crate) async fn list_responses(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    Query(query): Query<ListResponsesQuery>,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.list",
            AuthorizationGrantOperationV1::Read,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            "submissions:read_own",
        )
        .await?;
        let access = response_access(&grant.payload)?;
        let assignee_account_id = match query.delegate_account_id {
            None => None,
            Some(id)
                if id == access.actor_account_id
                    || access.delegated_account_ids.contains(&id)
                    || access.manage_all =>
            {
                Some(id)
            }
            Some(_) => return Err(ProductApiError::Forbidden),
        };
        let values = ResponseOwnerRepository::new(runtime.pool.clone())
            .list(
                &access,
                &ResponseListFilter {
                    status: query.status,
                    form_id: query.form_id,
                    node_id: query.node_id,
                    search: query.q,
                    assignee_account_id,
                },
            )
            .await?;
        Ok((StatusCode::OK, Json(values)).into_response())
    })
    .await
}

pub(crate) async fn list_start_options(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    Query(query): Query<StartOptionsQuery>,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.start_options",
            AuthorizationGrantOperationV1::Read,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            "submissions:respond",
        )
        .await?;
        let access = response_access(&grant.payload)?;
        let assignee_account_id = match query.delegate_account_id {
            None => grant.payload.original_actor_id,
            Some(id)
                if id == access.actor_account_id || access.delegated_account_ids.contains(&id) =>
            {
                id
            }
            Some(_) => return Err(ProductApiError::Forbidden),
        };
        let response: WorkflowResponseAssignmentCatalogResponse = provider_client::post(
            &runtime,
            &grant,
            ProviderAction {
                binding: RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                contract: WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_CONTRACT_ID,
                action: WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_ACTION,
                path: WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_PATH,
                media_type: WORKFLOW_RESPONSE_ASSIGNMENT_CATALOG_MEDIA_TYPE,
            },
            &WorkflowResponseAssignmentCatalogRequest {
                schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
                assignee_account_id,
            },
        )
        .await?;
        response
            .validate_for(assignee_account_id)
            .map_err(|_| ProductApiError::Incompatible)?;
        match response.state {
            WorkflowResponseContextState::Available => {}
            WorkflowResponseContextState::Undisclosed => return Err(ProductApiError::Forbidden),
            WorkflowResponseContextState::Unavailable => return Err(ProductApiError::Unavailable),
            WorkflowResponseContextState::Incompatible => {
                return Err(ProductApiError::Incompatible);
            }
        }
        let assignments = response
            .assignments
            .into_iter()
            .map(|item| ResponseStartOption {
                workflow_assignment_id: item.workflow_assignment_id,
                workflow_name: item.workflow_name,
                workflow_version_label: item.workflow_version_label,
                workflow_step_title: item.workflow_step_title,
                workflow_step_position: item.workflow_step_position,
                workflow_step_count: item.workflow_step_count,
                form_id: item.form_id,
                form_name: item.form_name,
                form_version_id: item.form_version_id,
                form_version_label: item.form_version_label,
                node_id: item.node_id,
                node_name: item.node_name,
                account_id: item.assignee_account_id,
                account_display_name: item.assignee_display_name,
            })
            .collect();
        Ok((StatusCode::OK, Json(ResponseStartOptions { assignments })).into_response())
    })
    .await
}

#[derive(Clone, Copy, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StartOptionsQuery {
    delegate_account_id: Option<Uuid>,
}

pub(crate) async fn start_response(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    Json(request): Json<StartResponseRequest>,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.start",
            AuthorizationGrantOperationV1::Mutation,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            "submissions:respond",
        )
        .await?;
        let idempotency_key_digest = idempotency_key_digest(&headers)?;
        let request_digest = request_digest("responses.start", Uuid::nil(), &request)?;
        let repository = ResponseOwnerRepository::new(runtime.pool.clone());
        if let Some(replayed) = repository
            .replay_create(
                grant.payload.original_actor_id,
                &idempotency_key_digest,
                &request_digest,
            )
            .await?
        {
            return Ok(start_result_response(
                replayed.response.response_id(),
                replayed.revision,
                true,
            ));
        }
        let workflow_response: WorkflowResponseContextResponse = provider_client::post(
            &runtime,
            &grant,
            ProviderAction {
                binding: RESPONSE_WORKFLOW_CONTEXT_BINDING,
                contract: WORKFLOW_RESPONSE_CONTEXT_CONTRACT_ID,
                action: WORKFLOW_RESPONSE_CONTEXT_ACTION,
                path: WORKFLOW_RESPONSE_CONTEXT_PATH,
                media_type: WORKFLOW_RESPONSE_CONTEXT_MEDIA_TYPE,
            },
            &WorkflowResponseContextRequest {
                schema_version: WORKFLOW_RESPONSE_CONTEXT_SCHEMA_VERSION,
                workflow_assignment_id: request.workflow_assignment_id,
            },
        )
        .await?;
        workflow_response
            .validate_for(request.workflow_assignment_id)
            .map_err(|_| ProductApiError::Incompatible)?;
        let workflow = match (workflow_response.state, workflow_response.context) {
            (WorkflowResponseContextState::Available, Some(context)) => context,
            (WorkflowResponseContextState::Undisclosed, None) => {
                return Err(ProductApiError::NotFound);
            }
            (WorkflowResponseContextState::Unavailable, None) => {
                return Err(ProductApiError::Unavailable);
            }
            (WorkflowResponseContextState::Incompatible, None) => {
                return Err(ProductApiError::Incompatible);
            }
            _ => return Err(ProductApiError::Incompatible),
        };
        validate_start_authority(&workflow, grant.payload.original_actor_id)?;
        let expires_at = chrono::DateTime::parse_from_rfc3339(&workflow.expires_at)
            .map_err(|_| ProductApiError::Incompatible)?
            .with_timezone(&chrono::Utc);
        repository
            .claim_start(&StartResponseClaimCommand {
                workflow_assignment_id: workflow.workflow_assignment_id,
                workflow_instance_id: workflow.workflow_instance_id,
                workflow_step_instance_id: workflow.workflow_step_instance_id,
                one_use_nonce: workflow.one_use_nonce,
                actor_account_id: grant.payload.original_actor_id,
                idempotency_key_digest: idempotency_key_digest.clone(),
                request_digest: request_digest.clone(),
                expires_at,
            })
            .await?;
        let form: FormVersionSchemaResponse = provider_client::post(
            &runtime,
            &grant,
            ProviderAction {
                binding: RESPONSE_FORM_BINDING,
                contract: FORM_VERSION_SCHEMA_CONTRACT_ID,
                action: RESPONSE_FORM_VERSION_SCHEMA_ACTION,
                path: RESPONSE_FORM_VERSION_SCHEMA_PATH,
                media_type: FORM_VERSION_SCHEMA_MEDIA_TYPE,
            },
            &FormVersionSchemaRequest {
                schema_version: FORM_VERSION_SCHEMA_VERSION,
                action: FormVersionSchemaAction::ResolveSchema,
                form_version_id: workflow.form_version_id,
            },
        )
        .await?;
        form.validate_for(workflow.form_version_id)
            .map_err(|_| ProductApiError::Incompatible)?;
        if form.form_id != workflow.form_id
            || !form.source_scope_node_ids.contains(&workflow.node_id)
        {
            return Err(ProductApiError::Incompatible);
        }
        let security = runtime
            .current_security_state()
            .await
            .map_err(|_| ProductApiError::Unavailable)?;
        let response_id = Uuid::new_v4();
        let reference = ResponseReference::from_parts(
            security.installation_id,
            security.module_instance_id,
            response_id,
        )
        .map_err(|_| ProductApiError::Internal)?;
        let form_snapshot = serde_json::to_value(&form).map_err(|_| ProductApiError::Internal)?;
        let workflow_context =
            serde_json::to_value(&workflow).map_err(|_| ProductApiError::Internal)?;
        let result = repository
            .create(&CreateResponseCommand {
                response: reference,
                form_id: workflow.form_id,
                form_version_id: workflow.form_version_id,
                node_id: workflow.node_id,
                workflow_assignment_id: workflow.workflow_assignment_id,
                workflow_version_id: workflow.workflow_version_id,
                workflow_step_id: workflow.workflow_step_id,
                workflow_instance_id: workflow.workflow_instance_id,
                workflow_step_instance_id: workflow.workflow_step_instance_id,
                workflow_start_nonce: workflow.one_use_nonce,
                assignee_account_id: workflow.assignee_account_id,
                started_by_account_id: workflow.started_by_account_id,
                delegation_basis: workflow.delegation_basis.clone(),
                form_snapshot_digest: canonical_digest(&form_snapshot)
                    .map_err(|_| ProductApiError::Internal)?,
                form_snapshot,
                workflow_context_digest: canonical_digest(&workflow_context)
                    .map_err(|_| ProductApiError::Internal)?,
                workflow_context,
                values: Vec::new(),
                idempotency_key_digest,
                request_digest,
            })
            .await?;
        Ok(match result {
            IdempotentCommit::Applied(snapshot) => {
                start_result_response(snapshot.response.response_id(), snapshot.revision, false)
            }
            IdempotentCommit::Replayed(snapshot) => {
                start_result_response(snapshot.response.response_id(), snapshot.revision, true)
            }
        })
    })
    .await
}

fn validate_start_authority(
    context: &tessara_workflows_contract::WorkflowResponseStartContext,
    actor_account_id: Uuid,
) -> Result<(), ProductApiError> {
    if context.started_by_account_id != actor_account_id {
        return Err(ProductApiError::Forbidden);
    }
    let issued_at = chrono::DateTime::parse_from_rfc3339(&context.issued_at)
        .map_err(|_| ProductApiError::Incompatible)?
        .with_timezone(&chrono::Utc);
    let expires_at = chrono::DateTime::parse_from_rfc3339(&context.expires_at)
        .map_err(|_| ProductApiError::Incompatible)?
        .with_timezone(&chrono::Utc);
    let now = chrono::Utc::now();
    if issued_at > now || expires_at <= now || expires_at <= issued_at {
        return Err(ProductApiError::Forbidden);
    }
    Ok(())
}

fn start_result_response(response_id: Uuid, revision: u64, replayed: bool) -> Response {
    let status = if replayed {
        StatusCode::OK
    } else {
        StatusCode::CREATED
    };
    let mut response = (
        status,
        Json(ResponseMutationResult {
            id: response_id,
            revision,
            status: "draft".into(),
        }),
    )
        .into_response();
    if replayed {
        response
            .headers_mut()
            .insert("x-tessara-idempotent-replay", "true".parse().unwrap());
    }
    response
}

pub(crate) async fn get_response(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    Path(response_id): Path<Uuid>,
    headers: HeaderMap,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.get",
            AuthorizationGrantOperationV1::Read,
            RESPONSE_RESOURCE_CONTRACT_ID,
            "submissions:read_own",
        )
        .await?;
        let detail = ResponseOwnerRepository::new(runtime.pool.clone())
            .detail(&response_access(&grant.payload)?, response_id)
            .await?
            .ok_or(ProductApiError::NotFound)?;
        Ok((StatusCode::OK, Json(detail)).into_response())
    })
    .await
}

pub(crate) async fn save_response(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    Path(response_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<SaveResponseValuesRequest>,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.save",
            AuthorizationGrantOperationV1::Mutation,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            "submissions:respond",
        )
        .await?;
        let idempotency_key_digest = idempotency_key_digest(&headers)?;
        let request_digest = request_digest("responses.save", response_id, &request)?;
        let result = ResponseOwnerRepository::new(runtime.pool.clone())
            .save(
                &response_access(&grant.payload)?,
                &SaveResponseCommand {
                    response_id,
                    expected_revision: request.expected_revision,
                    values: request.values,
                    actor_account_id: grant.payload.original_actor_id,
                    idempotency_key_digest,
                    request_digest,
                },
            )
            .await?;
        Ok(mutation_response(result))
    })
    .await
}

pub(crate) async fn submit_response(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    Path(response_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ResponseRevisionRequest>,
) -> Response {
    mutate_response(runtime, headers, response_id, request, MutationKind::Submit).await
}

pub(crate) async fn delete_response(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    Path(response_id): Path<Uuid>,
    headers: HeaderMap,
    Json(request): Json<ResponseRevisionRequest>,
) -> Response {
    mutate_response(runtime, headers, response_id, request, MutationKind::Delete).await
}

#[derive(Clone, Copy)]
enum MutationKind {
    Submit,
    Delete,
}

async fn mutate_response(
    runtime: std::sync::Arc<ResponseRuntime>,
    headers: HeaderMap,
    response_id: Uuid,
    request: ResponseRevisionRequest,
    kind: MutationKind,
) -> Response {
    product_response(async {
        let (action, capability) = match kind {
            MutationKind::Submit => ("responses.submit", "submissions:respond"),
            MutationKind::Delete => ("responses.delete", "submissions:manage"),
        };
        let grant = authorize_product(
            &runtime,
            &headers,
            action,
            AuthorizationGrantOperationV1::Mutation,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            capability,
        )
        .await?;
        let command = ResponseMutationCommand {
            response_id,
            expected_revision: request.expected_revision,
            actor_account_id: grant.payload.original_actor_id,
            idempotency_key_digest: idempotency_key_digest(&headers)?,
            request_digest: request_digest(action, response_id, &request)?,
        };
        let repository = ResponseOwnerRepository::new(runtime.pool.clone());
        let result = match kind {
            MutationKind::Submit => {
                repository
                    .submit(&response_access(&grant.payload)?, &command)
                    .await?
            }
            MutationKind::Delete => {
                repository
                    .delete(&response_access(&grant.payload)?, &command)
                    .await?
            }
        };
        Ok(mutation_response(result))
    })
    .await
}

fn mutation_response(result: IdempotentCommit<ResponseMutationResult>) -> Response {
    let (status, value, replayed) = match result {
        IdempotentCommit::Applied(value) => (StatusCode::OK, value, false),
        IdempotentCommit::Replayed(value) => (StatusCode::OK, value, true),
    };
    let mut response = (status, Json(value)).into_response();
    if replayed {
        response
            .headers_mut()
            .insert("x-tessara-idempotent-replay", "true".parse().unwrap());
    }
    response
}

async fn authorize_product(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    action: &str,
    operation: AuthorizationGrantOperationV1,
    contract: &str,
    required_capability: &str,
) -> Result<SignedEnvelopeV1<AuthorizationGrantV3>, ProductApiError> {
    let envelope: SignedEnvelopeV1<AuthorizationGrantV3> =
        decode_signed_envelope_header(headers, "x-tessara-authorization")
            .map_err(|_| ProductApiError::Forbidden)?;
    runtime
        .verifiers
        .authorization
        .verify(&envelope)
        .map_err(|_| ProductApiError::Forbidden)?;
    let security = runtime
        .current_security_state()
        .await
        .map_err(|_| ProductApiError::Unavailable)?;
    if !security.enabled || security.document_state != "enabled" {
        return Err(ProductApiError::Unavailable);
    }
    let correlation_id = request_correlation_id(headers).map_err(|_| ProductApiError::Forbidden)?;
    envelope
        .payload
        .validate_for(&AuthorizationValidationContextV3 {
            installation_id: security.installation_id,
            correlation_id,
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: security.module_instance_id,
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID)
                    .map_err(|_| ProductApiError::Internal)?,
            },
            dependency_binding: DependencyBindingKey::new(CORE_RESPONSE_BINDING)
                .map_err(|_| ProductApiError::Internal)?,
            functional_contract: FunctionalContractId::new(contract)
                .map_err(|_| ProductApiError::Internal)?,
            action: action.into(),
            operation,
            resource_assertion: None,
            authorization_revision: security.authorization_revision,
            organization_revision: security.organization_revision,
            now: chrono::Utc::now(),
        })
        .map_err(|_| ProductApiError::Forbidden)?;
    let required =
        SecurityCapabilityId::new(required_capability).map_err(|_| ProductApiError::Internal)?;
    if !envelope
        .payload
        .capability_scope_bindings
        .iter()
        .any(|binding| binding.capability == required)
    {
        return Err(ProductApiError::Forbidden);
    }
    Ok(envelope)
}

fn response_access(grant: &AuthorizationGrantV3) -> Result<ResponseAccess, ProductApiError> {
    let manage =
        SecurityCapabilityId::new("submissions:manage").map_err(|_| ProductApiError::Internal)?;
    let mut managed_node_ids = BTreeSet::new();
    let mut manage_all = false;
    for binding in grant
        .capability_scope_bindings
        .iter()
        .filter(|binding| binding.capability == manage)
    {
        if binding.organization_root_id == grant.installation_id {
            manage_all = true;
        } else {
            managed_node_ids.insert(binding.organization_root_id);
            managed_node_ids.extend(binding.authorized_organization_ids.iter().copied());
        }
    }
    Ok(ResponseAccess {
        installation_id: grant.installation_id,
        actor_account_id: grant.original_actor_id,
        delegated_account_ids: grant
            .delegation_basis
            .iter()
            .map(|basis| basis.delegated_by_actor_id)
            .collect(),
        managed_node_ids,
        manage_all,
    })
}

fn idempotency_key_digest(headers: &HeaderMap) -> Result<String, ProductApiError> {
    let key = headers
        .get(RESPONSE_IDEMPOTENCY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .ok_or(ProductApiError::BadRequest)?;
    canonical_digest(key).map_err(|_| ProductApiError::Internal)
}

fn request_digest<T: Serialize>(
    action: &str,
    response_id: Uuid,
    request: &T,
) -> Result<String, ProductApiError> {
    canonical_digest(&(action, response_id, request)).map_err(|_| ProductApiError::Internal)
}

async fn product_response(
    future: impl std::future::Future<Output = Result<Response, ProductApiError>>,
) -> Response {
    future.await.unwrap_or_else(IntoResponse::into_response)
}

#[derive(Debug)]
enum ProductApiError {
    BadRequest,
    Forbidden,
    NotFound,
    Conflict(String),
    Unavailable,
    Incompatible,
    Internal,
}

impl From<ResponseOwnerError> for ProductApiError {
    fn from(error: ResponseOwnerError) -> Self {
        match error {
            ResponseOwnerError::InvalidCommand
            | ResponseOwnerError::InvalidField(_)
            | ResponseOwnerError::MissingRequiredField(_) => Self::BadRequest,
            ResponseOwnerError::NotFound => Self::NotFound,
            ResponseOwnerError::Immutable
            | ResponseOwnerError::IdempotencyConflict
            | ResponseOwnerError::StartLeaseExpired
            | ResponseOwnerError::RevisionConflict { .. } => Self::Conflict(error.to_string()),
            ResponseOwnerError::Persistence(_) | ResponseOwnerError::SecurityStateUnavailable => {
                Self::Unavailable
            }
            ResponseOwnerError::CorruptReceipt
            | ResponseOwnerError::CorruptSnapshot
            | ResponseOwnerError::InvariantViolation(_) => Self::Internal,
        }
    }
}

impl From<ProviderClientError> for ProductApiError {
    fn from(error: ProviderClientError) -> Self {
        match error {
            ProviderClientError::Restricted => Self::Forbidden,
            ProviderClientError::Unavailable => Self::Unavailable,
            ProviderClientError::Incompatible => Self::Incompatible,
            ProviderClientError::Internal => Self::Internal,
        }
    }
}

impl IntoResponse for ProductApiError {
    fn into_response(self) -> Response {
        #[derive(Serialize)]
        struct ErrorBody {
            error: &'static str,
            message: String,
        }
        let (status, error, message) = match self {
            Self::BadRequest => (
                StatusCode::BAD_REQUEST,
                "bad_request",
                "Response request is invalid".into(),
            ),
            Self::Forbidden => (
                StatusCode::FORBIDDEN,
                "forbidden",
                "Response action is unavailable".into(),
            ),
            Self::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "Response was not found".into(),
            ),
            Self::Conflict(message) => (StatusCode::CONFLICT, "conflict", message),
            Self::Unavailable => (
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "Response service is unavailable".into(),
            ),
            Self::Incompatible => (
                StatusCode::FAILED_DEPENDENCY,
                "dependency_incompatible",
                "A required Response provider is incompatible".into(),
            ),
            Self::Internal => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "Response service failed".into(),
            ),
        };
        (status, Json(ErrorBody { error, message })).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn idempotency_header_is_bounded_and_required() {
        let mut headers = HeaderMap::new();
        assert!(matches!(
            idempotency_key_digest(&headers),
            Err(ProductApiError::BadRequest)
        ));
        headers.insert(RESPONSE_IDEMPOTENCY_HEADER, " key ".parse().unwrap());
        assert!(
            idempotency_key_digest(&headers)
                .unwrap()
                .starts_with("sha256:")
        );
        headers.insert(
            RESPONSE_IDEMPOTENCY_HEADER,
            "x".repeat(201).parse().unwrap(),
        );
        assert!(matches!(
            idempotency_key_digest(&headers),
            Err(ProductApiError::BadRequest)
        ));
    }
}
