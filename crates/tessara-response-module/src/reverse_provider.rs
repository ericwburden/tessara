//! Authenticated reverse consumers for Response-owned state.

use axum::{
    Router,
    body::{Body, Bytes},
    extract::State,
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Response},
    routing::post,
};
use semver::Version;
use serde::Serialize;
use sqlx::Row;
use tessara_module_contract::{
    AuthorizationGrantV3, CapabilityScopeBindingV1, ContractCompatibilityState,
    FunctionalContractId, ModuleInstanceOwnerState, OwnerDataState, ProviderAvailabilityState,
    ProviderContractIdentity, ResourceAccessState, ResourceIdentityState, ResourceLifecycleState,
    ResourceObservationStrategy, ResourceObservationV1, ResourceOwner, ResourceOwnerState,
    ResourceResolutionV1, ResourceRevision,
};
use tessara_module_runtime::SecurityStateProvider;
use tessara_responses_contract::{
    RESPONSE_CONTRACT_VERSION, RESPONSE_FORM_VERSION_USAGE_ACTION,
    RESPONSE_FORM_VERSION_USAGE_BINDING_KEY, RESPONSE_FORM_VERSION_USAGE_CONTRACT_ID,
    RESPONSE_FORM_VERSION_USAGE_MEDIA_TYPE, RESPONSE_FORM_VERSION_USAGE_PATH,
    RESPONSE_OPERATIONAL_STATUS_CONTRACT_ID, RESPONSE_OPERATIONS_STATUS_ACTION,
    RESPONSE_OPERATIONS_STATUS_BINDING_KEY, RESPONSE_OPERATIONS_STATUS_MEDIA_TYPE,
    RESPONSE_OPERATIONS_STATUS_PATH, RESPONSE_RESOLVE_ACTION, RESPONSE_RESOLVE_PATH,
    RESPONSE_RESOURCE_CONTRACT_ID, RESPONSE_RESOURCE_OBSERVATION_BINDING_KEY,
    RESPONSE_RESOURCE_OBSERVATION_CONTRACT_ID, RESPONSE_RESOURCE_OBSERVATION_MEDIA_TYPE,
    RESPONSE_REVERSE_SCHEMA_VERSION, RESPONSE_SUMMARY_ACTION, RESPONSE_SUMMARY_BINDING_KEY,
    RESPONSE_SUMMARY_CONTRACT_ID, RESPONSE_SUMMARY_MEDIA_TYPE, RESPONSE_SUMMARY_PATH,
    ResponseFormVersionUsageItem, ResponseFormVersionUsageRequest,
    ResponseFormVersionUsageResponse, ResponseLifecycleState, ResponseOperationsStatus,
    ResponseOperationsStatusRequest, ResponseOperationsStatusResponse, ResponseProviderResultState,
    ResponseReference, ResponseResourceObservationRequest, ResponseResourceObservationResponse,
    ResponseSummaryRequest, ResponseSummaryResponse,
};
use uuid::Uuid;

use crate::{ResponseRuntime, private_provider_auth::PrivateProviderContract};

pub(crate) fn routes() -> Router<std::sync::Arc<ResponseRuntime>> {
    Router::new()
        .route(RESPONSE_FORM_VERSION_USAGE_PATH, post(form_version_usage))
        .route(RESPONSE_SUMMARY_PATH, post(summary))
        .route(RESPONSE_OPERATIONS_STATUS_PATH, post(operations_status))
        .route(RESPONSE_RESOLVE_PATH, post(resolve))
}

