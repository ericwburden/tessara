const COMPLETE_DOCUMENT_ROOT_ID = "module-content";
const SVG_NS = "http://www.w3.org/2000/svg";
const CHART_COLORS = ["#3568d4", "#d45b35", "#2f8f63", "#8a55c5", "#c18a1d", "#267d91"];
const CATEGORY_COLOR_OPTIONS = [
  ["", "Default"],
  ["var(--semantic-primary)", "Primary"],
  ["var(--semantic-success)", "Success"],
  ["var(--semantic-info)", "Info"],
  ["var(--semantic-warning)", "Warning"],
  ["var(--semantic-danger)", "Danger"],
  ["var(--color-cyan)", "Cyan"],
  ["var(--semantic-secondary)", "Secondary"],
];

function readBootstrap(root = document) {
  const node = root.querySelector?.("#tessara-component-bootstrap") ??
    document.getElementById("tessara-component-bootstrap");
  if (!node) return null;
  try {
    return JSON.parse(node.textContent);
  } catch {
    return null;
  }
}

function status(message, error = false, root = document) {
  const node = root.querySelector?.("[data-component-status]");
  if (!node) return;
  node.textContent = message;
  node.setAttribute("role", error ? "alert" : "status");
  node.setAttribute("aria-busy", "false");
}

const pendingMutationKeys = new WeakMap();

function mutationKey(owner, action) {
  let keys = pendingMutationKeys.get(owner);
  if (!keys) {
    keys = new Map();
    pendingMutationKeys.set(owner, keys);
  }
  if (!keys.has(action)) {
    const nonce = globalThis.crypto?.randomUUID?.() ??
      `${Date.now()}-${Math.random().toString(16).slice(2)}`;
    keys.set(action, `component-ui-${action}-${nonce}`);
  }
  return keys.get(action);
}

function clearMutationKey(owner, action) {
  pendingMutationKeys.get(owner)?.delete(action);
}

async function request(url, method = "GET", payload, idempotencyKey, signal) {
  const headers = {};
  const options = { method, headers, signal };
  if (payload !== undefined) {
    headers["content-type"] = "application/json";
    options.body = JSON.stringify(payload);
  }
  if (idempotencyKey) headers["x-idempotency-key"] = idempotencyKey;
  const response = await fetch(url, options);
  if (!response.ok) {
    const failure = await response.json().catch(() => ({}));
    const error = new Error(failure.message || failure.error || `Component request failed (${response.status})`);
    error.status = response.status;
    error.code = failure.code;
    error.retryable = failure.retryable;
    throw error;
  }
  return response.json();
}

function element(name, text, className) {
  const node = document.createElement(name);
  if (text !== undefined && text !== null) node.textContent = String(text);
  if (className) node.className = className;
  return node;
}

function selectedValues(root, selector) {
  return Array.from(root.querySelectorAll(selector))
    .filter((input) => input.checked || input.getAttribute("aria-checked") === "true")
    .map((input) => input.value);
}

function componentKindLabel(kind) {
  return kind === "stat_card"
    ? "Stat Card"
    : `${kind.charAt(0).toUpperCase()}${kind.slice(1).replaceAll("_", " ")}`;
}

function managedDirectoryState(definition) {
  const versions = definition.versions || [];
  const draft = versions.find((version) => version.publication_state === "draft");
  const published = versions.find((version) => version.publication_state === "published");
  const current = published || draft || versions[0];
  return {
    definition,
    current,
    status: draft && published ? "updating" : draft ? "draft" : published ? "published" : "superseded",
  };
}

function directoryActions(routeRef) {
  const actions = element("div", null, "component-actions");
  for (const [label, suffix, className, managed] of [
    ["View", "/view", "button button--small", false],
    ["Versions", "/versions", "button button--small button--secondary", true],
    ["Edit", "/edit", "button button--small button--secondary", true],
  ]) {
    const link = element("a", label, className);
    link.href = `/components/${encodeURIComponent(routeRef)}${suffix}`;
    if (managed) {
      link.dataset.componentManageOnly = "";
      link.classList.add("is-authorized");
    }
    actions.append(link);
  }
  return actions;
}

function enrichManagedDirectory(directory, definitions) {
  const tableBody = directory.querySelector(".components-list-responsive-table tbody");
  const mobile = directory.querySelector(".components-list-mobile");
  for (const definition of definitions) {
    const state = managedDirectoryState(definition);
    if (!state.current) continue;
    const id = definition.component_id;
    const routeRef = definition.slug || id;
    const kind = state.current.component_type;
    const statusLabel = `${state.status.charAt(0).toUpperCase()}${state.status.slice(1)}`;
    const existing = Array.from(directory.querySelectorAll("[data-component-directory-item]"))
      .filter((item) => item.dataset.componentId === id);
    if (existing.length) {
      for (const item of existing) {
        item.dataset.kind = kind;
        item.dataset.status = state.status;
        item.querySelector("[data-component-directory-kind]").textContent = componentKindLabel(kind);
        item.querySelector("[data-component-directory-status]").textContent = statusLabel;
      }
      continue;
    }
    const row = document.createElement("tr");
    row.dataset.componentDirectoryItem = "";
    row.dataset.componentId = id;
    row.dataset.name = definition.name.toLowerCase();
    row.dataset.kind = kind;
    row.dataset.status = state.status;
    const identity = document.createElement("td");
    const name = element("a", definition.name);
    name.href = `/components/${encodeURIComponent(routeRef)}`;
    identity.append(name, element("span", definition.slug, "component-directory__slug"));
    const kindCell = element("td", componentKindLabel(kind));
    kindCell.dataset.componentDirectoryKind = "";
    const statusCell = document.createElement("td");
    const badge = element("span", statusLabel, "component-badge");
    badge.dataset.componentDirectoryStatus = "";
    statusCell.append(badge);
    const actionCell = document.createElement("td");
    actionCell.append(directoryActions(routeRef));
    row.append(identity, kindCell, statusCell, actionCell);
    tableBody.append(row);

    const card = element("article", null, "component-card components-list-mobile-card");
    card.dataset.componentDirectoryItem = "";
    card.dataset.componentId = id;
    card.dataset.name = definition.name.toLowerCase();
    card.dataset.kind = kind;
    card.dataset.status = state.status;
    const summary = element("p", null, "eyebrow");
    const kindText = element("span", componentKindLabel(kind));
    kindText.dataset.componentDirectoryKind = "";
    const statusText = element("span", statusLabel);
    statusText.dataset.componentDirectoryStatus = "";
    summary.append(kindText, document.createTextNode(" · "), statusText);
    const heading = document.createElement("h2");
    const cardLink = element("a", definition.name);
    cardLink.href = `/components/${encodeURIComponent(routeRef)}`;
    heading.append(cardLink);
    card.append(summary, heading, element("p", definition.description || ""),
      element("p", definition.slug, "component-directory__slug"), directoryActions(routeRef));
    mobile.append(card);
  }
}

function initializeDirectory(root, signal) {
  const directory = root.querySelector?.("[data-component-directory]");
  if (!directory) return;
  const search = directory.querySelector("[data-component-directory-search]");
  const apply = () => {
    const kinds = new Set(selectedValues(directory, "[data-component-kind-filter]"));
    const statuses = new Set(selectedValues(directory, "[data-component-status-filter]"));
    const mobileKind = directory.querySelector("[data-component-mobile-kind-filter]").value;
    const mobileStatus = directory.querySelector("[data-component-mobile-status-filter]").value;
    if (mobileKind !== "all") kinds.add(mobileKind);
    if (mobileStatus !== "all") statuses.add(mobileStatus);
    const query = search.value.trim().toLowerCase();
    let visible = 0;
    for (const item of directory.querySelectorAll("[data-component-directory-item]")) {
      const matches = (!query || item.dataset.name.includes(query)) &&
        (!kinds.size || kinds.has(item.dataset.kind)) &&
        (!statuses.size || statuses.has(item.dataset.status));
      item.hidden = !matches;
      if (matches) visible += 1;
    }
    directory.querySelector("[data-component-directory-empty]").hidden = visible > 0;
  };
  search.addEventListener("input", apply, { signal });
  directory.addEventListener("change", (event) => {
    if (event.target.matches("[data-component-mobile-kind-filter], [data-component-mobile-status-filter]")) {
      const isKind = event.target.matches("[data-component-mobile-kind-filter]");
      const selector = isKind ? "[data-component-kind-filter]" : "[data-component-status-filter]";
      for (const choice of directory.querySelectorAll(selector)) {
        choice.setAttribute("aria-checked", String(choice.value === event.target.value));
      }
      apply();
    }
  }, { signal });
  directory.addEventListener("click", (event) => {
    const toggle = event.target.closest("[data-component-filter-toggle]");
    if (toggle) {
      const panel = directory.querySelector(`[data-component-filter-menu="${toggle.dataset.componentFilterToggle}"]`);
      const opening = panel.hidden;
      for (const other of directory.querySelectorAll("[data-component-filter-menu]")) other.hidden = true;
      for (const other of directory.querySelectorAll("[data-component-filter-toggle]")) {
        other.setAttribute("aria-expanded", "false");
      }
      panel.hidden = !opening;
      toggle.setAttribute("aria-expanded", String(opening));
      if (opening) panel.querySelector('[role="menuitemradio"]')?.focus();
      return;
    }
    const choice = event.target.closest("[data-component-kind-filter], [data-component-status-filter]");
    if (choice) {
      const isKind = choice.matches("[data-component-kind-filter]");
      const selector = isKind ? "[data-component-kind-filter]" : "[data-component-status-filter]";
      const mobile = directory.querySelector(
        isKind ? "[data-component-mobile-kind-filter]" : "[data-component-mobile-status-filter]",
      );
      for (const peer of directory.querySelectorAll(selector)) {
        peer.setAttribute("aria-checked", String(peer === choice));
      }
      mobile.value = choice.value;
      apply();
      return;
    }
    if (event.target.closest("[data-component-clear-filters]")) {
      for (const input of directory.querySelectorAll("[data-component-kind-filter], [data-component-status-filter]")) {
        input.setAttribute("aria-checked", "false");
      }
      directory.querySelector("[data-component-mobile-kind-filter]").value = "all";
      directory.querySelector("[data-component-mobile-status-filter]").value = "all";
      search.value = "";
      apply();
      return;
    }
    if (event.target.closest("[data-component-mobile-filters-open]")) {
      const dialog = directory.querySelector("[data-component-mobile-filters]");
      if (typeof dialog.showModal === "function") dialog.showModal();
      else dialog.setAttribute("open", "");
    }
  }, { signal });
  directory.addEventListener("keydown", (event) => {
    const panel = event.target.closest?.("[data-component-filter-menu]");
    if (!panel) return;
    const items = Array.from(panel.querySelectorAll('[role="menuitemradio"]'));
    const index = items.indexOf(event.target);
    if (event.key === "Escape") {
      panel.hidden = true;
      const toggle = directory.querySelector(
        `[data-component-filter-toggle="${panel.dataset.componentFilterMenu}"]`,
      );
      toggle.setAttribute("aria-expanded", "false");
      toggle.focus();
    } else if (["ArrowDown", "ArrowUp"].includes(event.key)) {
      event.preventDefault();
      const offset = event.key === "ArrowDown" ? 1 : -1;
      items[(index + offset + items.length) % items.length]?.focus();
    }
  }, { signal });
  const mobileStatusFilter = directory.querySelector("[data-component-mobile-status-filter]");
  if (!Array.from(mobileStatusFilter.options).some((option) => option.value === "updating")) {
    mobileStatusFilter.insertBefore(new Option("Updating", "updating"), mobileStatusFilter.lastElementChild);
  }
  request("/api/admin/components", "GET", undefined, undefined, signal)
    .then((definitions) => {
      enrichManagedDirectory(directory, definitions);
      const manageableIds = new Set(definitions.map((definition) => definition.component_id));
      for (const control of directory.querySelectorAll("[data-component-manage-only]")) {
        const item = control.closest("[data-component-directory-item]");
        if (!item || manageableIds.has(item.dataset.componentId)) {
          control.hidden = false;
          control.classList.add("is-authorized");
        }
      }
      apply();
    })
    .catch(() => {});
  apply();
}

