//! Error mapping for Tessara HTTP handlers.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Error type returned by API handlers and command helpers.
///
/// The variants intentionally preserve user-facing HTTP semantics while still
/// allowing database and internal errors to be propagated with `?`.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The request payload or requested workflow transition is invalid.
    #[error("bad request: {0}")]
    BadRequest(String),
    /// Authentication is missing or invalid.
    #[error("unauthorized")]
    Unauthorized,
    /// The authenticated account lacks the required capability.
    #[error("forbidden: {0}")]
    Forbidden(String),
    /// A requested entity could not be found.
    #[error("not found: {0}")]
    NotFound(String),
    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    /// An internal operation outside the database failed.
    #[error("internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self {
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized => StatusCode::UNAUTHORIZED,
            ApiError::Forbidden(_) => StatusCode::FORBIDDEN,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::Database(_) | ApiError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorBody {
            error: self.to_string(),
        });

        (status, body).into_response()
    }
}

/// Result alias used by Tessara API handlers and helper functions.
pub type ApiResult<T> = Result<T, ApiError>;
