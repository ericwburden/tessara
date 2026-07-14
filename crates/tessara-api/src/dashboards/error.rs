//! Dashboard service errors and stable HTTP mappings.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tessara_dashboards::{CompositionError, DASHBOARD_GRID_CONSTRAINTS};
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug, thiserror::Error)]
pub enum DashboardServiceError {
    #[error(transparent)]
    Api(#[from] ApiError),
    #[error("invalid Dashboard placement geometry: {0}")]
    InvalidGeometry(String),
    #[error("Dashboard placements overlap: {0}")]
    Overlap(String),
    #[error("Dashboard placement capacity exceeded")]
    PlacementLimit,
    #[error("Component version is unavailable for Dashboard placement")]
    ComponentVersionUnavailable,
    #[error("Dashboard and Component visibility scopes are incompatible")]
    ScopeIncompatible,
    #[error("Dashboard composition changed before this save")]
    CompositionStale,
    #[error("Dashboard placement {0} was not found")]
    PlacementNotFound(Uuid),
}

impl From<sqlx::Error> for DashboardServiceError {
    fn from(error: sqlx::Error) -> Self {
        if let sqlx::Error::Database(database) = &error
            && database.constraint() == Some("dashboard_components_capacity_chk")
        {
            return Self::PlacementLimit;
        }
        Self::Api(ApiError::Database(error))
    }
}

impl From<CompositionError> for DashboardServiceError {
    fn from(error: CompositionError) -> Self {
        match error {
            CompositionError::Overlap => Self::Overlap(error.to_string()),
            CompositionError::PlacementLimitExceeded { .. } => Self::PlacementLimit,
            CompositionError::PlacementNotFound | CompositionError::DuplicatePlacementId => {
                Self::CompositionStale
            }
            other => Self::InvalidGeometry(other.to_string()),
        }
    }
}

#[derive(Serialize)]
struct DashboardErrorBody {
    code: &'static str,
    message: String,
    error: String,
}

impl IntoResponse for DashboardServiceError {
    fn into_response(self) -> Response {
        if let Self::Api(error) = self {
            return error.into_response();
        }

        let (status, code, message) = match self {
            Self::InvalidGeometry(message) => (
                StatusCode::BAD_REQUEST,
                "dashboard_layout_invalid_geometry",
                message,
            ),
            Self::Overlap(message) => {
                (StatusCode::BAD_REQUEST, "dashboard_layout_overlap", message)
            }
            Self::PlacementLimit => (
                StatusCode::BAD_REQUEST,
                "dashboard_placement_limit_exceeded",
                format!(
                    "A Dashboard can contain at most {} placements.",
                    DASHBOARD_GRID_CONSTRAINTS.max_placements()
                ),
            ),
            Self::ComponentVersionUnavailable => (
                StatusCode::BAD_REQUEST,
                "dashboard_component_version_unavailable",
                "The requested Component version is unavailable for this Dashboard.".to_string(),
            ),
            Self::ScopeIncompatible => (
                StatusCode::CONFLICT,
                "dashboard_scope_incompatible",
                "Dashboard visibility must contain every placed Component Dataset scope."
                    .to_string(),
            ),
            Self::CompositionStale => (
                StatusCode::CONFLICT,
                "dashboard_composition_stale",
                "The Dashboard composition changed. Reload it before saving again.".to_string(),
            ),
            Self::PlacementNotFound(placement_id) => (
                StatusCode::NOT_FOUND,
                "dashboard_placement_not_found",
                format!("Dashboard placement {placement_id} was not found."),
            ),
            Self::Api(_) => unreachable!("Api errors return before Dashboard error mapping"),
        };
        let body = Json(DashboardErrorBody {
            code,
            message: message.clone(),
            error: message,
        });
        (status, body).into_response()
    }
}

pub type DashboardResult<T> = Result<T, DashboardServiceError>;
