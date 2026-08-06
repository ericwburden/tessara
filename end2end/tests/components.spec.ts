import { expect, test, type APIResponse, type Page } from "@playwright/test";
import { invokeDemoSeedEndpoint } from "./support/demo-seed";

const RUN_ID = `pw-components-${Date.now()}`;

type DatasetReference = {
  reference: {
    installation_id: string;
    owner: { kind: "core_installation"; installation_id: string };
    resource_type: "tessara.transition.dataset_major_line";
    resource_id: string;
  };
};

type DatasetOption = {
  reference: DatasetReference;
  dataset_name: string;
  materialization_state?: string;
  fields: Array<{ key: string; label: string; field_type: string }>;
};

type DatasetSummary = {
  id: string;
  name: string;
  materialization_state: string;
  current_version_major?: number | null;
  major_versions?: number[];
  output_fields: Array<{ key: string; label: string; field_type: string }>;
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

type ValidationResponse = {
  schema_version: number;
  valid: boolean;
  findings: Array<{ code: string; field_path?: string | null }>;
};

async function expectJson<T>(response: APIResponse): Promise<T> {
  const text = await response.text();
  expect(response.ok(), `${response.url()} returned ${response.status()}: ${text}`).toBeTruthy();
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
      (response.status() === 400 && text.includes("Demo seed requires an empty database")),
    text,
  ).toBeTruthy();
}

async function datasetOption(page: Page) {
  const datasets = await expectJson<DatasetSummary[]>(await page.request.get("/api/datasets"));
  const dataset = datasets.find((candidate) => candidate.output_fields.length > 0);
  expect(dataset, "Component authoring requires one ready Dataset major line").toBeTruthy();
  const major = dataset!.major_versions?.[0] ?? dataset!.current_version_major;
  expect(major, `Dataset ${dataset!.name} must expose a positive major line`).toBeTruthy();
  const installationId = "01980000-0000-7000-8000-00000000008a";
  return {
    reference: {
      reference: {
        installation_id: installationId,
        owner: { kind: "core_installation" as const, installation_id: installationId },
        resource_type: "tessara.transition.dataset_major_line" as const,
        resource_id: `${dataset!.id}@${major}`,
      },
    },
    dataset_name: dataset!.name,
    materialization_state: dataset!.materialization_state ?? "ready",
    fields: dataset!.output_fields,
  };
}

function tableConfig(dataset: DatasetOption) {
  return { visible_columns: [dataset.fields[0].key], page_size: 25 };
}

function visualConfig(kind: string, fieldKey: string) {
  if (kind === "bar" || kind === "line") {
    return {
      mode: "summary",
      summary_field: fieldKey,
      summary_type: "count",
      ...(kind === "line" ? { x_field: fieldKey } : { category_field: fieldKey }),
      sort_field: "summary_value",
      sort_direction: "desc",
      number_of_points: 20,
      value_format: "integer",
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

async function publish(page: Page, definition: ComponentDefinition, action: string) {
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

test.describe.serial("Sprint 4A component workflow", () => {
  test("admin can create, update, publish, and view a major-line table component", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const dataset = await datasetOption(page);

    await page.goto("/components/new");
    await expect(page.getByRole("heading", { level: 1, name: "Create Component" })).toBeVisible();
    await expect(page.getByRole("combobox", { name: "Dataset major line" })).toBeVisible();
    await expect(page.getByRole("group", { name: "Displayed Fields" })).toHaveCount(0);
    const configuration = page.getByRole("textbox", { name: "Configuration JSON" });
    await expect(configuration).toHaveValue('{"visible_columns":[]}');
    await configuration.fill(JSON.stringify(tableConfig(dataset)));

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
    expect(invalid.findings[0]).toMatchObject({ code: "config.field_unavailable" });

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
            version: versionInput(dataset, "table", tableConfig(dataset), "Updated draft"),
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
    const rendered = await expectJson<{ materialization_state: string; component_type: string }>(
      await page.request.get(renderPath(definition.slug, "table")),
    );
    expect(rendered).toMatchObject({ materialization_state: "ready", component_type: "table" });

    for (const route of [
      `/components/${definition.slug}`,
      `/components/${definition.slug}/edit`,
      `/components/${definition.slug}/versions`,
      `/components/${definition.slug}/view`,
    ]) {
      await page.goto(route);
      await expect(page.getByRole("heading", { level: 1 }).first()).toBeVisible();
    }
  });

  test("admin can author, publish, and view visual components", async ({ page }) => {
    test.setTimeout(120_000);
    await signInAsAdmin(page);
    const dataset = await datasetOption(page);
    const fieldKey = dataset.fields[0].key;

    for (const kind of ["bar", "line", "pie", "donut", "stat_card"]) {
      const config = visualConfig(kind, fieldKey);
      const validation = await expectJson<ValidationResponse>(
        await page.request.post("/api/admin/components/validate", {
          data: versionInput(dataset, kind, config, `${kind} validation`),
        }),
      );
      expect(validation.valid, JSON.stringify(validation.findings)).toBe(true);

      const preview = await expectJson<{ component_type: string; materialization_state: string }>(
        await page.request.post("/api/admin/components/preview", {
          data: versionInput(dataset, kind, config, `${kind} preview`),
        }),
      );
      expect(preview.component_type).toBe(kind);
      expect(["ready", "pending"]).toContain(preview.materialization_state);

      const definition = await createComponent(page, dataset, kind, config, kind);
      await publish(page, definition, `publish-${kind}`);
      const rendered = await expectJson<{ component_type: string; materialization_state: string }>(
        await page.request.get(renderPath(definition.slug, kind)),
      );
      expect(rendered.component_type).toBe(kind);
      expect(rendered.materialization_state).toBe("ready");

      await page.goto(`/components/${definition.slug}/view`);
      await expect(page.getByRole("heading", { level: 1, name: definition.name })).toBeVisible();
    }
  });

  test("component editor exposes canonical configuration on mobile direct loads", async ({
    page,
  }) => {
    await signInAsAdmin(page);
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto("/components/new");
    await expect(page.getByRole("combobox", { name: "Dataset major line" })).toBeVisible();
    await expect(page.getByRole("textbox", { name: "Configuration JSON" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save draft" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Open mobile preview" })).toHaveCount(0);
    await page.reload();
    await expect(page.getByRole("heading", { level: 1, name: "Create Component" })).toBeVisible();
  });
});
