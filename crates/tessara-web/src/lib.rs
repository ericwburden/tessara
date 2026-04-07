//! Minimal local UI shell for the API-first Tessara vertical slice.
//!
//! This is intentionally small and static for now. It gives local developers a
//! visible control surface while the future Leptos implementation is still
//! being designed.

/// Returns the static HTML used for the current local admin shell.
///
/// The shell exercises the same API endpoints as the smoke test: development
/// login, deterministic demo seeding, node listing, and dashboard inspection.
pub fn admin_shell_html() -> &'static str {
    r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>Tessara</title>
    <style>
      :root {
        color-scheme: light dark;
        font-family: Inter, ui-sans-serif, system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
      }
      body {
        margin: 0;
        background: #111827;
        color: #f9fafb;
      }
      main {
        max-width: 960px;
        margin: 0 auto;
        padding: 48px 24px;
      }
      .shell {
        display: grid;
        gap: 24px;
      }
      .panel {
        border: 1px solid #374151;
        border-radius: 16px;
        background: #1f2937;
        padding: 24px;
      }
      .cards {
        display: grid;
        gap: 12px;
        grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
      }
      .card {
        border: 1px solid #374151;
        border-radius: 12px;
        background: #111827;
        padding: 16px;
      }
      .actions {
        display: flex;
        flex-wrap: wrap;
        gap: 12px;
        margin-top: 16px;
      }
      .inputs {
        display: grid;
        gap: 12px;
        margin-top: 16px;
      }
      button {
        border: 0;
        border-radius: 999px;
        background: #38bdf8;
        color: #082f49;
        cursor: pointer;
        font-weight: 700;
        padding: 10px 16px;
      }
      input {
        border: 1px solid #4b5563;
        border-radius: 12px;
        background: #111827;
        color: #f9fafb;
        padding: 10px 12px;
      }
      pre {
        overflow: auto;
        border-radius: 12px;
        background: #030712;
        color: #d1d5db;
        padding: 16px;
      }
      .muted {
        color: #9ca3af;
      }
    </style>
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
    <script>
      let token = null;
      let demoDashboardId = null;
      let demoReportId = null;

      function show(value) {
        document.getElementById("output").textContent =
          typeof value === "string" ? value : JSON.stringify(value, null, 2);
      }

      function escapeHtml(value) {
        return String(value ?? "")
          .replaceAll("&", "&amp;")
          .replaceAll("<", "&lt;")
          .replaceAll(">", "&gt;")
          .replaceAll('"', "&quot;")
          .replaceAll("'", "&#39;");
      }

      function showCards(records, render) {
        document.getElementById("screen").innerHTML = records.length
          ? records.map(render).join("")
          : '<p class="muted">No records found.</p>';
      }

      function inputValue(id) {
        return document.getElementById(id).value.trim();
      }

      async function request(path, options = {}) {
        const headers = { ...(options.headers || {}) };
        if (token) headers.Authorization = `Bearer ${token}`;
        const response = await fetch(path, { ...options, headers });
        const text = await response.text();
        const payload = text ? JSON.parse(text) : null;
        if (!response.ok) throw new Error(JSON.stringify(payload, null, 2));
        return payload;
      }

      async function login() {
        try {
          const payload = await request("/api/auth/login", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              email: "admin@tessara.local",
              password: "tessara-dev-admin"
            })
          });
          token = payload.token;
          show({ authenticated: true, token });
        } catch (error) {
          show(error.message);
        }
      }

      async function seedDemo() {
        try {
          if (!token) await login();
          const payload = await request("/api/demo/seed", { method: "POST" });
          demoDashboardId = payload.dashboard_id;
          demoReportId = payload.report_id;
          document.getElementById("form-version-id").value = payload.form_version_id;
          document.getElementById("form-id").value = payload.form_id;
          document.getElementById("node-id").value = payload.organization_node_id;
          document.getElementById("submission-id").value = payload.submission_id;
          document.getElementById("dashboard-id").value = demoDashboardId;
          document.getElementById("report-id").value = demoReportId;
          document.getElementById("chart-id").value = payload.chart_id;
          show(payload);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadNodeTypes() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/node-types");
          show(payload);
          showCards(payload, (nodeType) => `
            <article class="card">
              <h3>${escapeHtml(nodeType.name)}</h3>
              <p class="muted">${escapeHtml(nodeType.slug)}</p>
              <p>${nodeType.node_count} nodes</p>
              <code>${escapeHtml(nodeType.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function createNodeType() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/node-types", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("node-type-name"),
              slug: inputValue("node-type-slug")
            })
          });
          show(payload);
          await loadNodeTypes();
        } catch (error) {
          show(error.message);
        }
      }

      async function loadForms() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/forms");
          show(payload);
          showCards(payload, (form) => `
            <article class="card">
              <h3>${escapeHtml(form.name)}</h3>
              <p class="muted">${escapeHtml(form.slug)}</p>
              <p>Scope: ${escapeHtml(form.scope_node_type_name || "Global")}</p>
              <p>${form.versions.length} versions</p>
              <button type="button" onclick="useForm('${escapeHtml(form.id)}')">Use Form</button>
              <ul>
                ${form.versions.map((version) => `
                  <li>
                    ${escapeHtml(version.version_label)}:
                    ${escapeHtml(version.status)}
                    <button type="button" onclick="renderForm('${escapeHtml(version.id)}')">Render</button>
                  </li>
                `).join("")}
              </ul>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useForm(formId) {
        document.getElementById("form-id").value = formId;
      }

      async function createForm() {
        try {
          if (!token) await login();
          const scopeNodeTypeId = inputValue("form-scope-node-type-id");
          const payload = await request("/api/admin/forms", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("form-name"),
              slug: inputValue("form-slug"),
              scope_node_type_id: scopeNodeTypeId || null
            })
          });
          document.getElementById("form-id").value = payload.id;
          show(payload);
          await loadForms();
        } catch (error) {
          show(error.message);
        }
      }

      async function createFormVersion() {
        try {
          if (!token) await login();
          const formId = inputValue("form-id");
          if (!formId) throw new Error("Create or enter a form ID first.");
          const payload = await request(`/api/admin/forms/${formId}/versions`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              version_label: inputValue("form-version-label"),
              compatibility_group_name: inputValue("compatibility-group-name")
            })
          });
          document.getElementById("form-version-id").value = payload.id;
          show(payload);
          await loadForms();
        } catch (error) {
          show(error.message);
        }
      }

      async function createSection() {
        try {
          if (!token) await login();
          const formVersionId = inputValue("form-version-id");
          if (!formVersionId) throw new Error("Create or enter a form version ID first.");
          const payload = await request(`/api/admin/form-versions/${formVersionId}/sections`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              title: inputValue("section-title"),
              position: 0
            })
          });
          document.getElementById("section-id").value = payload.id;
          show(payload);
          await renderForm(formVersionId);
        } catch (error) {
          show(error.message);
        }
      }

      async function createField() {
        try {
          if (!token) await login();
          const formVersionId = inputValue("form-version-id");
          const sectionId = inputValue("section-id");
          if (!formVersionId || !sectionId) {
            throw new Error("Create or enter a form version ID and section ID first.");
          }
          const payload = await request(`/api/admin/form-versions/${formVersionId}/fields`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              section_id: sectionId,
              key: inputValue("field-key"),
              label: inputValue("field-label"),
              field_type: inputValue("field-type"),
              required: true,
              position: 0
            })
          });
          show(payload);
          await renderForm(formVersionId);
        } catch (error) {
          show(error.message);
        }
      }

      async function publishVersion() {
        try {
          if (!token) await login();
          const formVersionId = inputValue("form-version-id");
          if (!formVersionId) throw new Error("Create or enter a form version ID first.");
          const payload = await request(`/api/admin/form-versions/${formVersionId}/publish`, {
            method: "POST"
          });
          show(payload);
          await loadForms();
        } catch (error) {
          show(error.message);
        }
      }

      async function renderForm(formVersionId) {
        try {
          const payload = await request(`/api/form-versions/${formVersionId}/render`);
          show(payload);
          document.getElementById("screen").innerHTML = `
            <article class="card">
              <h3>Form ${escapeHtml(payload.version_label)}</h3>
              <p>Status: ${escapeHtml(payload.status)}</p>
              ${payload.sections.map((section) => `
                <section>
                  <h4>${escapeHtml(section.title)}</h4>
                  <ul>
                    ${section.fields.map((field) => `
                      <li>${escapeHtml(field.label)} (${escapeHtml(field.field_type)})</li>
                    `).join("")}
                  </ul>
                </section>
              `).join("")}
            </article>
          `;
        } catch (error) {
          show(error.message);
        }
      }

      async function loadNodes() {
        try {
          const payload = await request("/api/nodes");
          show(payload);
          showCards(payload, (node) => `
            <article class="card">
              <h3>${escapeHtml(node.name)}</h3>
              <p class="muted">Node type ${escapeHtml(node.node_type_id)}</p>
              <code>${escapeHtml(node.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadSubmissions() {
        try {
          if (!token) await login();
          const payload = await request("/api/submissions");
          show(payload);
          showCards(payload, (submission) => `
            <article class="card">
              <h3>${escapeHtml(submission.form_name)}</h3>
              <p>${escapeHtml(submission.version_label)} on ${escapeHtml(submission.node_name)}</p>
              <p>Status: ${escapeHtml(submission.status)}</p>
              <p>${submission.value_count} saved values</p>
              <code>${escapeHtml(submission.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function createDraft() {
        try {
          if (!token) await login();
          const payload = await request("/api/submissions/drafts", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              form_version_id: inputValue("form-version-id"),
              node_id: inputValue("node-id")
            })
          });
          document.getElementById("submission-id").value = payload.id;
          show(payload);
          await loadSubmissions();
        } catch (error) {
          show(error.message);
        }
      }

      async function saveParticipants() {
        try {
          if (!token) await login();
          const submissionId = inputValue("submission-id");
          if (!submissionId) throw new Error("Create or enter a draft submission first.");
          const value = Number(inputValue("participants-value"));
          const payload = await request(`/api/submissions/${submissionId}/values`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ values: { participants: value } })
          });
          show(payload);
          await loadSubmissions();
        } catch (error) {
          show(error.message);
        }
      }

      async function submitDraft() {
        try {
          if (!token) await login();
          const submissionId = inputValue("submission-id");
          if (!submissionId) throw new Error("Create or enter a draft submission first.");
          const payload = await request(`/api/submissions/${submissionId}/submit`, { method: "POST" });
          show(payload);
          await loadSubmissions();
        } catch (error) {
          show(error.message);
        }
      }

      async function refreshAnalytics() {
        try {
          if (!token) await login();
          show(await request("/api/admin/analytics/refresh", { method: "POST" }));
        } catch (error) {
          show(error.message);
        }
      }

      async function createReport() {
        try {
          if (!token) await login();
          const formId = inputValue("form-id");
          const payload = await request("/api/admin/reports", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("report-name"),
              form_id: formId || null,
              fields: [{
                logical_key: inputValue("report-logical-key"),
                source_field_key: inputValue("report-source-field-key"),
                missing_policy: "null"
              }]
            })
          });
          document.getElementById("report-id").value = payload.id;
          show(payload);
          await loadReports();
        } catch (error) {
          show(error.message);
        }
      }

      async function createChart() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          const payload = await request("/api/admin/charts", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("chart-name"),
              report_id: reportId || null,
              chart_type: "table"
            })
          });
          document.getElementById("chart-id").value = payload.id;
          show(payload);
        } catch (error) {
          show(error.message);
        }
      }

      async function createDashboard() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/dashboards", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name: inputValue("dashboard-name") })
          });
          document.getElementById("dashboard-id").value = payload.id;
          show(payload);
          await loadDashboards();
        } catch (error) {
          show(error.message);
        }
      }

      async function addDashboardComponent() {
        try {
          if (!token) await login();
          const dashboardId = inputValue("dashboard-id");
          const chartId = inputValue("chart-id");
          if (!dashboardId || !chartId) {
            throw new Error("Create or enter dashboard and chart IDs first.");
          }
          const payload = await request(`/api/admin/dashboards/${dashboardId}/components`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              chart_id: chartId,
              position: 0,
              config: { title: inputValue("chart-name") || "Chart" }
            })
          });
          show(payload);
          await loadDashboardByValue(dashboardId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDashboards() {
        try {
          const payload = await request("/api/dashboards");
          show(payload);
          showCards(payload, (dashboard) => `
            <article class="card">
              <h3>${escapeHtml(dashboard.name)}</h3>
              <p>${dashboard.component_count} components</p>
              <button type="button" onclick="loadDashboardByValue('${escapeHtml(dashboard.id)}')">Open</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadReports() {
        try {
          if (!token) await login();
          const payload = await request("/api/reports");
          show(payload);
          showCards(payload, (report) => `
            <article class="card">
              <h3>${escapeHtml(report.name)}</h3>
              <p class="muted">Form ${escapeHtml(report.form_id || "Any")}</p>
              <button type="button" onclick="loadReportByValue('${escapeHtml(report.id)}')">Run</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDashboard() {
        try {
          if (!demoDashboardId) await seedDemo();
          show(await request(`/api/dashboards/${demoDashboardId}`));
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDashboardById() {
        try {
          const dashboardId = inputValue("dashboard-id");
          if (!dashboardId) throw new Error("Enter a dashboard ID first.");
          await loadDashboardByValue(dashboardId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDashboardByValue(dashboardId) {
        const payload = await request(`/api/dashboards/${dashboardId}`);
        show(payload);
        showCards(payload.components, (component) => `
          <article class="card">
            <h3>${escapeHtml(component.chart.name)}</h3>
            <p>${escapeHtml(component.chart.chart_type)} chart</p>
            <p class="muted">Report ${escapeHtml(component.chart.report_id || "None")}</p>
          </article>
        `);
      }

      async function loadReportById() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          if (!reportId) throw new Error("Enter a report ID first.");
          await loadReportByValue(reportId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadReportByValue(reportId) {
        if (!token) await login();
        const payload = await request(`/api/reports/${reportId}/table`);
        show(payload);
        showCards(payload.rows, (row) => `
          <article class="card">
            <h3>${escapeHtml(row.node_name || "Unknown node")}</h3>
            <p>${escapeHtml(row.logical_key)}: ${escapeHtml(row.field_value)}</p>
            <p class="muted">${escapeHtml(row.submission_id)}</p>
          </article>
        `);
      }
    </script>
  </body>
</html>"#
}

#[cfg(test)]
mod tests {
    use super::admin_shell_html;

    #[test]
    fn shell_links_to_current_demo_api_contract() {
        let html = admin_shell_html();

        assert!(html.contains("/api/auth/login"));
        assert!(html.contains("/api/demo/seed"));
        assert!(html.contains("/api/nodes"));
        assert!(html.contains("/api/admin/node-types"));
        assert!(html.contains("/api/admin/forms"));
        assert!(html.contains("Create Node Type"));
        assert!(html.contains("Create Form"));
        assert!(html.contains("Create Version"));
        assert!(html.contains("Publish Version"));
        assert!(html.contains("Create Report"));
        assert!(html.contains("Create Chart"));
        assert!(html.contains("Add Component"));
        assert!(html.contains("/api/admin/form-versions/"));
        assert!(html.contains("/api/admin/reports"));
        assert!(html.contains("/api/admin/charts"));
        assert!(html.contains("/api/admin/dashboards"));
        assert!(html.contains("/api/form-versions/"));
        assert!(html.contains("/api/submissions"));
        assert!(html.contains("/api/submissions/drafts"));
        assert!(html.contains("/api/admin/analytics/refresh"));
        assert!(html.contains("/api/dashboards/"));
        assert!(html.contains("/api/dashboards"));
        assert!(html.contains("/api/reports/"));
        assert!(html.contains("/api/reports"));
        assert!(html.contains("Dashboard ID from seed or import output"));
        assert!(html.contains("Hierarchy Screen"));
        assert!(html.contains("Forms Screen"));
    }
}
