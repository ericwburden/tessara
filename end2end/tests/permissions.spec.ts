import { expect, request, test, type APIRequestContext, type APIResponse, type Page } from "@playwright/test";

const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const RUN_ID = `pw-permissions-${Date.now()}`;
const PASSWORD = "tessara-dev-permissions";

type IdResponse = { id: string };
type CapabilitySummary = { id: string; key: string };
type RoleSummary = { id: string; name: string };
type UserSummary = { id: string; email: string };
type NodeSummary = {
  id: string;
  name: string;
  node_type_id: string;
  node_type_name: string;
  parent_node_id: string | null;
};
type NodeTypeSummary = {
  id: string;
  name: string;
  singular_label: string;
  is_root_type: boolean;
  child_relationships: Array<{ node_type_id: string; singular_label: string }>;
};
type VisibilityNode = { node_id: string; node_name: string };
type FormSummary = { id: string; name: string; slug: string; visibility_nodes: VisibilityNode[] };
type WorkflowSummary = { id: string; name: string; slug: string; available_nodes: Array<{ id: string; name: string }> };
type WorkflowDefinition = WorkflowSummary & { versions: Array<{ id: string; status: string; steps: Array<{ form_version_id: string }> }> };
type DatasetSummary = {
  id: string;
  name: string;
  visibility_nodes: VisibilityNode[];
  current_revision_id: string | null;
};
type ComponentSummary = { id: string; name: string; slug: string };
type ComponentDefinition = { id: string; name: string; versions: unknown[] };
type DashboardSummary = { id: string; name: string; visibility_nodes: VisibilityNode[] };
type DashboardDefinition = DashboardSummary & { description: string | null };
type WorkflowAssignmentCandidate = {
  workflow_version_id: string;
  workflow_id: string;
  workflow_name: string;
  node_id: string;
  node_name: string;
};
type WorkflowAssigneeOption = { account_id: string; email: string };
type WorkflowAssignmentSummary = {
  id: string;
  workflow_id: string;
  workflow_version_id: string;
  node_id: string;
  account_id: string;
  account_email: string;
  has_draft: boolean;
  has_submitted: boolean;
};
type PendingWorkflowWork = { workflow_assignment_id: string; account_id: string };
type SubmissionSummary = {
  id: string;
  node_id: string;
};
type SubmissionDetail = {
  id: string;
  node_id: string;
  status: string;
  form_name?: string;
};
type SessionAccount = {
  account_id: string;
  email: string;
  capabilities: string[];
  scope_nodes: Array<{ node_id: string; node_name: string }>;
  delegations: Array<{ account_id: string; email: string }>;
};
type SessionState = { authenticated: boolean; account: SessionAccount | null };

type FixtureState = {
  admin: APIRequestContext;
  scopedManager: APIRequestContext;
  owner: APIRequestContext;
  outOfScopeOwner: APIRequestContext;
  delegate: APIRequestContext;
  delegator: APIRequestContext;
  noAccess: APIRequestContext;
  userIds: Record<string, string>;
  inScopeNodeId: string;
  outOfScopeNodeId: string;
  inScopeNodeIds: Set<string>;
  inScopeForm: FormSummary;
  outOfScopeForm: FormSummary;
  inScopeDataset: DatasetSummary;
  outOfScopeDataset: DatasetSummary;
  inScopeComponent: ComponentSummary;
  outOfScopeComponent: ComponentSummary;
  inScopeDashboard: DashboardSummary;
  outOfScopeDashboard: DashboardSummary;
  inScopeAssignmentId: string;
  outOfScopeAssignmentId: string;
  ownerAssignmentId: string;
  outOfScopeOwnerAssignmentId: string;
  delegateAssignmentId: string;
};

let fixtures: FixtureState;
const contexts: APIRequestContext[] = [];

async function newContext() {
  const context = await request.newContext({ baseURL: BASE_URL });
  contexts.push(context);
  return context;
}

async function expectJson<T>(response: APIResponse): Promise<T> {
  const text = await response.text();
  expect(response.ok(), `${response.url()} returned ${response.status()}: ${text}`).toBeTruthy();
  return JSON.parse(text) as T;
}

async function getJson<T>(context: APIRequestContext, url: string) {
  return expectJson<T>(await context.get(url));
}

async function postJson<T>(context: APIRequestContext, url: string, data?: Record<string, unknown>) {
  return expectJson<T>(await context.post(url, { data }));
}

async function putJson<T>(context: APIRequestContext, url: string, data?: Record<string, unknown>) {
  return expectJson<T>(await context.put(url, { data }));
}

async function expectStatus(
  context: APIRequestContext,
  method: "get" | "post" | "put" | "delete",
  url: string,
  statuses: number[],
  data?: Record<string, unknown>,
) {
  const response = await context[method](url, data ? { data } : undefined);
  expect(statuses, `${method.toUpperCase()} ${url} returned ${response.status()}: ${await response.text()}`).toContain(
    response.status(),
  );
  return response;
}

