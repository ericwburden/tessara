//! Signal-aware submit actions for organization node editor pages.

#[cfg(feature = "hydrate")]
use crate::features::administration::{CreateNodePayload, UpdateNodePayload};
use crate::features::organization::types::NodeMetadataFieldSummary;
#[cfg(feature = "hydrate")]
use crate::http::navigate_to_href;
#[cfg(feature = "hydrate")]
use crate::utils::text::IntoNonemptyString;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::node_metadata::collect_node_metadata;
#[cfg(feature = "hydrate")]
use super::api::{create_node, update_node};

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SubmitCreateNodeInput {
    pub(crate) selected_node_type_id: RwSignal<String>,
    pub(crate) selected_parent_node_id: RwSignal<String>,
    pub(crate) name: RwSignal<String>,
    pub(crate) metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    pub(crate) metadata_values: RwSignal<HashMap<String, String>>,
    pub(crate) metadata_booleans: RwSignal<HashMap<String, bool>>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) message: RwSignal<Option<String>>,
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
pub(crate) struct SubmitUpdateNodeInput {
    pub(crate) node_id: String,
    pub(crate) selected_parent_node_id: RwSignal<String>,
    pub(crate) name: RwSignal<String>,
    pub(crate) metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    pub(crate) metadata_values: RwSignal<HashMap<String, String>>,
    pub(crate) metadata_booleans: RwSignal<HashMap<String, bool>>,
    pub(crate) is_saving: RwSignal<bool>,
    pub(crate) message: RwSignal<Option<String>>,
}

/// Validates and submits a create-node request, then navigates to the created node.
pub(crate) fn submit_create_node(input: SubmitCreateNodeInput) {
    #[cfg(feature = "hydrate")]
    {
        let SubmitCreateNodeInput {
            selected_node_type_id,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_saving,
            message,
        } = input;

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

            match create_node(payload).await {
                Ok(created) => navigate_to_href(&format!("/organization/{}", created.id)),
                Err(error) => {
                    if error != "Authentication is required." {
                        message.set(Some(error));
                    }
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = input;
    }
}

/// Validates and submits an update-node request, then navigates to the updated node.
pub(crate) fn submit_update_node(input: SubmitUpdateNodeInput) {
    #[cfg(feature = "hydrate")]
    {
        let SubmitUpdateNodeInput {
            node_id,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_saving,
            message,
        } = input;

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

            match update_node(&node_id, payload).await {
                Ok(updated) => navigate_to_href(&format!("/organization/{}", updated.id)),
                Err(error) => {
                    if error != "Authentication is required." {
                        message.set(Some(error));
                    }
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = input;
    }
}
