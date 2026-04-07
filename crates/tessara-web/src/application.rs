//! Leptos components for the user-facing Tessara application shell.

use leptos::prelude::*;

/// Builds the application shell document used for human workflow testing.
pub fn application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <ApplicationShell/> }.to_html();

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara App</title>
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

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara Admin</title>
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
fn ApplicationShell() -> impl IntoView {
    view! {
        <main class="shell app-shell">
            <section class="panel hero">
                <p class="muted">"Tessara Application"</p>
                <h1>"Submission Workspace"</h1>
                <p>
                    "This screen is the first replacement-oriented application surface. "
                    "It uses the same API contracts as the migration workbench, but presents "
                    "the published-form, draft, save, and submit flow as an application task."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <a class="button-link" href="/app/admin">"Open Admin Setup"</a>
                    <a class="button-link" href="/">"Open Admin Workbench"</a>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <h2>"Workflow"</h2>
                    <nav class="app-nav" aria-label="Application workflow">
                        <a href="#submission-screen">"Submit Data"</a>
                        <a href="#review-screen">"Review Submissions"</a>
                        <a href="#report-screen">"View Reports"</a>
                    </nav>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <SubmissionScreen/>
                    <ReviewScreen/>
                    <ReportScreen/>
                    <section class="app-screen">
                        <h2>"Screen Output"</h2>
                        <div id="screen" class="cards"></div>
                    </section>
                    <section class="app-screen">
                        <h2>"Raw Output"</h2>
                        <pre id="output">"No API calls yet."</pre>
                    </section>
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
                <p class="muted">"Tessara Admin"</p>
                <h1>"Setup Workspace"</h1>
                <p>
                    "This screen starts the replacement-oriented admin workflow for configuring "
                    "hierarchy and form definitions without navigating the full workbench."
                </p>
                <div class="actions">
                    <button type="button" onclick="login()">"Log In"</button>
                    <button type="button" onclick="seedDemo()">"Seed Demo"</button>
                    <a class="button-link" href="/app">"Open Submission Workspace"</a>
                    <a class="button-link" href="/">"Open Admin Workbench"</a>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <h2>"Admin Workflow"</h2>
                    <nav class="app-nav" aria-label="Admin application workflow">
                        <a href="#hierarchy-admin-screen">"Hierarchy"</a>
                        <a href="#form-admin-screen">"Forms"</a>
                        <a href="#report-admin-screen">"Reports"</a>
                    </nav>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <HierarchyAdminScreen/>
                    <FormAdminScreen/>
                    <ReportAdminScreen/>
                    <section class="app-screen">
                        <h2>"Screen Output"</h2>
                        <div id="screen" class="cards"></div>
                    </section>
                    <section class="app-screen">
                        <h2>"Raw Output"</h2>
                        <pre id="output">"No API calls yet."</pre>
                    </section>
                </section>
            </section>
        </main>
    }
}

#[component]
fn SelectionContext() -> impl IntoView {
    view! {
        <section class="selection-panel">
            <h3>"Selected Context"</h3>
            <p class="muted">
                "Selections from published forms, nodes, and submissions populate this workflow."
            </p>
            <div id="selection-state" class="selection-grid">
                <p class="muted">"No records selected yet."</p>
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
                <button type="button" onclick="renderForm(inputValue('form-version-id'))">"Open Form"</button>
                <button type="button" onclick="createDraft()">"Create Draft"</button>
                <button type="button" onclick="saveRenderedFormValues()">"Save Values"</button>
                <button type="button" onclick="submitDraft()">"Submit"</button>
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
            <div class="actions">
                <button type="button" onclick="loadSubmissions()">"Load Submissions"</button>
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
                <label><span>"Metadata node type ID"</span><input id="metadata-node-type-id" placeholder="Metadata node type ID" value="" /></label>
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
                <button type="button" onclick="createNodeType()">"Create Node Type"</button>
                <button type="button" onclick="loadMetadataFields()">"Load Metadata Fields"</button>
                <button type="button" onclick="createMetadataField()">"Create Metadata Field"</button>
                <button type="button" onclick="loadNodes()">"Load Nodes"</button>
                <button type="button" onclick="createNode()">"Create Node"</button>
                <button type="button" onclick="updateNode()">"Update Node"</button>
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
                <label><span>"Field ID"</span><input id="field-id" placeholder="Selected field ID" value="" /></label>
                <label><span>"Field key"</span><input id="field-key" placeholder="participants" value="participants" /></label>
                <label><span>"Field label"</span><input id="field-label" placeholder="Participants" value="Participants" /></label>
                <label><span>"Field type"</span><input id="field-type" placeholder="number" value="number" /></label>
                <label><span>"Field required"</span><input id="field-required" placeholder="true or false" value="true" /></label>
            </div>
            <div class="actions">
                <button type="button" onclick="loadForms()">"Load Forms"</button>
                <button type="button" onclick="createForm()">"Create Form"</button>
                <button type="button" onclick="createFormVersion()">"Create Version"</button>
                <button type="button" onclick="createSection()">"Create Section"</button>
                <button type="button" onclick="updateSection()">"Update Section"</button>
                <button type="button" onclick="createField()">"Create Field"</button>
                <button type="button" onclick="updateField()">"Update Field"</button>
                <button type="button" onclick="publishVersion()">"Publish Version"</button>
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
                <label><span>"Report name"</span><input id="report-name" placeholder="Participants Report" value="Participants Report" /></label>
                <label><span>"Report logical key"</span><input id="report-logical-key" placeholder="participants" value="participants" /></label>
                <label><span>"Report source field key"</span><input id="report-source-field-key" placeholder="participants" value="participants" /></label>
                <label><span>"Report missing-data policy"</span><input id="report-missing-policy" placeholder="null" value="null" /></label>
                <label><span>"Report bindings JSON"</span><input id="report-fields-json" placeholder="Optional bindings JSON" value="" /></label>
                <label><span>"Report ID"</span><input id="report-id" placeholder="Selected report ID" value="" /></label>
            </div>
            <div class="actions">
                <button type="button" onclick="addReportBinding()">"Add Binding"</button>
                <button type="button" onclick="clearReportBindings()">"Clear Bindings"</button>
                <button type="button" onclick="createReport()">"Create Report"</button>
                <button type="button" onclick="loadReports()">"Load Reports"</button>
                <button type="button" onclick="loadReportDefinitionById()">"Inspect Report"</button>
                <button type="button" onclick="loadReportById()">"Run Report"</button>
            </div>
        </section>
    }
}
