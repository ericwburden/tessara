import { expect, test } from "@playwright/test";

const fixture = {
  sharedScopeNodeId: "01980000-0002-7000-8000-000000000002",
  blockedScopeNodeId: "01980000-0002-7000-8000-000000000007",
  datasetId: "01980000-0002-7000-8000-000000000003",
  blockedDatasetId: "01980000-0002-7000-8000-000000000008",
  metricComponentId: "01980000-0002-7000-8000-000000000004",
  metricComponentVersionId: "01980000-0001-7000-8000-000000000011",
  tableComponentId: "01980000-0002-7000-8000-000000000005",
  chartComponentId: "01980000-0002-7000-8000-000000000006",
  blockedComponentId: "01980000-0002-7000-8000-00000000000b",
  blockedComponentVersionId: "01980000-0001-7000-8000-000000000004",
  dashboardId: "01980000-0003-7000-8000-000000000001",
  metricPlacementId: "01980000-0003-7000-8000-000000000002",
  tablePlacementId: "01980000-0003-7000-8000-000000000003",
  chartPlacementId: "01980000-0003-7000-8000-000000000004",
  blockedPlacementId: "01980000-0003-7000-8000-000000000005",
  lifecycleUpgradePlacementId: "01980000-0003-7000-8000-000000000006",
  lifecycleReplacePlacementId: "01980000-0003-7000-8000-000000000007",
  lifecycleRemovePlacementId: "01980000-0003-7000-8000-000000000008",
};

async function signInAsAdmin(page: import("@playwright/test").Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok()).toBeTruthy();
}

async function signInAsMixedScope(page: import("@playwright/test").Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "mixed-sprint7a@tessara.local", password: "tessara-sprint-7a-mixed" },
  });
  expect(response.ok()).toBeTruthy();
}

test.describe("Sprint 7A scoped analytics boundary", () => {
  test("source-exact reference inventory exposes the canonical Dataset Components and Dashboard", async ({ page }) => {
    await signInAsAdmin(page);
    const datasets = await (await page.request.get("/api/datasets")).json();
    expect(datasets.some((dataset: { id: string }) => dataset.id === fixture.datasetId)).toBeTruthy();
    const components = await (await page.request.get("/api/components")).json();
    const componentIds = components.map((component: { component_id: string }) => component.component_id);
    expect(componentIds).toEqual(expect.arrayContaining([
      fixture.metricComponentId,
      fixture.tableComponentId,
      fixture.chartComponentId,
      fixture.blockedComponentId,
    ]));
    const dashboard = await (await page.request.get(`/api/dashboards/${fixture.dashboardId}`)).json();
    expect(dashboard.placements.map((placement: { placement_id: string }) => placement.placement_id)).toEqual([
      fixture.metricPlacementId,
      fixture.tablePlacementId,
      fixture.chartPlacementId,
      fixture.blockedPlacementId,
      fixture.lifecycleUpgradePlacementId,
      fixture.lifecycleReplacePlacementId,
      fixture.lifecycleRemovePlacementId,
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
    expect(stat.stat.display_value).toBe("30");
    const tableResponse = await page.request.get(
      `/api/dashboards/${fixture.dashboardId}/placements/${fixture.tablePlacementId}/render/table?page_size=100`,
    );
    expect(tableResponse.ok()).toBeTruthy();
    const table = await tableResponse.json();
    expect(table.materialization_state).toBe("ready");
    const rows = JSON.stringify(table.rows);
    expect(rows).toContain("UAT7A-PUBLIC");
    expect(rows).toContain("UAT7A-INTERNAL");
    expect(rows).toContain("UAT7A-RESTRICTED");
    expect(rows).toContain("UAT7A-CONFIDENTIAL-BLOCKED");
  });

  test("Dashboard and Component scopes must share a governing node before disclosure or render", async ({ page }) => {
    await signInAsAdmin(page);

    const dashboardResponse = await page.request.get(`/api/dashboards/${fixture.dashboardId}`);
    expect(dashboardResponse.ok()).toBeTruthy();
    const dashboard = await dashboardResponse.json() as {
      placements: Array<{
        placement_id: string;
        availability: string;
        resolution_state: string;
        title?: string;
        component?: {
          component_id: string;
          component_version_id: string;
          scope_node_ids: string[];
        };
      }>;
    };

    const sharedPlacement = dashboard.placements.find(
      (placement) => placement.placement_id === fixture.metricPlacementId,
    );
    expect(sharedPlacement).toMatchObject({
      availability: "available",
      resolution_state: "available",
      component: {
        component_id: fixture.metricComponentId,
        component_version_id: fixture.metricComponentVersionId,
        scope_node_ids: [fixture.sharedScopeNodeId],
      },
    });

    const blockedPlacement = dashboard.placements.find(
      (placement) => placement.placement_id === fixture.blockedPlacementId,
    );
    expect(blockedPlacement).toMatchObject({
      availability: "unavailable",
      resolution_state: "restricted",
    });
    expect(blockedPlacement?.component).toBeUndefined();
    expect(blockedPlacement?.title).toBeUndefined();

    const restrictedMarkers = [
      fixture.blockedScopeNodeId,
      fixture.blockedDatasetId,
      fixture.blockedComponentId,
      fixture.blockedComponentVersionId,
      "UAT7A-BLOCKED-DATASET",
    ];
    const blockedProjection = JSON.stringify(blockedPlacement);
    for (const marker of restrictedMarkers) {
      expect(blockedProjection).not.toContain(marker);
    }

    const sharedRender = await page.request.get(
      `/api/dashboards/${fixture.dashboardId}/placements/${fixture.metricPlacementId}/render/stat-card`,
    );
    expect(sharedRender.ok()).toBeTruthy();
    expect(await sharedRender.json()).toMatchObject({
      component_id: fixture.metricComponentId,
      component_version_id: fixture.metricComponentVersionId,
      component_type: "stat_card",
      materialization_state: "ready",
    });

    const blockedRender = await page.request.get(
      `/api/dashboards/${fixture.dashboardId}/placements/${fixture.blockedPlacementId}/render/table`,
    );
    expect([403, 404]).toContain(blockedRender.status());
    const blockedRenderBody = await blockedRender.text();
    for (const marker of restrictedMarkers) {
      expect(blockedRenderBody).not.toContain(marker);
    }
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

  test("mixed-scope Component lookups make known blocked and random identities indistinguishable", async ({ page }) => {
    await signInAsMixedScope(page);
    const known = await page.request.get(`/api/components/${fixture.blockedComponentId}`);
    const random = await page.request.get("/api/components/01980000-ffff-7000-8000-000000000001");

    expect(known.status()).toBe(404);
    expect(random.status()).toBe(known.status());
    expect(random.headers()["content-type"]).toBe(known.headers()["content-type"]);
    expect(await random.text()).toBe(await known.text());
  });
});
