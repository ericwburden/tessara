import { expect, test, type Page } from "@playwright/test";

async function signIn(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

async function useTheme(page: Page, theme: "light" | "dark") {
  await page.getByRole("button", { name: "Theme options" }).click();
  await page
    .getByRole("menuitemradio", {
      name: theme === "light" ? "Light" : "Dark",
      exact: true,
    })
    .click();
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

  for (const theme of ["light", "dark"] as const) {
    test(`Components directory at 1280 px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width: 1280, height: 900 });
      await visit(page, "/components", theme);
      const actions = page.locator(".data-table__action-group .icon-button");
      await expect(actions).toHaveCount(14);
      await expect(actions.first().locator("svg")).toBeVisible();
      await expect(page).toHaveScreenshot(`components-directory-${theme}-1280.png`, {
        animations: "disabled",
      });
    });

    test(`Components editor at 390 px (${theme})`, async ({ page }) => {
      await page.setViewportSize({ width: 390, height: 844 });
      await visit(page, "/components/sprint-8a-label-bar/edit", theme);
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
    await expect(page).toHaveScreenshot("components-viewer-dark-1280.png", {
      animations: "disabled",
    });
  });

  test("Components, Dashboards, and Scoped Records share one module canvas", async ({
    page,
  }) => {
    await page.setViewportSize({ width: 1280, height: 900 });
    for (const module of [
      { path: "/components", name: "components" },
      { path: "/dashboards", name: "dashboards" },
      { path: "/reference/scoped-records", name: "scoped-records" },
    ]) {
      await visit(page, module.path, "dark");
      await expect(page.locator(".app-main")).toHaveScreenshot(
        `module-parity-${module.name}-dark-1280.png`,
        { animations: "disabled" },
      );
    }
  });
});
