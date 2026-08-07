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
    "389ba2d13693f99119be3ecf455f595a9e7def0918372edbacc1948859e2fbb0";
pub(crate) const COMPONENT_LIFECYCLE_CSS_SHA256: &str =
    "f84262d3386f58d17b9ce5d005bb450aa3e78ee7ece5a4fcca6375571cdbb944";
pub(crate) const COMPONENT_JS_SHA256: &str =
    "7f13c08219f641055c1fb3bbabc9ab38f724db9b7bda2cc712ca830c9c736e1e";

#[derive(Clone, Serialize)]
#[serde(tag = "route", rename_all = "snake_case")]
enum ComponentRouteBootstrap {
    Directory {
        components: Value,
        can_manage: bool,
    },
    Create {
        datasets: Value,
        dataset_error: Option<String>,
    },
    Detail {
        component: Value,
        manageable: bool,
    },
    Edit {
        component: Value,
        datasets: Value,
        dataset_error: Option<String>,
    },
    Versions {
        component: Value,
        manageable: bool,
    },
    View {
        component: Value,
        manageable: bool,
    },
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
    let value = serde_json::to_value(component).map_err(internal)?;
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
        ComponentRouteBootstrap::Directory {
            components,
            can_manage,
        } => render_directory(components, *can_manage),
        ComponentRouteBootstrap::Create {
            datasets,
            dataset_error,
        } => render_form(None, datasets, dataset_error.as_deref()),
        ComponentRouteBootstrap::Edit {
            component,
            datasets,
            dataset_error,
        } => render_form(Some(component), datasets, dataset_error.as_deref()),
        ComponentRouteBootstrap::Detail {
            component,
            manageable,
        } => render_detail(component, *manageable),
        ComponentRouteBootstrap::View {
            component,
            manageable,
        } => render_view(component, *manageable),
        ComponentRouteBootstrap::Versions {
            component,
            manageable,
        } => render_versions(component, *manageable),
    }
}

fn directory_projection(public: Value, manageable: Value) -> Value {
    let mut by_id = std::collections::BTreeMap::new();
    for mut item in public.as_array().into_iter().flatten().cloned() {
        item["manageable"] = Value::Bool(false);
        by_id.insert(text(&item, "component_id"), item);
    }
    for mut item in manageable.as_array().into_iter().flatten().cloned() {
        item["manageable"] = Value::Bool(true);
        by_id.insert(text(&item, "component_id"), item);
    }
    let mut items = by_id.into_values().collect::<Vec<_>>();
    items.sort_by_key(|item| text(item, "name").to_lowercase());
    Value::Array(items)
}

fn directory_current_version(item: &Value) -> (&Value, String) {
    let versions = item.get("versions").and_then(Value::as_array);
    if let Some(versions) = versions {
        let draft = versions.iter().find(|version| {
            version.get("publication_state").and_then(Value::as_str) == Some("draft")
        });
        let published = versions.iter().find(|version| {
            version.get("publication_state").and_then(Value::as_str) == Some("published")
        });
        let status = match (draft, published) {
            (Some(_), Some(_)) => "updating",
            (Some(_), None) => "draft",
            (None, Some(_)) => "published",
            (None, None) => "superseded",
        };
        return (
            published
                .or(draft)
                .or_else(|| versions.first())
                .unwrap_or(&Value::Null),
            status.into(),
        );
    }
    let version = item.get("current_version").unwrap_or(&Value::Null);
    (version, text(version, "publication_state"))
}

