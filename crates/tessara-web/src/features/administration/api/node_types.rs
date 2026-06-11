//! Node type administration API loaders and save orchestration.

#[cfg(feature = "hydrate")]
use crate::features::organization::{
    CreateNodeMetadataFieldRequest, UpdateNodeMetadataFieldRequest,
};
use crate::features::organization::{NodeTypeCatalogEntry, NodeTypeDefinition};
#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_id_request};
use leptos::prelude::*;
use std::collections::HashSet;

#[allow(clippy::too_many_arguments)]
/// Saves a node type metadata field create or update request.
pub(crate) fn save_node_type_metadata_field(
    node_type_id: String,
    editing_field_id: RwSignal<Option<String>>,
    field_label: RwSignal<String>,
    field_key: RwSignal<String>,
    field_type: RwSignal<String>,
    field_required: RwSignal<bool>,
    is_saving_field: RwSignal<bool>,
    field_message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    clear_field_editor: impl Fn() + 'static + Copy + Send + Sync,
    on_metadata_changed: impl Fn() + 'static + Copy + Send + Sync,
) {
    let label = field_label.get().trim().to_string();
    let key = field_key.get().trim().to_string();
    let field_type_value = field_type.get();
    let required = field_required.get();
    if label.is_empty() || key.is_empty() {
        field_message.set(Some("Metadata label and key are required.".into()));
        return;
    }

    #[cfg(feature = "hydrate")]
    {
        let editing_id = editing_field_id.get_untracked();
        leptos::task::spawn_local(async move {
            is_saving_field.set(true);
            field_message.set(None);
            let result = if let Some(field_id) = editing_id {
                let request = UpdateNodeMetadataFieldRequest {
                    key,
                    label,
                    field_type: field_type_value,
                    required,
                };
                match serde_json::to_string(&request) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::put(&format!(
                                "/api/admin/node-metadata-fields/{field_id}"
                            )),
                            Some(body),
                            "Save metadata field",
                        )
                        .await
                    }
                    Err(_) => Err("Metadata field request could not be prepared.".into()),
                }
            } else {
                let request = CreateNodeMetadataFieldRequest {
                    node_type_id,
                    key,
                    label,
                    field_type: field_type_value,
                    required,
                };
                match serde_json::to_string(&request) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::post("/api/admin/node-metadata-fields"),
                            Some(body),
                            "Create metadata field",
                        )
                        .await
                    }
                    Err(_) => Err("Metadata field request could not be prepared.".into()),
                }
            };

            match result {
                Ok(_) => {
                    sheet_open.set(false);
                    clear_field_editor();
                    on_metadata_changed();
                }
                Err(error) => field_message.set(Some(error)),
            }
            is_saving_field.set(false);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        node_type_id,
        editing_field_id,
        label,
        key,
        field_type_value,
        required,
        is_saving_field,
        sheet_open,
        clear_field_editor,
        on_metadata_changed,
    );
}

/// Loads the administration node type catalog.
pub(crate) fn load_admin_node_type_catalog(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    selected_node_type_id: RwSignal<Option<String>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    preferred_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            match gloo_net::http::Request::get("/api/node-types").send().await {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<NodeTypeCatalogEntry>>().await {
                        Ok(items) => {
                            let selected = preferred_id
                                .or_else(|| {
                                    selected_node_type_id
                                        .get_untracked()
                                        .filter(|id| items.iter().any(|item| item.id == *id))
                                })
                                .or_else(|| items.first().map(|item| item.id.clone()));
                            node_types.set(items);
                            selected_node_type_id.set(selected);
                            message.set(None);
                        }
                        Err(_) => {
                            node_types.set(Vec::new());
                            message.set(Some("Node type response could not be read.".into()));
                        }
                    }
                    is_loading.set(false);
                }
                Ok(response) => {
                    let status = response.status();
                    node_types.set(Vec::new());
                    message.set(Some(format!(
                        "Load node types failed with status {status}."
                    )));
                    is_loading.set(false);
                }
                Err(_) => {
                    node_types.set(Vec::new());
                    message.set(Some("Could not reach the node type API.".into()));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_types, selected_node_type_id, message, preferred_id);
        is_loading.set(false);
    }
}

#[allow(clippy::too_many_arguments)]
/// Loads the selected administration node type detail.
pub(crate) fn load_admin_node_type_detail(
    node_type_id: String,
    selected_detail: RwSignal<Option<NodeTypeDefinition>>,
    detail_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    name: RwSignal<String>,
    slug: RwSignal<String>,
    plural_label: RwSignal<String>,
    parent_node_type_ids: RwSignal<HashSet<String>>,
    child_node_type_ids: RwSignal<HashSet<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            detail_loading.set(true);
            match gloo_net::http::Request::get(&format!("/api/admin/node-types/{node_type_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<NodeTypeDefinition>().await {
                        Ok(detail) => {
                            name.set(detail.name.clone());
                            slug.set(detail.slug.clone());
                            plural_label.set(detail.plural_label.clone());
                            parent_node_type_ids.set(
                                detail
                                    .parent_relationships
                                    .iter()
                                    .map(|peer| peer.node_type_id.clone())
                                    .collect(),
                            );
                            child_node_type_ids.set(
                                detail
                                    .child_relationships
                                    .iter()
                                    .map(|peer| peer.node_type_id.clone())
                                    .collect(),
                            );
                            selected_detail.set(Some(detail));
                            message.set(None);
                        }
                        Err(_) => {
                            selected_detail.set(None);
                            message
                                .set(Some("Node type detail response could not be read.".into()));
                        }
                    }
                    detail_loading.set(false);
                }
                Ok(response) => {
                    selected_detail.set(None);
                    message.set(Some(format!(
                        "Load node type detail failed with status {}.",
                        response.status()
                    )));
                    detail_loading.set(false);
                }
                Err(_) => {
                    selected_detail.set(None);
                    message.set(Some("Could not reach the node type detail API.".into()));
                    detail_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_type_id,
            selected_detail,
            message,
            name,
            slug,
            plural_label,
            parent_node_type_ids,
            child_node_type_ids,
        );
        detail_loading.set(false);
    }
}
