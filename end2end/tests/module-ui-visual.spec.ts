import { readFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { expect, test, type Browser, type Page } from "@playwright/test";

const THEME_STORAGE_KEY = "tessara.themePreference";
const RESPONSE_BASELINE_DIRECTORY = resolve(
  __dirname,
  "../../docs/audits/sprint-8c-response-ui-baseline",
);

type ResponseBaselineCase = {
  key: string;
  theme: string;
  viewport: string;
  screenshot: string;
  semantic_assertions: { title?: string };
};

const RESPONSE_BASELINE_CASES = (
  JSON.parse(
    readFileSync(join(RESPONSE_BASELINE_DIRECTORY, "baseline-index.json"), "utf8"),
  ) as { cases: ResponseBaselineCase[] }
).cases;

async function signIn(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

async function useTheme(page: Page, theme: "light" | "dark") {
  await page.evaluate(({ key, selectedTheme }) => {
    localStorage.setItem(key, selectedTheme);
    document.documentElement.dataset.themePreference = selectedTheme;
    document.documentElement.dataset.theme = selectedTheme;
    const applicationContent = document.querySelector<HTMLElement>(".app-main");
    applicationContent?.scrollTo({ top: 0, left: 0 });
  }, { key: THEME_STORAGE_KEY, selectedTheme: theme });
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

async function visitDocument(page: Page, path: string) {
  await page.goto(path);
  const stylesheetReadiness = await page.evaluate(() => {
    const declared = Array.from(
      document.querySelectorAll<HTMLLinkElement>('link[rel="stylesheet"][href]'),
    ).map((link) => link.href);
    const loaded = new Set(
      Array.from(document.styleSheets)
        .map((sheet) => sheet.href)
        .filter((href): href is string => href !== null),
    );
    return {
      declared,
      missing: declared.filter((href) => !loaded.has(href)),
    };
  });
  expect(
    stylesheetReadiness.declared.length,
    `document ${path} must declare release-owned stylesheets`,
  ).toBeGreaterThan(0);
  expect(
    stylesheetReadiness.missing,
    `document ${path} must load every declared stylesheet before UI readiness`,
  ).toEqual([]);
  if (
    path.startsWith("/components") ||
    path.startsWith("/dashboards") ||
    path.startsWith("/datasets") ||
    path.startsWith("/responses")
  ) {
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
      { timeout: 30_000 },
    );
  }
  if (path.startsWith("/reference/scoped-records")) {
    await page.waitForLoadState("networkidle");
  }
  await expect(
    page.locator("[data-hydration=ready], #tessara-module-outlet, #module-content").first(),
  ).toBeVisible();
}

async function visit(page: Page, path: string, theme: "light" | "dark") {
  await visitDocument(page, path);
  await useTheme(page, theme);
}

async function visitWithStoredThemePrecedence(
  page: Page,
  path: string,
  theme: "light" | "dark",
) {
  await page.emulateMedia({ colorScheme: theme === "light" ? "dark" : "light" });
  await page.addInitScript(
    ({ key, value }) => localStorage.setItem(key, value),
    { key: THEME_STORAGE_KEY, value: theme },
  );
  await visitDocument(page, path);
  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", theme);
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

async function visitWithSystemThemeFallback(
  page: Page,
  path: string,
  theme: "light" | "dark",
) {
  await page.emulateMedia({ colorScheme: theme });
  await page.addInitScript(
    (key) => localStorage.removeItem(key),
    THEME_STORAGE_KEY,
  );
  await visitDocument(page, path);
  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", "system");
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

type DatasetVisualFixture = { id: string; slug: string };

type ResponseVisualFixture = {
  id: string;
  form_name: string;
  status: "draft" | "submitted";
};

async function referenceResponses(page: Page): Promise<{
  draft: ResponseVisualFixture;
  submitted: ResponseVisualFixture;
}> {
  const response = await page.request.get("/api/responses");
  expect(response.ok(), await response.text()).toBeTruthy();
  const responses = await response.json();
  expect(Array.isArray(responses), "Response list must be the canonical array response").toBe(true);
  const draft = (responses as ResponseVisualFixture[]).find(
    (candidate) => candidate.status === "draft",
  );
  const submitted = (responses as ResponseVisualFixture[]).find(
    (candidate) => candidate.status === "submitted",
  );
  expect(draft, "draft Response visual fixture").toBeDefined();
  expect(submitted, "submitted Response visual fixture").toBeDefined();
  return { draft: draft!, submitted: submitted! };
}

type PixelRegion = { x: number; y: number; width: number; height: number };

type ResponseVisualContinuity = {
  ignoreSelectors?: string[];
  comparisonRegions?: PixelRegion[];
  maxDiffPixelRatio?: number;
  maxMeanColorDelta?: number;
  minComparedPixelRatio?: number;
};

async function expectAcceptedResponseVisualFrame(
  page: Page,
  baselineKey: string,
  options: ResponseVisualContinuity = {},
) {
  const matchingCases = RESPONSE_BASELINE_CASES.filter(
    (candidate) => candidate.key === baselineKey,
  );
  expect(matchingCases, `accepted Response baseline '${baselineKey}'`).toHaveLength(1);
  const baseline = matchingCases[0];
  const viewport = page.viewportSize();
  expect(viewport, "Response visual comparison requires a fixed viewport").not.toBeNull();
  expect(`${viewport!.width}x${viewport!.height}`).toBe(baseline.viewport);
  if (baseline.theme === "light" || baseline.theme === "dark") {
    await expect(page.locator("html")).toHaveAttribute("data-theme", baseline.theme);
  }
  const layout = await page.evaluate(() => ({
    clientWidth: document.documentElement.clientWidth,
    scrollWidth: document.documentElement.scrollWidth,
    title: document.title,
  }));
  if (baseline.semantic_assertions.title !== undefined) {
    expect(layout.title).toBe(baseline.semantic_assertions.title);
  } else {
    expect(layout.title).toMatch(/ · Tessara$/);
  }
  expect(
    layout.scrollWidth <= layout.clientWidth + 1,
    "visual route must not create document-level horizontal overflow",
  ).toBe(true);
  const currentPng = await page.screenshot({ animations: "disabled" });
  const acceptedPng = readFileSync(
    join(RESPONSE_BASELINE_DIRECTORY, baseline.screenshot),
  );
  for (const png of [currentPng, acceptedPng]) {
    expect(Array.from(png.subarray(0, 8))).toEqual([
      137, 80, 78, 71, 13, 10, 26, 10,
    ]);
    expect(png.byteLength, "Response visual frame must be nonempty").toBeGreaterThan(
      5_000,
    );
  }

  const ignoredRects: PixelRegion[] = [];
  for (const selector of options.ignoreSelectors ?? []) {
    ignoredRects.push(
      ...(await page.locator(selector).evaluateAll((elements) =>
        elements.map((element) => {
          const rect = element.getBoundingClientRect();
          return {
            x: Math.max(0, Math.floor(rect.x) - 2),
            y: Math.max(0, Math.floor(rect.y) - 2),
            width: Math.ceil(rect.width) + 4,
            height: Math.ceil(rect.height) + 4,
          };
        }),
      )),
    );
  }

  const comparison = await page.evaluate(
    async ({ actual, accepted, ignored, regions }) => {
      const decode = async (base64: string) => {
        const binary = atob(base64);
        const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0));
        const bitmap = await createImageBitmap(new Blob([bytes], { type: "image/png" }));
        const canvas = document.createElement("canvas");
        canvas.width = bitmap.width;
        canvas.height = bitmap.height;
        const context = canvas.getContext("2d", { willReadFrequently: true });
        if (context === null) throw new Error("2D canvas is unavailable");
        context.drawImage(bitmap, 0, 0);
        bitmap.close();
        return {
          width: canvas.width,
          height: canvas.height,
          pixels: context.getImageData(0, 0, canvas.width, canvas.height).data,
        };
      };
      const [actualImage, acceptedImage] = await Promise.all([
        decode(actual),
        decode(accepted),
      ]);
      if (
        actualImage.width !== acceptedImage.width ||
        actualImage.height !== acceptedImage.height
      ) {
        return {
          actualWidth: actualImage.width,
          actualHeight: actualImage.height,
          acceptedWidth: acceptedImage.width,
          acceptedHeight: acceptedImage.height,
          comparedPixels: 0,
          totalPixels: actualImage.width * actualImage.height,
          diffPixelRatio: 1,
          meanColorDelta: 255,
        };
      }
      const contains = (rect: PixelRegion, x: number, y: number) =>
        x >= rect.x &&
        y >= rect.y &&
        x < rect.x + rect.width &&
        y < rect.y + rect.height;
      let comparedPixels = 0;
      let differentPixels = 0;
      let colorDelta = 0;
      for (let y = 0; y < actualImage.height; y += 1) {
        for (let x = 0; x < actualImage.width; x += 1) {
          if (regions.length > 0 && !regions.some((region) => contains(region, x, y))) {
            continue;
          }
          if (ignored.some((rect) => contains(rect, x, y))) continue;
          const offset = (y * actualImage.width + x) * 4;
          const red = Math.abs(actualImage.pixels[offset] - acceptedImage.pixels[offset]);
          const green = Math.abs(
            actualImage.pixels[offset + 1] - acceptedImage.pixels[offset + 1],
          );
          const blue = Math.abs(
            actualImage.pixels[offset + 2] - acceptedImage.pixels[offset + 2],
          );
          const alpha = Math.abs(
            actualImage.pixels[offset + 3] - acceptedImage.pixels[offset + 3],
          );
          comparedPixels += 1;
          colorDelta += (red + green + blue + alpha) / 4;
          if (Math.max(red, green, blue, alpha) > 32) differentPixels += 1;
        }
      }
      return {
        actualWidth: actualImage.width,
        actualHeight: actualImage.height,
        acceptedWidth: acceptedImage.width,
        acceptedHeight: acceptedImage.height,
        comparedPixels,
        totalPixels: actualImage.width * actualImage.height,
        diffPixelRatio:
          comparedPixels === 0 ? 1 : differentPixels / comparedPixels,
        meanColorDelta: comparedPixels === 0 ? 255 : colorDelta / comparedPixels,
      };
    },
    {
      actual: currentPng.toString("base64"),
      accepted: acceptedPng.toString("base64"),
      ignored: ignoredRects,
      regions: options.comparisonRegions ?? [],
    },
  );
  expect(
    [comparison.actualWidth, comparison.actualHeight],
    `current '${baselineKey}' dimensions must match its accepted baseline`,
  ).toEqual([comparison.acceptedWidth, comparison.acceptedHeight]);
  expect(
    comparison.comparedPixels / comparison.totalPixels,
    `current '${baselineKey}' must retain a material accepted pixel comparison region`,
  ).toBeGreaterThanOrEqual(options.minComparedPixelRatio ?? 0.9);
  expect(
    comparison.diffPixelRatio,
    `current '${baselineKey}' must remain pixel-continuous with its accepted baseline`,
  ).toBeLessThanOrEqual(options.maxDiffPixelRatio ?? 0.05);
  expect(
    comparison.meanColorDelta,
    `current '${baselineKey}' must retain the accepted frame's color and spatial structure`,
  ).toBeLessThanOrEqual(options.maxMeanColorDelta ?? 4);
}

function attachVisualRuntimeGuard(page: Page) {
  const errors: string[] = [];
  const externalRequests: string[] = [];
  let applicationOrigin: string | undefined;
  page.on("console", (message) => {
    if (message.type() === "error") errors.push(message.text());
  });
  page.on("pageerror", (error) => errors.push(error.message));
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (request.isNavigationRequest() && request.frame() === page.mainFrame()) {
      applicationOrigin ??= url.origin;
    }
    if (
      applicationOrigin !== undefined &&
      (url.protocol === "http:" || url.protocol === "https:") &&
      url.origin !== applicationOrigin
    ) {
      externalRequests.push(request.url());
    }
  });
  return () => {
    expect(
      errors,
      `Response visual route must keep the browser console clean:\n${errors.join("\n")}`,
    ).toEqual([]);
    expect(
      externalRequests,
      `Response visual route must not fetch external assets:\n${externalRequests.join("\n")}`,
    ).toEqual([]);
  };
}

