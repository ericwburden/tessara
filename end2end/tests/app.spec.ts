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

test("migration route remains isolated and reachable as the first split candidate", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app");
  const homeScripts = new Set(await pkgScripts(page));

  await page.goto("/app/migration");
  await page.waitForLoadState("networkidle");
  await expect(page.getByRole("heading", { name: "Migration Workbench" })).toBeVisible();

  const migrationScripts = new Set(await pkgScripts(page));
  const newScripts = [...migrationScripts].filter((name) => !homeScripts.has(name));
  expect([...homeScripts]).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));
  expect([...migrationScripts]).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));
  expect(newScripts.length).toBeGreaterThan(0);
  await expect(page.locator('script[src="/bridge/app-legacy.js"]')).toHaveCount(1);

  await assertNoConsoleErrors();
});

test("core SSR surfaces remain readable without JavaScript", async ({ browser, baseURL }) => {
  const context = await browser.newContext({ javaScriptEnabled: false });
  const page = await context.newPage();

  await page.goto(`${baseURL}/app`);
  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();

  await page.goto(`${baseURL}/app/login`);
  await expect(page.getByRole("heading", { name: "Sign In" })).toBeVisible();

  await page.goto(`${baseURL}/app/dashboards`);
  await expect(page.getByRole("heading", { name: "Dashboards" })).toBeVisible();

  await context.close();
});
