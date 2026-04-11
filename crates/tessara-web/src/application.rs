//! Route-specific Tessara application pages.

use crate::brand::document_head_tags;

#[derive(Copy, Clone)]
struct ActionSpec {
    handler: &'static str,
    label: &'static str,
}

const STANDARD_ACTIONS: &[ActionSpec] = &[
    ActionSpec {
        handler: "openLogin()",
        label: "Sign In",
    },
    ActionSpec {
        handler: "loadCurrentUser()",
        label: "Session Status",
    },
    ActionSpec {
        handler: "logout()",
        label: "Sign Out",
    },
    ActionSpec {
        handler: "loadAppSummary()",
        label: "Refresh Summary",
    },
];

fn render_application_document(
    title: &str,
    description: &str,
    style: &str,
    script: &str,
    body_attrs: &str,
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
  <body {body_attrs}>
    {shell}
    <script>{script}</script>
  </body>
</html>"#
    )
}

fn escape_attr(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn breadcrumb(parts: &[(&str, Option<&str>)]) -> String {
    parts
        .iter()
        .map(|(label, href)| match href {
            Some(href) => format!(r#"<a href="{href}">{label}</a>"#),
            None => format!(r#"<span>{label}</span>"#),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn action_buttons(actions: &[ActionSpec]) -> String {
    actions
        .iter()
        .map(|action| {
            format!(
                r#"<button type="button" onclick="{}">{}</button>"#,
                action.handler, action.label
            )
        })
        .collect::<Vec<_>>()
        .join("")
}

fn sidebar(active_route: &str, include_internal_create: bool) -> String {
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

    let product_html = product_links
        .into_iter()
        .map(|(key, href, label)| {
            let class_name = if key == active_route { "active" } else { "" };
            format!(r#"<a class="{class_name}" href="{href}">{label}</a>"#)
        })
        .collect::<Vec<_>>()
        .join("");
    let internal_html = internal_links
        .into_iter()
        .map(|(key, href, label)| {
            let class_name = if key == active_route { "active" } else { "" };
            format!(r#"<a class="{class_name}" href="{href}">{label}</a>"#)
        })
        .collect::<Vec<_>>()
        .join("");

    let create_html = if include_internal_create {
        r#"
            <section class="nav-panel">
              <h2>Create Shortcuts</h2>
              <p class="muted">These shortcuts stay internal while product routes own normal entity workflows.</p>
              <div class="create-menu">
                <a class="create-link" href="/app/admin">Open Legacy Builder</a>
                <a class="create-link" href="/">Open Test Harness</a>
              </div>
            </section>
        "#
        .to_string()
    } else {
        String::new()
    };

    format!(
        r#"
        <aside class="panel app-sidebar">
          <section class="nav-panel">
            <h2>Product Areas</h2>
            <nav class="app-nav" aria-label="Product navigation">{product_html}</nav>
          </section>
          <section class="nav-panel nav-panel-secondary">
            <h2>Internal Areas</h2>
            <nav class="app-nav" aria-label="Internal navigation">{internal_html}</nav>
          </section>
          {create_html}
          <section class="selection-panel">
            <h3>Current Selections</h3>
            <p class="muted">Selections follow you between list, detail, and edit screens.</p>
            <p id="session-status" class="muted">Not signed in.</p>
            <div id="selection-state" class="selection-grid">
              <p class="muted">No records selected yet.</p>
            </div>
          </section>
        </aside>
        "#
    )
}

fn app_shell(
    active_route: &str,
    area_kind: &str,
    title: &str,
    description: &str,
    breadcrumbs: &[(&str, Option<&str>)],
    children: String,
    include_internal_create: bool,
) -> String {
    format!(
        r#"
        <main class="shell app-shell">
          <section class="panel hero">
            <div class="brand-lockup">
              <img class="brand-mark" src="/assets/tessara-icon-1024.svg" alt="" />
              <span>Tessara</span>
            </div>
            <nav class="breadcrumb-trail" aria-label="Breadcrumb">{}</nav>
            <p class="muted">{}</p>
            <h1>{}</h1>
            <p>{}</p>
            <div class="actions">{}</div>
          </section>
          <section class="app-layout">
            {}
            <section class="panel app-main">
              {}
            </section>
          </section>
          <pre id="output" hidden></pre>
        </main>
        "#,
        breadcrumb(breadcrumbs),
        area_kind,
        title,
        description,
        action_buttons(STANDARD_ACTIONS),
        sidebar(active_route, include_internal_create),
        children
    )
}

fn page_header(eyebrow: &str, title: &str, description: &str, actions: String) -> String {
    format!(
        r#"
        <section class="app-screen entity-page">
          <p class="eyebrow">{eyebrow}</p>
          <div class="page-title-row">
            <div>
              <h2>{title}</h2>
              <p class="muted">{description}</p>
            </div>
            <div class="actions">{actions}</div>
          </div>
        </section>
        "#
    )
}

fn empty_panel(title: &str, description: &str, body: &str) -> String {
    format!(
        r#"
        <section class="app-screen page-panel">
          <h3>{title}</h3>
          <p class="muted">{description}</p>
          {body}
        </section>
        "#
    )
}

fn home_body() -> String {
    let product_cards = [
        (
            "Organization",
            "Browse runtime organization records and move into related forms, responses, and dashboards.",
            "/app/organization",
            "Go to Organization",
        ),
        (
            "Forms",
            "Browse top-level forms and their current published and related-report summaries.",
            "/app/forms",
            "Go to Forms",
        ),
        (
            "Responses",
            "Start new responses, resume drafts, and review submitted responses.",
            "/app/responses",
            "Go to Responses",
        ),
        (
            "Reports",
            "Browse reports, inspect definitions, and run supported report views.",
            "/app/reports",
            "Go to Reports",
        ),
        (
            "Dashboards",
            "Browse dashboards, inspect component summaries, and open previews.",
            "/app/dashboards",
            "Go to Dashboards",
        ),
    ];
    let internal_cards = [
        (
            "Administration",
            "Internal configuration and legacy builder access stay here, not on product routes.",
            "/app/administration",
            "Go to Administration",
        ),
        (
            "Migration",
            "Validate, dry-run, and import representative legacy fixtures from the operator workbench.",
            "/app/migration",
            "Go to Migration",
        ),
    ];

    format!(
        r#"
        <section class="app-screen">
          <p class="eyebrow">Shared Home</p>
          <h2>Welcome to Tessara</h2>
          <p class="muted">Use this shared home as the application entry point. It is structured so future role-aware home variants can reuse the same modules without changing routes.</p>
        </section>
        <section class="app-screen">
          <p class="eyebrow">Shared Home</p>
          <h2>Role-Ready Home Modules</h2>
          <p class="muted">These modules define the shared home shape for future admin, scoped-operator, and respondent variants.</p>
          <div class="home-grid">
            <article class="home-card">
              <h3>Scoped Operations</h3>
              <p>Organization and form access for partner, program, activity, and session-style work.</p>
            </article>
            <article class="home-card">
              <h3>Response Delivery</h3>
              <p>Start, edit, submit, and review response work without exposing builder-first navigation.</p>
            </article>
            <article class="home-card">
              <h3>Oversight and Insight</h3>
              <p>Reports, dashboards, and internal oversight remain available without collapsing back into a control console.</p>
            </article>
          </div>
        </section>
        <section class="app-screen">
          <p class="eyebrow">Shared Home</p>
          <h2>Product Areas</h2>
          <p class="muted">These are the primary destinations for top-level entity browsing and normal workflow entry.</p>
          <div class="home-grid">
            {}
          </div>
        </section>
        <section class="app-screen">
          <p class="eyebrow">Shared Home</p>
          <h2>Current Deployment Readiness</h2>
          <p class="muted">Refresh Summary to confirm the current stack has enough configured data for response, reporting, and dashboard workflows.</p>
          <div id="home-summary-cards" class="cards summary-grid">
            <p class="muted">Refresh Summary to load current counters.</p>
          </div>
        </section>
        <section class="app-screen">
          <p class="eyebrow">Shared Home</p>
          <h2>Current Workflow Context</h2>
          <p class="muted">Current selections appear here and in the shared sidebar.</p>
          <div id="home-selection-state" class="selection-grid">
            <p class="muted">No records selected yet.</p>
          </div>
        </section>
        <section class="app-screen">
          <p class="eyebrow">Shared Home</p>
          <h2>Internal Areas</h2>
          <p class="muted">Internal operator surfaces stay available, but they are clearly secondary to the main product areas.</p>
          <div class="home-grid">
            {}
          </div>
        </section>
        "#,
        product_cards
            .iter()
            .map(|(title, description, href, label)| {
                format!(
                    r#"<article class="home-card"><h3>{title}</h3><p>{description}</p><a class="button-link" href="{href}">{label}</a></article>"#
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        internal_cards
            .iter()
            .map(|(title, description, href, label)| {
                format!(
                    r#"<article class="directory-card"><h3>{title}</h3><p>{description}</p><a class="button-link" href="{href}">{label}</a></article>"#
                )
            })
            .collect::<Vec<_>>()
            .join("")
    )
}

fn login_body() -> String {
    form_screen(
        "Authentication",
        "Sign In",
        "Sign in with one of the seeded local accounts to load the routes and data available to that role.",
        "login-form",
        "/app",
        r#"
        <div class="form-grid">
          <div class="form-field">
            <label for="login-email">Email</label>
            <input id="login-email" type="email" autocomplete="username" />
          </div>
          <div class="form-field">
            <label for="login-password">Password</label>
            <input id="login-password" type="password" autocomplete="current-password" />
          </div>
        </div>
        <section class="app-screen page-panel compact-panel">
          <h3>Demo Accounts</h3>
          <div class="record-list">
            <article class="record-card compact-record-card">
              <h4>Admin</h4>
              <p class="muted">admin@tessara.local</p>
              <p class="muted">tessara-dev-admin</p>
            </article>
            <article class="record-card compact-record-card">
              <h4>Operator</h4>
              <p class="muted">operator@tessara.local</p>
              <p class="muted">tessara-dev-operator</p>
            </article>
            <article class="record-card compact-record-card">
              <h4>Parent / Respondent</h4>
              <p class="muted">parent@tessara.local / respondent@tessara.local</p>
              <p class="muted">tessara-dev-parent / tessara-dev-respondent</p>
            </article>
          </div>
        </section>
        "#,
    )
}

fn administration_body() -> String {
    format!(
        r#"
        {}
        {}
        {}
        "#,
        page_header(
            "Internal Area",
            "Administration",
            "Administration stays internal. Use it for advanced configuration and legacy builder access, not for the normal top-level entity workflows now handled in product areas.",
            String::new(),
        ),
        empty_panel(
            "Advanced Configuration",
            "The legacy configuration surfaces remain available here while product routes become the canonical home for top-level entity list, detail, create, and edit work.",
            r#"
            <div class="record-list">
              <article class="record-card">
                <h4>Legacy Builder</h4>
                <p>Open the advanced builder route for node types, form versions, datasets, aggregations, charts, and dashboard components.</p>
                <div class="actions">
                  <a class="button-link" href="/app/admin">Open Legacy Builder</a>
                  <a class="button-link" href="/">Open Test Harness</a>
                </div>
              </article>
            </div>
            "#,
        ),
        empty_panel(
            "Internal Links",
            "Migration and older internal tooling remain available without taking over the product navigation model.",
            r#"
            <div class="record-list">
              <article class="record-card">
                <h4>Migration</h4>
                <p>Validate, dry-run, and import representative legacy fixtures.</p>
                <div class="actions"><a class="button-link" href="/app/migration">Open Migration</a></div>
              </article>
            </div>
            "#,
        )
    )
}

fn migration_body() -> String {
    format!(
        r#"
        {}
        {}
        {}
        "#,
        page_header(
            "Internal Area",
            "Migration Workbench",
            "Use this operator surface to validate, dry-run, and import representative legacy fixtures.",
            String::new(),
        ),
        empty_panel(
            "Fixture Intake",
            "Load bundled examples or paste fixture JSON directly.",
            r#"
            <div class="entity-form-shell">
              <div class="actions">
                <button type="button" onclick="loadLegacyFixtureExamples()">Load Fixture Examples</button>
                <button type="button" onclick="validateLegacyFixture()">Validate Fixture</button>
                <button type="button" onclick="dryRunLegacyFixture()">Dry-Run Fixture</button>
                <button type="button" onclick="importLegacyFixture()">Import Fixture</button>
              </div>
              <label class="wide-field" for="legacy-fixture-json">Fixture JSON</label>
              <textarea id="legacy-fixture-json" rows="18" placeholder="Paste legacy fixture JSON"></textarea>
            </div>
            <div id="migration-list" class="record-list">
              <p class="muted">Load fixture examples or validate a pasted fixture.</p>
            </div>
            "#,
        ),
        empty_panel(
            "Migration Results",
            "Validation, dry-run, and import results appear here.",
            r#"<div id="migration-results" class="record-detail"><p class="muted">No migration activity yet.</p></div>"#,
        )
    )
}

fn list_screen(
    eyebrow: &str,
    title: &str,
    description: &str,
    create_label: &str,
    create_href: &str,
    list_id: &str,
    item_label: &str,
) -> String {
    format!(
        r#"
        {}
        {}
        "#,
        page_header(
            eyebrow,
            title,
            description,
            format!(r#"<a class="button-link" href="{create_href}">{create_label}</a>"#),
        ),
        empty_panel(
            &format!("{item_label} List"),
            &format!(
                "This screen lists current {item_label} records and links to their detail and edit screens."
            ),
            &format!(
                r#"<div id="{list_id}" class="record-list"><p class="muted">Loading {item_label} records...</p></div>"#
            ),
        )
    )
}

fn detail_screen(
    eyebrow: &str,
    title: &str,
    description: &str,
    back_href: &str,
    edit_href: &str,
    detail_id: &str,
    detail_title: &str,
) -> String {
    format!(
        r#"
        {}
        {}
        "#,
        page_header(
            eyebrow,
            title,
            description,
            format!(
                r#"<a class="button-link" href="{back_href}">Back to List</a><a class="button-link" href="{edit_href}">Edit</a>"#
            ),
        ),
        empty_panel(
            detail_title,
            "This screen is read-only. Use Edit to make changes.",
            &format!(
                r#"<div id="{detail_id}" class="record-detail"><p class="muted">Loading record detail...</p></div>"#
            ),
        )
    )
}

fn form_screen(
    eyebrow: &str,
    title: &str,
    description: &str,
    form_id: &str,
    cancel_href: &str,
    fields_html: &str,
) -> String {
    format!(
        r#"
        {}
        <section class="app-screen page-panel">
          <form id="{form_id}" class="entity-form">
            {fields_html}
            <div class="actions form-actions">
              <button type="submit">Submit</button>
              <a class="button-link" href="{cancel_href}">Cancel</a>
            </div>
          </form>
        </section>
        "#,
        page_header(eyebrow, title, description, String::new())
    )
}

fn organization_form_fields(is_edit: bool) -> String {
    let disabled = if is_edit { " disabled" } else { "" };
    format!(
        r#"
        <div class="form-grid">
          <div class="form-field">
            <label for="organization-node-type">Node Type</label>
            <select id="organization-node-type"{disabled}></select>
          </div>
          <div class="form-field">
            <label for="organization-parent-node">Parent Organization</label>
            <select id="organization-parent-node"></select>
          </div>
          <div class="form-field wide-field">
            <label for="organization-name">Name</label>
            <input id="organization-name" type="text" autocomplete="off" />
          </div>
        </div>
        <section class="page-panel nested-form-panel">
          <h3>Metadata</h3>
          <p class="muted">Metadata inputs are generated from the selected node type.</p>
          <div id="organization-metadata-fields" class="form-grid">
            <p class="muted">Choose a node type to load metadata fields.</p>
          </div>
        </section>
        "#
    )
}

fn form_entity_fields() -> String {
    r#"
        <div class="form-grid">
          <div class="form-field wide-field">
            <label for="form-name">Name</label>
            <input id="form-name" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="form-slug">Slug</label>
            <input id="form-slug" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="form-scope-node-type">Scope Node Type</label>
            <select id="form-scope-node-type"></select>
          </div>
        </div>
    "#
    .to_string()
}

fn response_new_fields() -> String {
    r#"
        <div class="form-grid">
          <div class="form-field">
            <label for="response-form-version">Published Form</label>
            <select id="response-form-version"></select>
          </div>
          <div class="form-field">
            <label for="response-node">Target Organization</label>
            <select id="response-node"></select>
          </div>
        </div>
    "#
    .to_string()
}

fn report_form_fields() -> String {
    r#"
        <div class="form-grid">
          <div class="form-field wide-field">
            <label for="report-name">Name</label>
            <input id="report-name" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="report-source-type">Source Type</label>
            <select id="report-source-type">
              <option value="form">Form</option>
              <option value="dataset">Dataset</option>
            </select>
          </div>
          <div class="form-field">
            <label for="report-source-id">Source</label>
            <select id="report-source-id"></select>
          </div>
        </div>
        <section class="page-panel nested-form-panel">
          <div class="page-title-row compact-title-row">
            <div>
              <h3>Bindings</h3>
              <p class="muted">Each binding defines one logical field in the report output.</p>
            </div>
            <div class="actions">
              <button type="button" onclick="addReportBindingRow()">Add Binding</button>
            </div>
          </div>
          <div id="report-binding-rows" class="binding-list"></div>
        </section>
    "#
    .to_string()
}

fn dashboard_form_fields() -> String {
    r#"
        <div class="form-grid">
          <div class="form-field wide-field">
            <label for="dashboard-name">Name</label>
            <input id="dashboard-name" type="text" autocomplete="off" />
          </div>
        </div>
    "#
    .to_string()
}

pub fn application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Home",
        "Tessara application home for local replacement workflow testing.",
        style,
        script,
        r#"data-page-key="home" data-active-route="home""#,
        app_shell(
            "home",
            "Shared Home",
            "Application Overview",
            "This shared home is the primary entry point for Tessara. It organizes product areas, current readiness, and current workflow context without exposing configuration-first controls.",
            &[("Home", None)],
            home_body(),
            false,
        ),
    )
}

pub fn login_application_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Sign In",
        "Sign in to the Tessara application shell.",
        style,
        script,
        r#"data-page-key="login" data-active-route="login""#,
        app_shell(
            "home",
            "Shared Home",
            "Sign In",
            "Authenticate with a local demo account to load the application areas available to that role.",
            &[("Home", Some("/app")), ("Sign In", None)],
            login_body(),
            false,
        ),
    )
}

pub fn administration_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Administration",
        "Tessara internal administration landing page.",
        style,
        script,
        r#"data-page-key="administration" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "Administration",
            "Use this internal area for advanced configuration and legacy builder access.",
            &[("Home", Some("/app")), ("Administration", None)],
            administration_body(),
            true,
        ),
    )
}

pub fn migration_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Migration",
        "Tessara migration workbench.",
        style,
        script,
        r#"data-page-key="migration" data-active-route="migration""#,
        app_shell(
            "migration",
            "Internal Area",
            "Migration Workbench",
            "Validate, dry-run, and import representative legacy fixtures from this operator surface.",
            &[("Home", Some("/app")), ("Migration", None)],
            migration_body(),
            false,
        ),
    )
}

pub fn organization_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Organizations",
        "Tessara organization list screen.",
        style,
        script,
        r#"data-page-key="organization-list" data-active-route="organization""#,
        app_shell(
            "organization",
            "Product Area",
            "Organization",
            "Browse runtime organization records and move into their related forms, responses, and dashboards.",
            &[("Home", Some("/app")), ("Organization", None)],
            list_screen(
                "Organization",
                "Organizations",
                "This list screen contains the current runtime organization records.",
                "Create Organization",
                "/app/organization/new",
                "organization-list",
                "organization",
            ),
            false,
        ),
    )
}

