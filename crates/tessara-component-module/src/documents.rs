use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{IntoResponse, Response},
    routing::get,
};
use semver::Version;
use serde_json::Value;
use tessara_component_ui::{
    ComponentDefinitionBootstrap, ComponentDirectoryItem, ComponentRouteBootstrap,
    render_component_document,
};
use tessara_datasets_contract::{
    DATASET_CONTRACT_SCHEMA_VERSION, DatasetAction, DatasetCatalogRequest, DatasetCatalogResponse,
};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationGrantOperationV1, BrowserLifecycleAssetV1,
    BrowserLifecycleBootstrapV1, ModuleDefinitionId, SemanticRouteName,
};

use crate::{
    ComponentModuleError, ComponentModuleState, MODULE_RELEASE_VERSION, dataset_client, product,
    verified_shell_context,
};

pub(crate) use tessara_component_ui::{
    COMPONENT_BINDINGS_JS, COMPONENT_BINDINGS_JS_SHA256, COMPONENT_CSS, COMPONENT_CSS_SHA256,
    COMPONENT_JS, COMPONENT_JS_SHA256, COMPONENT_LIFECYCLE_CSS, COMPONENT_LIFECYCLE_CSS_SHA256,
    COMPONENT_WASM, COMPONENT_WASM_SHA256,
};

pub(super) fn routes() -> Router<ComponentModuleState> {
    Router::new()
        .route("/components", get(directory))
        .route("/components/new", get(create))
        .route("/components/{component_ref}/edit", get(edit))
        .route("/components/{component_ref}/versions", get(versions))
        .route("/components/{component_ref}/view", get(view_component))
        .route("/components/{component_ref}", get(detail))
}

