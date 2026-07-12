use axum::{
    Router,
    routing::{delete, get, patch, post},
};

use super::{
    create_dataset, delete_dataset, delete_dataset_revision, get_dataset, get_dataset_revision,
    list_dataset_distinct_values, list_dataset_revisions, list_datasets, preview_dataset_sql,
    preview_existing_dataset_sql, publish_dataset_revision, run_dataset_table,
    save_dataset_draft_revision, update_dataset_revision_label, update_dataset_revision_options,
    update_dataset_tags,
};
use crate::db::AppState;

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/admin/datasets", post(create_dataset))
        .route("/api/admin/datasets/sql-preview", post(preview_dataset_sql))
        .route(
            "/api/admin/datasets/{dataset_id}/sql-preview",
            post(preview_existing_dataset_sql),
        )
        .route("/api/admin/datasets/{dataset_id}", delete(delete_dataset))
        .route(
            "/api/admin/datasets/{dataset_id}/tags",
            patch(update_dataset_tags),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/draft-revision",
            post(save_dataset_draft_revision),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish",
            post(publish_dataset_revision),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/label",
            patch(update_dataset_revision_label),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/options",
            patch(update_dataset_revision_options),
        )
        .route(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}",
            delete(delete_dataset_revision),
        )
        .route("/api/datasets", get(list_datasets))
        .route("/api/datasets/{dataset_id}", get(get_dataset))
        .route(
            "/api/datasets/{dataset_id}/revisions",
            get(list_dataset_revisions),
        )
        .route(
            "/api/datasets/{dataset_id}/revisions/{revision_id}",
            get(get_dataset_revision),
        )
        .route("/api/datasets/{dataset_id}/table", get(run_dataset_table))
        .route(
            "/api/datasets/{dataset_id}/distinct-values",
            get(list_dataset_distinct_values),
        )
}
