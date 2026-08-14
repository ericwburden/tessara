import {
  expect,
  test,
  type APIResponse,
  type Page,
  type Response,
} from "@playwright/test";

type DatasetSummary = {
  id: string;
  name: string;
  slug: string;
};

type DatasetTable = {
  rows: Array<{
    submission_id: string;
    node_name: string;
    source_alias: string;
    values: Record<string, string | null>;
  }>;
};

type DatasetDetail = {
  id: string;
  name: string;
  slug: string;
  grain: string;
  visibility_nodes: Array<{ node_id: string }>;
  initial_source: Record<string, unknown> | null;
  operations: Array<Record<string, unknown>>;
  restriction_policy: Record<string, unknown> | null;
};

type DatasetRefresh = {
  dataset_id: string;
  changed: boolean;
  freshness: { state: string };
  materialization_receipt_ids: string[];
};

type OperationsStatus = {
  dataset_readiness: {
    state: "available" | "empty" | "unavailable" | "undisclosed";
    datasets: Array<Record<string, unknown>>;
  };
};

type FormSummary = { id: string; slug: string };

type FormDetail = {
  dataset_sources_state: "available" | "empty" | "unavailable" | "undisclosed";
  dataset_sources: Array<Record<string, unknown>>;
};

async function expectJson<T>(response: APIResponse | Response): Promise<T> {
  const body = await response.text();
  expect(
    response.ok(),
    `${response.status()} ${response.url()}: ${body}`,
  ).toBeTruthy();
  return JSON.parse(body) as T;
}

function requireDataset(datasets: DatasetSummary[], slug: string): DatasetSummary {
  const dataset = datasets.find((candidate) => candidate.slug === slug);
  expect(dataset, `Dataset fixture '${slug}'`).toBeDefined();
  return dataset!;
}