pub fn organization_create_application_html(style: &str, script: &str) -> String {
    render_application_document(
        "Create Organization",
        "Create a runtime organization record.",
        style,
        script,
        r#"data-page-key="organization-create" data-active-route="organization""#,
        app_shell(
            "organization",
            "Product Area",
            "Organization",
            "Create a new runtime organization record from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Organization", Some("/app/organization")),
                ("New Organization", None),
            ],
            form_screen(
                "Organization",
                "Create Organization",
                "Complete the fields below to create a new runtime organization record.",
                "organization-form",
                "/app/organization",
                &organization_form_fields(false),
            ),
            false,
        ),
    )
}

pub fn organization_detail_application_html(style: &str, script: &str, node_id: &str) -> String {
    let escaped = escape_attr(node_id);
    render_application_document(
        "Organization Detail",
        "Organization detail screen.",
        style,
        script,
        &format!(
            r#"data-page-key="organization-detail" data-active-route="organization" data-record-id="{escaped}""#
        ),
        app_shell(
            "organization",
            "Product Area",
            "Organization",
            "Review the selected organization record and its related records.",
            &[
                ("Home", Some("/app")),
                ("Organization", Some("/app/organization")),
                ("Organization Detail", None),
            ],
            detail_screen(
                "Organization",
                "Organization Detail",
                "This screen shows the selected runtime organization record in read-only form.",
                "/app/organization",
                &format!("/app/organization/{escaped}/edit"),
                "organization-detail",
                "Organization Record",
            ),
            false,
        ),
    )
}

