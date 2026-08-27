import {
  expect,
  request,
  test,
  type APIRequestContext,
  type APIResponse,
  type Browser,
  type Page,
} from "@playwright/test";
import {
  attachNativeRouteGuard,
  expectHydratedNativeRouteDirectLoadAndRefresh,
  expectNoJavaScriptNativeRouteDirectLoadAndRefresh,
} from "./support/native-route";

const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const RUN_ID = `pw-permissions-${Date.now()}`;
const PLAYWRIGHT_ENTITY_PREFIX = "pw-permissions-";
const PASSWORD = "tessara-dev-permissions";
const COMPONENT_DOCUMENT_ROOT = "#module-content";
const COMPONENT_CONTENT_ROOT = ".components-page";
const DASHBOARD_DOCUMENT_ROOT = "#module-content";
const DATASET_DOCUMENT_ROOT = "#module-content";

type IdResponse = { id: string };
type ResponseMutationResult = { id: string; revision: number; status: string };
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
  versions: Array<{ id: string; status: string; version_label?: string | null }>;
};
type FormWorkflowLink = {
  id: string;
  name: string;
  source: string;
  current_version_id: string | null;
  current_status: string | null;
};
type FormDefinition = FormSummary & { workflows: FormWorkflowLink[] };
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
type DatasetRevisionDetail = {
  id: string;
  dataset_id: string;
  version_number: number;
  version_label: string;
  version_major: number | null;
  version_minor: number | null;
  version_patch: number | null;
  status: "draft" | "published" | "superseded";
  metadata: {
    name: string;
    slug: string;
  };
};
type DatasetTable = {
  rows: Array<{
    node_name: string;
    values: Record<string, string | null>;
  }>;
};
type DatasetReference = {
  reference: {
    installation_id: string;
    owner: {
      kind: "module_instance";
      installation_id: string;
      module_instance_id: string;
    };
    resource_type: "tessara.datasets.dataset_major_line";
    resource_id: string;
  };
};
type ComponentDatasetCatalog = {
  schema_version: number;
  datasets: Array<{ reference: DatasetReference }>;
};
type ComponentListSummary = { component_id: string; name: string; slug: string };
type ComponentSummary = {
  component_id: string;
  component_version_id: string;
  name: string;
  slug: string;
};
type ComponentVersion = {
  component_version_id: string;
  dataset_reference: DatasetReference;
  component_type: string;
  publication_state: string;
  lifecycle_state: string;
  resource_revision: number;
};
type ComponentDefinition = {
  schema_version: number;
  component_id: string;
  name: string;
  slug: string;
  versions: ComponentVersion[];
};
type ComponentMutationResponse = {
  schema_version: number;
  component_id: string;
  component_version_id: string | null;
  outcome: string;
};
type ComponentTable = {
  schema_version: number;
  component_id: string;
  component_version_id: string;
  materialization_state: string;
  component_type: "table";
  rows: Array<{ values: Record<string, string | null> }>;
};
type ComponentVisual = {
  schema_version: number;
  component_id: string;
  component_version_id: string;
  materialization_state: string;
  component_type: "bar" | "line" | "pie" | "donut" | "stat_card";
  points: Array<{ x: string; value: number }>;
};
type DashboardSummary = { id: string; name: string; visibility_nodes: VisibilityNode[] };
type DashboardDefinition = DashboardSummary & { description: string | null };
type OperationsStatus = {
  summary: {
    open_workflow_assignment_count: number;
    draft_response_count: number;
    dataset_attention_count: number | null;
  };
  workflow_assignments: Array<{ workflow_assignment_id: string; workflow_id: string; workflow_name: string; node_id: string }>;
  dataset_readiness: {
    state: "available" | "empty" | "unavailable" | "undisclosed";
    datasets: Array<{ dataset_id: string; readiness: string }>;
  };
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
type PermissionResponseSummary = {
  id: string;
  node_id: string;
};
type PermissionResponseDetail = {
  id: string;
  node_id: string;
  status: string;
  revision: number;
  form_name?: string;
  values?: Array<{
    key: string;
    field_type: string;
    required: boolean;
  }>;
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
  error?: string;
  schema_version?: number;
  retryable?: boolean;
  findings?: unknown;
};

type FrozenNativeRoute = {
  path: string;
  expectedText: string;
  additionalExpectedTexts?: string[];
  expectedLabeledValues?: Array<{ label: string; value: string }>;
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
  inScopeNode: NodeSummary;
  outOfScopeNode: NodeSummary;
  creationNodeType: NodeTypeSummary;
  inScopeNodeIds: Set<string>;
  inScopeForm: FormSummary;
  outOfScopeForm: FormSummary;
  inScopeDataset: DatasetSummary;
  outOfScopeDataset: DatasetSummary;
  inScopeDatasetReference: DatasetReference;
  outOfScopeDatasetReference: DatasetReference;
  inScopeComponent: ComponentSummary;
  outOfScopeComponent: ComponentSummary;
  inScopeVisualComponent: ComponentSummary;
  outOfScopeVisualComponent: ComponentSummary;
  inScopeDashboard: DashboardSummary;
  outOfScopeDashboard: DashboardSummary;
  workflowVersionId: string;
  inScopeAssignmentId: string;
  outOfScopeAssignmentId: string;
  ownerAssignmentId: string;
  outOfScopeOwnerAssignmentId: string;
  delegateAssignmentId: string;
};

// This acceptance file runs only inside the sprint runner's owned Reference
// topology. Core intentionally has no DELETE API for published Forms/generated
// Workflows, assignments, nodes/node types, users, or roles, so exact Compose
// project teardown is the deterministic cleanup boundary for those resources.
// Product owners with supported cleanup APIs are removed explicitly below.

let fixtures: FixtureState;
const contexts: APIRequestContext[] = [];
let mutationSequence = 0;

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

function mutationHeaders(operation: string) {
  mutationSequence += 1;
  return {
    "x-idempotency-key": `${RUN_ID}-${operation}-${mutationSequence}`,
  };
}

async function postJson<T>(context: APIRequestContext, url: string, data?: Record<string, unknown>) {
  return expectJson<T>(
    await context.post(url, {
      headers: mutationHeaders("post"),
      ...(data ? { data } : {}),
    }),
  );
}

async function putJson<T>(context: APIRequestContext, url: string, data?: Record<string, unknown>) {
  return expectJson<T>(
    await context.put(url, {
      headers: mutationHeaders("put"),
      ...(data ? { data } : {}),
    }),
  );
}

async function expectStatus(
  context: APIRequestContext,
  method: "get" | "post" | "put" | "delete",
  url: string,
  statuses: number[],
  data?: Record<string, unknown>,
) {
  const response = await context[method](url, {
    ...(method === "get" ? {} : { headers: mutationHeaders(method) }),
    ...(data ? { data } : {}),
  });
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
  if (body.error !== undefined) expect(body.error).toBe(body.message);
  return body;
}

function expectComponentError(
  body: ApiErrorBody,
  code: string,
  message: string,
) {
  expect(body).toEqual({
    schema_version: 1,
    code,
    message,
    retryable: false,
    findings: null,
  });
}

function expectComponentForbidden(body: ApiErrorBody) {
  expectComponentError(body, "component.forbidden", "Forbidden");
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
        for (const expectedText of route.additionalExpectedTexts ?? []) {
          await expect(
            routeContent
              .getByText(expectedText, { exact: true })
              .filter({ visible: true })
              .first(),
          ).toBeVisible();
        }
        for (const expectedValue of route.expectedLabeledValues ?? []) {
          await expect(
            routeContent.getByLabel(expectedValue.label, { exact: true }),
          ).toHaveValue(expectedValue.value);
        }
      },
    });
  }
}

