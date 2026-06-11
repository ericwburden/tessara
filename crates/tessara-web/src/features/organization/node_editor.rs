//! Owns the features::organization::node_editor module behavior.

#[cfg(feature = "hydrate")]
use crate::api::client::redirect_to_login;
#[cfg(feature = "hydrate")]
use crate::features::administration::{CreateNodePayload, UpdateNodePayload};
#[cfg(feature = "hydrate")]
use crate::features::organization::api::IntoNonemptyString;
#[cfg(feature = "hydrate")]
use crate::features::organization::api::current_search_param;
use crate::features::organization::tree::build_organization_tree;
#[cfg(feature = "hydrate")]
use crate::features::organization::types::{IdResponse, NodeTypeDefinition};
use crate::features::organization::types::{
    NodeMetadataFieldSummary, NodeTypeCatalogEntry, OrganizationNode, OrganizationNodeDetail,
    OrganizationTreeNode, ParentNodeOption,
};
use leptos::prelude::*;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
#[component]
/// Renders the metadata field input view.
pub(crate) fn MetadataFieldInput(
    field: NodeMetadataFieldSummary,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
) -> impl IntoView {
    let key = field.key.clone();
    let input_id = format!("organization-metadata-{}", field.key);
    let required_label = if field.required { " *" } else { "" };

    match field.field_type.as_str() {
        "boolean" => view! {
            <label class="form-field form-field--checkbox" for=input_id.clone()>
                <input
                    id=input_id.clone()
                    type="checkbox"
                    prop:checked=move || metadata_booleans.with(|values| values.get(&key).copied().unwrap_or(false))
                    on:change=move |event| {
                        metadata_booleans.update(|values| {
                            values.insert(field.key.clone(), event_target_checked(&event));
                        });
                    }
                />
                <span>{format!("{}{}", field.label, required_label)}</span>
            </label>
        }
        .into_any(),
        field_type => {
            let input_type = match field_type {
                "number" => "number",
                "date" => "date",
                _ => "text",
            };

            view! {
                <label class="form-field" for=input_id.clone()>
                    <span>{format!("{}{}", field.label, required_label)}</span>
                    <input
                        id=input_id.clone()
                        type=input_type
                        prop:value=move || metadata_values.with(|values| values.get(&key).cloned().unwrap_or_default())
                        on:input=move |event| {
                            metadata_values.update(|values| {
                                values.insert(field.key.clone(), event_target_value(&event));
                            });
                        }
                        required=field.required
                    />
                </label>
            }
            .into_any()
        }
    }
}

/// Handles the parent node options behavior.
pub(crate) fn parent_node_options(nodes: &[OrganizationNode]) -> Vec<ParentNodeOption> {
    let branches = build_organization_tree(nodes.to_vec());
    let mut options = Vec::new();
    append_parent_node_options(&branches, 0, &mut options);
    options
}

/// Handles the parent node options for edit behavior.
pub(crate) fn parent_node_options_for_edit(
    nodes: &[OrganizationNode],
    node_types: &[NodeTypeCatalogEntry],
    edited_node_id: &str,
    edited_node_type_id: &str,
) -> Vec<ParentNodeOption> {
    let excluded_ids = descendant_node_ids(nodes, edited_node_id);
    parent_node_options(nodes)
        .into_iter()
        .filter(|option| !excluded_ids.contains(&option.id))
        .filter(|option| {
            nodes
                .iter()
                .find(|node| node.id == option.id)
                .and_then(|node| {
                    node_types
                        .iter()
                        .find(|node_type| node_type.id == node.node_type_id)
                })
                .map(|node_type| {
                    node_type
                        .child_relationships
                        .iter()
                        .any(|relationship| relationship.node_type_id == edited_node_type_id)
                })
                .unwrap_or(false)
        })
        .collect()
}

/// Handles the descendant node ids behavior.
pub(crate) fn descendant_node_ids(nodes: &[OrganizationNode], root_id: &str) -> HashSet<String> {
    let mut descendants = HashSet::from([root_id.to_string()]);
    let mut changed = true;

    while changed {
        changed = false;
        for node in nodes {
            if descendants.contains(&node.id) {
                continue;
            }

            if node
                .parent_node_id
                .as_ref()
                .map(|parent_id| descendants.contains(parent_id))
                .unwrap_or(false)
            {
                descendants.insert(node.id.clone());
                changed = true;
            }
        }
    }

    descendants
}

/// Handles the append parent node options behavior.
pub(crate) fn append_parent_node_options(
    branches: &[OrganizationTreeNode],
    depth: usize,
    options: &mut Vec<ParentNodeOption>,
) {
    for branch in branches {
        let prefix = if depth == 0 {
            String::new()
        } else {
            format!("{} ", "--".repeat(depth))
        };

        options.push(ParentNodeOption {
            id: branch.node.id.clone(),
            label: format!(
                "{}{} ({})",
                prefix, branch.node.name, branch.node.node_type_singular_label
            ),
        });
        append_parent_node_options(&branch.children, depth + 1, options);
    }
}