pub fn organization_edit_application_html(style: &str, script: &str, node_id: &str) -> String {
    let escaped = escape_attr(node_id);
    render_application_document(
        "Edit Organization",
        "Edit a runtime organization record.",
        style,
        script,
        &format!(
            r#"data-page-key="organization-edit" data-active-route="organization" data-record-id="{escaped}""#
        ),
        app_shell(
            "organization",
            "Product Area",
            "Organization",
            "Edit the selected runtime organization record from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Organization", Some("/app/organization")),
                (
                    "Organization Detail",
                    Some(&format!("/app/organization/{escaped}")),
                ),
                ("Edit Organization", None),
            ],
            form_screen(
                "Organization",
                "Edit Organization",
                "Update the selected runtime organization record and submit to save changes.",
                "organization-form",
                &format!("/app/organization/{escaped}"),
                &organization_form_fields(true),
            ),
            false,
        ),
    )
}

pub fn forms_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Forms",
        "Tessara forms list screen.",
        style,
        script,
        r#"data-page-key="form-list" data-active-route="forms""#,
        app_shell(
            "forms",
            "Product Area",
            "Forms",
            "Browse top-level forms and move into their dedicated detail and edit screens.",
            &[("Home", Some("/app")), ("Forms", None)],
            list_screen(
                "Forms",
                "Forms",
                "This list screen contains the current top-level form records.",
                "Create Form",
                "/app/forms/new",
                "form-list",
                "form",
            ),
            false,
        ),
    )
}

