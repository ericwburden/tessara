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

type SessionScopeNode = {
  node_id: string;
  node_name: string;
  node_type_name: string;
  parent_node_id: string | null;
};

type SessionDelegation = {
  account_id: string;
  display_name: string;
  email: string;
};

type SessionAccountContext = {
  account_id: string;
  display_name: string;
  email: string;
  scope_nodes: SessionScopeNode[];
  delegations: SessionDelegation[];
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

async function expectQueueFirstHome(page: Page) {
  await expect(page.getByRole("heading", { level: 1, name: "Current Work" })).toBeVisible();
  await expect(page.locator("#home-current-work")).toBeVisible();
  await expect(page.locator("#home-hierarchy-context")).toBeVisible();
  await expect(page.locator("#home-operational-snapshot")).toBeVisible();
  await expect(page.locator("#home-metric-strip")).toBeVisible();
  await expect(page.locator("body")).not.toContainText("Product Areas");
  await expect(page.locator("body")).not.toContainText("Internal Workspaces");
  await expect(page.locator("body")).not.toContainText("Transitional Reporting");
  await expect(page.getByRole("link", { name: "Go to Forms" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Workflows" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Responses" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Go to Organization" })).toHaveCount(0);
}

async function signOut(page: Page) {
  await page.getByRole("button", { name: "Sign Out" }).click();
  await expect(page).toHaveURL(/\/app\/login$/);
  await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Sign Out" })).toHaveCount(0);
}

async function fetchSessionAccount(page: Page) {
  const response = await page.request.get("/api/me");
  expect(response.ok()).toBeTruthy();
  return (await response.json()) as SessionAccountContext;
}

function preferredAccountLabel(displayName: string, email: string) {
  return displayName.trim() || email;
}

function scopeRootLabels(account: SessionAccountContext) {
  const labels = account.scope_nodes
    .filter(
      (node) =>
        !account.scope_nodes.some((candidate) => candidate.node_id === node.parent_node_id),
    )
    .map((node) => `${node.node_type_name}: ${node.node_name}`)
    .sort();
  return [...new Set(labels)];
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

  await expectQueueFirstHome(page);
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
  await expectQueueFirstHome(page);
  await expectNoLegacyBridge(page);

  const scripts = await pkgScripts(page);
  expect(scripts).toContainEqual(expect.stringContaining("/pkg/tessara-web.js"));

  await assertNoConsoleErrors();
});

test("theme preference persists on the native shell", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await page.locator("#shell-theme-context").getByRole("button", { name: "Dark" }).click();
  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", "dark");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");

  await page.reload();
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await expect(page.locator("html")).toHaveAttribute("data-theme-preference", "dark");
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await expect(page.locator("#shell-theme-context").getByRole("button", { name: "System" })).toBeVisible();
  await expect(page.locator("#shell-theme-context").getByRole("button", { name: "Light" })).toBeVisible();
  await expect(page.locator("#shell-theme-context").getByRole("button", { name: "Dark" })).toBeVisible();

  await assertNoConsoleErrors();
});

test("sign-in route stays bare and redirects authenticated browsers home", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await page.goto("/app/login");
  await expect(page.getByRole("heading", { level: 1, name: "Sign In" })).toBeVisible();
  await expect(page.getByRole("button", { name: "Sign In" })).toBeVisible();
  await expect(page.locator("[data-auth-surface]")).toHaveCount(1);
  await expect(page.locator(".top-app-bar")).toHaveCount(0);
  await expect(page.locator("#app-sidebar")).toHaveCount(0);
  await expect(page.locator("#global-search")).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Sign Out" })).toHaveCount(0);

  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await page.goto("/app/login");
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await expect(page).toHaveURL(/\/app$/);
  await expectQueueFirstHome(page);
  await expect(page.locator("#shell-account-context")).toContainText("admin@tessara.local");

  await assertNoConsoleErrors();
});

test("sidebar footer carries account and scope context on authenticated routes", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsOperator(page);
  await waitForAuthenticatedShell(page, "operator@tessara.local");

  const account = await fetchSessionAccount(page);
  const roots = scopeRootLabels(account);
  expect(roots.length).toBeGreaterThan(0);

  await page.goto("/app");
  await expect(page.locator("#shell-account-context")).toContainText(
    preferredAccountLabel(account.display_name, account.email),
  );
  await expect(page.locator("#shell-account-context")).toContainText(account.email);
  await expect(page.locator("#shell-scope-context")).toBeVisible();
  await expect(page.locator("#shell-scope-roots")).toContainText(roots[0]!);
  await expect(page.locator("#shell-theme-context")).toBeVisible();

  await assertNoConsoleErrors();
});

