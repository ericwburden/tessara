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

#[component]
fn HomeApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <span>"Home"</span>
                </nav>
                <p class="muted">"Shared Home"</p>
                <h1>"Application Overview"</h1>
                <p>
                    "This shared home is the primary entry point for the migration UI catch-up. "
                    "It exposes the target product areas while keeping the current backend-supported workflows intact."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="logout()">"Log Out"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="home"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <HomeScreen/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn OrganizationApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Organization"</span>
                </nav>
                <p class="muted">"Product Area"</p>
                <h1>"Organization"</h1>
                <p>
                    "This area establishes the operational hierarchy surface. It currently bridges into the existing hierarchy and node workflows without introducing unsupported organization behavior."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="loadNodes()">"Load Nodes"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="organization"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <OrganizationHomeScreen/>
                    <OrganizationWorkspaceShell/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn FormsApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Forms"</span>
                </nav>
                <p class="muted">"Product Area"</p>
                <h1>"Forms"</h1>
                <p>
                    "This area is the canonical entry point for form discovery and lifecycle work. It currently bridges product-facing form access and internal form configuration using existing supported routes."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="loadForms()">"Load Forms"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="forms"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <FormsHomeScreen/>
                    <FormsWorkspaceShell/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn ResponsesApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Responses"</span>
                </nav>
                <p class="muted">"Product Area"</p>
                <h1>"Responses"</h1>
                <p>
                    "This area handles response entry, drafts, submission, and review. It uses the current backend-supported form-render and submission lifecycle without relying on utility-style navigation."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="logout()">"Log Out"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="startDemoSubmissionFlow()">"Start Demo Response"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="responses"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <SubmissionHomeScreen/>
                    <SubmissionWorkspaceShell/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn AdministrationApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Administration"</span>
                </nav>
                <p class="muted">"Internal Area"</p>
                <h1>"Administration"</h1>
                <p>
                    "This internal area is for hierarchy, form, and reporting configuration. It remains visible during the migration, but it is intentionally scoped as an operator surface."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="administration"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <AdminHomeScreen/>
                    <AdminWorkspaceShell/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn MigrationApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Migration"</span>
                </nav>
                <p class="muted">"Internal Area"</p>
                <h1>"Migration Workbench"</h1>
                <p>
                    "This operator screen validates and dry-runs representative legacy fixtures "
                    "before running import rehearsals."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="logout()">"Log Out"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="migration"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <MigrationHomeScreen/>
                    <FixtureScreen/>
                    <section id="result-screen" class="app-screen">
                        <h2>"Validation Results"</h2>
                        <div id="screen" class="cards"></div>
                    </section>
                    <RawOutputPanel/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn ReportsApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Reports"</span>
                </nav>
                <p class="muted">"Product Area"</p>
                <h1>"Reports"</h1>
                <p>
                    "This area is the canonical route for report browsing, report execution, and reporting detail traversal. Dashboard preview remains linked here until the dashboard area is split further."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="logout()">"Log Out"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="openDemoDashboard()">"Open Demo Dashboard"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="reports"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <ReportingHomeScreen/>
                    <ReportingWorkspaceShell/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn DashboardsApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <nav class="breadcrumb-trail" aria-label="Breadcrumb">
                    <a href="/app">"Home"</a>
                    <span>"Dashboards"</span>
                </nav>
                <p class="muted">"Product Area"</p>
                <h1>"Dashboards"</h1>
                <p>
                    "This area is the dashboard viewing destination. It currently uses the supported dashboard preview path while the broader dashboard product surface catches up."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="logout()">"Log Out"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="openDemoDashboard()">"Open Demo Dashboard"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="dashboards"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <DashboardsHomeScreen/>
                    <DashboardsWorkspaceShell/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn BrandLockup() -> impl IntoView {
    view! {
        <div class="brand-lockup">
            <img class="brand-mark" src="/assets/tessara-icon-256.svg" alt="" />
            <span>"Tessara"</span>
        </div>
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
        ("Create Node", "/app/administration#hierarchy-admin-screen"),
        ("Create Form", "/app/forms#form-admin-screen"),
        ("Create Dataset", "/app/administration#report-admin-screen"),
        ("Create Report", "/app/administration#report-admin-screen"),
        (
            "Create Aggregation",
            "/app/administration#report-admin-screen",
        ),
        (
            "Create Dashboard",
            "/app/administration#report-admin-screen",
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
            "Open Organization",
        ),
        (
            "Forms",
            "Browse form definitions and move into the supported form lifecycle and publishing surfaces.",
            "/app/forms",
            "Open Forms",
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
        (
            "Browse Nodes",
            "Load the current runtime nodes and move through the operational hierarchy.",
            "#hierarchy-admin-screen",
            "Open Organization Tasks",
            "loadNodes()",
            "Load Nodes",
        ),
        (
            "Inspect Node Types",
            "Review the configured hierarchy structure and labels behind the organization area.",
            "#hierarchy-admin-screen",
            "Open Structure",
            "loadNodeTypes()",
            "Load Node Types",
        ),
        (
            "Open Forms",
            "Move from organization browsing into the scoped forms area.",
            "/app/forms",
            "Open Forms",
            "loadForms()",
            "Load Forms",
        ),
        (
            "Open Dashboards",
            "Move from organization browsing into current dashboard viewing surfaces.",
            "/app/dashboards",
            "Open Dashboards",
            "loadDashboards()",
            "Load Dashboards",
        ),
    ];

    view! {
        <section id="organization-home-screen" class="app-screen">
            <p class="eyebrow">"Organization Home"</p>
            <h2>"Organization Areas"</h2>
            <p class="muted">
                "This route is the structural bridge from the legacy partner/program model into Tessara's configurable hierarchy."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn FormsHomeScreen() -> impl IntoView {
    let management_cards = [
        (
            "Browse Forms",
            "Open the current forms directory and inspect configured forms and versions.",
            "#form-admin-screen",
            "Open Form Tasks",
            "loadForms()",
            "Load Forms",
        ),
        (
            "Published Response Path",
            "Move into the response workflow for published form completion and review.",
            "/app/responses",
            "Open Responses",
            "loadForms()",
            "Load Forms",
        ),
        (
            "Open Organization",
            "Return to the organization area for scoped navigation into forms.",
            "/app/organization",
            "Open Organization",
            "loadNodeTypes()",
            "Load Node Types",
        ),
        (
            "Open Administration",
            "Use the internal configuration surface for full hierarchy and reporting setup.",
            "/app/administration",
            "Open Administration",
            "loadForms()",
            "Load Forms",
        ),
    ];

    view! {
        <section id="forms-home-screen" class="app-screen">
            <p class="eyebrow">"Forms Home"</p>
            <h2>"Forms Areas"</h2>
            <p class="muted">
                "This route is the product-facing entry into form discovery, version awareness, and supported form lifecycle tasks."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn AdminHomeScreen() -> impl IntoView {
    let management_cards = [
        (
            "Hierarchy",
            "Manage node types, relationships, metadata fields, and runtime nodes.",
            "#hierarchy-admin-screen",
            "Open Hierarchy Setup",
            "loadNodeTypes()",
            "Load Node Types",
        ),
        (
            "Forms",
            "Create forms, draft versions, edit sections and fields, and publish revisions.",
            "#form-admin-screen",
            "Open Form Builder",
            "loadForms()",
            "Load Forms",
        ),
        (
            "Datasets and Reports",
            "Manage datasets, reports, and aggregations inside the reporting stack.",
            "#report-admin-screen",
            "Open Reporting Builder",
            "loadDatasets()",
            "Load Datasets",
        ),
        (
            "Dashboards",
            "Inspect charts, dashboards, and current preview outputs from one admin route.",
            "#report-admin-screen",
            "Open Dashboard Builder",
            "loadDashboards()",
            "Load Dashboards",
        ),
    ];

    let directory_cards = [
        (
            "Node Types",
            "Browse hierarchy types",
            "loadNodeTypes()",
            "Open",
        ),
        ("Nodes", "Browse runtime nodes", "loadNodes()", "Open"),
        ("Forms", "Browse forms and versions", "loadForms()", "Open"),
        (
            "Datasets",
            "Browse dataset definitions",
            "loadDatasets()",
            "Open",
        ),
        (
            "Reports",
            "Browse report definitions",
            "loadReports()",
            "Open",
        ),
        (
            "Aggregations",
            "Browse aggregation definitions",
            "loadAggregations()",
            "Open",
        ),
        ("Charts", "Browse charts", "loadCharts()", "Open"),
        (
            "Dashboards",
            "Browse dashboards",
            "loadDashboards()",
            "Open",
        ),
    ];

    view! {
        <section id="admin-home-screen" class="app-screen">
            <p class="eyebrow">"Admin Home"</p>
            <h2>"Management Areas"</h2>
            <p class="muted">
                "Use this admin landing section to jump into the main management areas before dropping into the detailed builder screens."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Admin Home"</p>
            <h2>"Entity Directory"</h2>
            <p class="muted">
                "These entry points mirror the original application's core management lists while keeping the current Tessara builder controls underneath."
            </p>
            <div class="directory-grid">
                {directory_cards
                    .into_iter()
                    .map(|(title, description, action, label)| {
                        view! {
                            <article class="directory-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <button type="button" onclick=action>{label}</button>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn SubmissionHomeScreen() -> impl IntoView {
    let management_cards = [
        (
            "Start a Response",
            "Choose a published form and target node, then open the form for draft entry.",
            "#submission-screen",
            "Open Response Entry",
            "loadPublishedForms()",
            "Load Published Forms",
        ),
        (
            "Choose a Target",
            "Browse nodes and carry the selected target directly into the response flow.",
            "#submission-screen",
            "Open Target Selection",
            "loadNodes()",
            "Load Target Nodes",
        ),
        (
            "Review Responses",
            "Browse draft and submitted responses, then reopen the selected submission in context.",
            "#review-screen",
            "Open Response Review",
            "loadSubmissions()",
            "Load Submissions",
        ),
        (
            "Open Related Reports",
            "Jump from the submission route into supporting report output while reviewing responses.",
            "#report-screen",
            "Open Related Reports",
            "loadReports()",
            "Load Reports",
        ),
    ];

    let directory_cards = [
        (
            "Published Forms",
            "Browse current published forms",
            "loadPublishedForms()",
            "Open",
        ),
        (
            "Target Nodes",
            "Browse submission targets",
            "loadNodes()",
            "Open",
        ),
        (
            "Draft Responses",
            "Filter to draft submissions",
            "showDraftSubmissions()",
            "Open",
        ),
        (
            "Submitted Responses",
            "Filter to submitted responses",
            "showSubmittedSubmissions()",
            "Open",
        ),
        (
            "All Responses",
            "Browse the full response list",
            "loadSubmissions()",
            "Open",
        ),
        ("Reports", "Browse related reports", "loadReports()", "Open"),
    ];

    view! {
        <section id="submission-home-screen" class="app-screen">
            <p class="eyebrow">"Responses Home"</p>
            <h2>"Response Stages"</h2>
            <p class="muted">
                "Use this route-level landing section to move between response entry, target selection, review, and related reporting without relying on one long stacked screen."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Responses Home"</p>
            <h2>"Response Directory"</h2>
            <p class="muted">
                "These entry points keep submissions aligned with the application shell by emphasizing common lists and review paths over raw-ID entry."
            </p>
            <div class="directory-grid">
                {directory_cards
                    .into_iter()
                    .map(|(title, description, action, label)| {
                        view! {
                            <article class="directory-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <button type="button" onclick=action>{label}</button>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn OrganizationWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        (
            "Runtime Nodes",
            "Browse current nodes and inspect operational hierarchy records.",
            "loadNodes()",
            "Open Nodes",
        ),
        (
            "Structure Types",
            "Review node types, relationships, and metadata definitions.",
            "loadNodeTypes()",
            "Open Structure",
        ),
        (
            "Forms Bridge",
            "Move from organization structure into the current forms area.",
            "",
            "Open Forms",
        ),
        (
            "Dashboards Bridge",
            "Move from organization structure into the current dashboards area.",
            "",
            "Open Dashboards",
        ),
    ];

    view! {
        <section class="app-screen organization-workspace-shell">
            <p class="eyebrow">"Organization Workspace"</p>
            <h2>"Organization Console"</h2>
            <p class="muted">
                "This route is the first organization-area bridge. It keeps hierarchy work discoverable now while later sprints replace more of the internal builder feel with directory and detail flows."
            </p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>"Organization Queues"</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|(title, description, action, label)| {
                                    let action_view = if action.is_empty() {
                                        if label == "Open Forms" {
                                            view! { <a class="button-link" href="/app/forms">{label}</a> }.into_any()
                                        } else {
                                            view! { <a class="button-link" href="/app/dashboards">{label}</a> }.into_any()
                                        }
                                    } else {
                                        view! { <button type="button" onclick=action>{label}</button> }.into_any()
                                    };
                                    view! {
                                        <article class="workspace-card">
                                            <h4>{title}</h4>
                                            <p>{description}</p>
                                            {action_view}
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>"Organization Path"</h3>
                        <ol class="app-list">
                            <li>"Browse nodes and hierarchy structure."</li>
                            <li>"Inspect the configured labels and relationships."</li>
                            <li>"Move into forms, responses, or dashboards from the scoped structure."</li>
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">
                    <HierarchyAdminScreen/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn FormsWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        (
            "Forms Directory",
            "Browse current form records and inspect definitions.",
            "loadForms()",
            "Open Forms",
        ),
        (
            "Response Bridge",
            "Move from form discovery into the supported responses area.",
            "",
            "Open Responses",
        ),
        (
            "Organization Bridge",
            "Return to the organization surface for scoped form navigation.",
            "",
            "Open Organization",
        ),
    ];

    view! {
        <section class="app-screen forms-workspace-shell">
            <p class="eyebrow">"Forms Workspace"</p>
            <h2>"Forms Console"</h2>
            <p class="muted">
                "This route is the current bridge between product-facing form discovery and the supported internal form lifecycle tasks."
            </p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>"Forms Queues"</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|(title, description, action, label)| {
                                    let action_view = if action.is_empty() {
                                        if label == "Open Responses" {
                                            view! { <a class="button-link" href="/app/responses">{label}</a> }.into_any()
                                        } else {
                                            view! { <a class="button-link" href="/app/organization">{label}</a> }.into_any()
                                        }
                                    } else {
                                        view! { <button type="button" onclick=action>{label}</button> }.into_any()
                                    };
                                    view! {
                                        <article class="workspace-card">
                                            <h4>{title}</h4>
                                            <p>{description}</p>
                                            {action_view}
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>"Forms Path"</h3>
                        <ol class="app-list">
                            <li>"Browse or inspect the form."</li>
                            <li>"Choose the relevant version or draft."</li>
                            <li>"Move into response entry or internal configuration as needed."</li>
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">
                    <FormAdminScreen/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn SubmissionWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        (
            "Published Forms",
            "Load the current published response options.",
            "loadPublishedForms()",
            "Open Forms",
        ),
        (
            "Target Directory",
            "Browse organizations, programs, and other submission targets.",
            "loadNodes()",
            "Open Targets",
        ),
        (
            "Draft Queue",
            "Review in-progress drafts that still need edits or submission.",
            "showDraftSubmissions()",
            "Open Drafts",
        ),
        (
            "Submitted Queue",
            "Review completed responses and continue into reporting.",
            "showSubmittedSubmissions()",
            "Open Submitted",
        ),
    ];

    view! {
        <section class="app-screen submission-workspace-shell">
            <p class="eyebrow">"Responses Workspace"</p>
            <h2>"Response Console"</h2>
            <p class="muted">
                "This route now acts as an application workspace: the left side focuses on queues and entry points, while the right side carries the active response, review, and reporting surfaces."
            </p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>"Response Queues"</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|(title, description, action, label)| {
                                    view! {
                                        <article class="workspace-card">
                                            <h4>{title}</h4>
                                            <p>{description}</p>
                                            <button type="button" onclick=action>{label}</button>
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>"Guided Path"</h3>
                        <ol class="app-list">
                            <li>"Choose a published form."</li>
                            <li>"Choose the target node."</li>
                            <li>"Open the response form and create a draft."</li>
                            <li>"Save values, submit, then review the resulting record."</li>
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">
                    <SubmissionScreen/>
                    <ReviewScreen/>
                    <ReportScreen/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn AdminWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        (
            "Hierarchy Types",
            "Open node-type and relationship management for structural changes.",
            "loadNodeTypes()",
            "Open Hierarchy",
        ),
        (
            "Forms Directory",
            "Browse forms, versions, and publishing status from the main admin route.",
            "loadForms()",
            "Open Forms",
        ),
        (
            "Reporting Assets",
            "Open datasets, reports, aggregations, charts, and dashboards.",
            "loadDatasets()",
            "Open Reporting",
        ),
        (
            "Runtime Nodes",
            "Browse and update real nodes without leaving the admin workspace.",
            "loadNodes()",
            "Open Nodes",
        ),
    ];

    view! {
        <section class="app-screen admin-workspace-shell">
            <p class="eyebrow">"Admin Workspace"</p>
            <h2>"Configuration Console"</h2>
            <p class="muted">
                "This route is now shifting from a builder stack toward an admin workspace. The rail keeps high-level management queues visible while the main area holds hierarchy, form, and reporting configuration."
            </p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>"Management Queues"</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|(title, description, action, label)| {
                                    view! {
                                        <article class="workspace-card">
                                            <h4>{title}</h4>
                                            <p>{description}</p>
                                            <button type="button" onclick=action>{label}</button>
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>"Admin Path"</h3>
                        <ol class="app-list">
                            <li>"Set or inspect hierarchy types and runtime nodes."</li>
                            <li>"Open the correct form and version draft."</li>
                            <li>"Publish or review reporting assets tied to that structure."</li>
                            <li>"Confirm the resulting dashboards and reporting surfaces."</li>
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">
                    <HierarchyAdminScreen/>
                    <FormAdminScreen/>
                    <ReportAdminScreen/>
                </div>
            </div>
        </section>
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
        (
            "Datasets",
            "Inspect dataset definitions and run source-aware dataset previews before binding reports.",
            "#report-runner-screen",
            "Open Dataset Workflows",
            "loadDatasets()",
            "Load Datasets",
        ),
        (
            "Reports",
            "Inspect report definitions, refresh analytics, and execute table-style outputs.",
            "#report-runner-screen",
            "Open Report Runner",
            "loadReports()",
            "Load Reports",
        ),
        (
            "Aggregations",
            "Review aggregation definitions and execute grouped metrics on current report outputs.",
            "#report-runner-screen",
            "Open Aggregations",
            "loadAggregations()",
            "Load Aggregations",
        ),
        (
            "Dashboards",
            "Preview charts and dashboards with current report or aggregation context.",
            "#dashboard-preview-screen",
            "Open Dashboard Preview",
            "loadDashboards()",
            "Load Dashboards",
        ),
    ];

    let directory_cards = [
        (
            "Datasets",
            "Browse dataset definitions",
            "loadDatasets()",
            "Open",
        ),
        (
            "Reports",
            "Browse report definitions",
            "loadReports()",
            "Open",
        ),
        (
            "Aggregations",
            "Browse aggregation definitions",
            "loadAggregations()",
            "Open",
        ),
        ("Charts", "Browse charts", "loadCharts()", "Open"),
        (
            "Dashboards",
            "Browse dashboards",
            "loadDashboards()",
            "Open",
        ),
    ];

    view! {
        <section id="reporting-home-screen" class="app-screen">
            <p class="eyebrow">"Reports Home"</p>
            <h2>"Report Areas"</h2>
            <p class="muted">
                "Use this reporting landing section to move between datasets, reports, aggregations, and dashboards without dropping immediately into builder-style controls."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Reports Home"</p>
            <h2>"Reporting Directory"</h2>
            <p class="muted">
                "These entry points start to replace workbench-only reporting flows with clearer entity lists inside the application shell."
            </p>
            <div class="directory-grid">
                {directory_cards
                    .into_iter()
                    .map(|(title, description, action, label)| {
                        view! {
                            <article class="directory-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <button type="button" onclick=action>{label}</button>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn DashboardsHomeScreen() -> impl IntoView {
    let management_cards = [
        (
            "Open Dashboards",
            "Browse dashboard surfaces and inspect current component previews.",
            "#dashboard-preview-screen",
            "Open Dashboard Viewer",
            "loadDashboards()",
            "Load Dashboards",
        ),
        (
            "Open Charts",
            "Inspect chart definitions that drive dashboard components.",
            "#dashboard-preview-screen",
            "Open Charts",
            "loadCharts()",
            "Load Charts",
        ),
        (
            "Open Reports",
            "Move into the reports area for related report and aggregation detail.",
            "/app/reports",
            "Open Reports",
            "loadReports()",
            "Load Reports",
        ),
        (
            "Open Demo Dashboard",
            "Jump directly into the seeded dashboard preview path.",
            "#dashboard-preview-screen",
            "Open Demo Preview",
            "openDemoDashboard()",
            "Open Demo Dashboard",
        ),
    ];

    view! {
        <section id="dashboards-home-screen" class="app-screen">
            <p class="eyebrow">"Dashboards Home"</p>
            <h2>"Dashboard Areas"</h2>
            <p class="muted">
                "This route separates dashboard viewing from the broader reporting route while still using the current supported preview path."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
    }
}

#[component]
fn ReportingWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        (
            "Datasets",
            "Inspect and run datasets before binding reports or aggregations.",
            "loadDatasets()",
            "Open Datasets",
        ),
        (
            "Reports",
            "Review report definitions, bindings, and current result sets.",
            "loadReports()",
            "Open Reports",
        ),
        (
            "Aggregations",
            "Check grouped metrics and the charts that depend on them.",
            "loadAggregations()",
            "Open Aggregations",
        ),
        (
            "Dashboards",
            "Open dashboard previews and chart context from one reporting route.",
            "loadDashboards()",
            "Open Dashboards",
        ),
    ];

    view! {
        <section class="app-screen reporting-workspace-shell">
            <p class="eyebrow">"Reports Workspace"</p>
            <h2>"Insight Console"</h2>
            <p class="muted">
                "This route now acts more like a reporting workspace: the rail keeps the reporting queues visible while the main area focuses on report execution and dashboard preview."
            </p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>"Reporting Queues"</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|(title, description, action, label)| {
                                    view! {
                                        <article class="workspace-card">
                                            <h4>{title}</h4>
                                            <p>{description}</p>
                                            <button type="button" onclick=action>{label}</button>
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>"Reporting Path"</h3>
                        <ol class="app-list">
                            <li>"Inspect the dataset or report you intend to use."</li>
                            <li>"Refresh analytics if the source data changed."</li>
                            <li>"Run the report or aggregation."</li>
                            <li>"Open the dashboard preview to verify the final surface."</li>
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">
                    <ReportRunnerScreen/>
                    <DashboardPreviewScreen/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn DashboardsWorkspaceShell() -> impl IntoView {
    let queue_cards = [
        (
            "Dashboards",
            "Open dashboard previews and current component layouts.",
            "loadDashboards()",
            "Open Dashboards",
        ),
        (
            "Charts",
            "Inspect chart definitions used by current dashboard components.",
            "loadCharts()",
            "Open Charts",
        ),
        (
            "Reports Bridge",
            "Move to the reports area for report and aggregation detail.",
            "",
            "Open Reports",
        ),
    ];

    view! {
        <section class="app-screen dashboards-workspace-shell">
            <p class="eyebrow">"Dashboards Workspace"</p>
            <h2>"Dashboard Console"</h2>
            <p class="muted">
                "This route keeps dashboard viewing separate from the broader reports area while the dashboard product surface catches up."
            </p>
            <div class="workspace-grid">
                <aside class="workspace-rail">
                    <section class="workspace-panel">
                        <h3>"Dashboard Queues"</h3>
                        <div class="workspace-card-grid">
                            {queue_cards
                                .into_iter()
                                .map(|(title, description, action, label)| {
                                    let action_view = if action.is_empty() {
                                        view! { <a class="button-link" href="/app/reports">{label}</a> }.into_any()
                                    } else {
                                        view! { <button type="button" onclick=action>{label}</button> }.into_any()
                                    };
                                    view! {
                                        <article class="workspace-card">
                                            <h4>{title}</h4>
                                            <p>{description}</p>
                                            {action_view}
                                        </article>
                                    }
                                })
                                .collect_view()}
                        </div>
                    </section>
                    <section class="workspace-panel">
                        <h3>"Dashboard Path"</h3>
                        <ol class="app-list">
                            <li>"Choose the dashboard."</li>
                            <li>"Inspect the current chart-backed components."</li>
                            <li>"Traverse into reports when deeper source detail is needed."</li>
                        </ol>
                    </section>
                </aside>
                <div class="workspace-stack">
                    <DashboardPreviewScreen/>
                </div>
            </div>
        </section>
    }
}

#[component]
fn MigrationHomeScreen() -> impl IntoView {
    let management_cards = [
        (
            "Fixture Intake",
            "Load bundled fixtures or paste fixture JSON to start a migration rehearsal.",
            "#fixture-screen",
            "Open Fixture Intake",
            "loadLegacyFixtureExamples()",
            "Load Fixture Examples",
        ),
        (
            "Validation",
            "Run validation before import so mapping and value problems are visible early.",
            "#fixture-screen",
            "Open Validation",
            "validateLegacyFixture()",
            "Validate Fixture",
        ),
        (
            "Dry Run",
            "Preview what the import would create before mutating the local rehearsal database.",
            "#fixture-screen",
            "Open Dry Run",
            "dryRunLegacyFixture()",
            "Dry-Run Fixture",
        ),
        (
            "Import",
            "Run the import rehearsal and inspect the resulting entities through the app shell.",
            "#result-screen",
            "Open Import Results",
            "importLegacyFixture()",
            "Import Fixture",
        ),
    ];

    let directory_cards = [
        (
            "Fixture Examples",
            "Load bundled fixtures",
            "loadLegacyFixtureExamples()",
            "Open",
        ),
        (
            "Validation Results",
            "Run validation now",
            "validateLegacyFixture()",
            "Open",
        ),
        (
            "Dry Runs",
            "Run a dry-run rehearsal",
            "dryRunLegacyFixture()",
            "Open",
        ),
        (
            "Imports",
            "Run import rehearsal",
            "importLegacyFixture()",
            "Open",
        ),
    ];

    view! {
        <section id="migration-home-screen" class="app-screen">
            <p class="eyebrow">"Migration Home"</p>
            <h2>"Migration Stages"</h2>
            <p class="muted">
                "Use this operator landing section to move through fixture intake, validation, dry run, and import without relying on a single workbench panel."
            </p>
            <div class="management-grid">
                {management_cards
                    .into_iter()
                    .map(|(title, description, href, href_label, action, action_label)| {
                        view! {
                            <article class="home-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <div class="actions">
                                    <a class="button-link" href=href>{href_label}</a>
                                    <button type="button" onclick=action>{action_label}</button>
                                </div>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Migration Home"</p>
            <h2>"Migration Directory"</h2>
            <p class="muted">
                "These entry points keep the migration workflow operator-focused while still fitting inside the shared application shell."
            </p>
            <div class="directory-grid">
                {directory_cards
                    .into_iter()
                    .map(|(title, description, action, label)| {
                        view! {
                            <article class="directory-card">
                                <h3>{title}</h3>
                                <p>{description}</p>
                                <button type="button" onclick=action>{label}</button>
                            </article>
                        }
                    })
                    .collect_view()}
            </div>
        </section>
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
        <section id="report-runner-screen" class="app-screen">
            <p class="eyebrow">"Reporting Screen"</p>
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
                        <button type="button" onclick="loadDatasets()">"Choose Dataset"</button>
                        <button type="button" onclick="loadDatasetDefinitionById()">"Inspect Dataset"</button>
                        <button type="button" onclick="loadDatasetTableById()">"Run Dataset"</button>
                        <button type="button" onclick="loadReports()">"Choose Report"</button>
                        <button type="button" onclick="loadReportDefinitionById()">"Inspect Report"</button>
                        <button type="button" onclick="refreshAnalyticsAndRunReport()">"Refresh and Run Report"</button>
                        <button type="button" onclick="loadReportById()">"Run Report"</button>
                        <button type="button" onclick="loadAggregations()">"Choose Aggregation"</button>
                        <button type="button" onclick="loadAggregationDefinitionById()">"Inspect Aggregation"</button>
                        <button type="button" onclick="loadAggregationById()">"Run Aggregation"</button>
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
        <section id="dashboard-preview-screen" class="app-screen">
            <p class="eyebrow">"Reporting Screen"</p>
            <h2>"Dashboard Preview"</h2>
            <p class="muted">
                "Choose a dashboard and preview each component with its current report rows."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Preview Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="loadDashboards()">"Choose Dashboard"</button>
                        <button type="button" onclick="refreshAnalyticsAndOpenDashboard()">"Refresh and Open Dashboard"</button>
                        <button type="button" onclick="loadDashboardById()">"Open Dashboard"</button>
                        <button type="button" onclick="loadCharts()">"Choose Chart"</button>
                        <button type="button" onclick="loadChartDefinitionById()">"Inspect Chart"</button>
                        <button type="button" onclick="loadAggregations()">"Choose Aggregation"</button>
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
        <section id="submission-screen" class="app-screen">
            <p class="eyebrow">"Application Screen"</p>
            <h2>"Submit Data"</h2>
            <p class="muted">
                "Pick a published form and target node, render the form, create a draft, save values, and submit."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Response Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="loadPublishedForms()">"Choose Published Form"</button>
                        <button type="button" onclick="loadNodes()">"Choose Target Node"</button>
                        <button type="button" onclick="useSelectedTargetNodeAndContinue()">"Use Selected Target"</button>
                        <button type="button" onclick="openSelectedFormVersion()">"Open Selected Form"</button>
                        <button type="button" onclick="renderForm(inputValue('form-version-id'))">"Open Form"</button>
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
        <section id="review-screen" class="app-screen">
            <p class="eyebrow">"Application Screen"</p>
            <h2>"Review Submissions"</h2>
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
                        <button type="button" onclick="loadSubmissionById()">"Open Selected Submission"</button>
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
        <section id="report-screen" class="app-screen">
            <p class="eyebrow">"Application Screen"</p>
            <h2>"View Reports"</h2>
            <p class="muted">
                "Refresh analytics and run table reports against submitted data."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Report Actions"</h3>
                    <div class="actions">
                        <button type="button" onclick="refreshAnalytics()">"Refresh Analytics"</button>
                        <button type="button" onclick="loadReports()">"Choose Report"</button>
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
fn HierarchyAdminScreen() -> impl IntoView {
    view! {
        <section id="hierarchy-admin-screen" class="app-screen">
            <p class="eyebrow">"Admin Screen"</p>
            <h2>"Hierarchy Setup"</h2>
            <p class="muted">
                "Create and update node types, metadata definitions, and runtime nodes."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Hierarchy Actions"</h3>
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
                    <h3>"Current Hierarchy Context"</h3>
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
fn FormAdminScreen() -> impl IntoView {
    view! {
        <section id="form-admin-screen" class="app-screen">
            <p class="eyebrow">"Admin Screen"</p>
            <h2>"Form Builder"</h2>
            <p class="muted">
                "Create draft form versions, edit sections and fields, and publish the version."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Form Actions"</h3>
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
                    <h3>"Current Form Context"</h3>
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
        <section id="report-admin-screen" class="app-screen">
            <p class="eyebrow">"Admin Screen"</p>
            <h2>"Report Builder"</h2>
            <p class="muted">
                "Build table report bindings from selected form fields and inspect report output."
            </p>
            <div class="task-grid">
                <section class="task-panel">
                    <h3>"Reporting Configuration Actions"</h3>
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
                    <h3>"Current Reporting Builder Context"</h3>
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
        <section id="fixture-screen" class="app-screen">
            <p class="eyebrow">"Migration Screen"</p>
            <h2>"Legacy Fixture Validation"</h2>
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
