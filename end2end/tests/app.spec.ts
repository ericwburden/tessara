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
  await expect(page).toHaveURL(/\/app\/login$/);
  await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();
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

async function signInAsOperator(page: Page) {
  await signIn(page, "operator@tessara.local", "tessara-dev-operator");
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
  await page.context().clearCookies();
  return payload.token as string;
}

async function signIn(page: Page, email: string, password: string) {
  const response = await page.request.post("/api/auth/login", {
    data: {
      email,
      password,
    },
  });
  expect(response.ok()).toBeTruthy();
  await page.goto("/app", { waitUntil: "domcontentloaded" });
  await expect(page).toHaveURL(/\/app$/);
}

async function provisionPendingAssignmentForAccount(page: Page, accountEmail: string) {
  const adminToken = await requestAuthToken(page, "admin@tessara.local", "tessara-dev-admin");
  const seedResponse = await page.request.post("/api/demo/seed", {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
  });
  expect(seedResponse.ok()).toBeTruthy();

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

  const nodesResponse = await page.request.get("/api/nodes", {
    headers: {
      Authorization: `Bearer ${adminToken}`,
    },
  });
  expect(nodesResponse.ok()).toBeTruthy();
  const nodes = (await nodesResponse.json()) as Array<{ id: string }>;

  const workflowVersionIds = [...new Set(assignments.map((assignment) => assignment.workflow_version_id))];
  const template = workflowVersionIds
    .flatMap((workflowVersionId) =>
      nodes.map((node) => ({
        workflow_version_id: workflowVersionId,
        node_id: node.id,
      })),
    )
    .find((candidate) => !usedPairs.has(`${candidate.workflow_version_id}:${candidate.node_id}`));
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
  await expect(page.getByRole("navigation", { name: "Primary navigation" }).getByRole("link", { name: "Components" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Primary navigation" }).getByRole("link", { name: "Dashboards" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Admin navigation" }).getByRole("link", { name: "Datasets" })).toBeVisible();
  await expect(page.locator("#app-sidebar").getByRole("link", { name: "Reports" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Notifications" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Help" })).toBeVisible();
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

  await page.goto("/app/login");
  await page.getByRole("button", { name: "Dark" }).click();

  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.reload();

  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", "dark");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await assertNoConsoleErrors();
});

test("migration route stays readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/migration");
  await expect(page.getByRole("heading", { name: "Migration Workbench" }).first()).toBeVisible();
  await expect(page.locator("#migration-list")).toHaveCount(1);
  await expect(page.locator("#migration-fixture-json")).toHaveCount(1);
  await expect(page.locator("#migration-results")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Operator import flow");
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Migration Workbench" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  const scripts = await pkgScripts(page);
  expect(scripts).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));

  await assertNoConsoleErrors();
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

