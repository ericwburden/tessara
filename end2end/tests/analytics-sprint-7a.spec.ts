import { expect, test } from "@playwright/test";

const fixture = {
  datasetId: "01980000-0002-7000-8000-000000000003",
  metricComponentId: "01980000-0002-7000-8000-000000000004",
  tableComponentId: "01980000-0002-7000-8000-000000000005",
  dashboardId: "01980000-0003-7000-8000-000000000001",
  metricPlacementId: "01980000-0003-7000-8000-000000000002",
  tablePlacementId: "01980000-0003-7000-8000-000000000003",
};

async function signInAsAdmin(page: import("@playwright/test").Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok()).toBeTruthy();
}

test.describe("Sprint 7A scoped analytics boundary", () => {
  test("source-exact reference inventory exposes the canonical Dataset Components and Dashboard", async ({ page }) => {
    await signInAsAdmin(page);
    const datasets = await (await page.request.get("/api/datasets")).json();
    expect(datasets.some((dataset: { id: string }) => dataset.id === fixture.datasetId)).toBeTruthy();
    const components = await (await page.request.get("/api/components")).json();
    expect(components.some((component: { id: string }) => component.id === fixture.metricComponentId)).toBeTruthy();
    expect(components.some((component: { id: string }) => component.id === fixture.tableComponentId)).toBeTruthy();
    const dashboard = await (await page.request.get(`/api/dashboards/${fixture.dashboardId}`)).json();
    expect(dashboard.placements.map((placement: { placement_id: string }) => placement.placement_id)).toEqual([
      fixture.metricPlacementId,
      fixture.tablePlacementId,
    ]);
  });

  test("real Dashboard mediation renders exact stat and table results", async ({ page }) => {
    await signInAsAdmin(page);
    const statResponse = await page.request.get(
      `/api/dashboards/${fixture.dashboardId}/placements/${fixture.metricPlacementId}/render/stat-card`,
    );
    expect(statResponse.ok()).toBeTruthy();
    const stat = await statResponse.json();
    expect(stat.materialization_state).toBe("ready");
    expect(stat.stat.display_value).toBe("1");
    const tableResponse = await page.request.get(
      `/api/dashboards/${fixture.dashboardId}/placements/${fixture.tablePlacementId}/render/table`,
    );
    expect(tableResponse.ok()).toBeTruthy();
    const table = await tableResponse.json();
    expect(table.materialization_state).toBe("ready");
    expect(JSON.stringify(table.rows)).toContain("Reference row");
  });

  test("private compatibility endpoints reject browser authority without disclosing resources", async ({ request }) => {
    for (const path of [
      "/api/private/dashboard-components/catalog",
      "/api/private/dashboard-components/resolve",
      "/api/private/dashboard-components/render",
    ]) {
      const response = await request.post(path, { data: {} });
      expect([401, 403, 422]).toContain(response.status());
      const body = await response.text();
      expect(body).not.toContain(fixture.datasetId);
      expect(body).not.toContain(fixture.metricComponentId);
      expect(body).not.toContain(fixture.dashboardId);
    }
  });
});
