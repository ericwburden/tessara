//! Same-origin Core gateway for the independently deployed Dashboard module.
//!
//! Browser cookies terminate in Core. Dashboard receives only short-lived,
//! action-bound grants plus private revision/Organization projections.

use axum::{
    Router,
    body::{Body, Bytes},
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::{get, post},
};
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Duration, Utc};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sqlx::{PgPool, Row};
use tessara_module_contract::{
    AuthorizationGrantOperationV1, AuthorizationGrantV1, AuthorizationValidationContextV1,
    DependencyBindingKey, FunctionalContractId, ModuleDefinitionId, ProtocolSignaturePurposeV1,
    SignedEnvelopeV1,
};
use uuid::Uuid;

use crate::{
    auth::AuthenticatedRequest,
    core_security::{capability_bindings, protocol_signer},
    db::AppState,
    error::{ApiError, ApiResult},
};

const DEFINITION_ID: &str = "tessara.dashboards";
const DEPENDENCY_BINDING: &str = "tessara.core.dashboards";
const CONTRACT_ID: &str = "tessara.dashboards.dashboard";

pub(crate) fn routes() -> Router<AppState> {
    Router::new()
        .route("/dashboards", get(directory_page))
        .route("/dashboards/new", get(create_page))
        .route("/dashboards/{dashboard_id}", get(detail_page))
        .route("/dashboards/{dashboard_id}/edit", get(editor_page))
        .route("/dashboards/{dashboard_id}/view", get(viewer_page))
        .route(
            "/api/dashboards",
            get(|state, request| {
                proxy(
                    state,
                    request,
                    "dashboards.list",
                    AuthorizationGrantOperationV1::Read,
                    reqwest::Method::GET,
                    "api/dashboards".into(),
                    None,
                )
            }),
        )
        .route(
            "/api/dashboards/{dashboard_id}",
            get(|state, request, Path(dashboard_id): Path<Uuid>| {
                proxy(
                    state,
                    request,
                    "dashboards.get",
                    AuthorizationGrantOperationV1::Read,
                    reqwest::Method::GET,
                    format!("api/dashboards/{dashboard_id}"),
                    None,
                )
            }),
        )
        .route("/api/admin/dashboards", post(proxy_create_dashboard))
        .route(
            "/api/admin/dashboards/visibility-nodes",
            get(|state, request| {
                proxy(
                    state,
                    request,
                    "dashboards.list_manageable",
                    AuthorizationGrantOperationV1::Read,
                    reqwest::Method::GET,
                    "api/admin/dashboards/visibility-nodes".into(),
                    None,
                )
            }),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}",
            axum::routing::put(proxy_update_dashboard).delete(proxy_delete_dashboard),
        )
        .route(
            "/api/admin/dashboards/{dashboard_id}/composition",
            get(proxy_get_composition).put(proxy_reconcile_composition),
        )
}

async fn proxy_create_dashboard(
    state: State<AppState>,
    request: AuthenticatedRequest,
    headers: HeaderMap,
    body: Bytes,
) -> ApiResult<Response> {
    proxy_metadata_mutation(
        state,
        request,
        "dashboards.create",
        reqwest::Method::POST,
        "api/admin/dashboards".into(),
        idempotency_key(&headers),
        body,
    )
    .await
}

async fn proxy_update_dashboard(
    state: State<AppState>,
    request: AuthenticatedRequest,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    body: Bytes,
) -> ApiResult<Response> {
    proxy_metadata_mutation(
        state,
        request,
        "dashboards.update",
        reqwest::Method::PUT,
        format!("api/admin/dashboards/{dashboard_id}"),
        idempotency_key(&headers),
        body,
    )
    .await
}

async fn proxy_metadata_mutation(
    state: State<AppState>,
    request: AuthenticatedRequest,
    action: &'static str,
    method: reqwest::Method,
    path: String,
    idempotency_key: String,
    body: Bytes,
) -> ApiResult<Response> {
    let mut payload: Value =
        serde_json::from_slice(&body).map_err(|_| ApiError::BadRequest("invalid JSON".into()))?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| ApiError::BadRequest("Dashboard payload must be an object".into()))?;
    object.insert("idempotency_key".into(), Value::String(idempotency_key));
    proxy(
        state,
        request,
        action,
        AuthorizationGrantOperationV1::Mutation,
        method,
        path,
        Some(Bytes::from(
            serde_json::to_vec(&payload).map_err(|error| ApiError::Internal(error.into()))?,
        )),
    )
    .await
}