/// Handles the available node types for parent behavior.
pub(crate) fn available_node_types_for_parent(
    parent_node_id: &str,
    node_types: &[NodeTypeCatalogEntry],
    nodes: &[OrganizationNode],
) -> Vec<NodeTypeCatalogEntry> {
    if parent_node_id.is_empty() {
        return node_types
            .iter()
            .filter(|node_type| node_type.is_root_type)
            .cloned()
            .collect();
    }

    let Some(parent_node) = nodes.iter().find(|node| node.id == parent_node_id) else {
        return Vec::new();
    };
    let Some(parent_type) = node_types
        .iter()
        .find(|node_type| node_type.id == parent_node.node_type_id)
    else {
        return Vec::new();
    };

    parent_type
        .child_relationships
        .iter()
        .filter_map(|relationship| {
            node_types
                .iter()
                .find(|node_type| node_type.id == relationship.node_type_id)
                .cloned()
        })
        .collect()
}

/// Loads the load organization create options data.
pub(crate) fn load_organization_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    selected_node_type_id: RwSignal<String>,
    selected_parent_node_id: RwSignal<String>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;

            match (node_type_response, node_response) {
                (Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_type_response), Ok(node_response))
                    if node_type_response.ok() && node_response.ok() =>
                {
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;

                    match (loaded_node_types, loaded_nodes) {
                        (Ok(loaded_node_types), Ok(loaded_nodes)) => {
                            let requested_node_type_id = current_search_param("node_type_id");
                            let requested_parent_id = current_search_param("parent_node_id")
                                .or_else(|| current_search_param("parent_id"));
                            let selected_parent = requested_parent_id
                                .filter(|requested| {
                                    loaded_nodes.iter().any(|node| node.id == *requested)
                                })
                                .unwrap_or_default();
                            let available_types = available_node_types_for_parent(
                                &selected_parent,
                                &loaded_node_types,
                                &loaded_nodes,
                            );
                            let selected_type = requested_node_type_id
                                .filter(|requested| {
                                    available_types
                                        .iter()
                                        .any(|node_type| node_type.id == *requested)
                                })
                                .or_else(|| {
                                    available_types
                                        .first()
                                        .map(|node_type| node_type.id.clone())
                                });

                            nodes.set(loaded_nodes);
                            node_types.set(loaded_node_types);
                            selected_node_type_id.set(selected_type.unwrap_or_default());
                            selected_parent_node_id.set(selected_parent);
                            is_loading.set(false);
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Create options could not be read.".into()));
                        }
                    }
                }
                (Ok(_), Ok(_)) => {
                    is_loading.set(false);
                    message.set(Some(
                        "Create options returned an unexpected response.".into(),
                    ));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the organization APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_types,
            nodes,
            selected_node_type_id,
            selected_parent_node_id,
            is_loading,
            message,
        );
    }
}

/// Loads the load node type metadata data.
pub(crate) fn load_node_type_metadata(
    node_type_id: String,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let response =
                gloo_net::http::Request::get(&format!("/api/admin/node-types/{node_type_id}"))
                    .send()
                    .await;

            match response {
                Ok(response) if response.status() == 401 => redirect_to_login(),
                Ok(response) if response.ok() => {
                    match response.json::<NodeTypeDefinition>().await {
                        Ok(definition) => {
                            metadata_fields.set(definition.metadata_fields);
                            metadata_values.set(HashMap::new());
                            metadata_booleans.set(HashMap::new());
                        }
                        Err(_) => {
                            metadata_fields.set(Vec::new());
                            message.set(Some("Metadata fields could not be read.".into()));
                        }
                    }
                }
                Ok(_) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some(
                        "Metadata fields returned an unexpected response.".into(),
                    ));
                }
                Err(_) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some("Could not reach the node type API.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_type_id,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            message,
        );
    }
}

/// Submits the submit create node request.
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

