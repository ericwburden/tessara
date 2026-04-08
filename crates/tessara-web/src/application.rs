//! Leptos components for the user-facing Tessara application shell.

use leptos::prelude::*;

use crate::brand::document_head_tags;

/// Builds the application shell document used for human workflow testing.
pub fn application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <HomeApplicationShell/> }.to_html();
    let brand = document_head_tags(
        "Tessara Home",
        "Tessara application home for local replacement workflow testing.",
    );

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara Home</title>
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

/// Builds the focused submission application shell document.
pub fn submission_application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <SubmissionApplicationShell/> }.to_html();
    let brand = document_head_tags(
        "Tessara Submissions",
        "Tessara submission workspace for local replacement workflow testing.",
    );

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara Submissions</title>
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

/// Builds the focused admin application shell document.
pub fn admin_application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <AdminApplicationShell/> }.to_html();
    let brand = document_head_tags(
        "Tessara Admin",
        "Tessara admin setup workspace for hierarchy, forms, reports, and dashboards.",
    );

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara Admin</title>
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

/// Builds the focused migration workbench application shell document.
pub fn migration_application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <MigrationApplicationShell/> }.to_html();
    let brand = document_head_tags(
        "Tessara Migration",
        "Tessara migration workbench for validating and rehearsing legacy imports.",
    );

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara Migration</title>
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

/// Builds the focused reporting application shell document.
pub fn reporting_application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <ReportingApplicationShell/> }.to_html();
    let brand = document_head_tags(
        "Tessara Reporting",
        "Tessara reporting workspace for analytics, table reports, and dashboard previews.",
    );

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara Reporting</title>
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

#[component]
fn HomeApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <p class="muted">"Tessara Home"</p>
                <h1>"Application Overview"</h1>
                <p>
                    "This shell is the transition from a migration workbench to a real application. "
                    "It keeps the replacement-oriented routes, but now organizes them around a home screen, "
                    "persistent navigation, and clear entry points for the main workflows."
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
fn SubmissionApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <p class="muted">"Tessara Application"</p>
                <h1>"Submission Workspace"</h1>
                <p>
                    "This screen is the first replacement-oriented application surface. "
                    "It uses the same API contracts as the migration workbench, but presents "
                    "the published-form, draft, save, and submit flow as an application task."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="loadCurrentUser()">"Current User"</button>
                    <button type="button" onclick="logout()">"Log Out"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="startDemoSubmissionFlow()">"Start Demo Submission"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="submissions"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <SubmissionHomeScreen/>
                    <SubmissionScreen/>
                    <ReviewScreen/>
                    <ReportScreen/>
                    <OutputPanels/>
                </section>
            </section>
        </main>
    }
}

#[component]
fn AdminApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <p class="muted">"Tessara Admin"</p>
                <h1>"Setup Workspace"</h1>
                <p>
                    "This screen starts the replacement-oriented admin workflow for configuring "
                    "hierarchy and form definitions without navigating the full workbench."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <button type="button" onclick="loadAppSummary()">"Load App Summary"</button>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <ApplicationNav active_route="admin"/>
                    <CreateMenu/>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <AdminHomeScreen/>
                    <HierarchyAdminScreen/>
                    <FormAdminScreen/>
                    <ReportAdminScreen/>
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
                <p class="muted">"Tessara Migration"</p>
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
fn ReportingApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <BrandLockup/>
                <p class="muted">"Tessara Reporting"</p>
                <h1>"Reporting Workspace"</h1>
                <p>
                    "This screen gives testers a focused place to refresh analytics, inspect "
                    "table reports, and preview dashboards after submissions or import rehearsals."
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
                    <ReportRunnerScreen/>
                    <DashboardPreviewScreen/>
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
    let route_links = [
        ("home", "/app", "Home"),
        ("submissions", "/app/submissions", "Submissions"),
        ("admin", "/app/admin", "Administration"),
        ("reports", "/app/reports", "Reports"),
        ("migration", "/app/migration", "Migration"),
    ];

    view! {
        <section class="nav-panel">
            <h2>"Navigation"</h2>
            <nav class="app-nav" aria-label="Application navigation">
                {route_links
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
    }
}

