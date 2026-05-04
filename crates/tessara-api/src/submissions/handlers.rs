use axum::{
    Json,
    extract::{Path, Query, State},
};
use uuid::Uuid;

use crate::{auth::AuthenticatedRequest, db::AppState, error::ApiResult, hierarchy::IdResponse};

use super::dto::{
    CreateDraftRequest, ListSubmissionsQuery, ResponseStartOptions, SaveSubmissionValuesRequest,
    SubmissionDetail, SubmissionSummary,
};
use super::service;

pub async fn list_response_start_options(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Query(query): Query<ListSubmissionsQuery>,
) -> ApiResult<Json<ResponseStartOptions>> {
    Ok(Json(
        service::list_response_start_options(&state.pool, &request.account, query).await?,
    ))
}

pub async fn create_draft(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Json(payload): Json<CreateDraftRequest>,
) -> ApiResult<Json<IdResponse>> {
    let id = service::create_draft(&state.pool, &request.account, payload).await?;
    Ok(Json(IdResponse { id }))
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
