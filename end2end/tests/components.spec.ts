import { expect, test, type APIResponse, type Page } from "@playwright/test";
import { invokeDemoSeedEndpoint } from "./support/demo-seed";

const RUN_ID = `pw-components-${Date.now()}`;
const TWO_HUNDRED_PERCENT_ZOOM_VIEWPORT = { width: 640, height: 450 };
const RENDERED_COMPONENT_CONTENT = [
  ".component-table-viewer__table",
  ".component-d3-chart__surface",
  ".component-stat-card",
].join(", ");

type DatasetReference = {
  reference: {
    installation_id: string;
    owner: {
      kind: "core_installation";
      installation_id: string;
    };
    resource_type: "tessara.transition.dataset_major_line";
    resource_id: string;
  };
};

type DatasetOption = {
  reference: DatasetReference;
  dataset_name: string;
  dataset_slug: string;
  grain: string;
  tags: string[];
  provenance: {
    forms: Array<{ id: string; name: string; slug?: string | null }>;
    datasets: Array<{ id: string; name: string; slug?: string | null }>;
  };
  materialization_state?: string;
  fields: Array<{
    key: string;
    label: string;
    field_type: string;
    restriction_tier: string;
  }>;
  scope_node_ids: string[];
};

type DatasetCatalog = {
  schema_version: number;
  datasets: DatasetOption[];
};

type ComponentVersion = {
  component_version_id: string;
  dataset_reference: DatasetReference;
  component_type: string;
  publication_state: string;
  lifecycle_state: string;
  version_note: string;
  config: Record<string, unknown>;
};

type ComponentDefinition = {
  schema_version: number;
  component_id: string;
  name: string;
  slug: string;
  versions: ComponentVersion[];
};

type ComponentSummary = {
  schema_version: number;
  component_id: string;
  name: string;
  slug: string;
  current_version: ComponentVersion;
};

type DashboardSummary = {
  id: string;
  name: string;
  placement_count: number;
};

type ValidationResponse = {
  schema_version: number;
  valid: boolean;
  findings: Array<{ code: string; field_path?: string | null }>;
};

async function expectJson<T>(response: APIResponse): Promise<T> {
  const text = await response.text();
  expect(
    response.ok(),
    `${response.url()} returned ${response.status()}: ${text}`,
  ).toBeTruthy();
  return JSON.parse(text) as T;
}

async function signInAsAdmin(page: Page) {
  await expectJson(
    await page.request.post("/api/auth/login", {
      data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
    }),
  );
}

async function ensureDemoSeed(page: Page) {
  const response = await invokeDemoSeedEndpoint(page.request);
  if (response === null) return;
  const text = await response.text();
  expect(
    response.ok() ||
      (response.status() === 400 &&
        text.includes("Demo seed requires an empty database")),
    text,
  ).toBeTruthy();
}

async function datasetOption(page: Page) {
  const catalog = await expectJson<DatasetCatalog>(
    await page.request.get("/api/admin/components/datasets"),
  );
  const datasets = catalog.datasets;
  const dataset =
    datasets.find((candidate) =>
      candidate.fields.some(
        (field) => field.field_type.toLowerCase() !== "number",
      ),
    ) ?? datasets.find((candidate) => candidate.fields.length > 0);
  expect(
    dataset,
    "Component authoring requires one ready Dataset major line",
  ).toBeTruthy();
  expect(dataset!.reference.reference.resource_type).toBe(
    "tessara.transition.dataset_major_line",
  );
  expect(dataset!.reference.reference.owner).toEqual({
    kind: "core_installation",
    installation_id: dataset!.reference.reference.installation_id,
  });
  expect(datasetMajor(dataset!.reference)).toBeGreaterThan(0);
  return dataset!;
}

function datasetMajor(reference: DatasetReference): number {
  const match = reference.reference.resource_id.match(
    /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}@([1-9][0-9]*)$/,
  );
  if (match === null) {
    throw new Error(
      `Dataset major-line reference has a non-canonical resource identity: ${reference.reference.resource_id}`,
    );
  }
  return Number.parseInt(match[1], 10);
}

function tableConfig(dataset: DatasetOption) {
  const visible = dataset.fields.slice(0, 3).map((field) => field.key);
  return { visible_columns: visible, search_fields: visible, page_size: 25 };
}

function visualConfig(kind: string, fieldKey: string) {
  if (kind === "bar") {
    return {
      mode: "summary",
      summary_field: fieldKey,
      summary_type: "count",
      category_field: fieldKey,
      sort_field: "summary_value",
      sort_direction: "desc",
      number_of_points: 20,
      value_format: "integer",
    };
  }
  if (kind === "line") {
    return {
      summary_field: fieldKey,
      summary_type: "count",
      x_field: fieldKey,
      sort_field: "summary_value",
      sort_direction: "desc",
      number_of_points: 20,
      value_format: "integer",
      x_axis_label: "Category",
      y_axis_label: "Responses",
    };
  }
  if (kind === "pie" || kind === "donut") {
    return {
      summary_field: fieldKey,
      summary_type: "count",
      category_field: fieldKey,
      max_slices: 20,
      value_format: "integer",
    };
  }
  return {
    summary_field: fieldKey,
    summary_type: "count",
    label: "Row count",
    value_format: "integer",
  };
}

