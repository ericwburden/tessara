//! Stable, non-internal error envelopes for the Core module HTTP boundary.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub(crate) enum ModuleHttpError {
    #[error("{message}")]
    Rejected {
        status: StatusCode,
        code: &'static str,
        message: &'static str,
    },
    #[error("module control-plane persistence failed")]
    Database(#[from] sqlx::Error),
    #[error("module control-plane integrity validation failed: {0}")]
    Integrity(&'static str),
    #[error("module control-plane invariant failed: {0}")]
    Internal(&'static str),
}

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: &'static str,
    error: &'static str,
}

impl ModuleHttpError {
    pub(crate) fn rejected(status: StatusCode, code: &'static str, message: &'static str) -> Self {
        Self::Rejected {
            status,
            code,
            message,
        }
    }

    pub(crate) fn bad_request(code: &'static str, message: &'static str) -> Self {
        Self::rejected(StatusCode::BAD_REQUEST, code, message)
    }

    pub(crate) fn forbidden(code: &'static str, message: &'static str) -> Self {
        Self::rejected(StatusCode::FORBIDDEN, code, message)
    }

    pub(crate) fn not_found(code: &'static str, message: &'static str) -> Self {
        Self::rejected(StatusCode::NOT_FOUND, code, message)
    }

    pub(crate) fn conflict(code: &'static str, message: &'static str) -> Self {
        Self::rejected(StatusCode::CONFLICT, code, message)
    }

    pub(crate) fn code(&self) -> &'static str {
        match self {
            Self::Rejected { code, .. } => code,
            Self::Database(_) => "module_control_plane_database_error",
            Self::Integrity(code) => code,
            Self::Internal(_) => "module_control_plane_invariant_failed",
        }
    }
}

impl IntoResponse for ModuleHttpError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            Self::Rejected {
                status,
                code,
                message,
            } => (*status, *code, *message),
            Self::Database(error) => {
                tracing::error!(error = ?error, "module control-plane database request failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    self.code(),
                    "An internal server error occurred.",
                )
            }
            Self::Integrity(code) => {
                tracing::error!(code, "module control-plane integrity validation failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    *code,
                    "An internal server error occurred.",
                )
            }
            Self::Internal(reason) => {
                tracing::error!(reason, "module control-plane invariant failed");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    self.code(),
                    "An internal server error occurred.",
                )
            }
        };
        let body = Json(ErrorBody {
            code,
            message,
            error: message,
        });
        (status, body).into_response()
    }
}

pub(crate) type ModuleHttpResult<T> = Result<T, ModuleHttpError>;

#[cfg(test)]
mod tests {
    use axum::{body::to_bytes, http::StatusCode, response::IntoResponse};

    use super::ModuleHttpError;

    #[tokio::test]
    async fn stable_error_envelope_never_serializes_internal_detail() {
        let response = ModuleHttpError::Internal("secret database state").into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let body = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("body reads");
        let body = String::from_utf8(body.to_vec()).expect("body is UTF-8");
        assert!(body.contains("module_control_plane_invariant_failed"));
        assert!(!body.contains("secret database state"));
    }
}
