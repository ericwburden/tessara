//! Leptos components for the user-facing Tessara application shell.

use leptos::prelude::*;

use crate::brand::document_head_tags;

fn render_application_document(
    title: &str,
    description: &str,
    style: &str,
    script: &str,
    shell: String,
) -> String {
    let brand = document_head_tags(title, description);

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    {brand}
    <style>{style}</style>
  </head>
  <body>
    {shell}
    <script>{script}</script>
  </body>
</html>"#
    )
}

/// Builds the application shell document used for human workflow testing.
pub fn application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Home",
        "Tessara application home for local replacement workflow testing.",
        style,
        script,
        view! { <HomeApplicationShell/> }.to_html(),
    )
}

/// Builds the organization application shell document.
pub fn organization_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Organization",
        "Tessara organization area for browsing hierarchy and scoped operational records.",
        style,
        script,
        view! { <OrganizationApplicationShell/> }.to_html(),
    )
}

/// Builds the forms application shell document.
pub fn forms_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Forms",
        "Tessara forms area for browsing published and configured forms.",
        style,
        script,
        view! { <FormsApplicationShell/> }.to_html(),
    )
}

/// Builds the responses application shell document.
pub fn responses_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Responses",
        "Tessara responses area for draft, submitted, and reviewable form workflows.",
        style,
        script,
        view! { <ResponsesApplicationShell/> }.to_html(),
    )
}

/// Builds the focused submission application shell document.
pub fn submission_application_shell_html(style: &str, script: &str) -> String {
    responses_application_shell_html(style, script)
}

/// Builds the dashboards application shell document.
pub fn dashboards_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Dashboards",
        "Tessara dashboards area for previewing dashboard surfaces and chart-backed views.",
        style,
        script,
        view! { <DashboardsApplicationShell/> }.to_html(),
    )
}

/// Builds the focused administration application shell document.
pub fn administration_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Administration",
        "Tessara administration area for internal hierarchy, form, and reporting configuration.",
        style,
        script,
        view! { <AdministrationApplicationShell/> }.to_html(),
    )
}

/// Builds the focused admin application shell document.
pub fn admin_application_shell_html(style: &str, script: &str) -> String {
    administration_application_shell_html(style, script)
}

/// Builds the focused migration workbench application shell document.
pub fn migration_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Migration",
        "Tessara migration workbench for validating and rehearsing legacy imports.",
        style,
        script,
        view! { <MigrationApplicationShell/> }.to_html(),
    )
}

/// Builds the focused reporting application shell document.
pub fn reporting_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Reports",
        "Tessara reporting workspace for analytics, table reports, and dashboard previews.",
        style,
        script,
        view! { <ReportsApplicationShell/> }.to_html(),
    )
}

#[derive(Copy, Clone)]
struct ActionSpec {
    handler: &'static str,
    label: &'static str,
}

#[derive(Copy, Clone)]
struct ManagementCardSpec {
    title: &'static str,
    description: &'static str,
    href: &'static str,
    href_label: &'static str,
    action: &'static str,
    action_label: &'static str,
}

#[derive(Copy, Clone)]
struct DirectoryCardSpec {
    title: &'static str,
    description: &'static str,
    action: &'static str,
    label: &'static str,
}

#[derive(Copy, Clone)]
struct QueueCardSpec {
    title: &'static str,
    description: &'static str,
    action: &'static str,
    href: &'static str,
    label: &'static str,
}

const HOME_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "logout()",
        label: "Log Out",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const ORGANIZATION_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "loadNodes()",
        label: "Load Nodes",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const FORMS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "loadForms()",
        label: "Load Forms",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const RESPONSES_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "logout()",
        label: "Log Out",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "startDemoSubmissionFlow()",
        label: "Start Demo Response",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const ADMINISTRATION_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const MIGRATION_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "logout()",
        label: "Log Out",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const REPORTS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "logout()",
        label: "Log Out",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "openDemoDashboard()",
        label: "Open Demo Dashboard",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

const DASHBOARDS_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "login()",
        label: "Log In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Current User",
    },
    ActionSpec {
        handler: "logout()",
        label: "Log Out",
    },
    ActionSpec {
        handler: "seedDemo()",
        label: "Seed Demo",
    },
    ActionSpec {
        handler: "openDemoDashboard()",
        label: "Open Demo Dashboard",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Load App Summary",
    },
];

#[component]
fn AppAreaShell(
    active_route: &'static str,
    area_kind: &'static str,
    title: &'static str,
    description: &'static str,
    show_create_shortcuts: bool,
    actions: &'static [ActionSpec],
    children: Children,
) -> impl IntoView {
    let is_home = active_route == "home";
    let breadcrumb = if is_home {
        view! { <span>"Home"</span> }.into_any()
    } else {
        view! {
            <>
                <a href="/app">"Home"</a>
                <span>{title}</span>
            </>
        }
        .into_any()
    };

    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">{breadcrumb}</nav>
                <p class="muted">{area_kind}</p>
                <h1>{title}</h1>
                <p>{description}</p>
                <div class="actions">
                    {actions
                        .iter()
                        .map(|action| {
                            view! {
                                <button type="button" onclick=action.handler>
                                    {action.label}
                                </button>
                            }
                        })
                        .collect_view()}
                </div>
            </section>
            <section class="app-layout">
                <AreaSidebar
                    active_route=active_route
                    show_create_shortcuts=show_create_shortcuts
                />
                <section class="panel app-main">{children()}</section>
            </section>
        </main>
    }
}

#[component]
fn HomeApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="home"
            area_kind="Shared Home"
            title="Application Overview"
            description="This shared home is the primary entry point for the migration UI catch-up. It exposes the target product areas while keeping the current backend-supported workflows intact."
            show_create_shortcuts=false
            actions=HOME_ACTIONS
        >
            <HomeScreen/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn OrganizationApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="organization"
            area_kind="Product Area"
            title="Organization"
            description="This area establishes the operational hierarchy surface. It currently bridges into the existing hierarchy and node workflows without introducing unsupported organization behavior."
            show_create_shortcuts=false
            actions=ORGANIZATION_ACTIONS
        >
            <OrganizationHomeScreen/>
            <OrganizationWorkspaceShell/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn FormsApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="forms"
            area_kind="Product Area"
            title="Forms"
            description="This area is the canonical entry point for form discovery and lifecycle work. It currently bridges product-facing form access and internal form configuration using existing supported routes."
            show_create_shortcuts=false
            actions=FORMS_ACTIONS
        >
            <FormsHomeScreen/>
            <FormsWorkspaceShell/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn ResponsesApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="responses"
            area_kind="Product Area"
            title="Responses"
            description="This area handles response entry, drafts, submission, and review. It uses the current backend-supported form-render and submission lifecycle without relying on utility-style navigation."
            show_create_shortcuts=false
            actions=RESPONSES_ACTIONS
        >
            <SubmissionHomeScreen/>
            <SubmissionWorkspaceShell/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn AdministrationApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="administration"
            area_kind="Internal Area"
            title="Administration"
            description="This internal area is for hierarchy, form, and reporting configuration. It remains visible during the migration, but it is intentionally scoped as an operator surface."
            show_create_shortcuts=true
            actions=ADMINISTRATION_ACTIONS
        >
            <AdminHomeScreen/>
            <AdminWorkspaceShell/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn MigrationApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="migration"
            area_kind="Internal Area"
            title="Migration Workbench"
            description="This operator screen validates and dry-runs representative legacy fixtures before running import rehearsals."
            show_create_shortcuts=false
            actions=MIGRATION_ACTIONS
        >
            <MigrationHomeScreen/>
            <FixtureScreen/>
            <section id="migration-results-screen" class="app-screen">
                <h2>"Validation Results"</h2>
                <div id="screen" class="cards"></div>
            </section>
            <RawOutputPanel/>
        </AppAreaShell>
    }
}

