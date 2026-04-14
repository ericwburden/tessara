//! Route-specific Tessara application pages.

use crate::pipeline;
use crate::{brand::document_head_tags, theme};

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
    bridge_script_path: &str,
    body_attrs: &str,
    shell: String,
) -> String {
    let brand = document_head_tags(title, description);
    let theme_bootstrap = theme::bootstrap_script();
    let stylesheets = theme::stylesheet_links();
    let hydration = pipeline::hydration_module_tag();

    format!(
        r#"<!doctype html>
<html lang="en" data-theme="light" data-theme-preference="system">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    {brand}
    <script>{theme_bootstrap}</script>
    {stylesheets}
  </head>
  <body class="tessara-app" {body_attrs}>
    <div id="{app_root_id}">{app_root_start}{shell}{app_root_end}</div>
    <script src="{bridge_script_path}" defer></script>
    {hydration}
  </body>
</html>"#,
        app_root_id = pipeline::APP_ROOT_ID,
        app_root_start = pipeline::APP_ROOT_START,
        app_root_end = pipeline::APP_ROOT_END,
    )
}

fn theme_toggle() -> &'static str {
    theme::control_html()
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
                r#"<button class="button is-primary is-light" type="button" onclick="{}">{}</button>"#,
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
            format!(r#"<li><a class="{class_name}" href="{href}">{label}</a></li>"#)
        })
        .collect::<Vec<_>>()
        .join("");
    let internal_html = internal_links
        .into_iter()
        .map(|(key, href, label)| {
            let class_name = if key == active_route { "active" } else { "" };
            format!(r#"<li><a class="{class_name}" href="{href}">{label}</a></li>"#)
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
        <aside class="panel box app-sidebar">
          <section class="nav-panel menu">
            <p class="menu-label">Product Areas</p>
            <nav aria-label="Product navigation">
              <ul class="menu-list app-nav">{product_html}</ul>
            </nav>
          </section>
          <section class="nav-panel nav-panel-secondary menu">
            <p class="menu-label">Internal Areas</p>
            <nav aria-label="Internal navigation">
              <ul class="menu-list app-nav">{internal_html}</ul>
            </nav>
          </section>
          {create_html}
          <section class="selection-panel">
            <h3>Session And Selections</h3>
            <p class="muted">The current signed-in account and your selected records appear here.</p>
            <p id="session-status" class="muted">Not signed in.</p>
            <div id="current-user-summary" class="selection-grid">
              <p class="muted">Sign in to load account context.</p>
            </div>
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
          <section class="panel box hero">
            <div class="hero-header">
              <div class="brand-lockup">
                <img class="brand-mark" src="/assets/tessara-icon-1024.svg" alt="" />
                <span>Tessara</span>
              </div>
              {}
            </div>
            <nav class="breadcrumb-trail" aria-label="Breadcrumb">{}</nav>
            <p class="muted">{}</p>
            <h1>{}</h1>
            <p>{}</p>
            <div class="actions">{}</div>
          </section>
          <section class="app-layout">
            {}
            <section class="panel box app-main">
              {}
            </section>
          </section>
          <pre id="output" hidden></pre>
        </main>
        "#,
        theme_toggle(),
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
        <section class="app-screen box entity-page">
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
    let description_html = if description.trim().is_empty() {
        String::new()
    } else {
        format!(r#"<p class="muted">{description}</p>"#)
    };
    format!(
        r#"
        <section class="app-screen box page-panel">
          <h3>{title}</h3>
          {description_html}
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
                    r#"<article class="home-card card"><div class="card-content"><h3>{title}</h3><p>{description}</p><a class="button-link button is-light" href="{href}">{label}</a></div></article>"#
                )
            })
            .collect::<Vec<_>>()
            .join(""),
        internal_cards
            .iter()
            .map(|(title, description, href, label)| {
                format!(
                    r#"<article class="directory-card card"><div class="card-content"><h3>{title}</h3><p>{description}</p><a class="button-link button is-light" href="{href}">{label}</a></div></article>"#
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
        <div id="login-feedback" class="notification is-danger is-light is-hidden"></div>
        <div class="form-grid">
          <div class="form-field">
            <label for="login-email">Email</label>
            <input class="input" id="login-email" type="email" autocomplete="username" />
          </div>
          <div class="form-field">
            <label for="login-password">Password</label>
            <input class="input" id="login-password" type="password" autocomplete="current-password" />
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
              <h4>Delegator / Delegate</h4>
              <p class="muted">delegator@tessara.local / delegate@tessara.local</p>
              <p class="muted">tessara-dev-delegator / tessara-dev-delegate</p>
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
            "User Management",
            "User accounts now have dedicated application screens for browse, detail, create, and edit work.",
            r#"
            <div class="record-list">
              <article class="record-card">
                <h4>User Accounts</h4>
                <p>Manage local users, passwords, role memberships, and active status without dropping into the legacy builder.</p>
                <div class="actions">
                  <a class="button-link" href="/app/administration/users">Open User Management</a>
                </div>
              </article>
            </div>
            "#,
        ),
        empty_panel(
            "Roles And Access",
            "Manage reusable capability bundles and the scoped access and delegation assignments that control application visibility.",
            r#"
            <div class="record-list">
              <article class="record-card">
                <h4>Role Catalog</h4>
                <p>Review the current role bundles, inspect their capabilities, create additional roles, and edit what each role grants.</p>
                <div class="actions">
                  <a class="button-link" href="/app/administration/roles">Open Roles</a>
                </div>
              </article>
            </div>
            "#,
        ),
        empty_panel(
            "Organization Schema",
            "Node-type labels and hierarchy rules stay in administration so product organization screens can remain hierarchy-first and terminology-aware.",
            r#"
            <div class="record-list">
              <article class="record-card">
                <h4>Organization Node Types</h4>
                <p>Manage singular/plural labels, slugs, and allowed parent/child relationships for the organization hierarchy.</p>
                <div class="actions">
                  <a class="button-link" href="/app/administration/node-types">Open Organization Node Types</a>
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
              <textarea class="textarea" id="legacy-fixture-json" rows="18" placeholder="Paste legacy fixture JSON"></textarea>
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
            format!(
                r#"<a class="button-link button is-primary" href="{create_href}">{create_label}</a>"#
            ),
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
                r#"<a class="button-link button is-light" href="{back_href}">Back to List</a><a class="button-link button is-primary is-light" href="{edit_href}">Edit</a>"#
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
        <section class="app-screen box page-panel">
          <form id="{form_id}" class="entity-form">
            {fields_html}
            <div class="actions form-actions">
              <button class="button is-primary" type="submit">Submit</button>
              <a class="button-link button is-light" href="{cancel_href}">Cancel</a>
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
        <p id="organization-form-status" class="muted">Loading organization schema. No Node ID is required for this page.</p>
        <div class="form-grid">
          <div class="form-field">
            <label id="organization-node-type-label" for="organization-node-type">Node Type</label>
            <select class="input" id="organization-node-type"{disabled}></select>
          </div>
          <div class="form-field">
            <label id="organization-parent-node-label" for="organization-parent-node">Parent Organization</label>
            <select class="input" id="organization-parent-node"></select>
          </div>
          <div class="form-field wide-field">
            <label id="organization-name-label" for="organization-name">Name</label>
            <input class="input" id="organization-name" type="text" autocomplete="off" />
          </div>
        </div>
        <section class="page-panel nested-form-panel">
          <h3 id="organization-metadata-title">Metadata</h3>
          <p id="organization-metadata-context" class="muted">Metadata inputs are generated from the selected node type.</p>
          <div id="organization-metadata-fields" class="form-grid">
            <p class="muted">Choose a node type to load metadata fields.</p>
          </div>
        </section>
        "#
    )
}

#[allow(dead_code)]
fn organization_list_script() -> String {
    r#"
    <script>
      (function () {
        const token = window.sessionStorage.getItem('tessara.devToken');
        const treeRoot = document.getElementById('organization-directory-tree');
        const titleElement = document.getElementById('organization-list-title');
        const contextElement = document.getElementById('organization-list-context');
        const statusElement = document.getElementById('organization-list-status');
        const createLink = document.getElementById('organization-create-link');
        const expandAllButton = document.getElementById('organization-expand-all');
        const collapseAllButton = document.getElementById('organization-collapse-all');

        function escapeHtml(value) {
          return String(value ?? '')
            .replaceAll('&', '&amp;')
            .replaceAll('<', '&lt;')
            .replaceAll('>', '&gt;')
            .replaceAll('"', '&quot;')
            .replaceAll("'", '&#39;');
        }

        function hasCapability(account, capability) {
          return Array.isArray(account?.capabilities)
            && (account.capabilities.includes('admin:all') || account.capabilities.includes(capability));
        }

        function request(path) {
          return fetch(path, {
            headers: token ? { Authorization: `Bearer ${token}` } : {}
          }).then(async (response) => {
            const text = await response.text();
            const payload = text ? JSON.parse(text) : [];
            if (!response.ok) {
              throw new Error(payload?.error || text || `Request failed: ${response.status}`);
            }
            return payload;
          });
        }

        function sortByName(items) {
          return [...items].sort((left, right) => String(left.name || '').localeCompare(String(right.name || '')));
        }

        function deriveScopeLabel(account, nodesById, nodeTypeById) {
          const scopes = Array.isArray(account?.scope_nodes) ? account.scope_nodes : [];
          if (!scopes.length) return 'Organizations';

          const scopedNodes = scopes
            .map((scope) => {
              const node = scope?.node_id ? nodesById.get(scope.node_id) : null;
              if (!node) return null;
              let depth = 0;
              let cursor = node;
              const seen = new Set();
              while (cursor?.parent_node_id && !seen.has(cursor.id)) {
                seen.add(cursor.id);
                const parent = nodesById.get(cursor.parent_node_id);
                if (!parent) break;
                cursor = parent;
                depth += 1;
              }
              return { node, depth };
            })
            .filter(Boolean);

          if (!scopedNodes.length) return 'Organizations';
          scopedNodes.sort((left, right) => left.depth - right.depth || String(left.node.name || '').localeCompare(String(right.node.name || '')));
          const topScopeType = nodeTypeById.get(scopedNodes[0].node.node_type_id);
          return topScopeType?.plural_label || scopedNodes[0].node.node_type_plural_label || 'Organizations';
        }

        function buildTree(nodes) {
          const normalized = (nodes || []).map((node) => ({ ...node, children: [] }));
          const nodeById = new Map(normalized.map((node) => [node.id, node]));
          const roots = [];

          for (const node of normalized) {
            const parent = node.parent_node_id ? nodeById.get(node.parent_node_id) : null;
            if (parent) {
              parent.children.push(node);
            } else {
              roots.push(node);
            }
          }
          for (const node of normalized) {
            node.children = sortByName(node.children);
          }
          return { nodeById, roots: sortByName(roots) };
        }

        function applyCreateShortcut(rootTypes, canCreate) {
          if (!createLink) return;
          if (!canCreate) {
            createLink.remove();
            return;
          }
          if (rootTypes.length === 1) {
            createLink.textContent = `Add ${rootTypes[0].singular_label}`;
            createLink.href = `/app/organization/new?node_type_id=${encodeURIComponent(rootTypes[0].id)}`;
            return;
          }
          createLink.textContent = 'Create Top-Level';
          createLink.href = '/app/organization/new';
        }

        function renderAddButtons(node, nodeTypeById, canCreate) {
          if (!canCreate) return '';
          const nodeType = nodeTypeById.get(node.node_type_id);
          const childTypes = Array.isArray(nodeType?.child_relationships) ? nodeType.child_relationships : [];
          return childTypes.map((childType) => `
            <a class="button-link button is-light" href="/app/organization/new?parent_id=${encodeURIComponent(node.id)}&node_type_id=${encodeURIComponent(childType.node_type_id)}">Add ${escapeHtml(childType.singular_label || childType.node_type_name || 'Child')}</a>
          `).join('');
        }

        function renderNode(node, nodeTypeById, canEdit) {
          const nodeType = nodeTypeById.get(node.node_type_id);
          const childCount = Array.isArray(node.children) ? node.children.length : 0;
          const childTypes = Array.isArray(nodeType?.child_relationships) ? nodeType.child_relationships : [];
          const description = node.parent_node_name
            ? `${nodeType?.singular_label || node.node_type_singular_label || node.node_type_name || 'Organization'} in ${node.parent_node_name} · ${childCount} visible children`
            : `Top-level ${nodeType?.singular_label || node.node_type_singular_label || node.node_type_name || 'organization'} · ${childCount} visible children`;
          const listChildren = node.children && node.children.length
            ? `<div class="organization-tree-children">${node.children.map((child) => renderNode(child, nodeTypeById, canEdit)).join('')}</div>`
            : '';
          return `
            <article class="organization-disclosure-card app-screen box">
              <details class="organization-tree-branch"${childCount > 0 ? ' open' : ''}>
                <summary class="organization-tree-summary">
                  <div class="organization-tree-heading">
                    <strong>${escapeHtml(node.name)}</strong>
                    <p class="muted">${escapeHtml(description)}</p>
                    <p class="muted">Valid lower-level types: ${childTypes.length ? childTypes.map((type) => escapeHtml(type.plural_label || type.node_type_name || 'Children')).join(', ') : 'No lower-level child types'}</p>
                  </div>
                </summary>
                <div class="actions">
                  <a class="button-link" href="/app/organization/${escapeHtml(node.id)}">View</a>
                  ${canEdit ? `<a class="button-link" href="/app/organization/${escapeHtml(node.id)}/edit">Edit</a>` : ''}
                  ${renderAddButtons(node, nodeTypeById, canEdit)}
                </div>
                ${listChildren}
              </details>
            </article>
          `;
        }

        function setDisclosureState(isOpen) {
          document.querySelectorAll('.organization-tree-branch').forEach((element) => {
            if (isOpen) {
              element.setAttribute('open', 'open');
            } else {
              element.removeAttribute('open');
            }
          });
        }

        function wireDisclosureControls() {
          if (expandAllButton) expandAllButton.onclick = () => setDisclosureState(true);
          if (collapseAllButton) collapseAllButton.onclick = () => setDisclosureState(false);
        }

        async function run() {
          if (!treeRoot || !titleElement || !contextElement || !statusElement) return;
          if (!token) {
            statusElement.textContent = 'Sign in to load organization hierarchy.';
            treeRoot.innerHTML = '<p class="muted">Sign in to load scoped organization records.</p>';
            return;
          }

          try {
            statusElement.textContent = 'Loading organization hierarchy.';
            const [account, nodes, nodeTypes] = await Promise.all([
              request('/api/me'),
              request('/api/nodes'),
              request('/api/node-types')
            ]);
            const canEdit = hasCapability(account, 'admin:all');
            const nodeTypeById = new Map((nodeTypes || []).map((nodeType) => [nodeType.id, nodeType]));
            const { nodeById, roots } = buildTree(nodes || []);

            titleElement.textContent = deriveScopeLabel(account, nodeById, nodeTypeById);
            contextElement.textContent = canEdit
              ? 'Expand or collapse the visible hierarchy, move into detail/edit screens, and add valid lower-level records.'
              : 'Expand or collapse the visible hierarchy and move into detail screens.';
            applyCreateShortcut((nodeTypes || []).filter((nodeType) => nodeType.is_root_type), canEdit);

            if (!roots.length) {
              treeRoot.innerHTML = '<p class="muted">No organization records are visible for this scope.</p>';
              statusElement.textContent = 'No organization records found.';
              return;
            }

            treeRoot.innerHTML = roots.map((node) => renderNode(node, nodeTypeById, canEdit)).join('');
            wireDisclosureControls();
            statusElement.textContent = 'Organization hierarchy loaded.';
          } catch (error) {
            statusElement.textContent = error.message;
            treeRoot.innerHTML = `<p class="muted">${escapeHtml(error.message)}</p>`;
          }
        }

        run();
      })();
    </script>
    "#
    .to_string()
}

fn organization_directory_script() -> String {
    r#"
    <script>
      (function () {
        const token = window.sessionStorage.getItem('tessara.devToken');
        const treeRoot = document.getElementById('organization-directory-tree');
        const createLink = document.getElementById('organization-create-link');

        function escapeHtml(value) {
          return String(value ?? '')
            .replaceAll('&', '&amp;')
            .replaceAll('<', '&lt;')
            .replaceAll('>', '&gt;')
            .replaceAll('"', '&quot;')
            .replaceAll("'", '&#39;');
        }

        function hasCapability(account, capability) {
          return Array.isArray(account?.capabilities)
            && (account.capabilities.includes('admin:all') || account.capabilities.includes(capability));
        }

        function request(path) {
          return fetch(path, {
            headers: token ? { Authorization: `Bearer ${token}` } : {}
          }).then(async (response) => {
            const text = await response.text();
            let payload = [];
            try {
              payload = text ? JSON.parse(text) : [];
            } catch (_error) {
              payload = [];
            }
            if (!response.ok) {
              throw new Error(payload?.error || text || 'Unable to load the organization directory.');
            }
            return payload;
          });
        }

        function sortByName(items) {
          return [...items].sort((left, right) => String(left.name || '').localeCompare(String(right.name || '')));
        }

        function buildTree(nodes) {
          const normalized = (nodes || []).map((node) => ({ ...node, children: [] }));
          const nodeById = new Map(normalized.map((node) => [node.id, node]));
          const roots = [];

          for (const node of normalized) {
            const parent = node.parent_node_id ? nodeById.get(node.parent_node_id) : null;
            if (parent) {
              parent.children.push(node);
            } else {
              roots.push(node);
            }
          }
          for (const node of normalized) {
            node.children = sortByName(node.children);
          }
          return { roots: sortByName(roots) };
        }

        function metadataLabel(key) {
          return String(key || '')
            .split(/[_-]+/)
            .filter(Boolean)
            .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
            .join(' ');
        }

        function metadataValue(value) {
          if (Array.isArray(value)) {
            return value.join(', ');
          }
          if (value && typeof value === 'object') {
            return JSON.stringify(value);
          }
          if (typeof value === 'boolean') {
            return value ? 'Yes' : 'No';
          }
          return String(value ?? '');
        }

        function renderMetadataSummary(node) {
          const entries = Object.entries(node?.metadata || {});
          if (!entries.length) {
            return '<p class="muted organization-card-metadata-empty">No metadata values defined.</p>';
          }
          return `
            <div class="organization-card-metadata">
              ${entries.map(([key, value]) => `
                <div class="organization-metadata-chip">
                  <span class="organization-metadata-chip-label">${escapeHtml(metadataLabel(key) || key)}</span>
                  <span class="organization-metadata-chip-value">${escapeHtml(metadataValue(value))}</span>
                </div>
              `).join('')}
            </div>
          `;
        }

        function iconMarkup(isOpen) {
          return `<i class="fa-solid ${isOpen ? 'fa-folder-open' : 'fa-folder'}" aria-hidden="true"></i>`;
        }

        function applyCreateShortcut(rootTypes, canCreate) {
          if (!createLink) return;
          if (!canCreate) {
            createLink.remove();
            return;
          }
          if (rootTypes.length === 1) {
            createLink.textContent = `Add ${rootTypes[0].singular_label || rootTypes[0].name}`;
            createLink.href = `/app/organization/new?node_type_id=${encodeURIComponent(rootTypes[0].id)}`;
            return;
          }
          createLink.textContent = 'Create Top-Level';
          createLink.href = '/app/organization/new';
        }

        function renderAddButtons(node, nodeTypeById, canCreate) {
          if (!canCreate) return '';
          const nodeType = nodeTypeById.get(node.node_type_id);
          const childTypes = Array.isArray(nodeType?.child_relationships) ? nodeType.child_relationships : [];
          return childTypes.map((childType) => `
            <a class="button-link button is-light is-small organization-action-button" href="/app/organization/new?parent_id=${encodeURIComponent(node.id)}&node_type_id=${encodeURIComponent(childType.node_type_id)}">Add ${escapeHtml(childType.singular_label || childType.node_type_name || 'Child')}</a>
          `).join('');
        }

        function renderNode(node, nodeTypeById, canEdit, depth = 0) {
          const nodeType = nodeTypeById.get(node.node_type_id);
          const childCount = Array.isArray(node.children) ? node.children.length : 0;
          const contentId = `organization-node-panel-${node.id}`;
          const description = node.parent_node_name
            ? `${nodeType?.singular_label || node.node_type_singular_label || node.node_type_name || 'Organization'} in ${node.parent_node_name} · ${childCount} visible ${childCount === 1 ? 'child' : 'children'}`
            : `Top-level ${nodeType?.singular_label || node.node_type_singular_label || node.node_type_name || 'organization'} · ${childCount} visible ${childCount === 1 ? 'child' : 'children'}`;
          const listChildren = node.children && node.children.length
            ? `<div class="organization-tree-children">${node.children.map((child) => renderNode(child, nodeTypeById, canEdit, depth + 1)).join('')}</div>`
            : '';
          const depthClass = `organization-depth-level-${Math.min(depth, 3)}`;
          return `
            <article class="organization-disclosure-card ${depthClass} app-screen box">
              <div class="organization-card-shell">
                <div class="organization-card-header">
                  <div class="organization-tree-heading">
                    <strong>${escapeHtml(node.name)}</strong>
                    <p class="muted">${escapeHtml(description)}</p>
                  </div>
                  ${childCount > 0
                    ? `<button type="button" class="organization-toggle-button" data-target="${escapeHtml(contentId)}" data-label="${escapeHtml(node.name)}" aria-expanded="false" aria-label="Expand ${escapeHtml(node.name)}">
                        ${iconMarkup(false)}
                      </button>`
                    : '<span class="organization-toggle-spacer" aria-hidden="true"></span>'}
                </div>
                <div class="actions organization-card-actions organization-card-actions-visible">
                  <a class="button-link button is-light is-small organization-action-button" href="/app/organization/${escapeHtml(node.id)}">View</a>
                  ${canEdit ? `<a class="button-link button is-light is-small organization-action-button" href="/app/organization/${escapeHtml(node.id)}/edit">Edit</a>` : ''}
                  ${renderAddButtons(node, nodeTypeById, canEdit)}
                </div>
                ${renderMetadataSummary(node)}
                <div id="${escapeHtml(contentId)}" class="organization-card-content" hidden>
                  ${listChildren}
                </div>
              </div>
            </article>
          `;
        }

        function wireToggleControls() {
          if (!treeRoot) return;
          function collapseDescendants(container) {
            container.querySelectorAll('.organization-card-content').forEach((content) => {
              content.setAttribute('hidden', 'hidden');
            });
            container.querySelectorAll('.organization-toggle-button').forEach((toggle) => {
              toggle.setAttribute('aria-expanded', 'false');
              toggle.setAttribute('aria-label', `Expand ${toggle.dataset.label || 'organization record'}`);
              toggle.innerHTML = iconMarkup(false);
            });
          }

          treeRoot.onclick = (event) => {
            const button = event.target.closest('.organization-toggle-button');
            if (!button || !treeRoot.contains(button)) return;
            const content = document.getElementById(button.dataset.target || '');
            if (!content) return;
            const isOpen = button.getAttribute('aria-expanded') === 'true';
            button.setAttribute('aria-expanded', String(!isOpen));
            button.setAttribute('aria-label', `${isOpen ? 'Expand' : 'Collapse'} ${button.dataset.label || 'organization record'}`);
            button.innerHTML = iconMarkup(!isOpen);
            if (isOpen) {
              collapseDescendants(content);
              content.setAttribute('hidden', 'hidden');
            } else {
              content.removeAttribute('hidden');
            }
          };
        }

        async function run() {
          if (!treeRoot) return;
          if (!token) {
            treeRoot.innerHTML = '<p class="muted">Sign in to load scoped organization records.</p>';
            return;
          }

          try {
            const [account, nodes, nodeTypes] = await Promise.all([
              request('/api/me'),
              request('/api/nodes'),
              request('/api/node-types')
            ]);
            const canEdit = hasCapability(account, 'admin:all');
            const nodeTypeById = new Map((nodeTypes || []).map((nodeType) => [nodeType.id, nodeType]));
            const { roots } = buildTree(nodes || []);

            applyCreateShortcut((nodeTypes || []).filter((nodeType) => nodeType.is_root_type), canEdit);

            if (!roots.length) {
              treeRoot.innerHTML = '<p class="muted">No organization records are visible for this scope.</p>';
              return;
            }

            treeRoot.innerHTML = roots.map((node) => renderNode(node, nodeTypeById, canEdit)).join('');
            wireToggleControls();
          } catch (error) {
            treeRoot.innerHTML = `<p class="muted">${escapeHtml(error.message)}</p>`;
          }
        }

        run();
      })();
    </script>
    "#
    .to_string()
}

fn organization_detail_script() -> String {
    r#"
    <script>
      (function () {
        const token = window.sessionStorage.getItem('tessara.devToken');
        const recordId = document.body.dataset.recordId || '';
        const statusElement = document.getElementById('organization-detail-status');
        const titleElement = document.getElementById('organization-detail-heading');
        const contextElement = document.getElementById('organization-detail-context');
        const pathElement = document.getElementById('organization-detail-path');
        const summaryElement = document.getElementById('organization-summary');
        const metadataElement = document.getElementById('organization-metadata');
        const relatedElement = document.getElementById('organization-related');
        const childActionsElement = document.getElementById('organization-child-actions');

        function escapeHtml(value) {
          return String(value ?? '')
            .replaceAll('&', '&amp;')
            .replaceAll('<', '&lt;')
            .replaceAll('>', '&gt;')
            .replaceAll('"', '&quot;')
            .replaceAll("'", '&#39;');
        }

        function hasCapability(account, capability) {
          return Array.isArray(account?.capabilities)
            && (account.capabilities.includes('admin:all') || account.capabilities.includes(capability));
        }

        function linkedList(items, hrefPrefix, itemLabel, itemSecondary = null) {
          if (!items.length) {
            return '<li class="muted">No related records.</li>';
          }
          return items.map((item) => {
            const href = `${hrefPrefix}${escapeHtml(item.id || item.submission_id || item.dashboard_id || item.form_id || '')}`;
            const label = escapeHtml(
              item[itemLabel]
              || item.form_name
              || item.dashboard_name
              || item.name
              || 'Unknown'
            );
            const secondary = itemSecondary ? ` (${escapeHtml(item[itemSecondary] || '')})` : '';
            return `<li><a href="${href}">${label}${secondary}</a></li>`;
          }).join('');
        }

        function buildPath(currentNodeId, nodeById) {
          const ancestors = [];
          const seen = new Set();
          let cursor = nodeById.get(currentNodeId);
          while (cursor && !seen.has(cursor.id)) {
            ancestors.unshift(cursor);
            seen.add(cursor.id);
            cursor = cursor.parent_node_id ? nodeById.get(cursor.parent_node_id) : null;
          }
          if (!ancestors.length) {
            return '<p class="muted">No path context available.</p>';
          }
          return `
            <nav class="organization-path-trail" aria-label="Visible organization path">
              ${ancestors.map((node, index) => `
                ${index > 0 ? '<span class="organization-path-separator" aria-hidden="true">&#8250;</span>' : ''}
                <a href="/app/organization/${escapeHtml(node.id)}">${escapeHtml(node.name)}</a>
              `).join('')}
            </nav>
          `;
        }

        async function request(path) {
          const response = await fetch(path, {
            headers: token ? { Authorization: `Bearer ${token}` } : {}
          });
          const text = await response.text();
          if (!response.ok) {
            throw new Error(text || `Request failed: ${response.status}`);
          }
          return text ? JSON.parse(text) : {};
        }

        function renderChildActions(detail, nodeTypeById, canCreate) {
          if (!childActionsElement) return;
          if (!canCreate) {
            childActionsElement.innerHTML = '<p class="muted">Child creation actions are available to administrators.</p>';
            return;
          }
          const nodeType = nodeTypeById.get(detail.node_type_id);
          const childTypes = Array.isArray(nodeType?.child_relationships) ? nodeType.child_relationships : [];
          if (!childTypes.length) {
            childActionsElement.innerHTML = '<p class="muted">This record does not allow lower-level child node types.</p>';
            return;
          }
          childActionsElement.innerHTML = `
            <div class="actions">
              ${childTypes.map((childType) => `
                <a class="button-link button is-light" href="/app/organization/new?parent_id=${encodeURIComponent(detail.id)}&node_type_id=${encodeURIComponent(childType.node_type_id)}">Add ${escapeHtml(childType.singular_label || childType.node_type_name || 'Child')}</a>
              `).join('')}
            </div>
          `;
        }

        async function run() {
          if (!recordId) {
            if (statusElement) statusElement.textContent = 'No Node ID was provided for this detail route.';
            if (summaryElement) summaryElement.innerHTML = '<p class="muted">No Node ID was provided.</p>';
            return;
          }
          if (!token) {
            if (statusElement) statusElement.textContent = 'Sign in to load organization detail.';
            return;
          }

          try {
            if (statusElement) statusElement.textContent = 'Loading organization detail.';
            const [account, detail, nodes, nodeTypes] = await Promise.all([
              request('/api/me'),
              request(`/api/nodes/${recordId}`),
              request('/api/nodes'),
              request('/api/node-types')
            ]);
            const nodeById = new Map((nodes || []).map((item) => [item.id, item]));
            const nodeTypeById = new Map((nodeTypes || []).map((item) => [item.id, item]));
            const nodeType = nodeTypeById.get(detail.node_type_id);
            const canCreate = hasCapability(account, 'admin:all');
            const childrenCount = (nodes || []).filter((item) => item.parent_node_id === recordId).length;
            const siblingNodes = (nodes || []).filter((item) => item.parent_node_id === detail.parent_node_id).length;

            if (pathElement) {
              pathElement.innerHTML = buildPath(recordId, nodeById);
            }
            if (titleElement) {
              titleElement.textContent = `${detail.node_type_singular_label || nodeType?.singular_label || detail.node_type_name || 'Organization'} Detail`;
            }
            if (contextElement) {
              contextElement.textContent = `Visible path, metadata, and linked records for this ${String(detail.node_type_singular_label || nodeType?.singular_label || 'organization').toLowerCase()}.`;
            }
            if (summaryElement) {
              summaryElement.innerHTML = `
                <p>${escapeHtml(detail.name)}</p>
                <p class="muted">Type: ${escapeHtml(detail.node_type_singular_label || nodeType?.singular_label || detail.node_type_name || 'Organization')}</p>
                <p class="muted">Parent: ${escapeHtml(detail.parent_node_name || 'None')}</p>
                <p class="muted">Children: ${childrenCount}</p>
                <p class="muted">Sibling nodes: ${siblingNodes}</p>
              `;
            }
            if (metadataElement) {
              const metadata = detail.metadata || {};
              const keys = Object.keys(metadata);
              metadataElement.innerHTML = keys.length
                ? `<dl class="detail-list">${keys.map((key) => `<div><dt>${escapeHtml(key)}</dt><dd>${escapeHtml(JSON.stringify(metadata[key]))}</dd></div>`).join('')}</dl>`
                : '<p class="muted">No metadata values defined.</p>';
            }
            if (relatedElement) {
              const forms = linkedList(detail.related_forms || [], '/app/forms/', 'form_name');
              const dashboards = linkedList(detail.related_dashboards || [], '/app/dashboards/', 'dashboard_name');
              relatedElement.innerHTML = `
                <section class="app-screen page-panel compact-panel">
                  <h3>Related Forms</h3>
                  <ul class="app-list">${forms}</ul>
                </section>
                <section class="app-screen page-panel compact-panel">
                  <h3>Related Dashboards</h3>
                  <ul class="app-list">${dashboards}</ul>
                </section>
              `;
            }
            renderChildActions(detail, nodeTypeById, canCreate);
            if (statusElement) statusElement.textContent = 'Organization detail loaded.';
          } catch (error) {
            if (statusElement) statusElement.textContent = error.message;
            if (summaryElement) summaryElement.innerHTML = `<p class="muted">${escapeHtml(error.message)}</p>`;
            if (metadataElement) metadataElement.innerHTML = '<p class="muted">Unable to load detail data.</p>';
            if (relatedElement) relatedElement.innerHTML = '';
            if (childActionsElement) childActionsElement.innerHTML = '';
          }
        }

        run();
      })();
    </script>
    "#
        .to_string()
}

fn organization_form_script() -> String {
    r#"
    <script>
      (function () {
        const token = window.sessionStorage.getItem('tessara.devToken');
        const isEdit = (document.body.dataset.recordId || '').length > 0;
        const recordId = document.body.dataset.recordId || '';
        const search = new URLSearchParams(window.location.search);
        const statusElement = document.getElementById('organization-form-status');
        const formElement = document.getElementById('organization-form');
        const nodeTypeSelect = document.getElementById('organization-node-type');
        const parentSelect = document.getElementById('organization-parent-node');
        const nameInput = document.getElementById('organization-name');
        const metadataElement = document.getElementById('organization-metadata-fields');
        const nodeTypeLabel = document.getElementById('organization-node-type-label');
        const parentLabel = document.getElementById('organization-parent-node-label');
        const nameLabel = document.getElementById('organization-name-label');
        const metadataTitle = document.getElementById('organization-metadata-title');
        const metadataContext = document.getElementById('organization-metadata-context');
        const requestedParentId = search.get('parent_id') || '';
        const requestedNodeTypeId = search.get('node_type_id') || '';
        let nodeTypes = [];
        let nodeTypeById = new Map();
        let nodes = [];
        let nodesById = new Map();
        let metadataFields = [];
        let metadataValues = {};
        let currentNodeTypeId = '';

        function escapeHtml(value) {
          return String(value ?? '')
            .replaceAll('&', '&amp;')
            .replaceAll('<', '&lt;')
            .replaceAll('>', '&gt;')
            .replaceAll('"', '&quot;')
            .replaceAll("'", '&#39;');
        }

        function setHtml(id, value) {
          const element = document.getElementById(id);
          if (element) element.innerHTML = value;
        }

        function hasCapability(account, capability) {
          return Array.isArray(account?.capabilities)
            && (account.capabilities.includes('admin:all') || account.capabilities.includes(capability));
        }

        function request(path, options = {}) {
          return fetch(path, {
            ...options,
            headers: {
              ...(options.headers || {}),
              ...(token ? { Authorization: `Bearer ${token}` } : {})
            }
          }).then(async (response) => {
            const text = await response.text();
            const payload = text ? JSON.parse(text) : {};
            if (!response.ok) {
              throw new Error(payload?.error || text || `Request failed: ${response.status}`);
            }
            return payload;
          });
        }

        function populateSelect(element, items, blankLabel = '') {
          if (!element) return;
          const options = [
            blankLabel ? `<option value="">${escapeHtml(blankLabel)}</option>` : '',
            ...items.map((item) => `<option value="${escapeHtml(item.id)}">${escapeHtml(item.label)}</option>`)
          ];
          element.innerHTML = options.join('');
        }

        function sortByName(items) {
          return [...items].sort((left, right) => String(left.name || '').localeCompare(String(right.name || '')));
        }

        function selectedNodeType() {
          return nodeTypeById.get(nodeTypeSelect?.value || currentNodeTypeId || '');
        }

        function availableCreateNodeTypes() {
          if (isEdit && currentNodeTypeId) {
            return nodeTypes.filter((item) => item.id === currentNodeTypeId);
          }
          if (requestedParentId) {
            const parentNode = nodesById.get(requestedParentId);
            const parentType = parentNode ? nodeTypeById.get(parentNode.node_type_id) : null;
            const allowedChildren = Array.isArray(parentType?.child_relationships) ? parentType.child_relationships : [];
            return sortByName(
              allowedChildren
                .map((relationship) => nodeTypeById.get(relationship.node_type_id))
                .filter(Boolean)
            );
          }
          return sortByName(nodeTypes.filter((item) => item.is_root_type));
        }

        function updateLabels(nodeType) {
          const singular = nodeType?.singular_label || 'Organization';
          if (nodeTypeLabel) nodeTypeLabel.textContent = 'Node Type';
          if (parentLabel) parentLabel.textContent = nodeType?.parent_relationships?.length ? `Parent ${singular}` : 'Parent Organization';
          if (nameLabel) nameLabel.textContent = `${singular} Name`;
          if (metadataTitle) metadataTitle.textContent = `${singular} Metadata`;
          if (metadataContext) metadataContext.textContent = `Metadata inputs are generated from the selected ${singular.toLowerCase()} type.`;
        }

        function renderMetadataFields(nodeTypeId, values) {
          const fields = metadataFields.filter((field) => field.node_type_id === nodeTypeId);
          if (!fields.length) {
            setHtml('organization-metadata-fields', '<p class="muted">No metadata fields are configured for this node type.</p>');
            return;
          }
          setHtml(
            'organization-metadata-fields',
            fields.map((field) => {
              const inputId = `organization-metadata-${field.key}`;
              const value = values[field.key];
              const hint = field.required ? 'required' : 'optional';
              if (field.field_type === 'boolean') {
                return `
                  <div class="form-field">
                    <label for="${escapeHtml(inputId)}">${escapeHtml(field.label)} (${hint})</label>
                    <input id="${escapeHtml(inputId)}" type="checkbox" ${value ? 'checked' : ''}>
                  </div>
                `;
              }
              const inputType = field.field_type === 'number'
                ? 'number'
                : field.field_type === 'date'
                  ? 'date'
                  : field.field_type === 'multi_choice'
                    ? 'text'
                    : 'text';
              const placeholder = field.field_type === 'multi_choice'
                ? 'Comma-separated values'
                : field.label;
              const initialValue = Array.isArray(value) ? value.join(', ') : (value || '');
              return `
                <div class="form-field">
                  <label for="${escapeHtml(inputId)}">${escapeHtml(field.label)} (${hint})</label>
                  <input id="${escapeHtml(inputId)}" type="${inputType}" value="${escapeHtml(initialValue)}" placeholder="${escapeHtml(placeholder)}">
                </div>
              `;
            }).join('')
          );
        }

        function collectMetadata(nodeTypeId) {
          const fields = metadataFields.filter((field) => field.node_type_id === nodeTypeId);
          const payload = {};
          for (const field of fields) {
            const element = document.getElementById(`organization-metadata-${field.key}`);
            if (!element) continue;
            if (field.field_type === 'boolean') {
              payload[field.key] = element.checked;
              continue;
            }
            const raw = String(element.value || '').trim();
            if (!raw) continue;
            if (field.field_type === 'number') {
              payload[field.key] = Number(raw);
            } else if (field.field_type === 'multi_choice') {
              payload[field.key] = raw.split(',').map((item) => item.trim()).filter(Boolean);
            } else {
              payload[field.key] = raw;
            }
          }
          return payload;
        }

        function populateNodeTypeOptions(availableTypes) {
          const options = availableTypes.map((item) => ({
            id: item.id,
            label: `${item.singular_label || item.name} (${item.plural_label || item.name})`
          }));
          populateSelect(nodeTypeSelect, options, availableTypes.length > 1 ? 'Choose node type' : '');
          const preferredId = requestedNodeTypeId && availableTypes.some((item) => item.id === requestedNodeTypeId)
            ? requestedNodeTypeId
            : (currentNodeTypeId && availableTypes.some((item) => item.id === currentNodeTypeId) ? currentNodeTypeId : (availableTypes[0]?.id || ''));
          nodeTypeSelect.value = preferredId;
          nodeTypeSelect.disabled = !!isEdit || availableTypes.length <= 1;
        }

        function populateParentOptions(nodeType) {
          const allowedParentTypeIds = Array.isArray(nodeType?.parent_relationships)
            ? nodeType.parent_relationships.map((relationship) => relationship.node_type_id)
            : [];
          if (!allowedParentTypeIds.length) {
            populateSelect(parentSelect, [], 'No parent');
            parentSelect.value = '';
            parentSelect.disabled = true;
            return;
          }
          const options = sortByName(
            nodes.filter((node) => node.id !== recordId && allowedParentTypeIds.includes(node.node_type_id))
          ).map((node) => ({
            id: node.id,
            label: `${node.name} (${node.node_type_singular_label || node.node_type_name || 'Organization'})`
          }));
          populateSelect(parentSelect, options, 'Choose parent');
          parentSelect.disabled = false;
          if (requestedParentId && options.some((option) => option.id === requestedParentId)) {
            parentSelect.value = requestedParentId;
            return;
          }
          if (parentSelect.dataset.initialValue && options.some((option) => option.id === parentSelect.dataset.initialValue)) {
            parentSelect.value = parentSelect.dataset.initialValue;
            return;
          }
          if (!options.some((option) => option.id === parentSelect.value)) {
            parentSelect.value = '';
          }
        }

        function applySelectedNodeType(message) {
          const nodeType = selectedNodeType();
          updateLabels(nodeType);
          populateParentOptions(nodeType);
          renderMetadataFields(nodeType?.id || '', metadataValues);
          if (message) statusElement.textContent = message;
        }

        async function init() {
          if (!formElement || !nodeTypeSelect || !parentSelect || !nameInput || !metadataElement) {
            return;
          }
          if (!token) {
            statusElement.textContent = 'Sign in to access organization forms.';
            return;
          }
          try {
            statusElement.textContent = 'Loading organization schema.';
            const [account, nodeTypeCatalog, allNodes, metadataSchema] = await Promise.all([
              request('/api/me'),
              request('/api/node-types'),
              request('/api/nodes'),
              request('/api/admin/node-metadata-fields')
            ]);
            if (!hasCapability(account, 'admin:all')) {
              statusElement.textContent = 'Administrator access is required for organization changes.';
              return;
            }
            nodeTypes = nodeTypeCatalog || [];
            nodeTypeById = new Map(nodeTypes.map((item) => [item.id, item]));
            nodes = allNodes || [];
            nodesById = new Map(nodes.map((item) => [item.id, item]));
            metadataFields = metadataSchema || [];

            nodeTypeSelect.onchange = () => {
              metadataValues = {};
              const selectedType = selectedNodeType();
              const allowedParentTypeIds = Array.isArray(selectedType?.parent_relationships)
                ? selectedType.parent_relationships.map((relationship) => relationship.node_type_id)
                : [];
              if (parentSelect.value) {
                const selectedParent = nodesById.get(parentSelect.value);
                if (!selectedParent || !allowedParentTypeIds.includes(selectedParent.node_type_id)) {
                  parentSelect.value = '';
                  statusElement.textContent = 'Parent selection was cleared because it is not valid for the selected node type.';
                }
              }
              applySelectedNodeType();
            };

            if (isEdit) {
              const payload = await request(`/api/nodes/${recordId}`);
              currentNodeTypeId = payload.node_type_id || '';
              metadataValues = payload.metadata || {};
              populateNodeTypeOptions(availableCreateNodeTypes());
              parentSelect.dataset.initialValue = payload.parent_node_id || '';
              nameInput.value = payload.name || '';
              applySelectedNodeType();
            } else {
              populateNodeTypeOptions(availableCreateNodeTypes());
              applySelectedNodeType();
            }

            if (!nodeTypeSelect.value) {
              statusElement.textContent = requestedParentId
                ? 'No valid child node types are available for the selected parent.'
                : 'No top-level node types are configured.';
              return;
            }

            const initialType = selectedNodeType();
            const rootTypes = nodeTypes.filter((item) => item.is_root_type);
            if (!isEdit && !requestedParentId) {
              statusElement.textContent = rootTypes.length === 1
                ? `${initialType?.singular_label || 'Organization'} is the only top-level type and has been preselected.`
                : 'Choose a top-level node type before creating a new record.';
            } else if (!isEdit && requestedParentId) {
              const parentNode = nodesById.get(requestedParentId);
              statusElement.textContent = parentNode
                ? `Creating a child record under ${parentNode.name}.`
                : 'Creating a child record.';
            } else {
              statusElement.textContent = 'Organization schema loaded.';
            }
          } catch (error) {
            statusElement.textContent = error.message;
          }

          formElement.onsubmit = async (event) => {
            event.preventDefault();
            try {
              if (!nameInput.value.trim()) {
                statusElement.textContent = `${selectedNodeType()?.singular_label || 'Organization'} name is required.`;
                return;
              }
              const nodeType = selectedNodeType();
              const nodeTypeId = nodeType?.id || '';
              const parentRequired = Array.isArray(nodeType?.parent_relationships) && nodeType.parent_relationships.length > 0;
              if (!nodeTypeId) {
                statusElement.textContent = 'Select a node type before continuing.';
                return;
              }
              if (parentRequired && !parentSelect.value) {
                statusElement.textContent = 'Choose a valid parent before submitting.';
                return;
              }
              const payload = {
                parent_node_id: parentRequired ? (parentSelect.value || null) : null,
                name: nameInput.value.trim(),
                metadata: collectMetadata(nodeTypeId || '')
              };
              if (!isEdit) {
                payload.node_type_id = nodeTypeId;
              }
              statusElement.textContent = isEdit ? 'Saving changes.' : 'Creating organization record.';
              const response = await request(isEdit ? `/api/admin/nodes/${recordId}` : '/api/admin/nodes', {
                method: isEdit ? 'PUT' : 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
              });
              window.location.assign(`/app/organization/${response.id}`);
            } catch (error) {
              statusElement.textContent = error.message;
            }
          };
        }

        init();
      })();
    </script>
    "#
        .to_string()
}

fn form_entity_fields() -> String {
    r#"
        <div class="form-grid">
          <div class="form-field wide-field">
            <label for="form-name">Name</label>
            <input class="input" id="form-name" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="form-slug">Slug</label>
            <input class="input" id="form-slug" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="form-scope-node-type">Scope Node Type</label>
            <select class="input" id="form-scope-node-type"></select>
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
            <select class="input" id="response-form-version"></select>
          </div>
          <div class="form-field">
            <label for="response-node">Target Organization</label>
            <select class="input" id="response-node"></select>
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
            <input class="input" id="report-name" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="report-source-type">Source Type</label>
            <select class="input" id="report-source-type">
              <option value="form">Form</option>
              <option value="dataset">Dataset</option>
            </select>
          </div>
          <div class="form-field">
            <label for="report-source-id">Source</label>
            <select class="input" id="report-source-id"></select>
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
            <input class="input" id="dashboard-name" type="text" autocomplete="off" />
          </div>
        </div>
    "#
    .to_string()
}

fn user_form_fields(is_edit: bool) -> String {
    let password_help = if is_edit {
        "Leave blank to keep the current password."
    } else {
        "Set the password used for local sign-in."
    };
    format!(
        r#"
        <div class="form-grid">
          <div class="form-field wide-field">
            <label for="user-display-name">Display Name</label>
            <input class="input" id="user-display-name" type="text" autocomplete="name" />
          </div>
          <div class="form-field">
            <label for="user-email">Email</label>
            <input class="input" id="user-email" type="email" autocomplete="username" />
          </div>
          <div class="form-field">
            <label for="user-password">Password</label>
            <input class="input" id="user-password" type="password" autocomplete="new-password" />
            <p class="muted">{password_help}</p>
          </div>
          <div class="form-field">
            <label class="checkbox-label" for="user-is-active">
              <input id="user-is-active" type="checkbox" checked>
              Account is active
            </label>
          </div>
        </div>
        <section class="page-panel nested-form-panel">
          <h3>Roles</h3>
          <p class="muted">Select the role memberships that should apply to this account.</p>
          <div id="user-role-options" class="form-grid">
            <p class="muted">Loading roles...</p>
          </div>
        </section>
        "#
    )
}

fn user_access_form_fields() -> String {
    r#"
        <section class="page-panel nested-form-panel">
          <h3>Effective Access</h3>
          <div id="user-access-summary" class="record-detail">
            <p class="muted">Loading current access summary...</p>
          </div>
        </section>
        <section class="page-panel nested-form-panel">
          <h3>Scope Assignments</h3>
          <p class="muted">Assigned scope nodes expand to all descendants automatically.</p>
          <div class="form-grid">
            <div class="form-field wide-field">
              <label for="user-scope-filter">Filter Scope Nodes</label>
              <input class="input" id="user-scope-filter" type="text" autocomplete="off" placeholder="Filter by name, type, or parent" />
            </div>
          </div>
          <div id="user-scope-editability" class="notification is-light">
            Scope assignments load here.
          </div>
          <div class="table-wrap">
            <table class="data-grid">
              <thead>
                <tr>
                  <th>Assigned</th>
                  <th>Node</th>
                  <th>Type</th>
                  <th>Parent</th>
                </tr>
              </thead>
              <tbody id="user-scope-options">
                <tr><td colspan="4" class="muted">Loading available organization nodes...</td></tr>
              </tbody>
            </table>
          </div>
        </section>
        <section class="page-panel nested-form-panel">
          <h3>Delegations</h3>
          <p class="muted">Delegations are generic account relationships. They currently affect delegated response context only.</p>
          <div class="form-grid">
            <div class="form-field wide-field">
              <label for="user-delegation-filter">Filter Delegate Accounts</label>
              <input class="input" id="user-delegation-filter" type="text" autocomplete="off" placeholder="Filter by display name or email" />
            </div>
          </div>
          <div class="table-wrap">
            <table class="data-grid">
              <thead>
                <tr>
                  <th>Assigned</th>
                  <th>Display Name</th>
                  <th>Email</th>
                </tr>
              </thead>
              <tbody id="user-delegation-options">
                <tr><td colspan="3" class="muted">Loading available delegate accounts...</td></tr>
              </tbody>
            </table>
          </div>
        </section>
    "#
    .to_string()
}

fn role_form_fields() -> String {
    r#"
        <section class="page-panel nested-form-panel">
          <h3>Capabilities</h3>
          <p class="muted">Filter and assign the capabilities included in this role bundle.</p>
          <div class="form-grid">
            <div class="form-field wide-field">
              <label for="role-name">Role Name</label>
              <input class="input" id="role-name" type="text" autocomplete="off" />
            </div>
            <div class="form-field wide-field">
              <label for="role-capability-filter">Filter Capabilities</label>
              <input class="input" id="role-capability-filter" type="text" autocomplete="off" placeholder="Filter by key or description" />
            </div>
          </div>
          <div class="table-wrap">
            <table class="data-grid">
              <thead>
                <tr>
                  <th>Assigned</th>
                  <th>Capability</th>
                  <th>Description</th>
                </tr>
              </thead>
              <tbody id="role-capability-options">
                <tr><td colspan="3" class="muted">Loading capabilities...</td></tr>
              </tbody>
            </table>
          </div>
        </section>
    "#
    .to_string()
}

fn node_type_form_fields() -> String {
    r#"
        <p id="node-type-form-status" class="muted">Loading node-type configuration.</p>
        <div class="form-grid">
          <div class="form-field">
            <label for="node-type-name">Name</label>
            <input class="input" id="node-type-name" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="node-type-slug">Slug</label>
            <input class="input" id="node-type-slug" type="text" autocomplete="off" />
          </div>
          <div class="form-field">
            <label for="node-type-plural-label">Plural Label</label>
            <input class="input" id="node-type-plural-label" type="text" autocomplete="off" placeholder="Falls back to a pluralized Name" />
          </div>
        </div>
        <div class="node-type-selector-grid">
          <section class="page-panel nested-form-panel node-type-selector-panel">
            <h3>Allowed Parents</h3>
            <p class="muted">These organization node types may contain this node type. Leave empty to make it top-level.</p>
            <div id="node-type-parent-tags" class="tags organization-node-type-tags">
              <span class="tag is-light">Loading node types...</span>
            </div>
            <div class="form-field">
              <label for="node-type-parent-filter">Add Parent</label>
              <input class="input" id="node-type-parent-filter" type="text" autocomplete="off" placeholder="Search organization node types" />
            </div>
            <div id="node-type-parent-options" class="node-type-option-list">
              <p class="muted">Loading node types...</p>
            </div>
          </section>
          <section class="page-panel nested-form-panel node-type-selector-panel">
            <h3>Allowed Children</h3>
            <p class="muted">These organization node types may be created beneath this node type.</p>
            <div id="node-type-child-tags" class="tags organization-node-type-tags">
              <span class="tag is-light">Loading node types...</span>
            </div>
            <div class="form-field">
              <label for="node-type-child-filter">Add Child</label>
              <input class="input" id="node-type-child-filter" type="text" autocomplete="off" placeholder="Search organization node types" />
            </div>
            <div id="node-type-child-options" class="node-type-option-list">
              <p class="muted">Loading node types...</p>
            </div>
          </section>
        </div>
        <section class="page-panel nested-form-panel node-type-metadata-panel">
          <div class="compact-title-row">
            <div>
              <h3>Metadata Fields</h3>
              <p class="muted">Define the metadata captured for organization nodes of this type. Existing fields can be edited here.</p>
            </div>
            <button type="button" id="node-type-metadata-add" class="button is-light is-small">Add Metadata Field</button>
          </div>
          <div id="node-type-metadata-fields-editor" class="node-type-metadata-editor" role="grid" aria-label="Metadata fields">
            <p class="muted">Loading metadata fields...</p>
          </div>
        </section>
        <div id="node-type-metadata-settings-modal" class="modal">
          <div class="modal-background" data-dismiss="modal"></div>
          <div class="modal-card node-type-metadata-modal-card">
            <header class="modal-card-head">
              <p class="modal-card-title">Field Settings</p>
              <button type="button" class="delete" data-dismiss="modal" aria-label="Close settings"></button>
            </header>
            <section class="modal-card-body">
              <p id="node-type-metadata-settings-title" class="mb-4"></p>
              <label class="checkbox-label" for="node-type-metadata-settings-required">
                <input id="node-type-metadata-settings-required" type="checkbox">
                <span>Required</span>
              </label>
            </section>
            <footer class="modal-card-foot is-justify-content-space-between">
              <button type="button" id="node-type-metadata-settings-remove" class="button is-light">Remove Field</button>
              <div class="actions">
                <button type="button" id="node-type-metadata-settings-cancel" class="button is-light" data-dismiss="modal">Cancel</button>
                <button type="button" id="node-type-metadata-settings-save" class="button is-primary">Save Settings</button>
              </div>
            </footer>
          </div>
        </div>
    "#
    .to_string()
}

pub fn application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Home",
        "Tessara application home for local replacement workflow testing.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn login_application_html(_script: &str) -> String {
    render_application_document(
        "Tessara Sign In",
        "Sign in to the Tessara application shell.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn administration_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Administration",
        "Tessara internal administration landing page.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn users_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara User Management",
        "Browse and manage Tessara user accounts.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="user-list" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "User Management",
            "Manage application users, passwords, active status, and assigned roles from dedicated administration screens.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Users", None),
            ],
            list_screen(
                "Administration",
                "Users",
                "This list screen contains the current local application accounts.",
                "Create User",
                "/app/administration/users/new",
                "user-list",
                "user",
            ),
            false,
        ),
    )
}