async fn form_version_usage(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    reverse_response(async {
        require_media_type(&headers, RESPONSE_FORM_VERSION_USAGE_MEDIA_TYPE)?;
        let grant = authorize(
            &runtime,
            &headers,
            &body,
            PrivateProviderContract {
                path: RESPONSE_FORM_VERSION_USAGE_PATH,
                binding: RESPONSE_FORM_VERSION_USAGE_BINDING_KEY,
                contract: RESPONSE_FORM_VERSION_USAGE_CONTRACT_ID,
                action: RESPONSE_FORM_VERSION_USAGE_ACTION,
                capability: "submissions:manage",
            },
        )
        .await?;
        let request: ResponseFormVersionUsageRequest = parse(&body)?;
        if request.schema_version != RESPONSE_REVERSE_SCHEMA_VERSION
            || request.form_version_id.is_nil()
        {
            return Err(());
        }
        let security = runtime.current_security_state().await.map_err(|_| ())?;
        let rows = sqlx::query(
            "SELECT id,status::text AS status,node_id,started_by_account_id,assignee_account_id FROM responses WHERE form_version_id=$1 ORDER BY id",
        )
        .bind(request.form_version_id)
        .fetch_all(&runtime.pool)
        .await
        .map_err(|_| ())?;
        let mut items = Vec::new();
        for row in rows {
            if !authorized_row(
                &grant.payload,
                &row,
                security.installation_id,
                "submissions:manage",
                RowAuthorizationMode::RequiredScopeOnly,
            )? {
                continue;
            }
            items.push(ResponseFormVersionUsageItem {
                response: ResponseReference::from_parts(
                    security.installation_id,
                    security.module_instance_id,
                    row.try_get("id").map_err(|_| ())?,
                )
                .map_err(|_| ())?,
                lifecycle_state: lifecycle(&row.try_get::<String, _>("status").map_err(|_| ())?)?,
            });
        }
        let response = ResponseFormVersionUsageResponse {
            schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
            state: if items.is_empty() {
                ResponseProviderResultState::Empty
            } else {
                ResponseProviderResultState::Available
            },
            items,
        };
        response.validate().map_err(|_| ())?;
        contract_response(&response, RESPONSE_FORM_VERSION_USAGE_MEDIA_TYPE)
    })
    .await
}

async fn summary(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    reverse_response(async {
        require_media_type(&headers, RESPONSE_SUMMARY_MEDIA_TYPE)?;
        let grant = authorize(
            &runtime,
            &headers,
            &body,
            PrivateProviderContract {
                path: RESPONSE_SUMMARY_PATH,
                binding: RESPONSE_SUMMARY_BINDING_KEY,
                contract: RESPONSE_SUMMARY_CONTRACT_ID,
                action: RESPONSE_SUMMARY_ACTION,
                capability: "submissions:read_own",
            },
        )
        .await?;
        let request: ResponseSummaryRequest = parse(&body)?;
        if request.schema_version != RESPONSE_REVERSE_SCHEMA_VERSION
            || request.requested_scope_node_ids.iter().any(Uuid::is_nil)
        {
            return Err(());
        }
        let security = runtime.current_security_state().await.map_err(|_| ())?;
        if !requested_scope_authorized(
            &grant.payload,
            &request.requested_scope_node_ids,
            security.installation_id,
            "submissions:read_own",
        ) {
            return nondisclosing_summary();
        }
        let rows = sqlx::query(
            "SELECT status::text AS status,node_id,started_by_account_id,assignee_account_id FROM responses WHERE status IN ('draft','submitted') ORDER BY id",
        )
        .fetch_all(&runtime.pool)
        .await
        .map_err(|_| ())?;
        let mut draft_count = 0_u64;
        let mut submitted_count = 0_u64;
        for row in rows {
            let node_id: Uuid = row.try_get("node_id").map_err(|_| ())?;
            if !request.requested_scope_node_ids.is_empty()
                && !request.requested_scope_node_ids.contains(&node_id)
            {
                continue;
            }
            if !authorized_row(
                &grant.payload,
                &row,
                security.installation_id,
                "submissions:read_own",
                RowAuthorizationMode::OwnershipOrRequiredScope,
            )? {
                continue;
            }
            match row.try_get::<String, _>("status").map_err(|_| ())?.as_str() {
                "draft" => draft_count += 1,
                "submitted" => submitted_count += 1,
                _ => return Err(()),
            }
        }
        let response = ResponseSummaryResponse {
            schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
            state: if draft_count == 0 && submitted_count == 0 {
                ResponseProviderResultState::Empty
            } else {
                ResponseProviderResultState::Available
            },
            draft_count: Some(draft_count),
            submitted_count: Some(submitted_count),
        };
        response.validate().map_err(|_| ())?;
        contract_response(&response, RESPONSE_SUMMARY_MEDIA_TYPE)
    })
    .await
}

