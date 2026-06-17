import {
  expect,
  test,
  type APIResponse,
  type Locator,
  type Page,
} from "@playwright/test";

const BENIGN_NAVIGATION_ABORT_ERRORS = [
  "WebAssembly compilation aborted: Network error: Response body loading was aborted",
  "Failed to load resource: the server responded with a status of 404 (Not Found)",
];

const PW_DATASET_PREFIX = "pw-dataset-authoring-";

type DemoSeed = {
  form_id: string;
  form_version_id: string;
  program_node_id: string;
};

type IdResponse = {
  id: string;
};

type DatasetSummary = {
  id: string;
  name: string;
  slug: string;
  current_revision_id?: string | null;
  visibility_nodes?: Array<{ node_id: string; node_name: string }>;
  source_count?: number;
  field_count?: number;
};

type DatasetFieldDefinition = {
  key: string;
  label: string;
  source_alias: string;
  source_field_key: string;
  field_type: string;
};

type DatasetDefinition = {
  id: string;
  name: string;
  slug: string;
  generated_sql?: string | null;
  sources: Array<{ source_alias: string; form_id?: string | null }>;
  fields: DatasetFieldDefinition[];
  output_fields: DatasetFieldDefinition[];
  aggregation?: {
    group_fields: string[];
    metrics: Array<{
      key: string;
      label: string;
      function: string;
      source_field_key?: string | null;
    }>;
    row_picker?: unknown;
  } | null;
};

type DatasetTable = {
  dataset_id: string;
  rows: Array<{ values: Record<string, string | null> }>;
};

type DatasetSqlPreview = {
  generated_sql: string;
};

type RenderedField = {
  field_id: string;
  key: string;
  label: string;
  field_type: string;
};

type RenderedForm = {
  sections: Array<{
    fields: RenderedField[];
  }>;
};

