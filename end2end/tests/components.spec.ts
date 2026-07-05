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
  return async () => {
    expect(
      errors,
      `browser console should stay clean: ${errors.join("\n")}`,
    ).toEqual([]);
  };
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

function isTextLikeField(field: DatasetFieldDefinition) {
  return field.field_type === "text" || field.field_type === "static_text";
}

function textLikeField(fields: DatasetFieldDefinition[]) {
  const field = fields.find((candidate) => isTextLikeField(candidate));
  expect(field, "dataset should expose a text-like output field").toBeTruthy();
  return field!;
}

function tableConfig(fieldKeys: string[], pageSize = 25) {
  return {
    visible_columns: fieldKeys,
    default_sort: fieldKeys[0]
      ? {
          field_key: fieldKeys[0],
          direction: "asc",
        }
      : null,
    page_size: pageSize,
  };
}

async function createComponentDraft(
  page: Page,
  name: string,
  slug: string,
  dataset: DatasetSummary,
  major: number,
  fieldKeys: string[],
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
          config: tableConfig(fieldKeys),
        },
      },
    }),
  );
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
    await page.getByLabel("Dataset Version").selectOption(`${dataset.id}|${major}`);
    const datasetContext = page.locator("section.route-panel__section").filter({
      has: page.getByRole("heading", { name: "Dataset Context" }),
    });
    await expect(datasetContext).toBeVisible();
    if (dataset.grain) {
      await expect(datasetContext).toContainText(dataset.grain);
    }
    if (dataset.tags?.length) {
      await expect(datasetContext).toContainText(dataset.tags[0]);
    }
    const columnPicker = page.getByRole("group", { name: "Columns" });
    const selectedColumns = await columnPicker.getByRole("checkbox").all();
    expect(selectedColumns.length).toBeGreaterThan(0);
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
    await expect(page.getByRole("cell", { name: "Draft" })).toBeVisible();
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

    await page.goto(`/components/${slug}/view`);
    await expect(page.getByRole("heading", { level: 1, name: slug })).toBeVisible();
    await expect(page.getByRole("columnheader", { name: firstField.label })).toBeVisible();
    const visibleColumns = page.getByRole("group", { name: "Visible Columns" });
    await expect(
      visibleColumns.getByRole("checkbox", { name: firstField.label }),
    ).toBeChecked();
    await page.getByLabel("Filter Field").selectOption(firstField.key);
    await page.getByLabel("Filter Operator").selectOption("is_not_null");
    await expect(page.getByRole("table")).toBeVisible();

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
    await expect(page.getByRole("heading", { level: 1, name: "Components" })).toBeVisible();
    await expect(page.getByRole("link", { name: renamedName })).toBeVisible();

    await page.goto(`/components/${slug}`);
    await expect(page.getByRole("heading", { level: 1, name: renamedName })).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Publish" })).toBeVisible();
    await expect(page.getByRole("link", { name: "View" })).toBeVisible();
    await expect(page.locator("tbody")).toContainText("Draft");
    await expect(page.locator("tbody")).toContainText("Published");
    await expect(page.locator("tbody")).toContainText(`v${major}`);

    await page.goto(`/components/${slug}/edit`);
    await expect(page.getByRole("heading", { level: 1, name: "Edit Component" })).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Name" })).toHaveValue(renamedName);
    await expect(page.getByRole("textbox", { name: "Slug" })).toHaveValue(slug);
    await expect(page.getByRole("textbox", { name: "Description" })).toHaveValue(
      renamedDescription,
    );
    await expect(page.getByRole("heading", { name: "Dataset Context" })).toBeVisible();

    await page.goto(`/components/${slug}/publish`);
    await expect(page.getByRole("heading", { level: 1, name: "Publish Component" })).toBeVisible();
    await page.getByRole("button", { name: "Publish Draft" }).click();
    await expect(page.getByText("Component published.")).toBeVisible();

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
});