async fn operations_status(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    reverse_response(async {
        require_media_type(&headers, RESPONSE_OPERATIONS_STATUS_MEDIA_TYPE)?;
        authorize(
            &runtime,
            &headers,
            &body,
            PrivateProviderContract {
                path: RESPONSE_OPERATIONS_STATUS_PATH,
                binding: RESPONSE_OPERATIONS_STATUS_BINDING_KEY,
                contract: RESPONSE_OPERATIONAL_STATUS_CONTRACT_ID,
                action: RESPONSE_OPERATIONS_STATUS_ACTION,
                capability: "submissions:manage",
            },
        )
        .await?;
        let request: ResponseOperationsStatusRequest = parse(&body)?;
        if request.schema_version != RESPONSE_REVERSE_SCHEMA_VERSION {
            return Err(());
        }
        let projection =
            crate::operational::ResponseOperationalProjection::load_cached(&runtime).await;
        let response = ResponseOperationsStatusResponse {
            schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
            state: ResponseProviderResultState::Available,
            status: Some(ResponseOperationsStatus {
                ready: projection.ready(),
                forms_binding_compatible: projection.forms_binding_compatible,
                workflow_binding_compatible: projection.workflow_binding_compatible,
                pending_workflow_event_count: projection.pending_workflow_event_count,
                export_head_sequence: projection.export_head_sequence,
                sanitized_findings: projection.sanitized_findings,
            }),
        };
        response.validate().map_err(|_| ())?;
        contract_response(&response, RESPONSE_OPERATIONS_STATUS_MEDIA_TYPE)
    })
    .await
}

