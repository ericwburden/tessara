//! Browser-side controller for focused application routes.
//!
//! The full admin workbench still uses `shell_script`; this controller keeps the
//! replacement-oriented routes from depending on admin-builder-only functions
//! while the Leptos screens are still server-rendered.

/// Focused JavaScript controller for `/app`, `/app/reports`, and
/// `/app/migration`.
pub const APPLICATION_SCRIPT: &str = r#"
      let token = window.sessionStorage.getItem("tessara.devToken");
      let renderedForm = null;
      let selectedSubmissionFormVersionId = null;
      let selectedSubmissionStatus = null;
      let selectedSubmissionValues = {};
      const selections = {};

      function escapeHtml(value) {
        return String(value ?? "")
          .replaceAll("&", "&amp;")
          .replaceAll("<", "&lt;")
          .replaceAll(">", "&gt;")
          .replaceAll('"', "&quot;")
          .replaceAll("'", "&#39;");
      }

      function show(value) {
        const output = document.getElementById("output");
        if (!output) return;
        output.textContent = typeof value === "string" ? value : JSON.stringify(value, null, 2);
      }

      function setScreen(html) {
        const screen = document.getElementById("screen");
        if (screen) screen.innerHTML = html;
      }

      function showCards(records, render) {
        setScreen(records.length
          ? records.map(render).join("")
          : '<p class="muted">No records found.</p>');
      }

      function reportRowsView(rows) {
        if (rows.length === 0) {
          return '<p class="muted">No submitted rows matched this report.</p>';
        }
        return `
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Node</th>
                  <th>Field</th>
                  <th>Value</th>
                  <th>Submission</th>
                </tr>
              </thead>
              <tbody>
                ${rows.map((row) => `
                  <tr>
                    <td>${escapeHtml(row.node_name || "Unknown node")}</td>
                    <td>${escapeHtml(row.logical_key || "")}</td>
                    <td>${escapeHtml(row.field_value ?? "")}</td>
                    <td>${row.submission_id ? `<button type="button" onclick="loadSubmissionByValue('${escapeHtml(row.submission_id)}')">Open</button>` : '<span class="muted">None</span>'}</td>
                  </tr>
                `).join("")}
              </tbody>
            </table>
          </div>
        `;
      }

      function inputValue(id) {
        return document.getElementById(id)?.value.trim() ?? "";
      }

      function setInput(id, value) {
        const element = document.getElementById(id);
        if (element) element.value = value ?? "";
      }

      function updateSessionStatus(account = null) {
        const element = document.getElementById("session-status");
        if (!element) return;
        if (!token) {
          element.textContent = "Not signed in.";
          return;
        }
        element.textContent = account?.email
          ? `Signed in as ${account.email}.`
          : "Authenticated for local testing.";
      }

      function selectRecord(kind, label, id, bindings = {}) {
        selections[kind] = { label, id };
        for (const [inputId, value] of Object.entries(bindings)) {
          setInput(inputId, value);
        }
        renderSelections();
      }

      function renderSelections() {
        const element = document.getElementById("selection-state");
        if (!element) return;
        const entries = Object.entries(selections);
        element.innerHTML = entries.length
          ? entries.map(([kind, record]) => `
              <article class="selection-item">
                <h3>${escapeHtml(kind)}</h3>
                <p>${escapeHtml(record.label)}</p>
                <code>${escapeHtml(record.id)}</code>
              </article>
            `).join("")
          : '<p class="muted">No records selected yet.</p>';
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
          window.sessionStorage.setItem("tessara.devToken", token);
          updateSessionStatus();
          show({ authenticated: true });
        } catch (error) {
          show(error.message);
        }
      }

      function logout() {
        token = null;
        window.sessionStorage.removeItem("tessara.devToken");
        updateSessionStatus();
        show({ authenticated: false });
      }

      async function loadCurrentUser() {
        try {
          if (!token) await login();
          const payload = await request("/api/me");
          updateSessionStatus(payload);
          show(payload);
          setScreen(`
            <article class="card">
              <h3>${escapeHtml(payload.display_name)}</h3>
              <p>${escapeHtml(payload.email)}</p>
              <p class="muted">${escapeHtml(payload.capabilities.join(", "))}</p>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function seedDemo() {
        try {
          await seedDemoForRoute();
        } catch (error) {
          show(error.message);
        }
      }

      async function startDemoSubmissionFlow() {
        try {
          const payload = await seedDemoForRoute();
          await renderForm(payload.form_version_id);
          await loadSubmissions();
        } catch (error) {
          show(error.message);
        }
      }

      async function openDemoDashboard() {
        try {
          const payload = await seedDemoForRoute();
          await refreshAnalytics();
          await loadDashboardByValue(payload.dashboard_id);
        } catch (error) {
          show(error.message);
        }
      }

      async function seedDemoForRoute() {
        if (!token) await login();
        const payload = await request("/api/demo/seed", { method: "POST" });
        setInput("form-version-id", payload.form_version_id);
        setInput("form-id", payload.form_id);
        setInput("node-id", payload.organization_node_id);
        setInput("submission-id", payload.submission_id);
        setInput("dashboard-id", payload.dashboard_id);
        setInput("report-id", payload.report_id);
        setInput("chart-id", payload.chart_id);
        selectRecord("form version", payload.form_version_id, payload.form_version_id, {
          "form-version-id": payload.form_version_id,
          "form-id": payload.form_id
        });
        selectRecord("node", payload.organization_node_id, payload.organization_node_id, {
          "node-id": payload.organization_node_id
        });
        selectRecord("submission", payload.submission_id, payload.submission_id, {
          "submission-id": payload.submission_id
        });
        selectRecord("report", payload.report_id, payload.report_id, {
          "report-id": payload.report_id
        });
        selectRecord("chart", payload.chart_id, payload.chart_id, {
          "chart-id": payload.chart_id
        });
        selectRecord("dashboard", payload.dashboard_id, payload.dashboard_id, {
          "dashboard-id": payload.dashboard_id
        });
        show(payload);
        return payload;
      }

      async function loadAppSummary() {
        try {
          if (!token) await login();
          const payload = await request("/api/app/summary");
          show(payload);
          showCards([
            ["Published forms", payload.published_form_versions],
            ["Draft submissions", payload.draft_submissions],
            ["Submitted submissions", payload.submitted_submissions],
            ["Reports", payload.reports],
            ["Dashboards", payload.dashboards],
            ["Charts", payload.charts]
          ], ([label, count]) => `
            <article class="card">
              <h3>${escapeHtml(label)}</h3>
              <p>${escapeHtml(count)}</p>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useForm(formId, formName = formId) {
        selectRecord("form", formName, formId, { "form-id": formId });
      }

      function useFormVersion(formVersionId, formId, label = formVersionId) {
        selectRecord("form version", label, formVersionId, {
          "form-version-id": formVersionId,
          "form-id": formId
        });
      }

      async function loadPublishedForms() {
        try {
          const payload = await request("/api/forms/published");
          show(payload);
          showCards(payload, (formVersion) => `
            <article class="card">
              <h3>${escapeHtml(formVersion.form_name)}</h3>
              <p class="muted">${escapeHtml(formVersion.form_slug)} ${escapeHtml(formVersion.version_label)}</p>
              <p>${formVersion.field_count} fields</p>
              <button type="button" onclick="useFormVersion('${escapeHtml(formVersion.form_version_id)}', '${escapeHtml(formVersion.form_id)}', '${escapeHtml(formVersion.form_name)} ${escapeHtml(formVersion.version_label)}')">Use Published Version</button>
              <button type="button" onclick="renderForm('${escapeHtml(formVersion.form_version_id)}')">Render Form</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadNodes() {
        try {
          const search = inputValue("node-search");
          const payload = await request(search ? `/api/nodes?q=${encodeURIComponent(search)}` : "/api/nodes");
          show(payload);
          showCards(payload, (node) => `
            <article class="card">
              <h3>${escapeHtml(node.name)}</h3>
              <p>${escapeHtml(node.node_type_name)}${node.parent_node_name ? ` under ${escapeHtml(node.parent_node_name)}` : ""}</p>
              <p class="muted">${escapeHtml(JSON.stringify(node.metadata))}</p>
              <button type="button" onclick="useTargetNode('${escapeHtml(node.id)}', '${escapeHtml(node.name)}')">Use Target</button>
              <code>${escapeHtml(node.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useTargetNode(nodeId, nodeName = nodeId) {
        selectRecord("node", nodeName, nodeId, {
          "node-id": nodeId,
          "node-name": nodeName
        });
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

      async function renderForm(formVersionId) {
        try {
          const payload = await request(`/api/form-versions/${formVersionId}/render`);
          renderedForm = payload;
          setInput("form-version-id", payload.form_version_id);
          setInput("form-id", payload.form_id);
          useFormVersion(payload.form_version_id, payload.form_id, `${payload.form_name} ${payload.version_label}`);
          show(payload);
          setScreen(`
            <article class="card form-screen">
              <h3>${escapeHtml(payload.form_name)} ${escapeHtml(payload.version_label)}</h3>
              <p>Status: ${escapeHtml(payload.status)}</p>
              <p class="muted">Target node: ${escapeHtml(selections.node?.label || inputValue("node-id") || "Select a node before creating a draft.")}</p>
              <p class="muted">Draft submission: ${escapeHtml(inputValue("submission-id") || "Create a draft after selecting a node.")}</p>
              ${payload.sections.map((section) => `
                <section class="form-section">
                  <h4>${escapeHtml(section.title)}</h4>
                  <div class="form-fields">
                    ${section.fields.map((field) => `
                      <div class="form-field">
                        <label for="${escapeHtml(fieldInputId(field))}">
                          ${escapeHtml(field.label)} (${escapeHtml(field.field_type)}${field.required ? ", required" : ""})
                        </label>
                        ${renderFieldInput(field)}
                      </div>
                    `).join("")}
                  </div>
                </section>
              `).join("")}
              ${renderResponseFormActions()}
            </article>
          `);
          prefillRenderedValues();
        } catch (error) {
          show(error.message);
        }
      }

      function renderResponseFormActions() {
        const hasMatchingSubmission = inputValue("submission-id")
          && selectedSubmissionFormVersionId === renderedForm?.form_version_id;
        if (hasMatchingSubmission && selectedSubmissionStatus === "submitted") {
          return `
            <div class="actions form-actions">
              <p class="muted">This submitted response is read-only. Open a draft submission to edit values.</p>
              <button type="button" onclick="clearResponseContext()">Clear Response Context</button>
            </div>
          `;
        }
        if (hasMatchingSubmission) {
          return `
            <div class="actions form-actions">
              <button type="button" onclick="saveRenderedFormValues()">Save Values</button>
              <button type="button" onclick="submitDraft()">Submit Draft</button>
              <button type="button" onclick="discardDraft()">Discard Draft</button>
              <button type="button" onclick="clearResponseContext()">Clear Response Context</button>
            </div>
          `;
        }
        return `
          <div class="actions form-actions">
            <button type="button" onclick="createDraft()">Create Draft</button>
          </div>
        `;
      }

      function clearResponseContext() {
        setInput("submission-id", "");
        selectedSubmissionFormVersionId = null;
        selectedSubmissionStatus = null;
        selectedSubmissionValues = {};
        show({ response_context: "cleared" });
        if (renderedForm) renderForm(renderedForm.form_version_id);
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

      function submissionValuesByKey(values) {
        return Object.fromEntries(
          values
            .filter((value) => value.value !== null)
            .map((value) => [value.key, value.value])
        );
      }

      function prefillRenderedValues() {
        if (!renderedForm || selectedSubmissionFormVersionId !== renderedForm.form_version_id) return;
        for (const field of renderedFields()) {
          const value = selectedSubmissionValues[field.key];
          if (value === undefined || value === null) continue;
          const element = document.getElementById(fieldInputId(field));
          if (!element) continue;
          if (field.field_type === "boolean") {
            element.checked = Boolean(value);
          } else if (Array.isArray(value)) {
            element.value = value.join(", ");
          } else {
            element.value = String(value);
          }
        }
      }

      function validateRenderedValues(values) {
        const missing = renderedFields()
          .filter((field) => field.required)
          .filter((field) => {
            const value = values[field.key];
            return value === undefined
              || value === null
              || value === ""
              || (Array.isArray(value) && value.length === 0);
          })
          .map((field) => field.label);

        if (missing.length > 0) {
          throw new Error(`Required fields missing: ${missing.join(", ")}`);
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
          setInput("submission-id", payload.id);
          selectedSubmissionFormVersionId = renderedForm?.form_version_id ?? inputValue("form-version-id");
          selectedSubmissionStatus = "draft";
          selectedSubmissionValues = {};
          selectRecord("submission", payload.id, payload.id, { "submission-id": payload.id });
          show(payload);
          await loadSubmissionByValue(payload.id);
        } catch (error) {
          show(error.message);
        }
      }

      async function saveRenderedFormValues() {
        try {
          if (!token) await login();
          const submissionId = inputValue("submission-id");
          if (!submissionId) throw new Error("Create or enter a draft submission first.");
          const values = collectRenderedValues();
          validateRenderedValues(values);
          const payload = await request(`/api/submissions/${submissionId}/values`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ values })
          });
          show(payload);
          await loadSubmissionByValue(submissionId);
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
          await loadSubmissionByValue(submissionId);
        } catch (error) {
          show(error.message);
        }
      }

      async function discardDraft() {
        try {
          if (!token) await login();
          const submissionId = inputValue("submission-id");
          if (!submissionId) throw new Error("Create or enter a draft submission first.");
          const payload = await request(`/api/submissions/${submissionId}`, { method: "DELETE" });
          setInput("submission-id", "");
          selectedSubmissionFormVersionId = null;
          selectedSubmissionStatus = null;
          selectedSubmissionValues = {};
          show(payload);
          await loadSubmissions();
        } catch (error) {
          show(error.message);
        }
      }

      async function loadSubmissions() {
        try {
          if (!token) await login();
          const payload = await request(submissionsUrl());
          show(payload);
          showCards(payload, (submission) => `
            <article class="card">
              <h3>${escapeHtml(submission.form_name)}</h3>
              <p>${escapeHtml(submission.version_label)} on ${escapeHtml(submission.node_name)}</p>
              <p>Status: ${escapeHtml(submission.status)}</p>
              <p>${submission.value_count} saved values</p>
              <p class="muted">Created ${escapeHtml(submission.created_at)}${submission.submitted_at ? `; submitted ${escapeHtml(submission.submitted_at)}` : ""}</p>
              <button type="button" onclick="useSubmission('${escapeHtml(submission.id)}', '${escapeHtml(submission.form_name)} ${escapeHtml(submission.version_label)}')">Use Submission</button>
              <button type="button" onclick="useFormVersion('${escapeHtml(submission.form_version_id)}', '${escapeHtml(submission.form_id)}', '${escapeHtml(submission.form_name)} ${escapeHtml(submission.version_label)}')">Use Form Version</button>
              <button type="button" onclick="useTargetNode('${escapeHtml(submission.node_id)}', '${escapeHtml(submission.node_name)}')">Use Node</button>
              <button type="button" onclick="loadSubmissionByValue('${escapeHtml(submission.id)}')">Open</button>
              <code>${escapeHtml(submission.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function submissionsUrl() {
        const params = new URLSearchParams();
        if (inputValue("submission-search")) params.set("q", inputValue("submission-search"));
        if (inputValue("submission-status-filter")) params.set("status", inputValue("submission-status-filter"));
        if (inputValue("form-id")) params.set("form_id", inputValue("form-id"));
        if (inputValue("node-id")) params.set("node_id", inputValue("node-id"));
        const query = params.toString();
        return query ? `/api/submissions?${query}` : "/api/submissions";
      }

      function filterSubmissionsByStatus(status) {
        setInput("submission-status-filter", status);
        loadSubmissions();
      }

      function showDraftSubmissions() {
        filterSubmissionsByStatus("draft");
      }

      function showSubmittedSubmissions() {
        filterSubmissionsByStatus("submitted");
      }

      function clearSubmissionReviewFilters() {
        setInput("submission-search", "");
        setInput("submission-status-filter", "");
        loadSubmissions();
      }

      function useSubmission(submissionId, label = submissionId) {
        selectRecord("submission", label, submissionId, { "submission-id": submissionId });
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
        setInput("submission-id", payload.id);
        setInput("form-version-id", payload.form_version_id);
        setInput("node-id", payload.node_id);
        selectedSubmissionFormVersionId = payload.form_version_id;
        selectedSubmissionStatus = payload.status;
        selectedSubmissionValues = submissionValuesByKey(payload.values);
        useSubmission(payload.id, `${payload.form_name} ${payload.version_label}`);
        useFormVersion(payload.form_version_id, payload.form_id, `${payload.form_name} ${payload.version_label}`);
        useTargetNode(payload.node_id, payload.node_name);
        show(payload);
        setScreen(`
          <article class="card">
            <h3>${escapeHtml(payload.form_name)} ${escapeHtml(payload.version_label)}</h3>
            <p>${escapeHtml(payload.node_name)}: ${escapeHtml(payload.status)}</p>
            <p class="muted">Created ${escapeHtml(payload.created_at)}${payload.submitted_at ? `; submitted ${escapeHtml(payload.submitted_at)}` : ""}</p>
            <button type="button" onclick="renderForm('${escapeHtml(payload.form_version_id)}')">Open Response Form</button>
            <h4>Values</h4>
            <ul>
              ${payload.values.map((value) => `<li>${escapeHtml(value.label)}${value.required ? " *" : ""}: ${value.value === null ? "<span class=\"muted\">missing</span>" : escapeHtml(JSON.stringify(value.value))}</li>`).join("")}
            </ul>
            <h4>Audit</h4>
            <ul>
              ${payload.audit_events.map((event) => `<li>${escapeHtml(event.event_type)} by ${escapeHtml(event.account_email || "system")}</li>`).join("")}
            </ul>
          </article>
        `);
      }

      async function refreshAnalytics() {
        try {
          if (!token) await login();
          show(await request("/api/admin/analytics/refresh", { method: "POST" }));
          await loadAppSummary();
        } catch (error) {
          show(error.message);
        }
      }

      function useReport(reportId, reportName = reportId) {
        selectRecord("report", reportName, reportId, {
          "report-id": reportId,
          "report-name": reportName
        });
      }

      async function loadReports() {
        try {
          if (!token) await login();
          const payload = await request("/api/reports");
          show(payload);
          showCards(payload, (report) => `
            <article class="card">
              <h3>${escapeHtml(report.name)}</h3>
              <p class="muted">Form ${escapeHtml(report.form_name || report.form_id || "Any")}</p>
              <button type="button" onclick="useReport('${escapeHtml(report.id)}', '${escapeHtml(report.name)}'); ${report.form_id ? `useForm('${escapeHtml(report.form_id)}', '${escapeHtml(report.form_name || report.form_id)}');` : ""}">Use Report Context</button>
              <button type="button" onclick="loadReportDefinition('${escapeHtml(report.id)}')">Inspect</button>
              <button type="button" onclick="loadReportByValue('${escapeHtml(report.id)}')">Run</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadReportDefinition(reportId) {
        if (!token) await login();
        const payload = await request(`/api/reports/${reportId}`);
        setInput("report-id", payload.id);
        if (payload.form_id) setInput("form-id", payload.form_id);
        useReport(payload.id, payload.name);
        if (payload.form_id) useForm(payload.form_id, payload.form_name || payload.form_id);
        setInput("report-fields-json", JSON.stringify(payload.bindings.map((binding) => ({
          logical_key: binding.logical_key,
          source_field_key: binding.source_field_key,
          missing_policy: binding.missing_policy
        }))));
        show(payload);
        showCards(payload.bindings, (binding) => `
          <article class="card">
            <h3>${escapeHtml(binding.logical_key)}</h3>
            <p>${escapeHtml(binding.source_field_key)} with ${escapeHtml(binding.missing_policy)}</p>
            <button type="button" onclick="useReportBinding('${escapeHtml(binding.logical_key)}', '${escapeHtml(binding.source_field_key)}', '${escapeHtml(binding.missing_policy)}')">Use Binding</button>
          </article>
        `);
      }

      function useReportBinding(logicalKey, sourceFieldKey, missingPolicy) {
        selectRecord("report binding", logicalKey, sourceFieldKey, {
          "report-logical-key": logicalKey,
          "report-source-field-key": sourceFieldKey,
          "report-missing-policy": missingPolicy
        });
      }

      async function loadReportDefinitionById() {
        try {
          const reportId = inputValue("report-id");
          if (!reportId) throw new Error("Enter a report ID first.");
          await loadReportDefinition(reportId);
        } catch (error) {
          show(error.message);
        }
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
        useReport(reportId);
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Report Results</h3>
            <p>${payload.rows.length} rows returned.</p>
            ${reportRowsView(payload.rows)}
          </article>
        `);
      }

      function useChart(chartId, chartName = chartId, reportId = "", reportName = reportId, chartType = "table") {
        selectRecord("chart", chartName, chartId, {
          "chart-id": chartId,
          "chart-name": chartName,
          "chart-type": chartType,
          ...(reportId ? { "report-id": reportId } : {})
        });
        if (reportId) useReport(reportId, reportName || reportId);
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
              <p class="muted">Report ${escapeHtml(chart.report_name || "None")}${chart.report_form_name ? ` on ${escapeHtml(chart.report_form_name)}` : ""}</p>
              <button type="button" onclick="useChart('${escapeHtml(chart.id)}', '${escapeHtml(chart.name)}', '${escapeHtml(chart.report_id || "")}', '${escapeHtml(chart.report_name || "")}', '${escapeHtml(chart.chart_type)}')">Use Chart Context</button>
              ${chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(chart.report_id)}')">Run Report</button>` : ""}
              <code>${escapeHtml(chart.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useDashboard(dashboardId, dashboardName = dashboardId) {
        selectRecord("dashboard", dashboardName, dashboardId, {
          "dashboard-id": dashboardId,
          "dashboard-name": dashboardName
        });
      }

      async function loadDashboards() {
        try {
          const payload = await request("/api/dashboards");
          show(payload);
          showCards(payload, (dashboard) => `
            <article class="card">
              <h3>${escapeHtml(dashboard.name)}</h3>
              <p>${dashboard.component_count} components</p>
              <button type="button" onclick="useDashboard('${escapeHtml(dashboard.id)}', '${escapeHtml(dashboard.name)}')">Use Dashboard</button>
              <button type="button" onclick="loadDashboardByValue('${escapeHtml(dashboard.id)}')">Open</button>
            </article>
          `);
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
        if (!token) await login();
        const payload = await request(`/api/dashboards/${dashboardId}`);
        useDashboard(payload.id, payload.name);
        show(payload);
        const cards = await Promise.all(payload.components.map(async (component) => {
          let rows = [];
          const componentTitle = component.config?.title || component.chart.name;
          if (component.chart.report_id) {
            const report = await request(`/api/reports/${component.chart.report_id}/table`);
            rows = report.rows;
          }
          return `
            <article class="card">
              <h3>${escapeHtml(componentTitle)}</h3>
              <p>${escapeHtml(component.chart.chart_type)} chart</p>
              <p class="muted">Chart ${escapeHtml(component.chart.name)}</p>
              <p>Position ${component.position}</p>
              <p class="muted">Report ${escapeHtml(component.chart.report_name || component.chart.report_id || "None")}${component.chart.report_form_name ? ` on ${escapeHtml(component.chart.report_form_name)}` : ""}</p>
              <button type="button" onclick="useChart('${escapeHtml(component.chart.id)}', '${escapeHtml(component.chart.name)}', '${escapeHtml(component.chart.report_id || "")}', '${escapeHtml(component.chart.report_name || "")}', '${escapeHtml(component.chart.chart_type)}')">Use Chart Context</button>
              ${component.chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(component.chart.report_id)}')">Open Report</button>` : ""}
              ${reportRowsView(rows)}
            </article>
          `;
        }));
        setScreen(cards.length ? cards.join("") : '<p class="muted">No dashboard components found.</p>');
      }

      async function loadLegacyFixtureExamples() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/legacy-fixtures/examples");
          show(payload.map((fixture) => ({ name: fixture.name, bytes: fixture.fixture_json.length })));
          showCards(payload, (fixture) => `
            <article class="card">
              <h3>${escapeHtml(fixture.name)}</h3>
              <p>${fixture.fixture_json.length} bytes</p>
              <button type="button" onclick="useLegacyFixture('${escapeHtml(fixture.name)}')">Use Fixture</button>
            </article>
          `);
          window.legacyFixtureExamples = Object.fromEntries(payload.map((fixture) => [fixture.name, fixture.fixture_json]));
        } catch (error) {
          show(error.message);
        }
      }

      function useLegacyFixture(name) {
        const fixture = window.legacyFixtureExamples?.[name];
        if (!fixture) {
          show(`Fixture example '${name}' has not been loaded.`);
          return;
        }
        setInput("legacy-fixture-json", fixture);
        show({ selected_fixture: name });
      }

      async function validateLegacyFixture() {
        try {
          if (!token) await login();
          const fixtureJson = inputValue("legacy-fixture-json");
          if (!fixtureJson) throw new Error("Paste legacy fixture JSON first.");
          const payload = await request("/api/admin/legacy-fixtures/validate", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ fixture_json: fixtureJson })
          });
          show(payload);
          showCards(payload.issues, (issue) => `
            <article class="card">
              <h3>${escapeHtml(issue.code)}</h3>
              <p>${escapeHtml(issue.path)}</p>
              <p>${escapeHtml(issue.message)}</p>
            </article>
          `);
          if (payload.issue_count === 0) {
            setScreen('<p class="muted">Legacy fixture validation passed.</p>');
          }
        } catch (error) {
          show(error.message);
        }
      }

      async function dryRunLegacyFixture() {
        try {
          if (!token) await login();
          const fixtureJson = inputValue("legacy-fixture-json");
          if (!fixtureJson) throw new Error("Paste legacy fixture JSON first.");
          const payload = await request("/api/admin/legacy-fixtures/dry-run", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ fixture_json: fixtureJson })
          });
          show(payload);
          setScreen(`
            <article class="card">
              <h3>${escapeHtml(payload.fixture_name)}</h3>
              <p>Would import: ${payload.would_import ? "yes" : "no"}</p>
              <p>${payload.validation.issue_count} validation issues</p>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function importLegacyFixture() {
        try {
          if (!token) await login();
          const fixtureJson = inputValue("legacy-fixture-json");
          if (!fixtureJson) throw new Error("Paste legacy fixture JSON first.");
          const payload = await request("/api/admin/legacy-fixtures/import", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ fixture_json: fixtureJson })
          });
          setInput("form-version-id", payload.form_version_id);
          setInput("form-id", payload.form_id);
          setInput("submission-id", payload.submission_id);
          setInput("dashboard-id", payload.dashboard_id);
          setInput("report-id", payload.report_id);
          selectRecord("submission", payload.submission_id, payload.submission_id, {
            "submission-id": payload.submission_id
          });
          selectRecord("report", payload.report_id, payload.report_id, {
            "report-id": payload.report_id
          });
          selectRecord("dashboard", payload.dashboard_id, payload.dashboard_id, {
            "dashboard-id": payload.dashboard_id
          });
          show(payload);
          setScreen(`
            <article class="card">
              <h3>${escapeHtml(payload.fixture_name)}</h3>
              <p>Imported submission ${escapeHtml(payload.submission_id)}</p>
              <p>${escapeHtml(payload.analytics_values)} analytics values projected</p>
              <button type="button" onclick="loadDashboardByValue('${escapeHtml(payload.dashboard_id)}')">Open Imported Dashboard</button>
              <button type="button" onclick="loadReportByValue('${escapeHtml(payload.report_id)}')">Run Imported Report</button>
            </article>
          `);
          await loadAppSummary();
        } catch (error) {
          show(error.message);
        }
      }

      updateSessionStatus();
      renderSelections();
"#;
