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