function versionInput(
  dataset: DatasetOption,
  componentType: string,
  config: Record<string, unknown>,
  versionNote: string,
) {
  return {
    dataset_reference: dataset.reference,
    component_type: componentType,
    config,
    version_note: versionNote,
  };
}

function idempotencyHeaders(action: string) {
  return { "x-idempotency-key": `${RUN_ID}-${action}` };
}

async function createComponent(
  page: Page,
  dataset: DatasetOption,
  componentType: string,
  config: Record<string, unknown>,
  suffix: string,
) {
  const slug = `${RUN_ID}-${suffix}`;
  const name = `Playwright ${suffix} ${RUN_ID}`;
  const definition = await expectJson<ComponentDefinition>(
    await page.request.post("/api/admin/components", {
      headers: idempotencyHeaders(`create-${suffix}`),
      data: {
        schema_version: 1,
        name,
        slug,
        description: "Extracted Component module acceptance fixture.",
        version: versionInput(dataset, componentType, config, "Initial draft"),
      },
    }),
  );
  expect(definition).toMatchObject({ schema_version: 1, name, slug });
  expect(definition.versions).toHaveLength(1);
  return definition;
}

async function publish(
  page: Page,
  definition: ComponentDefinition,
  action: string,
) {
  const version = definition.versions[0];
  await expectJson(
    await page.request.post(
      `/api/admin/components/${definition.component_id}/versions/${version.component_version_id}/publish`,
      { headers: idempotencyHeaders(action), data: {} },
    ),
  );
}

function renderPath(slug: string, kind: string) {
  return `/api/components/${slug}/${kind === "stat_card" ? "stat-card" : kind}`;
}

function attachConsoleGuard(page: Page) {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  return () => {
    expect(
      errors,
      `Component routes must not emit browser console or hydration errors:\n${errors.join("\n")}`,
    ).toEqual([]);
  };
}

async function chooseThemeWithKeyboard(page: Page, theme: "light" | "dark") {
  const themeTrigger = page.getByRole("button", { name: "Theme options" });
  await themeTrigger.focus();
  await page.keyboard.press("Enter");
  const themeOption = page.getByRole("menuitemradio", {
    name: theme === "light" ? "Light" : "Dark",
    exact: true,
  });
  await expect(themeOption).toBeVisible();
  await themeOption.focus();
  await page.keyboard.press("Enter");
  await expect(page.locator("html")).toHaveAttribute(
    "data-theme-preference",
    theme,
  );
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

async function expectHorizontalContainment(
  page: Page,
  selector: string,
  description: string,
) {
  const targets = page.locator(selector);
  await expect(
    targets.first(),
    `${description} should expose a visible containment target`,
  ).toBeVisible();
  const metrics = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    devicePixelRatio: window.devicePixelRatio,
    innerWidth: window.innerWidth,
    scrollWidth: document.documentElement.scrollWidth,
  }));
  expect(
    metrics.innerWidth,
    "200% zoom should expose the 640 CSS-pixel layout viewport",
  ).toBe(TWO_HUNDRED_PERCENT_ZOOM_VIEWPORT.width);
  expect(
    metrics.devicePixelRatio,
    "200% zoom should render at two device pixels per CSS pixel",
  ).toBe(2);
  expect(
    metrics.scrollWidth <= metrics.clientWidth + 1,
    `${description} should not create document-level horizontal overflow`,
  ).toBe(true);

  const outOfBounds = await targets.evaluateAll((elements) =>
    elements.flatMap((element, index) => {
      const style = window.getComputedStyle(element);
      if (style.display === "none" || style.visibility === "hidden") return [];
      const bounds = element.getBoundingClientRect();
      return bounds.left < -1 || bounds.right > window.innerWidth + 1
        ? [
            {
              index,
              left: bounds.left,
              right: bounds.right,
              viewport: window.innerWidth,
            },
          ]
        : [];
    }),
  );
  expect(
    outOfBounds,
    `${description} should remain within the layout viewport`,
  ).toEqual([]);
}

async function selectDatasetMajorLine(page: Page, dataset: DatasetOption) {
  const major = datasetMajor(dataset.reference);
  const picker = page.getByRole("combobox", { name: "Dataset Version" });
  await expect(picker).toBeVisible();
  await picker.click();
  const datasetTable = page
    .getByRole("table")
    .filter({ hasText: "Provenance" });
  for (const heading of ["Dataset", "Version", "Grain", "Tags", "Provenance"]) {
    await expect(
      datasetTable.getByRole("columnheader", { name: heading }),
    ).toBeVisible();
  }
  const search = page.getByRole("searchbox", {
    name: "Filter dataset versions",
  });
  await expect(search).toBeVisible();
  await search.fill(dataset.dataset_name);
  const option = page
    .getByRole("option")
    .filter({ hasText: dataset.dataset_name })
    .first();
  await expect(option).toContainText(`v${major}`);
  await expect(option).toContainText(dataset.grain || "—");
  await option.getByRole("button", { name: dataset.dataset_name }).click();
  await expect(picker).toContainText(dataset.dataset_name);
  await expect(
    page.getByRole("heading", {
      name: `${dataset.dataset_name} v${major} field preview`,
    }),
  ).toBeVisible();
}

async function selectComponentKind(page: Page, label: string) {
  await page.getByRole("radio", { name: label, exact: true }).click();
  const confirmation = page.getByRole("button", {
    name: `Change to ${label}`,
    exact: true,
  });
  if ((await confirmation.count()) > 0 && (await confirmation.isVisible())) {
    await confirmation.click();
  }
}

