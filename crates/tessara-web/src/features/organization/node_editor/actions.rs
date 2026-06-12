//! Signal-aware submit actions for organization node editor pages.

#[cfg(feature = "hydrate")]
use crate::features::administration::{CreateNodePayload, UpdateNodePayload};
#[cfg(feature = "hydrate")]
use crate::features::organization::types::IdResponse;
use crate::features::organization::types::NodeMetadataFieldSummary;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
#[cfg(feature = "hydrate")]
use crate::utils::text::IntoNonemptyString;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::node_metadata::collect_node_metadata;

/// Validates and submits a create-node request, then navigates to the created node.
pub(crate) fn submit_create_node(
    selected_node_type_id: RwSignal<String>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let node_type_id = selected_node_type_id.get();
        let node_name = name.get().trim().to_string();
        if node_type_id.is_empty() {
            message.set(Some("Select a node type before saving.".into()));
            return;
        }
        if node_name.is_empty() {
            message.set(Some("Name is required.".into()));
            return;
        }

        let metadata = match collect_node_metadata(
            &metadata_fields.get(),
            &metadata_values.get(),
            &metadata_booleans.get(),
        ) {
            Ok(metadata) => metadata,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };

        let parent_node_id = selected_parent_node_id
            .get()
            .trim()
            .to_string()
            .into_nonempty();
        let payload = CreateNodePayload {
            node_type_id,
            parent_node_id,
            name: node_name,
            metadata,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Create request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::post("/api/admin/nodes")
                .header("Content-Type", "application/json")
                .body(body)
                .expect("json request body should be valid")
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<IdResponse>().await {
                    Ok(created) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(&format!("/organization/{}", created.id));
                        }
                    }
                    Err(_) => {
                        message.set(Some("Create response could not be read.".into()));
                        is_saving.set(false);
                    }
                },
                Ok(response) => {
                    message.set(Some(format!(
                        "Create failed with status {}.",
                        response.status()
                    )));
                    is_saving.set(false);
                }
                Err(_) => {
                    message.set(Some("Could not reach the create node API.".into()));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            selected_node_type_id,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_saving,
            message,
        );
    }
}

/// Validates and submits an update-node request, then navigates to the updated node.
pub(crate) fn submit_update_node(
    node_id: String,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let node_name = name.get().trim().to_string();
        if node_name.is_empty() {
            message.set(Some("Name is required.".into()));
            return;
        }

        let metadata = match collect_node_metadata(
            &metadata_fields.get(),
            &metadata_values.get(),
            &metadata_booleans.get(),
        ) {
            Ok(metadata) => metadata,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };

        let payload = UpdateNodePayload {
            parent_node_id: selected_parent_node_id
                .get()
                .trim()
                .to_string()
                .into_nonempty(),
            name: node_name,
            metadata,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Update request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::put(&format!("/api/admin/nodes/{node_id}"))
                .header("Content-Type", "application/json")
                .body(body)
                .expect("json request body should be valid")
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<IdResponse>().await {
                    Ok(updated) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(&format!("/organization/{}", updated.id));
                        }
                    }
                    Err(_) => {
                        message.set(Some("Update response could not be read.".into()));
                        is_saving.set(false);
                    }
                },
                Ok(response) => {
                    message.set(Some(format!(
                        "Update failed with status {}.",
                        response.status()
                    )));
                    is_saving.set(false);
                }
                Err(_) => {
                    message.set(Some("Could not reach the update node API.".into()));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_id,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_saving,
            message,
        );
    }
}
