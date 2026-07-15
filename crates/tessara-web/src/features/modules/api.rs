//! Browser orchestration for Module Management.

#![cfg_attr(
    not(all(feature = "hydrate", target_arch = "wasm32")),
    allow(dead_code)
)]

use super::bootstrap::NavigationPolicyBootstrapV1;
#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
use super::models::UpdateNavigationPolicyRequestV1;
use super::models::{
    ModuleDetailResponseV1, ModuleInventoryResponseV1, ModuleManagementAccessV1,
    NavigationPolicyResponseV1,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModuleManagementClientError {
    Authentication,
    Restricted,
    NotFound,
    Unavailable(String),
    Failed(String),
}

impl ModuleManagementClientError {
    pub fn display_message(&self) -> String {
        match self {
            Self::Authentication => "Authentication is required.".into(),
            Self::Restricted => {
                "Global Module Management read access is required for this surface.".into()
            }
            Self::NotFound => "The requested module definition was not found.".into(),
            Self::Unavailable(message) | Self::Failed(message) => message.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDirectoryClientPayload {
    pub access: ModuleManagementAccessV1,
    pub inventory: ModuleInventoryResponseV1,
    pub navigation_policy: NavigationPolicyBootstrapV1,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ModuleDetailClientPayload {
    pub access: ModuleManagementAccessV1,
    pub detail: ModuleDetailResponseV1,
    pub navigation_policy: NavigationPolicyBootstrapV1,
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
fn classify(error: tessara_web_http::RequestError) -> ModuleManagementClientError {
    match error.status() {
        Some(401) => ModuleManagementClientError::Authentication,
        Some(403) => ModuleManagementClientError::Restricted,
        Some(404) => ModuleManagementClientError::NotFound,
        _ if error.is_retryable() => ModuleManagementClientError::Unavailable(error.into_message()),
        _ => ModuleManagementClientError::Failed(error.into_message()),
    }
}

#[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
async fn ensure_authenticated() -> Result<(), ModuleManagementClientError> {
    let session = tessara_web_http::fetch_json::<crate::features::auth::SessionStateResponse>(
        "/api/auth/session",
        "Session",
    )
    .await
    .map_err(classify)?;
    if !session.authenticated {
        return Err(ModuleManagementClientError::Authentication);
    }
    Ok(())
}

/// Loads one directory projection after client-side route navigation.
pub async fn fetch_module_directory()
-> Result<ModuleDirectoryClientPayload, ModuleManagementClientError> {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        ensure_authenticated().await?;
        let inventory = tessara_web_http::fetch_json::<ModuleInventoryResponseV1>(
            "/api/admin/modules",
            "Module inventory",
        )
        .await
        .map_err(classify)?;
        // Successful guarded reads prove global read. Only the policy API's
        // authoritative scope-aware flag may enable mutation controls.
        let (access, navigation_policy) =
            match tessara_web_http::fetch_json::<NavigationPolicyResponseV1>(
                "/api/admin/navigation-policy",
                "Navigation policy",
            )
            .await
            {
                Ok(policy) => {
                    let access = if policy.can_manage_navigation {
                        ModuleManagementAccessV1::manager()
                    } else {
                        ModuleManagementAccessV1::read_only()
                    };
                    (access, NavigationPolicyBootstrapV1::ready(policy))
                }
                Err(error) if error.status() == Some(401) => {
                    return Err(ModuleManagementClientError::Authentication);
                }
                Err(error) => (
                    ModuleManagementAccessV1::read_only(),
                    NavigationPolicyBootstrapV1::unavailable(error.into_message()),
                ),
            };
        return Ok(ModuleDirectoryClientPayload {
            access,
            inventory,
            navigation_policy,
        });
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    Err(ModuleManagementClientError::Unavailable(
        "Module inventory was not supplied for this server-rendered request.".into(),
    ))
}

/// Loads one authorized detail projection after client-side route navigation.
pub async fn fetch_module_detail(
    definition_id: &str,
) -> Result<ModuleDetailClientPayload, ModuleManagementClientError> {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        ensure_authenticated().await?;
        let detail = tessara_web_http::fetch_json::<ModuleDetailResponseV1>(
            // Definition ids are contract-validated portable identifiers and
            // therefore contain no path separators or query delimiters.
            &format!("/api/admin/modules/{definition_id}"),
            "Module detail",
        )
        .await
        .map_err(classify)?;
        let (access, navigation_policy) =
            match tessara_web_http::fetch_json::<NavigationPolicyResponseV1>(
                "/api/admin/navigation-policy",
                "Navigation policy",
            )
            .await
            {
                Ok(policy) => {
                    let access = if policy.can_manage_navigation {
                        ModuleManagementAccessV1::manager()
                    } else {
                        ModuleManagementAccessV1::read_only()
                    };
                    (access, NavigationPolicyBootstrapV1::ready(policy))
                }
                Err(error) if error.status() == Some(401) => {
                    return Err(ModuleManagementClientError::Authentication);
                }
                Err(error) => (
                    ModuleManagementAccessV1::read_only(),
                    NavigationPolicyBootstrapV1::unavailable(error.into_message()),
                ),
            };
        return Ok(ModuleDetailClientPayload {
            access,
            detail,
            navigation_policy,
        });
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    {
        let _ = definition_id;
        Err(ModuleManagementClientError::Unavailable(
            "Module detail was not supplied for this server-rendered request.".into(),
        ))
    }
}

/// Replaces the complete mutable contribution policy using optimistic revision
/// control. Core destinations never enter the request projection.
pub async fn put_navigation_policy(
    policy: &NavigationPolicyResponseV1,
) -> Result<NavigationPolicyResponseV1, ModuleManagementClientError> {
    #[cfg(all(feature = "hydrate", target_arch = "wasm32"))]
    {
        let request = UpdateNavigationPolicyRequestV1::from(policy);
        return tessara_web_http::send_json(
            gloo_net::http::Request::put("/api/admin/navigation-policy"),
            &request,
            "Navigation policy update",
        )
        .await
        .map_err(classify);
    }
    #[cfg(not(all(feature = "hydrate", target_arch = "wasm32")))]
    {
        let _ = policy;
        Err(ModuleManagementClientError::Unavailable(
            "Navigation policy changes require an interactive browser session.".into(),
        ))
    }
}