pub fn user_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create User",
        "Create a Tessara application account.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="user-create" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "Create User",
            "Create a local application account with login credentials and assigned roles.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Users", Some("/app/administration/users")),
                ("Create User", None),
            ],
            form_screen(
                "Administration",
                "Create User",
                "Create a local account for administration, scoped operational access, or delegated response testing flows.",
                "user-form",
                "/app/administration/users",
                &user_form_fields(false),
            ),
            false,
        ),
    )
}

pub fn user_detail_application_html(_script: &str, account_id: &str) -> String {
    let escaped = escape_attr(account_id);
    render_application_document(
        "User Detail",
        "Inspect a Tessara application account.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="user-detail" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "User Detail",
            "Inspect account status, role memberships, scope assignments, and delegations.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Users", Some("/app/administration/users")),
                ("User Detail", None),
            ],
            detail_screen(
                "Administration",
                "User Detail",
                "This screen is read-only. Use Edit to make changes.",
                "/app/administration/users",
                &format!("/app/administration/users/{escaped}/edit"),
                "user-detail",
                "User Detail",
            ),
            false,
        ),
    )
}

pub fn user_edit_application_html(_script: &str, account_id: &str) -> String {
    let escaped = escape_attr(account_id);
    render_application_document(
        "Edit User",
        "Edit a Tessara application account.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="user-edit" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "Edit User",
            "Update account details, password, active status, and assigned roles.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Users", Some("/app/administration/users")),
                ("Edit User", None),
            ],
            form_screen(
                "Administration",
                "Edit User",
                "Update this account and keep the application in a user-testable state.",
                "user-form",
                &format!("/app/administration/users/{escaped}"),
                &user_form_fields(true),
            ),
            false,
        ),
    )
}

