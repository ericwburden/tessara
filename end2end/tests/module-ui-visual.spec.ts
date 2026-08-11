import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

async function useTheme(page: Page, theme: "light" | "dark") {
  await page.evaluate((selectedTheme) => {
    localStorage.setItem("tessara.themePreference", selectedTheme);
    document.documentElement.dataset.themePreference = selectedTheme;
    document.documentElement.dataset.theme = selectedTheme;
    const applicationContent = document.querySelector<HTMLElement>(".app-main");
    applicationContent?.scrollTo({ top: 0, left: 0 });
  }, theme);
  await expect(page.locator("html")).toHaveAttribute("data-theme", theme);
}

async function visit(page: Page, path: string, theme: "light" | "dark") {
  await page.goto(path);
  if (path.startsWith("/components") || path.startsWith("/dashboards")) {
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
  }
  await useTheme(page, theme);
  await expect(
    page.locator("[data-hydration=ready], #tessara-module-outlet, #module-content").first(),
  ).toBeVisible();
}

test.describe("canonical module UI visual baselines", () => {
  test.beforeEach(async ({ page }) => signIn(page));

  const referenceComponents = [
    "Reference Label Bar",
    "Reference Label Donut",
    "Reference Label Line",
    "Reference Label Pie",
    "Reference Records Table",
    "Reference Row Count",
  ];

  async function showReferenceComponents(page: Page) {
    await page.getByPlaceholder("Search components").fill("Reference");
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
      await visit(page, "/components/sprint-8a-label-bar/edit", theme);
      await expect(page.locator(".mobile-nav__toggle")).toBeVisible();
      await expect(page).toHaveScreenshot(`components-editor-${theme}-390.png`, {
        animations: "disabled",
      });
    });
  }

  test("Components versions at 768 px", async ({ page }) => {
    await page.setViewportSize({ width: 768, height: 900 });
    await visit(page, "/components/sprint-8a-label-bar/versions", "dark");
    await expect(page).toHaveScreenshot("components-versions-dark-768.png", {
      animations: "disabled",
    });
  });

  test("Components viewer at 1280 px", async ({ page }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    await visit(page, "/components/sprint-8a-label-bar/view", "dark");
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
});
