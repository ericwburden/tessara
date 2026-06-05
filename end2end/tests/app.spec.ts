import { expect, test, type Page } from "@playwright/test";

const BENIGN_NAVIGATION_ABORT_ERRORS = [
  "WebAssembly compilation aborted: Network error: Response body loading was aborted",
  "Failed to load resource: the server responded with a status of 404 (Not Found)",
];

function isBenignNavigationAbort(message: string) {
  return BENIGN_NAVIGATION_ABORT_ERRORS.some((pattern) =>
    message.includes(pattern),
  );
}

function attachConsoleGuard(page: Page) {
  const errors: string[] = [];
  page.on("console", (message) => {
    if (message.type() === "error") {
      const text = message.text();
      if (!isBenignNavigationAbort(text)) {
        errors.push(text);
      }
    }
  });
  page.on("pageerror", (error) => {
    if (!isBenignNavigationAbort(error.message)) {
      errors.push(error.message);
    }
  });
  return async () => {
    expect(
      errors,
      `browser console should stay clean: ${errors.join("\n")}`,
    ).toEqual([]);
  };
}

async function signInAsAdmin(page: Page) {
  const response = await page.request.post("/api/auth/login", {
    data: {
      email: "admin@tessara.local",
      password: "tessara-dev-admin",
    },
  });
  expect(response.ok()).toBeTruthy();
}

test("root route is the native route inventory", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);

  await page.goto("/");
  await expect(page).toHaveURL(/\/$/);
  await expect(
    page.getByRole("heading", { level: 1, name: "Home" }),
  ).toBeVisible();
  await expect(
    page.getByText("Native UI Route Inventory"),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Administration" }),
  ).toBeVisible();
  await expect(page.locator('a[href="/forms"]').first()).toBeVisible();
  await expect(page.locator('a[href^="/app"]')).toHaveCount(0);
  await assertNoConsoleErrors();
});

test("login is a bare root-level route", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/login");
  await expect(
    page.getByRole("heading", { level: 1, name: "Welcome back" }),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();
  await expect(page.locator(".app-shell")).toHaveCount(0);
  await expect(page.locator(".sidebar")).toHaveCount(0);
  await assertNoConsoleErrors();
});

test("protected native routes redirect unauthenticated browsers to login", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/forms");
  await expect(page).toHaveURL(/\/login$/);
  await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();
  await assertNoConsoleErrors();
});

test("authenticated primary routes render in the native shell", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);

  await page.goto("/administration");
  await expect(
    page.getByRole("heading", { level: 1, name: "Administration" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Users" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Node Types" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", { name: "Open Roles" }),
  ).toBeVisible();
  await expect(page.locator(".sidebar")).toBeVisible();

  await page.goto("/datasets");
  await expect(
    page.getByRole("heading", { level: 1, name: "Datasets" }),
  ).toBeVisible();
  await expect(page.getByText("Dataset functionality will be filled in later.")).toBeVisible();
  await assertNoConsoleErrors();
});

test("old /app routes are no longer part of the mounted UI", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app");
  await expect(page).toHaveURL(/\/login$/);

  await signInAsAdmin(page);
  const response = await page.goto("/app");
  expect(response?.status()).toBe(404);

  await assertNoConsoleErrors();
});