#[component]
fn ReportsApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="reports"
            area_kind="Product Area"
            title="Reports"
            description="This area is the canonical route for report browsing, report execution, and reporting detail traversal. Dashboard preview remains linked here until the dashboard area is split further."
            show_create_shortcuts=false
            actions=REPORTS_ACTIONS
        >
            <ReportingHomeScreen/>
            <ReportingWorkspaceShell/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn DashboardsApplicationShell() -> impl IntoView {
    view! {
        <AppAreaShell
            active_route="dashboards"
            area_kind="Product Area"
            title="Dashboards"
            description="This area is the dashboard viewing destination. It currently uses the supported dashboard preview path while the broader dashboard product surface catches up."
            show_create_shortcuts=false
            actions=DASHBOARDS_ACTIONS
        >
            <DashboardsHomeScreen/>
            <DashboardsWorkspaceShell/>
            <OutputPanels/>
        </AppAreaShell>
    }
}

#[component]
fn BrandLockup() -> impl IntoView {
    view! {
        <div class="brand-lockup">
            <img class="brand-mark" src="/assets/tessara-icon-1024.svg" alt="" />
            <span>"Tessara"</span>
        </div>
    }
}

#[component]
fn AreaSidebar(active_route: &'static str, show_create_shortcuts: bool) -> impl IntoView {
    view! {
        <aside class="panel app-sidebar">
            <ApplicationNav active_route=active_route/>
            {show_create_shortcuts
                .then(|| view! { <CreateMenu/> })
                .into_any()}
            <SelectionContext/>
        </aside>
    }
}

#[component]
fn ScreenSection(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="app-screen">
            <p class="eyebrow">{eyebrow}</p>
            <h2>{title}</h2>
            <p class="muted">{description}</p>
            {children()}
        </section>
    }
}