function datasetRevisionVersion(revision: DatasetRevisionDetail) {
  if (
    revision.version_major !== null &&
    revision.version_minor !== null &&
    revision.version_patch !== null
  ) {
    return `v${revision.version_major}.${revision.version_minor}.${revision.version_patch}`;
  }
  return `Revision ${revision.version_number}`;
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

async function createPermissionNodes(admin: APIRequestContext) {
  const nodes = await getJson<NodeSummary[]>(admin, "/api/nodes");
  const referenceRoot = requireItem(
    nodes,
    (node) => node.name === "Reference Organization" && node.parent_node_id === null,
    "the receipt-backed Reference Organization should exist",
  );
  const disjointRoot = requireItem(
    nodes,
    (node) => node.name === "Disjoint Organization" && node.parent_node_id === null,
    "the receipt-backed Disjoint Organization should exist",
  );
  expect(disjointRoot.node_type_id).toBe(referenceRoot.node_type_id);

  const nodeType = await postJson<IdResponse>(admin, "/api/admin/node-types", {
    name: `${RUN_ID} Permission Scope`,
    slug: `${RUN_ID}-permission-scope`,
    plural_label: `${RUN_ID} Permission Scopes`,
    parent_node_type_ids: [referenceRoot.node_type_id],
    child_node_type_ids: [],
  });
  await postJson<IdResponse>(admin, "/api/admin/node-metadata-fields", {
    node_type_id: nodeType.id,
    key: "source_code",
    label: "Source Code",
    field_type: "text",
    required: true,
  });
  const rootCreationNodeType = await postJson<IdResponse>(
    admin,
    "/api/admin/node-types",
    {
      name: `${RUN_ID} Root Creation Scope`,
      slug: `${RUN_ID}-root-creation-scope`,
      plural_label: `${RUN_ID} Root Creation Scopes`,
      parent_node_type_ids: [],
      child_node_type_ids: [],
    },
  );
  await postJson<IdResponse>(admin, "/api/admin/node-metadata-fields", {
    node_type_id: rootCreationNodeType.id,
    key: "source_code",
    label: "Source Code",
    field_type: "text",
    required: true,
  });

  const inScope = await postJson<IdResponse>(admin, "/api/admin/nodes", {
    node_type_id: nodeType.id,
    parent_node_id: referenceRoot.id,
    name: `${RUN_ID} In Scope`,
    metadata: { source_code: `${RUN_ID}-IN` },
  });
  const outOfScope = await postJson<IdResponse>(admin, "/api/admin/nodes", {
    node_type_id: nodeType.id,
    parent_node_id: disjointRoot.id,
    name: `${RUN_ID} Out Of Scope`,
    metadata: { source_code: `${RUN_ID}-OUT` },
  });
  const containment = await postJson<IdResponse>(admin, "/api/admin/nodes", {
    node_type_id: nodeType.id,
    parent_node_id: referenceRoot.id,
    name: `${RUN_ID} Containment Scope`,
    metadata: { source_code: `${RUN_ID}-CONTAINMENT` },
  });
  const readableNodeTypes = await getJson<NodeTypeSummary[]>(admin, "/api/node-types");
  const fixtureNodeType = requireItem(
    readableNodeTypes,
    (candidate) => candidate.id === nodeType.id,
    "the scenario-owned permission node type should be readable",
  );
  const creationNodeType = requireItem(
    readableNodeTypes,
    (candidate) => candidate.id === rootCreationNodeType.id,
    "the scenario-owned root creation node type should be readable",
  );
  expect(creationNodeType.is_root_type).toBe(true);
  const inScopeNode = await getJson<NodeSummary>(admin, `/api/nodes/${inScope.id}`);
  const outOfScopeNode = await getJson<NodeSummary>(admin, `/api/nodes/${outOfScope.id}`);
  const containmentNode = await getJson<NodeSummary>(
    admin,
    `/api/nodes/${containment.id}`,
  );
  expect(inScopeNode.parent_node_id).toBe(referenceRoot.id);
  expect(outOfScopeNode.parent_node_id).toBe(disjointRoot.id);
  expect(containmentNode.parent_node_id).toBe(referenceRoot.id);
  return {
    fixtureNodeType,
    creationNodeType,
    inScopeNode,
    outOfScopeNode,
    containmentNode,
  };
}

async function createPublishedPermissionForm(
  admin: APIRequestContext,
  fixtureNodeType: NodeTypeSummary,
  nameSuffix: string,
  visibilityNodeIds: string[],
) {
  const slug = `${RUN_ID}-${nameSuffix.toLowerCase().replaceAll(" ", "-")}`;
  const form = await postJson<IdResponse>(admin, "/api/admin/forms", {
    name: `${RUN_ID} ${nameSuffix}`,
    slug,
    scope_node_type_id: fixtureNodeType.id,
    visibility_node_ids: visibilityNodeIds,
  });
  const version = await postJson<IdResponse>(
    admin,
    `/api/admin/forms/${form.id}/versions`,
    {},
  );
  const section = await postJson<IdResponse>(
    admin,
    `/api/admin/form-versions/${version.id}/sections`,
    {
      title: "Permission Evidence",
      description: "Scenario-owned authorization evidence.",
      position: 0,
    },
  );
  await postJson<IdResponse>(
    admin,
    `/api/admin/form-versions/${version.id}/fields`,
    {
      section_id: section.id,
      key: "evidence",
      label: "Evidence",
      field_type: "text",
      required: true,
      position: 0,
      grid_row: 1,
      grid_column: 1,
      grid_width: 12,
      grid_height: 2,
    },
  );
  await postJson<Record<string, unknown>>(
    admin,
    `/api/admin/form-versions/${version.id}/publish`,
    {},
  );
  const definition = await getJson<FormDefinition>(admin, `/api/forms/${form.id}`);
  const publishedVersion = requireItem(
    definition.versions,
    (candidate) => candidate.id === version.id && candidate.status === "published",
    `${definition.name} should expose its scenario-owned published version`,
  );
  const workflow = requireItem(
    definition.workflows,
    (candidate) =>
      candidate.source === "generated_form" &&
      candidate.current_status === "published" &&
      candidate.current_version_id !== null,
    `${definition.name} should expose its generated workflow candidate source`,
  );
  return {
    definition,
    formVersionId: publishedVersion.id,
    workflowVersionId: workflow.current_version_id!,
  };
}

async function submitPermissionResponse(
  context: APIRequestContext,
  assignmentId: string,
) {
  const submission = await postJson<ResponseMutationResult>(
    context,
    "/api/responses",
    { workflow_assignment_id: assignmentId },
  );
  const detail = await getJson<PermissionResponseDetail>(
    context,
    `/api/responses/${submission.id}`,
  );
  const requiredValues = Object.fromEntries(
    (detail.values ?? [])
      .filter((field) => field.required)
      .map((field) => [field.key, `Evidence for ${RUN_ID}`]),
  );
  expect(Object.keys(requiredValues).length).toBeGreaterThan(0);
  const saved = await putJson<ResponseMutationResult>(
    context,
    `/api/responses/${submission.id}/values`,
    {
      expected_revision: detail.revision,
      values: requiredValues,
    },
  );
  await postJson<ResponseMutationResult>(
    context,
    `/api/responses/${submission.id}/submit`,
    { expected_revision: saved.revision },
  );
}

async function createPermissionDataset(
  admin: APIRequestContext,
  nameSuffix: string,
  visibilityNodeIds: string[],
  form: FormSummary,
  formVersionId: string,
) {
  const slug = `${RUN_ID}-${nameSuffix.toLowerCase().replaceAll(" ", "-")}`;
  const created = await postJson<IdResponse>(admin, "/api/admin/datasets", {
    name: `${RUN_ID} ${nameSuffix}`,
    slug,
    grain: "submission",
    version_label: "Initial permission fixture",
    visibility_node_ids: visibilityNodeIds,
    initial_source: {
      kind: "form",
      alias: "fixture",
      form_id: form.id,
      form_version_id: formVersionId,
    },
    operations: [
      {
        kind: "projection",
        fields: [
          {
            key: "fixture__evidence",
            label: "Evidence",
            input_field_key: "fixture__evidence",
            position: 0,
          },
        ],
        position: 0,
      },
    ],
    restriction_policy: null,
  });
  const dataset = await getJson<DatasetDefinition>(admin, `/api/datasets/${created.id}`);
  expect(dataset.slug).toBe(slug);
  expect(dataset.output_fields.map((field) => field.key)).toContain("fixture__evidence");
  return dataset;
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

function componentDatasetReference(
  catalog: ComponentDatasetCatalog,
  dataset: DatasetSummary,
) {
  expect(catalog.schema_version).toBe(2);
  const resourceId = `${dataset.id}@${datasetMajor(dataset)}`;
  const option = requireItem(
    catalog.datasets,
    (candidate) => candidate.reference.reference.resource_id === resourceId,
    `Component authoring catalog should expose Dataset major line ${resourceId}`,
  );
  expect(option.reference.reference.resource_type).toBe(
    "tessara.datasets.dataset_major_line",
  );
  expect(option.reference.reference.owner).toMatchObject({
    kind: "module_instance",
    installation_id: option.reference.reference.installation_id,
  });
  expect(
    option.reference.reference.owner.module_instance_id,
    "Dataset major-line reference must name its owning Dataset Module Instance",
  ).toMatch(/^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/);
  return option.reference;
}

function componentVersionInput(
  datasetReference: DatasetReference,
  componentType: "table" | "bar",
  config: Record<string, unknown>,
  versionNote: string,
) {
  return {
    dataset_reference: datasetReference,
    component_type: componentType,
    config,
    version_note: versionNote,
  };
}

function requireComponentVersionId(response: ComponentMutationResponse) {
  expect(response.schema_version).toBe(1);
  expect(response.component_version_id).toBeTruthy();
  return response.component_version_id!;
}

function tableConfig(dataset: DatasetSummary) {
  const firstField = dataset.output_fields.find((field) => !field.key.startsWith("__"))?.key;
  expect(firstField, `dataset ${dataset.name} should expose output fields`).toBeTruthy();
  return {
    visible_columns: [firstField],
  };
}

function visualConfig(dataset: DatasetSummary) {
  const firstField = dataset.output_fields.find((field) => !field.key.startsWith("__"))?.key;
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
  datasetReference: DatasetReference,
  slug: string,
  name: string,
) {
  const component = await postJson<ComponentDefinition>(admin, "/api/admin/components", {
    schema_version: 1,
    name,
    slug,
    description: "Visual component permission fixture.",
    version: componentVersionInput(
      datasetReference,
      "bar",
      visualConfig(dataset),
      "Initial visual permission fixture",
    ),
  });
  const detail = await getJson<ComponentDefinition>(admin, `/api/admin/components/${slug}`);
  const version = detail.versions[0];
  expect(version.component_type).toBe("bar");
  await postJson<ComponentMutationResponse>(
    admin,
    `/api/admin/components/${component.component_id}/versions/${version.component_version_id}/publish`,
  );
  return {
    component_id: component.component_id,
    component_version_id: version.component_version_id,
    name,
    slug,
  };
}

async function createPublishedTableComponent(
  admin: APIRequestContext,
  dataset: DatasetSummary,
  datasetReference: DatasetReference,
  slug: string,
  name: string,
) {
  const component = await postJson<ComponentDefinition>(admin, "/api/admin/components", {
    schema_version: 1,
    name,
    slug,
    description: "Table component permission fixture.",
    version: componentVersionInput(
      datasetReference,
      "table",
      tableConfig(dataset),
      "Initial table permission fixture",
    ),
  });
  const detail = await getJson<ComponentDefinition>(admin, `/api/admin/components/${slug}`);
  const version = detail.versions[0];
  expect(version.component_type).toBe("table");
  await postJson<ComponentMutationResponse>(
    admin,
    `/api/admin/components/${component.component_id}/versions/${version.component_version_id}/publish`,
  );
  return {
    component_id: component.component_id,
    component_version_id: version.component_version_id,
    name,
    slug,
  };
}

async function createAssignmentFor(
  admin: APIRequestContext,
  candidates: WorkflowAssignmentCandidate[],
  nodeId: string,
  accountId: string,
  workflowVersionId?: string,
) {
  const candidate = requireItem(
    candidates,
    (item) =>
      item.node_id === nodeId &&
      (workflowVersionId === undefined || item.workflow_version_id === workflowVersionId),
    `workflow candidate should exist for node ${nodeId}${
      workflowVersionId ? ` and workflow version ${workflowVersionId}` : ""
    }`,
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
  await cleanupSupportedPermissionFixtures(admin);
  const {
    fixtureNodeType,
    creationNodeType,
    inScopeNode,
    outOfScopeNode,
    containmentNode,
  } = await createPermissionNodes(admin);

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
  const dataWriter = await newContext();
  await signIn(scopedManager, `${RUN_ID}-scoped-manager@tessara.local`, PASSWORD);
  await signIn(componentManager, `${RUN_ID}-component-manager@tessara.local`, PASSWORD);
  await signIn(partialComponentManager, `${RUN_ID}-partial-component-manager@tessara.local`, PASSWORD);
  await signIn(owner, `${RUN_ID}-owner@tessara.local`, PASSWORD);
  await signIn(outOfScopeOwner, `${RUN_ID}-out-owner@tessara.local`, PASSWORD);
  await signIn(delegate, `${RUN_ID}-delegate@tessara.local`, PASSWORD);
  await signIn(delegator, `${RUN_ID}-delegator@tessara.local`, PASSWORD);
  await signIn(noAccess, `${RUN_ID}-no-access@tessara.local`, PASSWORD);
  await signIn(dataWriter, `${RUN_ID}-global@tessara.local`, PASSWORD);

  const scopedNodes = await getJson<NodeSummary[]>(
    scopedManager,
    `/api/nodes?q=${encodeURIComponent(RUN_ID)}`,
  );
  const inScopeNodeIds = new Set(scopedNodes.map((node) => node.id));
  expect(inScopeNodeIds.has(inScopeNode.id)).toBe(true);
  expect(inScopeNodeIds.has(outOfScopeNode.id)).toBe(false);

  const inScopeFormFixture = await createPublishedPermissionForm(
    admin,
    fixtureNodeType,
    "In Scope Form",
    [inScopeNode.id, containmentNode.id],
  );
  const outOfScopeFormFixture = await createPublishedPermissionForm(
    admin,
    fixtureNodeType,
    "Out Of Scope Form",
    [outOfScopeNode.id],
  );
  const inScopeForm = inScopeFormFixture.definition;
  const outOfScopeForm = outOfScopeFormFixture.definition;
  expect(overlaps(inScopeForm.visibility_nodes, inScopeNodeIds)).toBe(true);
  expect(disjointFrom(outOfScopeForm.visibility_nodes, inScopeNodeIds)).toBe(true);

  const adminCandidates = await getJson<WorkflowAssignmentCandidate[]>(
    admin,
    "/api/workflow-assignment-candidates",
  );
  expect(
    adminCandidates.some(
      (item) =>
        item.node_id === inScopeNode.id &&
        item.workflow_version_id === inScopeFormFixture.workflowVersionId,
    ),
  ).toBe(true);
  expect(
    adminCandidates.some(
      (item) =>
        item.node_id === outOfScopeNode.id &&
        item.workflow_version_id === inScopeFormFixture.workflowVersionId,
    ),
  ).toBe(true);

  const inScopeAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.noAccess.id,
    inScopeFormFixture.workflowVersionId,
  );
  const outOfScopeAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    outOfScopeNode.id,
    users.outOfScopeOwner.id,
    inScopeFormFixture.workflowVersionId,
  );
  const ownerAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.owner.id,
    inScopeFormFixture.workflowVersionId,
  );
  const outOfScopeOwnerAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    outOfScopeNode.id,
    users.scopedManager.id,
    inScopeFormFixture.workflowVersionId,
  );
  const delegateAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.delegate.id,
    inScopeFormFixture.workflowVersionId,
  );

  const dataAssignment = await createAssignmentFor(
    admin,
    adminCandidates,
    inScopeNode.id,
    users.global.id,
    inScopeFormFixture.workflowVersionId,
  );
  await submitPermissionResponse(dataWriter, dataAssignment.id);

  const inScopeDataset = await createPermissionDataset(
    admin,
    "In Scope Dataset",
    [inScopeNode.id, containmentNode.id],
    inScopeForm,
    inScopeFormFixture.formVersionId,
  );
  expect(overlaps(inScopeDataset.visibility_nodes, inScopeNodeIds)).toBe(true);
  expect(
    inScopeDataset.visibility_nodes.some((node) => !inScopeNodeIds.has(node.node_id)),
  ).toBe(true);
  await assignAccess(
    admin,
    users.componentManager.id,
    inScopeDataset.visibility_nodes.map((node) => node.node_id),
  );
  const componentManagerNodeIds = new Set(
    inScopeDataset.visibility_nodes.map((node) => node.node_id),
  );
  const outOfScopeDataset = await createPermissionDataset(
    admin,
    "Out Of Scope Dataset",
    [outOfScopeNode.id],
    outOfScopeForm,
    outOfScopeFormFixture.formVersionId,
  );
  expect(disjointFrom(outOfScopeDataset.visibility_nodes, inScopeNodeIds)).toBe(true);
  expect(disjointFrom(outOfScopeDataset.visibility_nodes, componentManagerNodeIds)).toBe(true);
  const componentDatasetCatalog = await getJson<ComponentDatasetCatalog>(
    admin,
    "/api/admin/components/datasets",
  );
  const inScopeDatasetReference = componentDatasetReference(
    componentDatasetCatalog,
    inScopeDataset,
  );
  const outOfScopeDatasetReference = componentDatasetReference(
    componentDatasetCatalog,
    outOfScopeDataset,
  );
  const inScopeComponent = await createPublishedTableComponent(
    admin,
    inScopeDataset,
    inScopeDatasetReference,
    `${RUN_ID}-visible-table-component`,
    `${RUN_ID} Visible Table Component`,
  );
  const outOfScopeComponent = await createPublishedTableComponent(
    admin,
    outOfScopeDataset,
    outOfScopeDatasetReference,
    `${RUN_ID}-hidden-table-component`,
    `${RUN_ID} Hidden Table Component`,
  );
  const adminComponents = await getJson<ComponentListSummary[]>(admin, "/api/components");
  const scopedComponents = await getJson<ComponentListSummary[]>(scopedManager, "/api/components");
  expect(adminComponents.some((component) => component.component_id === inScopeComponent.component_id)).toBe(true);
  expect(adminComponents.some((component) => component.component_id === outOfScopeComponent.component_id)).toBe(true);
  expect(scopedComponents.some((component) => component.component_id === inScopeComponent.component_id)).toBe(true);
  expect(scopedComponents.some((component) => component.component_id === outOfScopeComponent.component_id)).toBe(false);
  const inScopeVisualComponent = await createPublishedVisualComponent(
    admin,
    inScopeDataset,
    inScopeDatasetReference,
    `${RUN_ID}-visible-bar-component`,
    `${RUN_ID} Visible Bar Component`,
  );
  const outOfScopeVisualComponent = await createPublishedVisualComponent(
    admin,
    outOfScopeDataset,
    outOfScopeDatasetReference,
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
    inScopeNode,
    outOfScopeNode,
    creationNodeType,
    inScopeNodeIds,
    inScopeForm,
    outOfScopeForm,
    inScopeDataset,
    outOfScopeDataset,
    inScopeDatasetReference,
    outOfScopeDatasetReference,
    inScopeComponent,
    outOfScopeComponent,
    inScopeVisualComponent,
    outOfScopeVisualComponent,
    inScopeDashboard,
    outOfScopeDashboard,
    workflowVersionId: inScopeFormFixture.workflowVersionId,
    inScopeAssignmentId: inScopeAssignment.id,
    outOfScopeAssignmentId: outOfScopeAssignment.id,
    ownerAssignmentId: ownerAssignment.id,
    outOfScopeOwnerAssignmentId: outOfScopeOwnerAssignment.id,
    delegateAssignmentId: delegateAssignment.id,
  };
}

