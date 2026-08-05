use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use semver::Version;
use serde::Serialize;
use serde_json::Value;
use tessara_datasets_contract::{
    DATASET_CONTRACT_SCHEMA_VERSION, DatasetAction, DatasetCatalogRequest, DatasetCatalogResponse,
};
use tessara_module_contract::{
    ArtifactDigest, AuthorizationGrantOperationV1, BrowserLifecycleAssetV1,
    BrowserLifecycleBootstrapV1, ModuleDefinitionId, SemanticRouteName,
};
use tessara_module_ui::{ShellPresentation, escape_attribute, escape_text, render_module_document};

use crate::{
    ComponentModuleError, ComponentModuleState, MODULE_RELEASE_VERSION, dataset_client, product,
    verified_shell_context,
};

pub(crate) const COMPONENT_CSS: &str = concat!(
    include_str!("../../tessara-module-ui/assets/module-shell.css"),
    "\n",
    include_str!("../assets/component.css")
);
pub(crate) const COMPONENT_LIFECYCLE_CSS: &str = concat!(
    include_str!("../assets/component.css"),
    "\n",
    include_str!("../assets/component-lifecycle.css")
);
pub(crate) const COMPONENT_JS: &str = include_str!("../assets/component.js");
pub(crate) const COMPONENT_CSS_SHA256: &str =
    "39a7a379a1ac2ee1ea1ed491eeb4aad438a14770a4496ba06b2d20aa998e5cb1";
pub(crate) const COMPONENT_LIFECYCLE_CSS_SHA256: &str =
    "dc06a4eee06e98884166baa646bda6064c1a1f88704673c8437bc2962ac7c373";
pub(crate) const COMPONENT_JS_SHA256: &str =
    "40cde4dbf5a67ef6e78c3b9610b157e084a5afa0637c412fe542258b7b269586";