type DatasetPayload = {
  name: string;
  slug: string;
  grain: "submission";
  composition_mode: string;
  visibility_node_ids: string[];
  definition_ast:
    | {
        kind: "form";
        alias: string;
        form_id: string;
        form_version_major: number | null;
      }
    | {
        kind: "operation";
        alias: string;
        operation: string;
        left: DatasetPayload["definition_ast"];
        right: DatasetPayload["definition_ast"];
        join_keys: Array<{ left_field: string; right_field: string }>;
      };
  aggregation?: {
    group_fields: string[];
    metrics: Array<{
      key: string;
      label: string;
      function: string;
      source_field_key?: string | null;
      position: number;
    }>;
    row_picker?: {
      sort_fields: Array<{ field_key: string; position: number }>;
      direction: "lowest" | "highest";
    } | null;
  } | null;
  fields: Array<{
    key: string;
    label: string;
    source_alias: string;
    source_field_key: string;
    position: number;
  }>;
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

async function seedDemo(page: Page) {
  return expectJson<DemoSeed>(
    await page.request.post("/api/demo/seed", { data: {} }),
  );
}

function renderedFields(renderedForm: RenderedForm) {
  return renderedForm.sections.flatMap((section) => section.fields);
}

function requireRenderedField(
  fields: RenderedField[],
  predicate: (field: RenderedField) => boolean,
  description: string,
) {
  const field = fields.find(predicate);
  expect(field, `expected rendered field: ${description}`).toBeTruthy();
  return field!;
}

function sourceFieldKey(alias: string, sourceFieldKeyValue: string) {
  const normalized = sourceFieldKeyValue.replace(/^_+/, "");
  return normalized ? `${alias}__${normalized}` : alias;
}

function datasetField(
  alias: string,
  sourceFieldKeyValue: string,
  label: string,
  position: number,
) {
  return {
    key: sourceFieldKey(alias, sourceFieldKeyValue),
    label,
    source_alias: alias,
    source_field_key: sourceFieldKeyValue,
    position,
  };
}

async function createDataset(page: Page, payload: DatasetPayload) {
  const response = await page.request.post("/api/admin/datasets", {
    data: payload,
  });
  return (await expectJson<IdResponse>(response)).id;
}

async function deleteDataset(page: Page, datasetId: string) {
  const response = await page.request.delete(`/api/admin/datasets/${datasetId}`);
  expect(
    [200, 204, 404].includes(response.status()),
    `dataset cleanup returned ${response.status()}`,
  ).toBeTruthy();
}

async function cleanupPlaywrightDatasets(page: Page) {
  const datasets = await expectJson<DatasetSummary[]>(
    await page.request.get("/api/datasets"),
  );
  for (const dataset of datasets.filter((item) =>
    item.slug.startsWith(PW_DATASET_PREFIX),
  )) {
    await deleteDataset(page, dataset.id);
  }
}

async function closeDesignerSheet(page: Page) {
  const sheet = page.getByRole("dialog", {
    name: "Dataset designer options",
  });
  await expect(sheet).toBeVisible();
  await sheet.getByLabel("Close dataset designer options").click();
  await expect(sheet).toBeHidden();
}

function fieldSourcePanel(page: Page, alias: string) {
  return page
    .locator("details.dataset-field-picker__source")
    .filter({ has: page.locator("summary", { hasText: `${alias} (` }) })
    .first();
}

async function openFieldSourcePanel(page: Page, alias: string) {
  const panel = fieldSourcePanel(page, alias);
  await expect(panel.locator("summary")).toBeVisible();
  const isOpen = await panel.evaluate((node) =>
    (node as HTMLDetailsElement).hasAttribute("open"),
  );
  if (!isOpen) {
    await panel.locator("summary").click();
  }
  await expect(panel.locator("tbody tr").first()).toBeVisible();
  return panel;
}

async function selectComboboxOption(
  scope: Locator,
  ariaLabel: string,
  search: string,
  optionName: string | RegExp,
) {
  await scope.getByRole("button", { name: ariaLabel }).click();
  const combo = scope.locator(".combobox.is-open").last();
  const input = combo.locator("input.combobox__input");
  await expect(input).toBeFocused();
  await input.fill(search);
  await combo.getByRole("option", { name: optionName }).first().click();
  await expect(combo).toHaveCount(0);
}

test("admin can author, edit, save, and view a Sprint 3A dataset", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  const seed = await seedDemo(page);
  await cleanupPlaywrightDatasets(page);

  const renderedForm = await expectJson<RenderedForm>(
    await page.request.get(`/api/form-versions/${seed.form_version_id}/render`),
  );
  const formFields = renderedFields(renderedForm);
  const numberField = requireRenderedField(
    formFields,
    (field) => field.field_type === "number",
    "numeric field",
  );
  const textField =
    formFields.find((field) => field.key === "snapshot_notes") ??
    requireRenderedField(formFields, (field) => field.field_type === "text", "text field");
  const runId = Date.now();
  const slug = `${PW_DATASET_PREFIX}${runId}`;
  const datasetName = `Playwright Dataset Authoring ${runId}`;
  const initialPayload: DatasetPayload = {
    name: datasetName,
    slug,
    grain: "submission",
    composition_mode: "union",
    visibility_node_ids: [seed.program_node_id],
    definition_ast: {
      kind: "form",
      alias: "program",
      form_id: seed.form_id,
      form_version_major: null,
    },
    aggregation: null,
    fields: [
      datasetField("program", "__node_id", "Attached Node ID", 0),
      datasetField("program", numberField.key, numberField.label, 1),
      datasetField("program", textField.key, textField.label, 2),
    ],
  };

  let datasetId: string | undefined;
  try {
    datasetId = await createDataset(page, initialPayload);

    await page.goto("/datasets");
    await expect(
      page.getByRole("heading", { level: 1, name: "Datasets" }),
    ).toBeVisible();
    await expect(page.getByRole("link", { name: "Create Dataset" })).toBeVisible();
    await page.getByPlaceholder("Search datasets").fill(slug);
    const datasetRow = page.locator("tbody tr", { hasText: datasetName }).first();
    await expect(datasetRow.getByRole("link", { name: datasetName })).toBeVisible();
    await expect(datasetRow.locator(".data-table__secondary-text")).toHaveText(slug);
    await expect(page.locator(".directory-table-pagination__actions")).not.toContainText(
      "page_count",
    );
    await expect(page.getByRole("button", { name: "Next" })).toBeVisible();
    await datasetRow.getByRole("link", { name: datasetName }).click();

    await expect(
      page.getByRole("heading", { level: 1, name: "Dataset Detail" }),
    ).toBeVisible();
    await expect(
      page.getByRole("heading", { level: 2, name: datasetName }),
    ).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit Dataset" })).toBeVisible();
    await expect(page.locator(".dataset-detail-summary")).toContainText("Slug");
    await expect(page.locator(".dataset-detail-summary")).toContainText("Grain");
    await expect(page.locator(".dataset-detail-summary")).toContainText("Visibility");
    await expect(page.locator(".dataset-detail-summary")).not.toContainText(
      "Composition",
    );

    await page.getByRole("button", { name: "Show dataset visibility nodes" }).click();
    const visibilityDialog = page.getByRole("dialog", {
      name: "Dataset visibility nodes",
    });
    await expect(visibilityDialog).toBeVisible();
    await expect(visibilityDialog).toContainText("Visible Nodes");
    await visibilityDialog.getByLabel("Close dataset visibility nodes").click();
    await expect(visibilityDialog).toBeHidden();

    await page.getByRole("button", { name: "Fields" }).click();
    const detailFieldLabel = page
      .locator("th.data-table__stacked-label", { hasText: numberField.label })
      .first();
    await expect(detailFieldLabel.locator(".data-table__secondary-text")).toHaveText(
      sourceFieldKey("program", numberField.key),
    );
    await page.getByRole("button", { name: "SQL" }).click();
    await expect(page.locator("pre.dataset-sql-panel code")).toContainText(
      "submission_value_fact.field_id",
    );
    await expect(page.locator("pre.dataset-sql-panel code")).not.toContainText(
      "field_key",
    );
    await expect(page.locator("pre.dataset-sql-panel code")).not.toContainText(
      "WITH ranked",
    );
    await page.goto(`/datasets/${datasetId}/edit`);
    await expect(
      page.getByRole("heading", { level: 1, name: "Edit Dataset" }),
    ).toBeVisible();
    for (const section of [
      "Dataset Definition",
      "Data Sources",
      "Fields",
      "Aggregation",
      "Filters",
      "Generated SQL",
      "Visibility",
    ]) {
      await expect(page.getByRole("heading", { name: section })).toBeVisible();
    }
    await expect(page.getByText("Operation Designer")).toHaveCount(0);
    await expect(page.getByText("Open Preview")).toHaveCount(0);
    await expect(page.locator("details.dataset-field-picker__source[open]")).toHaveCount(
      0,
    );

    await page.getByRole("button", { name: /^program$/ }).first().click();
    const sourceSheet = page.getByRole("dialog", {
      name: "Dataset designer options",
    });
    await expect(sourceSheet).toContainText("Source");
    await expect(sourceSheet).not.toContainText("Selection");
    await expect(sourceSheet).not.toContainText("Latest");
    await expect(sourceSheet).not.toContainText("Earliest");
    await expect(sourceSheet).not.toContainText("All");
    await sourceSheet.getByLabel("Alias").fill("program1");
    await closeDesignerSheet(page);
    await expect(page.getByRole("button", { name: /^program1$/ })).toBeVisible();

    await page.getByLabel("Convert program1 to expression").click();
    await closeDesignerSheet(page);
    await expect(page.getByRole("button", { name: /^program2$/ })).toBeVisible();
    await expect(page.getByRole("button", { name: "UNION" })).toBeVisible();
    await expect(fieldSourcePanel(page, "program1").locator("summary")).toContainText(
      "3 of",
    );

    await page.getByRole("button", { name: "UNION" }).click();
    const operationSheet = page.getByRole("dialog", {
      name: "Dataset designer options",
    });
    await operationSheet.getByLabel("Operation").selectOption("inner_join");
    await operationSheet.getByLabel("Left Join Key").selectOption({
      value: sourceFieldKey("program1", "__node_id"),
    });
    await operationSheet.getByLabel("Right Join Key").selectOption({
      value: sourceFieldKey("program2", "__node_id"),
    });
    await closeDesignerSheet(page);
    await expect(page.getByRole("button", { name: "INNER JOIN" })).toBeVisible();

    const program2Panel = await openFieldSourcePanel(page, "program2");
    await program2Panel.getByLabel("Include all fields from program2").uncheck();
    const program2NumberRow = program2Panel
      .locator("tbody tr", { hasText: numberField.label })
      .first();
    await program2NumberRow.getByLabel(`Include ${numberField.label}`).check();
    await program2NumberRow
      .getByLabel(`Display label for ${numberField.label}`)
      .fill("Program 2 Target");
    await expect(program2Panel.locator("summary")).toContainText("1 of");

    const aggregation = page.locator("section.dataset-aggregation-section");
    await aggregation.getByRole("button", { name: "Row", exact: true }).click();
    await expect(
      aggregation.locator(
        ".dataset-aggregation-panel--row .dataset-aggregation-selected-list li",
      ),
    ).toHaveCount(0);
    await selectComboboxOption(
      aggregation,
      "Add group field",
      sourceFieldKey("program1", "__node_id"),
      `Attached Node ID (${sourceFieldKey("program1", "__node_id")})`,
    );
    await selectComboboxOption(
      aggregation,
      "Add sort field",
      sourceFieldKey("program1", numberField.key),
      `${numberField.label} (${sourceFieldKey("program1", numberField.key)})`,
    );
    await aggregation
      .getByRole("button", { name: "Highest / latest first" })
      .click();
    await aggregation.getByRole("button", { name: "Field", exact: true }).click();
    await aggregation.getByRole("button", { name: "Add Metric" }).click();
    const metricRow = aggregation.locator(".dataset-aggregation-panel--metrics tbody tr").first();
    await metricRow.locator("select").first().selectOption("average");
    await metricRow
      .locator("select")
      .nth(1)
      .selectOption({ value: sourceFieldKey("program1", numberField.key) });
    const metricKey = metricRow.locator("input.form-control").nth(0);
    await metricKey.fill("avg_target");
    await metricKey.press("Tab");
    const metricLabel = metricRow.locator("input.form-control").nth(1);
    await metricLabel.fill("Average Target");
    await metricLabel.press("Tab");

    await expect(page.getByText("No filters configured.")).toBeVisible();

    await page.getByRole("button", { name: "Generated SQL" }).click();
    const sqlPanel = page.locator("pre.dataset-sql-panel code");
    await expect(sqlPanel).toContainText("INNER JOIN");
    await expect(sqlPanel).toContainText(
      `l."${sourceFieldKey("program1", "__node_id")}" = r."${sourceFieldKey(
        "program2",
        "__node_id",
      )}"`,
    );
    await expect(sqlPanel).toContainText("AVG");
    await expect(sqlPanel).toContainText("submission_value_fact.field_id");
    await expect(sqlPanel).not.toContainText("field_key");
    await expect(sqlPanel).not.toContainText("WITH ranked");

    const visibility = page.locator("section.dataset-editor-section", {
      has: page.getByRole("heading", { name: "Visibility" }),
    });
    await visibility.getByPlaceholder("Search nodes").fill("Community");
    await expect(visibility.locator(".dataset-visibility-node.is-search-match").first()).toBeVisible();

    const saveResponse = page.waitForResponse(
      (response) =>
        response.url().includes(`/api/admin/datasets/${datasetId}`) &&
        response.request().method() === "PUT",
    );
    await page.getByRole("button", { name: "Save Dataset" }).click();
    const response = await saveResponse;
    expect(response.ok()).toBeTruthy();
    await expect(page).toHaveURL(new RegExp(`/datasets/${datasetId}$`));

    const detail = await expectJson<DatasetDefinition>(
      await page.request.get(`/api/datasets/${datasetId}`),
    );
    expect(detail.sources.map((source) => source.source_alias).sort()).toEqual([
      "program1",
      "program2",
    ]);
    expect(detail.fields.map((field) => field.key)).toContain(
      sourceFieldKey("program2", numberField.key),
    );
    expect(detail.aggregation?.group_fields).toContain(
      sourceFieldKey("program1", "__node_id"),
    );
    expect(detail.aggregation?.metrics[0]).toMatchObject({
      key: "avg_target",
      label: "Average Target",
      function: "average",
      source_field_key: sourceFieldKey("program1", numberField.key),
    });
    expect(detail.generated_sql ?? "").toContain("submission_value_fact.field_id");
    expect(detail.generated_sql ?? "").not.toContain("field_key");
    expect(detail.generated_sql ?? "").not.toContain("WITH ranked");

    await page.getByRole("button", { name: "Fields" }).click();
    await expect(
      page.locator("th.data-table__stacked-label", { hasText: "Average Target" }),
    ).toContainText("avg_target");
    await page.getByRole("button", { name: "Preview" }).click();
    await expect(page.locator("table.data-table")).toContainText("Average Target");
    const table = await expectJson<DatasetTable>(
      await page.request.get(`/api/datasets/${datasetId}/table`),
    );
    expect(table.rows.length).toBeGreaterThan(0);
    const averageValue = table.rows
      .map((row) => row.values.avg_target)
      .find((value) => value !== null && value !== undefined);
    if (averageValue?.includes(".")) {
      await expect(page.locator("table.data-table")).toContainText(/\d+\.\d{2}/);
    }
  } finally {
    if (datasetId) {
      await deleteDataset(page, datasetId);
    }
    await assertNoConsoleErrors();
  }
});