async function signIn(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

test.describe("Sprint 8B independent Dataset module", () => {
  test.beforeEach(async ({ page }) => signIn(page));

  test("editor options use only Dataset-owned browser routes", async ({ page }) => {
    let auditDatasetTraffic = true;
    const forbidden: string[] = [];
    const external: string[] = [];
    const ownedOptions = new Set<string>();
    let applicationOrigin: string | undefined;
    page.on("request", (request) => {
      const url = new URL(request.url());
      const path = url.pathname;
      if (
        request.isNavigationRequest() &&
        request.frame() === page.mainFrame() &&
        path === "/datasets/new"
      ) {
        applicationOrigin ??= url.origin;
      }
      if (!auditDatasetTraffic) {
        return;
      }
      if (
        applicationOrigin !== undefined &&
        (url.protocol === "http:" || url.protocol === "https:") &&
        url.origin !== applicationOrigin
      ) {
        external.push(request.url());
      }
      if (
        path === "/api/me" ||
        path === "/api/forms" ||
        path === "/api/nodes" ||
        path === "/api/admin/users" ||
        path.startsWith("/api/form-versions/")
      ) {
        forbidden.push(path);
      }
      if (path.startsWith("/api/admin/datasets/editor-options/")) {
        ownedOptions.add(path);
      }
    });

    await page.goto("/datasets/new");
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    await expect(page.getByRole("heading", { name: /create dataset/i })).toBeVisible();
    expect(forbidden).toEqual([]);
    expect(external).toEqual([]);
    expect([...ownedOptions].sort()).toEqual([
      "/api/admin/datasets/editor-options/forms",
      "/api/admin/datasets/editor-options/principals",
      "/api/admin/datasets/editor-options/scopes",
    ]);

    await page.getByRole("button", { name: /initial data source/i }).click();
    const formPicker = page.getByRole("combobox", { name: "Form", exact: true }).first();
    await expect(formPicker).toContainText("Primary Responses");
    const schemaResponse = page.waitForResponse((response) =>
      response.request().method() === "GET" &&
      /^\/api\/admin\/datasets\/editor-options\/forms\/[^/]+$/.test(
        new URL(response.url()).pathname,
      ),
    );
    await formPicker.selectOption({ label: "Primary Responses" });
    const renderedForm = await expectJson<{
      form_name: string;
      sections: Array<{ fields: Array<{ key: string; field_type: string }> }>;
    }>(await schemaResponse);
    expect(renderedForm.form_name).toBe("Primary Responses");
    expect(
      renderedForm.sections.flatMap((section) => section.fields).map((field) => [
        field.key,
        field.field_type,
      ]),
    ).toEqual([
      ["label", "text"],
      ["amount", "number"],
      ["category", "text"],
    ]);
    const versionPicker = page.getByRole("combobox", { name: "Version", exact: true }).first();
    await expect(versionPicker.locator("option")).not.toHaveCount(0);
    expect([...ownedOptions].some((path) => /^\/api\/admin\/datasets\/editor-options\/forms\/[^/]+$/.test(path))).toBe(true);
    expect(forbidden).toEqual([]);
    expect(external).toEqual([]);

    // Enter through Core once so the browser lifecycle ABI, rather than the
    // complete-document fallback, owns subsequent Dataset navigation.
    auditDatasetTraffic = false;
    await page.goto("/");
    await expect(page.locator("#app-root")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    auditDatasetTraffic = true;
    await page.getByRole("link", { name: "Datasets", exact: true }).click();
    await expect(page).toHaveURL(/\/datasets$/);
    await expect(page.locator("#tessara-module-outlet")).toBeVisible();
    await expect(page.getByRole("heading", { name: "Datasets", exact: true })).toBeVisible();

    // Clean intra-module navigation is allowed, while a changed editor must
    // stay mounted when the user declines the lifecycle guard prompt.
    await page.getByRole("link", { name: "Create Dataset", exact: true }).click();
    await expect(page).toHaveURL(/\/datasets\/new$/);
    await expect(page.getByRole("heading", { name: "Create Dataset", exact: true })).toBeVisible();
    const lifecycleDraftName = `Lifecycle Draft ${Date.now()}`;
    const lifecycleName = page.getByLabel("Name", { exact: true });
    await lifecycleName.fill(lifecycleDraftName);
    await Promise.all([
      page.waitForEvent("dialog").then(async (dialog) => {
        expect(dialog.type()).toBe("confirm");
        expect(dialog.message()).toBe("Discard unsaved Dataset changes?");
        await dialog.dismiss();
      }),
      page.getByRole("link", { name: "Home", exact: true }).click(),
    ]);
    await expect(page).toHaveURL(/\/datasets\/new$/);
    await expect(lifecycleName).toHaveValue(lifecycleDraftName);
    expect(forbidden).toEqual([]);
    expect(external).toEqual([]);
  });

  test("synchronous refresh preserves last-good data and atomically promotes the full Dataset dependency closure", async ({
    page,
  }) => {
    const datasets = await expectJson<DatasetSummary[]>(
      await page.request.get("/api/datasets"),
    );
    const base = requireDataset(datasets, "base-responses");
    const derived = requireDataset(datasets, "derived-responses");
    const secondHop = requireDataset(datasets, "derived-second-hop");
    const independent = requireDataset(datasets, "independent-responses");
    const before = await Promise.all(
      [base, derived, secondHop, independent].map((dataset) =>
        page.request
          .get(`/api/datasets/${dataset.id}/table`)
          .then((response) => expectJson<DatasetTable>(response)),
      ),
    );

    await page.goto(`/datasets/${base.id}`);
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    const preview = page.locator("[data-dataset-preview]");
    await expect(preview).toBeVisible();
    await expect(preview).not.toHaveText("");

    const refreshResponse = page.waitForResponse((response) => {
      const url = new URL(response.url());
      return (
        response.request().method() === "POST" &&
        url.pathname === `/api/admin/datasets/${base.id}/refresh`
      );
    });
    await page.getByRole("button", { name: "Refresh now" }).click();
    const refresh = await expectJson<DatasetRefresh>(await refreshResponse);
    expect(refresh.dataset_id).toBe(base.id);
    expect(refresh.freshness.state).toBe("current");
    await expect(
      page.getByText(
        refresh.changed ? "Dataset refresh completed." : "Dataset is already current.",
        { exact: true },
      ),
    ).toBeVisible();
    await expect(page.getByText("Current", { exact: true })).toBeVisible();
    await expect(preview).not.toHaveText("");

    const after = await Promise.all(
      [base, derived, secondHop, independent].map((dataset) =>
        page.request
          .get(`/api/datasets/${dataset.id}/table`)
          .then((response) => expectJson<DatasetTable>(response)),
      ),
    );
    expect(after[3]).toEqual(before[3]);
    if (refresh.changed) {
      expect(refresh.materialization_receipt_ids.length).toBeGreaterThan(0);
      expect(after[0]).not.toEqual(before[0]);
      expect(after[1]).not.toEqual(before[1]);
      expect(after[2]).not.toEqual(before[2]);
    } else {
      expect(refresh.materialization_receipt_ids).toEqual([]);
      expect(after.slice(0, 3)).toEqual(before.slice(0, 3));
    }
  });

  test("reverse consumers distinguish authorized empty unavailable and undisclosed states", async ({
    page,
  }) => {
    const operations = await expectJson<OperationsStatus>(
      await page.request.get("/api/operations/status"),
    );
    await page.goto("/operations");
    const readiness = page.getByRole("region", { name: "Dataset readiness" });
    await expect(readiness).toBeVisible();
    if (operations.dataset_readiness.state === "unavailable") {
      await expect(readiness).toContainText("Dataset readiness unavailable");
    } else if (operations.dataset_readiness.state === "undisclosed") {
      await expect(readiness).toContainText("Dataset readiness restricted");
    } else if (operations.dataset_readiness.datasets.length === 0) {
      await expect(readiness).toContainText("No visible datasets");
    } else {
      await expect(readiness.getByPlaceholder("Search datasets")).toBeVisible();
    }
    await expect(readiness).not.toContainText(/opaque cursor|provider epoch|authorization digest/i);

    const forms = await expectJson<FormSummary[]>(await page.request.get("/api/forms"));
    const primaryForm = forms.find((form) => form.slug === "primary-responses");
    expect(primaryForm, "Primary Responses reverse-consumer fixture").toBeDefined();
    const form = await expectJson<FormDetail>(
      await page.request.get(`/api/forms/${primaryForm!.id}`),
    );
    await page.goto(`/forms/${primaryForm!.id}`);
    const datasetSourcesTab = page.getByRole("tab", { name: /^Dataset Sources/ });
    await expect(datasetSourcesTab).toBeVisible();
    await datasetSourcesTab.click();
    if (form.dataset_sources_state === "available") {
      expect(form.dataset_sources.length).toBeGreaterThan(0);
      await expect(page.getByLabel("Search dataset sources")).toBeVisible();
    } else if (form.dataset_sources_state === "empty") {
      await expect(page.getByText("No Related Dataset Sources to Display", { exact: true }).first()).toBeVisible();
    } else if (form.dataset_sources_state === "unavailable") {
      await expect(page.getByText(/Dataset source information is temporarily unavailable/)).toBeVisible();
    } else {
      await expect(page.getByText("Dataset source information is unavailable for this Form.", { exact: true })).toBeVisible();
    }
  });

  test("mutation replay and static route precedence remain exact", async ({ page }) => {
    // Exact replay is exercised at the owner boundary, where the same signed
    // authorization can be retried. Two browser Gateway calls intentionally
    // receive distinct authorization JTIs and therefore are not the same replay.
    const datasets = await expectJson<DatasetSummary[]>(
      await page.request.get("/api/datasets"),
    );
    const base = requireDataset(datasets, "base-responses");
    const detail = await expectJson<DatasetDetail>(
      await page.request.get(`/api/datasets/${base.id}`),
    );
    expect(detail.initial_source).not.toBeNull();
    const payload = {
      name: `Static Route Preview ${Date.now()}`,
      slug: `static-route-preview-${Date.now()}`,
      grain: detail.grain,
      visibility_node_ids: detail.visibility_nodes.map((node) => node.node_id),
      initial_source: detail.initial_source,
      operations: detail.operations,
      restriction_policy: detail.restriction_policy,
    };

    await page.goto("/datasets/new");
    await expect(page).toHaveURL(/\/datasets\/new$/);
    await expect(page.getByRole("heading", { name: /create dataset/i })).toBeVisible();
    const preview = await page.request.post("/api/admin/datasets/sql-preview", {
      data: payload,
    });
    expect(preview.status()).toBe(200);
    expect((await preview.json() as { generated_sql: string }).generated_sql).not.toBe("");
    const existingPreview = await page.request.post(
      `/api/admin/datasets/${base.id}/sql-preview`,
      { data: payload },
    );
    expect(existingPreview.status()).toBe(200);
    expect(
      (await existingPreview.json() as { generated_sql: string }).generated_sql,
    ).not.toBe("");

    const refreshStaticRoute = await page.request.post(
      `/api/admin/datasets/${base.id}/refresh`,
      {
        data: { unexpected: true },
        headers: { "x-idempotency-key": `route-precedence-${Date.now()}` },
      },
    );
    expect(refreshStaticRoute.status()).toBe(400);
    expect(refreshStaticRoute.status()).not.toBe(404);
    const body = await refreshStaticRoute.json() as { error: { code: string } };
    expect(body.error.code).toBe("dataset.malformed_request");
  });
});
