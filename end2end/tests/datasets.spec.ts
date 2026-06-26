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
  operations: DatasetOperation[];
  restriction_policy?: {
    tier_field_key?: string | null;
    internal_field_key?: string | null;
    restricted_field_key?: string | null;
    confidential_field_key?: string | null;
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

type DatasetSourcePayload =
  | {
      kind: "form";
      alias: string;
      form_id: string;
      form_version_id: string;
    }
  | {
      kind: "dataset";
      alias: string;
      dataset_id: string;
      dataset_revision_id: string;
    };

type DatasetProjectionFieldPayload = {
  key: string;
  label: string;
  input_field_key?: string | null;
  position: number;
};

type DatasetAggregationMetricPayload = {
  key: string;
  label: string;
  function: string;
  source_field_key?: string | null;
  position: number;
};

type DatasetRowPickerPayload = {
  sort_fields: Array<{ field_key: string; position: number }>;
  direction: "lowest" | "highest";
};

type DatasetCalculatedFieldPayload = {
  key: string;
  label: string;
  base_field_key: string;
  functions: Array<{
    function: string;
    argument?: string | null;
    argument_mode?: "value" | "field";
    argument_field_key?: string | null;
    position: number;
  }>;
  position: number;
};

type DatasetRowFilterPayload = {
  field_key: string;
  operator: string;
  value_mode: "value" | "field";
  value?: string | null;
  value_field_key?: string | null;
  position: number;
};

type DatasetOperation =
  | {
      kind: "join_source";
      source: DatasetSourcePayload;
      operation: string;
      join_keys: Array<{ left_field: string; right_field: string }>;
      position: number;
    }
  | {
      kind: "union_source";
      source: DatasetSourcePayload;
      position: number;
    }
  | {
      kind: "union_all_source";
      source: DatasetSourcePayload;
      position: number;
    }
  | {
      kind: "projection";
      fields: DatasetProjectionFieldPayload[];
      position: number;
    }
  | {
      kind: "aggregation";
      group_fields: string[];
      metrics: DatasetAggregationMetricPayload[];
      row_picker?: DatasetRowPickerPayload | null;
      position: number;
    }
  | {
      kind: "calculated_fields";
      fields: DatasetCalculatedFieldPayload[];
      position: number;
    }
  | {
      kind: "filter";
      filters: DatasetRowFilterPayload[];
      position: number;
    };

type DatasetPayload = {
  name: string;
  slug: string;
  grain: "submission";
  visibility_node_ids: string[];
  initial_source: DatasetSourcePayload;
  operations: DatasetOperation[];
  restriction_policy?: {
    internal_field_key?: string | null;
    restricted_field_key?: string | null;
    confidential_field_key?: string | null;
  } | null;
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
): DatasetProjectionFieldPayload {
  const key = sourceFieldKey(alias, sourceFieldKeyValue);
  return {
    key,
    label,
    input_field_key: key,
    position,
  };
}

function projectionOperation(
  fields: DatasetProjectionFieldPayload[],
  position = 0,
): DatasetOperation {
  return {
    kind: "projection",
    fields: fields.map((field, index) => ({
      key: field.key,
      label: field.label,
      input_field_key: field.input_field_key ?? field.key,
      position: field.position ?? index,
    })),
    position,
  };
}

function aggregationOperation(
  group_fields: string[],
  metrics: DatasetAggregationMetricPayload[],
  position: number,
  row_picker: DatasetRowPickerPayload | null = null,
): DatasetOperation {
  return {
    kind: "aggregation",
    group_fields,
    metrics,
    row_picker,
    position,
  };
}

function calculatedFieldsOperation(
  fields: DatasetCalculatedFieldPayload[],
  position: number,
): DatasetOperation {
  return {
    kind: "calculated_fields",
    fields,
    position,
  };
}

function filterOperation(
  filters: DatasetRowFilterPayload[],
  position: number,
): DatasetOperation {
  return {
    kind: "filter",
    filters,
    position,
  };
}

function detailOperation<K extends DatasetOperation["kind"]>(
  detail: DatasetDefinition,
  kind: K,
) {
  const operation = detail.operations.find((candidate) => candidate.kind === kind);
  expect(operation, `expected ${kind} operation to be persisted`).toBeTruthy();
  return operation as Extract<DatasetOperation, { kind: K }>;
}

async function createDataset(page: Page, payload: DatasetPayload) {
  const response = await page.request.post("/api/admin/datasets", {
    data: payload,
  });
  return (await expectJson<IdResponse>(response)).id;
}

async function updateDataset(page: Page, datasetId: string, payload: DatasetPayload) {
  const response = await page.request.put(`/api/admin/datasets/${datasetId}`, {
    data: payload,
  });
  expect(response.ok(), `dataset update returned ${response.status()}`).toBeTruthy();
}

async function deleteDataset(page: Page, datasetId: string, timeout = 20_000) {
  const response = await page.request.delete(`/api/admin/datasets/${datasetId}`, {
    timeout,
  });
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
  occurrence: number | "last" = 0,
) {
  const triggers = scope.getByRole("button", { name: ariaLabel, exact: true });
  const trigger = occurrence === "last" ? triggers.last() : triggers.nth(occurrence);
  await trigger.click();
  const combo = scope.locator(".combobox.is-open").last();
  const input = combo.locator("input.combobox__input");
  await expect(input).toBeFocused();
  await input.fill(search);
  await combo.getByRole("option", { name: optionName }).first().click();
  await expect(combo).toHaveCount(0);
}

async function openEditorSection(page: Page, heading: string) {
  const section = page.locator("section.dataset-editor-section", {
    has: page.getByRole("heading", { name: heading, exact: true }),
  });
  await expect(section).toBeVisible();
  const toggle = section.getByRole("button", { name: heading, exact: true }).first();
  if ((await toggle.getAttribute("aria-expanded")) === "false") {
    await toggle.click();
  }
  return section;
}

test("admin can author, edit, save, and view a Sprint 3A dataset", async ({
  page,
}) => {
  test.setTimeout(120_000);
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
  const booleanField = requireRenderedField(
    formFields,
    (field) => field.field_type === "boolean",
    "boolean field",
  );
  const runId = Date.now();
  const slug = `${PW_DATASET_PREFIX}${runId}`;
  const datasetName = `Playwright Dataset Authoring ${runId}`;
  const initialPayload: DatasetPayload = {
    name: datasetName,
    slug,
    grain: "submission",
    visibility_node_ids: [seed.program_node_id],
    initial_source: {
      kind: "form",
      alias: "program",
      form_id: seed.form_id,
      form_version_id: seed.form_version_id,
    },
    operations: [
      projectionOperation([
        datasetField("program", "__node_id", "Attached Node ID", 0),
        datasetField("program", numberField.key, numberField.label, 1),
        datasetField("program", textField.key, textField.label, 2),
        datasetField("program", booleanField.key, booleanField.label, 3),
      ]),
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
      await expect(
        page.getByRole("heading", { name: section, exact: true }),
      ).toBeVisible();
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
      "4 of",
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

    const aggregation = await openEditorSection(page, "Aggregation");
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

    const filters = await openEditorSection(page, "Filters");
    await expect(filters.getByText("No filters configured.")).toBeVisible();
    const calculations = await openEditorSection(page, "Calculated Fields");
    await calculations.getByRole("button", { name: "Add Calculated Field" }).click();
    const roundedCalculation = calculations.locator(".dataset-calculation-row").first();
    await roundedCalculation.getByLabel("Output Key").fill("avg_target_rounded");
    await roundedCalculation.getByLabel("Output Key").press("Tab");
    await roundedCalculation.getByLabel("Label").fill("Average Target Rounded");
    await roundedCalculation.getByLabel("Label").press("Tab");
    await roundedCalculation.getByLabel("Base Field").selectOption("avg_target");
    await roundedCalculation.getByRole("button", { name: "Add function" }).click();
    await selectComboboxOption(roundedCalculation, "Function", "round", "Round");
    await roundedCalculation.getByLabel("Argument").fill("1");
    await roundedCalculation.getByLabel("Argument").press("Tab");
    await expect(roundedCalculation.locator(".dataset-calculation-preview")).toContainText(
      "avg_target_rounded",
    );

    await calculations.getByRole("button", { name: "Add Calculated Field" }).click();
    const restrictionCalculation = calculations.locator(".dataset-calculation-row").nth(1);
    await restrictionCalculation.getByLabel("Output Key").fill("avg_target_restricted");
    await restrictionCalculation.getByLabel("Output Key").press("Tab");
    await restrictionCalculation
      .getByLabel("Label")
      .fill("Average Target Restricted");
    await restrictionCalculation.getByLabel("Label").press("Tab");
    await restrictionCalculation.getByLabel("Base Field").selectOption("avg_target");
    await restrictionCalculation.getByRole("button", { name: "Add function" }).click();
    await selectComboboxOption(
      restrictionCalculation,
      "Function",
      "greater",
      "Greater Than or Equal",
    );
    const restrictionArgument = restrictionCalculation
      .locator(".dataset-calculation-function")
      .first()
      .locator("label.form-field")
      .nth(1)
      .locator("input");
    await restrictionArgument.fill("0");
    await restrictionArgument.press("Tab");
    await expect(
      restrictionCalculation.locator(".dataset-calculation-preview"),
    ).toContainText("greater_than_or_equal(0)");

    const restrictions = await openEditorSection(page, "View Restrictions");
    await restrictions.getByLabel("Restricted flag enabled").check();
    await restrictions
      .getByLabel("Restricted flag field")
      .selectOption("avg_target_restricted");

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
    await expect(sqlPanel).toContainText("ROUND");
    await expect(sqlPanel).toContainText("__restriction_tier");
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
    expect(detail.operations.map((operation) => operation.kind)).toEqual([
      "projection",
      "aggregation",
      "calculated_fields",
    ]);
    const persistedAggregation = detailOperation(detail, "aggregation");
    expect(persistedAggregation.group_fields).toContain(
      sourceFieldKey("program1", "__node_id"),
    );
    expect(persistedAggregation.metrics[0]).toMatchObject({
      key: "avg_target",
      label: "Average Target",
      function: "average",
      source_field_key: sourceFieldKey("program1", numberField.key),
    });
    const persistedCalculations = detailOperation(detail, "calculated_fields");
    expect(persistedCalculations.fields[0]).toMatchObject({
      key: "avg_target_rounded",
      label: "Average Target Rounded",
      base_field_key: "avg_target",
    });
    expect(persistedCalculations.fields[1]).toMatchObject({
      key: "avg_target_restricted",
      label: "Average Target Restricted",
      base_field_key: "avg_target",
    });
    expect(detail.restriction_policy?.restricted_field_key).toBe("avg_target_restricted");
    expect(detail.generated_sql ?? "").toContain("submission_value_fact.field_id");
    expect(detail.generated_sql ?? "").toContain("ROUND");
    expect(detail.generated_sql ?? "").toContain("__restriction_tier");
    expect(detail.generated_sql ?? "").not.toContain("field_key");
    expect(detail.generated_sql ?? "").not.toContain("WITH ranked");

    await page.getByRole("button", { name: "Fields" }).click();
    await expect(
      page.getByRole("rowheader", { name: /^Average Target\s+avg_target$/ }),
    ).toContainText("avg_target");
    await expect(
      page.getByRole("rowheader", {
        name: /^Average Target Rounded\s+avg_target_rounded$/,
      }),
    ).toContainText("avg_target_rounded");
    await page.getByRole("button", { name: "Preview" }).click();
    await expect(page.locator("table.data-table")).toContainText("Average Target");
    await expect(page.locator("table.data-table")).toContainText("Average Target Rounded");
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
test("admin can UAT Sprint 3B advanced dataset authoring", async ({ page }) => {
  test.setTimeout(180_000);
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
  const booleanField = requireRenderedField(
    formFields,
    (field) => field.field_type === "boolean",
    "boolean field",
  );
  const dateFields = formFields.filter((field) =>
    ["date", "datetime", "timestamp"].includes(field.field_type),
  );
  expect(dateFields.length, "demo form should expose date-like fields").toBeGreaterThan(0);
  const dateField = dateFields[0];
  const alternateDateField = dateFields[1] ?? dateFields[0];
  const textField =
    formFields.find((field) => field.key === "submission_status") ??
    requireRenderedField(formFields, (field) => field.field_type === "text", "text field");
  const runId = Date.now();
  const slug = `${PW_DATASET_PREFIX}uat-${runId}`;
  const datasetName = `Playwright Sprint 3B UAT ${runId}`;
  const initialFields: DatasetProjectionFieldPayload[] = [
    datasetField("program", "__node_id", "Attached Node ID", 0),
    datasetField("program", numberField.key, numberField.label, 1),
    datasetField("program", booleanField.key, booleanField.label, 2),
    datasetField("program", dateField.key, dateField.label, 3),
    datasetField("program", textField.key, textField.label, 4),
  ];
  if (
    !initialFields.some(
      (field) => field.input_field_key === sourceFieldKey("program", alternateDateField.key),
    )
  ) {
    initialFields.push({
      ...datasetField(
        "program",
        alternateDateField.key,
        alternateDateField.label,
        initialFields.length,
      ),
    });
  }
  const initialFieldCount = initialFields.length;
  const initialPayload: DatasetPayload = {
    name: datasetName,
    slug,
    grain: "submission",
    visibility_node_ids: [seed.program_node_id],
    initial_source: {
      kind: "form",
      alias: "program",
      form_id: seed.form_id,
      form_version_id: seed.form_version_id,
    },
    operations: [projectionOperation(initialFields)],
  };

  let datasetId: string | undefined;
  try {
    await test.step("Demo: open editor with hidden helper fields available", async () => {
      datasetId = await createDataset(page, initialPayload);
      await page.goto(`/datasets/${datasetId}/edit`);
      await expect(
        page.getByRole("heading", { level: 1, name: "Edit Dataset" }),
      ).toBeVisible();
      await expect(fieldSourcePanel(page, "program").locator("summary")).toContainText(
        `${initialFieldCount} of`,
      );

      for (const heading of [
        "Aggregation",
        "Calculated Fields",
        "Filters",
        "View Restrictions",
      ]) {
        await expect(
          page
            .locator("section.dataset-editor-section", {
              has: page.getByRole("heading", { name: heading, exact: true }),
            })
            .getByRole("button", { name: heading, exact: true })
            .first(),
        ).toHaveAttribute("aria-expanded", "false");
      }
    });

    await test.step("Demo: add a second source without clearing existing field selections", async () => {
      await page.getByRole("button", { name: "Add Input" }).click();
      const sourceSheet = page.getByRole("dialog", {
        name: "Dataset designer options",
      });
      await expect(sourceSheet).toBeVisible();
      await sourceSheet.getByLabel("Alias").fill("program2");
      await sourceSheet.getByLabel("Alias").press("Tab");
      await sourceSheet
        .locator("label.form-field")
        .nth(2)
        .locator("select")
        .selectOption({ value: seed.form_id });
      await sourceSheet
        .locator("label.form-field")
        .nth(3)
        .locator("select")
        .selectOption({ value: seed.form_version_id });
      await closeDesignerSheet(page);
      await expect(page.getByRole("button", { name: /^program2$/ })).toBeVisible();
      await expect(fieldSourcePanel(page, "program").locator("summary")).toContainText(
        `${initialFieldCount} of`,
      );
      await expect(fieldSourcePanel(page, "program").locator("summary")).not.toContainText(
        "0 of",
      );
      await expect(fieldSourcePanel(page, "program2").locator("summary")).toContainText(
        /[1-9]\d* of [1-9]\d*/,
      );
    });

    await test.step("Demo: configure the source operation before projection", async () => {
      await page.getByRole("button", { name: "UNION" }).click();
      const operationSheet = page.getByRole("dialog", {
        name: "Dataset designer options",
      });
      await operationSheet.getByLabel("Operation").selectOption("inner_join");
      await operationSheet.getByLabel("Left Join Key").selectOption({
        value: sourceFieldKey("program", "__node_id"),
      });
      await operationSheet.getByLabel("Right Join Key").selectOption({
        value: sourceFieldKey("program2", "__node_id"),
      });
      await closeDesignerSheet(page);
      await expect(page.getByRole("button", { name: "INNER JOIN" })).toBeVisible();
    });

    await test.step("Demo: add typed calculated-field pipelines", async () => {
      const calculations = await openEditorSection(page, "Calculated Fields");
      await calculations.getByRole("button", { name: "Add Calculated Field" }).click();
      const calculation = calculations.locator(".dataset-calculation-row").first();
      await calculation.getByLabel("Output Key").fill("review_started_together");
      await calculation.getByLabel("Output Key").press("Tab");
      await calculation.getByLabel("Label").fill("Review Started Together");
      await calculation.getByLabel("Label").press("Tab");
      await calculation
        .getByLabel("Base Field")
        .selectOption(sourceFieldKey("program", dateField.key));
      await calculation.getByRole("button", { name: "Add function" }).click();
      await selectComboboxOption(
        calculation,
        "Function",
        "less",
        "Less Than or Equal",
      );
      await calculation.getByRole("button", { name: "Use a value argument" }).click();
      await calculation
        .locator(".dataset-calculation-function")
        .first()
        .locator("label.form-field")
        .nth(1)
        .locator("select")
        .selectOption(sourceFieldKey("program2", alternateDateField.key));
      await expect(calculation.locator(".dataset-calculation-preview")).toContainText(
        `less_than_or_equal(${sourceFieldKey("program2", alternateDateField.key)})`,
      );

      await calculations.getByRole("button", { name: "Add Calculated Field" }).click();
      const numericCalculation = calculations.locator(".dataset-calculation-row").nth(1);
      await numericCalculation.getByLabel("Output Key").fill("target_band");
      await numericCalculation.getByLabel("Output Key").press("Tab");
      await numericCalculation.getByLabel("Label").fill("Target Band");
      await numericCalculation.getByLabel("Label").press("Tab");
      await numericCalculation
        .getByLabel("Base Field")
        .selectOption(sourceFieldKey("program", numberField.key));
      await numericCalculation.getByRole("button", { name: "Add function" }).click();
      await selectComboboxOption(
        numericCalculation,
        "Function",
        "greater",
        "Greater Than or Equal",
      );
      const numericArgument = numericCalculation
        .locator(".dataset-calculation-function")
        .first()
        .locator("label.form-field")
        .nth(1)
        .locator("input");
      await numericArgument.fill("120");
      await numericArgument.press("Tab");
      await numericCalculation.getByRole("button", { name: "Add function" }).last().click();
      await selectComboboxOption(numericCalculation, "Function", "text", "Cast to Text", "last");
      await expect(numericCalculation.locator(".dataset-calculation-preview")).toContainText(
        "greater_than_or_equal(120) | to_text",
      );

      await calculations.getByRole("button", { name: "Add Calculated Field" }).click();
      const mapCalculation = calculations.locator(".dataset-calculation-row").nth(2);
      await mapCalculation.getByLabel("Output Key").fill("status_mapped");
      await mapCalculation.getByLabel("Output Key").press("Tab");
      await mapCalculation.getByLabel("Label").fill("Status Mapped");
      await mapCalculation.getByLabel("Label").press("Tab");
      await mapCalculation
        .getByLabel("Base Field")
        .selectOption(sourceFieldKey("program", textField.key));
      await mapCalculation.getByRole("button", { name: "Add function" }).click();
      await selectComboboxOption(mapCalculation, "Function", "map", "Map Value");
      const mapArgument = mapCalculation
        .locator(".dataset-calculation-function")
        .first()
        .locator("label.form-field")
        .nth(1)
        .locator("input");
      await mapArgument.fill("draft=>booger, submitted=>snot");
      await mapArgument.press("Tab");
      await expect(mapCalculation.locator(".dataset-calculation-preview")).toContainText(
        "map_value(draft=>booger, submitted=>snot)",
      );

      await expect(calculations.getByText("Add Function", { exact: true })).toHaveCount(0);
      await expect(calculations.getByText("Remove", { exact: true })).toHaveCount(0);
      await expect(
        calculations.getByRole("button", { name: /Remove calculated field/ }),
      ).toHaveCount(3);
      await expect(calculations.getByRole("button", { name: /Add function/ })).toHaveCount(7);
    });

    await test.step("Demo: add literal and field-comparison filters after calculations", async () => {
      const filters = await openEditorSection(page, "Filters");
      await filters.getByRole("button", { name: "Add Filter" }).click();
      let filterRows = filters.locator(".dataset-filter-row");
      const numberFilter = filterRows.first();
      await numberFilter
        .locator("select")
        .nth(0)
        .selectOption(sourceFieldKey("program", numberField.key));
      await numberFilter.locator("select").nth(1).selectOption("greater_than_or_equal");
      const numberFilterValue = numberFilter.locator("label.form-field").nth(2).locator("input");
      await numberFilterValue.fill("0");
      await numberFilterValue.press("Tab");
      await filters.getByRole("button", { name: "Add Filter" }).click();
      filterRows = filters.locator(".dataset-filter-row");
      const dateFilter = filterRows.nth(1);
      await dateFilter
        .locator("select")
        .nth(0)
        .selectOption(sourceFieldKey("program", dateField.key));
      await dateFilter.locator("select").nth(1).selectOption("less_than_or_equal");
      await dateFilter.getByRole("button", { name: "Compare against a value" }).click();
      await dateFilter
        .locator("label.form-field")
        .nth(2)
        .locator("select")
        .selectOption(sourceFieldKey("program2", alternateDateField.key));
      await expect(filters.getByRole("button", { name: "Compare against a field" })).toHaveCount(1);
      await expect(filters.getByRole("button", { name: "Compare against a value" })).toHaveCount(1);
      await expect(filters.getByText("Remove", { exact: true })).toHaveCount(0);
      await expect(filters.getByRole("button", { name: /Remove filter/ })).toHaveCount(2);
    });

    await test.step("Demo: configure row-tier restriction flags", async () => {
      const restrictions = await openEditorSection(page, "View Restrictions");
      await restrictions.getByLabel("Internal flag enabled").check();
      await restrictions
        .getByLabel("Internal flag field")
        .selectOption(sourceFieldKey("program", booleanField.key));
      await restrictions.getByLabel("Restricted flag enabled").check();
      await restrictions
        .getByLabel("Restricted flag field")
        .selectOption(sourceFieldKey("program2", booleanField.key));
    });

    await test.step("Demo: inspect generated SQL and save the authored definition", async () => {
      await page.getByRole("button", { name: "Generated SQL" }).click();
      const sqlPanel = page.locator("pre.dataset-sql-panel code");
      await expect(sqlPanel).toContainText("INNER JOIN");
      await expect(sqlPanel).toContainText("NULLIF");
      await expect(sqlPanel).toContainText("::date");
      await expect(sqlPanel).toContainText("__restriction_tier");
      await expect(sqlPanel).toContainText("booger");
      await expect(sqlPanel).toContainText("snot");
      await expect(sqlPanel).not.toContainText("field_key");

      const saveResponse = page.waitForResponse(
        (response) =>
          response.url().includes(`/api/admin/datasets/${datasetId}`) &&
          response.request().method() === "PUT",
      );
      await page.getByRole("button", { name: "Save Dataset" }).click();
      const response = await saveResponse;
      expect(response.ok()).toBeTruthy();
      await expect(page).toHaveURL(new RegExp(`/datasets/${datasetId}$`));
    });

    await test.step("Demo evidence: verify persisted state and reopen hydration", async () => {
      const detail = await expectJson<DatasetDefinition>(
        await page.request.get(`/api/datasets/${datasetId}`),
      );
      expect(detail.fields.filter((field) => field.source_alias === "program").length).toBe(
        initialFieldCount,
      );
      expect(
        detail.fields.filter((field) => field.source_alias === "program2").length,
      ).toBeGreaterThan(0);
      expect(detail.operations.map((operation) => operation.kind)).toEqual([
        "projection",
        "calculated_fields",
        "filter",
      ]);
      const persistedFilters = detailOperation(detail, "filter");
      expect(persistedFilters.filters).toMatchObject([
        {
          field_key: sourceFieldKey("program", numberField.key),
          operator: "greater_than_or_equal",
          value_mode: "value",
          value: "0",
        },
        {
          field_key: sourceFieldKey("program", dateField.key),
          operator: "less_than_or_equal",
          value_mode: "field",
          value_field_key: sourceFieldKey("program2", alternateDateField.key),
        },
      ]);
      const persistedCalculations = detailOperation(detail, "calculated_fields");
      expect(persistedCalculations.fields.map((field) => field.key)).toEqual([
        "review_started_together",
        "target_band",
        "status_mapped",
      ]);
      expect(persistedCalculations.fields[0].functions[0]).toMatchObject({
        function: "less_than_or_equal",
        argument_mode: "field",
        argument_field_key: sourceFieldKey("program2", alternateDateField.key),
      });
      expect(persistedCalculations.fields[1].functions.map((fn) => fn.function)).toEqual([
        "greater_than_or_equal",
        "to_text",
      ]);
      expect(detail.restriction_policy?.internal_field_key).toBe(
        sourceFieldKey("program", booleanField.key),
      );
      expect(detail.restriction_policy?.restricted_field_key).toBe(
        sourceFieldKey("program2", booleanField.key),
      );

      await page.goto(`/datasets/${datasetId}/edit`);
      await expect(fieldSourcePanel(page, "program").locator("summary")).toContainText(
        `${initialFieldCount} of`,
      );
      const reopenedCalculations = await openEditorSection(page, "Calculated Fields");
      await expect(reopenedCalculations).toContainText("review_started_together");
      const reopenedFilters = await openEditorSection(page, "Filters");
      await expect(reopenedFilters.locator(".dataset-filter-row")).toHaveCount(2);
    });

    await assertNoConsoleErrors();
  } finally {
    if (datasetId) {
      await deleteDataset(page, datasetId);
    }
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
        visibility_node_ids: [seed.program_node_id],
        initial_source: {
          kind: "form",
          alias: "left_source",
          form_id: seed.form_id,
          form_version_id: seed.form_version_id,
        },
        operations: [
          {
            kind: "join_source",
            source: {
              kind: "form",
              alias: "right_source",
              form_id: seed.form_id,
              form_version_id: seed.form_version_id,
            },
            operation: "inner_join",
            join_keys: [
              {
                left_field: sourceFieldKey("left_source", "__node_id"),
                right_field: sourceFieldKey("right_source", "__node_id"),
              },
            ],
            position: 0,
          },
          projectionOperation([
            datasetField("left_source", "__node_id", "Left Attached Node ID", 0),
            datasetField("left_source", field.key, `Left ${field.label}`, 1),
            datasetField("right_source", field.key, `Right ${field.label}`, 2),
          ], 1),
          aggregationOperation(
            [sourceFieldKey("left_source", "__node_id")],
            [
              {
                key: "avg_value",
                label: "Average Value",
                function: "average",
                source_field_key: sourceFieldKey("left_source", field.key),
                position: 0,
              },
            ],
            2,
          ),
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

test("dataset SQL preview renders ordered QuerySpec operations as sequential CTEs", async ({
  page,
}) => {
  await signInAsAdmin(page);
  const seed = await seedDemo(page);
  const renderedForm = await expectJson<RenderedForm>(
    await page.request.get(`/api/form-versions/${seed.form_version_id}/render`),
  );
  const formFields = renderedFields(renderedForm);
  const numberField = requireRenderedField(
    formFields,
    (candidate) => candidate.field_type === "number",
    "numeric field for ordered operation preview",
  );
  const numberKey = sourceFieldKey("program", numberField.key);

  const preview = await expectJson<DatasetSqlPreview>(
    await page.request.post("/api/admin/datasets/sql-preview", {
      data: {
        name: "Playwright Ordered Operations Dataset",
        slug: `${PW_DATASET_PREFIX}ordered-operations`,
        grain: "submission",
        visibility_node_ids: [seed.program_node_id],
        initial_source: {
          kind: "form",
          alias: "program",
          form_id: seed.form_id,
          form_version_id: seed.form_version_id,
        },
        operations: [
          projectionOperation([
            datasetField("program", numberField.key, numberField.label, 0),
          ]),
          filterOperation(
            [
              {
                field_key: numberKey,
                operator: "greater_than_or_equal",
                value_mode: "value",
                value: "0",
                position: 0,
              },
            ],
            1,
          ),
          calculatedFieldsOperation(
            [
              {
                key: "target_allowed",
                label: "Target Allowed",
                base_field_key: numberKey,
                functions: [
                  {
                    function: "greater_than_or_equal",
                    argument_mode: "value",
                    argument: "100",
                    position: 0,
                  },
                ],
                position: 0,
              },
              {
                key: "target_label",
                label: "Target Label",
                base_field_key: numberKey,
                functions: [
                  {
                    function: "to_text",
                    argument_mode: "value",
                    position: 0,
                  },
                ],
                position: 1,
              },
            ],
            2,
          ),
          filterOperation(
            [
              {
                field_key: "target_label",
                operator: "is_not_empty",
                value_mode: "value",
                position: 0,
              },
            ],
            3,
          ),
          projectionOperation(
            [
              {
                key: "target_label",
                label: "Target Label",
                input_field_key: "target_label",
                position: 0,
              },
              {
                key: "target_allowed",
                label: "Target Allowed",
                input_field_key: "target_allowed",
                position: 1,
              },
            ],
            4,
          ),
        ],
        restriction_policy: {
          restricted_field_key: "target_allowed",
        },
      },
    }),
  );

  const sql = preview.generated_sql;
  const projectionStart = sql.indexOf("projection_2 AS");
  const firstFilterStart = sql.indexOf("filtered_fields_3 AS");
  const calculationStart = sql.indexOf("calculated_fields_4 AS");
  const secondFilterStart = sql.indexOf("filtered_fields_5 AS");
  const finalProjectionStart = sql.indexOf("projection_6 AS");
  expect(projectionStart).toBeGreaterThan(-1);
  expect(firstFilterStart).toBeGreaterThan(projectionStart);
  expect(calculationStart).toBeGreaterThan(firstFilterStart);
  expect(secondFilterStart).toBeGreaterThan(calculationStart);
  expect(finalProjectionStart).toBeGreaterThan(secondFilterStart);
  expect(sql).toContain("target_allowed");
  expect(sql).toContain(">= NULLIF('100', '')::numeric");
  expect(sql).toContain("__restriction_tier");
  expect(sql.indexOf("__restriction_tier")).toBeGreaterThan(finalProjectionStart);
  expect(sql).toContain('FROM "projection_6"');
});
test("dataset operations keep operation-local state through reorder, save, and reload", async ({
  page,
}) => {
  test.setTimeout(300_000);
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
    "numeric field for operation-local UAT",
  );
  const statusField =
    formFields.find((field) => field.key === "submission_status") ??
    requireRenderedField(
      formFields,
      (field) => field.field_type === "text",
      "text field for operation-local UAT",
    );
  const numberKey = sourceFieldKey("program", numberField.key);
  const statusKey = sourceFieldKey("program", statusField.key);
  const runId = Date.now();
  const slug = `${PW_DATASET_PREFIX}operation-local-${runId}`;
  const basePayload = {
    name: `Playwright Operation Local UAT ${runId}`,
    slug,
    grain: "submission",
    visibility_node_ids: [seed.program_node_id],
    initial_source: {
      kind: "form" as const,
      alias: "program",
      form_id: seed.form_id,
      form_version_id: seed.form_version_id,
    },
  };
  const projection = projectionOperation([
    datasetField("program", statusField.key, statusField.label, 0),
    datasetField("program", numberField.key, numberField.label, 1),
  ]);
  const statusCalculation = calculatedFieldsOperation(
    [
      {
        key: "status_upper",
        label: "Status Upper",
        base_field_key: statusKey,
        functions: [
          {
            function: "uppercase",
            argument_mode: "value",
            position: 0,
          },
        ],
        position: 0,
      },
    ],
    1,
  );
  const targetCalculation = calculatedFieldsOperation(
    [
      {
        key: "target_plus_one",
        label: "Target Plus One",
        base_field_key: numberKey,
        functions: [
          {
            function: "add",
            argument_mode: "value",
            argument: "1",
            position: 0,
          },
        ],
        position: 0,
      },
    ],
    2,
  );
  const statusFilter = filterOperation(
    [
      {
        field_key: statusKey,
        operator: "equals",
        value_mode: "value",
        value: "submitted",
        position: 0,
      },
    ],
    3,
  );
  const targetFilter = filterOperation(
    [
      {
        field_key: "target_plus_one",
        operator: "greater_than_or_equal",
        value_mode: "value",
        value: "1",
        position: 0,
      },
    ],
    4,
  );
  const statusAggregation = aggregationOperation(
    ["status_upper"],
    [
      {
        key: "status_count",
        label: "Status Count",
        function: "count_rows",
        position: 0,
      },
    ],
    5,
  );
  const countAggregation = aggregationOperation(
    ["status_upper"],
    [
      {
        key: "max_status_count",
        label: "Max Status Count",
        function: "max",
        source_field_key: "status_count",
        position: 0,
      },
    ],
    6,
  );

  let datasetId: string | undefined;
  try {
    datasetId = await createDataset(page, {
      ...basePayload,
      operations: [
        projection,
        statusCalculation,
        targetCalculation,
        statusFilter,
        targetFilter,
        statusAggregation,
        countAggregation,
      ],
    });

    await updateDataset(page, datasetId, {
      ...basePayload,
      operations: [
        projectionOperation(projection.fields, 0),
        calculatedFieldsOperation(targetCalculation.fields, 1),
        calculatedFieldsOperation(statusCalculation.fields, 2),
        filterOperation(targetFilter.filters, 3),
        filterOperation(statusFilter.filters, 4),
        statusAggregation,
        countAggregation,
      ],
    });

    const detail = await expectJson<DatasetDefinition>(
      await page.request.get(`/api/datasets/${datasetId}`),
    );
    expect(detail.operations.map((operation) => operation.kind)).toEqual([
      "projection",
      "calculated_fields",
      "calculated_fields",
      "filter",
      "filter",
      "aggregation",
      "aggregation",
    ]);

    const calculations = detail.operations.filter(
      (operation): operation is Extract<DatasetOperation, { kind: "calculated_fields" }> =>
        operation.kind === "calculated_fields",
    );
    expect(calculations[0].fields[0]).toMatchObject({
      key: "target_plus_one",
      base_field_key: numberKey,
    });
    expect(calculations[0].fields[0].functions[0]).toMatchObject({
      function: "add",
      argument: "1",
    });
    expect(calculations[1].fields[0]).toMatchObject({
      key: "status_upper",
      base_field_key: statusKey,
    });
    expect(calculations[1].fields[0].functions[0].function).toBe("uppercase");

    const filters = detail.operations.filter(
      (operation): operation is Extract<DatasetOperation, { kind: "filter" }> =>
        operation.kind === "filter",
    );
    expect(filters[0].filters[0]).toMatchObject({
      field_key: "target_plus_one",
      operator: "greater_than_or_equal",
      value: "1",
    });
    expect(filters[1].filters[0]).toMatchObject({
      field_key: statusKey,
      operator: "equals",
      value: "submitted",
    });

    const aggregations = detail.operations.filter(
      (operation): operation is Extract<DatasetOperation, { kind: "aggregation" }> =>
        operation.kind === "aggregation",
    );
    expect(aggregations[0].metrics[0]).toMatchObject({
      key: "status_count",
      function: "count_rows",
    });
    expect(aggregations[1].metrics[0]).toMatchObject({
      key: "max_status_count",
      function: "max",
      source_field_key: "status_count",
    });

    const sql = detail.generated_sql ?? "";
    const targetCalcStart = sql.indexOf("target_plus_one");
    const statusCalcStart = sql.indexOf("status_upper");
    const targetFilterStart = sql.indexOf("filtered_fields_5 AS");
    const statusFilterStart = sql.indexOf("filtered_fields_6 AS");
    const firstAggregationStart = sql.indexOf("aggregation_7 AS");
    const secondAggregationStart = sql.indexOf("aggregation_8 AS");
    expect(targetCalcStart).toBeGreaterThan(-1);
    expect(statusCalcStart).toBeGreaterThan(-1);
    expect(targetFilterStart).toBeGreaterThan(statusCalcStart);
    expect(statusFilterStart).toBeGreaterThan(targetFilterStart);
    expect(firstAggregationStart).toBeGreaterThan(statusFilterStart);
    expect(secondAggregationStart).toBeGreaterThan(firstAggregationStart);

    await page.goto(`/datasets/${datasetId}/edit`);
    const firstCalcPanel = page
      .locator("article.dataset-operation-panel", { hasText: "Calculated Fields" })
      .nth(0);
    await firstCalcPanel.locator("button.dataset-operation-panel__toggle").click();
    await expect(firstCalcPanel.locator("input").first()).toHaveValue("target_plus_one");
    const secondCalcPanel = page
      .locator("article.dataset-operation-panel", { hasText: "Calculated Fields" })
      .nth(1);
    await secondCalcPanel.locator("button.dataset-operation-panel__toggle").click();
    await expect(secondCalcPanel.locator("input").first()).toHaveValue("status_upper");
  } finally {
    if (datasetId) {
      await deleteDataset(page, datasetId, 5_000).catch((error: unknown) => {
        console.warn(`dataset cleanup failed for ${datasetId}: ${String(error)}`);
      });
    }
  }
});
