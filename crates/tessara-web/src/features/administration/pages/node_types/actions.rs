//! Page-local save actions for node-type administration.

use super::state::AdministrationNodeTypesPageState;
#[cfg(feature = "hydrate")]
use crate::features::administration::api::load_admin_node_type_catalog;
use crate::features::organization::NodeTypeUpsertRequest;
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
use leptos::prelude::*;

pub(super) fn save_admin_node_type(state: AdministrationNodeTypesPageState) {
    let trimmed_name = state.name.get().trim().to_string();
    let trimmed_slug = state.slug.get().trim().to_string();
    let trimmed_plural = state.plural_label.get().trim().to_string();
    if trimmed_name.is_empty() || trimmed_slug.is_empty() {
        state
            .message
            .set(Some("Name and slug are required.".into()));
        return;
    }

    let request = NodeTypeUpsertRequest {
        name: trimmed_name,
        slug: trimmed_slug,
        plural_label: if trimmed_plural.is_empty() {
            None
        } else {
            Some(trimmed_plural)
        },
        parent_node_type_ids: state
            .parent_node_type_ids
            .get()
            .into_iter()
            .collect::<Vec<_>>(),
        child_node_type_ids: state
            .child_node_type_ids
            .get()
            .into_iter()
            .collect::<Vec<_>>(),
    };
    let body = match serde_json::to_string(&request) {
        Ok(body) => body,
        Err(_) => {
            state
                .message
                .set(Some("Node type request could not be prepared.".into()));
            return;
        }
    };
    let selected_id = state.selected_node_type_id.get_untracked();
    let creating = state.is_creating.get_untracked() || selected_id.is_none();

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            state.is_saving.set(true);
            state.message.set(None);
            let builder = if creating {
                gloo_net::http::Request::post("/api/admin/node-types")
            } else if let Some(node_type_id) = selected_id {
                gloo_net::http::Request::put(&format!("/api/admin/node-types/{node_type_id}"))
            } else {
                state.is_saving.set(false);
                state
                    .message
                    .set(Some("Select a node type before saving.".into()));
                return;
            };

            match send_json_id_request(builder, Some(body), "Save node type").await {
                Ok(response) => {
                    state.is_creating.set(false);
                    load_admin_node_type_catalog(
                        state.node_types,
                        state.selected_node_type_id,
                        state.is_loading,
                        state.message,
                        Some(response.id),
                    );
                }
                Err(error) => state.message.set(Some(error)),
            }
            state.is_saving.set(false);
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (body, creating, state.is_saving);
}
