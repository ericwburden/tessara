mod handlers;

pub mod dto;
mod repo;
mod service;

use axum::{
    Router,
    routing::{get, post, put},
};

use crate::db::AppState;

pub use handlers::{
    delete_draft_submission, get_submission, list_response_start_options, list_submissions,
    save_submission_values, submit_submission,
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/responses/options", get(list_response_start_options))
        .route("/api/submissions", get(list_submissions))
        .route(
            "/api/submissions/{submission_id}",
            get(get_submission).delete(delete_draft_submission),
        )
        .route(
            "/api/submissions/{submission_id}/values",
            put(save_submission_values),
        )
        .route(
            "/api/submissions/{submission_id}/submit",
            post(submit_submission),
        )
}
