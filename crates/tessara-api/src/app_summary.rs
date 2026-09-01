use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use tessara_datasets_contract::{
    DATASET_CORE_BINDING_KEY, DATASET_MODULE_DEFINITION_ID, DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
    DATASET_REVERSE_CONTRACT_VERSION, DATASET_SUMMARY_ACTION, DATASET_SUMMARY_PATH,
    DatasetProviderResultState, DatasetSummaryRequest, DatasetSummaryResponse,
};
use tessara_responses_contract::{
    RESPONSE_MODULE_DEFINITION_ID, RESPONSE_REVERSE_CONTRACT_VERSION,
    RESPONSE_REVERSE_SCHEMA_VERSION, RESPONSE_SUMMARY_ACTION, RESPONSE_SUMMARY_BINDING_KEY,
    RESPONSE_SUMMARY_CONTRACT_ID, RESPONSE_SUMMARY_MEDIA_TYPE, RESPONSE_SUMMARY_PATH,
    ResponseProviderResultState, ResponseSummaryRequest, ResponseSummaryResponse,
};

use crate::{
    auth::{self, AuthenticatedRequest},
    core_security::request_correlation_id_or_new,
    db::AppState,
    error::ApiResult,
    module_gateway::{
        CorePrivateProviderRequest, CorePrivateProviderResult, call_private_provider,
    },
};

/// High-level counters used by focused application screens.
#[derive(Serialize)]
pub struct ApplicationSummary {
    published_form_versions: i64,
    response_state: ResponseProviderResultState,
    draft_submissions: Option<i64>,
    submitted_submissions: Option<i64>,
    dataset_state: DatasetProviderResultState,
    datasets: Option<i64>,
    dataset_revisions: Option<i64>,
}

struct DatasetSummaryProjection {
    state: DatasetProviderResultState,
    datasets: Option<i64>,
    dataset_revisions: Option<i64>,
}

struct ResponseSummaryProjection {
    state: ResponseProviderResultState,
    draft: Option<i64>,
    submitted: Option<i64>,
}

/// Returns app-readiness counters for the current deployment.
pub async fn get_summary(
    State(state): State<AppState>,
    headers: HeaderMap,
    request: AuthenticatedRequest,
) -> ApiResult<Json<ApplicationSummary>> {
    if matches!(
        auth::capability_boundary(&state.pool, &request.account, "admin:all").await?,
        auth::CapabilityBoundary::Global
    ) {
        let published_form_versions =
            sqlx::query_scalar("SELECT COUNT(*) FROM form_versions WHERE status = 'published'")
                .fetch_one(&state.pool)
                .await?;
        let correlation_id = request_correlation_id_or_new(&headers);
        let responses =
            load_response_summary(&state, &request, correlation_id, Vec::new(), "admin:all")
                .await?;
        let datasets = load_dataset_summary(&state, &request, uuid::Uuid::new_v4()).await?;
        return Ok(summary(published_form_versions, responses, datasets));
    }

    if let auth::CapabilityBoundary::Scoped(scope_ids) =
        auth::capability_boundary(&state.pool, &request.account, "forms:read").await?
    {
        let published_form_versions = sqlx::query_scalar(
            r#"
            SELECT COUNT(DISTINCT form_versions.id)
            FROM form_versions
            JOIN forms ON forms.id = form_versions.form_id
            JOIN form_scope_nodes ON form_scope_nodes.form_id = forms.id
            WHERE form_versions.status = 'published'::form_version_status
              AND form_scope_nodes.node_id = ANY($1)
            "#,
        )
        .bind(&scope_ids)
        .fetch_one(&state.pool)
        .await?;
        let responses = load_response_summary(
            &state,
            &request,
            request_correlation_id_or_new(&headers),
            scope_ids,
            "forms:read",
        )
        .await?;
        return Ok(summary(
            published_form_versions,
            responses,
            undisclosed_dataset_summary(),
        ));
    }

    let responses = load_response_summary(
        &state,
        &request,
        request_correlation_id_or_new(&headers),
        Vec::new(),
        "submissions:read_own",
    )
    .await?;
    Ok(summary(0, responses, undisclosed_dataset_summary()))
}

async fn load_response_summary(
    state: &AppState,
    actor: &AuthenticatedRequest,
    correlation_id: uuid::Uuid,
    requested_scope_node_ids: Vec<uuid::Uuid>,
    actor_capability: &str,
) -> ApiResult<ResponseSummaryProjection> {
    let request = ResponseSummaryRequest {
        schema_version: RESPONSE_REVERSE_SCHEMA_VERSION,
        requested_scope_node_ids,
    };
    let response = call_private_provider::<_, ResponseSummaryResponse>(
        state,
        actor,
        CorePrivateProviderRequest {
            module_definition_id: RESPONSE_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding: RESPONSE_SUMMARY_BINDING_KEY,
            functional_contract: RESPONSE_SUMMARY_CONTRACT_ID,
            contract_version: RESPONSE_REVERSE_CONTRACT_VERSION,
            authorization_action: RESPONSE_SUMMARY_ACTION,
            path: RESPONSE_SUMMARY_PATH,
            media_type: RESPONSE_SUMMARY_MEDIA_TYPE,
            correlation_id,
            actor_capability,
            body: &request,
        },
    )
    .await?;
    Ok(match response {
        CorePrivateProviderResult::Response(response) => project_response_summary(response),
        CorePrivateProviderResult::Unavailable => {
            restricted_response_summary(ResponseProviderResultState::Unavailable)
        }
        CorePrivateProviderResult::Undisclosed => {
            restricted_response_summary(ResponseProviderResultState::Undisclosed)
        }
    })
}

