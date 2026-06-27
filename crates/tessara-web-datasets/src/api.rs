//! Client-side API orchestration for the Datasets feature.
//!
//! Keep endpoint calls, request assembly, and response handling for Datasets screens here; pure DTOs and display formatting belong in sibling modules.

#[cfg(feature = "hydrate")]
use super::types::DatasetSqlPreviewResponse;
#[cfg(feature = "hydrate")]
use super::types::{
    DatasetDefinition, DatasetFormOption, DatasetPayload, DatasetRenderedForm, DatasetSummary,
    DatasetTable, DatasetUserOption, NodeResponse, SessionAccount,
};

#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_request};

#[cfg(feature = "hydrate")]
/// Fetches the fetch json data.
async fn fetch_json<T>(url: &str, action: &str) -> Result<Option<T>, String>
where
    T: serde::de::DeserializeOwned,
{
    match gloo_net::http::Request::get(url).send().await {
        Ok(response) if response.status() == 401 => {
            redirect_to_login();
            Ok(None)
        }
        Ok(response) if response.ok() => response
            .json::<T>()
            .await
            .map(Some)
            .map_err(|_| format!("{action} could not be read.")),
        Ok(response) => Err(format!(
            "{action} failed with status {}.",
            response.status()
        )),
        Err(_) => Err(format!("Could not reach the {action} API.")),
    }
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
    match gloo_net::http::Request::get(&format!("/api/form-versions/{form_version_id}/render"))
        .send()
        .await
    {
        Ok(response) if response.ok() => response
            .json::<DatasetRenderedForm>()
            .await
            .map(Some)
            .map_err(|_| "Rendered form could not be read.".to_string()),
        _ => Ok(None),
    }
}

#[cfg(feature = "hydrate")]
pub(super) async fn save_dataset_payload(
    dataset_id: Option<&str>,
    payload: &DatasetPayload,
) -> Result<serde_json::Value, String> {
    let body = serde_json::to_string(payload)
        .map_err(|_| "Dataset payload could not be prepared.".to_string())?;

    if let Some(dataset_id) = dataset_id {
        send_json_request(
            gloo_net::http::Request::put(&format!("/api/admin/datasets/{dataset_id}")),
            Some(body),
            "dataset update",
        )
        .await
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
