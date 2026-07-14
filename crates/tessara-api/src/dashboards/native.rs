//! Authenticated native Dashboard route adapters.
//!
//! These handlers authenticate once, invoke the same Dashboard service
//! projections as the JSON endpoints, and (in SSR builds) adapt only those
//! authorization-filtered projections into web-owned request bootstrap DTOs.

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
use uuid::Uuid;

use crate::{
    auth::{self, AccountContext},
    db::AppState,
    error::ApiError,
};

use super::service;

pub(crate) async fn directory(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match dashboard_account(&state, &headers, "dashboards:read").await {
        Ok(account) => account,
        Err(response) => return response,
    };
    let dashboards = match service::list_dashboards(&state.pool, &account).await {
        Ok(dashboards) => dashboards,
        Err(error) => return error.into_response(),
    };

    #[cfg(feature = "ssr")]
    {
        let bootstrap = tessara_web::DashboardRouteBootstrap::directory(
            web_account(&account),
            dashboards.into_iter().map(web_summary).collect(),
        );
        dashboard_document(
            "/dashboards",
            "Tessara Dashboards",
            "Browse Tessara dashboards.",
            &bootstrap,
        )
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = dashboards;
        crate::native_app(
            "/dashboards",
            "Tessara Dashboards",
            "Browse Tessara dashboards.",
        )
        .into_response()
    }
}

pub(crate) async fn create(State(state): State<AppState>, headers: HeaderMap) -> Response {
    let account = match dashboard_account(&state, &headers, "dashboards:manage").await {
        Ok(account) => account,
        Err(response) => return response,
    };

    #[cfg(feature = "ssr")]
    {
        let visibility_nodes = match load_visibility_node_options(&state, &account).await {
            Ok(nodes) => nodes,
            Err(error) => return error.into_response(),
        };
        let bootstrap =
            tessara_web::DashboardRouteBootstrap::create(web_account(&account), visibility_nodes);
        dashboard_document(
            "/dashboards/new",
            "Create Dashboard",
            "Create a Tessara dashboard.",
            &bootstrap,
        )
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = account;
        crate::native_app(
            "/dashboards/new",
            "Create Dashboard",
            "Create a Tessara dashboard.",
        )
        .into_response()
    }
}

pub(crate) async fn detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Response {
    readable_dashboard_document(
        &state,
        &headers,
        dashboard_id,
        NativeDashboardSurface::Detail,
    )
    .await
}