async fn directory(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
) -> Result<Response, ComponentModuleError> {
    let grant = product::authorize(
        &state,
        &headers,
        "components.list",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let public = product::list_component_summaries(&state, &grant.payload).await?;
    let manageable = product::list_manageable_definitions(&state, &grant.payload).await?;
    let can_manage = product::has_component_manage_scope(&grant.payload);
    let components = directory_projection(
        serde_json::to_value(public).map_err(internal)?,
        serde_json::to_value(manageable).map_err(internal)?,
    );
    document(
        &state,
        &headers,
        "/components",
        "Components",
        ComponentRouteBootstrap::Directory {
            components,
            can_manage,
        },
    )
    .await
}

async fn create(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
) -> Result<Response, ComponentModuleError> {
    product::authorize(
        &state,
        &headers,
        "components.list_manageable",
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let (datasets, dataset_error) = resilient_datasets(&state, &headers).await;
    document(
        &state,
        &headers,
        "/components/new",
        "Create Component",
        ComponentRouteBootstrap::Create {
            datasets,
            dataset_error,
        },
    )
    .await
}

async fn detail(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> Result<Response, ComponentModuleError> {
    component_document(state, headers, component_ref, "detail").await
}
async fn edit(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> Result<Response, ComponentModuleError> {
    component_document(state, headers, component_ref, "edit").await
}
async fn versions(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> Result<Response, ComponentModuleError> {
    component_document(state, headers, component_ref, "versions").await
}
async fn view_component(
    State(state): State<ComponentModuleState>,
    headers: HeaderMap,
    Path(component_ref): Path<String>,
) -> Result<Response, ComponentModuleError> {
    component_document(state, headers, component_ref, "view").await
}

async fn component_document(
    state: ComponentModuleState,
    headers: HeaderMap,
    component_ref: String,
    route: &str,
) -> Result<Response, ComponentModuleError> {
    let action = if route == "edit" {
        "components.edit"
    } else {
        "components.get"
    };
    let grant = product::authorize(
        &state,
        &headers,
        action,
        AuthorizationGrantOperationV1::Read,
    )
    .await?;
    let component_id = product::resolve_component_id(&state, &component_ref).await?;
    let (component, manageable) = match route {
        "edit" => (
            product::get_manageable_definition_by_id(&state, &grant.payload, component_id).await?,
            true,
        ),
        "detail" | "versions" | "view" => {
            match product::get_manageable_definition_by_id(&state, &grant.payload, component_id)
                .await
            {
                Ok(component) => (component, true),
                Err(ComponentModuleError::NotFound(_)) => (
                    product::get_definition_by_id(&state, &grant.payload, component_id).await?,
                    false,
                ),
                Err(error) => return Err(error),
            }
        }
        _ => (
            product::get_definition_by_id(&state, &grant.payload, component_id).await?,
            false,
        ),
    };
    let value = serde_json::from_value::<ComponentDefinitionBootstrap>(
        serde_json::to_value(component).map_err(internal)?,
    )
    .map_err(internal)?;
    let (path, title, bootstrap) = match route {
        "edit" => (
            format!("/components/{component_ref}/edit"),
            "Edit Component",
            ComponentRouteBootstrap::Edit {
                component: value,
                datasets: Value::Null,
                dataset_error: None,
            },
        ),
        "versions" => (
            format!("/components/{component_ref}/versions"),
            "Component Versions",
            ComponentRouteBootstrap::Versions {
                component: value,
                manageable,
            },
        ),
        "view" => (
            format!("/components/{component_ref}/view"),
            "View Component",
            ComponentRouteBootstrap::View {
                component: value,
                manageable,
            },
        ),
        _ => (
            format!("/components/{component_ref}"),
            "Component Detail",
            ComponentRouteBootstrap::Detail {
                component: value,
                manageable,
            },
        ),
    };
    let bootstrap = if let ComponentRouteBootstrap::Edit { component, .. } = bootstrap {
        let (datasets, dataset_error) = resilient_datasets(&state, &headers).await;
        ComponentRouteBootstrap::Edit {
            component,
            datasets,
            dataset_error,
        }
    } else {
        bootstrap
    };
    document(&state, &headers, &path, title, bootstrap).await
}

async fn resilient_datasets(
    state: &ComponentModuleState,
    headers: &HeaderMap,
) -> (Value, Option<String>) {
    match datasets(state, headers).await {
        Ok(catalog) => (
            serde_json::to_value(catalog).unwrap_or(Value::Null),
            None,
        ),
        Err(_) => (
            Value::Null,
            Some(
                "Dataset metadata is temporarily unavailable. Your Component definition is still available; retry before previewing or saving."
                    .into(),
            ),
        ),
    }
}

async fn datasets(
    state: &ComponentModuleState,
    headers: &HeaderMap,
) -> Result<DatasetCatalogResponse, ComponentModuleError> {
    let authorization = headers
        .get("x-tessara-authorization")
        .and_then(|value| value.to_str().ok())
        .ok_or(ComponentModuleError::Forbidden)?;
    dataset_client::post(
        state,
        authorization,
        "/api/private/datasets/catalog",
        &DatasetCatalogRequest {
            schema_version: DATASET_CONTRACT_SCHEMA_VERSION,
            action: DatasetAction::Catalog,
        },
    )
    .await
}

async fn document(
    state: &ComponentModuleState,
    headers: &HeaderMap,
    path: &str,
    title: &str,
    bootstrap: ComponentRouteBootstrap,
) -> Result<Response, ComponentModuleError> {
    let context = verified_shell_context(state, headers).await?;
    if accepts_lifecycle_bootstrap(headers) {
        let projection = BrowserLifecycleBootstrapV1 {
            schema_version: BrowserLifecycleBootstrapV1::SCHEMA_VERSION,
            definition_id: ModuleDefinitionId::new(crate::MODULE_DEFINITION_ID)
                .map_err(internal)?,
            release_version: Version::parse(MODULE_RELEASE_VERSION).map_err(internal)?,
            lifecycle_abi: Version::new(1, 0, 0),
            destination: SemanticRouteName::new(destination(&bootstrap)).map_err(internal)?,
            path: path.to_string(),
            title: title.to_string(),
            document_state: context.document_state,
            entry_asset: lifecycle_asset(
                COMPONENT_JS_SHA256,
                "component.js",
                "text/javascript; charset=utf-8",
            ),
            stylesheet_assets: vec![lifecycle_asset(
                COMPONENT_LIFECYCLE_CSS_SHA256,
                "component-lifecycle.css",
                "text/css; charset=utf-8",
            )],
            payload: serde_json::to_value(&bootstrap).map_err(internal)?,
        };
        let mut response = Json(projection).into_response();
        response.headers_mut().insert(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/vnd.tessara.module-view+json; version=1"),
        );
        no_store(&mut response);
        return Ok(response);
    }
    let html = render_component_document(&context, path, title, &bootstrap, MODULE_RELEASE_VERSION);
    let mut response = axum::response::Html(html).into_response();
    no_store(&mut response);
    Ok(response)
}

fn directory_projection(public: Value, manageable: Value) -> Vec<ComponentDirectoryItem> {
    let mut by_id = std::collections::BTreeMap::new();
    for (value, can_manage) in public
        .as_array()
        .into_iter()
        .flatten()
        .map(|value| (value, false))
        .chain(
            manageable
                .as_array()
                .into_iter()
                .flatten()
                .map(|value| (value, true)),
        )
    {
        let current = value
            .get("versions")
            .and_then(Value::as_array)
            .and_then(|versions| {
                versions
                    .iter()
                    .find(|version| {
                        version.get("publication_state").and_then(Value::as_str)
                            == Some("published")
                    })
                    .or_else(|| {
                        versions.iter().find(|version| {
                            version.get("publication_state").and_then(Value::as_str)
                                == Some("draft")
                        })
                    })
                    .or_else(|| versions.first())
            })
            .or_else(|| value.get("current_version"))
            .unwrap_or(&Value::Null);
        let component_id = json_text(value, "component_id");
        by_id.insert(
            component_id.clone(),
            ComponentDirectoryItem {
                component_id,
                name: json_text(value, "name"),
                slug: json_text(value, "slug"),
                component_type: json_text(current, "component_type"),
                publication_state: json_text(current, "publication_state"),
                manageable: can_manage,
            },
        );
    }
    let mut values = by_id.into_values().collect::<Vec<_>>();
    values.sort_by_key(|item| item.name.to_lowercase());
    values
}

fn json_text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string()
}

fn destination(value: &ComponentRouteBootstrap) -> &'static str {
    match value {
        ComponentRouteBootstrap::Directory { .. } => "components.directory",
        ComponentRouteBootstrap::Create { .. } => "components.create",
        ComponentRouteBootstrap::Detail { .. } => "components.detail",
        ComponentRouteBootstrap::Edit { .. } => "components.edit",
        ComponentRouteBootstrap::Versions { .. } => "components.versions",
        ComponentRouteBootstrap::View { .. } => "components.view",
    }
}
fn accepts_lifecycle_bootstrap(headers: &HeaderMap) -> bool {
    headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|value| {
            value.split(',').any(|media| {
                media
                    .trim()
                    .starts_with("application/vnd.tessara.module-view+json")
            })
        })
}
fn asset_path(digest: &str, name: &str) -> String {
    format!("/_tessara/modules/tessara.components/{MODULE_RELEASE_VERSION}/sha256:{digest}/{name}")
}
fn lifecycle_asset(digest: &str, name: &str, content_type: &str) -> BrowserLifecycleAssetV1 {
    BrowserLifecycleAssetV1 {
        url: asset_path(digest, name),
        digest: ArtifactDigest::new(format!("sha256:{digest}")).expect("compiled asset digest"),
        content_type: content_type.into(),
    }
}
fn no_store(response: &mut Response) {
    response.headers_mut().insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("private, no-store"),
    );
    response.headers_mut().insert(
        header::VARY,
        HeaderValue::from_static("Accept, Authorization"),
    );
}
fn internal(error: impl std::fmt::Display) -> ComponentModuleError {
    ComponentModuleError::Internal(error.to_string())
}