#[component]
fn ManagementCardsSection(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    cards: Vec<ManagementCardSpec>,
) -> impl IntoView {
    view! {
        <ScreenSection eyebrow=eyebrow title=title description=description>
            <div class="management-grid">
                {cards
                    .into_iter()
                    .map(|card| {
                        view! {
                            <article class="home-card">
                                <h3>{card.title}</h3>
                                <p>{card.description}</p>
                                <div class="actions">
                                    <a class="button-link" href=card.href>{card.href_label}</a>
                                    <button type="button" onclick=card.action>
                                        {card.action_label}
                                    </button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </ScreenSection>
    }
}

#[component]
fn DirectoryCardsSection(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    cards: Vec<DirectoryCardSpec>,
) -> impl IntoView {
    view! {
        <ScreenSection eyebrow=eyebrow title=title description=description>
            <div class="directory-grid">
                {cards
                    .into_iter()
                    .map(|card| {
                        view! {
                            <article class="directory-card">
                                <h3>{card.title}</h3>
                                <p>{card.description}</p>
                                <button type="button" onclick=card.action>{card.label}</button>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </ScreenSection>
    }
}

#[component]
fn WorkspaceShellSection(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    queue_title: &'static str,
    queue_cards: Vec<QueueCardSpec>,
    path_title: &'static str,
    path_steps: Vec<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="app-screen">
            <p class="eyebrow">{eyebrow}</p>
            <h2>{title}</h2>
            <p class="muted">{description}</p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>{queue_title}</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|card| {
                                    let action_view = if card.href.is_empty() {
                                        view! {
                                            <button type="button" onclick=card.action>{card.label}</button>
                                        }
                                        .into_any()
                                    } else {
                                        view! { <a class="button-link" href=card.href>{card.label}</a> }
                                            .into_any()
                                    };

                                    view! {
                                        <article class="workspace-card">
                                            <h4>{card.title}</h4>
                                            <p>{card.description}</p>
                                            {action_view}
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>{path_title}</h3>
                        <ol class="app-list">
                            {path_steps
                                .into_iter()
                                .map(|step| view! { <li>{step}</li> })
                                .collect_view()}
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">{children()}</div>
            </div>
        </section>
    }
}

#[component]
fn SelectionContext() -> impl IntoView {
    view! {
        <section class="selection-panel">
            <h3>"Selection Context"</h3>
            <p class="muted">
                "Selections from published forms, nodes, and submissions populate this workflow."
            </p>
            <p id="session-status" class="muted">"Not signed in."</p>
            <div id="selection-state" class="selection-grid">
                <p class="muted">"No records selected yet."</p>
            </div>
        </section>
    }
}

#[component]
fn ApplicationNav(active_route: &'static str) -> impl IntoView {
    let product_links = [
        ("home", "/app", "Home"),
        ("organization", "/app/organization", "Organization"),
        ("forms", "/app/forms", "Forms"),
        ("responses", "/app/responses", "Responses"),
        ("reports", "/app/reports", "Reports"),
        ("dashboards", "/app/dashboards", "Dashboards"),
    ];
    let internal_links = [
        ("administration", "/app/administration", "Administration"),
        ("migration", "/app/migration", "Migration"),
    ];

    view! {
        <>
            <section class="nav-panel">
                <h2>"Product Areas"</h2>
                <nav class="app-nav" aria-label="Product navigation">
                    {product_links
                        .into_iter()
                        .map(|(route_key, href, label)| {
                            let class_name = if route_key == active_route {
                                "active"
                            } else {
                                ""
                            };
                            view! { <a class=class_name href=href>{label}</a> }
                        })
                        .collect_view()}
                </nav>
            </section>
            <section class="nav-panel nav-panel-secondary">
                <h2>"Internal Areas"</h2>
                <nav class="app-nav" aria-label="Internal navigation">
                    {internal_links
                        .into_iter()
                        .map(|(route_key, href, label)| {
                            let class_name = if route_key == active_route {
                                "active"
                            } else {
                                ""
                            };
                            view! { <a class=class_name href=href>{label}</a> }
                        })
                        .collect_view()}
                </nav>
            </section>
        </>
    }
}

#[component]
fn CreateMenu() -> impl IntoView {
    let create_links = [
        (
            "Create Node",
            "/app/administration#organization-setup-screen",
        ),
        (
            "Create Form",
            "/app/administration#forms-configuration-screen",
        ),
        (
            "Create Dataset",
            "/app/administration#reporting-configuration-screen",
        ),
        (
            "Create Report",
            "/app/administration#reporting-configuration-screen",
        ),
        (
            "Create Aggregation",
            "/app/administration#reporting-configuration-screen",
        ),
        (
            "Create Dashboard",
            "/app/administration#reporting-configuration-screen",
        ),
    ];

    view! {
        <section class="nav-panel">
            <h2>"Create Shortcuts"</h2>
            <p class="muted">
                "These links currently open supported creation flows in the internal configuration areas."
            </p>
            <div class="create-menu">
                {create_links
                    .into_iter()
                    .map(|(label, href)| view! { <a class="create-link" href=href>{label}</a> })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn HomeScreen() -> impl IntoView {
    let product_cards = [
        (
            "Organization",
            "Browse the configured hierarchy and move toward scoped forms, responses, and dashboards.",
            "/app/organization",
            "Browse Organization",
        ),
        (
            "Forms",
            "Browse form definitions and move into the supported form lifecycle and publishing surfaces.",
            "/app/forms",
            "Browse Forms",
        ),
        (
            "Responses",
            "Complete draft, save, submit, and review flows for published forms.",
            "/app/responses",
            "Open Responses",
        ),
        (
            "Reports",
            "Inspect reports, run aggregations, and traverse linked reporting assets.",
            "/app/reports",
            "Open Reports",
        ),
        (
            "Dashboards",
            "Open dashboard previews and chart-backed surfaces without dropping into reporting configuration first.",
            "/app/dashboards",
            "Open Dashboards",
        ),
    ];
    let internal_cards = [
        (
            "Administration",
            "Configure hierarchy, forms, datasets, reports, aggregations, charts, and dashboards.",
            "/app/administration",
            "Open Administration",
        ),
        (
            "Migration",
            "Validate, dry-run, and rehearse legacy imports from one operator-focused route.",
            "/app/migration",
            "Open Migration",
        ),
    ];

    view! {
        <section id="home-screen" class="app-screen">
            <p class="eyebrow">"Application Home"</p>
            <h2>"Welcome to Tessara"</h2>
            <p class="muted">
                "Use this home screen as the entry point for the migrated application. "
                "The structure reflects the original system's broad navigation model while "
                "keeping cleaner, selection-driven entry points."
            </p>
            <div class="actions">
                <button type="button" onclick="loadAppSummary()">"Refresh Overview"</button>
                <button type="button" onclick="seedDemo()">"Seed Demo Data"</button>
                <button type="button" onclick="startDemoSubmissionFlow()">"Start Demo Response"</button>
                <button type="button" onclick="openDemoDashboard()">"Open Demo Dashboard"</button>
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Application Home"</p>
            <h2>"Product Areas"</h2>
            <div class="home-grid">
                {product_cards
                    .into_iter()
                    .map(|(title, description, href, label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <a class="button-link" href=href>{label}</a>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Application Home"</p>
            <h2>"Internal Areas"</h2>
            <div class="home-grid">
                {internal_cards
                    .into_iter()
                    .map(|(title, description, href, label)| {
                        view! {
                            <article class="directory-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <a class="button-link" href=href>{label}</a>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Application Home"</p>
            <h2>"Route Map"</h2>
            <ul class="app-list">
                <li>"Home provides overview, quick starts, and product-area entry points."</li>
                <li>"Organization is the operational hierarchy surface."</li>
                <li>"Forms is the product-facing form area, while configuration stays scoped."</li>
                <li>"Responses is the current supported route for draft, submit, and review workflows."</li>
                <li>"Reports and Dashboards are separate viewing destinations, even where they still share underlying reporting support."</li>
                <li>"Administration and Migration remain visible internal/operator areas."</li>
            </ul>
        </section>
    }
}

#[component]
fn OrganizationHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Browse Nodes",
            description: "Load the current runtime nodes and move through the operational hierarchy.",
            href: "#organization-setup-screen",
            href_label: "Browse Organization",
            action: "loadNodes()",
            action_label: "Load Nodes",
        },
        ManagementCardSpec {
            title: "Inspect Node Types",
            description: "Review the configured hierarchy structure and labels behind the organization area.",
            href: "#organization-setup-screen",
            href_label: "Open Structure",
            action: "loadNodeTypes()",
            action_label: "Load Node Types",
        },
        ManagementCardSpec {
            title: "Browse Forms",
            description: "Move from organization browsing into the scoped forms area.",
            href: "/app/forms",
            href_label: "Go to Forms",
            action: "loadForms()",
            action_label: "Load Forms",
        },
        ManagementCardSpec {
            title: "View Dashboards",
            description: "Move from organization browsing into current dashboard viewing surfaces.",
            href: "/app/dashboards",
            href_label: "Go to Dashboards",
            action: "loadDashboards()",
            action_label: "Load Dashboards",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Organization Home"
            title="Organization Areas"
            description="This route is the structural bridge from the legacy partner/program model into Tessara's configurable hierarchy."
            cards=management_cards.to_vec()
        />
    }
}

#[component]
fn FormsHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Browse Forms",
            description: "Open the current forms directory and inspect configured forms and versions.",
            href: "#forms-configuration-screen",
            href_label: "Browse Forms",
            action: "loadForms()",
            action_label: "Load Forms",
        },
        ManagementCardSpec {
            title: "Go to Responses",
            description: "Move into the response workflow for published form completion and review.",
            href: "/app/responses",
            href_label: "Go to Responses",
            action: "loadForms()",
            action_label: "Load Forms",
        },
        ManagementCardSpec {
            title: "Go to Organization",
            description: "Return to the organization area for scoped navigation into forms.",
            href: "/app/organization",
            href_label: "Go to Organization",
            action: "loadNodeTypes()",
            action_label: "Load Node Types",
        },
        ManagementCardSpec {
            title: "Go to Administration",
            description: "Use the internal configuration surface for full hierarchy and reporting setup.",
            href: "/app/administration",
            href_label: "Go to Administration",
            action: "loadForms()",
            action_label: "Load Forms",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Forms Home"
            title="Forms Areas"
            description="This route is the product-facing entry into form discovery, version awareness, and supported form lifecycle tasks."
            cards=management_cards.to_vec()
        />
    }
}

#[component]
fn AdminHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Hierarchy",
            description: "Manage node types, relationships, metadata fields, and runtime nodes.",
            href: "#organization-setup-screen",
            href_label: "Open Organization Setup",
            action: "loadNodeTypes()",
            action_label: "Load Node Types",
        },
        ManagementCardSpec {
            title: "Forms",
            description: "Create forms, draft versions, edit sections and fields, and publish revisions.",
            href: "#forms-configuration-screen",
            href_label: "Open Forms Configuration",
            action: "loadForms()",
            action_label: "Load Forms",
        },
        ManagementCardSpec {
            title: "Datasets and Reports",
            description: "Manage datasets, reports, and aggregations inside the reporting stack.",
            href: "#reporting-configuration-screen",
            href_label: "Open Reporting Configuration",
            action: "loadDatasets()",
            action_label: "Load Datasets",
        },
        ManagementCardSpec {
            title: "Dashboards",
            description: "Inspect charts, dashboards, and current preview outputs from one admin route.",
            href: "#reporting-configuration-screen",
            href_label: "Open Dashboard Configuration",
            action: "loadDashboards()",
            action_label: "Load Dashboards",
        },
    ];

    let directory_cards = [
        DirectoryCardSpec {
            title: "Node Types",
            description: "Browse hierarchy types",
            action: "loadNodeTypes()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Nodes",
            description: "Browse runtime nodes",
            action: "loadNodes()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Forms",
            description: "Browse forms and versions",
            action: "loadForms()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Datasets",
            description: "Browse dataset definitions",
            action: "loadDatasets()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Reports",
            description: "Browse report definitions",
            action: "loadReports()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Aggregations",
            description: "Browse aggregation definitions",
            action: "loadAggregations()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Charts",
            description: "Browse charts",
            action: "loadCharts()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Dashboards",
            description: "Browse dashboards",
            action: "loadDashboards()",
            label: "Open",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Admin Home"
            title="Management Areas"
            description="Use this admin landing section to jump into the main management areas before dropping into the detailed builder screens."
            cards=management_cards.to_vec()
        />
        <DirectoryCardsSection
            eyebrow="Admin Home"
            title="Entity Directory"
            description="These entry points mirror the original application's core management lists while keeping the current Tessara builder controls underneath."
            cards=directory_cards.to_vec()
        />
    }
}

#[component]
fn SubmissionHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Start a Response",
            description: "Choose a published form and target node, then open the form for draft entry.",
            href: "#response-entry-screen",
            href_label: "Start Response Entry",
            action: "loadPublishedForms()",
            action_label: "Load Published Forms",
        },
        ManagementCardSpec {
            title: "Browse Targets",
            description: "Browse nodes and carry the selected target directly into the response flow.",
            href: "#response-entry-screen",
            href_label: "Browse Targets",
            action: "loadNodes()",
            action_label: "Load Target Nodes",
        },
        ManagementCardSpec {
            title: "Review Responses",
            description: "Browse draft and submitted responses, then reopen the selected submission in context.",
            href: "#response-review-screen",
            href_label: "Review Responses",
            action: "loadSubmissions()",
            action_label: "Load Submissions",
        },
        ManagementCardSpec {
            title: "View Related Reports",
            description: "Jump from the submission route into supporting report output while reviewing responses.",
            href: "#response-report-screen",
            href_label: "View Related Reports",
            action: "loadReports()",
            action_label: "Load Reports",
        },
    ];

    let directory_cards = [
        DirectoryCardSpec {
            title: "Published Forms",
            description: "Browse current published forms",
            action: "loadPublishedForms()",
            label: "Browse",
        },
        DirectoryCardSpec {
            title: "Target Nodes",
            description: "Browse submission targets",
            action: "loadNodes()",
            label: "Browse",
        },
        DirectoryCardSpec {
            title: "Draft Responses",
            description: "Filter to draft submissions",
            action: "showDraftSubmissions()",
            label: "Review",
        },
        DirectoryCardSpec {
            title: "Submitted Responses",
            description: "Filter to submitted responses",
            action: "showSubmittedSubmissions()",
            label: "Review",
        },
        DirectoryCardSpec {
            title: "All Responses",
            description: "Browse the full response list",
            action: "loadSubmissions()",
            label: "Review",
        },
        DirectoryCardSpec {
            title: "Reports",
            description: "Browse related reports",
            action: "loadReports()",
            label: "View",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Responses Home"
            title="Response Stages"
            description="Use this route-level landing section to move between response entry, target selection, review, and related reporting without relying on one long stacked screen."
            cards=management_cards.to_vec()
        />
        <DirectoryCardsSection
            eyebrow="Responses Home"
            title="Response Directory"
            description="These entry points keep submissions aligned with the application shell by emphasizing common lists and review paths over raw-ID entry."
            cards=directory_cards.to_vec()
        />
    }
}

#[component]
fn OrganizationWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        QueueCardSpec {
            title: "Runtime Nodes",
            description: "Browse current nodes and inspect operational hierarchy records.",
            action: "loadNodes()",
            href: "",
            label: "Browse Nodes",
        },
        QueueCardSpec {
            title: "Structure Types",
            description: "Review node types, relationships, and metadata definitions.",
            action: "loadNodeTypes()",
            href: "",
            label: "Browse Structure",
        },
        QueueCardSpec {
            title: "Forms Bridge",
            description: "Move from organization structure into the current forms area.",
            action: "",
            href: "/app/forms",
            label: "Go to Forms",
        },
        QueueCardSpec {
            title: "Dashboards Bridge",
            description: "Move from organization structure into the current dashboards area.",
            action: "",
            href: "/app/dashboards",
            label: "Go to Dashboards",
        },
    ];
    let path_steps = vec![
        "Browse nodes and hierarchy structure.",
        "Inspect the configured labels and relationships.",
        "Move into forms, responses, or dashboards from the scoped structure.",
    ];

    view! {
        <WorkspaceShellSection
            eyebrow="Organization Workspace"
            title="Organization Console"
            description="This route is the first organization-area bridge. It keeps hierarchy work discoverable now while later sprints replace more of the internal builder feel with directory and detail flows."
            queue_title="Organization Queues"
            queue_cards=queue_cards.to_vec()
            path_title="Organization Path"
            path_steps=path_steps
        >
            <HierarchyAdminScreen
                eyebrow="Organization Screen"
                title="Organization Directory"
                description="Browse hierarchy structure, metadata definitions, and runtime nodes while the organization area still reuses the shared management surface."
                action_title="Organization Actions"
                context_title="Current Organization Context"
            />
        </WorkspaceShellSection>
    }
}

#[component]
fn FormsWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        QueueCardSpec {
            title: "Forms Directory",
            description: "Browse current form records and inspect definitions.",
            action: "loadForms()",
            href: "",
            label: "Browse Forms",
        },
        QueueCardSpec {
            title: "Response Bridge",
            description: "Move from form discovery into the supported responses area.",
            action: "",
            href: "/app/responses",
            label: "Go to Responses",
        },
        QueueCardSpec {
            title: "Organization Bridge",
            description: "Return to the organization surface for scoped form navigation.",
            action: "",
            href: "/app/organization",
            label: "Go to Organization",
        },
    ];
    let path_steps = vec![
        "Browse or inspect the form.",
        "Choose the relevant version or draft.",
        "Move into response entry or internal configuration as needed.",
    ];

    view! {
        <WorkspaceShellSection
            eyebrow="Forms Workspace"
            title="Forms Console"
            description="This route is the current bridge between product-facing form discovery and the supported internal form lifecycle tasks."
            queue_title="Forms Queues"
            queue_cards=queue_cards.to_vec()
            path_title="Forms Path"
            path_steps=path_steps
        >
            <FormAdminScreen
                eyebrow="Forms Screen"
                title="Forms Directory"
                description="Browse current forms, versions, and publish state while the forms area still reuses the shared configuration surface."
                action_title="Forms Actions"
                context_title="Current Forms Context"
            />
        </WorkspaceShellSection>
    }
}

#[component]
fn SubmissionWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        QueueCardSpec {
            title: "Published Forms",
            description: "Load the current published response options.",
            action: "loadPublishedForms()",
            href: "",
            label: "Browse Forms",
        },
        QueueCardSpec {
            title: "Target Directory",
            description: "Browse organizations, programs, and other submission targets.",
            action: "loadNodes()",
            href: "",
            label: "Browse Targets",
        },
        QueueCardSpec {
            title: "Draft Queue",
            description: "Review in-progress drafts that still need edits or submission.",
            action: "showDraftSubmissions()",
            href: "",
            label: "Review Drafts",
        },
        QueueCardSpec {
            title: "Submitted Queue",
            description: "Review completed responses and continue into reporting.",
            action: "showSubmittedSubmissions()",
            href: "",
            label: "Review Submitted",
        },
    ];
    let path_steps = vec![
        "Choose a published form.",
        "Choose the target node.",
        "Open the response form and create a draft.",
        "Save values, submit, then review the resulting record.",
    ];

    view! {
        <WorkspaceShellSection
            eyebrow="Responses Workspace"
            title="Response Console"
            description="This route now acts as an application workspace: the left side focuses on queues and entry points, while the right side carries the active response, review, and reporting surfaces."
            queue_title="Response Queues"
            queue_cards=queue_cards.to_vec()
            path_title="Guided Path"
            path_steps=path_steps
        >
            <SubmissionScreen/>
            <ReviewScreen/>
            <ReportScreen/>
        </WorkspaceShellSection>
    }
}

#[component]
fn AdminWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        QueueCardSpec {
            title: "Hierarchy Types",
            description: "Open node-type and relationship management for structural changes.",
            action: "loadNodeTypes()",
            href: "",
            label: "Open Hierarchy",
        },
        QueueCardSpec {
            title: "Forms Directory",
            description: "Browse forms, versions, and publishing status from the main admin route.",
            action: "loadForms()",
            href: "",
            label: "Open Forms",
        },
        QueueCardSpec {
            title: "Reporting Assets",
            description: "Open datasets, reports, aggregations, charts, and dashboards.",
            action: "loadDatasets()",
            href: "",
            label: "Open Reporting",
        },
        QueueCardSpec {
            title: "Runtime Nodes",
            description: "Browse and update real nodes without leaving the admin workspace.",
            action: "loadNodes()",
            href: "",
            label: "Open Nodes",
        },
    ];
    let path_steps = vec![
        "Set or inspect hierarchy types and runtime nodes.",
        "Open the correct form and version draft.",
        "Publish or review reporting assets tied to that structure.",
        "Confirm the resulting dashboards and reporting surfaces.",
    ];

    view! {
        <WorkspaceShellSection
            eyebrow="Admin Workspace"
            title="Configuration Console"
            description="This route is now shifting from a builder stack toward an admin workspace. The rail keeps high-level management queues visible while the main area holds hierarchy, form, and reporting configuration."
            queue_title="Management Queues"
            queue_cards=queue_cards.to_vec()
            path_title="Admin Path"
            path_steps=path_steps
        >
            <HierarchyAdminScreen
                eyebrow="Administration Screen"
                title="Organization Setup"
                description="Create and update node types, metadata definitions, and runtime nodes."
                action_title="Organization Actions"
                context_title="Current Organization Context"
            />
            <FormAdminScreen
                eyebrow="Administration Screen"
                title="Forms Configuration"
                description="Create draft form versions, edit sections and fields, and publish the version."
                action_title="Forms Actions"
                context_title="Current Forms Context"
            />
            <ReportAdminScreen/>
        </WorkspaceShellSection>
    }
}

#[component]
fn OutputPanels() -> impl IntoView {
    view! {
        <section class="app-screen">
            <h2>"Screen Output"</h2>
            <div id="screen" class="cards"></div>
        </section>
        <RawOutputPanel/>
    }
}

#[component]
fn ReportingHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Browse Datasets",
            description: "Inspect dataset definitions and run source-aware dataset previews before binding reports.",
            href: "#reports-runner-screen",
            href_label: "Open Dataset Workspace",
            action: "loadDatasets()",
            action_label: "Load Datasets",
        },
        ManagementCardSpec {
            title: "Review Reports",
            description: "Inspect report definitions, refresh analytics, and execute table-style outputs.",
            href: "#reports-runner-screen",
            href_label: "Open Reports Workspace",
            action: "loadReports()",
            action_label: "Load Reports",
        },
        ManagementCardSpec {
            title: "Review Aggregations",
            description: "Review aggregation definitions and execute grouped metrics on current report outputs.",
            href: "#reports-runner-screen",
            href_label: "Open Aggregations Workspace",
            action: "loadAggregations()",
            action_label: "Load Aggregations",
        },
        ManagementCardSpec {
            title: "View Dashboards",
            description: "Preview charts and dashboards with current report or aggregation context.",
            href: "#dashboard-viewer-screen",
            href_label: "Open Dashboard Viewer",
            action: "loadDashboards()",
            action_label: "Load Dashboards",
        },
    ];

    let directory_cards = [
        DirectoryCardSpec {
            title: "Datasets",
            description: "Browse dataset definitions",
            action: "loadDatasets()",
            label: "Browse",
        },
        DirectoryCardSpec {
            title: "Reports",
            description: "Browse report definitions",
            action: "loadReports()",
            label: "Review",
        },
        DirectoryCardSpec {
            title: "Aggregations",
            description: "Browse aggregation definitions",
            action: "loadAggregations()",
            label: "Review",
        },
        DirectoryCardSpec {
            title: "Charts",
            description: "Browse charts",
            action: "loadCharts()",
            label: "Browse",
        },
        DirectoryCardSpec {
            title: "Dashboards",
            description: "Browse dashboards",
            action: "loadDashboards()",
            label: "View",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Reports Home"
            title="Report Areas"
            description="Use this reporting landing section to move between datasets, reports, aggregations, and dashboards without dropping immediately into builder-style controls."
            cards=management_cards.to_vec()
        />
        <DirectoryCardsSection
            eyebrow="Reports Home"
            title="Reporting Directory"
            description="These entry points start to replace workbench-only reporting flows with clearer entity lists inside the application shell."
            cards=directory_cards.to_vec()
        />
    }
}

#[component]
fn DashboardsHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Browse Dashboards",
            description: "Browse dashboard surfaces and inspect current component previews.",
            href: "#dashboard-viewer-screen",
            href_label: "Browse Dashboards",
            action: "loadDashboards()",
            action_label: "Load Dashboards",
        },
        ManagementCardSpec {
            title: "Browse Charts",
            description: "Inspect chart definitions that drive dashboard components.",
            href: "#dashboard-viewer-screen",
            href_label: "Browse Charts",
            action: "loadCharts()",
            action_label: "Load Charts",
        },
        ManagementCardSpec {
            title: "View Reports",
            description: "Move into the reports area for related report and aggregation detail.",
            href: "/app/reports",
            href_label: "Go to Reports",
            action: "loadReports()",
            action_label: "Load Reports",
        },
        ManagementCardSpec {
            title: "Open Demo Dashboard",
            description: "Jump directly into the seeded dashboard preview path.",
            href: "#dashboard-viewer-screen",
            href_label: "Open Demo Preview",
            action: "openDemoDashboard()",
            action_label: "Open Demo Dashboard",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Dashboards Home"
            title="Dashboard Areas"
            description="This route separates dashboard viewing from the broader reporting route while still using the current supported preview path."
            cards=management_cards.to_vec()
        />
    }
}