pub(crate) async fn viewer(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Response {
    readable_dashboard_document(
        &state,
        &headers,
        dashboard_id,
        NativeDashboardSurface::Viewer,
    )
    .await
}

pub(crate) async fn editor(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Response {
    let account = match dashboard_account(&state, &headers, "dashboards:manage").await {
        Ok(account) => account,
        Err(response) => return response,
    };
    let composition = match service::load_composition(&state.pool, &account, dashboard_id).await {
        Ok(composition) => composition,
        Err(error) => return error.into_response(),
    };
    let path = format!("/dashboards/{dashboard_id}/edit");

    #[cfg(feature = "ssr")]
    {
        let visibility_nodes = match load_visibility_node_options(&state, &account).await {
            Ok(nodes) => nodes,
            Err(error) => return error.into_response(),
        };
        let bootstrap = tessara_web::DashboardRouteBootstrap::editor(
            web_account(&account),
            web_composition(composition),
            visibility_nodes,
        );
        dashboard_document(
            &path,
            "Edit Dashboard",
            "Edit a Tessara dashboard.",
            &bootstrap,
        )
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = composition;
        crate::native_app(&path, "Edit Dashboard", "Edit a Tessara dashboard.").into_response()
    }
}

#[derive(Clone, Copy)]
enum NativeDashboardSurface {
    Detail,
    Viewer,
}

async fn readable_dashboard_document(
    state: &AppState,
    headers: &HeaderMap,
    dashboard_id: Uuid,
    surface: NativeDashboardSurface,
) -> Response {
    let account = match dashboard_account(state, headers, "dashboards:read").await {
        Ok(account) => account,
        Err(response) => return response,
    };
    let dashboard = match service::get_dashboard(&state.pool, &account, dashboard_id).await {
        Ok(dashboard) => dashboard,
        Err(error) => return error.into_response(),
    };
    let (path, title, description) = match surface {
        NativeDashboardSurface::Detail => (
            format!("/dashboards/{dashboard_id}"),
            "Dashboard Detail",
            "Inspect a Tessara dashboard.",
        ),
        NativeDashboardSurface::Viewer => (
            format!("/dashboards/{dashboard_id}/view"),
            "Dashboard Viewer",
            "View a Tessara dashboard.",
        ),
    };

    #[cfg(feature = "ssr")]
    {
        let dashboard = web_dashboard(dashboard);
        let bootstrap = match surface {
            NativeDashboardSurface::Detail => {
                tessara_web::DashboardRouteBootstrap::detail(web_account(&account), dashboard)
            }
            NativeDashboardSurface::Viewer => {
                tessara_web::DashboardRouteBootstrap::viewer(web_account(&account), dashboard)
            }
        };
        dashboard_document(&path, title, description, &bootstrap)
    }

    #[cfg(not(feature = "ssr"))]
    {
        let _ = dashboard;
        crate::native_app(&path, title, description).into_response()
    }
}

async fn dashboard_account(
    state: &AppState,
    headers: &HeaderMap,
    required_capability: &str,
) -> Result<AccountContext, Response> {
    let (account, _) = match auth::authenticate_request(&state.pool, &state.config, headers).await {
        Ok(authenticated) => authenticated,
        Err(
            ApiError::Unauthorized
            | ApiError::SessionExpired
            | ApiError::SessionRevoked
            | ApiError::InvalidCredentials,
        ) => return Err(Redirect::to("/login").into_response()),
        Err(error) => return Err(error.into_response()),
    };
    auth::ensure_capability(&account, required_capability).map_err(IntoResponse::into_response)?;
    Ok(account)
}

#[cfg(feature = "ssr")]
fn dashboard_document(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: &tessara_web::DashboardRouteBootstrap,
) -> Response {
    let mut response = axum::response::Html(
        tessara_web::application_html_with_dashboard_bootstrap(path, title, description, bootstrap),
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

#[cfg(feature = "ssr")]
async fn load_visibility_node_options(
    state: &AppState,
    account: &AccountContext,
) -> super::error::DashboardResult<Vec<tessara_web::VisibilityNodeOption>> {
    service::list_visibility_node_options(&state.pool, account)
        .await
        .map(|nodes| {
            nodes
                .into_iter()
                .map(|node| tessara_web::VisibilityNodeOption {
                    id: node.id.to_string(),
                    node_type_name: node.node_type_name,
                    parent_node_name: node.parent_node_name,
                    name: node.name,
                })
                .collect()
        })
}

#[cfg(feature = "ssr")]
fn web_account(account: &AccountContext) -> tessara_web::SessionAccount {
    tessara_web::SessionAccount {
        capabilities: account
            .capabilities
            .iter()
            .filter(|capability| {
                matches!(
                    capability.as_str(),
                    "admin:all" | "dashboards:read" | "dashboards:manage"
                )
            })
            .cloned()
            .collect(),
    }
}

#[cfg(feature = "ssr")]
fn web_summary(summary: super::dto::DashboardSummary) -> tessara_web::DashboardSummary {
    tessara_web::DashboardSummary {
        id: summary.id.to_string(),
        name: summary.name,
        description: summary.description,
        visibility_nodes: summary
            .visibility_nodes
            .into_iter()
            .map(web_visibility_node)
            .collect(),
        placement_count: summary.placement_count,
        can_manage: summary.can_manage,
    }
}

#[cfg(feature = "ssr")]
fn web_dashboard(dashboard: super::dto::DashboardResponse) -> tessara_web::Dashboard {
    tessara_web::Dashboard {
        id: dashboard.id.to_string(),
        name: dashboard.name,
        description: dashboard.description,
        visibility_nodes: dashboard
            .visibility_nodes
            .into_iter()
            .map(web_visibility_node)
            .collect(),
        placement_count: dashboard.placement_count,
        can_manage: dashboard.can_manage,
        placements: dashboard
            .placements
            .into_iter()
            .map(web_placement)
            .collect(),
    }
}

#[cfg(feature = "ssr")]
fn web_composition(
    composition: super::dto::DashboardCompositionResponse,
) -> tessara_web::DashboardComposition {
    tessara_web::DashboardComposition {
        dashboard: web_dashboard(composition.dashboard),
        available_component_versions: composition
            .available_component_versions
            .into_iter()
            .map(web_component_option)
            .collect(),
        new_placement_ids: composition
            .new_placement_ids
            .into_iter()
            .map(|mapping| tessara_web::DashboardPlacementIdMapping {
                client_key: mapping.client_key,
                placement_id: mapping.placement_id.to_string(),
            })
            .collect(),
    }
}

#[cfg(feature = "ssr")]
fn web_visibility_node(
    node: super::dto::DashboardVisibilityNodeSummary,
) -> tessara_web::DashboardVisibilityNode {
    tessara_web::DashboardVisibilityNode {
        node_id: node.node_id.to_string(),
        node_name: node.node_name,
        node_type_name: node.node_type_name,
        parent_node_id: node.parent_node_id.map(|id| id.to_string()),
        node_path: node.node_path,
    }
}

#[cfg(feature = "ssr")]
fn web_placement(
    placement: super::dto::DashboardPlacementResponse,
) -> tessara_web::DashboardPlacement {
    tessara_web::DashboardPlacement {
        placement_id: placement.placement_id.to_string(),
        position: placement.position,
        grid_row: placement.grid_row,
        grid_column: placement.grid_column,
        grid_width: placement.grid_width,
        grid_height: placement.grid_height,
        availability: match placement.availability {
            super::dto::DashboardPlacementAvailability::Available => {
                tessara_web::DashboardPlacementAvailability::Available
            }
            super::dto::DashboardPlacementAvailability::Unavailable => {
                tessara_web::DashboardPlacementAvailability::Unavailable
            }
        },
        config_state: placement.config_state,
        title: placement.title,
        component: placement.component.map(web_component),
        allowed_operations: placement.allowed_operations,
    }
}

#[cfg(feature = "ssr")]
fn web_component(
    component: super::dto::DashboardComponentVersionSummary,
) -> tessara_web::DashboardComponentVersion {
    tessara_web::DashboardComponentVersion {
        component_version_id: component.component_version_id.to_string(),
        component_id: component.component_id.to_string(),
        component_name: component.component_name,
        component_slug: component.component_slug,
        component_type: component.component_type,
        version_number: component.version_number,
        version_label: component.version_label,
        version_status: component.version_status,
    }
}

#[cfg(feature = "ssr")]
fn web_component_option(
    component: super::dto::DashboardComponentVersionOption,
) -> tessara_web::DashboardComponentVersionOption {
    tessara_web::DashboardComponentVersionOption {
        component_version_id: component.component_version_id.to_string(),
        component_id: component.component_id.to_string(),
        component_name: component.component_name,
        component_slug: component.component_slug,
        component_type: component.component_type,
        version_number: component.version_number,
        version_label: component.version_label,
        version_status: component.version_status,
        default_grid_width: i32::from(component.default_grid_width),
        default_grid_height: i32::from(component.default_grid_height),
    }
}