pub fn form_create_application_html(style: &str, script: &str) -> String {
    render_application_document(
        "Create Form",
        "Create a top-level form.",
        style,
        script,
        r#"data-page-key="form-create" data-active-route="forms""#,
        app_shell(
            "forms",
            "Product Area",
            "Forms",
            "Create a top-level form from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Forms", Some("/app/forms")),
                ("New Form", None),
            ],
            form_screen(
                "Forms",
                "Create Form",
                "Complete the fields below to create a top-level form.",
                "form-entity-form",
                "/app/forms",
                &form_entity_fields(),
            ),
            false,
        ),
    )
}

pub fn form_detail_application_html(style: &str, script: &str, form_id: &str) -> String {
    let escaped = escape_attr(form_id);
    render_application_document(
        "Form Detail",
        "Form detail screen.",
        style,
        script,
        &format!(
            r#"data-page-key="form-detail" data-active-route="forms" data-record-id="{escaped}""#
        ),
        app_shell(
            "forms",
            "Product Area",
            "Forms",
            "Review the selected form, its scope, current version summary, and related records.",
            &[
                ("Home", Some("/app")),
                ("Forms", Some("/app/forms")),
                ("Form Detail", None),
            ],
            detail_screen(
                "Forms",
                "Form Detail",
                "This screen shows the selected top-level form in read-only form.",
                "/app/forms",
                &format!("/app/forms/{escaped}/edit"),
                "form-detail",
                "Form Record",
            ),
            false,
        ),
    )
}