function initializeManagementAffordances(root, signal) {
  if (root.querySelector?.("[data-component-directory]")) return;
  const controls = root.querySelectorAll?.(
    '[data-component-manage-only], [data-component-version-action], a[href^="/components/"][href$="/edit"]',
  ) || [];
  if (!controls.length) return;
  for (const control of controls) {
    control.dataset.componentManageOnly = "";
    if (!control.classList.contains("is-authorized")) control.hidden = true;
  }
  const componentId = root.querySelector?.("[data-component-render]")?.dataset.componentRef ||
    root.querySelector?.("[data-component-versions]")?.dataset.componentId ||
    root.querySelector?.("[data-component-create]")?.dataset.componentId;
  if (!componentId) return;
  request(`/api/admin/components/${componentId}`, "GET", undefined, undefined, signal)
    .then(() => {
      for (const control of controls) {
        control.hidden = false;
        control.classList.add("is-authorized");
      }
    })
    .catch(() => {});
}

function datasetReferenceKey(reference) {
  try {
    return JSON.stringify(reference);
  } catch {
    return "";
  }
}

function datasetForForm(form) {
  const key = form.querySelector("[data-component-dataset-picker]")?.value;
  return (form.__componentDatasets || []).find((dataset) => datasetReferenceKey(dataset.reference) === key);
}

function datasetMajor(dataset) {
  return dataset?.reference?.major ??
    String(dataset?.reference?.reference?.resource_id || "").split("@").at(-1) ?? "?";
}

function datasetProvenanceLabel(dataset) {
  const provenance = dataset?.provenance || {};
  const sources = [
    ...(provenance.forms || []).map((item) => item.name),
    ...(provenance.datasets || []).map((item) => `Dataset: ${item.name}`),
  ].filter(Boolean);
  return sources.length ? sources.join(", ") : "No direct sources";
}

function datasetPickerLabel(dataset) {
  if (!dataset) return "Select a Dataset version";
  const parts = [`${dataset.dataset_name} · v${datasetMajor(dataset)}`];
  if (dataset.grain) parts.push(`Grain: ${dataset.grain}`);
  if (dataset.tags?.length) parts.push(dataset.tags.join(", "));
  const provenance = datasetProvenanceLabel(dataset);
  if (provenance !== "No direct sources") parts.push(provenance);
  return parts.join(" · ");
}

function datasetPickerSearchText(dataset) {
  return [
    dataset.dataset_name,
    dataset.dataset_slug,
    `v${datasetMajor(dataset)}`,
    datasetMajor(dataset),
    dataset.grain,
    ...(dataset.tags || []),
    datasetProvenanceLabel(dataset),
    ...(dataset.fields || []).flatMap((field) => [field.key, field.label, field.field_type]),
  ].filter(Boolean).join(" ").toLowerCase();
}

function renderDatasetFieldPreview(target, dataset) {
  target.replaceChildren();
  const heading = element(
    "h3",
    dataset ? `${dataset.dataset_name} v${datasetMajor(dataset)} field preview` : "Dataset field preview",
  );
  target.append(heading);
  if (!dataset) {
    target.append(element("p", "Choose an available Dataset version to inspect its fields."));
    return;
  }
  const fields = dataset.fields || [];
  if (!fields.length) {
    target.append(element("p", "This Dataset version does not expose any fields."));
    return;
  }
  const table = element("table", null, "component-table component-dataset-picker__field-table");
  const head = document.createElement("thead");
  const header = document.createElement("tr");
  for (const label of ["Field", "Key", "Type", "Restriction"]) header.append(element("th", label));
  head.append(header);
  const body = document.createElement("tbody");
  for (const field of fields) {
    const row = document.createElement("tr");
    for (const value of [field.label, field.key, field.field_type, field.restriction_tier]) {
      row.append(element("td", value || "—"));
    }
    body.append(row);
  }
  table.append(head, body);
  target.append(table);
}

function initializeDatasetPicker(form, signal) {
  const picker = form.querySelector("[data-component-dataset-picker]");
  const mount = form.querySelector("[data-component-dataset-picker-enhancement]");
  if (!picker || !mount) return;
  const shell = element("div", null, "component-dataset-picker");
  const trigger = element("button", null, "component-dataset-picker__trigger");
  trigger.type = "button";
  trigger.setAttribute("role", "combobox");
  trigger.setAttribute("aria-label", "Dataset Version");
  trigger.setAttribute("aria-haspopup", "listbox");
  trigger.setAttribute("aria-expanded", "false");
  trigger.setAttribute("aria-controls", "component-dataset-picker-listbox");
  const menu = element("div", null, "component-dataset-picker__menu");
  menu.hidden = true;
  const search = document.createElement("input");
  search.type = "search";
  search.placeholder = "Filter datasets, versions, tags, or provenance";
  search.setAttribute("aria-label", "Filter dataset versions");
  search.className = "component-dataset-picker__search";
  const tableWrap = element("div", null, "table-wrap");
  const table = element("table", null, "component-table component-dataset-picker__table");
  const head = document.createElement("thead");
  const header = document.createElement("tr");
  for (const label of ["Dataset", "Version", "Grain", "Tags", "Provenance"]) {
    header.append(element("th", label));
  }
  head.append(header);
  const body = document.createElement("tbody");
  body.id = "component-dataset-picker-listbox";
  body.setAttribute("role", "listbox");
  body.setAttribute("aria-label", "Dataset versions");
  table.append(head, body);
  tableWrap.append(table);
  const empty = element("p", "No dataset versions match this filter.", "component-dataset-picker__empty");
  empty.hidden = true;
  menu.append(search, tableWrap, empty);
  const preview = element("div", null, "component-dataset-picker__field-preview");
  preview.dataset.componentDatasetFieldPreview = "";
  shell.append(trigger, menu, preview);
  mount.replaceChildren(shell);
  form.querySelector("[data-component-dataset-native-label]")?.classList.add("is-enhanced");
  picker.setAttribute("aria-hidden", "true");
  picker.tabIndex = -1;

  let activeIndex = 0;
  const close = () => {
    menu.hidden = true;
    trigger.setAttribute("aria-expanded", "false");
  };
  const currentDataset = () => (form.__componentDatasets || []).find(
    (dataset) => datasetReferenceKey(dataset.reference) === picker.value,
  );
  const updateSelection = () => {
    const dataset = currentDataset();
    trigger.textContent = datasetPickerLabel(dataset);
    if (!dataset && picker.value) trigger.textContent = "Current Dataset major line";
    renderDatasetFieldPreview(preview, dataset);
  };
  const renderRows = () => {
    const query = search.value.trim().toLowerCase();
    const datasets = (form.__componentDatasets || []).filter(
      (dataset) => !query || datasetPickerSearchText(dataset).includes(query),
    );
    body.replaceChildren();
    for (const [index, dataset] of datasets.entries()) {
      const row = document.createElement("tr");
      row.setAttribute("role", "option");
      row.setAttribute(
        "aria-selected",
        String(datasetReferenceKey(dataset.reference) === picker.value),
      );
      const nameCell = document.createElement("td");
      const select = element("button", dataset.dataset_name, "component-dataset-picker__option");
      select.type = "button";
      select.dataset.datasetPickerOption = String(index);
      select.addEventListener("click", () => {
        picker.value = datasetReferenceKey(dataset.reference);
        picker.dispatchEvent(new Event("change", { bubbles: true }));
        updateSelection();
        close();
        trigger.focus();
      }, { signal });
      nameCell.append(select);
      row.append(
        nameCell,
        element("td", `v${datasetMajor(dataset)}`),
        element("td", dataset.grain || "—"),
        element("td", dataset.tags?.length ? dataset.tags.join(", ") : "None"),
        element("td", datasetProvenanceLabel(dataset)),
      );
      body.append(row);
    }
    activeIndex = Math.min(activeIndex, Math.max(0, datasets.length - 1));
    empty.hidden = datasets.length > 0;
    tableWrap.hidden = datasets.length === 0;
  };
  const open = () => {
    if (trigger.disabled) return;
    menu.hidden = false;
    trigger.setAttribute("aria-expanded", "true");
    renderRows();
    search.focus();
  };
  trigger.addEventListener("click", () => menu.hidden ? open() : close(), { signal });
  trigger.addEventListener("keydown", (event) => {
    if (["ArrowDown", "Enter", " "].includes(event.key) && menu.hidden) {
      event.preventDefault();
      open();
    }
  }, { signal });
  search.addEventListener("input", renderRows, { signal });
  menu.addEventListener("keydown", (event) => {
    const options = Array.from(body.querySelectorAll("[data-dataset-picker-option]"));
    if (event.key === "Escape") {
      event.preventDefault();
      close();
      trigger.focus();
    } else if (["ArrowDown", "ArrowUp"].includes(event.key) && options.length) {
      event.preventDefault();
      activeIndex = (activeIndex + (event.key === "ArrowDown" ? 1 : -1) + options.length) % options.length;
      options[activeIndex].focus();
    }
  }, { signal });
  form.addEventListener("click", (event) => {
    if (!shell.contains(event.target)) close();
  }, { signal });
  picker.addEventListener("change", updateSelection, { signal });
  form.__refreshDatasetPicker = () => {
    trigger.disabled = form.dataset.datasetUnavailable === "true";
    renderRows();
    updateSelection();
  };
  form.__refreshDatasetPicker();
}

function configControl(form, name) {
  return form.querySelector(`[data-config-control="${name}"]`);
}

function setControlValue(form, name, value) {
  const control = configControl(form, name);
  if (!control || value === undefined || value === null) return;
  if (control.type === "checkbox") control.checked = Boolean(value);
  else {
    const text = String(value);
    if (control instanceof HTMLSelectElement &&
        text && !Array.from(control.options).some((option) => option.value === text)) {
      const fallback = new Option(`${text} (stored field)`, text);
      fallback.dataset.storedField = "";
      control.append(fallback);
    }
    control.value = text;
  }
}

function fieldChoice(field, checked, name) {
  const label = element("label", null, "dataset-projection-builder__option");
  const input = document.createElement("input");
  input.type = "checkbox";
  input.value = field.key;
  input.name = name;
  input.checked = checked;
  label.append(input, document.createTextNode(` ${field.label} (${field.field_type})`));
  return label;
}

function visibleFieldChoice(field, checked, displayLabel) {
  const option = element("div", null, "dataset-projection-builder__option");
  option.dataset.fieldKey = field.key;
  const membership = element("label");
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.value = field.key;
  checkbox.name = "visible_column";
  checkbox.checked = checked;
  membership.append(checkbox, document.createTextNode(` ${field.label} (${field.field_type})`));
  const label = element("label", `Display label for ${field.label}`);
  const input = document.createElement("input");
  input.dataset.columnDisplayLabel = "";
  input.value = displayLabel || "";
  input.placeholder = field.label;
  label.append(input);
  option.append(membership, label);
  return option;
}

function fillFieldControls(form, config, selectAll = false) {
  const fields = datasetForForm(form)?.fields || [];
  for (const select of form.querySelectorAll("[data-field-select]")) {
    const prior = select.value;
    for (const option of Array.from(select.options).slice(1)) option.remove();
    for (const field of fields) {
      const option = document.createElement("option");
      option.value = field.key;
      option.textContent = `${field.label} (${field.field_type})`;
      select.append(option);
    }
    if (fields.some((field) => field.key === prior)) select.value = prior;
  }
  const visible = new Set((config.visible_columns || []).filter((item) =>
    typeof item === "string"));
  const searchable = new Set(config.search_fields || []);
  const visibleHost = form.querySelector("[data-component-visible-fields]");
  const searchHost = form.querySelector("[data-component-search-fields]");
  const known = new Set(fields.map((field) => field.key));
  const storedVisible = Array.from(visible).filter((key) => !known.has(key)).map((key) => ({
    key, label: `${key} (metadata unavailable)`, field_type: "stored",
  }));
  const storedSearchable = Array.from(searchable).filter((key) => !known.has(key)).map((key) => ({
    key, label: `${key} (metadata unavailable)`, field_type: "stored",
  }));
  const fieldByKey = new Map(fields.map((field) => [field.key, field]));
  const orderedKnown = (form.__visibleColumnOrder || [])
    .map((key) => fieldByKey.get(key))
    .filter(Boolean);
  const orderedKeys = new Set(orderedKnown.map((field) => field.key));
  const orderedFields = [...orderedKnown, ...fields.filter((field) => !orderedKeys.has(field.key))];
  visibleHost.replaceChildren(...[...orderedFields, ...storedVisible].map((field) =>
    visibleFieldChoice(
      field,
      selectAll || visible.has(field.key),
      form.__tableDisplayLabels?.[field.key],
    )));
  searchHost.replaceChildren(...[...fields, ...storedSearchable].map((field) =>
    fieldChoice(field, searchable.has(field.key), "search_field")));
}