pub fn user_access_application_html(_script: &str, account_id: &str) -> String {
    let escaped = escape_attr(account_id);
    render_application_document(
        "User Access",
        "Manage scoped access assignments for a Tessara application account.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="user-access" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "User Access",
            "Manage scope assignments, delegations, and the effective access this user currently receives from assigned roles.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Users", Some("/app/administration/users")),
                ("User Access", None),
            ],
            form_screen(
                "Administration",
                "User Access",
                "Update the scoped access and delegations that govern what this account can see and who it can act for.",
                "user-access-form",
                &format!("/app/administration/users/{escaped}"),
                &user_access_form_fields(),
            ),
            false,
        ),
    )
}

pub fn node_types_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Organization Node Types",
        "Browse and manage Tessara organization node types.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="node-type-list" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "Organization Node Types",
            "Manage organization node-type naming and hierarchy rules from dedicated administration screens.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Organization Node Types", None),
            ],
            format!(
                r#"
                {}
                {}
                "#,
                page_header(
                    "Administration",
                    "Organization Node Types",
                    "Manage organization node-type naming and hierarchy rules from dedicated administration screens.",
                    r#"<a class="button-link button is-primary" href="/app/administration/node-types/new">Create Organization Node Type</a>"#.to_string()
                ),
                empty_panel(
                    "Organization Node Types",
                    "",
                    r#"<div id="node-type-list" class="record-list"><p class="muted">Loading organization node types...</p></div>"#
                )
            ),
            false,
        ),
    )
}