async fn proxy_delete_dashboard(
    state: State<AppState>,
    request: AuthenticatedRequest,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Response> {
    proxy_with_idempotency(
        state,
        request,
        "dashboards.delete",
        reqwest::Method::DELETE,
        format!("api/admin/dashboards/{dashboard_id}"),
        idempotency_key(&headers),
        None,
    )
    .await
}

async fn proxy_get_composition(
    state: State<AppState>,
    request: AuthenticatedRequest,
    Path(dashboard_id): Path<Uuid>,
) -> ApiResult<Response> {
    proxy(
        state,
        request,
        "dashboards.load_composition",
        AuthorizationGrantOperationV1::Read,
        reqwest::Method::GET,
        format!("api/admin/dashboards/{dashboard_id}/composition"),
        None,
    )
    .await
}

async fn proxy_reconcile_composition(
    state: State<AppState>,
    request: AuthenticatedRequest,
    headers: HeaderMap,
    Path(dashboard_id): Path<Uuid>,
    body: Bytes,
) -> ApiResult<Response> {
    proxy_with_idempotency(
        state,
        request,
        "dashboards.reconcile_composition",
        reqwest::Method::PUT,
        format!("api/admin/dashboards/{dashboard_id}/composition"),
        idempotency_key(&headers),
        Some(body),
    )
    .await
}

async fn proxy_with_idempotency(
    state: State<AppState>,
    request: AuthenticatedRequest,
    action: &'static str,
    method: reqwest::Method,
    path: String,
    idempotency_key: String,
    body: Option<Bytes>,
) -> ApiResult<Response> {
    proxy_with_headers(
        state,
        request,
        action,
        AuthorizationGrantOperationV1::Mutation,
        method,
        path,
        body,
        Some(idempotency_key),
    )
    .await
}

fn idempotency_key(headers: &HeaderMap) -> String {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty() && value.chars().count() <= 200)
        .map(str::to_owned)
        .unwrap_or_else(|| Uuid::new_v4().to_string())
}

async fn proxy(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    action: &'static str,
    operation: AuthorizationGrantOperationV1,
    method: reqwest::Method,
    path: String,
    body: Option<Bytes>,
) -> ApiResult<Response> {
    proxy_with_headers(
        State(state),
        request,
        action,
        operation,
        method,
        path,
        body,
        None,
    )
    .await
}

async fn proxy_with_headers(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    action: &'static str,
    operation: AuthorizationGrantOperationV1,
    method: reqwest::Method,
    path: String,
    body: Option<Bytes>,
    idempotency_key: Option<String>,
) -> ApiResult<Response> {
    let grant = dashboard_authorization(&state.pool, &request, action, operation).await?;
    let encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&grant).map_err(|error| ApiError::Internal(error.into()))?);
    let mut outbound = reqwest::Client::new()
        .request(method, format!("{}/{}", dashboard_url(), path))
        .header("x-tessara-authorization", encoded)
        .header(reqwest::header::CONTENT_TYPE, "application/json");
    if let Some(idempotency_key) = idempotency_key {
        outbound = outbound.header("x-idempotency-key", idempotency_key);
    }
    if let Some(body) = body {
        outbound = outbound.body(body.to_vec());
    }
    let response = outbound.send().await.map_err(|_| module_unavailable())?;
    module_response(response).await
}