pub fn form_edit_application_html(style: &str, script: &str, form_id: &str) -> String {
    let escaped = escape_attr(form_id);
    render_application_document(
        "Edit Form",
        "Edit a top-level form.",
        style,
        script,
        &format!(
            r#"data-page-key="form-edit" data-active-route="forms" data-record-id="{escaped}""#
        ),
        app_shell(
            "forms",
            "Product Area",
            "Forms",
            "Edit the selected top-level form from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Forms", Some("/app/forms")),
                ("Form Detail", Some(&format!("/app/forms/{escaped}"))),
                ("Edit Form", None),
            ],
            form_screen(
                "Forms",
                "Edit Form",
                "Update the selected top-level form and submit to save changes.",
                "form-entity-form",
                &format!("/app/forms/{escaped}"),
                &form_entity_fields(),
            ),
            false,
        ),
    )
}

pub fn responses_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Responses",
        "Tessara responses list screen.",
        style,
        script,
        r#"data-page-key="response-list" data-active-route="responses""#,
        app_shell(
            "responses",
            "Product Area",
            "Responses",
            "Start new responses, resume drafts, and review submitted responses from dedicated screens.",
            &[("Home", Some("/app")), ("Responses", None)],
            format!(
                r#"
                {}
                {}
                {}
                {}
                {}
                "#,
                page_header(
                    "Responses",
                    "Responses",
                    "This list screen separates new work, drafts, and submitted responses.",
                    r#"<a class="button-link" href="/app/responses/new">Start Response</a>"#
                        .to_string(),
                ),
                r#"<div id="response-context-switcher"></div>"#,
                empty_panel(
                    "Start New Response",
                    "Published forms ready to start a response appear here.",
                    r#"<div id="response-pending-list" class="record-list"><p class="muted">Loading published forms...</p></div>"#,
                ),
                empty_panel(
                    "Draft Responses",
                    "Draft responses link to detail and edit screens.",
                    r#"<div id="response-draft-list" class="record-list"><p class="muted">Loading draft responses...</p></div>"#,
                ),
                empty_panel(
                    "Submitted Responses",
                    "Submitted responses remain read-only and link to their detail screens.",
                    r#"<div id="response-submitted-list" class="record-list"><p class="muted">Loading submitted responses...</p></div>"#,
                ),
            ),
            false,
        ),
    )
}

