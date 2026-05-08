use icons::{LockKeyhole, Mail};
use leptos::prelude::*;

use crate::ui::components::{
    AppShell, DataTable, EmptyState, InfoListTable, InfoRow, PageHeader, StatusBadge,
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
        status: "In Progress",
    },
    RouteMigration {
        name: "Create Organization Node",
        route: "/organization/new",
        href: "/organization/new",
        status: "Pending",
    },
    RouteMigration {
        name: "Organization Detail",
        route: "/organization/:node_id",
        href: "/organization/demo-partner-north-star-services",
        status: "Pending",
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
    view! { <ResetRoute active_route="organization" title="Organization" route="/organization" status="Next rebuild target" next_step="Restore hierarchy DTOs, node APIs, and native nested collapsibles."/> }
}

#[component]
pub fn OrganizationNewPage() -> impl IntoView {
    view! { <ResetRoute active_route="organization" title="Create Organization Node" route="/organization/new" status="Registered" next_step="Restore native create form after organization list lands."/> }
}

#[component]
pub fn OrganizationDetailPage() -> impl IntoView {
    view! { <ResetRoute active_route="organization" title="Organization Detail" route="/organization/:node_id" status="Registered" next_step="Restore details sheet and related work after organization list lands."/> }
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