async function cleanupPlaywrightDashboards(admin: APIRequestContext) {
  const dashboards = await getJson<DashboardSummary[]>(admin, "/api/dashboards");
  for (const dashboard of dashboards.filter((candidate) =>
    candidate.name.startsWith(PLAYWRIGHT_ENTITY_PREFIX),
  )) {
    const response = await admin.delete(`/api/admin/dashboards/${dashboard.id}`, {
      headers: mutationHeaders("delete-dashboard"),
    });
    expect(
      response.ok(),
      `Dashboard cleanup for ${dashboard.id} returned ${response.status()}`,
    ).toBeTruthy();
  }
}

async function cleanupPlaywrightComponents(admin: APIRequestContext) {
  const components = await getJson<ComponentListSummary[]>(
    admin,
    "/api/admin/components",
  );
  for (const component of components.filter((candidate) =>
    candidate.slug.startsWith(PLAYWRIGHT_ENTITY_PREFIX),
  )) {
    const definition = await getJson<ComponentDefinition>(
      admin,
      `/api/admin/components/${component.component_id}`,
    );
    for (const version of definition.versions) {
      if (version.publication_state === "draft") {
        await expectStatus(
          admin,
          "delete",
          `/api/admin/components/${component.component_id}/versions/${version.component_version_id}`,
          [200],
        );
        continue;
      }
      let resourceRevision = version.resource_revision;
      if (version.lifecycle_state === "active" || version.lifecycle_state === "inactive") {
        await postJson<ComponentMutationResponse>(
          admin,
          `/api/admin/components/${component.component_id}/versions/${version.component_version_id}/lifecycle`,
          {
            schema_version: 1,
            action: "archive",
            expected_resource_revision: resourceRevision,
          },
        );
        resourceRevision += 1;
      }
      if (version.lifecycle_state !== "tombstoned") {
        await postJson<ComponentMutationResponse>(
          admin,
          `/api/admin/components/${component.component_id}/versions/${version.component_version_id}/lifecycle`,
          {
            schema_version: 1,
            action: "tombstone",
            expected_resource_revision: resourceRevision,
          },
        );
      }
    }
  }
}