async fn dashboard_authorization(
    pool: &PgPool,
    request: &AuthenticatedRequest,
    action: &str,
    operation: AuthorizationGrantOperationV1,
) -> ApiResult<SignedEnvelopeV1<AuthorizationGrantV1>> {
    let instance = dashboard_instance(pool).await?;
    let instance_id: Uuid = instance.try_get("id")?;
    let installation_id: Uuid = instance.try_get("installation_id")?;
    let required_capability: String = sqlx::query_scalar(
        "SELECT required_capability FROM core_module_action_declarations
         WHERE target_definition_id=$1 AND dependency_binding=$2
           AND functional_contract=$3 AND action=$4 AND operation=$5",
    )
    .bind(DEFINITION_ID)
    .bind(DEPENDENCY_BINDING)
    .bind(CONTRACT_ID)
    .bind(action)
    .bind(operation_text(operation))
    .fetch_optional(pool)
    .await?
    .ok_or_else(restricted_authorization)?;
    let bindings =
        capability_bindings(pool, request.account.account_id, &required_capability).await?;
    if bindings.is_empty() {
        return Err(restricted_authorization());
    }
    let revisions = sqlx::query(
        "SELECT authorization_revision,organization_revision
         FROM core_security_revisions WHERE singleton=true",
    )
    .fetch_one(pool)
    .await?;
    let authorization_revision: i64 = revisions.try_get("authorization_revision")?;
    let organization_revision: i64 = revisions.try_get("organization_revision")?;
    sync_dashboard_state(
        pool,
        installation_id,
        instance_id,
        authorization_revision,
        organization_revision,
    )
    .await?;
    let now = Utc::now();
    let grant = AuthorizationGrantV1 {
        schema_version: 1,
        installation_id,
        original_actor_id: request.account.account_id,
        presenting_service: ModuleDefinitionId::new("tessara.core")
            .map_err(|error| ApiError::Internal(error.into()))?,
        audience_module_instance_id: instance_id,
        dependency_binding: DependencyBindingKey::new(DEPENDENCY_BINDING)
            .map_err(|error| ApiError::Internal(error.into()))?,
        functional_contract: FunctionalContractId::new(CONTRACT_ID)
            .map_err(|error| ApiError::Internal(error.into()))?,
        action: action.into(),
        operation,
        capability_scope_bindings: bindings,
        resource_assertion: None,
        delegation_basis: Vec::new(),
        authorization_revision: authorization_revision as u64,
        organization_revision: organization_revision as u64,
        jti: Uuid::new_v4(),
        issued_at: now,
        expires_at: now
            + Duration::seconds(match operation {
                AuthorizationGrantOperationV1::Read => 60,
                AuthorizationGrantOperationV1::Mutation => 30,
            }),
    };
    let signed = protocol_signer(ProtocolSignaturePurposeV1::AuthorizationGrant)?
        .sign(grant)
        .map_err(|error| ApiError::Internal(error.into()))?;
    // Keep construction and consumption contexts mechanically aligned.
    signed
        .payload
        .validate_for(&AuthorizationValidationContextV1 {
            installation_id,
            presenting_service: ModuleDefinitionId::new("tessara.core")
                .map_err(|error| ApiError::Internal(error.into()))?,
            audience_module_instance_id: instance_id,
            dependency_binding: DependencyBindingKey::new(DEPENDENCY_BINDING)
                .map_err(|error| ApiError::Internal(error.into()))?,
            functional_contract: FunctionalContractId::new(CONTRACT_ID)
                .map_err(|error| ApiError::Internal(error.into()))?,
            action: action.into(),
            operation,
            authorization_revision: authorization_revision as u64,
            organization_revision: organization_revision as u64,
            now,
        })
        .map_err(|_| restricted_authorization())?;
    Ok(signed)
}

async fn dashboard_instance(pool: &PgPool) -> ApiResult<sqlx::postgres::PgRow> {
    sqlx::query(
        "SELECT id,installation_id FROM module_instances
         WHERE definition_id=$1 AND identity_state='live' AND installed=true
           AND deployed=true AND configured=true AND ready=true AND enabled=true AND healthy=true",
    )
    .bind(DEFINITION_ID)
    .fetch_optional(pool)
    .await?
    .ok_or_else(module_unavailable)
}

async fn sync_dashboard_state(
    pool: &PgPool,
    installation_id: Uuid,
    instance_id: Uuid,
    authorization_revision: i64,
    organization_revision: i64,
) -> ApiResult<()> {
    let client = reqwest::Client::new();
    let control_key = std::env::var("TESSARA_MODULE_CONTROL_SHARED_KEY")
        .unwrap_or_else(|_| "development-module-control-only".into());
    client
        .put(format!("{}/api/private/security-state", dashboard_url()))
        .header("x-tessara-module-control-key", &control_key)
        .json(&json!({
            "schema_version": 1,
            "installation_id": installation_id,
            "module_instance_id": instance_id,
            "authorization_revision": authorization_revision,
            "organization_revision": organization_revision,
            "enabled": true,
            "document_state": "enabled"
        }))
        .send()
        .await
        .map_err(|_| module_unavailable())?
        .error_for_status()
        .map_err(|_| module_unavailable())?;
    let rows = sqlx::query(
        "WITH RECURSIVE organization AS (
           SELECT n.id,n.name,n.node_type_id,n.parent_node_id,n.name::text AS node_path
           FROM nodes n WHERE n.parent_node_id IS NULL
           UNION ALL
           SELECT child.id,child.name,child.node_type_id,child.parent_node_id,
                  parent.node_path || ' / ' || child.name
           FROM nodes child JOIN organization parent ON child.parent_node_id=parent.id
         )
         SELECT organization.id,organization.name,node_types.name AS node_type_name,
                organization.parent_node_id,organization.node_path
         FROM organization JOIN node_types ON node_types.id=organization.node_type_id
         ORDER BY organization.node_path,organization.id",
    )
    .fetch_all(pool)
    .await?;
    let nodes = rows
        .into_iter()
        .map(|row| {
            Ok(json!({
                "node_id": row.try_get::<Uuid,_>("id")?,
                "node_name": row.try_get::<String,_>("name")?,
                "node_type_name": row.try_get::<String,_>("node_type_name")?,
                "parent_node_id": row.try_get::<Option<Uuid>,_>("parent_node_id")?,
                "node_path": row.try_get::<String,_>("node_path")?,
            }))
        })
        .collect::<Result<Vec<_>, sqlx::Error>>()?;
    client
        .put(format!(
            "{}/api/private/organization-projection",
            dashboard_url()
        ))
        .header("x-tessara-module-control-key", control_key)
        .json(&json!({
            "schema_version": 1,
            "organization_revision": organization_revision,
            "nodes": nodes
        }))
        .send()
        .await
        .map_err(|_| module_unavailable())?
        .error_for_status()
        .map_err(|_| module_unavailable())?;
    Ok(())
}