async function referenceDataset(page: Page): Promise<DatasetVisualFixture> {
  const response = await page.request.get("/api/datasets");
  expect(response.ok(), await response.text()).toBeTruthy();
  const datasets = await response.json();
  expect(Array.isArray(datasets), "Dataset list must be the canonical array response").toBe(true);
  const dataset = (datasets as DatasetVisualFixture[]).find(
    (candidate) => candidate.slug === "base-responses",
  );
  expect(dataset, "base-responses visual fixture").toBeDefined();
  return dataset!;
}

async function expectDatasetZoomContainment(browser: Browser) {
  const context = await browser.newContext({
    viewport: { width: 640, height: 450 },
    deviceScaleFactor: 2,
    colorScheme: "light",
  });
  try {
    const page = await context.newPage();
    await signIn(page);
    await visitWithSystemThemeFallback(page, "/datasets", "light");
    const metrics = await page.evaluate(() => ({
      clientWidth: document.documentElement.clientWidth,
      devicePixelRatio: window.devicePixelRatio,
      innerWidth: window.innerWidth,
      scrollWidth: document.documentElement.scrollWidth,
    }));
    expect(metrics.innerWidth, "200% zoom CSS viewport").toBe(640);
    expect(metrics.devicePixelRatio, "200% zoom output scale").toBe(2);
    expect(
      metrics.scrollWidth <= metrics.clientWidth + 1,
      "Dataset directory should not create document-level horizontal overflow at 200% zoom",
    ).toBe(true);
  } finally {
    await context.close();
  }
}