function addFilterRow(form, filter = {}) {
  const fields = datasetForForm(form)?.fields || [];
  const row = element("div", null, "component-filter-row");
  row.dataset.componentFilterRow = "";
  const fieldLabel = element("label", "Field");
  const field = document.createElement("select");
  field.dataset.filterField = "";
  field.append(new Option("Select a field", ""));
  for (const item of fields) {
    const option = new Option(item.label, item.key);
    option.dataset.fieldType = item.field_type;
    field.append(option);
  }
  const storedField = filter.field_key || "";
  if (storedField && !fields.some((item) => item.key === storedField)) {
    field.append(new Option(`${storedField} (stored field)`, storedField));
  }
  field.value = storedField;
  fieldLabel.append(field);
  const operatorLabel = element("label", "Operator");
  const operator = document.createElement("select");
  operator.dataset.filterOperator = "";
  operatorLabel.append(operator);
  const valueLabel = element("label", "Value");
  const value = document.createElement("input");
  value.dataset.filterValue = "";
  value.value = filter.value ?? "";
  valueLabel.append(value);
  const remove = element("button", "Remove filter", "button button--secondary");
  remove.type = "button";
  remove.dataset.componentRemoveFilter = "";
  row.append(fieldLabel, operatorLabel, valueLabel, remove);
  form.querySelector("[data-component-filter-rows]").append(row);
  updateFilterOperators(row, filter.operator);
}

function filterOperatorChoices(fieldType) {
  const nullChoices = [
    ["is_null", "Is null"], ["is_not_null", "Is not null"],
  ];
  if (["number", "integer", "decimal", "date", "datetime"].includes(fieldType)) {
    return [
      ["equals", "Equals"], ["not_equals", "Does not equal"],
      ["lt", "Less than"], ["lte", "Less than or equal"],
      ["gt", "Greater than"], ["gte", "Greater than or equal"],
      ["between", "Between"], ["not_between", "Not between"],
      ...nullChoices,
    ];
  }
  if (["text", "string"].includes(fieldType)) {
    return [
      ["equals", "Equals"], ["not_equals", "Does not equal"],
      ["contains", "Contains"], ["not_contains", "Does not contain"],
      ["starts_with", "Starts with"], ["ends_with", "Ends with"],
      ["is_empty", "Is empty"], ["is_not_empty", "Is not empty"],
      ...nullChoices,
    ];
  }
  return [["equals", "Equals"], ["not_equals", "Does not equal"], ...nullChoices];
}

function updateFilterOperators(row, requested) {
  const field = row.querySelector("[data-filter-field]");
  const operator = row.querySelector("[data-filter-operator]");
  const type = field.selectedOptions[0]?.dataset.fieldType || "text";
  const normalized = requested === "eq" ? "equals" : requested === "not_eq" ? "not_equals" : requested;
  const choices = filterOperatorChoices(type);
  operator.replaceChildren(...choices.map(([value, label]) => new Option(label, value)));
  operator.value = choices.some(([value]) => value === normalized) ? normalized : choices[0][0];
  const withoutValue = ["is_empty", "is_not_empty", "is_null", "is_not_null"].includes(operator.value);
  const value = row.querySelector("[data-filter-value]");
  value.disabled = withoutValue;
  value.closest("label").hidden = withoutValue;
}

function readFilters(form) {
  return Array.from(form.querySelectorAll("[data-component-filter-row]"))
    .map((row) => {
      const operator = row.querySelector("[data-filter-operator]").value;
      const raw = row.querySelector("[data-filter-value]").value;
      const withoutValue = ["is_empty", "is_not_empty", "is_null", "is_not_null"].includes(operator);
      return {
        field_key: row.querySelector("[data-filter-field]").value,
        operator,
        ...(withoutValue ? {} : {
          value: ["between", "not_between"].includes(operator)
            ? raw.split(",").map((value) => value.trim()).filter(Boolean)
            : raw,
        }),
      };
    })
    .filter((filter) => filter.field_key);
}

function numericValue(control, fallback) {
  const value = Number(control?.value);
  return Number.isFinite(value) && value > 0 ? value : fallback;
}

function buildConfig(form) {
  const kind = form.querySelector("[data-component-kind-value]").value;
  const filters = readFilters(form);
  if (kind === "table") {
    const sortField = configControl(form, "sort_field").value;
    const visibleColumns = selectedValues(form, 'input[name="visible_column"]');
    const displayLabels = {};
    for (const option of form.querySelectorAll("[data-component-visible-fields] [data-field-key]")) {
      if (!visibleColumns.includes(option.dataset.fieldKey)) continue;
      const label = option.querySelector("[data-column-display-label]").value.trim();
      if (label) displayLabels[option.dataset.fieldKey] = label;
    }
    return {
      visible_columns: visibleColumns,
      filters,
      search_fields: selectedValues(form, 'input[name="search_field"]'),
      default_sort: sortField ? {
        field_key: sortField,
        direction: configControl(form, "sort_direction").value,
      } : null,
      page_size: numericValue(configControl(form, "page_size"), 25),
      display_labels: displayLabels,
    };
  }
  const summaryType = configControl(form, "summary_type").value;
  const shared = {
    summary_field: configControl(form, "summary_field").value,
    summary_type: summaryType,
    value_format: configControl(form, "value_format").value,
    missing_policy: configControl(form, "value_missing_policy").value,
    value_missing_policy: configControl(form, "value_missing_policy").value,
    sort_direction: configControl(form, "visual_sort_direction").value,
    filters,
  };
  const sortField = configControl(form, "visual_sort_field").value;
  if (kind !== "stat_card" && sortField) shared.sort_field = sortField;
  if (kind === "bar") {
    const comparison = configControl(form, "split_bars").checked
      ? configControl(form, "comparison_field").value
      : "";
    const orientation = configControl(form, "orientation").value;
    const categoryAxisTitle = configControl(form, "x_axis_label").value || null;
    const valueAxisTitle = configControl(form, "y_axis_label").value || null;
    const overrides = categoryOverrides(form);
    return {
      ...shared,
      mode: comparison ? "comparison" : "summary",
      category_field: configControl(form, "category_field").value,
      category_missing_policy: configControl(form, "category_missing_policy").value,
      ...(comparison ? {
        comparison_field: comparison,
        comparison_missing_policy: configControl(form, "comparison_missing_policy").value,
      } : {}),
      orientation,
      comparison_layout: configControl(form, "comparison_layout").value,
      number_of_points: numericValue(configControl(form, "number_of_points"), 20),
      category_labels: overrides.labels,
      category_colors: overrides.colors,
      legend_title: configControl(form, "legend_title").value || null,
      x_axis_label: orientation === "horizontal" ? valueAxisTitle : categoryAxisTitle,
      y_axis_label: orientation === "horizontal" ? categoryAxisTitle : valueAxisTitle,
    };
  }
  if (kind === "line") {
    return {
      ...shared,
      x_field: configControl(form, "x_field").value,
      x_missing_policy: configControl(form, "x_missing_policy").value,
      smoothing: configControl(form, "smoothing").checked,
      number_of_points: numericValue(configControl(form, "line_number_of_points"), 20),
      x_axis_label: configControl(form, "line_x_axis_label").value || null,
      y_axis_label: configControl(form, "line_y_axis_label").value || null,
    };
  }
  if (kind === "pie" || kind === "donut") {
    const overrides = categoryOverrides(form);
    return {
      ...shared,
      category_field: configControl(form, "pie_category_field").value,
      category_missing_policy: configControl(form, "pie_category_missing_policy").value,
      max_slices: numericValue(configControl(form, "max_slices"), 20),
      category_labels: overrides.labels,
      category_colors: overrides.colors,
      legend_title: configControl(form, "legend_title").value || null,
    };
  }
  return {
    ...shared,
    label: configControl(form, "stat_label").value || null,
    supporting_text: configControl(form, "supporting_text").value || null,
    panel_style: configControl(form, "panel_style").value,
  };
}

function categoryOverrides(form) {
  const labels = { ...(form.__storedCategoryLabels || {}) };
  const colors = { ...(form.__storedCategoryColors || {}) };
  for (const row of form.querySelectorAll("[data-component-category-label-row]")) {
    const raw = row.dataset.categoryValue;
    const label = row.querySelector("[data-category-display-label]").value.trim();
    const color = row.querySelector("[data-category-color]").value;
    if (label) labels[raw] = label;
    else delete labels[raw];
    if (color) colors[raw] = color;
    else delete colors[raw];
  }
  return { labels, colors };
}

function renderCategoryOverrides(form, rawValues = []) {
  const body = form.querySelector("[data-component-category-labels]");
  if (!body) return;
  const keys = new Set([
    ...Object.keys(form.__storedCategoryLabels || {}),
    ...Object.keys(form.__storedCategoryColors || {}),
    ...rawValues.map(String),
  ]);
  body.replaceChildren();
  if (!keys.size) {
    const row = document.createElement("tr");
    const cell = element("td", "Choose a category field to load values.");
    cell.colSpan = 3;
    row.append(cell);
    body.append(row);
    return;
  }
  for (const raw of keys) {
    const row = document.createElement("tr");
    row.dataset.componentCategoryLabelRow = "";
    row.dataset.categoryValue = raw;
    const valueCell = element("th", raw);
    valueCell.scope = "row";
    const labelCell = document.createElement("td");
    const label = document.createElement("input");
    label.dataset.categoryDisplayLabel = "";
    label.value = form.__storedCategoryLabels?.[raw] || "";
    label.placeholder = raw;
    label.setAttribute("aria-label", `Display label for ${raw}`);
    labelCell.append(label);
    const colorCell = document.createElement("td");
    const color = document.createElement("select");
    color.dataset.categoryColor = "";
    color.append(...CATEGORY_COLOR_OPTIONS.map(([value, text]) => new Option(text, value)));
    const storedColor = form.__storedCategoryColors?.[raw] || "";
    if (storedColor && !CATEGORY_COLOR_OPTIONS.some(([value]) => value === storedColor)) {
      color.append(new Option(`Stored (${storedColor})`, storedColor));
    }
    color.value = storedColor;
    color.setAttribute("aria-label", `Color for ${raw}`);
    colorCell.append(color);
    row.append(valueCell, labelCell, colorCell);
    body.append(row);
  }
}

function updateOverrideSurface(form, comparison) {
  const table = form.querySelector(".component-category-labels");
  const section = table?.closest("fieldset");
  if (!table || !section) return;
  const noun = comparison ? "Series" : "Category";
  table.setAttribute("aria-label", `${noun} Labels`);
  if (table.caption) table.caption.textContent = `${noun} Labels`;
  const legend = section.querySelector("legend");
  if (legend) legend.textContent = `${noun} display`;
}

function prepareCategoryOverrides(form, config) {
  form.__storedCategoryLabels = { ...(config.category_labels || {}) };
  form.__storedCategoryColors = { ...(config.category_colors || {}) };
  form.__storedLegendTitle = config.legend_title || "";
  form.__activeOverrideField = config.comparison_field || config.category_field || config.x_field || "";
  form.__activeOverrideComparison = Boolean(config.comparison_field);
  const table = form.querySelector(".component-category-labels");
  if (!table) return;
  const header = table.querySelector("thead tr");
  if (header.children.length < 3) header.append(element("th", "Color"));
  const section = document.createElement("fieldset");
  section.dataset.componentConfigSection = "category_labels";
  const legendTitle = configControl(form, "legend_title")?.closest("label");
  section.append(element("legend", "Category display"));
  if (legendTitle) section.append(legendTitle);
  section.append(table);
  form.querySelector('[data-component-config-section="visual"]').append(section);
  updateOverrideSurface(form, form.__activeOverrideComparison);
  renderCategoryOverrides(form);
}

