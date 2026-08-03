import {
  expect,
  request,
  test,
  type APIRequestContext,
  type APIResponse,
  type Browser,
  type Page,
} from "@playwright/test";
import { invokeDemoSeedEndpoint } from "./support/demo-seed";
import { runPlaywrightSql } from "./support/postgres";
import {
  attachNativeRouteGuard,
  expectHydratedNativeRouteDirectLoadAndRefresh,
  expectNoJavaScriptNativeRouteDirectLoadAndRefresh,
} from "./support/native-route";

const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const RUN_ID = `pw-permissions-${Date.now()}`;
const PLAYWRIGHT_ENTITY_PREFIX = "pw-permissions-";
const PASSWORD = "tessara-dev-permissions";
const DASHBOARD_DOCUMENT_ROOT = "#module-content";

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
type FormSummary = {
  id: string;
  name: string;
  slug: string;
  visibility_nodes: VisibilityNode[];
  versions: Array<{ id: string; status: string }>;
};
type WorkflowSummary = { id: string; name: string; slug: string; available_nodes: Array<{ id: string; name: string }> };
type WorkflowDefinition = WorkflowSummary & { versions: Array<{ id: string; status: string; steps: Array<{ form_version_id: string }> }> };
type DatasetSummary = {
  id: string;
  name: string;
  slug?: string;
  visibility_nodes: VisibilityNode[];
  output_fields: Array<{ key: string; label: string; field_type: string }>;
  current_revision_id: string | null;
  current_version_major?: number | null;
  current_version_minor?: number | null;
  current_version_patch?: number | null;
  major_versions?: number[];
};
type DatasetDefinition = DatasetSummary & {
  slug: string;
  initial_source: Record<string, unknown>;
  operations: Array<Record<string, unknown>>;
  restriction_policy?: Record<string, unknown> | null;
};
type DatasetDraftRevisionResponse = {
  dataset_id: string;
  revision_id: string;
  status: "draft";
};
type DatasetTable = {
  rows: Array<{
    node_name: string;
    values: Record<string, string | null>;
  }>;
};
type ComponentSummary = { id: string; name: string; slug: string };
type ComponentDefinition = {
  id: string;
  name: string;
  slug: string;
  versions: Array<{ id: string; status: string; component_type: string }>;
};
type ComponentTable = {
  component_version_id: string;
  materialization_state: string;
  rows: Array<{ values: Record<string, string | null> }>;
};
type ComponentVisual = {
  component_version_id: string;
  materialization_state: string;
  component_type: string;
  points: Array<{ x: string; value: number }>;
};
type DashboardSummary = { id: string; name: string; visibility_nodes: VisibilityNode[] };
type DashboardDefinition = DashboardSummary & { description: string | null };
type OperationsStatus = {
  summary: {
    open_workflow_assignment_count: number;
    draft_response_count: number;
    dataset_attention_count: number;
  };
  workflow_assignments: Array<{ workflow_assignment_id: string; workflow_id: string; workflow_name: string; node_id: string }>;
  dataset_readiness: { datasets: Array<{ dataset_id: string; readiness: string }> };
};
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
type ApiErrorBody = {
  code: string;
  message: string;
  error: string;
};

type FrozenNativeRoute = {
  path: string;
  expectedText: string;
  expectedRootMarkup?: string;
  documentRootSelector?: string;
  contentSelector?: string;
};

