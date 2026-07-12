import { expect, test, type APIResponse, type Page } from "@playwright/test";
import { execFileSync } from "node:child_process";
import { resolve } from "node:path";

const BENIGN_NAVIGATION_ABORT_ERRORS = [
  "WebAssembly compilation aborted: Network error: Response body loading was aborted",
  "Failed to fetch",
  "Failed to load resource: the server responded with a status of 404 (Not Found)",
];

const COMPONENT_PREFIX = "pw-components-";
const RUN_ID = Date.now();

type IdResponse = { id: string };

type DatasetFieldDefinition = {
  key: string;
  label: string;
  field_type: string;
};

type DatasetSummary = {
  id: string;
  name: string;
  slug?: string;
  grain?: string;
  tags?: string[];
  provenance?: {
    forms?: Array<{ id: string; name: string; slug?: string | null }>;
    datasets?: Array<{ id: string; name: string; slug?: string | null }>;
  };
  visibility_nodes: Array<{ node_id: string; node_name: string }>;
  current_version_major?: number | null;
  major_versions?: number[];
  output_fields: DatasetFieldDefinition[];
};

type ComponentVersionSummary = {
  id: string;
  component_id: string;
  dataset_id: string;
  dataset_version_major: number;
  binding_mode: string;
  component_type: string;
  status: string;
  version_label: string;
  config: Record<string, unknown>;
};

type ComponentDefinition = {
  id: string;
  name: string;
  slug: string;
  description?: string | null;
  versions: ComponentVersionSummary[];
};

type ComponentSummary = {
  slug: string;
  current_component_type?: string | null;
};

type ComponentTable = {
  component_version_id: string;
  materialization_state: string;
  component_type: string;
  columns: Array<{ key: string; label: string; field_type: string }>;
  rows: Array<{ values: Record<string, string | null> }>;
  pagination: {
    page_size: number;
    next_cursor?: string | null;
    has_more: boolean;
  };
};

type ComponentVisual = {
  component_version_id: string;
  materialization_state: string;
  component_type: string;
  bar_orientation?: string | null;
  bar_comparison_layout?: string | null;
  x_axis_label?: string | null;
  y_axis_label?: string | null;
  stat?: { label: string; display_value?: string | null } | null;
  points: Array<{ x: string; value: number; display_value: string }>;
  slices: Array<{ category: string; value: number; display_value: string }>;
};

type ComponentValidationResponse = {
  valid: boolean;
  findings: Array<{
    code: string;
    severity: string;
    field_path?: string | null;
    message: string;
  }>;
};

type ApiErrorBody = {
  code: string;
  message: string;
  error: string;
};

type ComponentFilterConfig = {
  field_key: string;
  operator: string;
  value?: string;
};

type DashboardSummary = {
  id: string;
  component_count: number;
};

type DashboardResponse = {
  id: string;
  components: Array<{ component_version_id: string }>;
};

function isBenignNavigationAbort(message: string) {
  return BENIGN_NAVIGATION_ABORT_ERRORS.some((pattern) =>
    message.includes(pattern),
  );
}

function attachConsoleGuard(page: Page) {
  const errors: string[] = [];
  const httpErrors: string[] = [];
  page.on("response", (response) => {
    if (response.status() >= 400) {
      httpErrors.push(`${response.status()} ${response.url()}`);
    }
  });
  page.on("console", (message) => {
    if (message.type() === "error") {
      const text = message.text();
      if (!isBenignNavigationAbort(text)) {
        errors.push(text);
      }
    }
  });
  page.on("pageerror", (error) => {
    if (!isBenignNavigationAbort(error.message)) {
      errors.push(error.message);
    }
  });
  const assertClean = async () => {
    expect(
      errors,
      `browser console should stay clean: ${errors.join("\n")}\nPage HTTP errors: ${httpErrors.join("\n")}`,
    ).toEqual([]);
  };
  assertClean.reset = () => {
    errors.splice(0, errors.length);
    httpErrors.splice(0, httpErrors.length);
  };
  return assertClean;
}

async function signInAsAdmin(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: {
      email: "admin@tessara.local",
      password: "tessara-dev-admin",
    },
  });
  expect(response.ok()).toBeTruthy();
}

async function expectJson<T>(response: APIResponse) {
  const text = await response.text();
  expect(
    response.ok(),
    `${response.url()} returned ${response.status()}: ${text}`,
  ).toBeTruthy();
  return JSON.parse(text) as T;
}

async function expectStatus(response: APIResponse, expectedStatus: number) {
  const text = await response.text();
  expect(
    response.status(),
    `${response.url()} returned ${response.status()}: ${text}`,
  ).toBe(expectedStatus);
  return text;
}

async function ensureDemoSeed(page: Page) {
  const response = await page.request.post("/api/demo/seed", { data: {} });
  const text = await response.text();
  if (
    response.ok() ||
    (response.status() === 400 &&
      text.includes("Demo seed requires an empty database"))
  ) {
    return;
  }
  expect(response.ok(), `${response.url()} returned ${response.status()}: ${text}`).toBeTruthy();
}

async function pickDatasetMajor(page: Page) {
  const datasets = await expectJson<DatasetSummary[]>(
    await page.request.get("/api/datasets"),
  );
  const dataset = datasets.find(
    (candidate) =>
      candidate.output_fields.some((field) => isTextLikeField(field)) &&
      (candidate.major_versions?.length || candidate.current_version_major),
  );
  expect(dataset, "a published dataset with a text-like output field should exist").toBeTruthy();
  const major =
    dataset!.major_versions?.[0] ?? dataset!.current_version_major ?? undefined;
  expect(major, "dataset should expose a major version").toBeTruthy();
  return { dataset: dataset!, major: major! };
}

async function pickDemoSessionLogDataset(page: Page) {
  const datasets = await expectJson<DatasetSummary[]>(
    await page.request.get("/api/datasets"),
  );
  const dataset = datasets.find(
    (candidate) =>
      candidate.slug === "demo-session-log" ||
      candidate.name === "Demo Session Log Dataset",
  );
  expect(dataset, "Demo Session Log Dataset should exist").toBeTruthy();
  const major =
    dataset!.major_versions?.[0] ?? dataset!.current_version_major ?? undefined;
  expect(major, "Demo Session Log Dataset should expose a major version").toBeTruthy();
  return { dataset: dataset!, major: major! };
}

