use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{auth::AuthenticatedRequest, db::AppState, error::ApiResult, hierarchy::IdResponse};

use super::dto::{
    AssignmentResponseStartOptions, ListSubmissionsQuery, SaveSubmissionValuesRequest,
    SubmissionDetail, SubmissionSummary,
};
use super::service;

/// Returns workflow-assignment-backed response start choices only.
///
/// Form-first shortcuts must create or reuse a generated single-form workflow
/// assignment before a response can be started.
pub async fn list_response_start_options(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<AssignmentResponseStartOptions>> {
    Ok(Json(
        service::list_response_start_options(&state.pool, &request.account, query).await?,
    ))
}

/// Lists submissions for the current local workflow shell.
pub async fn list_submissions(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<Vec<SubmissionSummary>>> {
    Ok(Json(
        service::list_submissions(&state.pool, &request.account, query).await?,
    ))
}

/// Returns a submission with saved values and audit history for inspection.
pub async fn get_submission(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(submission_id): Path<Uuid>,
) -> ApiResult<Json<SubmissionDetail>> {
    Ok(Json(
        service::get_submission(&state.pool, &request.account, submission_id).await?,
    ))
}

pub async fn save_submission_values(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(submission_id): Path<Uuid>,
    Json(payload): Json<SaveSubmissionValuesRequest>,
) -> ApiResult<Json<IdResponse>> {
    let id = service::save_submission_values(
        &state.pool,
        &request.account,
        submission_id,
        payload.values,
    )
    .await?;
    Ok(Json(IdResponse { id }))
}

pub async fn submit_submission(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(submission_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let id = service::submit_submission(&state.pool, &request.account, submission_id).await?;
    Ok(Json(IdResponse { id }))
}

/// Deletes an unsubmitted draft submission.
pub async fn delete_draft_submission(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(submission_id): Path<Uuid>,
) -> ApiResult<Json<IdResponse>> {
    let id = service::delete_draft_submission(&state.pool, &request.account, submission_id).await?;
    Ok(Json(IdResponse { id }))
}
