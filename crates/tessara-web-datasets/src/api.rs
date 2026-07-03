//! Client-side API orchestration for the Datasets feature.
//!
//! Keep endpoint calls, request assembly, and response handling for Datasets screens here; pure DTOs and display formatting belong in sibling modules.

#[cfg(feature = "hydrate")]
use super::types::DatasetSqlPreviewResponse;
#[cfg(feature = "hydrate")]
use super::types::{
    DatasetDefinition, DatasetDraftRevisionResponse, DatasetFormOption, DatasetPayload,
    DatasetPublishRevisionResponse, DatasetRenderedForm, DatasetRevisionDetail,
    DatasetRevisionLabelRequest, DatasetRevisionLabelResponse, DatasetRevisionOptionsRequest,
    DatasetRevisionSummary, DatasetSummary, DatasetTable, DatasetUserOption, NodeResponse,
    SessionAccount,
};

#[cfg(feature = "hydrate")]
use crate::http::{fetch_json_request, send_json_request};

#[cfg(feature = "hydrate")]
/// Fetches the fetch json data.
async fn fetch_json<T>(url: &str, action: &str) -> Result<Option<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    fetch_json_request(url, action).await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch account data.
pub(super) async fn fetch_account() -> Result<Option<SessionAccount>, String> {
    fetch_json("/api/me", "account").await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch datasets data.
pub(super) async fn fetch_datasets() -> Result<Option<Vec<DatasetSummary>>, String> {
    fetch_json("/api/datasets", "Dataset list").await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch dataset detail data.
pub(super) async fn fetch_dataset_detail(
    dataset_id: &str,
) -> Result<Option<DatasetDefinition>, String> {
    fetch_json(&format!("/api/datasets/{dataset_id}"), "Dataset detail").await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch dataset table data.
pub(super) async fn fetch_dataset_table(dataset_id: &str) -> Result<Option<DatasetTable>, String> {
    fetch_json(
        &format!("/api/datasets/{dataset_id}/table"),
        "Dataset preview",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_dataset_revisions(
    dataset_id: &str,
) -> Result<Option<Vec<DatasetRevisionSummary>>, String> {
    fetch_json(
        &format!("/api/datasets/{dataset_id}/revisions"),
        "Dataset revisions",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_dataset_revision(
    dataset_id: &str,
    revision_id: &str,
) -> Result<Option<DatasetRevisionDetail>, String> {
    fetch_json(
        &format!("/api/datasets/{dataset_id}/revisions/{revision_id}"),
        "Dataset revision",
    )
    .await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch forms data.
pub(super) async fn fetch_forms() -> Result<Option<Vec<DatasetFormOption>>, String> {
    fetch_json("/api/forms", "Form options").await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch nodes data.
pub(super) async fn fetch_nodes() -> Result<Option<Vec<NodeResponse>>, String> {
    fetch_json("/api/nodes", "Visibility nodes").await
}

#[cfg(feature = "hydrate")]
pub(super) async fn fetch_users() -> Result<Option<Vec<DatasetUserOption>>, String> {
    fetch_json("/api/admin/users", "User options").await
}

#[cfg(feature = "hydrate")]
/// Fetches the fetch rendered form data.
pub(super) async fn fetch_rendered_form(
    form_version_id: &str,
) -> Result<Option<DatasetRenderedForm>, String> {
    fetch_json(
        &format!("/api/form-versions/{form_version_id}/render"),
        "Rendered form",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn save_dataset_payload(
    dataset_id: Option<&str>,
    payload: &DatasetPayload,
) -> Result<serde_json::Value, String> {
    let body = serde_json::to_string(payload)
        .map_err(|_| "Dataset payload could not be prepared.".to_string())?;

    if let Some(dataset_id) = dataset_id {
        let response: DatasetDraftRevisionResponse = send_json_request(
            gloo_net::http::Request::post(&format!(
                "/api/admin/datasets/{dataset_id}/draft-revision"
            )),
            Some(body),
            "dataset draft revision",
        )
        .await?;
        Ok(serde_json::json!({
            "dataset_id": response.dataset_id,
            "revision_id": response.revision_id
        }))
    } else {
        send_json_request(
            gloo_net::http::Request::post("/api/admin/datasets"),
            Some(body),
            "dataset creation",
        )
        .await
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn publish_dataset_revision(
    dataset_id: &str,
    revision_id: &str,
) -> Result<DatasetPublishRevisionResponse, String> {
    send_json_request(
        gloo_net::http::Request::post(&format!(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/publish"
        )),
        None,
        "dataset revision publish",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn delete_dataset_revision(
    dataset_id: &str,
    revision_id: &str,
) -> Result<serde_json::Value, String> {
    send_json_request(
        gloo_net::http::Request::delete(&format!(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}"
        )),
        None,
        "dataset revision delete",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_dataset_revision_label(
    dataset_id: &str,
    revision_id: &str,
    version_label: String,
    revision_notes: String,
) -> Result<DatasetRevisionLabelResponse, String> {
    let label = version_label.trim();
    let notes = revision_notes.trim();
    let body = serde_json::to_string(&DatasetRevisionLabelRequest {
        version_label: Some(label.to_string()),
        revision_notes: Some(notes.to_string()),
    })
    .map_err(|_| "Revision label could not be prepared.".to_string())?;

    send_json_request(
        gloo_net::http::Request::patch(&format!(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/label"
        )),
        Some(body),
        "dataset revision label",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn update_dataset_revision_options(
    dataset_id: &str,
    revision_id: &str,
    force_new_major_version: bool,
) -> Result<DatasetRevisionDetail, String> {
    let body = serde_json::to_string(&DatasetRevisionOptionsRequest {
        force_new_major_version,
    })
    .map_err(|_| "Revision options could not be prepared.".to_string())?;

    send_json_request(
        gloo_net::http::Request::patch(&format!(
            "/api/admin/datasets/{dataset_id}/revisions/{revision_id}/options"
        )),
        Some(body),
        "dataset revision options",
    )
    .await
}

#[cfg(feature = "hydrate")]
pub(super) async fn preview_dataset_sql_payload(
    dataset_id: Option<&str>,
    payload: &DatasetPayload,
) -> Result<DatasetSqlPreviewResponse, String> {
    let body = serde_json::to_string(payload)
        .map_err(|_| "Dataset payload could not be prepared.".to_string())?;
    let request = if let Some(dataset_id) = dataset_id {
        gloo_net::http::Request::post(&format!("/api/admin/datasets/{dataset_id}/sql-preview"))
    } else {
        gloo_net::http::Request::post("/api/admin/datasets/sql-preview")
    };

    send_json_request(request, Some(body), "dataset SQL preview").await
}