async function pinScopedRecordsVisualFacts(page: Page) {
  const accessSummary = page.locator(".scoped-records-scope-summary");
  await expect(accessSummary).toContainText(
    /Read access across \d+ accessible Organizations/,
  );
  const organizationOptions = page.locator(
    'select[aria-label="Filter by Organization"] option',
  );
  await expect
    .poll(() => organizationOptions.count())
    .toBeGreaterThanOrEqual(5);
  await expect(organizationOptions.first()).toHaveText(
    "All accessible Organizations",
  );
  await expect(
    page.locator('select[aria-label="Filter by Organization"]'),
  ).toContainText(/Disjoint Organization.*Reference Organization/s);
  await accessSummary.evaluate((summary) => {
    summary.innerHTML = summary.innerHTML
      .replaceAll(/\d+ accessible/g, "N accessible")
      .replaceAll(/\d+ include manage/g, "N include manage");
  });
  await organizationOptions.evaluateAll((options) => {
    options.forEach((option, index) => {
      option.textContent =
        index === 0 ? "All accessible Organizations" : "Accessible Organization";
    });
  });
  await page.locator("tbody tr td:nth-child(3)").evaluate((cell) => {
    cell.textContent = "Pinned fixture time";
  });
}

test.describe("canonical module UI visual baselines", () => {
  test.beforeEach(async ({ page }) => signIn(page));

  const referenceComponents = [
    "Dataset Chart",
    "Dataset Disjoint Probe",
    "Dataset Row Count",
    "Dataset Table",
  ];

  async function showReferenceComponents(page: Page) {
    await page.getByPlaceholder("Search components").fill("Dataset");
    await expect(page.locator('tbody tr [scope="row"]')).toHaveText(
      referenceComponents,
    );
    for (const name of referenceComponents) {
      const row = page
        .locator("tbody tr")
        .filter({ has: page.getByText(name, { exact: true }) });
      await expect(row).toHaveCount(1);
      await expect(
        row.getByRole("link", { name: "Edit component", exact: true }),
      ).toBeVisible();
      await expect(
        row.getByRole("link", {
          name: "View component versions",
          exact: true,
        }),
      ).toBeVisible();
    }
  }

  async function showSprint8BDatasetComponents(page: Page) {
    const expected = [
      "Dataset Chart",
      "Dataset Disjoint Probe",
      "Dataset Row Count",
      "Dataset Table",
    ];
    await page.getByPlaceholder("Search components").fill("Dataset");
    await expect(page.locator('tbody tr [scope="row"]')).toHaveText(expected);
    for (const name of expected) {
      const row = page
        .locator("tbody tr")
        .filter({ has: page.getByText(name, { exact: true }) });
      await expect(row).toHaveCount(1);
    }
  }

  for (const theme of ["light", "dark"] as const) {
    test(`Components directory at 1280 px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 900 });
      await visit(page, "/components", theme);
      await showReferenceComponents(page);
      await expect(page.locator(".components-list-mobile-cards")).toBeHidden();
      await expect(page.locator(".mobile-nav__toggle")).toBeHidden();
      await expect(page.locator(".mobile-nav__panel")).toBeHidden();
      await expect(page).toHaveScreenshot(`components-directory-${theme}-1280.png`, {
        animations: "disabled",
      });
    });

    test(`Components editor at 390 px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width: 390, height: 844 });
      await visit(page, "/components/dataset-table/edit", theme);
      const mobileNavigation = page.locator(".mobile-nav");
      const mobileToggle = mobileNavigation.locator(".mobile-nav__toggle");
      await expect(mobileToggle).toBeVisible();
      await mobileToggle.click();
      await expect(mobileToggle).toHaveAttribute("aria-expanded", "true");
      await expect(mobileNavigation.locator(".mobile-nav__panel")).toBeVisible();
      await expect(
        mobileNavigation.getByRole("link", { name: "Datasets", exact: true }),
      ).toBeVisible();
      await mobileNavigation.locator(".mobile-nav__scrim").click();
      await expect(mobileToggle).toHaveAttribute("aria-expanded", "false");
      await expect(mobileNavigation.locator(".mobile-nav__panel")).toBeHidden();
      await expect(page).toHaveScreenshot(`components-editor-${theme}-390.png`, {
        animations: "disabled",
      });
    });
  }

  test("Components versions at 768 px", async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 900 });
    await visit(page, "/components/dataset-table/versions", "dark");
    await expect(page).toHaveScreenshot("components-versions-dark-768.png", {
      animations: "disabled",
    });
  });

  test("Components viewer at 1280 px", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await visit(page, "/components/dataset-table/view", "dark");
    await expect(
      page
        .locator(
          ".component-table-viewer__table, .component-d3-chart__surface, .component-stat-card",
        )
        .first(),
    ).toBeVisible();
    const viewerRows = page.locator(".component-table-viewer__table tbody tr");
    await expect(viewerRows).toHaveCount(2);
    await viewerRows.evaluateAll((rows) => {
      const pinnedRows = [
        ["Pinned row A", "10"],
        ["Pinned row B", "20"],
      ];
      rows.forEach((row, rowIndex) => {
        Array.from(row.querySelectorAll("td")).forEach((cell, columnIndex) => {
          cell.textContent = pinnedRows[rowIndex]?.[columnIndex] ?? "Pinned value";
        });
      });
    });
    await expect(page).toHaveScreenshot("components-viewer-dark-1280.png", {
      animations: "disabled",
    });
  });

  test("Components, Dashboards, and Scoped Records share one module canvas", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    for (const module of [
      { path: "/components", name: "components", title: "Components" },
      { path: "/dashboards", name: "dashboards", title: "Dashboards" },
      {
        path: "/reference/scoped-records",
        name: "scoped-records",
        title: "Scoped Records",
      },
    ]) {
      await visit(page, module.path, "dark");
      await expect(page.locator(".top-app-bar__title")).toHaveText(module.title);
      if (module.name === "components") {
        await showReferenceComponents(page);
      }
      if (module.name === "scoped-records") {
        await pinScopedRecordsVisualFacts(page);
      }
      await expect(page.locator(".app-main")).toHaveScreenshot(
        `module-parity-${module.name}-dark-1280.png`,
        { animations: "disabled" },
      );
    }
  });

  for (const theme of ["light", "dark"] as const) {
    test(`Datasets directory at 1440 px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width: 1440, height: 1000 });
      await referenceDataset(page);
      await visitWithStoredThemePrecedence(page, "/datasets", theme);
      await expect(page.locator("#module-content")).toHaveAttribute(
        "data-hydration",
        "ready",
      );
      await expect(page.locator(".dataset-table-section > .table-wrap")).toBeVisible();
      await expect(page.locator(".related-work-mobile-cards")).toBeHidden();
      await expect(page).toHaveScreenshot(`datasets-directory-${theme}-1440.png`, {
        animations: "disabled",
      });
    });

    test(`Datasets editor at 390 px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width: 390, height: 844 });
      await visitWithSystemThemeFallback(page, "/datasets/new", theme);
      await expect(page.locator("#module-content")).toHaveAttribute(
        "data-hydration",
        "ready",
      );
      await expect(page).toHaveScreenshot(`datasets-editor-${theme}-390.png`, {
        animations: "disabled",
      });
    });
  }

  test("Datasets revisions at 1024 px", async ({ page }) => {
    await page.setViewportSize({ width: 1024, height: 1366 });
    const dataset = await referenceDataset(page);
    await visit(page, `/datasets/${dataset.id}/revisions`, "dark");
    await page.locator("tbody tr td:nth-child(6)").evaluate((cell) => {
      cell.textContent = "Pinned fixture time";
    });
    await expect(page).toHaveScreenshot("datasets-revisions-dark-1024.png", {
      animations: "disabled",
    });
  });

  test("Datasets preview at 1440 px", async ({ page }) => {
    await page.setViewportSize({ width: 1440, height: 1000 });
    const dataset = await referenceDataset(page);
    await visit(page, `/datasets/${dataset.id}/preview`, "light");
    await expect(page.locator(".dataset-preview-page")).toBeVisible();
    await page.getByRole("button", { name: "Sort and filter Label" }).click();
    await page.getByRole("menuitem", { name: "Sort ascending" }).click();
    await expect(page.locator("tbody tr").first()).toContainText("Corrected complete values");
    await expect(page).toHaveScreenshot("datasets-preview-light-1440.png", {
      animations: "disabled",
    });
  });

  test("Datasets, Components, Dashboards, and Scoped Records share one module canvas", async ({
    page,
    browser,
  }) => {
    test.setTimeout(90_000);
    await page.setViewportSize({ width: 1440, height: 1000 });
    for (const module of [
      { path: "/datasets", name: "datasets", title: "Datasets" },
      { path: "/components", name: "components", title: "Components" },
      { path: "/dashboards", name: "dashboards", title: "Dashboards" },
      {
        path: "/reference/scoped-records",
        name: "scoped-records",
        title: "Scoped Records",
      },
    ]) {
      await visit(page, module.path, "dark");
      await expect(page.locator(".top-app-bar__title")).toHaveText(module.title);
      await expect(page.locator(".app-main")).toHaveAttribute(
        "aria-label",
        "Application content",
      );
      if (module.name === "components") {
        await showSprint8BDatasetComponents(page);
      }
      if (module.name === "scoped-records") {
        await pinScopedRecordsVisualFacts(page);
      }
      await expect(page.locator(".app-main")).toHaveScreenshot(
        `module-parity-with-datasets-${module.name}-dark-1440.png`,
        { animations: "disabled" },
      );
    }
    await expectDatasetZoomContainment(browser);
  });

  test("Responses directory at 1440 px (light)", async ({ page }) => {
    const assertCleanRuntime = attachVisualRuntimeGuard(page);
    await page.setViewportSize({ width: 1440, height: 1000 });
    await referenceResponses(page);
    await visitWithStoredThemePrecedence(page, "/responses", "light");
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    await expect(page.locator(".responses-list .table-wrap")).toBeVisible();
    await expect(page.locator(".responses-mobile-cards")).toBeHidden();
    await expect(page.locator(".top-app-bar__title")).toHaveText("Responses");
    await expectAcceptedResponseVisualFrame(page, "response-directory", {
      ignoreSelectors: [".responses-list time"],
      // The immutable pre-extraction frame contains historical fixture rows.
      // Retain pixel continuity across the complete current top bar and the
      // stable Response heading, search, and table-header presentation.
      comparisonRegions: [{ x: 288, y: 0, width: 1152, height: 278 }],
      maxDiffPixelRatio: 0.025,
      maxMeanColorDelta: 3,
      minComparedPixelRatio: 0.22,
    });
    assertCleanRuntime();
  });

  test("Responses start at 390 px (dark)", async ({ page }) => {
    const assertCleanRuntime = attachVisualRuntimeGuard(page);
    await page.setViewportSize({ width: 390, height: 844 });
    await visitWithSystemThemeFallback(page, "/responses/new", "dark");
    await expect(
      page.getByRole("heading", { name: "Start Response", exact: true }),
    ).toBeVisible();
    await expect(
      page.locator(".response-start-form, .organization-state").first(),
    ).toBeVisible();
    const mobileNavigation = page.locator(".mobile-nav");
    const mobileToggle = mobileNavigation.locator(".mobile-nav__toggle");
    await expect(mobileToggle).toBeVisible();
    await mobileToggle.click();
    await expect(mobileToggle).toHaveAttribute("aria-expanded", "true");
    await expect(
      mobileNavigation.getByRole("link", { name: "Responses", exact: true }),
    ).toBeVisible();
    await mobileNavigation.locator(".mobile-nav__scrim").click();
    await expectAcceptedResponseVisualFrame(page, "response-start");
    assertCleanRuntime();
  });

  test("Responses draft detail at 1024 px (light)", async ({ page }) => {
    const assertCleanRuntime = attachVisualRuntimeGuard(page);
    await page.setViewportSize({ width: 1024, height: 1366 });
    const { draft } = await referenceResponses(page);
    await visit(page, `/responses/${draft.id}`, "light");
    await expect(
      page.getByRole("heading", { name: "Response Detail", exact: true }),
    ).toBeVisible();
    await expect(page.locator(".response-detail-content")).toContainText(draft.form_name);
    await expect(page.getByRole("link", { name: "Edit Draft", exact: true })).toBeVisible();
    await expectAcceptedResponseVisualFrame(page, "response-draft-detail", {
      ignoreSelectors: [".response-detail-content time"],
      // The accepted frame carries the historical Demo Partner fixture. Keep
      // pixel continuity through the complete current top bar, breadcrumb,
      // owner canvas heading, and stable section label before fixture content.
      comparisonRegions: [{ x: 76, y: 0, width: 948, height: 210 }],
      maxDiffPixelRatio: 0.025,
      maxMeanColorDelta: 3,
      minComparedPixelRatio: 0.14,
    });
    assertCleanRuntime();
  });

  test("Responses draft editor at 1440 px (dark)", async ({ page }) => {
    const assertCleanRuntime = attachVisualRuntimeGuard(page);
    await page.setViewportSize({ width: 1440, height: 1000 });
    const { draft } = await referenceResponses(page);
    await visitWithStoredThemePrecedence(page, `/responses/${draft.id}/edit`, "dark");
    const editor = page.locator(".response-edit-form");
    await expect(editor).toBeVisible();
    await expect(editor).toContainText(draft.form_name);
    await expect(editor.getByRole("button", { name: "Save Draft" })).toBeVisible();
    await expect(editor.getByRole("button", { name: "Submit Response" })).toBeVisible();
    await expectAcceptedResponseVisualFrame(page, "response-draft-edit", {
      // Retain the complete current top bar, breadcrumb, owner canvas heading,
      // and editor frame before canonical fixture-specific fields begin.
      comparisonRegions: [{ x: 288, y: 0, width: 1152, height: 192 }],
      maxDiffPixelRatio: 0.025,
      maxMeanColorDelta: 3,
      minComparedPixelRatio: 0.15,
    });
    assertCleanRuntime();
  });

  test("Responses submitted detail at 390 px (light)", async ({ page }) => {
    const assertCleanRuntime = attachVisualRuntimeGuard(page);
    await page.setViewportSize({ width: 390, height: 844 });
    const { submitted } = await referenceResponses(page);
    await visitWithSystemThemeFallback(page, `/responses/${submitted.id}`, "light");
    await expect(page.locator(".response-detail-content")).toContainText(
      submitted.form_name,
    );
    await expect(page.getByText("Submitted", { exact: true }).first()).toBeVisible();
    await expect(page.getByRole("link", { name: "Edit Draft", exact: true })).toHaveCount(0);
    await expectAcceptedResponseVisualFrame(page, "response-submitted-detail", {
      ignoreSelectors: [".response-detail-content time"],
      // Preserve the full mobile top bar, breadcrumb, canvas heading, and
      // stable Response section label while fixture identity remains semantic.
      comparisonRegions: [{ x: 0, y: 0, width: 390, height: 270 }],
      maxDiffPixelRatio: 0.025,
      maxMeanColorDelta: 3,
      minComparedPixelRatio: 0.31,
    });
    assertCleanRuntime();
  });

  test("Responses Module Management at 1440 px (light)", async ({ page }) => {
    const assertCleanRuntime = attachVisualRuntimeGuard(page);
    await page.setViewportSize({ width: 1440, height: 1000 });
    await visit(page, "/administration/modules/tessara.responses", "light");
    await expect(
      page.getByRole("heading", { level: 1, name: "Responses", exact: true }),
    ).toBeVisible();
    await expect(page.getByText("Independently deployed", { exact: true })).toBeVisible();
    await expect(page.getByText("Healthy and enabled", { exact: true })).toBeVisible();
    await page.getByRole("tab", { name: "Dependencies" }).click();
    await expect(
      page.locator('.module-detail-sections > [data-module-section="dependencies"]'),
    ).toContainText("tessara.forms.form-version-schema");
    await expectAcceptedResponseVisualFrame(page, "module-management-response", {
      // Independent ownership deliberately changes management content, while
      // separately owned navigation has grown since acceptance. The complete
      // current top bar and stable management breadcrumb remain pixel-exact;
      // the assertions above pin the new owner semantics.
      comparisonRegions: [{ x: 288, y: 0, width: 1152, height: 150 }],
      maxDiffPixelRatio: 0.025,
      maxMeanColorDelta: 3,
      minComparedPixelRatio: 0.12,
    });
    assertCleanRuntime();
  });
});