function activeOverrideControl(form) {
  const kind = form.querySelector("[data-component-kind-value]").value;
  if (kind === "bar") {
    const comparison = configControl(form, "comparison_field");
    if (configControl(form, "split_bars").checked && comparison.value) return comparison;
    return configControl(form, "category_field");
  }
  if (["pie", "donut"].includes(kind)) return configControl(form, "pie_category_field");
  return null;
}

function refreshActiveOverrides(form) {
  const control = activeOverrideControl(form);
  if (control) loadDistinctValues(form, control);
}

function versionInput(form) {
  const rawReference = form.querySelector("[data-component-dataset-picker]").value;
  if (!rawReference) throw new Error("Choose a Dataset major line");
  return {
    dataset_reference: JSON.parse(rawReference),
    component_type: form.querySelector("[data-component-kind-value]").value,
    config: buildConfig(form),
    version_note: String(new FormData(form).get("version_note") || ""),
  };
}

function setPreviewState(form, valid, message) {
  const badge = form.querySelector("[data-component-preview-badge]");
  badge.textContent = valid ? "Valid config" : "Needs attention";
  badge.classList.toggle("is-valid", valid);
  form.querySelector("[data-component-preview-findings]").textContent = message;
}

function previewTargets(form) {
  return [
    form.querySelector("[data-component-preview-content]"),
    form.querySelector("[data-component-mobile-preview-content]"),
  ];
}

async function refreshPreview(form) {
  const sequence = (form.__previewSequence || 0) + 1;
  form.__previewSequence = sequence;
  if (form.dataset.datasetUnavailable === "true") {
    setPreviewState(form, false, "Dataset metadata is unavailable. Retry to restore preview.");
    return;
  }
  let input;
  try {
    input = versionInput(form);
  } catch (error) {
    setPreviewState(form, false, error.message);
    return;
  }
  setPreviewState(form, false, "Validating configuration…");
  try {
    const validation = await request("/api/admin/components/validate", "POST", input);
    if (sequence !== form.__previewSequence) return;
    if (!validation.valid) {
      const message = (validation.findings || [])
        .map((finding) => finding.message || finding.code)
        .join("; ") || "Complete the required configuration.";
      setPreviewState(form, false, message);
      for (const target of previewTargets(form)) target.replaceChildren();
      return;
    }
    const execution = await request("/api/admin/components/preview", "POST", input);
    if (sequence !== form.__previewSequence) return;
    for (const target of previewTargets(form)) renderExecution(target, execution);
    setPreviewState(form, true, "Preview uses the current unsaved definition.");
  } catch (error) {
    if (sequence !== form.__previewSequence) return;
    if (isDatasetUnavailable(error)) showDatasetOutage(form, error);
    setPreviewState(form, false, error.message);
  }
}

function showDatasetOutage(form, error) {
  form.dataset.datasetUnavailable = "true";
  const outage = form.closest("[data-component-editor-root]").querySelector("[data-component-dataset-outage]");
  outage.hidden = false;
  outage.querySelector("p").textContent =
    `Dataset metadata is temporarily unavailable. Your unsaved Component changes are preserved. ${error.message}`;
  for (const button of form.querySelectorAll(
    "[data-component-save-action], [data-component-open-consumer-review]",
  )) button.disabled = true;
  form.__refreshDatasetPicker?.();
}

function isDatasetUnavailable(error) {
  return error.code === "component.dependency_unavailable" ||
    error.code === "dataset.unavailable";
}

function schedulePreview(form) {
  clearTimeout(form.__previewTimer);
  form.__previewTimer = setTimeout(() => refreshPreview(form), 250);
}

function updateKindSurface(form, kind) {
  form.querySelector("[data-component-kind-value]").value = kind;
  for (const button of form.querySelectorAll("[data-component-kind]")) {
    const selected = button.dataset.componentKind === kind;
    button.classList.toggle("is-selected", selected);
    button.setAttribute("aria-checked", String(selected));
  }
  form.querySelector('[data-component-config-section="table"]').hidden = kind !== "table";
  form.querySelector('[data-component-config-section="visual"]').hidden = kind === "table";
  for (const section of form.querySelectorAll('[data-component-config-section="bar"], [data-component-config-section="line"], [data-component-config-section="pie"], [data-component-config-section="stat_card"], [data-component-config-section="category_labels"]')) {
    const expected = section.dataset.componentConfigSection;
    section.hidden = expected === "pie"
      ? !["pie", "donut"].includes(kind)
      : expected === "category_labels"
        ? !["bar", "pie", "donut"].includes(kind)
        : expected !== kind;
  }
  const descriptions = {
    table: "Show Dataset records with configurable columns, search, sorting, and pagination.",
    bar: "Compare calculated values across categories and optional series.",
    line: "Plot a calculated value across an ordered Dataset field.",
    pie: "Compare category shares in a circular chart.",
    donut: "A donut chart is a pie chart with a hole in the center.",
    stat_card: "Highlight one calculated Dataset value with supporting context.",
  };
  form.querySelector("[data-component-kind-description]").textContent = descriptions[kind];
  configControl(form, "comparison_field").closest("label").hidden = !configControl(form, "split_bars").checked;
  const sort = configControl(form, "visual_sort_field");
  const priorSort = sort.value;
  const sortChoices = kind === "line"
    ? [["", "Default"], ["x", "Category"], ["summary_value", "Summary Value"]]
    : kind === "bar" && configControl(form, "split_bars").checked
      ? [["", "Default"], ["category", "Category"], ["comparison", "Comparison"], ["summary_value", "Summary Value"]]
      : [["", "Default"], ["category", "Category"], ["summary_value", "Summary Value"]];
  sort.replaceChildren(...sortChoices.map(([value, label]) => new Option(label, value)));
  sort.value = sortChoices.some(([value]) => value === priorSort) ? priorSort : sortChoices[0][0];
  const sortHelp = form.querySelector("[data-component-sort-field-help]");
  if (sortHelp) {
    const help = {
      "": "Default: uses the order produced by the current grouping and summarization.",
      category: "Category: sorts by the displayed category label.",
      x: "Category: sorts by the displayed horizontal category value.",
      comparison: "Comparison: sorts by the displayed comparison group label.",
      summary_value: "Summary Value: sorts by the summarized numeric value.",
    };
    sortHelp.textContent = sortChoices.map(([value]) => help[value]).join("\n");
  }
  configControl(form, "category_field").setAttribute("aria-label", "Category field");
  const summary = configControl(form, "summary_type");
  if (!Array.from(summary.options).some((option) => option.value === "none")) {
    summary.append(new Option("Do not summarize", "none"));
  }
  const format = configControl(form, "value_format");
  if (!Array.from(format.options).some((option) => option.value === "decimal")) {
    format.insertBefore(new Option("Decimal", "decimal"), format.querySelector('option[value="percent"]'));
  }
  for (const limit of [
    configControl(form, "number_of_points"),
    configControl(form, "line_number_of_points"),
    configControl(form, "max_slices"),
  ]) limit.max = "100";
  updateCalculationDependencies(form);
}

function updateCalculationDependencies(form) {
  const summaryType = configControl(form, "summary_type").value;
  const summaryField = configControl(form, "summary_field");
  const rowCount = summaryType === "row_count";
  summaryField.disabled = rowCount;
  summaryField.closest("[data-component-value-field]").hidden = rowCount;
  configControl(form, "value_missing_policy")
    .closest("[data-component-value-missing-policy]").hidden = rowCount;
  form.querySelector("[data-component-calculation-warning]").hidden = summaryType !== "none";
  const stacked = configControl(form, "comparison_layout")
    .querySelector('option[value="stacked"]');
  stacked.disabled = !["row_count", "count", "sum"].includes(summaryType);
  if (stacked.disabled && configControl(form, "comparison_layout").value === "stacked") {
    configControl(form, "comparison_layout").value = "grouped";
  }
}

function applyStoredConfig(form, config) {
  setControlValue(form, "page_size", config.page_size ?? 25);
  setControlValue(form, "sort_field", config.default_sort?.field_key ?? "");
  setControlValue(form, "sort_direction", config.default_sort?.direction ?? "asc");
  for (const name of ["summary_type", "summary_field", "value_format", "value_missing_policy",
    "category_field", "category_missing_policy", "comparison_field", "comparison_missing_policy",
    "comparison_layout", "orientation", "number_of_points",
    "x_field", "x_missing_policy", "smoothing", "max_slices", "legend_title", "panel_style"] ) {
    setControlValue(form, name, config[name]);
  }
  setControlValue(
    form,
    "value_missing_policy",
    config.value_missing_policy ?? config.missing_policy,
  );
  setControlValue(form, "visual_sort_field", config.sort_field ?? "");
  setControlValue(form, "visual_sort_direction", config.sort_direction ?? "asc");
  const orientation = config.orientation ?? "horizontal";
  setControlValue(
    form,
    "x_axis_label",
    orientation === "horizontal" ? config.y_axis_label : config.x_axis_label,
  );
  setControlValue(
    form,
    "y_axis_label",
    orientation === "horizontal" ? config.x_axis_label : config.y_axis_label,
  );
  setControlValue(form, "line_number_of_points", config.number_of_points ?? 20);
  setControlValue(form, "line_x_axis_label", config.x_axis_label);
  setControlValue(form, "line_y_axis_label", config.y_axis_label);
  setControlValue(form, "pie_category_field", config.category_field);
  setControlValue(form, "pie_category_missing_policy", config.category_missing_policy);
  setControlValue(form, "stat_label", config.label);
  setControlValue(form, "supporting_text", config.supporting_text);
  setControlValue(form, "split_bars", Boolean(config.comparison_field));
  form.querySelector("[data-component-filter-rows]").replaceChildren();
  for (const filter of config.filters || []) addFilterRow(form, filter);
}

function resetKindSpecificConfig(form) {
  form.__visibleColumnOrder = [];
  form.__tableDisplayLabels = {};
  for (const input of form.querySelectorAll(
    '[data-component-visible-fields] input[type="checkbox"], [data-component-search-fields] input[type="checkbox"]',
  )) input.checked = false;
  for (const input of form.querySelectorAll("[data-component-display-label]")) {
    input.value = input.dataset.defaultLabel || "";
  }
  const defaults = {
    page_size: 50,
    sort_field: "",
    sort_direction: "asc",
    summary_type: "count",
    summary_field: "",
    value_format: "plain",
    value_missing_policy: "omit",
    category_field: "",
    category_missing_policy: "omit",
    comparison_field: "",
    comparison_missing_policy: "omit",
    comparison_layout: "grouped",
    orientation: "horizontal",
    x_axis_label: "",
    y_axis_label: "",
    number_of_points: 20,
    x_field: "",
    x_missing_policy: "omit",
    smoothing: true,
    line_number_of_points: 20,
    line_x_axis_label: "",
    line_y_axis_label: "",
    pie_category_field: "",
    pie_category_missing_policy: "omit",
    max_slices: 20,
    legend_title: "",
    stat_label: "",
    supporting_text: "",
    panel_style: "default",
    split_bars: false,
  };
  for (const [name, value] of Object.entries(defaults)) setControlValue(form, name, value);
  form.__storedCategoryLabels = {};
  form.__storedCategoryColors = {};
  form.__storedLegendTitle = "";
  renderCategoryOverrides(form);
  updateCalculationDependencies(form);
}