#[component]
fn CreateMenu() -> impl IntoView {
    let create_links = [
        ("Create Node", "/app/admin#hierarchy-admin-screen"),
        ("Create Form", "/app/admin#form-admin-screen"),
        ("Create Dataset", "/app/admin#report-admin-screen"),
        ("Create Report", "/app/admin#report-admin-screen"),
        ("Create Aggregation", "/app/admin#report-admin-screen"),
        ("Create Dashboard", "/app/admin#report-admin-screen"),
    ];

    view! {
        <section class="nav-panel">
            <h2>"Create"</h2>
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
    let workflow_cards = [
        (
            "Submissions",
            "Complete draft, save, submit, and review flows for published forms.",
            "/app/submissions",
            "Open Submission Workspace",
        ),
        (
            "Administration",
            "Configure hierarchy, forms, datasets, reports, aggregations, charts, and dashboards.",
            "/app/admin",
            "Open Administration",
        ),
        (
            "Reports",
            "Inspect reports, run aggregations, and preview dashboards in one place.",
            "/app/reports",
            "Open Reporting",
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
                <button type="button" onclick="startDemoSubmissionFlow()">"Start Demo Submission"</button>
                <button type="button" onclick="openDemoDashboard()">"Open Demo Dashboard"</button>
            </div>
        </section>
        <section class="app-screen">
            <p class="eyebrow">"Application Home"</p>
            <h2>"Workflow Areas"</h2>
            <div class="home-grid">
                {workflow_cards
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
            <h2>"Navigation Model"</h2>
            <ul class="app-list">
                <li>"Home provides overview, quick starts, and the main creation entry points."</li>
                <li>"Submissions focuses on published forms, drafts, and response review."</li>
                <li>"Administration focuses on hierarchy, forms, datasets, reports, and dashboards."</li>
                <li>"Reports focuses on table output, aggregations, and dashboard preview."</li>
                <li>"Migration focuses on fixture validation, dry-run, and import rehearsal."</li>
            </ul>
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
            <p class="eyebrow">"Submission Home"</p>
            <h2>"Submission Stages"</h2>
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
            <p class="eyebrow">"Submission Home"</p>
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
            <p class="eyebrow">"Reporting Home"</p>
            <h2>"Reporting Areas"</h2>
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
            <p class="eyebrow">"Reporting Home"</p>
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
            <div class="inputs">
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
                <label>
                    <span>"Report bindings JSON"</span>
                    <input id="report-fields-json" placeholder="Loaded report bindings" value="" />
                </label>
            </div>
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
            <div class="inputs">
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
            <div class="actions">
                <button type="button" onclick="loadDashboards()">"Choose Dashboard"</button>
                <button type="button" onclick="refreshAnalyticsAndOpenDashboard()">"Refresh and Open Dashboard"</button>
                <button type="button" onclick="loadDashboardById()">"Open Dashboard"</button>
                <button type="button" onclick="loadCharts()">"Choose Chart"</button>
                <button type="button" onclick="loadChartDefinitionById()">"Inspect Chart"</button>
                <button type="button" onclick="loadAggregations()">"Choose Aggregation"</button>
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
            <div class="inputs">
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
            <div class="inputs">
                <label>
                    <span>"Submission search"</span>
                    <input id="submission-search" placeholder="Search form, node, or version" value="" />
                </label>
                <label>
                    <span>"Submission status filter"</span>
                    <input id="submission-status-filter" placeholder="draft or submitted" value="" />
                </label>
            </div>
            <div class="actions">
                <button type="button" onclick="loadSubmissions()">"Load Submissions"</button>
                <button type="button" onclick="showDraftSubmissions()">"Show Drafts"</button>
                <button type="button" onclick="showSubmittedSubmissions()">"Show Submitted"</button>
                <button type="button" onclick="clearSubmissionReviewFilters()">"Clear Review Filters"</button>
                <button type="button" onclick="loadSubmissionById()">"Open Selected Submission"</button>
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
            <div class="inputs">
                <label>
                    <span>"Report ID"</span>
                    <input id="report-id" placeholder="Selected report ID" value="" />
                </label>
            </div>
            <div class="actions">
                <button type="button" onclick="refreshAnalytics()">"Refresh Analytics"</button>
                <button type="button" onclick="loadReports()">"Choose Report"</button>
                <button type="button" onclick="refreshAnalyticsAndRunReport()">"Refresh and Run Report"</button>
                <button type="button" onclick="loadReportById()">"Run Selected Report"</button>
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
            <div class="inputs">
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
                <label><span>"Node metadata JSON"</span><input id="node-metadata-json" placeholder="{\"region\":\"North\"}" value="{\"region\":\"North\"}" /></label>
                <label><span>"Node search"</span><input id="node-search" placeholder="Search nodes" value="" /></label>
                <label><span>"Node ID"</span><input id="node-id" placeholder="Selected node ID" value="" /></label>
            </div>
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
            <div class="inputs">
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
            <div class="inputs">
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
                <label><span>"Dashboard component config JSON"</span><input id="dashboard-component-config-json" placeholder="{\"title\":\"Chart\"}" value="" /></label>
            </div>
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