async function selectDatasetVersion(
  page: Page,
  dataset: DatasetSummary,
  major: number,
) {
  const picker = page.getByRole("combobox", { name: "Dataset Version" });
  await expect(async () => {
    await picker.click();
    await expect(picker).toHaveAttribute("aria-expanded", "true", {
      timeout: 1_000,
    });
  }).toPass({ timeout: 10_000 });
  const filter = page.getByRole("searchbox", { name: "Filter dataset versions" });
  await expect(filter).toBeVisible();
  await filter.fill(dataset.name);
  const row = page
    .getByRole("option")
    .filter({ hasText: dataset.name })
    .filter({ hasText: `v${major}` });
  await expect(row).toHaveCount(1);
  await row.getByRole("button", { name: dataset.name }).click();
  await expect(picker).toContainText(`${dataset.name} · v${major}`);
}

function isTextLikeField(field: DatasetFieldDefinition) {
  return field.field_type === "text" || field.field_type === "static_text";
}

function textLikeField(fields: DatasetFieldDefinition[]) {
  const field = fields.find((candidate) => isTextLikeField(candidate));
  expect(field, "dataset should expose a text-like output field").toBeTruthy();
  return field!;
}

function tableConfig(
  fieldKeys: string[],
  pageSize = 25,
  filters: ComponentFilterConfig[] = [],
) {
  return {
    visible_columns: fieldKeys,
    filters,
    default_sort: fieldKeys[0]
      ? {
          field_key: fieldKeys[0],
          direction: "asc",
        }
      : null,
    page_size: pageSize,
  };
}

function visualConfig(kind: string, fieldKey: string) {
  if (kind === "bar") {
    return {
      mode: "summary",
      summary_field: fieldKey,
      summary_type: "count",
      category_field: fieldKey,
      orientation: "horizontal",
      sort_field: "summary_value",
      sort_direction: "desc",
      number_of_points: 20,
      value_format: "integer",
      x_axis_label: "Submissions",
      y_axis_label: "Category",
    };
  }
  if (kind === "line") {
    return {
      summary_field: fieldKey,
      summary_type: "count",
      x_field: fieldKey,
      number_of_points: 20,
    };
  }
  if (kind === "pie" || kind === "donut") {
    return {
      summary_field: fieldKey,
      summary_type: "count",
      category_field: fieldKey,
      max_slices: 20,
    };
  }
  return {
    summary_field: fieldKey,
    summary_type: "count",
    label: "Submission count",
    value_format: "integer",
    panel_style: "accent",
  };
}

function visualPath(kind: string) {
  return kind === "stat_card" ? "stat-card" : kind;
}

async function createComponentDraft(
  page: Page,
  name: string,
  slug: string,
  dataset: DatasetSummary,
  major: number,
  fieldKeys: string[],
  filters: ComponentFilterConfig[] = [],
) {
  return expectJson<IdResponse>(
    await page.request.post("/api/admin/components", {
      data: {
        name,
        slug,
        description: "Playwright Sprint 4A table component workflow fixture.",
        version: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "table",
          config: tableConfig(fieldKeys, 25, filters),
        },
      },
    }),
  );
}

async function createVisualComponentDraft(
  page: Page,
  name: string,
  slug: string,
  dataset: DatasetSummary,
  major: number,
  kind: string,
  fieldKey: string,
) {
  return expectJson<IdResponse>(
    await page.request.post("/api/admin/components", {
      data: {
        name,
        slug,
        description: "Playwright Sprint 4B visual component workflow fixture.",
        version: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: kind,
          config: visualConfig(kind, fieldKey),
        },
      },
    }),
  );
}

async function saveVisualDraft(
  page: Page,
  componentId: string,
  dataset: DatasetSummary,
  major: number,
  kind: string,
  fieldKey: string,
) {
  return expectJson<IdResponse>(
    await page.request.post(`/api/admin/components/${componentId}/versions`, {
      data: {
        dataset_id: dataset.id,
        dataset_version_major: major,
        component_type: kind,
        config: visualConfig(kind, fieldKey),
        version_note: "Playwright visual replacement version.",
      },
    }),
  );
}

async function createAndPublishVisualComponentThroughUi(
  page: Page,
  dataset: DatasetSummary,
  major: number,
  kind: string,
  fieldKey: string,
) {
  const slug = `${COMPONENT_PREFIX}${RUN_ID}-ui-${kind}`;
  const name = `Playwright UI Visual ${kind} ${RUN_ID}`;

  await page.goto("/components/new");
  await page.waitForLoadState("networkidle");
  await expect(page.getByRole("button", { name: "Save Draft" })).toBeVisible();
  const nameInput = page.getByRole("textbox", { name: "Name", exact: true });
  const slugInput = page.getByRole("textbox", { name: "Slug" });
  await nameInput.fill(name);
  await nameInput.blur();
  await slugInput.fill(slug);
  await page
    .getByRole("textbox", { name: "Description" })
    .fill(`UI-created ${kind} visual component.`);
  await expect(page.getByRole("textbox", { name: "Name", exact: true })).toHaveValue(name);
  await expect(slugInput).toHaveValue(slug);
  await selectDatasetVersion(page, dataset, major);
  const kindLabel = kind === "stat_card" ? "Stat Card" : `${kind[0].toUpperCase()}${kind.slice(1)}`;
  await page.getByRole("radio", { name: kindLabel, exact: true }).click();
  await expect(page.getByRole("textbox", { name: "Name", exact: true })).toHaveValue(name);
  await expect(slugInput).toHaveValue(slug);
  if (kind === "bar") {
    await page.locator(".component-editor__role-card--measure select").first().selectOption("count");
    await page
      .locator(".component-editor__role-card--measure .component-editor__measure-grid label.form-field")
      .nth(1)
      .locator("select")
      .selectOption(fieldKey);
    await page.getByLabel("Value format").selectOption("integer");
    await page.getByLabel("Missing categories").selectOption("explicit_missing");
    await page.getByLabel("Missing values").selectOption("zero");
  } else {
    await page.locator(".component-editor__value-field select").selectOption(fieldKey);
    await page.getByLabel("Calculation", { exact: true }).selectOption("count");
    await page.getByLabel("Format", { exact: true }).selectOption("integer");
    await page.getByLabel("Missing measure values", { exact: true }).selectOption("omit");
  }

  if (kind === "bar") {
    await page
      .locator(".component-editor__role-grid > .component-editor__role-card")
      .first()
      .locator("select")
      .first()
      .selectOption(fieldKey);
  } else if (kind === "pie" || kind === "donut") {
    await page.locator(".component-editor__category-field select").selectOption(fieldKey);
  } else if (kind === "line") {
    await page.locator(".component-editor__category-field select").selectOption(fieldKey);
  } else {
    await page.getByLabel("Label", { exact: true }).fill("Submission count");
    await page.getByLabel("Panel Style", { exact: true }).selectOption("accent");
  }

  await page.locator(".component-editor__publish-button").click();
  await page.getByRole("menuitem", { name: "Create New Version" }).click();
  const consumerDialog = page.getByRole("dialog", { name: "Review component consumers" });
  await expect(consumerDialog).toBeVisible();
  await consumerDialog
    .getByLabel("New Version Note")
    .fill(`Initial UI publish for ${kind}.`);
  const saveResponsePromise = page.waitForResponse(
      (response) =>
        response.url().endsWith("/api/admin/components/save") &&
        response.request().method() === "POST",
      { timeout: 15_000 },
    );
  await consumerDialog.getByRole("button", { name: "Create New Version" }).click();
  const saveResponse = await saveResponsePromise.catch(async (error) => {
    const visibleErrors = await page.locator(".form-status.is-error").allTextContents();
    throw new Error(`${String(error)}\nVisible form errors: ${visibleErrors.join(" | ")}`);
  });
  expect(saveResponse.status(), `${saveResponse.url()} returned ${saveResponse.status()}`).toBe(
    200,
  );
  await page.waitForURL(`**/components/${slug}`);
  await page.goto(`/components/${slug}/view`);
  await expect(page.getByRole("heading", { level: 1, name })).toBeVisible();
  await expect(page.locator(".component-visual-preview")).toBeVisible();

  return { name, slug };
}