async function loadDistinctValues(form, control) {
  const fieldKey = control.value;
  const comparison = control.matches('[data-config-control="comparison_field"]');
  const reference = form.querySelector("[data-component-dataset-picker]").value;
  const currentOverrides = categoryOverrides(form);
  form.__storedCategoryLabels = currentOverrides.labels;
  form.__storedCategoryColors = currentOverrides.colors;
  if (fieldKey !== form.__activeOverrideField || comparison !== form.__activeOverrideComparison) {
    form.__storedCategoryLabels = {};
    form.__storedCategoryColors = {};
    form.__activeOverrideField = fieldKey;
    form.__activeOverrideComparison = comparison;
    const selected = datasetForForm(form)?.fields?.find((field) => field.key === fieldKey);
    setControlValue(form, "legend_title", selected?.label || "");
  }
  updateOverrideSurface(form, comparison);
  renderCategoryOverrides(form);
  control.closest("label")?.querySelector("[data-component-distinct-values]")?.remove();
  if (!fieldKey || !reference || form.dataset.datasetUnavailable === "true") return;
  try {
    const response = await request("/api/admin/components/datasets/distinct-values", "POST", {
      schema_version: 1,
      action: "distinct_values",
      reference: JSON.parse(reference),
      field_key: fieldKey,
      limit: 12,
    });
    if (control.value !== fieldKey || activeOverrideControl(form) !== control) return;
    const values = element("p", null, "component-distinct-values");
    values.dataset.componentDistinctValues = "";
    const noun = comparison ? "Series" : "Category";
    values.textContent = response.values?.length
      ? `${noun} values: ${response.values.map(String).join(", ")}`
      : `No ${noun.toLowerCase()} values are currently available.`;
    control.closest("label")?.append(values);
    if (control.matches('[data-config-control="category_field"], [data-config-control="pie_category_field"], [data-config-control="comparison_field"]')) {
      renderCategoryOverrides(form, response.values || []);
    }
  } catch {
    // Distinct values are helpful authoring metadata, not a preview prerequisite.
  }
}

function initializeEditor(root, bootstrap, signal) {
  const form = root.querySelector?.("[data-component-create]");
  if (!form) return;
  form.__componentDatasets = bootstrap?.datasets?.datasets || [];
  initializeDatasetPicker(form, signal);
  let storedConfig = {};
  try {
    storedConfig = JSON.parse(form.querySelector("[data-component-config]").value || "{}");
  } catch {
    storedConfig = {};
  }
  const isCreate = !form.dataset.componentId;
  form.__tableDisplayLabels = { ...(storedConfig.display_labels || {}) };
  form.__visibleColumnOrder = (storedConfig.visible_columns || []).filter((item) =>
    typeof item === "string");
  prepareCategoryOverrides(form, storedConfig);
  fillFieldControls(form, storedConfig, isCreate && !(storedConfig.visible_columns || []).length);
  applyStoredConfig(form, storedConfig);
  const initialKind = form.querySelector("[data-component-kind-value]").value;
  updateKindSurface(form, initialKind);
  form.__committedKind = initialKind;
  let pendingKind = null;
  const nameInput = form.elements.name;
  const slugInput = form.elements.slug;
  slugInput.__autoValue = slugInput.value;
  slugInput.__manuallyEdited = Boolean(slugInput.value);
  slugInput.addEventListener("input", () => {
    slugInput.__manuallyEdited = slugInput.value !== slugInput.__autoValue;
  }, { signal });
  nameInput.addEventListener("blur", () => {
    if (slugInput.__manuallyEdited) return;
    const generated = nameInput.value
      .normalize("NFKD")
      .replace(/[\u0300-\u036f]/g, "")
      .toLowerCase()
      .replace(/[^a-z0-9]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 120);
    slugInput.value = generated;
    slugInput.__autoValue = generated;
  }, { signal });
  root.querySelector?.("[data-component-dataset-retry]")?.addEventListener("click", () => {
    retryDatasets(form);
  }, { signal });

  form.addEventListener("click", (event) => {
    if (event.target.closest("[data-component-open-consumer-review]")) {
      const dialog = form.querySelector("[data-component-consumer-review]");
      const note = dialog.querySelector("[data-component-new-version-note]");
      const error = dialog.querySelector("[data-component-consumer-review-error]");
      note.value = "";
      error.hidden = true;
      if (typeof dialog.showModal === "function") dialog.showModal();
      else dialog.setAttribute("open", "");
      note.focus();
      return;
    }
    if (event.target.closest("[data-component-consumer-review-cancel]")) {
      const dialog = form.querySelector("[data-component-consumer-review]");
      if (typeof dialog.close === "function") dialog.close();
      else dialog.removeAttribute("open");
      form.querySelector("[data-component-open-consumer-review]")?.focus();
      return;
    }
    if (event.target.closest("[data-component-consumer-review-confirm]")) {
      const dialog = form.querySelector("[data-component-consumer-review]");
      const note = dialog.querySelector("[data-component-new-version-note]");
      const error = dialog.querySelector("[data-component-consumer-review-error]");
      if (!note.value.trim()) {
        error.textContent = "New versions require a version note.";
        error.hidden = false;
        note.focus();
        return;
      }
      form.elements.version_note.value = note.value.trim();
      if (typeof dialog.close === "function") dialog.close();
      else dialog.removeAttribute("open");
      submitComponentSave(form, "create_new_version").catch(() => {});
      return;
    }
    const kindButton = event.target.closest("[data-component-kind]");
    if (kindButton) {
      const next = kindButton.dataset.componentKind;
      if (next === form.__committedKind) return;
      const needsConfirmation = Boolean(form.dataset.componentId) ||
        form.dataset.dirty === "true" || form.__kindWasChanged;
      if (needsConfirmation) {
        pendingKind = next;
        const label = kindButton.textContent.trim();
        form.querySelector("[data-component-kind-confirmation-copy]").textContent =
          `Change to ${label}? Kind-specific settings in this draft will be cleared. Dataset binding and filters will be kept.`;
        form.querySelector("[data-component-kind-confirmation]").hidden = false;
        form.querySelector("[data-component-kind-confirm]").textContent = `Change to ${label}`;
      } else {
        form.__committedKind = next;
        form.__kindWasChanged = true;
        resetKindSpecificConfig(form);
        updateKindSurface(form, next);
        form.dataset.dirty = "true";
        schedulePreview(form);
      }
      return;
    }
    if (event.target.closest("[data-component-kind-cancel]")) {
      pendingKind = null;
      form.querySelector("[data-component-kind-confirmation]").hidden = true;
      return;
    }
    if (event.target.closest("[data-component-kind-confirm]")) {
      if (!pendingKind) return;
      form.__committedKind = pendingKind;
      form.__kindWasChanged = true;
      resetKindSpecificConfig(form);
      updateKindSurface(form, pendingKind);
      pendingKind = null;
      form.querySelector("[data-component-kind-confirmation]").hidden = true;
      form.querySelector("[data-component-kind-editor]").focus();
      form.dataset.dirty = "true";
      schedulePreview(form);
      return;
    }
    if (event.target.closest("[data-component-add-filter]")) {
      addFilterRow(form);
      form.dataset.dirty = "true";
      return;
    }
    if (event.target.closest("[data-component-remove-filter]")) {
      event.target.closest("[data-component-filter-row]").remove();
      form.dataset.dirty = "true";
      schedulePreview(form);
      return;
    }
    if (event.target.closest("[data-component-preview-open]")) {
      const dialog = form.querySelector("[data-component-preview-dialog]");
      if (typeof dialog.showModal === "function") dialog.showModal();
      else dialog.setAttribute("open", "");
      dialog.focus();
      return;
    }
    if (event.target.closest("[data-component-preview-close]")) {
      const dialog = form.querySelector("[data-component-preview-dialog]");
      if (typeof dialog.close === "function") dialog.close();
      else dialog.removeAttribute("open");
      form.querySelector("[data-component-preview-open]").focus();
      return;
    }
    if (event.target.closest("[data-component-dataset-retry]")) {
      retryDatasets(form);
    }
  }, { signal });

  form.addEventListener("change", (event) => {
    if (event.target.matches("[data-component-dataset-picker]")) {
      form.__visibleColumnOrder = [];
      form.__tableDisplayLabels = {};
      fillFieldControls(form, {}, true);
      applyStoredConfig(form, {});
    }
    if (event.target.matches('[data-config-control="split_bars"]')) {
      configControl(form, "comparison_field").closest("label").hidden = !event.target.checked;
      updateKindSurface(form, form.querySelector("[data-component-kind-value]").value);
      refreshActiveOverrides(form);
    }
    if (event.target.matches('[data-config-control="summary_type"]')) {
      updateCalculationDependencies(form);
    }
    if (event.target.matches('[data-config-control="category_field"], [data-config-control="pie_category_field"], [data-config-control="comparison_field"]')) {
      if (event.target.matches('[data-config-control="comparison_field"]')) {
        refreshActiveOverrides(form);
      } else if (activeOverrideControl(form) === event.target) {
        loadDistinctValues(form, event.target);
      }
    }
    if (event.target.matches("[data-filter-field]")) {
      updateFilterOperators(event.target.closest("[data-component-filter-row]"));
    }
    if (event.target.matches("[data-filter-operator]")) {
      updateFilterOperators(event.target.closest("[data-component-filter-row]"), event.target.value);
    }
    schedulePreview(form);
  }, { signal });
  form.addEventListener("input", () => schedulePreview(form), { signal });
  form.querySelector("[data-component-preview-dialog]").addEventListener("close", () => {
    form.querySelector("[data-component-preview-open]").focus();
  }, { signal });
  form.querySelector("[data-component-consumer-review]")?.addEventListener("close", () => {
    form.querySelector("[data-component-open-consumer-review]")?.focus();
  }, { signal });
  if (form.dataset.datasetUnavailable !== "true") schedulePreview(form);
  else setPreviewState(form, false, "Dataset metadata is unavailable. Retry to restore preview.");
}

async function retryDatasets(form) {
  const button = form.closest("[data-component-editor-root]").querySelector("[data-component-dataset-retry]");
  button.disabled = true;
  button.textContent = "Retrying…";
  try {
    const catalog = await request("/api/admin/components/datasets");
    const preservedConfig = buildConfig(form);
    form.__componentDatasets = catalog.datasets || [];
    const picker = form.querySelector("[data-component-dataset-picker]");
    const prior = picker.value;
    picker.replaceChildren();
    for (const dataset of form.__componentDatasets) {
      picker.append(new Option(dataset.dataset_name, datasetReferenceKey(dataset.reference)));
    }
    if (Array.from(picker.options).some((option) => option.value === prior)) {
      picker.value = prior;
    } else if (prior) {
      picker.prepend(new Option("Current Dataset major line", prior, true, true));
    }
    form.dataset.datasetUnavailable = "false";
    form.closest("[data-component-editor-root]").querySelector("[data-component-dataset-outage]").hidden = true;
    for (const save of form.querySelectorAll(
      "[data-component-save-action], [data-component-open-consumer-review]",
    )) save.disabled = false;
    form.__tableDisplayLabels = { ...(preservedConfig.display_labels || {}) };
    form.__visibleColumnOrder = preservedConfig.visible_columns || [];
    form.__refreshDatasetPicker?.();
    fillFieldControls(form, preservedConfig, false);
    schedulePreview(form);
  } catch (error) {
    const outage = form.closest("[data-component-editor-root]").querySelector("[data-component-dataset-outage]");
    outage.hidden = false;
    outage.querySelector("p").textContent = `Dataset metadata is still unavailable: ${error.message}`;
  } finally {
    button.disabled = false;
    button.textContent = "Retry Dataset metadata";
  }
}

async function save(form, action) {
  const version = versionInput(form);
  const data = new FormData(form);
  const componentId = form.dataset.componentId || null;
  const payload = {
    schema_version: 1,
    component_id: componentId,
    draft_version_id: componentId && ["save_draft", "create_new_version"].includes(action)
      ? form.dataset.draftId || null
      : null,
    published_version_id: action === "update_existing_version"
      ? form.dataset.publishedId || null
      : null,
    action,
    component: {
      schema_version: 1,
      name: String(data.get("name") || ""),
      slug: String(data.get("slug") || ""),
      description: String(data.get("description") || "") || null,
    },
    version,
  };
  if (action === "create_new_version" && !version.version_note.trim()) {
    throw new Error("Add a version note before publishing a new version");
  }
  status("Saving Component definition and version…", false, form);
  const keyAction = `save-${action}-${componentId || "new"}`;
  const result = await request(
    "/api/admin/components/save",
    "POST",
    payload,
    mutationKey(form, keyAction),
  );
  clearMutationKey(form, keyAction);
  form.dataset.dirty = "false";
  location.assign(`/components/${encodeURIComponent(payload.component.slug)}/versions`);
}