pub fn submission_application_shell_html(style: &str, script: &str) -> String {
    responses_application_shell_html(style, script)
}

pub fn response_create_application_html(style: &str, script: &str) -> String {
    render_application_document(
        "Start Response",
        "Start a new response draft.",
        style,
        script,
        r#"data-page-key="response-create" data-active-route="responses""#,
        app_shell(
            "responses",
            "Product Area",
            "Responses",
            "Start a new response from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Responses", Some("/app/responses")),
                ("New Response", None),
            ],
            form_screen(
                "Responses",
                "Start Response",
                "Choose a published form and target organization to create a draft response.",
                "response-start-form",
                "/app/responses",
                &format!(
                    r#"<div id="response-create-context-switcher"></div>{}"#,
                    response_new_fields()
                ),
            ),
            false,
        ),
    )
}

pub fn response_detail_application_html(style: &str, script: &str, submission_id: &str) -> String {
    let escaped = escape_attr(submission_id);
    render_application_document(
        "Response Detail",
        "Response detail screen.",
        style,
        script,
        &format!(
            r#"data-page-key="response-detail" data-active-route="responses" data-record-id="{escaped}""#
        ),
        app_shell(
            "responses",
            "Product Area",
            "Responses",
            "Review the selected response and its audit history.",
            &[
                ("Home", Some("/app")),
                ("Responses", Some("/app/responses")),
                ("Response Detail", None),
            ],
            format!(
                r#"
                {}
                {}
                "#,
                page_header(
                    "Responses",
                    "Response Detail",
                    "This screen shows the selected response in read-only form. Drafts expose their edit action inside the detail content.",
                    r#"<a class="button-link" href="/app/responses">Back to List</a>"#.to_string(),
                ),
                empty_panel(
                    "Response Record",
                    "Response values and audit trail appear here.",
                    r#"<div id="response-detail" class="record-detail"><p class="muted">Loading record detail...</p></div>"#,
                ),
            ),
            false,
        ),
    )
}