pub fn node_type_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create Organization Node Type",
        "Create a Tessara organization node type.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="node-type-create" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "Create Organization Node Type",
            "Create a node type, choose its labels, and define the allowed hierarchy relationships.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                (
                    "Organization Node Types",
                    Some("/app/administration/node-types"),
                ),
                ("Create Organization Node Type", None),
            ],
            form_screen(
                "Administration",
                "Create Organization Node Type",
                "Create a node type that product organization screens can use immediately for labels and hierarchy rules.",
                "node-type-form",
                "/app/administration/node-types",
                &node_type_form_fields(),
            ),
            false,
        ),
    )
}

pub fn node_type_detail_application_html(_script: &str, node_type_id: &str) -> String {
    let escaped = escape_attr(node_type_id);
    render_application_document(
        "Organization Node Type Detail",
        "Inspect a Tessara organization node type.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="node-type-detail" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "Organization Node Type Detail",
            "Inspect labels, hierarchy relationships, metadata fields, and scoped forms for this node type.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                (
                    "Organization Node Types",
                    Some("/app/administration/node-types"),
                ),
                ("Organization Node Type Detail", None),
            ],
            detail_screen(
                "Administration",
                "Organization Node Type Detail",
                "This screen is read-only. Use Edit to update labels or relationship rules.",
                "/app/administration/node-types",
                &format!("/app/administration/node-types/{escaped}/edit"),
                "node-type-detail",
                "Organization Node Type Detail",
            ),
            false,
        ),
    )
}

