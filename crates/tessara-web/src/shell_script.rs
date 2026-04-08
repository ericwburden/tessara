//! JavaScript controller for the local Tessara shell.

/// Browser-side workflow controller for the local shell.
pub const SCRIPT: &str = r#"
      let token = null;
      let demoDashboardId = null;
      let demoReportId = null;
      let renderedForm = null;
      let selectedSubmissionFormVersionId = null;
      let selectedSubmissionStatus = null;
      let selectedSubmissionValues = {};
      let datasetSources = [];
      let datasetFields = [];
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

      function jsStringArg(value) {
        return String(value ?? "")
          .replaceAll("\\", "\\\\")
          .replaceAll("'", "\\'")
          .replaceAll("\n", "\\n")
          .replaceAll("\r", "\\r");
      }

      function showCards(records, render) {
        document.getElementById("screen").innerHTML = records.length
          ? records.map(render).join("")
          : '<p class="muted">No records found.</p>';
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
        return `<button type="button" onclick="loadSubmissionByValue('${escapeHtml(row.submission_id)}')">Open Submission</button>`;
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
            ["Datasets", payload.datasets],
            ["Reports", payload.reports],
            ["Aggregations", payload.aggregations],
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
              <button type="button" onclick="useNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}', '${escapeHtml(nodeType.slug)}')">Use Node Type</button>
              <button type="button" onclick="loadNodeTypeByValue('${escapeHtml(nodeType.id)}')">Inspect Node Type</button>
              <button type="button" onclick="useNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}', '${escapeHtml(nodeType.slug)}')">Edit Node Type</button>
              <button type="button" onclick="useFormScopeNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Form Scope</button>
              <button type="button" onclick="useMetadataNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Metadata Target</button>
              <button type="button" onclick="useParentNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Parent Type</button>
              <button type="button" onclick="useChildNodeType('${escapeHtml(nodeType.id)}', '${escapeHtml(nodeType.name)}')">Use Child Type</button>
              <code>${escapeHtml(nodeType.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadNodeTypeById() {
        try {
          const nodeTypeId = inputValue("node-type-id");
          if (!nodeTypeId) throw new Error("Enter or select a node type ID first.");
          await loadNodeTypeByValue(nodeTypeId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadNodeTypeByValue(nodeTypeId) {
        if (!token) await login();
        const payload = await request(`/api/admin/node-types/${nodeTypeId}`);
        useNodeType(payload.id, payload.name, payload.slug);
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Node Type Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">${escapeHtml(payload.slug)}</p>
            <p>${payload.node_count} nodes, ${payload.metadata_fields.length} metadata fields, ${payload.scoped_forms.length} scoped forms</p>
            <button type="button" onclick="useNodeType('${escapeHtml(payload.id)}', '${escapeHtml(payload.name)}', '${escapeHtml(payload.slug)}')">Use Node Type</button>
          </article>
          ${payload.parent_relationships.map((parentType) => `
            <article class="card">
              <h3>Allowed Parent</h3>
              <p>${escapeHtml(parentType.node_type_name)}</p>
              <button type="button" onclick="useParentNodeType('${escapeHtml(parentType.node_type_id)}', '${escapeHtml(parentType.node_type_name)}')">Use Parent Type</button>
            </article>
          `).join("") || '<p class="muted">No parent node types configured.</p>'}
          ${payload.child_relationships.map((childType) => `
            <article class="card">
              <h3>Allowed Child</h3>
              <p>${escapeHtml(childType.node_type_name)}</p>
              <button type="button" onclick="useChildNodeType('${escapeHtml(childType.node_type_id)}', '${escapeHtml(childType.node_type_name)}')">Use Child Type</button>
            </article>
          `).join("") || '<p class="muted">No child node types configured.</p>'}
          ${payload.metadata_fields.map((field) => `
            <article class="card">
              <h3>${escapeHtml(field.label)}</h3>
              <p>${escapeHtml(field.key)} (${escapeHtml(field.field_type)})${field.required ? " required" : ""}</p>
              <button type="button" onclick="useMetadataField('${escapeHtml(field.id)}', '${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.field_type)}', ${field.required ? "true" : "false"})">Use Metadata Field</button>
            </article>
          `).join("") || '<p class="muted">No metadata fields configured.</p>'}
          ${payload.scoped_forms.map((form) => `
            <article class="card">
              <h3>${escapeHtml(form.form_name)}</h3>
              <p class="muted">${escapeHtml(form.form_slug)}</p>
              <button type="button" onclick="useFormScopeNodeType('${escapeHtml(payload.id)}', '${escapeHtml(payload.name)}')">Use As Form Scope</button>
              <button type="button" onclick="loadFormByValue('${escapeHtml(form.form_id)}')">Open Scoped Form</button>
            </article>
          `).join("") || '<p class="muted">No forms are scoped to this node type yet.</p>'}
        `;
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

      async function updateNodeType() {
        try {
          if (!token) await login();
          const nodeTypeId = inputValue("node-type-id");
          if (!nodeTypeId) throw new Error("Select or enter a node type ID first.");
          const payload = await request(`/api/admin/node-types/${nodeTypeId}`, {
            method: "PUT",
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

      function useNodeType(nodeTypeId, nodeTypeName = nodeTypeId, nodeTypeSlug = "") {
        selectRecord("node type", nodeTypeName, nodeTypeId, {
          "node-type-id": nodeTypeId,
          "metadata-node-type-id": nodeTypeId,
          "form-scope-node-type-id": nodeTypeId,
          ...(nodeTypeName ? { "node-type-name": nodeTypeName } : {}),
          ...(nodeTypeSlug ? { "node-type-slug": nodeTypeSlug } : {})
        });
      }

      function useFormScopeNodeType(nodeTypeId, nodeTypeName = nodeTypeId) {
        selectRecord("form scope node type", nodeTypeName, nodeTypeId, {
          "form-scope-node-type-id": nodeTypeId
        });
      }

      function useMetadataNodeType(nodeTypeId, nodeTypeName = nodeTypeId) {
        selectRecord("metadata node type", nodeTypeName, nodeTypeId, {
          "metadata-node-type-id": nodeTypeId
        });
      }

      function useSelectedNodeTypeAsFormScope() {
        const nodeTypeId = inputValue("node-type-id");
        if (!nodeTypeId) throw new Error("Select or enter a node type ID first.");
        useFormScopeNodeType(nodeTypeId, inputValue("node-type-name") || nodeTypeId);
      }

      function useSelectedNodeTypeAsMetadataTarget() {
        const nodeTypeId = inputValue("node-type-id");
        if (!nodeTypeId) throw new Error("Select or enter a node type ID first.");
        useMetadataNodeType(nodeTypeId, inputValue("node-type-name") || nodeTypeId);
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
              <button type="button" onclick="useRelationship('${escapeHtml(relationship.parent_node_type_id)}', '${escapeHtml(relationship.child_node_type_id)}', '${escapeHtml(relationship.parent_name)} -> ${escapeHtml(relationship.child_name)}')">Use Relationship</button>
              <button type="button" onclick="useParentNodeType('${escapeHtml(relationship.parent_node_type_id)}', '${escapeHtml(relationship.parent_name)}')">Use Parent Type</button>
              <button type="button" onclick="useChildNodeType('${escapeHtml(relationship.child_node_type_id)}', '${escapeHtml(relationship.child_name)}')">Use Child Type</button>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useRelationship(parentNodeTypeId, childNodeTypeId, label) {
        selectRecord("node type relationship", label, `${parentNodeTypeId} -> ${childNodeTypeId}`, {
          "parent-node-type-id": parentNodeTypeId,
          "child-node-type-id": childNodeTypeId
        });
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

      async function deleteRelationship() {
        try {
          if (!token) await login();
          const parentNodeTypeId = inputValue("parent-node-type-id");
          const childNodeTypeId = inputValue("child-node-type-id");
          if (!parentNodeTypeId || !childNodeTypeId) {
            throw new Error("Select or enter parent and child node type IDs first.");
          }
          const payload = await request(`/api/admin/node-type-relationships/${parentNodeTypeId}/${childNodeTypeId}`, {
            method: "DELETE"
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
              <button type="button" onclick="useMetadataField('${escapeHtml(field.id)}', '${escapeHtml(field.node_type_id)}', '${escapeHtml(field.node_type_name)}', '${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.field_type)}', ${field.required ? "true" : "false"})">Use Metadata Field</button>
              <button type="button" onclick="useMetadataNodeType('${escapeHtml(field.node_type_id)}', '${escapeHtml(field.node_type_name)}')">Use Metadata Target</button>
              <button type="button" onclick="useFormScopeNodeType('${escapeHtml(field.node_type_id)}', '${escapeHtml(field.node_type_name)}')">Use Form Scope</button>
              <code>${escapeHtml(field.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useMetadataField(fieldId, nodeTypeId, nodeTypeName, key, label, fieldType, required) {
        selectRecord("metadata field", label, fieldId, {
          "metadata-field-id": fieldId,
          "metadata-node-type-id": nodeTypeId,
          "metadata-key": key,
          "metadata-label": label,
          "metadata-field-type": fieldType,
          "metadata-required": required ? "true" : "false"
        });
        useNodeType(nodeTypeId, nodeTypeName);
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

      async function updateMetadataField() {
        try {
          if (!token) await login();
          const fieldId = inputValue("metadata-field-id");
          if (!fieldId) throw new Error("Select or enter a metadata field ID first.");
          const payload = await request(`/api/admin/node-metadata-fields/${fieldId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
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
              <button type="button" onclick="loadFormByValue('${escapeHtml(form.id)}')">Inspect Form</button>
              <button type="button" onclick="useFormForEditing('${escapeHtml(form.id)}', '${escapeHtml(form.name)}', '${escapeHtml(form.slug)}', '${escapeHtml(form.scope_node_type_id || "")}', '${escapeHtml(form.scope_node_type_name || "")}')">Edit Form</button>
              <ul>
                ${form.versions.map((version) => `
                  <li>
                    ${escapeHtml(version.version_label)}:
                    ${escapeHtml(version.status)}
                    <button type="button" onclick="useFormVersion('${escapeHtml(version.id)}', '${escapeHtml(form.id)}', '${escapeHtml(form.name)} ${escapeHtml(version.version_label)}')">Use Version</button>
                    ${version.compatibility_group_id ? `<button type="button" onclick="useCompatibilityGroup('${escapeHtml(version.compatibility_group_id)}', '${escapeHtml(version.compatibility_group_name || version.compatibility_group_id)}')">Use Compatibility Group</button>` : ""}
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

      async function loadFormById() {
        try {
          const formId = inputValue("form-id");
          if (!formId) throw new Error("Enter or select a form ID first.");
          await loadFormByValue(formId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadFormByValue(formId) {
        if (!token) await login();
        const payload = await request(`/api/admin/forms/${formId}`);
        useFormForEditing(
          payload.id,
          payload.name,
          payload.slug,
          payload.scope_node_type_id || "",
          payload.scope_node_type_name || ""
        );
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Form Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">${escapeHtml(payload.slug)}</p>
            <p>Scope: ${escapeHtml(payload.scope_node_type_name || "Global")}</p>
            <p>${payload.versions.length} versions, ${payload.reports.length} linked reports, ${payload.dataset_sources.length} dataset sources</p>
            <button type="button" onclick="useForm('${escapeHtml(payload.id)}', '${escapeHtml(payload.name)}')">Use Form</button>
            <button type="button" onclick="useFormForEditing('${escapeHtml(payload.id)}', '${escapeHtml(payload.name)}', '${escapeHtml(payload.slug)}', '${escapeHtml(payload.scope_node_type_id || "")}', '${escapeHtml(payload.scope_node_type_name || "")}')">Edit Form</button>
          </article>
          ${payload.versions.map((version) => `
            <article class="card">
              <h3>${escapeHtml(version.version_label)}</h3>
              <p>${escapeHtml(version.status)} with ${version.field_count} fields</p>
              <button type="button" onclick="useFormVersion('${escapeHtml(version.id)}', '${escapeHtml(payload.id)}', '${escapeHtml(payload.name)} ${escapeHtml(version.version_label)}')">Use Version</button>
              <button type="button" onclick="renderForm('${escapeHtml(version.id)}')">Preview Version</button>
              ${version.compatibility_group_id ? `<button type="button" onclick="useCompatibilityGroup('${escapeHtml(version.compatibility_group_id)}', '${escapeHtml(version.compatibility_group_name || version.compatibility_group_id)}')">Use Compatibility Group</button>` : ""}
            </article>
          `).join("") || '<p class="muted">No versions exist yet.</p>'}
          ${payload.reports.map((report) => `
            <article class="card">
              <h3>${escapeHtml(report.name)}</h3>
              <p class="muted">Linked report</p>
              <button type="button" onclick="useReport('${escapeHtml(report.id)}', '${escapeHtml(report.name)}')">Use Report Context</button>
              <button type="button" onclick="loadReportDefinition('${escapeHtml(report.id)}')">Open Linked Report</button>
            </article>
          `).join("") || '<p class="muted">No reports use this form yet.</p>'}
          ${payload.dataset_sources.map((datasetSource) => `
            <article class="card">
              <h3>${escapeHtml(datasetSource.dataset_name)}</h3>
              <p>Source ${escapeHtml(datasetSource.source_alias)} with ${escapeHtml(datasetSource.selection_rule)}</p>
              <button type="button" onclick="useDataset('${escapeHtml(datasetSource.dataset_id)}', '${escapeHtml(datasetSource.dataset_name)}')">Use Dataset</button>
              <button type="button" onclick="loadDatasetByValue('${escapeHtml(datasetSource.dataset_id)}')">Open Linked Dataset</button>
            </article>
          `).join("") || '<p class="muted">No datasets source from this form yet.</p>'}
        `;
      }

      function useForm(formId, formName = formId) {
        selectRecord("form", formName, formId, {
          "form-id": formId,
          "dataset-form-id": formId
        });
      }

      function useFormForEditing(formId, formName, formSlug, scopeNodeTypeId, scopeNodeTypeName = "") {
        selectRecord("form for editing", formName, formId, {
          "form-id": formId,
          "form-name": formName,
          "form-slug": formSlug,
          "form-scope-node-type-id": scopeNodeTypeId
        });
        if (scopeNodeTypeId) {
          useFormScopeNodeType(scopeNodeTypeId, scopeNodeTypeName || scopeNodeTypeId);
        }
      }

      function useFormVersion(formVersionId, formId, label = formVersionId) {
        selectRecord("form version", label, formVersionId, {
          "form-version-id": formVersionId,
          "form-id": formId,
          "dataset-form-id": formId
        });
      }

      function useCompatibilityGroup(compatibilityGroupId, label = compatibilityGroupId) {
        selectRecord("compatibility group", label, compatibilityGroupId, {
          "dataset-form-id": "",
          "dataset-compatibility-group-id": compatibilityGroupId
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

      async function updateForm() {
        try {
          if (!token) await login();
          const formId = inputValue("form-id");
          if (!formId) throw new Error("Create or select a form first.");
          const scopeNodeTypeId = inputValue("form-scope-node-type-id");
          const payload = await request(`/api/admin/forms/${formId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("form-name"),
              slug: inputValue("form-slug"),
              scope_node_type_id: scopeNodeTypeId || null
            })
          });
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

      async function createBasicFormVersion() {
        try {
          if (!token) await login();
          const formId = inputValue("form-id");
          if (!formId) throw new Error("Create or select a form first.");
          const version = await request(`/api/admin/forms/${formId}/versions`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              version_label: inputValue("form-version-label"),
              compatibility_group_name: inputValue("compatibility-group-name")
            })
          });
          setInput("form-version-id", version.id);
          const section = await request(`/api/admin/form-versions/${version.id}/sections`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              title: inputValue("section-title"),
              position: Number(inputValue("section-position") || 0)
            })
          });
          setInput("section-id", section.id);
          const field = await request(`/api/admin/form-versions/${version.id}/fields`, {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              section_id: section.id,
              key: inputValue("field-key"),
              label: inputValue("field-label"),
              field_type: inputValue("field-type"),
              required: booleanInputValue("field-required"),
              position: Number(inputValue("field-position") || 0)
            })
          });
          setInput("field-id", field.id);
          show({ version, section, field });
          await renderForm(version.id);
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
              position: Number(inputValue("section-position") || 0)
            })
          });
          setInput("section-id", payload.id);
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
              position: Number(inputValue("section-position") || 0)
            })
          });
          show(payload);
          if (inputValue("form-version-id")) await renderForm(inputValue("form-version-id"));
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteSection() {
        try {
          if (!token) await login();
          const sectionId = inputValue("section-id");
          if (!sectionId) throw new Error("Select or enter a section ID first.");
          const payload = await request(`/api/admin/form-sections/${sectionId}`, {
            method: "DELETE"
          });
          setInput("section-id", "");
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
              position: Number(inputValue("field-position") || 0)
            })
          });
          setInput("field-id", payload.id);
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
              position: Number(inputValue("field-position") || 0)
            })
          });
          show(payload);
          if (inputValue("form-version-id")) await renderForm(inputValue("form-version-id"));
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteField() {
        try {
          if (!token) await login();
          const fieldId = inputValue("field-id");
          if (!fieldId) throw new Error("Select or enter a field ID first.");
          const payload = await request(`/api/admin/form-fields/${fieldId}`, {
            method: "DELETE"
          });
          setInput("field-id", "");
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

      async function publishAndPreviewVersion() {
        try {
          if (!token) await login();
          const formVersionId = inputValue("form-version-id");
          if (!formVersionId) throw new Error("Create or select a form version first.");
          const payload = await request(`/api/admin/form-versions/${formVersionId}/publish`, {
            method: "POST"
          });
          show(payload);
          await renderForm(formVersionId);
        } catch (error) {
          show(error.message);
        }
      }

      async function renderForm(formVersionId) {
        try {
          if (!formVersionId) throw new Error("Choose a published form before opening the response form.");
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
                  <button type="button" onclick="useSection('${escapeHtml(section.id)}', '${escapeHtml(section.title)}', ${section.position})">Use Section</button>
                  <div class="form-fields">
                    ${section.fields.map((field) => `
                      <div class="form-field">
                        <label for="${escapeHtml(fieldInputId(field))}">
                          ${escapeHtml(field.label)} (${escapeHtml(field.field_type)}${field.required ? ", required" : ""})
                        </label>
                        ${renderFieldInput(field)}
                        <button type="button" onclick="useField('${escapeHtml(field.id)}', '${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.field_type)}', ${field.required ? "true" : "false"}, ${field.position})">Use Field Settings</button>
                        <button type="button" onclick="useReportField('${escapeHtml(field.key)}', '${escapeHtml(field.label)}')">Use Report Source</button>
                      </div>
                    `).join("")}
                  </div>
                </section>
              `).join("")}
              ${renderResponseFormActions()}
            </article>
          `;
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

      function useSection(sectionId, sectionTitle = sectionId, position = 0) {
        selectRecord("form section", sectionTitle, sectionId, {
          "section-id": sectionId,
          "section-title": sectionTitle,
          "section-position": String(position)
        });
      }

      function useField(fieldId, fieldKey, fieldLabel = fieldKey, fieldType = "text", required = true, position = 0) {
        selectRecord("form field", fieldLabel, fieldId, {
          "field-id": fieldId,
          "field-key": fieldKey,
          "field-label": fieldLabel,
          "field-type": fieldType,
          "field-required": required ? "true" : "false",
          "field-position": String(position)
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
              <button type="button" onclick="useTargetNodeAndContinue('${escapeHtml(node.id)}', '${escapeHtml(node.name)}')">Use Target and Continue</button>
              <button type="button" onclick="useParentNode('${escapeHtml(node.id)}', '${escapeHtml(node.name)}')">Use Parent</button>
              <button type="button" onclick="useNodeForEditing('${escapeHtml(node.id)}', '${escapeHtml(node.name)}', '${escapeHtml(node.parent_node_id || "")}', '${escapeHtml(node.node_type_id)}', '${escapeHtml(node.node_type_name)}', '${escapeHtml(jsStringArg(JSON.stringify(node.metadata)))}')">Edit Node</button>
              <button type="button" onclick="useNodeType('${escapeHtml(node.node_type_id)}', '${escapeHtml(node.node_type_name)}')">Use Node Type</button>
              <code>${escapeHtml(node.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useNodeForEditing(nodeId, nodeName, parentNodeId, nodeTypeId, nodeTypeName, metadataJson) {
        selectRecord("node for editing", nodeName, nodeId, {
          "node-id": nodeId,
          "node-name": nodeName,
          "parent-node-id": parentNodeId,
          "node-type-id": nodeTypeId,
          "node-metadata-json": metadataJson
        });
        useNodeType(nodeTypeId, nodeTypeName);
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

      function useParentNode(nodeId, nodeName = nodeId) {
        selectRecord("parent node", nodeName, nodeId, {
          "parent-node-id": nodeId
        });
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
          selectedSubmissionFormVersionId = renderedForm?.form_version_id ?? inputValue("form-version-id");
          selectedSubmissionStatus = "draft";
          selectedSubmissionValues = {};
          show(payload);
          await loadSubmissionByValue(payload.id);
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
        selectedSubmissionFormVersionId = payload.form_version_id;
        selectedSubmissionStatus = payload.status;
        selectedSubmissionValues = submissionValuesByKey(payload.values);
        useSubmission(payload.id, `${payload.form_name} ${payload.version_label}`);
        useFormVersion(payload.form_version_id, payload.form_id, `${payload.form_name} ${payload.version_label}`);
        useTargetNode(payload.node_id, payload.node_name);
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>${escapeHtml(payload.form_name)} ${escapeHtml(payload.version_label)}</h3>
            <p>${escapeHtml(payload.node_name)}: ${escapeHtml(payload.status)}</p>
            <p class="muted">Created ${escapeHtml(payload.created_at)}${payload.submitted_at ? `; submitted ${escapeHtml(payload.submitted_at)}` : ""}</p>
            <button type="button" onclick="renderForm('${escapeHtml(payload.form_version_id)}')">Open Response Form</button>
            <h4>Values</h4>
            <ul>
              ${payload.values.map((value) => `
                <li>${escapeHtml(value.label)}${value.required ? " *" : ""}: ${value.value === null ? "<span class=\"muted\">missing</span>" : escapeHtml(JSON.stringify(value.value))}</li>
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
          await loadSubmissionByValue(submissionId);
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

      async function refreshAnalytics() {
        try {
          if (!token) await login();
          show(await request("/api/admin/analytics/refresh", { method: "POST" }));
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

      function datasetSourceDraftFromInputs() {
        const formId = inputValue("dataset-form-id") || inputValue("form-id");
        const compatibilityGroupId = inputValue("dataset-compatibility-group-id");
        return {
          source_alias: inputValue("dataset-source-alias") || "service",
          form_id: formId || null,
          compatibility_group_id: compatibilityGroupId || null,
          selection_rule: inputValue("dataset-selection-rule") || "all"
        };
      }

      function datasetFieldDraftFromInputs() {
        return {
          key: inputValue("dataset-field-key") || inputValue("report-logical-key"),
          label: inputValue("dataset-field-label") || inputValue("report-logical-key"),
          source_alias: inputValue("dataset-source-alias") || "service",
          source_field_key: inputValue("dataset-source-field-key") || inputValue("report-source-field-key"),
          position: datasetFields.length
        };
      }

      function renderDatasetDraft() {
        show({
          dataset_sources: datasetSources,
          dataset_fields: datasetFields
        });
        const sourceCards = datasetSources.map((source) => `
          <article class="card">
            <h3>Source ${escapeHtml(source.source_alias)}</h3>
            <p class="muted">${escapeHtml(source.form_id || source.compatibility_group_id || "Unbound source")}</p>
            <p>${escapeHtml(source.selection_rule)} records</p>
            <button type="button" onclick="useDatasetSource('${escapeHtml(source.source_alias)}', '${escapeHtml(source.form_id || "")}', '${escapeHtml(source.compatibility_group_id || "")}', '${escapeHtml(source.selection_rule)}')">Use Source</button>
            <button type="button" onclick="useDatasetSource('${escapeHtml(source.source_alias)}', '${escapeHtml(source.form_id || "")}', '${escapeHtml(source.compatibility_group_id || "")}', '${escapeHtml(source.selection_rule)}'); removeSelectedDatasetSource()">Remove Source</button>
          </article>
        `);
        const fieldCards = datasetFields.map((field) => `
          <article class="card">
            <h3>${escapeHtml(field.label)}</h3>
            <p>${escapeHtml(field.key)} from ${escapeHtml(field.source_alias)}.${escapeHtml(field.source_field_key)}</p>
            <button type="button" onclick="useDatasetField('${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.source_alias)}', '${escapeHtml(field.source_field_key)}', '${escapeHtml(field.field_type || inputValue("dataset-field-type") || "")}')">Use Dataset Field</button>
            <button type="button" onclick="useDatasetField('${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.source_alias)}', '${escapeHtml(field.source_field_key)}', '${escapeHtml(field.field_type || inputValue("dataset-field-type") || "")}'); removeSelectedDatasetField()">Remove Field</button>
          </article>
        `);
        document.getElementById("screen").innerHTML = (sourceCards.join("") + fieldCards.join("")) || '<p class="muted">No dataset draft sources or fields staged yet.</p>';
      }

      function addDatasetSource() {
        try {
          const source = datasetSourceDraftFromInputs();
          if (!source.source_alias) throw new Error("Enter a dataset source alias first.");
          if (!source.form_id && !source.compatibility_group_id) {
            throw new Error("Select a source form or compatibility group first.");
          }
          const index = datasetSources.findIndex((existing) => existing.source_alias === source.source_alias);
          if (index >= 0) {
            datasetSources[index] = source;
          } else {
            datasetSources.push(source);
          }
          renderDatasetDraft();
        } catch (error) {
          show(error.message);
        }
      }

      function removeSelectedDatasetSource() {
        try {
          const sourceAlias = inputValue("dataset-source-alias");
          if (!sourceAlias) throw new Error("Select or enter a dataset source alias first.");
          datasetSources.splice(0, datasetSources.length, ...datasetSources.filter((source) => source.source_alias !== sourceAlias));
          datasetFields.splice(0, datasetFields.length, ...datasetFields.filter((field) => field.source_alias !== sourceAlias).map((field, position) => ({ ...field, position })));
          renderDatasetDraft();
        } catch (error) {
          show(error.message);
        }
      }

      function clearDatasetSources() {
        datasetSources.splice(0, datasetSources.length);
        datasetFields.splice(0, datasetFields.length);
        renderDatasetDraft();
      }

      function addDatasetField() {
        try {
          const field = datasetFieldDraftFromInputs();
          if (!field.key) throw new Error("Enter a dataset field key first.");
          if (!field.source_alias) throw new Error("Enter a dataset source alias first.");
          const index = datasetFields.findIndex((existing) => existing.key === field.key);
          if (index >= 0) {
            datasetFields[index] = { ...field, position: index };
          } else {
            datasetFields.push(field);
          }
          datasetFields.forEach((existing, position) => { existing.position = position; });
          renderDatasetDraft();
        } catch (error) {
          show(error.message);
        }
      }

      function removeSelectedDatasetField() {
        try {
          const key = inputValue("dataset-field-key");
          if (!key) throw new Error("Select or enter a dataset field key first.");
          datasetFields.splice(0, datasetFields.length, ...datasetFields.filter((field) => field.key !== key).map((field, position) => ({ ...field, position })));
          renderDatasetDraft();
        } catch (error) {
          show(error.message);
        }
      }

      function clearDatasetFields() {
        datasetFields.splice(0, datasetFields.length);
        renderDatasetDraft();
      }

      function datasetDefinitionFromInputs() {
        const sources = datasetSources.length ? datasetSources : [datasetSourceDraftFromInputs()];
        const fields = datasetFields.length ? datasetFields : [datasetFieldDraftFromInputs()];
        return {
          name: inputValue("dataset-name"),
          slug: inputValue("dataset-slug"),
          grain: inputValue("dataset-grain") || "submission",
          composition_mode: inputValue("dataset-composition-mode") || "union",
          sources,
          fields
        };
      }

      async function createDataset() {
        try {
          if (!token) await login();
          const payload = await request("/api/admin/datasets", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(datasetDefinitionFromInputs())
          });
          setInput("dataset-id", payload.id);
          show(payload);
          await loadDatasets();
        } catch (error) {
          show(error.message);
        }
      }

      async function updateDataset() {
        try {
          if (!token) await login();
          const datasetId = inputValue("dataset-id");
          if (!datasetId) throw new Error("Select or enter a dataset ID first.");
          const payload = await request(`/api/admin/datasets/${datasetId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify(datasetDefinitionFromInputs())
          });
          show(payload);
          await loadDatasetByValue(datasetId);
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteDataset() {
        try {
          if (!token) await login();
          const datasetId = inputValue("dataset-id");
          if (!datasetId) throw new Error("Select or enter a dataset ID first.");
          const payload = await request(`/api/admin/datasets/${datasetId}`, {
            method: "DELETE"
          });
          setInput("dataset-id", "");
          show(payload);
          await loadDatasets();
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDatasets() {
        try {
          if (!token) await login();
          const payload = await request("/api/datasets");
          show(payload);
          showCards(payload, (dataset) => `
            <article class="card">
              <h3>${escapeHtml(dataset.name)}</h3>
              <p class="muted">${escapeHtml(dataset.slug)} · ${escapeHtml(dataset.grain)} grain · ${escapeHtml(dataset.composition_mode)} composition</p>
              <p>${dataset.source_count} sources, ${dataset.field_count} fields</p>
              <button type="button" onclick="useDataset('${escapeHtml(dataset.id)}', '${escapeHtml(dataset.name)}', '${escapeHtml(dataset.slug)}', '${escapeHtml(dataset.grain)}', '${escapeHtml(dataset.composition_mode)}')">Use Dataset</button>
              <button type="button" onclick="loadDatasetByValue('${escapeHtml(dataset.id)}')">Inspect Dataset</button>
              <button type="button" onclick="loadDatasetTableByValue('${escapeHtml(dataset.id)}')">Run Dataset</button>
              <code>${escapeHtml(dataset.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useDataset(datasetId, datasetName = datasetId, datasetSlug = "", grain = "submission", compositionMode = "union") {
        selectRecord("dataset", datasetName, datasetId, {
          "dataset-id": datasetId,
          "dataset-name": datasetName,
          "dataset-slug": datasetSlug,
          "dataset-grain": grain,
          "dataset-composition-mode": compositionMode
        });
      }

      function useDatasetSource(sourceAlias, formId = "", compatibilityGroupId = "", selectionRule = "all") {
        selectRecord("dataset source", sourceAlias, sourceAlias, {
          "dataset-source-alias": sourceAlias,
          "dataset-form-id": formId,
          "dataset-compatibility-group-id": compatibilityGroupId,
          "dataset-selection-rule": selectionRule
        });
        if (formId) useForm(formId, formId);
        if (compatibilityGroupId) useCompatibilityGroup(compatibilityGroupId, compatibilityGroupId);
      }

      async function loadDatasetById() {
        try {
          const datasetId = inputValue("dataset-id");
          if (!datasetId) throw new Error("Enter or select a dataset ID first.");
          await loadDatasetByValue(datasetId);
        } catch (error) {
          show(error.message);
        }
      }

      async function loadDatasetByValue(datasetId) {
        if (!token) await login();
        const payload = await request(`/api/datasets/${datasetId}`);
        useDataset(
          payload.id,
          payload.name,
          payload.slug,
          payload.grain,
          payload.composition_mode
        );
        datasetSources.splice(0, datasetSources.length, ...payload.sources.map((source) => ({
          source_alias: source.source_alias,
          form_id: source.form_id,
          compatibility_group_id: source.compatibility_group_id,
          selection_rule: source.selection_rule
        })));
        datasetFields.splice(0, datasetFields.length, ...payload.fields.map((field, position) => ({
          key: field.key,
          label: field.label,
          source_alias: field.source_alias,
          source_field_key: field.source_field_key,
          field_type: field.field_type,
          position
        })));
        const firstSource = payload.sources[0];
        if (firstSource) {
          useDatasetSource(
            firstSource.source_alias,
            firstSource.form_id || "",
            firstSource.compatibility_group_id || "",
            firstSource.selection_rule
          );
        }
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Dataset Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">${escapeHtml(payload.slug)} · ${escapeHtml(payload.grain)} grain · ${escapeHtml(payload.composition_mode)} composition</p>
            <p>${payload.sources.length} sources, ${payload.fields.length} fields</p>
            <p>${payload.reports.length} linked reports</p>
            <button type="button" onclick="renderDatasetDraft()">Review Draft Inputs</button>
          </article>
          ${payload.sources.map((source) => `
            <article class="card">
              <h3>Source ${escapeHtml(source.source_alias)}</h3>
              <p class="muted">${escapeHtml(source.form_name || source.compatibility_group_name || "Unresolved source")}</p>
              <p>${escapeHtml(source.selection_rule)} records</p>
              <button type="button" onclick="useDatasetSource('${escapeHtml(source.source_alias)}', '${escapeHtml(source.form_id || "")}', '${escapeHtml(source.compatibility_group_id || "")}', '${escapeHtml(source.selection_rule)}')">Use Source</button>
            </article>
          `).join("")}
          ${payload.fields.map((field) => `
            <article class="card">
              <h3>${escapeHtml(field.label)}</h3>
              <p>${escapeHtml(field.key)} from ${escapeHtml(field.source_alias)}.${escapeHtml(field.source_field_key)}</p>
              <p class="muted">${escapeHtml(field.field_type)}</p>
              <button type="button" onclick="useDatasetField('${escapeHtml(field.key)}', '${escapeHtml(field.label)}', '${escapeHtml(field.source_alias)}', '${escapeHtml(field.source_field_key)}', '${escapeHtml(field.field_type)}')">Use Dataset Field</button>
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
        `;
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
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Dataset Rows</h3>
            <p>${payload.rows.length} rows returned.</p>
          </article>
          ${payload.rows.map((row) => `
            <article class="card">
              <h3>${escapeHtml(row.node_name)}</h3>
              <p class="muted">${escapeHtml(row.source_alias)} source · ${escapeHtml(row.submission_id)}</p>
              <div class="table-wrap">
                <table>
                  <thead>
                    <tr><th>Field</th><th>Value</th></tr>
                  </thead>
                  <tbody>
                    ${Object.entries(row.values).map(([key, value]) => `
                      <tr>
                        <td>${escapeHtml(key)}</td>
                        <td>${escapeHtml(value ?? "")}</td>
                      </tr>
                    `).join("")}
                  </tbody>
                </table>
              </div>
              <div class="actions">
                ${datasetRowSubmissionActions(row)}
              </div>
            </article>
          `).join("") || '<p class="muted">No submitted rows matched this dataset.</p>'}
        `;
      }

      function useDatasetField(key, label, sourceAlias, sourceFieldKey, fieldType) {
        selectRecord("dataset field", label || key, key, {
          "dataset-field-key": key,
          "dataset-field-label": label,
          "dataset-source-alias": sourceAlias,
          "dataset-source-field-key": sourceFieldKey,
          "dataset-field-type": fieldType,
          "report-logical-key": key,
          "report-source-field-key": sourceFieldKey,
          "report-computed-expression": ""
        });
      }

      async function createReport() {
        try {
          if (!token) await login();
          const formId = inputValue("form-id");
          const datasetId = inputValue("dataset-id");
          const bindingsJson = inputValue("report-fields-json");
          const fields = bindingsJson ? JSON.parse(bindingsJson) : [{
            logical_key: inputValue("report-logical-key"),
            source_field_key: inputValue("report-computed-expression") ? null : inputValue("report-source-field-key"),
            computed_expression: inputValue("report-computed-expression") || null,
            missing_policy: inputValue("report-missing-policy") || "null"
          }];
          const payload = await request("/api/admin/reports", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("report-name"),
              form_id: datasetId ? null : formId || null,
              dataset_id: datasetId || null,
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
          const datasetId = inputValue("dataset-id");
          const bindingsJson = inputValue("report-fields-json");
          const fields = bindingsJson ? JSON.parse(bindingsJson) : [{
            logical_key: inputValue("report-logical-key"),
            source_field_key: inputValue("report-computed-expression") ? null : inputValue("report-source-field-key"),
            computed_expression: inputValue("report-computed-expression") || null,
            missing_policy: inputValue("report-missing-policy") || "null"
          }];
          const payload = await request(`/api/admin/reports/${reportId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("report-name"),
              form_id: datasetId ? null : formId || null,
              dataset_id: datasetId || null,
              fields
            })
          });
          show(payload);
          await loadReportDefinition(reportId);
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteReport() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          if (!reportId) throw new Error("Select or enter a report ID first.");
          const payload = await request(`/api/admin/reports/${reportId}`, {
            method: "DELETE"
          });
          setInput("report-id", "");
          setInput("report-fields-json", "");
          reportBindings = [];
          show(payload);
          await loadReports();
        } catch (error) {
          show(error.message);
        }
      }

      function addReportBinding() {
        try {
          const binding = {
            logical_key: inputValue("report-logical-key"),
            source_field_key: inputValue("report-computed-expression") ? null : inputValue("report-source-field-key"),
            computed_expression: inputValue("report-computed-expression") || null,
            missing_policy: inputValue("report-missing-policy") || "null"
          };
          if (!binding.logical_key || (!binding.source_field_key && !binding.computed_expression)) {
            throw new Error("Select or enter a report logical key plus a source field key or computed expression first.");
          }
          reportBindings = reportBindings.filter((existing) => existing.logical_key !== binding.logical_key);
          reportBindings.push(binding);
          document.getElementById("report-fields-json").value = JSON.stringify(reportBindings);
          renderReportBindings();
        } catch (error) {
          show(error.message);
        }
      }

      function removeSelectedReportBinding() {
        try {
          const logicalKey = inputValue("report-logical-key");
          if (!logicalKey) throw new Error("Select or enter a report logical key first.");
          reportBindings = reportBindings.filter((existing) => existing.logical_key !== logicalKey);
          document.getElementById("report-fields-json").value = reportBindings.length ? JSON.stringify(reportBindings) : "";
          renderReportBindings();
        } catch (error) {
          show(error.message);
        }
      }

      function clearReportBindings() {
        reportBindings = [];
        document.getElementById("report-fields-json").value = "";
        renderReportBindings();
      }

      function renderReportBindings() {
        show({ report_bindings: reportBindings });
        showCards(reportBindings, (binding) => `
            <article class="card">
              <h3>${escapeHtml(binding.logical_key)}</h3>
              <p>${escapeHtml(binding.source_field_key || binding.computed_expression)} with ${escapeHtml(binding.missing_policy)}</p>
              <button type="button" onclick="useReportBinding('${escapeHtml(binding.logical_key)}', '${escapeHtml(binding.source_field_key || "")}', '${escapeHtml(binding.missing_policy)}', '${escapeHtml(binding.computed_expression || "")}')">Use Binding</button>
              <button type="button" onclick="useReportBinding('${escapeHtml(binding.logical_key)}', '${escapeHtml(binding.source_field_key || "")}', '${escapeHtml(binding.missing_policy)}', '${escapeHtml(binding.computed_expression || "")}'); removeSelectedReportBinding()">Remove Binding</button>
            </article>
          `);
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

      async function createAggregation() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          if (!reportId) throw new Error("Select or enter a report ID first.");
          const metricKind = inputValue("aggregation-metric-kind") || "sum";
          const sourceLogicalKey = inputValue("aggregation-source-logical-key");
          const payload = await request("/api/admin/aggregations", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("aggregation-name") || "Aggregation",
              report_id: reportId,
              group_by_logical_key: inputValue("aggregation-group-by-logical-key") || null,
              metrics: [{
                metric_key: inputValue("aggregation-metric-key") || "value",
                source_logical_key: metricKind === "count" ? null : sourceLogicalKey || null,
                metric_kind: metricKind
              }]
            })
          });
          setInput("aggregation-id", payload.id);
          show(payload);
          await loadAggregations();
        } catch (error) {
          show(error.message);
        }
      }

      async function updateAggregation() {
        try {
          if (!token) await login();
          const aggregationId = inputValue("aggregation-id");
          const reportId = inputValue("report-id");
          if (!aggregationId) throw new Error("Select or enter an aggregation ID first.");
          if (!reportId) throw new Error("Select or enter a report ID first.");
          const metricKind = inputValue("aggregation-metric-kind") || "sum";
          const sourceLogicalKey = inputValue("aggregation-source-logical-key");
          const payload = await request(`/api/admin/aggregations/${aggregationId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("aggregation-name") || "Aggregation",
              report_id: reportId,
              group_by_logical_key: inputValue("aggregation-group-by-logical-key") || null,
              metrics: [{
                metric_key: inputValue("aggregation-metric-key") || "value",
                source_logical_key: metricKind === "count" ? null : sourceLogicalKey || null,
                metric_kind: metricKind
              }]
            })
          });
          show(payload);
          await loadAggregationDefinitionByValue(aggregationId);
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteAggregation() {
        try {
          if (!token) await login();
          const aggregationId = inputValue("aggregation-id");
          if (!aggregationId) throw new Error("Select or enter an aggregation ID first.");
          const payload = await request(`/api/admin/aggregations/${aggregationId}`, {
            method: "DELETE"
          });
          setInput("aggregation-id", "");
          show(payload);
          await loadAggregations();
        } catch (error) {
          show(error.message);
        }
      }

      function useAggregation(aggregationId, aggregationName = aggregationId, reportId = "", reportName = reportId) {
        selectRecord("aggregation", aggregationName, aggregationId, {
          "aggregation-id": aggregationId,
          "aggregation-name": aggregationName,
          ...(reportId ? { "report-id": reportId } : {})
        });
        if (reportId) {
          useReport(reportId, reportName || reportId);
        }
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
              <code>${escapeHtml(aggregation.id)}</code>
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
        setInput("aggregation-name", payload.name);
        setInput("report-id", payload.report_id);
        setInput("aggregation-group-by-logical-key", payload.group_by_logical_key || "");
        if (payload.metrics?.length) {
          const metric = payload.metrics[0];
          setInput("aggregation-metric-key", metric.metric_key || "");
          setInput("aggregation-source-logical-key", metric.source_logical_key || "");
          setInput("aggregation-metric-kind", metric.metric_kind || "");
        }
        useAggregation(payload.id, payload.name, payload.report_id, payload.report_name);
        show(payload);
        document.getElementById("screen").innerHTML = `
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
        `;
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
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Aggregation Results</h3>
            <p>${payload.rows.length} rows returned.</p>
            ${aggregationRowsView(payload.rows)}
          </article>
        `;
      }

      async function createChart() {
        try {
          if (!token) await login();
          const reportId = inputValue("report-id");
          const aggregationId = inputValue("aggregation-id");
          const payload = await request("/api/admin/charts", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("chart-name"),
              report_id: aggregationId ? null : reportId || null,
              aggregation_id: aggregationId || null,
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
          const aggregationId = inputValue("aggregation-id");
          const payload = await request(`/api/admin/charts/${chartId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              name: inputValue("chart-name"),
              report_id: aggregationId ? null : reportId || null,
              aggregation_id: aggregationId || null,
              chart_type: inputValue("chart-type") || "table"
            })
          });
          show(payload);
          await loadCharts();
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteChart() {
        try {
          if (!token) await login();
          const chartId = inputValue("chart-id");
          if (!chartId) throw new Error("Select or enter a chart ID first.");
          const payload = await request(`/api/admin/charts/${chartId}`, {
            method: "DELETE"
          });
          setInput("chart-id", "");
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
              <p class="muted">${chart.aggregation_id ? `Aggregation ${escapeHtml(chart.aggregation_name || chart.aggregation_id)}${chart.aggregation_report_name ? ` from ${escapeHtml(chart.aggregation_report_name)}` : ""}` : `Report ${escapeHtml(chart.report_name || "None")}${chart.report_form_name ? ` on ${escapeHtml(chart.report_form_name)}` : ""}`}</p>
              <button type="button" onclick="useChart('${escapeHtml(chart.id)}', '${escapeHtml(chart.name)}', '${escapeHtml(chart.report_id || "")}', '${escapeHtml(chart.report_name || "")}', '${escapeHtml(chart.chart_type)}', '${escapeHtml(chart.aggregation_id || "")}', '${escapeHtml(chart.aggregation_name || "")}')">Use Chart Context</button>
              ${chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(chart.report_id)}')">Run Report</button>` : ""}
              ${chart.aggregation_id ? `<button type="button" onclick="loadAggregationByValue('${escapeHtml(chart.aggregation_id)}')">Run Aggregation</button>` : ""}
              <code>${escapeHtml(chart.id)}</code>
            </article>
          `);
        } catch (error) {
          show(error.message);
        }
      }

      function useChart(chartId, chartName = chartId, reportId = "", reportName = reportId, chartType = "table", aggregationId = "", aggregationName = aggregationId) {
        selectRecord("chart", chartName, chartId, {
          "chart-id": chartId,
          "chart-name": chartName,
          "chart-type": chartType,
          ...(reportId ? { "report-id": reportId, "aggregation-id": "" } : {}),
          ...(aggregationId ? { "aggregation-id": aggregationId, "report-id": "" } : {})
        });
        if (reportId) {
          selectRecord("report", reportName || reportId, reportId, {
            "report-id": reportId
          });
        }
        if (aggregationId) {
          useAggregation(aggregationId, aggregationName || aggregationId);
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

      async function deleteDashboard() {
        try {
          if (!token) await login();
          const dashboardId = inputValue("dashboard-id");
          if (!dashboardId) throw new Error("Select or enter a dashboard ID first.");
          const payload = await request(`/api/admin/dashboards/${dashboardId}`, {
            method: "DELETE"
          });
          setInput("dashboard-id", "");
          setInput("dashboard-component-id", "");
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
              position: Number(inputValue("dashboard-component-position") || 0),
              config: dashboardComponentConfig()
            })
          });
          setInput("dashboard-component-id", payload.id);
          show(payload);
          await loadDashboardByValue(dashboardId);
        } catch (error) {
          show(error.message);
        }
      }

      async function updateDashboardComponent() {
        try {
          if (!token) await login();
          const componentId = inputValue("dashboard-component-id");
          const dashboardId = inputValue("dashboard-id");
          const chartId = inputValue("chart-id");
          if (!componentId || !chartId) {
            throw new Error("Select or enter a dashboard component ID and chart ID first.");
          }
          const payload = await request(`/api/admin/dashboard-components/${componentId}`, {
            method: "PUT",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({
              chart_id: chartId,
              position: Number(inputValue("dashboard-component-position") || 0),
              config: dashboardComponentConfig()
            })
          });
          show(payload);
          if (dashboardId) await loadDashboardByValue(dashboardId);
        } catch (error) {
          show(error.message);
        }
      }

      async function deleteDashboardComponent() {
        try {
          if (!token) await login();
          const componentId = inputValue("dashboard-component-id");
          const dashboardId = inputValue("dashboard-id");
          if (!componentId) {
            throw new Error("Select or enter a dashboard component ID first.");
          }
          const payload = await request(`/api/admin/dashboard-components/${componentId}`, {
            method: "DELETE"
          });
          setInput("dashboard-component-id", "");
          show(payload);
          if (dashboardId) await loadDashboardByValue(dashboardId);
        } catch (error) {
          show(error.message);
        }
      }

      function dashboardComponentConfig() {
        const configJson = inputValue("dashboard-component-config-json");
        const config = configJson ? JSON.parse(configJson) : {};
        const title = inputValue("dashboard-component-title");
        if (title) {
          config.title = title;
        } else if (!config.title) {
          config.title = inputValue("chart-name") || "Chart";
        }
        return config;
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
          "dashboard-id": dashboardId,
          "dashboard-name": dashboardName
        });
      }

      function useReport(reportId, reportName = reportId) {
        demoReportId = reportId;
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
        document.getElementById("report-id").value = payload.id;
        if (payload.form_id) document.getElementById("form-id").value = payload.form_id;
        if (payload.dataset_id) document.getElementById("dataset-id").value = payload.dataset_id;
        useReport(payload.id, payload.name);
        if (payload.form_id) useForm(payload.form_id, payload.form_name || payload.form_id);
        if (payload.dataset_id) useDataset(payload.dataset_id, payload.dataset_name || payload.dataset_id);
        document.getElementById("report-fields-json").value = JSON.stringify(payload.bindings.map((binding) => ({
          logical_key: binding.logical_key,
            source_field_key: binding.source_field_key,
            computed_expression: binding.computed_expression,
            missing_policy: binding.missing_policy
          })));
        reportBindings = payload.bindings.map((binding) => ({
          logical_key: binding.logical_key,
          source_field_key: binding.source_field_key,
          missing_policy: binding.missing_policy
        }));
        show(payload);
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Report Definition</h3>
            <p>${escapeHtml(payload.name)}</p>
            <p class="muted">${payload.dataset_id ? `Dataset ${escapeHtml(payload.dataset_name || payload.dataset_id)}` : `Form ${escapeHtml(payload.form_name || payload.form_id || "Any form")}`}</p>
            <p>${payload.bindings.length} field bindings</p>
            <button type="button" onclick="loadReportByValue('${escapeHtml(payload.id)}')">Run This Report</button>
          </article>
          ${payload.bindings.map((binding) => `
            <article class="card">
              <h3>${escapeHtml(binding.logical_key)}</h3>
              <p>${escapeHtml(binding.source_field_key || binding.computed_expression)} with ${escapeHtml(binding.missing_policy)}</p>
              <button type="button" onclick="useReportBinding('${escapeHtml(binding.logical_key)}', '${escapeHtml(binding.source_field_key || "")}', '${escapeHtml(binding.missing_policy)}', '${escapeHtml(binding.computed_expression || "")}')">Use Binding</button>
              <button type="button" onclick="useReportBinding('${escapeHtml(binding.logical_key)}', '${escapeHtml(binding.source_field_key || "")}', '${escapeHtml(binding.missing_policy)}', '${escapeHtml(binding.computed_expression || "")}'); removeSelectedReportBinding()">Remove Binding</button>
            </article>
          `).join("") || '<p class="muted">No report bindings configured.</p>'}
        `;
      }

      function useReportBinding(logicalKey, sourceFieldKey, missingPolicy, computedExpression = "") {
        selectRecord("report binding", logicalKey, sourceFieldKey || computedExpression, {
          "report-logical-key": logicalKey,
          "report-source-field-key": sourceFieldKey,
          "report-computed-expression": computedExpression,
          "report-missing-policy": missingPolicy,
          "dataset-field-key": logicalKey,
          "dataset-field-label": logicalKey,
          "dataset-source-field-key": sourceFieldKey
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
              <button type="button" onclick="useDashboardComponent('${escapeHtml(component.id)}', '${escapeHtml(component.chart.id)}', ${component.position}, '${escapeHtml(jsStringArg(JSON.stringify(component.config)))}', '${escapeHtml(component.chart.name)}', '${escapeHtml(component.chart.chart_type)}', '${escapeHtml(component.chart.report_id || "")}', '${escapeHtml(component.chart.report_name || "")}', '${escapeHtml(component.chart.aggregation_id || "")}', '${escapeHtml(component.chart.aggregation_name || "")}')">Use Component Context</button>
              ${component.chart.report_id ? `<button type="button" onclick="loadReportByValue('${escapeHtml(component.chart.report_id)}')">Open Report</button>` : ""}
              ${component.chart.aggregation_id ? `<button type="button" onclick="loadAggregationByValue('${escapeHtml(component.chart.aggregation_id)}')">Open Aggregation</button>` : ""}
              ${component.chart.aggregation_id ? aggregationRowsView(aggregationRows) : reportRowsView(rows)}
            </article>
          `;
        }));
        document.getElementById("screen").innerHTML = cards.length
          ? header + cards.join("")
          : header + '<p class="muted">No dashboard components found.</p>';
      }

      function useDashboardComponent(componentId, chartId, position, configJson, chartName = chartId, chartType = "table", reportId = "", reportName = reportId, aggregationId = "", aggregationName = aggregationId) {
        const config = JSON.parse(configJson || "{}");
        selectRecord("dashboard component", componentId, componentId, {
          "dashboard-component-id": componentId,
          "chart-id": chartId,
          "chart-name": chartName,
          "chart-type": chartType,
          "dashboard-component-position": String(position),
          "dashboard-component-title": config.title || chartName,
          "dashboard-component-config-json": configJson,
          ...(reportId ? { "report-id": reportId, "aggregation-id": "" } : {}),
          ...(aggregationId ? { "aggregation-id": aggregationId, "report-id": "" } : {})
        });
        if (reportId) {
          useReport(reportId, reportName || reportId);
        }
        if (aggregationId) {
          useAggregation(aggregationId, aggregationName || aggregationId);
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
        document.getElementById("screen").innerHTML = `
          <article class="card">
            <h3>Report Results</h3>
            <p>${payload.rows.length} rows returned.</p>
            ${reportRowsView(payload.rows)}
          </article>
        `;
      }
"#;
