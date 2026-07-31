use axum::{
    Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use tessara_dashboard_ui::{DashboardRouteBootstrap, SessionAccount};
use tessara_module_contract::AuthorizationGrantOperationV1;
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY, MODULE_RELEASE_VERSION,
    composition, product, verified_shell_context,
};

pub(super) fn routes() -> Router<DashboardModuleState> {
    Router::new()
        .route("/dashboards", get(directory))
        .route("/dashboards/new", get(create))
        .route("/dashboards/{dashboard_id}/edit", get(editor))
        .route("/dashboards/{dashboard_id}/view", get(viewer))
        .route("/dashboards/{dashboard_id}", get(detail))
}

async fn directory(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
) -> Result<Response, DashboardModuleError> {
    let account = account_for(&state, &headers, "dashboards.list").await?;
    let dashboards = product::list_dashboards(State(state.clone()), headers.clone())
        .await?
        .0;
    document(
        &state,
        &headers,
        "/dashboards",
        "Dashboards",
        DashboardRouteBootstrap::directory(account, dashboards_to_ui(dashboards)?),
    )
    .await
}

async fn create(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
) -> Result<Response, DashboardModuleError> {
    let account = account_for(&state, &headers, "dashboards.list_manageable").await?;
    let nodes = composition::list_visibility_nodes(State(state.clone()), headers.clone())
        .await?
        .0;
    document(
        &state,
        &headers,
        "/dashboards/new",
        "Create Dashboard",
        DashboardRouteBootstrap::create(account, json_round_trip(nodes)?),
    )
    .await
}

async fn detail(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Response, DashboardModuleError> {
    read_document(state, headers, dashboard_id, false).await
}

async fn viewer(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Response, DashboardModuleError> {
    read_document(state, headers, dashboard_id, true).await
}

async fn read_document(
    state: DashboardModuleState,
    headers: HeaderMap,
    dashboard_id: Uuid,
    viewer: bool,
) -> Result<Response, DashboardModuleError> {
    let account = account_for(&state, &headers, "dashboards.get").await?;
    let dashboard =
        composition::get_dashboard(State(state.clone()), headers.clone(), Path(dashboard_id))
            .await?
            .0;
    let path = if viewer {
        format!("/dashboards/{dashboard_id}/view")
    } else {
        format!("/dashboards/{dashboard_id}")
    };
    let bootstrap = if viewer {
        DashboardRouteBootstrap::viewer(account, json_round_trip(dashboard)?)
    } else {
        DashboardRouteBootstrap::detail(account, json_round_trip(dashboard)?)
    };
    document(
        &state,
        &headers,
        &path,
        if viewer {
            "Dashboard Viewer"
        } else {
            "Dashboard Detail"
        },
        bootstrap,
    )
    .await
}

async fn editor(
    State(state): State<DashboardModuleState>,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Response, DashboardModuleError> {
    let grant = product::authorize(
        &state,
        &headers,
        "dashboards.load_composition",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let account = account_from_grant(&grant.payload);
    let scope = product::authorized_organizations(&grant.payload, MANAGE_CAPABILITY);
    let composition =
        composition::get_composition(State(state.clone()), headers.clone(), Path(dashboard_id))
            .await?
            .0;
    let nodes = composition::load_visibility_nodes_for_scope(&state, scope).await?;
    let path = format!("/dashboards/{dashboard_id}/edit");
    document(
        &state,
        &headers,
        &path,
        "Edit Dashboard",
        DashboardRouteBootstrap::editor(
            account,
            json_round_trip(composition)?,
            json_round_trip(nodes)?,
        ),
    )
    .await
}

async fn account_for(
    state: &DashboardModuleState,
    headers: &HeaderMap,
    action: &str,
) -> Result<SessionAccount, DashboardModuleError> {
    let grant =
        product::authorize(state, headers, action, AuthorizationGrantOperationV1::Read).await?;
    Ok(account_from_grant(&grant.payload))
}

fn account_from_grant(grant: &tessara_module_contract::AuthorizationGrantV1) -> SessionAccount {
    let mut capabilities = grant
        .capability_scope_bindings
        .iter()
        .map(|binding| binding.capability.as_str().to_string())
        .collect::<Vec<_>>();
    capabilities.sort();
    capabilities.dedup();
    SessionAccount { capabilities }
}

async fn document(
    state: &DashboardModuleState,
    headers: &HeaderMap,
    path: &str,
    title: &str,
    bootstrap: DashboardRouteBootstrap,
) -> Result<Response, DashboardModuleError> {
    let context = verified_shell_context(state, headers).await?;
    let html = tessara_dashboard_ui::render_dashboard_document(
        &context,
        path,
        title,
        &bootstrap,
        MODULE_RELEASE_VERSION,
    );
    let mut response = Html(html).into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("Authorization"));
    Ok(response)
}

fn dashboards_to_ui<T: serde::Serialize>(
    dashboards: Vec<T>,
) -> Result<Vec<tessara_dashboard_ui::DashboardSummary>, DashboardModuleError> {
    json_round_trip(dashboards)
}

fn json_round_trip<T, U>(value: T) -> Result<U, DashboardModuleError>
where
    T: serde::Serialize,
    U: serde::de::DeserializeOwned,
{
    serde_json::from_value(
        serde_json::to_value(value)
            .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?,
    )
    .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))
}
