use std::collections::BTreeMap;
use std::collections::HashMap;
use std::collections::HashSet;

use icons::{
    ChevronDown, ChevronRight, ExternalLink, ListFilter, LockKeyhole, Mail, PanelRight, Pencil,
    Plus, Search, X,
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
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Form Detail",
        route: "/forms/:form_id",
        href: "/forms/demo-partner-profile",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Edit Form",
        route: "/forms/:form_id/edit",
        href: "/forms/demo-partner-profile/edit",
        status: "Pending",
        rbac_status: "Pending",
    },
    RouteMigration {
        name: "Workflows",
        route: "/workflows",
        href: "/workflows",
        status: "Pending",
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
    version_label: Option<String>,
    status: String,
    published_at: Option<String>,
    field_count: i64,
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

fn form_version_label(version: Option<&FormVersionSummary>) -> String {
    version
        .and_then(|version| version.version_label.as_deref())
        .map(str::to_string)
        .unwrap_or_else(|| "-".to_string())
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
