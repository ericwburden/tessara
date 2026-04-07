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
          <button type="button" onclick="loadNodes()">Load Nodes</button>
          <button type="button" onclick="loadDashboard()">Load Demo Dashboard</button>
        </div>
        <div class="inputs">
          <input id="dashboard-id" placeholder="Dashboard ID from seed or import output">
          <input id="report-id" placeholder="Report ID from seed or import output">
          <div class="actions">
            <button type="button" onclick="loadDashboardById()">Load Dashboard By ID</button>
            <button type="button" onclick="loadReportById()">Load Report By ID</button>
          </div>
        </div>
      </section>
      <section class="panel">
        <h2>Output</h2>
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
          document.getElementById("dashboard-id").value = demoDashboardId;
          document.getElementById("report-id").value = demoReportId;
          show(payload);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadNodes() {
        try {
          show(await request("/api/nodes"));
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
          show(await request(`/api/dashboards/${dashboardId}`));
        } catch (error) {
          show(error.message);
        }
      }

      async function loadReportById() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          if (!reportId) throw new Error("Enter a report ID first.");
          show(await request(`/api/reports/${reportId}/table`));
        } catch (error) {
          show(error.message);
        }
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
        assert!(html.contains("/api/dashboards/"));
        assert!(html.contains("/api/reports/"));
        assert!(html.contains("Dashboard ID from seed or import output"));
    }
}
