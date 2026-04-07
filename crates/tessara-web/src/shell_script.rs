//! JavaScript controller for the local Tessara shell.

/// Browser-side workflow controller for the local shell.
pub const SCRIPT: &str = r#"
      let token = null;
      let demoDashboardId = null;
      let demoReportId = null;
      let renderedForm = null;
      let reportBindings = [];
      const selections = {};

      function show(value) {
        document.getElementById("output").textContent =
          typeof value === "string" ? value : JSON.stringify(value, null, 2);
      }

      function setInput(id, value) {
        const element = document.getElementById(id);
        if (element) element.value = value ?? "";
      }

      function selectRecord(kind, label, id, bindings = {}) {
        selections[kind] = { label, id };
        for (const [inputId, value] of Object.entries(bindings)) {
          setInput(inputId, value);
        }
        renderSelections();
      }

      function renderSelections() {
        const entries = Object.entries(selections);
        document.getElementById("selection-state").innerHTML = entries.length
          ? entries.map(([kind, record]) => `
              <article class="selection-item">
                <h3>${escapeHtml(kind)}</h3>
                <p>${escapeHtml(record.label)}</p>
                <code>${escapeHtml(record.id)}</code>
              </article>
            `).join("")
          : '<p class="muted">No records selected yet.</p>';
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

      function booleanInputValue(id) {
        const value = inputValue(id).toLowerCase();
        return value === "true" || value === "yes" || value === "1";
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
          setInput("form-version-id", payload.form_version_id);
          setInput("form-id", payload.form_id);
          setInput("node-id", payload.organization_node_id);
          setInput("submission-id", payload.submission_id);
          setInput("dashboard-id", demoDashboardId);
          setInput("report-id", demoReportId);
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
        } catch (error) {
          show(error.message);
        }
      }

      async function loadAppSummary() {
        try {
          if (!token) await login();
          const payload = await request("/api/app/summary");
          show(payload);
          const cards = [
            ["Published forms", payload.published_form_versions],
            ["Draft submissions", payload.draft_submissions],
            ["Submitted submissions", payload.submitted_submissions],
            ["Reports", payload.reports],
            ["Dashboards", payload.dashboards],
            ["Charts", payload.charts]
          ];
          showCards(cards, ([label, count]) => `
            <article class="card">
              <h3>${escapeHtml(label)}</h3>
              <p>${escapeHtml(count)}</p>
            </article>
          `);
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
              <button type="button" onclick="useNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Node Type</button>
              <button type="button" onclick="useParentNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Parent Type</button>
              <button type="button" onclick="useChildNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Child Type</button>
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

      function useNodeType(nodeTypeId, nodeTypeName = nodeTypeId) {
        selectRecord("node type", nodeTypeName, nodeTypeId, {
          "node-type-id": nodeTypeId,
          "metadata-node-type-id": nodeTypeId,
          "form-scope-node-type-id": nodeTypeId
        });
      }

      function useParentNodeType(nodeTypeId, nodeTypeName = nodeTypeId) {
        selectRecord("parent node type", nodeTypeName, nodeTypeId, {
          "parent-node-type-id": nodeTypeId
        });
      }

      function useChildNodeType(nodeTypeId, nodeTypeName = nodeTypeId) {
        selectRecord("child node type", nodeTypeName, nodeTypeId, {
          "child-node-type-id": nodeTypeId
        });
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
              <button type="button" onclick="useParentNodeType('${escapeHtml(relationship.parent_node_type_id)}', '${escapeHtml(relationship.parent_name)}')">Use Parent Type</button>
              <button type="button" onclick="useChildNodeType('${escapeHtml(relationship.child_node_type_id)}', '${escapeHtml(relationship.child_name)}')">Use Child Type</button>
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
              <button type="button" onclick="useNodeType('${escapeHtml(field.node_type_id)}', '${escapeHtml(field.node_type_name)}')">Use Node Type</button>
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
              required: booleanInputValue("metadata-required")
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

      async function updateNode() {
        try {
          if (!token) await login();
          const nodeId = inputValue("node-id");
          if (!nodeId) throw new Error("Select or enter a node ID first.");
          const parentNodeId = inputValue("parent-node-id");
          const payload = await request(`/api/admin/nodes/${nodeId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              parent_node_id: parentNodeId || null,
              name: inputValue("node-name"),
              metadata: jsonInputValue("node-metadata-json")
            })
          });
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
              <button type="button" onclick="useForm('${escapeHtml(form.id)}', '${escapeHtml(form.name)}')">Use Form</button>
              <ul>
                ${form.versions.map((version) => `
                  <li>
                    ${escapeHtml(version.version_label)}:
                    ${escapeHtml(version.status)}
                    <button type="button" onclick="useFormVersion('${escapeHtml(version.id)}', '${escapeHtml(form.id)}', '${escapeHtml(form.name)} ${escapeHtml(version.version_label)}')">Use Version</button>
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

      function useForm(formId, formName = formId) {
        selectRecord("form", formName, formId, {
          "form-id": formId
        });
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

      async function updateSection() {
        try {
          if (!token) await login();
          const sectionId = inputValue("section-id");
          if (!sectionId) throw new Error("Select or enter a section ID first.");
          const payload = await request(`/api/admin/form-sections/${sectionId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              title: inputValue("section-title"),
              position: 0
            })
          });
          show(payload);
          if (inputValue("form-version-id")) await renderForm(inputValue("form-version-id"));
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
              required: booleanInputValue("field-required"),
              position: 0
            })
          });
          show(payload);
          await renderForm(formVersionId);
        } catch (error) {
          show(error.message);
        }
      }

      async function updateField() {
        try {
          if (!token) await login();
          const fieldId = inputValue("field-id");
          const sectionId = inputValue("section-id");
          if (!fieldId || !sectionId) {
            throw new Error("Select or enter a field ID and section ID first.");
          }
          const payload = await request(`/api/admin/form-fields/${fieldId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              section_id: sectionId,
              key: inputValue("field-key"),
              label: inputValue("field-label"),
              field_type: inputValue("field-type"),
              required: booleanInputValue("field-required"),
              position: 0
            })
          });
          show(payload);
          if (inputValue("form-version-id")) await renderForm(inputValue("form-version-id"));
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
          useFormVersion(payload.form_version_id, payload.form_id, `${payload.form_name} ${payload.version_label}`);
          show(payload);
          document.getElementById("screen").innerHTML = `
            <article class="card form-screen">
              <h3>${escapeHtml(payload.form_name)} ${escapeHtml(payload.version_label)}</h3>
              <p>Status: ${escapeHtml(payload.status)}</p>
              <p class="muted">Target node: ${escapeHtml(selections.node?.label || inputValue("node-id") || "Select a node before creating a draft.")}</p>
              <p class="muted">Draft submission: ${escapeHtml(inputValue("submission-id") || "Create a draft after selecting a node.")}</p>
              ${payload.sections.map((section) => `
                <section class="form-section">
                  <h4>${escapeHtml(section.title)}</h4>
                  <button type="button" onclick="useSection('${escapeHtml(section.id)}', '${escapeHtml(section.title)}')">Use Section</button>
                  <div class="form-fields">
                    ${section.fields.map((field) => `
                      <div class="form-field">
                        <label for="${escapeHtml(fieldInputId(field))}">
                          ${escapeHtml(field.label)} (${escapeHtml(field.field_type)}${field.required ? ", required" : ""})
                        </label>
                        ${renderFieldInput(field)}
                        <button type="button" onclick="useField('${escapeHtml(field.id)}', '${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.field_type)}', ${field.required ? "true" : "false"})">Use Field Settings</button>
                        <button type="button" onclick="useReportField('${escapeHtml(field.key)}', '${escapeHtml(field.label)}')">Use Report Source</button>
                      </div>
                    `).join("")}
                  </div>
                </section>
              `).join("")}
              <div class="actions form-actions">
                <button type="button" onclick="createDraft()">Create Draft</button>
                <button type="button" onclick="saveRenderedFormValues()">Save Values</button>
                <button type="button" onclick="submitDraft()">Submit Draft</button>
              </div>
            </article>
          `;
        } catch (error) {
          show(error.message);
        }
      }

      function useSection(sectionId, sectionTitle = sectionId) {
        selectRecord("form section", sectionTitle, sectionId, {
          "section-id": sectionId,
          "section-title": sectionTitle
        });
      }

      function useField(fieldId, fieldKey, fieldLabel = fieldKey, fieldType = "text", required = true) {
        selectRecord("form field", fieldLabel, fieldId, {
          "field-id": fieldId,
          "field-key": fieldKey,
          "field-label": fieldLabel,
          "field-type": fieldType,
          "field-required": required ? "true" : "false"
        });
      }

      function useReportField(fieldKey, fieldLabel = fieldKey) {
        selectRecord("report field", fieldLabel, fieldKey, {
          "report-logical-key": fieldKey,
          "report-source-field-key": fieldKey
        });
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
              <button type="button" onclick="useParentNode('${escapeHtml(node.id)}', '${escapeHtml(node.name)}')">Use Parent</button>
              <button type="button" onclick="useNodeType('${escapeHtml(node.node_type_id)}', '${escapeHtml(node.node_type_name)}')">Use Node Type</button>
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

      function useParentNode(nodeId, nodeName = nodeId) {
        selectRecord("parent node", nodeName, nodeId, {
          "parent-node-id": nodeId
        });
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
              <button type="button" onclick="useSubmission('${escapeHtml(submission.id)}', '${escapeHtml(submission.form_name)} ${escapeHtml(submission.version_label)}')">Use Submission</button>
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

      function useSubmission(submissionId, label = submissionId) {
        selectRecord("submission", label, submissionId, {
          "submission-id": submissionId
        });
      }

      async function loadSubmissionByValue(submissionId) {
        if (!token) await login();
        const payload = await request(`/api/submissions/${submissionId}`);
        document.getElementById("submission-id").value = payload.id;
        document.getElementById("form-version-id").value = payload.form_version_id;
        document.getElementById("node-id").value = payload.node_id;
        useSubmission(payload.id, `${payload.form_name} ${payload.version_label}`);
        useFormVersion(payload.form_version_id, payload.form_id, `${payload.form_name} ${payload.version_label}`);
        useTargetNode(payload.node_id, payload.node_name);
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
          const values = collectRenderedValues();
          validateRenderedValues(values);
          const payload = await request(`/api/submissions/${submissionId}/values`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ values })
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
          const bindingsJson = inputValue("report-fields-json");
          const fields = bindingsJson ? JSON.parse(bindingsJson) : [{
            logical_key: inputValue("report-logical-key"),
            source_field_key: inputValue("report-source-field-key"),
            missing_policy: inputValue("report-missing-policy") || "null"
          }];
          const payload = await request("/api/admin/reports", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("report-name"),
              form_id: formId || null,
              fields
            })
          });
          document.getElementById("report-id").value = payload.id;
          show(payload);
          await loadReports();
        } catch (error) {
          show(error.message);
        }
      }

      async function updateReport() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          if (!reportId) throw new Error("Select or enter a report ID first.");
          const formId = inputValue("form-id");
          const bindingsJson = inputValue("report-fields-json");
          const fields = bindingsJson ? JSON.parse(bindingsJson) : [{
            logical_key: inputValue("report-logical-key"),
            source_field_key: inputValue("report-source-field-key"),
            missing_policy: inputValue("report-missing-policy") || "null"
          }];
          const payload = await request(`/api/admin/reports/${reportId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("report-name"),
              form_id: formId || null,
              fields
            })
          });
          show(payload);
          await loadReportDefinition(reportId);
        } catch (error) {
          show(error.message);
        }
      }

      function addReportBinding() {
        try {
          const binding = {
            logical_key: inputValue("report-logical-key"),
            source_field_key: inputValue("report-source-field-key"),
            missing_policy: inputValue("report-missing-policy") || "null"
          };
          if (!binding.logical_key || !binding.source_field_key) {
            throw new Error("Select or enter a report logical key and source field key first.");
          }
          reportBindings = reportBindings.filter((existing) => existing.logical_key !== binding.logical_key);
          reportBindings.push(binding);
          document.getElementById("report-fields-json").value = JSON.stringify(reportBindings);
          show({ report_bindings: reportBindings });
        } catch (error) {
          show(error.message);
        }
      }

      function clearReportBindings() {
        reportBindings = [];
        document.getElementById("report-fields-json").value = "";
        show({ report_bindings: reportBindings });
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
            document.getElementById("screen").innerHTML = '<p class="muted">Legacy fixture validation passed.</p>';
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
          document.getElementById("screen").innerHTML = `
            <article class="card">
              <h3>${escapeHtml(payload.fixture_name)}</h3>
              <p>Would import: ${payload.would_import ? "yes" : "no"}</p>
              <p>${payload.validation.issue_count} validation issues</p>
            </article>
          `;
        } catch (error) {
          show(error.message);
        }
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
        document.getElementById("legacy-fixture-json").value = fixture;
        show({ selected_fixture: name });
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

      async function updateChart() {
        try {
          if (!token) await login();
          const chartId = inputValue("chart-id");
          if (!chartId) throw new Error("Select or enter a chart ID first.");
          const reportId = inputValue("report-id");
          const payload = await request(`/api/admin/charts/${chartId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("chart-name"),
              report_id: reportId || null,
              chart_type: inputValue("chart-type") || "table"
            })
          });
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
              <p class="muted">Report ${escapeHtml(chart.report_name || "None")}${chart.report_form_name ? ` on ${escapeHtml(chart.report_form_name)}` : ""}</p>
              <button type="button" onclick="useChart('${escapeHtml(chart.id)}', '${escapeHtml(chart.name)}', '${escapeHtml(chart.report_id || "")}', '${escapeHtml(chart.report_name || "")}')">Use Chart</button>
              ${chart.report_id ? `<button type="button" onclick="useReport('${escapeHtml(chart.report_id)}', '${escapeHtml(chart.report_name || `Report for ${chart.name}`)}')">Use Report</button>` : ""}
              <code>${escapeHtml(chart.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useChart(chartId, chartName = chartId, reportId = "", reportName = reportId) {
        selectRecord("chart", chartName, chartId, {
          "chart-id": chartId,
          ...(reportId ? { "report-id": reportId } : {})
        });
        if (reportId) {
          selectRecord("report", reportName || reportId, reportId, {
            "report-id": reportId
          });
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

      async function updateDashboard() {
        try {
          if (!token) await login();
          const dashboardId = inputValue("dashboard-id");
          if (!dashboardId) throw new Error("Select or enter a dashboard ID first.");
          const payload = await request(`/api/admin/dashboards/${dashboardId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ name: inputValue("dashboard-name") })
          });
          show(payload);
          await loadDashboardByValue(dashboardId);
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
              <button type="button" onclick="useDashboard('${escapeHtml(dashboard.id)}', '${escapeHtml(dashboard.name)}')">Use Dashboard</button>
              <button type="button" onclick="loadDashboardByValue('${escapeHtml(dashboard.id)}')">Open</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useDashboard(dashboardId, dashboardName = dashboardId) {
        demoDashboardId = dashboardId;
        selectRecord("dashboard", dashboardName, dashboardId, {
          "dashboard-id": dashboardId
        });
      }

      function useReport(reportId, reportName = reportId) {
        demoReportId = reportId;
        selectRecord("report", reportName, reportId, {
          "report-id": reportId
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
              <button type="button" onclick="useReport('${escapeHtml(report.id)}', '${escapeHtml(report.name)}')">Use Report</button>
              ${report.form_id ? `<button type="button" onclick="useForm('${escapeHtml(report.form_id)}', '${escapeHtml(report.form_name || report.form_id)}')">Use Form</button>` : ""}
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
        document.getElementById("report-id").value = payload.id;
        if (payload.form_id) document.getElementById("form-id").value = payload.form_id;
        useReport(payload.id, payload.name);
        if (payload.form_id) useForm(payload.form_id, payload.form_name || payload.form_id);
        document.getElementById("report-fields-json").value = JSON.stringify(payload.bindings.map((binding) => ({
          logical_key: binding.logical_key,
          source_field_key: binding.source_field_key,
          missing_policy: binding.missing_policy
        })));
        reportBindings = payload.bindings.map((binding) => ({
          logical_key: binding.logical_key,
          source_field_key: binding.source_field_key,
          missing_policy: binding.missing_policy
        }));
        show(payload);
        showCards(payload.bindings, (binding) => `
          <article class="card">
            <h3>${escapeHtml(binding.logical_key)}</h3>
            <p>${escapeHtml(binding.source_field_key)} with ${escapeHtml(binding.missing_policy)}</p>
          </article>
        `);
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
        if (!token) await login();
        const payload = await request(`/api/dashboards/${dashboardId}`);
        useDashboard(payload.id, payload.name);
        show(payload);
        const cards = await Promise.all(payload.components.map(async (component) => {
          let rows = [];
          if (component.chart.report_id) {
            const report = await request(`/api/reports/${component.chart.report_id}/table`);
            rows = report.rows;
          }
          return `
            <article class="card">
              <h3>${escapeHtml(component.chart.name)}</h3>
              <p>${escapeHtml(component.chart.chart_type)} chart</p>
              <p class="muted">Report ${escapeHtml(component.chart.report_name || component.chart.report_id || "None")}${component.chart.report_form_name ? ` on ${escapeHtml(component.chart.report_form_name)}` : ""}</p>
              <ul>
                ${rows.map((row) => `
                  <li>${escapeHtml(row.node_name || "Unknown node")}: ${escapeHtml(row.logical_key)} = ${escapeHtml(row.field_value)}</li>
                `).join("")}
              </ul>
            </article>
          `;
        }));
        document.getElementById("screen").innerHTML = cards.length
          ? cards.join("")
          : '<p class="muted">No dashboard components found.</p>';
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
        showCards(payload.rows, (row) => `
          <article class="card">
            <h3>${escapeHtml(row.node_name || "Unknown node")}</h3>
            <p>${escapeHtml(row.logical_key)}: ${escapeHtml(row.field_value)}</p>
            <p class="muted">${escapeHtml(row.submission_id)}</p>
          </article>
        `);
      }
"#;