/// Loads the load organization edit options data.
pub(crate) fn load_organization_edit_options(
    node_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;
            let detail_response = gloo_net::http::Request::get(&format!("/api/nodes/{node_id}"))
                .send()
                .await;

            match (node_type_response, node_response, detail_response) {
                (Ok(response), _, _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_type_response), Ok(node_response), Ok(detail_response))
                    if node_type_response.ok() && node_response.ok() && detail_response.ok() =>
                {
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;
                    let loaded_detail = detail_response.json::<OrganizationNodeDetail>().await;

                    match (loaded_node_types, loaded_nodes, loaded_detail) {
                        (Ok(loaded_node_types), Ok(loaded_nodes), Ok(loaded_detail)) => {
                            let metadata_response = gloo_net::http::Request::get(&format!(
                                "/api/admin/node-types/{}",
                                loaded_detail.node_type_id
                            ))
                            .send()
                            .await;

                            match metadata_response {
                                Ok(response) if response.status() == 401 => {
                                    is_loading.set(false);
                                    redirect_to_login();
                                }
                                Ok(response) if response.ok() => {
                                    match response.json::<NodeTypeDefinition>().await {
                                        Ok(definition) => {
                                            let (text_values, boolean_values) =
                                                metadata_input_state(
                                                    &definition.metadata_fields,
                                                    &loaded_detail.metadata,
                                                );

                                            selected_parent_node_id.set(
                                                loaded_detail
                                                    .parent_node_id
                                                    .clone()
                                                    .unwrap_or_default(),
                                            );
                                            name.set(loaded_detail.name.clone());
                                            metadata_fields.set(definition.metadata_fields);
                                            metadata_values.set(text_values);
                                            metadata_booleans.set(boolean_values);
                                            detail.set(Some(loaded_detail));
                                            nodes.set(loaded_nodes);
                                            node_types.set(loaded_node_types);
                                            is_loading.set(false);
                                        }
                                        Err(_) => {
                                            is_loading.set(false);
                                            message.set(Some(
                                                "Metadata fields could not be read.".into(),
                                            ));
                                        }
                                    }
                                }
                                Ok(_) => {
                                    is_loading.set(false);
                                    message.set(Some(
                                        "Metadata fields returned an unexpected response.".into(),
                                    ));
                                }
                                Err(_) => {
                                    is_loading.set(false);
                                    message.set(Some("Could not reach the node type API.".into()));
                                }
                            }
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Edit options could not be read.".into()));
                        }
                    }
                }
                (Ok(_), Ok(_), Ok(_)) => {
                    is_loading.set(false);
                    message.set(Some("Edit options returned an unexpected response.".into()));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the organization APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_id,
            node_types,
            nodes,
            detail,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_loading,
            message,
        );
    }
}

/// Submits the submit update node request.
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

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Collects the collect node metadata values.
pub(crate) fn collect_node_metadata(
    fields: &[NodeMetadataFieldSummary],
    values: &HashMap<String, String>,
    booleans: &HashMap<String, bool>,
) -> Result<serde_json::Map<String, Value>, String> {
    let mut metadata = serde_json::Map::new();

    for field in fields {
        match field.field_type.as_str() {
            "boolean" => {
                metadata.insert(
                    field.key.clone(),
                    Value::Bool(booleans.get(&field.key).copied().unwrap_or(false)),
                );
            }
            "number" => {
                let raw = values
                    .get(&field.key)
                    .map(|value| value.trim())
                    .unwrap_or_default();
                if raw.is_empty() {
                    if field.required {
                        return Err(format!("{} is required.", field.label));
                    }
                } else {
                    let parsed = raw
                        .parse::<f64>()
                        .map_err(|_| format!("{} must be a number.", field.label))?;
                    metadata.insert(field.key.clone(), serde_json::json!(parsed));
                }
            }
            "multi_choice" => {
                let raw = values
                    .get(&field.key)
                    .map(|value| value.trim())
                    .unwrap_or_default();
                let selected = raw
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| Value::String(value.to_string()))
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    if field.required {
                        return Err(format!("{} is required.", field.label));
                    }
                } else {
                    metadata.insert(field.key.clone(), Value::Array(selected));
                }
            }
            _ => {
                let raw = values
                    .get(&field.key)
                    .map(|value| value.trim())
                    .unwrap_or_default();
                if raw.is_empty() {
                    if field.required {
                        return Err(format!("{} is required.", field.label));
                    }
                } else {
                    metadata.insert(field.key.clone(), Value::String(raw.to_string()));
                }
            }
        }
    }

    Ok(metadata)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the metadata input state behavior.
pub(crate) fn metadata_input_state(
    fields: &[NodeMetadataFieldSummary],
    metadata: &Value,
) -> (HashMap<String, String>, HashMap<String, bool>) {
    let values = metadata.as_object();
    let mut text_values = HashMap::new();
    let mut boolean_values = HashMap::new();

    for field in fields {
        let value = values.and_then(|values| values.get(&field.key));
        if field.field_type == "boolean" {
            boolean_values.insert(
                field.key.clone(),
                value.and_then(Value::as_bool).unwrap_or(false),
            );
        } else if let Some(value) = value {
            text_values.insert(field.key.clone(), metadata_input_value(value));
        }
    }

    (text_values, boolean_values)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
/// Handles the metadata input value behavior.
pub(crate) fn metadata_input_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(_) => value.to_string(),
    }
}
