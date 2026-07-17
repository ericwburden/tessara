//! Authenticated native Module Management document adapters.
//!
//! The handlers authenticate before lookup, derive installation-global access
//! from the full account scope model, and provide the same versioned projection
//! used by the JSON APIs to server rendering and hydration.

#[cfg(feature = "ssr")]
use axum::http::{
    HeaderValue,
    header::{CACHE_CONTROL, VARY},
};
use axum::{
    extract::{Path, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};

use crate::{
    auth::{self, AccountContext},
    db::AppState,
    error::ApiError,
};

#[cfg(feature = "ssr")]
use super::{
    dto::{MODULE_HTTP_SCHEMA_VERSION_V1, ModuleDetailResponseV1},
    routes::{inventory_response, navigation_policy_response_v2},
    service::{self, CatalogReadError},
    shell_navigation,
};

pub(crate) async fn directory(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match module_account(&state, &headers).await {
        Ok(account) => account,
        Err(response) => return response,
    };

    #[cfg(feature = "ssr")]
    {
        let bootstrap = if !account.has_global_capability("modules:read") {
            tessara_web::ModuleManagementRouteBootstrapV1::restricted(
                tessara_web::ModuleManagementSurfaceV1::Directory,
            )
        } else {
            directory_bootstrap(&state, &account).await
        };
        let shell_navigation = shell_navigation_bootstrap(&state, &account).await;
        module_document(
            "/administration/modules",
            "Tessara Module Management",
            "Inspect module inventory and transition contribution metadata.",
            &bootstrap,
            &shell_navigation,
        )
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = account;
        crate::native_app(
            "/administration/modules",
            "Tessara Module Management",
            "Inspect module inventory and transition contribution metadata.",
        )
        .into_response()
    }
}

pub(crate) async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(definition_id): Path<String>,
) -> Response {
    let account = match module_account(&state, &headers).await {
        Ok(account) => account,
        Err(response) => return response,
    };
    let path = format!("/administration/modules/{definition_id}");

    #[cfg(feature = "ssr")]
    {
        let bootstrap = if !account.has_global_capability("modules:read") {
            tessara_web::ModuleManagementRouteBootstrapV1::restricted(
                tessara_web::ModuleManagementSurfaceV1::Detail,
            )
        } else {
            detail_bootstrap(&state, &account, &definition_id).await
        };
        let shell_navigation = shell_navigation_bootstrap(&state, &account).await;
        module_document(
            &path,
            "Tessara Module Detail",
            "Inspect one transition contribution descriptor.",
            &bootstrap,
            &shell_navigation,
        )
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = account;
        crate::native_app(
            &path,
            "Tessara Module Detail",
            "Inspect one transition contribution descriptor.",
        )
        .into_response()
    }
}

async fn module_account(state: &AppState, headers: &HeaderMap) -> Result<AccountContext, Response> {
    match auth::authenticate_request(&state.pool, &state.config, headers).await {
        Ok((account, _)) => Ok(account),
        Err(
            ApiError::Unauthorized
            | ApiError::SessionExpired
            | ApiError::SessionRevoked
            | ApiError::InvalidCredentials,
        ) => Err(Redirect::to("/login").into_response()),
        Err(error) => Err(error.into_response()),
    }
}

#[cfg(feature = "ssr")]
async fn directory_bootstrap(
    state: &AppState,
    account: &AccountContext,
) -> tessara_web::ModuleManagementRouteBootstrapV1 {
    let inventory = match service::load_module_inventory(&state.pool).await {
        Ok(inventory) => inventory_response(inventory),
        Err(error) => {
            return unavailable_bootstrap(tessara_web::ModuleManagementSurfaceV1::Directory, error);
        }
    };
    let inventory = match web_projection(inventory) {
        Ok(inventory) => inventory,
        Err(message) => {
            return tessara_web::ModuleManagementRouteBootstrapV1::unavailable(
                tessara_web::ModuleManagementSurfaceV1::Directory,
                message,
            );
        }
    };

    tessara_web::ModuleManagementRouteBootstrapV1::directory(
        web_access(account),
        inventory,
        navigation_policy_bootstrap(state, account).await,
    )
}