pub fn node_type_edit_application_html(_script: &str, node_type_id: &str) -> String {
    let escaped = escape_attr(node_type_id);
    render_application_document(
        "Edit Organization Node Type",
        "Edit a Tessara organization node type.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="node-type-edit" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "Edit Organization Node Type",
            "Update labels, slugs, and hierarchy rules for this organization node type.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                (
                    "Organization Node Types",
                    Some("/app/administration/node-types"),
                ),
                ("Edit Organization Node Type", None),
            ],
            form_screen(
                "Administration",
                "Edit Organization Node Type",
                "Update this node type so organization list, detail, and create flows use the current terminology and hierarchy rules.",
                "node-type-form",
                &format!("/app/administration/node-types/{escaped}"),
                &node_type_form_fields(),
            ),
            false,
        ),
    )
}

pub fn roles_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Roles",
        "Browse and inspect Tessara role bundles.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="role-list" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "Roles",
            "Review the current role bundles and the capabilities each one grants.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Roles", None),
            ],
            list_screen(
                "Administration",
                "Roles",
                "This list screen contains the current role bundles used for access control.",
                "Create Role",
                "/app/administration/roles/new",
                "role-list",
                "role",
            ),
            false,
        ),
    )
}

pub fn role_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create Role",
        "Create a Tessara role bundle.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="role-create" data-active-route="administration""#,
        app_shell(
            "administration",
            "Internal Area",
            "Create Role",
            "Create a new role bundle and assign the capability set it should grant.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Roles", Some("/app/administration/roles")),
                ("Create Role", None),
            ],
            form_screen(
                "Administration",
                "Create Role",
                "Create a reusable capability bundle for future account assignments.",
                "role-form",
                "/app/administration/roles",
                &role_form_fields(),
            ),
            false,
        ),
    )
}

