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
                  <th>Source</th>
                  <th>Field</th>
                  <th>Value</th>
                  <th>Submission</th>
                </tr>
              </thead>
              <tbody>
                ${rows.map((row) => `
                  <tr>
                    <td>${escapeHtml(row.node_name || "Unknown node")}</td>
                    <td>${escapeHtml(row.source_alias || "Direct")}</td>
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

      function aggregationRowsView(rows) {
        if (rows.length === 0) {
          return '<p class="muted">No rows matched this aggregation.</p>';
        }
        return `
          <div class="table-wrap">
            <table>
              <thead>
                <tr>
                  <th>Group</th>
                  <th>Metric</th>
                  <th>Value</th>
                </tr>
              </thead>
              <tbody>
                ${rows.flatMap((row) => Object.entries(row.metrics || {}).map(([metric, value]) => `
                  <tr>
                    <td>${escapeHtml(row.group_key || "All")}</td>
                    <td>${escapeHtml(metric)}</td>
                    <td>${escapeHtml(value)}</td>
                  </tr>
                `)).join("")}
              </tbody>
            </table>
          </div>
        `;
      }

      function datasetRowSubmissionActions(row) {
        if (!row?.submission_id) {
          return '<span class="muted">None</span>';
        }
        if (row.source_alias === "join" && row.submission_id.includes("|")) {
          const parts = row.submission_id
            .split("|")
            .map((part) => part.trim())
            .filter(Boolean);
          return parts.map((part) => {
            const separator = part.indexOf(":");
            if (separator <= 0) {
              return `<span class="muted">${escapeHtml(part)}</span>`;
            }
            const sourceAlias = part.slice(0, separator);
            const submissionId = part.slice(separator + 1);
            return `<button type="button" onclick="loadSubmissionByValue('${escapeHtml(submissionId)}')">Open ${escapeHtml(sourceAlias)}</button>`;
          }).join(" ");
        }
        return `<button type="button" onclick="loadSubmissionByValue('${escapeHtml(row.submission_id)}')">Open</button>`;
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
            ["Datasets", payload.datasets],
            ["Reports", payload.reports],
            ["Aggregations", payload.aggregations],
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
              <button type="button" onclick="openPublishedFormVersion('${escapeHtml(formVersion.form_version_id)}', '${escapeHtml(formVersion.form_id)}', '${escapeHtml(formVersion.form_name)} ${escapeHtml(formVersion.version_label)}')">Open This Form</button>
              <button type="button" onclick="renderForm('${escapeHtml(formVersion.form_version_id)}')">Render Form</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function openPublishedFormVersion(formVersionId, formId, label = formVersionId) {
        useFormVersion(formVersionId, formId, label);
        await renderForm(formVersionId);
      }

      async function openSelectedFormVersion() {
        await openPublishedFormVersion(inputValue("form-version-id"), inputValue("form-id"), "Selected Form");
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
              <button type="button" onclick="useTargetNodeAndContinue('${escapeHtml(node.id)}', '${escapeHtml(node.name)}')">Use Target and Continue</button>
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

      async function useTargetNodeAndContinue(nodeId, nodeName = nodeId) {
        useTargetNode(nodeId, nodeName);
        if (renderedForm) {
          await renderForm(renderedForm.form_version_id);
        }
      }

      async function useSelectedTargetNodeAndContinue() {
        const nodeId = inputValue("node-id");
        if (!nodeId) throw new Error("Choose a target node before continuing.");
        await useTargetNodeAndContinue(nodeId, inputValue("node-name") || nodeId);
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
          if (!formVersionId) throw new Error("Choose a published form before opening the response form.");
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

      async function refreshAnalyticsAndRunReport() {
        await refreshAnalytics();
        await loadReportById();
      }

      async function refreshAnalyticsAndOpenDashboard() {
        await refreshAnalytics();
        await loadDashboardById();
      }

      function useReport(reportId, reportName = reportId) {
        selectRecord("report", reportName, reportId, {
          "report-id": reportId,
          "report-name": reportName
        });
      }

      function useDataset(datasetId, datasetName = datasetId, datasetSlug = "", datasetGrain = "", compositionMode = "") {
        selectRecord("dataset", datasetName, datasetId, {
          "dataset-id": datasetId,
          ...(datasetSlug ? { "dataset-slug": datasetSlug } : {}),
          ...(datasetGrain ? { "dataset-grain": datasetGrain } : {}),
          ...(compositionMode ? { "dataset-composition-mode": compositionMode } : {})
        });
      }

      async function loadDatasets() {
        try {
          if (!token) await login();
          const payload = await request("/api/datasets");
          show(payload);
          showCards(payload, (dataset) => `
            <article class="card">
              <h3>${escapeHtml(dataset.name)}</h3>
              <p>${escapeHtml(dataset.grain)} dataset with ${escapeHtml(dataset.composition_mode)} composition</p>
              <p class="muted">${dataset.source_count} sources and ${dataset.field_count} fields</p>
              <button type="button" onclick="useDataset('${escapeHtml(dataset.id)}', '${escapeHtml(dataset.name)}', '${escapeHtml(dataset.slug)}', '${escapeHtml(dataset.grain)}', '${escapeHtml(dataset.composition_mode)}')">Use Dataset</button>
              <button type="button" onclick="loadDatasetDefinitionByValue('${escapeHtml(dataset.id)}')">Inspect Dataset</button>
              <button type="button" onclick="loadDatasetTableByValue('${escapeHtml(dataset.id)}')">Run Dataset</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDatasetDefinitionById() {
        try {
          const datasetId = inputValue("dataset-id");
          if (!datasetId) throw new Error("Enter or select a dataset ID first.");
          await loadDatasetDefinitionByValue(datasetId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDatasetDefinitionByValue(datasetId) {
        if (!token) await login();
        const payload = await request(`/api/datasets/${datasetId}`);
        setInput("dataset-id", payload.id);
        useDataset(payload.id, payload.name, payload.slug, payload.grain, payload.composition_mode);
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Dataset Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">${escapeHtml(payload.grain)} dataset with ${escapeHtml(payload.composition_mode)} composition</p>
            <p>${payload.sources.length} sources and ${payload.fields.length} fields</p>
            <p>${payload.reports.length} reports use this dataset.</p>
            <button type="button" onclick="loadDatasetTableByValue('${escapeHtml(payload.id)}')">Run This Dataset</button>
          </article>
          ${payload.sources.map((source) => `
            <article class="card">
              <h3>${escapeHtml(source.source_alias)}</h3>
              <p>${escapeHtml(source.form_name || source.compatibility_group_name || source.form_id || source.compatibility_group_id || "Unknown source")}</p>
              <p class="muted">${escapeHtml(source.selection_rule)}</p>
            </article>
          `).join("")}
          ${payload.fields.map((field) => `
            <article class="card">
              <h3>${escapeHtml(field.label)}</h3>
              <p>${escapeHtml(field.key)} from ${escapeHtml(field.source_alias)}.${escapeHtml(field.source_field_key)}</p>
              <p class="muted">${escapeHtml(field.field_type)}</p>
            </article>
          `).join("")}
          ${payload.reports.map((report) => `
            <article class="card">
              <h3>${escapeHtml(report.name)}</h3>
              <p class="muted">Uses this dataset</p>
              <button type="button" onclick="useReport('${escapeHtml(report.id)}', '${escapeHtml(report.name)}')">Use Report Context</button>
              <button type="button" onclick="loadReportDefinition('${escapeHtml(report.id)}')">Open Linked Report</button>
            </article>
          `).join("") || '<p class="muted">No reports use this dataset yet.</p>'}
        `);
      }

      async function loadDatasetTableById() {
        try {
          const datasetId = inputValue("dataset-id");
          if (!datasetId) throw new Error("Enter or select a dataset ID first.");
          await loadDatasetTableByValue(datasetId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDatasetTableByValue(datasetId) {
        if (!token) await login();
        const payload = await request(`/api/datasets/${datasetId}/table`);
        useDataset(datasetId);
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Dataset Results</h3>
            <p>${payload.rows.length} rows returned.</p>
            ${payload.rows.length === 0 ? '<p class="muted">No rows matched this dataset.</p>' : `
              <div class="table-wrap">
                <table>
                  <thead>
                    <tr>
                      <th>Node</th>
                      <th>Source</th>
                      <th>Submission</th>
                      <th>Values</th>
                      <th>Actions</th>
                    </tr>
                  </thead>
                  <tbody>
                    ${payload.rows.map((row) => `
                      <tr>
                        <td>${escapeHtml(row.node_name || "Unknown node")}</td>
                        <td>${escapeHtml(row.source_alias || "Direct")}</td>
                        <td>${escapeHtml(row.submission_id || "")}</td>
                        <td>${escapeHtml(JSON.stringify(row.values || {}))}</td>
                        <td>${datasetRowSubmissionActions(row)}</td>
                      </tr>
                    `).join("")}
                  </tbody>
                </table>
              </div>
            `}
          </article>
        `);
      }

      async function loadReports() {
        try {
          if (!token) await login();
          const payload = await request("/api/reports");
          show(payload);
          showCards(payload, (report) => `
            <article class="card">
              <h3>${escapeHtml(report.name)}</h3>
              <p class="muted">${report.dataset_id ? `Dataset ${escapeHtml(report.dataset_name || report.dataset_id)}` : `Form ${escapeHtml(report.form_name || report.form_id || "Any")}`}</p>
              <button type="button" onclick="useReport('${escapeHtml(report.id)}', '${escapeHtml(report.name)}'); ${report.form_id ? `useForm('${escapeHtml(report.form_id)}', '${escapeHtml(report.form_name || report.form_id)}');` : ""} ${report.dataset_id ? `useDataset('${escapeHtml(report.dataset_id)}', '${escapeHtml(report.dataset_name || report.dataset_id)}');` : ""}">Use Report Context</button>
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
        if (payload.dataset_id) setInput("dataset-id", payload.dataset_id);
        useReport(payload.id, payload.name);
        if (payload.form_id) useForm(payload.form_id, payload.form_name || payload.form_id);
        if (payload.dataset_id) useDataset(payload.dataset_id, payload.dataset_name || payload.dataset_id);
        setInput("report-fields-json", JSON.stringify(payload.bindings.map((binding) => ({
          logical_key: binding.logical_key,
          source_field_key: binding.source_field_key,
          missing_policy: binding.missing_policy
        }))));
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Report Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">${escapeHtml(payload.dataset_name || payload.dataset_id || payload.form_name || payload.form_id || "Any form")}</p>
            <p>${payload.bindings.length} field bindings</p>
            <p>${payload.aggregations.length} aggregations and ${payload.charts.length} charts depend on this report.</p>
            <button type="button" onclick="loadReportByValue('${escapeHtml(payload.id)}')">Run This Report</button>
          </article>
          ${payload.bindings.map((binding) => `
            <article class="card">
              <h3>${escapeHtml(binding.logical_key)}</h3>
              <p>${escapeHtml(binding.source_field_key)} with ${escapeHtml(binding.missing_policy)}</p>
              <button type="button" onclick="useReportBinding('${escapeHtml(binding.logical_key)}', '${escapeHtml(binding.source_field_key)}', '${escapeHtml(binding.missing_policy)}')">Use Binding</button>
            </article>
          `).join("") || '<p class="muted">No report bindings configured.</p>'}
          ${payload.aggregations.map((aggregation) => `
            <article class="card">
              <h3>${escapeHtml(aggregation.name)}</h3>
              <p>${aggregation.metric_count} metrics</p>
              <button type="button" onclick="useAggregation('${escapeHtml(aggregation.id)}', '${escapeHtml(aggregation.name)}', '${escapeHtml(payload.id)}', '${escapeHtml(payload.name)}')">Use Aggregation</button>
              <button type="button" onclick="loadAggregationDefinitionByValue('${escapeHtml(aggregation.id)}')">Open Aggregation</button>
            </article>
          `).join("") || '<p class="muted">No aggregations depend on this report yet.</p>'}
          ${payload.charts.map((chart) => `
            <article class="card">
              <h3>${escapeHtml(chart.name)}</h3>
              <p>${escapeHtml(chart.chart_type)} chart${chart.aggregation_name ? ` via ${escapeHtml(chart.aggregation_name)}` : ""}</p>
              <button type="button" onclick="useChart('${escapeHtml(chart.id)}', '${escapeHtml(chart.name)}', '${chart.aggregation_id ? "" : escapeHtml(payload.id)}', '${chart.aggregation_id ? "" : escapeHtml(payload.name)}', '${escapeHtml(chart.chart_type)}', '${escapeHtml(chart.aggregation_id || "")}', '${escapeHtml(chart.aggregation_name || "")}')">Use Chart Context</button>
              <button type="button" onclick="loadChartDefinitionByValue('${escapeHtml(chart.id)}')">Open Chart</button>
            </article>
          `).join("") || '<p class="muted">No charts depend on this report yet.</p>'}
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

      function useAggregation(aggregationId, aggregationName = aggregationId, reportId = "", reportName = reportId) {
        selectRecord("aggregation", aggregationName, aggregationId, {
          "aggregation-id": aggregationId,
          ...(reportId ? { "report-id": reportId } : {})
        });
        if (reportId) useReport(reportId, reportName || reportId);
      }

      async function loadAggregations() {
        try {
          if (!token) await login();
          const payload = await request("/api/aggregations");
          show(payload);
          showCards(payload, (aggregation) => `
            <article class="card">
              <h3>${escapeHtml(aggregation.name)}</h3>
              <p>${aggregation.metric_count} metrics</p>
              <p class="muted">Report ${escapeHtml(aggregation.report_name || aggregation.report_id)}${aggregation.group_by_logical_key ? ` grouped by ${escapeHtml(aggregation.group_by_logical_key)}` : ""}</p>
              <button type="button" onclick="useAggregation('${escapeHtml(aggregation.id)}', '${escapeHtml(aggregation.name)}', '${escapeHtml(aggregation.report_id)}', '${escapeHtml(aggregation.report_name)}')">Use Aggregation</button>
              <button type="button" onclick="loadAggregationDefinitionByValue('${escapeHtml(aggregation.id)}')">Inspect Aggregation</button>
              <button type="button" onclick="loadAggregationByValue('${escapeHtml(aggregation.id)}')">Run Aggregation</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadAggregationDefinitionById() {
        try {
          const aggregationId = inputValue("aggregation-id");
          if (!aggregationId) throw new Error("Enter or select an aggregation ID first.");
          await loadAggregationDefinitionByValue(aggregationId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadAggregationDefinitionByValue(aggregationId) {
        if (!token) await login();
        const payload = await request(`/api/aggregations/${aggregationId}`);
        setInput("aggregation-id", payload.id);
        setInput("report-id", payload.report_id);
        useAggregation(payload.id, payload.name, payload.report_id, payload.report_name);
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Aggregation Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">Report ${escapeHtml(payload.report_name)}</p>
            <p>${payload.metrics.length} metrics</p>
            <button type="button" onclick="loadAggregationByValue('${escapeHtml(payload.id)}')">Run This Aggregation</button>
          </article>
          ${payload.metrics.map((metric) => `
            <article class="card">
              <h3>${escapeHtml(metric.metric_key)}</h3>
              <p>${escapeHtml(metric.metric_kind)}${metric.source_logical_key ? ` from ${escapeHtml(metric.source_logical_key)}` : ""}</p>
            </article>
          `).join("") || '<p class="muted">No aggregation metrics configured.</p>'}
        `);
      }

      async function loadAggregationById() {
        try {
          const aggregationId = inputValue("aggregation-id");
          if (!aggregationId) throw new Error("Enter or select an aggregation ID first.");
          await loadAggregationByValue(aggregationId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadAggregationByValue(aggregationId) {
        if (!token) await login();
        const payload = await request(`/api/aggregations/${aggregationId}/table`);
        useAggregation(aggregationId);
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Aggregation Results</h3>
            <p>${payload.rows.length} rows returned.</p>
            ${aggregationRowsView(payload.rows)}
          </article>
        `);
      }

      function useChart(chartId, chartName = chartId, reportId = "", reportName = reportId, chartType = "table", aggregationId = "", aggregationName = aggregationId) {
        selectRecord("chart", chartName, chartId, {
          "chart-id": chartId,
          "chart-name": chartName,
          "chart-type": chartType,
          ...(reportId ? { "report-id": reportId, "aggregation-id": "" } : {}),
          ...(aggregationId ? { "aggregation-id": aggregationId, "report-id": "" } : {})
        });
        if (reportId) useReport(reportId, reportName || reportId);
        if (aggregationId) useAggregation(aggregationId, aggregationName || aggregationId);
      }

      async function loadChartDefinitionById() {
        try {
          const chartId = inputValue("chart-id");
          if (!chartId) throw new Error("Enter or select a chart ID first.");
          await loadChartDefinitionByValue(chartId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadChartDefinitionByValue(chartId) {
        if (!token) await login();
        const payload = await request(`/api/charts/${chartId}`);
        const chart = payload.chart;
        useChart(
          chart.id,
          chart.name,
          chart.report_id || "",
          chart.report_name || "",
          chart.chart_type,
          chart.aggregation_id || "",
          chart.aggregation_name || ""
        );
        show(payload);
        setScreen(`
          <article class="card">
            <h3>Chart Definition</h3>
            <p>${escapeHtml(chart.name)}</p>
            <p class="muted">${escapeHtml(chart.chart_type)} chart</p>
            <p class="muted">${chart.aggregation_id ? `Aggregation ${escapeHtml(chart.aggregation_name || chart.aggregation_id)}${chart.aggregation_report_name ? ` from ${escapeHtml(chart.aggregation_report_name)}` : ""}` : `Report ${escapeHtml(chart.report_name || chart.report_id || "None")}${chart.report_form_name ? ` on ${escapeHtml(chart.report_form_name)}` : ""}`}</p>
            <p>${payload.dashboards.length} dashboards use this chart.</p>
            <div class="actions">
              ${chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(chart.report_id)}')">Open Linked Report</button>` : ""}
              ${chart.aggregation_id ? `<button type="button" onclick="loadAggregationByValue('${escapeHtml(chart.aggregation_id)}')">Open Linked Aggregation</button>` : ""}
            </div>
          </article>
          ${payload.dashboards.length
            ? payload.dashboards.map((dashboard) => `
                <article class="card">
                  <h3>${escapeHtml(dashboard.name)}</h3>
                  <p>${dashboard.component_count} components</p>
                  <button type="button" onclick="useDashboard('${escapeHtml(dashboard.id)}', '${escapeHtml(dashboard.name)}')">Use Dashboard</button>
                  <button type="button" onclick="loadDashboardByValue('${escapeHtml(dashboard.id)}')">Open Dashboard</button>
                </article>
              `).join("")
            : '<p class="muted">No dashboards use this chart yet.</p>'}
        `);
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
              <p class="muted">${chart.aggregation_id ? `Aggregation ${escapeHtml(chart.aggregation_name || chart.aggregation_id)}${chart.aggregation_report_name ? ` from ${escapeHtml(chart.aggregation_report_name)}` : ""}` : `Report ${escapeHtml(chart.report_name || "None")}${chart.report_form_name ? ` on ${escapeHtml(chart.report_form_name)}` : ""}`}</p>
              <button type="button" onclick="useChart('${escapeHtml(chart.id)}', '${escapeHtml(chart.name)}', '${escapeHtml(chart.report_id || "")}', '${escapeHtml(chart.report_name || "")}', '${escapeHtml(chart.chart_type)}', '${escapeHtml(chart.aggregation_id || "")}', '${escapeHtml(chart.aggregation_name || "")}')">Use Chart Context</button>
              <button type="button" onclick="loadChartDefinitionByValue('${escapeHtml(chart.id)}')">Inspect Chart</button>
              ${chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(chart.report_id)}')">Run Report</button>` : ""}
              ${chart.aggregation_id ? `<button type="button" onclick="loadAggregationByValue('${escapeHtml(chart.aggregation_id)}')">Run Aggregation</button>` : ""}
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
        const header = `
          <article class="card">
            <h3>Dashboard Preview</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p>${payload.components.length} components</p>
            <button type="button" onclick="refreshAnalyticsAndOpenDashboard()">Refresh and Reopen Dashboard</button>
          </article>
        `;
        const cards = await Promise.all(payload.components.map(async (component) => {
          let rows = [];
          let aggregationRows = [];
          const componentTitle = component.config?.title || component.chart.name;
          if (component.chart.report_id) {
            const report = await request(`/api/reports/${component.chart.report_id}/table`);
            rows = report.rows;
          }
          if (component.chart.aggregation_id) {
            const aggregation = await request(`/api/aggregations/${component.chart.aggregation_id}/table`);
            aggregationRows = aggregation.rows;
          }
          return `
            <article class="card">
              <h3>${escapeHtml(componentTitle)}</h3>
              <p>${escapeHtml(component.chart.chart_type)} chart</p>
              <p class="muted">Chart ${escapeHtml(component.chart.name)}</p>
              <p>Position ${component.position}</p>
              <p class="muted">${component.chart.aggregation_id ? `Aggregation ${escapeHtml(component.chart.aggregation_name || component.chart.aggregation_id)}${component.chart.aggregation_report_name ? ` from ${escapeHtml(component.chart.aggregation_report_name)}` : ""}` : `Report ${escapeHtml(component.chart.report_name || component.chart.report_id || "None")}${component.chart.report_form_name ? ` on ${escapeHtml(component.chart.report_form_name)}` : ""}`}</p>
              <button type="button" onclick="useChart('${escapeHtml(component.chart.id)}', '${escapeHtml(component.chart.name)}', '${escapeHtml(component.chart.report_id || "")}', '${escapeHtml(component.chart.report_name || "")}', '${escapeHtml(component.chart.chart_type)}', '${escapeHtml(component.chart.aggregation_id || "")}', '${escapeHtml(component.chart.aggregation_name || "")}')">Use Chart Context</button>
              <button type="button" onclick="loadChartDefinitionByValue('${escapeHtml(component.chart.id)}')">Inspect Chart</button>
              ${component.chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(component.chart.report_id)}')">Open Report</button>` : ""}
              ${component.chart.aggregation_id ? `<button type="button" onclick="loadAggregationByValue('${escapeHtml(component.chart.aggregation_id)}')">Open Aggregation</button>` : ""}
              ${component.chart.aggregation_id ? aggregationRowsView(aggregationRows) : reportRowsView(rows)}
            </article>
          `;
        }));
        setScreen(header + (cards.length ? cards.join("") : '<p class="muted">No dashboard components found.</p>'));
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
