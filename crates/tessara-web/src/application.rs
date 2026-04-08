//! Leptos components for the user-facing Tessara application shell.

use leptos::prelude::*;

use crate::brand::document_head_tags;

/// Builds the application shell document used for human workflow testing.
pub fn application_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <ApplicationShell/> }.to_html();
    let brand = document_head_tags(
        "Tessara App",
        "Tessara submission workspace for local replacement workflow testing.",
    );

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara App</title>
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
fn ApplicationShell() -> impl IntoView {
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
                    <a class="button-link" href="/app/admin">"Open Admin Setup"</a>
                    <a class="button-link" href="/app/reports">"Open Reporting Workspace"</a>
                    <a class="button-link" href="/app/migration">"Open Migration Workbench"</a>
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
                    <a class="button-link" href="/app">"Open Submission Workspace"</a>
                    <a class="button-link" href="/app/reports">"Open Reporting Workspace"</a>
                    <a class="button-link" href="/app/migration">"Open Migration Workbench"</a>
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
                    <a class="button-link" href="/app">"Open Submission Workspace"</a>
                    <a class="button-link" href="/app/admin">"Open Admin Setup"</a>
                    <a class="button-link" href="/app/reports">"Open Reporting Workspace"</a>
                    <a class="button-link" href="/">"Open Admin Workbench"</a>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <h2>"Migration Workflow"</h2>
                    <nav class="app-nav" aria-label="Migration workflow">
                        <a href="#fixture-screen">"Fixtures"</a>
                        <a href="#result-screen">"Results"</a>
                    </nav>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <FixtureScreen/>
                    <section id="result-screen" class="app-screen">
                        <h2>"Validation Results"</h2>
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
                    <a class="button-link" href="/app">"Open Submission Workspace"</a>
                    <a class="button-link" href="/app/admin">"Open Admin Setup"</a>
                    <a class="button-link" href="/app/migration">"Open Migration Workbench"</a>
                    <a class="button-link" href="/">"Open Admin Workbench"</a>
                </div>
            </section>
            <section class="app-layout">
                <aside class="panel app-sidebar">
                    <h2>"Reporting Workflow"</h2>
                    <nav class="app-nav" aria-label="Reporting workflow">
                        <a href="#report-runner-screen">"Reports"</a>
                        <a href="#dashboard-preview-screen">"Dashboards"</a>
                    </nav>
                    <SelectionContext/>
                </aside>
                <section class="panel app-main">
                    <ReportRunnerScreen/>
                    <DashboardPreviewScreen/>
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
            <h3>"Selected Context"</h3>
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
                <label><span>"Aggregation metric kind"</span><input id="aggregation-metric-kind" placeholder="count or sum" value="sum" /></label>
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
                <button type="button" onclick="createDataset()">"Create Dataset"</button>
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