async function saveTableDraft(
  page: Page,
  componentId: string,
  dataset: DatasetSummary,
  major: number,
  fieldKeys: string[],
) {
  return expectJson<IdResponse>(
    await page.request.post(`/api/admin/components/${componentId}/versions`, {
      data: {
        dataset_id: dataset.id,
        dataset_version_major: major,
        component_type: "table",
        config: tableConfig(fieldKeys),
      },
    }),
  );
}

async function patchTableDraft(
  page: Page,
  componentId: string,
  versionId: string,
  dataset: DatasetSummary,
  major: number,
  fieldKeys: string[],
) {
  return expectJson<IdResponse>(
    await page.request.patch(
      `/api/admin/components/${componentId}/versions/${versionId}`,
      {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "table",
          config: tableConfig(fieldKeys),
        },
      },
    ),
  );
}

async function loadComponent(page: Page, slug: string) {
  return expectJson<ComponentDefinition>(
    await page.request.get(`/api/admin/components/${slug}`),
  );
}

async function publishComponentVersion(
  page: Page,
  componentId: string,
  versionId: string,
) {
  return expectJson<IdResponse>(
    await page.request.post(
      `/api/admin/components/${componentId}/versions/${versionId}/publish`,
      { data: {} },
    ),
  );
}

function cleanupPlaywrightComponents() {
  const sql = `
CREATE TEMP TABLE pw_cleanup_components AS
SELECT id FROM components
WHERE slug LIKE '${COMPONENT_PREFIX}%'
   OR name LIKE 'Playwright Component Workflow %';

DELETE FROM dashboard_components
WHERE component_version_id IN (
  SELECT id FROM component_versions
  WHERE component_id IN (SELECT id FROM pw_cleanup_components)
);

DELETE FROM dashboards
WHERE name LIKE 'Playwright Component Workflow Dashboard %';

DELETE FROM component_versions
WHERE component_id IN (SELECT id FROM pw_cleanup_components);

DELETE FROM components
WHERE id IN (SELECT id FROM pw_cleanup_components);
`;

  try {
    execFileSync(
      "docker",
      [
        "compose",
        "exec",
        "-T",
        "postgres",
        "psql",
        "-v",
        "ON_ERROR_STOP=1",
        "-U",
        "tessara",
        "-d",
        "tessara",
      ],
      {
        cwd: resolve(process.cwd(), ".."),
        input: sql,
        stdio: ["pipe", "pipe", "pipe"],
      },
    );
  } catch (error) {
    console.warn(`component cleanup skipped: ${String(error)}`);
  }
}

function runComponentSql(sql: string) {
  execFileSync(
    "docker",
    [
      "compose",
      "exec",
      "-T",
      "postgres",
      "psql",
      "-v",
      "ON_ERROR_STOP=1",
      "-U",
      "tessara",
      "-d",
      "tessara",
    ],
    {
      cwd: resolve(process.cwd(), ".."),
      input: sql,
      stdio: ["pipe", "pipe", "pipe"],
    },
  );
}