test("organization routes stay readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  const nodesResponse = await page.request.get("/api/nodes");
  expect(nodesResponse.ok()).toBeTruthy();
  const nodes = (await nodesResponse.json()) as Array<{ id: string; name: string }>;
  expect(nodes.length).toBeGreaterThan(0);
  const nodeId = nodes[0]!.id;

  await page.goto("/app/organization");
  await expect(page.getByRole("heading", { name: "Organization" }).first()).toBeVisible();
  await expect(page.locator("#organization-directory-tree")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Hierarchy Navigator");
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Organization" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await page.goto("/app/organization/new");
  await expect(page.getByRole("heading", { name: "Create Organization" }).first()).toBeVisible();
  await expect(page.locator("#organization-form-status")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Create Organization" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await page.goto(`/app/organization/${nodeId}`);
  await expect(page.getByRole("heading", { name: "Organization Detail" }).first()).toBeVisible();
  await expect(page.locator("#organization-detail-path")).toHaveCount(1);
  await expect(page.locator("#organization-child-actions")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Organization Detail" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await page.goto(`/app/organization/${nodeId}/edit`);
  await expect(page.getByRole("heading", { name: "Edit Organization" }).first()).toBeVisible();
  await expect(page.locator("#organization-form-status")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Edit Organization" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await assertNoConsoleErrors();
});

test("organization navigator uses scope-aware labels and selection sync for scoped operators", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsOperator(page);
  await waitForAuthenticatedShell(page, "operator@tessara.local");

  await page.goto("/app/organization");
  await expect(page.getByRole("heading", { name: "Organization" }).first()).toBeVisible();
  await expect(page.locator("#organization-list-title")).toHaveText("Activity List");

  const tree = page.locator("#organization-directory-tree");
  await expect(tree).toContainText("Demo Activity Job Coaching");
  await expect(tree).toContainText("Demo Program Family Outreach");
  await expect(tree).not.toContainText("Demo Partner Community Bridge");

  const firstCard = tree.locator("article.organization-disclosure-card").first();
  const selectedName = (await firstCard.locator("h4").first().textContent())?.trim();
  expect(selectedName).toBeTruthy();

  await firstCard.locator("button[data-select-node-id]").first().click();
  await expect(page.locator("#organization-selection-preview")).toContainText(selectedName!);

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
  const detailHref = await page
    .locator('a[href^="/app/forms/"]:not([href$="/edit"])')
    .filter({ hasText: "View" })
    .first()
    .getAttribute("href");
  expect(detailHref).toBeTruthy();

  await page.goto(detailHref!);
  await expect(page.getByRole("heading", { name: "Form Detail" }).first()).toBeVisible();
  await expect(page.locator("#form-detail")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  const editHref = `${detailHref!}/edit`;
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

test("settled protected routes redirect unauthenticated browsers to sign in", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app/forms");
  await expect(page).toHaveURL(/\/app\/login$/);
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();

  await page.goto("/app/organization");
  await expect(page).toHaveURL(/\/app\/login$/);
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();

  await assertNoConsoleErrors();
});

test("dashboards routes stay readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/dashboards");
  await expect(page.getByRole("heading", { name: "Dashboards" }).first()).toBeVisible();
  await expect(page.locator("#dashboard-list")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Dashboard Directory");
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Dashboards" }).first()).toBeVisible();
  await expectNoLegacyBridge(page);

  await page.goto("/app/dashboards/new");
  await expect(page.getByRole("heading", { name: "Create Dashboard" }).first()).toBeVisible();
  await expect(page.locator("#dashboard-form")).toHaveCount(1);
  await expect(page.locator("#dashboard-form-status")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto("/app/dashboards");
  const dashboardDetailHref = await page.locator('a[href^="/app/dashboards/"]').filter({ hasText: "View" }).first().getAttribute("href");
  expect(dashboardDetailHref).toBeTruthy();

  await page.goto(dashboardDetailHref!);
  await expect(page.getByRole("heading", { name: "Dashboard Detail" }).first()).toBeVisible();
  await expect(page.locator("#dashboard-detail")).toHaveCount(1);
  await expect(page.locator("#dashboard-component-summary")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto(`${dashboardDetailHref!}/edit`);
  await expect(page.getByRole("heading", { name: "Edit Dashboard" }).first()).toBeVisible();
  await expect(page.locator("#dashboard-form")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await assertNoConsoleErrors();
});

test("administration routes stay readable and console-clean on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.goto("/app/administration");
  await expect(page.getByRole("heading", { name: "Administration" }).first()).toBeVisible();
  await expect(page.locator("body")).toContainText("Administration Workspace");
  await expectNoLegacyBridge(page);

  await page.goto("/app/administration/users");
  await expect(page.getByRole("heading", { name: "User Management" }).first()).toBeVisible();
  await expect(page.locator("#admin-user-list")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto("/app/administration/users/new");
  await expect(page.getByRole("heading", { name: "Create User" }).first()).toBeVisible();
  await expect(page.locator("#user-form")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto("/app/administration/users");
  const userDetailHref = await page.locator('a[href^="/app/administration/users/"]').filter({ hasText: "View" }).first().getAttribute("href");
  expect(userDetailHref).toBeTruthy();
  await page.goto(userDetailHref!);
  await expect(page.getByRole("heading", { name: "User Detail" }).first()).toBeVisible();
  await expect(page.locator("#user-detail-summary")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto(`${userDetailHref!}/edit`);
  await expect(page.getByRole("heading", { name: "Edit User" }).first()).toBeVisible();
  await expect(page.locator("#user-form")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto(`${userDetailHref!}/access`);
  await expect(page.getByRole("heading", { name: "User Access" }).first()).toBeVisible();
  await expect(page.locator("#user-access-form")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto("/app/administration/roles");
  await expect(page.getByRole("heading", { name: "Roles" }).first()).toBeVisible();
  await expect(page.locator("#admin-role-list")).toHaveCount(1);
  await expectNoLegacyBridge(page);

  await page.goto("/app/administration/node-types");
  await expect(page.getByRole("heading", { name: "Organization Node Types" }).first()).toBeVisible();
  await expect(page.locator("#admin-node-type-list")).toHaveCount(1);
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
  await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();

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

  const productNav = page.getByRole("navigation", { name: "Primary navigation" });
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

  const productNav = page.getByRole("navigation", { name: "Primary navigation" });
  await expect(productNav.getByRole("link", { name: "Home" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Organization" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Forms" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Workflows" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Responses" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Components" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Dashboards" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Admin navigation" }).getByRole("link", { name: "Datasets" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Admin navigation" }).getByRole("link", { name: "Administration" })).toBeVisible();
  await expect(page.getByRole("navigation", { name: "Admin navigation" }).getByRole("link", { name: "Migration" })).toBeVisible();
  await expect(page.locator("#app-sidebar").getByRole("link", { name: "Reports" })).toHaveCount(0);

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
  await provisionPendingAssignmentForAccount(page, "respondent@tessara.local");
  await signInAsRespondent(page);
  await waitForAuthenticatedShell(page, "respondent@tessara.local");

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

  const submissionResponse = await page.request.get(`/api/submissions/${submissionId}`);
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

  const pendingResponse = await page.request.get("/api/workflow-assignments/pending");
  expect(pendingResponse.ok()).toBeTruthy();
  const pendingAssignments = await pendingResponse.json();
  expect(
    pendingAssignments.some(
      (item: { workflow_assignment_id: string }) => item.workflow_assignment_id === assignmentId,
    ),
  ).toBeFalsy();

  await page.goto("/app/responses");
  await expect(page.locator(`#response-draft-list a[href="/app/responses/${submissionId}/edit"]`)).toHaveCount(0);
  await expect(
    page.locator(`#response-pending-list button[data-workflow-assignment-id="${assignmentId}"]`),
  ).toHaveCount(0);

  await assertNoConsoleErrors();
});

test("draft resume actions resolve to the same in-progress response item", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await provisionPendingAssignmentForAccount(page, "respondent@tessara.local");
  await signInAsRespondent(page);
  await waitForAuthenticatedShell(page, "respondent@tessara.local");

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

  await signOut(page);
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
  await expect(page.getByRole("heading", { name: "Workflows" }).first()).toBeVisible();
  await expect(page.locator("#workflow-list")).not.toContainText("Loading workflow records...", {
    timeout: 30000,
  });
  await expect(
    page.locator('#workflow-list a[href^="/app/workflows/assignments?workflowId="]'),
  ).not.toHaveCount(0);

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

  const productNav = page.getByRole("navigation", { name: "Primary navigation" });

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
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await page.goto("/app/organization/00000000-0000-0000-0000-000000000001");
  await expect(page.getByRole("heading", { name: "Organization Detail" }).first()).toBeVisible();
  await expect(page.locator(".breadcrumb-item")).toHaveCount(3);
  await expect(page.locator("body")).not.toContainText("Administration Workspace");
  await expect(page.locator("body")).not.toContainText("Migration Workspace");

  await page.goto("/app/administration");
  await expect(page.getByRole("heading", { name: "Administration" }).first()).toBeVisible();
  await expect(page.getByText("Administration Workspace")).toBeVisible();
  await expect(
    page.getByRole("navigation", { name: "Internal navigation" }).getByRole("link", { name: "Migration" }),
  ).toBeVisible();

  await page.goto("/app/migration");
  await expect(page.getByRole("heading", { name: "Migration Workbench" }).first()).toBeVisible();
  await expect(page.getByText("Operator import flow")).toBeVisible();
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
  await signInAsAdmin(page);
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
  await expect(page.locator("body")).toContainText("Loading Session");
  await expect(page.locator("#global-search")).toBeVisible();

  await page.goto(`${baseURL}/app/login`);
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();

  await page.goto(`${baseURL}/app/dashboards`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/organization`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/forms`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/forms/new`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/forms/00000000-0000-0000-0000-000000000002`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/forms/00000000-0000-0000-0000-000000000002/edit`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/administration`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await page.goto(`${baseURL}/app/datasets`);
  await expect(page.getByRole("heading", { level: 1, name: "Datasets" })).toBeVisible();

  await page.goto(`${baseURL}/app/components`);
  await expect(page.getByRole("heading", { level: 1, name: "Components" })).toBeVisible();

  await page.goto(`${baseURL}/app/migration`);
  await expect(page.locator("body")).toContainText("Loading Session");

  await context.close();
});
