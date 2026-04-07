//! Leptos shell components for the local Tessara frontend.

use leptos::prelude::*;

const PRIMARY_ACTIONS: &[(&str, &str)] = &[
    ("login()", "Log In"),
    ("seedDemo()", "Seed Demo"),
    ("loadNodeTypes()", "Hierarchy Screen"),
    ("loadForms()", "Forms Screen"),
    ("loadNodes()", "Load Nodes"),
    ("loadSubmissions()", "Load Submissions"),
    ("loadDashboards()", "Load Dashboards"),
    ("loadReports()", "Load Reports"),
    ("loadDashboard()", "Load Demo Dashboard"),
];

const BUILDER_ACTIONS: &[(&str, &str)] = &[
    ("createNodeType()", "Create Node Type"),
    ("loadRelationships()", "Load Relationships"),
    ("createRelationship()", "Create Relationship"),
    ("loadMetadataFields()", "Load Metadata Fields"),
    ("createMetadataField()", "Create Metadata Field"),
    ("createNode()", "Create Node"),
    ("createForm()", "Create Form"),
    ("createFormVersion()", "Create Version"),
    ("createSection()", "Create Section"),
    ("createField()", "Create Field"),
    ("publishVersion()", "Publish Version"),
    ("createReport()", "Create Report"),
    ("createChart()", "Create Chart"),
    ("loadCharts()", "Load Charts"),
    ("createDashboard()", "Create Dashboard"),
    ("addDashboardComponent()", "Add Component"),
    ("createDraft()", "Create Draft"),
    ("saveRenderedFormValues()", "Save Rendered Values"),
    ("saveParticipants()", "Save Participants"),
    ("submitDraft()", "Submit Draft"),
    ("loadSubmissionById()", "Load Submission By ID"),
    ("refreshAnalytics()", "Refresh Analytics"),
    ("loadDashboardById()", "Load Dashboard By ID"),
    ("loadReportById()", "Load Report By ID"),
    ("loadReportDefinitionById()", "Inspect Report By ID"),
    ("validateLegacyFixture()", "Validate Legacy Fixture"),
];

const TEXT_INPUTS: &[TextInput] = &[
    TextInput::new("node-type-name", "Node type name", ""),
    TextInput::new("node-type-slug", "Node type slug", ""),
    TextInput::new("parent-node-type-id", "Parent node type ID", ""),
    TextInput::new("child-node-type-id", "Child node type ID", ""),
    TextInput::new("metadata-node-type-id", "Metadata node type ID", ""),
    TextInput::new("metadata-key", "Metadata key", "region"),
    TextInput::new("metadata-label", "Metadata label", "Region"),
    TextInput::new("metadata-field-type", "Metadata field type", "text"),
    TextInput::new("node-type-id", "Node type ID for node creation", ""),
    TextInput::new("parent-node-id", "Optional parent node ID", ""),
    TextInput::new("node-name", "Node name", "Local Organization"),
    TextInput::new(
        "node-metadata-json",
        "Node metadata JSON, e.g. {\"region\":\"North\"}",
        "{\"region\":\"North\"}",
    ),
    TextInput::new("node-search", "Search nodes", ""),
    TextInput::new("form-name", "Form name", ""),
    TextInput::new("form-slug", "Form slug", ""),
    TextInput::new(
        "form-scope-node-type-id",
        "Optional form scope node type ID",
        "",
    ),
    TextInput::new("form-id", "Form ID", ""),
    TextInput::new("form-version-label", "Form version label", "v1"),
    TextInput::new(
        "compatibility-group-name",
        "Compatibility group name",
        "Default compatibility",
    ),
    TextInput::new("form-version-id", "Published form version ID", ""),
    TextInput::new("section-id", "Section ID", ""),
    TextInput::new("section-title", "Section title", "Main"),
    TextInput::new("field-key", "Field key", "participants"),
    TextInput::new("field-label", "Field label", "Participants"),
    TextInput::new("field-type", "Field type", "number"),
    TextInput::new("report-name", "Report name", "Participants Report"),
    TextInput::new("report-logical-key", "Report logical key", "participants"),
    TextInput::new(
        "report-source-field-key",
        "Report source field key",
        "participants",
    ),
    TextInput::new("report-fields-json", "Optional report bindings JSON", ""),
    TextInput::new("chart-id", "Chart ID", ""),
    TextInput::new("chart-name", "Chart name", "Participants Table"),
    TextInput::new("chart-type", "Chart type", "table"),
    TextInput::new("dashboard-name", "Dashboard name", "Local Dashboard"),
    TextInput::new("node-id", "Target node ID", ""),
    TextInput::new("submission-id", "Draft submission ID", ""),
    TextInput::new("participants-value", "Participants value", "42"),
    TextInput::new(
        "dashboard-id",
        "Dashboard ID from seed or import output",
        "",
    ),
    TextInput::new("report-id", "Report ID from seed or import output", ""),
];

struct TextInput {
    id: &'static str,
    placeholder: &'static str,
    value: &'static str,
}

impl TextInput {
    const fn new(id: &'static str, placeholder: &'static str, value: &'static str) -> Self {
        Self {
            id,
            placeholder,
            value,
        }
    }
}

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
            <section class="panel">
                <p class="muted">"Tessara Core"</p>
                <h1>"Admin Shell"</h1>
                <p>
                    "This is the first local UI surface for the API-first vertical slice. "
                    "It can authenticate with the development admin, seed demo data, and "
                    "inspect the current node and dashboard state."
                </p>
                <ActionBar actions=PRIMARY_ACTIONS/>
                <BuilderInputs/>
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
fn BuilderInputs() -> impl IntoView {
    view! {
        <div class="inputs">
            {TEXT_INPUTS
                .iter()
                .map(|input| {
                    view! {
                        <input id=input.id placeholder=input.placeholder value=input.value />
                    }
                })
                .collect_view()}
            <textarea
                id="legacy-fixture-json"
                placeholder="Paste legacy fixture JSON for validation"
            ></textarea>
            <ActionBar actions=BUILDER_ACTIONS/>
        </div>
    }
}

#[component]
fn ActionBar(actions: &'static [(&'static str, &'static str)]) -> impl IntoView {
    view! {
        <div class="actions">
            {actions
                .iter()
                .map(|(action, label)| {
                    view! {
                        <button type="button" onclick=*action>{*label}</button>
                    }
                })
                .collect_view()}
        </div>
    }
}