test.describe("Sprint 8A extracted Component UI parity", () => {
  test("admin can create, update, publish, and view a major-line table component", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const dataset = await datasetOption(page);

    await page.goto("/components/new");
    await expect(
      page.getByRole("heading", { level: 1, name: "Create Component" }),
    ).toBeVisible();
    await selectDatasetMajorLine(page, dataset);
    await expect(
      page.getByRole("group", { name: "Dataset Context" }),
    ).toHaveCount(0);
    const displayedFields = page.getByRole("group", {
      name: "Displayed Fields",
    });
    await expect(displayedFields).toBeVisible();
    const availableFields = displayedFields.getByRole("listbox", {
      name: "Available fields",
    });
    await expect(
      availableFields.locator(".dataset-projection-builder__option"),
    ).not.toHaveCount(0);
    await expect(
      page.getByRole("textbox", { name: "Configuration JSON" }),
    ).toHaveCount(0);

    const invalid = await expectJson<ValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: versionInput(
          dataset,
          "table",
          { visible_columns: [`missing_${RUN_ID}`] },
          "Invalid field probe",
        ),
      }),
    );
    expect(invalid.valid).toBe(false);
    expect(invalid.findings[0]).toMatchObject({
      code: "config.field_unavailable",
    });

    const definition = await createComponent(
      page,
      dataset,
      "table",
      tableConfig(dataset),
      "table",
    );
    const draft = definition.versions[0];
    await expectJson(
      await page.request.put(
        `/api/admin/components/${definition.component_id}/versions/${draft.component_version_id}`,
        {
          headers: idempotencyHeaders("update-table"),
          data: {
            schema_version: 1,
            version: versionInput(
              dataset,
              "table",
              tableConfig(dataset),
              "Updated draft",
            ),
          },
        },
      ),
    );
    await publish(page, definition, "publish-table");

    const current = await expectJson<ComponentDefinition>(
      await page.request.get(`/api/admin/components/${definition.slug}`),
    );
    expect(current.versions[0]).toMatchObject({
      component_version_id: draft.component_version_id,
      publication_state: "published",
      component_type: "table",
      version_note: "Updated draft",
    });
    const rendered = await expectJson<{
      materialization_state: string;
      component_type: string;
    }>(await page.request.get(renderPath(definition.slug, "table")));
    expect(rendered).toMatchObject({
      materialization_state: "ready",
      component_type: "table",
    });

    await page.goto(`/components/${definition.slug}/view`);
    await expect(
      page.getByRole("heading", { level: 1, name: definition.name }),
    ).toBeVisible();
    await expect(
      page.getByRole("searchbox", { name: "Search component rows" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Choose visible columns" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Reset table controls" }),
    ).toBeVisible();
    await expect(page.locator(".component-table-viewer__table")).toBeVisible();
    await expect(
      page.getByRole("table").filter({ hasText: dataset.fields[0].label }),
    ).toBeVisible();

    const firstField = dataset.fields[0];
    const filterTrigger = page.getByRole("button", {
      name: `Filter ${firstField.label}`,
    });
    await filterTrigger.click();
    const filterDialog = page.getByRole("dialog", {
      name: `Filter ${firstField.label}`,
    });
    await filterDialog.getByLabel("Operator").selectOption("equals");
    await filterDialog.getByLabel("Value").fill(`unlikely-${RUN_ID}`);
    const filterResponse = page.waitForResponse(
      (response) =>
        response
          .url()
          .includes(
            `filter%5B${encodeURIComponent(firstField.key)}%5D%5Boperator%5D=equals`,
          ) && response.request().method() === "GET",
    );
    await filterDialog.getByRole("button", { name: "Apply filter" }).click();
    await filterResponse;

    const rowSearch = page.getByRole("searchbox", {
      name: "Search component rows",
    });
    await rowSearch.fill(RUN_ID);
    await page.waitForResponse((response) =>
      response.url().includes(`search=${RUN_ID}`),
    );
    const clearedSearch = page.waitForResponse((response) => {
      const url = new URL(response.url());
      return (
        url.pathname.endsWith(renderPath(definition.slug, "table")) &&
        !url.searchParams.has("search") &&
        url.searchParams.get(`filter[${firstField.key}][operator]`) === "equals"
      );
    });
    await rowSearch.fill("");
    await clearedSearch;

    if (dataset.fields.length > 1) {
      await page
        .getByRole("button", { name: "Choose visible columns" })
        .click();
      const secondField = dataset.fields[1];
      const projectionResponse = page.waitForResponse((response) => {
        const url = new URL(response.url());
        return (
          url.pathname.endsWith(renderPath(definition.slug, "table")) &&
          url.searchParams.has("visible_columns") &&
          !url.searchParams
            .get("visible_columns")!
            .split(",")
            .includes(secondField.key)
        );
      });
      await page
        .getByRole("group", { name: "Visible columns" })
        .getByLabel(secondField.label, { exact: false })
        .uncheck();
      await projectionResponse;
    }
    const invalidProjection = await page.request.get(
      `${renderPath(definition.slug, "table")}?visible_columns=missing_${RUN_ID}`,
    );
    expect(invalidProjection.status()).toBe(400);

    await page.goto(`/components/${definition.slug}`);
    await expect(
      page.getByRole("heading", { level: 1, name: definition.name }),
    ).toBeVisible();
    await expect(
      page.getByRole("searchbox", { name: "Search component rows" }),
    ).toBeVisible();
    await expect(
      page.getByRole("button", { name: "Reset table controls" }),
    ).toBeVisible();
    await expect(page.locator(".component-table-viewer__table")).toBeVisible();
    await expect(
      page.getByRole("heading", { level: 2, name: "Versions" }),
    ).toHaveCount(0);

    await page.goto("/components");
    await expect(
      page.getByRole("link", { name: "Create Component" }),
    ).toBeVisible();
    await page
      .getByRole("searchbox", { name: "Search components by name" })
      .fill(definition.name);
    const componentLink = page
      .getByRole("link", { name: definition.name })
      .first();
    await expect(componentLink).toBeVisible();
    await expect(componentLink).toHaveAttribute(
      "href",
      `/components/${definition.slug}`,
    );
    await expect(componentLink).not.toHaveAttribute(
      "href",
      new RegExp(definition.component_id),
    );
    await page.getByRole("button", { name: "Filter Kind" }).click();
    await page.getByRole("menuitemradio", { name: "Table" }).click();
    await page.getByRole("button", { name: "Filter Status" }).click();
    await page.getByRole("menuitemradio", { name: "Published" }).click();
    await expect(
      page.getByRole("link", { name: definition.name }),
    ).toBeVisible();

    await page.setViewportSize({ width: 768, height: 900 });
    await expect(
      page.getByRole("searchbox", { name: "Search components by name" }),
    ).toBeVisible();
    expect(
      await page.evaluate(
        () =>
          document.documentElement.scrollWidth >
          document.documentElement.clientWidth,
      ),
    ).toBe(false);

    await page.setViewportSize({ width: 390, height: 844 });
    const openFilters = page.getByRole("button", {
      name: "Open component filters",
    });
    await openFilters.focus();
    await page.keyboard.press("Enter");
    const filtersDialog = page.getByRole("dialog", {
      name: "Component filters",
    });
    await expect(filtersDialog).toBeVisible();
    await page.getByLabel("Filter components by kind").selectOption("table");
    await page
      .getByLabel("Filter components by status")
      .selectOption("published");
    await page.getByRole("button", { name: "Clear All" }).click();
    await expect(page.getByLabel("Filter components by kind")).toHaveValue(
      "all",
    );
    await expect(page.getByLabel("Filter components by status")).toHaveValue(
      "all",
    );
    await page.getByLabel("Filter components by kind").selectOption("table");
    await page
      .getByLabel("Filter components by status")
      .selectOption("published");
    await page.getByTitle("Close component filters").click();
    await expect(
      page
        .locator(".components-list-mobile-card")
        .filter({ hasText: definition.name }),
    ).toBeVisible();
    await expect(
      page.locator(".components-list-responsive-table .table-wrap"),
    ).toBeHidden();

    await page.setViewportSize({ width: 1280, height: 900 });
    await page.goto(`/components/${definition.slug}/versions`);
    await expect(page.getByRole("heading", { level: 1 })).toContainText(
      definition.name,
    );
    const versionsTable = page
      .getByRole("table")
      .filter({ hasText: "Publication" });
    await expect(
      versionsTable.getByRole("columnheader", { name: "Publication" }),
    ).toBeVisible();
    await expect(
      versionsTable.getByRole("columnheader", { name: "Lifecycle" }),
    ).toBeVisible();
    await expect(
      versionsTable.getByRole("columnheader", { name: "Dataset Version" }),
    ).toBeVisible();
    await expect(
      versionsTable.getByRole("columnheader", { name: "Version Note" }),
    ).toBeVisible();
    await expect(
      versionsTable.getByRole("columnheader", { name: "Actions" }),
    ).toBeVisible();
    await expect(versionsTable).toContainText("Published");
    await expect(versionsTable).toContainText("Updated draft");
    const actions = page.getByText(/Open actions for/).first();
    await actions.click();
    await page.getByRole("button", { name: "Archive", exact: true }).click();
    const actionDialog = page.getByRole("dialog", {
      name: "Archive Component version?",
    });
    await expect(actionDialog).toContainText("cannot be reactivated");
    await actionDialog.getByRole("button", { name: "Cancel" }).click();
    await expect(actionDialog).not.toBeVisible();

    await page.goto(`/components/${definition.slug}/edit`);
    await expect(
      page.getByRole("heading", { level: 1, name: "Edit Component" }),
    ).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "Name", exact: true }),
    ).toHaveValue(definition.name);
    await expect(
      page.getByRole("group", { name: "Displayed Fields" }),
    ).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "Configuration JSON" }),
    ).toHaveCount(0);
    await page.getByText("Publish", { exact: true }).click();
    await page
      .getByRole("button", { name: "Create New Version", exact: true })
      .click();
    const consumerReview = page.getByRole("dialog", {
      name: "Review component consumers",
    });
    await expect(consumerReview).toContainText(
      "Existing consumers remain pinned",
    );
    await consumerReview
      .getByRole("button", { name: "Create New Version" })
      .click();
    await expect(consumerReview.getByRole("alert")).toContainText(
      "New versions require",
    );
    await consumerReview
      .getByLabel("New Version Note")
      .fill("Behavioral review only");
    await consumerReview.getByRole("button", { name: "Cancel" }).click();
    assertNoConsoleErrors();
  });

  test("admin can author, publish, and view visual components", async ({
    page,
  }) => {
    test.setTimeout(180_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const dataset = await datasetOption(page);
    const field =
      dataset.fields.find(
        (candidate) => candidate.field_type.toLowerCase() !== "number",
      ) ?? dataset.fields[0];
    const fieldKey = field.key;

    await page.goto("/components/new");
    const unsavedName = `Playwright structured editor ${RUN_ID}`;
    await page
      .getByRole("textbox", { name: "Name", exact: true })
      .fill(unsavedName);
    await page
      .getByRole("textbox", { name: "Slug" })
      .fill(`${RUN_ID}-structured-editor`);
    await selectDatasetMajorLine(page, dataset);
    await expect(
      page.getByRole("group", { name: "Component Kind" }),
    ).toBeVisible();
    await expect(
      page.getByRole("textbox", { name: "Configuration JSON" }),
    ).toHaveCount(0);

    await selectComponentKind(page, "Bar");
    await expect(
      page.getByRole("heading", { name: "Build the bars" }),
    ).toBeVisible();
    for (const role of ["Category", "Series", "Measure"]) {
      await expect(
        page
          .locator(".component-editor__role-card")
          .filter({ hasText: role })
          .first(),
      ).toBeVisible();
    }
    const fieldsAndCalculation = page.getByRole("group", {
      name: "Fields & Calculation",
    });
    await expect(fieldsAndCalculation).toBeVisible();
    const calculation = page.getByLabel("Calculation", { exact: true });
    await expect(calculation).toBeVisible();
    await expect(calculation.locator("option")).toHaveText([
      "Count rows",
      "Count non-empty values",
      "Count unique values",
      "Sum",
      "Average",
      "Median",
      "Do not summarize",
    ]);
    const visualEditor = page.locator(
      '[data-component-config-section="visual"]',
    );
    const valueField = visualEditor.locator("[data-component-value-field]");
    const valueFieldSelect = valueField.getByLabel("Value field", {
      exact: true,
    });
    const valueMissingPolicy = visualEditor.locator(
      "[data-component-value-missing-policy]",
    );
    const calculationWarning = visualEditor.locator(
      "[data-component-calculation-warning]",
    );
    await calculation.selectOption("row_count");
    await expect(valueField).toBeHidden();
    await expect(valueFieldSelect).toBeDisabled();
    await expect(valueMissingPolicy).toBeHidden();

    await calculation.selectOption("none");
    await expect(valueField).toBeVisible();
    await expect(valueFieldSelect).toBeEnabled();
    await expect(calculationWarning).toBeVisible();
    await expect(calculationWarning).toHaveText(
      "Every category and series group must resolve to exactly one row. " +
        "Preview and execution will report an error when duplicates exist.",
    );
    const visualSort = visualEditor.getByLabel("Sort Field", { exact: true });
    await expect(visualSort.locator("option")).toHaveText([
      "Default",
      "Category",
      "Summary Value",
    ]);
    const sortHelpTrigger = visualEditor.locator(
      'summary[aria-label="Show help for Sort Field"]',
    );
    await sortHelpTrigger.focus();
    await page.keyboard.press("Enter");
    const sortHelp = visualEditor.getByRole("tooltip");
    await expect(sortHelp).toBeVisible();
    expect(await sortHelp.textContent()).toBe(
      "Default: uses the order produced by the current grouping and summarization.\n" +
        "Category: sorts by the displayed category label.\n" +
        "Summary Value: sorts by the summarized numeric value.",
    );

    await calculation.selectOption("count");
    await expect(calculationWarning).toBeHidden();
    await valueFieldSelect.selectOption(fieldKey);
    const barOptions = page.locator('[data-component-config-section="bar"]');
    await barOptions
      .getByLabel("Category field", { exact: true })
      .selectOption(fieldKey);
    await barOptions.getByLabel("Split bars", { exact: true }).check();
    await expect(visualSort.locator("option")).toHaveText([
      "Default",
      "Category",
      "Comparison",
      "Summary Value",
    ]);
    expect(await sortHelp.textContent()).toBe(
      "Default: uses the order produced by the current grouping and summarization.\n" +
        "Category: sorts by the displayed category label.\n" +
        "Comparison: sorts by the displayed comparison group label.\n" +
        "Summary Value: sorts by the summarized numeric value.",
    );
    await sortHelpTrigger.focus();
    await page.keyboard.press("Enter");
    await expect(sortHelp).toBeHidden();
    await expect(
      barOptions.getByLabel("Series field", { exact: true }),
    ).toBeVisible();
    await barOptions
      .getByLabel("Series field", { exact: true })
      .selectOption(fieldKey);
    const seriesLabels = page.getByRole("table", { name: "Series Labels" });
    await expect(seriesLabels).toBeVisible();
    const firstSeriesLabel = seriesLabels
      .locator("[data-category-display-label]")
      .first();
    await firstSeriesLabel.fill("Custom series label");
    await barOptions.getByLabel("Split bars", { exact: true }).uncheck();
    const categoryOverrideTable = page.getByRole("table", {
      name: "Category Labels",
    });
    await expect(categoryOverrideTable).toBeVisible();
    await expect(
      categoryOverrideTable.locator("[data-category-display-label]").first(),
    ).not.toHaveValue("Custom series label");
    await barOptions.getByLabel("Split bars", { exact: true }).check();
    await barOptions
      .getByLabel("Series field", { exact: true })
      .selectOption(fieldKey);
    await expect(
      page.getByRole("table", { name: "Series Labels" }),
    ).toBeVisible();
    await expect(
      barOptions.getByLabel("Missing categories", { exact: true }),
    ).toBeVisible();
    await expect(
      barOptions.getByLabel("Missing series", { exact: true }),
    ).toBeVisible();
    await expect(
      page.getByLabel("Missing values", { exact: true }),
    ).toBeVisible();
    await expect(
      barOptions.getByLabel("Comparison Layout", { exact: true }),
    ).toBeVisible();
    await barOptions
      .getByLabel("Category axis title", { exact: true })
      .fill("Category");
    await barOptions
      .getByLabel("Value axis title", { exact: true })
      .fill("Responses");
    await expect(page.locator(".component-editor-preview__badge")).toHaveText(
      "Valid config",
    );
    await expect(page.locator(".component-editor-preview svg")).toBeVisible();

    await calculation.selectOption("sum");
    await expect(page.locator(".component-editor-preview__badge")).toHaveText(
      "Needs attention",
    );
    await page.getByRole("button", { name: "Save Draft", exact: true }).click();
    const validationFindings = page.getByRole("region", {
      name: "Validation Findings",
    });
    await expect(validationFindings).toBeVisible();
    await expect(validationFindings).toHaveAttribute("aria-live", "polite");
    await expect(
      page.getByRole("textbox", { name: "Name", exact: true }),
    ).toHaveValue(unsavedName);
    await expect(page).toHaveURL(/\/components\/new$/);
    await calculation.selectOption("count");
    await expect(page.locator(".component-editor-preview__badge")).toHaveText(
      "Valid config",
    );

    const canonicalSave = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/admin/components/save") &&
        response.request().method() === "POST" &&
        response.ok(),
    );
    await page.getByRole("button", { name: "Save Draft", exact: true }).click();
    await canonicalSave;
    await expect(
      page.getByRole("heading", { level: 1, name: unsavedName }),
    ).toBeVisible();
    await page.getByRole("link", { name: "Edit" }).click();
    await expect(
      page.getByRole("heading", { level: 1, name: "Edit Component" }),
    ).toBeVisible();

    await selectComponentKind(page, "Donut");
    await expect(page.locator("[data-component-kind-editor]")).toBeFocused();
    await expect(
      page.getByText("A donut chart is a pie chart with a hole in the center."),
    ).toBeVisible();
    const pieOptions = page.locator('[data-component-config-section="pie"]');
    await page
      .getByLabel("Value field", { exact: true })
      .selectOption(fieldKey);
    await pieOptions
      .getByLabel("Category Field", { exact: true })
      .selectOption(fieldKey);
    await expect(pieOptions.getByLabel("Limit", { exact: true })).toBeVisible();
    await expect(
      pieOptions.getByLabel("Legend Title", { exact: true }),
    ).toBeVisible();
    const categoryLabels = page.getByRole("table", { name: "Category Labels" });
    await expect(categoryLabels).toBeVisible();
    await expect(categoryLabels).not.toContainText(
      "Choose a category field to load values.",
    );
    await expect(categoryLabels.locator("tbody tr")).not.toHaveCount(0);

    await selectComponentKind(page, "Pie");
    await expect(pieOptions.getByLabel("Limit", { exact: true })).toBeVisible();
    await selectComponentKind(page, "Line");
    const lineOptions = page.locator('[data-component-config-section="line"]');
    await expect(
      lineOptions.getByLabel("Category Field", { exact: true }),
    ).toBeVisible();
    await expect(
      lineOptions.getByLabel("Smoothing", { exact: true }),
    ).toBeVisible();
    await expect(
      lineOptions.getByLabel("Category axis title", { exact: true }),
    ).toBeVisible();
    await expect(
      lineOptions.getByLabel("Value axis title", { exact: true }),
    ).toBeVisible();
    await expect(
      lineOptions.getByLabel("Limit", { exact: true }),
    ).toBeVisible();
    await selectComponentKind(page, "Stat Card");
    const statOptions = page.locator(
      '[data-component-config-section="stat_card"]',
    );
    await expect(
      statOptions.getByLabel("Panel Style", { exact: true }),
    ).toBeVisible();
    await expect(
      statOptions.getByLabel("Label", { exact: true }),
    ).toBeVisible();
    await expect(
      statOptions.getByLabel("Supporting Text", { exact: true }),
    ).toBeVisible();
    await selectComponentKind(page, "Table");
    await expect(
      page.getByRole("group", { name: "Displayed Fields" }),
    ).toBeVisible();

    for (const kind of ["bar", "line", "pie", "donut", "stat_card"]) {
      const config = visualConfig(kind, fieldKey);
      if (kind === "bar") {
        Object.assign(config, {
          mode: "comparison",
          comparison_field: fieldKey,
          comparison_layout: "grouped",
          legend_title: field.label,
        });
      }
      const validation = await expectJson<ValidationResponse>(
        await page.request.post("/api/admin/components/validate", {
          data: versionInput(dataset, kind, config, `${kind} validation`),
        }),
      );
      expect(validation.valid, JSON.stringify(validation.findings)).toBe(true);

      const preview = await expectJson<{
        component_type: string;
        materialization_state: string;
      }>(
        await page.request.post("/api/admin/components/preview", {
          data: versionInput(dataset, kind, config, `${kind} preview`),
        }),
      );
      expect(preview.component_type).toBe(kind);
      expect(["ready", "pending"]).toContain(preview.materialization_state);

      const definition = await createComponent(
        page,
        dataset,
        kind,
        config,
        kind,
      );
      await publish(page, definition, `publish-${kind}`);
      const rendered = await expectJson<{
        component_type: string;
        materialization_state: string;
      }>(await page.request.get(renderPath(definition.slug, kind)));
      expect(rendered.component_type).toBe(kind);
      expect(rendered.materialization_state).toBe("ready");

      await page.goto(`/components/${definition.slug}/view`);
      await expect(
        page.getByRole("heading", { level: 1, name: definition.name }),
      ).toBeVisible();
      await expect(page.locator(".component-visual-preview")).toBeVisible();
      if (kind === "stat_card") {
        await expect(page.locator(".component-stat-card")).toBeVisible();
      } else {
        await expect(page.locator(".component-d3-svg")).toBeVisible({
          timeout: 15_000,
        });
        if (kind === "bar") {
          await expect(
            page.locator(
              ".component-d3-chart__surface > .component-d3-chart__legend + svg.component-d3-svg--bar",
            ),
          ).toBeVisible();
          const mark = page.locator("svg.component-d3-svg--bar rect").first();
          await mark.focus();
          await expect(page.locator(".component-d3-tooltip")).toBeVisible();
        }
        if (kind === "line") {
          await expect(
            page.locator("svg.component-d3-svg--line"),
          ).toContainText("Category");
          await expect(
            page.locator("svg.component-d3-svg--line"),
          ).toContainText("Responses");
          await expect(
            page.locator(".component-d3-chart__category-label").first(),
          ).toBeVisible();
        }
      }
    }

    const distinctValues = await expectJson<{ values: unknown[] }>(
      await page.request.post(
        "/api/admin/components/datasets/distinct-values",
        {
          data: {
            schema_version: 1,
            action: "distinct_values",
            reference: dataset.reference,
            field_key: fieldKey,
            limit: 12,
          },
        },
      ),
    );
    expect(distinctValues.values.length).toBeGreaterThan(0);
    const rawCategory = distinctValues.values
      .map(String)
      .find((value) => value.length > 0);
    expect(
      rawCategory,
      "distinct category fixture should expose a non-empty value",
    ).toBeTruthy();
    const semanticWarning = "var(--semantic-warning)";
    const noOpConfig: Record<string, unknown> = {
      summary_field: fieldKey,
      summary_type: "count",
      value_format: "integer",
      missing_policy: "omit",
      value_missing_policy: "omit",
      sort_direction: "asc",
      filters: [],
      category_field: fieldKey,
      category_missing_policy: "omit",
      max_slices: 20,
      category_labels: {},
      category_colors: { [rawCategory!]: semanticWarning },
      legend_title: null,
    };
    const roundTrip = await createComponent(
      page,
      dataset,
      "donut",
      noOpConfig,
      "semantic-color-round-trip",
    );
    const storedConfig = roundTrip.versions[0].config;
    expect(storedConfig).not.toHaveProperty("sort_field");
    expect(storedConfig.category_colors).toEqual({
      [rawCategory!]: semanticWarning,
    });

    await page.goto(`/components/${roundTrip.slug}/edit`);
    const noOpEditor = page.locator('[data-component-config-section="visual"]');
    await expect(
      noOpEditor.getByLabel("Sort Field", { exact: true }),
    ).toHaveValue("");
    const noOpOverrides = page.getByRole("table", { name: "Category Labels" });
    const originalValue = noOpOverrides.getByRole("rowheader", {
      name: rawCategory!,
      exact: true,
    });
    await expect(originalValue).toBeVisible();
    await expect(originalValue).toHaveAttribute("scope", "row");
    const noOpRow = originalValue.locator("..");
    const labelOverride = noOpRow.getByLabel(
      `Display label for ${rawCategory!}`,
      {
        exact: true,
      },
    );
    await expect(labelOverride).toHaveValue("");
    await expect(labelOverride).toHaveAttribute("placeholder", rawCategory!);
    const colorOverride = noOpRow.getByLabel(`Color for ${rawCategory!}`, {
      exact: true,
    });
    await expect(colorOverride).toHaveValue(semanticWarning);
    await expect(colorOverride.locator('option[value=""]')).toHaveText(
      "Default",
    );
    await expect(
      colorOverride.locator(`option[value="${semanticWarning}"]`),
    ).toHaveText("Warning");

    const noOpSave = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/admin/components/save") &&
        response.request().method() === "POST" &&
        response.ok(),
    );
    await page.getByRole("button", { name: "Save Draft", exact: true }).click();
    await noOpSave;
    const roundTripped = await expectJson<ComponentDefinition>(
      await page.request.get(`/api/admin/components/${roundTrip.slug}`),
    );
    const roundTrippedVersion = roundTripped.versions.find(
      (version) =>
        version.component_version_id ===
        roundTrip.versions[0].component_version_id,
    );
    expect(roundTrippedVersion).toBeDefined();
    expect(roundTrippedVersion!.config).toEqual(storedConfig);
    expect(roundTrippedVersion!.config).not.toHaveProperty("sort_field");
    expect(roundTrippedVersion!.config.category_colors).toEqual({
      [rawCategory!]: semanticWarning,
    });
    assertNoConsoleErrors();
  });

  test("component editor remains contained and uses an accessible mobile preview dialog", async ({
    page,
  }) => {
    test.setTimeout(60_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const dataset = await datasetOption(page);
    const fieldKey = dataset.fields[0].key;
    const definition = await createComponent(
      page,
      dataset,
      "line",
      visualConfig("line", fieldKey),
      "mobile-line",
    );
    await publish(page, definition, "publish-mobile-line");

    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto(`/components/${definition.slug}/edit`);
    const kindPanel = page.getByRole("group", { name: "Component Kind" });
    const filtersPanel = page.getByRole("group", { name: "Filters" });
    await expect(kindPanel).toBeVisible();
    await expect(filtersPanel).toBeVisible();
    await expect
      .poll(async () => {
        const [kindBox, filtersBox] = await Promise.all([
          kindPanel.boundingBox(),
          filtersPanel.boundingBox(),
        ]);
        return (
          kindBox !== null && filtersBox !== null && kindBox.y < filtersBox.y
        );
      })
      .toBe(true);

    const hasHorizontalOverflow = await page.evaluate(
      () =>
        document.documentElement.scrollWidth >
        document.documentElement.clientWidth,
    );
    expect(hasHorizontalOverflow).toBe(false);

    const previewButton = page.getByRole("button", { name: "Open preview" });
    await expect(previewButton).toBeVisible();
    await previewButton.click();
    const previewDialog = page.getByRole("dialog", {
      name: "Component preview",
    });
    await expect(previewDialog).toBeVisible();
    await expect(previewDialog).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(previewDialog).not.toBeVisible();
    await expect(previewButton).toBeFocused();

    await page.reload();
    await expect(
      page.getByRole("heading", { level: 1, name: "Edit Component" }),
    ).toBeVisible();
    await expect(
      page.getByRole("group", { name: "Component Kind" }),
    ).toBeVisible();
    assertNoConsoleErrors();
  });
});