async function cleanupPlaywrightDatasets(admin: APIRequestContext) {
  const datasets = await getJson<DatasetSummary[]>(admin, "/api/datasets");
  for (const dataset of datasets.filter((candidate) =>
    candidate.slug?.startsWith(PLAYWRIGHT_ENTITY_PREFIX),
  )) {
    await expectStatus(
      admin,
      "delete",
      `/api/admin/datasets/${dataset.id}`,
      [200, 204, 404],
    );
  }
}

async function cleanupSupportedPermissionFixtures(admin: APIRequestContext) {
  await cleanupPlaywrightDashboards(admin);
  await cleanupPlaywrightComponents(admin);
  await cleanupPlaywrightDatasets(admin);
}

test.describe.serial("capability + scope + ownership permissions", () => {
  test.beforeAll(async () => {
    fixtures = await setupFixtures();
  });

  test.afterAll(async () => {
    try {
      if (fixtures) {
        await cleanupSupportedPermissionFixtures(fixtures.admin);
      }
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
      "/api/responses",
      "/api/operations/status",
      "/api/datasets",
      `/api/datasets/${fixtures.inScopeDataset.id}/table`,
      "/api/components",
      "/api/dashboards",
    ]) {
      await expectStatus(fixtures.noAccess, "get", url, [403]);
    }
    await expectStatus(
      fixtures.noAccess,
      "get",
      "/api/workflow-assignments/pending",
      [405],
    );
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
    await expect(page.getByText(fixtures.inScopeNode.name).first()).toBeVisible();
    await expect(page.getByText(fixtures.outOfScopeNode.name)).toHaveCount(0);

    await expectHydratedRoute(page, {
      path: `/organization/${fixtures.inScopeNode.id}`,
      expectedText: "Organization Detail",
    });
    await expect(page.getByRole("heading", { name: "Organization Detail" })).toBeVisible();
    await expect(page.getByText(fixtures.inScopeNode.name).first()).toBeVisible();

    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/nodes/${fixtures.outOfScopeNode.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/organization/${fixtures.outOfScopeNode.id}`,
        expectedText: "Organization detail unavailable",
      });
      await expect(page.getByRole("heading", { name: "Organization detail unavailable" })).toBeVisible();
    });

    await expectHydratedRoute(page, {
      path: `/organization/${fixtures.inScopeNode.id}/edit`,
      expectedText: "Edit Organization Node",
    });
    await expect(page.getByRole("heading", { name: "Edit Organization Node" })).toBeVisible();
    await expect(page.locator("#organization-name")).toHaveValue(
      fixtures.inScopeNode.name,
    );
    const editSourceCode = page.locator("#organization-metadata-source_code");
    await expect(editSourceCode).toBeVisible();
    await expect(editSourceCode).toHaveValue(`${RUN_ID}-IN`);
    await assertNativeRouteGuard();

    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      { path: `/api/nodes/${fixtures.outOfScopeNode.id}`, count: 2 },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/organization/${fixtures.outOfScopeNode.id}/edit`,
        expectedText: "Organization node unavailable",
      });
      await expect(page.getByRole("heading", { name: "Organization node unavailable" })).toBeVisible();
    });

    const readableNodeTypes = await getJson<NodeTypeSummary[]>(
      fixtures.scopedManager,
      "/api/node-types",
    );
    const permissionNodeType = requireItem(
      readableNodeTypes,
      (nodeType) => nodeType.id === fixtures.creationNodeType.id,
      "the scenario-owned root creation node type should remain readable",
    );
    await expectHydratedRoute(page, {
      path: "/organization/new",
      expectedText: "Create Organization Node",
    });
    await expect(page.getByRole("heading", { name: "Create Organization Node" })).toBeVisible();
    const nodeTypeSelect = page.locator("#organization-node-type");
    await nodeTypeSelect.selectOption(permissionNodeType.id);
    await expect(nodeTypeSelect).toHaveValue(permissionNodeType.id);
    const createSourceCode = page.locator("#organization-metadata-source_code");
    await expect(createSourceCode).toBeVisible();
    await expect(createSourceCode).toHaveJSProperty("required", true);
    await assertNativeRouteGuard();
  });

  test("form create and edit routes exercise scoped manage permission", async ({ page }) => {
    const assertNativeRouteGuard = attachNativeRouteGuard(page);
    const formSlug = `${RUN_ID}-managed-form`;
    const created = await postJson<IdResponse>(fixtures.scopedManager, "/api/admin/forms", {
      name: `${RUN_ID} Managed Form`,
      slug: formSlug,
      scope_node_type_id: null,
      visibility_node_ids: [fixtures.inScopeNode.id],
    });
    await getJson(fixtures.scopedManager, `/api/forms/${created.id}`);

    await expectStatus(fixtures.scopedManager, "post", "/api/admin/forms", [403], {
      name: `${RUN_ID} Out Form`,
      slug: `${RUN_ID}-out-form`,
      scope_node_type_id: null,
      visibility_node_ids: [fixtures.outOfScopeNode.id],
    });

    await putJson<IdResponse>(fixtures.scopedManager, `/api/admin/forms/${created.id}`, {
      name: `${RUN_ID} Managed Form Updated`,
      slug: formSlug,
      scope_node_type_id: null,
      visibility_node_ids: [fixtures.inScopeNode.id],
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
        visibility_node_ids: [fixtures.outOfScopeNode.id],
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
      available_node_ids: [fixtures.inScopeNode.id],
    });
    await getJson<WorkflowDefinition>(fixtures.scopedManager, `/api/workflows/${inWorkflow.id}`);

    await expectStatus(fixtures.scopedManager, "post", "/api/workflows", [403], {
      name: `${RUN_ID} Out Workflow`,
      slug: `${RUN_ID}-out-workflow`,
      description: "Out-of-scope workflow permission fixture.",
      available_node_ids: [fixtures.outOfScopeNode.id],
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
        available_node_ids: [fixtures.outOfScopeNode.id],
      },
    );

    const outWorkflow = await postJson<IdResponse>(fixtures.admin, "/api/workflows", {
      name: `${RUN_ID} Admin Out Workflow`,
      slug: `${RUN_ID}-admin-out-workflow`,
      description: "Out-of-scope workflow permission fixture.",
      available_node_ids: [fixtures.outOfScopeNode.id],
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
      fixtures.inScopeNode.id,
      editor.id,
      fixtures.workflowVersionId,
    );
    const draft = await postJson<ResponseMutationResult>(
      editorContext,
      "/api/responses",
      { workflow_assignment_id: assignment.id },
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
      { path: `/api/responses/${draft.id}`, count: 2, status: 404 },
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
      visibility_node_ids: [fixtures.inScopeNode.id],
    });
    await getJson<DashboardDefinition>(fixtures.scopedManager, `/api/dashboards/${dashboard.id}`);
    await expectStatus(fixtures.scopedManager, "post", "/api/admin/dashboards", [403], {
      name: `${RUN_ID} Out Dashboard Denied`,
      description: "Should be rejected.",
      visibility_node_ids: [fixtures.outOfScopeNode.id],
    });
    await putJson<IdResponse>(fixtures.scopedManager, `/api/admin/dashboards/${dashboard.id}`, {
      name: `${RUN_ID} Managed Dashboard Updated`,
      description: "Scoped dashboard permission fixture updated.",
      visibility_node_ids: [fixtures.inScopeNode.id],
    });
    await expectStatus(
      fixtures.scopedManager,
      "put",
      `/api/admin/dashboards/${dashboard.id}`,
      [403],
      {
        name: `${RUN_ID} Managed Dashboard Out`,
        description: "Should be rejected.",
        visibility_node_ids: [fixtures.outOfScopeNode.id],
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

  test("dashboard composition redacts reader-hidden Components before viewer execution", async ({
    page,
  }) => {
    const dashboard = await postJson<IdResponse>(fixtures.admin, "/api/admin/dashboards", {
      name: `${RUN_ID} Redacted Dashboard`,
      description: "Dashboard redaction browser fixture.",
      visibility_node_ids: [
        fixtures.inScopeNode.id,
        ...fixtures.outOfScopeDataset.visibility_nodes.map((node) => node.node_id),
      ],
    });

    try {
      const composition = await getJson<{
        available_component_versions: Array<{
          component_reference: unknown;
          component_version_id: string;
          component_slug: string;
          default_grid_width: number;
          default_grid_height: number;
        }>;
      }>(fixtures.admin, `/api/admin/dashboards/${dashboard.id}/composition`);
      const hiddenOption = composition.available_component_versions.find(
        (option) => option.component_slug === fixtures.outOfScopeVisualComponent.slug,
      );
      expect(
        hiddenOption,
        "an authorized manager may bind a Component contained by the Dashboard's complete scope",
      ).toBeTruthy();
      await putJson(fixtures.admin, `/api/admin/dashboards/${dashboard.id}/composition`, {
        commands: [
          {
            operation: "bind",
            client_key: `${RUN_ID}-redacted-placement`,
            component_reference: hiddenOption!.component_reference,
            geometry: {
              grid_row: 1,
              grid_column: 1,
              grid_width: hiddenOption!.default_grid_width,
              grid_height: hiddenOption!.default_grid_height,
            },
          },
        ],
      });

      const scopedDashboard = await getJson<{
        placements: Array<{
          placement_id: string;
          availability: "available" | "unavailable";
          component?: { component_slug: string };
        }>;
      }>(fixtures.scopedManager, `/api/dashboards/${dashboard.id}`);
      expect(scopedDashboard.placements).toHaveLength(1);
      expect(scopedDashboard.placements[0]).toMatchObject({ availability: "unavailable" });
      expect(scopedDashboard.placements[0].component).toBeUndefined();

      const hiddenExecutionRequests: string[] = [];
      const hiddenPlacementRenderPath = new RegExp(
        `^/api/dashboards/${dashboard.id}/placements/${scopedDashboard.placements[0].placement_id}/render/`,
      );
      page.on("request", (request) => {
        const pathname = new URL(request.url()).pathname;
        if (
          request.method() === "GET" &&
          (pathname.includes(`/api/components/${fixtures.outOfScopeVisualComponent.slug}/`) ||
            hiddenPlacementRenderPath.test(pathname))
        ) {
          hiddenExecutionRequests.push(pathname);
        }
      });
      await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
      await page.goto(`/dashboards/${dashboard.id}`);
      await expect(page.getByRole("heading", { level: 1, name: `${RUN_ID} Redacted Dashboard` })).toBeVisible();
      await expect(page.locator(".dashboard-placement-card.is-unavailable")).toHaveCount(1);
      await page.goto(`/dashboards/${dashboard.id}/view`);
      await expect(page.getByRole("heading", { level: 1, name: `${RUN_ID} Redacted Dashboard` })).toBeVisible();
      await expect(page.locator(".dashboard-redacted-placeholder")).toHaveCount(1);
      await page.waitForLoadState("networkidle");
      expect(hiddenExecutionRequests).toEqual([]);
    } finally {
      const response = await fixtures.admin.delete(`/api/admin/dashboards/${dashboard.id}`, {
        headers: mutationHeaders("delete-dashboard"),
      });
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
    expect(operations.dataset_readiness.state).toBe("available");
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
      [404],
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/datasets/${fixtures.outOfScopeDataset.id}/table`,
      [404],
    );

    const components = await getJson<ComponentListSummary[]>(fixtures.scopedManager, "/api/components");
    expect(components.some((component) => component.component_id === fixtures.inScopeComponent.component_id)).toBe(true);
    expect(components.some((component) => component.component_id === fixtures.outOfScopeComponent.component_id)).toBe(false);
    expect(components.some((component) => component.component_id === fixtures.inScopeVisualComponent.component_id)).toBe(true);
    expect(components.some((component) => component.component_id === fixtures.outOfScopeVisualComponent.component_id)).toBe(false);
    const inComponent = await getJson<ComponentDefinition>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeComponent.slug}`,
    );
    expect(inComponent.component_id).toBe(fixtures.inScopeComponent.component_id);
    expect(inComponent.versions.length).toBeGreaterThan(0);
    expect(
      inComponent.versions.some(
        (version) =>
          version.component_version_id ===
          fixtures.inScopeComponent.component_version_id,
      ),
    ).toBe(true);
    const componentTable = await getJson<ComponentTable>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeComponent.slug}/table`,
    );
    expect(componentTable).toMatchObject({
      schema_version: 1,
      component_id: fixtures.inScopeComponent.component_id,
      component_version_id: fixtures.inScopeComponent.component_version_id,
      component_type: "table",
    });
    expect(componentTable.materialization_state).toBe("ready");
    expect(componentTable.rows.length).toBeGreaterThan(0);
    const visualComponent = await getJson<ComponentDefinition>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeVisualComponent.slug}`,
    );
    expect(visualComponent.component_id).toBe(
      fixtures.inScopeVisualComponent.component_id,
    );
    expect(visualComponent.versions.some((version) => version.component_type === "bar")).toBe(true);
    const visual = await getJson<ComponentVisual>(
      fixtures.scopedManager,
      `/api/components/${fixtures.inScopeVisualComponent.slug}/bar`,
    );
    expect(visual.materialization_state).toBe("ready");
    expect(visual).toMatchObject({
      schema_version: 1,
      component_id: fixtures.inScopeVisualComponent.component_id,
      component_version_id: fixtures.inScopeVisualComponent.component_version_id,
      component_type: "bar",
    });
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
      {
        path: `/api/admin/components/${fixtures.inScopeComponent.slug}`,
        count: 2,
      },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/components/${fixtures.inScopeComponent.slug}`,
        expectedText: fixtures.inScopeComponent.name,
        documentRootSelector: COMPONENT_DOCUMENT_ROOT,
      });
    });
    await expect(
      page.getByRole("heading", { level: 1, name: fixtures.inScopeComponent.name }),
    ).toBeVisible();
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      {
        path: `/api/admin/components/${fixtures.inScopeComponent.slug}`,
        count: 2,
      },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/components/${fixtures.inScopeComponent.slug}/view`,
        expectedText: fixtures.inScopeComponent.name,
        documentRootSelector: COMPONENT_DOCUMENT_ROOT,
      });
    });
    await expect(
      page.getByRole("heading", { level: 1, name: fixtures.inScopeComponent.name }),
    ).toBeVisible();
    await expect(page.getByRole("table")).toBeVisible();
    await assertNativeRouteGuard.whileExpectedForbiddenGets([
      {
        path: `/api/admin/components/${fixtures.inScopeVisualComponent.slug}`,
        count: 2,
      },
    ], async () => {
      await expectHydratedRoute(page, {
        path: `/components/${fixtures.inScopeVisualComponent.slug}/view`,
        expectedText: fixtures.inScopeVisualComponent.name,
        documentRootSelector: COMPONENT_DOCUMENT_ROOT,
      });
    });
    await expect(
      page.getByRole("heading", { level: 1, name: fixtures.inScopeVisualComponent.name }),
    ).toBeVisible();
    await expect(page.locator(".component-visual-preview")).toBeVisible();

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
    expect(operations.dataset_readiness.state).toBe("available");
    expect(operations.dataset_readiness.datasets.some((item) => item.dataset_id === fixtures.inScopeDataset.id)).toBe(true);
    expect(operations.dataset_readiness.datasets.some((item) => item.dataset_id === fixtures.outOfScopeDataset.id)).toBe(false);
    expect(operations.workflow_assignments.every((item) => fixtures.inScopeNodeIds.has(item.node_id))).toBe(true);
    await assertNativeRouteGuard();
  });

  test("scoped component manager cannot bind or publish out-of-scope dataset major lines", async () => {
    const outOfScopeSlug = `${RUN_ID}-component-manage-out`;
    const manageableSlug = `${RUN_ID}-component-manage-in`;
    const componentSession = await getJson<SessionState>(fixtures.componentManager, "/api/auth/session");
    expect(componentSession.account?.capabilities).toContain("components:manage");
    const partialSession = await getJson<SessionState>(fixtures.partialComponentManager, "/api/auth/session");
    expect(partialSession.account?.capabilities).toContain("components:manage");
    expect(fixtures.inScopeDataset.visibility_nodes.length).toBeGreaterThan(1);
    const partialContainmentError = await expectErrorStatus(
      fixtures.partialComponentManager,
      "post",
      "/api/admin/components",
      403,
      "component.forbidden",
      {
        schema_version: 1,
        name: `${RUN_ID} Partial Containment Component`,
        slug: `${RUN_ID}-partial-containment-component`,
        description: "Partial-overlap authoring containment fixture.",
        version: componentVersionInput(
          fixtures.inScopeDatasetReference,
          "table",
          tableConfig(fixtures.inScopeDataset),
          "Partial-overlap authoring containment fixture",
        ),
      },
    );
    expectComponentForbidden(partialContainmentError);
    const manageableComponent = await postJson<ComponentDefinition>(
      fixtures.componentManager,
      "/api/admin/components",
      {
        schema_version: 1,
        name: `${RUN_ID} Manageable Component`,
        slug: manageableSlug,
        description: "In-scope component management permission fixture.",
        version: componentVersionInput(
          fixtures.inScopeDatasetReference,
          "table",
          tableConfig(fixtures.inScopeDataset),
          "In-scope component management permission fixture",
        ),
      },
    );
    const manageableComponents = await getJson<ComponentListSummary[]>(
      fixtures.componentManager,
      "/api/admin/components",
    );
    expect(manageableComponents.length).toBeGreaterThan(0);
    expect(
      manageableComponents.some(
        (component) =>
          component.component_id === manageableComponent.component_id,
      ),
    ).toBe(true);

    const bindError = await expectErrorStatus(
      fixtures.componentManager,
      "post",
      `/api/admin/components/${manageableComponent.component_id}/versions`,
      403,
      "component.forbidden",
      {
        schema_version: 1,
        version: componentVersionInput(
          fixtures.outOfScopeDatasetReference,
          "table",
          tableConfig(fixtures.outOfScopeDataset),
          "Out-of-scope version binding probe",
        ),
      },
    );
    expectComponentForbidden(bindError);

    const validateError = await expectErrorStatus(
      fixtures.componentManager,
      "post",
      "/api/admin/components/validate",
      403,
      "component.forbidden",
      componentVersionInput(
        fixtures.outOfScopeDatasetReference,
        "table",
        tableConfig(fixtures.outOfScopeDataset),
        "Out-of-scope validation probe",
      ),
    );
    expectComponentForbidden(validateError);

    const outOfScopeDraft = await postJson<ComponentDefinition>(fixtures.admin, "/api/admin/components", {
      schema_version: 1,
      name: `${RUN_ID} Out Component`,
      slug: outOfScopeSlug,
      description: "Out-of-scope component management permission fixture.",
      version: componentVersionInput(
        fixtures.outOfScopeDatasetReference,
        "table",
        tableConfig(fixtures.outOfScopeDataset),
        "Out-of-scope component management permission fixture",
      ),
    });
    const outOfScopeComponent = await getJson<ComponentDefinition>(
      fixtures.admin,
      `/api/admin/components/${outOfScopeDraft.component_id}`,
    );
    expect(outOfScopeComponent.component_id).toBe(outOfScopeDraft.component_id);
    const outVersion = outOfScopeComponent.versions[0];

    const publishError = await expectErrorStatus(
      fixtures.componentManager,
      "post",
      `/api/admin/components/${outOfScopeDraft.component_id}/versions/${outVersion.component_version_id}/publish`,
      404,
      "component.not_found",
      {},
    );
    expectComponentError(
      publishError,
      "component.not_found",
      "Component resource was not found",
    );
  });

  test("explicit historical component table checks selected version dataset scope", async () => {
    const slug = `${RUN_ID}-historical-component-scope`;
    const component = await postJson<ComponentDefinition>(fixtures.admin, "/api/admin/components", {
      schema_version: 1,
      name: `${RUN_ID} Historical Component Scope`,
      slug,
      description: "Historical version selected-dataset permission fixture.",
      version: componentVersionInput(
        fixtures.outOfScopeDatasetReference,
        "table",
        tableConfig(fixtures.outOfScopeDataset),
        "Initial out-of-scope historical version",
      ),
    });
    const firstVersion = await getJson<ComponentDefinition>(
      fixtures.admin,
      `/api/admin/components/${component.component_id}`,
    );
    expect(firstVersion.component_id).toBe(component.component_id);
    const hiddenHistoryVersion = firstVersion.versions[0];
    await postJson<ComponentMutationResponse>(
      fixtures.admin,
      `/api/admin/components/${component.component_id}/versions/${hiddenHistoryVersion.component_version_id}/publish`,
      {},
    );

    const secondVersion = await postJson<ComponentMutationResponse>(
      fixtures.admin,
      `/api/admin/components/${component.component_id}/versions`,
      {
        schema_version: 1,
        version: componentVersionInput(
          fixtures.inScopeDatasetReference,
          "table",
          tableConfig(fixtures.inScopeDataset),
          "Switch visible published history to the in-scope dataset.",
        ),
      },
    );
    const secondVersionId = requireComponentVersionId(secondVersion);
    expect(secondVersion.component_id).toBe(component.component_id);
    await postJson<ComponentMutationResponse>(
      fixtures.admin,
      `/api/admin/components/${component.component_id}/versions/${secondVersionId}/publish`,
      {},
    );

    const currentTable = await getJson<ComponentTable>(
      fixtures.scopedManager,
      `/api/components/${slug}/table`,
    );
    expect(currentTable).toMatchObject({
      schema_version: 1,
      component_id: component.component_id,
      component_version_id: secondVersionId,
      component_type: "table",
    });
    expect(currentTable.materialization_state).toBe("ready");
    expect(currentTable.rows.length).toBeGreaterThan(0);
    expect(await getJson<ComponentTable>(fixtures.admin, `/api/components/${slug}/versions/${hiddenHistoryVersion.component_version_id}/table`))
      .toMatchObject({
        schema_version: 1,
        component_id: component.component_id,
        component_version_id: hiddenHistoryVersion.component_version_id,
        component_type: "table",
      });
    const hiddenHistoryError = await expectErrorStatus(
      fixtures.scopedManager,
      "get",
      `/api/components/${slug}/versions/${hiddenHistoryVersion.component_version_id}/table`,
      404,
      "component.not_found",
    );
    expectComponentError(
      hiddenHistoryError,
      "component.not_found",
      "Component resource was not found",
    );
    const selectedCurrentTable = await getJson<ComponentTable>(
      fixtures.scopedManager,
      `/api/components/${slug}/versions/${secondVersionId}/table`,
    );
    expect(selectedCurrentTable).toMatchObject({
      schema_version: 1,
      component_id: component.component_id,
      component_version_id: secondVersionId,
      component_type: "table",
    });
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
        documentRootSelector: DATASET_DOCUMENT_ROOT,
      });
      await expect(page.getByRole("heading", { level: 1, name: "Dataset Revisions" })).toBeVisible();
      await expect(page.locator("tbody")).toContainText("Draft");
      await expectHydratedRoute(page, {
        path: `/datasets/${dataset.id}/revisions/${draft.revision_id}`,
        expectedText: "Dataset Revision",
        documentRootSelector: DATASET_DOCUMENT_ROOT,
      });
      await expect(page.getByRole("heading", { level: 1, name: "Dataset Revision" })).toBeVisible();
      await expect(page.locator(".route-panel__section").filter({ hasText: "Status" }).first()).toContainText("Draft");

      await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
      await assertNativeRouteGuard.whileExpectedForbiddenGets([{
        path: `/api/admin/datasets/${dataset.id}/revisions`,
        count: 2,
      }], async () => {
        await expectHydratedRoute(page, {
          path: `/datasets/${dataset.id}/revisions`,
          expectedText: "Dataset Revisions",
          documentRootSelector: DATASET_DOCUMENT_ROOT,
        });
      });
      await expect(page.getByRole("heading", { level: 1, name: "Dataset Revisions" })).toBeVisible();
      await expect(page.locator("tbody")).toContainText("Published current");
      await expect(page.locator("tbody")).not.toContainText("Draft");
      await assertNativeRouteGuard.whileExpectedForbiddenGets([{
        path: `/api/admin/datasets/${dataset.id}/revisions/${draft.revision_id}`,
        count: 2,
      }], async () => {
        await expectHydratedRoute(page, {
          path: `/datasets/${dataset.id}/revisions/${draft.revision_id}`,
          expectedText: "Revision unavailable",
          documentRootSelector: DATASET_DOCUMENT_ROOT,
        });
      });
      const hiddenDraftResponse = await fixtures.scopedManager.get(
        `/api/datasets/${dataset.id}/revisions/${draft.revision_id}`,
      );
      expect(hiddenDraftResponse.status()).toBe(404);
      const hiddenDraftBody = (await hiddenDraftResponse.json()) as {
        error: { code: string };
      };
      expect(hiddenDraftBody.error.code).toBe("dataset.not_found_or_forbidden");
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
      (item) =>
        item.node_id === fixtures.inScopeNode.id &&
        item.workflow_version_id === fixtures.workflowVersionId,
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

    await postJson<ResponseMutationResult>(
      fixtures.scopedManager,
      "/api/responses",
      { workflow_assignment_id: fixtures.inScopeAssignmentId },
    );
    await expectStatus(
      fixtures.scopedManager,
      "post",
      "/api/responses",
      [404],
      { workflow_assignment_id: fixtures.outOfScopeAssignmentId },
    );
    await expectStatus(
      fixtures.scopedManager,
      "post",
      "/api/workflow-assignments",
      [400, 403],
      {
        workflow_version_id: inCandidate.workflow_version_id,
        node_id: fixtures.outOfScopeNode.id,
        account_id: fixtures.userIds.owner,
      },
    );
  });

  test("submission management combines scope with response ownership", async () => {
    const ownOutOfScope = await expectStatus(
      fixtures.scopedManager,
      "post",
      "/api/responses",
      [404],
      { workflow_assignment_id: fixtures.outOfScopeOwnerAssignmentId },
    );
    expect(await ownOutOfScope.json()).toEqual({
      error: "not_found",
      message: "Response was not found",
    });

    const unrelatedOutOfScope = await postJson<ResponseMutationResult>(
      fixtures.outOfScopeOwner,
      "/api/responses",
      { workflow_assignment_id: fixtures.outOfScopeAssignmentId },
    );
    await expectStatus(
      fixtures.scopedManager,
      "get",
      `/api/responses/${unrelatedOutOfScope.id}`,
      [404],
    );

    const responses = await getJson<PermissionResponseSummary[]>(
      fixtures.scopedManager,
      "/api/responses",
    );
    expect(responses.some((item) => item.id === unrelatedOutOfScope.id)).toBe(false);
    expect(responses.every((item) => fixtures.inScopeNodeIds.has(item.node_id))).toBe(true);
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

    const ownerSubmission = await postJson<ResponseMutationResult>(
      fixtures.owner,
      "/api/responses",
      { workflow_assignment_id: fixtures.ownerAssignmentId },
    );
    await getJson(fixtures.owner, `/api/responses/${ownerSubmission.id}`);

    await expectStatus(
      fixtures.owner,
      "post",
      "/api/responses",
      [404],
      { workflow_assignment_id: fixtures.delegateAssignmentId },
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
    const delegatedSubmission = await postJson<ResponseMutationResult>(
      fixtures.delegator,
      "/api/responses",
      { workflow_assignment_id: fixtures.delegateAssignmentId },
    );
    await getJson(fixtures.delegator, `/api/responses/${delegatedSubmission.id}`);
  });

  test("session metadata exposes capabilities, scopes, and delegations without legacy access switches", async () => {
    const scopedSession = await getJson<SessionState>(fixtures.scopedManager, "/api/auth/session");
    expect(scopedSession.authenticated).toBe(true);
    expect(scopedSession.account?.capabilities).toEqual(
      expect.arrayContaining(["forms:read", "workflows:manage", "submissions:manage"]),
    );
    expect(scopedSession.account?.scope_nodes.map((node) => node.node_name)).toContain(
      fixtures.inScopeNode.name,
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
          path: `/organization/${fixtures.inScopeNode.id}`,
          expectedText: "Loading detail",
        },
        {
          path: `/organization/${fixtures.inScopeNode.id}/edit`,
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
      fixtures.inScopeNode.id,
      editor.id,
      fixtures.workflowVersionId,
    );
    const responseDraft = await postJson<ResponseMutationResult>(
      editorContext,
      "/api/responses",
      { workflow_assignment_id: assignment.id },
    );

    const dataset = await getJson<DatasetDefinition>(
      fixtures.admin,
      `/api/datasets/${fixtures.inScopeDataset.id}`,
    );
    const datasetDraftName = `${dataset.name} Native Route Draft`;
    const datasetDraftLabel = `${RUN_ID} Native Route Label`;
    const datasetDraft = await postJson<DatasetDraftRevisionResponse>(
      fixtures.admin,
      `/api/admin/datasets/${dataset.id}/draft-revision`,
      {
        name: datasetDraftName,
        slug: dataset.slug,
        grain: "submission",
        visibility_node_ids: dataset.visibility_nodes.map((node) => node.node_id),
        initial_source: dataset.initial_source,
        operations: dataset.operations,
        restriction_policy: dataset.restriction_policy ?? null,
        version_label: datasetDraftLabel,
      },
    );
    const datasetDraftDetail = await getJson<DatasetRevisionDetail>(
      fixtures.admin,
      `/api/datasets/${dataset.id}/revisions/${datasetDraft.revision_id}`,
    );
    expect(datasetDraftDetail).toMatchObject({
      id: datasetDraft.revision_id,
      dataset_id: dataset.id,
      version_label: datasetDraftLabel,
      status: "draft",
      metadata: {
        name: datasetDraftName,
        slug: dataset.slug,
      },
    });
    const datasetDraftVersion = datasetRevisionVersion(datasetDraftDetail);

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
          {
            path: "/datasets",
            expectedText: "Datasets",
            additionalExpectedTexts: [dataset.name],
            documentRootSelector: DATASET_DOCUMENT_ROOT,
          },
          {
            path: "/datasets/new",
            expectedText: "Create Dataset",
            documentRootSelector: DATASET_DOCUMENT_ROOT,
          },
          {
            path: `/datasets/${dataset.id}`,
            expectedText: dataset.name,
            documentRootSelector: DATASET_DOCUMENT_ROOT,
          },
          {
            path: `/datasets/${dataset.id}/edit`,
            expectedText: "Edit Dataset",
            documentRootSelector: DATASET_DOCUMENT_ROOT,
            expectedLabeledValues: [
              { label: "Name", value: dataset.name },
              { label: "Slug", value: dataset.slug },
            ],
          },
          {
            path: `/datasets/${dataset.id}/preview`,
            expectedText: dataset.name,
            expectedRootMarkup: 'class="dataset-preview-page"',
            contentSelector: ".dataset-preview-page",
            documentRootSelector: DATASET_DOCUMENT_ROOT,
          },
          {
            path: `/datasets/${dataset.id}/revisions`,
            expectedText: dataset.name,
            documentRootSelector: DATASET_DOCUMENT_ROOT,
            additionalExpectedTexts: [
              datasetDraftVersion,
              datasetDraftLabel,
              "Draft",
            ],
          },
          {
            path: `/datasets/${dataset.id}/revisions/${datasetDraft.revision_id}`,
            expectedText: datasetDraftName,
            documentRootSelector: DATASET_DOCUMENT_ROOT,
            additionalExpectedTexts: [datasetDraftVersion, "Draft"],
            expectedLabeledValues: [
              { label: "Revision label", value: datasetDraftLabel },
            ],
          },
          {
            path: `/datasets/${dataset.id}/revisions/${datasetDraft.revision_id}/edit`,
            expectedText: "Edit Revision",
            documentRootSelector: DATASET_DOCUMENT_ROOT,
            expectedLabeledValues: [
              { label: "Name", value: datasetDraftName },
              { label: "Slug", value: dataset.slug },
            ],
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
    const draftOnly = await postJson<ComponentDefinition>(fixtures.admin, "/api/admin/components", {
      schema_version: 1,
      // The no-JavaScript directory is server-rendered with the same ten-row
      // pagination contract as the hydrated view. Keep this scenario-owned
      // identity deterministically on the first page even after sibling tests
      // have created additional Components.
      name: `000 ${RUN_ID} Native Draft Component`,
      slug: `${RUN_ID}-native-draft-component`,
      description: "Isolated native-route draft visibility fixture.",
      version: componentVersionInput(
        fixtures.inScopeDatasetReference,
        "table",
        tableConfig(fixtures.inScopeDataset),
        "Isolated native-route draft visibility fixture.",
      ),
    });
    const draftOnlyDetail = await getJson<ComponentDefinition>(
      fixtures.admin,
      `/api/admin/components/${draftOnly.component_id}`,
    );
    const draftOnlyVersion = requireItem(
      draftOnlyDetail.versions,
      (version) => version.publication_state === "draft",
      "the native-route fixture should contain its own draft version",
    );
    try {
      const manageableComponents = await expectJson<Array<{
        component_id: string;
        name: string;
        slug: string;
        versions: Array<{ publication_state: string }>;
      }>>(
        await fixtures.admin.get("/api/admin/components"),
      );
      expect(
        manageableComponents.some(
          (component) =>
            component.component_id === draftOnly.component_id &&
            component.versions.some((version) => version.publication_state === "draft") &&
            !component.versions.some((version) => version.publication_state === "published"),
        ),
        "the scenario-owned draft Component should be manager-visible",
      ).toBe(true);
      await withNoJavaScriptPage(browser, async (page) => {
        await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
        await expectNoJavaScriptRoutes(page, [
          {
            path: "/components",
            expectedText: "Components",
            documentRootSelector: COMPONENT_DOCUMENT_ROOT,
            contentSelector: COMPONENT_CONTENT_ROOT,
          },
          {
            path: "/components/new",
            expectedText: "Create Component",
            documentRootSelector: COMPONENT_DOCUMENT_ROOT,
            contentSelector: COMPONENT_CONTENT_ROOT,
          },
          {
            path: `/components/${fixtures.inScopeComponent.slug}`,
            expectedText: fixtures.inScopeComponent.name,
            documentRootSelector: COMPONENT_DOCUMENT_ROOT,
            contentSelector: COMPONENT_CONTENT_ROOT,
          },
          {
            path: `/components/${fixtures.inScopeComponent.slug}/edit`,
            expectedText: "Edit Component",
            documentRootSelector: COMPONENT_DOCUMENT_ROOT,
            contentSelector: COMPONENT_CONTENT_ROOT,
          },
          {
            path: `/components/${fixtures.inScopeComponent.slug}/versions`,
            expectedText: `${fixtures.inScopeComponent.name} versions`,
            documentRootSelector: COMPONENT_DOCUMENT_ROOT,
            contentSelector: COMPONENT_CONTENT_ROOT,
          },
          {
            path: `/components/${fixtures.inScopeComponent.slug}/view`,
            expectedText: fixtures.inScopeComponent.name,
            documentRootSelector: COMPONENT_DOCUMENT_ROOT,
            contentSelector: COMPONENT_CONTENT_ROOT,
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

        await page.goto("/components");
        await expect(page.getByRole("link", { name: "Create Component" })).toBeVisible();
        const draftEntry = page
          .locator(`[data-component-directory-item][data-component-id="${draftOnly.component_id}"]`)
          .filter({ visible: true });
        await expect(draftEntry).toHaveCount(1);
        await expect(draftEntry.getByText(draftOnly.name, { exact: true })).toBeVisible();
        await expect(draftEntry.getByRole("link", { name: "Edit" })).toBeVisible();
        await page.goto(`/components/${fixtures.inScopeComponent.slug}`);
        await expect(page.getByRole("link", { name: "Versions" })).toBeVisible();
        await expect(page.getByRole("link", { name: "Edit" })).toBeVisible();
      });

      await withNoJavaScriptPage(browser, async (page) => {
        await signInPage(page, `${RUN_ID}-scoped-manager@tessara.local`);
        await page.goto("/components");
        await expect(page.getByRole("heading", { level: 1, name: "Components" })).toBeVisible();
        await expect(page.getByRole("link", { name: "Create Component" })).toHaveCount(0);
        await expect(page.locator(`[data-component-id="${draftOnly.component_id}"]`)).toHaveCount(0);
        await page.goto(`/components/${fixtures.inScopeComponent.slug}`);
        await expect(
          page.getByRole("heading", { level: 1, name: fixtures.inScopeComponent.name }),
        ).toBeVisible();
        await expect(page.getByRole("link", { name: "Versions" })).toHaveCount(0);
        await expect(page.getByRole("link", { name: "Edit" })).toHaveCount(0);
      });

    } finally {
      await expectStatus(
        fixtures.admin,
        "delete",
        `/api/admin/components/${draftOnly.component_id}/versions/${draftOnlyVersion.component_version_id}`,
        [200],
      );
    }
  });
});
