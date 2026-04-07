//! HTML structure for the local Tessara shell.

/// Builds the local shell document from separately maintained style and script
/// assets.
pub fn admin_shell_html(style: &str, script: &str) -> String {
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
    <main class="shell">
      <section class="panel">
        <p class="muted">Tessara Core</p>
        <h1>Admin Shell</h1>
        <p>
          This is the first local UI surface for the API-first vertical slice.
          It can authenticate with the development admin, seed demo data, and
          inspect the current node and dashboard state.
        </p>
        <div class="actions">
          <button type="button" onclick="login()">Log In</button>
          <button type="button" onclick="seedDemo()">Seed Demo</button>
          <button type="button" onclick="loadNodeTypes()">Hierarchy Screen</button>
          <button type="button" onclick="loadForms()">Forms Screen</button>
          <button type="button" onclick="loadNodes()">Load Nodes</button>
          <button type="button" onclick="loadSubmissions()">Load Submissions</button>
          <button type="button" onclick="loadDashboards()">Load Dashboards</button>
          <button type="button" onclick="loadReports()">Load Reports</button>
          <button type="button" onclick="loadDashboard()">Load Demo Dashboard</button>
        </div>
        <div class="inputs">
          <input id="node-type-name" placeholder="Node type name">
          <input id="node-type-slug" placeholder="Node type slug">
          <input id="parent-node-type-id" placeholder="Parent node type ID">
          <input id="child-node-type-id" placeholder="Child node type ID">
          <input id="metadata-node-type-id" placeholder="Metadata node type ID">
          <input id="metadata-key" placeholder="Metadata key" value="region">
          <input id="metadata-label" placeholder="Metadata label" value="Region">
          <input id="metadata-field-type" placeholder="Metadata field type" value="text">
          <input id="form-name" placeholder="Form name">
          <input id="form-slug" placeholder="Form slug">
          <input id="form-scope-node-type-id" placeholder="Optional form scope node type ID">
          <input id="form-id" placeholder="Form ID">
          <input id="form-version-label" placeholder="Form version label" value="v1">
          <input id="compatibility-group-name" placeholder="Compatibility group name" value="Default compatibility">
          <input id="form-version-id" placeholder="Published form version ID">
          <input id="section-id" placeholder="Section ID">
          <input id="section-title" placeholder="Section title" value="Main">
          <input id="field-key" placeholder="Field key" value="participants">
          <input id="field-label" placeholder="Field label" value="Participants">
          <input id="field-type" placeholder="Field type" value="number">
          <input id="report-name" placeholder="Report name" value="Participants Report">
          <input id="report-logical-key" placeholder="Report logical key" value="participants">
          <input id="report-source-field-key" placeholder="Report source field key" value="participants">
          <input id="chart-id" placeholder="Chart ID">
          <input id="chart-name" placeholder="Chart name" value="Participants Table">
          <input id="dashboard-name" placeholder="Dashboard name" value="Local Dashboard">
          <input id="node-id" placeholder="Target node ID">
          <input id="submission-id" placeholder="Draft submission ID">
          <input id="participants-value" placeholder="Participants value" value="42">
          <input id="dashboard-id" placeholder="Dashboard ID from seed or import output">
          <input id="report-id" placeholder="Report ID from seed or import output">
          <div class="actions">
            <button type="button" onclick="createNodeType()">Create Node Type</button>
            <button type="button" onclick="loadRelationships()">Load Relationships</button>
            <button type="button" onclick="createRelationship()">Create Relationship</button>
            <button type="button" onclick="loadMetadataFields()">Load Metadata Fields</button>
            <button type="button" onclick="createMetadataField()">Create Metadata Field</button>
            <button type="button" onclick="createForm()">Create Form</button>
            <button type="button" onclick="createFormVersion()">Create Version</button>
            <button type="button" onclick="createSection()">Create Section</button>
            <button type="button" onclick="createField()">Create Field</button>
            <button type="button" onclick="publishVersion()">Publish Version</button>
            <button type="button" onclick="createReport()">Create Report</button>
            <button type="button" onclick="createChart()">Create Chart</button>
            <button type="button" onclick="createDashboard()">Create Dashboard</button>
            <button type="button" onclick="addDashboardComponent()">Add Component</button>
            <button type="button" onclick="createDraft()">Create Draft</button>
            <button type="button" onclick="saveParticipants()">Save Participants</button>
            <button type="button" onclick="submitDraft()">Submit Draft</button>
            <button type="button" onclick="refreshAnalytics()">Refresh Analytics</button>
            <button type="button" onclick="loadDashboardById()">Load Dashboard By ID</button>
            <button type="button" onclick="loadReportById()">Load Report By ID</button>
          </div>
        </div>
      </section>
      <section class="panel">
        <h2>Screen</h2>
        <div id="screen" class="cards"></div>
      </section>
      <section class="panel">
        <h2>Raw Output</h2>
        <pre id="output">No API calls yet.</pre>
      </section>
    </main>
    <script>{script}</script>
  </body>
</html>"#
    )
}