async function signIn(context: APIRequestContext, email: string, password: string) {
  await postJson(context, "/api/auth/login", { email, password });
}

async function signInPage(page: Page, email: string, password = PASSWORD) {
  const response = await page.request.post("/api/auth/login", {
    data: { email, password },
  });
  expect(response.ok(), `login for ${email} returned ${response.status()}`).toBeTruthy();
}

async function createRole(admin: APIRequestContext, name: string, capabilityKeys: string[]) {
  const capabilities = await getJson<CapabilitySummary[]>(admin, "/api/admin/capabilities");
  const ids = capabilityKeys.map((key) => {
    const capability = capabilities.find((item) => item.key === key);
    expect(capability, `capability ${key} should exist`).toBeTruthy();
    return capability!.id;
  });
  return postJson<IdResponse>(admin, "/api/admin/roles", {
    name,
    capability_ids: ids,
  });
}

async function createUser(admin: APIRequestContext, email: string, displayName: string, roleIds: string[]) {
  return postJson<IdResponse>(admin, "/api/admin/users", {
    email,
    display_name: displayName,
    password: PASSWORD,
    is_active: true,
    role_ids: roleIds,
  });
}

async function assignAccess(
  admin: APIRequestContext,
  accountId: string,
  scopeNodeIds: string[],
  delegateAccountIds: string[] = [],
) {
  await putJson<IdResponse>(admin, `/api/admin/users/${accountId}/access`, {
    scope_node_ids: scopeNodeIds,
    delegate_account_ids: delegateAccountIds,
  });
}

function requireItem<T>(items: T[], predicate: (item: T) => boolean, message: string) {
  const item = items.find(predicate);
  expect(item, message).toBeTruthy();
  return item!;
}

function disjointFrom(nodes: VisibilityNode[], allowed: Set<string>) {
  return nodes.length > 0 && nodes.every((node) => !allowed.has(node.node_id));
}

function overlaps(nodes: VisibilityNode[], allowed: Set<string>) {
  return nodes.some((node) => allowed.has(node.node_id));
}

async function createAssignmentFor(
  admin: APIRequestContext,
  candidates: WorkflowAssignmentCandidate[],
  nodeId: string,
  accountId: string,
) {
  const candidate = requireItem(
    candidates,
    (item) => item.node_id === nodeId,
    `workflow candidate should exist for node ${nodeId}`,
  );
  return postJson<IdResponse>(admin, "/api/workflow-assignments", {
    workflow_version_id: candidate.workflow_version_id,
    node_id: candidate.node_id,
    account_id: accountId,
  });
}

