//! Error mapping for Tessara HTTP handlers.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tracing::{error, warn};

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
    /// Login credentials were rejected.
    #[error("authentication failed")]
    InvalidCredentials,
    /// The current session has expired.
    #[error("session expired")]
    SessionExpired,
    /// The current session has been revoked.
    #[error("session revoked")]
    SessionRevoked,
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
    code: &'static str,
    message: String,
    error: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            ApiError::BadRequest(message) => {
                (StatusCode::BAD_REQUEST, "bad_request", message.clone())
            }
            ApiError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "auth_unauthorized",
                "Authentication is required.".to_string(),
            ),
            ApiError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "auth_invalid_credentials",
                "Email or password is incorrect.".to_string(),
            ),
            ApiError::SessionExpired => (
                StatusCode::UNAUTHORIZED,
                "auth_session_expired",
                "Your session has expired. Sign in again.".to_string(),
            ),
            ApiError::SessionRevoked => (
                StatusCode::UNAUTHORIZED,
                "auth_session_revoked",
                "Your session is no longer active. Sign in again.".to_string(),
            ),
            ApiError::Forbidden(capability) => (
                StatusCode::FORBIDDEN,
                "forbidden",
                format!("The current account is missing required capability '{capability}'."),
            ),
            ApiError::NotFound(message) => (StatusCode::NOT_FOUND, "not_found", message.clone()),
            ApiError::Database(error) => {
                error!(error = ?error, "database request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_database_error",
                    "An internal server error occurred.".to_string(),
                )
            }
            ApiError::Internal(error) => {
                error!(error = ?error, "internal request handling failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "internal_error",
                    "An internal server error occurred.".to_string(),
                )
            }
        };

        if matches!(
            self,
            ApiError::Unauthorized
                | ApiError::InvalidCredentials
                | ApiError::SessionExpired
                | ApiError::SessionRevoked
                | ApiError::Forbidden(_)
        ) {
            warn!(status = %status, code, "request rejected");
        };

        let body = Json(ErrorBody {
            code,
            message: message.clone(),
            error: message,
        });

        (status, body).into_response()
    }
}

/// Result alias used by Tessara API handlers and helper functions.
pub type ApiResult<T> = Result<T, ApiError>;
