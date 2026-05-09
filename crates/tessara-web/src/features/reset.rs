use std::collections::HashMap;
use std::collections::HashSet;

use icons::{
    ChevronDown, ChevronRight, ExternalLink, LockKeyhole, Mail, PanelRight, Pencil, Plus, Search,
    SlidersHorizontal, X,
};
use leptos::portal::Portal;
use leptos::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::infra::routing::{NodeRouteParams, require_route_params};
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
}

const ROUTE_MIGRATIONS: [RouteMigration; 32] = [
    RouteMigration {
        name: "Home",
        route: "/",
        href: "/",
        status: "In Progress",
    },
    RouteMigration {
        name: "Login",
        route: "/login",
        href: "/login",
        status: "Done",
    },
    RouteMigration {
        name: "Organization",
        route: "/organization",
        href: "/organization",
        status: "Done",
    },
    RouteMigration {
        name: "Create Organization Node",
        route: "/organization/new",
        href: "/organization/new",
        status: "Done",
    },
    RouteMigration {
        name: "Organization Detail",
        route: "/organization/:node_id",
        href: "/organization/fb3fb3c8-2670-4c85-bcda-be59dd3aa119",
        status: "Done",
    },
    RouteMigration {
        name: "Edit Organization Node",
        route: "/organization/:node_id/edit",
        href: "/organization/demo-partner-north-star-services/edit",
        status: "Pending",
    },
    RouteMigration {
        name: "Forms",
        route: "/forms",
        href: "/forms",
        status: "Pending",
    },
    RouteMigration {
        name: "Create Form",
        route: "/forms/new",
        href: "/forms/new",
        status: "Pending",
    },
    RouteMigration {
        name: "Form Detail",
        route: "/forms/:form_id",
        href: "/forms/demo-partner-profile",
        status: "Pending",
    },
    RouteMigration {
        name: "Edit Form",
        route: "/forms/:form_id/edit",
        href: "/forms/demo-partner-profile/edit",
        status: "Pending",
    },
    RouteMigration {
        name: "Workflows",
        route: "/workflows",
        href: "/workflows",
        status: "Pending",
    },
    RouteMigration {
        name: "Create Workflow",
        route: "/workflows/new",
        href: "/workflows/new",
        status: "Pending",
    },
    RouteMigration {
        name: "Workflow Assignments",
        route: "/workflows/assignments",
        href: "/workflows/assignments",
        status: "Pending",
    },
    RouteMigration {
        name: "Workflow Detail",
        route: "/workflows/:workflow_id",
        href: "/workflows/demo-intake-workflow",
        status: "Pending",
    },
    RouteMigration {
        name: "Edit Workflow",
        route: "/workflows/:workflow_id/edit",
        href: "/workflows/demo-intake-workflow/edit",
        status: "Pending",
    },
    RouteMigration {
        name: "Responses",
        route: "/responses",
        href: "/responses",
        status: "Pending",
    },
    RouteMigration {
        name: "Start Response",
        route: "/responses/new",
        href: "/responses/new",
        status: "Pending",
    },
    RouteMigration {
        name: "Response Detail",
        route: "/responses/:submission_id",
        href: "/responses/demo-submission",
        status: "Pending",
    },
    RouteMigration {
        name: "Edit Response",
        route: "/responses/:submission_id/edit",
        href: "/responses/demo-submission/edit",
        status: "Pending",
    },
    RouteMigration {
        name: "Components",
        route: "/components",
        href: "/components",
        status: "Pending",
    },
    RouteMigration {
        name: "Component Detail",
        route: "/components/:component_ref",
        href: "/components/demo-component",
        status: "Pending",
    },
    RouteMigration {
        name: "Dashboards",
        route: "/dashboards",
        href: "/dashboards",
        status: "Pending",
    },
    RouteMigration {
        name: "Create Dashboard",
        route: "/dashboards/new",
        href: "/dashboards/new",
        status: "Pending",
    },
    RouteMigration {
        name: "Dashboard Detail",
        route: "/dashboards/:dashboard_id",
        href: "/dashboards/demo-operations-dashboard",
        status: "Pending",
    },
    RouteMigration {
        name: "Edit Dashboard",
        route: "/dashboards/:dashboard_id/edit",
        href: "/dashboards/demo-operations-dashboard/edit",
        status: "Pending",
    },
    RouteMigration {
        name: "Datasets",
        route: "/datasets",
        href: "/datasets",
        status: "Pending",
    },
    RouteMigration {
        name: "Dataset Detail",
        route: "/datasets/:dataset_id",
        href: "/datasets/demo-dataset",
        status: "Pending",
    },
    RouteMigration {
        name: "Administration",
        route: "/administration",
        href: "/administration",
        status: "Pending",
    },
    RouteMigration {
        name: "Users",
        route: "/administration/users",
        href: "/administration/users",
        status: "Pending",
    },
    RouteMigration {
        name: "Node Types",
        route: "/administration/node-types",
        href: "/administration/node-types",
        status: "Pending",
    },
    RouteMigration {
        name: "Roles",
        route: "/administration/roles",
        href: "/administration/roles",
        status: "Pending",
    },
    RouteMigration {
        name: "Migration",
        route: "/migration",
        href: "/migration",
        status: "Pending",
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
    let route_rows = ROUTE_MIGRATIONS
        .iter()
        .map(|route| {
            view! {
                <tr>
                    <th scope="row">{route.name}</th>
                    <td><a href=route.href>{route.route}</a></td>
                    <td><StatusBadge label=route.status/></td>
                </tr>
            }
        })
        .collect_view();

    view! {
        <AppShell active_route="home" title="Home">
            <section class="route-panel">
                <PageHeader title="Native UI Migration" description="Routes are registered in the clean native Leptos SSR shell and will be rebuilt in navigation order."/>
                <DataTable>
                    <thead>
                        <tr>
                            <th scope="col">"Name"</th>
                            <th scope="col">"Route"</th>
                            <th scope="col">"Status"</th>
                        </tr>
                    </thead>
                    <tbody>{route_rows}</tbody>
                </DataTable>
            </section>
        </AppShell>
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
    node_type_name: String,
    node_type_singular_label: String,
    node_type_plural_label: String,
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
                        <button class="icon-button sheet-panel__close" type="button" aria-label="Close details" title="Close details" on:click=close>
                            <X class="icon-button__icon"/>
                        </button>
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
            <RelatedWorkSummary detail/>
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
fn RelatedWorkSummary(detail: OrganizationNodeDetail) -> impl IntoView {
    let active_tab = RwSignal::new("forms".to_string());
    let forms_count = detail.related_forms.len();
    let responses_count = detail.related_responses.len();
    let dashboards_count = detail.related_dashboards.len();

    view! {
        <div class="related-work-summary">
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
    let filtered_forms = move || {
        let query = search.get();
        forms
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
        <SearchableDataTable search_label="Search forms" placeholder="Search related forms" search>
            <thead>
                <tr>
                    <th scope="col">"Form name"</th>
                    <th scope="col">"Slug"</th>
                    <th scope="col">"Active version"</th>
                    <th scope="col">"View"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let rows = filtered_forms();
                    if rows.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="4">"No Related Forms to Display"</td>
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
                                        <th scope="row">{form.form_name}</th>
                                        <td>{form.form_slug}</td>
                                        <td>{form.active_version_label.unwrap_or_else(|| "-".to_string())}</td>
                                        <td>
                                            <a class="data-table__action" href=href aria-label="View form" title="View form">
                                                <ExternalLink/>
                                            </a>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </tbody>
        </SearchableDataTable>
    }
}

#[component]
fn RelatedResponsesTable(responses: Vec<NodeSubmissionLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let status_filter = RwSignal::new("all".to_string());
    let filtered_responses = move || {
        let query = search.get();
        let status = status_filter.get();
        responses
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
        <div class="searchable-data-table">
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
                    <th scope="col">"View"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let rows = filtered_responses();
                    if rows.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="6">"No Related Responses to Display"</td>
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
                                        <th scope="row">{response.form_name}</th>
                                        <td>{response.version_label}</td>
                                        <td>{sentence_label(&response.status)}</td>
                                        <td><Timestamp value=submitted_date/></td>
                                        <td>{response.submitted_by.unwrap_or_else(|| "Unknown".to_string())}</td>
                                        <td>
                                            <a class="data-table__action" href=href aria-label="View response" title="View response">
                                                <ExternalLink/>
                                            </a>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </tbody>
            </DataTable>
        </div>
    }
}

#[component]
fn StatusFilterHeader(status_filter: RwSignal<String>) -> impl IntoView {
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

    view! {
        <div class=menu_class>
            <span>"Status"</span>
            <button
                class="data-table-filter__trigger"
                type="button"
                aria-label=button_label
                title=button_label
                aria-haspopup="menu"
                aria-expanded=move || is_open.get().to_string()
                on:click=move |_| is_open.update(|open| *open = !*open)
            >
                <SlidersHorizontal/>
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
fn RelatedDashboardsTable(dashboards: Vec<NodeDashboardLink>) -> impl IntoView {
    let search = RwSignal::new(String::new());
    let filtered_dashboards = move || {
        let query = search.get();
        dashboards
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
        <SearchableDataTable search_label="Search dashboards" placeholder="Search related dashboards" search>
            <thead>
                <tr>
                    <th scope="col">"Dashboard name"</th>
                    <th scope="col">"Component Count"</th>
                    <th scope="col">"Description"</th>
                    <th scope="col">"View"</th>
                </tr>
            </thead>
            <tbody>
                {move || {
                    let rows = filtered_dashboards();
                    if rows.is_empty() {
                        view! {
                            <tr>
                                <td class="data-table__empty" colspan="4">"No Related Dashboards to Display"</td>
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
                                        <th scope="row">{dashboard.dashboard_name}</th>
                                        <td>{dashboard.component_count}</td>
                                        <td>{nonempty_text(dashboard.description.as_deref(), "No description")}</td>
                                        <td>
                                            <a class="data-table__action" href=href aria-label="View dashboard" title="View dashboard">
                                                <ExternalLink/>
                                            </a>
                                        </td>
                                    </tr>
                                }
                            })
                            .collect_view()
                            .into_any()
                    }
                }}
            </tbody>
        </SearchableDataTable>
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
                                <Button label="Back to Organization" href="/organization"/>
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
    view! { <ResetRoute active_route="organization" title="Edit Organization Node" route="/organization/:node_id/edit" status="Registered" next_step="Restore native edit form after detail behavior is stable."/> }
}

#[component]
pub fn FormsPage() -> impl IntoView {
    view! { <ResetRoute active_route="forms" title="Forms" route="/forms" status="Queued" next_step="Restore form list APIs and native table."/> }
}

#[component]
pub fn FormsNewPage() -> impl IntoView {
    view! { <ResetRoute active_route="forms" title="Create Form" route="/forms/new" status="Registered" next_step="Restore form builder primitives."/> }
}

#[component]
pub fn FormsDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="forms" title="Form Detail" route="/forms/:form_id" status="Registered" next_step="Restore published version and field inspection."/> }
}

#[component]
pub fn FormsEditPage() -> impl IntoView {
    view! { <ResetRoute active_route="forms" title="Edit Form" route="/forms/:form_id/edit" status="Registered" next_step="Restore form editing workflow."/> }
}

#[component]
pub fn WorkflowsPage() -> impl IntoView {
    view! { <ResetRoute active_route="workflows" title="Workflows" route="/workflows" status="Queued" next_step="Restore workflow list and definitions."/> }
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
