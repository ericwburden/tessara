//! Role-management API loaders.

use crate::features::administration::models::{AdminCapabilitySummary, AdminRoleDetail};
use crate::features::organization::AdminRoleSummary;
#[cfg(feature = "hydrate")]
use crate::http::redirect_to_login;
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