#[derive(Clone, Serialize)]
#[serde(tag = "route", rename_all = "snake_case")]
enum ComponentRouteBootstrap {
    Directory { components: Value },
    Create { datasets: Value },
    Detail { component: Value },
    Edit { component: Value, datasets: Value },
    Versions { component: Value },
    View { component: Value },
}

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
    let components = product::list_components(State(state.clone()), headers.clone())
        .await?
        .0;
    document(
        &state,
        &headers,
        "/components",
        "Components",
        ComponentRouteBootstrap::Directory {
            components: serde_json::to_value(components).map_err(internal)?,
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
    let datasets = datasets(&state, &headers).await?;
    document(
        &state,
        &headers,
        "/components/new",
        "Create Component",
        ComponentRouteBootstrap::Create {
            datasets: serde_json::to_value(datasets).map_err(internal)?,
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
    let component = product::get_definition_by_id(&state, &grant.payload, component_id).await?;
    let value = serde_json::to_value(component).map_err(internal)?;
    let (path, title, bootstrap) = match route {
        "edit" => (
            format!("/components/{component_ref}/edit"),
            "Edit Component",
            ComponentRouteBootstrap::Edit {
                component: value,
                datasets: serde_json::to_value(datasets(&state, &headers).await?)
                    .map_err(internal)?,
            },
        ),
        "versions" => (
            format!("/components/{component_ref}/versions"),
            "Component Versions",
            ComponentRouteBootstrap::Versions { component: value },
        ),
        "view" => (
            format!("/components/{component_ref}/view"),
            "View Component",
            ComponentRouteBootstrap::View { component: value },
        ),
        _ => (
            format!("/components/{component_ref}"),
            "Component Detail",
            ComponentRouteBootstrap::Detail { component: value },
        ),
    };
    document(&state, &headers, &path, title, bootstrap).await
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
        "/api/private/component-datasets/catalog",
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
    let presentation = ShellPresentation::from_verified_context(&context, path, title);
    let content = render_content(&bootstrap);
    let stylesheet = asset_path(COMPONENT_CSS_SHA256, "component.css");
    let script = asset_path(COMPONENT_JS_SHA256, "component.js");
    let mut html = render_module_document(&presentation, &stylesheet, Some(&script), &content);
    let bootstrap_json = serde_json::to_string(&bootstrap)
        .map_err(internal)?
        .replace('&', "\\u0026")
        .replace('<', "\\u003c")
        .replace('>', "\\u003e");
    html = html.replacen("</head>", &format!(r#"<meta name="tessara-module-definition" content="tessara.components"><meta name="tessara-module-release" content="{}"><script id="tessara-component-bootstrap" type="application/json">{}</script></head>"#, escape_attribute(MODULE_RELEASE_VERSION), bootstrap_json), 1);
    let mut response = Html(html).into_response();
    no_store(&mut response);
    Ok(response)
}

fn render_content(bootstrap: &ComponentRouteBootstrap) -> String {
    match bootstrap {
        ComponentRouteBootstrap::Directory { components } => {
            let cards = components.as_array().into_iter().flatten().map(|item| {
                let id = text(item, "component_id"); let name = text(item, "name"); let slug = text(item, "slug");
                format!(r#"<article class="component-card"><h2><a href="/components/{id}">{name}</a></h2><p>{slug}</p><div class="component-actions"><a class="button" href="/components/{id}/view">View</a><a class="button button--secondary" href="/components/{id}/versions">Versions</a><a class="button button--secondary" href="/components/{id}/edit">Edit</a></div></article>"#)
            }).collect::<String>();
            format!(
                r#"<section class="components-page"><header class="components-page__header"><div><p class="eyebrow">Reusable presentation</p><h1>Components</h1></div><a class="button" href="/components/new">Create Component</a></header><div class="components-grid">{cards}</div></section>"#
            )
        }
        ComponentRouteBootstrap::Create { datasets } => render_form(None, datasets),
        ComponentRouteBootstrap::Edit {
            component,
            datasets,
        } => render_form(Some(component), datasets),
        ComponentRouteBootstrap::Detail { component } => render_detail(component),
        ComponentRouteBootstrap::View { component } => render_view(component),
        ComponentRouteBootstrap::Versions { component } => render_versions(component),
    }
}

fn render_detail(component: &Value) -> String {
    let name = text(component, "name");
    let id = text(component, "component_id");
    let description = text(component, "description");
    format!(
        r#"<section class="components-page"><header class="components-page__header"><div><p class="eyebrow">Component</p><h1>{name}</h1><p>{description}</p></div><div class="component-actions"><a class="button" href="/components/{id}/edit">Edit</a><a class="button button--secondary" href="/components/{id}/versions">Versions</a></div></header>{}</section>"#,
        render_versions_table(component)
    )
}
fn render_view(component: &Value) -> String {
    let name = text(component, "name");
    let component_ref = text(component, "component_id");
    let version = component
        .get("versions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|version| {
            version.get("publication_state").and_then(Value::as_str) == Some("published")
                && version.get("lifecycle_state").and_then(Value::as_str) == Some("active")
        });
    let Some(version) = version else {
        return format!(
            r#"<section class="components-page"><header><p class="eyebrow">Component viewer</p><h1>{name}</h1></header><div class="component-panel" role="status"><h2>Component unavailable</h2><p>No active published version is available to render.</p></div></section>"#
        );
    };
    let version_id = text(version, "component_version_id");
    let kind = text(version, "component_type");
    format!(
        r#"<section class="components-page"><header class="components-page__header"><div><p class="eyebrow">Component viewer</p><h1>{name}</h1></div><a class="button button--secondary" href="/components/{component_ref}">Component details</a></header><div class="component-panel component-render" data-component-render data-component-ref="{component_ref}" data-version-id="{version_id}" data-component-kind="{kind}" aria-busy="true"><p class="component-status" data-component-status role="status">Loading Component data…</p><div data-component-render-content></div><noscript><p>The Component definition and version metadata remain available from <a href="/components/{component_ref}">Component details</a>. JavaScript is required to execute Dataset-backed rendering.</p></noscript></div></section>"#
    )
}
fn render_versions(component: &Value) -> String {
    format!(
        r#"<section class="components-page"><header><p class="eyebrow">Component</p><h1>{} versions</h1></header>{}</section>"#,
        text(component, "name"),
        render_versions_table(component)
    )
}
fn render_versions_table(component: &Value) -> String {
    let rows = component
        .get("versions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|version| {
            let version_id = text(version, "component_version_id");
            let publication = text(version, "publication_state");
            let lifecycle = text(version, "lifecycle_state");
            let revision = text(version, "resource_revision");
            let mut actions = String::new();
            if publication == "draft" {
                actions.push_str(&format!(
                    r#"<button class="button button--small" type="button" data-component-version-action="publish" data-version-id="{version_id}">Publish</button><button class="button button--small button--secondary" type="button" data-component-version-action="delete" data-version-id="{version_id}">Delete draft</button>"#
                ));
            } else if lifecycle != "tombstoned" {
                for action in match lifecycle.as_str() {
                    "active" => &["deactivate", "archive", "tombstone"][..],
                    "inactive" => &["activate", "archive", "tombstone"][..],
                    "archived" => &["activate", "tombstone"][..],
                    _ => &[][..],
                } {
                    actions.push_str(&format!(
                        r#"<button class="button button--small button--secondary" type="button" data-component-version-action="{action}" data-version-id="{version_id}" data-resource-revision="{revision}">{}</button>"#,
                        escape_text(&title_case(action))
                    ));
                }
            }
            format!(
                r#"<tr><td>{}</td><td>{}</td><td>{publication}</td><td>{lifecycle}</td><td>{revision}</td><td><div class="component-actions">{actions}</div></td></tr>"#,
                text(version, "version_label"),
                text(version, "component_type"),
            )
        })
        .collect::<String>();
    format!(
        r#"<div class="component-panel" data-component-versions data-component-id="{}"><table class="component-table"><thead><tr><th>Version</th><th>Kind</th><th>Publication</th><th>Lifecycle</th><th>Revision</th><th>Actions</th></tr></thead><tbody>{rows}</tbody></table><p class="component-status" data-component-status aria-live="polite"></p></div>"#,
        text(component, "component_id")
    )
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
        .unwrap_or_default()
}
fn render_form(component: Option<&Value>, datasets: &Value) -> String {
    let name = component.map(|v| text(v, "name")).unwrap_or_default();
    let slug = component.map(|v| text(v, "slug")).unwrap_or_default();
    let description = component
        .map(|v| text(v, "description"))
        .unwrap_or_default();
    let version = component
        .and_then(|value| value.get("versions"))
        .and_then(Value::as_array)
        .and_then(|versions| {
            versions
                .iter()
                .find(|version| {
                    version.get("publication_state").and_then(Value::as_str) == Some("draft")
                })
                .or_else(|| versions.first())
        });
    let selected_reference = version
        .and_then(|value| value.get("dataset_reference"))
        .and_then(|value| serde_json::to_string(value).ok())
        .unwrap_or_default();
    let options = datasets
        .get("datasets")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|dataset| {
            let raw = serde_json::to_string(dataset.get("reference").unwrap_or(&Value::Null))
                .unwrap_or_default();
            let selected = if raw == selected_reference {
                " selected"
            } else {
                ""
            };
            let reference = escape_attribute(&raw);
            format!(
                r#"<option value="{reference}"{selected}>{} — major {}</option>"#,
                text(dataset, "dataset_name"),
                dataset
                    .get("reference")
                    .and_then(|v| v.get("reference"))
                    .and_then(|v| v.get("resource_id"))
                    .and_then(Value::as_str)
                    .and_then(|v| v.rsplit_once('@'))
                    .map(|(_, major)| major)
                    .unwrap_or("?")
            )
        })
        .collect::<String>();
    let component_id = component
        .map(|v| text(v, "component_id"))
        .unwrap_or_default();
    let draft_id = version
        .filter(|v| v.get("publication_state").and_then(Value::as_str) == Some("draft"))
        .map(|v| text(v, "component_version_id"))
        .unwrap_or_default();
    let kind = version
        .and_then(|v| v.get("component_type"))
        .and_then(Value::as_str)
        .unwrap_or("table");
    let kinds = ["table", "bar", "line", "pie", "donut", "stat_card"]
        .into_iter()
        .map(|value| {
            format!(
                r#"<option{}>{value}</option>"#,
                if value == kind { " selected" } else { "" }
            )
        })
        .collect::<String>();
    let config = version
        .and_then(|v| v.get("config"))
        .map(|v| escape_text(&serde_json::to_string_pretty(v).unwrap_or_default()))
        .unwrap_or_else(|| "{\"visible_columns\":[]}".into());
    let note = version.map(|v| text(v, "version_note")).unwrap_or_default();
    format!(
        r#"<section class="components-page"><header><p class="eyebrow">Component authoring</p><h1>{} Component</h1></header><form class="component-form component-panel" data-component-create data-component-id="{component_id}" data-draft-id="{draft_id}"><label>Name<input name="name" required maxlength="120" value="{name}"></label><label>Slug<input name="slug" required maxlength="120" value="{slug}"></label><label>Description<input name="description" value="{description}"></label><label>Dataset major line<select name="dataset_reference" required>{options}</select></label><label>Kind<select name="component_type">{kinds}</select></label><label>Configuration JSON<textarea name="config" required>{config}</textarea></label><label>Version note<input name="version_note" value="{note}"></label><div class="component-actions"><button class="button" type="submit">Save draft</button><a class="button button--secondary" href="/components">Cancel</a></div><p class="component-status" data-component-status aria-live="polite"></p><noscript><p>JavaScript is required to save; all Component details remain available without JavaScript.</p></noscript></form></section>"#,
        if component.is_some() {
            "Edit"
        } else {
            "Create"
        }
    )
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .map(|v| {
            v.as_str()
                .map(str::to_string)
                .unwrap_or_else(|| v.to_string())
        })
        .map(|v| escape_text(&v))
        .unwrap_or_default()
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

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue, header};
    use sha2::{Digest, Sha256};

    use super::{
        COMPONENT_CSS, COMPONENT_CSS_SHA256, COMPONENT_JS, COMPONENT_JS_SHA256,
        COMPONENT_LIFECYCLE_CSS, COMPONENT_LIFECYCLE_CSS_SHA256, accepts_lifecycle_bootstrap,
        asset_path, render_versions, render_view,
    };

    #[test]
    fn component_assets_are_digest_pinned_and_lifecycle_compatible() {
        for (content, expected) in [
            (COMPONENT_CSS, COMPONENT_CSS_SHA256),
            (COMPONENT_LIFECYCLE_CSS, COMPONENT_LIFECYCLE_CSS_SHA256),
            (COMPONENT_JS, COMPONENT_JS_SHA256),
        ] {
            assert_eq!(
                format!("{:x}", Sha256::digest(content.as_bytes())),
                expected
            );
        }
        assert!(COMPONENT_JS.contains("export async function createModule(host)"));
        assert!(COMPONENT_JS.contains("async canDeactivate()"));
        assert!(COMPONENT_CSS.contains(".app-shell"));
        assert!(COMPONENT_CSS.contains("@media"));
        assert!(
            asset_path(COMPONENT_JS_SHA256, "component.js")
                .contains("/tessara.components/1.0.0/sha256:")
        );
    }

    #[test]
    fn lifecycle_bootstrap_requires_the_versioned_media_type() {
        let mut headers = HeaderMap::new();
        assert!(!accepts_lifecycle_bootstrap(&headers));
        headers.insert(header::ACCEPT, HeaderValue::from_static("text/html"));
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
    fn module_documents_expose_view_and_lifecycle_controls() {
        let component = serde_json::json!({
            "component_id":"01980000-0002-7000-8000-000000000010",
            "name":"Reference table",
            "versions":[
                {"component_version_id":"01980000-0001-7000-8000-000000000010","version_label":"1.0.0","component_type":"table","publication_state":"published","lifecycle_state":"active","resource_revision":2},
                {"component_version_id":"01980000-0001-7000-8000-000000000011","version_label":"2.0.0","component_type":"table","publication_state":"draft","lifecycle_state":"active","resource_revision":1}
            ]
        });
        let versions = render_versions(&component);
        assert!(versions.contains("data-component-version-action=\"publish\""));
        assert!(versions.contains("data-component-version-action=\"delete\""));
        assert!(versions.contains("data-component-version-action=\"deactivate\""));
        let viewer = render_view(&component);
        assert!(viewer.contains("data-component-render"));
        assert!(viewer.contains("data-component-kind=\"table\""));
        assert!(COMPONENT_JS.contains("loadComponentRenderers"));
    }
}
