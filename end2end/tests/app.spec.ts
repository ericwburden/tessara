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

test("home route SSR shell stays visible and bridge script is present", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app");

  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();
  await expect(page.locator('script[src="/bridge/app-legacy.js"]')).toHaveCount(1);
  await expect(page.locator('link[href="/pkg/tessara-web.css"]')).toHaveCount(1);

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

  await page.goto("/app/migration");
  await page.waitForLoadState("networkidle");
  await expect(page.getByRole("heading", { name: "Migration Workbench" })).toBeVisible();

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

  await assertNoConsoleErrors();
});

test("core SSR surfaces remain readable without JavaScript", async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();

  await page.goto(`${baseURL}/app`);
  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();

  await page.goto(`${baseURL}/app/login`);
    await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();

  await page.goto(`${baseURL}/app/dashboards`);
  await expect(page.getByRole("heading", { level: 1, name: "Dashboards" })).toBeVisible();

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

  await context.close();
});