test("sidebar footer surfaces the active delegated account when delegation is in use", async ({
  page,
}) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsDelegator(page);
  await waitForAuthenticatedShell(page, "delegator@tessara.local");

  const account = await fetchSessionAccount(page);
  expect(account.delegations.length).toBeGreaterThan(0);
  const delegate = account.delegations[0]!;

  await page.goto(`/app/responses?delegateAccountId=${delegate.account_id}`);
  await expect(page.locator("#shell-delegation-context")).toContainText("Acting for");
  await expect(page.locator("#shell-delegation-context [data-active-delegate]")).toContainText(
    preferredAccountLabel(delegate.display_name, delegate.email),
  );
  await expect(page.locator("#shell-delegation-context [data-active-delegate]")).toContainText(
    delegate.email,
  );
  await expect(page.locator("body")).not.toContainText("Delegated Response Context");
  await expect(page.locator("#response-pending-list")).not.toContainText("Acting for:");

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
  await expect(page.locator("#organization-page-title")).toHaveText(/Explorer$/);
  await expect(page.locator("#organization-directory-tree")).toHaveCount(1);
  await expect
    .poll(async () => page.locator('#organization-directory-tree button[data-select-node-id]').count())
    .toBeGreaterThan(0);
  await expect(page.locator("article.organization-disclosure-card")).toHaveCount(0);
  await expect(page.locator("#organization-selection-preview")).toContainText("Related Work");
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.locator("#organization-page-title")).toHaveText(/Explorer$/);
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
  await expect(page.locator("#organization-page-title")).toHaveText("Activity Explorer");
  await expect(page.locator("#organization-list-title")).toHaveText("Visible Hierarchy");

  const tree = page.locator("#organization-directory-tree");
  await expect(tree).toContainText("Demo Activity Job Coaching");
  await expect(tree).toContainText("Demo Program Family Outreach");
  await expect(tree).not.toContainText("Demo Partner Community Bridge");
  await expect(page.locator("article.organization-disclosure-card")).toHaveCount(0);

  const firstRow = tree.locator("button[data-select-node-id]").first();
  const selectedName = (await firstRow.locator(".organization-explorer-row__name").textContent())?.trim();
  expect(selectedName).toBeTruthy();

  await firstRow.click();
  await expect(page.locator("#organization-selection-preview")).toContainText(selectedName!);
  await expect(page.locator("#organization-selection-preview")).toContainText("Related Work");

  await assertNoConsoleErrors();
});