test.describe.serial("Sprint 4A component workflow", () => {
  test.afterAll(() => cleanupPlaywrightComponents());

  test("admin can create, update, publish, and view a major-line table component", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    cleanupPlaywrightComponents();

    const { dataset, major } = await pickDatasetMajor(page);
    const firstField = textLikeField(dataset.output_fields);
    const slug = `${COMPONENT_PREFIX}${RUN_ID}`;
    const name = `Playwright Component Workflow ${RUN_ID}`;

    await page.goto("/components/new");
    await selectDatasetVersion(page, dataset, major);
    await expect(page.getByRole("group", { name: "Dataset Context" })).toHaveCount(0);
    const displayedFields = page.getByRole("group", { name: "Displayed Fields" });
    await expect(displayedFields).toBeVisible();
    const availableFields = displayedFields.getByRole("listbox", { name: "Available fields" });
    await expect(availableFields.locator(".dataset-projection-builder__option")).not.toHaveCount(0);
    const invalidValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "table",
          config: tableConfig([`missing_${RUN_ID}`]),
        },
      }),
    );
    expect(invalidValidation.valid).toBe(false);
    expect(invalidValidation.findings[0]).toMatchObject({
      code: "COMPONENT_FIELD_NOT_IN_MAJOR_LINE",
      field_path: "config",
    });

    const created = await createComponentDraft(
      page,
      name,
      slug,
      dataset,
      major,
      [firstField.key],
    );
    let component = await loadComponent(page, slug);
    expect(component.versions).toHaveLength(1);
    expect(component.versions[0]).toMatchObject({
      dataset_id: dataset.id,
      dataset_version_major: major,
      binding_mode: "major_line",
      component_type: "table",
      status: "draft",
    });
    expect(component.versions[0]).not.toHaveProperty("dataset_revision_id");

    const readerComponentsBeforePublish = await expectJson<Array<{ slug: string }>>(
      await page.request.get("/api/components"),
    );
    expect(readerComponentsBeforePublish.some((item) => item.slug === slug)).toBe(false);

    await page.goto(`/components/${slug}`);
    await expect(page.getByRole("heading", { level: 1, name })).toBeVisible();
    await expect(page.getByRole("heading", { name: "No published version" })).toBeVisible();
    await expect(page.getByText("Component unavailable")).toHaveCount(0);

    const draftDashboard = await expectJson<IdResponse>(
      await page.request.post("/api/admin/dashboards", {
        data: {
          name: `Playwright Component Workflow Dashboard ${RUN_ID}`,
          description: "Dashboard placement guard fixture.",
          visibility_node_ids: dataset.visibility_nodes.map((node) => node.node_id),
        },
      }),
    );
    const draftDashboardPlacement = await expectStatus(
      await page.request.post(`/api/admin/dashboards/${draftDashboard.id}/components`, {
        data: {
          component_version_id: component.versions[0].id,
          position: 1,
          config: {},
        },
      }),
      400,
    );
    const draftDashboardPlacementBody = JSON.parse(draftDashboardPlacement) as ApiErrorBody;
    expect(draftDashboardPlacementBody).toMatchObject({
      code: "bad_request",
      error: expect.stringContaining("draft"),
    });
    runComponentSql(`
INSERT INTO dashboard_components (dashboard_id, component_version_id, position, config)
VALUES ('${draftDashboard.id}', '${component.versions[0].id}', 99, '{}'::jsonb);
`);
    const dashboardsWithLegacyDraftPlacement = await expectJson<DashboardSummary[]>(
      await page.request.get("/api/dashboards"),
    );
    expect(
      dashboardsWithLegacyDraftPlacement.find((dashboard) => dashboard.id === draftDashboard.id)
        ?.component_count,
    ).toBe(0);
    const dashboardWithLegacyDraftPlacement = await expectJson<DashboardResponse>(
      await page.request.get(`/api/dashboards/${draftDashboard.id}`),
    );
    expect(dashboardWithLegacyDraftPlacement.components).toEqual([]);

    const validValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "table",
          config: tableConfig([firstField.key]),
        },
      }),
    );
    expect(validValidation.valid).toBe(true);
    expect(validValidation.findings).toEqual([]);

    const unsupportedKindValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "aggregate_table",
          config: tableConfig([firstField.key]),
        },
      }),
    );
    expect(unsupportedKindValidation.valid).toBe(false);
    expect(unsupportedKindValidation.findings[0]).toMatchObject({
      code: "COMPONENT_UNSUPPORTED_KIND",
      severity: "error",
    });

    await publishComponentVersion(page, created.id, component.versions[0].id);
    component = await loadComponent(page, slug);
    expect(component.versions[0].status).toBe("published");
    const firstPublishedVersionId = component.versions[0].id;

    const table = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          page_size: "25",
          visible_columns: firstField.key,
          sort: `${firstField.key}:asc`,
        },
      }),
    );
    expect(table.materialization_state).toBe("ready");
    expect(table.component_version_id).toBe(firstPublishedVersionId);
    expect(table.component_type).toBe("table");
    expect(table.columns.map((column) => column.key)).toEqual([firstField.key]);
    expect(table.rows.length).toBeGreaterThan(0);

    const filterValue = table.rows
      .map((row) => row.values[firstField.key])
      .find((value): value is string => Boolean(value));
    expect(filterValue, `component table should include a value for ${firstField.key}`).toBeTruthy();
    const filteredTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: firstField.key,
          [`filter[${firstField.key}][operator]`]: "equals",
          [`filter[${firstField.key}][value]`]: filterValue!,
        },
      }),
    );
    expect(filteredTable.rows.length).toBeGreaterThan(0);
    expect(
      filteredTable.rows.every((row) => row.values[firstField.key] === filterValue),
    ).toBe(true);

    const savedFilterSlug = `${slug}_saved_filter`;
    const savedFilterCreated = await createComponentDraft(
      page,
      `${name} Saved Filter`,
      savedFilterSlug,
      dataset,
      major,
      [firstField.key],
      [
        {
          field_key: firstField.key,
          operator: "equals",
          value: filterValue!,
        },
      ],
    );
    const savedFilterComponent = await loadComponent(page, savedFilterSlug);
    await publishComponentVersion(
      page,
      savedFilterCreated.id,
      savedFilterComponent.versions[0].id,
    );
    const savedFilterTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${savedFilterSlug}/table`, {
        params: {
          visible_columns: firstField.key,
        },
      }),
    );
    expect(savedFilterTable.rows.length).toBeGreaterThan(0);
    expect(
      savedFilterTable.rows.every((row) => row.values[firstField.key] === filterValue),
    ).toBe(true);

    const searchTerm = filterValue!.slice(0, Math.min(4, filterValue!.length));
    const searchedTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          q: searchTerm,
          visible_columns: firstField.key,
        },
      }),
    );
    expect(searchedTable.rows.length).toBeGreaterThan(0);
    expect(
      searchedTable.rows.every((row) =>
        (row.values[firstField.key] ?? "")
          .toLowerCase()
          .includes(searchTerm.toLowerCase()),
      ),
    ).toBe(true);

    const pagedTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          page_size: "1",
          visible_columns: firstField.key,
          sort: `${firstField.key}:asc`,
        },
      }),
    );
    expect(pagedTable.pagination.page_size).toBe(1);
    expect(pagedTable.columns.map((column) => column.key)).toEqual([firstField.key]);
    expect(pagedTable.rows.length).toBeLessThanOrEqual(1);
    if (pagedTable.pagination.has_more) {
      expect(pagedTable.pagination.next_cursor).toMatch(/^offset:/);
    }

    const badVisibleColumns = await expectStatus(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: `${firstField.key},missing_${RUN_ID}`,
        },
      }),
      400,
    );
    const badVisibleColumnsBody = JSON.parse(badVisibleColumns) as ApiErrorBody;
    expect(badVisibleColumnsBody).toMatchObject({
      code: "bad_request",
      error: expect.stringContaining("visible column"),
    });

    await page.goto(`/components/${slug}`);
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Components");
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText(name);
    await expect(page.getByRole("heading", { level: 1, name })).toBeVisible();
    await expect(page.getByRole("searchbox", { name: "Search component rows" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Choose visible columns" })).toBeVisible();
    await expect(page.getByRole("table").filter({ hasText: firstField.label })).toBeVisible();

    const renamedName = `${name} Updated`;
    const renamedDescription = "Updated by the Sprint 4A component workflow.";
    await expectJson<IdResponse>(
      await page.request.patch(`/api/admin/components/${created.id}`, {
        data: {
          name: renamedName,
          slug,
          description: renamedDescription,
        },
      }),
    );
    component = await loadComponent(page, slug);
    expect(component.name).toBe(renamedName);
    expect(component.description).toBe(renamedDescription);

    const secondDraft = await saveTableDraft(
      page,
      created.id,
      dataset,
      major,
      [firstField.key],
    );
    const patchedDraft = await patchTableDraft(
      page,
      created.id,
      secondDraft.id,
      dataset,
      major,
      [firstField.key],
    );
    expect(patchedDraft.id).toBe(secondDraft.id);
    component = await loadComponent(page, slug);
    expect(component.versions.some((version) => version.status === "draft")).toBe(true);
    expect(component.versions.some((version) => version.status === "published")).toBe(true);

    const readerComponentWhileDraftExists = await expectJson<ComponentDefinition>(
      await page.request.get(`/api/components/${slug}`),
    );
    expect(
      readerComponentWhileDraftExists.versions.every(
        (version) => version.status === "published",
      ),
    ).toBe(true);
    const tableWhileDraftExists = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: firstField.key,
        },
      }),
    );
    expect(tableWhileDraftExists.component_version_id).toBe(firstPublishedVersionId);
    expect(tableWhileDraftExists.component_type).toBe("table");
    expect(tableWhileDraftExists.columns.map((column) => column.key)).toEqual([
      firstField.key,
    ]);

    await page.goto("/components");
    await expect(page.getByRole("link", { name: "Create Component" })).toBeVisible();
    await page.getByRole("searchbox", { name: "Search components by name" }).fill(renamedName);
    await expect(page.getByRole("link", { name: renamedName })).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit" }).first()).toHaveAttribute(
      "href",
      `/components/${slug}/edit`,
    );
    await expect(page.getByRole("link", { name: "Versions" }).first()).toHaveAttribute(
      "href",
      `/components/${slug}/versions`,
    );
    await page.getByRole("button", { name: "Filter Kind" }).click();
    await page.getByRole("menuitemradio", { name: "Table" }).click();
    await page.getByRole("button", { name: "Filter Status" }).click();
    await page.getByRole("menuitemradio", { name: "Updating" }).click();
    await expect(page.getByRole("link", { name: renamedName })).toBeVisible();
    await page.setViewportSize({ width: 544, height: 912 });
    await page.getByRole("button", { name: "Open component filters" }).click();
    await expect(page.getByRole("dialog", { name: "Component filters" })).toBeVisible();
    await page.getByLabel("Filter components by kind").selectOption("Table");
    await page.getByLabel("Filter components by status").selectOption("Published");
    await page.getByRole("button", { name: "Clear All" }).click();
    await expect(page.getByLabel("Filter components by kind")).toHaveValue("all");
    await expect(page.getByLabel("Filter components by status")).toHaveValue("all");
    await page.getByLabel("Filter components by kind").selectOption("Table");
    await page.getByLabel("Filter components by status").selectOption("Updating");
    await page.getByTitle("Close component filters").click();
    await expect(page.locator(".components-list-mobile-card").filter({ hasText: renamedName })).toBeVisible();
    await expect(page.locator(".components-list-responsive-table .table-wrap")).toBeHidden();
    await page.setViewportSize({ width: 1491, height: 912 });

    await page.goto(`/components/${slug}`);
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Components");
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText(renamedName);
    await expect(page.getByRole("heading", { level: 1, name: renamedName })).toBeVisible();
    await expect(page.getByRole("searchbox", { name: "Search component rows" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Reset table controls" })).toBeVisible();
    await expect(page.getByRole("table").filter({ hasText: firstField.label })).toBeVisible();
    await expect(page.getByRole("heading", { level: 2, name: "Versions" })).toHaveCount(0);

    await page.goto(`/components/${slug}/versions`);
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Components");
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText(renamedName);
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Versions");
    await expect(page.getByRole("heading", { level: 1, name: renamedName })).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit" })).toBeVisible();
    await expect(page.getByRole("link", { name: "View" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Publish" })).toHaveCount(0);
    let versionsTable = page.getByRole("table").filter({ hasText: "Dataset Version" });
    await expect(versionsTable).toContainText("Draft");
    await expect(versionsTable).toContainText("Published");
    await expect(versionsTable).toContainText(`v${major}`);

    await page.goto(`/components/${slug}/edit`);
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Components");
    await expect(page.getByRole("navigation", { name: "Breadcrumb" })).toContainText("Edit Component");
    await expect(page.getByRole("heading", { level: 1, name: "Edit Component" })).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Name", exact: true })).toHaveValue(renamedName);
    await expect(page.getByRole("textbox", { name: "Slug" })).toHaveValue(slug);
    await expect(page.getByRole("textbox", { name: "Description" })).toHaveValue(
      renamedDescription,
    );
    await expect(page.getByRole("group", { name: "Dataset Context" })).toHaveCount(0);

    await page.goto(`/components/${slug}/edit`);
    await page.waitForLoadState("networkidle");
    const publishMenu = page.locator(".component-editor__publish-menu");
    await publishMenu.locator(".component-editor__publish-button").click();
    await expect(publishMenu).toHaveClass(/is-open/);
    await publishMenu.getByRole("menuitem", { name: "Create New Version" }).click();
    await expect(page.getByRole("dialog", { name: "Review component consumers" })).toBeVisible();
    await page.getByLabel("New Version Note").fill("Playwright replacement version.");
    await Promise.all([
      page.waitForResponse(
        (response) =>
          response.url().endsWith("/api/admin/components/save") &&
          response.request().method() === "POST" &&
          response.ok(),
      ),
      page.getByRole("button", { name: "Create New Version" }).click(),
    ]);
    await page.waitForURL(`**/components/${slug}`);
    await expect(page.getByRole("heading", { level: 1, name: renamedName })).toBeVisible();

    component = await loadComponent(page, slug);
    expect(component.versions.find((version) => version.id === secondDraft.id)?.status).toBe(
      "published",
    );
    expect(component.versions.some((version) => version.status === "superseded")).toBe(true);

    const supersededTable = await expectJson<ComponentTable>(
      await page.request.get(
        `/api/components/${slug}/versions/${firstPublishedVersionId}/table`,
        {
          params: {
            visible_columns: firstField.key,
          },
        },
      ),
    );
    expect(supersededTable.materialization_state).toBe("ready");
    expect(supersededTable.component_version_id).toBe(firstPublishedVersionId);
    expect(supersededTable.component_type).toBe("table");

    await expectStatus(
      await page.request.patch(
        `/api/admin/components/${created.id}/versions/${secondDraft.id}`,
        {
          data: {
            dataset_id: dataset.id,
            dataset_version_major: major,
            component_type: "table",
            config: tableConfig([firstField.key]),
          },
        },
      ),
      400,
    );
    await expectStatus(
      await page.request.post(
        `/api/admin/components/${created.id}/versions/${firstPublishedVersionId}/publish`,
        { data: {} },
      ),
      400,
    );

    await assertNoConsoleErrors();
  });

  test("admin can author, publish, and view visual components", async ({ page }) => {
    test.setTimeout(180_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    const bridgeRequests: string[] = [];
    page.on("request", (request) => {
      if (request.url().includes("/bridge/")) {
        bridgeRequests.push(request.url());
      }
    });
    await signInAsAdmin(page);
    await ensureDemoSeed(page);

    const { dataset, major } = await pickDatasetMajor(page);
    const field = textLikeField(dataset.output_fields);

    await page.goto("/components/new");
    await page.waitForLoadState("networkidle");
    await expect(page.getByRole("button", { name: "Save Draft" })).toBeVisible();
    await page.getByRole("textbox", { name: "Name", exact: true }).fill(`Draft Slug Behavior ${RUN_ID}`);
    await expect(page.getByRole("textbox", { name: "Slug" })).toHaveValue("");
    await page.getByRole("textbox", { name: "Name", exact: true }).blur();
    await expect(page.getByRole("textbox", { name: "Slug" })).toHaveValue(
      `draft_slug_behavior_${String(RUN_ID).toLowerCase()}`,
    );
    await selectDatasetVersion(page, dataset, major);
    await page.getByRole("radio", { name: "Bar", exact: true }).click();
    await expect(page.getByRole("group", { name: "Fields & Calculation" })).toBeVisible();
    const barCalculation = page.locator(".component-editor__role-card--measure select").first();
    const barValueField = page
      .locator(".component-editor__role-card--measure .component-editor__measure-grid label.form-field")
      .nth(1)
      .locator("select");
    const barCategoryField = page
      .locator(".component-editor__role-grid > .component-editor__role-card")
      .first()
      .locator("select")
      .first();
    await barCalculation.selectOption("count");
    await expect(barValueField).toBeVisible();
    await expect(barCategoryField).toBeVisible();
    await barCategoryField.selectOption(field.key);
    await barValueField.selectOption(field.key);
    await page.getByLabel("Split bars", { exact: true }).check();
    await page.getByLabel("Series field").selectOption(field.key);
    await expect(page.getByLabel("Missing categories")).toBeVisible();
    await expect(page.getByLabel("Missing series")).toBeVisible();
    await expect(page.getByLabel("Missing values")).toBeVisible();
    await page.getByLabel("Missing series").selectOption("explicit_missing");
    const comparisonLayout = page
      .locator(".component-editor__bar-display label.form-field", { hasText: "Comparison Layout" })
      .locator("select");
    await expect(comparisonLayout).toBeVisible();
    await expect(barCalculation.locator("option")).toHaveText([
      "Count rows",
      "Count non-empty values",
      "Count unique values",
      "Sum",
      "Average",
      "Median",
      "Do not summarize",
    ]);
    await barCalculation.selectOption("row_count");
    await expect(barValueField).toHaveCount(0);
    await barCalculation.selectOption("none");
    await expect(barValueField).toBeVisible();
    await expect(page.locator(".component-editor__calculation-warning")).toBeVisible();
    await expect(page.locator(".component-editor-preview__badge")).toHaveText("Needs attention");
    await expect(comparisonLayout.locator('option[value="stacked"]')).toHaveAttribute(
      "disabled",
      "",
    );
    assertNoConsoleErrors.reset();
    await barCalculation.selectOption("sum");
    await expect(page.locator(".component-editor-preview__badge")).toHaveText("Needs attention");
    await page.getByRole("button", { name: "Save Draft", exact: true }).click();
    const validationFindings = page.getByRole("region", { name: "Validation Findings" });
    await expect(validationFindings).toBeVisible();
    await expect(validationFindings).toHaveAttribute("aria-live", "polite");
    await expect(page.getByRole("textbox", { name: "Name", exact: true })).toHaveValue(`Draft Slug Behavior ${RUN_ID}`);
    await expect(page).toHaveURL(/\/components\/new$/);
    assertNoConsoleErrors.reset();
    await barCalculation.selectOption("count");
    await comparisonLayout.selectOption("stacked");
    await barCalculation.selectOption("unique_count");
    await expect(comparisonLayout).toHaveValue("grouped");
    await expect(comparisonLayout.locator('option[value="stacked"]')).toHaveAttribute(
      "disabled",
      "",
    );
    await barCalculation.selectOption("count");
    await comparisonLayout.selectOption("stacked");
    await page.getByLabel("Category axis title").fill("Submission status");
    await page.getByLabel("Value axis title").fill("Responses");
    await page.getByLabel("Orientation").selectOption("vertical");
    await expect(page.getByLabel("Category axis title")).toHaveValue("Submission status");
    await expect(page.getByLabel("Value axis title")).toHaveValue("Responses");
    await expect(page.locator(".component-editor-preview svg")).toBeVisible();
    await expect(page.locator(".component-editor-preview__badge")).toHaveText("Valid config");
    await page.getByRole("radio", { name: "Donut", exact: true }).click();
    await page.getByRole("button", { name: "Change to Donut", exact: true }).click();
    await expect(page.locator("[data-component-kind-editor]")).toBeFocused();
    await expect(page.getByText("A donut chart is a pie chart with a hole in the center.")).toBeVisible();
    await page.locator(".component-editor__value-field select").selectOption(field.key);
    await page.locator(".component-editor__category-field select").selectOption(field.key);
    await expect(page.getByLabel("Legend Title", { exact: true })).toHaveValue(field.label);
    await expect(page.getByRole("table", { name: "Category Labels" })).toBeVisible();
    await expect(page.getByLabel("Sort Field", { exact: true }).locator("option")).toHaveText([
      "Default",
      "Category",
      "Summary Value",
    ]);
    const sortFieldHelp = page.locator("label.form-field", { hasText: "Sort Field" }).locator(".component-field-help");
    await sortFieldHelp.locator("summary").click();
    const sortFieldTooltip = sortFieldHelp.locator(".component-field-help__content");
    await expect(sortFieldTooltip).toBeVisible();
    await expect(sortFieldTooltip).toContainText("Summary Value: sorts by the summarized numeric value.");
    await expect(sortFieldTooltip).not.toContainText("X:");
    await expect(sortFieldTooltip).not.toContainText("Comparison:");

    const {
      dataset: demoSessionDataset,
      major: demoSessionMajor,
    } = await pickDemoSessionLogDataset(page);
    const participantsField = demoSessionDataset.output_fields.find(
      (candidate) => candidate.key === "session__participants",
    );
    const completedField = demoSessionDataset.output_fields.find(
      (candidate) => candidate.key === "session__completed_as_planned",
    );
    const topicsField = demoSessionDataset.output_fields.find(
      (candidate) => candidate.key === "session__topics_covered",
    );
    expect(participantsField).toBeTruthy();
    expect(completedField).toBeTruthy();
    expect(topicsField).toBeTruthy();

    await page.goto("/components/demo-session-log-bar/edit");
    await expect(page.getByLabel("Series field")).toHaveValue(completedField!.key);
    await expect(page.getByLabel("Legend Title", { exact: true })).toHaveValue("Completion Status");
    await expect(page.getByRole("table", { name: "Series Labels" })).toBeVisible();
    await expect
      .poll(async () =>
        page
          .locator("table.component-category-labels__table tbody th")
          .allTextContents(),
      )
      .toEqual(["false", "true"]);

    await page.goto("/components/demo-session-log-bar");
    const barSurface = page.locator(".component-d3-chart__surface");
    await expect(barSurface.locator(":scope > .component-d3-chart__legend + svg.component-d3-svg--bar")).toBeVisible();
    await expect(barSurface.locator("svg .component-d3-legend")).toHaveCount(0);

    const staleCategorySlug = `${COMPONENT_PREFIX}${RUN_ID}-category-reset`;
    await expectJson<IdResponse>(
      await page.request.post("/api/admin/components", {
        data: {
          name: `Playwright Category Reset ${RUN_ID}`,
          slug: staleCategorySlug,
          description: "Regression fixture for category display field changes.",
          version: {
            dataset_id: demoSessionDataset.id,
            dataset_version_major: demoSessionMajor,
            component_type: "donut",
            config: {
              summary_field: participantsField!.key,
              summary_type: "sum",
              category_field: completedField!.key,
              category_labels: {
                false: "No",
                true: "Yes",
              },
              category_colors: {
                false: "var(--semantic-secondary)",
                true: "var(--semantic-warning)",
              },
              sort_field: "summary_value",
              sort_direction: "desc",
              max_slices: 10,
              value_format: "integer",
            },
          },
        },
      }),
    );
    await page.goto(`/components/${staleCategorySlug}/edit`);
    await expect(page.locator(".component-editor__category-field select")).toHaveValue(completedField!.key);
    await expect(page.getByRole("row", { name: /false\s+No/i })).toBeVisible();
    await page.locator(".component-editor__category-field select").selectOption(topicsField!.key);
    await expect(page.getByLabel("Legend Title", { exact: true })).toHaveValue(topicsField!.label);
    await expect
      .poll(async () =>
        page
          .locator("table.component-category-labels__table tbody th")
          .allTextContents(),
      )
      .toEqual([
        "[\"attendance\", \"check_in\"]",
        "[\"family_support\", \"wellness\"]",
        "[\"intake\", \"welcome\"]",
        "[\"mentoring\", \"onboarding\"]",
        "[\"nutrition\", \"follow_up\"]",
        "[\"resume\", \"job_search\"]",
      ]);

    await page.getByRole("radio", { name: "Stat Card", exact: true }).click();
    await page.getByRole("button", { name: "Change to Stat Card", exact: true }).click();
        await expect(page.getByLabel("Panel Style", { exact: true })).toBeVisible();

    const validVisualValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "bar",
          config: visualConfig("bar", field.key),
        },
      }),
    );
    expect(validVisualValidation.valid).toBe(true);

    const draftPreview = await expectJson<{
      component_type: string;
      materialization_state: string;
      points: Array<{ x: string; value: number }>;
    }>(
      await page.request.post("/api/admin/components/preview", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "bar",
          config: visualConfig("bar", field.key),
        },
      }),
    );
    expect(draftPreview.component_type).toBe("bar");
    expect(["ready", "pending"]).toContain(draftPreview.materialization_state);

    const invalidVisualValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "bar",
          config: {
            ...visualConfig("bar", field.key),
            category_field: `missing_${RUN_ID}`,
          },
        },
      }),
    );
    expect(invalidVisualValidation.valid).toBe(false);
    expect(invalidVisualValidation.findings[0]).toMatchObject({
      code: "COMPONENT_CATEGORY_FIELD_NOT_IN_MAJOR_LINE",
      severity: "error",
    });

    for (const kind of ["bar", "line", "pie", "donut", "stat_card"]) {
      const uiComponent = await createAndPublishVisualComponentThroughUi(
        page,
        dataset,
        major,
        kind,
        field.key,
      );
      if (kind === "bar") {
        await page.goto(`/components/${uiComponent.slug}/edit`);
        await expect(
          page
            .locator(".component-editor__role-card--measure .component-editor__measure-grid label.form-field")
            .nth(1)
            .locator("select"),
        ).toHaveValue(field.key);
        await expect(page.locator(".component-editor__role-card--measure select").first()).toHaveValue("count");
        await expect(
          page
            .locator(".component-editor__role-grid > .component-editor__role-card")
            .first()
            .locator("select")
            .first(),
        ).toHaveValue(field.key);
        await expect(page.getByLabel("Missing categories")).toHaveValue("explicit_missing");
        await expect(page.getByLabel("Missing values")).toHaveValue("zero");
      }
    }

    for (const kind of ["bar", "line", "pie", "donut", "stat_card"]) {
      const slug = `${COMPONENT_PREFIX}${RUN_ID}-${kind}`;
      const name = `Playwright Visual Component ${kind} ${RUN_ID}`;
      const created = await createVisualComponentDraft(
        page,
        name,
        slug,
        dataset,
        major,
        kind,
        field.key,
      );
      const component = await loadComponent(page, slug);
      expect(component.versions[0]).toMatchObject({
        component_type: kind,
        status: "draft",
        binding_mode: "major_line",
      });
      await publishComponentVersion(page, created.id, component.versions[0].id);
      const visual = await expectJson<ComponentVisual>(
        await page.request.get(`/api/components/${slug}/${visualPath(kind)}`),
      );
      expect(visual).toMatchObject({
        component_version_id: component.versions[0].id,
        component_type: kind,
        materialization_state: "ready",
      });
      if (kind === "stat_card") {
        expect(visual.stat?.label).toBe("Submission count");
      } else if (kind === "pie" || kind === "donut") {
        expect(visual.slices.length).toBeGreaterThan(0);
      } else {
        expect(visual.points.length).toBeGreaterThan(0);
        if (kind === "bar") {
          expect(visual.bar_orientation).toBe("horizontal");
          expect(visual.x_axis_label).toBe("Submissions");
          expect(visual.y_axis_label).toBe("Category");
        }
      }

      await page.goto(`/components/${slug}/view`);
      await expect(page.getByRole("heading", { level: 1, name })).toBeVisible();
      await expect(page.getByRole("link", { name: "Versions" })).toHaveAttribute(
        "href",
        `/components/${slug}/versions`,
      );
      await expect(page.locator(".component-visual-preview")).toBeVisible();
      if (kind !== "stat_card") {
        await expect(page.locator(".component-d3-svg")).toBeVisible({ timeout: 15_000 });
      }
    }

    const historySlug = `${COMPONENT_PREFIX}${RUN_ID}-visual-history`;
    const historyName = `Playwright Visual History ${RUN_ID}`;
    const historyCreated = await createVisualComponentDraft(
      page,
      historyName,
      historySlug,
      dataset,
      major,
      "bar",
      field.key,
    );
    let historyComponent = await loadComponent(page, historySlug);
    const firstVisualVersionId = historyComponent.versions[0].id;
    await publishComponentVersion(page, historyCreated.id, firstVisualVersionId);
    const replacementDraft = await saveVisualDraft(
      page,
      historyCreated.id,
      dataset,
      major,
      "line",
      field.key,
    );
    await publishComponentVersion(page, historyCreated.id, replacementDraft.id);
    historyComponent = await loadComponent(page, historySlug);
    expect(historyComponent.versions.some((version) => version.status === "superseded")).toBe(
      true,
    );

    await page.goto(`/components/${historySlug}/versions`);
    await expect(page.getByRole("heading", { level: 1, name: historyName })).toBeVisible();
    const visualVersionsTable = page.getByRole("table").filter({ hasText: "Dataset Version" });
    await expect(visualVersionsTable).toContainText("Bar");
    await expect(visualVersionsTable).toContainText("Line");
    await expect(visualVersionsTable).toContainText("Superseded");
    await expect(visualVersionsTable).toContainText("Published");

    const supersededVisual = await expectJson<ComponentVisual>(
      await page.request.get(
        `/api/components/${historySlug}/versions/${firstVisualVersionId}/bar`,
      ),
    );
    expect(supersededVisual).toMatchObject({
      component_version_id: firstVisualVersionId,
      component_type: "bar",
      materialization_state: "ready",
    });
    const currentReplacementVisual = await expectJson<ComponentVisual>(
      await page.request.get(`/api/components/${historySlug}/line`),
    );
    expect(currentReplacementVisual).toMatchObject({
      component_version_id: replacementDraft.id,
      component_type: "line",
      materialization_state: "ready",
    });

    const wrongKindStatus = await expectStatus(
      await page.request.get(`/api/components/${COMPONENT_PREFIX}${RUN_ID}-bar/line`),
      400,
    );
    const wrongKindBody = JSON.parse(wrongKindStatus) as ApiErrorBody;
    expect(wrongKindBody.error).toContain("expected component type 'line'");
    await expectStatus(
      await page.request.get(`/api/components/${COMPONENT_PREFIX}${RUN_ID}-stat_card/stat_card`),
      404,
    );

    expect(bridgeRequests).toEqual([]);
    await assertNoConsoleErrors();
  });

  test("component editor remains contained and uses an accessible mobile preview drawer", async ({
    page,
  }) => {
    test.setTimeout(60_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await page.setViewportSize({ width: 600, height: 900 });
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const components = await expectJson<ComponentSummary[]>(
      await page.request.get("/api/components"),
    );
    const line = components.find(
      (component) => component.current_component_type === "line",
    );
    expect(line, "the demo seed should include a line component").toBeTruthy();

    await page.goto(`/components/${line!.slug}/edit`);
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
        return kindBox !== null && filtersBox !== null && kindBox.y < filtersBox.y;
      })
      .toBe(true);

    const hasHorizontalOverflow = await page.evaluate(
      () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
    );
    expect(hasHorizontalOverflow).toBe(false);

    const previewButton = page.getByRole("button", { name: "Open preview" });
    await expect(previewButton).toBeVisible();
    await previewButton.click();
    const previewDialog = page.getByRole("dialog", { name: "Component preview" });
    await expect(previewDialog).toBeVisible();
    await expect(previewDialog).toBeFocused();
    await page.keyboard.press("Escape");
    await expect(previewDialog).not.toBeVisible();
    await expect(previewButton).toBeFocused();

    const calculationHelp = page.locator('summary[aria-label="Show help for Calculation"]');
    await calculationHelp.click();
    await expect(page.getByRole("tooltip")).toBeVisible();
    await page.locator("main").click({ position: { x: 2, y: 2 } });
    await expect(page.getByRole("tooltip")).not.toBeVisible();
    await assertNoConsoleErrors();
  });
});