test("dataset SQL preview uses pre-projection join keys and stable field identities", async ({
  page,
}) => {
  await signInAsAdmin(page);
  const seed = await seedDemo(page);
  const renderedForm = await expectJson<RenderedForm>(
    await page.request.get(`/api/form-versions/${seed.form_version_id}/render`),
  );
  const field = requireRenderedField(
    renderedFields(renderedForm),
    (candidate) => candidate.field_type === "number",
    "numeric field for SQL preview",
  );

  const preview = await expectJson<DatasetSqlPreview>(
    await page.request.post("/api/admin/datasets/sql-preview", {
      data: {
        name: "Playwright Joined Dataset",
        slug: `${PW_DATASET_PREFIX}sql-preview`,
        grain: "submission",
        composition_mode: "inner_join",
        visibility_node_ids: [seed.program_node_id],
        definition_ast: {
          kind: "operation",
          alias: "joined",
          operation: "inner_join",
          left: {
            kind: "form",
            alias: "left_source",
            form_id: seed.form_id,
            form_version_major: null,
          },
          right: {
            kind: "form",
            alias: "right_source",
            form_id: seed.form_id,
            form_version_major: null,
          },
          join_keys: [
            {
              left_field: sourceFieldKey("left_source", "__node_id"),
              right_field: sourceFieldKey("right_source", "__node_id"),
            },
          ],
        },
        aggregation: {
          group_fields: [sourceFieldKey("left_source", "__node_id")],
          metrics: [
            {
              key: "avg_value",
              label: "Average Value",
              function: "average",
              source_field_key: sourceFieldKey("left_source", field.key),
              position: 0,
            },
          ],
          row_picker: null,
        },
        fields: [
          datasetField("left_source", "__node_id", "Left Attached Node ID", 0),
          datasetField("left_source", field.key, `Left ${field.label}`, 1),
          datasetField("right_source", field.key, `Right ${field.label}`, 2),
        ],
      },
    }),
  );

  expect(preview.generated_sql).toContain("INNER JOIN");
  expect(preview.generated_sql).toContain(
    'l."left_source__node_id" = r."right_source__node_id"',
  );
  expect(preview.generated_sql).toContain("AVG");
  expect(preview.generated_sql).toContain("submission_value_fact.field_id");
  expect(preview.generated_sql).toContain("submission_value_fact.form_version_id");
  expect(preview.generated_sql).not.toContain("submission_value_fact.field_key");
  expect(preview.generated_sql).not.toContain("field_dim.field_key");
  expect(preview.generated_sql).not.toContain("WITH ranked");
  expect(preview.generated_sql).not.toContain("selection_rank");
});