pub fn role_detail_application_html(_script: &str, role_id: &str) -> String {
    let escaped = escape_attr(role_id);
    render_application_document(
        "Role Detail",
        "Inspect a Tessara role bundle.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="role-detail" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "Role Detail",
            "Inspect the capabilities in this role and the accounts currently assigned to it.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Roles", Some("/app/administration/roles")),
                ("Role Detail", None),
            ],
            detail_screen(
                "Administration",
                "Role Detail",
                "This screen is read-only. Use Edit to change the role bundle.",
                "/app/administration/roles",
                &format!("/app/administration/roles/{escaped}/edit"),
                "role-detail",
                "Role Detail",
            ),
            false,
        ),
    )
}

pub fn role_edit_application_html(_script: &str, role_id: &str) -> String {
    let escaped = escape_attr(role_id);
    render_application_document(
        "Edit Role",
        "Edit a Tessara role bundle.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="role-edit" data-active-route="administration" data-record-id="{escaped}""#
        ),
        app_shell(
            "administration",
            "Internal Area",
            "Edit Role",
            "Update the capability bundle granted by this role.",
            &[
                ("Home", Some("/app")),
                ("Administration", Some("/app/administration")),
                ("Roles", Some("/app/administration/roles")),
                ("Edit Role", None),
            ],
            form_screen(
                "Administration",
                "Edit Role",
                "Choose which capabilities belong to this role bundle.",
                "role-form",
                &format!("/app/administration/roles/{escaped}"),
                &role_form_fields(),
            ),
            false,
        ),
    )
}