async fn resolve(
    State(runtime): State<std::sync::Arc<ResponseRuntime>>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    reverse_response(async {
        require_media_type(&headers, RESPONSE_RESOURCE_OBSERVATION_MEDIA_TYPE)?;
        let grant = authorize(
            &runtime,
            &headers,
            &body,
            PrivateProviderContract {
                path: RESPONSE_RESOLVE_PATH,
                binding: RESPONSE_RESOURCE_OBSERVATION_BINDING_KEY,
                contract: RESPONSE_RESOURCE_OBSERVATION_CONTRACT_ID,
                action: RESPONSE_RESOLVE_ACTION,
                capability: "submissions:read_own",
            },
        )
        .await?;
        let request: ResponseResourceObservationRequest = parse(&body)?;
        if request.schema_version != RESPONSE_REVERSE_SCHEMA_VERSION {
            return Err(());
        }
        let security = runtime.current_security_state().await.map_err(|_| ())?;
        let owner_matches = request.reference.installation_id() == security.installation_id
            && matches!(
                request.reference.owner(),
                ResourceOwner::ModuleInstance { installation_id, module_instance_id }
                    if *installation_id == security.installation_id
                        && *module_instance_id == security.module_instance_id
            );
        let Ok(reference) = ResponseReference::new(request.reference.clone()) else {
            return restricted_observation(ResourceAccessState::NotEvaluated);
        };
        if !owner_matches {
            return restricted_observation(ResourceAccessState::NotEvaluated);
        }
        let row = sqlx::query(
            "SELECT status::text AS status,revision,node_id,started_by_account_id,assignee_account_id FROM responses WHERE id=$1",
        )
        .bind(reference.response_id())
        .fetch_optional(&runtime.pool)
        .await
        .map_err(|_| ())?;
        let Some(row) = row else {
            return restricted_observation(ResourceAccessState::Unauthorized);
        };
        if !authorized_row(
            &grant.payload,
            &row,
            security.installation_id,
            "submissions:read_own",
            RowAuthorizationMode::OwnershipOrRequiredScope,
        )? {
            return restricted_observation(ResourceAccessState::Unauthorized);
        }
        let lifecycle = row.try_get::<String, _>("status").map_err(|_| ())?;
        let revision = u64::try_from(row.try_get::<i64, _>("revision").map_err(|_| ())?)
            .map_err(|_| ())?;
        let observation = ResourceObservationV1::new(
            request.reference.clone(),
            ProviderContractIdentity::new(
                FunctionalContractId::new(RESPONSE_RESOURCE_CONTRACT_ID).map_err(|_| ())?,
                Version::parse(RESPONSE_CONTRACT_VERSION).map_err(|_| ())?,
            ),
            ResourceObservationStrategy::LiveResolutionWithRevision,
            ResourceRevision::new(revision).map_err(|_| ())?,
        );
        let response = ResponseResourceObservationResponse {
            schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
            resolution: ResourceResolutionV1::authorized(
                ResourceOwnerState::ModuleInstance {
                    instance_state: ModuleInstanceOwnerState::Live,
                    data_state: OwnerDataState::Retained,
                },
                ResourceIdentityState::Resolved,
                ResourceLifecycleState::ProviderDefined { state: lifecycle },
                ContractCompatibilityState::Compatible,
                ProviderAvailabilityState::Available,
            )
            .map_err(|_| ())?,
            observation: Some(observation),
        };
        response
            .validate_for(&request.reference)
            .map_err(|_| ())?;
        contract_response(&response, RESPONSE_RESOURCE_OBSERVATION_MEDIA_TYPE)
    })
    .await
}

async fn authorize(
    runtime: &ResponseRuntime,
    headers: &HeaderMap,
    body: &[u8],
    boundary: PrivateProviderContract,
) -> Result<tessara_module_contract::SignedEnvelopeV1<AuthorizationGrantV3>, ()> {
    crate::private_provider_auth::authorize(runtime, headers, body, boundary).await
}

