import { expect, test, type Browser, type Page } from "@playwright/test";

const THEME_STORAGE_KEY = "tessara.themePreference";

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
  if (
    path.startsWith("/components") ||
    path.startsWith("/dashboards") ||
    path.startsWith("/datasets")
  ) {
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
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
      await mobileNavigation
        .locator(".mobile-nav__scrim")
        .click({ position: { x: 380, y: 100 } });
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
        await page.locator("tbody tr td:nth-child(3)").evaluate((cell) => {
          cell.textContent = "Pinned fixture time";
        });
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
        await page.locator("tbody tr td:nth-child(3)").evaluate((cell) => {
          cell.textContent = "Pinned fixture time";
        });
      }
      await expect(page.locator(".app-main")).toHaveScreenshot(
        `module-parity-with-datasets-${module.name}-dark-1440.png`,
        { animations: "disabled" },
      );
    }
    await expectDatasetZoomContainment(browser);
  });
});
