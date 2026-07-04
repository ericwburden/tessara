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

type VisibilityNode = { node_id: string; node_name: string };

type DatasetSummary = {
  id: string;
  name: string;
  visibility_nodes: VisibilityNode[];
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

type DatasetTable = {
  rows: Array<{ values: Record<string, string | null> }>;
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
  return pickDatasetMajorMatching(
    page,
    (candidate) => candidate.output_fields.some((field) => isTextLikeField(field)),
    "a published dataset with a text-like output field should exist",
  );
}

async function pickDatasetMajorMatching(
  page: Page,
  predicate: (dataset: DatasetSummary) => boolean,
  message: string,
) {
  const datasets = await expectJson<DatasetSummary[]>(
    await page.request.get("/api/datasets"),
  );
  const dataset = datasets.find(
    (candidate) =>
      predicate(candidate) &&
      (candidate.major_versions?.length || candidate.current_version_major),
  );
  expect(dataset, message).toBeTruthy();
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

function numericField(fields: DatasetFieldDefinition[]) {
  const field = fields.find((candidate) => candidate.field_type === "number");
  expect(field, "dataset should expose a numeric output field").toBeTruthy();
  return field!;
}

function detailConfig(fields: DatasetFieldDefinition[]) {
  return {
    columns: [textLikeField(fields).key],
  };
}

function aggregateConfig(
  fields: DatasetFieldDefinition[],
  preFilterValue?: string,
) {
  const groupField = textLikeField(fields);
  const config: Record<string, unknown> = {
    group_fields: [groupField.key],
    metrics: [
      {
        key: "row_count",
        label: "Rows",
        function: "count",
      },
    ],
  };
  if (preFilterValue) {
    config.pre_filters = [
      {
        field_key: groupField.key,
        operator: "equals",
        value: preFilterValue,
      },
    ];
    config.post_filters = [
      {
        field_key: "row_count",
        operator: "gt",
        value: "0",
      },
    ];
  }
  return config;
}

async function createComponentDraft(
  page: Page,
  name: string,
  slug: string,
  dataset: DatasetSummary,
  major: number,
) {
  const response = await page.request.post("/api/admin/components", {
    data: {
      name,
      slug,
      description: "Playwright Sprint 4A component workflow fixture.",
      version: {
        dataset_id: dataset.id,
        dataset_version_major: major,
        component_type: "detail_table",
        config: detailConfig(dataset.output_fields),
        publish: false,
      },
    },
  });
  return expectJson<IdResponse>(response);
}

async function saveAggregateDraft(
  page: Page,
  componentId: string,
  dataset: DatasetSummary,
  major: number,
  preFilterValue?: string,
) {
  return expectJson<IdResponse>(
    await page.request.post(`/api/admin/components/${componentId}/versions`, {
      data: {
        dataset_id: dataset.id,
        dataset_version_major: major,
        component_type: "aggregate_table",
        config: aggregateConfig(dataset.output_fields, preFilterValue),
        publish: false,
      },
    }),
  );
}

async function saveDetailDraft(
  page: Page,
  componentId: string,
  dataset: DatasetSummary,
  major: number,
) {
  return expectJson<IdResponse>(
    await page.request.post(`/api/admin/components/${componentId}/versions`, {
      data: {
        dataset_id: dataset.id,
        dataset_version_major: major,
        component_type: "detail_table",
        config: {
          columns: [dataset.output_fields[0].key],
        },
        publish: false,
      },
    }),
  );
}

async function patchDetailDraft(
  page: Page,
  componentId: string,
  versionId: string,
  dataset: DatasetSummary,
  major: number,
) {
  return expectJson<IdResponse>(
    await page.request.patch(
      `/api/admin/components/${componentId}/versions/${versionId}`,
      {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "detail_table",
          config: {
            columns: [dataset.output_fields[0].key],
          },
          publish: false,
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
    const slug = `${COMPONENT_PREFIX}${RUN_ID}`;
    const name = `Playwright Component Workflow ${RUN_ID}`;
    const aggregateGroupField = textLikeField(dataset.output_fields);
    const sourceTable = await expectJson<DatasetTable>(
      await page.request.get(`/api/datasets/${dataset.id}/table`, {
        params: {
          visible_columns: aggregateGroupField.key,
          page_size: "25",
        },
      }),
    );
    const aggregatePreFilterValue = sourceTable.rows
      .map((row) => row.values[aggregateGroupField.key])
      .find((value): value is string => Boolean(value));
    expect(
      aggregatePreFilterValue,
      `dataset table should include a value for ${aggregateGroupField.key}`,
    ).toBeTruthy();

    await page.goto("/components/new");
    await page.getByLabel("Dataset Version").selectOption(`${dataset.id}|${major}`);
    const columnPicker = page.getByRole("group", { name: "Columns" });
    const selectedColumns = await columnPicker.getByRole("checkbox").all();
    expect(selectedColumns.length).toBeGreaterThan(0);
    for (const checkbox of selectedColumns) {
      if (await checkbox.isChecked()) {
        await checkbox.uncheck();
      }
    }
    await page.getByRole("button", { name: "Validate Draft" }).click();
    await expect(
      page.getByRole("region", { name: "Validation Findings" }),
    ).toBeVisible();
    await expect(page.locator('[data-field-path="config"]')).toContainText(
      "COMPONENT_FIELD_NOT_IN_MAJOR_LINE",
    );
    await expect(page.locator('[data-field-path="config"]')).toContainText(
      "requires at least one column",
    );

    const created = await createComponentDraft(page, name, slug, dataset, major);
    let component = await loadComponent(page, slug);
    expect(component.versions).toHaveLength(1);
    expect(component.versions[0]).toMatchObject({
      dataset_id: dataset.id,
      dataset_version_major: major,
      binding_mode: "major_line",
      component_type: "detail_table",
      status: "draft",
    });
    expect(component.versions[0]).not.toHaveProperty("dataset_revision_id");
    const readerComponentsBeforePublish = await expectJson<Array<{ slug: string }>>(
      await page.request.get("/api/components"),
    );
    expect(readerComponentsBeforePublish.some((item) => item.slug === slug)).toBe(false);
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

    const updatedDraft = await saveAggregateDraft(
      page,
      created.id,
      dataset,
      major,
      aggregatePreFilterValue,
    );
    expect(updatedDraft.id).toBe(component.versions[0].id);
    component = await loadComponent(page, slug);
    expect(component.versions).toHaveLength(1);
    expect(component.versions[0].component_type).toBe("aggregate_table");

    const validValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "aggregate_table",
          config: aggregateConfig(dataset.output_fields, aggregatePreFilterValue),
        },
      }),
    );
    expect(validValidation.valid).toBe(true);
    expect(validValidation.findings).toEqual([]);

    const textField = textLikeField(dataset.output_fields);
    const numericMajor = await pickDatasetMajorMatching(
      page,
      (candidate) => candidate.output_fields.some((field) => field.field_type === "number"),
      "a published dataset with a numeric output field should exist",
    );
    const numberField = numericField(numericMajor.dataset.output_fields);
    for (const metric of [
      {
        dataset,
        major,
        groupField: textField.key,
        metric: { key: "row_count", label: "Rows", function: "count" },
      },
      {
        dataset,
        major,
        groupField: textField.key,
        metric: {
          key: "distinct_values",
          label: "Distinct Values",
          function: "count_distinct",
          source_field_key: textField.key,
        },
      },
      {
        dataset: numericMajor.dataset,
        major: numericMajor.major,
        groupField: "",
        metric: {
          key: "total_value",
          label: "Total Value",
          function: "sum",
          source_field_key: numberField.key,
        },
      },
      {
        dataset: numericMajor.dataset,
        major: numericMajor.major,
        groupField: "",
        metric: {
          key: "average_value",
          label: "Average Value",
          function: "avg",
          source_field_key: numberField.key,
        },
      },
      {
        dataset: numericMajor.dataset,
        major: numericMajor.major,
        groupField: "",
        metric: {
          key: "minimum_value",
          label: "Minimum Value",
          function: "min",
          source_field_key: numberField.key,
        },
      },
      {
        dataset: numericMajor.dataset,
        major: numericMajor.major,
        groupField: "",
        metric: {
          key: "maximum_value",
          label: "Maximum Value",
          function: "max",
          source_field_key: numberField.key,
        },
      },
    ]) {
      const aggregateFunctionValidation = await expectJson<ComponentValidationResponse>(
        await page.request.post("/api/admin/components/validate", {
          data: {
            dataset_id: metric.dataset.id,
            dataset_version_major: metric.major,
            component_type: "aggregate_table",
            config: {
              group_fields: metric.groupField ? [metric.groupField] : [],
              metrics: [metric.metric],
            },
          },
        }),
      );
      expect(aggregateFunctionValidation.valid).toBe(true);
      expect(aggregateFunctionValidation.findings).toEqual([]);
    }

    const countValuesValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "aggregate_table",
          config: {
            group_fields: [textField.key],
            metrics: [
              {
                key: "value_count",
                label: "Value Count",
                function: "count_values",
                source_field_key: textField.key,
              },
            ],
          },
        },
      }),
    );
    expect(countValuesValidation.valid).toBe(true);
    expect(countValuesValidation.findings).toEqual([]);

    const invalidValidation = await expectJson<ComponentValidationResponse>(
      await page.request.post("/api/admin/components/validate", {
        data: {
          dataset_id: dataset.id,
          dataset_version_major: major,
          component_type: "stat_card",
          config: {},
        },
      }),
    );
    expect(invalidValidation.valid).toBe(false);
    expect(invalidValidation.findings[0]).toMatchObject({
      code: "COMPONENT_UNSUPPORTED_KIND",
      severity: "error",
    });

    await publishComponentVersion(page, created.id, updatedDraft.id);
    component = await loadComponent(page, slug);
    expect(component.versions[0].status).toBe("published");
    const firstPublishedVersionId = updatedDraft.id;

    const aggregateTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          page_size: "25",
          visible_columns: `${aggregateGroupField.key},row_count`,
          sort: "row_count:desc",
        },
      }),
    );
    expect(aggregateTable.materialization_state).toBe("ready");
    expect(aggregateTable.component_version_id).toBe(firstPublishedVersionId);
    expect(aggregateTable.component_type).toBe("aggregate_table");
    expect(aggregateTable.columns.map((column) => column.key)).toEqual([
      aggregateGroupField.key,
      "row_count",
    ]);
    expect(aggregateTable.rows.length).toBeGreaterThan(0);
    expect(
      aggregateTable.rows.every(
        (row) => row.values[aggregateGroupField.key] === aggregatePreFilterValue,
      ),
    ).toBe(true);
    expect(
      aggregateTable.rows.every(
        (row) => Number(row.values.row_count ?? "0") > 0,
      ),
    ).toBe(true);
    const aggregateVersionTable = await expectJson<ComponentTable>(
      await page.request.get(
        `/api/components/${slug}/versions/${firstPublishedVersionId}/table`,
        {
          params: {
            visible_columns: "row_count",
            sort: "row_count:desc",
          },
        },
      ),
    );
    expect(aggregateVersionTable.materialization_state).toBe("ready");
    expect(aggregateVersionTable.component_version_id).toBe(firstPublishedVersionId);
    expect(aggregateVersionTable.columns.map((column) => column.key)).toEqual([
      "row_count",
    ]);
    const filteredAggregateTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: "row_count",
          "filter[row_count][operator]": "gt",
          "filter[row_count][value]": "0",
        },
      }),
    );
    expect(filteredAggregateTable.materialization_state).toBe("ready");
    expect(filteredAggregateTable.rows.length).toBeGreaterThan(0);
    expect(
      filteredAggregateTable.rows.every(
        (row) => Number(row.values.row_count ?? "0") > 0,
      ),
    ).toBe(true);
    const betweenAggregateTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: "row_count",
          "filter[row_count][operator]": "between",
          "filter[row_count][value]": "1,100000",
        },
      }),
    );
    expect(betweenAggregateTable.materialization_state).toBe("ready");
    expect(betweenAggregateTable.rows.length).toBeGreaterThan(0);
    expect(
      betweenAggregateTable.rows.every((row) => {
        const rowCount = Number(row.values.row_count ?? "0");
        return rowCount >= 1 && rowCount <= 100000;
      }),
    ).toBe(true);
    await page.goto(`/components/${slug}/view`);
    await expect(
      page.getByRole("heading", { level: 1, name: slug }),
    ).toBeVisible();
    await expect(
      page.getByRole("columnheader", { name: aggregateGroupField.label }),
    ).toBeVisible();
    await expect(page.getByRole("columnheader", { name: "Rows" })).toBeVisible();
    const aggregateVisibleColumns = page.getByRole("group", {
      name: "Visible Columns",
    });
    await aggregateVisibleColumns
      .getByRole("checkbox", { name: /Rows/ })
      .uncheck();
    await expect(
      page.getByRole("columnheader", { name: aggregateGroupField.label }),
    ).toBeVisible();
    await expect(page.getByRole("columnheader", { name: "Rows" })).toHaveCount(0);
    await aggregateVisibleColumns.getByRole("button", { name: "Show All" }).click();
    await expect(page.getByRole("columnheader", { name: "Rows" })).toBeVisible();

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

    const secondDraft = await saveDetailDraft(page, created.id, dataset, major);
    const patchedDraft = await patchDetailDraft(
      page,
      created.id,
      secondDraft.id,
      dataset,
      major,
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
          visible_columns: "row_count",
        },
      }),
    );
    expect(tableWhileDraftExists.component_version_id).toBe(firstPublishedVersionId);
    expect(tableWhileDraftExists.component_type).toBe("aggregate_table");
    expect(tableWhileDraftExists.columns.map((column) => column.key)).toEqual([
      "row_count",
    ]);

    await page.goto("/components");
    await expect(
      page.getByRole("heading", { level: 1, name: "Components" }),
    ).toBeVisible();
    await expect(page.getByRole("link", { name: renamedName })).toBeVisible();

    await page.goto(`/components/${slug}`);
    await expect(page.getByRole("heading", { level: 1, name: renamedName })).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Publish" })).toBeVisible();
    await expect(page.getByRole("link", { name: "View" })).toBeVisible();
    await expect(page.locator("tbody")).toHaveText(/Published/);
    await expect(page.locator("tbody")).not.toContainText("Draft");
    await expect(page.locator("tbody")).toContainText("Published");
    await expect(page.locator("tbody")).toContainText(`v${major}`);

    await page.goto(`/components/${slug}/edit`);
    await expect(
      page.getByRole("heading", { level: 1, name: "Edit Component" }),
    ).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Name" })).toHaveValue(renamedName);
    await expect(page.getByRole("textbox", { name: "Slug" })).toHaveValue(slug);
    await expect(page.getByRole("textbox", { name: "Description" })).toHaveValue(
      renamedDescription,
    );

    await page.goto(`/components/${slug}/publish`);
    await expect(
      page.getByRole("heading", { level: 1, name: "Publish Component" }),
    ).toBeVisible();
    await page.getByRole("button", { name: "Publish Draft" }).click();
    await expect(page.getByText("Component published.")).toBeVisible();

    component = await loadComponent(page, slug);
    expect(component.versions.find((version) => version.id === secondDraft.id)?.status).toBe(
      "published",
    );
    expect(component.versions.some((version) => version.status === "superseded")).toBe(true);
    const supersededAggregateTable = await expectJson<ComponentTable>(
      await page.request.get(
        `/api/components/${slug}/versions/${firstPublishedVersionId}/table`,
        {
          params: {
            visible_columns: "row_count",
          },
        },
      ),
    );
    expect(supersededAggregateTable.materialization_state).toBe("ready");
    expect(supersededAggregateTable.component_version_id).toBe(firstPublishedVersionId);
    expect(supersededAggregateTable.component_type).toBe("aggregate_table");
    await expectStatus(
      await page.request.patch(
        `/api/admin/components/${created.id}/versions/${secondDraft.id}`,
        {
          data: {
            dataset_id: dataset.id,
            dataset_version_major: major,
            component_type: "detail_table",
            config: detailConfig(dataset.output_fields),
            publish: false,
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

    const table = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`),
    );
    expect(table.materialization_state).toBe("ready");
    const filterField = textLikeField(dataset.output_fields).key;
    expect(table.columns.map((column) => column.key)).toEqual([filterField]);
    const filterValue = table.rows
      .map((row) => row.values[filterField])
      .find((value): value is string => Boolean(value));
    expect(filterValue, `component table should include a value for ${filterField}`).toBeTruthy();
    const filteredTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: filterField,
          [`filter[${filterField}][operator]`]: "equals",
          [`filter[${filterField}][value]`]: filterValue!,
        },
      }),
    );
    expect(filteredTable.materialization_state).toBe("ready");
    expect(filteredTable.rows.length).toBeGreaterThan(0);
    expect(
      filteredTable.rows.every((row) => row.values[filterField] === filterValue),
    ).toBe(true);
    const searchTerm = filterValue!.slice(0, Math.min(4, filterValue!.length));
    const searchedTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          q: searchTerm,
          visible_columns: filterField,
        },
      }),
    );
    expect(searchedTable.materialization_state).toBe("ready");
    expect(searchedTable.rows.length).toBeGreaterThan(0);
    expect(
      searchedTable.rows.every((row) =>
        (row.values[filterField] ?? "")
          .toLowerCase()
          .includes(searchTerm.toLowerCase()),
      ),
    ).toBe(true);

    const negativeValue = `not-present-${RUN_ID}`;
    const notEqualsTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: filterField,
          [`filter[${filterField}][operator]`]: "not_equals",
          [`filter[${filterField}][value]`]: negativeValue,
        },
      }),
    );
    expect(notEqualsTable.materialization_state).toBe("ready");
    expect(notEqualsTable.rows.length).toBeGreaterThan(0);
    expect(
      notEqualsTable.rows.every((row) => row.values[filterField] !== negativeValue),
    ).toBe(true);

    const notContainsTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: filterField,
          [`filter[${filterField}][operator]`]: "not_contains",
          [`filter[${filterField}][value]`]: negativeValue,
        },
      }),
    );
    expect(notContainsTable.materialization_state).toBe("ready");
    expect(notContainsTable.rows.length).toBeGreaterThan(0);
    expect(
      notContainsTable.rows.every(
        (row) => !(row.values[filterField] ?? "").includes(negativeValue),
      ),
    ).toBe(true);

    const notNullTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: filterField,
          [`filter[${filterField}][operator]`]: "is_not_null",
        },
      }),
    );
    expect(notNullTable.materialization_state).toBe("ready");
    expect(notNullTable.rows.length).toBeGreaterThan(0);
    expect(notNullTable.rows.every((row) => row.values[filterField] !== null)).toBe(
      true,
    );
    const emptyTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: filterField,
          [`filter[${filterField}][operator]`]: "is_empty",
        },
      }),
    );
    expect(emptyTable.materialization_state).toBe("ready");
    expect(
      emptyTable.rows.every((row) => {
        const value = row.values[filterField];
        return value === null || value === "";
      }),
    ).toBe(true);

    const pagedTable = await expectJson<ComponentTable>(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          page_size: "1",
          visible_columns: filterField,
          sort: `${filterField}:asc`,
        },
      }),
    );
    expect(pagedTable.pagination.page_size).toBe(1);
    expect(pagedTable.columns.map((column) => column.key)).toEqual([filterField]);
    expect(pagedTable.rows.length).toBeLessThanOrEqual(1);
    if (pagedTable.pagination.has_more) {
      expect(pagedTable.pagination.next_cursor).toMatch(/^offset:/);
    }
    const badVisibleColumns = await expectStatus(
      await page.request.get(`/api/components/${slug}/table`, {
        params: {
          visible_columns: `${filterField},missing_${RUN_ID}`,
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
    await expect(
      page.getByRole("heading", { level: 1, name: slug }),
    ).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();
    await expect(page.getByLabel("Filter Field")).toBeVisible();
    await page.getByLabel("Filter Field").selectOption(filterField);
    await page.getByLabel("Filter Operator").selectOption("is_not_null");
    await expect(page.getByRole("table")).toBeVisible();
    await assertNoConsoleErrors();
  });
});