function submitComponentSave(form, action) {
  const buttons = form.querySelectorAll(
    "[data-component-save-action], [data-component-open-consumer-review]",
  );
  for (const button of buttons) button.disabled = true;
  return save(form, action).catch((error) => {
    if (isDatasetUnavailable(error)) showDatasetOutage(form, error);
    status(error.message, true, form);
    if (form.dataset.datasetUnavailable !== "true") {
      for (const button of buttons) button.disabled = false;
    }
    throw error;
  });
}

function confirmVersionAction(panel, button, action) {
  const labels = {
    publish: "Publish Component version?",
    delete: "Discard draft?",
    activate: "Activate Component version?",
    deactivate: "Deactivate Component version?",
    archive: "Archive Component version?",
    tombstone: "Tombstone Component version?",
  };
  const messages = {
    publish: "Publishing this draft will supersede the current published version.",
    delete: "Discarding this draft cannot be undone.",
    activate: "This version will become available for normal execution again.",
    deactivate: "This version will stop serving normal execution until it is reactivated.",
    archive: "Archived versions cannot be reactivated.",
    tombstone: "Tombstoning is terminal and suppresses external metadata and rendering.",
  };
  const dialog = document.createElement("dialog");
  dialog.className = "component-version-confirmation";
  dialog.dataset.componentVersionConfirmation = "";
  dialog.setAttribute("aria-labelledby", "component-version-confirmation-title");
  const form = document.createElement("form");
  form.method = "dialog";
  const title = element("h2", labels[action] || "Confirm Component action");
  title.id = "component-version-confirmation-title";
  const message = element("p", messages[action] || "Confirm this Component version action.");
  if (["delete", "archive", "tombstone"].includes(action)) {
    message.classList.add("component-version-confirmation__warning");
  }
  const actions = element("div", null, "component-actions");
  const cancel = element("button", "Cancel", "button button--secondary");
  cancel.value = "cancel";
  const confirm = element(
    "button",
    action === "delete" ? "Discard draft" : `${action.charAt(0).toUpperCase()}${action.slice(1)}`,
    "button",
  );
  confirm.value = "confirm";
  actions.append(cancel, confirm);
  form.append(title, message, actions);
  dialog.append(form);
  panel.append(dialog);
  return new Promise((resolve) => {
    dialog.addEventListener("close", () => {
      const accepted = dialog.returnValue === "confirm";
      dialog.remove();
      button.focus();
      resolve(accepted);
    }, { once: true });
    if (typeof dialog.showModal === "function") dialog.showModal();
    else dialog.setAttribute("open", "");
    cancel.focus();
  });
}

function installAuthoringHandler(root, signal) {
  const markDirty = (event) => {
    const form = event.target.closest?.("[data-component-create]");
    if (!form) return;
    form.dataset.dirty = "true";
    pendingMutationKeys.delete(form);
  };
  root.addEventListener("input", markDirty, { signal });
  root.addEventListener("change", markDirty, { signal });
  const documentNode = root.nodeType === 9 ? root : root.ownerDocument;
  documentNode?.defaultView?.addEventListener("beforeunload", (event) => {
    if (!hasUnsavedChanges(root)) return;
    event.preventDefault();
    event.returnValue = "";
  }, { signal });
  root.addEventListener("submit", (event) => {
    const form = event.target.closest("[data-component-create]");
    if (!form) return;
    event.preventDefault();
    const action = event.submitter?.dataset.componentSaveAction || "save_draft";
    submitComponentSave(form, action).catch(() => {});
  }, { signal });
  root.addEventListener("click", async (event) => {
    const button = event.target.closest("[data-component-version-action]");
    if (!button) return;
    const panel = button.closest("[data-component-versions]");
    const componentId = panel?.dataset.componentId;
    const versionId = button.dataset.versionId;
    const action = button.dataset.componentVersionAction;
    if (!componentId || !versionId || !action) return;
    const confirmed = await confirmVersionAction(panel, button, action);
    if (!confirmed) return;
    button.disabled = true;
    status(`${action[0].toUpperCase()}${action.slice(1)} in progress…`, false, panel);
    let operation;
    if (action === "publish") {
      operation = request(`/api/admin/components/${componentId}/versions/${versionId}/publish`, "POST", undefined, mutationKey(button, action));
    } else if (action === "delete") {
      operation = request(`/api/admin/components/${componentId}/versions/${versionId}`, "DELETE", undefined, mutationKey(button, action));
    } else {
      operation = request(`/api/admin/components/${componentId}/versions/${versionId}/lifecycle`, "POST", {
        schema_version: 1,
        action,
        expected_resource_revision: Number(button.dataset.resourceRevision),
      }, mutationKey(button, action));
    }
    operation.then(() => {
      clearMutationKey(button, action);
      location.reload();
    }).catch((error) => {
      status(error.message, true, panel);
      button.disabled = false;
    });
  }, { signal });
}

function hasUnsavedChanges(root) {
  return Boolean(root?.querySelector?.('[data-component-create][data-dirty="true"]'));
}

function svgElement(name, attributes = {}) {
  const node = document.createElementNS(SVG_NS, name);
  for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, String(value));
  return node;
}

function chartTooltip(target) {
  const tooltip = element("div", null, "component-d3-tooltip");
  tooltip.setAttribute("role", "tooltip");
  tooltip.hidden = true;
  target.append(tooltip);
  return tooltip;
}

function interactiveMark(mark, label, tooltip) {
  mark.setAttribute("tabindex", "0");
  mark.setAttribute("aria-label", label);
  const show = () => {
    tooltip.textContent = label;
    tooltip.hidden = false;
  };
  const hide = () => { tooltip.hidden = true; };
  mark.addEventListener("pointerenter", show);
  mark.addEventListener("focus", show);
  mark.addEventListener("pointerleave", hide);
  mark.addEventListener("blur", hide);
}

function stableChartColor(label, fallbackIndex = 0) {
  if (!label) return CHART_COLORS[fallbackIndex % CHART_COLORS.length];
  let hash = 0;
  for (const character of String(label)) hash = ((hash << 5) - hash + character.charCodeAt(0)) | 0;
  return CHART_COLORS[Math.abs(hash) % CHART_COLORS.length];
}

function tableFilterOperators(fieldType) {
  const common = [
    ["equals", "Equals"],
    ["not_equals", "Does not equal"],
  ];
  const type = String(fieldType || "").toLowerCase();
  if (["integer", "number", "decimal", "float", "date", "datetime", "timestamp"].includes(type)) {
    return [
      ...common,
      ["greater_than", type.includes("date") || type.includes("time") ? "After" : "Greater than"],
      ["greater_than_or_equal", type.includes("date") || type.includes("time") ? "On or after" : "At least"],
      ["less_than", type.includes("date") || type.includes("time") ? "Before" : "Less than"],
      ["less_than_or_equal", type.includes("date") || type.includes("time") ? "On or before" : "At most"],
      ["between", "Between"],
      ["is_null", "Has no value"],
      ["is_not_null", "Has a value"],
    ];
  }
  if (["boolean", "bool"].includes(type)) {
    return [...common, ["is_null", "Has no value"], ["is_not_null", "Has a value"]];
  }
  return [
    ["contains", "Contains"],
    ["not_contains", "Does not contain"],
    ["starts_with", "Starts with"],
    ["ends_with", "Ends with"],
    ...common,
    ["is_empty", "Is empty"],
    ["is_not_empty", "Is not empty"],
    ["is_null", "Has no value"],
    ["is_not_null", "Has a value"],
  ];
}

function tableFilterNeedsValue(operator) {
  return !["is_empty", "is_not_empty", "is_null", "is_not_null"].includes(operator);
}