#[component]
fn ReportingWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        QueueCardSpec {
            title: "Datasets",
            description: "Inspect and run datasets before binding reports or aggregations.",
            action: "loadDatasets()",
            href: "",
            label: "Browse Datasets",
        },
        QueueCardSpec {
            title: "Reports",
            description: "Review report definitions, bindings, and current result sets.",
            action: "loadReports()",
            href: "",
            label: "Review Reports",
        },
        QueueCardSpec {
            title: "Aggregations",
            description: "Check grouped metrics and the charts that depend on them.",
            action: "loadAggregations()",
            href: "",
            label: "Review Aggregations",
        },
        QueueCardSpec {
            title: "Dashboards",
            description: "Open dashboard previews and chart context from one reporting route.",
            action: "loadDashboards()",
            href: "",
            label: "View Dashboards",
        },
    ];
    let path_steps = vec![
        "Inspect the dataset or report you intend to use.",
        "Refresh analytics if the source data changed.",
        "Run the report or aggregation.",
        "Open the dashboard preview to verify the final surface.",
    ];

    view! {
        <WorkspaceShellSection
            eyebrow="Reports Workspace"
            title="Insight Console"
            description="This route now acts more like a reporting workspace: the rail keeps the reporting queues visible while the main area focuses on report execution and dashboard preview."
            queue_title="Reporting Queues"
            queue_cards=queue_cards.to_vec()
            path_title="Reporting Path"
            path_steps=path_steps
        >
            <ReportRunnerScreen/>
            <DashboardPreviewScreen/>
        </WorkspaceShellSection>
    }
}

