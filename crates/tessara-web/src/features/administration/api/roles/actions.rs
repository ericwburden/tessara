//! Role-management save actions.

#[cfg(feature = "hydrate")]
use super::{load_admin_role_detail, load_admin_roles_context};
use crate::features::administration::models::{AdminCapabilitySummary, AdminRoleDetail};
#[cfg(feature = "hydrate")]
use crate::features::administration::models::{CreateAdminRolePayload, UpdateAdminRolePayload};
use crate::features::organization::AdminRoleSummary;
#[cfg(feature = "hydrate")]
use crate::http::send_json_id_request;
use leptos::prelude::*;

#[allow(clippy::too_many_arguments)]
/// Saves an administration role create or update request.
pub(crate) fn save_admin_role(
    editing_role_id: RwSignal<Option<String>>,
    role_name: RwSignal<String>,
    selected_capability_ids: RwSignal<Vec<String>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    sheet_open: RwSignal<bool>,
    roles: RwSignal<Vec<AdminRoleSummary>>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_role_id: RwSignal<Option<String>>,
    selected_role_detail: RwSignal<Option<AdminRoleDetail>>,
    detail_loading: RwSignal<bool>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);
            let capability_ids = selected_capability_ids.get();
            let editing_id = editing_role_id.get_untracked();
            let result = if let Some(role_id) = editing_id.clone() {
                let payload = UpdateAdminRolePayload { capability_ids };
                match serde_json::to_string(&payload) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::put(&format!("/api/admin/roles/{role_id}")),
                            Some(body),
                            "Save role",
                        )
                        .await
                    }
                    Err(_) => Err("Role update could not be prepared.".into()),
                }
            } else {
                let name = role_name.get().trim().to_string();
                if name.is_empty() {
                    is_saving.set(false);
                    message.set(Some("Role name is required.".into()));
                    return;
                }
                let payload = CreateAdminRolePayload {
                    name,
                    capability_ids,
                };
                match serde_json::to_string(&payload) {
                    Ok(body) => {
                        send_json_id_request(
                            gloo_net::http::Request::post("/api/admin/roles"),
                            Some(body),
                            "Create role",
                        )
                        .await
                    }
                    Err(_) => Err("Role create request could not be prepared.".into()),
                }
            };

            match result {
                Ok(response) => {
                    let next_role_id = editing_id.unwrap_or(response.id);
                    sheet_open.set(false);
                    load_admin_roles_context(
                        roles,
                        capabilities,
                        selected_role_id,
                        is_saving,
                        message,
                        Some(next_role_id.clone()),
                    );
                    load_admin_role_detail(
                        next_role_id,
                        selected_role_detail,
                        detail_loading,
                        message,
                    );
                }
                Err(error) => {
                    message.set(Some(error));
                    is_saving.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    let _ = (
        editing_role_id,
        role_name,
        selected_capability_ids,
        is_saving,
        message,
        sheet_open,
        roles,
        capabilities,
        selected_role_id,
        selected_role_detail,
        detail_loading,
    );
}