test.describe("Sprint 8A extracted Component UI parity", () => {
  // Playwright controls the post-zoom CSS viewport and output scale separately.
  // A 1280 x 900 browser viewport at 200% is 640 x 450 CSS pixels at 2x output.
  test.use({
    viewport: TWO_HUNDRED_PERCENT_ZOOM_VIEWPORT,
    deviceScaleFactor: 2,
    colorScheme: "light",
  });

  test("Component and Dashboard visuals stay contained at 200% zoom in light and dark themes", async ({
    page,
  }) => {
    test.setTimeout(90_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);

    const components = await expectJson<ComponentSummary[]>(
      await page.request.get("/api/components"),
    );
    const component = components.find(
      (candidate) =>
        candidate.current_version.publication_state === "published" &&
        candidate.current_version.lifecycle_state === "active" &&
        candidate.current_version.component_type !== "table",
    );
    expect(
      component,
      "demo seed should expose an active published visual Component",
    ).toBeTruthy();

    const dashboards = await expectJson<DashboardSummary[]>(
      await page.request.get("/api/dashboards"),
    );
    const dashboard = dashboards
      .filter((candidate) => candidate.placement_count > 0)
      .sort((left, right) => right.placement_count - left.placement_count)[0];
    expect(
      dashboard,
      "demo seed should expose a Dashboard with Component placements",
    ).toBeTruthy();

    for (const theme of ["light", "dark"] as const) {
      await page.goto(`/components/${component!.slug}/view`);
      await expect(page.locator("#module-content")).toHaveAttribute(
        "data-hydration",
        "ready",
      );
      await chooseThemeWithKeyboard(page, theme);
      await expect(
        page.getByRole("heading", { level: 1, name: component!.name }),
      ).toBeVisible();
      await expect(page.locator(".component-visual-preview")).toBeVisible();
      await expect(
        page
          .locator(".component-visual-preview")
          .locator(RENDERED_COMPONENT_CONTENT)
          .first(),
      ).toBeVisible({ timeout: 15_000 });
      await expectHorizontalContainment(
        page,
        ".component-visual-preview",
        `${theme} Component visual at 200% zoom`,
      );

      await page.goto(`/dashboards/${dashboard!.id}/view`);
      await expect(page.locator("#module-content")).toHaveAttribute(
        "data-hydration",
        "ready",
      );
      await expect(page.locator("html")).toHaveAttribute(
        "data-theme-preference",
        theme,
      );
      await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
      await expect(
        page.getByRole("heading", { level: 1, name: dashboard!.name }),
      ).toBeVisible();
      await expect(page.locator(".dashboard-viewer-placement")).toHaveCount(
        dashboard!.placement_count,
      );
      await page
        .locator(".dashboard-viewer-placement")
        .first()
        .scrollIntoViewIfNeeded();
      const renderedPlacement = page
        .locator(".dashboard-viewer-placement")
        .locator(RENDERED_COMPONENT_CONTENT)
        .first();
      await expect(renderedPlacement).toBeVisible({ timeout: 15_000 });
      await expectHorizontalContainment(
        page,
        ".dashboard-viewer-placement",
        `${theme} Dashboard placements at 200% zoom`,
      );
    }

    assertNoConsoleErrors();
  });
});
