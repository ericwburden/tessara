//! Node type administration save actions.

#[cfg(feature = "hydrate")]
use crate::features::organization::{
    CreateNodeMetadataFieldRequest, UpdateNodeMetadataFieldRequest,
};
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
use leptos::prelude::*;

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