pub fn response_edit_application_html(style: &str, script: &str, submission_id: &str) -> String {
    let escaped = escape_attr(submission_id);
    render_application_document(
        "Edit Response",
        "Edit a draft response.",
        style,
        script,
        &format!(
            r#"data-page-key="response-edit" data-active-route="responses" data-record-id="{escaped}""#
        ),
        app_shell(
            "responses",
            "Product Area",
            "Responses",
            "Edit the selected draft response. Submitted responses remain read-only.",
            &[
                ("Home", Some("/app")),
                ("Responses", Some("/app/responses")),
                (
                    "Response Detail",
                    Some(&format!("/app/responses/{escaped}")),
                ),
                ("Edit Response", None),
            ],
            format!(
                r#"
                {}
                {}
                "#,
                page_header(
                    "Responses",
                    "Edit Response",
                    "Save changes to the current draft or submit it from this dedicated response form screen.",
                    format!(r#"<a class="button-link" href="/app/responses/{escaped}">Cancel</a>"#),
                ),
                empty_panel(
                    "Draft Response Form",
                    "The current draft loads here. Submitted responses show a read-only guard instead of editable controls.",
                    r#"<div id="response-edit-surface" class="record-detail"><p class="muted">Loading response form...</p></div>"#,
                ),
            ),
            false,
        ),
    )
}

pub fn reporting_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Reports",
        "Tessara reports list screen.",
        style,
        script,
        r#"data-page-key="report-list" data-active-route="reports""#,
        app_shell(
            "reports",
            "Product Area",
            "Reports",
            "Browse reports and move into dedicated detail and edit screens.",
            &[("Home", Some("/app")), ("Reports", None)],
            list_screen(
                "Reports",
                "Reports",
                "This list screen contains the current report records.",
                "Create Report",
                "/app/reports/new",
                "report-list",
                "report",
            ),
            false,
        ),
    )
}

pub fn report_create_application_html(style: &str, script: &str) -> String {
    render_application_document(
        "Create Report",
        "Create a top-level report.",
        style,
        script,
        r#"data-page-key="report-create" data-active-route="reports""#,
        app_shell(
            "reports",
            "Product Area",
            "Reports",
            "Create a top-level report from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Reports", Some("/app/reports")),
                ("New Report", None),
            ],
            form_screen(
                "Reports",
                "Create Report",
                "Complete the fields below to create a report and its initial bindings.",
                "report-form",
                "/app/reports",
                &report_form_fields(),
            ),
            false,
        ),
    )
}

pub fn report_detail_application_html(style: &str, script: &str, report_id: &str) -> String {
    let escaped = escape_attr(report_id);
    render_application_document(
        "Report Detail",
        "Report detail screen.",
        style,
        script,
        &format!(
            r#"data-page-key="report-detail" data-active-route="reports" data-record-id="{escaped}""#
        ),
        app_shell(
            "reports",
            "Product Area",
            "Reports",
            "Review the selected report, run it, and inspect related reporting assets.",
            &[
                ("Home", Some("/app")),
                ("Reports", Some("/app/reports")),
                ("Report Detail", None),
            ],
            format!(
                r#"
                {}
                {}
                {}
                "#,
                page_header(
                    "Reports",
                    "Report Detail",
                    "This screen shows the selected report in read-only form and supports running it.",
                    format!(
                        r#"<a class="button-link" href="/app/reports">Back to List</a><a class="button-link" href="/app/reports/{escaped}/edit">Edit</a><button type="button" onclick="runCurrentReport()">Run</button>"#
                    ),
                ),
                empty_panel(
                    "Report Record",
                    "Report metadata, binding summary, and related assets appear here.",
                    r#"<div id="report-detail" class="record-detail"><p class="muted">Loading report detail...</p></div>"#,
                ),
                empty_panel(
                    "Report Results",
                    "Run the current report to see its table output.",
                    r#"<div id="report-results" class="record-detail"><p class="muted">Run the report to load results.</p></div>"#,
                ),
            ),
            false,
        ),
    )
}