#[cfg(feature = "ssr")]
async fn detail_bootstrap(
    state: &AppState,
    account: &AccountContext,
    definition_id: &str,
) -> tessara_web::ModuleManagementRouteBootstrapV1 {
    let inventory = match service::load_module_inventory(&state.pool).await {
        Ok(inventory) => inventory,
        Err(error) => {
            return unavailable_bootstrap(tessara_web::ModuleManagementSurfaceV1::Detail, error);
        }
    };
    let installation_id = inventory.installation_id;
    let Some(entry) = inventory
        .transitions
        .into_iter()
        .find(|entry| entry.definition_id == definition_id)
    else {
        return tessara_web::ModuleManagementRouteBootstrapV1::not_found(definition_id);
    };
    let detail = ModuleDetailResponseV1 {
        schema_version: MODULE_HTTP_SCHEMA_VERSION_V1,
        installation_id,
        entry: entry.normalized_projection,
    };
    let detail = match web_projection(detail) {
        Ok(detail) => detail,
        Err(message) => {
            return tessara_web::ModuleManagementRouteBootstrapV1::unavailable(
                tessara_web::ModuleManagementSurfaceV1::Detail,
                message,
            );
        }
    };

    tessara_web::ModuleManagementRouteBootstrapV1::detail(
        web_access(account),
        detail,
        navigation_policy_bootstrap(state, account).await,
    )
}

#[cfg(feature = "ssr")]
async fn navigation_policy_bootstrap(
    state: &AppState,
    account: &AccountContext,
) -> tessara_web::NavigationPolicyBootstrapV1 {
    let policy = match service::load_navigation_policy_v2(&state.pool).await {
        Ok(policy) => navigation_policy_response_v2(
            policy,
            account.has_global_capability("modules:manage_navigation"),
        ),
        Err(_) => {
            return tessara_web::NavigationPolicyBootstrapV1::unavailable(
                "Navigation policy is temporarily unavailable.",
            );
        }
    };
    match web_projection(policy) {
        Ok(policy) => tessara_web::NavigationPolicyBootstrapV1::ready(policy),
        Err(message) => tessara_web::NavigationPolicyBootstrapV1::unavailable(message),
    }
}

#[cfg(feature = "ssr")]
fn web_access(account: &AccountContext) -> tessara_web::ModuleManagementAccessV1 {
    if account.has_global_capability("modules:manage_navigation") {
        tessara_web::ModuleManagementAccessV1::manager()
    } else if account.has_global_capability("modules:read") {
        tessara_web::ModuleManagementAccessV1::read_only()
    } else {
        tessara_web::ModuleManagementAccessV1::restricted()
    }
}

#[cfg(feature = "ssr")]
async fn shell_navigation_bootstrap(
    state: &AppState,
    account: &AccountContext,
) -> tessara_web::ShellNavigationResponseV1 {
    web_projection(shell_navigation::load_response(state, account).await)
        .ok()
        .filter(tessara_web::ShellNavigationResponseV1::is_supported)
        .unwrap_or_else(fail_closed_shell_navigation)
}

#[cfg(feature = "ssr")]
fn fail_closed_shell_navigation() -> tessara_web::ShellNavigationResponseV1 {
    tessara_web::ShellNavigationResponseV1 {
        schema_version: 2,
        policy_revision: None,
        state: tessara_web::ShellNavigationStateV1::Unavailable,
        groups: vec![tessara_web::ShellNavigationGroupV1 {
            id: "core.main".to_string(),
            name: "Main".to_string(),
            items: vec![tessara_web::ShellNavigationItemV1 {
                key: "home".to_string(),
                label: "Home".to_string(),
                href: "/".to_string(),
                owner: tessara_web::ShellNavigationItemOwnerV1::Core,
                contribution_id: None,
            }],
        }],
        unavailable: Some(tessara_web::ShellNavigationUnavailableV1 {
            code: "shell_navigation_unavailable".to_string(),
            message: "Contribution navigation is temporarily unavailable.".to_string(),
        }),
    }
}

#[cfg(feature = "ssr")]
fn web_projection<T, W>(projection: T) -> Result<W, String>
where
    T: serde::Serialize,
    W: serde::de::DeserializeOwned,
{
    serde_json::to_value(projection)
        .map_err(|_| "The Module Management projection could not be serialized.".to_string())
        .and_then(|value| {
            serde_json::from_value(value).map_err(|_| {
                "The Module Management projection does not match the web contract.".to_string()
            })
        })
}

#[cfg(feature = "ssr")]
fn unavailable_bootstrap(
    surface: tessara_web::ModuleManagementSurfaceV1,
    error: CatalogReadError,
) -> tessara_web::ModuleManagementRouteBootstrapV1 {
    tracing::error!(code = error.stable_code(), error = ?error, "native module projection failed");
    tessara_web::ModuleManagementRouteBootstrapV1::unavailable(
        surface,
        "Module Management is temporarily unavailable.",
    )
}

#[cfg(feature = "ssr")]
fn module_document(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: &tessara_web::ModuleManagementRouteBootstrapV1,
    shell_navigation: &tessara_web::ShellNavigationResponseV1,
) -> Response {
    let mut response = axum::response::Html(
        tessara_web::application_html_with_module_management_and_shell_navigation_bootstrap(
            path,
            title,
            description,
            bootstrap,
            shell_navigation,
        ),
    )
    .into_response();
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
    response
        .headers_mut()
        .insert(VARY, HeaderValue::from_static("Cookie, Authorization"));
    response
}