pub fn migration_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Migration",
        "Tessara migration workbench.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn organization_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Organizations",
        "Tessara organization list screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        r#"data-page-key="organization-list" data-active-route="organization""#,
        app_shell(
            "organization",
            "Product Area",
            "Organization",
            "Browse runtime organization records and move into their related forms, responses, and dashboards.",
            &[("Home", Some("/app")), ("Organization", None)],
            format!(
                r#"
                {}
                {}
                {}
                "#,
                page_header(
                    "Organization",
                    "Organization",
                    "Browse your scope-aware hierarchy from a full-width navigator.",
                    r#"<a id="organization-create-link" class="button-link button is-primary" href="/app/organization/new">Create Organization</a>"#.to_string(),
                ),
                empty_panel(
                    "Organization Directory",
                    "",
                    r#"
                    <div id="organization-directory-tree" class="organization-disclosure-list">
                      <article class="organization-skeleton-card">
                        <div class="organization-skeleton-heading"></div>
                        <div class="organization-skeleton-line"></div>
                        <div class="organization-skeleton-actions">
                          <span class="organization-skeleton-chip"></span>
                          <span class="organization-skeleton-chip"></span>
                        </div>
                      </article>
                      <article class="organization-skeleton-card organization-skeleton-card-nested">
                        <div class="organization-skeleton-heading"></div>
                        <div class="organization-skeleton-line"></div>
                      </article>
                    </div>
                    "#,
                ),
                organization_directory_script()
            ),
            false,
        ),
    )
}

