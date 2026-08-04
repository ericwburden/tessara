import { expect, test, type APIResponse, type Page } from "@playwright/test";
import { invokeDemoSeedEndpoint } from "./support/demo-seed";

const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const RUN_ID = `pw-dashboards-${Date.now()}`;
let fixtureSequence = 0;

const BENIGN_NAVIGATION_ABORT =
  "WebAssembly compilation aborted: Network error: Response body loading was aborted";
const RENDERED_COMPONENT_CONTENT = [
  ".component-table-viewer__table",
  ".component-d3-chart__surface",
  ".component-stat-card strong",
].join(", ");

type IdResponse = { id: string };

type VisibilityNode = {
  node_id: string;
  node_name: string;
  node_path: string;
};

type DashboardSummary = {
  id: string;
  name: string;
  placement_count: number;
  visibility_nodes: VisibilityNode[];
};

type ComponentVersionOption = {
  component_version_id: string;
  component_slug: string;
  component_type: string;
  default_grid_width: number;
  default_grid_height: number;
};

type DashboardPlacement = {
  placement_id: string;
  grid_row: number;
  grid_column: number;
  grid_width: number;
  grid_height: number;
  availability: "available" | "unavailable";
  component?: {
    component_version_id: string;
    component_slug: string;
    component_type: string;
  };
};

type DashboardComposition = {
  dashboard: {
    id: string;
    name: string;
    placement_count: number;
    placements: DashboardPlacement[];
  };
  available_component_versions: ComponentVersionOption[];
};

type DashboardDefinition = DashboardSummary & {
  placements: DashboardPlacement[];
};

type DashboardFixture = {
  id: string;
  name: string;
  composition: DashboardComposition;
};

function isBenignNavigationError(message: string, lastNavigationStartedAt: number) {
  return (
    message.includes(BENIGN_NAVIGATION_ABORT) &&
    Date.now() - lastNavigationStartedAt < 5_000
  );
}