async fn directory_page(State(state): State<AppState>, request: AuthenticatedRequest) -> Response {
    let result = proxy_json::<Vec<tessara_web::DashboardSummary>>(
        &state,
        &request,
        "dashboards.list",
        "api/dashboards",
    )
    .await;
    match result {
        Ok(dashboards) => dashboard_document(
            "/dashboards",
            "Tessara Dashboards",
            "Browse Tessara dashboards.",
            tessara_web::DashboardRouteBootstrap::directory(web_account(&request), dashboards),
        ),
        Err(_) => unavailable_document("/dashboards"),
    }
}

async fn create_page(State(state): State<AppState>, request: AuthenticatedRequest) -> Response {
    let result = proxy_json::<Vec<tessara_web::VisibilityNodeOption>>(
        &state,
        &request,
        "dashboards.list_manageable",
        "api/admin/dashboards/visibility-nodes",
    )
    .await;
    match result {
        Ok(nodes) => dashboard_document(
            "/dashboards/new",
            "Create Dashboard",
            "Create a Tessara dashboard.",
            tessara_web::DashboardRouteBootstrap::create(web_account(&request), nodes),
        ),
        Err(_) => unavailable_document("/dashboards/new"),
    }
}

async fn detail_page(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(dashboard_id): Path<Uuid>,
) -> Response {
    dashboard_read_page(&state, &request, dashboard_id, false).await
}

async fn viewer_page(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(dashboard_id): Path<Uuid>,
) -> Response {
    dashboard_read_page(&state, &request, dashboard_id, true).await
}

async fn dashboard_read_page(
    state: &AppState,
    request: &AuthenticatedRequest,
    dashboard_id: Uuid,
    viewer: bool,
) -> Response {
    let result = proxy_json::<tessara_web::Dashboard>(
        state,
        request,
        "dashboards.get",
        &format!("api/dashboards/{dashboard_id}"),
    )
    .await;
    match result {
        Ok(dashboard) => {
            let path = if viewer {
                format!("/dashboards/{dashboard_id}/view")
            } else {
                format!("/dashboards/{dashboard_id}")
            };
            let bootstrap = if viewer {
                tessara_web::DashboardRouteBootstrap::viewer(web_account(request), dashboard)
            } else {
                tessara_web::DashboardRouteBootstrap::detail(web_account(request), dashboard)
            };
            dashboard_document(
                &path,
                if viewer {
                    "Dashboard Viewer"
                } else {
                    "Dashboard Detail"
                },
                "Inspect a Tessara dashboard.",
                bootstrap,
            )
        }
        Err(_) => unavailable_document(&format!("/dashboards/{dashboard_id}")),
    }
}

async fn editor_page(
    State(state): State<AppState>,
    request: AuthenticatedRequest,
    Path(dashboard_id): Path<Uuid>,
) -> Response {
    let composition = proxy_json::<tessara_web::DashboardComposition>(
        &state,
        &request,
        "dashboards.load_composition",
        &format!("api/admin/dashboards/{dashboard_id}/composition"),
    )
    .await;
    let nodes = proxy_json::<Vec<tessara_web::VisibilityNodeOption>>(
        &state,
        &request,
        "dashboards.list_manageable",
        "api/admin/dashboards/visibility-nodes",
    )
    .await;
    match (composition, nodes) {
        (Ok(composition), Ok(nodes)) => dashboard_document(
            &format!("/dashboards/{dashboard_id}/edit"),
            "Edit Dashboard",
            "Edit a Tessara dashboard.",
            tessara_web::DashboardRouteBootstrap::editor(web_account(&request), composition, nodes),
        ),
        _ => unavailable_document(&format!("/dashboards/{dashboard_id}/edit")),
    }
}