function renderTable(target, execution, loadPage) {
  const columns = execution.columns || [];
  const initialRows = [...(execution.rows || [])];
  let rows = [...initialRows];
  let nextCursor = execution.pagination?.next_cursor || null;
  let sort = null;
  const filters = new Map();
  let previousPages = [];
  const viewer = element("section", null, "component-table-viewer");
  const controls = element("div", null, "component-table-viewer__controls");
  const searchLabel = element("label", "Search component rows");
  const search = document.createElement("input");
  search.type = "search";
  searchLabel.append(search);
  const columnsButton = element("button", "Choose visible columns", "button button--secondary");
  columnsButton.type = "button";
  columnsButton.setAttribute("aria-expanded", "false");
  const columnMenu = element("fieldset", null, "component-table-viewer__columns");
  columnMenu.hidden = true;
  columnMenu.append(element("legend", "Visible columns"));
  const visible = new Set(columns.map((column) => column.key));
  for (const column of columns) {
    const label = element("label");
    const input = document.createElement("input");
    input.type = "checkbox";
    input.value = column.key;
    input.checked = true;
    label.append(input, document.createTextNode(` ${column.label || column.key}`));
    columnMenu.append(label);
  }
  const reset = element("button", "Reset table controls", "button button--secondary");
  reset.type = "button";
  const pageSizeLabel = element("label", "Rows per page");
  const pageSize = document.createElement("select");
  const returnedPageSize = Number(execution.pagination?.page_size) || 25;
  const sizes = Array.from(new Set([10, 25, 50, 100, 200, returnedPageSize])).sort((a, b) => a - b);
  for (const size of sizes) pageSize.append(new Option(String(size), String(size)));
  pageSize.value = String(returnedPageSize);
  pageSizeLabel.append(pageSize);
  controls.append(searchLabel, reset, columnsButton, pageSizeLabel);
  const wrap = element("div", null, "table-wrap");
  const table = element("table", null, "component-table component-table-viewer__table");
  const head = document.createElement("thead");
  const body = document.createElement("tbody");
  table.append(head, body);
  wrap.append(table);
  const page = element("p", `${rows.length} rows shown`, "component-help");
  const previousPage = element("button", "Previous page", "button button--secondary");
  previousPage.type = "button";
  previousPage.setAttribute("aria-label", "Previous page");
  previousPage.disabled = true;
  const nextPage = element("button", "Next page", "button button--secondary");
  nextPage.type = "button";
  nextPage.setAttribute("aria-label", "Next page");
  nextPage.disabled = !loadPage || !execution.pagination?.has_more;
  const paging = element("div", null, "component-table-viewer__paging");
  paging.append(previousPage, nextPage);
  viewer.append(controls, columnMenu, wrap, page, paging);
  target.append(viewer);
  const draw = () => {
    const query = search.value.trim().toLowerCase();
    head.replaceChildren();
    body.replaceChildren();
    const heading = document.createElement("tr");
    for (const column of columns.filter((column) => visible.has(column.key))) {
      const cell = document.createElement("th");
      const headerControls = element("div", null, "component-table-viewer__header-controls");
      const sortButton = element("button", column.label || column.key, "component-table-viewer__sort");
      sortButton.type = "button";
      sortButton.dataset.sortField = column.key;
      sortButton.setAttribute("aria-label", `Sort by ${column.label || column.key}`);
      if (sort?.field === column.key) sortButton.dataset.direction = sort.direction;
      const filterButton = element("button", "Filter", "component-table-viewer__filter-trigger");
      filterButton.type = "button";
      filterButton.dataset.tableFilterToggle = column.key;
      filterButton.setAttribute("aria-label", `Filter ${column.label || column.key}`);
      filterButton.setAttribute("aria-haspopup", "dialog");
      filterButton.setAttribute("aria-expanded", "false");
      filterButton.classList.toggle("is-filtered", filters.has(column.key));
      const menu = element("div", null, "component-table-viewer__filter-menu");
      menu.dataset.tableFilterMenu = column.key;
      menu.setAttribute("role", "dialog");
      menu.setAttribute("aria-label", `Filter ${column.label || column.key}`);
      menu.hidden = true;
      const operatorLabel = element("label", "Operator");
      const operator = document.createElement("select");
      operator.dataset.tableFilterOperator = column.key;
      const active = filters.get(column.key);
      for (const [value, label] of tableFilterOperators(column.field_type)) {
        operator.append(new Option(label, value));
      }
      if (active) operator.value = active.operator;
      operatorLabel.append(operator);
      const valueLabel = element("label", "Value");
      const value = document.createElement("input");
      value.type = ["integer", "number", "decimal", "float"].includes(column.field_type)
        ? "text"
        : ["date", "datetime", "timestamp"].includes(column.field_type) ? "text" : "search";
      value.placeholder = operator.value === "between" ? "Start..end" : "Filter value";
      value.value = active?.value || "";
      value.dataset.tableFilterValue = column.key;
      valueLabel.hidden = !tableFilterNeedsValue(operator.value);
      valueLabel.append(value);
      const menuActions = element("div", null, "component-actions");
      const clearFilter = element("button", "Clear", "button button--secondary");
      clearFilter.type = "button";
      clearFilter.dataset.tableFilterClear = column.key;
      const applyFilter = element("button", "Apply filter", "button");
      applyFilter.type = "button";
      applyFilter.dataset.tableFilterApply = column.key;
      menuActions.append(clearFilter, applyFilter);
      menu.append(operatorLabel, valueLabel, menuActions);
      headerControls.append(sortButton, filterButton, menu);
      cell.append(headerControls);
      heading.append(cell);
    }
    head.append(heading);
    let count = 0;
    for (const row of rows) {
      const matches = !query || Object.values(row.values || {}).some((value) =>
        String(value ?? "").toLowerCase().includes(query));
      if (!matches) continue;
      count += 1;
      const rendered = document.createElement("tr");
      for (const column of columns.filter((column) => visible.has(column.key))) {
        rendered.append(element("td", row.values?.[column.key] ?? "—"));
      }
      body.append(rendered);
    }
    page.textContent = `${count} rows shown`;
  };
  const parameters = (cursor) => {
    const query = new URLSearchParams();
    if (search.value.trim()) query.set("search", search.value.trim());
    const selected = Array.from(visible);
    if (selected.length && selected.length < columns.length) {
      query.set("visible_columns", selected.join(","));
    }
    if (sort) query.set("sort", `${sort.field}:${sort.direction}`);
    for (const [field, filter] of filters) {
      query.set(`filter[${field}][operator]`, filter.operator);
      if (tableFilterNeedsValue(filter.operator)) {
        query.set(`filter[${field}][value]`, filter.value);
      }
    }
    query.set("page_size", pageSize.value);
    if (cursor) query.set("cursor", cursor);
    return query.toString();
  };
  const fetchPage = async (moveNext) => {
    if (!loadPage) return;
    nextPage.disabled = true;
    try {
      const result = await loadPage(parameters(moveNext ? nextCursor : null));
      if (moveNext) previousPages.push({ rows, nextCursor });
      else previousPages = [];
      rows = [...(result.rows || [])];
      nextCursor = result.pagination?.next_cursor || null;
      nextPage.disabled = !result.pagination?.has_more;
      previousPage.disabled = !previousPages.length;
      draw();
    } finally {
      if (nextCursor) nextPage.disabled = false;
    }
  };
  columnsButton.addEventListener("click", () => {
    columnMenu.hidden = !columnMenu.hidden;
    columnsButton.setAttribute("aria-expanded", String(!columnMenu.hidden));
  });
  columnMenu.addEventListener("change", (event) => {
    if (event.target.checked) visible.add(event.target.value);
    else visible.delete(event.target.value);
    draw();
    fetchPage(false).catch(() => {});
  });
  let searchTimer;
  search.addEventListener("input", () => {
    draw();
    clearTimeout(searchTimer);
    searchTimer = setTimeout(() => {
      if (loadPage) fetchPage(false).catch(() => {});
      else {
        rows = [...initialRows];
        nextCursor = execution.pagination?.next_cursor || null;
        previousPages = [];
        previousPage.disabled = true;
        nextPage.disabled = !loadPage || !execution.pagination?.has_more;
        draw();
      }
    }, 250);
  });
  head.addEventListener("click", (event) => {
    const filterToggle = event.target.closest("[data-table-filter-toggle]");
    if (filterToggle) {
      const menu = head.querySelector(
        `[data-table-filter-menu="${filterToggle.dataset.tableFilterToggle}"]`,
      );
      const opening = menu.hidden;
      for (const other of head.querySelectorAll("[data-table-filter-menu]")) other.hidden = true;
      for (const other of head.querySelectorAll("[data-table-filter-toggle]")) {
        other.setAttribute("aria-expanded", "false");
      }
      menu.hidden = !opening;
      filterToggle.setAttribute("aria-expanded", String(opening));
      if (opening) menu.querySelector("select")?.focus();
      return;
    }
    const applyFilter = event.target.closest("[data-table-filter-apply]");
    if (applyFilter) {
      const field = applyFilter.dataset.tableFilterApply;
      const menu = applyFilter.closest("[data-table-filter-menu]");
      const operator = menu.querySelector("[data-table-filter-operator]").value;
      const value = menu.querySelector("[data-table-filter-value]").value.trim();
      if (!tableFilterNeedsValue(operator) || value) filters.set(field, { operator, value });
      else filters.delete(field);
      menu.hidden = true;
      fetchPage(false).catch(() => {});
      return;
    }
    const clearFilter = event.target.closest("[data-table-filter-clear]");
    if (clearFilter) {
      filters.delete(clearFilter.dataset.tableFilterClear);
      clearFilter.closest("[data-table-filter-menu]").hidden = true;
      fetchPage(false).catch(() => {});
      return;
    }
    const button = event.target.closest("[data-sort-field]");
    if (!button) return;
    sort = sort?.field === button.dataset.sortField
      ? { field: button.dataset.sortField, direction: sort.direction === "asc" ? "desc" : "asc" }
      : { field: button.dataset.sortField, direction: "asc" };
    fetchPage(false).catch(() => {});
  });
  head.addEventListener("change", (event) => {
    if (!event.target.matches("[data-table-filter-operator]")) return;
    const menu = event.target.closest("[data-table-filter-menu]");
    const valueLabel = menu.querySelector("[data-table-filter-value]").closest("label");
    valueLabel.hidden = !tableFilterNeedsValue(event.target.value);
    menu.querySelector("[data-table-filter-value]").placeholder =
      event.target.value === "between" ? "Start..end" : "Filter value";
  });
  head.addEventListener("keydown", (event) => {
    if (event.key !== "Escape") return;
    const menu = event.target.closest("[data-table-filter-menu]");
    if (!menu) return;
    menu.hidden = true;
    const toggle = head.querySelector(`[data-table-filter-toggle="${menu.dataset.tableFilterMenu}"]`);
    toggle?.setAttribute("aria-expanded", "false");
    toggle?.focus();
  });
  nextPage.addEventListener("click", () => fetchPage(true).catch(() => {}));
  previousPage.addEventListener("click", () => {
    const previous = previousPages.pop();
    if (!previous) return;
    rows = previous.rows;
    nextCursor = previous.nextCursor;
    previousPage.disabled = !previousPages.length;
    nextPage.disabled = false;
    draw();
  });
  pageSize.addEventListener("change", () => fetchPage(false).catch(() => {}));
  reset.addEventListener("click", () => {
    search.value = "";
    visible.clear();
    for (const input of columnMenu.querySelectorAll("input")) {
      input.checked = true;
      visible.add(input.value);
    }
    sort = null;
    filters.clear();
    if (loadPage) fetchPage(false).catch(() => {});
    else {
      rows = [...initialRows];
      nextCursor = execution.pagination?.next_cursor || null;
      previousPages = [];
      previousPage.disabled = true;
      nextPage.disabled = true;
      draw();
    }
  });
  draw();
}

function renderBarChart(svg, values, tooltip, execution) {
  const width = 640;
  const height = 340;
  const left = 120;
  const bottom = 292;
  const orientation = execution.bar_orientation || "horizontal";
  const comparison = values.some((item) => item.comparison);
  const stacked = execution.bar_comparison_layout === "stacked" &&
    comparison;
  const groups = Array.from(values.reduce((map, item) => {
    const key = item.x || "Value";
    if (!map.has(key)) map.set(key, []);
    map.get(key).push(item);
    return map;
  }, new Map()));
  const positives = stacked
    ? groups.map(([, items]) => items.reduce((sum, item) => sum + Math.max(Number(item.value) || 0, 0), 0))
    : values.map((item) => Math.max(Number(item.value) || 0, 0));
  const negatives = stacked
    ? groups.map(([, items]) => items.reduce((sum, item) => sum + Math.min(Number(item.value) || 0, 0), 0))
    : values.map((item) => Math.min(Number(item.value) || 0, 0));
  const domainMax = Math.max(...positives, 0);
  const domainMin = Math.min(...negatives, 0);
  const span = Math.max(domainMax - domainMin, 1);
  const horizontalSpace = width - left - 35;
  const verticalSpace = bottom - 34;
  const zeroX = left + ((0 - domainMin) / span) * horizontalSpace;
  const zeroY = bottom - ((0 - domainMin) / span) * verticalSpace;
  if (orientation === "vertical") {
    const entries = stacked || comparison ? groups : values.map((item) => [item.x || "Value", [item]]);
    const groupWidth = (width - left - 30) / Math.max(entries.length, 1);
    entries.forEach(([category, items], groupIndex) => {
      let positiveAccumulated = 0;
      let negativeAccumulated = 0;
      const itemWidth = stacked ? groupWidth * 0.62 : (groupWidth * 0.76) / items.length;
      items.forEach((item, itemIndex) => {
        const value = Number(item.value) || 0;
        const barHeight = (verticalSpace * Math.abs(value)) / span;
        const x = stacked
          ? left + groupIndex * groupWidth + (groupWidth - itemWidth) / 2
          : left + groupIndex * groupWidth + groupWidth * 0.12 + itemIndex * itemWidth;
        const offset = value >= 0 ? positiveAccumulated : negativeAccumulated;
        const y = value >= 0
          ? zeroY - (verticalSpace * (offset + value)) / span
          : zeroY + (verticalSpace * Math.abs(offset)) / span;
        const rect = svgElement("rect", {
          x, y, width: itemWidth, height: barHeight,
          fill: item.color || stableChartColor(item.comparison || category, itemIndex), rx: 3,
        });
        interactiveMark(rect, `${category}${item.comparison ? `, ${item.comparison}` : ""}: ${item.display_value ?? value}`, tooltip);
        svg.append(rect);
        if (stacked) {
          if (value >= 0) positiveAccumulated += value;
          else negativeAccumulated += value;
        }
      });
      const label = svgElement("text", { x: left + groupIndex * groupWidth + groupWidth / 2, y: bottom + 20, "text-anchor": "middle" });
      label.textContent = category;
      svg.append(label);
    });
  } else {
    const entries = stacked || comparison ? groups : values.map((item) => [item.x || "Value", [item]]);
    const rowHeight = Math.min(46, 250 / Math.max(entries.length, 1));
    entries.forEach(([category, items], groupIndex) => {
      let positiveAccumulated = 0;
      let negativeAccumulated = 0;
      items.forEach((item, itemIndex) => {
        const value = Number(item.value) || 0;
        const offset = value >= 0 ? positiveAccumulated : negativeAccumulated;
        const barWidth = (horizontalSpace * Math.abs(value)) / span;
        const x = value >= 0
          ? zeroX + (horizontalSpace * offset) / span
          : zeroX - (horizontalSpace * Math.abs(offset + value)) / span;
        const y = 34 + groupIndex * rowHeight + (stacked ? 0 : itemIndex * Math.max(8, (rowHeight - 8) / items.length));
        const rect = svgElement("rect", {
          x,
          y,
          width: barWidth,
          height: stacked ? Math.max(10, rowHeight - 8) : Math.max(8, (rowHeight - 8) / items.length),
          fill: item.color || stableChartColor(item.comparison || category, itemIndex),
          rx: 3,
        });
        interactiveMark(rect, `${category}${item.comparison ? `, ${item.comparison}` : ""}: ${item.display_value ?? value}`, tooltip);
        svg.append(rect);
        if (stacked) {
          if (value >= 0) positiveAccumulated += value;
          else negativeAccumulated += value;
        }
      });
      const text = svgElement("text", { x: left - 8, y: 34 + groupIndex * rowHeight + rowHeight / 2, "text-anchor": "end" });
      text.textContent = category;
      svg.append(text);
    });
  }
  svg.prepend(orientation === "vertical"
    ? svgElement("line", { class: "component-d3-chart__zero-line", x1: left, x2: width - 30, y1: zeroY, y2: zeroY, stroke: "currentColor" })
    : svgElement("line", { class: "component-d3-chart__zero-line", x1: zeroX, x2: zeroX, y1: 28, y2: bottom, stroke: "currentColor" }));
  if (execution.x_axis_label) {
    const label = svgElement("text", { x: width / 2, y: height - 8, "text-anchor": "middle" });
    label.textContent = execution.x_axis_label;
    svg.append(label);
  }
  if (execution.y_axis_label) {
    const label = svgElement("text", { x: 16, y: height / 2, transform: `rotate(-90 16 ${height / 2})`, "text-anchor": "middle" });
    label.textContent = execution.y_axis_label;
    svg.append(label);
  }
}

