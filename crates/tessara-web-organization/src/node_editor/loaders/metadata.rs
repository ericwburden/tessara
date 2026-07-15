//! Metadata loaders for the organization node editor.

#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
use crate::types::NodeMetadataFieldSummary;
use leptos::prelude::*;
use std::collections::HashMap;

#[cfg(feature = "hydrate")]
use super::super::api::{NodeEditorApiError, fetch_node_type_metadata_fields};

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
            match fetch_node_type_metadata_fields(&node_type_id).await {
                Ok(fields) => {
                    metadata_fields.set(fields);
                    metadata_values.set(HashMap::new());
                    metadata_booleans.set(HashMap::new());
                }
                Err(NodeEditorApiError::Unauthorized) => redirect_to_login(),
                Err(NodeEditorApiError::Message(error)) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some(error));
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
