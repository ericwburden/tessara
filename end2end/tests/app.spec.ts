import { expect, test, type Page } from "@playwright/test";

function attachConsoleGuard(page: Page) {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      errors.push(message.text());
    }
  });
  page.on("pageerror", (error) => {
    errors.push(error.message);
  });
  return async () => {
    expect(errors, `browser console should stay clean: ${errors.join("\n")}`).toEqual([]);
  };
}

async function pkgScripts(page: Page) {
  return page.evaluate(() =>
    performance
      .getEntriesByType("resource")
      .map((entry) => entry.name)
      .filter((name) => name.includes("/pkg/") && name.endsWith(".js")),
  );
}

async function signInAsAdmin(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: {
      email: "admin@tessara.local",
      password: "tessara-dev-admin",
    },
  });

  expect(response.ok()).toBeTruthy();
  const payload = await response.json();

  await page.goto("/app");
  await page.evaluate((token: string) => {
    window.sessionStorage.setItem("tessara.devToken", token);
  }, payload.token);
}

test("home route SSR shell stays visible and bridge script is present", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app");

  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();
  await expect(page.locator("#global-search")).toBeVisible();
  await expect(page.getByText("Transitional Reporting")).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Data and component navigation" }).getByRole("link", { name: "Datasets" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Data and component navigation" }).getByRole("link", { name: "Components" })).toBeVisible();
  await expect(page.locator('script[src="/bridge/app-legacy.js"]')).toHaveCount(1);
  await expect(page.locator('link[href="/pkg/tessara-web.css"]')).toHaveCount(1);
  await expect(page.locator(".breadcrumb-item")).toHaveCount(0);

  const scripts = await pkgScripts(page);
  expect(scripts).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));

  await assertNoConsoleErrors();
});

test("theme preference persists on the login route", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app/login");
  await page.getByRole("button", { name: "Dark" }).click();

  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.reload();

  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", "dark");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await assertNoConsoleErrors();
});

test("migration route remains isolated and reachable", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app");
  const homeScripts = new Set(await pkgScripts(page));
  await signInAsAdmin(page);

  await page.goto("/app/migration");
  await page.waitForLoadState("networkidle");
  await expect(page.locator("h1").filter({ hasText: "Migration Workbench" })).toBeVisible();
  await expect(page.getByText("Migration Workspace")).toBeVisible();

  const migrationScripts = new Set(await pkgScripts(page));
  expect([...homeScripts]).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));
  expect([...migrationScripts]).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));
  await expect(page.locator('script[src="/bridge/app-legacy.js"]')).toHaveCount(1);

  await assertNoConsoleErrors();
});

test("forms list route stays readable and console-clean", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app/forms");

  await expect(page.getByRole("heading", { name: "Forms" }).first()).toBeVisible();
  await expect(page.locator("#form-list")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Lifecycle Summary");
  await expect(page.locator(".breadcrumb-item")).toHaveCount(0);

  await assertNoConsoleErrors();
});

test("deeper routes show breadcrumbs and internal cues stay subtle", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app/organization/00000000-0000-0000-0000-000000000001");
  await expect(page.getByRole("heading", { level: 1, name: "Organization Detail" })).toBeVisible();
  await expect(page.locator(".breadcrumb-item")).toHaveCount(3);

  await signInAsAdmin(page);
  await page.goto("/app/administration");
  await expect(page.getByRole("heading", { level: 1, name: "Administration" })).toBeVisible();
  await expect(page.getByText("Administration Workspace")).toBeVisible();

  await assertNoConsoleErrors();
});

test("shell navigation collapses on tablet and overlays on mobile", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.setViewportSize({ width: 900, height: 900 });
  await page.goto("/app");
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "tablet");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "collapsed");
  await expect(page.locator(".app-nav__label").first()).toBeHidden();

  await page.getByRole("button", { name: /expand navigation/i }).click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "expanded");
  await expect(page.locator(".app-nav__label").first()).toBeVisible();

  await page.setViewportSize({ width: 390, height: 844 });
  await page.reload();
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "mobile");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-closed");

  await page.getByRole("button", { name: /open navigation/i }).click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-open");

  await page.locator(".app-sidebar-backdrop").click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-closed");

  await assertNoConsoleErrors();
});

test("narrow-width routes avoid shell-level horizontal overflow", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/app");

  const overflow = await page.evaluate(() => {
    const root = document.documentElement;
    return root.scrollWidth - root.clientWidth;
  });

  expect(overflow).toBeLessThanOrEqual(1);
  await expect(page.locator("#global-search")).toBeVisible();

  await assertNoConsoleErrors();
});

test("dataset and component routes stay readable in the shared shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await signInAsAdmin(page);

  await page.goto("/app/datasets");
  await expect(page.getByRole("heading", { level: 1, name: "Datasets" })).toBeVisible();
  await expect(page.locator("#dataset-list")).toHaveCount(1);

  await page.goto("/app/components");
  await expect(page.getByRole("heading", { level: 1, name: "Components" })).toBeVisible();
  await expect(page.locator("#component-list")).toHaveCount(1);

  await assertNoConsoleErrors();
});

test("core SSR surfaces remain readable without JavaScript", async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();

  await page.goto(`${baseURL}/app`);
  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();
  await expect(page.locator("#global-search")).toBeVisible();

  await page.goto(`${baseURL}/app/login`);
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();

  await page.goto(`${baseURL}/app/dashboards`);
  await expect(page.getByRole("heading", { level: 1, name: "Dashboards" })).toBeVisible();

  await page.goto(`${baseURL}/app/organization`);
  await expect(page.getByRole("heading", { level: 1, name: "Organization" })).toBeVisible();

  await page.goto(`${baseURL}/app/forms`);
  await expect(page.getByRole("heading", { name: "Forms" }).first()).toBeVisible();
  await expect(page.locator("body")).toContainText("Form Directory");

  await page.goto(`${baseURL}/app/forms/new`);
  await expect(page.locator("body")).toContainText("Create Form");
  await expect(page.locator("#form-editor-status")).toHaveCount(1);

  await page.goto(`${baseURL}/app/forms/00000000-0000-0000-0000-000000000002`);
  await expect(page.locator("body")).toContainText("Form Detail");
  await expect(page.locator("body")).toContainText("Version Summary");

  await page.goto(`${baseURL}/app/forms/00000000-0000-0000-0000-000000000002/edit`);
  await expect(page.locator("body")).toContainText("Edit Form");
  await expect(page.locator("body")).toContainText("Draft Version Workspace");

  await page.goto(`${baseURL}/app/administration`);
  await expect(page.getByRole("heading", { level: 1, name: "Administration" })).toBeVisible();

  await page.goto(`${baseURL}/app/datasets`);
  await expect(page.getByRole("heading", { level: 1, name: "Datasets" })).toBeVisible();

  await page.goto(`${baseURL}/app/components`);
  await expect(page.getByRole("heading", { level: 1, name: "Components" })).toBeVisible();

  await page.goto(`${baseURL}/app/migration`);
  await expect(page.getByRole("heading", { level: 1, name: "Migration Workbench" })).toBeVisible();

  await context.close();
});