function attachConsoleGuard(page: Page) {
  const errors: string[] = [];
  let lastNavigationStartedAt = Number.NEGATIVE_INFINITY;
  page.on("request", (request) => {
    if (request.isNavigationRequest() && request.frame() === page.mainFrame()) {
      lastNavigationStartedAt = Date.now();
    }
  });
  page.on("console", (message) => {
    if (
      message.type() === "error" &&
      !isBenignNavigationError(message.text(), lastNavigationStartedAt)
    ) {
      errors.push(`${page.url()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => {
    if (!isBenignNavigationError(error.message, lastNavigationStartedAt)) {
      errors.push(`${page.url()}: ${error.message}`);
    }
  });
  page.on("response", (response) => {
    if (response.status() >= 400) {
      errors.push(`${response.status()} ${response.request().method()} ${response.url()}`);
    }
  });
  return () => {
    expect(errors, `browser console should stay clean: ${errors.join("\n")}`).toEqual([]);
  };
}

async function gotoHydrated(page: Page, url: string) {
  await page.goto(url);
  await expect(page.locator("#module-content")).toHaveAttribute(
    "data-hydration",
    "ready",
  );
}

function isComponentExecutionPath(pathname: string) {
  return (
    /\/api\/components\/[^/]+\/(?:versions\/[^/]+\/)?(?:table|bar|line|pie|donut|stat-card)$/.test(
      pathname,
    ) ||
    /\/api\/dashboards\/[^/]+\/placements\/[^/]+\/render\/(?:table|bar|line|pie|donut|stat-card)$/.test(
      pathname,
    )
  );
}

function attachComponentExecutionTracker(page: Page) {
  const paths: string[] = [];
  page.on("request", (request) => {
    const pathname = new URL(request.url()).pathname;
    if (request.method() === "GET" && isComponentExecutionPath(pathname)) {
      paths.push(pathname);
    }
  });
  return paths;
}

function attachSuccessfulComponentExecutionTracker(page: Page) {
  const paths: string[] = [];
  page.on("response", (response) => {
    const request = response.request();
    const pathname = new URL(response.url()).pathname;
    if (
      request.method() === "GET" &&
      response.ok() &&
      isComponentExecutionPath(pathname)
    ) {
      paths.push(pathname);
    }
  });
  return paths;
}

async function expectJson<T>(response: APIResponse) {
  const text = await response.text();
  expect(
    response.ok(),
    `${response.url()} returned ${response.status()}: ${text}`,
  ).toBeTruthy();
  return JSON.parse(text) as T;
}

async function signIn(page: Page, email: string, password: string) {
  await expectJson(
    await page.request.post("/api/auth/login", {
      data: {
        email,
        password,
      },
    }),
  );
}

async function signInAsAdmin(page: Page) {
  await signIn(page, "admin@tessara.local", "tessara-dev-admin");
}

async function ensureDemoSeed(page: Page) {
  const response = await invokeDemoSeedEndpoint(page.request);
  if (response === null) {
    return;
  }
  const text = await response.text();
  if (
    response.ok() ||
    (response.status() === 400 && text.includes("Demo seed requires an empty database"))
  ) {
    return;
  }
  expect(response.ok(), `${response.url()} returned ${response.status()}: ${text}`).toBeTruthy();
}

async function createDashboardFixture(page: Page): Promise<DashboardFixture> {
  const dashboards = await expectJson<DashboardSummary[]>(
    await page.request.get("/api/dashboards"),
  );
  const visibilitySource = dashboards.find(
    (dashboard) => dashboard.visibility_nodes.length > 0,
  );
  expect(
    visibilitySource,
    "demo seed should expose a Dashboard with visibility nodes",
  ).toBeTruthy();

  fixtureSequence += 1;
  const name = `${RUN_ID}-${fixtureSequence}`;
  const created = await expectJson<IdResponse>(
    await page.request.post("/api/admin/dashboards", {
      data: {
        name,
        description: "Sprint 5A Dashboard browser fixture.",
        visibility_node_ids: visibilitySource!.visibility_nodes.map((node) => node.node_id),
      },
    }),
  );
  try {
    const composition = await expectJson<DashboardComposition>(
      await page.request.get(`/api/admin/dashboards/${created.id}/composition`),
    );
    expect(
      composition.available_component_versions.length,
      "demo seed should expose at least one placeable published Component version",
    ).toBeGreaterThan(0);
    return { id: created.id, name, composition };
  } catch (error) {
    await page.request.delete(`/api/admin/dashboards/${created.id}`);
    throw error;
  }
}

async function deleteDashboardFixture(page: Page, dashboardId: string) {
  const response = await page.request.delete(`/api/admin/dashboards/${dashboardId}`);
  expect(
    response.ok(),
    `dashboard cleanup returned ${response.status()}: ${await response.text()}`,
  ).toBeTruthy();
}

function bindCommand(
  option: ComponentVersionOption,
  clientKey: string,
  row: number,
) {
  return bindGeometryCommand(option, clientKey, {
    grid_row: row,
    grid_column: 1,
    grid_width: Math.min(option.default_grid_width, 12),
    grid_height: option.default_grid_height,
  });
}

function bindGeometryCommand(
  option: ComponentVersionOption,
  clientKey: string,
  geometry: {
    grid_row: number;
    grid_column: number;
    grid_width: number;
    grid_height: number;
  },
) {
  return {
    operation: "bind",
    client_key: clientKey,
    component_version_id: option.component_version_id,
    geometry,
  };
}

function executionSuffix(componentType: string) {
  return componentType === "stat_card" ? "stat-card" : componentType;
}

test.describe.serial("Sprint 5A Dashboard routes and composition", () => {
  test("all native routes keep metadata-only surfaces inert and the focused viewer executes exact versions", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    const executionPaths = attachComponentExecutionTracker(page);
    const successfulExecutionPaths = attachSuccessfulComponentExecutionTracker(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);

    try {
      const option = fixture.composition.available_component_versions[0];
      await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: [
              bindCommand(option, `${RUN_ID}-route-1`, 1),
              bindCommand(
                option,
                `${RUN_ID}-route-2`,
                option.default_grid_height + 1,
              ),
            ],
          },
        }),
      );

      await gotoHydrated(page, "/dashboards");
      await expect(
        page.getByRole("heading", { level: 1, name: "Dashboards" }),
      ).toBeVisible();
      await expect(page.getByLabel("Search Dashboards")).toBeVisible();
      const directoryRow = page.locator(`[data-dashboard-id="${fixture.id}"]`);
      await expect(directoryRow).toBeVisible();
      const directoryActions = directoryRow.locator("td.data-table__actions");
      await expect(directoryActions).toHaveCSS("display", "table-cell");
      await expect(
        directoryRow.getByRole("link", { name: `View ${fixture.name}` }),
      ).toHaveAttribute("href", `/dashboards/${fixture.id}/view`);
      await expect(
        directoryRow.getByRole("link", { name: `Edit ${fixture.name}` }),
      ).toHaveAttribute("href", `/dashboards/${fixture.id}/edit`);
      await expect(directoryActions.locator(".data-table__action-group")).toHaveCount(1);
      await directoryRow
        .getByRole("button", { name: /^View \d+ nodes? in .+ visibility scope$/ })
        .click();
      const visibilityDialog = page.getByRole("dialog", {
        name: "Visibility scope",
      });
      await expect(visibilityDialog).toBeVisible();
      await visibilityDialog
        .getByRole("button", { name: "Close visibility scope" })
        .click();

      await gotoHydrated(page, "/dashboards/new");
      await expect(
        page.getByRole("heading", { level: 1, name: "Create Dashboard" }),
      ).toBeVisible();
      await expect(page.getByLabel("Name")).toBeVisible();
      await expect(page.getByRole("group", { name: "Visibility scope" })).toBeVisible();

      await gotoHydrated(page, `/dashboards/${fixture.id}`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      await expect(page.getByRole("button", { name: /Dependency health/ })).toBeVisible();
      await expect(page.getByText("One placement needs review before this Dashboard is healthy.")).toHaveCount(0);
      await expect(page.getByText("PROTOTYPE CONTROL", { exact: false })).toHaveCount(0);
      const detailVisibility = page.getByRole("button", {
        name: /^Visibility \d+ Nodes?$/,
      });
      await expect(detailVisibility).toBeVisible();
      await expect(page.locator(".metric-card")).toHaveCount(2);
      await expect(page.getByText("12-column saved grid", { exact: true })).toHaveCount(0);
      await detailVisibility.click();
      const detailVisibilityDialog = page.getByRole("dialog", {
        name: "Visibility scope",
      });
      await expect(detailVisibilityDialog.getByLabel("Search visibility nodes")).toBeVisible();
      await expect(
        detailVisibilityDialog.locator('a[href^="/organization/"]').first(),
      ).toBeVisible();
      await detailVisibilityDialog
        .getByRole("button", { name: "Close visibility scope" })
        .click();
      await expect(page.getByRole("heading", { name: "Saved layout" })).toBeVisible();
      await expect(page.locator(".dashboard-placement-card")).toHaveCount(2);

      await gotoHydrated(page, `/dashboards/${fixture.id}/edit`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      const componentsTrigger = page.getByRole("button", { name: "Components" });
      const placementDetailsTrigger = page.getByRole("button", {
        name: "Placement details",
      });
      await expect(componentsTrigger).toBeVisible();
      await expect(placementDetailsTrigger).toBeDisabled();
      await componentsTrigger.click();
      const componentsDialog = page.getByRole("dialog", { name: "Components" });
      await expect(page.getByLabel("Available Components")).toBeVisible();
      await expect(componentsDialog.getByLabel("Search Components")).toBeVisible();
      await expect(componentsDialog.getByLabel("Filter components by kind")).toBeVisible();
      await componentsDialog
        .getByRole("button", { name: "Close Components" })
        .click();
      await expect(page.locator(".dashboard-composition-canvas")).toBeVisible();
      await expect(page.locator(".dashboard-composition-tile")).toHaveCount(2);
      await page.locator(".dashboard-composition-tile").first().click();
      await expect(placementDetailsTrigger).toBeEnabled();
      await placementDetailsTrigger.click();
      const placementDetailsDialog = page.getByRole("dialog", {
        name: "Placement details",
      });
      await expect(placementDetailsDialog).toBeVisible();
      await placementDetailsDialog
        .getByRole("button", { name: "Close Placement details" })
        .click();
      expect(
        executionPaths,
        "directory, create, detail, and editor routes must remain metadata-only",
      ).toEqual([]);

      for (const width of [775, 780]) {
        await page.setViewportSize({ width, height: 900 });
        await gotoHydrated(page, `/dashboards/${fixture.id}`);
        const savedGrid = page.locator(".dashboard-saved-grid");
        await expect(savedGrid).toBeVisible();
        expect(
          await savedGrid.evaluate((grid) => {
            const style = window.getComputedStyle(grid);
            return {
              display: style.display,
              flexDirection: style.flexDirection,
            };
          }),
          `Dashboard canvas should stack at ${width}px`,
        ).toEqual({ display: "flex", flexDirection: "column" });
        const cardTops = await page.locator(".dashboard-placement-card").evaluateAll((cards) =>
          cards.map((card) => card.getBoundingClientRect().top),
        );
        expect(cardTops).toEqual([...cardTops].sort((left, right) => left - right));
        expect(
          await page.evaluate(
            () =>
              document.documentElement.scrollWidth <=
              document.documentElement.clientWidth + 1,
          ),
          `Dashboard route should not overflow horizontally at ${width}px`,
        ).toBe(true);
      }
      expect(executionPaths).toEqual([]);

      await page.setViewportSize({ width: 1440, height: 1000 });
      await gotoHydrated(page, `/dashboards/${fixture.id}/view`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      await expect(page.getByRole("link", { name: "Edit Dashboard" })).toBeVisible();
      await expect(page.locator(".dashboard-viewer-placement")).toHaveCount(2);
      await expect(page.locator(".dashboard-viewer-placement .dashboard-placement-card__size"))
        .toHaveCount(0);

      const mediatedPath = new RegExp(
        `^/api/dashboards/${fixture.id}/placements/[^/]+/render/${executionSuffix(option.component_type)}$`,
      );
      await expect
        .poll(() => executionPaths.filter((path) => mediatedPath.test(path)).length)
        .toBeGreaterThanOrEqual(2);
      await expect
        .poll(
          () =>
            successfulExecutionPaths.filter((path) => mediatedPath.test(path))
              .length,
        )
        .toBeGreaterThanOrEqual(1);
      await expect(
        page.locator(".dashboard-viewer").locator(RENDERED_COMPONENT_CONTENT).first(),
      ).toBeVisible();
      expect(
        executionPaths.every((path) => mediatedPath.test(path)),
        `focused Dashboard viewer requests should stay placement-bound: ${executionPaths.join(", ")}`,
      ).toBe(true);
      assertNoConsoleErrors();
    } finally {
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("saved viewer uses the shared Table toolbar while stat cards stay intrinsic and charts stay absent", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);

    try {
      const tableOption = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "table",
      );
      const chartOption = fixture.composition.available_component_versions.find(
        (option) => ["bar", "line", "pie", "donut"].includes(option.component_type),
      );
      const statCardOption = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "stat_card",
      );
      expect(tableOption, "demo seed should expose a placeable Table").toBeTruthy();
      expect(
        chartOption,
        "the Sprint 7A reference inventory should expose a placeable chart Component",
      ).toBeTruthy();
      expect(statCardOption, "demo seed should expose a placeable stat card").toBeTruthy();

      // This focused fixture intentionally binds only Table and Stat Card so the
      // viewer can prove their intrinsic presentation without a chart placement.
      const options = [tableOption!, statCardOption!];
      let nextRow = 1;
      await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: options.map((option, index) => {
              const command = bindCommand(
                option,
                `${RUN_ID}-viewer-presentation-${index + 1}`,
                nextRow,
              );
              nextRow += option.default_grid_height;
              return command;
            }),
          },
        }),
      );

      await page.setViewportSize({ width: 1600, height: 1200 });
      await gotoHydrated(page, `/dashboards/${fixture.id}/view`);

      const tablePlacement = page.locator(
        '[data-placement-presentation="table"]',
      );
      await expect(tablePlacement).toHaveCount(1);
      await tablePlacement.scrollIntoViewIfNeeded();
      await expect(
        tablePlacement.locator(":scope > .dashboard-viewer-placement__header"),
      ).toHaveCount(0);
      await expect(
        tablePlacement.locator(
          ".interactive-data-table__toolbar .interactive-data-table__title",
        ),
      ).toBeVisible();
      await expect(
        tablePlacement.getByRole("button", { name: "View fullscreen" }),
      ).toBeVisible();
      const tableToolbarActions = tablePlacement.locator(
        ".interactive-data-table__toolbar-actions",
      );
      await expect(tableToolbarActions.getByRole("button")).toHaveCount(3);
      await expect(tableToolbarActions.getByRole("button").nth(0)).toHaveAccessibleName(
        "Reset table controls",
      );
      await expect(tableToolbarActions.getByRole("button").nth(1)).toHaveAccessibleName(
        "Choose visible columns",
      );
      await expect(tableToolbarActions.getByRole("button").nth(2)).toHaveAccessibleName(
        "View fullscreen",
      );
      await expect(tablePlacement.locator(".component-table-viewer__table")).toBeVisible();

      const chartPlacement = page.locator(
        '[data-placement-presentation="chart"]',
      );
      await expect(chartPlacement).toHaveCount(0);

      const statCardPlacement = page.locator(
        '[data-placement-presentation="stat-card"]',
      );
      await expect(statCardPlacement).toHaveCount(1);
      await statCardPlacement.scrollIntoViewIfNeeded();
      await expect(
        statCardPlacement.locator(":scope > .dashboard-viewer-placement__header"),
      ).toHaveCount(0);
      await expect(statCardPlacement.locator(".component-stat-card")).toBeVisible();
      await expect(statCardPlacement.locator(".component-stat-card strong")).toBeVisible();
      assertNoConsoleErrors();
    } finally {
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("viewer request permits execute twelve visible placements fairly without exceeding six in flight", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);
    const allExecutionPaths = attachComponentExecutionTracker(page);
    const executionRoutePattern = `**/api/dashboards/${fixture.id}/placements/**/render/stat-card**`;
    const mediatedStatCardPath = new RegExp(
      `^/api/dashboards/${fixture.id}/placements/[^/]+/render/stat-card$`,
    );
    let inFlight = 0;
    let maxInFlight = 0;
    const startedPaths: string[] = [];
    const successfulResponses: boolean[] = [];

    try {
      const statCard = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "stat_card",
      );
      expect(statCard, "demo seed should expose a published Stat Card").toBeTruthy();
      await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: Array.from({ length: 12 }, (_, index) =>
              bindGeometryCommand(statCard!, `${RUN_ID}-ceiling-${index + 1}`, {
                grid_row: 1,
                grid_column: index + 1,
                grid_width: 1,
                grid_height: 1,
              }),
            ),
          },
        }),
      );
      await page.route(executionRoutePattern, async (route) => {
        const request = route.request();
        const pathname = new URL(request.url()).pathname;
        if (request.method() !== "GET" || !mediatedStatCardPath.test(pathname)) {
          await route.continue();
          return;
        }

        startedPaths.push(pathname);
        inFlight += 1;
        maxInFlight = Math.max(maxInFlight, inFlight);
        try {
          await new Promise<void>((resolve) => setTimeout(resolve, 300));
          const response = await route.fetch();
          successfulResponses.push(response.ok());
          await route.fulfill({ response });
        } finally {
          inFlight -= 1;
        }
      });

      await page.setViewportSize({ width: 1920, height: 1080 });
      await gotoHydrated(page, `/dashboards/${fixture.id}/view`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      const placements = page.locator(".dashboard-viewer-placement");
      await expect(placements).toHaveCount(12);
      expect(
        await placements.evaluateAll((items) =>
          items.every((item) => {
            const bounds = item.getBoundingClientRect();
            return (
              bounds.bottom > 0 &&
              bounds.right > 0 &&
              bounds.top < window.innerHeight &&
              bounds.left < window.innerWidth
            );
          }),
        ),
        "all twelve one-cell placements should intersect the viewport together",
      ).toBe(true);

      await expect
        .poll(() => startedPaths.length, { timeout: 30_000 })
        .toBe(12);
      await expect(
        placements.locator(".component-stat-card strong"),
      ).toHaveCount(12, { timeout: 30_000 });
      await expect.poll(() => inFlight).toBe(0);
      expect(startedPaths.every((path) => mediatedStatCardPath.test(path))).toBe(true);
      expect(allExecutionPaths.length).toBeGreaterThanOrEqual(12);
      expect(
        allExecutionPaths.every((path) => mediatedStatCardPath.test(path)),
      ).toBe(true);
      expect(successfulResponses).toHaveLength(12);
      expect(successfulResponses.every(Boolean)).toBe(true);
      expect(
        maxInFlight,
        "six delayed requests should saturate, but never exceed, the permit ceiling",
      ).toBe(6);
      assertNoConsoleErrors();
    } finally {
      await page.unrouteAll({ behavior: "wait" });
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("viewer keeps provider failure contained while bounded retries run", async ({
    page,
  }) => {
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);

    try {
      const statCard = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "stat_card",
      );
      expect(statCard, "demo seed should expose a published Stat Card").toBeTruthy();
      const composition = await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: [
              bindGeometryCommand(statCard!, `${RUN_ID}-contained-failure`, {
                grid_row: 1,
                grid_column: 1,
                grid_width: statCard!.default_grid_width,
                grid_height: statCard!.default_grid_height,
              }),
              bindGeometryCommand(statCard!, `${RUN_ID}-contained-success`, {
                grid_row: statCard!.default_grid_height + 1,
                grid_column: 1,
                grid_width: statCard!.default_grid_width,
                grid_height: statCard!.default_grid_height,
              }),
            ],
          },
        }),
      );
      expect(composition.dashboard.placements).toHaveLength(2);
      const failedPlacementId = composition.dashboard.placements[0].placement_id;
      const failedPath = `/api/dashboards/${fixture.id}/placements/${failedPlacementId}/render/stat-card`;
      let failedAttempts = 0;
      await page.route(`**${failedPath}`, async (route) => {
        failedAttempts += 1;
        await route.fulfill({
          status: 503,
          contentType: "application/json",
          body: JSON.stringify({ error: "Component provider unavailable" }),
        });
      });

      await gotoHydrated(page, `/dashboards/${fixture.id}/view`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      const failedPlacement = page.locator(
        `.dashboard-viewer-placement[data-placement-id="${failedPlacementId}"]`,
      );
      await expect(
        failedPlacement.getByRole("heading", { name: "Preview unavailable" }),
      ).toBeVisible();
      await expect(failedPlacement).toContainText("Component provider unavailable");
      await expect(
        page.locator(".dashboard-viewer-placement .component-stat-card strong"),
      ).toHaveCount(1);
      await expect.poll(() => failedAttempts).toBeGreaterThanOrEqual(2);
      await expect(
        failedPlacement.getByRole("heading", { name: "Preview unavailable" }),
      ).toBeVisible();
      await expect(failedPlacement).not.toContainText("Loading preview");
    } finally {
      await page.unrouteAll({ behavior: "wait" });
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("embedded Table keeps full server-backed paging and page-size controls", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);
    const allExecutionPaths = attachComponentExecutionTracker(page);
    const executionUrls: string[] = [];

    try {
      const tableOption = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "table",
      );
      expect(
        tableOption,
        "the reference composition should expose a placeable Table",
      ).toBeTruthy();
      await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: [
              bindGeometryCommand(tableOption!, `${RUN_ID}-paged-table`, {
                grid_row: 1,
                grid_column: 1,
                grid_width: 12,
                grid_height: 6,
              }),
            ],
          },
        }),
      );
      const mediatedTablePath = new RegExp(
        `^/api/dashboards/${fixture.id}/placements/[^/]+/render/table$`,
      );
      page.on("request", (request) => {
        const url = new URL(request.url());
        if (request.method() === "GET" && mediatedTablePath.test(url.pathname)) {
          executionUrls.push(request.url());
        }
      });

      await page.setViewportSize({ width: 1600, height: 1200 });
      await gotoHydrated(page, `/dashboards/${fixture.id}/view`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      const tableViewer = page.locator(".component-table-viewer__table");
      await expect(tableViewer).toBeVisible();
      await expect(page.getByPlaceholder("Search table")).toBeVisible();
      await expect(page.getByRole("button", { name: "Reset table controls" })).toBeVisible();
      await expect(page.getByRole("button", { name: "Choose visible columns" })).toBeVisible();

      if (tableOption!.component_slug === "sprint-7a-record-table") {
        const rows = tableViewer.locator("tbody tr[data-row-id]");
        await expect(rows).not.toHaveCount(0);
        await expect.poll(() => executionUrls.length).toBeGreaterThanOrEqual(1);
        const requestCountBeforeFullscreen = executionUrls.length;
        const fullscreenTrigger = page.getByRole("button", {
          name: "View fullscreen",
        });
        await fullscreenTrigger.click();
        const fullscreenDialog = page.getByRole("dialog", {
          name: /fullscreen Table$/,
        });
        await expect(fullscreenDialog).toBeVisible();
        await expect(fullscreenDialog.locator("tbody tr[data-row-id]")).not.toHaveCount(0);
        expect(
          executionUrls.length,
          "opening fullscreen must not create a second Table request state machine",
        ).toBe(requestCountBeforeFullscreen);
        await page.keyboard.press("Escape");
        await expect(fullscreenDialog).toBeHidden();
        expect(
          allExecutionPaths.length >= 1 &&
            allExecutionPaths.every((path) => mediatedTablePath.test(path)),
          "the reference Table must stay bound to its Dashboard placement endpoint",
        ).toBe(true);
      } else {
        const pagination = page.locator(
        '.interactive-data-table__pagination[aria-label="Table pagination"]',
      );
      await expect(pagination).toBeVisible();
      const pageSize = pagination.getByLabel("Rows");
      const previous = pagination.getByRole("button", { name: "Previous page" });
      const next = pagination.getByRole("button", { name: "Next page" });
      await expect(pageSize).toBeVisible();
      await expect(previous).toBeDisabled();
      await expect(next).toBeEnabled();

      const rows = tableViewer.locator("tbody tr[data-row-id]");
      await expect(rows).toHaveCount(10);
      const firstPageFirstRow = await rows.first().getAttribute("data-row-id");
      expect(firstPageFirstRow).toBeTruthy();
      await expect.poll(() => executionUrls.length).toBeGreaterThanOrEqual(1);

      const nextRequestPromise = page.waitForRequest((request) => {
        const url = new URL(request.url());
        return (
          request.method() === "GET" &&
          mediatedTablePath.test(url.pathname) &&
          Boolean(url.searchParams.get("cursor"))
        );
      });
      const nextResponsePromise = page.waitForResponse((response) => {
        const url = new URL(response.url());
        return (
          response.request().method() === "GET" &&
          mediatedTablePath.test(url.pathname) &&
          Boolean(url.searchParams.get("cursor"))
        );
      });
      await next.click();
      const [nextRequest, nextResponse] = await Promise.all([
        nextRequestPromise,
        nextResponsePromise,
      ]);
      expect(nextResponse.ok()).toBe(true);
      const nextUrl = new URL(nextRequest.url());
      expect(mediatedTablePath.test(nextUrl.pathname)).toBe(true);
      expect(nextUrl.searchParams.get("cursor")).toBeTruthy();
      await expect(pagination.getByText("Page 2", { exact: true })).toBeVisible();
      await expect
        .poll(() => rows.first().getAttribute("data-row-id"))
        .not.toBe(firstPageFirstRow);

      const pageSizeRequestPromise = page.waitForRequest((request) => {
        const url = new URL(request.url());
        return (
          request.method() === "GET" &&
          mediatedTablePath.test(url.pathname) &&
          url.searchParams.get("page_size") === "25"
        );
      });
      const pageSizeResponsePromise = page.waitForResponse((response) => {
        const url = new URL(response.url());
        return (
          response.request().method() === "GET" &&
          mediatedTablePath.test(url.pathname) &&
          url.searchParams.get("page_size") === "25"
        );
      });
      await pageSize.selectOption("25");
      const [pageSizeRequest, pageSizeResponse] = await Promise.all([
        pageSizeRequestPromise,
        pageSizeResponsePromise,
      ]);
      expect(pageSizeResponse.ok()).toBe(true);
      const pageSizeUrl = new URL(pageSizeRequest.url());
      expect(mediatedTablePath.test(pageSizeUrl.pathname)).toBe(true);
      expect(pageSizeUrl.searchParams.get("page_size")).toBe("25");
      expect(pageSizeUrl.searchParams.has("cursor")).toBe(false);
      await expect(pagination.getByText("Page 1", { exact: true })).toBeVisible();
      await expect(rows).toHaveCount(25);
      await expect(page.locator(".component-table-preview__header")).toHaveCount(0);
      await expect(rows.first()).toHaveAttribute("data-row-id", firstPageFirstRow!);

      const requestCountBeforeFullscreen = executionUrls.length;
      const fullscreenTrigger = page.getByRole("button", {
        name: "View fullscreen",
      });
      await expect(fullscreenTrigger).toHaveAttribute("aria-haspopup", "dialog");
      await expect(fullscreenTrigger).toHaveAttribute("aria-expanded", "false");
      const fullscreenDialogControlId = await fullscreenTrigger.getAttribute(
        "aria-controls",
      );
      const inlineColumnsTrigger = page.getByRole("button", {
        name: "Choose visible columns",
      });
      const inlineColumnsControlId = await inlineColumnsTrigger.getAttribute(
        "aria-controls",
      );
      expect(fullscreenDialogControlId).toBeTruthy();
      expect(inlineColumnsControlId).toBeTruthy();
      expect(inlineColumnsControlId).not.toBe(fullscreenDialogControlId);
      await fullscreenTrigger.click();
      const fullscreenDialog = page.getByRole("dialog", {
        name: /fullscreen Table$/,
      });
      await expect(fullscreenDialog).toBeVisible();
      await expect(fullscreenTrigger).toHaveAttribute("aria-expanded", "true");
      await expect(
        fullscreenDialog.getByRole("button", { name: "View fullscreen" }),
      ).toHaveCount(0);
      const dialogColumnsTrigger = fullscreenDialog.getByRole("button", {
        name: "Choose visible columns",
      });
      const dialogColumnsControlId = await dialogColumnsTrigger.getAttribute(
        "aria-controls",
      );
      expect(dialogColumnsControlId).toBeTruthy();
      expect(dialogColumnsControlId).not.toBe(inlineColumnsControlId);
      expect(dialogColumnsControlId).not.toBe(fullscreenDialogControlId);
      await expect(
        fullscreenDialog.getByRole("button", { name: "Close fullscreen view" }),
      ).toBeFocused();
      const fullscreenPagination = fullscreenDialog.locator(
        '.interactive-data-table__pagination[aria-label="Table pagination"]',
      );
      const fullscreenRows = fullscreenDialog.locator("tbody tr[data-row-id]");
      await expect(fullscreenPagination.getByLabel("Rows")).toHaveValue("25");
      await expect(fullscreenPagination.getByText("Page 1", { exact: true })).toBeVisible();
      await expect(fullscreenRows).toHaveCount(25);
      await expect(fullscreenRows.first()).toHaveAttribute(
        "data-row-id",
        firstPageFirstRow!,
      );
      expect(
        executionUrls.length,
        "opening fullscreen must not create a second Table request state machine",
      ).toBe(requestCountBeforeFullscreen);

      const fullscreenNextRequest = page.waitForResponse((response) => {
        const url = new URL(response.url());
        return (
          response.request().method() === "GET" &&
          mediatedTablePath.test(url.pathname) &&
          url.searchParams.get("page_size") === "25" &&
          Boolean(url.searchParams.get("cursor"))
        );
      });
      await fullscreenPagination
        .getByRole("button", { name: "Next page" })
        .click();
      expect((await fullscreenNextRequest).ok()).toBe(true);
      await expect(fullscreenPagination.getByText("Page 2", { exact: true })).toBeVisible();
      const fullscreenSecondPageFirstRow = await fullscreenRows
        .first()
        .getAttribute("data-row-id");
      expect(fullscreenSecondPageFirstRow).toBeTruthy();
      await page.keyboard.press("Escape");
      await expect(fullscreenDialog).toBeHidden();
      await expect(fullscreenTrigger).toBeFocused();
      await expect(pagination.getByText("Page 2", { exact: true })).toBeVisible();
      await expect(pageSize).toHaveValue("25");
      await expect(rows.first()).toHaveAttribute(
        "data-row-id",
        fullscreenSecondPageFirstRow!,
      );

      expect(executionUrls.length).toBeGreaterThanOrEqual(4);
      expect(
        allExecutionPaths.length >= 3 &&
          allExecutionPaths.every((path) => mediatedTablePath.test(path)),
        "embedded Table controls must stay bound to the Dashboard placement endpoint",
      ).toBe(true);
      }
      assertNoConsoleErrors();
    } finally {
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("mobile Table fullscreen survives returning to a lazily executed viewer", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);

    try {
      const referenceTable = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "table",
      );
      expect(referenceTable).toBeTruthy();
      const composition = await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: [
              bindGeometryCommand(referenceTable!, `${RUN_ID}-mobile-session`, {
                grid_row: 1,
                grid_column: 1,
                grid_width: 12,
                grid_height: 6,
              }),
              bindGeometryCommand(referenceTable!, `${RUN_ID}-mobile-activity`, {
                grid_row: 7,
                grid_column: 1,
                grid_width: 12,
                grid_height: 6,
              }),
            ],
          },
        }),
      );
      const tablePlacements = composition.dashboard.placements
        .filter(
          (placement) =>
            placement.component?.component_version_id ===
            referenceTable!.component_version_id,
        )
        .sort((left, right) => left.grid_row - right.grid_row);
      const [sessionPlacement, activityPlacement] = tablePlacements;
      expect(sessionPlacement).toBeTruthy();
      expect(activityPlacement).toBeTruthy();

      await page.setViewportSize({ width: 1600, height: 1200 });
      await gotoHydrated(page, `/dashboards/${fixture.id}/view`);
      const sessionCard = page.locator(
        `[data-placement-id="${sessionPlacement!.placement_id}"]`,
      );
      const sessionTrigger = sessionCard.getByRole("button", {
        name: "View fullscreen",
      });
      await expect(sessionCard.locator("tbody tr[data-row-id]")).not.toHaveCount(0, {
        timeout: 30_000,
      });
      await expect(sessionTrigger).toHaveAttribute("aria-expanded", "false");
      await sessionTrigger.click();
      await expect(sessionTrigger).toHaveAttribute("aria-expanded", "true");
      await expect(page.getByRole("dialog")).toBeVisible();
      await page.keyboard.press("Escape");
      await expect(page.getByRole("dialog")).toBeHidden();
      await expect(sessionTrigger).toHaveAttribute("aria-expanded", "false");

      await page
        .getByLabel("Breadcrumb")
        .getByRole("link", { name: "Dashboards", exact: true })
        .click();
      await expect(page).toHaveURL(`${BASE_URL}/dashboards`);
      await page.setViewportSize({ width: 390, height: 844 });
      await page.locator(`a[href="/dashboards/${fixture.id}/view"]`).click();
      await expect(page).toHaveURL(`${BASE_URL}/dashboards/${fixture.id}/view`);
      await expect(page.locator("#module-content")).toHaveAttribute(
        "data-hydration",
        "ready",
      );

      await sessionCard.scrollIntoViewIfNeeded();
      await expect(sessionCard.locator("tbody tr[data-row-id]")).not.toHaveCount(0, {
        timeout: 30_000,
      });
      await sessionTrigger.click();
      await expect(sessionTrigger).toHaveAttribute("aria-expanded", "true");
      await expect(page.getByRole("dialog")).toBeVisible();
      await page.keyboard.press("Escape");
      await expect(page.getByRole("dialog")).toBeHidden();

      const activityCard = page.locator(
        `[data-placement-id="${activityPlacement!.placement_id}"]`,
      );
      await activityCard.scrollIntoViewIfNeeded();
      await expect(activityCard.locator("tbody tr[data-row-id]")).not.toHaveCount(0, {
        timeout: 30_000,
      });
      await activityCard.getByRole("button", { name: "View fullscreen" }).click();
      await expect(page.getByRole("dialog")).toBeVisible();
      assertNoConsoleErrors();
    } finally {
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("invalid inspector geometry resets without dirtying or saving the layout", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);

    try {
      const option = fixture.composition.available_component_versions[0];
      const savedBefore = await expectJson<DashboardComposition>(
        await page.request.put(`/api/admin/dashboards/${fixture.id}/composition`, {
          data: {
            commands: [bindCommand(option, `${RUN_ID}-invalid-geometry`, 1)],
          },
        }),
      );
      const canonicalGeometry = savedBefore.dashboard.placements[0];
      const compositionWrites: string[] = [];
      page.on("request", (request) => {
        if (
          request.method() === "PUT" &&
          new URL(request.url()).pathname ===
            `/api/admin/dashboards/${fixture.id}/composition`
        ) {
          compositionWrites.push(request.url());
        }
      });

      await gotoHydrated(page, `/dashboards/${fixture.id}/edit`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      await page.locator(".dashboard-composition-tile").click();
      await page.getByRole("button", { name: "Placement details" }).click();

      const saveLayout = page.getByRole("button", { name: "Save layout" });
      await expect(saveLayout).toBeDisabled();
      for (const geometryCase of [
        {
          name: "Row",
          invalid: "241",
          canonical: canonicalGeometry.grid_row,
          error: "Placement could not move:",
        },
        {
          name: "Column",
          invalid: "13",
          canonical: canonicalGeometry.grid_column,
          error: "Placement could not move:",
        },
        {
          name: "Width",
          invalid: "13",
          canonical: canonicalGeometry.grid_width,
          error: "Placement size was not changed:",
        },
        {
          name: "Height",
          invalid: "241",
          canonical: canonicalGeometry.grid_height,
          error: "Placement size was not changed:",
        },
      ]) {
        const input = page.getByRole("spinbutton", { name: geometryCase.name });
        await expect(input).toHaveValue(String(geometryCase.canonical));
        await input.fill(geometryCase.invalid);
        await input.blur();
        await expect(input).toHaveValue(String(geometryCase.canonical));
        await expect(page.locator(".dashboard-editor__status")).toContainText(
          geometryCase.error,
        );
        await expect(saveLayout).toBeDisabled();
      }
      expect(
        compositionWrites,
        "invalid inspector geometry must not trigger a composition save",
      ).toEqual([]);

      const savedAfter = await expectJson<DashboardComposition>(
        await page.request.get(`/api/admin/dashboards/${fixture.id}/composition`),
      );
      expect(savedAfter.dashboard.placements[0]).toMatchObject({
        grid_row: canonicalGeometry.grid_row,
        grid_column: canonicalGeometry.grid_column,
        grid_width: canonicalGeometry.grid_width,
        grid_height: canonicalGeometry.grid_height,
      });
      assertNoConsoleErrors();
    } finally {
      await deleteDashboardFixture(page, fixture.id);
    }
  });

  test("JavaScript-disabled direct loads preserve useful and redacted-safe Dashboard SSR", async ({
    browser,
  }) => {
    test.setTimeout(120_000);
    const context = await browser.newContext({
      baseURL: BASE_URL,
      javaScriptEnabled: false,
    });

    try {
      const page = await context.newPage();
      await signInAsAdmin(page);
      await ensureDemoSeed(page);
      const adminDashboards = await expectJson<DashboardSummary[]>(
        await page.request.get("/api/dashboards"),
      );
      const dashboard = adminDashboards.find(
        (candidate) =>
          candidate.placement_count > 0 && candidate.visibility_nodes.length > 0,
      );
      expect(
        dashboard,
        "reference composition should expose a visible Dashboard with placements",
      ).toBeTruthy();
      expect(dashboard!.placement_count).toBeGreaterThan(0);
      const adminDefinition = await expectJson<DashboardDefinition>(
        await page.request.get(`/api/dashboards/${dashboard!.id}`),
      );
      expect(adminDefinition.placements).toHaveLength(dashboard!.placement_count);

      await page.goto("/dashboards");
      await expect(page.getByRole("heading", { level: 1, name: "Dashboards" })).toBeVisible();
      await expect(page.locator(`[data-dashboard-id="${dashboard!.id}"]`)).toContainText(
        String(dashboard!.placement_count),
      );

      await page.goto(`/dashboards/${dashboard!.id}`);
      await expect(
        page.getByRole("heading", { level: 1, name: dashboard!.name }),
      ).toBeVisible();
      await expect(page.locator(".dashboard-placement-card")).toHaveCount(
        dashboard!.placement_count,
      );

      await page.goto(`/dashboards/${dashboard!.id}/edit`);
      await expect(
        page.getByRole("heading", { level: 1, name: dashboard!.name }),
      ).toBeVisible();
      await expect(page.getByText("Dashboard builder", { exact: true })).toBeVisible();
      await expect(page.locator(".dashboard-composition-tile")).toHaveCount(
        dashboard!.placement_count,
      );

      await page.goto(`/dashboards/${dashboard!.id}/view`);
      await expect(
        page.getByRole("heading", { level: 1, name: dashboard!.name }),
      ).toBeVisible();
      await expect(page.locator(".dashboard-viewer-placement")).toHaveCount(
        dashboard!.placement_count,
      );

      await signIn(
        page,
        "scoped-sprint7a@tessara.local",
        "tessara-sprint-7a-scoped",
      );
      const operatorDashboards = await expectJson<DashboardSummary[]>(
        await page.request.get("/api/dashboards"),
      );
      const operatorDashboard = operatorDashboards.find(
        (candidate) => candidate.id === dashboard!.id,
      );
      expect(
        operatorDashboard,
        "scoped operator should see the reference Dashboard",
      ).toBeTruthy();
      expect(operatorDashboard!.placement_count).toBe(dashboard!.placement_count);
      const operatorDefinition = await expectJson<DashboardDefinition>(
        await page.request.get(`/api/dashboards/${dashboard!.id}`),
      );
      expect(operatorDefinition.placements).toHaveLength(dashboard!.placement_count);
      const unavailablePlacements = operatorDefinition.placements.filter(
        (placement) => placement.availability === "unavailable",
      );
      expect(
        unavailablePlacements.length,
        "reference operator should receive at least one unavailable placement footprint",
      ).toBeGreaterThan(0);
      const redactedPlacements = unavailablePlacements.filter(
        (placement) => placement.component === undefined,
      );

      const hiddenBindings = redactedPlacements.map((hidden) => {
        const adminPlacement = adminDefinition.placements.find(
          (placement) => placement.placement_id === hidden.placement_id,
        );
        expect(adminPlacement?.component, "admin projection should identify hidden binding").toBeTruthy();
        return adminPlacement!.component!;
      });

      await page.goto("/dashboards");
      await expect(page.locator(`[data-dashboard-id="${dashboard!.id}"]`)).toContainText(
        String(dashboard!.placement_count),
      );

      await page.goto(`/dashboards/${dashboard!.id}`);
      await expect(page.locator(".dashboard-placement-card")).toHaveCount(
        dashboard!.placement_count,
      );
      await expect(page.locator(".dashboard-placement-card.is-unavailable")).toHaveCount(
        unavailablePlacements.length,
      );
      const detailHtml = await page.content();
      for (const binding of hiddenBindings) {
        expect(detailHtml).not.toContain(binding.component_version_id);
        expect(detailHtml).not.toContain(binding.component_slug);
      }

      await page.goto(`/dashboards/${dashboard!.id}/view`);
      await expect(page.locator(".dashboard-viewer-placement")).toHaveCount(
        dashboard!.placement_count,
      );
      await expect(page.locator(".dashboard-redacted-placeholder")).toHaveCount(
        unavailablePlacements.length,
      );
      const viewerHtml = await page.content();
      for (const binding of hiddenBindings) {
        expect(viewerHtml).not.toContain(binding.component_version_id);
        expect(viewerHtml).not.toContain(binding.component_slug);
      }
    } finally {
      await context.close();
    }
  });

  test("editor add, direct move, resize, preview, save, and remove preserve dirty-state rules", async ({
    page,
  }) => {
    test.setTimeout(120_000);
    const assertNoConsoleErrors = attachConsoleGuard(page);
    const executionPaths = attachComponentExecutionTracker(page);
    const successfulExecutionPaths = attachSuccessfulComponentExecutionTracker(page);
    await signInAsAdmin(page);
    await ensureDemoSeed(page);
    const fixture = await createDashboardFixture(page);
    const mediatedExecutionPath = new RegExp(
      `^/api/dashboards/${fixture.id}/placements/[^/]+/render/(?:table|bar|line|pie|donut|stat-card)$`,
    );

    try {
      await page.setViewportSize({ width: 1440, height: 1000 });
      await gotoHydrated(page, `/dashboards/${fixture.id}/edit`);
      await expect(
        page.getByRole("heading", { level: 1, name: fixture.name }),
      ).toBeVisible();
      const editorOption = fixture.composition.available_component_versions.find(
        (option) => option.component_type === "stat_card",
      );
      expect(editorOption, "demo seed should expose a Stat Card").toBeTruthy();
      const exactUnsavedPreviewPath =
        `/api/components/${editorOption!.component_slug}/versions/${editorOption!.component_version_id}/stat-card`;
      const isEditorPreviewExecutionPath = (path: string) =>
        path === exactUnsavedPreviewPath || mediatedExecutionPath.test(path);

      await page.getByRole("button", { name: "Components" }).click();
      await page
        .getByRole("combobox", { name: "Filter components by kind" })
        .selectOption("stat_card");
      await page
        .getByRole("button", {
          name: /^Add .+ exact version .+ to Dashboard$/,
        })
        .first()
        .click();
      const tile = page.locator(".dashboard-composition-tile");
      await expect(tile).toHaveCount(1);
      await expect(tile).toHaveAttribute("data-placement-selected", "true");
      const symbolSize = await tile.locator(".dashboard-composition-tile__symbol svg").evaluate((icon) => {
        const style = getComputedStyle(icon);
        return { width: style.width, height: style.height };
      });
      expect(symbolSize).toEqual({ width: "31px", height: "31px" });
      await expect(page.getByRole("button", { name: "Save layout" })).toBeEnabled();
      await expect(
        page.getByRole("button", { name: "Preview Dashboard" }),
      ).toBeDisabled();
      expect(executionPaths).toEqual([]);
      await page
        .getByRole("dialog", { name: "Components" })
        .getByRole("button", { name: "Close Components" })
        .click();

      const pointerRow = editorOption!.default_grid_height + 1;
      const directRow = pointerRow + 1;
      const targetCell = page.locator(
        `[data-placement-grid-cell="true"][data-row="${pointerRow}"][data-column="1"]`,
      );
      await tile.dragTo(targetCell);
      await expect(page.locator(".dashboard-editor__status")).toContainText(
        `Placement moved to row ${pointerRow}, column 1.`,
      );

      await page.getByRole("button", { name: "Placement details" }).click();
      const placementDetails = page.getByRole("dialog", {
        name: "Placement details",
      });
      await expect(placementDetails).toBeVisible();
      const rowInput = page.getByRole("spinbutton", { name: "Row" });
      const widthInput = page.getByRole("spinbutton", { name: "Width" });
      const heightInput = page.getByRole("spinbutton", { name: "Height" });
      await rowInput.fill(String(directRow));
      await rowInput.blur();
      await expect(page.locator(".dashboard-editor__status")).toContainText(
        `Placement moved to row ${directRow}, column 1.`,
      );
      await expect(heightInput).toHaveAttribute("max", String(241 - directRow));
      const currentWidth = Number(await widthInput.inputValue());
      const nextWidth = currentWidth === 4 ? 5 : 4;
      const nextHeight = 7;
      await widthInput.fill(String(nextWidth));
      await widthInput.blur();
      await heightInput.fill(String(nextHeight));
      await heightInput.blur();
      await expect(page.locator(".dashboard-editor__status")).toContainText(
        `Placement resized to ${nextWidth} by ${nextHeight}.`,
      );

      const previewTrigger = page.getByRole("button", { name: "Preview selected" });
      await previewTrigger.click();
      const previewDialog = page.getByRole("dialog", { name: "Selected Component" });
      const previewClose = previewDialog.getByRole("button", { name: "Close" });
      await expect(previewDialog).toBeVisible();
      await expect(
        previewDialog.getByRole("heading", { name: "Selected Component" }),
      ).toBeVisible();
      await expect(previewClose).toBeFocused();
      await expect.poll(() => executionPaths.length).toBeGreaterThan(0);
      expect(executionPaths.every(isEditorPreviewExecutionPath)).toBe(true);
      await expect.poll(() => successfulExecutionPaths.length).toBeGreaterThan(0);
      expect(
        successfulExecutionPaths.every(isEditorPreviewExecutionPath),
      ).toBe(true);
      await expect(
        previewDialog.locator(RENDERED_COMPONENT_CONTENT).first(),
      ).toBeVisible();
      await page.keyboard.press("Escape");
      await expect(previewDialog).toHaveCount(0);
      await expect(previewTrigger).toBeFocused();
      await placementDetails
        .getByRole("button", { name: "Close Placement details" })
        .click();
      await expect(placementDetails).toHaveCount(0);

      await page.getByRole("button", { name: "Save layout" }).click();
      await expect(page.locator(".dashboard-editor__status")).toContainText(
        "Dashboard layout saved. Preview Dashboard is now available.",
      );
      await expect(page.getByRole("button", { name: "Save layout" })).toBeDisabled();
      await expect(
        page.getByRole("button", { name: "Preview Dashboard" }),
      ).toBeEnabled();

      let saved = await expectJson<DashboardComposition>(
        await page.request.get(`/api/admin/dashboards/${fixture.id}/composition`),
      );
      expect(saved.dashboard.placements).toHaveLength(1);
      expect(saved.dashboard.placements[0]).toMatchObject({
        grid_row: directRow,
        grid_column: 1,
        grid_width: nextWidth,
        grid_height: nextHeight,
      });

      await gotoHydrated(page, `/dashboards/${fixture.id}/edit`);
      await expect(tile).toHaveCount(1);
      await tile.click();
      await page.getByRole("button", { name: "Placement details" }).click();
      await expect(page.getByRole("spinbutton", { name: "Height" })).toHaveValue(
        String(nextHeight),
      );

      await page.getByRole("button", { name: "Remove placement" }).click();
      await expect(tile).toHaveCount(0);
      await expect(
        page.getByRole("button", { name: "Preview Dashboard" }),
      ).toBeDisabled();
      await expect(page.getByRole("button", { name: "Save layout" })).toBeEnabled();
      await placementDetails
        .getByRole("button", { name: "Close Placement details" })
        .click();
      await expect(placementDetails).toHaveCount(0);
      await page.getByRole("button", { name: "Save layout" }).click();
      await expect(page.locator(".dashboard-editor__status")).toContainText(
        "Dashboard layout saved. Preview Dashboard is now available.",
      );

      saved = await expectJson<DashboardComposition>(
        await page.request.get(`/api/admin/dashboards/${fixture.id}/composition`),
      );
      expect(saved.dashboard.placements).toHaveLength(0);
      assertNoConsoleErrors();
    } finally {
      await deleteDashboardFixture(page, fixture.id);
    }
  });
});