async function setupFixtures(): Promise<FixtureState> {
  const admin = await newContext();
  await signIn(admin, "admin@tessara.local", "tessara-dev-admin");
  await postJson(admin, "/api/demo/seed", {});

  const [
    noAccessRole,
    ownerRole,
    scopedRole,
    globalRole,
  ] = await Promise.all([
    createRole(admin, `${RUN_ID}-no-access`, []),
    createRole(admin, `${RUN_ID}-response-owner`, ["submissions:read_own", "submissions:respond"]),
    createRole(admin, `${RUN_ID}-scoped-operator`, [
      "hierarchy:read",
      "hierarchy:manage",
      "forms:read",
      "forms:manage",
      "workflows:read",
      "workflows:manage",
      "submissions:read_own",
      "submissions:respond",
      "submissions:manage",
      "datasets:read",
      "components:read",
      "dashboards:read",
      "dashboards:manage",
    ]),
    createRole(admin, `${RUN_ID}-global-reader-manager`, [
      "hierarchy:read",
      "forms:read",
      "workflows:read",
      "workflows:manage",
      "submissions:read_own",
      "submissions:respond",
      "submissions:manage",
      "datasets:read",
      "components:read",
      "dashboards:read",
    ]),
  ]);

  const users = {
    scopedManager: await createUser(
      admin,
      `${RUN_ID}-scoped-manager@tessara.local`,
      `${RUN_ID} Scoped Manager`,
      [scopedRole.id],
    ),
    owner: await createUser(admin, `${RUN_ID}-owner@tessara.local`, `${RUN_ID} Owner`, [
      ownerRole.id,
    ]),
    outOfScopeOwner: await createUser(
      admin,
      `${RUN_ID}-out-owner@tessara.local`,
      `${RUN_ID} Out Owner`,
      [ownerRole.id],
    ),
    delegate: await createUser(admin, `${RUN_ID}-delegate@tessara.local`, `${RUN_ID} Delegate`, [
      ownerRole.id,
    ]),
    delegator: await createUser(admin, `${RUN_ID}-delegator@tessara.local`, `${RUN_ID} Delegator`, [
      ownerRole.id,
    ]),
    noAccess: await createUser(admin, `${RUN_ID}-no-access@tessara.local`, `${RUN_ID} No Access`, [
      noAccessRole.id,
    ]),
    global: await createUser(admin, `${RUN_ID}-global@tessara.local`, `${RUN_ID} Global`, [
      globalRole.id,
    ]),
  };

  const adminNodes = await getJson<NodeSummary[]>(admin, "/api/nodes?q=Demo");
  const inScopeNode = requireItem(
    adminNodes,
    (node) => node.name === "Demo Program Family Outreach",
    "Demo Program Family Outreach should exist",
  );
  const outOfScopeNode = requireItem(
    adminNodes,
    (node) => node.name === "Demo Program Workforce Readiness",
    "Demo Program Workforce Readiness should exist",
  );

  await assignAccess(admin, users.scopedManager.id, [inScopeNode.id]);
  await assignAccess(admin, users.delegator.id, [], [users.delegate.id]);

  const scopedManager = await newContext();
  const owner = await newContext();
  const outOfScopeOwner = await newContext();
  const delegate = await newContext();
  const delegator = await newContext();
  const noAccess = await newContext();
  await signIn(scopedManager, `${RUN_ID}-scoped-manager@tessara.local`, PASSWORD);
  await signIn(owner, `${RUN_ID}-owner@tessara.local`, PASSWORD);
  await signIn(outOfScopeOwner, `${RUN_ID}-out-owner@tessara.local`, PASSWORD);
  await signIn(delegate, `${RUN_ID}-delegate@tessara.local`, PASSWORD);
  await signIn(delegator, `${RUN_ID}-delegator@tessara.local`, PASSWORD);
  await signIn(noAccess, `${RUN_ID}-no-access@tessara.local`, PASSWORD);

  const scopedNodes = await getJson<NodeSummary[]>(scopedManager, "/api/nodes?q=Demo");
  const inScopeNodeIds = new Set(scopedNodes.map((node) => node.id));
  expect(inScopeNodeIds.has(inScopeNode.id)).toBe(true);
  expect(inScopeNodeIds.has(outOfScopeNode.id)).toBe(false);

  const adminForms = await getJson<FormSummary[]>(admin, "/api/forms");
  const inScopeForm = requireItem(
    adminForms,
    (form) => overlaps(form.visibility_nodes, inScopeNodeIds),
    "an in-scope form should exist",
  );
  const outOfScopeForm = requireItem(
    adminForms,
    (form) => disjointFrom(form.visibility_nodes, inScopeNodeIds),
    "an out-of-scope form should exist",
  );

  const adminCandidates = await getJson<WorkflowAssignmentCandidate[]>(
    admin,
    "/api/workflow-assignment-candidates",
  );
  expect(adminCandidates.some((item) => item.node_id === inScopeNode.id)).toBe(true);
  expect(adminCandidates.some((item) => item.node_id === outOfScopeNode.id)).toBe(true);

  const inScopeAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.noAccess.id,
  );
  const outOfScopeAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    outOfScopeNode.id,
    users.outOfScopeOwner.id,
  );
  const ownerAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.owner.id,
  );
  const outOfScopeOwnerAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    outOfScopeNode.id,
    users.scopedManager.id,
  );
  const delegateAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.delegate.id,
  );

  const adminDatasets = await getJson<DatasetSummary[]>(admin, "/api/datasets");
  const inScopeDataset = requireItem(
    adminDatasets,
    (dataset) => overlaps(dataset.visibility_nodes, inScopeNodeIds),
    "an in-scope dataset should exist",
  );
  const outOfScopeDataset = requireItem(
    adminDatasets,
    (dataset) => disjointFrom(dataset.visibility_nodes, inScopeNodeIds),
    "an out-of-scope dataset should exist",
  );

  const adminComponents = await getJson<ComponentSummary[]>(admin, "/api/components");
  const scopedComponents = await getJson<ComponentSummary[]>(scopedManager, "/api/components");
  const scopedComponentIds = new Set(scopedComponents.map((component) => component.id));
  const inScopeComponent = requireItem(
    adminComponents,
    (component) => scopedComponentIds.has(component.id),
    "an in-scope component should exist",
  );
  const outOfScopeComponent = requireItem(
    adminComponents,
    (component) => !scopedComponentIds.has(component.id),
    "an out-of-scope component should exist",
  );

  const outDashboard = await postJson<IdResponse>(admin, "/api/admin/dashboards", {
    name: `${RUN_ID} Out Dashboard`,
    description: "Out-of-scope Playwright permission fixture.",
    visibility_node_ids: [outOfScopeNode.id],
  });
  const adminDashboards = await getJson<DashboardSummary[]>(admin, "/api/dashboards");
  const inScopeDashboard = requireItem(
    adminDashboards,
    (dashboard) => overlaps(dashboard.visibility_nodes, inScopeNodeIds),
    "an in-scope dashboard should exist",
  );
  const outOfScopeDashboard = requireItem(
    adminDashboards,
    (dashboard) => dashboard.id === outDashboard.id,
    "the out-of-scope dashboard fixture should exist",
  );

  return {
    admin,
    scopedManager,
    owner,
    outOfScopeOwner,
    delegate,
    delegator,
    noAccess,
    userIds: {
      scopedManager: users.scopedManager.id,
      owner: users.owner.id,
      outOfScopeOwner: users.outOfScopeOwner.id,
      delegate: users.delegate.id,
      delegator: users.delegator.id,
      noAccess: users.noAccess.id,
    },
    inScopeNodeId: inScopeNode.id,
    outOfScopeNodeId: outOfScopeNode.id,
    inScopeNodeIds,
    inScopeForm,
    outOfScopeForm,
    inScopeDataset,
    outOfScopeDataset,
    inScopeComponent,
    outOfScopeComponent,
    inScopeDashboard,
    outOfScopeDashboard,
    inScopeAssignmentId: inScopeAssignment.id,
    outOfScopeAssignmentId: outOfScopeAssignment.id,
    ownerAssignmentId: ownerAssignment.id,
    outOfScopeOwnerAssignmentId: outOfScopeOwnerAssignment.id,
    delegateAssignmentId: delegateAssignment.id,
  };
}