async fn proxy_json<T: DeserializeOwned>(
    state: &AppState,
    request: &AuthenticatedRequest,
    action: &'static str,
    path: &str,
) -> ApiResult<T> {
    let grant = dashboard_authorization(
        &state.pool,
        request,
        action,
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let encoded = URL_SAFE_NO_PAD
        .encode(serde_json::to_vec(&grant).map_err(|error| ApiError::Internal(error.into()))?);
    reqwest::Client::new()
        .get(format!("{}/{}", dashboard_url(), path))
        .header("x-tessara-authorization", encoded)
        .send()
        .await
        .map_err(|_| module_unavailable())?
        .error_for_status()
        .map_err(|_| module_unavailable())?
        .json()
        .await
        .map_err(|_| module_unavailable())
}

fn web_account(request: &AuthenticatedRequest) -> tessara_web::SessionAccount {
    tessara_web::SessionAccount {
        capabilities: request.account.capabilities.clone(),
    }
}

fn dashboard_document(
    path: &str,
    title: &str,
    description: &str,
    bootstrap: tessara_web::DashboardRouteBootstrap,
) -> Response {
    let mut response = Html(tessara_web::application_html_with_dashboard_bootstrap(
        path,
        title,
        description,
        &bootstrap,
    ))
    .into_response();
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Cookie, Authorization"),
    );
    response
}

fn unavailable_document(route: &str) -> Response {
    Html(format!(
        r#"<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"><title>Dashboard module unavailable · Tessara</title><link rel="stylesheet" href="/pkg/tessara-web.css"></head><body class="tessara-app"><main class="login-shell"><section class="login-panel blurred-surface"><a class="login-brand" href="/">Tessara</a><div class="login-panel__header"><p class="eyebrow">Dashboard module</p><h1>Dashboards are temporarily unavailable</h1><p>The Dashboard Module Instance cannot currently be reached. Dashboard data remains in its isolated Module Instance database; Core credentials, browser cookies, configuration, and saved Component references have not been forwarded or replaced.</p></div><div class="button-row"><a class="button" href="{route}">Try Dashboards again</a><a class="button button--secondary" href="/administration/modules/tessara.dashboards#diagnostics">Open Module diagnostics</a></div></section></main></body></html>"#
    ))
    .into_response()
}

async fn module_response(response: reqwest::Response) -> ApiResult<Response> {
    let status = StatusCode::from_u16(response.status().as_u16())
        .map_err(|error| ApiError::Internal(error.into()))?;
    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();
    let bytes = response
        .bytes()
        .await
        .map_err(|error| ApiError::Internal(error.into()))?;
    Response::builder()
        .status(status)
        .header(header::CONTENT_TYPE, content_type)
        .body(Body::from(bytes))
        .map_err(|error| ApiError::Internal(error.into()))
}

fn dashboard_url() -> String {
    std::env::var("TESSARA_DASHBOARD_MODULE_URL")
        .unwrap_or_else(|_| "http://dashboards:8091".into())
        .trim_end_matches('/')
        .to_string()
}

fn operation_text(operation: AuthorizationGrantOperationV1) -> &'static str {
    match operation {
        AuthorizationGrantOperationV1::Read => "read",
        AuthorizationGrantOperationV1::Mutation => "mutation",
    }
}

fn restricted_authorization() -> ApiError {
    ApiError::Forbidden("Dashboard action unavailable".into())
}

fn module_unavailable() -> ApiError {
    ApiError::ServiceUnavailable("Dashboard module unavailable".into())
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};

    use super::idempotency_key;

    #[test]
    fn valid_client_idempotency_key_survives_the_core_gateway() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "x-idempotency-key",
            HeaderValue::from_static("dashboard-save-42"),
        );
        assert_eq!(idempotency_key(&headers), "dashboard-save-42");
    }

    #[test]
    fn absent_or_invalid_client_idempotency_key_gets_a_safe_fallback() {
        assert!(!idempotency_key(&HeaderMap::new()).is_empty());
        let mut headers = HeaderMap::new();
        headers.insert("x-idempotency-key", HeaderValue::from_static(" "));
        assert!(!idempotency_key(&headers).is_empty());
    }
}
