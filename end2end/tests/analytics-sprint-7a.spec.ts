import { expect, test, type APIResponse, type Page } from "@playwright/test";

type DatasetSummary = { id: string; slug: string };
type ComponentSummary = { component_id: string; slug: string };
type DashboardSummary = { id: string; name: string };
type DashboardPlacement = {
  placement_id: string;
  availability: string;
  resolution_state: string;
  title?: string;
  component?: {
    component_id: string;
    component_version_id: string;
    scope_node_ids: string[];
  };
};
type DashboardDetail = { id: string; placements: DashboardPlacement[] };

async function signIn(page: Page, email: string, password: string) {
  const response = await page.request.post("/api/auth/login", {
    data: { email, password },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

async function json<T>(response: APIResponse): Promise<T> {
  const text = await response.text();
  expect(response.ok(), `${response.url()} returned ${response.status()}: ${text}`).toBeTruthy();
  return JSON.parse(text) as T;
}

async function referenceInventory(page: Page) {
  const datasets = await json<DatasetSummary[]>(await page.request.get("/api/datasets"));
  const components = await json<ComponentSummary[]>(await page.request.get("/api/components"));
  const dashboards = await json<DashboardSummary[]>(await page.request.get("/api/dashboards"));
  const dataset = datasets.find((item) => item.slug === "derived-second-hop");
  const table = components.find((item) => item.slug === "dataset-table");
  const chart = components.find((item) => item.slug === "dataset-chart");
  const stat = components.find((item) => item.slug === "dataset-row-count");
  const disjoint = components.find((item) => item.slug === "dataset-disjoint-probe");
  const dashboard = dashboards.find((item) => item.name === "Dataset Components");
  for (const [label, value] of Object.entries({ dataset, table, chart, stat, disjoint, dashboard })) {
    expect(value, `Reference topology should expose ${label}`).toBeTruthy();
  }
  const detail = await json<DashboardDetail>(
    await page.request.get(`/api/dashboards/${dashboard!.id}`),
  );
  const placementFor = (componentId: string) => {
    const placement = detail.placements.find(
      (item) => item.component?.component_id === componentId,
    );
    expect(placement, `Dashboard should bind Component ${componentId}`).toBeTruthy();
    return placement!;
  };
  const disjointPlacement = detail.placements.find(
    (item) =>
      item.availability === "unavailable" &&
      item.resolution_state === "restricted" &&
      item.component === undefined,
  );
  expect(
    disjointPlacement,
    "Dashboard should retain one nondisclosing disjoint placement",
  ).toBeTruthy();
  return {
    dataset: dataset!, table: table!, chart: chart!, stat: stat!, disjoint: disjoint!,
    dashboard: dashboard!, detail,
    tablePlacement: placementFor(table!.component_id),
    chartPlacement: placementFor(chart!.component_id),
    statPlacement: placementFor(stat!.component_id),
    disjointPlacement: disjointPlacement!,
  };
}

test.describe("source-exact scoped analytics boundary", () => {
  test("Reference inventory exposes the canonical Dataset Components and Dashboard", async ({ page }) => {
    await signIn(page, "admin@tessara.local", "tessara-dev-admin");
    const fixture = await referenceInventory(page);
    expect(fixture.detail.placements).toHaveLength(4);
    expect(fixture.detail.placements.map((item) => item.component?.component_id)).toEqual(
      expect.arrayContaining([
        fixture.table.component_id,
        fixture.chart.component_id,
        fixture.stat.component_id,
        undefined,
      ]),
    );
  });

  test("real Dashboard mediation renders exact stat and table results", async ({ page }) => {
    await signIn(page, "admin@tessara.local", "tessara-dev-admin");
    const fixture = await referenceInventory(page);
    const stat = await json<any>(await page.request.get(
      `/api/dashboards/${fixture.dashboard.id}/placements/${fixture.statPlacement.placement_id}/render/stat-card`,
    ));
    expect(stat.materialization_state).toBe("ready");
    expect(stat.stat.display_value).toBe("1");
    const table = await json<any>(await page.request.get(
      `/api/dashboards/${fixture.dashboard.id}/placements/${fixture.tablePlacement.placement_id}/render/table?page_size=100`,
    ));
    expect(table.materialization_state).toBe("ready");
    expect(table.rows).toHaveLength(1);
    const rows = JSON.stringify(table.rows);
    expect(rows).toContain("Submitted owner");
    expect(rows).toContain('"amount":"20"');
    expect(rows).not.toContain("New after initial sync");
    expect(rows).not.toContain("Outside scope");
  });

  test("Dashboard and Component scopes must share a governing node before disclosure or render", async ({ page }) => {
    await signIn(page, "full-reader@tessara.local", "sprint-8b-full-reader");
    const dashboards = await json<DashboardSummary[]>(await page.request.get("/api/dashboards"));
    const dashboard = dashboards.find((item) => item.name === "Dataset Components");
    expect(dashboard).toBeTruthy();
    const detail = await json<DashboardDetail>(
      await page.request.get(`/api/dashboards/${dashboard!.id}`),
    );
    const available = detail.placements.filter((item) => item.availability === "available");
    const blocked = detail.placements.filter((item) => item.availability !== "available");
    expect(available).toHaveLength(3);
    expect(blocked).toHaveLength(1);
    expect(blocked[0]).toMatchObject({ resolution_state: "restricted" });
    expect(blocked[0].component).toBeUndefined();
    expect(blocked[0].title).toBeUndefined();
    const blockedRender = await page.request.get(
      `/api/dashboards/${dashboard!.id}/placements/${blocked[0].placement_id}/render/table`,
    );
    expect([403, 404]).toContain(blockedRender.status());
    expect(await blockedRender.text()).not.toContain("dataset-disjoint-probe");
  });

  test("private compatibility endpoints reject browser authority without disclosing resources", async ({ request }) => {
    for (const path of [
      "/api/private/dashboard-components/catalog",
      "/api/private/dashboard-components/resolve",
      "/api/private/dashboard-components/render",
    ]) {
      const response = await request.post(path, { data: {} });
      expect([401, 403, 422]).toContain(response.status());
      expect(await response.text()).not.toContain("derived-second-hop");
    }
  });

  test("disjoint Component lookups make known blocked and random identities indistinguishable", async ({ page }) => {
    await signIn(page, "admin@tessara.local", "tessara-dev-admin");
    const components = await json<ComponentSummary[]>(await page.request.get("/api/components"));
    const disjoint = components.find((item) => item.slug === "dataset-disjoint-probe");
    expect(disjoint).toBeTruthy();
    await signIn(page, "full-reader@tessara.local", "sprint-8b-full-reader");
    const known = await page.request.get(`/api/components/${disjoint!.component_id}`);
    const random = await page.request.get("/api/components/01980000-ffff-7000-8000-000000000001");
    expect(known.status()).toBe(404);
    expect(random.status()).toBe(known.status());
    expect(random.headers()["content-type"]).toBe(known.headers()["content-type"]);
    expect(await random.text()).toBe(await known.text());
  });
});