test("organization explorer stacks the selected detail below the tree on mobile", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsOperator(page);
  await waitForAuthenticatedShell(page, "operator@tessara.local");
  await page.setViewportSize({ width: 390, height: 844 });

  await page.goto("/app/organization");
  await expect(page.locator("#organization-page-title")).toHaveText("Activity Explorer");

  const firstRow = page.locator('#organization-directory-tree button[data-select-node-id]').first();
  await firstRow.click();
  await expect(page.locator("#organization-selection-preview")).toContainText("Related Work");

  const treeBox = await page.locator("#organization-directory-tree").boundingBox();
  const previewBox = await page.locator("#organization-selection-preview").boundingBox();
  expect(treeBox).toBeTruthy();
  expect(previewBox).toBeTruthy();
  expect(previewBox!.y).toBeGreaterThan(treeBox!.y);

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
  await page.getByRole("button", { name: "Create Draft Version" }).click();
  await expect(page.locator(".form-builder-shell")).toHaveCount(1);
  await expect(page.locator(".form-builder-rail")).toContainText("Jump Between Sections");
  await expect(page.locator(".form-builder-insert-rail")).toContainText("Quick Actions");
  await expect(page.locator(".form-builder-canvas")).toBeVisible();

  await page.locator('.form-builder-insert-rail [data-form-section-create="quick"]').click();
  await expect(page.locator(".form-builder-section").last()).toBeVisible();
  const sectionLinkCount = await page.locator(".form-builder-section-link").count();
  expect(sectionLinkCount).toBeGreaterThan(0);

  const railBox = await page.locator(".form-builder-rail").boundingBox();
  const canvasBox = await page.locator(".form-builder-canvas").boundingBox();
  expect(railBox).toBeTruthy();
  expect(canvasBox).toBeTruthy();
  expect(railBox!.x).toBeLessThan(canvasBox!.x);

  await page.getByRole("button", { name: "Add Field To Selected Section" }).click();
  await expect(page.locator("#form-builder-properties")).toBeVisible();
  await expect(page.locator("#form-builder-properties")).toContainText("Field Properties");
  await expect(page.locator(".form-builder-field-card.is-selected")).toContainText("Untitled Field");

  const insertBox = await page.locator(".form-builder-insert-rail").boundingBox();
  const propertiesBox = await page.locator("#form-builder-properties").boundingBox();
  expect(insertBox).toBeTruthy();
  expect(propertiesBox).toBeTruthy();
  expect(insertBox!.x).toBeLessThan(canvasBox!.x);
  expect(propertiesBox!.x).toBeGreaterThan(railBox!.x);
  expect(propertiesBox!.x).toBeGreaterThan(insertBox!.x);
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

  await expect(page.getByRole("heading", { name: "Workflow Directory" }).first()).toBeVisible();
  await expect(page.locator("#workflow-list")).toHaveCount(1);
  await expect(page.locator("body")).toContainText("Workflow Directory");
  await expect(page.getByRole("link", { name: "Open Assignment Management" })).toBeVisible();
  await expectNoLegacyBridge(page);

  await page.reload();
  await expect(page.getByRole("heading", { name: "Workflow Directory" }).first()).toBeVisible();
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
  await expect(page.locator("body")).toContainText("Assigned Starts");
  await expect(page.locator("body")).toContainText("Draft Queue");
  await expect(page.locator("body")).toContainText("Submitted Responses");
  await expect(page.getByRole("link", { name: "Manual Start" })).toHaveCount(0);
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
  await expectQueueFirstHome(page);

  await page.reload();
  await waitForAuthenticatedShell(page, "admin@tessara.local");
  await expectQueueFirstHome(page);

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
  await expectQueueFirstHome(page);

  const productNav = page.getByRole("navigation", { name: "Primary navigation" });
  await expect(productNav.getByRole("link", { name: "Home" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Responses" })).toBeVisible();
  await expect(productNav.getByRole("link", { name: "Organization" })).toHaveCount(0);
  await expect(productNav.getByRole("link", { name: "Forms" })).toHaveCount(0);
  await expect(productNav.getByRole("link", { name: "Workflows" })).toHaveCount(0);
  await expect(productNav.getByRole("link", { name: "Dashboards" })).toHaveCount(0);
  await expect(page.getByRole("link", { name: "Open Responses" })).toBeVisible();

  await assertNoConsoleErrors();
});

test("admins see the full authorized native navigation model", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  await expect(page).toHaveURL(/\/app$/);
  await expectQueueFirstHome(page);

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

  await expect(page.getByRole("link", { name: "Open Responses" })).toBeVisible();
  await expect(page.getByRole("link", { name: "Open Organization" })).toBeVisible();

  await assertNoConsoleErrors();
});

test("unauthorized deep links redirect home with transient shell feedback", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);
  await signInAsRespondent(page);
  await waitForAuthenticatedShell(page, "respondent@tessara.local");

  await page.goto("/app/forms");

  await expect(page).toHaveURL(/\/app$/);
  await expectQueueFirstHome(page);
  await expect(page.locator("[data-shell-toast]")).toContainText("You do not have access to that screen.");
  await expect(page.locator("[data-shell-toast]")).toContainText("Tessara returned you to Home.");
  await expect(page.locator("#app-sidebar").getByRole("link", { name: "Forms" })).toHaveCount(0);
  await expect(page.locator("[data-shell-toast]")).toHaveCount(1);
  await expect(page).not.toHaveURL(/notice=access-denied/);
  await expect(page.locator("[data-shell-toast]")).toHaveCount(0, { timeout: 7000 });

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
  const assignmentId = await pendingCard
    .locator("button[data-workflow-assignment-id]")
    .getAttribute("data-workflow-assignment-id");
  expect(assignmentId).toBeTruthy();
  const pendingBeforeStartResponse = await page.request.get("/api/workflow-assignments/pending");
  expect(pendingBeforeStartResponse.ok()).toBeTruthy();
  const pendingBeforeStart = (await pendingBeforeStartResponse.json()) as Array<{
    workflow_assignment_id: string;
    form_name: string;
    form_version_label: string | null;
  }>;
  const startedAssignment = pendingBeforeStart.find(
    (item) => item.workflow_assignment_id === assignmentId,
  );
  expect(startedAssignment).toBeTruthy();

  await pendingCard.getByRole("button", { name: "Start" }).click();

  await expect(page).toHaveURL(/\/app\/responses\/[^/]+\/edit$/);

  const submissionId = page.url().match(/\/app\/responses\/([^/]+)\/edit$/)?.[1];
  expect(submissionId).toBeTruthy();

  const submissionResponse = await page.request.get(`/api/submissions/${submissionId}`);
  expect(submissionResponse.ok()).toBeTruthy();
  const submission = await submissionResponse.json();
  expect(submission.form_name).toBe(startedAssignment?.form_name);
  expect(submission.version_label).toBe(startedAssignment?.form_version_label);

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

  const pendingAfterSubmitResponse = await page.request.get("/api/workflow-assignments/pending");
  expect(pendingAfterSubmitResponse.ok()).toBeTruthy();
  const pendingAssignments = await pendingAfterSubmitResponse.json();
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
  await expect(page.getByRole("heading", { name: "Workflow Directory" }).first()).toBeVisible();
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
  await expect(page.getByRole("heading", { name: "Assignment Management" }).first()).toBeVisible();
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
  await expect(page.getByRole("heading", { name: "Workflow Directory" }).first()).toBeVisible();

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
    page.getByRole("navigation", { name: "Admin navigation" }).getByRole("link", { name: "Migration" }),
  ).toBeVisible();

  await page.goto("/app/migration");
  await expect(page.getByRole("heading", { name: "Migration Workbench" }).first()).toBeVisible();
  await expect(page.getByText("Operator import flow")).toBeVisible();
});

