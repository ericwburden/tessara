//! Metadata field mutation helpers for node type administration components.

use leptos::prelude::*;

/// Deletes a metadata field and refreshes the node type detail state.
pub(super) fn delete_node_type_metadata_field(
    field_id: String,
    field_message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    clear_field_editor: impl Fn() + 'static + Copy,
    on_metadata_changed: impl Fn() + 'static + Copy,
) {
    #[cfg(feature = "hydrate")]
    {
        use crate::http::send_json_id_request;

        leptos::task::spawn_local(async move {
            field_message.set(None);
            match send_json_id_request(
                gloo_net::http::Request::delete(&format!(
                    "/api/admin/node-metadata-fields/{field_id}"
                )),
                None,
                "Delete metadata field",
            )
            .await
            {
                Ok(_) => {
                    sheet_open.set(false);
                    clear_field_editor();
                    on_metadata_changed();
                }
                Err(error) => field_message.set(Some(error)),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            field_id,
            field_message,
            sheet_open,
            clear_field_editor,
            on_metadata_changed,
        );
    }
}