pub fn organization_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create Organization",
        "Create a runtime organization record.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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
            format!(
                "{}{}",
                form_screen(
                    "Organization",
                    "Create Organization",
                    "Complete the fields below to create a new runtime organization record.",
                    "organization-form",
                    "/app/organization",
                    &organization_form_fields(false),
                ),
                organization_form_script()
            ),
            false,
        ),
    )
}

pub fn organization_detail_application_html(_script: &str, node_id: &str) -> String {
    let escaped = escape_attr(node_id);
    render_application_document(
        "Organization Detail",
        "Organization detail screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
        &format!(
            r#"data-page-key="organization-detail" data-active-route="organization" data-record-id="{escaped}""#
        ),
        app_shell(
            "organization",
            "Product Area",
            "Organization",
            "Review the selected organization record and its visible hierarchy context.",
            &[
                ("Home", Some("/app")),
                ("Organization", Some("/app/organization")),
                ("Organization Detail", None),
            ],
            format!(
                r#"
                {}
                {}
                {}
                {}
                {}
                {}
                <div id="organization-related" class="app-screen"></div>
                <p id="organization-detail-status" class="muted">Loading organization detail...</p>
                {}"#,
                page_header(
                    "Organization",
                    "Organization Detail",
                    "Review details, metadata, path context, and hierarchy-driven child actions for this node.",
                    format!(
                        r#"<a class="button-link button is-light" href="/app/organization">Back to List</a><a class="button-link button is-primary is-light" href="/app/organization/{escaped}/edit">Edit</a>"#
                    ),
                ),
                r#"<section class="app-screen box entity-page"><div class="page-title-row"><div><h2 id="organization-detail-heading">Organization Detail</h2><p id="organization-detail-context" class="muted">Loading visible hierarchy context.</p></div></div></section>"#,
                empty_panel(
                    "Path",
                    "Visible breadcrumb path through the organization hierarchy.",
                    r#"<div id="organization-detail-path"><p class="muted">Loading path...</p></div>"#
                ),
                empty_panel(
                    "Summary",
                    "Primary metadata for the selected organization node.",
                    r#"<div id="organization-summary"><p class="muted">Loading summary...</p></div>"#
                ),
                empty_panel(
                    "Metadata",
                    "Node metadata is rendered from configured node-type fields.",
                    r#"<div id="organization-metadata"><p class="muted">Loading metadata...</p></div>"#
                ),
                empty_panel(
                    "Add Lower-Level Records",
                    "Available child actions are derived from the current node type's configured hierarchy rules.",
                    r#"<div id="organization-child-actions"><p class="muted">Loading available child actions...</p></div>"#
                ),
                organization_detail_script()
            ),
            false,
        ),
    )
}

pub fn organization_edit_application_html(_script: &str, node_id: &str) -> String {
    let escaped = escape_attr(node_id);
    render_application_document(
        "Edit Organization",
        "Edit a runtime organization record.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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
            format!(
                "{}{}",
                form_screen(
                    "Organization",
                    "Edit Organization",
                    "Update the selected runtime organization record and submit to save changes.",
                    "organization-form",
                    &format!("/app/organization/{escaped}"),
                    &organization_form_fields(true),
                ),
                organization_form_script()
            ),
            false,
        ),
    )
}

pub fn forms_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Forms",
        "Tessara forms list screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn form_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create Form",
        "Create a top-level form.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn form_detail_application_html(_script: &str, form_id: &str) -> String {
    let escaped = escape_attr(form_id);
    render_application_document(
        "Form Detail",
        "Form detail screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn form_edit_application_html(_script: &str, form_id: &str) -> String {
    let escaped = escape_attr(form_id);
    render_application_document(
        "Edit Form",
        "Edit a top-level form.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn responses_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Responses",
        "Tessara responses list screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn submission_application_shell_html(_script: &str) -> String {
    responses_application_shell_html(_script)
}

pub fn response_create_application_html(_script: &str) -> String {
    render_application_document(
        "Start Response",
        "Start a new response draft.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn response_detail_application_html(_script: &str, submission_id: &str) -> String {
    let escaped = escape_attr(submission_id);
    render_application_document(
        "Response Detail",
        "Response detail screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn response_edit_application_html(_script: &str, submission_id: &str) -> String {
    let escaped = escape_attr(submission_id);
    render_application_document(
        "Edit Response",
        "Edit a draft response.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn reporting_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Reports",
        "Tessara reports list screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn report_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create Report",
        "Create a top-level report.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn report_detail_application_html(_script: &str, report_id: &str) -> String {
    let escaped = escape_attr(report_id);
    render_application_document(
        "Report Detail",
        "Report detail screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn report_edit_application_html(_script: &str, report_id: &str) -> String {
    let escaped = escape_attr(report_id);
    render_application_document(
        "Edit Report",
        "Edit a top-level report.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn dashboards_application_shell_html(_script: &str) -> String {
    render_application_document(
        "Tessara Dashboards",
        "Tessara dashboards list screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn dashboard_create_application_html(_script: &str) -> String {
    render_application_document(
        "Create Dashboard",
        "Create a top-level dashboard.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn dashboard_detail_application_html(_script: &str, dashboard_id: &str) -> String {
    let escaped = escape_attr(dashboard_id);
    render_application_document(
        "Dashboard Detail",
        "Dashboard detail screen.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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

pub fn dashboard_edit_application_html(_script: &str, dashboard_id: &str) -> String {
    let escaped = escape_attr(dashboard_id);
    render_application_document(
        "Edit Dashboard",
        "Edit a top-level dashboard.",
        &pipeline::bridge_asset_path("app-legacy.js"),
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
