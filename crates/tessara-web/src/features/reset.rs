use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;
#[cfg(feature = "hydrate")]
use std::{cell::Cell, cell::RefCell, rc::Rc};

use icons::{
    CalendarDays, ChevronDown, ChevronRight, CircleDot, ExternalLink, Hash, ListChecks, ListFilter,
    LockKeyhole, Mail, PanelRight, Pencil, Plus, Search, SquareCheckBig, TextCursorInput,
    TextQuote, Trash2, X,
};
use leptos::portal::Portal;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(feature = "hydrate")]
use wasm_bindgen::JsCast;
#[cfg(feature = "hydrate")]
use wasm_bindgen::closure::Closure;

use crate::infra::routing::{FormRouteParams, NodeRouteParams, require_route_params};
use crate::ui::components::{
    AppShell, Breadcrumb, BreadcrumbItem, BreadcrumbLink, BreadcrumbPage, BreadcrumbSeparator,
    Button, DataTable, DropdownMenu, EmptyState, InfoListTable, InfoRow, PageHeader,
    SearchableDataTable, StatusBadge, Tabs, TabsContent, TabsList, TabsTrigger, Timestamp,
};

#[derive(Clone, Copy)]
struct RouteMigration {
    name: &'static str,
    route: &'static str,
    href: &'static str,
    status: &'static str,
    rbac_status: &'static str,
}

const ROUTE_MIGRATIONS: [RouteMigration; 32] = [
    RouteMigration {
        name: "Home",
        route: "/",
        href: "/",
        status: "In Progress",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Login",
        route: "/login",
        href: "/login",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Organization",
        route: "/organization",
        href: "/organization",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Create Organization Node",
        route: "/organization/new",
        href: "/organization/new",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Organization Detail",
        route: "/organization/:node_id",
        href: "/organization/fb3fb3c8-2670-4c85-bcda-be59dd3aa119",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Edit Organization Node",
        route: "/organization/:node_id/edit",
        href: "/organization/fb3fb3c8-2670-4c85-bcda-be59dd3aa119/edit",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Forms",
        route: "/forms",
        href: "/forms",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Create Form",
        route: "/forms/new",
        href: "/forms/new",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Form Detail",
        route: "/forms/:form_id",
        href: "/forms/650af9e7-f428-4a4f-ae9c-7f4e1ca12edd",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Edit Form",
        route: "/forms/:form_id/edit",
        href: "/forms/650af9e7-f428-4a4f-ae9c-7f4e1ca12edd/edit",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Workflows",
        route: "/workflows",
        href: "/workflows",
        status: "Done",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Create Workflow",
        route: "/workflows/new",
        href: "/workflows/new",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Workflow Assignments",
        route: "/workflows/assignments",
        href: "/workflows/assignments",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Workflow Detail",
        route: "/workflows/:workflow_id",
        href: "/workflows/demo-intake-workflow",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Edit Workflow",
        route: "/workflows/:workflow_id/edit",
        href: "/workflows/demo-intake-workflow/edit",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Responses",
        route: "/responses",
        href: "/responses",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Start Response",
        route: "/responses/new",
        href: "/responses/new",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Response Detail",
        route: "/responses/:submission_id",
        href: "/responses/demo-submission",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Edit Response",
        route: "/responses/:submission_id/edit",
        href: "/responses/demo-submission/edit",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Components",
        route: "/components",
        href: "/components",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Component Detail",
        route: "/components/:component_ref",
        href: "/components/demo-component",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Dashboards",
        route: "/dashboards",
        href: "/dashboards",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Create Dashboard",
        route: "/dashboards/new",
        href: "/dashboards/new",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Dashboard Detail",
        route: "/dashboards/:dashboard_id",
        href: "/dashboards/demo-operations-dashboard",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Edit Dashboard",
        route: "/dashboards/:dashboard_id/edit",
        href: "/dashboards/demo-operations-dashboard/edit",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Datasets",
        route: "/datasets",
        href: "/datasets",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Dataset Detail",
        route: "/datasets/:dataset_id",
        href: "/datasets/demo-dataset",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Administration",
        route: "/administration",
        href: "/administration",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Users",
        route: "/administration/users",
        href: "/administration/users",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Node Types",
        route: "/administration/node-types",
        href: "/administration/node-types",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Roles",
        route: "/administration/roles",
        href: "/administration/roles",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Migration",
        route: "/migration",
        href: "/migration",
        status: "Pending",
        rbac_status: "Pending",
    },
];

#[component]
fn ResetRoute(
    active_route: &'static str,
    title: &'static str,
    route: &'static str,
    status: &'static str,
    next_step: &'static str,
) -> impl IntoView {
    view! {
        <AppShell active_route title>
            <section class="route-panel">
                <PageHeader title description="This route is registered in the native Leptos SSR shell. Functional code will be reintroduced route-by-route from the reference worktree.">
                    <StatusBadge label=status/>
                </PageHeader>
                <InfoListTable>
                    <InfoRow label="Route" value=route/>
                    <InfoRow label="Rendering" value="Native Leptos SSR component"/>
                    <InfoRow label="Next step" value=next_step/>
                </InfoListTable>
                <EmptyState
                    title="Implementation reset baseline"
                    message="The transitional shell and string-rendered route UI have been removed from active routing."
                />
            </section>
        </AppShell>
    }
}

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <AppShell active_route="home" title="Home">
            <section class="route-panel">
                <PageHeader title="Native UI Migration" description="Routes are registered in the clean native Leptos SSR shell and will be rebuilt in navigation order."/>
                <RouteMigrationOverview/>
            </section>
        </AppShell>
    }
}

#[component]
fn RouteMigrationOverview() -> impl IntoView {
    let route_rows = ROUTE_MIGRATIONS
        .iter()
        .map(|route| {
            view! {
                <tr>
                    <th scope="row">{route.name}</th>
                    <td><a href=route.href>{route.route}</a></td>
                    <td><StatusBadge label=route.status/></td>
                    <td><StatusBadge label=route.rbac_status/></td>
                </tr>
            }
        })
        .collect_view();

    let route_cards = ROUTE_MIGRATIONS
        .iter()
        .map(|route| {
            view! {
                <article class="route-migration-card">
                    <div class="route-migration-card__header">
                        <h3>{route.name}</h3>
                        <a href=route.href>{route.route}</a>
                    </div>
                    <dl class="route-migration-card__meta">
                        <div>
                            <dt>"Status"</dt>
                            <dd><StatusBadge label=route.status/></dd>
                        </div>
                        <div>
                            <dt>"RBAC"</dt>
                            <dd><StatusBadge label=route.rbac_status/></dd>
                        </div>
                    </dl>
                </article>
            }
        })
        .collect_view();

    view! {
        <div class="route-migration-overview">
            <div class="route-migration-overview__table">
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Name"</th>
                            <th scope="col">"Route"</th>
                            <th scope="col">"Status"</th>
                            <th scope="col">"RBAC"</th>
                        </tr>
                    </thead>
                    <tbody>{route_rows}</tbody>
                </DataTable>
            </div>
            <div class="route-migration-overview__cards" aria-label="Route migration status">
                {route_cards}
            </div>
        </div>
    }
}

#[component]
pub fn LoginPage() -> impl IntoView {
    let email = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let is_submitting = RwSignal::new(false);
    let error_message = RwSignal::new(None::<String>);

    let submit = move |event: leptos::ev::SubmitEvent| {
        event.prevent_default();
        error_message.set(None);
        is_submitting.set(true);
        submit_login(
            email.get_untracked(),
            password.get_untracked(),
            error_message,
            is_submitting,
        );
    };

    view! {
        <main class="login-shell">
            <section class="login-panel blurred-surface" aria-labelledby="login-title">
                <a class="login-brand" href="/" aria-label="Tessara home">
                    <img src="/assets/tessara-icon-256.svg" alt=""/>
                    <span>"Tessara"</span>
                </a>
                <div class="login-panel__header">
                    <h1 id="login-title">"Welcome back"</h1>
                    <p>"Sign in to continue to the Tessara workspace."</p>
                </div>
                <form class="login-form" on:submit=submit>
                    <label class="login-field">
                        <span class="login-field__label">"Email"</span>
                        <span class="login-input-shell">
                            <Mail class="login-field__icon"/>
                            <input
                                type="email"
                                autocomplete="username"
                                placeholder="admin@tessara.local"
                                required
                                prop:value=move || email.get()
                                on:input=move |event| email.set(event_target_value(&event))
                            />
                        </span>
                    </label>
                    <label class="login-field">
                        <span class="login-field__label">"Password"</span>
                        <span class="login-input-shell">
                            <LockKeyhole class="login-field__icon"/>
                            <input
                                type="password"
                                autocomplete="current-password"
                                placeholder="Password"
                                required
                                prop:value=move || password.get()
                                on:input=move |event| password.set(event_target_value(&event))
                            />
                        </span>
                    </label>
                    <Show when=move || error_message.get().is_some()>
                        <p class="login-error" role="alert">
                            {move || error_message.get().unwrap_or_default()}
                        </p>
                    </Show>
                    <button class="button login-submit" type="submit" disabled=move || is_submitting.get()>
                        {move || if is_submitting.get() { "Signing in..." } else { "Sign In" }}
                    </button>
                </form>
            </section>
        </main>
    }
}