#[component]
fn DashboardsWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        QueueCardSpec {
            title: "Dashboards",
            description: "Open dashboard previews and current component layouts.",
            action: "loadDashboards()",
            href: "",
            label: "View Dashboards",
        },
        QueueCardSpec {
            title: "Charts",
            description: "Inspect chart definitions used by current dashboard components.",
            action: "loadCharts()",
            href: "",
            label: "Browse Charts",
        },
        QueueCardSpec {
            title: "Reports Bridge",
            description: "Move to the reports area for report and aggregation detail.",
            action: "",
            href: "/app/reports",
            label: "Go to Reports",
        },
    ];
    let path_steps = vec![
        "Choose the dashboard.",
        "Inspect the current chart-backed components.",
        "Traverse into reports when deeper source detail is needed.",
    ];

    view! {
        <WorkspaceShellSection
            eyebrow="Dashboards Workspace"
            title="Dashboard Console"
            description="This route keeps dashboard viewing separate from the broader reports area while the dashboard product surface catches up."
            queue_title="Dashboard Queues"
            queue_cards=queue_cards.to_vec()
            path_title="Dashboard Path"
            path_steps=path_steps
        >
            <DashboardPreviewScreen/>
        </WorkspaceShellSection>
    }
}

#[component]
fn MigrationHomeScreen() -> impl IntoView {
    let management_cards = [
        ManagementCardSpec {
            title: "Fixture Intake",
            description: "Load bundled fixtures or paste fixture JSON to start a migration rehearsal.",
            href: "#migration-fixture-screen",
            href_label: "Open Fixture Intake",
            action: "loadLegacyFixtureExamples()",
            action_label: "Load Fixture Examples",
        },
        ManagementCardSpec {
            title: "Validation",
            description: "Run validation before import so mapping and value problems are visible early.",
            href: "#migration-fixture-screen",
            href_label: "Open Validation",
            action: "validateLegacyFixture()",
            action_label: "Validate Fixture",
        },
        ManagementCardSpec {
            title: "Dry Run",
            description: "Preview what the import would create before mutating the local rehearsal database.",
            href: "#migration-fixture-screen",
            href_label: "Open Dry Run",
            action: "dryRunLegacyFixture()",
            action_label: "Dry-Run Fixture",
        },
        ManagementCardSpec {
            title: "Import",
            description: "Run the import rehearsal and inspect the resulting entities through the app shell.",
            href: "#migration-results-screen",
            href_label: "Open Import Results",
            action: "importLegacyFixture()",
            action_label: "Import Fixture",
        },
    ];

    let directory_cards = [
        DirectoryCardSpec {
            title: "Fixture Examples",
            description: "Load bundled fixtures",
            action: "loadLegacyFixtureExamples()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Validation Results",
            description: "Run validation now",
            action: "validateLegacyFixture()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Dry Runs",
            description: "Run a dry-run rehearsal",
            action: "dryRunLegacyFixture()",
            label: "Open",
        },
        DirectoryCardSpec {
            title: "Imports",
            description: "Run import rehearsal",
            action: "importLegacyFixture()",
            label: "Open",
        },
    ];

    view! {
        <ManagementCardsSection
            eyebrow="Migration Home"
            title="Migration Stages"
            description="Use this operator landing section to move through fixture intake, validation, dry run, and import without relying on a single workbench panel."
            cards=management_cards.to_vec()
        />
        <DirectoryCardsSection
            eyebrow="Migration Home"
            title="Migration Directory"
            description="These entry points keep the migration workflow operator-focused while still fitting inside the shared application shell."
            cards=directory_cards.to_vec()
        />
    }
}

