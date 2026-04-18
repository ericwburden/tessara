import { expect, test, type Page } from "@playwright/test";

const BENIGN_NAVIGATION_ABORT_ERRORS = [
  "WebAssembly compilation aborted: Network error: Response body loading was aborted",
];

type WorkflowAssignmentSummary = {
  workflow_version_id: string;
  node_id: string;
  account_id: string;
  account_email: string;
  is_active: boolean;
  has_draft: boolean;
  has_submitted: boolean;
};

function isBenignNavigationAbort(message: string) {
  return BENIGN_NAVIGATION_ABORT_ERRORS.some((pattern) => message.includes(pattern));
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
    expect(errors, `browser console should stay clean: ${errors.join("\n")}`).toEqual([]);
  };
}

async function expectNoLegacyBridge(page: Page) {
  await expect(page.locator('script[src="/bridge/app-legacy.js"]')).toHaveCount(0);
}

async function waitForAuthenticatedShell(page: Page, email?: string) {
  await expect(page.getByRole("button", { name: "Sign Out" })).toBeVisible();
  if (email) {
    await expect(page.locator("body")).toContainText(email);
  }
}

async function signOut(page: Page) {
  await page.getByRole("button", { name: "Sign Out" }).click();
  await expect(page.getByRole("button", { name: "Sign Out" })).toHaveCount(0);
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
  await signIn(page, "admin@tessara.local", "tessara-dev-admin");
}

async function signInAsRespondent(page: Page) {
  await signIn(page, "respondent@tessara.local", "tessara-dev-respondent");
}

async function signInAsDelegate(page: Page) {
  await signIn(page, "delegate@tessara.local", "tessara-dev-delegate");
}

async function signInAsDelegator(page: Page) {
  await signIn(page, "delegator@tessara.local", "tessara-dev-delegator");
}

async function requestAuthToken(page: Page, email: string, password: string) {
  const response = await page.request.post("/api/auth/login", {
    data: {
      email,
      password,
    },
  });

  expect(response.ok()).toBeTruthy();
  const payload = await response.json();
  return payload.token as string;
}

async function signIn(page: Page, email: string, password: string) {
  const token = await requestAuthToken(page, email, password);

  await page.addInitScript((token: string) => {
    window.sessionStorage.setItem("tessara.devToken", token);
  }, token);
  await page.goto("/app", { waitUntil: "domcontentloaded" });
}

async function provisionPendingAssignmentForAccount(page: Page, accountEmail: string) {
  const adminToken = await requestAuthToken(page, "admin@tessara.local", "tessara-dev-admin");
  const assignmentsResponse = await page.request.get("/api/workflow-assignments", {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
  });
  expect(assignmentsResponse.ok()).toBeTruthy();

  const assignments = (await assignmentsResponse.json()) as WorkflowAssignmentSummary[];
  const targetAssignment = assignments.find((assignment) => assignment.account_email === accountEmail);
  expect(targetAssignment).toBeTruthy();

  const targetAccountId = targetAssignment!.account_id;
  const usedPairs = new Set(
    assignments
      .filter((assignment) => assignment.account_id === targetAccountId)
      .map((assignment) => `${assignment.workflow_version_id}:${assignment.node_id}`),
  );
  const template = assignments.find(
    (assignment) =>
      assignment.is_active &&
      !assignment.has_draft &&
      !assignment.has_submitted &&
      assignment.account_id !== targetAccountId &&
      !usedPairs.has(`${assignment.workflow_version_id}:${assignment.node_id}`),
  );
  expect(template).toBeTruthy();

  const createResponse = await page.request.post("/api/workflow-assignments", {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
    data: {
      workflow_version_id: template!.workflow_version_id,
      node_id: template!.node_id,
      account_id: targetAccountId,
    },
  });
  expect(createResponse.ok()).toBeTruthy();
}

test("home route stays on the native SSR shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app");

  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();
  await expect(page.locator("#global-search")).toBeVisible();
  await expect(page.getByText("Transitional Reporting")).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Data and component navigation" }).getByRole("link", { name: "Datasets" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Data and component navigation" }).getByRole("link", { name: "Components" })).toBeVisible();
  await expectNoLegacyBridge(page);
  await expect(page.locator('link[href="/pkg/tessara-web.css"]')).toHaveCount(1);
  await expect(page.locator(".breadcrumb-item")).toHaveCount(0);

  await page.reload();
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();
  await expectNoLegacyBridge(page);

  const scripts = await pkgScripts(page);
  expect(scripts).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));

  await assertNoConsoleErrors();
});

