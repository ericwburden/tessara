mod generated;
mod handlers;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use crate::db::AppState;

pub mod dto;

pub use generated::{
    ensure_workflow_assignment_for_form_version, ensure_workflow_for_published_form_version_tx,
};
pub use handlers::{
    bulk_create_workflow_assignments, create_workflow, create_workflow_assignment,
    create_workflow_version, delete_workflow_version, get_workflow,
    list_assignment_candidate_assignees, list_assignment_candidates, list_workflow_assignments,
    list_workflows, publish_workflow_version, replace_workflow_version_steps, update_workflow,
    update_workflow_assignment,
};

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/workflows", get(list_workflows).post(create_workflow))
        .route(
            "/api/workflows/{workflow_id}",
            get(get_workflow).put(update_workflow),
        )
        .route(
            "/api/workflows/{workflow_id}/versions",
            post(create_workflow_version),
        )
        .route(
            "/api/workflow-versions/{workflow_version_id}/publish",
            post(publish_workflow_version),
        )
        .route(
            "/api/workflow-versions/{workflow_version_id}/steps",
            put(replace_workflow_version_steps),
        )
        .route(
            "/api/workflow-versions/{workflow_version_id}",
            delete(delete_workflow_version),
        )
        .route(
            "/api/workflow-assignment-candidates",
            get(list_assignment_candidates),
        )
        .route(
            "/api/workflow-assignment-candidates/assignees",
            get(list_assignment_candidate_assignees),
        )
        .route(
            "/api/workflow-assignments",
            get(list_workflow_assignments).post(create_workflow_assignment),
        )
        .route(
            "/api/workflow-assignments/bulk",
            post(bulk_create_workflow_assignments),
        )
        .route(
            "/api/workflow-assignments/{workflow_assignment_id}",
            put(update_workflow_assignment),
        )
}

pub(crate) use handlers::{
    ensure_specific_workflow_assignment_tx, ensure_workflow_assignment_for_form_version_tx,
};