test("shell navigation collapses on tablet and overlays on mobile", async ({ page }) => {
  const assertNoConsoleErrors = attachConsoleGuard(page);

  await signInAsAdmin(page);
  await waitForAuthenticatedShell(page, "admin@tessara.local");

  const navToggle = page.locator("#app-nav-toggle");
  const sidebarClose = page.locator(".app-sidebar-close");
  const sidebarBackdrop = page.locator(".app-sidebar-backdrop");

  await page.setViewportSize({ width: 1280, height: 900 });
  await page.goto("/app");
  await expect(page.locator("html")).toHaveAttribute("data-shell-ready", "true");
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "desktop");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "expanded");
  await expect(navToggle).toHaveAttribute("hidden", "");
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

  await page.setViewportSize({ width: 900, height: 900 });
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "tablet");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "collapsed");
  await expect(navToggle).not.toHaveAttribute("hidden", "");
  await expect(navToggle).toHaveAttribute("aria-label", /expand navigation/i);
  await expect(page.locator(".app-nav__label").first()).toBeHidden();
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

  await navToggle.click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "expanded");
  await expect(navToggle).toHaveAttribute("aria-label", /collapse navigation/i);
  await expect(page.locator(".app-nav__label").first()).toBeVisible();
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "mobile");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-closed");
  await expect(navToggle).not.toHaveAttribute("hidden", "");
  await expect(navToggle).toHaveAttribute("aria-label", /open navigation/i);
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

  await navToggle.click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-open");
  await expect(navToggle).toHaveAttribute("aria-label", /close navigation/i);
  await expect(sidebarClose).not.toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).not.toHaveAttribute("hidden", "");

  await sidebarBackdrop.click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-closed");
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

  await navToggle.click();
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-open");
  await expect(sidebarClose).not.toHaveAttribute("hidden", "");

  await page.setViewportSize({ width: 1280, height: 900 });
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "desktop");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "expanded");
  await expect(navToggle).toHaveAttribute("hidden", "");
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.locator("body")).toHaveAttribute("data-shell-viewport", "mobile");
  await expect(page.locator("body")).toHaveAttribute("data-sidebar-state", "overlay-closed");
  await expect(navToggle).toHaveAttribute("aria-label", /open navigation/i);
  await expect(sidebarClose).toHaveAttribute("hidden", "");
  await expect(sidebarBackdrop).toHaveAttribute("hidden", "");

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
