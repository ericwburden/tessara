use axum::{Json, extract::State, http::HeaderMap};
use serde::Serialize;
use sqlx::Row;
use tessara_datasets_contract::{
    DATASET_CORE_BINDING_KEY, DATASET_MODULE_DEFINITION_ID, DATASET_OPERATIONAL_STATUS_CONTRACT_ID,
    DATASET_REVERSE_CONTRACT_VERSION, DATASET_SUMMARY_ACTION, DATASET_SUMMARY_PATH,
    DatasetProviderResultState, DatasetSummaryRequest, DatasetSummaryResponse,
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
    draft_submissions: i64,
    submitted_submissions: i64,
    dataset_state: DatasetProviderResultState,
    datasets: Option<i64>,
    dataset_revisions: Option<i64>,
}

struct DatasetSummaryProjection {
    state: DatasetProviderResultState,
    datasets: Option<i64>,
    dataset_revisions: Option<i64>,
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
        let row = sqlx::query(
            r#"
            SELECT
                (SELECT COUNT(*) FROM form_versions WHERE status = 'published') AS published_form_versions,
                (SELECT COUNT(*) FROM submissions WHERE status = 'draft') AS draft_submissions,
                (SELECT COUNT(*) FROM submissions WHERE status = 'submitted') AS submitted_submissions
            "#,
        )
        .fetch_one(&state.pool)
        .await?;

        let datasets =
            load_dataset_summary(&state, &request, request_correlation_id_or_new(&headers)).await?;
        return summary_from_row(row, datasets);
    }

    if let auth::CapabilityBoundary::Scoped(scope_ids) =
        auth::capability_boundary(&state.pool, &request.account, "forms:read").await?
    {
        let row = sqlx::query(
            r#"
            SELECT
                (
                    SELECT COUNT(DISTINCT form_versions.id)
                    FROM form_versions
                    JOIN forms ON forms.id = form_versions.form_id
                    JOIN form_scope_nodes ON form_scope_nodes.form_id = forms.id
                    WHERE form_versions.status = 'published'::form_version_status
                      AND form_scope_nodes.node_id = ANY($1)
                ) AS published_form_versions,
                (
                    SELECT COUNT(*)
                    FROM submissions
                    WHERE submissions.status = 'draft'::submission_status
                      AND submissions.node_id = ANY($1)
                ) AS draft_submissions,
                (
                    SELECT COUNT(*)
                    FROM submissions
                    WHERE submissions.status = 'submitted'::submission_status
                      AND submissions.node_id = ANY($1)
                ) AS submitted_submissions
            "#,
        )
        .bind(scope_ids)
        .fetch_one(&state.pool)
        .await?;

        return summary_from_row(row, undisclosed_dataset_summary());
    }

    let accessible_account_ids = {
        let mut ids = vec![request.account.account_id];
        ids.extend(
            request
                .account
                .delegations
                .iter()
                .map(|delegate| delegate.account_id),
        );
        ids
    };
    let row = sqlx::query(
        r#"
        SELECT
            0::bigint AS published_form_versions,
            (
                SELECT COUNT(*)
                FROM submissions
                JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
                WHERE submissions.status = 'draft'::submission_status
                  AND workflow_assignments.account_id = ANY($1)
            ) AS draft_submissions,
            (
                SELECT COUNT(*)
                FROM submissions
                JOIN workflow_assignments ON workflow_assignments.id = submissions.workflow_assignment_id
                WHERE submissions.status = 'submitted'::submission_status
                  AND workflow_assignments.account_id = ANY($1)
            ) AS submitted_submissions
        "#,
    )
    .bind(accessible_account_ids)
    .fetch_one(&state.pool)
    .await?;

    summary_from_row(row, undisclosed_dataset_summary())
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

fn summary_from_row(
    row: sqlx::postgres::PgRow,
    datasets: DatasetSummaryProjection,
) -> ApiResult<Json<ApplicationSummary>> {
    Ok(Json(ApplicationSummary {
        published_form_versions: row.try_get("published_form_versions")?,
        draft_submissions: row.try_get("draft_submissions")?,
        submitted_submissions: row.try_get("submitted_submissions")?,
        dataset_state: datasets.state,
        datasets: datasets.datasets,
        dataset_revisions: datasets.dataset_revisions,
    }))
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