function renderLineChart(svg, values, tooltip, execution) {
  const width = 640;
  const height = 340;
  const pad = 48;
  const numeric = values.map((item) => Number(item.value) || 0);
  const max = Math.max(...numeric, 0);
  const min = Math.min(...numeric, 0);
  const span = Math.max(max - min, 1);
  const points = values.map((item, index) => ({
    x: pad + index * ((width - pad * 2) / Math.max(values.length - 1, 1)),
    y: height - pad - (((Number(item.value) || 0) - min) / span) * (height - pad * 2),
    item,
  }));
  svg.append(
    svgElement("line", {
      class: "component-d3-chart__axis component-d3-chart__axis--x",
      x1: pad, x2: width - pad, y1: height - pad, y2: height - pad, stroke: "currentColor",
    }),
    svgElement("line", {
      class: "component-d3-chart__axis component-d3-chart__axis--y",
      x1: pad, x2: pad, y1: pad, y2: height - pad, stroke: "currentColor",
    }),
  );
  if (min <= 0 && max >= 0) {
    const zeroY = height - pad - ((0 - min) / span) * (height - pad * 2);
    svg.append(svgElement("line", {
      class: "component-d3-chart__zero-line",
      x1: pad, x2: width - pad, y1: zeroY, y2: zeroY, stroke: "currentColor",
    }));
  }
  for (const [value, y] of [[max, pad], [min, height - pad]]) {
    const tick = svgElement("text", { x: pad - 8, y: y + 4, "text-anchor": "end" });
    tick.textContent = String(value);
    svg.append(tick);
  }
  const labelStride = Math.max(1, Math.ceil(points.length / 8));
  points.forEach((point, index) => {
    if (index % labelStride !== 0 && index !== points.length - 1) return;
    const label = svgElement("text", {
      class: "component-d3-chart__category-label",
      x: point.x,
      y: height - pad + 18,
      "text-anchor": "middle",
    });
    label.textContent = point.item.x || "Value";
    svg.append(label);
  });
  const path = execution.line_smoothing
    ? svgElement("path", {
      d: points.reduce((pathData, point, index) => {
        if (index === 0) return `M ${point.x} ${point.y}`;
        const prior = points[index - 1];
        const midpoint = (prior.x + point.x) / 2;
        return `${pathData} C ${midpoint} ${prior.y}, ${midpoint} ${point.y}, ${point.x} ${point.y}`;
      }, ""),
      fill: "none",
      stroke: CHART_COLORS[0],
      "stroke-width": 4,
    })
    : svgElement("polyline", {
      points: points.map((point) => `${point.x},${point.y}`).join(" "),
      fill: "none",
      stroke: CHART_COLORS[0],
      "stroke-width": 4,
      "stroke-linejoin": "round",
    });
  svg.append(path);
  for (const point of points) {
    const mark = svgElement("circle", { cx: point.x, cy: point.y, r: 6, fill: CHART_COLORS[0] });
    interactiveMark(mark, `${point.item.x || "Value"}: ${point.item.display_value ?? point.item.value}`, tooltip);
    svg.append(mark);
  }
  if (execution.x_axis_label) {
    const label = svgElement("text", { x: width / 2, y: height - 4, "text-anchor": "middle" });
    label.textContent = execution.x_axis_label;
    svg.append(label);
  }
  if (execution.y_axis_label) {
    const label = svgElement("text", {
      x: 14,
      y: height / 2,
      transform: `rotate(-90 14 ${height / 2})`,
      "text-anchor": "middle",
    });
    label.textContent = execution.y_axis_label;
    svg.append(label);
  }
}

function polar(cx, cy, radius, angle) {
  return { x: cx + radius * Math.cos(angle), y: cy + radius * Math.sin(angle) };
}

function renderPieChart(svg, values, tooltip, donut) {
  const cx = 250;
  const cy = 170;
  const radius = 125;
  const total = values.reduce((sum, item) => sum + Math.max(Number(item.value) || 0, 0), 0) || 1;
  if (values.length === 1) {
    const item = values[0];
    const mark = svgElement("circle", {
      cx, cy, r: radius,
      fill: item.color || CHART_COLORS[0],
    });
    interactiveMark(mark, `${item.category || "Value"}: ${item.display_value ?? item.value}`, tooltip);
    svg.append(mark);
    if (donut) svg.append(svgElement("circle", { cx, cy, r: radius * 0.55, fill: "var(--surface-raised, #fff)" }));
    const legend = svgElement("text", { x: 420, y: 45 });
    legend.textContent = `${item.category || "Value"} · ${item.display_value ?? item.value}`;
    svg.append(legend);
    return;
  }
  let angle = -Math.PI / 2;
  values.forEach((item, index) => {
    const portion = (Math.max(Number(item.value) || 0, 0) / total) * Math.PI * 2;
    const end = angle + portion;
    const startPoint = polar(cx, cy, radius, angle);
    const endPoint = polar(cx, cy, radius, end);
    const inner = donut ? radius * 0.55 : 0;
    const innerEnd = polar(cx, cy, inner, end);
    const innerStart = polar(cx, cy, inner, angle);
    const path = svgElement("path", {
      d: `M ${startPoint.x} ${startPoint.y} A ${radius} ${radius} 0 ${portion > Math.PI ? 1 : 0} 1 ${endPoint.x} ${endPoint.y} L ${innerEnd.x} ${innerEnd.y} A ${inner} ${inner} 0 ${portion > Math.PI ? 1 : 0} 0 ${innerStart.x} ${innerStart.y} Z`,
      fill: item.color || CHART_COLORS[index % CHART_COLORS.length],
    });
    interactiveMark(path, `${item.category || "Value"}: ${item.display_value ?? item.value}`, tooltip);
    svg.append(path);
    const legend = svgElement("text", { x: 420, y: 45 + index * 28 });
    legend.textContent = `${item.category || "Value"} · ${item.display_value ?? item.value}`;
    svg.append(legend);
    angle = end;
  });
}

function renderVisual(target, execution) {
  const chart = element("div", null, "component-d3-chart__surface");
  const values = execution.slices?.length ? execution.slices : execution.points || [];
  const visualHeight = ["pie", "donut"].includes(execution.component_type)
    ? Math.max(340, values.length * 28 + 96)
    : 340;
  const svg = svgElement("svg", {
    class: `component-d3-svg component-d3-svg--${["pie", "donut"].includes(execution.component_type) ? "slices" : execution.component_type}`,
    viewBox: `0 0 640 ${visualHeight}`,
    role: "img",
    "aria-label": `${execution.component_type.replaceAll("_", " ")} Component chart`,
  });
  const title = svgElement("title");
  title.textContent = execution.legend_title || "Component chart";
  svg.append(title);
  const tooltip = chartTooltip(chart);
  if (!values.length) {
    const empty = element("section", null, "component-empty-state");
    empty.append(
      element("h3", "No visual data"),
      element("p", "No data is available for this Component configuration."),
    );
    target.append(empty);
    return;
  }
  if (execution.component_type === "bar") renderBarChart(svg, values, tooltip, execution);
  else if (execution.component_type === "line") renderLineChart(svg, values, tooltip, execution);
  else renderPieChart(svg, values, tooltip, execution.component_type === "donut");
  chart.prepend(svg);
  if (execution.component_type === "bar") {
    const series = new Map();
    for (const item of values) {
      if (item.comparison && !series.has(item.comparison)) {
        series.set(item.comparison, item.color || stableChartColor(item.comparison));
      }
    }
    if (series.size) {
      const legend = element("ul", null, "component-d3-chart__legend");
      for (const [label, color] of series) {
        const item = document.createElement("li");
        const swatch = element("span", null, "component-d3-chart__legend-swatch");
        swatch.style.backgroundColor = color;
        swatch.dataset.series = label;
        item.append(swatch, document.createTextNode(label));
        legend.append(item);
      }
      chart.insertBefore(legend, svg);
    }
  }
  chart.addEventListener("keydown", (event) => {
    if (event.key === "Escape") tooltip.hidden = true;
  });
  target.append(chart);
}

function renderExecution(target, execution, options = {}) {
  target.replaceChildren();
  target.classList.toggle("component-visual-preview", execution.component_type !== "table");
  if (execution.component_type === "table") {
    renderTable(target, execution, options.loadPage);
    return;
  }
  if (execution.component_type === "stat_card") {
    const card = element("article", null, `component-stat-card component-stat-card--${execution.stat?.panel_style || "default"}`);
    card.append(
      element("p", execution.stat?.label || "Value", "eyebrow"),
      element("p", execution.stat?.display_value ?? "—", "component-stat-card__value"),
    );
    if (execution.stat?.supporting_text) card.append(element("p", execution.stat.supporting_text));
    target.append(card);
    return;
  }
  renderVisual(target, execution);
}

function loadComponentRenderers(root, signal) {
  for (const panel of root.querySelectorAll?.("[data-component-render]") || []) {
    const componentRef = panel.dataset.componentRef;
    const versionId = panel.dataset.versionId;
    const kind = panel.dataset.componentKind.replaceAll("_", "-");
    const endpoint = `/api/components/${componentRef}/versions/${versionId}/${kind}`;
    request(endpoint, "GET", undefined, undefined, signal)
      .then((execution) => {
        renderExecution(panel.querySelector("[data-component-render-content]"), execution, {
          loadPage: (query) => request(`${endpoint}${query ? `?${query}` : ""}`, "GET", undefined, undefined, signal),
        });
        panel.setAttribute("aria-busy", "false");
        status("Component data loaded.", false, panel);
      })
      .catch((error) => {
        panel.setAttribute("aria-busy", "false");
        status(error.message, true, panel);
      });
  }
}

function activate(root, bootstrap, signal) {
  initializeDirectory(root, signal);
  initializeManagementAffordances(root, signal);
  initializeEditor(root, bootstrap, signal);
  installAuthoringHandler(root, signal);
  loadComponentRenderers(root, signal);
}

async function renderCompleteDocumentContent(outlet, path, signal) {
  const response = await fetch(path, { headers: { Accept: "text/html" }, signal });
  if (!response.ok) throw new Error(`Component document returned HTTP ${response.status}`);
  const parsed = new DOMParser().parseFromString(await response.text(), "text/html");
  const content = parsed.getElementById(COMPLETE_DOCUMENT_ROOT_ID);
  if (!content) throw new Error("Component document did not contain its module content");
  const bootstrap = readBootstrap(parsed);
  outlet.replaceChildren(...Array.from(content.childNodes));
  return bootstrap;
}

if (document.getElementById(COMPLETE_DOCUMENT_ROOT_ID)) {
  const root = document.getElementById(COMPLETE_DOCUMENT_ROOT_ID);
  const controller = new AbortController();
  activate(document, readBootstrap(document), controller.signal);
  root.setAttribute("data-hydration", "ready");
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
    const bootstrap = await renderCompleteDocumentContent(outlet, input.bootstrap.path, controller.signal);
    activate(outlet, bootstrap || input.bootstrap.payload, controller.signal);
  };
  return {
    async mount(input) { await render(input); },
    async navigate(input) { await render(input); },
    async canDeactivate() {
      assertActive();
      return hasUnsavedChanges(outlet ?? document)
        ? { allowed: false, prompt: "Discard unsaved Component changes?" }
        : { allowed: true };
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
      if (disposed) return;
      controller.abort();
      if (outlet) outlet.replaceChildren();
      disposed = true;
    },
  };
}
