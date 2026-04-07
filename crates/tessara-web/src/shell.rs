//! Leptos shell components for the local Tessara frontend.

use leptos::prelude::*;

use crate::shell_model::{Action, PRIMARY_SECTION, WORKFLOW_SECTIONS, WorkflowSection};

/// Builds the local shell document from separately maintained style and script
/// assets.
pub fn admin_shell_html(style: &str, script: &str) -> String {
    let shell = view! { <AdminShell/> }.to_html();

    format!(
        r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara</title>
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
fn AdminShell() -> impl IntoView {
    view! {
        <main class="shell">
            <section class="panel hero">
                <p class="muted">"Tessara Core"</p>
                <h1>"Admin Shell"</h1>
                <p>
                    "This is the first local UI surface for the API-first vertical slice. "
                    "It can authenticate with the development admin, seed demo data, and "
                    "inspect the current node and dashboard state."
                </p>
                <div class="actions">
                    <a class="button-link" href="/app">"Open Application Shell"</a>
                </div>
                <WorkflowSectionView section=&PRIMARY_SECTION/>
            </section>
            <UserTestingGuide/>
            <SelectionContext/>
            <section class="panel">
                <h2>"Builder Workflows"</h2>
                <p class="muted">
                    "These sections follow the migration roadmap slices: hierarchy, forms, "
                    "submissions, reporting/dashboards, and migration rehearsal."
                </p>
                <div class="workflow-grid">
                    {WORKFLOW_SECTIONS
                        .iter()
                        .map(|section| view! { <WorkflowSectionView section/> })
                        .collect_view()}
                </div>
            </section>
            <section class="panel">
                <h2>"Screen"</h2>
                <div id="screen" class="cards"></div>
            </section>
            <section class="panel">
                <h2>"Raw Output"</h2>
                <pre id="output">"No API calls yet."</pre>
            </section>
        </main>
    }
}

#[component]
fn SelectionContext() -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"Selected Context"</h2>
            <p class="muted">
                "Selections from cards populate workflow inputs and are summarized here."
            </p>
            <div id="selection-state" class="selection-grid">
                <p class="muted">"No records selected yet."</p>
            </div>
        </section>
    }
}

#[component]
fn UserTestingGuide() -> impl IntoView {
    view! {
        <section class="panel">
            <h2>"User Testing Guide"</h2>
            <p class="muted">
                "Recommended path for the current Docker Compose test deployment."
            </p>
            <ol class="test-guide">
                <li>"Click Log In to create a development admin session."</li>
                <li>"Click Seed Demo to populate the deterministic hierarchy, form, submission, report, and dashboard."</li>
                <li>"Open Hierarchy Screen, Forms Screen, Load Submissions, Load Reports, and Load Dashboards to inspect seeded records."</li>
                <li>"Use the Form Builder and Submission Workflow sections to create a new form path manually."</li>
                <li>"Use Reports and Dashboards to create and inspect report/chart/dashboard configuration."</li>
                <li>"Paste a legacy fixture into the Migration Workbench to validate migration inputs."</li>
                <li>"Open /app for the replacement-oriented submission workspace."</li>
            </ol>
        </section>
    }
}

#[component]
fn WorkflowSectionView(section: &'static WorkflowSection) -> impl IntoView {
    view! {
        <section class="workflow-section">
            <h3>{section.title}</h3>
            <p class="muted">{section.description}</p>
            <div class="inputs">
                {section
                    .inputs
                    .iter()
                    .map(|input| {
                        view! {
                            <label>
                                <span>{input.label}</span>
                                <input id=input.id placeholder=input.placeholder value=input.value />
                            </label>
                        }
                    })
                    .collect_view()}
                {section
                    .text_area
                    .map(|text_area| {
                        view! {
                            <label>
                                <span>{text_area.label}</span>
                                <textarea
                                    id=text_area.id
                                    placeholder=text_area.placeholder
                                ></textarea>
                            </label>
                        }
                    })}
            </div>
            <ActionBar actions=section.actions/>
        </section>
    }
}

#[component]
fn ActionBar(actions: &'static [Action]) -> impl IntoView {
    view! {
        <div class="actions">
            {actions
                .iter()
                .map(|action| {
                    view! {
                        <button type="button" onclick=action.handler>{action.label}</button>
                    }
                })
                .collect_view()}
        </div>
    }
}