fn directory_actions(route_ref: &str, manageable: bool) -> String {
    let mut actions =
        format!(r#"<a class="button button--small" href="/components/{route_ref}/view">View</a>"#);
    if manageable {
        actions.push_str(&format!(
            r#"<a class="button button--small button--secondary is-authorized" href="/components/{route_ref}/versions" data-component-manage-only>Versions</a><a class="button button--small button--secondary is-authorized" href="/components/{route_ref}/edit" data-component-manage-only>Edit</a>"#
        ));
    }
    actions
}

fn render_directory(components: &Value, can_manage: bool) -> String {
    let mut rows = String::new();
    let mut cards = String::new();
    for item in components.as_array().into_iter().flatten() {
        let id = text(item, "component_id");
        let name = text(item, "name");
        let slug = text(item, "slug");
        let route_ref = component_route_ref(item);
        let description = text(item, "description");
        let (version, status) = directory_current_version(item);
        let kind = text(version, "component_type");
        let manageable = item
            .get("manageable")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let actions = directory_actions(&route_ref, manageable);
        let kind_label = component_kind_label(&kind);
        let status_label = title_case(&status);
        rows.push_str(&format!(
            r#"<tr data-component-directory-item data-component-id="{id}" data-name="{}" data-kind="{kind}" data-status="{status}"><td><a href="/components/{route_ref}">{name}</a><span class="component-directory__slug">{slug}</span></td><td data-component-directory-kind>{kind_label}</td><td><span class="component-badge" data-component-directory-status>{status_label}</span></td><td><div class="component-actions">{actions}</div></td></tr>"#,
            escape_attribute(&name.to_lowercase())
        ));
        cards.push_str(&format!(
            r#"<article class="component-card components-list-mobile-card" data-component-directory-item data-component-id="{id}" data-name="{}" data-kind="{kind}" data-status="{status}"><p class="eyebrow"><span data-component-directory-kind>{kind_label}</span> · <span data-component-directory-status>{status_label}</span></p><h2><a href="/components/{route_ref}">{name}</a></h2><p>{description}</p><p class="component-directory__slug">{slug}</p><div class="component-actions">{actions}</div></article>"#,
            escape_attribute(&name.to_lowercase())
        ));
    }
    let kind_choices = ["table", "bar", "line", "pie", "donut", "stat_card"]
        .into_iter()
        .map(|kind| {
            format!(
                r#"<button type="button" role="menuitemradio" aria-checked="false" value="{kind}" data-component-kind-filter>{}</button>"#,
                component_kind_label(kind)
            )
        })
        .collect::<String>();
    let status_choices = ["published", "draft", "updating", "superseded"]
        .into_iter()
        .map(|status| {
            format!(
                r#"<button type="button" role="menuitemradio" aria-checked="false" value="{status}" data-component-status-filter>{}</button>"#,
                title_case(status)
            )
        })
        .collect::<String>();
    let create_action = if can_manage {
        r#"<a class="button is-authorized" href="/components/new" data-component-manage-only>Create Component</a>"#
    } else {
        ""
    };
    format!(
        r#"<section class="components-page" data-component-directory><header class="components-page__header"><div><p class="eyebrow">Reusable presentation</p><h1>Components</h1><p>Find, inspect, and reuse published Component definitions.</p></div>{create_action}</header><div class="component-directory__toolbar"><label class="component-directory__search"><span>Search components by name</span><input type="search" data-component-directory-search autocomplete="off"></label><div class="component-directory__desktop-filters"><div class="component-filter-menu"><button class="button button--secondary" type="button" data-component-filter-toggle="kind" aria-expanded="false">Filter Kind</button><div class="component-filter-menu__panel" role="menu" aria-label="Filter Kind" data-component-filter-menu="kind" hidden>{kind_choices}</div></div><div class="component-filter-menu"><button class="button button--secondary" type="button" data-component-filter-toggle="status" aria-expanded="false">Filter Status</button><div class="component-filter-menu__panel" role="menu" aria-label="Filter Status" data-component-filter-menu="status" hidden>{status_choices}</div></div></div><button class="button button--secondary component-directory__mobile-filter" type="button" data-component-mobile-filters-open>Open component filters</button></div><div class="components-list-responsive-table"><div class="table-wrap"><table class="component-table"><thead><tr><th>Component</th><th>Kind</th><th>Status</th><th>Actions</th></tr></thead><tbody>{rows}</tbody></table></div></div><div class="components-list-mobile">{cards}</div><p class="component-panel" data-component-directory-empty hidden>No Components match these filters.</p><dialog class="component-filter-dialog" data-component-mobile-filters aria-labelledby="component-filter-dialog-title"><form method="dialog"><header class="components-page__header"><h2 id="component-filter-dialog-title">Component filters</h2><button class="button button--secondary" value="close" title="Close component filters">Close</button></header><label>Filter components by kind<select data-component-mobile-kind-filter><option value="all">All kinds</option><option value="table">Table</option><option value="bar">Bar</option><option value="line">Line</option><option value="pie">Pie</option><option value="donut">Donut</option><option value="stat_card">Stat Card</option></select></label><label>Filter components by status<select data-component-mobile-status-filter><option value="all">All statuses</option><option value="published">Published</option><option value="draft">Draft</option><option value="superseded">Superseded</option></select></label><div class="component-actions"><button class="button button--secondary" type="button" data-component-clear-filters>Clear All</button><button class="button" value="close">Apply filters</button></div></form></dialog></section>"#
    )
}

fn render_detail(component: &Value, manageable: bool) -> String {
    render_view(component, manageable)
}
fn render_view(component: &Value, manageable: bool) -> String {
    let name = text(component, "name");
    let component_ref = component_route_ref(component);
    let manage_actions = if manageable {
        format!(
            r#"<a class="button button--secondary is-authorized" href="/components/{component_ref}/versions" data-component-manage-only>Versions</a><a class="button button--secondary is-authorized" href="/components/{component_ref}/edit" data-component-manage-only>Edit</a>"#
        )
    } else {
        String::new()
    };
    let versions = component
        .get("versions")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let version = versions.iter().find(|version| {
        version.get("publication_state").and_then(Value::as_str) == Some("published")
            && version.get("lifecycle_state").and_then(Value::as_str) == Some("active")
    });
    let Some(version) = version else {
        let (heading, message) = if versions.iter().any(|version| {
            version.get("publication_state").and_then(Value::as_str) == Some("published")
        }) {
            (
                "Component unavailable",
                "No active published version is available to render.",
            )
        } else {
            (
                "No published version",
                "Publish a Component version before using the viewer.",
            )
        };
        return format!(
            r#"<section class="components-page"><header class="components-page__header"><div><p class="eyebrow">Component viewer</p><h1>{name}</h1></div><div class="component-actions">{manage_actions}</div></header><div class="component-panel" role="status"><h2>{heading}</h2><p>{message}</p></div></section>"#
        );
    };
    let version_id = text(version, "component_version_id");
    let kind = text(version, "component_type");
    let version_label = text(version, "version_label");
    let dataset_reference = version
        .get("dataset_reference")
        .map(|value| escape_text(&serde_json::to_string_pretty(value).unwrap_or_default()))
        .unwrap_or_default();
    let config = version
        .get("config")
        .map(|value| escape_text(&serde_json::to_string_pretty(value).unwrap_or_default()))
        .unwrap_or_default();
    let html = format!(
        r#"<section class="components-page"><header class="components-page__header"><div><p class="eyebrow">Component viewer</p><h1>{name}</h1></div><div class="component-actions">{manage_actions}</div></header><div class="component-panel component-render" data-component-render data-component-ref="{component_ref}" data-version-id="{version_id}" data-component-kind="{kind}" aria-busy="true"><p class="component-status" data-component-status role="status">Loading Component data…</p><div data-component-render-content></div><noscript><p>JavaScript is required only for Dataset-backed execution. The exact stored definition remains available below.</p></noscript></div><details class="component-panel component-definition-summary"><summary>Definition and output identity</summary><dl class="component-meta"><dt>Version</dt><dd>{version_label}</dd><dt>Kind</dt><dd>{kind}</dd><dt>Version identity</dt><dd>{version_id}</dd></dl><h2>Dataset major-line identity</h2><pre>{dataset_reference}</pre><h2>Stored configuration</h2><pre>{config}</pre></details></section>"#
    );
    html
}
fn render_versions(component: &Value, manageable: bool) -> String {
    let component_ref = component_route_ref(component);
    let edit_action = if manageable {
        format!(
            r#"<a class="button button--secondary is-authorized" href="/components/{component_ref}/edit" data-component-manage-only>Edit</a>"#
        )
    } else {
        String::new()
    };
    format!(
        r#"<section class="components-page"><header class="components-page__header"><div><p class="eyebrow">Component</p><h1>{} versions</h1></div><div class="component-actions"><a class="button" href="/components/{component_ref}/view">View</a>{edit_action}</div></header>{}</section>"#,
        text(component, "name"),
        render_versions_table(component, manageable)
    )
}

fn component_route_ref(component: &Value) -> String {
    let raw = component
        .get("slug")
        .and_then(Value::as_str)
        .filter(|slug| !slug.is_empty())
        .or_else(|| component.get("component_id").and_then(Value::as_str))
        .unwrap_or_default();
    let mut encoded = String::with_capacity(raw.len());
    for byte in raw.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    escape_attribute(&encoded)
}
fn render_versions_table(component: &Value, manageable: bool) -> String {
    let rows = component
        .get("versions")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .map(|version| {
            let version_id = text(version, "component_version_id");
            let publication = text(version, "publication_state");
            let lifecycle = text(version, "lifecycle_state");
            let publication_label = title_case(&publication);
            let lifecycle_label = title_case(&lifecycle);
            let revision = text(version, "resource_revision");
            let dataset_version = dataset_version_label(version);
            let version_note = match text(version, "version_note") {
                note if note.trim().is_empty() => "—".into(),
                note => note,
            };
            let mut actions = String::new();
            if manageable && publication == "draft" {
                actions.push_str(&format!(
                    r#"<button class="button button--small is-authorized" type="button" role="menuitem" data-component-manage-only data-component-version-action="publish" data-version-id="{version_id}">Publish</button><button class="button button--small button--secondary is-authorized" type="button" role="menuitem" data-component-manage-only data-component-version-action="delete" data-version-id="{version_id}">Delete draft</button>"#
                ));
            } else if manageable && lifecycle != "tombstoned" {
                for action in match lifecycle.as_str() {
                    "active" => &["deactivate", "archive"][..],
                    "inactive" => &["activate", "archive"][..],
                    "archived" => &["tombstone"][..],
                    _ => &[][..],
                } {
                    actions.push_str(&format!(
                        r#"<button class="button button--small button--secondary is-authorized" type="button" role="menuitem" data-component-manage-only data-component-version-action="{action}" data-version-id="{version_id}" data-resource-revision="{revision}">{}</button>"#,
                        escape_text(&title_case(action))
                    ));
                }
            }
            let action_menu = if actions.is_empty() {
                r#"<span aria-label="No lifecycle actions">—</span>"#.into()
            } else {
                format!(
                    r#"<details class="component-version-actions is-authorized" data-component-manage-only><summary class="button button--small button--secondary">Open actions for {}</summary><div class="component-version-actions__menu" role="menu">{actions}</div></details>"#,
                    text(version, "version_label")
                )
            };
            format!(
                r#"<tr data-publication-state="{publication}" data-lifecycle-state="{lifecycle}"><td>{}</td><td>{}</td><td>{dataset_version}</td><td>{publication_label}</td><td>{lifecycle_label}</td><td>{version_note}</td><td>{revision}</td><td>{action_menu}</td></tr>"#,
                text(version, "version_label"),
                text(version, "component_type"),
            )
        })
        .collect::<String>();
    format!(
        r#"<div class="component-panel" data-component-versions data-component-id="{}"><table class="component-table"><thead><tr><th>Version</th><th>Kind</th><th>Dataset Version</th><th>Publication</th><th>Lifecycle</th><th>Version Note</th><th>Revision</th><th>Actions</th></tr></thead><tbody>{rows}</tbody></table><p class="component-status" data-component-status aria-live="polite"></p></div>"#,
        text(component, "component_id")
    )
}

fn dataset_version_label(version: &Value) -> String {
    version
        .pointer("/dataset_reference/reference/resource_id")
        .and_then(Value::as_str)
        .and_then(|resource_id| resource_id.rsplit_once('@').map(|(_, major)| major))
        .filter(|major| !major.is_empty())
        .map(|major| escape_text(&format!("v{major}")))
        .unwrap_or_else(|| "—".into())
}

fn title_case(value: &str) -> String {
    let mut characters = value.chars();
    characters
        .next()
        .map(|first| first.to_uppercase().collect::<String>() + characters.as_str())
        .unwrap_or_default()
}

fn component_kind_label(value: &str) -> String {
    match value {
        "stat_card" => "Stat Card".into(),
        other => title_case(&other.replace('_', " ")),
    }
}

fn render_form(component: Option<&Value>, datasets: &Value, dataset_error: Option<&str>) -> String {
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
    let mut options = datasets
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
    if !selected_reference.is_empty() && !options.contains(" selected") {
        options.insert_str(
            0,
            &format!(
                r#"<option value="{}" selected>Current Dataset major line</option>"#,
                escape_attribute(&selected_reference)
            ),
        );
    }
    let component_id = component
        .map(|v| text(v, "component_id"))
        .unwrap_or_default();
    let draft_id = version
        .filter(|v| v.get("publication_state").and_then(Value::as_str) == Some("draft"))
        .map(|v| text(v, "component_version_id"))
        .unwrap_or_default();
    let published_id = component
        .and_then(|value| value.get("versions"))
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .find(|version| {
            version.get("publication_state").and_then(Value::as_str) == Some("published")
        })
        .map(|version| text(version, "component_version_id"))
        .unwrap_or_default();
    let kind = version
        .and_then(|v| v.get("component_type"))
        .and_then(Value::as_str)
        .unwrap_or("table");
    let kinds = ["table", "bar", "line", "pie", "donut", "stat_card"]
        .into_iter()
        .map(|value| {
            let label = component_kind_label(value);
            format!(
                r#"<button type="button" role="radio" class="component-editor__kind-button{}" aria-checked="{}" data-component-kind="{value}">{label}</button>"#,
                if value == kind { " is-selected" } else { "" },
                if value == kind { "true" } else { "false" },
            )
        })
        .collect::<String>();
    let config = version
        .and_then(|v| v.get("config"))
        .map(|v| escape_text(&serde_json::to_string(v).unwrap_or_default()))
        .unwrap_or_else(|| "{\"visible_columns\":[]}".into());
    let note = version.map(|v| text(v, "version_note")).unwrap_or_default();
    let dataset_available = dataset_error.is_none();
    let outage = dataset_error
        .map(|message| {
            format!(
                r#"<div class="component-provider-outage" role="alert" data-component-dataset-outage><p>{}</p><button class="button button--secondary" type="button" data-component-dataset-retry>Retry Dataset metadata</button></div>"#,
                escape_text(message)
            )
        })
        .unwrap_or_else(|| {
            r#"<div class="component-provider-outage" role="alert" data-component-dataset-outage hidden><p></p><button class="button button--secondary" type="button" data-component-dataset-retry>Retry Dataset metadata</button></div>"#.into()
        });
    let unavailable = if dataset_available { "false" } else { "true" };
    let save_disabled = if dataset_available { "" } else { " disabled" };
    let update_action = if published_id.is_empty() {
        String::new()
    } else {
        format!(
            r#"<button class="button button--secondary" type="submit" role="menuitem" data-component-save-action="update_existing_version"{save_disabled}>Update Existing Version</button>"#
        )
    };
    let publish_menu = format!(
        r#"<details class="component-editor__publish-menu"><summary class="button button--secondary">Publish</summary><div class="component-editor__publish-actions" role="menu">{update_action}<button class="button" type="button" role="menuitem" data-component-open-consumer-review{save_disabled}>Create New Version</button></div></details>"#
    );
    let direct_publish = format!(
        r#"<button class="button button--secondary" type="submit" data-component-save-action="create_new_version"{save_disabled}>Publish new version</button>"#
    );
    let html = format!(
        r#"<section class="components-page component-editor" data-component-editor-root><header><p class="eyebrow">Component authoring</p><h1>{} Component</h1><p>Bind a Dataset major line, choose a presentation kind, and verify the live preview before saving.</p></header>{outage}<form class="component-form" data-component-create data-component-id="{component_id}" data-draft-id="{draft_id}" data-published-id="{published_id}" data-dataset-unavailable="{unavailable}" data-dirty="false"><textarea name="config" data-component-config hidden>{config}</textarea><input type="hidden" name="component_type" value="{kind}" data-component-kind-value><div class="component-editor__layout"><div class="component-editor__fields"><fieldset class="component-panel"><legend>Component details</legend><label>Name<input name="name" required maxlength="120" value="{name}"></label><label>Slug<input name="slug" required maxlength="120" value="{slug}"></label><label>Description<input name="description" maxlength="500" value="{description}"></label></fieldset><fieldset class="component-panel component-editor__kind-panel"><legend>Component Kind</legend><div class="component-editor__kind-grid" role="radiogroup" aria-label="Component kind">{kinds}</div><p class="component-kind-description" data-component-kind-description></p><div class="component-editor__kind-confirmation" role="alertdialog" aria-label="Confirm component kind change" data-component-kind-confirmation hidden><p data-component-kind-confirmation-copy></p><div class="component-actions"><button class="button button--secondary" type="button" data-component-kind-cancel>Cancel</button><button class="button" type="button" data-component-kind-confirm>Change kind</button></div></div></fieldset><fieldset class="component-panel"><legend>Dataset binding</legend><label data-component-dataset-native-label>Dataset Version<select name="dataset_reference" required data-component-dataset-picker>{options}</select></label><div data-component-dataset-picker-enhancement></div><p class="component-help">Component versions remain bound to this exact Dataset major line.</p></fieldset><div class="component-panel" data-component-kind-editor tabindex="-1"><section data-component-config-section="table"><fieldset><legend>Displayed Fields</legend><div class="component-editor__check-grid" role="listbox" aria-label="Available fields" aria-multiselectable="true" data-component-visible-fields></div></fieldset><fieldset><legend>Table defaults</legend><label>Page Size<input type="number" min="1" max="200" value="25" data-config-control="page_size"></label><label>Sort Field<select data-field-select data-config-control="sort_field"><option value="">Default row order</option></select></label><label>Sort Direction<select data-config-control="sort_direction"><option value="asc">Ascending</option><option value="desc">Descending</option></select></label><fieldset><legend>Searchable fields</legend><div class="component-editor__check-grid" data-component-search-fields></div></fieldset></fieldset></section><section data-component-config-section="visual" hidden><fieldset><legend>Fields &amp; Calculation</legend><label>Calculation<select data-config-control="summary_type"><option value="row_count">Count rows</option><option value="count">Count non-empty values</option><option value="unique_count">Count unique values</option><option value="sum">Sum</option><option value="average">Average</option><option value="median">Median</option></select></label><label>Value field<select data-field-select data-config-control="summary_field"><option value="">Select a field</option></select></label><label>Format<select data-config-control="value_format"><option value="plain">Plain number</option><option value="integer">Integer</option><option value="percent">Percent</option></select></label><label>Missing values<select data-config-control="value_missing_policy"><option value="omit">Omit</option><option value="zero">Use zero</option><option value="explicit_missing">Show missing</option></select></label><label>Order categories by<select data-config-control="visual_sort_field"><option value="category">Category</option><option value="summary_value">Calculated value</option></select></label><label>Sort direction<select data-config-control="visual_sort_direction"><option value="asc">Ascending</option><option value="desc">Descending</option></select></label></fieldset><fieldset data-component-config-section="bar"><legend>Bar options</legend><label>Category Field<select data-field-select data-config-control="category_field"><option value="">Select a field</option></select></label><label><input type="checkbox" aria-label="Split bars" data-config-control="split_bars"> Split bars by a series field</label><label data-component-comparison-control>Series field<select data-field-select data-config-control="comparison_field"><option value="">Select a field</option></select></label><label>Missing categories<select data-config-control="category_missing_policy"><option value="omit">Omit</option><option value="explicit_missing">Show missing</option></select></label><label>Missing series<select data-config-control="comparison_missing_policy"><option value="omit">Omit</option><option value="explicit_missing">Show missing</option></select></label><label>Comparison Layout<select data-config-control="comparison_layout"><option value="grouped">Grouped</option><option value="stacked">Stacked</option></select></label><label>Orientation<select data-config-control="orientation"><option value="horizontal">Horizontal</option><option value="vertical">Vertical</option></select></label><label>Category axis title<input data-config-control="x_axis_label"></label><label>Value axis title<input data-config-control="y_axis_label"></label><label>Limit<input type="number" min="1" max="1000" value="20" data-config-control="number_of_points"></label></fieldset><fieldset data-component-config-section="line" hidden><legend>Line options</legend><label>Category Field<select data-field-select data-config-control="x_field"><option value="">Select a field</option></select></label><label>Missing categories<select data-config-control="x_missing_policy"><option value="omit">Omit</option><option value="explicit_missing">Show missing</option></select></label><label><input type="checkbox" aria-label="Smoothing" checked data-config-control="smoothing"> Smooth line</label><label>Limit<input type="number" min="1" max="1000" value="20" data-config-control="line_number_of_points"></label></fieldset><fieldset data-component-config-section="pie" hidden><legend>Pie and donut options</legend><label>Category Field<select data-field-select data-config-control="pie_category_field"><option value="">Select a field</option></select></label><label>Missing categories<select data-config-control="pie_category_missing_policy"><option value="omit">Omit</option><option value="explicit_missing">Show missing</option></select></label><label>Limit<input type="number" min="1" max="1000" value="20" data-config-control="max_slices"></label><label>Legend Title<input data-config-control="legend_title"></label><table class="component-category-labels" aria-label="Category Labels"><caption>Category Labels</caption><thead><tr><th>Dataset value</th><th>Display label</th></tr></thead><tbody data-component-category-labels><tr><td colspan="2">Choose a category field to load values.</td></tr></tbody></table></fieldset><fieldset data-component-config-section="stat_card" hidden><legend>Stat card options</legend><label>Label<input data-config-control="stat_label"></label><label>Supporting Text<input data-config-control="supporting_text"></label><label>Panel Style<select data-config-control="panel_style"><option value="default">Default</option><option value="muted">Muted</option><option value="accent">Accent</option></select></label></fieldset></section></div><fieldset class="component-panel"><legend>Filters</legend><div data-component-filter-rows></div><button class="button button--secondary" type="button" data-component-add-filter>Add filter</button></fieldset><fieldset class="component-panel"><legend>Version</legend><label>Version note<input name="version_note" maxlength="2000" value="{note}"></label><p class="component-help">Publishing a new version requires a note so consumers can review the change.</p></fieldset><div class="component-actions"><button class="button" type="submit" data-component-save-action="save_draft"{save_disabled}>Save Draft</button><button class="button button--secondary" type="submit" data-component-save-action="create_new_version"{save_disabled}>Publish new version</button>{}<a class="button button--secondary" href="/components">Cancel</a></div><p class="component-status" data-component-status aria-live="polite"></p></div><aside class="component-panel component-editor-preview" aria-label="Component preview"><header class="component-editor-preview__header"><div><p class="eyebrow">Live preview</p><h2>Component preview</h2></div><span class="component-editor-preview__badge" data-component-preview-badge>Needs attention</span></header><div class="component-editor-preview__findings" data-component-preview-findings role="region" aria-label="Validation Findings" aria-live="polite">Choose a Dataset and complete the required fields.</div><div data-component-preview-content></div></aside></div><button class="button component-editor-preview__fab" id="component-editor-preview-fab" type="button" data-component-preview-open>Open preview</button><dialog class="component-editor-preview__dialog" tabindex="-1" data-component-preview-dialog aria-labelledby="component-preview-dialog-title"><header class="components-page__header"><h2 id="component-preview-dialog-title">Component preview</h2><button class="button button--secondary" id="component-editor-preview-close" type="button" data-component-preview-close>Close</button></header><div data-component-mobile-preview-content></div></dialog><noscript><div class="component-panel"><p>JavaScript is required to preview and save this definition. Name, Dataset binding, Component kind, version note, and stored configuration remain visible in this document.</p><details><summary>Stored configuration</summary><pre>{config}</pre></details></div></noscript></form></section>"#,
        if component.is_some() {
            "Edit"
        } else {
            "Create"
        },
        String::new(),
    );
    html.replace(&direct_publish, &publish_menu)
    .replace(
        "</form>",
        r#"<dialog class="component-consumers-dialog" data-component-consumer-review aria-labelledby="component-consumer-review-title"><div><h2 id="component-consumer-review-title">Review component consumers</h2><p>Review consumers before publishing. Existing consumers remain pinned until they deliberately adopt the new Component version.</p><div class="component-consumers-dialog__inventory" role="status">No registered consumers currently require repinning.</div><label>New Version Note<textarea data-component-new-version-note required maxlength="2000" placeholder="Summarize what changed in this version"></textarea></label><p class="component-status" data-component-consumer-review-error role="alert" hidden></p><div class="component-actions"><button class="button button--secondary" type="button" data-component-consumer-review-cancel>Cancel</button><button class="button" type="button" data-component-consumer-review-confirm>Create New Version</button></div></div></dialog></form>"#,
    )
    .replace(
        r#"<option value="median">Median</option>"#,
        r#"<option value="median">Median</option><option value="none">Do not summarize</option>"#,
    )
    .replace(
        r#"<option value="integer">Integer</option>"#,
        r#"<option value="integer">Integer</option><option value="decimal">Decimal</option>"#,
    )
    .replace(
        r#"<label>Sort Field<select data-field-select"#,
        r#"<label title="Choose the Dataset field used for the initial row order. Default row order preserves provider order.">Sort Field<select aria-label="Sort Field" data-field-select"#,
    )
    .replace(
        r#"<section data-component-config-section="visual" hidden><fieldset><legend>Fields &amp; Calculation</legend><label>Calculation"#,
        r#"<section data-component-config-section="visual" hidden><fieldset class="component-editor__role-card component-editor__role-card--measure"><legend>Fields &amp; Calculation</legend><header class="component-editor__role-card-header"><strong>Measure</strong><button class="component-field-help" type="button" aria-label="Measure help" title="Calculates one value for every category and series group using the selected Calculation and Value field.">?</button></header><p>What number should determine the visual value?</p><label title="Count rows counts every participating row. Count non-empty values counts rows with a Value field value. Count unique values counts distinct values. Sum, Average, and Median summarize numeric values. Do not summarize requires exactly one row per group.">Calculation"#,
    )
    .replace(
        r#"<label>Value field<select data-field-select data-config-control="summary_field">"#,
        r#"<label data-component-value-field>Value field<select data-field-select data-config-control="summary_field">"#,
    )
    .replace(
        r#"<label>Missing values<select data-config-control="value_missing_policy"><option value="omit">Omit</option><option value="zero">Use zero</option><option value="explicit_missing">Show missing</option></select></label>"#,
        r#"<label data-component-value-missing-policy>Missing values<select data-config-control="value_missing_policy"><option value="omit">Omit</option><option value="zero">Use zero</option><option value="explicit_missing">Show missing</option></select></label><p class="component-help component-editor__calculation-warning" role="status" data-component-calculation-warning hidden>Every category and series group must resolve to exactly one row. Preview and execution will report an error when duplicates exist.</p>"#,
    )
    .replace(
        r#"<label>Order categories by<select data-config-control="visual_sort_field"><option value="category">Category</option><option value="summary_value">Calculated value</option></select></label>"#,
        r#"<label class="component-editor__sort-field"><span class="component-field-label"><span>Sort Field</span><details class="component-field-help-menu"><summary title="Show help for Sort Field" aria-label="Show help for Sort Field">?</summary><span class="component-field-help-menu__content" role="tooltip" data-component-sort-field-help>Default: uses the order produced by the current grouping and summarization.
Category: sorts by the displayed category label.
Summary Value: sorts by the summarized numeric value.</span></details></span><select aria-label="Sort Field" data-config-control="visual_sort_field"><option value="">Default</option><option value="category">Category</option><option value="summary_value">Summary Value</option></select></label>"#,
    )
    .replace(
        r#"<fieldset data-component-config-section="bar"><legend>Bar options</legend>"#,
        r#"<fieldset data-component-config-section="bar"><legend>Bar options</legend><header class="component-editor__section-heading"><h2>Build the bars</h2><p>Choose what creates each bar and how its value is calculated.</p></header><div class="component-editor__role-grid" aria-label="Bar data roles"><section class="component-editor__role-card"><header class="component-editor__role-card-header"><strong>Category</strong><button class="component-field-help" type="button" aria-label="Category help" title="Creates one group for every distinct Category Field value.">?</button></header><p>What should each row or column of bars represent?</p></section><section class="component-editor__role-card"><header class="component-editor__role-card-header"><strong>Series</strong></header><p>Optionally compare a second dimension within each category.</p></section></div>"#,
    )
    .replace(
        r#"<label><input type="checkbox" aria-label="Smoothing" checked data-config-control="smoothing"> Smooth line</label><label>Limit"#,
        r#"<label><input type="checkbox" aria-label="Smoothing" checked data-config-control="smoothing"> Smooth line</label><label>Category axis title<input data-config-control="line_x_axis_label"></label><label>Value axis title<input data-config-control="line_y_axis_label"></label><label>Limit"#,
    )
    .replace(r#"max="1000""#, r#"max="100""#)
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
        asset_path, render_directory, render_form, render_versions, render_view,
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
        assert!(COMPONENT_JS.contains("Discard unsaved Component changes?"));
        assert!(COMPONENT_JS.contains("headers[\"x-idempotency-key\"]"));
        assert!(COMPONENT_JS.contains("/api/admin/components/validate"));
        assert!(COMPONENT_JS.contains("\"/api/admin/components/save\""));
        assert!(!COMPONENT_JS.contains("metadataAction"));
        assert!(COMPONENT_JS.contains("showDatasetOutage"));
        assert!(COMPONENT_JS.contains("component-d3-svg--"));
        assert!(COMPONENT_JS.contains("component-d3-chart__legend"));
        assert!(COMPONENT_JS.contains("chart.insertBefore(legend, svg)"));
        assert!(COMPONENT_JS.contains("activeOverrideControl(form) !== control"));
        assert!(COMPONENT_JS.contains("if (loadPage) fetchPage(false)"));
        assert!(COMPONENT_JS.contains("query.set(\"visible_columns\""));
        assert!(COMPONENT_JS.contains("`filter[${field}][operator]`"));
        assert!(!COMPONENT_JS.contains("item.field_key || item.key || item.field"));
        assert!(!COMPONENT_JS.contains("filter.field_key || filter.field"));
        assert!(!COMPONENT_JS.contains("\n    missing_policy:"));
        assert!(!COMPONENT_JS.contains("config.missing_policy"));
        assert!(
            COMPONENT_JS.contains(
                "value_missing_policy: configControl(form, \"value_missing_policy\").value"
            )
        );
        assert!(COMPONENT_JS.contains("component-d3-chart__category-label"));
        assert!(COMPONENT_JS.contains("line_x_axis_label"));
        assert!(COMPONENT_JS.contains("data-component-consumer-review"));
        assert!(COMPONENT_JS.contains("const CATEGORY_COLOR_OPTIONS"));
        assert!(COMPONENT_JS.contains("var(--semantic-warning)"));
        assert!(COMPONENT_JS.contains("valueCell.scope = \"row\""));
        assert!(COMPONENT_JS.contains("label.placeholder = raw"));
        assert!(COMPONENT_JS.contains("new Option(`Stored (${storedColor})`, storedColor)"));
        assert!(COMPONENT_JS.contains("config.sort_field ?? \"\""));
        assert!(!COMPONENT_JS.contains("color.type = \"color\""));
        assert!(COMPONENT_JS.contains("Previous page"));
        assert!(COMPONENT_JS.contains("Next page"));
        assert!(COMPONENT_JS.contains("root.setAttribute(\"data-hydration\", \"ready\")"));
        assert!(COMPONENT_JS.contains("\"component-visual-preview\""));
        assert!(COMPONENT_CSS.contains(".app-shell"));
        assert!(COMPONENT_CSS.contains("@media"));
        assert!(COMPONENT_CSS.contains(".components-page {"));
        assert!(COMPONENT_CSS.contains("minmax(min(100%, 16rem), 1fr)"));
        assert!(COMPONENT_CSS.contains("background: var(--surface-raised, #fff)"));
        assert!(COMPONENT_CSS.contains(".component-visual-preview {"));
        assert!(COMPONENT_LIFECYCLE_CSS.contains(".component-editor__layout"));
        assert!(COMPONENT_LIFECYCLE_CSS.contains(".components-list-mobile"));
        assert!(
            asset_path(COMPONENT_JS_SHA256, "component.js").contains(&format!(
                "/tessara.components/{}/sha256:",
                crate::MODULE_RELEASE_VERSION
            ))
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
        let versions = render_versions(&component, true);
        assert!(versions.contains("data-component-version-action=\"publish\""));
        assert!(versions.contains("data-component-manage-only"));
        assert!(versions.contains("is-authorized"));
        assert!(!versions.contains("hidden>Publish"));
        assert!(versions.contains("data-component-version-action=\"delete\""));
        assert!(versions.contains("data-component-version-action=\"deactivate\""));
        let viewer = render_view(&component, true);
        assert!(viewer.contains("data-component-render"));
        assert!(viewer.contains("data-component-kind=\"table\""));
        assert!(viewer.contains("href=\"/components/01980000-0002-7000-8000-000000000010/edit\""));
        assert!(COMPONENT_JS.contains("loadComponentRenderers"));

        let reader_versions = render_versions(&component, false);
        assert!(!reader_versions.contains("data-component-manage-only"));
        assert!(!reader_versions.contains("data-component-version-action"));
        let reader_viewer = render_view(&component, false);
        assert!(!reader_viewer.contains("/edit"));
        assert!(!reader_viewer.contains("/versions"));
    }

    #[test]
    fn authoring_and_directory_documents_preserve_the_accepted_ui_contract() {
        let component = serde_json::json!({
            "component_id":"01980000-0002-7000-8000-000000000010",
            "name":"Reference chart",
            "slug":"reference-chart",
            "versions":[{
                "component_version_id":"01980000-0001-7000-8000-000000000010",
                "dataset_reference":{"reference":{"resource_id":"01980000-0003-7000-8000-000000000010@1"}},
                "component_type":"bar",
                "publication_state":"published",
                "version_note":"Accepted visual",
                "config":{
                    "summary_field":"amount",
                    "summary_type":"none",
                    "category_field":"region",
                    "value_format":"decimal",
                    "number_of_points":100,
                    "category_labels":{"north":"North Region"},
                    "category_colors":{"north":"#3568d4"}
                }
            }]
        });
        let editor = render_form(Some(&component), &serde_json::Value::Null, Some("offline"));
        assert!(editor.contains("Displayed Fields"));
        assert!(editor.contains("aria-label=\"Available fields\""));
        assert!(editor.contains("Do not summarize"));
        assert!(editor.contains("Count rows"));
        assert!(editor.contains("Count non-empty values"));
        assert!(editor.contains("Count unique values"));
        assert!(editor.contains("component-editor__role-grid"));
        assert!(editor.contains("component-editor__role-card--measure"));
        assert!(editor.contains("Build the bars"));
        assert!(editor.contains("Measure help"));
        assert!(editor.contains("data-component-value-field"));
        assert!(editor.contains("data-component-value-missing-policy"));
        assert!(editor.contains("data-component-calculation-warning"));
        assert!(editor.contains(
            "Every category and series group must resolve to exactly one row. Preview and execution will report an error when duplicates exist."
        ));
        assert!(editor.contains("aria-label=\"Sort Field\""));
        assert!(editor.contains("Show help for Sort Field"));
        assert!(editor.contains(
            "Default: uses the order produced by the current grouping and summarization."
        ));
        assert!(editor.contains("<option value=\"\">Default</option>"));
        assert!(editor.contains("Summary Value: sorts by the summarized numeric value."));
        assert!(!editor.contains("Calculated value"));
        assert!(editor.contains("Decimal"));
        assert!(editor.contains("max=\"100\""));
        assert!(!editor.contains("max=\"1000\""));
        assert!(editor.contains("Validation Findings"));
        assert!(editor.contains("Retry Dataset metadata"));
        assert!(editor.contains("data-component-save-action=\"save_draft\" disabled"));
        assert!(editor.contains("Review component consumers"));
        assert!(editor.contains("Create New Version"));
        assert!(editor.contains("line_x_axis_label"));

        let directory = render_directory(&serde_json::json!([]), false);
        assert!(directory.contains("Search components by name"));
        assert!(directory.contains("role=\"menuitemradio\""));
        assert!(directory.contains("value=\"updating\""));
        assert!(directory.contains("Open component filters"));
        assert!(directory.contains("Filter components by kind"));
    }

    #[test]
    fn directory_ssr_exposes_drafts_and_actions_only_to_authorized_managers() {
        let public = serde_json::json!([{
            "component_id":"01980000-0002-7000-8000-000000000010",
            "name":"Published table",
            "slug":"published-table",
            "current_version":{
                "component_type":"table",
                "publication_state":"published"
            }
        }]);
        let manageable = serde_json::json!([{
            "component_id":"01980000-0002-7000-8000-000000000011",
            "name":"Manager draft",
            "slug":"manager-draft",
            "versions":[{
                "component_type":"bar",
                "publication_state":"draft"
            }]
        }]);
        let manager_projection = super::directory_projection(public.clone(), manageable);
        let manager = render_directory(&manager_projection, true);
        assert!(manager.contains("Manager draft"));
        assert!(manager.contains("data-status=\"draft\""));
        assert!(manager.contains("href=\"/components/manager-draft/edit\""));
        assert!(manager.contains("href=\"/components/new\""));
        assert!(!manager.contains("data-component-manage-only hidden"));

        let reader_projection = super::directory_projection(public, serde_json::json!([]));
        let reader = render_directory(&reader_projection, false);
        assert!(reader.contains("Published table"));
        assert!(!reader.contains("Manager draft"));
        assert!(!reader.contains("/components/new"));
        assert!(!reader.contains("/edit"));
        assert!(!reader.contains("/versions"));
    }
}