test.describe.serial("capability + scope + ownership permissions", () => {
  test.beforeAll(async () => {
    fixtures = await setupFixtures();
  });

  test.afterAll(async () => {
    await Promise.all(contexts.map((context) => context.dispose()));
  });

  test("no-capability users are denied protected capability surfaces", async () => {
    for (const url of [
      "/api/admin/capabilities",
      "/api/admin/roles",
      "/api/admin/users",
      "/api/admin/node-types",
      "/api/forms",
      "/api/workflows",
      "/api/workflow-assignment-candidates",
      "/api/workflow-assignments",
      "/api/workflow-assignments/pending",
      "/api/submissions",
      "/api/datasets",
      "/api/components",
      "/api/dashboards",
    ]) {
      await expectStatus(fixtures.noAccess, "get", url, [403]);
    }
  });

  test("non-admin shell hides Administration navigation", async ({ page }) => {
    const login = await page.request.post("/api/auth/login", {
      data: {
        email: `${RUN_ID}-scoped-manager@tessara.local`,
        password: PASSWORD,
      },
    });
    expect(login.ok()).toBeTruthy();

    await page.goto("/");
    await expect(page.getByRole("link", { name: "Administration" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Forms" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Responses" })).toBeVisible();
  });

  test("scoped form UI shows visible forms and blocks out-of-scope detail", async ({ page }) => {
    const login = await page.request.post("/api/auth/login", {
      data: {
        email: `${RUN_ID}-scoped-manager@tessara.local`,
        password: PASSWORD,
      },
    });
    expect(login.ok()).toBeTruthy();

    await page.goto("/forms");
    await expect(page.getByRole("heading", { level: 1, name: "Forms" })).toBeVisible();
    await expect(page.getByRole("link", { name: fixtures.inScopeForm.name })).toBeVisible();
    await expect(page.getByRole("link", { name: fixtures.outOfScopeForm.name })).toHaveCount(0);

    await page.goto(`/forms/${fixtures.outOfScopeForm.id}`);
    await expect(page.getByRole("heading", { name: "Form detail unavailable" })).toBeVisible();
  });

  test("admin can create a role and load the roles route", async ({ page }) => {
    const roleName = `${RUN_ID}-ui-role`;
    await createRole(fixtures.admin, roleName, ["forms:read"]);
    const roles = await getJson<RoleSummary[]>(fixtures.admin, "/api/admin/roles");
    expect(roles.some((role) => role.name === roleName)).toBe(true);
    await signInPage(page, "admin@tessara.local", "tessara-dev-admin");

    await page.goto("/administration/roles");
    await expect(page.getByRole("heading", { level: 1, name: "Roles" })).toBeVisible();
  });

  test("hierarchy routes enforce scoped read visibility", async ({ page }) => {
    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);

    await page.goto("/organization");
    await expect(page.getByRole("heading", { name: "Organization Explorer" })).toBeVisible();
    await expect(page.getByText("Demo Program Family Outreach").first()).toBeVisible();
    await expect(page.getByText("Demo Program Workforce Readiness")).toHaveCount(0);

    await page.goto(`/organization/${fixtures.inScopeNodeId}`);
    await expect(page.getByRole("heading", { name: "Organization Detail" })).toBeVisible();
    await expect(page.getByText("Demo Program Family Outreach").first()).toBeVisible();

    await page.goto(`/organization/${fixtures.outOfScopeNodeId}`);
    await expect(page.getByRole("heading", { name: "Organization detail unavailable" })).toBeVisible();

    await page.goto(`/organization/${fixtures.inScopeNodeId}/edit`);
    await expect(page.getByRole("heading", { name: "Edit Organization Node" })).toBeVisible();

    await page.goto(`/organization/${fixtures.outOfScopeNodeId}/edit`);
    await expect(page.getByRole("heading", { name: "Organization node unavailable" })).toBeVisible();

    await page.goto("/organization/new");
    await expect(page.getByRole("heading", { name: "Create Organization Node" })).toBeVisible();
  });

  test("form create and edit routes exercise scoped manage permission", async ({ page }) => {
    const formSlug = `${RUN_ID}-managed-form`;
    const created = await postJson<IdResponse>(fixtures.scopedManager, "/api/admin/forms", {
      name: `${RUN_ID} Managed Form`,
      slug: formSlug,
      scope_node_type_id: null,
      visibility_node_ids: [fixtures.inScopeNodeId],
    });
    await getJson(fixtures.scopedManager, `/api/forms/${created.id}`);

    await expectStatus(fixtures.scopedManager, "post", "/api/admin/forms", [403], {
      name: `${RUN_ID} Out Form`,
      slug: `${RUN_ID}-out-form`,
      scope_node_type_id: null,
      visibility_node_ids: [fixtures.outOfScopeNodeId],
    });

    await putJson<IdResponse>(fixtures.scopedManager, `/api/admin/forms/${created.id}`, {
      name: `${RUN_ID} Managed Form Updated`,
      slug: formSlug,
      scope_node_type_id: null,
      visibility_node_ids: [fixtures.inScopeNodeId],
    });
    await expectStatus(
      fixtures.scopedManager,
      "put",
      `/api/admin/forms/${created.id}`,
      [403],
      {
        name: `${RUN_ID} Managed Form Out`,
        slug: formSlug,
        scope_node_type_id: null,
        visibility_node_ids: [fixtures.outOfScopeNodeId],
      },
    );

    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
    await page.goto("/forms/new");
    await expect(page.getByRole("heading", { name: "Create Form" })).toBeVisible();
    await page.goto(`/forms/${created.id}/edit`);
    await expect(page.getByRole("heading", { name: "Edit Form" })).toBeVisible();
    await page.goto(`/forms/${fixtures.outOfScopeForm.id}/edit`);
    await expect(page.getByRole("button", { name: "Save as Draft" })).toHaveCount(0);
  });

  test("workflow create detail and edit routes exercise scoped manage permission", async ({ page }) => {
    const inWorkflow = await postJson<IdResponse>(fixtures.scopedManager, "/api/workflows", {
      name: `${RUN_ID} Managed Workflow`,
      slug: `${RUN_ID}-managed-workflow`,
      description: "Scoped workflow permission fixture.",
      available_node_ids: [fixtures.inScopeNodeId],
    });
    await getJson<WorkflowDefinition>(fixtures.scopedManager, `/api/workflows/${inWorkflow.id}`);

    await expectStatus(fixtures.scopedManager, "post", "/api/workflows", [403], {
      name: `${RUN_ID} Out Workflow`,
      slug: `${RUN_ID}-out-workflow`,
      description: "Out-of-scope workflow permission fixture.",
      available_node_ids: [fixtures.outOfScopeNodeId],
    });
    await expectStatus(
      fixtures.scopedManager,
      "put",
      `/api/workflows/${inWorkflow.id}`,
      [403],
      {
        name: `${RUN_ID} Managed Workflow Out`,
        slug: `${RUN_ID}-managed-workflow`,
        description: "Should be rejected.",
        available_node_ids: [fixtures.outOfScopeNodeId],
      },
    );

    const outWorkflow = await postJson<IdResponse>(fixtures.admin, "/api/workflows", {
      name: `${RUN_ID} Admin Out Workflow`,
      slug: `${RUN_ID}-admin-out-workflow`,
      description: "Out-of-scope workflow permission fixture.",
      available_node_ids: [fixtures.outOfScopeNodeId],
    });
    await expectStatus(fixtures.scopedManager, "get", `/api/workflows/${outWorkflow.id}`, [403]);

    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
    await page.goto("/workflows/new");
    await expect(page.getByRole("heading", { name: "Create Workflow" })).toBeVisible();
    await page.goto(`/workflows/${inWorkflow.id}`);
    await expect(page.getByRole("heading", { name: `${RUN_ID} Managed Workflow` })).toBeVisible();
    await page.goto(`/workflows/${outWorkflow.id}`);
    await expect(page.getByRole("heading", { name: "Workflow detail unavailable" })).toBeVisible();
    await page.goto(`/workflows/${inWorkflow.id}/edit`);
    await expect(page.getByRole("heading", { name: "Edit Workflow" })).toBeVisible();
    await page.goto(`/workflows/${outWorkflow.id}/edit`);
    await expect(page.getByRole("button", { name: "Save Changes" })).toHaveCount(0);
  });

  test("response edit route follows ownership and delegation permissions", async ({ page }) => {
    const editorRole = await createRole(fixtures.admin, `${RUN_ID}-response-editor`, [
      "submissions:read_own",
      "submissions:respond",
    ]);
    const editorEmail = `${RUN_ID}-response-editor@tessara.local`;
    const editor = await createUser(
      fixtures.admin,
      editorEmail,
      `${RUN_ID} Response Editor`,
      [editorRole.id],
    );
    const editorContext = await newContext();
    await signIn(editorContext, editorEmail, PASSWORD);

    const candidates = await getJson<WorkflowAssignmentCandidate[]>(
      fixtures.admin,
      "/api/workflow-assignment-candidates",
    );
    const assignment = await createAssignmentFor(
      fixtures.admin,
      candidates,
      fixtures.inScopeNodeId,
      editor.id,
    );
    const draft = await postJson<IdResponse>(
      editorContext,
      `/api/workflow-assignments/${assignment.id}/start`,
      {},
    );

    await signInPage(page, editorEmail);
    await page.goto(`/responses/${draft.id}/edit`);
    await expect(page.getByRole("heading", { level: 1, name: "Edit Response" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save Draft" })).toBeVisible();

    await signInPage(page, `${RUN_ID}-delegate@tessara.local`);
    await page.goto(`/responses/${draft.id}/edit`);
    await expect(page.getByRole("heading", { name: "Response unavailable" })).toBeVisible();
  });

  test("dashboard placeholder routes and APIs exercise scoped manage permission", async ({ page }) => {
    const dashboard = await postJson<IdResponse>(fixtures.scopedManager, "/api/admin/dashboards", {
      name: `${RUN_ID} Managed Dashboard`,
      description: "Scoped dashboard permission fixture.",
      visibility_node_ids: [fixtures.inScopeNodeId],
    });
    await getJson<DashboardDefinition>(fixtures.scopedManager, `/api/dashboards/${dashboard.id}`);
    await expectStatus(fixtures.scopedManager, "post", "/api/admin/dashboards", [403], {
      name: `${RUN_ID} Out Dashboard Denied`,
      description: "Should be rejected.",
      visibility_node_ids: [fixtures.outOfScopeNodeId],
    });
    await putJson<IdResponse>(fixtures.scopedManager, `/api/admin/dashboards/${dashboard.id}`, {
      name: `${RUN_ID} Managed Dashboard Updated`,
      description: "Scoped dashboard permission fixture updated.",
      visibility_node_ids: [fixtures.inScopeNodeId],
    });
    await expectStatus(
      fixtures.scopedManager,
      "put",
      `/api/admin/dashboards/${dashboard.id}`,
      [403],
      {
        name: `${RUN_ID} Managed Dashboard Out`,
        description: "Should be rejected.",
        visibility_node_ids: [fixtures.outOfScopeNodeId],
      },
    );

    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
    await page.goto("/dashboards/new");
    await expect(page.getByRole("heading", { level: 1, name: "Create Dashboard" })).toBeVisible();
    await page.goto(`/dashboards/${dashboard.id}/edit`);
    await expect(page.getByRole("heading", { level: 1, name: "Edit Dashboard" })).toBeVisible();
  });

  test("administration user and node-type routes are admin-only", async ({ page }) => {
    const nodeType = await postJson<IdResponse>(fixtures.admin, "/api/admin/node-types", {
      name: `${RUN_ID} Node Type`,
      slug: `${RUN_ID}-node-type`,
      plural_label: `${RUN_ID} Node Types`,
      parent_node_type_ids: [],
      child_node_type_ids: [],
    });
    await putJson<IdResponse>(fixtures.admin, `/api/admin/node-types/${nodeType.id}`, {
      name: `${RUN_ID} Node Type Updated`,
      slug: `${RUN_ID}-node-type`,
      plural_label: `${RUN_ID} Node Types`,
      parent_node_type_ids: [],
      child_node_type_ids: [],
    });

    await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
    await page.goto("/administration/users");
    await expect(page.getByRole("heading", { level: 1, name: "Users" })).toBeVisible();
    await page.getByPlaceholder("Search users").fill(`${RUN_ID} Owner`);
    await expect(page.getByRole("link", { name: `${RUN_ID} Owner` })).toBeVisible();

    await page.goto(`/administration/users/${fixtures.userIds.owner}`);
    await expect(page.getByRole("heading", { name: `${RUN_ID} Owner` })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save Permissions" })).toBeVisible();

    await page.goto(`/administration/users/${fixtures.userIds.owner}/access`);
    await expect(page.getByRole("heading", { name: `${RUN_ID} Owner` })).toBeVisible();

    await page.goto(`/administration/users/${fixtures.userIds.owner}/edit`);
    await expect(page.getByRole("heading", { name: "Edit User" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save User" })).toBeVisible();

    await page.goto("/administration/node-types");
    await expect(page.getByRole("heading", { level: 1, name: "Node Types" })).toBeVisible();

    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
    for (const url of [
      "/api/admin/users",
      `/api/admin/users/${fixtures.userIds.owner}`,
      `/api/admin/users/${fixtures.userIds.owner}/access`,
      "/api/admin/node-types",
    ]) {
      await expectStatus(fixtures.scopedManager, "get", url, [403]);
    }
  });

  test("admin has global access to in-scope and out-of-scope fixtures", async () => {
    await getJson(fixtures.admin, `/api/forms/${fixtures.inScopeForm.id}`);
    await getJson(fixtures.admin, `/api/forms/${fixtures.outOfScopeForm.id}`);
    await getJson(fixtures.admin, `/api/datasets/${fixtures.inScopeDataset.id}`);
    await getJson(fixtures.admin, `/api/datasets/${fixtures.outOfScopeDataset.id}`);
    await getJson(fixtures.admin, `/api/components/${fixtures.inScopeComponent.slug}`);
    await getJson(fixtures.admin, `/api/components/${fixtures.outOfScopeComponent.slug}`);
    await getJson(fixtures.admin, `/api/dashboards/${fixtures.inScopeDashboard.id}`);
    await getJson(fixtures.admin, `/api/dashboards/${fixtures.outOfScopeDashboard.id}`);

    const assignments = await getJson<WorkflowAssignmentSummary[]>(
      fixtures.admin,
      "/api/workflow-assignments",
    );
    expect(assignments.some((item) => item.id === fixtures.inScopeAssignmentId)).toBe(true);
    expect(assignments.some((item) => item.id === fixtures.outOfScopeAssignmentId)).toBe(true);
  });

  test("scoped manager reads in-scope surfaces and is denied out-of-scope surfaces", async () => {
    const forms = await getJson<FormSummary[]>(fixtures.scopedManager, "/api/forms");
    expect(forms.some((form) => form.id === fixtures.inScopeForm.id)).toBe(true);
    expect(forms.some((form) => form.id === fixtures.outOfScopeForm.id)).toBe(false);
    await getJson(fixtures.scopedManager, `/api/forms/${fixtures.inScopeForm.id}`);
    await expectStatus(fixtures.scopedManager, "get", `/api/forms/${fixtures.outOfScopeForm.id}`, [
      403,
    ]);

    const datasets = await getJson<DatasetSummary[]>(fixtures.scopedManager, "/api/datasets");
    expect(datasets.some((dataset) => dataset.id === fixtures.inScopeDataset.id)).toBe(true);
    expect(datasets.some((dataset) => dataset.id === fixtures.outOfScopeDataset.id)).toBe(false);
    await getJson(fixtures.scopedManager, `/api/datasets/${fixtures.inScopeDataset.id}`);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/datasets/${fixtures.outOfScopeDataset.id}`,
      [403],
    );

    const components = await getJson<ComponentSummary[]>(fixtures.scopedManager, "/api/components");
    expect(components.some((component) => component.id === fixtures.inScopeComponent.id)).toBe(true);
    expect(components.some((component) => component.id === fixtures.outOfScopeComponent.id)).toBe(false);
    const inComponent = await getJson<ComponentDefinition>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeComponent.slug}`,
    );
    expect(inComponent.versions.length).toBeGreaterThan(0);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${fixtures.outOfScopeComponent.slug}`,
      [403],
    );

    const dashboards = await getJson<DashboardSummary[]>(fixtures.scopedManager, "/api/dashboards");
    expect(dashboards.some((dashboard) => dashboard.id === fixtures.inScopeDashboard.id)).toBe(true);
    expect(dashboards.some((dashboard) => dashboard.id === fixtures.outOfScopeDashboard.id)).toBe(false);
    await getJson(fixtures.scopedManager, `/api/dashboards/${fixtures.inScopeDashboard.id}`);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/dashboards/${fixtures.outOfScopeDashboard.id}`,
      [403],
    );
  });

  test("workflow assignment candidates and starts respect manager scope", async () => {
    const candidates = await getJson<WorkflowAssignmentCandidate[]>(
      fixtures.scopedManager,
      "/api/workflow-assignment-candidates",
    );
    expect(candidates.length).toBeGreaterThan(0);
    expect(candidates.every((item) => fixtures.inScopeNodeIds.has(item.node_id))).toBe(true);

    const inCandidate = requireItem(
      candidates,
      (item) => item.node_id === fixtures.inScopeNodeId,
      "scoped manager should have an in-scope workflow candidate",
    );
    const assignees = await getJson<WorkflowAssigneeOption[]>(
      fixtures.scopedManager,
      `/api/workflow-assignment-candidates/assignees?workflow_version_id=${inCandidate.workflow_version_id}&node_id=${inCandidate.node_id}`,
    );
    expect(assignees.some((item) => item.account_id === fixtures.userIds.owner)).toBe(true);

    const visibleAssignments = await getJson<WorkflowAssignmentSummary[]>(
      fixtures.scopedManager,
      "/api/workflow-assignments",
    );
    expect(visibleAssignments.some((item) => item.id === fixtures.inScopeAssignmentId)).toBe(true);
    expect(visibleAssignments.some((item) => item.id === fixtures.outOfScopeAssignmentId)).toBe(false);

    await postJson<IdResponse>(
      fixtures.scopedManager,
      `/api/workflow-assignments/${fixtures.inScopeAssignmentId}/start`,
      {},
    );
    await expectStatus(
      fixtures.scopedManager,
      "post",
      `/api/workflow-assignments/${fixtures.outOfScopeAssignmentId}/start`,
      [403],
      {},
    );
    await expectStatus(
      fixtures.scopedManager,
      "post",
      "/api/workflow-assignments",
      [400, 403],
      {
        workflow_version_id: inCandidate.workflow_version_id,
        node_id: fixtures.outOfScopeNodeId,
        account_id: fixtures.userIds.owner,
      },
    );
  });

  test("submission management combines scope with response ownership", async () => {
    const ownOutOfScope = await postJson<IdResponse>(
      fixtures.scopedManager,
      `/api/workflow-assignments/${fixtures.outOfScopeOwnerAssignmentId}/start`,
      {},
    );
    const ownOutDetail = await getJson<SubmissionDetail>(
      fixtures.scopedManager,
      `/api/submissions/${ownOutOfScope.id}`,
    );
    expect(ownOutDetail.id).toBe(ownOutOfScope.id);
    expect(ownOutDetail.node_id).toBe(fixtures.outOfScopeNodeId);

    const unrelatedOutOfScope = await postJson<IdResponse>(
      fixtures.outOfScopeOwner,
      `/api/workflow-assignments/${fixtures.outOfScopeAssignmentId}/start`,
      {},
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/submissions/${unrelatedOutOfScope.id}`,
      [403],
    );

    const submissions = await getJson<SubmissionSummary[]>(fixtures.scopedManager, "/api/submissions");
    expect(submissions.some((item) => item.id === ownOutOfScope.id)).toBe(false);
    expect(submissions.every((item) => fixtures.inScopeNodeIds.has(item.node_id))).toBe(true);
  });

  test("owners and delegators can access owned or delegated work only", async () => {
    const ownerPending = await getJson<PendingWorkflowWork[]>(
      fixtures.owner,
      "/api/workflow-assignments/pending",
    );
    expect(ownerPending.some((item) => item.workflow_assignment_id === fixtures.ownerAssignmentId)).toBe(
      true,
    );
    expect(ownerPending.some((item) => item.workflow_assignment_id === fixtures.delegateAssignmentId)).toBe(
      false,
    );

    const ownerSubmission = await postJson<IdResponse>(
      fixtures.owner,
      `/api/workflow-assignments/${fixtures.ownerAssignmentId}/start`,
      {},
    );
    await getJson(fixtures.owner, `/api/submissions/${ownerSubmission.id}`);

    await expectStatus(
      fixtures.owner,
      "post",
      `/api/workflow-assignments/${fixtures.delegateAssignmentId}/start`,
      [403],
      {},
    );

    const delegatePending = await getJson<PendingWorkflowWork[]>(
      fixtures.delegate,
      "/api/workflow-assignments/pending",
    );
    expect(delegatePending.some((item) => item.workflow_assignment_id === fixtures.delegateAssignmentId)).toBe(
      true,
    );

    const delegatedPending = await getJson<PendingWorkflowWork[]>(
      fixtures.delegator,
      `/api/workflow-assignments/pending?delegate_account_id=${fixtures.userIds.delegate}`,
    );
    expect(delegatedPending.map((item) => item.workflow_assignment_id)).toContain(
      fixtures.delegateAssignmentId,
    );
    const delegatedSubmission = await postJson<IdResponse>(
      fixtures.delegator,
      `/api/workflow-assignments/${fixtures.delegateAssignmentId}/start`,
      {},
    );
    await getJson(fixtures.delegator, `/api/submissions/${delegatedSubmission.id}`);
  });

  test("session metadata exposes capabilities, scopes, and delegations without legacy access switches", async () => {
    const scopedSession = await getJson<SessionState>(fixtures.scopedManager, "/api/auth/session");
    expect(scopedSession.authenticated).toBe(true);
    expect(scopedSession.account?.capabilities).toEqual(
      expect.arrayContaining(["forms:read", "workflows:manage", "submissions:manage"]),
    );
    expect(scopedSession.account?.scope_nodes.map((node) => node.node_name)).toContain(
      "Demo Program Family Outreach",
    );

    const delegatorSession = await getJson<SessionState>(fixtures.delegator, "/api/auth/session");
    expect(delegatorSession.account?.delegations.map((item) => item.account_id)).toContain(
      fixtures.userIds.delegate,
    );
  });
});