fn submit_login(
    email: String,
    password: String,
    error_message: RwSignal<Option<String>>,
    is_submitting: RwSignal<bool>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let body = serde_json::json!({
                "email": email,
                "password": password,
            })
            .to_string();

            let response = match gloo_net::http::Request::post("/api/auth/login")
                .header("Content-Type", "application/json")
                .body(body)
            {
                Ok(request) => request.send().await,
                Err(error) => Err(error),
            };

            match response {
                Ok(response) if response.ok() => {
                    if let Some(window) = web_sys::window() {
                        let _ = window.location().set_href("/");
                    }
                }
                Ok(_) => {
                    error_message.set(Some("Email or password did not match.".into()));
                    is_submitting.set(false);
                }
                Err(_) => {
                    error_message.set(Some("Could not reach Tessara. Try again.".into()));
                    is_submitting.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (email, password, error_message, is_submitting);
    }
}

#[component]
pub fn OrganizationPage() -> impl IntoView {
    let tree = RwSignal::new(Vec::<OrganizationTreeNode>::new());
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let expanded_nodes = RwSignal::new(HashSet::<String>::new());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);
    let detail = RwSignal::new(None::<OrganizationNodeDetail>);
    let detail_is_loading = RwSignal::new(false);
    let detail_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_organization_tree(tree, node_types, expanded_nodes, is_loading, load_error);
    });

    view! {
        <AppShell active_route="organization" title="Organization">
            <section class="route-panel organization-page">
                <PageHeader
                    title="Organization Explorer"
                >
                    <Button label="Create Node" href="/organization/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading hierarchy"</h3>
                                <p>"Fetching visible organization nodes."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Organization unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if tree.get().is_empty() {
                        view! {
                            <EmptyState
                                title="No visible organization nodes"
                                message="Create a node or update account scope to populate this explorer."
                            />
                        }
                        .into_any()
                    } else {
                        view! {
                            {organization_tree_view(
                                tree.get(),
                                node_types.get(),
                                expanded_nodes,
                                detail,
                                detail_is_loading,
                                detail_error,
                                0,
                                Vec::new(),
                            )}
                        }
                        .into_any()
                    }
                }}
                <OrganizationDetailSheet
                    detail
                    is_loading=detail_is_loading
                    error=detail_error
                />
            </section>
        </AppShell>
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct OrganizationNode {
    id: String,
    node_type_name: String,
    node_type_singular_label: String,
    node_type_plural_label: String,
    parent_node_id: Option<String>,
    parent_node_name: Option<String>,
    node_type_id: String,
    name: String,
    #[serde(default)]
    metadata: Value,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct OrganizationNodeDetail {
    id: String,
    node_type_id: String,
    node_type_name: String,
    node_type_singular_label: String,
    node_type_plural_label: String,
    parent_node_id: Option<String>,
    parent_node_name: Option<String>,
    name: String,
    #[serde(default)]
    metadata: Value,
    #[serde(default)]
    related_forms: Vec<NodeFormLink>,
    #[serde(default)]
    related_responses: Vec<NodeSubmissionLink>,
    #[serde(default)]
    related_dashboards: Vec<NodeDashboardLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeFormLink {
    form_id: String,
    form_name: String,
    form_slug: String,
    published_version_count: i64,
    active_version_label: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeSubmissionLink {
    submission_id: String,
    form_name: String,
    version_label: String,
    status: String,
    created_at: String,
    submitted_at: Option<String>,
    submitted_by: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeDashboardLink {
    dashboard_id: String,
    dashboard_name: String,
    component_count: i64,
    description: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeTypeCatalogEntry {
    id: String,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    is_root_type: bool,
    node_count: i64,
    #[serde(default)]
    parent_relationships: Vec<NodeTypePeerLink>,
    #[serde(default)]
    child_relationships: Vec<NodeTypePeerLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeTypePeerLink {
    node_type_id: String,
    node_type_name: String,
    node_type_slug: String,
    singular_label: String,
    plural_label: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct NodeTypeDefinition {
    id: String,
    name: String,
    slug: String,
    singular_label: String,
    plural_label: String,
    #[serde(default)]
    metadata_fields: Vec<NodeMetadataFieldSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct NodeMetadataFieldSummary {
    key: String,
    label: String,
    field_type: String,
    required: bool,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct IdResponse {
    id: String,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct CreateNodePayload {
    node_type_id: String,
    parent_node_id: Option<String>,
    name: String,
    metadata: serde_json::Map<String, Value>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct UpdateNodePayload {
    parent_node_id: Option<String>,
    name: String,
    metadata: serde_json::Map<String, Value>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct CreateFormPayload {
    name: String,
    slug: String,
    scope_node_type_id: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct UpdateFormPayload {
    name: String,
    slug: String,
    scope_node_type_id: Option<String>,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct CreateFormSectionPayload {
    title: String,
    position: i32,
    description: String,
    column_count: i32,
}

#[derive(Serialize)]
#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
struct CreateFormFieldPayload {
    section_id: String,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    position: i32,
    grid_row: i32,
    grid_column: i32,
    grid_width: i32,
    grid_height: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct FormBuilderSectionDraft {
    id: usize,
    remote_id: Option<String>,
    title: String,
    description: String,
    column_count: i32,
    default_column_width: i32,
    position: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct FormBuilderFieldDraft {
    id: usize,
    remote_id: Option<String>,
    section_id: usize,
    label: String,
    key: String,
    field_type: String,
    required: bool,
    grid_row: i32,
    grid_column: i32,
    grid_width: i32,
    grid_height: i32,
    key_was_edited: bool,
}

const FORM_BUILDER_COLUMN_COUNT: i32 = 12;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct FormBuilderDragPreview {
    field_id: usize,
    section_id: usize,
    row: i32,
    column: i32,
}

#[derive(Clone, Copy)]
enum FormBuilderResizeAxis {
    Width,
    Height,
}

fn set_form_builder_drag_preview(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    next_preview: FormBuilderDragPreview,
) {
    if builder_drag_preview.get_untracked() != Some(next_preview) {
        builder_drag_preview.set(Some(next_preview));
    }
}

#[cfg(feature = "hydrate")]
fn form_builder_grid_cell_from_drag_event(
    event: &leptos::ev::DragEvent,
) -> Option<(i32, i32, String)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let cell = target.closest(".form-builder-grid-cell").ok().flatten()?;
    let row = cell.get_attribute("data-row")?.parse::<i32>().ok()?;
    let column = cell.get_attribute("data-column")?.parse::<i32>().ok()?;
    Some((row, column, cell.id()))
}

#[cfg(not(feature = "hydrate"))]
fn form_builder_grid_cell_from_drag_event(
    _event: &leptos::ev::DragEvent,
) -> Option<(i32, i32, String)> {
    None
}

#[cfg(feature = "hydrate")]
fn form_builder_grid_cell_from_pointer(
    event: &leptos::ev::DragEvent,
    row_count: i32,
) -> Option<(i32, i32, String)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let grid = target.closest(".form-builder-layout-grid").ok().flatten()?;
    let grid_id = grid.get_attribute("data-section-id")?;
    let bounds_fn = js_sys::Reflect::get(&grid, &"getBoundingClientRect".into())
        .ok()?
        .dyn_into::<js_sys::Function>()
        .ok()?;
    let bounds = bounds_fn.call0(&grid).ok()?;
    let left = js_sys::Reflect::get(&bounds, &"left".into())
        .ok()?
        .as_f64()?;
    let top = js_sys::Reflect::get(&bounds, &"top".into())
        .ok()?
        .as_f64()?;
    let width = js_sys::Reflect::get(&bounds, &"width".into())
        .ok()?
        .as_f64()?;
    let height = js_sys::Reflect::get(&bounds, &"height".into())
        .ok()?
        .as_f64()?;

    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    let row_count = row_count.max(1);
    let x = (f64::from(event.client_x()) - left).clamp(0.0, width - 1.0);
    let y = (f64::from(event.client_y()) - top).clamp(0.0, height - 1.0);
    let column_width = width / f64::from(FORM_BUILDER_COLUMN_COUNT);
    let row_height = height / f64::from(row_count);
    let column = ((x / column_width).floor() as i32 + 1).clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let row = ((y / row_height).floor() as i32 + 1).clamp(1, row_count);

    Some((
        row,
        column,
        format!("form-builder-section-{grid_id}-cell-r{row}-c{column}"),
    ))
}

#[cfg(not(feature = "hydrate"))]
fn form_builder_grid_cell_from_pointer(
    _event: &leptos::ev::DragEvent,
    _row_count: i32,
) -> Option<(i32, i32, String)> {
    None
}

#[cfg(feature = "hydrate")]
fn form_builder_add_tile_from_click_event(event: &leptos::ev::MouseEvent) -> Option<(i32, i32)> {
    let target = event.target()?.dyn_into::<web_sys::Element>().ok()?;
    let add_cell = target
        .closest(".form-builder-grid-cell[data-empty]")
        .ok()
        .flatten()?;
    let row = add_cell.get_attribute("data-row")?.parse::<i32>().ok()?;
    let column = add_cell.get_attribute("data-column")?.parse::<i32>().ok()?;
    Some((row, column))
}

#[cfg(not(feature = "hydrate"))]
fn form_builder_add_tile_from_click_event(_event: &leptos::ev::MouseEvent) -> Option<(i32, i32)> {
    None
}

#[cfg(feature = "hydrate")]
fn clear_form_builder_drag_target_dom() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(targets) = document.query_selector_all(".form-builder-grid-cell.is-drop-target") else {
        return;
    };

    for index in 0..targets.length() {
        if let Some(target) = targets.item(index) {
            if let Ok(element) = target.dyn_into::<web_sys::Element>() {
                let _ = element.class_list().remove_1("is-drop-target");
            }
        }
    }
}

#[cfg(feature = "hydrate")]
fn set_form_builder_drag_target_dom(target_id: &str) {
    clear_form_builder_drag_target_dom();

    if let Some(element) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id(target_id))
    {
        let _ = element.class_list().add_1("is-drop-target");
    }
}

fn clear_form_builder_drag_intent(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
) {
    pending_builder_drag_preview.set(None);
    builder_drag_preview.set(None);

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (
            web_sys::window(),
            builder_drag_preview_timeout.get_untracked(),
        ) {
            window.clear_timeout_with_handle(timeout_handle);
        }
        clear_form_builder_drag_target_dom();
    }

    builder_drag_preview_timeout.set(None);
}

fn schedule_form_builder_drag_preview(
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    _builder_drag_preview_timeout: RwSignal<Option<i32>>,
    next_preview: FormBuilderDragPreview,
    target_id: String,
) {
    if builder_drag_preview.get_untracked() == Some(next_preview) {
        return;
    }

    pending_builder_drag_preview.set(Some(next_preview));

    #[cfg(feature = "hydrate")]
    {
        if let (Some(window), Some(timeout_handle)) = (
            web_sys::window(),
            _builder_drag_preview_timeout.get_untracked(),
        ) {
            window.clear_timeout_with_handle(timeout_handle);
        }

        let pending_preview = pending_builder_drag_preview;
        let preview_signal = builder_drag_preview;
        let timeout_signal = _builder_drag_preview_timeout;
        let callback = Closure::wrap(Box::new(move || {
            if pending_preview.get_untracked() == Some(next_preview) {
                set_form_builder_drag_preview(preview_signal, next_preview);
                set_form_builder_drag_target_dom(&target_id);
            }
            timeout_signal.set(None);
        }) as Box<dyn FnMut()>);

        if let Some(window) = web_sys::window() {
            if let Ok(timeout_handle) = window
                .set_timeout_with_callback_and_timeout_and_arguments_0(
                    callback.as_ref().unchecked_ref(),
                    1_000,
                )
            {
                _builder_drag_preview_timeout.set(Some(timeout_handle));
                callback.forget();
                return;
            }
        }
    }

    #[cfg(not(feature = "hydrate"))]
    {
        set_form_builder_drag_preview(builder_drag_preview, next_preview);
        let _ = target_id;
    }
}

fn commit_form_builder_drag_preview(
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) {
    let preview = builder_drag_preview.get_untracked();

    if let Some(preview) = preview {
        builder_fields.update(|fields| {
            *fields = form_builder_reflow_section_fields(fields, preview);
        });
        suppress_builder_field_click.set(Some(preview.field_id));
    }

    clear_form_builder_drag_intent(
        builder_drag_preview,
        pending_builder_drag_preview,
        builder_drag_preview_timeout,
    );
    dragged_builder_field.set(None);
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormSummary {
    id: String,
    name: String,
    slug: String,
    scope_node_type_name: Option<String>,
    #[serde(default)]
    versions: Vec<FormVersionSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormVersionSummary {
    id: String,
    version_label: Option<String>,
    status: String,
    version_major: Option<i32>,
    version_minor: Option<i32>,
    version_patch: Option<i32>,
    compatibility_group_name: Option<String>,
    published_at: Option<String>,
    field_count: i64,
    semantic_bump: Option<String>,
    started_new_major_line: Option<bool>,
    #[serde(default)]
    assignment_nodes: Vec<FormVersionAssignmentNodeSummary>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormVersionAssignmentNodeSummary {
    node_id: String,
    node_name: String,
    parent_node_id: Option<String>,
    node_path: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct WorkflowSummary {
    id: String,
    form_id: String,
    form_name: String,
    form_slug: String,
    name: String,
    slug: String,
    description: String,
    current_version_id: Option<String>,
    current_version_label: Option<String>,
    current_form_version_id: Option<String>,
    current_status: Option<String>,
    assignment_count: i64,
    version_count: i64,
    #[serde(default)]
    assignment_node_names: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormDefinition {
    id: String,
    name: String,
    slug: String,
    scope_node_type_id: Option<String>,
    scope_node_type_name: Option<String>,
    #[serde(default)]
    versions: Vec<FormVersionSummary>,
    #[serde(default)]
    workflows: Vec<FormWorkflowLink>,
    #[serde(default)]
    reports: Vec<FormReportLink>,
    #[serde(default)]
    dataset_sources: Vec<FormDatasetSourceLink>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormWorkflowLink {
    id: String,
    name: String,
    slug: String,
    current_version_label: Option<String>,
    current_status: Option<String>,
    assignment_count: i64,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormReportLink {
    id: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct FormDatasetSourceLink {
    dataset_id: String,
    dataset_name: String,
    source_alias: String,
    selection_rule: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RenderedForm {
    form_version_id: String,
    form_id: String,
    form_name: String,
    version_label: Option<String>,
    status: String,
    #[serde(default)]
    sections: Vec<RenderedSection>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RenderedSection {
    id: String,
    title: String,
    description: String,
    column_count: i32,
    position: i32,
    #[serde(default)]
    fields: Vec<RenderedField>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
struct RenderedField {
    id: String,
    key: String,
    label: String,
    field_type: String,
    required: bool,
    position: i32,
    grid_row: i32,
    grid_column: i32,
    grid_width: i32,
    grid_height: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct FormAttachmentLink {
    href: String,
    label: String,
    title: String,
}

#[derive(Clone, Debug, PartialEq)]
struct FormsAttachedNodesSheetData {
    form_name: String,
    form_href: String,
    nodes: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
struct WorkflowAssignedNodesSheetData {
    workflow_name: String,
    workflow_href: String,
    nodes: Vec<FormAttachmentLink>,
}

#[derive(Clone, Debug, PartialEq)]
struct FormNodeFilterOption {
    id: String,
    name: String,
    parent_node_id: Option<String>,
    path: String,
    depth: usize,
}

#[derive(Clone, Debug, PartialEq)]
struct OrganizationTreeNode {
    node: OrganizationNode,
    children: Vec<OrganizationTreeNode>,
}

#[derive(Clone, Debug, PartialEq)]
struct CreateChildLink {
    href: String,
    label: String,
}

#[derive(Clone, Debug, PartialEq)]
struct ParentNodeOption {
    id: String,
    label: String,
}

fn organization_tree_view(
    nodes: Vec<OrganizationTreeNode>,
    node_types: Vec<NodeTypeCatalogEntry>,
    expanded_nodes: RwSignal<HashSet<String>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    detail_is_loading: RwSignal<bool>,
    detail_error: RwSignal<Option<String>>,
    depth: usize,
    lineage: Vec<String>,
) -> AnyView {
    view! {
        <div class="organization-tree" role=if depth == 0 { "tree" } else { "group" }>
            {nodes
                .into_iter()
                .map(|branch| {
                    organization_branch_view(
                        branch,
                        node_types.clone(),
                        expanded_nodes,
                        detail,
                        detail_is_loading,
                        detail_error,
                        depth,
                        lineage.clone(),
                    )
                })
                .collect_view()}
        </div>
    }
    .into_any()
}

fn organization_branch_view(
    branch: OrganizationTreeNode,
    node_types: Vec<NodeTypeCatalogEntry>,
    expanded_nodes: RwSignal<HashSet<String>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    detail_is_loading: RwSignal<bool>,
    detail_error: RwSignal<Option<String>>,
    depth: usize,
    lineage: Vec<String>,
) -> AnyView {
    let node = branch.node;
    let children = branch.children;
    let node_id = node.id.clone();
    let row_id = node.id.clone();
    let row_class_id = node.id.clone();
    let child_link_node_type_id = node.node_type_id.clone();
    let expanded_id = node.id.clone();
    let toggle_icon_id = node.id.clone();
    let child_visibility_id = node.id.clone();
    let details_id = node.id.clone();
    let action_label = format!("Open actions for {}", node.name);
    let has_children = !children.is_empty();
    let child_count = children.len();
    let child_lineage = {
        let mut next_lineage = lineage.clone();
        next_lineage.push(node.id.clone());
        next_lineage
    };
    let row_class = move || {
        if has_children && expanded_nodes.with(|nodes| nodes.contains(&row_class_id)) {
            "organization-node is-open"
        } else {
            "organization-node"
        }
    };
    let edit_href = format!("/organization/{node_id}/edit");
    let create_links = child_create_links(&child_link_node_type_id, &node_types, &node_id);
    let child_label = visible_child_label(child_count);

    view! {
        <section class="organization-branch" style=format!("--organization-depth: {depth};")>
            <div class=row_class>
                <button
                    class="organization-node__main"
                    type="button"
                    aria-expanded=move || {
                        (has_children && expanded_nodes.with(|nodes| nodes.contains(&expanded_id))).to_string()
                    }
                    on:click=move |_| {
                        if has_children {
                            toggle_organization_branch(
                                expanded_nodes,
                                row_id.clone(),
                                lineage.clone(),
                            );
                        }
                    }
                >
                    <span class="organization-node__toggle" aria-hidden="true">
                        {move || {
                            if has_children && expanded_nodes.with(|nodes| nodes.contains(&toggle_icon_id)) {
                                view! { <ChevronDown class="organization-node__toggle-icon"/> }.into_any()
                            } else {
                                view! { <ChevronRight class="organization-node__toggle-icon"/> }.into_any()
                            }
                        }}
                    </span>
                    <span class="organization-node__copy">
                        <span class="organization-node__type">{node.node_type_singular_label}</span>
                        <strong>{node.name}</strong>
                        <span class="organization-node__context">
                            {node.parent_node_name.unwrap_or_else(|| "Top-level".to_string())}
                        </span>
                    </span>
                    <span class="organization-node__count">{child_label}</span>
                </button>
                <DropdownMenu label=action_label>
                    <button
                        class="dropdown-menu__item"
                        type="button"
                        role="menuitem"
                        on:click=move |_| {
                            load_organization_detail(
                                details_id.clone(),
                                detail,
                                detail_is_loading,
                                detail_error,
                            );
                        }
                    >
                        <PanelRight class="dropdown-menu__item-icon"/>
                        <span>"Details"</span>
                    </button>
                    <a class="dropdown-menu__item" role="menuitem" href=edit_href>
                        <Pencil class="dropdown-menu__item-icon"/>
                        <span>"Edit"</span>
                    </a>
                    {create_links
                        .into_iter()
                        .map(|link| {
                            view! {
                                <a class="dropdown-menu__item" role="menuitem" href=link.href>
                                    <Plus class="dropdown-menu__item-icon"/>
                                    <span>{link.label}</span>
                                </a>
                            }
                        })
                        .collect_view()}
                </DropdownMenu>
            </div>

            <Show when=move || has_children && expanded_nodes.with(|nodes| nodes.contains(&child_visibility_id))>
                {organization_tree_view(
                    children.clone(),
                    node_types.clone(),
                    expanded_nodes,
                    detail,
                    detail_is_loading,
                    detail_error,
                    depth + 1,
                    child_lineage.clone(),
                )}
            </Show>
        </section>
    }
    .into_any()
}

#[component]
fn OrganizationDetailSheet(
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    is_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let close = move |_| {
        detail.set(None);
        error.set(None);
        is_loading.set(false);
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some() || is_loading.get() || error.get().is_some()>
                <section class="sheet-overlay organization-detail-overlay" aria-label="Organization detail overlay">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close details" on:click=close></button>
                    <aside class="sheet-panel blurred-surface organization-detail-sheet" role="dialog" aria-modal="true" aria-label="Organization details">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|node_detail| {
                                        let href = format!("/organization/{}", node_detail.id);
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=href aria-label="Open detail page" title="Open detail page">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(|| view! {}.into_any())
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close details" title="Close details" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            if is_loading.get() {
                                view! {
                                    <div class="sheet-panel__state" aria-live="polite">
                                        <h2>"Loading details"</h2>
                                        <p>"Fetching organization node details."</p>
                                    </div>
                                }
                                .into_any()
                            } else if let Some(message) = error.get() {
                                view! {
                                    <div class="sheet-panel__state is-error" role="alert">
                                        <h2>"Details unavailable"</h2>
                                        <p>{message}</p>
                                    </div>
                                }
                                .into_any()
                            } else if let Some(node_detail) = detail.get() {
                                view! { <OrganizationDetailContent detail=node_detail/> }.into_any()
                            } else {
                                view! {}.into_any()
                            }
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
fn OrganizationDetailContent(detail: OrganizationNodeDetail) -> impl IntoView {
    let metadata_rows = metadata_rows(&detail.metadata);
    let node_type = detail.node_type_singular_label.clone();

    view! {
        <header class="sheet-panel__header">
            <p>{format!("{} Detail", node_type)}</p>
            <h2>{detail.name.clone()}</h2>
        </header>
        <section class="sheet-panel__section">
            <h3>"Details"</h3>
            <DynamicInfoTable rows=vec![
                ("Parent".to_string(), detail.parent_node_name.clone().unwrap_or_else(|| "Top-level".to_string())),
                ("Type".to_string(), detail.node_type_name.clone()),
                ("Plural".to_string(), detail.node_type_plural_label.clone()),
            ]/>
        </section>
        <section class="sheet-panel__section">
            <h3>"Metadata"</h3>
            {if metadata_rows.is_empty() {
                view! { <p class="muted">"No metadata recorded."</p> }.into_any()
            } else {
                view! { <DynamicInfoTable rows=metadata_rows/> }.into_any()
            }}
        </section>
        <section class="sheet-panel__section">
            <h3>"Related Work"</h3>
            <RelatedWorkSummary detail cards_only=true/>
        </section>
    }
}

#[component]
fn OrganizationDetailFullContent(detail: OrganizationNodeDetail) -> impl IntoView {
    let metadata_rows = metadata_rows(&detail.metadata);
    let node_type = detail.node_type_singular_label.clone();

    view! {
        <div class="organization-detail-content">
            <header class="organization-detail-content__header">
                <p>{format!("{} Detail", node_type)}</p>
                <h3>{detail.name.clone()}</h3>
            </header>
            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Details"</h3>
                    <DynamicInfoTable rows=vec![
                        ("Parent".to_string(), detail.parent_node_name.clone().unwrap_or_else(|| "Top-level".to_string())),
                        ("Type".to_string(), detail.node_type_name.clone()),
                        ("Plural".to_string(), detail.node_type_plural_label.clone()),
                    ]/>
                </section>
                <section class="organization-detail-card">
                    <h3>"Metadata"</h3>
                    {if metadata_rows.is_empty() {
                        view! { <p class="muted">"No metadata recorded."</p> }.into_any()
                    } else {
                        view! { <DynamicInfoTable rows=metadata_rows/> }.into_any()
                    }}
                </section>
                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Related Work"</h3>
                    <RelatedWorkSummary detail/>
                </section>
            </div>
        </div>
    }
}

#[component]
fn DynamicInfoTable(rows: Vec<(String, String)>) -> impl IntoView {
    view! {
        <table class="info-list-table">
            <tbody>
                {rows
                    .into_iter()
                    .map(|(label, value)| view! {
                        <tr>
                            <th scope="row">{label}</th>
                            <td>{value}</td>
                        </tr>
                    })
                    .collect_view()}
            </tbody>
        </table>
    }
}

#[component]
fn RelatedWorkSummary(
    detail: OrganizationNodeDetail,
    #[prop(optional)] cards_only: bool,
) -> impl IntoView {
    let active_tab = RwSignal::new("forms".to_string());
    let summary_class = if cards_only {
        "related-work-summary related-work-summary--cards-only"
    } else {
        "related-work-summary"
    };
    let forms_count = detail.related_forms.len();
    let responses_count = detail.related_responses.len();
    let dashboards_count = detail.related_dashboards.len();

    view! {
        <div class=summary_class>
            <Tabs active=active_tab>
                <TabsList>
                    <TabsTrigger active=active_tab value="forms">
                        {format!("Forms ({forms_count})")}
                    </TabsTrigger>
                    <TabsTrigger active=active_tab value="responses">
                        {format!("Responses ({responses_count})")}
                    </TabsTrigger>
                    <TabsTrigger active=active_tab value="dashboards">
                        {format!("Dashboards ({dashboards_count})")}
                    </TabsTrigger>
                </TabsList>
                <TabsContent active=active_tab value="forms">
                    <RelatedFormsTable forms=detail.related_forms/>
                </TabsContent>
                <TabsContent active=active_tab value="responses">
                    <RelatedResponsesTable responses=detail.related_responses/>
                </TabsContent>
                <TabsContent active=active_tab value="dashboards">
                    <RelatedDashboardsTable dashboards=detail.related_dashboards/>
                </TabsContent>
            </Tabs>
        </div>
    }
}

#[component]
fn RelatedFormsTable(forms: Vec<NodeFormLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let forms_for_table = forms.clone();
    let forms_for_cards = forms;
    let filtered_forms = move || {
        let query = search.get();
        forms_for_table
            .iter()
            .filter(|form| {
                text_matches(
                    &query,
                    &[
                        &form.form_name,
                        &form.form_slug,
                        form.active_version_label.as_deref().unwrap_or(""),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let filtered_form_cards = move || {
        let query = search.get();
        forms_for_cards
            .iter()
            .filter(|form| {
                text_matches(
                    &query,
                    &[
                        &form.form_name,
                        &form.form_slug,
                        form.active_version_label.as_deref().unwrap_or(""),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search forms" placeholder="Search related forms" search>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">"Slug"</th>
                        <th scope="col">"Active version"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_forms();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="3">"No Related Forms to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            rows
                                .into_iter()
                                .map(|form| {
                                    let href = format!("/forms/{}", form.form_id);
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{form.form_name}</a>
                                            </th>
                                            <td>{form.form_slug}</td>
                                            <td>{form.active_version_label.unwrap_or_else(|| "-".to_string())}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_form_cards();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Forms to Display"</p> }.into_any()
                    } else {
                        rows
                            .into_iter()
                            .map(|form| {
                                let href = format!("/forms/{}", form.form_id);
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=href>{form.form_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Slug"</dt>
                                                <dd>{form.form_slug}</dd>
                                            </div>
                                            <div>
                                                <dt>"Active version"</dt>
                                                <dd>{form.active_version_label.unwrap_or_else(|| "-".to_string())}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn RelatedResponsesTable(responses: Vec<NodeSubmissionLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let responses_for_table = responses.clone();
    let responses_for_cards = responses;
    let filtered_responses = move || {
        let query = search.get();
        let status = status_filter.get();
        responses_for_table
            .iter()
            .filter(|response| status == "all" || response.status == status)
            .filter(|response| {
                text_matches(
                    &query,
                    &[
                        &response.form_name,
                        &response.version_label,
                        &response.status,
                        response.submitted_by.as_deref().unwrap_or("Unknown"),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let filtered_response_cards = move || {
        let query = search.get();
        let status = status_filter.get();
        responses_for_cards
            .iter()
            .filter(|response| status == "all" || response.status == status)
            .filter(|response| {
                text_matches(
                    &query,
                    &[
                        &response.form_name,
                        &response.version_label,
                        &response.status,
                        response.submitted_by.as_deref().unwrap_or("Unknown"),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="searchable-data-table related-work-responsive-table">
            <div class="searchable-data-table__toolbar related-work-mobile-toolbar">
                <label class="searchable-data-table__search searchable-data-table__control">
                    <Search class="searchable-data-table__control-icon"/>
                    <span class="sr-only">"Search responses"</span>
                    <input
                        type="search"
                        placeholder="Search related responses"
                        prop:value=move || search.get()
                        on:input=move |event| search.set(event_target_value(&event))
                    />
                </label>
                <div class="related-work-mobile-filter">
                    <StatusFilterHeader status_filter compact_control=true/>
                </div>
            </div>
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">"Version"</th>
                        <th scope="col">
                            <StatusFilterHeader status_filter/>
                        </th>
                        <th scope="col">"Submitted Date"</th>
                        <th scope="col">"Submitted By"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_responses();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="5">"No Related Responses to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            rows
                                .into_iter()
                                .map(|response| {
                                    let href = format!("/responses/{}", response.submission_id);
                                    let submitted_date = response
                                        .submitted_at
                                        .clone()
                                        .unwrap_or_else(|| response.created_at.clone());
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{response.form_name}</a>
                                            </th>
                                            <td>{response.version_label}</td>
                                            <td>{sentence_label(&response.status)}</td>
                                            <td><Timestamp value=submitted_date/></td>
                                            <td>{response.submitted_by.unwrap_or_else(|| "Unknown".to_string())}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </DataTable>
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_response_cards();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Responses to Display"</p> }.into_any()
                    } else {
                        rows
                            .into_iter()
                            .map(|response| {
                                let href = format!("/responses/{}", response.submission_id);
                                let submitted_date = response
                                    .submitted_at
                                    .clone()
                                    .unwrap_or_else(|| response.created_at.clone());
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=href>{response.form_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Version"</dt>
                                                <dd>{response.version_label}</dd>
                                            </div>
                                            <div>
                                                <dt>"Status"</dt>
                                                <dd>{sentence_label(&response.status)}</dd>
                                            </div>
                                            <div>
                                                <dt>"Submitted Date"</dt>
                                                <dd><Timestamp value=submitted_date/></dd>
                                            </div>
                                            <div>
                                                <dt>"Submitted By"</dt>
                                                <dd>{response.submitted_by.unwrap_or_else(|| "Unknown".to_string())}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn StatusFilterHeader(
    status_filter: RwSignal<String>,
    #[prop(optional)] compact_control: bool,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_class = move || {
        if is_open.get() {
            "data-table-filter is-open"
        } else {
            "data-table-filter"
        }
    };
    let button_label = move || {
        let current = status_filter.get();
        if current == "all" {
            "Filter Status".to_string()
        } else {
            format!("Filter Status: {}", sentence_label(&current))
        }
    };
    let trigger_class = move || {
        let size_class = if compact_control {
            " icon-button--compact-control"
        } else {
            ""
        };
        let filtered_class = if status_filter.get() == "all" {
            ""
        } else {
            " is-filtered"
        };
        format!("icon-button{size_class} data-table-filter__trigger{filtered_class}")
    };

    view! {
        <div class=menu_class>
            <span>"Status"</span>
            <button
                class=trigger_class
                type="button"
                aria-label=button_label
                title=button_label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
            </button>
            <button
                class="data-table-filter__scrim"
                type="button"
                aria-label="Close status filter"
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="data-table-filter__menu blurred-surface" role="menu">
                <button
                    class=move || filter_option_class(&status_filter.get(), "all")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (status_filter.get() == "all").to_string()
                    on:click=move |_| {
                        status_filter.set("all".to_string());
                        is_open.set(false);
                    }
                >
                    "All statuses"
                </button>
                <button
                    class=move || filter_option_class(&status_filter.get(), "draft")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (status_filter.get() == "draft").to_string()
                    on:click=move |_| {
                        status_filter.set("draft".to_string());
                        is_open.set(false);
                    }
                >
                    "Draft"
                </button>
                <button
                    class=move || filter_option_class(&status_filter.get(), "submitted")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (status_filter.get() == "submitted").to_string()
                    on:click=move |_| {
                        status_filter.set("submitted".to_string());
                        is_open.set(false);
                    }
                >
                    "Submitted"
                </button>
            </div>
        </div>
    }
}

fn filter_option_class(current: &str, value: &str) -> &'static str {
    if current == value {
        "data-table-filter__option is-active"
    } else {
        "data-table-filter__option"
    }
}

#[component]
fn FilterHeader(
    label: &'static str,
    all_label: &'static str,
    filter: RwSignal<String>,
    options: Vec<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let menu_class = move || {
        if is_open.get() {
            "data-table-filter is-open"
        } else {
            "data-table-filter"
        }
    };
    let button_label = move || {
        let current = filter.get();
        if current == "all" {
            format!("Filter {label}")
        } else {
            format!("Filter {label}: {current}")
        }
    };
    let trigger_class = move || {
        if filter.get() == "all" {
            "icon-button data-table-filter__trigger"
        } else {
            "icon-button data-table-filter__trigger is-filtered"
        }
    };
    let option_buttons = options
        .into_iter()
        .map(|option| {
            let option_for_class = option.clone();
            let option_for_checked = option.clone();
            let option_for_click = option.clone();
            view! {
                <button
                    class=move || filter_option_class(&filter.get(), &option_for_class)
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (filter.get() == option_for_checked).to_string()
                    on:click=move |_| {
                        filter.set(option_for_click.clone());
                        is_open.set(false);
                    }
                >
                    {option}
                </button>
            }
        })
        .collect_view();

    view! {
        <div class=menu_class>
            <span>{label}</span>
            <button
                class=trigger_class
                type="button"
                aria-label=button_label
                title=button_label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
            </button>
            <button
                class="data-table-filter__scrim"
                type="button"
                aria-label=format!("Close {label} filter")
                on:click=move |_| is_open.set(false)
            ></button>
            <div class="data-table-filter__menu blurred-surface" role="menu">
                <button
                    class=move || filter_option_class(&filter.get(), "all")
                    type="button"
                    role="menuitemradio"
                    aria-checked=move || (filter.get() == "all").to_string()
                    on:click=move |_| {
                        filter.set("all".to_string());
                        is_open.set(false);
                    }
                >
                    {all_label}
                </button>
                {option_buttons}
            </div>
        </div>
    }
}

#[component]
fn RelatedDashboardsTable(dashboards: Vec<NodeDashboardLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let dashboards_for_table = dashboards.clone();
    let dashboards_for_cards = dashboards;
    let filtered_dashboards = move || {
        let query = search.get();
        dashboards_for_table
            .iter()
            .filter(|dashboard| {
                text_matches(
                    &query,
                    &[
                        &dashboard.dashboard_name,
                        &dashboard.component_count.to_string(),
                        dashboard.description.as_deref().unwrap_or("No description"),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };
    let filtered_dashboard_cards = move || {
        let query = search.get();
        dashboards_for_cards
            .iter()
            .filter(|dashboard| {
                text_matches(
                    &query,
                    &[
                        &dashboard.dashboard_name,
                        &dashboard.component_count.to_string(),
                        dashboard.description.as_deref().unwrap_or("No description"),
                    ],
                )
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    view! {
        <div class="related-work-responsive-table">
            <SearchableDataTable search_label="Search dashboards" placeholder="Search related dashboards" search>
                <thead>
                    <tr>
                        <th scope="col">"Dashboard name"</th>
                        <th scope="col">"Component Count"</th>
                        <th scope="col">"Description"</th>
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let rows = filtered_dashboards();
                        if rows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="3">"No Related Dashboards to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            rows
                                .into_iter()
                                .map(|dashboard| {
                                    let href = format!("/dashboards/{}", dashboard.dashboard_id);
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=href>{dashboard.dashboard_name}</a>
                                            </th>
                                            <td>{dashboard.component_count}</td>
                                            <td>{nonempty_text(dashboard.description.as_deref(), "No description")}</td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </tbody>
            </SearchableDataTable>
            <div class="related-work-mobile-cards">
                {move || {
                    let rows = filtered_dashboard_cards();
                    if rows.is_empty() {
                        view! { <p class="related-work-mobile-empty">"No Related Dashboards to Display"</p> }.into_any()
                    } else {
                        rows
                            .into_iter()
                            .map(|dashboard| {
                                let href = format!("/dashboards/{}", dashboard.dashboard_id);
                                view! {
                                    <article class="related-work-mobile-card">
                                        <div class="related-work-mobile-card__header">
                                            <h4><a href=href>{dashboard.dashboard_name}</a></h4>
                                        </div>
                                        <dl>
                                            <div>
                                                <dt>"Component Count"</dt>
                                                <dd>{dashboard.component_count}</dd>
                                            </div>
                                            <div>
                                                <dt>"Description"</dt>
                                                <dd>{nonempty_text(dashboard.description.as_deref(), "No description")}</dd>
                                            </div>
                                        </dl>
                                    </article>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </div>
        </div>
    }
}

fn text_matches(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return true;
    }

    values
        .iter()
        .any(|value| value.to_lowercase().contains(&query))
}

fn nonempty_text(value: Option<&str>, fallback: &'static str) -> String {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| fallback.to_string())
}

fn sentence_label(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}

fn metadata_rows(metadata: &Value) -> Vec<(String, String)> {
    match metadata {
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| (metadata_label(key), metadata_value(value)))
            .collect(),
        _ => Vec::new(),
    }
}

fn metadata_label(key: &str) -> String {
    key.split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn metadata_value(value: &Value) -> String {
    match value {
        Value::Null => "-".to_string(),
        Value::Bool(value) => {
            if *value {
                "Yes".to_string()
            } else {
                "No".to_string()
            }
        }
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .map(metadata_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(_) => value.to_string(),
    }
}

fn visible_child_label(count: usize) -> String {
    match count {
        0 => "No visible children".to_string(),
        1 => "1 visible child".to_string(),
        count => format!("{count} visible children"),
    }
}

fn active_form_version(form: &FormSummary) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "published")
        .or_else(|| form.versions.last())
}

fn active_form_definition_version(form: &FormDefinition) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "published")
        .or_else(|| form.versions.last())
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn editable_form_definition_version(form: &FormDefinition) -> Option<&FormVersionSummary> {
    form.versions
        .iter()
        .rev()
        .find(|version| version.status == "draft")
        .or_else(|| active_form_definition_version(form))
}

fn form_version_label(version: Option<&FormVersionSummary>) -> String {
    version
        .and_then(|version| version.version_label.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
}

fn form_version_sort_label(version: &FormVersionSummary) -> String {
    version.version_label.clone().unwrap_or_else(|| {
        match (
            version.version_major,
            version.version_minor,
            version.version_patch,
        ) {
            (Some(major), Some(minor), Some(patch)) => format!("{major}.{minor}.{patch}"),
            _ => "-".to_string(),
        }
    })
}

fn form_status_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| sentence_label(&version.status))
        .unwrap_or_else(|| "No versions".to_string())
}

fn form_field_count_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| version.field_count.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn form_scope_label(form: &FormSummary) -> String {
    nonempty_text(form.scope_node_type_name.as_deref(), "All node types")
}

fn form_definition_scope_label(form: &FormDefinition) -> String {
    nonempty_text(form.scope_node_type_name.as_deref(), "All node types")
}

fn workflow_version_label(workflow: &WorkflowSummary) -> String {
    nonempty_text(workflow.current_version_label.as_deref(), "-")
}

fn workflow_status_key(workflow: &WorkflowSummary) -> &str {
    workflow.current_status.as_deref().unwrap_or("none")
}

fn workflow_status_label(workflow: &WorkflowSummary) -> String {
    workflow
        .current_status
        .as_deref()
        .map(sentence_label)
        .unwrap_or_else(|| "No versions".to_string())
}

fn workflow_description_label(workflow: &WorkflowSummary) -> String {
    nonempty_text(Some(workflow.description.as_str()), "No description")
}

fn workflow_assigned_to_label(workflow: &WorkflowSummary) -> String {
    if workflow.assignment_node_names.is_empty() {
        "No active assignments".to_string()
    } else {
        workflow.assignment_node_names.join(", ")
    }
}

fn workflow_assignment_count_label(workflow: &WorkflowSummary) -> String {
    workflow.assignment_count.to_string()
}

fn workflow_assignment_links(
    workflow: &WorkflowSummary,
    nodes: &[OrganizationNode],
) -> Vec<FormAttachmentLink> {
    workflow
        .assignment_node_names
        .iter()
        .filter_map(|name| {
            let node = nodes.iter().find(|node| node.name == *name)?;
            Some(FormAttachmentLink {
                href: format!("/organization/{}", node.id),
                label: node.name.clone(),
                title: organization_node_path(node, nodes),
            })
        })
        .collect()
}

fn organization_node_path(node: &OrganizationNode, nodes: &[OrganizationNode]) -> String {
    let mut names = vec![node.name.clone()];
    let mut current_parent_id = node.parent_node_id.as_deref();

    while let Some(parent_id) = current_parent_id {
        let Some(parent) = nodes.iter().find(|candidate| candidate.id == parent_id) else {
            break;
        };
        names.push(parent.name.clone());
        current_parent_id = parent.parent_node_id.as_deref();
    }

    names.reverse();
    names.join(" > ")
}

fn form_attached_to_label(version: Option<&FormVersionSummary>) -> String {
    version
        .map(|version| {
            version
                .assignment_nodes
                .iter()
                .map(|node| node.node_name.as_str())
                .filter(|name| !name.trim().is_empty())
                .collect::<Vec<_>>()
                .join(", ")
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Not attached".to_string())
}

fn form_attached_nodes(version: Option<&FormVersionSummary>) -> Vec<FormAttachmentLink> {
    version
        .map(|version| {
            version
                .assignment_nodes
                .iter()
                .filter(|node| !node.node_name.trim().is_empty())
                .map(|node| FormAttachmentLink {
                    href: format!("/organization/{}", node.node_id),
                    label: node.node_name.clone(),
                    title: if node.node_path.trim().is_empty() {
                        node.node_name.clone()
                    } else {
                        node.node_path.replace(" / ", " > ")
                    },
                })
                .collect::<Vec<_>>()
        })
        .filter(|nodes| !nodes.is_empty())
        .unwrap_or_default()
}

fn rendered_field_type_label(field_type: &str) -> String {
    match field_type {
        "static_text" => "Static text".to_string(),
        "single_choice" => "Single choice".to_string(),
        "multi_choice" => "Multi choice".to_string(),
        "boolean" => "Checkbox".to_string(),
        _ => sentence_label(field_type),
    }
}

fn form_node_filter_options(forms: &[FormSummary]) -> Vec<FormNodeFilterOption> {
    let mut options_by_id = BTreeMap::<String, FormNodeFilterOption>::new();

    for form in forms {
        for version in &form.versions {
            for node in &version.assignment_nodes {
                if node.node_id.trim().is_empty() || node.node_name.trim().is_empty() {
                    continue;
                }

                let path = if node.node_path.trim().is_empty() {
                    node.node_name.clone()
                } else {
                    node.node_path.clone()
                };

                options_by_id
                    .entry(node.node_id.clone())
                    .or_insert_with(|| FormNodeFilterOption {
                        id: node.node_id.clone(),
                        name: node.node_name.clone(),
                        parent_node_id: node.parent_node_id.clone(),
                        path,
                        depth: 0,
                    });
            }
        }
    }

    let options_map = options_by_id.clone();
    let mut options = options_by_id
        .into_values()
        .map(|mut option| {
            option.depth = form_node_filter_depth(&option.id, &options_map, &mut HashSet::new());
            option.path = form_node_filter_path(&option.id, &options_map, &mut HashSet::new());
            option
        })
        .collect::<Vec<_>>();
    options.sort_by(|left, right| left.path.cmp(&right.path).then(left.name.cmp(&right.name)));
    options
}

fn form_node_filter_depth(
    node_id: &str,
    options_by_id: &BTreeMap<String, FormNodeFilterOption>,
    visited: &mut HashSet<String>,
) -> usize {
    if !visited.insert(node_id.to_string()) {
        return 0;
    }

    options_by_id
        .get(node_id)
        .and_then(|option| option.parent_node_id.as_deref())
        .and_then(|parent_id| {
            options_by_id
                .contains_key(parent_id)
                .then(|| 1 + form_node_filter_depth(parent_id, options_by_id, visited))
        })
        .unwrap_or(0)
}

fn form_node_filter_path(
    node_id: &str,
    options_by_id: &BTreeMap<String, FormNodeFilterOption>,
    visited: &mut HashSet<String>,
) -> String {
    if !visited.insert(node_id.to_string()) {
        return options_by_id
            .get(node_id)
            .map(|option| option.name.clone())
            .unwrap_or_else(|| node_id.to_string());
    }

    let Some(option) = options_by_id.get(node_id) else {
        return node_id.to_string();
    };

    option
        .parent_node_id
        .as_deref()
        .filter(|parent_id| options_by_id.contains_key(*parent_id))
        .map(|parent_id| {
            format!(
                "{} / {}",
                form_node_filter_path(parent_id, options_by_id, visited),
                option.name
            )
        })
        .unwrap_or_else(|| option.name.clone())
}

fn form_matches_node_filter(
    form: &FormSummary,
    selected_node_id: Option<&str>,
    options: &[FormNodeFilterOption],
) -> bool {
    let Some(selected_node_id) = selected_node_id else {
        return true;
    };

    form.versions.iter().any(|version| {
        version.assignment_nodes.iter().any(|node| {
            node.node_id == selected_node_id
                || form_node_is_descendant_of_selected(&node.node_id, selected_node_id, options)
        })
    })
}

fn form_node_is_descendant_of_selected(
    node_id: &str,
    selected_node_id: &str,
    options: &[FormNodeFilterOption],
) -> bool {
    let by_id = options
        .iter()
        .map(|option| (option.id.as_str(), option))
        .collect::<HashMap<_, _>>();
    let mut current_parent = by_id
        .get(node_id)
        .and_then(|option| option.parent_node_id.as_deref());
    let mut visited = HashSet::<String>::new();

    while let Some(parent_id) = current_parent {
        if parent_id == selected_node_id {
            return true;
        }
        if !visited.insert(parent_id.to_string()) {
            return false;
        }
        current_parent = by_id
            .get(parent_id)
            .and_then(|option| option.parent_node_id.as_deref());
    }

    false
}

fn visible_form_node_filter_options(
    options: &[FormNodeFilterOption],
    selected_node_id: Option<&str>,
    query: &str,
) -> Vec<FormNodeFilterOption> {
    let query = query.trim().to_lowercase();

    options
        .iter()
        .filter(|option| {
            if selected_node_id == Some(option.id.as_str()) {
                return false;
            }

            let Some(selected_node_id) = selected_node_id else {
                return true;
            };

            form_node_is_descendant_of_selected(&option.id, selected_node_id, options)
        })
        .filter(|option| {
            query.is_empty()
                || option.name.to_lowercase().contains(&query)
                || option.path.to_lowercase().contains(&query)
        })
        .cloned()
        .collect()
}

fn indented_node_label(option: &FormNodeFilterOption) -> String {
    format!("{}{}", " ".repeat(option.depth), option.name)
}

fn unique_filter_options(values: impl IntoIterator<Item = String>) -> Vec<String> {
    let mut options = values
        .into_iter()
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>();
    options.sort();
    options.dedup();
    options
}

fn slug_from_label(label: &str) -> String {
    let mut slug = String::new();
    let mut last_was_dash = false;

    for character in label
        .trim()
        .chars()
        .flat_map(|character| character.to_lowercase())
    {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            last_was_dash = false;
        } else if !last_was_dash && !slug.is_empty() {
            slug.push('-');
            last_was_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    slug
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn unique_slug_from_label(label: &str, existing_slugs: &[String]) -> String {
    let base = slug_from_label(label);
    if base.is_empty() {
        return String::new();
    }

    let existing = existing_slugs.iter().cloned().collect::<HashSet<_>>();
    if !existing.contains(&base) {
        return base;
    }

    let mut suffix = 2;
    loop {
        let candidate = format!("{base}-{suffix}");
        if !existing.contains(&candidate) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn existing_form_slugs(forms: &[FormSummary]) -> Vec<String> {
    forms.iter().map(|form| form.slug.clone()).collect()
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn existing_form_slugs_for_update(forms: &[FormSummary], current_form_id: &str) -> Vec<String> {
    forms
        .iter()
        .filter(|form| form.id != current_form_id)
        .map(|form| form.slug.clone())
        .collect()
}

fn blank_form_builder_section(id: usize) -> FormBuilderSectionDraft {
    FormBuilderSectionDraft {
        id,
        remote_id: None,
        title: if id == 1 {
            "Main".into()
        } else {
            format!("Section {id}")
        },
        description: String::new(),
        column_count: FORM_BUILDER_COLUMN_COUNT,
        default_column_width: 6,
        position: id as i32,
    }
}

fn blank_form_builder_field_at(
    id: usize,
    section_id: usize,
    grid_row: i32,
    grid_column: i32,
    grid_width: i32,
) -> FormBuilderFieldDraft {
    FormBuilderFieldDraft {
        id,
        remote_id: None,
        section_id,
        label: String::new(),
        key: String::new(),
        field_type: "text".into(),
        required: false,
        grid_row,
        grid_column,
        grid_width: grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
        grid_height: 1,
        key_was_edited: false,
    }
}

fn form_builder_field_default_label(field_type: &str, id: usize) -> String {
    if field_type == "static_text" {
        "Static text".into()
    } else {
        format!("Field {id}")
    }
}

#[derive(Clone, Debug, PartialEq)]
struct FormBuilderGridCell {
    row: i32,
    column: i32,
}

#[derive(Clone, Debug, PartialEq)]
struct FormBuilderSectionLayout {
    fields: Vec<FormBuilderFieldDraft>,
    occupied_cells: HashSet<(i32, i32)>,
    column_count: i32,
    row_count: i32,
}

fn form_builder_section_fields(
    section_id: usize,
    fields: &[FormBuilderFieldDraft],
) -> Vec<FormBuilderFieldDraft> {
    fields
        .iter()
        .filter(|field| field.section_id == section_id)
        .cloned()
        .collect()
}

fn form_builder_occupancy_map(fields: &[FormBuilderFieldDraft]) -> HashSet<(i32, i32)> {
    let mut occupied = HashSet::new();

    for field in fields {
        let row_start = field.grid_row.max(1);
        let row_end = row_start + field.grid_height.max(1) - 1;
        let column_start = field.grid_column.max(1);
        let column_end = column_start + field.grid_width.max(1) - 1;

        for row in row_start..=row_end {
            for column in column_start..=column_end {
                occupied.insert((row, column));
            }
        }
    }

    occupied
}

fn form_builder_section_layout(
    section: &FormBuilderSectionDraft,
    fields: &[FormBuilderFieldDraft],
) -> FormBuilderSectionLayout {
    let section_fields = form_builder_section_fields(section.id, fields);
    let occupied_cells = form_builder_occupancy_map(&section_fields);
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let bottom_occupied_row = section_fields
        .iter()
        .map(|field| field.grid_row.max(1) + field.grid_height.max(1) - 1)
        .max()
        .unwrap_or(0);
    let row_count = (bottom_occupied_row + 1).max(2);

    FormBuilderSectionLayout {
        fields: section_fields,
        occupied_cells,
        column_count,
        row_count,
    }
}

fn form_builder_fields_overlap(
    left: &FormBuilderFieldDraft,
    right: &FormBuilderFieldDraft,
) -> bool {
    if left.section_id != right.section_id || left.id == right.id {
        return false;
    }

    let left_row_start = left.grid_row.max(1);
    let left_row_end = left_row_start + left.grid_height.max(1) - 1;
    let left_column_start = left.grid_column.max(1);
    let left_column_end = left_column_start + left.grid_width.max(1) - 1;

    let right_row_start = right.grid_row.max(1);
    let right_row_end = right_row_start + right.grid_height.max(1) - 1;
    let right_column_start = right.grid_column.max(1);
    let right_column_end = right_column_start + right.grid_width.max(1) - 1;

    left_row_start <= right_row_end
        && left_row_end >= right_row_start
        && left_column_start <= right_column_end
        && left_column_end >= right_column_start
}

fn form_builder_field_has_collision(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> bool {
    fields
        .iter()
        .any(|candidate| candidate.id != field.id && form_builder_fields_overlap(field, candidate))
}

fn form_builder_linear_grid_index(field: &FormBuilderFieldDraft, column_count: i32) -> i32 {
    let column_count = column_count.max(1);
    (field.grid_row.max(1) - 1) * column_count + field.grid_column.max(1) - 1
}

fn form_builder_reflow_section_fields(
    fields: &[FormBuilderFieldDraft],
    preview: FormBuilderDragPreview,
) -> Vec<FormBuilderFieldDraft> {
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let mut section_fields = fields
        .iter()
        .filter(|field| field.section_id == preview.section_id)
        .cloned()
        .map(|mut field| {
            if field.id == preview.field_id {
                field.grid_row = preview.row.max(1);
                field.grid_column = preview.column.max(1);
                field.grid_width = field.grid_width.min(column_count).max(1);
                let max_column = (column_count - field.grid_width + 1).max(1);
                field.grid_column = field.grid_column.clamp(1, max_column);
            }
            field
        })
        .collect::<Vec<_>>();

    section_fields.sort_by(|left, right| {
        form_builder_linear_grid_index(left, column_count)
            .cmp(&form_builder_linear_grid_index(right, column_count))
            .then_with(|| {
                let left_dragged = left.id == preview.field_id;
                let right_dragged = right.id == preview.field_id;
                right_dragged.cmp(&left_dragged)
            })
            .then(left.id.cmp(&right.id))
    });

    let mut placed = Vec::new();

    for field in section_fields {
        let width = field.grid_width.clamp(1, column_count);
        let height = field.grid_height.clamp(1, 6);
        let start_index = form_builder_linear_grid_index(&field, column_count).max(0);

        for index in start_index..=(column_count * 240) {
            let row = index / column_count + 1;
            let column = index % column_count + 1;

            if column + width - 1 > column_count {
                continue;
            }

            let mut candidate = field.clone();
            candidate.grid_row = row;
            candidate.grid_column = column;
            candidate.grid_width = width;
            candidate.grid_height = height;

            if !placed
                .iter()
                .any(|placed_field| form_builder_fields_overlap(&candidate, placed_field))
            {
                placed.push(candidate);
                break;
            }
        }
    }

    fields
        .iter()
        .filter(|field| field.section_id != preview.section_id)
        .cloned()
        .chain(placed)
        .collect()
}

fn max_form_builder_new_field_width_at(
    section_id: usize,
    row: i32,
    column: i32,
    fields: &[FormBuilderFieldDraft],
) -> i32 {
    let row = row.max(1);
    let column = column.clamp(1, FORM_BUILDER_COLUMN_COUNT);
    let mut width = 0;

    for candidate_column in column..=FORM_BUILDER_COLUMN_COUNT {
        let candidate = FormBuilderFieldDraft {
            id: usize::MAX,
            remote_id: None,
            section_id,
            label: String::new(),
            key: String::new(),
            field_type: "text".into(),
            required: false,
            grid_row: row,
            grid_column: column,
            grid_width: candidate_column - column + 1,
            grid_height: 1,
            key_was_edited: false,
        };

        if form_builder_field_has_collision(&candidate, fields) {
            break;
        }

        width += 1;
    }

    width.max(1)
}

fn max_form_builder_field_width(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> i32 {
    let row = field.grid_row.max(1);
    let column = field.grid_column.max(1);
    let column_count = FORM_BUILDER_COLUMN_COUNT;
    let mut width = 0;

    for candidate_column in column..=column_count {
        let mut candidate = field.clone();
        candidate.grid_row = row;
        candidate.grid_column = column;
        candidate.grid_width = candidate_column - column + 1;

        let blocked = form_builder_field_has_collision(&candidate, fields);

        if blocked {
            break;
        }

        width += 1;
    }

    width.max(1)
}

fn max_form_builder_field_height(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
) -> i32 {
    let mut height = 0;

    for candidate_height in 1..=6 {
        let mut candidate = field.clone();
        candidate.grid_height = candidate_height;

        if form_builder_field_has_collision(&candidate, fields) {
            break;
        }

        height += 1;
    }

    height.max(1)
}

fn form_builder_layout_candidate(
    field: &FormBuilderFieldDraft,
    control_index: usize,
    value: i32,
) -> FormBuilderFieldDraft {
    let mut candidate = field.clone();

    match control_index {
        0 => candidate.grid_row = value,
        1 => {
            let max_column = (FORM_BUILDER_COLUMN_COUNT - candidate.grid_width.max(1) + 1)
                .clamp(1, FORM_BUILDER_COLUMN_COUNT);
            candidate.grid_column = value.clamp(1, max_column);
        }
        2 => candidate.grid_width = value,
        _ => candidate.grid_height = value.clamp(1, 6),
    }

    candidate
}

fn valid_form_builder_layout_values(
    field: &FormBuilderFieldDraft,
    fields: &[FormBuilderFieldDraft],
    control_index: usize,
    max_value: i32,
) -> Vec<i32> {
    let current_value = match control_index {
        0 => field.grid_row,
        1 => field.grid_column,
        2 => field.grid_width,
        _ => field.grid_height,
    }
    .max(1);

    let mut values = (1..=max_value.max(1))
        .filter(|value| {
            let candidate = form_builder_layout_candidate(field, control_index, *value);
            let candidate_column_end =
                candidate.grid_column.max(1) + candidate.grid_width.max(1) - 1;

            candidate_column_end <= FORM_BUILDER_COLUMN_COUNT
                && !form_builder_field_has_collision(&candidate, fields)
        })
        .collect::<Vec<_>>();

    let current_candidate = form_builder_layout_candidate(field, control_index, current_value);
    let current_column_end =
        current_candidate.grid_column.max(1) + current_candidate.grid_width.max(1) - 1;
    let current_is_valid = current_column_end <= FORM_BUILDER_COLUMN_COUNT
        && !form_builder_field_has_collision(&current_candidate, fields);

    if current_is_valid && !values.contains(&current_value) {
        values.push(current_value);
        values.sort_unstable();
    }

    values
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn set_form_builder_field_size(
    fields: &mut [FormBuilderFieldDraft],
    field_id: usize,
    width: i32,
    height: i32,
) {
    let Some(position) = fields.iter().position(|field| field.id == field_id) else {
        return;
    };

    let mut candidate = fields[position].clone();
    candidate.grid_width = width.clamp(1, FORM_BUILDER_COLUMN_COUNT);
    candidate.grid_height = height.clamp(1, 6);

    let column_end = candidate.grid_column.max(1) + candidate.grid_width.max(1) - 1;
    if column_end > FORM_BUILDER_COLUMN_COUNT {
        return;
    }

    if form_builder_field_has_collision(&candidate, fields) {
        return;
    }

    fields[position] = candidate;
}

fn form_builder_grid_tile_style(field: &FormBuilderFieldDraft) -> String {
    format!(
        "grid-column: {} / span {}; grid-row: {} / span {};",
        field.grid_column.max(1),
        field.grid_width.max(1),
        field.grid_row.max(1),
        field.grid_height.max(1),
    )
}

#[cfg(feature = "hydrate")]
fn start_form_builder_field_resize(
    event: leptos::ev::MouseEvent,
    axis: FormBuilderResizeAxis,
    field_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) {
    event.prevent_default();
    event.stop_propagation();

    let Some(window) = web_sys::window() else {
        return;
    };
    if window
        .match_media("(max-width: 767px)")
        .ok()
        .flatten()
        .is_some_and(|query| query.matches())
    {
        return;
    }

    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Some(tile) = target.closest(".form-builder-grid-tile").ok().flatten() else {
        return;
    };
    let Some(grid) = target.closest(".form-builder-layout-grid").ok().flatten() else {
        return;
    };
    let Some(start_field) = builder_fields
        .get_untracked()
        .into_iter()
        .find(|field| field.id == field_id)
    else {
        return;
    };

    let Some(grid_element) = grid.dyn_ref::<web_sys::HtmlElement>() else {
        return;
    };
    let cell_width = f64::from(grid_element.client_width()) / f64::from(FORM_BUILDER_COLUMN_COUNT);
    let row_height = 80.0;
    if cell_width <= 0.0 {
        return;
    }

    suppress_builder_field_click.set(Some(field_id));
    let _ = tile.class_list().add_1("is-resizing");

    let active = Rc::new(Cell::new(true));
    let last_valid_width = Rc::new(Cell::new(start_field.grid_width.max(1)));
    let last_valid_height = Rc::new(Cell::new(start_field.grid_height.max(1)));
    let start_x = event.client_x();
    let start_y = event.client_y();

    let move_callback: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>>> =
        Rc::new(RefCell::new(None));
    let up_callback: Rc<RefCell<Option<Closure<dyn FnMut(web_sys::MouseEvent)>>>> =
        Rc::new(RefCell::new(None));

    let active_for_move = active.clone();
    let tile_for_move = tile.clone();
    let last_width_for_move = last_valid_width.clone();
    let last_height_for_move = last_valid_height.clone();
    let builder_fields_for_move = builder_fields;
    let start_field_for_move = start_field.clone();
    *move_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_move.get() {
            return;
        }
        event.prevent_default();

        let mut candidate = start_field_for_move.clone();
        match axis {
            FormBuilderResizeAxis::Width => {
                let width_delta =
                    (f64::from(event.client_x() - start_x) / cell_width).round() as i32;
                candidate.grid_width = (start_field_for_move.grid_width + width_delta)
                    .clamp(1, FORM_BUILDER_COLUMN_COUNT);
            }
            FormBuilderResizeAxis::Height => {
                let height_delta =
                    (f64::from(event.client_y() - start_y) / row_height).round() as i32;
                candidate.grid_height =
                    (start_field_for_move.grid_height + height_delta).clamp(1, 6);
            }
        }

        let column_end = candidate.grid_column.max(1) + candidate.grid_width.max(1) - 1;
        if column_end > FORM_BUILDER_COLUMN_COUNT {
            return;
        }

        let fields = builder_fields_for_move.get_untracked();
        if form_builder_field_has_collision(&candidate, &fields) {
            return;
        }

        last_width_for_move.set(candidate.grid_width.max(1));
        last_height_for_move.set(candidate.grid_height.max(1));
        let _ = tile_for_move.set_attribute("style", &form_builder_grid_tile_style(&candidate));
    }) as Box<dyn FnMut(_)>));

    let active_for_up = active.clone();
    let tile_for_up = tile.clone();
    let last_width_for_up = last_valid_width.clone();
    let last_height_for_up = last_valid_height.clone();
    let move_callback_for_up = move_callback.clone();
    let up_callback_for_up = up_callback.clone();
    *up_callback.borrow_mut() = Some(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
        if !active_for_up.replace(false) {
            return;
        }
        event.prevent_default();
        let _ = tile_for_up.class_list().remove_1("is-resizing");
        builder_fields.update(|fields| {
            set_form_builder_field_size(
                fields,
                field_id,
                last_width_for_up.get(),
                last_height_for_up.get(),
            );
        });

        if let Some(window) = web_sys::window() {
            if let Some(callback) = move_callback_for_up.borrow().as_ref() {
                let _ = window.remove_event_listener_with_callback(
                    "mousemove",
                    callback.as_ref().unchecked_ref(),
                );
            }
            if let Some(callback) = up_callback_for_up.borrow().as_ref() {
                let _ = window.remove_event_listener_with_callback(
                    "mouseup",
                    callback.as_ref().unchecked_ref(),
                );
            }
        }
        move_callback_for_up.borrow_mut().take();
        up_callback_for_up.borrow_mut().take();
    }) as Box<dyn FnMut(_)>));

    if let Some(callback) = move_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mousemove", callback.as_ref().unchecked_ref());
    }
    if let Some(callback) = up_callback.borrow().as_ref() {
        let _ =
            window.add_event_listener_with_callback("mouseup", callback.as_ref().unchecked_ref());
    }
}

#[cfg(not(feature = "hydrate"))]
fn start_form_builder_field_resize(
    _event: leptos::ev::MouseEvent,
    _axis: FormBuilderResizeAxis,
    _field_id: usize,
    _builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    _suppress_builder_field_click: RwSignal<Option<usize>>,
) {
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn prepared_form_builder_sections(
    sections: &[FormBuilderSectionDraft],
) -> Result<Vec<FormBuilderSectionDraft>, String> {
    let mut prepared = Vec::new();

    for (index, section) in sections.iter().enumerate() {
        let title = section.title.trim();
        if title.is_empty() {
            return Err("Every section needs a title.".into());
        }
        let mut section = section.clone();
        section.column_count = FORM_BUILDER_COLUMN_COUNT;
        section.title = title.to_string();
        section.description = section.description.trim().to_string();
        section.position = (index + 1) as i32;
        prepared.push(section);
    }

    Ok(prepared)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn prepared_form_builder_fields(
    fields: &[FormBuilderFieldDraft],
) -> Result<Vec<FormBuilderFieldDraft>, String> {
    let mut prepared = Vec::new();
    let mut keys = HashSet::new();

    for field in fields {
        let label = field.label.trim();
        let key = field.key.trim();
        if label.is_empty() && key.is_empty() {
            continue;
        }
        if label.is_empty() {
            return Err("Every builder field needs a label.".into());
        }
        if key.is_empty() {
            return Err(format!("{label} needs a field key."));
        }

        let normalized_key = slug_from_label(key);
        if normalized_key.is_empty() {
            return Err(format!("{label} needs a valid field key."));
        }
        if !keys.insert(normalized_key.clone()) {
            return Err(format!("Field key {normalized_key} is already used."));
        }
        if field.grid_row < 1 {
            return Err(format!("{label} must start on row 1 or later."));
        }
        if field.grid_column < 1 {
            return Err(format!("{label} must start on column 1 or later."));
        }
        if field.grid_width < 1 {
            return Err(format!("{label} must span at least 1 column."));
        }
        if field.grid_height < 1 {
            return Err(format!("{label} must span at least 1 row."));
        }

        let mut field = field.clone();
        field.label = label.to_string();
        field.key = normalized_key;
        prepared.push(field);
    }

    Ok(prepared)
}

fn form_builder_field_type_icon(field_type: &str) -> AnyView {
    match field_type {
        "static_text" => view! { <TextQuote /> }.into_any(),
        "number" => view! { <Hash /> }.into_any(),
        "date" => view! { <CalendarDays /> }.into_any(),
        "boolean" => view! { <SquareCheckBig /> }.into_any(),
        "single_choice" => view! { <CircleDot /> }.into_any(),
        "multi_choice" => view! { <ListChecks /> }.into_any(),
        _ => view! { <TextCursorInput /> }.into_any(),
    }
}

fn status_badge_class(status: &str) -> &'static str {
    match status {
        "published" | "done" | "active" => "status-badge is-success",
        "draft" | "in_progress" => "status-badge is-warning",
        "error" | "archived" => "status-badge is-danger",
        _ => "status-badge is-info",
    }
}

fn toggle_organization_branch(
    expanded_nodes: RwSignal<HashSet<String>>,
    node_id: String,
    lineage: Vec<String>,
) {
    expanded_nodes.update(|nodes| {
        let was_open = nodes.contains(&node_id);
        let lineage: HashSet<String> = lineage.into_iter().collect();

        nodes.retain(|open_id| lineage.contains(open_id));

        if !was_open {
            nodes.insert(node_id);
        }
    });
}

fn load_organization_tree(
    tree: RwSignal<Vec<OrganizationTreeNode>>,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    expanded_nodes: RwSignal<HashSet<String>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;
            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;

            match (node_response, node_type_response) {
                (Ok(response), _) if response.status() == 401 => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_response), Ok(node_type_response))
                    if node_response.ok() && node_type_response.ok() =>
                {
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;

                    match (loaded_nodes, loaded_node_types) {
                        (Ok(nodes), Ok(loaded_node_types)) => {
                            let branches = build_organization_tree(nodes);
                            let first_open = branches
                                .iter()
                                .find(|branch| !branch.children.is_empty())
                                .map(|branch| branch.node.id.clone());

                            expanded_nodes.set(first_open.into_iter().collect());
                            tree.set(branches);
                            node_types.set(loaded_node_types);
                            is_loading.set(false);
                        }
                        _ => {
                            tree.set(Vec::new());
                            node_types.set(Vec::new());
                            load_error
                                .set(Some("The hierarchy response could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(_), Ok(_)) => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    load_error.set(Some(
                        "The hierarchy API returned an unexpected response.".into(),
                    ));
                    is_loading.set(false);
                }
                _ => {
                    tree.set(Vec::new());
                    node_types.set(Vec::new());
                    load_error.set(Some("Could not reach the hierarchy API.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (tree, node_types, expanded_nodes, is_loading, load_error);
    }
}

fn load_forms(
    forms: RwSignal<Vec<FormSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/forms").send().await {
                Ok(response) if response.status() == 401 => {
                    forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<Vec<FormSummary>>().await {
                    Ok(loaded_forms) => {
                        forms.set(loaded_forms);
                        is_loading.set(false);
                    }
                    Err(error) => {
                        forms.set(Vec::new());
                        load_error.set(Some(format!("Unable to parse forms: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    forms.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load forms. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    forms.set(Vec::new());
                    load_error.set(Some(format!("Unable to load forms: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (forms, is_loading, load_error);
    }
}

fn load_workflows(
    workflows: RwSignal<Vec<WorkflowSummary>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);

            match gloo_net::http::Request::get("/api/workflows").send().await {
                Ok(response) if response.status() == 401 => {
                    workflows.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<Vec<WorkflowSummary>>().await {
                        Ok(loaded_workflows) => {
                            workflows.set(loaded_workflows);
                            is_loading.set(false);
                        }
                        Err(error) => {
                            workflows.set(Vec::new());
                            load_error.set(Some(format!("Unable to parse workflows: {error}")));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(response) => {
                    workflows.set(Vec::new());
                    load_error.set(Some(format!(
                        "Unable to load workflows. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    workflows.set(Vec::new());
                    load_error.set(Some(format!("Unable to load workflows: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (workflows, is_loading, load_error);
    }
}

fn load_workflow_assignment_nodes(nodes: RwSignal<Vec<OrganizationNode>>) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match gloo_net::http::Request::get("/api/nodes").send().await {
                Ok(response) if response.status() == 401 => {
                    nodes.set(Vec::new());
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    if let Ok(loaded_nodes) = response.json::<Vec<OrganizationNode>>().await {
                        nodes.set(loaded_nodes);
                    }
                }
                _ => nodes.set(Vec::new()),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = nodes;
    }
}

fn load_form_detail(
    form_id: String,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    is_loading: RwSignal<bool>,
    load_error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            load_error.set(None);
            rendered_form.set(None);

            match gloo_net::http::Request::get(&format!("/api/forms/{form_id}"))
                .send()
                .await
            {
                Ok(response) if response.status() == 401 => {
                    detail.set(None);
                    rendered_form.set(None);
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<FormDefinition>().await {
                    Ok(form) => {
                        let active_version_id =
                            active_form_definition_version(&form).map(|version| version.id.clone());
                        detail.set(Some(form));
                        if let Some(version_id) = active_version_id {
                            load_rendered_form_version(version_id, rendered_form);
                        }
                        is_loading.set(false);
                    }
                    Err(error) => {
                        detail.set(None);
                        load_error.set(Some(format!("Unable to parse form detail: {error}")));
                        is_loading.set(false);
                    }
                },
                Ok(response) => {
                    detail.set(None);
                    load_error.set(Some(format!(
                        "Unable to load form detail. Server returned {}.",
                        response.status()
                    )));
                    is_loading.set(false);
                }
                Err(error) => {
                    detail.set(None);
                    load_error.set(Some(format!("Unable to load form detail: {error}")));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (form_id, detail, rendered_form, is_loading, load_error);
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn load_rendered_form_version(
    form_version_id: String,
    rendered_form: RwSignal<Option<RenderedForm>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match gloo_net::http::Request::get(&format!(
                "/api/form-versions/{form_version_id}/render"
            ))
            .send()
            .await
            {
                Ok(response) if response.ok() => {
                    if let Ok(rendered) = response.json::<RenderedForm>().await {
                        rendered_form.set(Some(rendered));
                    }
                }
                _ => rendered_form.set(None),
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (form_version_id, rendered_form);
    }
}

#[cfg(feature = "hydrate")]
async fn send_json_id_request(
    builder: gloo_net::http::RequestBuilder,
    body: Option<String>,
    action: &str,
) -> Result<IdResponse, String> {
    let response = if let Some(body) = body {
        builder
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|_| format!("{action} request could not be prepared."))?
            .send()
            .await
    } else {
        builder.send().await
    };

    match response {
        Ok(response) if response.status() == 401 => {
            redirect_to_login();
            Err("Authentication is required.".into())
        }
        Ok(response) if response.ok() => response
            .json::<IdResponse>()
            .await
            .map_err(|_| format!("{action} response could not be read.")),
        Ok(response) => Err(format!(
            "{action} failed with status {}.",
            response.status()
        )),
        Err(_) => Err(format!("Could not reach the {action} API.")),
    }
}

fn load_form_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;

            match (node_types_response, forms_response) {
                (Ok(response), _) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_types_response), Ok(forms_response))
                    if node_types_response.ok() && forms_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;

                    match (loaded_node_types, loaded_forms) {
                        (Ok(loaded_node_types), Ok(loaded_forms)) => {
                            node_types.set(loaded_node_types);
                            existing_forms.set(loaded_forms);
                            is_loading.set(false);
                        }
                        _ => {
                            node_types.set(Vec::new());
                            existing_forms.set(Vec::new());
                            message.set(Some("Form options could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                (Ok(node_types_response), Ok(forms_response)) => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some(format!(
                        "Form options failed with status {} / {}.",
                        node_types_response.status(),
                        forms_response.status()
                    )));
                    is_loading.set(false);
                }
                _ => {
                    node_types.set(Vec::new());
                    existing_forms.set(Vec::new());
                    message.set(Some("Could not reach the form option APIs.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_types, existing_forms, is_loading, message);
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn hydrate_form_builder_from_rendered(
    rendered_form: Option<&RenderedForm>,
) -> (
    Vec<FormBuilderSectionDraft>,
    Vec<FormBuilderFieldDraft>,
    usize,
    usize,
) {
    let Some(rendered_form) = rendered_form else {
        return (vec![blank_form_builder_section(1)], Vec::new(), 2, 1);
    };

    let mut sections = rendered_form.sections.clone();
    sections.sort_by(|left, right| {
        left.position
            .cmp(&right.position)
            .then(left.title.cmp(&right.title))
    });

    if sections.is_empty() {
        return (vec![blank_form_builder_section(1)], Vec::new(), 2, 1);
    }

    let mut section_id_by_remote = HashMap::new();
    let mut builder_sections = Vec::new();
    let mut builder_fields = Vec::new();
    let mut next_section_id = 1usize;
    let mut next_field_id = 1usize;

    for section in &sections {
        let local_section_id = next_section_id;
        next_section_id += 1;
        section_id_by_remote.insert(section.id.clone(), local_section_id);

        builder_sections.push(FormBuilderSectionDraft {
            id: local_section_id,
            remote_id: Some(section.id.clone()),
            title: nonempty_text(Some(section.title.as_str()), "Main"),
            description: section.description.clone(),
            column_count: FORM_BUILDER_COLUMN_COUNT,
            default_column_width: 6,
            position: section.position,
        });
    }

    for section in &sections {
        let Some(section_id) = section_id_by_remote.get(&section.id).copied() else {
            continue;
        };
        let mut fields = section.fields.clone();
        fields.sort_by(|left, right| {
            left.position
                .cmp(&right.position)
                .then(left.label.cmp(&right.label))
        });

        for field in fields {
            let local_field_id = next_field_id;
            next_field_id += 1;
            builder_fields.push(FormBuilderFieldDraft {
                id: local_field_id,
                remote_id: Some(field.id),
                section_id,
                label: field.label,
                key: field.key,
                field_type: field.field_type,
                required: field.required,
                grid_row: field.grid_row.max(1),
                grid_column: field.grid_column.clamp(1, FORM_BUILDER_COLUMN_COUNT),
                grid_width: field.grid_width.clamp(1, FORM_BUILDER_COLUMN_COUNT),
                grid_height: field.grid_height.clamp(1, 6),
                key_was_edited: true,
            });
        }
    }

    (
        builder_sections,
        builder_fields,
        next_section_id,
        next_field_id,
    )
}

fn load_form_edit_options(
    form_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    detail: RwSignal<Option<FormDefinition>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    name: RwSignal<String>,
    scope_node_type_id: RwSignal<String>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_section: RwSignal<String>,
    next_builder_section_id: RwSignal<usize>,
    next_builder_field_id: RwSignal<usize>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);
            detail.set(None);
            rendered_form.set(None);
            edit_version_id.set(None);
            edit_version_status.set(None);

            let node_types_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let forms_response = gloo_net::http::Request::get("/api/forms").send().await;
            let detail_response =
                gloo_net::http::Request::get(&format!("/api/admin/forms/{form_id}"))
                    .send()
                    .await;

            match (node_types_response, forms_response, detail_response) {
                (Ok(response), _, _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_types_response), Ok(forms_response), Ok(detail_response))
                    if node_types_response.ok() && forms_response.ok() && detail_response.ok() =>
                {
                    let loaded_node_types = node_types_response
                        .json::<Vec<NodeTypeCatalogEntry>>()
                        .await;
                    let loaded_forms = forms_response.json::<Vec<FormSummary>>().await;
                    let loaded_detail = detail_response.json::<FormDefinition>().await;

                    match (loaded_node_types, loaded_forms, loaded_detail) {
                        (Ok(loaded_node_types), Ok(loaded_forms), Ok(form)) => {
                            let selected_version = editable_form_definition_version(&form).cloned();
                            let mut loaded_rendered_form = None;

                            if let Some(version) = selected_version.as_ref() {
                                match gloo_net::http::Request::get(&format!(
                                    "/api/form-versions/{}/render",
                                    version.id
                                ))
                                .send()
                                .await
                                {
                                    Ok(response) if response.ok() => {
                                        loaded_rendered_form =
                                            response.json::<RenderedForm>().await.ok();
                                    }
                                    Ok(response) if response.status() == 401 => {
                                        is_loading.set(false);
                                        redirect_to_login();
                                        return;
                                    }
                                    _ => {
                                        loaded_rendered_form = None;
                                    }
                                }
                            }

                            let (sections, fields, next_section, next_field) =
                                hydrate_form_builder_from_rendered(loaded_rendered_form.as_ref());
                            let active_section = sections
                                .first()
                                .map(|section| section.id.to_string())
                                .unwrap_or_else(|| "1".to_string());

                            name.set(form.name.clone());
                            scope_node_type_id
                                .set(form.scope_node_type_id.clone().unwrap_or_default());
                            edit_version_id
                                .set(selected_version.as_ref().map(|version| version.id.clone()));
                            edit_version_status.set(
                                selected_version
                                    .as_ref()
                                    .map(|version| version.status.clone()),
                            );
                            active_builder_section.set(active_section);
                            next_builder_section_id.set(next_section);
                            next_builder_field_id.set(next_field);
                            builder_sections.set(sections);
                            builder_fields.set(fields);
                            rendered_form.set(loaded_rendered_form);
                            detail.set(Some(form));
                            node_types.set(loaded_node_types);
                            existing_forms.set(loaded_forms);
                            is_loading.set(false);
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Form edit options could not be read.".into()));
                        }
                    }
                }
                (Ok(node_types_response), Ok(forms_response), Ok(detail_response)) => {
                    is_loading.set(false);
                    message.set(Some(format!(
                        "Form edit options failed with status {} / {} / {}.",
                        node_types_response.status(),
                        forms_response.status(),
                        detail_response.status()
                    )));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the form edit APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            form_id,
            node_types,
            existing_forms,
            detail,
            rendered_form,
            edit_version_id,
            edit_version_status,
            name,
            scope_node_type_id,
            builder_sections,
            builder_fields,
            active_builder_section,
            next_builder_section_id,
            next_builder_field_id,
            is_loading,
            message,
        );
    }
}

fn load_organization_detail(
    node_id: String,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    is_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            error.set(None);

            let response = gloo_net::http::Request::get(&format!("/api/nodes/{node_id}"))
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => {
                    match response.json::<OrganizationNodeDetail>().await {
                        Ok(payload) => {
                            detail.set(Some(payload));
                            is_loading.set(false);
                        }
                        Err(_) => {
                            error.set(Some("The detail response could not be read.".into()));
                            is_loading.set(false);
                        }
                    }
                }
                Ok(_) => {
                    error.set(Some(
                        "The detail API returned an unexpected response.".into(),
                    ));
                    is_loading.set(false);
                }
                Err(_) => {
                    error.set(Some("Could not reach the detail API.".into()));
                    is_loading.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (node_id, detail, is_loading, error);
    }
}

#[cfg(feature = "hydrate")]
fn redirect_to_login() {
    if let Some(window) = web_sys::window() {
        let _ = window.location().set_href("/login");
    }
}

fn build_organization_tree(nodes: Vec<OrganizationNode>) -> Vec<OrganizationTreeNode> {
    let visible_ids = nodes
        .iter()
        .map(|node| node.id.clone())
        .collect::<HashSet<_>>();
    let mut children_by_parent = HashMap::<Option<String>, Vec<OrganizationNode>>::new();

    for node in nodes {
        let parent_id = node
            .parent_node_id
            .clone()
            .filter(|parent_id| visible_ids.contains(parent_id));
        children_by_parent.entry(parent_id).or_default().push(node);
    }

    for siblings in children_by_parent.values_mut() {
        siblings.sort_by(|left, right| {
            left.node_type_name
                .cmp(&right.node_type_name)
                .then(left.name.cmp(&right.name))
        });
    }

    build_organization_branches(None, &mut children_by_parent)
}

fn build_organization_branches(
    parent_id: Option<String>,
    children_by_parent: &mut HashMap<Option<String>, Vec<OrganizationNode>>,
) -> Vec<OrganizationTreeNode> {
    children_by_parent
        .remove(&parent_id)
        .unwrap_or_default()
        .into_iter()
        .map(|node| {
            let children = build_organization_branches(Some(node.id.clone()), children_by_parent);
            OrganizationTreeNode { node, children }
        })
        .collect()
}

fn child_create_links(
    parent_node_type_id: &str,
    node_types: &[NodeTypeCatalogEntry],
    parent_node_id: &str,
) -> Vec<CreateChildLink> {
    let Some(parent_type) = node_types
        .iter()
        .find(|node_type| node_type.id == parent_node_type_id)
    else {
        return Vec::new();
    };

    parent_type
        .child_relationships
        .iter()
        .map(|relationship| CreateChildLink {
            href: format!(
                "/organization/new?parent_node_id={parent_node_id}&node_type_id={}",
                relationship.node_type_id
            ),
            label: format!("Create {}", relationship.singular_label),
        })
        .collect()
}

#[component]
pub fn OrganizationNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let selected_node_type_id = RwSignal::new(String::new());
    let selected_parent_node_id = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let metadata_fields = RwSignal::new(Vec::<NodeMetadataFieldSummary>::new());
    let metadata_values = RwSignal::new(HashMap::<String, String>::new());
    let metadata_booleans = RwSignal::new(HashMap::<String, bool>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_organization_create_options(
            node_types,
            nodes,
            selected_node_type_id,
            selected_parent_node_id,
            is_loading,
            message,
        );
    });

    Effect::new(move |_| {
        let node_type_id = selected_node_type_id.get();
        if node_type_id.is_empty() {
            metadata_fields.set(Vec::new());
            metadata_values.set(HashMap::new());
            metadata_booleans.set(HashMap::new());
            return;
        }

        load_node_type_metadata(
            node_type_id,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            message,
        );
    });

    let parent_options = move || parent_node_options(&nodes.get());
    let node_type_options = move || {
        available_node_types_for_parent(
            &selected_parent_node_id.get(),
            &node_types.get(),
            &nodes.get(),
        )
    };

    let can_submit = move || {
        !is_loading.get()
            && !is_saving.get()
            && !selected_node_type_id.get().is_empty()
            && !name.get().trim().is_empty()
    };

    view! {
        <AppShell active_route="organization" title="Organization">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/organization">"Organization"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Create Node"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel organization-page">
                <PageHeader title="Create Organization Node">
                    <Button label="Back to Organization" href="/organization"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading create options"</h3>
                                <p>"Fetching organization node types and visible parent records."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <form
                                class="native-form organization-node-form"
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit_create_node(
                                        selected_node_type_id,
                                        selected_parent_node_id,
                                        name,
                                        metadata_fields,
                                        metadata_values,
                                        metadata_booleans,
                                        is_saving,
                                        message,
                                    );
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field" for="organization-parent-node">
                                        <span>"Parent Node"</span>
                                        <select
                                            id="organization-parent-node"
                                            prop:value=move || selected_parent_node_id.get()
                                            on:change=move |event| {
                                                let parent_id = event_target_value(&event);
                                                let available_types = available_node_types_for_parent(
                                                    &parent_id,
                                                    &node_types.get(),
                                                    &nodes.get(),
                                                );
                                                let current_type_id = selected_node_type_id.get();

                                                selected_parent_node_id.set(parent_id);

                                                if !available_types.iter().any(|node_type| node_type.id == current_type_id) {
                                                    selected_node_type_id.set(
                                                        available_types
                                                            .first()
                                                            .map(|node_type| node_type.id.clone())
                                                            .unwrap_or_default(),
                                                    );
                                                }
                                            }
                                        >
                                            <option value="">"Top-level record"</option>
                                            {move || parent_options().into_iter().map(|option| {
                                                view! {
                                                    <option value=option.id>{option.label}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </label>

                                    <label class="form-field" for="organization-node-type">
                                        <span>"Node Type"</span>
                                        <select
                                            id="organization-node-type"
                                            prop:value=move || selected_node_type_id.get()
                                            on:change=move |event| selected_node_type_id.set(event_target_value(&event))
                                        >
                                            <option value="">"Select node type"</option>
                                            {move || node_type_options().into_iter().map(|node_type| {
                                                view! {
                                                    <option value=node_type.id>{node_type.singular_label}</option>
                                                }
                                            }).collect_view()}
                                        </select>
                                    </label>

                                    <label class="form-field form-field--wide" for="organization-name">
                                        <span>"Name"</span>
                                        <input
                                            id="organization-name"
                                            type="text"
                                            autocomplete="off"
                                            prop:value=move || name.get()
                                            on:input=move |event| name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>
                                </div>

                                <section class="form-section">
                                    <h3>"Metadata"</h3>
                                    {move || {
                                        let fields = metadata_fields.get();
                                        if fields.is_empty() {
                                            view! { <p class="muted">"No metadata fields are configured for this node type."</p> }.into_any()
                                        } else {
                                            view! {
                                                <div class="form-grid">
                                                    {fields.into_iter().map(|field| {
                                                        view! {
                                                            <MetadataFieldInput
                                                                field
                                                                metadata_values
                                                                metadata_booleans
                                                            />
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }
                                            .into_any()
                                        }
                                    }}
                                </section>

                                {move || message.get().map(|message| view! {
                                    <p class="form-message" role="status">{message}</p>
                                })}

                                <div class="form-actions">
                                    <Button label="Cancel" href="/organization"/>
                                    <button class="button" type="submit" disabled=move || !can_submit()>
                                        {move || if is_saving.get() { "Saving..." } else { "Create Node" }}
                                    </button>
                                </div>
                            </form>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn MetadataFieldInput(
    field: NodeMetadataFieldSummary,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
) -> impl IntoView {
    let key = field.key.clone();
    let input_id = format!("organization-metadata-{}", field.key);
    let required_label = if field.required { " *" } else { "" };

    match field.field_type.as_str() {
        "boolean" => view! {
            <label class="form-field form-field--checkbox" for=input_id.clone()>
                <input
                    id=input_id.clone()
                    type="checkbox"
                    prop:checked=move || metadata_booleans.with(|values| values.get(&key).copied().unwrap_or(false))
                    on:change=move |event| {
                        metadata_booleans.update(|values| {
                            values.insert(field.key.clone(), event_target_checked(&event));
                        });
                    }
                />
                <span>{format!("{}{}", field.label, required_label)}</span>
            </label>
        }
        .into_any(),
        field_type => {
            let input_type = match field_type {
                "number" => "number",
                "date" => "date",
                _ => "text",
            };

            view! {
                <label class="form-field" for=input_id.clone()>
                    <span>{format!("{}{}", field.label, required_label)}</span>
                    <input
                        id=input_id.clone()
                        type=input_type
                        prop:value=move || metadata_values.with(|values| values.get(&key).cloned().unwrap_or_default())
                        on:input=move |event| {
                            metadata_values.update(|values| {
                                values.insert(field.key.clone(), event_target_value(&event));
                            });
                        }
                        required=field.required
                    />
                </label>
            }
            .into_any()
        }
    }
}

fn parent_node_options(nodes: &[OrganizationNode]) -> Vec<ParentNodeOption> {
    let branches = build_organization_tree(nodes.to_vec());
    let mut options = Vec::new();
    append_parent_node_options(&branches, 0, &mut options);
    options
}

fn parent_node_options_for_edit(
    nodes: &[OrganizationNode],
    node_types: &[NodeTypeCatalogEntry],
    edited_node_id: &str,
    edited_node_type_id: &str,
) -> Vec<ParentNodeOption> {
    let excluded_ids = descendant_node_ids(nodes, edited_node_id);
    parent_node_options(nodes)
        .into_iter()
        .filter(|option| !excluded_ids.contains(&option.id))
        .filter(|option| {
            nodes
                .iter()
                .find(|node| node.id == option.id)
                .and_then(|node| {
                    node_types
                        .iter()
                        .find(|node_type| node_type.id == node.node_type_id)
                })
                .map(|node_type| {
                    node_type
                        .child_relationships
                        .iter()
                        .any(|relationship| relationship.node_type_id == edited_node_type_id)
                })
                .unwrap_or(false)
        })
        .collect()
}

fn descendant_node_ids(nodes: &[OrganizationNode], root_id: &str) -> HashSet<String> {
    let mut descendants = HashSet::from([root_id.to_string()]);
    let mut changed = true;

    while changed {
        changed = false;
        for node in nodes {
            if descendants.contains(&node.id) {
                continue;
            }

            if node
                .parent_node_id
                .as_ref()
                .map(|parent_id| descendants.contains(parent_id))
                .unwrap_or(false)
            {
                descendants.insert(node.id.clone());
                changed = true;
            }
        }
    }

    descendants
}

fn append_parent_node_options(
    branches: &[OrganizationTreeNode],
    depth: usize,
    options: &mut Vec<ParentNodeOption>,
) {
    for branch in branches {
        let prefix = if depth == 0 {
            String::new()
        } else {
            format!("{} ", "--".repeat(depth))
        };

        options.push(ParentNodeOption {
            id: branch.node.id.clone(),
            label: format!(
                "{}{} ({})",
                prefix, branch.node.name, branch.node.node_type_singular_label
            ),
        });
        append_parent_node_options(&branch.children, depth + 1, options);
    }
}

fn available_node_types_for_parent(
    parent_node_id: &str,
    node_types: &[NodeTypeCatalogEntry],
    nodes: &[OrganizationNode],
) -> Vec<NodeTypeCatalogEntry> {
    if parent_node_id.is_empty() {
        return node_types
            .iter()
            .filter(|node_type| node_type.is_root_type)
            .cloned()
            .collect();
    }

    let Some(parent_node) = nodes.iter().find(|node| node.id == parent_node_id) else {
        return Vec::new();
    };
    let Some(parent_type) = node_types
        .iter()
        .find(|node_type| node_type.id == parent_node.node_type_id)
    else {
        return Vec::new();
    };

    parent_type
        .child_relationships
        .iter()
        .filter_map(|relationship| {
            node_types
                .iter()
                .find(|node_type| node_type.id == relationship.node_type_id)
                .cloned()
        })
        .collect()
}

fn load_organization_create_options(
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    selected_node_type_id: RwSignal<String>,
    selected_parent_node_id: RwSignal<String>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;

            match (node_type_response, node_response) {
                (Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_type_response), Ok(node_response))
                    if node_type_response.ok() && node_response.ok() =>
                {
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;

                    match (loaded_node_types, loaded_nodes) {
                        (Ok(loaded_node_types), Ok(loaded_nodes)) => {
                            let requested_node_type_id = current_search_param("node_type_id");
                            let requested_parent_id = current_search_param("parent_node_id")
                                .or_else(|| current_search_param("parent_id"));
                            let selected_parent = requested_parent_id
                                .filter(|requested| {
                                    loaded_nodes.iter().any(|node| node.id == *requested)
                                })
                                .unwrap_or_default();
                            let available_types = available_node_types_for_parent(
                                &selected_parent,
                                &loaded_node_types,
                                &loaded_nodes,
                            );
                            let selected_type = requested_node_type_id
                                .filter(|requested| {
                                    available_types
                                        .iter()
                                        .any(|node_type| node_type.id == *requested)
                                })
                                .or_else(|| {
                                    available_types
                                        .first()
                                        .map(|node_type| node_type.id.clone())
                                });

                            nodes.set(loaded_nodes);
                            node_types.set(loaded_node_types);
                            selected_node_type_id.set(selected_type.unwrap_or_default());
                            selected_parent_node_id.set(selected_parent);
                            is_loading.set(false);
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Create options could not be read.".into()));
                        }
                    }
                }
                (Ok(_), Ok(_)) => {
                    is_loading.set(false);
                    message.set(Some(
                        "Create options returned an unexpected response.".into(),
                    ));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the organization APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_types,
            nodes,
            selected_node_type_id,
            selected_parent_node_id,
            is_loading,
            message,
        );
    }
}

fn load_node_type_metadata(
    node_type_id: String,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            let response =
                gloo_net::http::Request::get(&format!("/api/admin/node-types/{node_type_id}"))
                    .send()
                    .await;

            match response {
                Ok(response) if response.status() == 401 => redirect_to_login(),
                Ok(response) if response.ok() => {
                    match response.json::<NodeTypeDefinition>().await {
                        Ok(definition) => {
                            metadata_fields.set(definition.metadata_fields);
                            metadata_values.set(HashMap::new());
                            metadata_booleans.set(HashMap::new());
                        }
                        Err(_) => {
                            metadata_fields.set(Vec::new());
                            message.set(Some("Metadata fields could not be read.".into()));
                        }
                    }
                }
                Ok(_) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some(
                        "Metadata fields returned an unexpected response.".into(),
                    ));
                }
                Err(_) => {
                    metadata_fields.set(Vec::new());
                    message.set(Some("Could not reach the node type API.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_type_id,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            message,
        );
    }
}

fn submit_create_node(
    selected_node_type_id: RwSignal<String>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let node_type_id = selected_node_type_id.get();
        let node_name = name.get().trim().to_string();
        if node_type_id.is_empty() {
            message.set(Some("Select a node type before saving.".into()));
            return;
        }
        if node_name.is_empty() {
            message.set(Some("Name is required.".into()));
            return;
        }

        let metadata = match collect_node_metadata(
            &metadata_fields.get(),
            &metadata_values.get(),
            &metadata_booleans.get(),
        ) {
            Ok(metadata) => metadata,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };

        let parent_node_id = selected_parent_node_id
            .get()
            .trim()
            .to_string()
            .into_nonempty();
        let payload = CreateNodePayload {
            node_type_id,
            parent_node_id,
            name: node_name,
            metadata,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Create request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::post("/api/admin/nodes")
                .header("Content-Type", "application/json")
                .body(body)
                .expect("json request body should be valid")
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<IdResponse>().await {
                    Ok(created) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(&format!("/organization/{}", created.id));
                        }
                    }
                    Err(_) => {
                        message.set(Some("Create response could not be read.".into()));
                        is_saving.set(false);
                    }
                },
                Ok(response) => {
                    message.set(Some(format!(
                        "Create failed with status {}.",
                        response.status()
                    )));
                    is_saving.set(false);
                }
                Err(_) => {
                    message.set(Some("Could not reach the create node API.".into()));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            selected_node_type_id,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_saving,
            message,
        );
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(unused_variables))]
fn submit_create_form(
    name: RwSignal<String>,
    scope_node_type_id: RwSignal<String>,
    sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let form_name = name.get().trim().to_string();
        if form_name.is_empty() {
            message.set(Some("Form name is required.".into()));
            return;
        }

        let form_slug = unique_slug_from_label(
            &form_name,
            &existing_form_slugs(existing_forms.get_untracked().as_slice()),
        );
        if form_slug.is_empty() {
            message.set(Some("Form name must contain letters or numbers.".into()));
            return;
        }

        let current_fields = fields.get_untracked();
        let prepared_sections = match prepared_form_builder_sections(&sections.get_untracked()) {
            Ok(sections) => sections,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let prepared_fields = match prepared_form_builder_fields(&current_fields) {
            Ok(fields) => fields,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        if prepared_fields.is_empty() {
            message.set(Some("Add at least one field to the form builder.".into()));
            return;
        }

        let payload = CreateFormPayload {
            name: form_name,
            slug: form_slug,
            scope_node_type_id: scope_node_type_id.get().trim().to_string().into_nonempty(),
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Create request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::post("/api/admin/forms")
                .header("Content-Type", "application/json")
                .body(body)
                .expect("json request body should be valid")
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<IdResponse>().await {
                    Ok(created) => {
                        let version_response = gloo_net::http::Request::post(&format!(
                            "/api/admin/forms/{}/versions",
                            created.id
                        ))
                        .header("Content-Type", "application/json")
                        .body("{}")
                        .expect("json request body should be valid")
                        .send()
                        .await;

                        match version_response {
                            Ok(response) if response.status() == 401 => {
                                is_saving.set(false);
                                redirect_to_login();
                            }
                            Ok(response) if response.ok() => {
                                let created_version = match response.json::<IdResponse>().await {
                                    Ok(created_version) => created_version,
                                    Err(_) => {
                                        message.set(Some(
                                            "Form was created, but draft version response could not be read."
                                                .into(),
                                        ));
                                        is_saving.set(false);
                                        return;
                                    }
                                };

                                let mut section_ids = HashMap::new();
                                for section in &prepared_sections {
                                    let section_payload = CreateFormSectionPayload {
                                        title: section.title.clone(),
                                        position: section.position,
                                        description: section.description.clone(),
                                        column_count: section.column_count,
                                    };
                                    let section_body = match serde_json::to_string(&section_payload)
                                    {
                                        Ok(body) => body,
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "{} section request could not be prepared.",
                                                section.title
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    };
                                    let section_response = gloo_net::http::Request::post(&format!(
                                        "/api/admin/form-versions/{}/sections",
                                        created_version.id
                                    ))
                                    .header("Content-Type", "application/json")
                                    .body(section_body)
                                    .expect("json request body should be valid")
                                    .send()
                                    .await;

                                    let created_section = match section_response {
                                        Ok(response) if response.status() == 401 => {
                                            is_saving.set(false);
                                            redirect_to_login();
                                            return;
                                        }
                                        Ok(response) if response.ok() => {
                                            match response.json::<IdResponse>().await {
                                                Ok(created_section) => created_section,
                                                Err(_) => {
                                                    message.set(Some(format!(
                                                        "{} section response could not be read.",
                                                        section.title
                                                    )));
                                                    is_saving.set(false);
                                                    return;
                                                }
                                            }
                                        }
                                        Ok(response) => {
                                            message.set(Some(format!(
                                                "Form was created, but {} section setup failed with status {}.",
                                                section.title,
                                                response.status()
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "Form was created, but the {} section API could not be reached.",
                                                section.title
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    };
                                    section_ids.insert(section.id, created_section.id);
                                }

                                for (index, field) in prepared_fields.iter().enumerate() {
                                    let Some(section_id) = section_ids.get(&field.section_id)
                                    else {
                                        message.set(Some(format!(
                                            "{} field could not be matched to a section.",
                                            field.label
                                        )));
                                        is_saving.set(false);
                                        return;
                                    };
                                    let field_payload = CreateFormFieldPayload {
                                        section_id: section_id.clone(),
                                        key: field.key.clone(),
                                        label: field.label.clone(),
                                        field_type: field.field_type.clone(),
                                        required: field.required,
                                        position: (index + 1) as i32,
                                        grid_row: field.grid_row,
                                        grid_column: field.grid_column,
                                        grid_width: field.grid_width,
                                        grid_height: field.grid_height,
                                    };
                                    let field_body = match serde_json::to_string(&field_payload) {
                                        Ok(body) => body,
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "{} field request could not be prepared.",
                                                field.label
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    };
                                    let field_response = gloo_net::http::Request::post(&format!(
                                        "/api/admin/form-versions/{}/fields",
                                        created_version.id
                                    ))
                                    .header("Content-Type", "application/json")
                                    .body(field_body)
                                    .expect("json request body should be valid")
                                    .send()
                                    .await;

                                    match field_response {
                                        Ok(response) if response.status() == 401 => {
                                            is_saving.set(false);
                                            redirect_to_login();
                                            return;
                                        }
                                        Ok(response) if response.ok() => {}
                                        Ok(response) => {
                                            message.set(Some(format!(
                                                "{} field setup failed with status {}.",
                                                field.label,
                                                response.status()
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                        Err(_) => {
                                            message.set(Some(format!(
                                                "{} field API could not be reached.",
                                                field.label
                                            )));
                                            is_saving.set(false);
                                            return;
                                        }
                                    }
                                }

                                if let Some(window) = web_sys::window() {
                                    let _ = window
                                        .location()
                                        .set_href(&format!("/forms/{}", created.id));
                                }
                            }
                            Ok(response) => {
                                message.set(Some(format!(
                                    "Form was created, but draft version setup failed with status {}.",
                                    response.status()
                                )));
                                is_saving.set(false);
                            }
                            Err(_) => {
                                message.set(Some(
                                    "Form was created, but the draft version API could not be reached."
                                        .into(),
                                ));
                                is_saving.set(false);
                            }
                        }
                    }
                    Err(_) => {
                        message.set(Some("Create response could not be read.".into()));
                        is_saving.set(false);
                    }
                },
                Ok(response) => {
                    message.set(Some(format!(
                        "Create failed with status {}.",
                        response.status()
                    )));
                    is_saving.set(false);
                }
                Err(_) => {
                    message.set(Some("Could not reach the create form API.".into()));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            name,
            scope_node_type_id,
            fields,
            existing_forms,
            is_saving,
            message,
        );
    }
}

fn load_organization_edit_options(
    node_id: String,
    node_types: RwSignal<Vec<NodeTypeCatalogEntry>>,
    nodes: RwSignal<Vec<OrganizationNode>>,
    detail: RwSignal<Option<OrganizationNodeDetail>>,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_loading: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            is_loading.set(true);
            message.set(None);

            let node_type_response = gloo_net::http::Request::get("/api/node-types").send().await;
            let node_response = gloo_net::http::Request::get("/api/nodes").send().await;
            let detail_response = gloo_net::http::Request::get(&format!("/api/nodes/{node_id}"))
                .send()
                .await;

            match (node_type_response, node_response, detail_response) {
                (Ok(response), _, _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, Ok(response), _) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (_, _, Ok(response)) if response.status() == 401 => {
                    is_loading.set(false);
                    redirect_to_login();
                }
                (Ok(node_type_response), Ok(node_response), Ok(detail_response))
                    if node_type_response.ok() && node_response.ok() && detail_response.ok() =>
                {
                    let loaded_node_types =
                        node_type_response.json::<Vec<NodeTypeCatalogEntry>>().await;
                    let loaded_nodes = node_response.json::<Vec<OrganizationNode>>().await;
                    let loaded_detail = detail_response.json::<OrganizationNodeDetail>().await;

                    match (loaded_node_types, loaded_nodes, loaded_detail) {
                        (Ok(loaded_node_types), Ok(loaded_nodes), Ok(loaded_detail)) => {
                            let metadata_response = gloo_net::http::Request::get(&format!(
                                "/api/admin/node-types/{}",
                                loaded_detail.node_type_id
                            ))
                            .send()
                            .await;

                            match metadata_response {
                                Ok(response) if response.status() == 401 => {
                                    is_loading.set(false);
                                    redirect_to_login();
                                }
                                Ok(response) if response.ok() => {
                                    match response.json::<NodeTypeDefinition>().await {
                                        Ok(definition) => {
                                            let (text_values, boolean_values) =
                                                metadata_input_state(
                                                    &definition.metadata_fields,
                                                    &loaded_detail.metadata,
                                                );

                                            selected_parent_node_id.set(
                                                loaded_detail
                                                    .parent_node_id
                                                    .clone()
                                                    .unwrap_or_default(),
                                            );
                                            name.set(loaded_detail.name.clone());
                                            metadata_fields.set(definition.metadata_fields);
                                            metadata_values.set(text_values);
                                            metadata_booleans.set(boolean_values);
                                            detail.set(Some(loaded_detail));
                                            nodes.set(loaded_nodes);
                                            node_types.set(loaded_node_types);
                                            is_loading.set(false);
                                        }
                                        Err(_) => {
                                            is_loading.set(false);
                                            message.set(Some(
                                                "Metadata fields could not be read.".into(),
                                            ));
                                        }
                                    }
                                }
                                Ok(_) => {
                                    is_loading.set(false);
                                    message.set(Some(
                                        "Metadata fields returned an unexpected response.".into(),
                                    ));
                                }
                                Err(_) => {
                                    is_loading.set(false);
                                    message.set(Some("Could not reach the node type API.".into()));
                                }
                            }
                        }
                        _ => {
                            is_loading.set(false);
                            message.set(Some("Edit options could not be read.".into()));
                        }
                    }
                }
                (Ok(_), Ok(_), Ok(_)) => {
                    is_loading.set(false);
                    message.set(Some("Edit options returned an unexpected response.".into()));
                }
                _ => {
                    is_loading.set(false);
                    message.set(Some("Could not reach the organization APIs.".into()));
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_id,
            node_types,
            nodes,
            detail,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_loading,
            message,
        );
    }
}

fn submit_update_node(
    node_id: String,
    selected_parent_node_id: RwSignal<String>,
    name: RwSignal<String>,
    metadata_fields: RwSignal<Vec<NodeMetadataFieldSummary>>,
    metadata_values: RwSignal<HashMap<String, String>>,
    metadata_booleans: RwSignal<HashMap<String, bool>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let node_name = name.get().trim().to_string();
        if node_name.is_empty() {
            message.set(Some("Name is required.".into()));
            return;
        }

        let metadata = match collect_node_metadata(
            &metadata_fields.get(),
            &metadata_values.get(),
            &metadata_booleans.get(),
        ) {
            Ok(metadata) => metadata,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };

        let payload = UpdateNodePayload {
            parent_node_id: selected_parent_node_id
                .get()
                .trim()
                .to_string()
                .into_nonempty(),
            name: node_name,
            metadata,
        };

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Update request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            let response = gloo_net::http::Request::put(&format!("/api/admin/nodes/{node_id}"))
                .header("Content-Type", "application/json")
                .body(body)
                .expect("json request body should be valid")
                .send()
                .await;

            match response {
                Ok(response) if response.status() == 401 => {
                    is_saving.set(false);
                    redirect_to_login();
                }
                Ok(response) if response.ok() => match response.json::<IdResponse>().await {
                    Ok(updated) => {
                        if let Some(window) = web_sys::window() {
                            let _ = window
                                .location()
                                .set_href(&format!("/organization/{}", updated.id));
                        }
                    }
                    Err(_) => {
                        message.set(Some("Update response could not be read.".into()));
                        is_saving.set(false);
                    }
                },
                Ok(response) => {
                    message.set(Some(format!(
                        "Update failed with status {}.",
                        response.status()
                    )));
                    is_saving.set(false);
                }
                Err(_) => {
                    message.set(Some("Could not reach the update node API.".into()));
                    is_saving.set(false);
                }
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            node_id,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_saving,
            message,
        );
    }
}

fn submit_update_form(
    form_id: String,
    name: RwSignal<String>,
    scope_node_type_id: RwSignal<String>,
    sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    existing_forms: RwSignal<Vec<FormSummary>>,
    edit_version_id: RwSignal<Option<String>>,
    edit_version_status: RwSignal<Option<String>>,
    rendered_form: RwSignal<Option<RenderedForm>>,
    is_saving: RwSignal<bool>,
    message: RwSignal<Option<String>>,
) {
    #[cfg(feature = "hydrate")]
    {
        if is_saving.get() {
            return;
        }

        let form_name = name.get().trim().to_string();
        if form_name.is_empty() {
            message.set(Some("Form name is required.".into()));
            return;
        }

        let form_slug = unique_slug_from_label(
            &form_name,
            &existing_form_slugs_for_update(existing_forms.get_untracked().as_slice(), &form_id),
        );
        if form_slug.is_empty() {
            message.set(Some("Form name must contain letters or numbers.".into()));
            return;
        }

        let current_sections = sections.get_untracked();
        let current_fields = fields.get_untracked();
        let prepared_sections = match prepared_form_builder_sections(&current_sections) {
            Ok(sections) => sections,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        let prepared_fields = match prepared_form_builder_fields(&current_fields) {
            Ok(fields) => fields,
            Err(error) => {
                message.set(Some(error));
                return;
            }
        };
        if prepared_fields.is_empty() {
            message.set(Some("Add at least one field to the form builder.".into()));
            return;
        }

        let payload = UpdateFormPayload {
            name: form_name,
            slug: form_slug,
            scope_node_type_id: scope_node_type_id.get().trim().to_string().into_nonempty(),
        };
        let current_rendered_form = rendered_form.get_untracked();
        let original_section_ids = current_rendered_form
            .as_ref()
            .map(|rendered| {
                rendered
                    .sections
                    .iter()
                    .map(|section| section.id.clone())
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        let original_field_ids = current_rendered_form
            .as_ref()
            .map(|rendered| {
                rendered
                    .sections
                    .iter()
                    .flat_map(|section| section.fields.iter().map(|field| field.id.clone()))
                    .collect::<HashSet<_>>()
            })
            .unwrap_or_default();
        let kept_section_ids = prepared_sections
            .iter()
            .filter_map(|section| section.remote_id.clone())
            .collect::<HashSet<_>>();
        let kept_field_ids = prepared_fields
            .iter()
            .filter_map(|field| field.remote_id.clone())
            .collect::<HashSet<_>>();
        let update_existing_draft = edit_version_status.get_untracked().as_deref() == Some("draft");
        let existing_version_id = edit_version_id.get_untracked();

        leptos::task::spawn_local(async move {
            is_saving.set(true);
            message.set(None);

            let body = match serde_json::to_string(&payload) {
                Ok(body) => body,
                Err(_) => {
                    message.set(Some("Update request could not be prepared.".into()));
                    is_saving.set(false);
                    return;
                }
            };

            if let Err(error) = send_json_id_request(
                gloo_net::http::Request::put(&format!("/api/admin/forms/{form_id}")),
                Some(body),
                "Update form",
            )
            .await
            {
                message.set(Some(error));
                is_saving.set(false);
                return;
            }

            let version_id = if update_existing_draft {
                match existing_version_id {
                    Some(version_id) => version_id,
                    None => {
                        message.set(Some("No editable draft version was available.".into()));
                        is_saving.set(false);
                        return;
                    }
                }
            } else {
                match send_json_id_request(
                    gloo_net::http::Request::post(&format!("/api/admin/forms/{form_id}/versions")),
                    Some("{}".into()),
                    "Create draft version",
                )
                .await
                {
                    Ok(created) => created.id,
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            };

            if update_existing_draft {
                for field_id in original_field_ids.difference(&kept_field_ids) {
                    if let Err(error) = send_json_id_request(
                        gloo_net::http::Request::delete(&format!(
                            "/api/admin/form-fields/{field_id}"
                        )),
                        None,
                        "Delete form field",
                    )
                    .await
                    {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }

                for section_id in original_section_ids.difference(&kept_section_ids) {
                    if let Err(error) = send_json_id_request(
                        gloo_net::http::Request::delete(&format!(
                            "/api/admin/form-sections/{section_id}"
                        )),
                        None,
                        "Delete form section",
                    )
                    .await
                    {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            }

            let mut section_ids = HashMap::new();
            for section in &prepared_sections {
                let section_payload = CreateFormSectionPayload {
                    title: section.title.clone(),
                    position: section.position,
                    description: section.description.clone(),
                    column_count: section.column_count,
                };
                let section_body = match serde_json::to_string(&section_payload) {
                    Ok(body) => body,
                    Err(_) => {
                        message.set(Some(format!(
                            "{} section request could not be prepared.",
                            section.title
                        )));
                        is_saving.set(false);
                        return;
                    }
                };

                let request = if update_existing_draft {
                    section
                        .remote_id
                        .as_ref()
                        .map(|section_id| {
                            (
                                gloo_net::http::Request::put(&format!(
                                    "/api/admin/form-sections/{section_id}"
                                )),
                                "Update form section",
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                gloo_net::http::Request::post(&format!(
                                    "/api/admin/form-versions/{version_id}/sections"
                                )),
                                "Create form section",
                            )
                        })
                } else {
                    (
                        gloo_net::http::Request::post(&format!(
                            "/api/admin/form-versions/{version_id}/sections"
                        )),
                        "Create form section",
                    )
                };

                match send_json_id_request(request.0, Some(section_body), request.1).await {
                    Ok(saved_section) => {
                        section_ids.insert(section.id, saved_section.id);
                    }
                    Err(error) => {
                        message.set(Some(error));
                        is_saving.set(false);
                        return;
                    }
                }
            }

            for (index, field) in prepared_fields.iter().enumerate() {
                let Some(section_id) = section_ids.get(&field.section_id) else {
                    message.set(Some(format!(
                        "{} field could not be matched to a section.",
                        field.label
                    )));
                    is_saving.set(false);
                    return;
                };
                let field_payload = CreateFormFieldPayload {
                    section_id: section_id.clone(),
                    key: field.key.clone(),
                    label: field.label.clone(),
                    field_type: field.field_type.clone(),
                    required: field.required,
                    position: (index + 1) as i32,
                    grid_row: field.grid_row,
                    grid_column: field.grid_column,
                    grid_width: field.grid_width,
                    grid_height: field.grid_height,
                };
                let field_body = match serde_json::to_string(&field_payload) {
                    Ok(body) => body,
                    Err(_) => {
                        message.set(Some(format!(
                            "{} field request could not be prepared.",
                            field.label
                        )));
                        is_saving.set(false);
                        return;
                    }
                };

                let request = if update_existing_draft {
                    field
                        .remote_id
                        .as_ref()
                        .map(|field_id| {
                            (
                                gloo_net::http::Request::put(&format!(
                                    "/api/admin/form-fields/{field_id}"
                                )),
                                "Update form field",
                            )
                        })
                        .unwrap_or_else(|| {
                            (
                                gloo_net::http::Request::post(&format!(
                                    "/api/admin/form-versions/{version_id}/fields"
                                )),
                                "Create form field",
                            )
                        })
                } else {
                    (
                        gloo_net::http::Request::post(&format!(
                            "/api/admin/form-versions/{version_id}/fields"
                        )),
                        "Create form field",
                    )
                };

                if let Err(error) =
                    send_json_id_request(request.0, Some(field_body), request.1).await
                {
                    message.set(Some(error));
                    is_saving.set(false);
                    return;
                }
            }

            if let Some(window) = web_sys::window() {
                let _ = window.location().set_href(&format!("/forms/{form_id}"));
            }
        });
    }

    #[cfg(not(feature = "hydrate"))]
    {
        let _ = (
            form_id,
            name,
            scope_node_type_id,
            sections,
            fields,
            existing_forms,
            edit_version_id,
            edit_version_status,
            rendered_form,
            is_saving,
            message,
        );
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn collect_node_metadata(
    fields: &[NodeMetadataFieldSummary],
    values: &HashMap<String, String>,
    booleans: &HashMap<String, bool>,
) -> Result<serde_json::Map<String, Value>, String> {
    let mut metadata = serde_json::Map::new();

    for field in fields {
        match field.field_type.as_str() {
            "boolean" => {
                metadata.insert(
                    field.key.clone(),
                    Value::Bool(booleans.get(&field.key).copied().unwrap_or(false)),
                );
            }
            "number" => {
                let raw = values
                    .get(&field.key)
                    .map(|value| value.trim())
                    .unwrap_or_default();
                if raw.is_empty() {
                    if field.required {
                        return Err(format!("{} is required.", field.label));
                    }
                } else {
                    let parsed = raw
                        .parse::<f64>()
                        .map_err(|_| format!("{} must be a number.", field.label))?;
                    metadata.insert(field.key.clone(), serde_json::json!(parsed));
                }
            }
            "multi_choice" => {
                let raw = values
                    .get(&field.key)
                    .map(|value| value.trim())
                    .unwrap_or_default();
                let selected = raw
                    .split(',')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| Value::String(value.to_string()))
                    .collect::<Vec<_>>();
                if selected.is_empty() {
                    if field.required {
                        return Err(format!("{} is required.", field.label));
                    }
                } else {
                    metadata.insert(field.key.clone(), Value::Array(selected));
                }
            }
            _ => {
                let raw = values
                    .get(&field.key)
                    .map(|value| value.trim())
                    .unwrap_or_default();
                if raw.is_empty() {
                    if field.required {
                        return Err(format!("{} is required.", field.label));
                    }
                } else {
                    metadata.insert(field.key.clone(), Value::String(raw.to_string()));
                }
            }
        }
    }

    Ok(metadata)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn metadata_input_state(
    fields: &[NodeMetadataFieldSummary],
    metadata: &Value,
) -> (HashMap<String, String>, HashMap<String, bool>) {
    let values = metadata.as_object();
    let mut text_values = HashMap::new();
    let mut boolean_values = HashMap::new();

    for field in fields {
        let value = values.and_then(|values| values.get(&field.key));
        if field.field_type == "boolean" {
            boolean_values.insert(
                field.key.clone(),
                value.and_then(Value::as_bool).unwrap_or(false),
            );
        } else if let Some(value) = value {
            text_values.insert(field.key.clone(), metadata_input_value(value));
        }
    }

    (text_values, boolean_values)
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
fn metadata_input_value(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(value) => value.to_string(),
        Value::Number(value) => value.to_string(),
        Value::String(value) => value.clone(),
        Value::Array(values) => values
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Object(_) => value.to_string(),
    }
}

#[cfg_attr(not(feature = "hydrate"), allow(dead_code))]
trait IntoNonemptyString {
    fn into_nonempty(self) -> Option<String>;
}

impl IntoNonemptyString for String {
    fn into_nonempty(self) -> Option<String> {
        if self.is_empty() { None } else { Some(self) }
    }
}

#[cfg(feature = "hydrate")]
fn current_search_param(name: &str) -> Option<String> {
    let search = web_sys::window().and_then(|window| window.location().search().ok())?;
    let params = web_sys::UrlSearchParams::new_with_str(&search).ok()?;
    params.get(name).filter(|value| !value.is_empty())
}

#[component]
pub fn OrganizationDetailPage() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;
    let detail = RwSignal::new(None::<OrganizationNodeDetail>);
    let is_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_organization_detail(node_id.clone(), detail, is_loading, error);
    });

    view! {
        <AppShell active_route="organization" title="Organization">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/organization">"Organization"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Detail"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>

            <section class="route-panel organization-page organization-detail-page">
                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading detail"</h3>
                                <p>"Fetching organization node details."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Organization detail unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(node_detail) = detail.get() {
                        let edit_href = format!("/organization/{}/edit", node_detail.id);
                        view! {
                            <PageHeader title="Organization Detail">
                                <a class="button" href=edit_href>"Edit Node"</a>
                            </PageHeader>
                            <OrganizationDetailFullContent detail=node_detail/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Organization detail unavailable"
                                message="The selected node could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn OrganizationEditPage() -> impl IntoView {
    let params = require_route_params::<NodeRouteParams>();
    let node_id = params.node_id;
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let detail = RwSignal::new(None::<OrganizationNodeDetail>);
    let selected_parent_node_id = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let metadata_fields = RwSignal::new(Vec::<NodeMetadataFieldSummary>::new());
    let metadata_values = RwSignal::new(HashMap::<String, String>::new());
    let metadata_booleans = RwSignal::new(HashMap::<String, bool>::new());
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);

    let load_node_id = node_id.clone();
    Effect::new(move |_| {
        load_organization_edit_options(
            load_node_id.clone(),
            node_types,
            nodes,
            detail,
            selected_parent_node_id,
            name,
            metadata_fields,
            metadata_values,
            metadata_booleans,
            is_loading,
            message,
        );
    });

    let option_node_id = node_id.clone();
    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="organization" title="Organization">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/organization">"Organization"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail.get().map(|node| {
                        let href = format!("/organization/{}", node.id);
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbLink href=href>{node.name}</BreadcrumbLink>
                            </BreadcrumbItem>
                            <BreadcrumbSeparator/>
                        }
                    })
                }}
                <BreadcrumbItem>
                    <BreadcrumbPage>"Edit Node"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel organization-page organization-edit-page">
                <PageHeader title="Edit Organization Node"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading edit options"</h3>
                                <p>"Fetching organization node details."</p>
                            </section>
                        }
                        .into_any()
                    } else if detail.get().is_none() {
                        view! {
                            <EmptyState
                                title="Organization node unavailable"
                                message="The selected node could not be loaded for editing."
                            />
                        }
                        .into_any()
                    } else {
                        let node = detail.get().expect("detail is checked above");
                        let option_node_id_for_options = option_node_id.clone();
                        let submit_node_id = node_id.clone();
                        view! {
                            <form
                                class="native-form organization-node-form"
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit_update_node(
                                        submit_node_id.clone(),
                                        selected_parent_node_id,
                                        name,
                                        metadata_fields,
                                        metadata_values,
                                        metadata_booleans,
                                        is_saving,
                                        message,
                                    );
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field" for="organization-node-type">
                                        <span>"Node Type"</span>
                                        <input
                                            id="organization-node-type"
                                            type="text"
                                            value=node.node_type_singular_label
                                            readonly
                                        />
                                    </label>

                                    <label class="form-field" for="organization-parent-node">
                                        <span>"Parent Node"</span>
                                        <select
                                            id="organization-parent-node"
                                            prop:value=move || selected_parent_node_id.get()
                                            on:change=move |event| selected_parent_node_id.set(event_target_value(&event))
                                        >
                                            <Show when=move || {
                                                detail
                                                    .get()
                                                    .and_then(|detail| {
                                                        node_types
                                                            .get()
                                                            .into_iter()
                                                            .find(|node_type| node_type.id == detail.node_type_id)
                                                    })
                                                    .map(|node_type| node_type.is_root_type)
                                                    .unwrap_or(false)
                                            }>
                                                <option value="">"Top-level record"</option>
                                            </Show>
                                            {move || {
                                                detail
                                                    .get()
                                                    .map(|detail| {
                                                        parent_node_options_for_edit(
                                                            &nodes.get(),
                                                            &node_types.get(),
                                                            &option_node_id_for_options,
                                                            &detail.node_type_id,
                                                        )
                                                    })
                                                    .unwrap_or_default()
                                                    .into_iter()
                                                    .map(|option| {
                                                        view! {
                                                            <option value=option.id>{option.label}</option>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                        </select>
                                    </label>

                                    <label class="form-field form-field--wide" for="organization-name">
                                        <span>"Name"</span>
                                        <input
                                            id="organization-name"
                                            type="text"
                                            autocomplete="off"
                                            prop:value=move || name.get()
                                            on:input=move |event| name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>
                                </div>

                                <section class="form-section">
                                    <h3>"Metadata"</h3>
                                    {move || {
                                        let fields = metadata_fields.get();
                                        if fields.is_empty() {
                                            view! { <p class="muted">"No metadata fields are configured for this node type."</p> }.into_any()
                                        } else {
                                            view! {
                                                <div class="form-grid">
                                                    {fields.into_iter().map(|field| {
                                                        view! {
                                                            <MetadataFieldInput
                                                                field
                                                                metadata_values
                                                                metadata_booleans
                                                            />
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            }
                                            .into_any()
                                        }
                                    }}
                                </section>

                                {move || message.get().map(|message| view! {
                                    <p class="form-message" role="status">{message}</p>
                                })}

                                <div class="form-actions">
                                    <Button label="Cancel" href="/organization"/>
                                    <button class="button" type="submit" disabled=move || !can_submit()>
                                        {move || if is_saving.get() { "Saving..." } else { "Save Changes" }}
                                    </button>
                                </div>
                            </form>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn FormsPage() -> impl IntoView {
    let forms = RwSignal::new(Vec::<FormSummary>::new());
    let search = RwSignal::new(String::new());
    let scope_filter = RwSignal::new("all".to_string());
    let status_filter = RwSignal::new("all".to_string());
    let node_filter_query = RwSignal::new(String::new());
    let selected_node_id = RwSignal::new(None::<String>);
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_forms(forms, is_loading, load_error);
    });

    let filtered_forms = move || {
        let query = search.get();
        let selected_scope = scope_filter.get();
        let selected_status = status_filter.get();
        let selected_node = selected_node_id.get();
        let loaded_forms = forms.get();
        let node_options = form_node_filter_options(&loaded_forms);

        loaded_forms
            .into_iter()
            .filter(|form| {
                let active_version = active_form_version(form);
                let scope = form_scope_label(form);
                let attached_to = form_attached_to_label(active_version);
                let status = form_status_label(active_version);
                let matches_scope = selected_scope == "all" || scope == selected_scope;
                let matches_status = selected_status == "all" || status == selected_status;
                let matches_node_filter =
                    form_matches_node_filter(form, selected_node.as_deref(), &node_options);
                if !matches_scope || !matches_status || !matches_node_filter {
                    return false;
                }
                text_matches(
                    &query,
                    &[
                        &form.name,
                        &form.slug,
                        &scope,
                        &attached_to,
                        &form_version_label(active_version),
                        &status,
                    ],
                )
            })
            .collect::<Vec<_>>()
    };

    let scope_options =
        move || unique_filter_options(forms.get().iter().map(form_scope_label).collect::<Vec<_>>());
    let status_options = move || {
        unique_filter_options(
            forms
                .get()
                .iter()
                .map(|form| form_status_label(active_form_version(form)))
                .collect::<Vec<_>>(),
        )
    };
    let node_filter_options = move || form_node_filter_options(&forms.get());

    view! {
        <AppShell active_route="forms" title="Forms">
            <section class="route-panel forms-page">
                <PageHeader title="Forms">
                    <Button label="Create Form" href="/forms/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading forms"</h3>
                                <p>"Fetching available form definitions."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Forms unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <FormsList
                                forms=filtered_forms()
                                search
                                scope_filter
                                status_filter
                                node_filter_query
                                selected_node_id
                                scope_options=scope_options()
                                status_options=status_options()
                                node_filter_options=node_filter_options()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn FormsNodeLineageFilter(
    options: Vec<FormNodeFilterOption>,
    selected_node_id: RwSignal<Option<String>>,
    query: RwSignal<String>,
) -> impl IntoView {
    let is_open = RwSignal::new(false);
    let options_for_visible = options.clone();
    let options_for_label = options.clone();
    let options_for_selected = options.clone();
    let trigger_label = move || {
        let selected = selected_node_id.get();
        selected
            .as_deref()
            .and_then(|id| {
                options_for_label
                    .iter()
                    .find(|option| option.id == id)
                    .map(|option| option.name.clone())
            })
            .unwrap_or_else(|| "Filter by node".to_string())
    };
    let trigger_class = move || {
        if selected_node_id.get().is_none() {
            "forms-node-filter__trigger"
        } else {
            "forms-node-filter__trigger is-filtered"
        }
    };
    let visible_options = move || {
        visible_form_node_filter_options(
            &options_for_visible,
            selected_node_id.get().as_deref(),
            &query.get(),
        )
    };
    let selected_options = move || {
        selected_node_id
            .get()
            .as_deref()
            .and_then(|selected| {
                options_for_selected
                    .iter()
                    .find(|option| option.id == selected)
                    .cloned()
            })
            .into_iter()
            .collect::<Vec<_>>()
    };

    view! {
        <div class=move || if is_open.get() { "forms-node-filter is-open" } else { "forms-node-filter" }>
            <button
                class=trigger_class
                type="button"
                role="combobox"
                aria-haspopup="listbox"
                aria-expanded=move || is_open.get().to_string()
                aria-label="Filter forms by organization node"
                title="Filter forms by organization node"
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <ListFilter/>
                <span>{trigger_label}</span>
                <ChevronDown/>
            </button>
            <button
                class="forms-node-filter__scrim"
                type="button"
                aria-label="Close node filter"
                on:click=move |_| is_open.set(false)
            ></button>
            <div
                class="forms-node-filter__menu blurred-surface floating-layer"
                data-mobile-behavior="dialog"
                role="dialog"
                aria-label="Filter by organization node"
            >
                <label class="forms-node-filter__search">
                    <Search/>
                    <span class="sr-only">"Search organization nodes"</span>
                    <input
                        type="search"
                        placeholder="Search organization nodes"
                        prop:value=move || query.get()
                        on:input=move |event| query.set(event_target_value(&event))
                    />
                </label>
                <div class="forms-node-filter__selected">
                    {move || {
                        let selected = selected_options();
                        if selected.is_empty() {
                            view! { <p>"No node selected"</p> }.into_any()
                        } else {
                            view! {
                                <div class="forms-node-filter__chips">
                                    {selected
                                        .into_iter()
                                        .map(|option| {
                                            let option_id = option.id.clone();
                                            view! {
                                                <button
                                                    class="forms-node-filter__chip"
                                                    type="button"
                                                    title=option.path
                                                    on:click=move |_| {
                                                        if selected_node_id.get().as_deref() == Some(option_id.as_str()) {
                                                            selected_node_id.set(None);
                                                        }
                                                    }
                                                >
                                                    <span>{option.name}</span>
                                                    <X/>
                                                </button>
                                            }
                                        })
                                        .collect_view()}
                                </div>
                            }
                            .into_any()
                        }
                    }}
                    <button
                        class="forms-node-filter__clear"
                        type="button"
                        disabled=move || selected_node_id.get().is_none() && query.get().is_empty()
                        on:click=move |_| {
                            selected_node_id.set(None);
                            query.set(String::new());
                        }
                    >
                        "Clear"
                    </button>
                </div>
                <div class="forms-node-filter__options" role="listbox">
                    {move || {
                        let visible = visible_options();
                        if visible.is_empty() {
                            view! {
                                <p class="forms-node-filter__empty">"No matching nodes to display"</p>
                            }
                            .into_any()
                        } else {
                            visible
                                .into_iter()
                                .map(|option| {
                                    let option_id = option.id.clone();
                                    let label = indented_node_label(&option);
                                    let path = option.path.clone();
                                    view! {
                                        <button
                                            class="forms-node-filter__option"
                                            type="button"
                                            role="option"
                                            aria-selected="false"
                                            title=path
                                            on:click=move |_| {
                                                selected_node_id.set(Some(option_id.clone()));
                                                query.set(String::new());
                                            }
                                        >
                                            <span>{label}</span>
                                        </button>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

#[component]
fn FormsList(
    forms: Vec<FormSummary>,
    search: RwSignal<String>,
    scope_filter: RwSignal<String>,
    status_filter: RwSignal<String>,
    node_filter_query: RwSignal<String>,
    selected_node_id: RwSignal<Option<String>>,
    scope_options: Vec<String>,
    status_options: Vec<String>,
    node_filter_options: Vec<FormNodeFilterOption>,
) -> impl IntoView {
    let table_forms = forms.clone();
    let card_forms = forms;
    let attached_nodes_sheet = RwSignal::new(None::<FormsAttachedNodesSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search forms"</span>
                        <input
                            type="search"
                            placeholder="Search forms"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                    <FormsNodeLineageFilter
                        options=node_filter_options
                        selected_node_id
                        query=node_filter_query
                    />
                </div>
                <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Form name"</th>
                        <th scope="col">
                            <FilterHeader
                                label="Scope"
                                all_label="All scopes"
                                filter=scope_filter
                                options=scope_options
                            />
                        </th>
                        <th scope="col">"Attached To"</th>
                        <th class="data-table__cell--center" scope="col">"Active version"</th>
                        <th class="data-table__cell--center" scope="col">
                            <FilterHeader
                                label="Status"
                                all_label="All statuses"
                                filter=status_filter
                                options=status_options
                            />
                        </th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {if table_forms.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="6">"No Forms to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_forms
                            .into_iter()
                            .map(|form| {
                                let href = format!("/forms/{}", form.id);
                                let active_version = active_form_version(&form);
                                let status = active_version
                                    .map(|version| version.status.as_str())
                                    .unwrap_or("none");
                                let name = form.name.clone();
                                let scope = form_scope_label(&form);
                                let attached_nodes = form_attached_nodes(active_version);
                                let attached_nodes_form_name = name.clone();
                                let version_label = form_version_label(active_version);
                                let status_label = form_status_label(active_version);
                                let field_count = form_field_count_label(active_version);
                                view! {
                                    <tr>
                                        <th scope="row">
                                            <a class="data-table__primary-link" href=href.clone()>{name}</a>
                                        </th>
                                        <td>{scope}</td>
                                        <td>
                                            <FormsAttachedNodesList
                                                nodes=attached_nodes
                                                form_name=attached_nodes_form_name
                                                form_href=href
                                                sheet=attached_nodes_sheet
                                            />
                                        </td>
                                        <td class="data-table__cell--center">{version_label}</td>
                                        <td class="data-table__cell--center"><span class=status_badge_class(status)>{status_label}</span></td>
                                        <td class="data-table__cell--center">{field_count}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
                </DataTable>
            </div>
            <div class="forms-list-mobile-cards">
                {if card_forms.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Forms to Display"</p> }.into_any()
                } else {
                    card_forms
                        .into_iter()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            let active_version = active_form_version(&form);
                            let status = active_version
                                .map(|version| version.status.as_str())
                                .unwrap_or("none");
                            let name = form.name.clone();
                            let scope = form_scope_label(&form);
                            let attached_nodes = form_attached_nodes(active_version);
                            let attached_nodes_form_name = name.clone();
                            let version_label = form_version_label(active_version);
                            let status_label = form_status_label(active_version);
                            let field_count = form_field_count_label(active_version);
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=href.clone()>{name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Scope"</dt>
                                            <dd>{scope}</dd>
                                        </div>
                                        <div>
                                            <dt>"Attached To"</dt>
                                            <dd>
                                                <FormsAttachedNodesList
                                                    nodes=attached_nodes
                                                    form_name=attached_nodes_form_name
                                                    form_href=href
                                                    sheet=attached_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Active version"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(status)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{field_count}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <FormsAttachedNodesSheet detail=attached_nodes_sheet/>
        </div>
    }
}

#[component]
fn FormsAttachedNodesList(
    nodes: Vec<FormAttachmentLink>,
    form_name: String,
    form_href: String,
    sheet: RwSignal<Option<FormsAttachedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let visible_nodes = if total_nodes > 5 {
        nodes[total_nodes - 4..].to_vec()
    } else {
        nodes.clone()
    };
    let nodes_for_sheet = nodes.clone();
    let form_name_for_sheet = form_name.clone();
    let form_href_for_sheet = form_href.clone();

    view! {
        <div class="forms-attached-list">
            {if visible_nodes.is_empty() {
                view! { <p>"Not attached"</p> }.into_any()
            } else {
                visible_nodes
                    .into_iter()
                    .map(|node| {
                        view! {
                            <p>
                                <a href=node.href title=node.title>{node.label}</a>
                            </p>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
            {if total_nodes > 5 {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        on:click=move |_| {
                            sheet.set(Some(FormsAttachedNodesSheetData {
                                form_name: form_name_for_sheet.clone(),
                                form_href: form_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        "More Nodes..."
                    </button>
                }
                .into_any()
            } else {
                view! {}.into_any()
            }}
        </div>
    }
}

#[component]
fn FormsAttachedNodesSheet(detail: RwSignal<Option<FormsAttachedNodesSheetData>>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Attached organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close attached nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Attached organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.form_href aria-label="Open form detail" title="Open form detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(|| view! {}.into_any())
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close attached nodes" title="Close attached nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Attached Nodes"</p>
                                            <h2>{data.form_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search attached nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search attached nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Attached Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(|| view! {}.into_any())
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
fn WorkflowsList(
    workflows: Vec<WorkflowSummary>,
    search: RwSignal<String>,
    status_filter: RwSignal<String>,
    status_options: Vec<String>,
    organization_nodes: Vec<OrganizationNode>,
) -> impl IntoView {
    let table_workflows = workflows.clone();
    let card_workflows = workflows;
    let table_nodes = organization_nodes.clone();
    let card_nodes = organization_nodes;
    let assigned_nodes_sheet = RwSignal::new(None::<WorkflowAssignedNodesSheetData>);

    view! {
        <div class="forms-list forms-list-responsive-table">
            <div class="searchable-data-table">
                <div class="searchable-data-table__toolbar forms-list__toolbar">
                    <label class="searchable-data-table__search searchable-data-table__control">
                        <Search class="searchable-data-table__control-icon"/>
                        <span class="sr-only">"Search workflows"</span>
                        <input
                            type="search"
                            placeholder="Search workflows"
                            prop:value=move || search.get()
                            on:input=move |event| search.set(event_target_value(&event))
                        />
                    </label>
                </div>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Workflow name"</th>
                            <th scope="col">"Form"</th>
                            <th class="data-table__cell--center" scope="col">"Active version"</th>
                            <th class="data-table__cell--center" scope="col">
                                <FilterHeader
                                    label="Status"
                                    all_label="All statuses"
                                    filter=status_filter
                                    options=status_options
                                />
                            </th>
                            <th class="data-table__cell--center" scope="col">"Assignments"</th>
                            <th scope="col">"Assigned to"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {if table_workflows.is_empty() {
                            view! {
                                <tr>
                                    <td class="data-table__empty" colspan="6">"No Workflows to Display"</td>
                                </tr>
                            }
                            .into_any()
                        } else {
                            table_workflows
                                .into_iter()
                                .map(|workflow| {
                                    let workflow_href = format!("/workflows/{}", workflow.id);
                                    let form_href = format!("/forms/{}", workflow.form_id);
                                    let status_key = workflow_status_key(&workflow).to_string();
                                    let status_label = workflow_status_label(&workflow);
                                    let version_label = workflow_version_label(&workflow);
                                    let assignments = workflow_assignment_count_label(&workflow);
                                    let assigned_to = workflow_assignment_links(&workflow, &table_nodes);
                                    let assigned_to_label = workflow_assigned_to_label(&workflow);
                                    let workflow_name = workflow.name.clone();
                                    view! {
                                        <tr>
                                            <th scope="row">
                                                <a class="data-table__primary-link" href=workflow_href.clone()>{workflow.name}</a>
                                            </th>
                                            <td>
                                                <a class="data-table__primary-link" href=form_href>{workflow.form_name}</a>
                                            </td>
                                            <td class="data-table__cell--center">{version_label}</td>
                                            <td class="data-table__cell--center">
                                                <span class=status_badge_class(&status_key)>{status_label}</span>
                                            </td>
                                            <td class="data-table__cell--center">{assignments}</td>
                                            <td>
                                                <WorkflowAssignedNodesList
                                                    nodes=assigned_to
                                                    fallback_label=assigned_to_label
                                                    workflow_name=workflow_name
                                                    workflow_href=workflow_href
                                                    sheet=assigned_nodes_sheet
                                                />
                                            </td>
                                        </tr>
                                    }
                                })
                                .collect_view()
                                .into_any()
                        }}
                    </tbody>
                </DataTable>
            </div>
            <div class="forms-list-mobile-cards">
                {if card_workflows.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Workflows to Display"</p> }.into_any()
                } else {
                    card_workflows
                        .into_iter()
                        .map(|workflow| {
                            let workflow_href = format!("/workflows/{}", workflow.id);
                            let form_href = format!("/forms/{}", workflow.form_id);
                            let status_key = workflow_status_key(&workflow).to_string();
                            let status_label = workflow_status_label(&workflow);
                            let version_label = workflow_version_label(&workflow);
                            let assignments = workflow_assignment_count_label(&workflow);
                            let assigned_to = workflow_assignment_links(&workflow, &card_nodes);
                            let assigned_to_label = workflow_assigned_to_label(&workflow);
                            let workflow_name = workflow.name.clone();
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <div>
                                            <h3><a href=workflow_href.clone()>{workflow.name}</a></h3>
                                        </div>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Form"</dt>
                                            <dd><a href=form_href>{workflow.form_name}</a></dd>
                                        </div>
                                        <div>
                                            <dt>"Active version"</dt>
                                            <dd>{version_label}</dd>
                                        </div>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status_key)>{status_label}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Assignments"</dt>
                                            <dd>{assignments}</dd>
                                        </div>
                                        <div>
                                            <dt>"Assigned to"</dt>
                                            <dd>
                                                <WorkflowAssignedNodesList
                                                    nodes=assigned_to
                                                    fallback_label=assigned_to_label
                                                    workflow_name=workflow_name
                                                    workflow_href=workflow_href
                                                    sheet=assigned_nodes_sheet
                                                />
                                            </dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
            <WorkflowAssignedNodesSheet detail=assigned_nodes_sheet/>
        </div>
    }
}

#[component]
fn WorkflowAssignedNodesList(
    nodes: Vec<FormAttachmentLink>,
    fallback_label: String,
    workflow_name: String,
    workflow_href: String,
    sheet: RwSignal<Option<WorkflowAssignedNodesSheetData>>,
) -> impl IntoView {
    let total_nodes = nodes.len();
    let visible_nodes = if total_nodes > 5 {
        nodes[total_nodes - 4..].to_vec()
    } else {
        nodes.clone()
    };
    let nodes_for_sheet = nodes.clone();
    let workflow_name_for_sheet = workflow_name.clone();
    let workflow_href_for_sheet = workflow_href.clone();

    view! {
        <div class="forms-attached-list">
            {if visible_nodes.is_empty() {
                view! { <p>{fallback_label}</p> }.into_any()
            } else {
                visible_nodes
                    .into_iter()
                    .map(|node| {
                        view! {
                            <p>
                                <a href=node.href title=node.title>{node.label}</a>
                            </p>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
            {if total_nodes > 5 {
                view! {
                    <button
                        class="forms-attached-list__more"
                        type="button"
                        on:click=move |_| {
                            sheet.set(Some(WorkflowAssignedNodesSheetData {
                                workflow_name: workflow_name_for_sheet.clone(),
                                workflow_href: workflow_href_for_sheet.clone(),
                                nodes: nodes_for_sheet.clone(),
                            }));
                        }
                    >
                        "More..."
                    </button>
                }
                .into_any()
            } else {
                view! {}.into_any()
            }}
        </div>
    }
}

#[component]
fn WorkflowAssignedNodesSheet(
    detail: RwSignal<Option<WorkflowAssignedNodesSheetData>>,
) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let close = move |_| {
        detail.set(None);
        search.set(String::new());
    };
    let filtered_nodes = move || {
        let query = search.get().trim().to_lowercase();
        detail
            .get()
            .map(|data| {
                data.nodes
                    .into_iter()
                    .filter(|node| {
                        query.is_empty()
                            || node.label.to_lowercase().contains(&query)
                            || node.title.to_lowercase().contains(&query)
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    };

    view! {
        <Portal>
            <Show when=move || detail.get().is_some()>
                <section class="sheet-overlay forms-attached-overlay" aria-label="Assigned organization nodes">
                    <button class="sheet-overlay__scrim" type="button" aria-label="Close assigned nodes" on:click=close></button>
                    <aside class="sheet-panel blurred-surface forms-attached-sheet" role="dialog" aria-modal="true" aria-label="Assigned organization nodes">
                        <div class="sheet-panel__actions">
                            {move || {
                                detail
                                    .get()
                                    .map(|data| {
                                        view! {
                                            <a class="icon-button sheet-panel__open" href=data.workflow_href aria-label="Open workflow detail" title="Open workflow detail">
                                                <ExternalLink class="icon-button__icon"/>
                                            </a>
                                        }
                                        .into_any()
                                    })
                                    .unwrap_or_else(|| view! {}.into_any())
                            }}
                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close assigned nodes" title="Close assigned nodes" on:click=close>
                                <X class="icon-button__icon"/>
                            </button>
                        </div>
                        {move || {
                            detail
                                .get()
                                .map(|data| {
                                    let total = data.nodes.len();
                                    view! {
                                        <header class="sheet-panel__header">
                                            <p>"Assigned Nodes"</p>
                                            <h2>{data.workflow_name}</h2>
                                            <span class="forms-attached-sheet__count">{format!("{total} nodes")}</span>
                                        </header>
                                        <section class="sheet-panel__section">
                                            <label class="searchable-data-table__search searchable-data-table__control forms-attached-sheet__search">
                                                <Search class="searchable-data-table__control-icon"/>
                                                <span class="sr-only">"Search assigned nodes"</span>
                                                <input
                                                    type="search"
                                                    placeholder="Search assigned nodes"
                                                    prop:value=move || search.get()
                                                    on:input=move |event| search.set(event_target_value(&event))
                                                />
                                            </label>
                                            <div class="forms-attached-sheet__list">
                                                {move || {
                                                    let nodes = filtered_nodes();
                                                    if nodes.is_empty() {
                                                        view! { <p class="forms-attached-sheet__empty">"No Assigned Nodes to Display"</p> }.into_any()
                                                    } else {
                                                        nodes
                                                            .into_iter()
                                                            .map(|node| {
                                                                let node_title = node.title.clone();
                                                                view! {
                                                                    <a class="forms-attached-sheet__item" href=node.href title=node_title>
                                                                        <span>{node.label}</span>
                                                                        <small>{node.title}</small>
                                                                    </a>
                                                                }
                                                            })
                                                            .collect_view()
                                                            .into_any()
                                                    }
                                                }}
                                            </div>
                                        </section>
                                    }
                                    .into_any()
                                })
                                .unwrap_or_else(|| view! {}.into_any())
                        }}
                    </aside>
                </section>
            </Show>
        </Portal>
    }
}

#[component]
fn FormBuilderSection(
    section_id: usize,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) -> impl IntoView {
    let section = Memo::new(move |_| {
        builder_sections
            .get()
            .into_iter()
            .find(|section| section.id == section_id)
            .unwrap_or_else(|| blank_form_builder_section(section_id))
    });
    let layout = Memo::new(move |_| {
        let section = section.get();
        let fields = builder_fields.get();
        form_builder_section_layout(&section, &fields)
    });
    let default_column_width = Memo::new(move |_| section.get().default_column_width);

    view! {
        <article class="form-builder-section-card">
            <div class="form-builder-section-card__header">
                <h4>{move || section.get().title}</h4>
            </div>

            <div class="form-grid form-builder-section-card__settings">
                <label class="form-field" for=format!("form-section-title-{section_id}")>
                    <span>"Section Title"</span>
                    <input
                        id=format!("form-section-title-{section_id}")
                        type="text"
                        autocomplete="off"
                        prop:value=move || section.get().title
                        on:input=move |event| {
                            let next_title = event_target_value(&event);
                            builder_sections.update(|sections| {
                                if let Some(section) = sections.iter_mut().find(|section| section.id == section_id) {
                                    section.title = next_title.clone();
                                }
                            });
                        }
                    />
                </label>

                <label class="form-field" for=format!("form-section-default-width-{section_id}")>
                    <span>"Default Column Width"</span>
                    <select
                        id=format!("form-section-default-width-{section_id}")
                        prop:value=move || section.get().default_column_width.to_string()
                        on:change=move |event| {
                            let next_width = event_target_value(&event)
                                .parse::<i32>()
                                .unwrap_or(6)
                                .clamp(1, FORM_BUILDER_COLUMN_COUNT);
                            builder_sections.update(|sections| {
                                if let Some(section) = sections.iter_mut().find(|section| section.id == section_id) {
                                    section.default_column_width = next_width;
                                }
                            });
                        }
                    >
                        {(1..=FORM_BUILDER_COLUMN_COUNT)
                            .map(|width| view! { <option value=width.to_string()>{width}</option> })
                            .collect_view()}
                    </select>
                </label>

                <label class="form-field form-field--wide" for=format!("form-section-description-{section_id}")>
                    <span>"Description"</span>
                    <textarea
                        id=format!("form-section-description-{section_id}")
                        prop:value=move || section.get().description
                        on:input=move |event| {
                            let next_description = event_target_value(&event);
                            builder_sections.update(|sections| {
                                if let Some(section) = sections.iter_mut().find(|section| section.id == section_id) {
                                    section.description = next_description.clone();
                                }
                            });
                        }
                    ></textarea>
                </label>
            </div>

            <FormBuilderGrid
                section_id=section_id
                layout=layout
                default_column_width=default_column_width
                builder_fields=builder_fields
                active_builder_field=active_builder_field
                dragged_builder_field=dragged_builder_field
                builder_drag_preview=builder_drag_preview
                pending_builder_drag_preview=pending_builder_drag_preview
                builder_drag_preview_timeout=builder_drag_preview_timeout
                suppress_builder_field_click=suppress_builder_field_click
                next_builder_field_id=next_builder_field_id
            />
        </article>
    }
}

#[component]
fn FormBuilderGrid(
    section_id: usize,
    layout: Memo<FormBuilderSectionLayout>,
    default_column_width: Memo<i32>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
    next_builder_field_id: RwSignal<usize>,
) -> impl IntoView {
    let grid_rows = Memo::new(move |_| layout.get().row_count);
    let grid_cells = Memo::new(move |_| {
        let row_count = grid_rows.get();
        (1..=row_count)
            .flat_map(|row| {
                (1..=FORM_BUILDER_COLUMN_COUNT)
                    .map(move |column| FormBuilderGridCell { row, column })
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div
            data-section-id=section_id
            class=move || {
                if dragged_builder_field.get().is_some() {
                    "form-builder-layout-grid is-dragging"
                } else {
                    "form-builder-layout-grid"
                }
            }
            style=move || {
                let row_count = grid_rows.get();
                format!(
                    "--form-builder-rows: {}; --form-builder-max-height: {}px;",
                    row_count,
                    row_count * 80,
                )
            }
            on:dragenter=move |event| {
                let Some(field_id) = dragged_builder_field.get_untracked() else {
                    return;
                };
                let Some((row, column, target_id)) = form_builder_grid_cell_from_drag_event(&event) else {
                    return;
                };
                event.prevent_default();
                schedule_form_builder_drag_preview(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    FormBuilderDragPreview {
                        field_id,
                        section_id,
                        row,
                        column,
                    },
                    target_id,
                );
            }
            on:dragover=move |event| {
                let Some(field_id) = dragged_builder_field.get_untracked() else {
                    return;
                };
                event.prevent_default();
                let Some((row, column, target_id)) =
                    form_builder_grid_cell_from_pointer(&event, grid_rows.get_untracked())
                else {
                    return;
                };
                schedule_form_builder_drag_preview(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    FormBuilderDragPreview {
                        field_id,
                        section_id,
                        row,
                        column,
                    },
                    target_id,
                );
            }
            on:drop=move |event| {
                event.prevent_default();
                if let Some(field_id) = dragged_builder_field.get_untracked() {
                    if let Some((row, column, _)) =
                        form_builder_grid_cell_from_pointer(&event, grid_rows.get_untracked())
                    {
                        set_form_builder_drag_preview(
                            builder_drag_preview,
                            FormBuilderDragPreview {
                                field_id,
                                section_id,
                                row,
                                column,
                            },
                        );
                    }
                }
                commit_form_builder_drag_preview(
                    builder_fields,
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                    dragged_builder_field,
                    suppress_builder_field_click,
                );
            }
            on:mouseleave=move |_| {
                if dragged_builder_field.get_untracked().is_some() {
                    clear_form_builder_drag_intent(
                        builder_drag_preview,
                        pending_builder_drag_preview,
                        builder_drag_preview_timeout,
                    );
                }
            }
            on:click=move |event| {
                let Some((row, column)) = form_builder_add_tile_from_click_event(&event) else {
                    return;
                };
                event.prevent_default();
                if suppress_builder_field_click.get_untracked().is_some() {
                    suppress_builder_field_click.set(None);
                    return;
                }
                let fields = builder_fields.get_untracked();
                let occupied_cells = {
                    let section_fields = form_builder_section_fields(section_id, &fields);
                    form_builder_occupancy_map(&section_fields)
                };
                if occupied_cells.contains(&(row, column)) {
                    return;
                }
                let field_id = next_builder_field_id.get_untracked();
                next_builder_field_id.set(field_id + 1);
                let default_width = default_column_width
                    .get_untracked()
                    .clamp(1, FORM_BUILDER_COLUMN_COUNT);
                let available_width =
                    max_form_builder_new_field_width_at(section_id, row, column, &fields);
                let new_field = blank_form_builder_field_at(
                    field_id,
                    section_id,
                    row,
                    column,
                    default_width.min(available_width),
                );
                builder_fields.update(|fields| fields.push(new_field));
                active_builder_field.set(Some(field_id));
            }
        >
            <div class="form-builder-grid-cells">
                <For
                    each=move || grid_cells.get()
                    key=|cell| (cell.row, cell.column)
                    children=move |cell| {
                        let cell_label =
                            format!("Add field at row {}, column {}", cell.row, cell.column);
                        view! {
                            <div
                                id=format!("form-builder-section-{section_id}-cell-r{}-c{}", cell.row, cell.column)
                                class="form-builder-grid-cell form-builder-grid-cell--empty"
                                data-row=cell.row
                                data-column=cell.column
                                data-empty=true
                                aria-label=cell_label
                                style=format!("grid-column: {}; grid-row: {};", cell.column, cell.row)
                            ></div>
                        }
                    }
                />
            </div>
            <For
                each=move || layout.get().fields
                key=|field| field.id
                children=move |field| {
                    view! {
                        <FormBuilderGridTile
                            field_id=field.id
                            section_id=section_id
                            builder_fields=builder_fields
                            active_builder_field=active_builder_field
                            dragged_builder_field=dragged_builder_field
                            builder_drag_preview=builder_drag_preview
                            pending_builder_drag_preview=pending_builder_drag_preview
                            builder_drag_preview_timeout=builder_drag_preview_timeout
                            suppress_builder_field_click=suppress_builder_field_click
                        />
                    }
                }
            />
        </div>
    }
}

#[component]
fn FormBuilderGridTile(
    field_id: usize,
    section_id: usize,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
    active_builder_field: RwSignal<Option<usize>>,
    dragged_builder_field: RwSignal<Option<usize>>,
    builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    pending_builder_drag_preview: RwSignal<Option<FormBuilderDragPreview>>,
    builder_drag_preview_timeout: RwSignal<Option<i32>>,
    suppress_builder_field_click: RwSignal<Option<usize>>,
) -> impl IntoView {
    let field = Memo::new(move |_| {
        builder_fields
            .get()
            .into_iter()
            .find(|field| field.id == field_id)
    });
    let display_label = move || {
        field
            .get()
            .map(|field| {
                if field.label.trim().is_empty() {
                    form_builder_field_default_label(&field.field_type, field_id)
                } else {
                    field.label
                }
            })
            .unwrap_or_else(|| format!("Field {field_id}"))
    };
    view! {
        <div
            class=move || {
                let width_class = field
                    .get()
                    .map(|field| {
                        if field.grid_width <= 2 {
                            " form-builder-grid-tile--icon-only"
                        } else if field.grid_width >= 4 {
                            " form-builder-grid-tile--mobile-label"
                        } else {
                            ""
                        }
                    })
                    .unwrap_or("");
                if dragged_builder_field.get() == Some(field_id) {
                    format!(
                        "form-builder-grid-tile form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary is-dragging{width_class}"
                    )
                } else {
                    format!(
                        "form-builder-grid-tile form-builder-grid-tile--field form-builder-grid-field form-builder-grid-field--summary{width_class}"
                    )
                }
            }
            draggable="true"
            style=move || {
                field
                    .get()
                    .map(|field| form_builder_grid_tile_style(&field))
                    .unwrap_or_else(|| "display: none;".into())
            }
            on:dragstart=move |_event: leptos::ev::DragEvent| {
                #[cfg(feature = "hydrate")]
                {
                    if let Some(target) = _event
                        .target()
                        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
                    {
                        if target.closest(".form-builder-resize-handle").ok().flatten().is_some() {
                            _event.prevent_default();
                            return;
                        }
                    }
                }
                clear_form_builder_drag_intent(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                );
                dragged_builder_field.set(Some(field_id));
            }
            on:dragenter=move |event| {
                if let Some(dragged_field_id) = dragged_builder_field.get_untracked() {
                    event.prevent_default();
                    let Some(field) = field.get_untracked() else {
                        return;
                    };
                    schedule_form_builder_drag_preview(
                        builder_drag_preview,
                        pending_builder_drag_preview,
                        builder_drag_preview_timeout,
                        FormBuilderDragPreview {
                            field_id: dragged_field_id,
                            section_id,
                            row: field.grid_row.max(1),
                            column: field.grid_column.max(1),
                        },
                        format!(
                            "form-builder-section-{section_id}-cell-r{}-c{}",
                            field.grid_row.max(1),
                            field.grid_column.max(1),
                        ),
                    );
                }
            }
            on:click=move |_| {
                if suppress_builder_field_click.get_untracked() == Some(field_id) {
                    suppress_builder_field_click.set(None);
                } else {
                    dragged_builder_field.set(None);
                    active_builder_field.set(Some(field_id));
                }
            }
            on:dragend=move |_| {
                clear_form_builder_drag_intent(
                    builder_drag_preview,
                    pending_builder_drag_preview,
                    builder_drag_preview_timeout,
                );
                dragged_builder_field.set(None);
            }
        >
            <button
                class="form-builder-grid-field__summary"
                type="button"
                title=display_label
                aria-label=move || format!("Configure {}", display_label())
                on:click=move |event| {
                    event.stop_propagation();
                    if suppress_builder_field_click.get_untracked() == Some(field_id) {
                        suppress_builder_field_click.set(None);
                    } else {
                        dragged_builder_field.set(None);
                        active_builder_field.set(Some(field_id));
                    }
                }
            >
                <span class="form-builder-field-type-icon">
                    {move || {
                        field
                            .get()
                            .map(|field| form_builder_field_type_icon(&field.field_type))
                            .unwrap_or_else(|| form_builder_field_type_icon("text"))
                    }}
                </span>
                <div>
                    <h5>{display_label}</h5>
                </div>
            </button>
            <span
                class="form-builder-resize-handle form-builder-resize-handle--width"
                title="Resize field width"
                aria-hidden="true"
                on:mousedown=move |event| {
                    start_form_builder_field_resize(
                        event,
                        FormBuilderResizeAxis::Width,
                        field_id,
                        builder_fields,
                        suppress_builder_field_click,
                    );
                }
            ></span>
            <span
                class="form-builder-resize-handle form-builder-resize-handle--height"
                title="Resize field height"
                aria-hidden="true"
                on:mousedown=move |event| {
                    start_form_builder_field_resize(
                        event,
                        FormBuilderResizeAxis::Height,
                        field_id,
                        builder_fields,
                        suppress_builder_field_click,
                    );
                }
            ></span>
        </div>
    }
}

#[component]
fn FieldConfigSheet(
    active_builder_field: RwSignal<Option<usize>>,
    builder_sections: RwSignal<Vec<FormBuilderSectionDraft>>,
    builder_fields: RwSignal<Vec<FormBuilderFieldDraft>>,
) -> impl IntoView {
    view! {
        <Portal>
            <Show when=move || active_builder_field.get().is_some()>
                {move || {
                    let close = move |_| active_builder_field.set(None);
                    let field_id = active_builder_field.get().unwrap_or_default();
                    let field = builder_fields
                        .get()
                        .into_iter()
                        .find(|field| field.id == field_id);
                    field
                        .map(|field| {
                            let display_label = if field.label.trim().is_empty() {
                                format!("Field {}", field.id)
                            } else {
                                field.label.clone()
                            };
                            let section = builder_sections
                                .get()
                                .into_iter()
                                .find(|section| section.id == field.section_id)
                                .unwrap_or_else(|| blank_form_builder_section(field.section_id));
                            let all_fields = builder_fields.get();
                            let layout = form_builder_section_layout(&section, &all_fields);
                            let section_column_count = layout.column_count;
                            let section_fields_for_bounds = layout.fields;
                            let row_max = layout.row_count;
                            let width_max = max_form_builder_field_width(
                                &field,
                                &section_fields_for_bounds,
                            );
                            let height_max = max_form_builder_field_height(
                                &field,
                                &section_fields_for_bounds,
                            );
                            view! {
                                <section class="sheet-overlay form-field-config-overlay" aria-label="Field configuration">
                                    <button class="sheet-overlay__scrim" type="button" aria-label="Close field configuration" on:click=close></button>
                                    <aside class="sheet-panel blurred-surface form-field-config-sheet" role="dialog" aria-modal="true" aria-label="Field configuration">
                                        <div class="sheet-panel__actions">
                                            <button
                                                class="icon-button icon-button--danger"
                                                type="button"
                                                aria-label="Delete field"
                                                title="Delete field"
                                                on:click=move |_| {
                                                    builder_fields.update(|fields| {
                                                        fields.retain(|field| field.id != field_id);
                                                    });
                                                    active_builder_field.set(None);
                                                }
                                            >
                                                <Trash2/>
                                            </button>
                                            <button class="icon-button sheet-panel__close" type="button" aria-label="Close field configuration" title="Close field configuration" on:click=close>
                                                <X/>
                                            </button>
                                        </div>

                                        <header class="sheet-panel__header">
                                            <p>"Field Configuration"</p>
                                            <h2>{display_label}</h2>
                                        </header>

                                        <section class="sheet-panel__section">
                                            <div class="form-grid form-builder-field-sheet-controls">
                                                <label class="form-field" for=format!("sheet-form-field-label-{field_id}")>
                                                    <span>"Field Label"</span>
                                                    <input
                                                        id=format!("sheet-form-field-label-{field_id}")
                                                        type="text"
                                                        autocomplete="off"
                                                        prop:value=field.label.clone()
                                                        on:input=move |event| {
                                                            let next_label = event_target_value(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    field.label = next_label.clone();
                                                                    if !field.key_was_edited {
                                                                        field.key = slug_from_label(&next_label);
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    />
                                                </label>

                                                <label class="form-field" for=format!("sheet-form-field-key-{field_id}")>
                                                    <span>"Field Key"</span>
                                                    <input
                                                        id=format!("sheet-form-field-key-{field_id}")
                                                        type="text"
                                                        autocomplete="off"
                                                        prop:value=field.key.clone()
                                                        on:input=move |event| {
                                                            let next_key = slug_from_label(&event_target_value(&event));
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    field.key = next_key.clone();
                                                                    field.key_was_edited = true;
                                                                }
                                                            });
                                                        }
                                                    />
                                                </label>

                                                <label class="form-field" for=format!("sheet-form-field-type-{field_id}")>
                                                    <span>"Field Type"</span>
                                                    <select
                                                        id=format!("sheet-form-field-type-{field_id}")
                                                        prop:value=field.field_type.clone()
                                                        on:change=move |event| {
                                                            let next_type = event_target_value(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(position) = fields.iter().position(|field| field.id == field_id) {
                                                                    let mut next_field = fields[position].clone();
                                                                    next_field.field_type = next_type.clone();
                                                                    if next_type == "static_text" {
                                                                        next_field.required = false;
                                                                        if next_field.label.trim().is_empty() {
                                                                            next_field.label = form_builder_field_default_label(&next_type, next_field.id);
                                                                        }
                                                                        if next_field.key.trim().is_empty() || !next_field.key_was_edited {
                                                                            next_field.key = slug_from_label(&next_field.label);
                                                                        }
                                                                        let mut candidate = next_field.clone();
                                                                        candidate.grid_width = candidate.grid_width.max(4);
                                                                        if candidate.grid_column + candidate.grid_width - 1 <= FORM_BUILDER_COLUMN_COUNT
                                                                            && !form_builder_field_has_collision(&candidate, fields)
                                                                        {
                                                                            next_field.grid_width = candidate.grid_width;
                                                                        }
                                                                    }
                                                                    fields[position] = next_field;
                                                                }
                                                            });
                                                        }
                                                    >
                                                        <option value="static_text">"Static text"</option>
                                                        <option value="text">"Text"</option>
                                                        <option value="number">"Number"</option>
                                                        <option value="date">"Date"</option>
                                                        <option value="boolean">"Checkbox"</option>
                                                        <option value="single_choice">"Single choice"</option>
                                                        <option value="multi_choice">"Multi choice"</option>
                                                    </select>
                                                </label>

                                                <label class="form-field form-field--checkbox form-builder-field__required">
                                                    <input
                                                        type="checkbox"
                                                        prop:checked=field.required
                                                        disabled=field.field_type == "static_text"
                                                        on:change=move |event| {
                                                            let checked = event_target_checked(&event);
                                                            builder_fields.update(|fields| {
                                                                if let Some(field) = fields.iter_mut().find(|field| field.id == field_id) {
                                                                    if field.field_type != "static_text" {
                                                                        field.required = checked;
                                                                    }
                                                                }
                                                            });
                                                        }
                                                    />
                                                    <span>"Required"</span>
                                                </label>

                                                {["Row", "Column", "Width", "Height"]
                                                    .into_iter()
                                                    .enumerate()
                                                    .map(|(index, label)| {
                                                        let value = match index {
                                                            0 => field.grid_row,
                                                            1 => field.grid_column,
                                                            2 => field.grid_width,
                                                            _ => field.grid_height,
                                                        };
                                                        let max_value = match index {
                                                            0 => row_max,
                                                            1 => (section_column_count - field.grid_width.max(1) + 1)
                                                                .clamp(1, section_column_count.max(1)),
                                                            2 => width_max,
                                                            _ => height_max,
                                                        }
                                                        .max(1);
                                                        let value = value.clamp(1, max_value);
                                                        let valid_values = valid_form_builder_layout_values(
                                                            &field,
                                                            &section_fields_for_bounds,
                                                            index,
                                                            max_value,
                                                        );
                                                        let control_id = format!("sheet-form-field-layout-{index}-{field_id}");
                                                        let input_id = control_id.clone();
                                                        view! {
                                                            <label class="form-field" for=control_id>
                                                                <span>{label}</span>
                                                                <select
                                                                    id=input_id
                                                                    on:change=move |event| {
                                                                        let value = event_target_value(&event)
                                                                            .parse::<i32>()
                                                                            .unwrap_or(1)
                                                                            .clamp(1, max_value);
                                                                        builder_fields.update(|fields| {
                                                                            if let Some(position) = fields.iter().position(|field| field.id == field_id) {
                                                                                let candidate = form_builder_layout_candidate(
                                                                                    &fields[position],
                                                                                    index,
                                                                                    value,
                                                                                );

                                                                                if !form_builder_field_has_collision(&candidate, fields) {
                                                                                    fields[position] = candidate;
                                                                                }
                                                                            }
                                                                        });
                                                                    }
                                                                >
                                                                    {valid_values
                                                                        .into_iter()
                                                                        .map(|option_value| {
                                                                            view! {
                                                                                <option
                                                                                    value=option_value.to_string()
                                                                                    selected=option_value == value
                                                                                >
                                                                                    {option_value}
                                                                                </option>
                                                                            }
                                                                        })
                                                                        .collect_view()}
                                                                </select>
                                                            </label>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </div>
                                        </section>
                                    </aside>
                                </section>
                            }
                            .into_any()
                        })
                        .unwrap_or_else(|| view! {}.into_any())
                }}
            </Show>
        </Portal>
    }
}

#[component]
pub fn FormsNewPage() -> impl IntoView {
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let existing_forms = RwSignal::new(Vec::<FormSummary>::new());
    let name = RwSignal::new(String::new());
    let scope_node_type_id = RwSignal::new(String::new());
    let builder_sections = RwSignal::new(vec![blank_form_builder_section(1)]);
    let active_builder_section = RwSignal::new("1".to_string());
    let next_builder_section_id = RwSignal::new(2usize);
    let builder_fields = RwSignal::new(Vec::<FormBuilderFieldDraft>::new());
    let active_builder_field = RwSignal::new(None::<usize>);
    let dragged_builder_field = RwSignal::new(None::<usize>);
    let builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let pending_builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let builder_drag_preview_timeout = RwSignal::new(None::<i32>);
    let suppress_builder_field_click = RwSignal::new(None::<usize>);
    let next_builder_field_id = RwSignal::new(1usize);
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let builder_field_count = Memo::new(move |_| builder_fields.get().len());

    Effect::new(move |_| {
        load_form_create_options(node_types, existing_forms, is_loading, message);
    });

    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                <BreadcrumbItem>
                    <BreadcrumbPage>"Create Form"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>
            <section class="route-panel forms-page form-editor-panel">
                <PageHeader title="Create Form"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form options"</h3>
                                <p>"Fetching available organization scopes."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <div class="form-create-workspace">
                            <form
                                class="native-form form-create-form"
                                on:submit=move |event| {
                                    event.prevent_default();
                                    submit_create_form(
                                        name,
                                        scope_node_type_id,
                                        builder_sections,
                                        builder_fields,
                                        existing_forms,
                                        is_saving,
                                        message,
                                    );
                                }
                            >
                                <div class="form-grid">
                                    <label class="form-field form-field--wide" for="form-name">
                                        <span>"Form Name"</span>
                                        <input
                                            id="form-name"
                                            type="text"
                                            autocomplete="off"
                                            prop:value=move || name.get()
                                            on:input=move |event| name.set(event_target_value(&event))
                                            required
                                        />
                                    </label>

                                    <label class="form-field" for="form-scope-node-type">
                                        <span>"Scope"</span>
                                        <select
                                            id="form-scope-node-type"
                                            prop:value=move || scope_node_type_id.get()
                                            on:change=move |event| scope_node_type_id.set(event_target_value(&event))
                                        >
                                            <option value="">"No scope"</option>
                                            {move || {
                                                let mut options = node_types.get();
                                                options.sort_by(|left, right| {
                                                    left.singular_label
                                                        .cmp(&right.singular_label)
                                                        .then(left.name.cmp(&right.name))
                                                });
                                                options
                                                    .into_iter()
                                                    .map(|node_type| {
                                                        view! {
                                                            <option value=node_type.id>{node_type.singular_label}</option>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                        </select>
                                    </label>

                                </div>

                                <section class="form-section">
                                    <h3>"Initial Version"</h3>
                                    <InfoListTable>
                                        <InfoRow label="Status" value="Draft"/>
                                        <tr>
                                            <th scope="row">"Fields"</th>
                                            <td>
                                                {move || builder_field_count.get().to_string()}
                                            </td>
                                        </tr>
                                    </InfoListTable>
                                </section>

                                <section class="form-builder form-section">
                                    <div class="form-builder__header">
                                        <h3>"Form Builder"</h3>
                                    </div>

                                    <Tabs active=active_builder_section>
                                        <TabsList>
                                            {move || {
                                                builder_sections
                                                    .get()
                                                    .into_iter()
                                                    .map(|section| {
                                                        let section_value = section.id.to_string();
                                                        let section_tab_value = section_value.clone();
                                                        view! {
                                                            <button
                                                                class=move || {
                                                                    if active_builder_section.get() == section_tab_value {
                                                                        "tabs-trigger is-active"
                                                                    } else {
                                                                        "tabs-trigger"
                                                                    }
                                                                }
                                                                type="button"
                                                                role="tab"
                                                                aria-selected=move || (active_builder_section.get() == section_value).to_string()
                                                                on:click=move |_| active_builder_section.set(section.id.to_string())
                                                            >
                                                                {section.title}
                                                            </button>
                                                        }
                                                    })
                                                    .collect_view()
                                            }}
                                            <button
                                                class="tabs-trigger form-builder__add-section-tab"
                                                type="button"
                                                on:click=move |_| {
                                                    let section_id = next_builder_section_id.get_untracked();
                                                    next_builder_section_id.set(section_id + 1);
                                                    builder_sections.update(|sections| {
                                                        let mut section = blank_form_builder_section(section_id);
                                                        section.position = (sections.len() + 1) as i32;
                                                        sections.push(section);
                                                    });
                                                    active_builder_section.set(section_id.to_string());
                                                }
                                            >
                                                <Plus/>
                                                "Section"
                                            </button>
                                        </TabsList>
                                    </Tabs>

                                    <div class="form-builder__sections">
                                        <For
                                            each=move || {
                                                builder_sections
                                                    .get()
                                                    .into_iter()
                                                    .filter(|section| {
                                                        active_builder_section.get() == section.id.to_string()
                                                    })
                                                    .map(|section| section.id)
                                                    .collect::<Vec<_>>()
                                            }
                                            key=|section_id| *section_id
                                            children=move |section_id| {
                                                view! {
                                                    <FormBuilderSection
                                                        section_id=section_id
                                                        builder_sections=builder_sections
                                                        builder_fields=builder_fields
                                                        active_builder_field=active_builder_field
                                                        dragged_builder_field=dragged_builder_field
                                                        builder_drag_preview=builder_drag_preview
                                                        pending_builder_drag_preview=pending_builder_drag_preview
                                                        builder_drag_preview_timeout=builder_drag_preview_timeout
                                                        suppress_builder_field_click=suppress_builder_field_click
                                                        next_builder_field_id=next_builder_field_id
                                                    />
                                                }
                                            }
                                        />
                                    </div>
                                </section>
                                {move || message.get().map(|message| view! {
                                    <p class="form-message" role="status">{message}</p>
                                })}

                                <div class="form-actions">
                                    <Button label="Cancel" href="/forms"/>
                                    <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                        {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                                    </button>
                                    <button class="button" type="submit" disabled=move || !can_submit()>
                                        {move || if is_saving.get() { "Creating..." } else { "Create Form" }}
                                    </button>
                                </div>
                            </form>
                            <FieldConfigSheet
                                active_builder_field=active_builder_field
                                builder_sections=builder_sections
                                builder_fields=builder_fields
                            />
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn FormsDetailPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;
    let detail = RwSignal::new(None::<FormDefinition>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let is_loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_form_detail(form_id.clone(), detail, rendered_form, is_loading, error);
    });

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail.get().map(|form| {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>{form.name}</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                    })
                }}
                {move || {
                    if detail.get().is_none() {
                        view! {
                            <BreadcrumbItem>
                                <BreadcrumbPage>"Detail"</BreadcrumbPage>
                            </BreadcrumbItem>
                        }
                        .into_any()
                    } else {
                        view! {}.into_any()
                    }
                }}
            </Breadcrumb>

            <section class="route-panel forms-page form-detail-page">
                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form"</h3>
                                <p>"Fetching form details."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(message) = error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Form detail unavailable"</h3>
                                <p>{message}</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(form) = detail.get() {
                        let edit_href = format!("/forms/{}/edit", form.id);
                        view! {
                            <PageHeader title="Form Detail">
                                <a class="button" href=edit_href>"Edit Form"</a>
                            </PageHeader>
                            <FormDetailContent form rendered_form=rendered_form.get()/>
                        }
                        .into_any()
                    } else {
                        view! {
                            <EmptyState
                                title="Form detail unavailable"
                                message="The selected form could not be loaded."
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
fn FormDetailContent(form: FormDefinition, rendered_form: Option<RenderedForm>) -> impl IntoView {
    let active_version = active_form_definition_version(&form).cloned();
    let attached_nodes = form_attached_nodes(active_version.as_ref());
    let active_status = active_version
        .as_ref()
        .map(|version| version.status.clone())
        .unwrap_or_else(|| "none".to_string());
    let active_version_label = form_version_label(active_version.as_ref());
    let active_status_label = form_status_label(active_version.as_ref());
    let active_field_count = form_field_count_label(active_version.as_ref());
    let published_at = active_version
        .as_ref()
        .and_then(|version| version.published_at.clone());
    let form_name = form.name.clone();
    let form_slug = form.slug.clone();
    let form_scope = form_definition_scope_label(&form);
    let version_count = form.versions.len().to_string();
    let versions = form.versions.clone();
    let workflows = form.workflows.clone();
    let reports = form.reports.clone();
    let dataset_sources = form.dataset_sources.clone();

    view! {
        <div class="organization-detail-content form-detail-content">
            <header class="organization-detail-content__header">
                <p>"Form Detail"</p>
                <h2>{form_name}</h2>
            </header>

            <div class="organization-detail-content__grid">
                <section class="organization-detail-card">
                    <h3>"Details"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Slug"</th>
                            <td>{form_slug}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Scope"</th>
                            <td>{form_scope}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Versions"</th>
                            <td>{version_count}</td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card">
                    <h3>"Active Version"</h3>
                    <InfoListTable>
                        <tr>
                            <th scope="row">"Version"</th>
                            <td>{active_version_label}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Status"</th>
                            <td><span class=status_badge_class(&active_status)>{active_status_label}</span></td>
                        </tr>
                        <tr>
                            <th scope="row">"Fields"</th>
                            <td>{active_field_count}</td>
                        </tr>
                        <tr>
                            <th scope="row">"Published"</th>
                            <td>
                                {published_at
                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                            </td>
                        </tr>
                    </InfoListTable>
                </section>

                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Fields"</h3>
                    <RenderedFormSections rendered_form/>
                </section>

                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Versions"</h3>
                    <FormVersionsTable versions=versions/>
                </section>

                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Attached To"</h3>
                    <FormAttachedNodesDetail nodes=attached_nodes/>
                </section>

                <section class="organization-detail-card organization-detail-card--wide">
                    <h3>"Related Work"</h3>
                    <FormRelatedLinks
                        workflows=workflows
                        reports=reports
                        dataset_sources=dataset_sources
                    />
                </section>
            </div>
        </div>
    }
}

#[component]
fn FormAttachedNodesDetail(nodes: Vec<FormAttachmentLink>) -> impl IntoView {
    view! {
        <div class="form-detail-attached-list">
            {if nodes.is_empty() {
                view! { <p class="related-work-mobile-empty">"No Attached Nodes to Display"</p> }.into_any()
            } else {
                nodes
                    .into_iter()
                    .map(|node| {
                        let title = node.title.clone();
                        view! {
                            <a class="forms-attached-sheet__item" href=node.href title=title>
                                <span>{node.label}</span>
                                <small>{node.title}</small>
                            </a>
                        }
                    })
                    .collect_view()
                    .into_any()
            }}
        </div>
    }
}

#[component]
fn FormVersionsTable(versions: Vec<FormVersionSummary>) -> impl IntoView {
    let table_versions = versions.clone();
    let card_versions = versions;

    view! {
        <div class="forms-list-responsive-table">
            <DataTable>
                <thead>
                    <tr>
                        <th scope="col">"Version"</th>
                        <th scope="col">"Status"</th>
                        <th scope="col">"Compatibility"</th>
                        <th scope="col">"Published"</th>
                        <th class="data-table__cell--center" scope="col">"Fields"</th>
                    </tr>
                </thead>
                <tbody>
                    {if table_versions.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="5">"No Versions to Display"</td>
                            </tr>
                        }
                        .into_any()
                    } else {
                        table_versions
                            .into_iter()
                            .map(|version| {
                                let status = version.status.clone();
                                let published_at = version.published_at.clone();
                                view! {
                                    <tr>
                                        <th scope="row">{form_version_sort_label(&version)}</th>
                                        <td><span class=status_badge_class(&status)>{sentence_label(&status)}</span></td>
                                        <td>{nonempty_text(version.compatibility_group_name.as_deref(), "-")}</td>
                                        <td>
                                            {published_at
                                                .map(|value| view! { <Timestamp value/> }.into_any())
                                                .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                        </td>
                                        <td class="data-table__cell--center">{version.field_count.to_string()}</td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }}
                </tbody>
            </DataTable>
            <div class="forms-list-mobile-cards">
                {if card_versions.is_empty() {
                    view! { <p class="forms-list-mobile-empty">"No Versions to Display"</p> }.into_any()
                } else {
                    card_versions
                        .into_iter()
                        .map(|version| {
                            let status = version.status.clone();
                            let published_at = version.published_at.clone();
                            view! {
                                <article class="forms-list-mobile-card">
                                    <div class="forms-list-mobile-card__header">
                                        <h3>{form_version_sort_label(&version)}</h3>
                                    </div>
                                    <dl>
                                        <div>
                                            <dt>"Status"</dt>
                                            <dd><span class=status_badge_class(&status)>{sentence_label(&status)}</span></dd>
                                        </div>
                                        <div>
                                            <dt>"Compatibility"</dt>
                                            <dd>{nonempty_text(version.compatibility_group_name.as_deref(), "-")}</dd>
                                        </div>
                                        <div>
                                            <dt>"Published"</dt>
                                            <dd>
                                                {published_at
                                                    .map(|value| view! { <Timestamp value/> }.into_any())
                                                    .unwrap_or_else(|| view! { <span>"-"</span> }.into_any())}
                                            </dd>
                                        </div>
                                        <div>
                                            <dt>"Fields"</dt>
                                            <dd>{version.field_count.to_string()}</dd>
                                        </div>
                                    </dl>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </div>
        </div>
    }
}

#[component]
fn RenderedFormSections(rendered_form: Option<RenderedForm>) -> impl IntoView {
    view! {
        <div class="form-detail-sections">
            {if let Some(rendered_form) = rendered_form {
                if rendered_form.sections.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No Fields to Display"</p> }.into_any()
                } else {
                    rendered_form
                        .sections
                        .into_iter()
                        .map(|section| {
                            view! {
                                <article class="form-detail-section">
                                    <header>
                                        <div>
                                            <h4>{section.title}</h4>
                                            {if section.description.trim().is_empty() {
                                                view! {}.into_any()
                                            } else {
                                                view! { <p>{section.description}</p> }.into_any()
                                            }}
                                        </div>
                                    </header>
                                    <DataTable>
                                        <thead>
                                            <tr>
                                                <th scope="col">"Field"</th>
                                                <th scope="col">"Key"</th>
                                                <th scope="col">"Type"</th>
                                                <th scope="col">"Required"</th>
                                                <th scope="col">"Layout"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {if section.fields.is_empty() {
                                                view! {
                                                    <tr>
                                                        <td class="data-table__empty" colspan="5">"No Fields to Display"</td>
                                                    </tr>
                                                }
                                                .into_any()
                                            } else {
                                                section
                                                    .fields
                                                    .into_iter()
                                                    .map(|field| {
                                                        view! {
                                                            <tr>
                                                                <th scope="row">{field.label}</th>
                                                                <td>{field.key}</td>
                                                                <td>{rendered_field_type_label(&field.field_type)}</td>
                                                                <td>{if field.required { "Yes" } else { "No" }}</td>
                                                                <td>{format!(
                                                                    "R{} C{} · W{} H{}",
                                                                    field.grid_row,
                                                                    field.grid_column,
                                                                    field.grid_width,
                                                                    field.grid_height,
                                                                )}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()
                                                    .into_any()
                                            }}
                                        </tbody>
                                    </DataTable>
                                </article>
                            }
                        })
                        .collect_view()
                        .into_any()
                }
            } else {
                view! { <p class="related-work-mobile-empty">"No Fields to Display"</p> }.into_any()
            }}
        </div>
    }
}

#[component]
fn FormRelatedLinks(
    workflows: Vec<FormWorkflowLink>,
    reports: Vec<FormReportLink>,
    dataset_sources: Vec<FormDatasetSourceLink>,
) -> impl IntoView {
    view! {
        <div class="related-work-summary related-work-summary--cards-only form-detail-related">
            <section class="related-work-summary__group">
                <h4>{format!("Workflows ({})", workflows.len())}</h4>
                {if workflows.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No Related Workflows to Display"</p> }.into_any()
                } else {
                    workflows
                        .into_iter()
                        .map(|workflow| {
                            let href = format!("/workflows/{}", workflow.id);
                            let status = workflow.current_status.clone().unwrap_or_else(|| "none".to_string());
                            view! {
                                <a class="related-work-card" href=href>
                                    <span>
                                        <strong>{workflow.name}</strong>
                                        <small>{workflow.slug}</small>
                                    </span>
                                    <span class="related-work-card__meta">{form_version_label_from_option(workflow.current_version_label)}</span>
                                    <span class=status_badge_class(&status)>{sentence_label(&status)}</span>
                                </a>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </section>
            <section class="related-work-summary__group">
                <h4>{format!("Reports ({})", reports.len())}</h4>
                {if reports.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No Related Reports to Display"</p> }.into_any()
                } else {
                    reports
                        .into_iter()
                        .map(|report| {
                            view! {
                                <a class="related-work-card" href=format!("/reports/{}", report.id)>
                                    <span>
                                        <strong>{report.name}</strong>
                                    </span>
                                </a>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </section>
            <section class="related-work-summary__group">
                <h4>{format!("Dataset Sources ({})", dataset_sources.len())}</h4>
                {if dataset_sources.is_empty() {
                    view! { <p class="related-work-mobile-empty">"No Related Dataset Sources to Display"</p> }.into_any()
                } else {
                    dataset_sources
                        .into_iter()
                        .map(|source| {
                            view! {
                                <a class="related-work-card" href=format!("/datasets/{}", source.dataset_id)>
                                    <span>
                                        <strong>{source.dataset_name}</strong>
                                        <small>{source.source_alias}</small>
                                    </span>
                                    <span class="related-work-card__meta">{sentence_label(&source.selection_rule)}</span>
                                </a>
                            }
                        })
                        .collect_view()
                        .into_any()
                }}
            </section>
        </div>
    }
}

fn form_version_label_from_option(label: Option<String>) -> String {
    label.unwrap_or_else(|| "-".to_string())
}

#[component]
pub fn FormsEditPage() -> impl IntoView {
    let params = require_route_params::<FormRouteParams>();
    let form_id = params.form_id;
    let form_id_for_load = form_id.clone();
    let form_id_for_submit = form_id.clone();
    let cancel_href = format!("/forms/{form_id}");
    let node_types = RwSignal::new(Vec::<NodeTypeCatalogEntry>::new());
    let existing_forms = RwSignal::new(Vec::<FormSummary>::new());
    let detail = RwSignal::new(None::<FormDefinition>);
    let rendered_form = RwSignal::new(None::<RenderedForm>);
    let edit_version_id = RwSignal::new(None::<String>);
    let edit_version_status = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let scope_node_type_id = RwSignal::new(String::new());
    let builder_sections = RwSignal::new(vec![blank_form_builder_section(1)]);
    let active_builder_section = RwSignal::new("1".to_string());
    let next_builder_section_id = RwSignal::new(2usize);
    let builder_fields = RwSignal::new(Vec::<FormBuilderFieldDraft>::new());
    let active_builder_field = RwSignal::new(None::<usize>);
    let dragged_builder_field = RwSignal::new(None::<usize>);
    let builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let pending_builder_drag_preview = RwSignal::new(None::<FormBuilderDragPreview>);
    let builder_drag_preview_timeout = RwSignal::new(None::<i32>);
    let suppress_builder_field_click = RwSignal::new(None::<usize>);
    let next_builder_field_id = RwSignal::new(1usize);
    let is_loading = RwSignal::new(true);
    let is_saving = RwSignal::new(false);
    let message = RwSignal::new(None::<String>);
    let builder_field_count = Memo::new(move |_| builder_fields.get().len());

    Effect::new(move |_| {
        load_form_edit_options(
            form_id_for_load.clone(),
            node_types,
            existing_forms,
            detail,
            rendered_form,
            edit_version_id,
            edit_version_status,
            name,
            scope_node_type_id,
            builder_sections,
            builder_fields,
            active_builder_section,
            next_builder_section_id,
            next_builder_field_id,
            is_loading,
            message,
        );
    });

    let can_submit = move || !is_loading.get() && !is_saving.get() && !name.get().trim().is_empty();

    view! {
        <AppShell active_route="forms" title="Forms">
            <Breadcrumb>
                <BreadcrumbItem>
                    <BreadcrumbLink href="/forms">"Forms"</BreadcrumbLink>
                </BreadcrumbItem>
                <BreadcrumbSeparator/>
                {move || {
                    detail
                        .get()
                        .map(|form| {
                            let href = format!("/forms/{}", form.id);
                            view! {
                                <BreadcrumbItem>
                                    <BreadcrumbLink href=href>{form.name}</BreadcrumbLink>
                                </BreadcrumbItem>
                                <BreadcrumbSeparator/>
                            }
                            .into_any()
                        })
                        .unwrap_or_else(|| view! {}.into_any())
                }}
                <BreadcrumbItem>
                    <BreadcrumbPage>"Edit Form"</BreadcrumbPage>
                </BreadcrumbItem>
            </Breadcrumb>

            <section class="route-panel forms-page form-editor-panel">
                <PageHeader title="Edit Form"/>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading form"</h3>
                                <p>"Fetching form definition and editable version."</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        let form_id_for_submit = form_id_for_submit.clone();
                        view! {
                            <div class="form-create-workspace">
                                <form
                                    class="native-form form-create-form"
                                    on:submit=move |event| {
                                        event.prevent_default();
                                        submit_update_form(
                                            form_id_for_submit.clone(),
                                            name,
                                            scope_node_type_id,
                                            builder_sections,
                                            builder_fields,
                                            existing_forms,
                                            edit_version_id,
                                            edit_version_status,
                                            rendered_form,
                                            is_saving,
                                            message,
                                        );
                                    }
                                >
                                    <div class="form-grid">
                                        <label class="form-field form-field--wide" for="form-name">
                                            <span>"Form Name"</span>
                                            <input
                                                id="form-name"
                                                type="text"
                                                autocomplete="off"
                                                prop:value=move || name.get()
                                                on:input=move |event| name.set(event_target_value(&event))
                                                required
                                            />
                                        </label>

                                        <label class="form-field" for="form-scope-node-type">
                                            <span>"Scope"</span>
                                            <select
                                                id="form-scope-node-type"
                                                prop:value=move || scope_node_type_id.get()
                                                on:change=move |event| scope_node_type_id.set(event_target_value(&event))
                                            >
                                                <option value="">"No scope"</option>
                                                {move || {
                                                    let mut options = node_types.get();
                                                    options.sort_by(|left, right| {
                                                        left.singular_label
                                                            .cmp(&right.singular_label)
                                                            .then(left.name.cmp(&right.name))
                                                    });
                                                    options
                                                        .into_iter()
                                                        .map(|node_type| {
                                                            view! {
                                                                <option value=node_type.id>{node_type.singular_label}</option>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </select>
                                        </label>
                                    </div>

                                    <section class="form-section">
                                        <h3>"Editable Version"</h3>
                                        <InfoListTable>
                                            <tr>
                                                <th scope="row">"Status"</th>
                                                <td>
                                                    {move || {
                                                        edit_version_status
                                                            .get()
                                                            .map(|status| {
                                                                view! {
                                                                    <span class=status_badge_class(&status)>
                                                                        {sentence_label(&status)}
                                                                    </span>
                                                                }
                                                                .into_any()
                                                            })
                                                            .unwrap_or_else(|| view! { <span>"Draft"</span> }.into_any())
                                                    }}
                                                </td>
                                            </tr>
                                            <tr>
                                                <th scope="row">"Fields"</th>
                                                <td>
                                                    {move || builder_field_count.get().to_string()}
                                                </td>
                                            </tr>
                                        </InfoListTable>
                                    </section>

                                    <section class="form-builder form-section">
                                        <div class="form-builder__header">
                                            <h3>"Form Builder"</h3>
                                        </div>

                                        <Tabs active=active_builder_section>
                                            <TabsList>
                                                {move || {
                                                    builder_sections
                                                        .get()
                                                        .into_iter()
                                                        .map(|section| {
                                                            let section_value = section.id.to_string();
                                                            let section_tab_value = section_value.clone();
                                                            view! {
                                                                <button
                                                                    class=move || {
                                                                        if active_builder_section.get() == section_tab_value {
                                                                            "tabs-trigger is-active"
                                                                        } else {
                                                                            "tabs-trigger"
                                                                        }
                                                                    }
                                                                    type="button"
                                                                    role="tab"
                                                                    aria-selected=move || (active_builder_section.get() == section_value).to_string()
                                                                    on:click=move |_| active_builder_section.set(section.id.to_string())
                                                                >
                                                                    {section.title}
                                                                </button>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                                <button
                                                    class="tabs-trigger form-builder__add-section-tab"
                                                    type="button"
                                                    on:click=move |_| {
                                                        let section_id = next_builder_section_id.get_untracked();
                                                        next_builder_section_id.set(section_id + 1);
                                                        builder_sections.update(|sections| {
                                                            let mut section = blank_form_builder_section(section_id);
                                                            section.position = (sections.len() + 1) as i32;
                                                            sections.push(section);
                                                        });
                                                        active_builder_section.set(section_id.to_string());
                                                    }
                                                >
                                                    <Plus/>
                                                    "Section"
                                                </button>
                                            </TabsList>
                                        </Tabs>

                                        <div class="form-builder__sections">
                                            <For
                                                each=move || {
                                                    builder_sections
                                                        .get()
                                                        .into_iter()
                                                        .filter(|section| {
                                                            active_builder_section.get() == section.id.to_string()
                                                        })
                                                        .map(|section| section.id)
                                                        .collect::<Vec<_>>()
                                                }
                                                key=|section_id| *section_id
                                                children=move |section_id| {
                                                    view! {
                                                        <FormBuilderSection
                                                            section_id=section_id
                                                            builder_sections=builder_sections
                                                            builder_fields=builder_fields
                                                            active_builder_field=active_builder_field
                                                            dragged_builder_field=dragged_builder_field
                                                            builder_drag_preview=builder_drag_preview
                                                            pending_builder_drag_preview=pending_builder_drag_preview
                                                            builder_drag_preview_timeout=builder_drag_preview_timeout
                                                            suppress_builder_field_click=suppress_builder_field_click
                                                            next_builder_field_id=next_builder_field_id
                                                        />
                                                    }
                                                }
                                            />
                                        </div>
                                    </section>
                                    {move || message.get().map(|message| view! {
                                        <p class="form-message" role="status">{message}</p>
                                    })}

                                    <div class="form-actions">
                                        <a class="button" href=cancel_href.clone()>"Cancel"</a>
                                        <button class="button button--secondary" type="submit" disabled=move || !can_submit()>
                                            {move || if is_saving.get() { "Saving..." } else { "Save as Draft" }}
                                        </button>
                                    </div>
                                </form>
                                <FieldConfigSheet
                                    active_builder_field=active_builder_field
                                    builder_sections=builder_sections
                                    builder_fields=builder_fields
                                />
                            </div>
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn WorkflowsPage() -> impl IntoView {
    let workflows = RwSignal::new(Vec::<WorkflowSummary>::new());
    let organization_nodes = RwSignal::new(Vec::<OrganizationNode>::new());
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let is_loading = RwSignal::new(true);
    let load_error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        load_workflows(workflows, is_loading, load_error);
        load_workflow_assignment_nodes(organization_nodes);
    });

    let filtered_workflows = move || {
        let query = search.get();
        let selected_status = status_filter.get();
        workflows
            .get()
            .into_iter()
            .filter(|workflow| {
                let version_label = workflow_version_label(workflow);
                let status_label = workflow_status_label(workflow);
                let assigned_to = workflow_assigned_to_label(workflow);
                let description = workflow_description_label(workflow);
                text_matches(
                    &query,
                    &[
                        workflow.name.as_str(),
                        workflow.slug.as_str(),
                        description.as_str(),
                        workflow.form_name.as_str(),
                        workflow.form_slug.as_str(),
                        version_label.as_str(),
                        status_label.as_str(),
                        assigned_to.as_str(),
                    ],
                ) && (selected_status == "all" || selected_status == status_label)
            })
            .collect::<Vec<_>>()
    };

    let status_options = move || {
        unique_filter_options(
            workflows
                .get()
                .iter()
                .map(workflow_status_label)
                .collect::<Vec<_>>(),
        )
    };

    view! {
        <AppShell active_route="workflows" title="Workflows">
            <section class="route-panel workflows-page">
                <PageHeader title="Workflows">
                    <Button label="Create Workflow" href="/workflows/new"/>
                </PageHeader>

                {move || {
                    if is_loading.get() {
                        view! {
                            <section class="organization-state" aria-live="polite">
                                <h3>"Loading workflows"</h3>
                                <p>"Fetching workflow definitions."</p>
                            </section>
                        }
                        .into_any()
                    } else if let Some(error) = load_error.get() {
                        view! {
                            <section class="organization-state is-error" role="alert">
                                <h3>"Workflows unavailable"</h3>
                                <p>{error}</p>
                            </section>
                        }
                        .into_any()
                    } else {
                        view! {
                            <WorkflowsList
                                workflows=filtered_workflows()
                                search=search
                                status_filter=status_filter
                                status_options=status_options()
                                organization_nodes=organization_nodes.get()
                            />
                        }
                        .into_any()
                    }
                }}
            </section>
        </AppShell>
    }
}

#[component]
pub fn WorkflowsNewPage() -> impl IntoView {
    view! { <ResetRoute active_route="workflows" title="Create Workflow" route="/workflows/new" status="Registered" next_step="Restore native workflow create form."/> }
}

#[component]
pub fn WorkflowAssignmentsPage() -> impl IntoView {
    view! { <ResetRoute active_route="workflows" title="Workflow Assignments" route="/workflows/assignments" status="Registered" next_step="Restore assignment console."/> }
}

#[component]
pub fn WorkflowsDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="workflows" title="Workflow Detail" route="/workflows/:workflow_id" status="Registered" next_step="Restore workflow detail screen."/> }
}

#[component]
pub fn WorkflowsEditPage() -> impl IntoView {
    view! { <ResetRoute active_route="workflows" title="Edit Workflow" route="/workflows/:workflow_id/edit" status="Registered" next_step="Restore workflow editor."/> }
}

#[component]
pub fn ResponsesPage() -> impl IntoView {
    view! { <ResetRoute active_route="responses" title="Responses" route="/responses" status="Queued" next_step="Restore response list and RFC2822 timestamp formatting."/> }
}

#[component]
pub fn ResponsesNewPage() -> impl IntoView {
    view! { <ResetRoute active_route="responses" title="Start Response" route="/responses/new" status="Registered" next_step="Restore response start workflow."/> }
}

#[component]
pub fn ResponsesDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="responses" title="Response Detail" route="/responses/:submission_id" status="Registered" next_step="Restore response detail inspection."/> }
}

#[component]
pub fn ResponsesEditPage() -> impl IntoView {
    view! { <ResetRoute active_route="responses" title="Edit Response" route="/responses/:submission_id/edit" status="Registered" next_step="Restore response edit workflow."/> }
}

#[component]
pub fn ComponentsPage() -> impl IntoView {
    view! { <ResetRoute active_route="components" title="Components" route="/components" status="Queued" next_step="Restore component catalog."/> }
}

#[component]
pub fn ComponentsDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="components" title="Component Detail" route="/components/:component_ref" status="Registered" next_step="Restore component detail view."/> }
}

#[component]
pub fn DashboardsPage() -> impl IntoView {
    view! { <ResetRoute active_route="dashboards" title="Dashboards" route="/dashboards" status="Queued" next_step="Restore dashboard list and chart cards."/> }
}

#[component]
pub fn DashboardsNewPage() -> impl IntoView {
    view! { <ResetRoute active_route="dashboards" title="Create Dashboard" route="/dashboards/new" status="Registered" next_step="Restore dashboard builder."/> }
}

#[component]
pub fn DashboardsDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="dashboards" title="Dashboard Detail" route="/dashboards/:dashboard_id" status="Registered" next_step="Restore dashboard detail."/> }
}

#[component]
pub fn DashboardsEditPage() -> impl IntoView {
    view! { <ResetRoute active_route="dashboards" title="Edit Dashboard" route="/dashboards/:dashboard_id/edit" status="Registered" next_step="Restore dashboard edit workflow."/> }
}

#[component]
pub fn DatasetsPage() -> impl IntoView {
    view! { <ResetRoute active_route="datasets" title="Datasets" route="/datasets" status="Queued" next_step="Restore dataset list and definitions."/> }
}

#[component]
pub fn DatasetsDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="datasets" title="Dataset Detail" route="/datasets/:dataset_id" status="Registered" next_step="Restore dataset detail."/> }
}

#[component]
pub fn AdministrationPage() -> impl IntoView {
    view! { <ResetRoute active_route="administration" title="Administration" route="/administration" status="Queued" next_step="Restore admin landing links and summaries."/> }
}

#[component]
pub fn AdministrationUsersPage() -> impl IntoView {
    view! { <ResetRoute active_route="administration" title="Users" route="/administration/users" status="Registered" next_step="Restore user management."/> }
}

#[component]
pub fn AdministrationNodeTypesPage() -> impl IntoView {
    view! { <ResetRoute active_route="administration" title="Node Types" route="/administration/node-types" status="Registered" next_step="Restore node type management."/> }
}

#[component]
pub fn AdministrationRolesPage() -> impl IntoView {
    view! { <ResetRoute active_route="administration" title="Roles" route="/administration/roles" status="Registered" next_step="Restore role management."/> }
}

#[component]
pub fn MigrationPage() -> impl IntoView {
    view! { <ResetRoute active_route="migration" title="Migration" route="/migration" status="Queued" next_step="Restore migration import workflow."/> }
}

#[component]
pub fn NotFoundPage() -> impl IntoView {
    view! {
        <AppShell active_route="home" title="Not Found">
            <section class="route-panel">
                <EmptyState title="Route not found" message="This route is not registered in the native reset shell."/>
            </section>
        </AppShell>
    }
}