test("theme preference persists on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app");
  await page.getByRole("button", { name: "Dark" }).click();

  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.reload();

  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", "dark");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await assertNoConsoleErrors();
});

test("migration route remains isolated and reachable", async ({ page }) => {
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
});

test("forms list route stays readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/forms");

  await expect(page.getByRole("heading", { name: "Forms" }).first()).toBeVisible();
  await expect(page.locator("#form-list")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Lifecycle Summary");
  await expect(page.locator(".breadcrumb-item")).toHaveCount(0);
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Forms" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await assertNoConsoleErrors();
});

test("forms authoring routes stay native and console-clean", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/forms/new");
  await expect(page.getByRole("heading", { name: "Create Form" }).first()).toBeVisible();
  await expect(page.locator("#form-editor-status")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Create Form" }).first()).toBeVisible();
  await expect(page.locator("#form-editor-status")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto("/app/forms");
  const editHref = await page
    .locator('a[href^="/app/forms/"][href$="/edit"]')
    .first()
    .getAttribute("href");
  expect(editHref).toBeTruthy();

  await page.goto(editHref!);
  await expect(page.getByRole("heading", { name: "Edit Form" }).first()).toBeVisible();
  await expect(page.locator("body")).toContainText("Draft Version Workspace");
  await expect(page.locator("#form-version-workspace")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Edit Form" }).first()).toBeVisible();
  await expect(page.locator("#form-version-workspace")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await assertNoConsoleErrors();
});

test("workflows route stays readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/workflows");

  await expect(page.getByRole("heading", { name: "Workflows" }).first()).toBeVisible();
  await expect(page.locator("#workflow-list")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Workflow Directory");
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Workflows" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await assertNoConsoleErrors();
});

test("responses route stays readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsRespondent(page);
  await waitForAuthenticatedShell(page, "respondent@tessara.local");

  await page.goto("/app/responses");

  await expect(page.getByRole("heading", { name: "Responses" }).first()).toBeVisible();
  await expect(page.locator("#response-pending-list")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Draft Responses");
  await expect(page.locator("body")).toContainText("Submitted Responses");
  await expect(page.getByRole("link", { name: "Start Response" })).toHaveCount(0);
  await expectNoLegacyBridge(page);

  await page.reload();
  await waitForAuthenticatedShell(page, "respondent@tessara.local");
  await expect(page.getByRole("heading", { name: "Responses" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await assertNoConsoleErrors();
});

test("post-login home entry persists across refresh and clears on logout", async ({ page }) => {
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await expect(page).toHaveURL(/\/app$/);
  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();

  await page.reload();
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await expect(page.getByRole("heading", { name: "Application Overview" })).toBeVisible();

  await signOut(page);
  await expect(page).toHaveURL(/\/app\/login$/);
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Submit" })).toBeVisible();

  await page.reload();
  await expect(page).toHaveURL(/\/app\/login$/);
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Sign Out" })).toHaveCount(0);
});

test("response users only see sidebar areas they can access", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsRespondent(page);
  await waitForAuthenticatedShell(page, "respondent@tessara.local");

  await expect(page).toHaveURL(/\/app$/);

  const productNav = page.getByRole("navigation", { name: "Product navigation" });
  await expect(productNav.getByRole("link", { name: "Home" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Responses" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Organization" })).toHaveCount(0);
  await expect(productNav.getByRole("link", { name: "Forms" })).toHaveCount(0);
  await expect(productNav.getByRole("link", { name: "Workflows" })).toHaveCount(0);
  await expect(productNav.getByRole("link", { name: "Dashboards" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Forms" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Workflows" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Organization" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Responses" })).toBeVisible();

  await assertNoConsoleErrors();
});

test("admins see the full authorized native navigation model", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await expect(page).toHaveURL(/\/app$/);

  const productNav = page.getByRole("navigation", { name: "Product navigation" });
  await expect(productNav.getByRole("link", { name: "Home" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Organization" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Forms" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Workflows" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Responses" })).toBeVisible();

  await expect(page.getByRole("link", { name: "Go to Forms" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Go to Workflows" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Go to Responses" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Go to Organization" })).toBeVisible();

  await assertNoConsoleErrors();
});

test("assignee pending start opens the matching draft directly and removes it from pending after submit", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await provisionPendingAssignmentForAccount(page, "delegate@tessara.local");
  await signInAsDelegate(page);
  await waitForAuthenticatedShell(page, "delegate@tessara.local");

  await page.goto("/app/responses");

  const pendingCards = page.locator("#response-pending-list .record-card");
  await expect.poll(async () => pendingCards.count()).toBeGreaterThan(0);

  const pendingCard = pendingCards.last();
  const expectedFormLabel = (await pendingCard.locator("p.muted").first().textContent())
    ?.replace(/^Form:\s*/, "")
    .trim();
  expect(expectedFormLabel).toBeTruthy();
  const assignmentId = await pendingCard
    .locator("button[data-workflow-assignment-id]")
    .getAttribute("data-workflow-assignment-id");
  expect(assignmentId).toBeTruthy();

  await pendingCard.getByRole("button", { name: "Start" }).click();

  await expect(page).toHaveURL(/\/app\/responses\/[^/]+\/edit$/);

  const submissionId = page.url().match(/\/app\/responses\/([^/]+)\/edit$/)?.[1];
  expect(submissionId).toBeTruthy();

  const token = await page.evaluate(() => window.sessionStorage.getItem("tessara.devToken"));
  expect(token).toBeTruthy();

  const submissionResponse = await page.request.get(`/api/submissions/${submissionId}`, {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  expect(submissionResponse.ok()).toBeTruthy();
  const submission = await submissionResponse.json();
  expect(`${submission.form_name} ${submission.version_label}`).toBe(expectedFormLabel);

  const inputs = page.locator("#response-edit-form input");
  const inputCount = await inputs.count();
  for (let index = 0; index < inputCount; index += 1) {
    const input = inputs.nth(index);
    const type = await input.getAttribute("type");
    if (type === "checkbox") {
      await input.check();
    } else if (type === "date") {
      await input.fill("2026-04-17");
    } else if (type === "number") {
      await input.fill("1");
    } else {
      await input.fill(`Auto value ${index + 1}`);
    }
  }

  await page.getByRole("button", { name: "Submit" }).click();
  await expect(page).toHaveURL(new RegExp(`/app/responses/${submissionId}(?:/edit)?$`));

  const pendingResponse = await page.request.get("/api/workflow-assignments/pending", {
    headers: {
      Authorization: `Bearer ${token}`,
    },
  });
  expect(pendingResponse.ok()).toBeTruthy();
  const pendingAssignments = await pendingResponse.json();
  expect(
    pendingAssignments.some(
      (item: { workflow_assignment_id: string }) => item.workflow_assignment_id === assignmentId,
    ),
  ).toBeFalsy();

  await page.goto("/app/responses");
  await expect(page.locator("#response-draft-list")).not.toContainText(submission.form_name);
  await expect(page.locator("#response-pending-list")).not.toContainText(submission.form_name);

  await assertNoConsoleErrors();
});

test("draft resume actions resolve to the same in-progress response item", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await provisionPendingAssignmentForAccount(page, "delegate@tessara.local");
  await signInAsDelegate(page);
  await waitForAuthenticatedShell(page, "delegate@tessara.local");

  await page.goto("/app/responses");

  const pendingCards = page.locator("#response-pending-list .record-card");
  await expect.poll(async () => pendingCards.count()).toBeGreaterThan(0);

  const pendingCard = pendingCards.last();
  const expectedFormLabel = (await pendingCard.locator("p.muted").first().textContent())
    ?.replace(/^Form:\s*/, "")
    .trim();
  expect(expectedFormLabel).toBeTruthy();

  await pendingCard.getByRole("button", { name: "Start" }).click();
  await expect(page).toHaveURL(/\/app\/responses\/[^/]+\/edit$/);

  const submissionId = page.url().match(/\/app\/responses\/([^/]+)\/edit$/)?.[1];
  expect(submissionId).toBeTruthy();

  await page.goto("/app/responses");
  const draftCard = page
    .locator("#response-draft-list .record-card")
    .filter({ has: page.locator(`a[href="/app/responses/${submissionId}/edit"]`) });
  await expect(draftCard).toHaveCount(1);

  await draftCard.getByRole("link", { name: "Edit" }).click();
  await expect(page).toHaveURL(new RegExp(`/app/responses/${submissionId}/edit$`));

  await page.goto("/app/responses");
  await draftCard.getByRole("link", { name: "View" }).click();
  await expect(page).toHaveURL(new RegExp(`/app/responses/${submissionId}$`));

  await assertNoConsoleErrors();
});

test("response users are redirected away from the manual start screen while admins keep it", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await signInAsRespondent(page);
  await page.goto("/app/responses/new");
  await expect(page).toHaveURL(/\/app\/responses$/);
  await expect(page.getByRole("heading", { name: "Responses" }).first()).toBeVisible();

  await signInAsAdmin(page);
  await page.goto("/app/responses/new");
  await expect(page.getByRole("heading", { name: "Start Response" }).first()).toBeVisible();
  await expect
    .poll(async () => page.locator("#response-form-version option").count())
    .toBeGreaterThan(1);
  await expect.poll(async () => page.locator("#response-node option").count()).toBeGreaterThan(1);

  await assertNoConsoleErrors();
});

test("workflow assignment deep links preserve workflow context in the assignment console", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/workflows", { waitUntil: "domcontentloaded" });

  const assignmentLink = page
    .locator('#workflow-list a[href^="/app/workflows/assignments?workflowId="]')
    .first();
  const href = await assignmentLink.getAttribute("href");
  expect(href).toBeTruthy();

  const workflowId = new URL(`http://localhost:8080${href}`).searchParams.get("workflowId");
  expect(workflowId).toBeTruthy();

  await assignmentLink.click();

  await expect(page).toHaveURL(new RegExp(`/app/workflows/assignments\\?workflowId=${workflowId}`));
  await expect(page.locator("#workflow-assignment-workflow")).toHaveValue(workflowId!);
  await expect(page.locator("#workflow-assignment-node")).not.toHaveValue("");

  await assertNoConsoleErrors();
});

test("left nav updates visible content across touched Sprint 2A routes", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/forms/new");
  await expect(page.getByRole("heading", { name: "Create Form" }).first()).toBeVisible();

  const productNav = page.getByRole("navigation", { name: "Product navigation" });

  await productNav.getByRole("link", { name: "Workflows" }).click();
  await expect(page).toHaveURL(/\/app\/workflows$/);
  await expect(page.getByRole("heading", { name: "Workflows" }).first()).toBeVisible();

  await productNav.getByRole("link", { name: "Responses" }).click();
  await expect(page).toHaveURL(/\/app\/responses$/);
  await expect(page.getByRole("heading", { name: "Responses" }).first()).toBeVisible();

  await productNav.getByRole("link", { name: "Forms" }).click();
  await expect(page).toHaveURL(/\/app\/forms$/);
  await expect(page.getByRole("heading", { name: "Forms" }).first()).toBeVisible();

  await assertNoConsoleErrors();
});

test("deeper routes show breadcrumbs and internal cues stay subtle", async ({ page }) => {
  await page.goto("/app/organization/00000000-0000-0000-0000-000000000001");
  await expect(page.getByRole("heading", { level: 1, name: "Organization Detail" })).toBeVisible();
  await expect(page.locator(".breadcrumb-item")).toHaveCount(3);
  await expect(page.locator("body")).not.toContainText("Administration Workspace");
  await expect(page.locator("body")).not.toContainText("Migration Workspace");

  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await page.goto("/app/administration");
  await expect(page.getByRole("heading", { level: 1, name: "Administration" })).toBeVisible();
  await expect(page.getByText("Administration Workspace")).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Internal navigation" }).getByRole("link", { name: "Migration" }),
  ).toBeVisible();

  await page.goto("/app/migration");
  await expect(page.getByRole("heading", { level: 1, name: "Migration Workbench" })).toBeVisible();
  await expect(page.getByText("Migration Workspace")).toBeVisible();
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
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto("/app");

  const overflow = await page.evaluate(() => {
    const root = document.documentElement;
    return root.scrollWidth - root.clientWidth;
  });

  expect(overflow).toBeLessThanOrEqual(1);
  await expect(page.locator("#global-search")).toBeVisible();

  await page.goto("/app/organization/00000000-0000-0000-0000-000000000001");
  await expect(page.locator(".breadcrumb-item")).toHaveCount(3);
  const deepRouteOverflow = await page.evaluate(() => {
    const root = document.documentElement;
    return root.scrollWidth - root.clientWidth;
  });
  expect(deepRouteOverflow).toBeLessThanOrEqual(1);
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
