//! JavaScript controller for the local Tessara shell.

/// Browser-side workflow controller for the local shell.
pub const SCRIPT: &str = r#"
      let token = null;
      let demoDashboardId = null;
      let demoReportId = null;
      let renderedForm = null;

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

      function jsonInputValue(id) {
        const value = inputValue(id);
        return value ? JSON.parse(value) : {};
      }

      function fieldInputId(field) {
        return `form-field-${field.id}`;
      }

      function renderFieldInput(field) {
        const required = field.required ? " required" : "";
        if (field.field_type === "boolean") {
          return `<input id="${escapeHtml(fieldInputId(field))}" type="checkbox"${required}>`;
        }
        const inputType = field.field_type === "number"
          ? "number"
          : field.field_type === "date"
            ? "date"
            : "text";
        const placeholder = field.field_type === "multi_choice"
          ? "Comma-separated choices"
          : field.label;
        return `<input id="${escapeHtml(fieldInputId(field))}" type="${inputType}" placeholder="${escapeHtml(placeholder)}"${required}>`;
      }

      function renderedFields() {
        if (!renderedForm) throw new Error("Render a form version first.");
        return renderedForm.sections.flatMap((section) => section.fields);
      }

      function collectRenderedValues() {
        const values = {};
        for (const field of renderedFields()) {
          const element = document.getElementById(fieldInputId(field));
          if (!element) continue;
          if (field.field_type === "boolean") {
            values[field.key] = element.checked;
            continue;
          }
          const raw = element.value.trim();
          if (raw === "") continue;
          if (field.field_type === "number") {
            values[field.key] = Number(raw);
          } else if (field.field_type === "multi_choice") {
            values[field.key] = raw.split(",").map((item) => item.trim()).filter(Boolean);
          } else {
            values[field.key] = raw;
          }
        }
        return values;
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
              <button type="button" onclick="useNodeType('${escapeHtml(nodeType.id)}')">Use Node Type</button>
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
          document.getElementById("node-type-id").value = payload.id;
          document.getElementById("metadata-node-type-id").value = payload.id;
          show(payload);
          await loadNodeTypes();
        } catch (error) {
          show(error.message);
        }
      }

      function useNodeType(nodeTypeId) {
        document.getElementById("node-type-id").value = nodeTypeId;
        document.getElementById("metadata-node-type-id").value = nodeTypeId;
        document.getElementById("form-scope-node-type-id").value = nodeTypeId;
      }

      async function loadRelationships() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/node-type-relationships");
          show(payload);
          showCards(payload, (relationship) => `
            <article class="card">
              <h3>${escapeHtml(relationship.parent_name)} -> ${escapeHtml(relationship.child_name)}</h3>
              <p class="muted">${escapeHtml(relationship.parent_node_type_id)} -> ${escapeHtml(relationship.child_node_type_id)}</p>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function createRelationship() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/node-type-relationships", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              parent_node_type_id: inputValue("parent-node-type-id"),
              child_node_type_id: inputValue("child-node-type-id")
            })
          });
          show(payload);
          await loadRelationships();
        } catch (error) {
          show(error.message);
        }
      }

      async function loadMetadataFields() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/node-metadata-fields");
          show(payload);
          showCards(payload, (field) => `
            <article class="card">
              <h3>${escapeHtml(field.label)}</h3>
              <p>${escapeHtml(field.node_type_name)}.${escapeHtml(field.key)}</p>
              <p>${escapeHtml(field.field_type)}${field.required ? " required" : ""}</p>
              <code>${escapeHtml(field.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function createMetadataField() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/node-metadata-fields", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              node_type_id: inputValue("metadata-node-type-id"),
              key: inputValue("metadata-key"),
              label: inputValue("metadata-label"),
              field_type: inputValue("metadata-field-type"),
              required: false
            })
          });
          show(payload);
          await loadMetadataFields();
        } catch (error) {
          show(error.message);
        }
      }

      async function createNode() {
        try {
          if (!token) await login();
          const parentNodeId = inputValue("parent-node-id");
          const payload = await request("/api/admin/nodes", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              node_type_id: inputValue("node-type-id"),
              parent_node_id: parentNodeId || null,
              name: inputValue("node-name"),
              metadata: jsonInputValue("node-metadata-json")
            })
          });
          document.getElementById("node-id").value = payload.id;
          show(payload);
          await loadNodes();
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
          renderedForm = payload;
          document.getElementById("form-version-id").value = payload.form_version_id;
          document.getElementById("form-id").value = payload.form_id;
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
                      <li>
                        <label for="${escapeHtml(fieldInputId(field))}">
                          ${escapeHtml(field.label)} (${escapeHtml(field.field_type)})
                        </label>
                        ${renderFieldInput(field)}
                      </li>
                    `).join("")}
                  </ul>
                </section>
              `).join("")}
              <button type="button" onclick="createDraft()">Create Draft</button>
              <button type="button" onclick="saveRenderedFormValues()">Save Values</button>
              <button type="button" onclick="submitDraft()">Submit Draft</button>
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
              <p>${escapeHtml(node.node_type_name)}${node.parent_node_name ? ` under ${escapeHtml(node.parent_node_name)}` : ""}</p>
              <p class="muted">${escapeHtml(JSON.stringify(node.metadata))}</p>
              <button type="button" onclick="useTargetNode('${escapeHtml(node.id)}')">Use Target</button>
              <button type="button" onclick="useParentNode('${escapeHtml(node.id)}')">Use Parent</button>
              <code>${escapeHtml(node.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useTargetNode(nodeId) {
        document.getElementById("node-id").value = nodeId;
      }

      function useParentNode(nodeId) {
        document.getElementById("parent-node-id").value = nodeId;
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
              <button type="button" onclick="loadSubmissionByValue('${escapeHtml(submission.id)}')">Open</button>
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

      async function loadSubmissionById() {
        try {
          if (!token) await login();
          const submissionId = inputValue("submission-id");
          if (!submissionId) throw new Error("Enter a submission ID first.");
          await loadSubmissionByValue(submissionId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadSubmissionByValue(submissionId) {
        if (!token) await login();
        const payload = await request(`/api/submissions/${submissionId}`);
        document.getElementById("submission-id").value = payload.id;
        document.getElementById("form-version-id").value = payload.form_version_id;
        document.getElementById("node-id").value = payload.node_id;
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>${escapeHtml(payload.form_name)} ${escapeHtml(payload.version_label)}</h3>
            <p>${escapeHtml(payload.node_name)}: ${escapeHtml(payload.status)}</p>
            <h4>Values</h4>
            <ul>
              ${payload.values.map((value) => `
                <li>${escapeHtml(value.label)}: ${escapeHtml(JSON.stringify(value.value))}</li>
              `).join("")}
            </ul>
            <h4>Audit</h4>
            <ul>
              ${payload.audit_events.map((event) => `
                <li>${escapeHtml(event.event_type)} by ${escapeHtml(event.account_email || "system")}</li>
              `).join("")}
            </ul>
          </article>
        `;
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

      async function saveRenderedFormValues() {
        try {
          if (!token) await login();
          const submissionId = inputValue("submission-id");
          if (!submissionId) throw new Error("Create or enter a draft submission first.");
          const payload = await request(`/api/submissions/${submissionId}/values`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ values: collectRenderedValues() })
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
              chart_type: inputValue("chart-type") || "table"
            })
          });
          document.getElementById("chart-id").value = payload.id;
          show(payload);
          await loadCharts();
        } catch (error) {
          show(error.message);
        }
      }

      async function loadCharts() {
        try {
          if (!token) await login();
          const payload = await request("/api/charts");
          show(payload);
          showCards(payload, (chart) => `
            <article class="card">
              <h3>${escapeHtml(chart.name)}</h3>
              <p>${escapeHtml(chart.chart_type)} chart</p>
              <p class="muted">Report ${escapeHtml(chart.report_id || "None")}</p>
              <button type="button" onclick="useChart('${escapeHtml(chart.id)}')">Use Chart</button>
              <code>${escapeHtml(chart.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useChart(chartId) {
        document.getElementById("chart-id").value = chartId;
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
"#;
