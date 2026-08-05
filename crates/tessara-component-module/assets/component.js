const COMPLETE_DOCUMENT_ROOT_ID = "module-content";

function bootstrap() {
  const node = document.getElementById("tessara-component-bootstrap");
  if (!node) return null;
  try {
    return JSON.parse(node.textContent);
  } catch {
    return null;
  }
}

function status(message, error = false, root = document) {
  const node = root.querySelector("[data-component-status]");
  if (node) {
    node.textContent = message;
    node.setAttribute("role", error ? "alert" : "status");
    node.setAttribute("aria-busy", "false");
  }
}

async function request(url, method, payload) {
  const response = await fetch(url, {
    method,
    headers: { "content-type": "application/json" },
    body: JSON.stringify(payload),
  });
  if (!response.ok) {
    const failure = await response.json().catch(() => ({}));
    throw new Error(failure.message || failure.error || "Component could not be saved");
  }
  return response.json();
}

async function save(form) {
  const data = new FormData(form);
  const dataset = JSON.parse(String(data.get("dataset_reference")));
  const version = {
    schema_version: 1,
    version: {
      dataset_reference: dataset,
      component_type: String(data.get("component_type")),
      config: JSON.parse(String(data.get("config"))),
      version_note: String(data.get("version_note") || ""),
    },
  };
  const metadata = {
    schema_version: 1,
    name: String(data.get("name")),
    slug: String(data.get("slug")),
    description: String(data.get("description") || "") || null,
  };
  const componentId = form.dataset.componentId;
  status("Saving…", false, form);
  let result;
  if (componentId) {
    await request(`/api/admin/components/${componentId}`, "PUT", metadata);
    const draftId = form.dataset.draftId;
    result = await request(
      draftId
        ? `/api/admin/components/${componentId}/versions/${draftId}`
        : `/api/admin/components/${componentId}/versions`,
      draftId ? "PUT" : "POST",
      version,
    );
  } else {
    result = await request("/api/admin/components", "POST", {
      ...metadata,
      version: version.version,
    });
  }
  location.assign(`/components/${result.component_id}/versions`);
}

function element(name, text, className) {
  const node = document.createElement(name);
  if (text !== undefined && text !== null) node.textContent = String(text);
  if (className) node.className = className;
  return node;
}

function renderExecution(target, execution) {
  target.replaceChildren();
  if (execution.component_type === "table") {
    const table = element("table", null, "component-table");
    const head = document.createElement("thead");
    const heading = document.createElement("tr");
    for (const column of execution.columns || []) {
      heading.append(element("th", column.label || column.key));
    }
    head.append(heading);
    const body = document.createElement("tbody");
    for (const row of execution.rows || []) {
      const rendered = document.createElement("tr");
      for (const column of execution.columns || []) {
        rendered.append(element("td", row.values?.[column.key] ?? "—"));
      }
      body.append(rendered);
    }
    table.append(head, body);
    target.append(table);
    return;
  }
  if (execution.component_type === "stat_card") {
    const card = element("article", null, "component-stat");
    card.append(
      element("p", execution.stat?.label || "Value", "eyebrow"),
      element("p", execution.stat?.display_value ?? "—", "component-stat__value"),
    );
    if (execution.stat?.supporting_text) {
      card.append(element("p", execution.stat.supporting_text));
    }
    target.append(card);
    return;
  }
  const list = element("ul", null, "component-visual-list");
  const values = execution.slices?.length ? execution.slices : execution.points || [];
  for (const value of values) {
    const label = value.category ?? value.x ?? "Value";
    const comparison = value.comparison ? ` · ${value.comparison}` : "";
    list.append(element("li", `${label}${comparison}: ${value.display_value ?? value.value}`));
  }
  target.append(list);
}

function loadComponentRenderers(root) {
  for (const panel of root.querySelectorAll("[data-component-render]")) {
    const componentRef = panel.dataset.componentRef;
    const versionId = panel.dataset.versionId;
    const kind = panel.dataset.componentKind.replaceAll("_", "-");
    request(
      `/api/components/${componentRef}/versions/${versionId}/${kind}`,
      "GET",
    )
      .then((execution) => {
        renderExecution(panel.querySelector("[data-component-render-content]"), execution);
        panel.setAttribute("aria-busy", "false");
        status("Component data loaded.", false, panel);
      })
      .catch((error) => {
        panel.setAttribute("aria-busy", "false");
        status(error.message, true, panel);
      });
  }
}

