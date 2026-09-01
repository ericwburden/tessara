use std::collections::BTreeSet;

use axum::{
    Json,
    body::Bytes,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use sha2::{Digest, Sha256};
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
const RESPONSE_READ_CAPABILITIES: &[&str] = &["submissions:read_own", "submissions:manage"];
const RESPONSE_START_CAPABILITIES: &[&str] = &["submissions:respond", "submissions:manage"];

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
            RESPONSE_READ_CAPABILITIES,
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
            &["submissions:respond"],
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
        if response.validate_for(assignee_account_id).is_err() {
            provider_client::record_provider_observation(
                &runtime,
                RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                ProviderClientError::Incompatible,
            )
            .await;
            return Err(ProductApiError::Incompatible);
        }
        match response.state {
            WorkflowResponseContextState::Available => {
                provider_client::record_provider_compatible(
                    &runtime,
                    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                )
                .await;
            }
            WorkflowResponseContextState::Undisclosed => {
                provider_client::record_provider_compatible(
                    &runtime,
                    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                )
                .await;
                return Err(ProductApiError::Forbidden);
            }
            WorkflowResponseContextState::Unavailable => {
                provider_client::record_provider_observation(
                    &runtime,
                    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                    ProviderClientError::Unavailable,
                )
                .await;
                return Err(ProductApiError::Unavailable);
            }
            WorkflowResponseContextState::Incompatible => {
                provider_client::record_provider_observation(
                    &runtime,
                    RESPONSE_WORKFLOW_ASSIGNMENTS_BINDING,
                    ProviderClientError::Incompatible,
                )
                .await;
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
    body: Bytes,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.start",
            AuthorizationGrantOperationV1::Mutation,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            RESPONSE_START_CAPABILITIES,
        )
        .await?;
        let idempotency_key_digest = idempotency_key_digest(&headers)?;
        let request_digest = public_mutation_request_digest(
            &grant.payload,
            "POST",
            "/api/responses",
            &body,
            &idempotency_key_digest,
        )?;
        let request: StartResponseRequest = decode_public_mutation(&headers, &body)?;
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
        if workflow_response
            .validate_for(request.workflow_assignment_id)
            .is_err()
        {
            provider_client::record_provider_observation(
                &runtime,
                RESPONSE_WORKFLOW_CONTEXT_BINDING,
                ProviderClientError::Incompatible,
            )
            .await;
            return Err(ProductApiError::Incompatible);
        }
        let workflow = match (workflow_response.state, workflow_response.context) {
            (WorkflowResponseContextState::Available, Some(context)) => {
                provider_client::record_provider_compatible(
                    &runtime,
                    RESPONSE_WORKFLOW_CONTEXT_BINDING,
                )
                .await;
                context
            }
            (WorkflowResponseContextState::Undisclosed, None) => {
                provider_client::record_provider_compatible(
                    &runtime,
                    RESPONSE_WORKFLOW_CONTEXT_BINDING,
                )
                .await;
                return Err(ProductApiError::NotFound);
            }
            (WorkflowResponseContextState::Unavailable, None) => {
                provider_client::record_provider_observation(
                    &runtime,
                    RESPONSE_WORKFLOW_CONTEXT_BINDING,
                    ProviderClientError::Unavailable,
                )
                .await;
                return Err(ProductApiError::Unavailable);
            }
            (WorkflowResponseContextState::Incompatible, None) => {
                provider_client::record_provider_observation(
                    &runtime,
                    RESPONSE_WORKFLOW_CONTEXT_BINDING,
                    ProviderClientError::Incompatible,
                )
                .await;
                return Err(ProductApiError::Incompatible);
            }
            _ => {
                provider_client::record_provider_observation(
                    &runtime,
                    RESPONSE_WORKFLOW_CONTEXT_BINDING,
                    ProviderClientError::Incompatible,
                )
                .await;
                return Err(ProductApiError::Incompatible);
            }
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
                authorization_grant_jti: grant.payload.jti,
                authorization_correlation_id: grant.payload.correlation_id,
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
        if !form_schema_matches_workflow(&form, workflow.form_id, workflow.form_version_id) {
            provider_client::record_provider_observation(
                &runtime,
                RESPONSE_FORM_BINDING,
                ProviderClientError::Incompatible,
            )
            .await;
            return Err(ProductApiError::Incompatible);
        }
        provider_client::record_provider_compatible(&runtime, RESPONSE_FORM_BINDING).await;
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

fn form_schema_matches_workflow(
    form: &FormVersionSchemaResponse,
    workflow_form_id: Uuid,
    workflow_form_version_id: Uuid,
) -> bool {
    form.validate_for(workflow_form_version_id).is_ok() && form.form_id == workflow_form_id
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
            RESPONSE_READ_CAPABILITIES,
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
    body: Bytes,
) -> Response {
    product_response(async {
        let grant = authorize_product(
            &runtime,
            &headers,
            "responses.save",
            AuthorizationGrantOperationV1::Mutation,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            &["submissions:respond"],
        )
        .await?;
        let idempotency_key_digest = idempotency_key_digest(&headers)?;
        let path = format!("/api/responses/{response_id}/values");
        let request_digest = public_mutation_request_digest(
            &grant.payload,
            "PUT",
            &path,
            &body,
            &idempotency_key_digest,
        )?;
        let request: SaveResponseValuesRequest = decode_public_mutation(&headers, &body)?;
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
                    authorization_grant_jti: grant.payload.jti,
                    authorization_correlation_id: grant.payload.correlation_id,
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
    body: Bytes,
) -> Response {
    mutate_response(runtime, headers, response_id, body, MutationKind::Submit).await
}

pub(crate) async fn delete_response(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    Path(response_id): Path<Uuid>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    mutate_response(runtime, headers, response_id, body, MutationKind::Delete).await
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
    body: Bytes,
    kind: MutationKind,
) -> Response {
    product_response(async {
        let (action, method, path, capabilities): (&str, &str, String, &[&str]) = match kind {
            MutationKind::Submit => (
                "responses.submit",
                "POST",
                format!("/api/responses/{response_id}/submit"),
                &["submissions:respond"],
            ),
            MutationKind::Delete => (
                "responses.delete",
                "DELETE",
                format!("/api/responses/{response_id}"),
                &["submissions:respond", "submissions:manage"],
            ),
        };
        let grant = authorize_product(
            &runtime,
            &headers,
            action,
            AuthorizationGrantOperationV1::Mutation,
            RESPONSE_LIFECYCLE_CONTRACT_ID,
            capabilities,
        )
        .await?;
        let idempotency_key_digest = idempotency_key_digest(&headers)?;
        let request_digest = public_mutation_request_digest(
            &grant.payload,
            method,
            &path,
            &body,
            &idempotency_key_digest,
        )?;
        let request: ResponseRevisionRequest = decode_public_mutation(&headers, &body)?;
        let command = ResponseMutationCommand {
            response_id,
            expected_revision: request.expected_revision,
            actor_account_id: grant.payload.original_actor_id,
            idempotency_key_digest,
            request_digest,
            authorization_grant_jti: grant.payload.jti,
            authorization_correlation_id: grant.payload.correlation_id,
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
    required_capabilities_any_of: &[&str],
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
    let required = required_capabilities_any_of
        .iter()
        .map(|capability| SecurityCapabilityId::new(*capability))
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(|_| ProductApiError::Internal)?;
    if required.is_empty()
        || !envelope
            .payload
            .capability_scope_bindings
            .iter()
            .any(|binding| required.contains(&binding.capability))
    {
        return Err(ProductApiError::Forbidden);
    }
    Ok(envelope)
}

fn response_access(grant: &AuthorizationGrantV3) -> Result<ResponseAccess, ProductApiError> {
    let read_own =
        SecurityCapabilityId::new("submissions:read_own").map_err(|_| ProductApiError::Internal)?;
    let respond =
        SecurityCapabilityId::new("submissions:respond").map_err(|_| ProductApiError::Internal)?;
    let manage =
        SecurityCapabilityId::new("submissions:manage").map_err(|_| ProductApiError::Internal)?;
    let mut respond_node_ids = BTreeSet::new();
    let mut respond_all = false;
    let mut managed_node_ids = BTreeSet::new();
    let mut manage_all = false;
    for binding in &grant.capability_scope_bindings {
        let (all, nodes) = if binding.capability == respond {
            (&mut respond_all, &mut respond_node_ids)
        } else if binding.capability == manage {
            (&mut manage_all, &mut managed_node_ids)
        } else {
            continue;
        };
        if binding.organization_root_id == grant.installation_id {
            *all = true;
        } else {
            nodes.insert(binding.organization_root_id);
            nodes.extend(binding.authorized_organization_ids.iter().copied());
        }
    }
    Ok(ResponseAccess {
        installation_id: grant.installation_id,
        actor_account_id: grant.original_actor_id,
        delegated_account_ids: grant
            .delegation_basis
            .iter()
            .filter(|basis| basis.capability == read_own || basis.capability == respond)
            .map(|basis| basis.delegated_by_actor_id)
            .collect(),
        respond_node_ids,
        respond_all,
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

fn decode_public_mutation<T: DeserializeOwned>(
    headers: &HeaderMap,
    body: &[u8],
) -> Result<T, ProductApiError> {
    if headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        != Some("application/json")
        || body.is_empty()
        || body.len() > crate::RESPONSE_PUBLIC_MUTATION_BODY_LIMIT_BYTES
    {
        return Err(ProductApiError::BadRequest);
    }
    serde_json::from_slice(body).map_err(|_| ProductApiError::BadRequest)
}

fn public_mutation_request_digest(
    grant: &AuthorizationGrantV3,
    method: &str,
    path: &str,
    raw_body: &[u8],
    idempotency_key_digest: &str,
) -> Result<String, ProductApiError> {
    let module_instance_id = match &grant.audience {
        AuthorizationAudienceV1::ModuleInstance {
            module_instance_id,
            module_definition_id,
        } if module_definition_id.as_str() == MODULE_DEFINITION_ID => *module_instance_id,
        _ => return Err(ProductApiError::Forbidden),
    };
    let raw_body_digest = format!("sha256:{:x}", Sha256::digest(raw_body));
    let mut capability_scope_bindings = grant
        .capability_scope_bindings
        .iter()
        .map(|binding| {
            let mut authorized_organization_ids = binding.authorized_organization_ids.clone();
            authorized_organization_ids.sort_unstable();
            serde_json::json!({
                "capability": &binding.capability,
                "organization_root_id": binding.organization_root_id,
                "authorized_organization_ids": authorized_organization_ids,
            })
        })
        .collect::<Vec<_>>();
    capability_scope_bindings.sort_by_key(serde_json::Value::to_string);
    let mut delegation_basis = grant
        .delegation_basis
        .iter()
        .map(|basis| {
            serde_json::json!({
                "delegation_id": basis.delegation_id,
                "delegated_by_actor_id": basis.delegated_by_actor_id,
                "capability": &basis.capability,
                "organization_root_id": basis.organization_root_id,
            })
        })
        .collect::<Vec<_>>();
    delegation_basis.sort_by_key(serde_json::Value::to_string);
    canonical_digest(&serde_json::json!({
        "schema_version": 2,
        "installation_id": grant.installation_id,
        "module_instance_id": module_instance_id,
        "actor_account_id": grant.original_actor_id,
        "presenting_service": &grant.presenting_service,
        "audience": &grant.audience,
        "grant_action": &grant.action,
        "grant_operation": grant.operation,
        "dependency_binding": &grant.dependency_binding,
        "functional_contract": &grant.functional_contract,
        "capability_scope_bindings": capability_scope_bindings,
        "resource_assertion": &grant.resource_assertion,
        "delegation_basis": delegation_basis,
        "method": method,
        "path": path,
        "raw_body_digest": raw_body_digest,
        "idempotency_key_digest": idempotency_key_digest,
    }))
    .map_err(|_| ProductApiError::Internal)
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
            | ResponseOwnerError::BootstrapFaultInjected
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
    use chrono::Duration;
    use tessara_module_contract::{
        AUTHORIZATION_GRANT_SCHEMA_VERSION_V3, CapabilityScopeBindingV1, DelegationBasisV1,
    };

    use super::*;

    fn mutation_grant() -> AuthorizationGrantV3 {
        let now = chrono::Utc::now();
        AuthorizationGrantV3 {
            schema_version: AUTHORIZATION_GRANT_SCHEMA_VERSION_V3,
            installation_id: Uuid::from_u128(1),
            original_actor_id: Uuid::from_u128(2),
            correlation_id: Uuid::from_u128(3),
            presenting_service: ModuleServicePrincipalV1::CoreGateway,
            audience: AuthorizationAudienceV1::ModuleInstance {
                module_instance_id: Uuid::from_u128(4),
                module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
            },
            dependency_binding: DependencyBindingKey::new(CORE_RESPONSE_BINDING).unwrap(),
            functional_contract: FunctionalContractId::new(RESPONSE_LIFECYCLE_CONTRACT_ID).unwrap(),
            action: "responses.save".into(),
            operation: AuthorizationGrantOperationV1::Mutation,
            capability_scope_bindings: vec![CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("submissions:respond").unwrap(),
                organization_root_id: Uuid::from_u128(1),
                authorized_organization_ids: Vec::new(),
            }],
            resource_assertion: None,
            delegation_basis: Vec::new(),
            authorization_revision: 7,
            organization_revision: 8,
            jti: Uuid::from_u128(5),
            issued_at: now,
            expires_at: now + Duration::seconds(30),
        }
    }

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

    #[test]
    fn read_authority_preserves_scoped_and_global_manage_bindings() {
        assert_eq!(
            RESPONSE_READ_CAPABILITIES,
            &["submissions:read_own", "submissions:manage"]
        );
        let manage = SecurityCapabilityId::new("submissions:manage").unwrap();
        let mut grant = mutation_grant();
        grant.capability_scope_bindings = vec![CapabilityScopeBindingV1 {
            capability: manage.clone(),
            organization_root_id: Uuid::from_u128(9),
            authorized_organization_ids: vec![Uuid::from_u128(10)],
        }];
        let scoped = response_access(&grant).unwrap();
        assert!(!scoped.manage_all);
        assert_eq!(
            scoped.managed_node_ids,
            BTreeSet::from([Uuid::from_u128(9), Uuid::from_u128(10)])
        );

        grant.capability_scope_bindings = vec![CapabilityScopeBindingV1 {
            capability: manage,
            organization_root_id: grant.installation_id,
            authorized_organization_ids: Vec::new(),
        }];
        let global = response_access(&grant).unwrap();
        assert!(global.manage_all);
        assert!(global.managed_node_ids.is_empty());
    }

    #[test]
    fn response_start_accepts_only_respond_or_manage_authority() {
        assert_eq!(
            RESPONSE_START_CAPABILITIES,
            &["submissions:respond", "submissions:manage"]
        );
    }

    #[test]
    fn response_form_schema_does_not_reimpose_workflow_node_scope() {
        let form_id = Uuid::from_u128(20);
        let form_version_id = Uuid::from_u128(21);
        let schema = FormVersionSchemaResponse {
            schema_version: FORM_VERSION_SCHEMA_VERSION,
            form_id,
            form_version_id,
            form_name: "Assigned response form".into(),
            form_slug: "assigned-response-form".into(),
            version_label: Some("1.0.0".into()),
            version_major: Some(1),
            source_scope_node_ids: vec![Uuid::from_u128(22)],
            source_scope_revision: String::new(),
            source_scope_digest: String::new(),
            content_revision: String::new(),
            content_digest: String::new(),
            sections: Vec::new(),
            fields: Vec::new(),
        }
        .with_recomputed_digests()
        .unwrap();

        assert!(form_schema_matches_workflow(
            &schema,
            form_id,
            form_version_id
        ));
        assert!(!form_schema_matches_workflow(
            &schema,
            Uuid::from_u128(23),
            form_version_id
        ));
        assert!(!form_schema_matches_workflow(
            &schema,
            form_id,
            Uuid::from_u128(24)
        ));
    }

    #[test]
    fn mutation_identity_binds_exact_wire_route_and_grant_identity() {
        let grant = mutation_grant();
        let compact = br#"{"expected_revision":1}"#;
        let spaced = br#"{ "expected_revision": 1 }"#;
        let key_digest = canonical_digest("same-idempotency-key").unwrap();
        let identity = public_mutation_request_digest(
            &grant,
            "PUT",
            "/api/responses/00000000-0000-0000-0000-000000000009/values",
            compact,
            &key_digest,
        )
        .unwrap();
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(compact).unwrap(),
            serde_json::from_slice::<serde_json::Value>(spaced).unwrap()
        );
        assert_ne!(
            identity,
            public_mutation_request_digest(
                &grant,
                "PUT",
                "/api/responses/00000000-0000-0000-0000-000000000009/values",
                spaced,
                &key_digest,
            )
            .unwrap(),
            "semantically equal but byte-distinct JSON must conflict"
        );
        for (method, path) in [
            (
                "POST",
                "/api/responses/00000000-0000-0000-0000-000000000009/values",
            ),
            (
                "PUT",
                "/api/responses/00000000-0000-0000-0000-000000000010/values",
            ),
        ] {
            assert_ne!(
                identity,
                public_mutation_request_digest(&grant, method, path, compact, &key_digest).unwrap()
            );
        }

        let mut changed_audience = grant.clone();
        changed_audience.audience = AuthorizationAudienceV1::ModuleInstance {
            module_instance_id: Uuid::from_u128(7),
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
        };
        assert_ne!(
            identity,
            public_mutation_request_digest(
                &changed_audience,
                "PUT",
                "/api/responses/00000000-0000-0000-0000-000000000009/values",
                compact,
                &key_digest,
            )
            .unwrap()
        );
        let mut delegated = grant.clone();
        delegated.delegation_basis = vec![
            DelegationBasisV1 {
                delegation_id: Uuid::from_u128(20),
                delegated_by_actor_id: Uuid::from_u128(21),
                capability: SecurityCapabilityId::new("submissions:read_own").unwrap(),
                organization_root_id: Uuid::from_u128(1),
            },
            DelegationBasisV1 {
                delegation_id: Uuid::from_u128(22),
                delegated_by_actor_id: Uuid::from_u128(23),
                capability: SecurityCapabilityId::new("submissions:respond").unwrap(),
                organization_root_id: Uuid::from_u128(1),
            },
            DelegationBasisV1 {
                delegation_id: Uuid::from_u128(24),
                delegated_by_actor_id: Uuid::from_u128(25),
                capability: SecurityCapabilityId::new("submissions:manage").unwrap(),
                organization_root_id: Uuid::from_u128(1),
            },
        ];
        assert_eq!(
            response_access(&delegated).unwrap().delegated_account_ids,
            BTreeSet::from([Uuid::from_u128(21), Uuid::from_u128(23)])
        );
        let mut changed_service = grant;
        changed_service.presenting_service = ModuleServicePrincipalV1::ModuleInstance {
            module_instance_id: Uuid::from_u128(8),
            module_definition_id: ModuleDefinitionId::new("tessara.gateway.test").unwrap(),
        };
        assert_ne!(
            identity,
            public_mutation_request_digest(
                &changed_service,
                "PUT",
                "/api/responses/00000000-0000-0000-0000-000000000009/values",
                compact,
                &key_digest,
            )
            .unwrap()
        );
    }

    #[test]
    fn fresh_gateway_grants_share_stable_mutation_identity_and_authority_changes_conflict() {
        let first_gateway_grant = mutation_grant();
        let mut refreshed_gateway_grant = first_gateway_grant.clone();
        refreshed_gateway_grant.jti = Uuid::from_u128(50);
        refreshed_gateway_grant.correlation_id = Uuid::from_u128(51);
        refreshed_gateway_grant.issued_at += Duration::seconds(1);
        refreshed_gateway_grant.expires_at += Duration::seconds(1);
        assert_ne!(first_gateway_grant.jti, refreshed_gateway_grant.jti);
        assert_ne!(
            first_gateway_grant.correlation_id,
            refreshed_gateway_grant.correlation_id
        );

        let method = "PUT";
        let path = "/api/responses/00000000-0000-0000-0000-000000000009/values";
        let raw_body = br#"{"expected_revision":1,"values":{"name":"Grace"}}"#;
        let key_digest = canonical_digest("fresh-gateway-grant-key").unwrap();
        let stable_identity = public_mutation_request_digest(
            &first_gateway_grant,
            method,
            path,
            raw_body,
            &key_digest,
        )
        .unwrap();
        assert_eq!(
            stable_identity,
            public_mutation_request_digest(
                &refreshed_gateway_grant,
                method,
                path,
                raw_body,
                &key_digest,
            )
            .unwrap(),
            "fresh valid Gateway JTI/correlation/window values must replay the same stable request"
        );

        let mut changed_actor = refreshed_gateway_grant.clone();
        changed_actor.original_actor_id = Uuid::from_u128(52);
        let mut changed_action = refreshed_gateway_grant.clone();
        changed_action.action = "responses.delete".into();
        let mut changed_audience = refreshed_gateway_grant.clone();
        changed_audience.audience = AuthorizationAudienceV1::ModuleInstance {
            module_instance_id: Uuid::from_u128(53),
            module_definition_id: ModuleDefinitionId::new(MODULE_DEFINITION_ID).unwrap(),
        };
        let mut changed_bindings = refreshed_gateway_grant.clone();
        changed_bindings.capability_scope_bindings[0].organization_root_id = Uuid::from_u128(54);
        for changed in [
            &changed_actor,
            &changed_action,
            &changed_audience,
            &changed_bindings,
        ] {
            assert_ne!(
                stable_identity,
                public_mutation_request_digest(changed, method, path, raw_body, &key_digest)
                    .unwrap()
            );
        }
        assert_ne!(
            stable_identity,
            public_mutation_request_digest(
                &refreshed_gateway_grant,
                method,
                path,
                br#"{"expected_revision":1,"values":{"name":"Ada"}}"#,
                &key_digest,
            )
            .unwrap(),
            "exact raw body bytes remain part of stable replay identity"
        );
    }

    #[test]
    fn public_mutation_decode_is_strict_and_bounded() {
        let mut headers = HeaderMap::new();
        headers.insert(header::CONTENT_TYPE, "application/json".parse().unwrap());
        assert!(
            decode_public_mutation::<ResponseRevisionRequest>(
                &headers,
                br#"{"expected_revision":1}"#
            )
            .is_ok()
        );
        assert!(matches!(
            decode_public_mutation::<ResponseRevisionRequest>(
                &headers,
                br#"{"expected_revision":1,"unexpected":true}"#
            ),
            Err(ProductApiError::BadRequest)
        ));
        assert!(matches!(
            decode_public_mutation::<ResponseRevisionRequest>(
                &headers,
                &vec![b' '; crate::RESPONSE_PUBLIC_MUTATION_BODY_LIMIT_BYTES + 1]
            ),
            Err(ProductApiError::BadRequest)
        ));
        headers.insert(
            header::CONTENT_TYPE,
            "application/json; charset=utf-8".parse().unwrap(),
        );
        assert!(matches!(
            decode_public_mutation::<ResponseRevisionRequest>(
                &headers,
                br#"{"expected_revision":1}"#
            ),
            Err(ProductApiError::BadRequest)
        ));
    }
}