#[component]
fn RawOutputPanel() -> impl IntoView {
    view! {
        <section class="app-screen">
            <h2>"Raw Output"</h2>
            <pre id="output">"No API calls yet."</pre>
        </section>
    }
}

#[component]
fn ReportRunnerScreen() -> impl IntoView {
    view! {
        <section id="reports-runner-screen" class="app-screen">
            <p class="eyebrow">"Reports Screen"</p>
            <h2>"Report Runner"</h2>
            <p class="muted">
                "Choose a report, inspect its field bindings, and run the table output against refreshed analytics."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Reporting Actions"</h3>
                    <p class="muted">"Use the current selection context to inspect or run reporting assets."</p>
                    <div class="actions">
                        <button type="button" onclick="refreshAnalytics()">"Refresh Analytics"</button>
                        <button type="button" onclick="loadDatasets()">"Browse Datasets"</button>
                        <button type="button" onclick="loadDatasetDefinitionById()">"Inspect Dataset"</button>
                        <button type="button" onclick="loadDatasetTableById()">"View Dataset Rows"</button>
                        <button type="button" onclick="loadReports()">"Browse Reports"</button>
                        <button type="button" onclick="loadReportDefinitionById()">"Inspect Report"</button>
                        <button type="button" onclick="refreshAnalyticsAndRunReport()">"Refresh and Run Report"</button>
                        <button type="button" onclick="loadReportById()">"View Report"</button>
                        <button type="button" onclick="loadAggregations()">"Browse Aggregations"</button>
                        <button type="button" onclick="loadAggregationDefinitionById()">"Inspect Aggregation"</button>
                        <button type="button" onclick="loadAggregationById()">"View Aggregation"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>"Current Reporting Context"</h3>
                    <div class="inputs compact-inputs">
                        <label>
                            <span>"Dataset ID"</span>
                            <input id="dataset-id" placeholder="Selected dataset ID" value="" />
                        </label>
                        <label>
                            <span>"Report ID"</span>
                            <input id="report-id" placeholder="Selected report ID" value="" />
                        </label>
                        <label>
                            <span>"Aggregation ID"</span>
                            <input id="aggregation-id" placeholder="Selected aggregation ID" value="" />
                        </label>
                        <label>
                            <span>"Form ID"</span>
                            <input id="form-id" placeholder="Report form context" value="" />
                        </label>
                        <label class="wide-field">
                            <span>"Report bindings JSON"</span>
                            <input id="report-fields-json" placeholder="Loaded report bindings" value="" />
                        </label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn DashboardPreviewScreen() -> impl IntoView {
    view! {
        <section id="dashboard-viewer-screen" class="app-screen">
            <p class="eyebrow">"Dashboards Screen"</p>
            <h2>"Dashboard Viewer"</h2>
            <p class="muted">
                "Choose a dashboard and preview each component with its current report rows."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Preview Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="loadDashboards()">"Browse Dashboards"</button>
                        <button type="button" onclick="refreshAnalyticsAndOpenDashboard()">"Refresh and Open Dashboard"</button>
                        <button type="button" onclick="loadDashboardById()">"View Dashboard"</button>
                        <button type="button" onclick="loadCharts()">"Browse Charts"</button>
                        <button type="button" onclick="loadChartDefinitionById()">"Inspect Chart"</button>
                        <button type="button" onclick="loadAggregations()">"Browse Aggregations"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>"Current Preview Context"</h3>
                    <div class="inputs compact-inputs">
                        <label>
                            <span>"Dashboard ID"</span>
                            <input id="dashboard-id" placeholder="Selected dashboard ID" value="" />
                        </label>
                        <label>
                            <span>"Chart ID"</span>
                            <input id="chart-id" placeholder="Selected chart ID" value="" />
                        </label>
                        <label>
                            <span>"Aggregation ID"</span>
                            <input id="aggregation-id" placeholder="Selected aggregation ID" value="" />
                        </label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn SubmissionScreen() -> impl IntoView {
    view! {
        <section id="response-entry-screen" class="app-screen">
            <p class="eyebrow">"Responses Screen"</p>
            <h2>"Response Entry"</h2>
            <p class="muted">
                "Pick a published form and target node, render the form, create a draft, save values, and submit."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Response Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="loadPublishedForms()">"Browse Published Forms"</button>
                        <button type="button" onclick="loadNodes()">"Browse Target Nodes"</button>
                        <button type="button" onclick="useSelectedTargetNodeAndContinue()">"Use Current Target"</button>
                        <button type="button" onclick="openSelectedFormVersion()">"Open Current Form"</button>
                        <button type="button" onclick="renderForm(inputValue('form-version-id'))">"View Form"</button>
                        <button type="button" onclick="createDraft()">"Create Draft"</button>
                        <button type="button" onclick="saveRenderedFormValues()">"Save Values"</button>
                        <button type="button" onclick="submitDraft()">"Submit"</button>
                        <button type="button" onclick="discardDraft()">"Discard Draft"</button>
                        <button type="button" onclick="clearResponseContext()">"Clear Response Context"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>"Current Response Context"</h3>
                    <div class="inputs compact-inputs">
                        <label>
                            <span>"Node search"</span>
                            <input id="node-search" placeholder="Search target nodes" value="" />
                        </label>
                        <label>
                            <span>"Target node ID"</span>
                            <input id="node-id" placeholder="Selected node ID" value="" />
                        </label>
                        <label>
                            <span>"Published form version ID"</span>
                            <input id="form-version-id" placeholder="Selected form version ID" value="" />
                        </label>
                        <label>
                            <span>"Form ID"</span>
                            <input id="form-id" placeholder="Selected form ID" value="" />
                        </label>
                        <label>
                            <span>"Draft submission ID"</span>
                            <input id="submission-id" placeholder="Draft submission ID" value="" />
                        </label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn ReviewScreen() -> impl IntoView {
    view! {
        <section id="response-review-screen" class="app-screen">
            <p class="eyebrow">"Responses Screen"</p>
            <h2>"Response Review"</h2>
            <p class="muted">
                "Inspect saved and submitted responses with their audit trail."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Review Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="loadSubmissions()">"Load Submissions"</button>
                        <button type="button" onclick="showDraftSubmissions()">"Show Drafts"</button>
                        <button type="button" onclick="showSubmittedSubmissions()">"Show Submitted"</button>
                        <button type="button" onclick="clearSubmissionReviewFilters()">"Clear Review Filters"</button>
                        <button type="button" onclick="loadSubmissionById()">"Review Selected Response"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>"Review Filters"</h3>
                    <div class="inputs compact-inputs">
                        <label>
                            <span>"Submission search"</span>
                            <input id="submission-search" placeholder="Search form, node, or version" value="" />
                        </label>
                        <label>
                            <span>"Submission status filter"</span>
                            <input id="submission-status-filter" placeholder="draft or submitted" value="" />
                        </label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn ReportScreen() -> impl IntoView {
    view! {
        <section id="response-report-screen" class="app-screen">
            <p class="eyebrow">"Responses Screen"</p>
            <h2>"Response Reports"</h2>
            <p class="muted">
                "Refresh analytics and run table reports against submitted data."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Report Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="refreshAnalytics()">"Refresh Analytics"</button>
                        <button type="button" onclick="loadReports()">"Browse Reports"</button>
                        <button type="button" onclick="refreshAnalyticsAndRunReport()">"Refresh and Run Report"</button>
                        <button type="button" onclick="loadReportById()">"Run Selected Report"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>"Current Report Context"</h3>
                    <div class="inputs compact-inputs">
                        <label>
                            <span>"Report ID"</span>
                            <input id="report-id" placeholder="Selected report ID" value="" />
                        </label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn HierarchyAdminScreen(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    action_title: &'static str,
    context_title: &'static str,
) -> impl IntoView {
    view! {
        <section id="organization-setup-screen" class="app-screen">
            <p class="eyebrow">{eyebrow}</p>
            <h2>{title}</h2>
            <p class="muted">{description}</p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>{action_title}</h3>
                    <p class="muted">"Inspect structure first, then create or update the selected type, relationship, metadata field, or node."</p>
                    <div class="actions">
                        <button type="button" onclick="loadNodeTypes()">"Load Node Types"</button>
                        <button type="button" onclick="loadNodeTypeById()">"Inspect Node Type"</button>
                        <button type="button" onclick="createNodeType()">"Create Node Type"</button>
                        <button type="button" onclick="updateNodeType()">"Update Node Type"</button>
                        <button type="button" onclick="useSelectedNodeTypeAsFormScope()">"Use Node Type As Form Scope"</button>
                        <button type="button" onclick="useSelectedNodeTypeAsMetadataTarget()">"Use Node Type As Metadata Target"</button>
                        <button type="button" onclick="loadRelationships()">"Load Relationships"</button>
                        <button type="button" onclick="createRelationship()">"Create Relationship"</button>
                        <button type="button" onclick="deleteRelationship()">"Remove Relationship"</button>
                        <button type="button" onclick="loadMetadataFields()">"Load Metadata Fields"</button>
                        <button type="button" onclick="createMetadataField()">"Create Metadata Field"</button>
                        <button type="button" onclick="updateMetadataField()">"Update Metadata Field"</button>
                        <button type="button" onclick="loadNodes()">"Load Nodes"</button>
                        <button type="button" onclick="createNode()">"Create Node"</button>
                        <button type="button" onclick="updateNode()">"Update Node"</button>
                        <button type="button" onclick="loadNodes()">"Choose Node To Edit"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>{context_title}</h3>
                    <div class="inputs compact-inputs">
                        <label><span>"Node type name"</span><input id="node-type-name" placeholder="Organization" value="" /></label>
                        <label><span>"Node type slug"</span><input id="node-type-slug" placeholder="organization" value="" /></label>
                        <label><span>"Node type ID"</span><input id="node-type-id" placeholder="Selected node type ID" value="" /></label>
                        <label><span>"Parent node type ID"</span><input id="parent-node-type-id" placeholder="Relationship parent type ID" value="" /></label>
                        <label><span>"Child node type ID"</span><input id="child-node-type-id" placeholder="Relationship child type ID" value="" /></label>
                        <label><span>"Metadata node type ID"</span><input id="metadata-node-type-id" placeholder="Metadata node type ID" value="" /></label>
                        <label><span>"Metadata field ID"</span><input id="metadata-field-id" placeholder="Selected metadata field ID" value="" /></label>
                        <label><span>"Metadata key"</span><input id="metadata-key" placeholder="region" value="region" /></label>
                        <label><span>"Metadata label"</span><input id="metadata-label" placeholder="Region" value="Region" /></label>
                        <label><span>"Metadata field type"</span><input id="metadata-field-type" placeholder="text" value="text" /></label>
                        <label><span>"Metadata required"</span><input id="metadata-required" placeholder="true or false" value="false" /></label>
                        <label><span>"Parent node ID"</span><input id="parent-node-id" placeholder="Optional parent node ID" value="" /></label>
                        <label><span>"Node name"</span><input id="node-name" placeholder="Local Organization" value="Local Organization" /></label>
                        <label class="wide-field"><span>"Node metadata JSON"</span><input id="node-metadata-json" placeholder="{\"region\":\"North\"}" value="{\"region\":\"North\"}" /></label>
                        <label><span>"Node search"</span><input id="node-search" placeholder="Search nodes" value="" /></label>
                        <label><span>"Node ID"</span><input id="node-id" placeholder="Selected node ID" value="" /></label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn FormAdminScreen(
    eyebrow: &'static str,
    title: &'static str,
    description: &'static str,
    action_title: &'static str,
    context_title: &'static str,
) -> impl IntoView {
    view! {
        <section id="forms-configuration-screen" class="app-screen">
            <p class="eyebrow">{eyebrow}</p>
            <h2>{title}</h2>
            <p class="muted">{description}</p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>{action_title}</h3>
                    <p class="muted">"Choose a form first, then use the selected version, section, and field context to shape the draft."</p>
                    <div class="actions">
                        <button type="button" onclick="loadForms()">"Load Forms"</button>
                        <button type="button" onclick="loadFormById()">"Inspect Form"</button>
                        <button type="button" onclick="createForm()">"Create Form"</button>
                        <button type="button" onclick="updateForm()">"Update Form"</button>
                        <button type="button" onclick="createFormVersion()">"Create Version"</button>
                        <button type="button" onclick="createBasicFormVersion()">"Create Basic Version"</button>
                        <button type="button" onclick="createSection()">"Create Section"</button>
                        <button type="button" onclick="updateSection()">"Update Section"</button>
                        <button type="button" onclick="deleteSection()">"Remove Section"</button>
                        <button type="button" onclick="createField()">"Create Field"</button>
                        <button type="button" onclick="updateField()">"Update Field"</button>
                        <button type="button" onclick="deleteField()">"Remove Field"</button>
                        <button type="button" onclick="publishVersion()">"Publish Version"</button>
                        <button type="button" onclick="publishAndPreviewVersion()">"Publish and Preview Version"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>{context_title}</h3>
                    <div class="inputs compact-inputs">
                        <label><span>"Form name"</span><input id="form-name" placeholder="Monthly Report" value="" /></label>
                        <label><span>"Form slug"</span><input id="form-slug" placeholder="monthly-report" value="" /></label>
                        <label><span>"Scope node type ID"</span><input id="form-scope-node-type-id" placeholder="Optional scope node type ID" value="" /></label>
                        <label><span>"Form ID"</span><input id="form-id" placeholder="Selected form ID" value="" /></label>
                        <label><span>"Version label"</span><input id="form-version-label" placeholder="v1" value="v1" /></label>
                        <label><span>"Compatibility group"</span><input id="compatibility-group-name" placeholder="Default compatibility" value="Default compatibility" /></label>
                        <label><span>"Form version ID"</span><input id="form-version-id" placeholder="Selected form version ID" value="" /></label>
                        <label><span>"Section ID"</span><input id="section-id" placeholder="Selected section ID" value="" /></label>
                        <label><span>"Section title"</span><input id="section-title" placeholder="Main" value="Main" /></label>
                        <label><span>"Section position"</span><input id="section-position" placeholder="0" value="0" /></label>
                        <label><span>"Field ID"</span><input id="field-id" placeholder="Selected field ID" value="" /></label>
                        <label><span>"Field key"</span><input id="field-key" placeholder="participants" value="participants" /></label>
                        <label><span>"Field label"</span><input id="field-label" placeholder="Participants" value="Participants" /></label>
                        <label><span>"Field type"</span><input id="field-type" placeholder="number" value="number" /></label>
                        <label><span>"Field required"</span><input id="field-required" placeholder="true or false" value="true" /></label>
                        <label><span>"Field position"</span><input id="field-position" placeholder="0" value="0" /></label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn ReportAdminScreen() -> impl IntoView {
    view! {
        <section id="reporting-configuration-screen" class="app-screen">
            <p class="eyebrow">"Administration Screen"</p>
            <h2>"Reporting Configuration"</h2>
            <p class="muted">
                "Build table report bindings from selected form fields and inspect report output."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Reporting Asset Actions"</h3>
                    <p class="muted">"Use the selected dataset, report, aggregation, chart, and dashboard context to build reporting assets in sequence."</p>
                    <div class="actions">
                        <button type="button" onclick="addDatasetSource()">"Add Dataset Source"</button>
                        <button type="button" onclick="removeSelectedDatasetSource()">"Remove Dataset Source"</button>
                        <button type="button" onclick="clearDatasetSources()">"Clear Dataset Sources"</button>
                        <button type="button" onclick="addDatasetField()">"Add Dataset Field"</button>
                        <button type="button" onclick="removeSelectedDatasetField()">"Remove Dataset Field"</button>
                        <button type="button" onclick="clearDatasetFields()">"Clear Dataset Fields"</button>
                        <button type="button" onclick="renderDatasetDraft()">"Review Dataset Draft"</button>
                        <button type="button" onclick="createDataset()">"Create Dataset"</button>
                        <button type="button" onclick="updateDataset()">"Update Dataset"</button>
                        <button type="button" onclick="deleteDataset()">"Remove Dataset"</button>
                        <button type="button" onclick="loadDatasets()">"Load Datasets"</button>
                        <button type="button" onclick="loadDatasetById()">"Inspect Dataset"</button>
                        <button type="button" onclick="loadDatasetTableById()">"Run Dataset"</button>
                        <button type="button" onclick="addReportBinding()">"Add Binding"</button>
                        <button type="button" onclick="removeSelectedReportBinding()">"Remove Selected Binding"</button>
                        <button type="button" onclick="clearReportBindings()">"Clear Bindings"</button>
                        <button type="button" onclick="createReport()">"Create Report"</button>
                        <button type="button" onclick="updateReport()">"Update Report"</button>
                        <button type="button" onclick="deleteReport()">"Remove Report"</button>
                        <button type="button" onclick="loadReports()">"Load Reports"</button>
                        <button type="button" onclick="loadReportDefinitionById()">"Inspect Report"</button>
                        <button type="button" onclick="refreshAnalyticsAndRunReport()">"Refresh and Run Report"</button>
                        <button type="button" onclick="loadReportById()">"Run Report"</button>
                        <button type="button" onclick="createAggregation()">"Create Aggregation"</button>
                        <button type="button" onclick="loadAggregations()">"Load Aggregations"</button>
                        <button type="button" onclick="loadAggregationDefinitionById()">"Inspect Aggregation"</button>
                        <button type="button" onclick="updateAggregation()">"Update Aggregation"</button>
                        <button type="button" onclick="deleteAggregation()">"Remove Aggregation"</button>
                        <button type="button" onclick="loadAggregationById()">"Run Aggregation"</button>
                        <button type="button" onclick="createChart()">"Create Chart"</button>
                        <button type="button" onclick="updateChart()">"Update Chart"</button>
                        <button type="button" onclick="deleteChart()">"Remove Chart"</button>
                        <button type="button" onclick="loadCharts()">"Load Charts"</button>
                        <button type="button" onclick="createDashboard()">"Create Dashboard"</button>
                        <button type="button" onclick="updateDashboard()">"Update Dashboard"</button>
                        <button type="button" onclick="deleteDashboard()">"Remove Dashboard"</button>
                        <button type="button" onclick="addDashboardComponent()">"Add Component"</button>
                        <button type="button" onclick="updateDashboardComponent()">"Update Component"</button>
                        <button type="button" onclick="deleteDashboardComponent()">"Remove Component"</button>
                        <button type="button" onclick="refreshAnalyticsAndOpenDashboard()">"Refresh and Open Dashboard"</button>
                        <button type="button" onclick="loadDashboardById()">"Load Dashboard"</button>
                    </div>
                </section>
                <section class="task-panel context-panel">
                    <h3>"Current Reporting Configuration Context"</h3>
                    <div class="inputs compact-inputs">
                <label><span>"Dataset name"</span><input id="dataset-name" placeholder="Participant Dataset" value="Participant Dataset" /></label>
                <label><span>"Dataset slug"</span><input id="dataset-slug" placeholder="participant-dataset" value="participant-dataset" /></label>
                <label><span>"Dataset grain"</span><input id="dataset-grain" placeholder="submission" value="submission" /></label>
                <label><span>"Dataset composition mode"</span><input id="dataset-composition-mode" placeholder="union" value="union" /></label>
                <label><span>"Dataset ID"</span><input id="dataset-id" placeholder="Selected dataset ID" value="" /></label>
                <label><span>"Dataset source alias"</span><input id="dataset-source-alias" placeholder="service" value="service" /></label>
                <label><span>"Dataset form ID"</span><input id="dataset-form-id" placeholder="Selected source form ID" value="" /></label>
                <label><span>"Dataset compatibility group ID"</span><input id="dataset-compatibility-group-id" placeholder="Optional compatibility group ID" value="" /></label>
                <label><span>"Dataset selection rule"</span><input id="dataset-selection-rule" placeholder="all" value="all" /></label>
                <label><span>"Dataset field key"</span><input id="dataset-field-key" placeholder="participant_count" value="participant_count" /></label>
                <label><span>"Dataset field label"</span><input id="dataset-field-label" placeholder="Participant Count" value="Participant Count" /></label>
                <label><span>"Dataset source field key"</span><input id="dataset-source-field-key" placeholder="participants" value="participants" /></label>
                <label><span>"Dataset field type"</span><input id="dataset-field-type" placeholder="number" value="number" /></label>
                <label><span>"Report name"</span><input id="report-name" placeholder="Participants Report" value="Participants Report" /></label>
                <label><span>"Report logical key"</span><input id="report-logical-key" placeholder="participants" value="participants" /></label>
                <label><span>"Report source field key"</span><input id="report-source-field-key" placeholder="participants" value="participants" /></label>
                <label><span>"Report computed expression"</span><input id="report-computed-expression" placeholder="literal:Submitted" value="" /></label>
                <label><span>"Report missing-data policy"</span><input id="report-missing-policy" placeholder="null" value="null" /></label>
                <label><span>"Report bindings JSON"</span><input id="report-fields-json" placeholder="Optional bindings JSON" value="" /></label>
                <label><span>"Report ID"</span><input id="report-id" placeholder="Selected report ID" value="" /></label>
                <label><span>"Aggregation ID"</span><input id="aggregation-id" placeholder="Selected aggregation ID" value="" /></label>
                <label><span>"Aggregation name"</span><input id="aggregation-name" placeholder="Participants Totals" value="Participants Totals" /></label>
                <label><span>"Aggregation group-by logical key"</span><input id="aggregation-group-by-logical-key" placeholder="Optional group logical key" value="" /></label>
                <label><span>"Aggregation metric key"</span><input id="aggregation-metric-key" placeholder="participants_total" value="participants_total" /></label>
                <label><span>"Aggregation source logical key"</span><input id="aggregation-source-logical-key" placeholder="participants" value="participants" /></label>
                <label><span>"Aggregation metric kind"</span><input id="aggregation-metric-kind" placeholder="count, sum, avg, min, or max" value="sum" /></label>
                <label><span>"Chart ID"</span><input id="chart-id" placeholder="Selected chart ID" value="" /></label>
                <label><span>"Chart name"</span><input id="chart-name" placeholder="Participants Table" value="Participants Table" /></label>
                <label><span>"Chart type"</span><input id="chart-type" placeholder="table" value="table" /></label>
                <label><span>"Dashboard ID"</span><input id="dashboard-id" placeholder="Selected dashboard ID" value="" /></label>
                <label><span>"Dashboard name"</span><input id="dashboard-name" placeholder="Local Dashboard" value="Local Dashboard" /></label>
                <label><span>"Dashboard component ID"</span><input id="dashboard-component-id" placeholder="Selected dashboard component ID" value="" /></label>
                <label><span>"Dashboard component position"</span><input id="dashboard-component-position" placeholder="0" value="0" /></label>
                <label><span>"Dashboard component title"</span><input id="dashboard-component-title" placeholder="Chart title" value="" /></label>
                <label class="wide-field"><span>"Dashboard component config JSON"</span><input id="dashboard-component-config-json" placeholder="{\"title\":\"Chart\"}" value="" /></label>
                    </div>
                </section>
            </div>
        </section>
    }
}

#[component]
fn FixtureScreen() -> impl IntoView {
    view! {
        <section id="migration-fixture-screen" class="app-screen">
            <p class="eyebrow">"Migration Screen"</p>
            <h2>"Fixture Intake and Validation"</h2>
            <p class="muted">
                "Load a bundled fixture or paste fixture JSON, then validate or dry-run before import rehearsal."
            </p>
            <div class="inputs">
                <label>
                    <span>"Legacy fixture JSON"</span>
                    <textarea
                        id="legacy-fixture-json"
                        placeholder="Paste legacy fixture JSON"
                    ></textarea>
                </label>
            </div>
            <div class="actions">
                <button type="button" onclick="loadLegacyFixtureExamples()">"Load Fixture Examples"</button>
                <button type="button" onclick="validateLegacyFixture()">"Validate Fixture"</button>
                <button type="button" onclick="dryRunLegacyFixture()">"Dry-Run Fixture"</button>
                <button type="button" onclick="importLegacyFixture()">"Import Fixture"</button>
            </div>
        </section>
    }
}
