use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use semver::Version;
use tessara_dashboard_ui::{DashboardRouteBootstrap, SessionAccount};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationGrantOperationV1, BrowserLifecycleAssetV1,
    BrowserLifecycleBootstrapV1, ModuleDefinitionId, SemanticRouteName,
};
use uuid::Uuid;

use crate::{
    DashboardModuleError, DashboardModuleState, MANAGE_CAPABILITY, MODULE_RELEASE_VERSION,
    composition, dependencies, product, verified_shell_context,
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
    let dependency_health =
        dependencies::refresh_for_editor(&state, &headers, dashboard_id).await?;
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
            json_round_trip(dependency_health)?,
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

fn account_from_grant(grant: &tessara_module_contract::AuthorizationGrantV2) -> SessionAccount {
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
    if accepts_lifecycle_bootstrap(headers) {
        let destination = match &bootstrap {
            DashboardRouteBootstrap::Unavailable { .. } => "dashboards.directory",
            DashboardRouteBootstrap::Directory { .. } => "dashboards.directory",
            DashboardRouteBootstrap::Create { .. } => "dashboards.create",
            DashboardRouteBootstrap::Detail { .. } => "dashboards.detail",
            DashboardRouteBootstrap::Editor { .. } => "dashboards.edit",
            DashboardRouteBootstrap::Viewer { .. } => "dashboards.view",
        };
        let projection = BrowserLifecycleBootstrapV1 {
            schema_version: BrowserLifecycleBootstrapV1::SCHEMA_VERSION,
            definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
                .expect("static Dashboard definition id is valid"),
            release_version: Version::parse(MODULE_RELEASE_VERSION)
                .expect("static Dashboard release is valid"),
            lifecycle_abi: Version::new(1, 0, 0),
            destination: SemanticRouteName::new(destination)
                .expect("static Dashboard destination is valid"),
            path: path.to_string(),
            title: title.to_string(),
            document_state: context.document_state,
            entry_asset: lifecycle_asset(
                tessara_dashboard_ui::DASHBOARD_JS_SHA256,
                "dashboard.js",
                "text/javascript; charset=utf-8",
            ),
            stylesheet_assets: vec![lifecycle_asset(
                tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS_SHA256,
                "dashboard-lifecycle.css",
                "text/css; charset=utf-8",
            )],
            payload: serde_json::to_value(&bootstrap)
                .map_err(|error| DashboardModuleError::BadRequest(error.to_string()))?,
        };
        let mut response = Json(projection).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.tessara.module-view+json; version=1"),
        );
        no_store_vary(&mut response);
        return Ok(response);
    }
    let html = tessara_dashboard_ui::render_dashboard_document(
        &context,
        path,
        title,
        &bootstrap,
        MODULE_RELEASE_VERSION,
    );
    let mut response = Html(html).into_response();
    no_store_vary(&mut response);
    Ok(response)
}

fn accepts_lifecycle_bootstrap(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| {
            value.split(',').any(|media| {
                media
                    .trim()
                    .starts_with("application/vnd.tessara.module-view+json")
            })
        })
}

fn lifecycle_asset(digest: &str, name: &str, content_type: &str) -> BrowserLifecycleAssetV1 {
    BrowserLifecycleAssetV1 {
        url: tessara_dashboard_ui::dashboard_asset_path(MODULE_RELEASE_VERSION, digest, name),
        digest: ArtifactDigest::new(format!("sha256:{digest}"))
            .expect("compiled Dashboard asset digest is valid"),
        content_type: content_type.to_string(),
    }
}

fn no_store_vary(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Accept, Authorization"),
    );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_bootstrap_requires_the_versioned_accept_media_type() {
        let mut headers = HeaderMap::new();
        assert!(!accepts_lifecycle_bootstrap(&headers));
        headers.insert(
            header::ACCEPT,
            HeaderValue::from_static(
                "text/html, application/vnd.tessara.module-view+json; version=1",
            ),
        );
        assert!(accepts_lifecycle_bootstrap(&headers));
    }

    #[test]
    fn projected_lifecycle_assets_are_release_and_digest_addressed() {
        let asset = lifecycle_asset(
            tessara_dashboard_ui::DASHBOARD_LIFECYCLE_CSS_SHA256,
            "dashboard-lifecycle.css",
            "text/css; charset=utf-8",
        );
        assert!(asset.url.contains("/tessara.dashboards/2.1.0/"));
        assert!(asset.url.contains(asset.digest.as_str()));
    }
}