fn authorized_row(
    grant: &AuthorizationGrantV3,
    row: &sqlx::postgres::PgRow,
    installation_id: Uuid,
    required_capability: &str,
    mode: RowAuthorizationMode,
) -> Result<bool, ()> {
    let actor = grant.original_actor_id;
    let started_by: Uuid = row.try_get("started_by_account_id").map_err(|_| ())?;
    let assignee: Uuid = row.try_get("assignee_account_id").map_err(|_| ())?;
    if mode == RowAuthorizationMode::OwnershipOrRequiredScope
        && (actor == started_by
            || actor == assignee
            || grant
                .delegation_basis
                .iter()
                .any(|basis| basis.delegated_by_actor_id == assignee))
    {
        return Ok(true);
    }
    let node_id: Uuid = row.try_get("node_id").map_err(|_| ())?;
    Ok(scope_authorized(
        grant,
        required_capability,
        node_id,
        installation_id,
    ))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RowAuthorizationMode {
    OwnershipOrRequiredScope,
    RequiredScopeOnly,
}

fn requested_scope_authorized(
    grant: &AuthorizationGrantV3,
    requested: &[Uuid],
    installation_id: Uuid,
    required_capability: &str,
) -> bool {
    requested.is_empty()
        || requested
            .iter()
            .all(|node_id| scope_authorized(grant, required_capability, *node_id, installation_id))
}

fn scope_authorized(
    grant: &AuthorizationGrantV3,
    required_capability: &str,
    node_id: Uuid,
    installation_id: Uuid,
) -> bool {
    capability_scope_authorized(
        &grant.capability_scope_bindings,
        required_capability,
        node_id,
        installation_id,
    )
}

fn capability_scope_authorized(
    bindings: &[CapabilityScopeBindingV1],
    required_capability: &str,
    node_id: Uuid,
    installation_id: Uuid,
) -> bool {
    bindings.iter().any(|binding| {
        binding.capability.as_str() == required_capability
            && (binding.organization_root_id == installation_id
                || binding.organization_root_id == node_id
                || binding.authorized_organization_ids.contains(&node_id))
    })
}

fn lifecycle(value: &str) -> Result<ResponseLifecycleState, ()> {
    match value {
        "draft" => Ok(ResponseLifecycleState::Draft),
        "submitted" => Ok(ResponseLifecycleState::Submitted),
        "deleted" => Ok(ResponseLifecycleState::Deleted),
        _ => Err(()),
    }
}

fn nondisclosing_summary() -> Result<Response, ()> {
    contract_response(
        &ResponseSummaryResponse {
            schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
            state: ResponseProviderResultState::Undisclosed,
            draft_count: None,
            submitted_count: None,
        },
        RESPONSE_SUMMARY_MEDIA_TYPE,
    )
}

fn restricted_observation(access: ResourceAccessState) -> Result<Response, ()> {
    contract_response(
        &ResponseResourceObservationResponse {
            schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
            resolution: ResourceResolutionV1::restricted(access).map_err(|_| ())?,
            observation: None,
        },
        RESPONSE_RESOURCE_OBSERVATION_MEDIA_TYPE,
    )
}

fn parse<T: serde::de::DeserializeOwned>(body: &[u8]) -> Result<T, ()> {
    serde_json::from_slice(body).map_err(|_| ())
}

fn require_media_type(headers: &HeaderMap, expected: &str) -> Result<(), ()> {
    (headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        == Some(expected))
    .then_some(())
    .ok_or(())
}

fn contract_response<T: Serialize>(value: &T, media_type: &str) -> Result<Response, ()> {
    Response::builder()
        .header(header::CONTENT_TYPE, media_type)
        .body(Body::from(serde_json::to_vec(value).map_err(|_| ())?))
        .map_err(|_| ())
}

async fn reverse_response(future: impl Future<Output = Result<Response, ()>>) -> Response {
    future
        .await
        .unwrap_or_else(|()| StatusCode::NOT_FOUND.into_response())
}

#[cfg(test)]
mod tests {
    use tessara_module_contract::{CapabilityScopeBindingV1, SecurityCapabilityId};
    use uuid::Uuid;

    use super::capability_scope_authorized;
    use crate::operational::EXPORT_HEAD_SEQUENCE_QUERY;

    #[test]
    fn operations_status_reads_the_owned_export_sequence_column() {
        assert_eq!(
            EXPORT_HEAD_SEQUENCE_QUERY,
            "SELECT COALESCE(MAX(sequence),0) FROM response_export_changes"
        );
        assert!(!EXPORT_HEAD_SEQUENCE_QUERY.contains("change_sequence"));
    }

    #[test]
    fn reverse_scope_uses_only_the_required_capability_binding() {
        let installation_id = Uuid::from_u128(1);
        let allowed_node = Uuid::from_u128(2);
        let unrelated_node = Uuid::from_u128(3);
        let bindings = vec![
            CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("submissions:manage").unwrap(),
                organization_root_id: allowed_node,
                authorized_organization_ids: Vec::new(),
            },
            CapabilityScopeBindingV1 {
                capability: SecurityCapabilityId::new("datasets:manage").unwrap(),
                organization_root_id: unrelated_node,
                authorized_organization_ids: Vec::new(),
            },
        ];
        assert!(capability_scope_authorized(
            &bindings,
            "submissions:manage",
            allowed_node,
            installation_id,
        ));
        assert!(!capability_scope_authorized(
            &bindings,
            "submissions:manage",
            unrelated_node,
            installation_id,
        ));
    }
}
