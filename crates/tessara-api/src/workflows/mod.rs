mod generated;
mod handlers;
mod repo;
mod runtime;
mod service;

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
    list_assignment_candidate_assignees, list_assignment_candidates, list_pending_work,
    list_workflow_assignments, list_workflows, publish_workflow_version,
    replace_workflow_version_steps, start_assignment, update_workflow, update_workflow_assignment,
};
pub use runtime::{
    complete_workflow_step_and_advance, ensure_submission_runtime_linkage,
    list_pending_assignments_for_account,
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
        .route("/api/workflow-assignments/pending", get(list_pending_work))
        .route(
            "/api/workflow-assignments/{workflow_assignment_id}",
            put(update_workflow_assignment),
        )
        .route(
            "/api/workflow-assignments/{workflow_assignment_id}/start",
            post(start_assignment),
        )
}