type FixtureState = {
  admin: APIRequestContext;
  scopedManager: APIRequestContext;
  componentManager: APIRequestContext;
  partialComponentManager: APIRequestContext;
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
  inScopeVisualComponent: ComponentSummary;
  outOfScopeVisualComponent: ComponentSummary;
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

async function ensureDemoSeed(admin: APIRequestContext) {
  const response = await invokeDemoSeedEndpoint(admin);
  if (response === null) {
    return;
  }
  const text = await response.text();
  if (response.ok()) {
    return;
  }
  if (response.status() === 400 && text.includes("Demo seed requires an empty database")) {
    return;
  }
  expect(response.ok(), `${response.url()} returned ${response.status()}: ${text}`).toBeTruthy();
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

async function expectErrorStatus(
  context: APIRequestContext,
  method: "get" | "post" | "put" | "delete",
  url: string,
  status: number,
  code: string,
  data?: Record<string, unknown>,
) {
  const response = await expectStatus(context, method, url, [status], data);
  const body = (await response.json()) as ApiErrorBody;
  expect(body.code).toBe(code);
  expect(body.message).toBeTruthy();
  expect(body.error).toBe(body.message);
  return body;
}

async function signIn(context: APIRequestContext, email: string, password: string) {
  await postJson(context, "/api/auth/login", { email, password });
}

async function signInPage(page: Page, email: string, password = PASSWORD) {
  const response = await page.request.post("/api/auth/login", {
    data: { email, password },
  });
  expect(response.ok(), `login for ${email} returned ${response.status()}`).toBeTruthy();
  const body = (await response.json()) as { token: string };
  await page.context().addCookies([
    {
      name: "tessara_session",
      value: body.token,
      url: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080",
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

async function expectNoJavaScriptRoutes(
  page: Page,
  routes: FrozenNativeRoute[],
) {
  for (const route of routes) {
    await expectNoJavaScriptNativeRouteDirectLoadAndRefresh(page, {
      path: route.path,
      expectedRootMarkup: route.expectedRootMarkup,
      documentRootSelector: route.documentRootSelector,
      ready: async (routePage) => {
        const routeContent = routePage.locator(
          route.contentSelector ?? ".route-panel",
        );
        await expect(routeContent).toHaveCount(1);
        await expect(routeContent).toBeVisible();
        await expect(
          routeContent
            .getByText(route.expectedText, { exact: true })
            .filter({ visible: true })
            .first(),
        ).toBeVisible();
      },
    });
  }
}

async function expectHydratedRoute(page: Page, route: FrozenNativeRoute) {
  await expectHydratedNativeRouteDirectLoadAndRefresh(page, {
    path: route.path,
    expectedRootMarkup: route.expectedRootMarkup,
    documentRootSelector: route.documentRootSelector,
    ready: async (routePage) => {
      await expect(
        routePage
          .getByText(route.expectedText, { exact: true })
          .filter({ visible: true })
          .first(),
      ).toBeVisible();
    },
  });
}

async function withNoJavaScriptPage(
  browser: Browser,
  run: (page: Page) => Promise<void>,
) {
  const context = await browser.newContext({
    baseURL: BASE_URL,
    javaScriptEnabled: false,
  });
  try {
    const page = await context.newPage();
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    await run(page);
    await assertNativeRouteGuard();
  } finally {
    await context.close();
  }
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

function datasetMajor(dataset: DatasetSummary) {
  const major = dataset.major_versions?.[0] ?? dataset.current_version_major ?? undefined;
  expect(major, `dataset ${dataset.name} should expose a major version`).toBeTruthy();
  return major!;
}

function tableConfig(dataset: DatasetSummary) {
  const firstField = dataset.output_fields[0]?.key;
  expect(firstField, `dataset ${dataset.name} should expose output fields`).toBeTruthy();
  return {
    visible_columns: [firstField],
  };
}

function visualConfig(dataset: DatasetSummary) {
  const firstField = dataset.output_fields[0]?.key;
  expect(firstField, `dataset ${dataset.name} should expose output fields`).toBeTruthy();
  return {
    mode: "summary",
    summary_field: firstField,
    summary_type: "count",
    category_field: firstField,
    sort_field: "summary_value",
    sort_direction: "desc",
    number_of_points: 20,
    value_format: "integer",
  };
}

async function createPublishedVisualComponent(
  admin: APIRequestContext,
  dataset: DatasetSummary,
  slug: string,
  name: string,
) {
  const component = await postJson<IdResponse>(admin, "/api/admin/components", {
    name,
    slug,
    description: "Visual component permission fixture.",
    version: {
      dataset_id: dataset.id,
      dataset_version_major: datasetMajor(dataset),
      component_type: "bar",
      config: visualConfig(dataset),
    },
  });
  const detail = await getJson<ComponentDefinition>(admin, `/api/admin/components/${slug}`);
  const version = detail.versions[0];
  expect(version.component_type).toBe("bar");
  await postJson<IdResponse>(
    admin,
    `/api/admin/components/${component.id}/versions/${version.id}/publish`,
  );
  return { id: component.id, name, slug };
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
  await ensureDemoSeed(admin);
  await cleanupPlaywrightDashboards(admin);

  const [
    noAccessRole,
    ownerRole,
    scopedRole,
    componentManagerRole,
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
      "operations:view",
      "datasets:read",
      "components:read",
      "dashboards:read",
      "dashboards:manage",
    ]),
    createRole(admin, `${RUN_ID}-component-manager`, [
      "datasets:read",
      "components:read",
      "components:manage",
    ]),
    createRole(admin, `${RUN_ID}-global-reader-manager`, [
      "hierarchy:read",
      "forms:read",
      "workflows:read",
      "workflows:manage",
      "submissions:read_own",
      "submissions:respond",
      "submissions:manage",
      "operations:view",
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
    componentManager: await createUser(
      admin,
      `${RUN_ID}-component-manager@tessara.local`,
      `${RUN_ID} Component Manager`,
      [componentManagerRole.id],
    ),
    partialComponentManager: await createUser(
      admin,
      `${RUN_ID}-partial-component-manager@tessara.local`,
      `${RUN_ID} Partial Component Manager`,
      [componentManagerRole.id],
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
  await assignAccess(admin, users.componentManager.id, [inScopeNode.id]);
  await assignAccess(admin, users.partialComponentManager.id, [inScopeNode.id]);
  await assignAccess(admin, users.delegator.id, [], [users.delegate.id]);

  const scopedManager = await newContext();
  const componentManager = await newContext();
  const partialComponentManager = await newContext();
  const owner = await newContext();
  const outOfScopeOwner = await newContext();
  const delegate = await newContext();
  const delegator = await newContext();
  const noAccess = await newContext();
  await signIn(scopedManager, `${RUN_ID}-scoped-manager@tessara.local`, PASSWORD);
  await signIn(componentManager, `${RUN_ID}-component-manager@tessara.local`, PASSWORD);
  await signIn(partialComponentManager, `${RUN_ID}-partial-component-manager@tessara.local`, PASSWORD);
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
    (dataset) =>
      overlaps(dataset.visibility_nodes, inScopeNodeIds) &&
      dataset.visibility_nodes.some((node) => !inScopeNodeIds.has(node.node_id)),
    "a partial-overlap in-scope dataset should exist",
  );
  await assignAccess(
    admin,
    users.componentManager.id,
    inScopeDataset.visibility_nodes.map((node) => node.node_id),
  );
  const componentManagerNodeIds = new Set(inScopeDataset.visibility_nodes.map((node) => node.node_id));
  const outOfScopeDataset = requireItem(
    adminDatasets,
    (dataset) =>
      disjointFrom(dataset.visibility_nodes, inScopeNodeIds) &&
      disjointFrom(dataset.visibility_nodes, componentManagerNodeIds),
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
  const inScopeVisualComponent = await createPublishedVisualComponent(
    admin,
    inScopeDataset,
    `${RUN_ID}-visible-bar-component`,
    `${RUN_ID} Visible Bar Component`,
  );
  const outOfScopeVisualComponent = await createPublishedVisualComponent(
    admin,
    outOfScopeDataset,
    `${RUN_ID}-hidden-bar-component`,
    `${RUN_ID} Hidden Bar Component`,
  );

  const inDashboard = await postJson<IdResponse>(admin, "/api/admin/dashboards", {
    name: `${RUN_ID} In Dashboard`,
    description: "In-scope Playwright permission fixture.",
    visibility_node_ids: [inScopeNode.id],
  });
  const outDashboard = await postJson<IdResponse>(admin, "/api/admin/dashboards", {
    name: `${RUN_ID} Out Dashboard`,
    description: "Out-of-scope Playwright permission fixture.",
    visibility_node_ids: [outOfScopeNode.id],
  });
  const adminDashboards = await getJson<DashboardSummary[]>(admin, "/api/dashboards");
  const inScopeDashboard = requireItem(
    adminDashboards,
    (dashboard) => dashboard.id === inDashboard.id,
    "the in-scope dashboard fixture should exist",
  );
  const outOfScopeDashboard = requireItem(
    adminDashboards,
    (dashboard) => dashboard.id === outDashboard.id,
    "the out-of-scope dashboard fixture should exist",
  );

  return {
    admin,
    scopedManager,
    componentManager,
    partialComponentManager,
    owner,
    outOfScopeOwner,
    delegate,
    delegator,
    noAccess,
    userIds: {
      scopedManager: users.scopedManager.id,
      componentManager: users.componentManager.id,
      partialComponentManager: users.partialComponentManager.id,
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
    inScopeVisualComponent,
    outOfScopeVisualComponent,
    inScopeDashboard,
    outOfScopeDashboard,
    inScopeAssignmentId: inScopeAssignment.id,
    outOfScopeAssignmentId: outOfScopeAssignment.id,
    ownerAssignmentId: ownerAssignment.id,
    outOfScopeOwnerAssignmentId: outOfScopeOwnerAssignment.id,
    delegateAssignmentId: delegateAssignment.id,
  };
}

function cleanupPlaywrightEntities() {
  const sql = `
CREATE TEMP TABLE pw_cleanup_accounts AS
SELECT id FROM accounts
  WHERE email LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
     OR display_name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

CREATE TEMP TABLE pw_cleanup_forms AS
SELECT id FROM forms
  WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
     OR slug LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

CREATE TEMP TABLE pw_cleanup_workflows AS
SELECT id FROM workflows
  WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
     OR slug LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

CREATE TEMP TABLE pw_cleanup_components AS
SELECT id FROM components
  WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
     OR slug LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

CREATE TEMP TABLE pw_cleanup_workflow_versions AS
SELECT workflow_versions.id
  FROM workflow_versions
  JOIN pw_cleanup_workflows ON pw_cleanup_workflows.id = workflow_versions.workflow_id;

CREATE TEMP TABLE pw_cleanup_workflow_assignments AS
SELECT workflow_assignments.id
  FROM workflow_assignments
  LEFT JOIN pw_cleanup_accounts account_scope ON account_scope.id = workflow_assignments.account_id
  LEFT JOIN pw_cleanup_accounts assigner_scope ON assigner_scope.id = workflow_assignments.assigned_by_account_id
  LEFT JOIN pw_cleanup_workflow_versions ON pw_cleanup_workflow_versions.id = workflow_assignments.workflow_version_id
  WHERE account_scope.id IS NOT NULL
     OR assigner_scope.id IS NOT NULL
     OR pw_cleanup_workflow_versions.id IS NOT NULL;

CREATE TEMP TABLE pw_cleanup_workflow_instances AS
SELECT workflow_instances.id
  FROM workflow_instances
  LEFT JOIN pw_cleanup_workflow_assignments ON pw_cleanup_workflow_assignments.id = workflow_instances.workflow_assignment_id
  LEFT JOIN pw_cleanup_accounts assignee_scope ON assignee_scope.id = workflow_instances.assignee_account_id
  LEFT JOIN pw_cleanup_accounts starter_scope ON starter_scope.id = workflow_instances.started_by_account_id
  WHERE pw_cleanup_workflow_assignments.id IS NOT NULL
     OR assignee_scope.id IS NOT NULL
     OR starter_scope.id IS NOT NULL;

CREATE TEMP TABLE pw_cleanup_submissions AS
SELECT submissions.id
  FROM submissions
  LEFT JOIN pw_cleanup_workflow_assignments ON pw_cleanup_workflow_assignments.id = submissions.workflow_assignment_id
  LEFT JOIN pw_cleanup_workflow_instances ON pw_cleanup_workflow_instances.id = submissions.workflow_instance_id
  LEFT JOIN form_versions ON form_versions.id = submissions.form_version_id
  LEFT JOIN pw_cleanup_forms ON pw_cleanup_forms.id = form_versions.form_id
  WHERE pw_cleanup_workflow_assignments.id IS NOT NULL
     OR pw_cleanup_workflow_instances.id IS NOT NULL
     OR pw_cleanup_forms.id IS NOT NULL;

DELETE FROM analytics.submission_value_fact
WHERE submission_id IN (SELECT id FROM pw_cleanup_submissions);

DELETE FROM analytics.submission_fact
WHERE submission_id IN (SELECT id FROM pw_cleanup_submissions);

DELETE FROM submissions
WHERE id IN (SELECT id FROM pw_cleanup_submissions);

DELETE FROM workflow_instances
WHERE id IN (SELECT id FROM pw_cleanup_workflow_instances);

DELETE FROM workflow_assignments
WHERE id IN (SELECT id FROM pw_cleanup_workflow_assignments);

DELETE FROM component_versions
WHERE component_id IN (SELECT id FROM pw_cleanup_components);

DELETE FROM components
WHERE id IN (SELECT id FROM pw_cleanup_components);

DELETE FROM workflows
WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
   OR slug LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

DELETE FROM forms
WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
   OR slug LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

DELETE FROM node_types
WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
   OR slug LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

DELETE FROM accounts
WHERE email LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%'
   OR display_name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';

DELETE FROM roles
WHERE name LIKE '${PLAYWRIGHT_ENTITY_PREFIX}%';
`;

  runPlaywrightSql(sql);
}

async function cleanupPlaywrightDashboards(admin: APIRequestContext) {
  const dashboards = await getJson<DashboardSummary[]>(admin, "/api/dashboards");
  for (const dashboard of dashboards.filter((candidate) =>
    candidate.name.startsWith(PLAYWRIGHT_ENTITY_PREFIX),
  )) {
    const response = await admin.delete(`/api/admin/dashboards/${dashboard.id}`);
    expect(
      response.ok(),
      `Dashboard cleanup for ${dashboard.id} returned ${response.status()}`,
    ).toBeTruthy();
  }
}

test.describe.serial("capability + scope + ownership permissions", () => {
  test.beforeAll(async () => {
    cleanupPlaywrightEntities();
    fixtures = await setupFixtures();
  });

  test.afterAll(async () => {
    try {
      if (fixtures) {
        await cleanupPlaywrightDashboards(fixtures.admin);
      }
      cleanupPlaywrightEntities();
    } finally {
      await Promise.all(contexts.map((context) => context.dispose()));
    }
  });

  test("no-capability users are denied protected capability surfaces", async () => {
    const inScopePublishedVersion = fixtures.inScopeForm.versions.find((version) => version.status === "published");
    expect(inScopePublishedVersion).toBeTruthy();
    for (const url of [
      "/api/admin/capabilities",
      "/api/admin/roles",
      "/api/admin/users",
      "/api/admin/node-types",
      "/api/admin/components",
      "/api/forms",
      `/api/form-versions/${inScopePublishedVersion!.id}/render`,
      "/api/workflows",
      "/api/workflow-assignment-candidates",
      "/api/workflow-assignments",
      "/api/workflow-assignments/pending",
      "/api/submissions",
      "/api/operations/status",
      "/api/datasets",
      `/api/datasets/${fixtures.inScopeDataset.id}/table`,
      "/api/components",
      "/api/dashboards",
    ]) {
      await expectStatus(fixtures.noAccess, "get", url, [403]);
    }
  });

  test("non-admin shell contains only eligible configured destinations", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    const login = await page.request.post("/api/auth/login", {
      data: {
        email: `${RUN_ID}-scoped-manager@tessara.local`,
        password: PASSWORD,
      },
    });
    expect(login.ok()).toBeTruthy();

    await expectHydratedRoute(page, { path: "/", expectedText: "Home" });
    await expect(page.getByRole("link", { name: "Module Management" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "User Management" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Roles & Access" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Node Types" })).toHaveCount(0);
    await expect(page.getByRole("link", { name: "Operations" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Forms" })).toBeVisible();
    await expect(page.getByRole("link", { name: "Responses" })).toBeVisible();
    await assertNativeRouteGuard();
  });

  test("scoped form UI shows visible forms and blocks out-of-scope detail", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    const login = await page.request.post("/api/auth/login", {
      data: {
        email: `${RUN_ID}-scoped-manager@tessara.local`,
        password: PASSWORD,
      },
    });
    expect(login.ok()).toBeTruthy();

    await expectHydratedRoute(page, { path: "/forms", expectedText: "Forms" });
    await expect(page.getByRole("heading", { level: 1, name: "Forms" })).toBeVisible();
    await expect(page.getByRole("link", { name: fixtures.inScopeForm.name })).toBeVisible();
    await expect(page.getByRole("link", { name: fixtures.outOfScopeForm.name })).toHaveCount(0);

    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/forms/${fixtures.outOfScopeForm.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/forms/${fixtures.outOfScopeForm.id}`,
        expectedText: "Form detail unavailable",
      });
      await expect(page.getByRole("heading", { name: "Form detail unavailable" })).toBeVisible();
    });
    await assertNativeRouteGuard();
  });

  test("admin can create a role and load the roles route", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    const roleName = `${RUN_ID}-ui-role`;
    await createRole(fixtures.admin, roleName, ["forms:read"]);
    const roles = await getJson<RoleSummary[]>(fixtures.admin, "/api/admin/roles");
    expect(roles.some((role) => role.name === roleName)).toBe(true);
    await signInPage(page, "admin@tessara.local", "tessara-dev-admin");

    await page.goto("/administration/roles");
    await expect(page.locator("#app-root")).toHaveAttribute("data-hydration", "ready");
    await expect(page.getByRole("heading", { level: 1, name: "Roles" })).toBeVisible();

    await page.getByRole("button", { name: "New Role" }).click();
    const sheet = page.locator(".sheet-panel");
    await expect(sheet.getByRole("heading", { level: 2, name: "New Role" })).toBeVisible();
    await expect(sheet.getByText("Capability scope", { exact: true })).toBeVisible();
    await expect(
      sheet.getByText(/dedicated global module role alongside separate scoped product roles/),
    ).toBeVisible();

    const formsRead = sheet.getByRole("checkbox", { name: /forms:read/ });
    const modulesRead = sheet.getByRole("checkbox", { name: /modules:read/ });
    const adminAll = sheet.getByRole("checkbox", { name: /admin:all/ });
    await expect(formsRead).toHaveAttribute("aria-describedby", /-metadata$/);
    await expect(modulesRead).toHaveAttribute("aria-describedby", /-metadata$/);
    await sheet.getByText("forms:read", { exact: true }).click();
    await expect(formsRead).toBeChecked();
    await expect(sheet.getByRole("checkbox", { name: /modules:read/ })).toHaveCount(0);
    await expect(adminAll).toBeVisible();

    await sheet.getByText("admin:all", { exact: true }).click();
    await expect(adminAll).toBeChecked();
    await expect(sheet.getByText("Global admin exception", { exact: true })).toBeVisible();
    await expect(sheet.getByText(/complete role is installation-global/)).toBeVisible();
    await expect(sheet.getByRole("checkbox", { name: /modules:read/ })).toBeVisible();
    await expect(sheet.getByRole("button", { name: "Save Role" })).toBeEnabled();

    await sheet.getByRole("button", { name: "Cancel" }).click();
    await page.getByRole("button", { name: "New Role" }).click();
    const globalSheet = page.locator(".sheet-panel");
    await globalSheet.getByText("modules:read", { exact: true }).click();
    await expect(globalSheet.getByRole("checkbox", { name: /modules:read/ })).toBeChecked();
    await expect(globalSheet.getByRole("checkbox", { name: /forms:read/ })).toHaveCount(0);
    await expect(globalSheet.getByRole("checkbox", { name: /admin:all/ })).toBeVisible();
    await expectHydratedRoute(page, {
      path: "/administration/roles",
      expectedText: "Roles",
    });
    await assertNativeRouteGuard();
  });

  test("hierarchy routes enforce scoped read visibility", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);

    await expectHydratedRoute(page, {
      path: "/organization",
      expectedText: "Organization Explorer",
    });
    await expect(page.getByRole("heading", { name: "Organization Explorer" })).toBeVisible();
    await expect(page.getByText("Demo Program Family Outreach").first()).toBeVisible();
    await expect(page.getByText("Demo Program Workforce Readiness")).toHaveCount(0);

    await expectHydratedRoute(page, {
      path: `/organization/${fixtures.inScopeNodeId}`,
      expectedText: "Organization Detail",
    });
    await expect(page.getByRole("heading", { name: "Organization Detail" })).toBeVisible();
    await expect(page.getByText("Demo Program Family Outreach").first()).toBeVisible();

    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/nodes/${fixtures.outOfScopeNodeId}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/organization/${fixtures.outOfScopeNodeId}`,
        expectedText: "Organization detail unavailable",
      });
      await expect(page.getByRole("heading", { name: "Organization detail unavailable" })).toBeVisible();
    });

    await expectHydratedRoute(page, {
      path: `/organization/${fixtures.inScopeNodeId}/edit`,
      expectedText: "Edit Organization Node",
    });
    await expect(page.getByRole("heading", { name: "Edit Organization Node" })).toBeVisible();
    await expect(page.locator("#organization-name")).toHaveValue(
      "Demo Program Family Outreach",
    );
    const programCode = page.locator("#organization-metadata-program_code");
    await expect(programCode).toBeVisible();
    await expect(programCode).toHaveValue("FO-01");
    await assertNativeRouteGuard();

    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/nodes/${fixtures.outOfScopeNodeId}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/organization/${fixtures.outOfScopeNodeId}/edit`,
        expectedText: "Organization node unavailable",
      });
      await expect(page.getByRole("heading", { name: "Organization node unavailable" })).toBeVisible();
    });

    const readableNodeTypes = await getJson<NodeTypeSummary[]>(
      fixtures.scopedManager,
      "/api/node-types",
    );
    const partnerNodeType = requireItem(
      readableNodeTypes,
      (nodeType) => nodeType.name === "Partner",
      "the seeded Partner node type should remain readable",
    );
    await expectHydratedRoute(page, {
      path: "/organization/new",
      expectedText: "Create Organization Node",
    });
    await expect(page.getByRole("heading", { name: "Create Organization Node" })).toBeVisible();
    const nodeTypeSelect = page.locator("#organization-node-type");
    await nodeTypeSelect.selectOption(partnerNodeType.id);
    await expect(nodeTypeSelect).toHaveValue(partnerNodeType.id);
    const sourceCode = page.locator("#organization-metadata-source_code");
    await expect(sourceCode).toBeVisible();
    await expect(sourceCode).toHaveJSProperty("required", true);
    await assertNativeRouteGuard();
  });

  test("form create and edit routes exercise scoped manage permission", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
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
    await expectHydratedRoute(page, {
      path: "/forms/new",
      expectedText: "Create Form",
    });
    await expect(page.getByRole("heading", { name: "Create Form" })).toBeVisible();
    await expectHydratedRoute(page, {
      path: `/forms/${created.id}/edit`,
      expectedText: "Edit Form",
    });
    await expect(page.getByRole("heading", { name: "Edit Form" })).toBeVisible();
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/admin/forms/${fixtures.outOfScopeForm.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/forms/${fixtures.outOfScopeForm.id}/edit`,
        expectedText: "Form unavailable",
      });
      await expect(page.getByRole("heading", { name: "Form unavailable" })).toBeVisible();
      await expect(page.getByRole("button", { name: "Save as Draft" })).toHaveCount(0);
    });
    await assertNativeRouteGuard();
  });

  test("workflow create detail and edit routes exercise scoped manage permission", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
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
    await expectHydratedRoute(page, {
      path: "/workflows/new",
      expectedText: "Create Workflow",
    });
    await expect(page.getByRole("heading", { name: "Create Workflow" })).toBeVisible();
    await expectHydratedRoute(page, {
      path: `/workflows/${inWorkflow.id}`,
      expectedText: `${RUN_ID} Managed Workflow`,
    });
    await expect(page.getByRole("heading", { name: `${RUN_ID} Managed Workflow` })).toBeVisible();
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/workflows/${outWorkflow.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/workflows/${outWorkflow.id}`,
        expectedText: "Workflow detail unavailable",
      });
      await expect(page.getByRole("heading", { name: "Workflow detail unavailable" })).toBeVisible();
    });
    await expectHydratedRoute(page, {
      path: `/workflows/${inWorkflow.id}/edit`,
      expectedText: "Edit Workflow",
    });
    await expect(page.getByRole("heading", { name: "Edit Workflow" })).toBeVisible();
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/workflows/${outWorkflow.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/workflows/${outWorkflow.id}/edit`,
        expectedText: "Workflow unavailable",
      });
      await expect(page.getByRole("button", { name: "Save Changes" })).toHaveCount(0);
    });
    await assertNativeRouteGuard();
  });

  test("response edit route follows ownership and delegation permissions", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
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
    await expectHydratedRoute(page, {
      path: `/responses/${draft.id}/edit`,
      expectedText: "Edit Response",
    });
    await expect(page.getByRole("heading", { level: 1, name: "Edit Response" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save Draft" })).toBeVisible();

    await signInPage(page, `${RUN_ID}-delegate@tessara.local`);
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/submissions/${draft.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/responses/${draft.id}/edit`,
        expectedText: "Response unavailable",
      });
      await expect(page.getByRole("heading", { name: "Response unavailable" })).toBeVisible();
    });
    await assertNativeRouteGuard();
  });

  test("dashboard native routes and APIs exercise scoped manage permission", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
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
    await expectHydratedRoute(page, {
      path: "/dashboards/new",
      expectedText: "Create Dashboard",
      documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
    });
    await expect(page.getByRole("heading", { level: 1, name: "Create Dashboard" })).toBeVisible();
    await expectHydratedRoute(page, {
      path: `/dashboards/${dashboard.id}`,
      expectedText: `${RUN_ID} Managed Dashboard Updated`,
      documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
    });
    await expect(
      page.getByRole("heading", { level: 1, name: `${RUN_ID} Managed Dashboard Updated` }),
    ).toBeVisible();
    await expectHydratedRoute(page, {
      path: `/dashboards/${dashboard.id}/edit`,
      expectedText: `${RUN_ID} Managed Dashboard Updated`,
      documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
    });
    await expect(
      page.getByRole("heading", { level: 1, name: `${RUN_ID} Managed Dashboard Updated` }),
    ).toBeVisible();
    await expect(page.getByText("Dashboard builder", { exact: true })).toBeVisible();
    await expectHydratedRoute(page, {
      path: `/dashboards/${dashboard.id}/view`,
      expectedText: `${RUN_ID} Managed Dashboard Updated`,
      documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
    });
    await expect(
      page.getByRole("heading", { level: 1, name: `${RUN_ID} Managed Dashboard Updated` }),
    ).toBeVisible();
    await assertNativeRouteGuard();
  });

  test("dashboard viewer preserves a redacted footprint without executing the hidden Component", async ({
    page,
  }) => {
    const dashboard = await postJson<IdResponse>(fixtures.admin, "/api/admin/dashboards", {
      name: `${RUN_ID} Redacted Dashboard`,
      description: "Dashboard redaction browser fixture.",
      visibility_node_ids: [
        fixtures.inScopeNodeId,
        ...fixtures.outOfScopeDataset.visibility_nodes.map((node) => node.node_id),
      ],
    });

    try {
      const composition = await getJson<{
        available_component_versions: Array<{
          component_version_id: string;
          component_slug: string;
          default_grid_width: number;
          default_grid_height: number;
        }>;
      }>(fixtures.admin, `/api/admin/dashboards/${dashboard.id}/composition`);
      const hiddenVersion = requireItem(
        composition.available_component_versions,
        (option) => option.component_slug === fixtures.outOfScopeVisualComponent.slug,
        "the hybrid-scope Dashboard should allow the hidden visual Component version",
      );
      await putJson(fixtures.admin, `/api/admin/dashboards/${dashboard.id}/composition`, {
        commands: [
          {
            operation: "bind",
            client_key: `${RUN_ID}-redacted-placement`,
            component_version_id: hiddenVersion.component_version_id,
            geometry: {
              grid_row: 1,
              grid_column: 1,
              grid_width: hiddenVersion.default_grid_width,
              grid_height: hiddenVersion.default_grid_height,
            },
          },
        ],
      });

      const scopedDashboard = await getJson<{
        placements: Array<{
          availability: "available" | "unavailable";
          component?: { component_slug: string };
        }>;
      }>(fixtures.scopedManager, `/api/dashboards/${dashboard.id}`);
      expect(scopedDashboard.placements).toHaveLength(1);
      expect(scopedDashboard.placements[0].availability).toBe("unavailable");
      expect(scopedDashboard.placements[0].component).toBeUndefined();

      const hiddenExecutionRequests: string[] = [];
      page.on("request", (request) => {
        const pathname = new URL(request.url()).pathname;
        if (
          request.method() === "GET" &&
          pathname.includes(`/api/components/${fixtures.outOfScopeVisualComponent.slug}/`)
        ) {
          hiddenExecutionRequests.push(pathname);
        }
      });
      await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
      await page.goto(`/dashboards/${dashboard.id}`);
      await expect(page.locator(".dashboard-placement-card.is-unavailable")).toBeVisible();
      await page.goto(`/dashboards/${dashboard.id}/view`);
      await expect(page.locator(".dashboard-redacted-placeholder")).toBeVisible();
      await page.waitForLoadState("networkidle");
      expect(hiddenExecutionRequests).toEqual([]);
    } finally {
      const response = await fixtures.admin.delete(`/api/admin/dashboards/${dashboard.id}`);
      expect(response.ok(), `Dashboard cleanup returned ${response.status()}`).toBeTruthy();
    }
  });

  test("administration user and node-type routes are admin-only", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
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
    await expectHydratedRoute(page, {
      path: "/administration/users",
      expectedText: "Users",
    });
    await expect(page.getByRole("heading", { level: 1, name: "Users" })).toBeVisible();
    await page.getByPlaceholder("Search users").fill(`${RUN_ID} Owner`);
    await expect(page.getByRole("link", { name: `${RUN_ID} Owner` })).toBeVisible();

    await expectHydratedRoute(page, {
      path: `/administration/users/${fixtures.userIds.owner}`,
      expectedText: `${RUN_ID} Owner`,
    });
    await expect(page.getByRole("heading", { name: `${RUN_ID} Owner` })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save Permissions" })).toBeVisible();

    await expectHydratedRoute(page, {
      path: `/administration/users/${fixtures.userIds.owner}/access`,
      expectedText: `${RUN_ID} Owner`,
    });
    await expect(page.getByRole("heading", { name: `${RUN_ID} Owner` })).toBeVisible();

    await expectHydratedRoute(page, {
      path: `/administration/users/${fixtures.userIds.owner}/edit`,
      expectedText: "Edit User",
    });
    await expect(page.getByRole("heading", { name: "Edit User" })).toBeVisible();
    await expect(page.getByRole("button", { name: "Save User" })).toBeVisible();

    await expectHydratedRoute(page, {
      path: "/administration/node-types",
      expectedText: "Node Types",
    });
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
    await assertNativeRouteGuard();
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

    const operations = await getJson<OperationsStatus>(fixtures.admin, "/api/operations/status");
    expect(operations.summary.open_workflow_assignment_count).toBeGreaterThanOrEqual(0);
    expect(operations.summary.dataset_attention_count).toBe(
      operations.dataset_readiness.datasets.filter((item) => item.readiness !== "Ready").length,
    );
    expect(operations.dataset_readiness.datasets.some((item) => item.dataset_id === fixtures.inScopeDataset.id)).toBe(true);
    expect(operations.dataset_readiness.datasets.some((item) => item.dataset_id === fixtures.outOfScopeDataset.id)).toBe(true);
  });

  test("scoped manager reads in-scope surfaces and is denied out-of-scope surfaces", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    const forms = await getJson<FormSummary[]>(fixtures.scopedManager, "/api/forms");
    expect(forms.some((form) => form.id === fixtures.inScopeForm.id)).toBe(true);
    expect(forms.some((form) => form.id === fixtures.outOfScopeForm.id)).toBe(false);
    await getJson(fixtures.scopedManager, `/api/forms/${fixtures.inScopeForm.id}`);
    const inScopePublishedVersion = fixtures.inScopeForm.versions.find((version) => version.status === "published");
    const outOfScopePublishedVersion = fixtures.outOfScopeForm.versions.find((version) => version.status === "published");
    expect(inScopePublishedVersion).toBeTruthy();
    expect(outOfScopePublishedVersion).toBeTruthy();
    await getJson(fixtures.scopedManager, `/api/form-versions/${inScopePublishedVersion!.id}/render`);
    await expectStatus(fixtures.scopedManager, "get", `/api/forms/${fixtures.outOfScopeForm.id}`, [
      403,
    ]);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/form-versions/${outOfScopePublishedVersion!.id}/render`,
      [403],
    );

    const datasets = await getJson<DatasetSummary[]>(fixtures.scopedManager, "/api/datasets");
    expect(datasets.some((dataset) => dataset.id === fixtures.inScopeDataset.id)).toBe(true);
    expect(datasets.some((dataset) => dataset.id === fixtures.outOfScopeDataset.id)).toBe(false);
    await getJson(fixtures.scopedManager, `/api/datasets/${fixtures.inScopeDataset.id}`);
    const table = await getJson<DatasetTable>(
      fixtures.scopedManager,
      `/api/datasets/${fixtures.inScopeDataset.id}/table`,
    );
    const adminTable = await getJson<DatasetTable>(
      fixtures.admin,
      `/api/datasets/${fixtures.inScopeDataset.id}/table`,
    );
    expect(table.rows.length).toBeGreaterThan(0);
    expect(table.rows.length).toBe(adminTable.rows.length);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/datasets/${fixtures.outOfScopeDataset.id}`,
      [403],
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/datasets/${fixtures.outOfScopeDataset.id}/table`,
      [403],
    );

    const components = await getJson<ComponentSummary[]>(fixtures.scopedManager, "/api/components");
    expect(components.some((component) => component.id === fixtures.inScopeComponent.id)).toBe(true);
    expect(components.some((component) => component.id === fixtures.outOfScopeComponent.id)).toBe(false);
    expect(components.some((component) => component.id === fixtures.inScopeVisualComponent.id)).toBe(true);
    expect(components.some((component) => component.id === fixtures.outOfScopeVisualComponent.id)).toBe(false);
    const inComponent = await getJson<ComponentDefinition>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeComponent.slug}`,
    );
    expect(inComponent.versions.length).toBeGreaterThan(0);
    const componentTable = await getJson<ComponentTable>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeComponent.slug}/table`,
    );
    expect(componentTable.materialization_state).toBe("ready");
    expect(componentTable.rows.length).toBeGreaterThan(0);
    const visualComponent = await getJson<ComponentDefinition>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeVisualComponent.slug}`,
    );
    expect(visualComponent.versions.some((version) => version.component_type === "bar")).toBe(true);
    const visual = await getJson<ComponentVisual>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeVisualComponent.slug}/bar`,
    );
    expect(visual.materialization_state).toBe("ready");
    expect(visual.component_type).toBe("bar");
    expect(visual.points.length).toBeGreaterThan(0);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${fixtures.outOfScopeComponent.slug}`,
      [404],
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${fixtures.outOfScopeComponent.slug}/table`,
      [404],
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${fixtures.outOfScopeVisualComponent.slug}`,
      [404],
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${fixtures.outOfScopeVisualComponent.slug}/bar`,
      [404],
    );
    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/admin/components/${fixtures.inScopeComponent.slug}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/components/${fixtures.inScopeComponent.slug}`,
        expectedText: fixtures.inScopeComponent.name,
      });
      await expect(
        page.getByRole("heading", { level: 1, name: fixtures.inScopeComponent.name }),
      ).toBeVisible();
    });
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/admin/components/${fixtures.inScopeComponent.slug}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/components/${fixtures.inScopeComponent.slug}/view`,
        expectedText: fixtures.inScopeComponent.name,
      });
      await expect(
        page.getByRole("heading", { level: 1, name: fixtures.inScopeComponent.name }),
      ).toBeVisible();
      await expect(page.getByRole("table")).toBeVisible();
    });
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/admin/components/${fixtures.inScopeVisualComponent.slug}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/components/${fixtures.inScopeVisualComponent.slug}/view`,
        expectedText: fixtures.inScopeVisualComponent.name,
      });
      await expect(
        page.getByRole("heading", { level: 1, name: fixtures.inScopeVisualComponent.name }),
      ).toBeVisible();
      await expect(page.locator(".component-visual-preview")).toBeVisible();
    });

    const dashboards = await getJson<DashboardSummary[]>(fixtures.scopedManager, "/api/dashboards");
    expect(dashboards.some((dashboard) => dashboard.id === fixtures.inScopeDashboard.id)).toBe(true);
    expect(dashboards.some((dashboard) => dashboard.id === fixtures.outOfScopeDashboard.id)).toBe(false);
    await getJson(fixtures.scopedManager, `/api/dashboards/${fixtures.inScopeDashboard.id}`);
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/dashboards/${fixtures.outOfScopeDashboard.id}`,
      [404],
    );

    const operations = await getJson<OperationsStatus>(fixtures.scopedManager, "/api/operations/status");
    expect(operations.dataset_readiness.datasets.some((item) => item.dataset_id === fixtures.inScopeDataset.id)).toBe(true);
    expect(operations.dataset_readiness.datasets.some((item) => item.dataset_id === fixtures.outOfScopeDataset.id)).toBe(false);
    expect(operations.workflow_assignments.every((item) => fixtures.inScopeNodeIds.has(item.node_id))).toBe(true);
    await assertNativeRouteGuard();
  });

  test("scoped component manager cannot bind or publish out-of-scope dataset major lines", async () => {
    const inScopeMajor = datasetMajor(fixtures.inScopeDataset);
    const outOfScopeMajor = datasetMajor(fixtures.outOfScopeDataset);
    const outOfScopeSlug = `${RUN_ID}-component-manage-out`;
    const manageableSlug = `${RUN_ID}-component-manage-in`;
    const componentSession = await getJson<SessionState>(fixtures.componentManager, "/api/auth/session");
    expect(componentSession.account?.capabilities).toContain("components:manage");
    const partialSession = await getJson<SessionState>(fixtures.partialComponentManager, "/api/auth/session");
    expect(partialSession.account?.capabilities).toContain("components:manage");
    expect(fixtures.inScopeDataset.visibility_nodes.length).toBeGreaterThan(1);
    await expectErrorStatus(
      fixtures.partialComponentManager,
      "post",
      "/api/admin/components",
      403,
      "forbidden",
      {
        name: `${RUN_ID} Partial Containment Component`,
        slug: `${RUN_ID}-partial-containment-component`,
        description: "Partial-overlap authoring containment fixture.",
        version: {
          dataset_id: fixtures.inScopeDataset.id,
          dataset_version_major: inScopeMajor,
          component_type: "table",
          config: tableConfig(fixtures.inScopeDataset),
        },
      },
    );
    const manageableComponent = await postJson<IdResponse>(
      fixtures.componentManager,
      "/api/admin/components",
      {
        name: `${RUN_ID} Manageable Component`,
        slug: manageableSlug,
        description: "In-scope component management permission fixture.",
        version: {
          dataset_id: fixtures.inScopeDataset.id,
          dataset_version_major: inScopeMajor,
          component_type: "table",
          config: tableConfig(fixtures.inScopeDataset),
        },
      },
    );
    const manageableComponents = await getJson<ComponentSummary[]>(
      fixtures.componentManager,
      "/api/admin/components",
    );
    expect(manageableComponents.length).toBeGreaterThan(0);
    expect(manageableComponents.some((component) => component.id === manageableComponent.id)).toBe(true);

    const bindError = await expectErrorStatus(
      fixtures.componentManager,
      "post",
      `/api/admin/components/${manageableComponent.id}/versions`,
      403,
      "forbidden",
      {
        dataset_id: fixtures.outOfScopeDataset.id,
        dataset_version_major: outOfScopeMajor,
        component_type: "table",
        config: tableConfig(fixtures.outOfScopeDataset),
      },
    );
    expect(bindError.message).toContain("components:manage");

    const validateError = await expectErrorStatus(
      fixtures.componentManager,
      "post",
      "/api/admin/components/validate",
      403,
      "forbidden",
      {
        dataset_id: fixtures.outOfScopeDataset.id,
        dataset_version_major: outOfScopeMajor,
        component_type: "table",
        config: tableConfig(fixtures.outOfScopeDataset),
      },
    );
    expect(validateError.message).toContain("components:manage");

    const outOfScopeDraft = await postJson<IdResponse>(fixtures.admin, "/api/admin/components", {
      name: `${RUN_ID} Out Component`,
      slug: outOfScopeSlug,
      description: "Out-of-scope component management permission fixture.",
      version: {
        dataset_id: fixtures.outOfScopeDataset.id,
        dataset_version_major: outOfScopeMajor,
        component_type: "table",
        config: tableConfig(fixtures.outOfScopeDataset),
      },
    });
    const outOfScopeComponent = await getJson<ComponentDefinition>(
      fixtures.admin,
      `/api/admin/components/${outOfScopeDraft.id}`,
    );
    const outVersion = outOfScopeComponent.versions[0] as { id: string };

    const publishError = await expectErrorStatus(
      fixtures.componentManager,
      "post",
      `/api/admin/components/${outOfScopeDraft.id}/versions/${outVersion.id}/publish`,
      403,
      "forbidden",
      {},
    );
    expect(publishError.message).toContain("components:manage");
  });

  test("explicit historical component table checks selected version dataset scope", async () => {
    const inScopeMajor = datasetMajor(fixtures.inScopeDataset);
    const outOfScopeMajor = datasetMajor(fixtures.outOfScopeDataset);
    const slug = `${RUN_ID}-historical-component-scope`;
    const component = await postJson<IdResponse>(fixtures.admin, "/api/admin/components", {
      name: `${RUN_ID} Historical Component Scope`,
      slug,
      description: "Historical version selected-dataset permission fixture.",
      version: {
        dataset_id: fixtures.outOfScopeDataset.id,
        dataset_version_major: outOfScopeMajor,
        component_type: "table",
        config: tableConfig(fixtures.outOfScopeDataset),
      },
    });
    const firstVersion = await getJson<ComponentDefinition>(
      fixtures.admin,
      `/api/admin/components/${component.id}`,
    );
    const hiddenHistoryVersion = firstVersion.versions[0] as { id: string };
    await postJson<IdResponse>(
      fixtures.admin,
      `/api/admin/components/${component.id}/versions/${hiddenHistoryVersion.id}/publish`,
      {},
    );

    const secondVersion = await postJson<IdResponse>(
      fixtures.admin,
      `/api/admin/components/${component.id}/versions`,
      {
        dataset_id: fixtures.inScopeDataset.id,
        dataset_version_major: inScopeMajor,
        component_type: "table",
        config: tableConfig(fixtures.inScopeDataset),
        version_note: "Switch visible published history to the in-scope dataset.",
      },
    );
    await postJson<IdResponse>(
      fixtures.admin,
      `/api/admin/components/${component.id}/versions/${secondVersion.id}/publish`,
      {},
    );

    const currentTable = await getJson<ComponentTable>(
      fixtures.scopedManager,
      `/api/components/${slug}/table`,
    );
    expect(currentTable.materialization_state).toBe("ready");
    expect(currentTable.rows.length).toBeGreaterThan(0);
    expect(await getJson<ComponentTable>(fixtures.admin, `/api/components/${slug}/versions/${hiddenHistoryVersion.id}/table`))
      .toMatchObject({ component_version_id: hiddenHistoryVersion.id });
    const hiddenHistoryError = await expectErrorStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${slug}/versions/${hiddenHistoryVersion.id}/table`,
      404,
      "not_found",
    );
    expect(hiddenHistoryError).toEqual({
      code: "not_found",
      message: "component not found",
      error: "component not found",
    });
    await getJson<ComponentTable>(
      fixtures.scopedManager,
      `/api/components/${slug}/versions/${secondVersion.id}/table`,
    );
  });

  test("dataset revision UI hides drafts from scoped readers", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    const dataset = await getJson<DatasetDefinition>(
      fixtures.admin,
      `/api/datasets/${fixtures.inScopeDataset.id}`,
    );
    const draft = await postJson<DatasetDraftRevisionResponse>(
      fixtures.admin,
      `/api/admin/datasets/${dataset.id}/draft-revision`,
      {
        name: `${dataset.name} Permission Draft`,
        slug: dataset.slug,
        grain: "submission",
        visibility_node_ids: dataset.visibility_nodes.map((node) => node.node_id),
        initial_source: dataset.initial_source,
        operations: dataset.operations,
        restriction_policy: dataset.restriction_policy ?? null,
      },
    );

    try {
      await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
      await expectHydratedRoute(page, {
        path: `/datasets/${dataset.id}/revisions`,
        expectedText: "Dataset Revisions",
      });
      await expect(page.getByRole("heading", { level: 1, name: "Dataset Revisions" })).toBeVisible();
      await expect(page.locator("tbody")).toContainText("Draft");
      await expectHydratedRoute(page, {
        path: `/datasets/${dataset.id}/revisions/${draft.revision_id}`,
        expectedText: "Dataset Revision",
      });
      await expect(page.getByRole("heading", { level: 1, name: "Dataset Revision" })).toBeVisible();
      await expect(page.locator(".route-panel__section").filter({ hasText: "Status" }).first()).toContainText("Draft");

      await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
      await expectHydratedRoute(page, {
        path: `/datasets/${dataset.id}/revisions`,
        expectedText: "Dataset Revisions",
      });
      await expect(page.getByRole("heading", { level: 1, name: "Dataset Revisions" })).toBeVisible();
      await expect(page.locator("tbody")).toContainText("Published current");
      await expect(page.locator("tbody")).not.toContainText("Draft");
      await expectStatus(
        fixtures.scopedManager,
        "get",
        `/api/datasets/${dataset.id}/revisions/${draft.revision_id}`,
        [403],
      );
      await assertNativeRouteGuard();
    } finally {
      await expectStatus(
        fixtures.admin,
        "delete",
        `/api/admin/datasets/${dataset.id}/revisions/${draft.revision_id}`,
        [200, 204, 404],
      );
    }
  });

  test("operations route is visible only to operations viewers", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
    const operations = await getJson<OperationsStatus>(fixtures.scopedManager, "/api/operations/status");
    const linkedWorkflow = requireItem(
      operations.workflow_assignments,
      (item) => item.workflow_assignment_id.length > 0,
      "operations should include a workflow assignment",
    );
    const linkedDataset = requireItem(
      operations.dataset_readiness.datasets,
      (item) => item.dataset_id.length > 0,
      "operations should include a dataset readiness row",
    );

    await expectHydratedRoute(page, {
      path: "/operations",
      expectedText: "Operations",
    });
    await expect(page.getByRole("heading", { level: 1, name: "Operations" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Workflow Assignments" })).toBeVisible();
    await expect(page.locator(`a[href="/workflows/assignments?assignment_id=${linkedWorkflow.workflow_assignment_id}"]`)).not.toHaveCount(0);
    await expect(page.locator(`a[href="/datasets/${linkedDataset.dataset_id}"]`)).not.toHaveCount(0);

    await signInPage(page, `${RUN_ID}-no-access@tessara.local`);
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: "/api/workflow-assignments/pending", count: 2 },
    ], async () => {
      await expectHydratedRoute(page, { path: "/", expectedText: "Home" });
      await expect(page.getByRole("link", { name: "Operations" })).toHaveCount(0);
    });
    await expectStatus(fixtures.noAccess, "get", "/api/operations/status", [403]);
    await assertNativeRouteGuard();
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

  test("JavaScript-disabled Core, Organization, and direct Admin routes preserve native SSR ownership", async ({
    browser,
  }) => {
    await withNoJavaScriptPage(browser, async (page) => {
      await expectNoJavaScriptRoutes(page, [
        {
          path: "/login",
          expectedText: "Welcome back",
          expectedRootMarkup: 'class="login-shell"',
          contentSelector: ".login-panel",
        },
      ]);

      await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
      const removedAdministration = await page.request.get("/administration", {
        maxRedirects: 0,
      });
      expect(removedAdministration.status()).toBe(404);
      expect(removedAdministration.headers().location).toBeUndefined();
      await expectNoJavaScriptRoutes(page, [
        { path: "/", expectedText: "Home" },
        { path: "/operations", expectedText: "Operations" },
        { path: "/organization", expectedText: "Organization Explorer" },
        { path: "/organization/new", expectedText: "Create Organization Node" },
        {
          path: `/organization/${fixtures.inScopeNodeId}`,
          expectedText: "Loading detail",
        },
        {
          path: `/organization/${fixtures.inScopeNodeId}/edit`,
          expectedText: "Edit Organization Node",
        },
        { path: "/administration/users", expectedText: "Users" },
        {
          path: `/administration/users/${fixtures.userIds.owner}`,
          expectedText: "User Detail",
        },
        {
          path: `/administration/users/${fixtures.userIds.owner}/edit`,
          expectedText: "Edit User",
        },
        {
          path: `/administration/users/${fixtures.userIds.owner}/access`,
          expectedText: "User Detail",
        },
        { path: "/administration/node-types", expectedText: "Node Types" },
        { path: "/administration/roles", expectedText: "Roles" },
        { path: "/administration/modules", expectedText: "Module Management" },
      ]);
    });
  });

  test("JavaScript-disabled Form and Workflow routes preserve native SSR ownership", async ({
    browser,
  }) => {
    const workflows = await getJson<WorkflowSummary[]>(fixtures.admin, "/api/workflows");
    const workflow = requireItem(
      workflows,
      (candidate) => candidate.id.length > 0,
      "a workflow should exist for native route proof",
    );

    await withNoJavaScriptPage(browser, async (page) => {
      await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
      await expectNoJavaScriptRoutes(page, [
        { path: "/forms", expectedText: "Forms" },
        { path: "/forms/new", expectedText: "Create Form" },
        { path: `/forms/${fixtures.inScopeForm.id}`, expectedText: "Loading form" },
        {
          path: `/forms/${fixtures.inScopeForm.id}/edit`,
          expectedText: "Edit Form",
        },
        { path: "/workflows", expectedText: "Workflows" },
        { path: "/workflows/new", expectedText: "Create Workflow" },
        {
          path: "/workflows/assignments",
          expectedText: "Workflow Assignments",
        },
        { path: `/workflows/${workflow.id}`, expectedText: "Loading workflow" },
        { path: `/workflows/${workflow.id}/edit`, expectedText: "Edit Workflow" },
      ]);
    });
  });

  test("JavaScript-disabled Response and Dataset routes preserve native SSR ownership", async ({
    browser,
  }) => {
    const editorRole = await createRole(fixtures.admin, `${RUN_ID}-native-response-editor`, [
      "submissions:read_own",
      "submissions:respond",
    ]);
    const editorEmail = `${RUN_ID}-native-response-editor@tessara.local`;
    const editor = await createUser(
      fixtures.admin,
      editorEmail,
      `${RUN_ID} Native Response Editor`,
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
    const responseDraft = await postJson<IdResponse>(
      editorContext,
      `/api/workflow-assignments/${assignment.id}/start`,
      {},
    );

    const dataset = await getJson<DatasetDefinition>(
      fixtures.admin,
      `/api/datasets/${fixtures.inScopeDataset.id}`,
    );
    const datasetDraft = await postJson<DatasetDraftRevisionResponse>(
      fixtures.admin,
      `/api/admin/datasets/${dataset.id}/draft-revision`,
      {
        name: `${dataset.name} Native Route Draft`,
        slug: dataset.slug,
        grain: "submission",
        visibility_node_ids: dataset.visibility_nodes.map((node) => node.node_id),
        initial_source: dataset.initial_source,
        operations: dataset.operations,
        restriction_policy: dataset.restriction_policy ?? null,
      },
    );

    try {
      await withNoJavaScriptPage(browser, async (page) => {
        await signInPage(page, editorEmail);
        await expectNoJavaScriptRoutes(page, [
          { path: "/responses", expectedText: "Responses" },
          { path: "/responses/new", expectedText: "Start Response" },
          {
            path: `/responses/${responseDraft.id}`,
            expectedText: "Response Detail",
          },
          {
            path: `/responses/${responseDraft.id}/edit`,
            expectedText: "Edit Response",
          },
        ]);

        await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
        await expectNoJavaScriptRoutes(page, [
          { path: "/datasets", expectedText: "Datasets" },
          { path: "/datasets/new", expectedText: "Create Dataset" },
          {
            path: `/datasets/${dataset.id}`,
            expectedText: "Loading dataset",
          },
          {
            path: `/datasets/${dataset.id}/edit`,
            expectedText: "Edit Dataset",
          },
          {
            path: `/datasets/${dataset.id}/preview`,
            expectedText: "Loading preview",
            expectedRootMarkup: 'class="dataset-preview-page"',
            contentSelector: ".dataset-preview-page",
          },
          {
            path: `/datasets/${dataset.id}/revisions`,
            expectedText: "Dataset Revisions",
          },
          {
            path: `/datasets/${dataset.id}/revisions/${datasetDraft.revision_id}`,
            expectedText: "Loading revision",
          },
          {
            path: `/datasets/${dataset.id}/revisions/${datasetDraft.revision_id}/edit`,
            expectedText: "Edit Revision",
          },
        ]);
      });
    } finally {
      await expectStatus(
        fixtures.admin,
        "delete",
        `/api/admin/datasets/${dataset.id}/revisions/${datasetDraft.revision_id}`,
        [200, 204, 404],
      );
    }
  });

  test("JavaScript-disabled Component and Dashboard routes preserve native SSR ownership", async ({
    browser,
  }) => {
    await withNoJavaScriptPage(browser, async (page) => {
      await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
      await expectNoJavaScriptRoutes(page, [
        { path: "/components", expectedText: "Loading components" },
        { path: "/components/new", expectedText: "Create Component" },
        {
          path: `/components/${fixtures.inScopeComponent.slug}`,
          expectedText: "Loading configuration",
        },
        {
          path: `/components/${fixtures.inScopeComponent.slug}/edit`,
          expectedText: "Edit Component",
        },
        {
          path: `/components/${fixtures.inScopeComponent.slug}/versions`,
          expectedText: "Loading component",
        },
        {
          path: `/components/${fixtures.inScopeComponent.slug}/view`,
          expectedText: "Loading configuration",
        },
        {
          path: "/dashboards",
          expectedText: "Dashboards",
          documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
        },
        {
          path: "/dashboards/new",
          expectedText: "Create Dashboard",
          documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
        },
        {
          path: `/dashboards/${fixtures.inScopeDashboard.id}`,
          expectedText: "Dashboard Detail",
          documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
        },
        {
          path: `/dashboards/${fixtures.inScopeDashboard.id}/edit`,
          expectedText: "Dashboard builder",
          documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
        },
        {
          path: `/dashboards/${fixtures.inScopeDashboard.id}/view`,
          expectedText: "Viewer",
          documentRootSelector: DASHBOARD_DOCUMENT_ROOT,
        },
      ]);
    });
  });
});