function installAuthoringHandler(root, signal) {
  root.addEventListener(
    "submit",
    (event) => {
      const form = event.target.closest("[data-component-create]");
      if (!form) return;
      event.preventDefault();
      form.querySelector("button[type=submit]")?.setAttribute("disabled", "");
      save(form).catch((error) => {
        status(error.message, true, form);
        form.querySelector("button[type=submit]")?.removeAttribute("disabled");
      });
    },
    { signal },
  );
  root.addEventListener(
    "click",
    (event) => {
      const button = event.target.closest("[data-component-version-action]");
      if (!button) return;
      const panel = button.closest("[data-component-versions]");
      const componentId = panel?.dataset.componentId;
      const versionId = button.dataset.versionId;
      const action = button.dataset.componentVersionAction;
      if (!componentId || !versionId || !action) return;
      button.setAttribute("disabled", "");
      status(`${action[0].toUpperCase()}${action.slice(1)} in progress…`, false, panel);
      let operation;
      if (action === "publish") {
        operation = request(
          `/api/admin/components/${componentId}/versions/${versionId}/publish`,
          "POST",
        );
      } else if (action === "delete") {
        operation = request(
          `/api/admin/components/${componentId}/versions/${versionId}`,
          "DELETE",
        );
      } else {
        operation = request(
          `/api/admin/components/${componentId}/versions/${versionId}/lifecycle`,
          "POST",
          {
            schema_version: 1,
            action,
            expected_resource_revision: Number(button.dataset.resourceRevision),
          },
        );
      }
      operation
        .then(() => location.reload())
        .catch((error) => {
          status(error.message, true, panel);
          button.removeAttribute("disabled");
        });
    },
    { signal },
  );
}

async function renderCompleteDocumentContent(outlet, path, signal) {
  const response = await fetch(path, {
    headers: { Accept: "text/html" },
    signal,
  });
  if (!response.ok) {
    throw new Error(`Component document returned HTTP ${response.status}`);
  }
  const parsed = new DOMParser().parseFromString(await response.text(), "text/html");
  const content = parsed.getElementById(COMPLETE_DOCUMENT_ROOT_ID);
  if (!content) throw new Error("Component document did not contain its module content");
  outlet.replaceChildren(...Array.from(content.childNodes));
}

if (document.getElementById(COMPLETE_DOCUMENT_ROOT_ID)) {
  const controller = new AbortController();
  installAuthoringHandler(document, controller.signal);
  loadComponentRenderers(document);
  bootstrap();
}

export async function createModule(host) {
  if (!host || host.lifecycleAbi !== "1.0.0") {
    throw new Error("Components requires Tessara browser lifecycle ABI 1.0.0");
  }
  let controller = new AbortController();
  let outlet = null;
  let disposed = false;
  const assertActive = () => {
    if (disposed) throw new Error("Component lifecycle instance is disposed");
  };
  const render = async (input) => {
    assertActive();
    outlet = document.getElementById(input.outletId);
    if (!outlet) throw new Error("Component lifecycle outlet is unavailable");
    controller.abort();
    controller = new AbortController();
    await renderCompleteDocumentContent(outlet, input.bootstrap.path, controller.signal);
    installAuthoringHandler(outlet, controller.signal);
    loadComponentRenderers(outlet);
  };
  return {
    async mount(input) {
      await render(input);
    },
    async navigate(input) {
      await render(input);
    },
    async canDeactivate() {
      return { allowed: true };
    },
    async suspend() {
      assertActive();
      if (outlet) outlet.hidden = true;
    },
    async resume() {
      assertActive();
      if (outlet) outlet.hidden = false;
    },
    async unmount() {
      controller.abort();
      if (outlet) outlet.replaceChildren();
    },
    async dispose() {
      if (!disposed) {
        controller.abort();
        if (outlet) outlet.replaceChildren();
        disposed = true;
      }
    },
  };
}
