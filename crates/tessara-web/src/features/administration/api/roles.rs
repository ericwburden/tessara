//! Role-management API loaders and save orchestration.

use crate::features::administration::models::{AdminCapabilitySummary, AdminRoleDetail};
#[cfg(feature = "hydrate")]
use crate::features::administration::models::{CreateAdminRolePayload, UpdateAdminRolePayload};
use crate::features::organization::AdminRoleSummary;
#[cfg(feature = "hydrate")]
use crate::http::{redirect_to_login, send_json_id_request};
use leptos::prelude::*;

/// Loads the administration roles list and capability catalog.
pub(crate) fn load_admin_roles_context(
    roles: RwSignal<Vec<AdminRoleSummary>>,
    capabilities: RwSignal<Vec<AdminCapabilitySummary>>,
    selected_role_id: RwSignal<Option<String>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
    preferred_role_id: Option<String>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            let roles_response = gloo_net::http::Request::get("/api/admin/roles")
                .send()
                .await;
            let capabilities_response = gloo_net::http::Request::get("/api/admin/capabilities")
                .send()
                .await;

            match (roles_response, capabilities_response) {
                (Ok(roles_response), _) if roles_response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(roles_response), Ok(capabilities_response))
                    if roles_response.ok() && capabilities_response.ok() =>
                {
                    let loaded_roles = roles_response.json::<Vec<AdminRoleSummary>>().await;
                    let loaded_capabilities = capabilities_response
                        .json::<Vec<AdminCapabilitySummary>>()
                        .await;
                    match (loaded_roles, loaded_capabilities) {
                        (Ok(role_items), Ok(capability_items)) => {
                            let selected = preferred_role_id
                                .or_else(|| {
                                    selected_role_id
                                        .get_untracked()
                                        .filter(|id| role_items.iter().any(|role| role.id == *id))
                                })
                                .or_else(|| role_items.first().map(|role| role.id.clone()));
                            roles.set(role_items);
                            capabilities.set(capability_items);
                            selected_role_id.set(selected);
                            message.set(None);
                        }
                        (Err(error), _) => {
                            roles.set(Vec::new());
                            message.set(Some(format!("Unable to parse roles: {error}")));
                        }
                        (_, Err(error)) => {
                            capabilities.set(Vec::new());
                            message.set(Some(format!("Unable to parse capabilities: {error}")));
                        }
                    }
                    is_loading.set(false);
                }
                (Ok(roles_response), _) if !roles_response.ok() => {
                    roles.set(Vec::new());
                    message.set(Some(format!(
                        "Unable to load roles. Server returned {}.",
                        roles_response.status()
                    )));
                    is_loading.set(false);
                }
                (_, Ok(capabilities_response)) if !capabilities_response.ok() => {
                    capabilities.set(Vec::new());
                    message.set(Some(format!(
                        "Unable to load capabilities. Server returned {}.",
                        capabilities_response.status()
                    )));
                    is_loading.set(false);
                }
                (Err(error), _) => {
                    roles.set(Vec::new());
                    message.set(Some(format!("Unable to load roles: {error}")));
                    is_loading.set(false);
                }
                (_, Err(error)) => {
                    capabilities.set(Vec::new());
                    message.set(Some(format!("Unable to load capabilities: {error}")));
                    is_loading.set(false);
                }
                _ => {
                    message.set(Some("Unable to load role context.".into()));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            roles,
            capabilities,
            selected_role_id,
            message,
            preferred_role_id,
        );
        is_loading.set(false);
    }
}

/// Loads the selected administration role detail.
pub(crate) fn load_admin_role_detail(
    role_id: String,
    detail: RwSignal<Option<AdminRoleDetail>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            match gloo_net::http::Request::get(&format!("/api/admin/roles/{role_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<AdminRoleDetail>().await {
                    Ok(loaded_detail) => {
                        detail.set(Some(loaded_detail));
                        message.set(None);
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        message.set(Some(format!("Unable to parse role detail: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    message.set(Some(format!(
                        "Unable to load role detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    message.set(Some(format!("Unable to load role detail: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }
    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (role_id, detail, message);
        is_loading.set(false);
    }
}

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
