import { expect, test } from "@playwright/test";

async function signInAsAdmin(page: import("@playwright/test").Page) {
  const response = await page.request.post("/api/auth/login", {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  expect(response.ok()).toBeTruthy();
}

test.describe("Sprint 6F Application Composition", () => {
  test("administrator sees desired, approval, observed, drift, and emergency controls", async ({ page }) => {
    await signInAsAdmin(page);
    await page.goto("/administration/composition");
    await expect(page.getByRole("heading", { level: 1, name: "Application Composition" })).toBeVisible();
    await expect(page.getByText("Blueprint", { exact: true })).toBeVisible();
    await expect(page.getByText("Resolved plan", { exact: true })).toBeVisible();
    await expect(page.getByText("Observed receipt", { exact: true })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Drift" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Emergency module disable" })).toBeVisible();
  });

  test("composition read-back exposes lockfile, receipt, and explicit drift arrays", async ({ page }) => {
    await signInAsAdmin(page);
    const response = await page.request.get("/api/admin/composition");
    expect(response.ok()).toBeTruthy();
    const summary = await response.json();
    expect(summary.schema_version).toBe(1);
    expect(summary.latest_lockfile).toBeTruthy();
    expect(summary.latest_receipt).toBeTruthy();
    expect(Array.isArray(summary.drift_findings)).toBeTruthy();
    expect(Array.isArray(summary.emergency_overrides)).toBeTruthy();
  });

  test("unauthenticated users cannot inspect or mutate composition", async ({ request }) => {
    expect((await request.get("/api/admin/composition")).status()).toBe(401);
    expect((await request.post("/api/admin/composition/modules/tessara.dashboards/emergency-disable", {
      data: { reason: "must not be accepted", expires_in_minutes: 1 },
    })).status()).toBe(401);
  });
});