pub fn report_edit_application_html(style: &str, script: &str, report_id: &str) -> String {
    let escaped = escape_attr(report_id);
    render_application_document(
        "Edit Report",
        "Edit a top-level report.",
        style,
        script,
        &format!(
            r#"data-page-key="report-edit" data-active-route="reports" data-record-id="{escaped}""#
        ),
        app_shell(
            "reports",
            "Product Area",
            "Reports",
            "Edit the selected report from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Reports", Some("/app/reports")),
                ("Report Detail", Some(&format!("/app/reports/{escaped}"))),
                ("Edit Report", None),
            ],
            form_screen(
                "Reports",
                "Edit Report",
                "Update the selected report and its bindings.",
                "report-form",
                &format!("/app/reports/{escaped}"),
                &report_form_fields(),
            ),
            false,
        ),
    )
}

pub fn dashboards_application_shell_html(style: &str, script: &str) -> String {
    render_application_document(
        "Tessara Dashboards",
        "Tessara dashboards list screen.",
        style,
        script,
        r#"data-page-key="dashboard-list" data-active-route="dashboards""#,
        app_shell(
            "dashboards",
            "Product Area",
            "Dashboards",
            "Browse dashboards and move into dedicated detail and edit screens.",
            &[("Home", Some("/app")), ("Dashboards", None)],
            list_screen(
                "Dashboards",
                "Dashboards",
                "This list screen contains the current dashboard records.",
                "Create Dashboard",
                "/app/dashboards/new",
                "dashboard-list",
                "dashboard",
            ),
            false,
        ),
    )
}

pub fn dashboard_create_application_html(style: &str, script: &str) -> String {
    render_application_document(
        "Create Dashboard",
        "Create a top-level dashboard.",
        style,
        script,
        r#"data-page-key="dashboard-create" data-active-route="dashboards""#,
        app_shell(
            "dashboards",
            "Product Area",
            "Dashboards",
            "Create a top-level dashboard from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Dashboards", Some("/app/dashboards")),
                ("New Dashboard", None),
            ],
            form_screen(
                "Dashboards",
                "Create Dashboard",
                "Complete the fields below to create a dashboard.",
                "dashboard-form",
                "/app/dashboards",
                &dashboard_form_fields(),
            ),
            false,
        ),
    )
}

pub fn dashboard_detail_application_html(style: &str, script: &str, dashboard_id: &str) -> String {
    let escaped = escape_attr(dashboard_id);
    render_application_document(
        "Dashboard Detail",
        "Dashboard detail screen.",
        style,
        script,
        &format!(
            r#"data-page-key="dashboard-detail" data-active-route="dashboards" data-record-id="{escaped}""#
        ),
        app_shell(
            "dashboards",
            "Product Area",
            "Dashboards",
            "Review the selected dashboard and its current component summary.",
            &[
                ("Home", Some("/app")),
                ("Dashboards", Some("/app/dashboards")),
                ("Dashboard Detail", None),
            ],
            format!(
                r#"
                {}
                {}
                "#,
                page_header(
                    "Dashboards",
                    "Dashboard Detail",
                    "This screen shows the selected dashboard in read-only form and supports previewing it.",
                    format!(
                        r#"<a class="button-link" href="/app/dashboards">Back to List</a><a class="button-link" href="/app/dashboards/{escaped}/edit">Edit</a><button type="button" onclick="viewCurrentDashboard()">View</button>"#
                    ),
                ),
                empty_panel(
                    "Dashboard Record",
                    "Dashboard metadata, component summary, and linked chart context appear here.",
                    r#"<div id="dashboard-detail" class="record-detail"><p class="muted">Loading dashboard detail...</p></div>"#,
                ),
            ),
            false,
        ),
    )
}

pub fn dashboard_edit_application_html(style: &str, script: &str, dashboard_id: &str) -> String {
    let escaped = escape_attr(dashboard_id);
    render_application_document(
        "Edit Dashboard",
        "Edit a top-level dashboard.",
        style,
        script,
        &format!(
            r#"data-page-key="dashboard-edit" data-active-route="dashboards" data-record-id="{escaped}""#
        ),
        app_shell(
            "dashboards",
            "Product Area",
            "Dashboards",
            "Edit the selected dashboard from a dedicated form screen.",
            &[
                ("Home", Some("/app")),
                ("Dashboards", Some("/app/dashboards")),
                (
                    "Dashboard Detail",
                    Some(&format!("/app/dashboards/{escaped}")),
                ),
                ("Edit Dashboard", None),
            ],
            form_screen(
                "Dashboards",
                "Edit Dashboard",
                "Update the selected dashboard and submit to save changes.",
                "dashboard-form",
                &format!("/app/dashboards/{escaped}"),
                &dashboard_form_fields(),
            ),
            false,
        ),
    )
}