fn project_response_summary(response: ResponseSummaryResponse) -> ResponseSummaryProjection {
    if response.validate().is_err() {
        return restricted_response_summary(ResponseProviderResultState::Unavailable);
    }
    match response.state {
        ResponseProviderResultState::Available | ResponseProviderResultState::Empty => {
            let Some(draft) = response
                .draft_count
                .and_then(|value| i64::try_from(value).ok())
            else {
                return restricted_response_summary(ResponseProviderResultState::Unavailable);
            };
            let Some(submitted) = response
                .submitted_count
                .and_then(|value| i64::try_from(value).ok())
            else {
                return restricted_response_summary(ResponseProviderResultState::Unavailable);
            };
            ResponseSummaryProjection {
                state: response.state,
                draft: Some(draft),
                submitted: Some(submitted),
            }
        }
        state => restricted_response_summary(state),
    }
}

fn restricted_response_summary(state: ResponseProviderResultState) -> ResponseSummaryProjection {
    ResponseSummaryProjection {
        state,
        draft: None,
        submitted: None,
    }
}

async fn load_dataset_summary(
    state: &AppState,
    actor: &AuthenticatedRequest,
    correlation_id: uuid::Uuid,
) -> ApiResult<DatasetSummaryProjection> {
    let request = DatasetSummaryRequest { schema_version: 1 };
    let response = call_private_provider::<_, DatasetSummaryResponse>(
        state,
        actor,
        CorePrivateProviderRequest {
            module_definition_id: DATASET_MODULE_DEFINITION_ID,
            expected_owner: None,
            dependency_binding: DATASET_CORE_BINDING_KEY,
            functional_contract: DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
            contract_version: DATASET_REVERSE_CONTRACT_VERSION,
            authorization_action: DATASET_SUMMARY_ACTION,
            path: DATASET_SUMMARY_PATH,
            media_type: "application/json",
            correlation_id,
            actor_capability: "admin:all",
            body: &request,
        },
    )
    .await?;
    Ok(match response {
        CorePrivateProviderResult::Response(response) => project_dataset_summary(response),
        CorePrivateProviderResult::Unavailable => unavailable_dataset_summary(),
        CorePrivateProviderResult::Undisclosed => undisclosed_dataset_summary(),
    })
}

fn project_dataset_summary(response: DatasetSummaryResponse) -> DatasetSummaryProjection {
    if response.validate().is_err() {
        return unavailable_dataset_summary();
    }
    match response.state {
        DatasetProviderResultState::Available | DatasetProviderResultState::Empty => {
            let Some(datasets) = response
                .dataset_count
                .and_then(|count| i64::try_from(count).ok())
            else {
                return unavailable_dataset_summary();
            };
            let Some(dataset_revisions) = response
                .published_revision_count
                .and_then(|count| i64::try_from(count).ok())
            else {
                return unavailable_dataset_summary();
            };
            DatasetSummaryProjection {
                state: response.state,
                datasets: Some(datasets),
                dataset_revisions: Some(dataset_revisions),
            }
        }
        DatasetProviderResultState::Unavailable => unavailable_dataset_summary(),
        DatasetProviderResultState::Undisclosed => undisclosed_dataset_summary(),
    }
}

fn unavailable_dataset_summary() -> DatasetSummaryProjection {
    DatasetSummaryProjection {
        state: DatasetProviderResultState::Unavailable,
        datasets: None,
        dataset_revisions: None,
    }
}

fn undisclosed_dataset_summary() -> DatasetSummaryProjection {
    DatasetSummaryProjection {
        state: DatasetProviderResultState::Undisclosed,
        datasets: None,
        dataset_revisions: None,
    }
}

fn summary(
    published_form_versions: i64,
    responses: ResponseSummaryProjection,
    datasets: DatasetSummaryProjection,
) -> Json<ApplicationSummary> {
    Json(ApplicationSummary {
        published_form_versions,
        response_state: responses.state,
        draft_submissions: responses.draft,
        submitted_submissions: responses.submitted,
        dataset_state: datasets.state,
        datasets: datasets.datasets,
        dataset_revisions: datasets.dataset_revisions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summary_outage_and_undisclosed_states_never_carry_false_zero_counts() {
        for projection in [unavailable_dataset_summary(), undisclosed_dataset_summary()] {
            assert_eq!(projection.datasets, None);
            assert_eq!(projection.dataset_revisions, None);
            assert!(matches!(
                projection.state,
                DatasetProviderResultState::Unavailable | DatasetProviderResultState::Undisclosed
            ));
        }
    }

    #[test]
    fn malformed_owner_summary_is_unavailable_not_zero() {
        let projection = project_dataset_summary(DatasetSummaryResponse {
            schema_version: 1,
            state: DatasetProviderResultState::Unavailable,
            dataset_count: Some(0),
            published_revision_count: Some(0),
        });
        assert_eq!(projection.state, DatasetProviderResultState::Unavailable);
        assert_eq!(projection.datasets, None);
        assert_eq!(projection.dataset_revisions, None);
    }

    #[test]
    fn explicit_empty_owner_summary_preserves_real_zero() {
        let projection = project_dataset_summary(DatasetSummaryResponse {
            schema_version: 1,
            state: DatasetProviderResultState::Empty,
            dataset_count: Some(0),
            published_revision_count: Some(0),
        });
        assert_eq!(projection.state, DatasetProviderResultState::Empty);
        assert_eq!(projection.datasets, Some(0));
        assert_eq!(projection.dataset_revisions, Some(0));
    }
}
