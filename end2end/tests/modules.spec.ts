import {
  expect,
  request,
  test,
  type APIRequestContext,
  type APIResponse,
  type Page,
} from "@playwright/test";
import { createHash } from "node:crypto";
import { invokeDemoSeedEndpoint } from "./support/demo-seed";
import {
  attachNativeRouteGuard,
  expectNoJavaScriptNativeRouteDirectLoadAndRefresh,
} from "./support/native-route";
import { runPlaywrightSql } from "./support/postgres";

const BASE_URL = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const RUN_ID = `pw-modules-${Date.now()}`;
const ENTITY_PREFIX = "pw-modules-";
const PASSWORD = "tessara-dev-modules";
const FORMS_DEFINITION = "tessara.forms";
const RESPONSES_DEFINITION = "tessara.responses";
const MIGRATION_DEFINITION = "tessara.migration";
const UNKNOWN_DEFINITION = "tessara.unknown-definition";
const BENIGN_NAVIGATION_ABORT =
  "WebAssembly compilation aborted: Network error: Response body loading was aborted";
const EXPECTED_SHELL_OUTAGE_CONSOLE =
  "Failed to load resource: the server responded with a status of 503 (Service Unavailable)";
const EXPECTED_PENDING_WORK_DENIAL_CONSOLE =
  "Failed to load resource: the server responded with a status of 403 (Forbidden)";
const EXPECTED_UNAUTHENTICATED_CONSOLE =
  "Failed to load resource: the server responded with a status of 401 (Unauthorized)";

type IdResponse = { id: string };
type CapabilitySummary = { id: string; key: string };
type ApiErrorBody = { code: string; message: string; error: string };
type Actor = { context: APIRequestContext; email: string };
type FeatureDeclaration = {
  id: string;
  name: string;
  description: string;
  use_cases: string[];
  inputs: string[];
  outcomes: string[];
  constraints: string[];
  contracts: string[];
  resource_types: string[];
  destinations: string[];
  capabilities: string[];
  configuration_pointers: string[];
};
type FunctionalContractKind = "api" | "event" | "resource" | "behavior";
type FunctionalContractDeclaration = {
  id: string;
  version: string;
  kind: FunctionalContractKind;
  description: string;
};
type ResourceTypeDeclaration = { id: string; description: string };
type RouteParameterType = "string" | "integer" | "boolean" | "uuid";
type RouteParameterDeclaration = {
  name: string;
  value_type: RouteParameterType;
  required: boolean;
};
type RouteKind = "product" | "administration" | "configuration" | "diagnostics";
type RouteDeclaration = {
  name: string;
  kind: RouteKind;
  parameters: RouteParameterDeclaration[];
  resolved_path?: string;
};
type SecurityCapabilityDeclaration = { id: string; description: string };
type TransitionAvailability =
  | "active_in_process"
  | "unavailable"
  | "retired";
type FunctionalDependency = {
  contract_id: string;
  version_requirement: string;
  binding_key: string;
  optional: boolean;
};
type NavigationDeclaration = {
  id: string;
  destination: string;
  label: string;
  group: string;
  order_hint: number;
  required_capabilities_any_of: string[];
};
type ModuleFinding = { code: string; path: string; message: string };
type ModuleDescriptor = Record<string, unknown> & {
  schema_version: number;
  reserved_definition_id: string;
  display_name: string;
  description: string;
  availability: TransitionAvailability;
  features: FeatureDeclaration[];
  provided_contracts: FunctionalContractDeclaration[];
  dependencies: FunctionalDependency[];
  resource_types: ResourceTypeDeclaration[];
  routes: RouteDeclaration[];
  navigation: NavigationDeclaration[];
  security_capabilities: SecurityCapabilityDeclaration[];
  configuration_schema: unknown | null;
};
type TransitionalModuleInventoryEntry = {
  kind: "transitional_in_process";
  descriptor: ModuleDescriptor;
  source_digest: string;
  findings: ModuleFinding[];
};
type IndependentModuleInventoryEntry = {
  kind: "independently_deployed";
  definition: {
    id: string;
    display_name: string;
    description: string;
  };
  release: {
    manifest_digest: string;
    version: string;
  };
  instance: {
    id: string;
    ready: boolean;
    enabled: boolean;
    healthy: boolean;
  };
  manifest: Record<string, unknown> | null;
  findings: ModuleFinding[];
};
type ModuleInventoryEntry =
  | TransitionalModuleInventoryEntry
  | IndependentModuleInventoryEntry;
type ModuleInventoryResponse = {
  schema_version: number;
  installation: Record<string, unknown>;
  core_runtime: Record<string, unknown>;
  entries: ModuleInventoryEntry[];
};
type ModuleDetailResponse = {
  schema_version: number;
  installation_id: string;
  entry: ModuleInventoryEntry;
};
type ModuleManagementBootstrap =
  | {
      route: "directory";
      inventory: ModuleInventoryResponse;
      navigation_policy: unknown;
      access: unknown;
    }
  | {
      route: "detail";
      detail: ModuleDetailResponse;
      navigation_policy: unknown;
      access: unknown;
    }
  | { route: "restricted" | "not_found" | "unavailable" };
type ModuleIdentity = { definition_id: string; source_digest: string };
type NavigationPolicyGroup = {
  id: string;
  label: string;
  order: number;
  owner: "core" | "custom";
  can_rename: boolean;
  can_move: boolean;
  can_delete: boolean;
};
type NavigationPolicyDestination = {
  id: string;
  key: string;
  label: string;
  route: string;
  semantic_destination?: string;
  definition_id?: string;
  owner: "core" | "contribution";
  required_capabilities_any_of: string[];
  group_id: string;
  visible: boolean;
  order: number;
  available: boolean;
  can_hide: boolean;
  can_move_between_groups: boolean;
  can_reorder: boolean;
};
type NavigationPolicyResponse = {
  schema_version: number;
  installation_id: string;
  revision: number;
  can_manage_navigation: boolean;
  groups: NavigationPolicyGroup[];
  destinations: NavigationPolicyDestination[];
};
type FixtureState = {
  admin: APIRequestContext;
  reader: Actor;
  manager: Actor;
  scopedReader: Actor;
  productOnly: Actor;
  noAccess: Actor;
  originalPolicy: NavigationPolicyResponse;
};

let fixtures: FixtureState;
const contexts: APIRequestContext[] = [];

async function newContext() {
  const context = await request.newContext({ baseURL: BASE_URL });
  contexts.push(context);
  return context;
}

async function expectJson<T>(response: APIResponse) {
  const text = await response.text();
  expect(
    response.ok(),
    `${response.url()} returned ${response.status()}: ${text}`,
  ).toBeTruthy();
  return JSON.parse(text) as T;
}

async function getJson<T>(context: APIRequestContext, url: string) {
  return expectJson<T>(await context.get(url));
}

async function postJson<T>(
  context: APIRequestContext,
  url: string,
  data?: Record<string, unknown>,
) {
  return expectJson<T>(await context.post(url, { data }));
}

async function putJson<T>(
  context: APIRequestContext,
  url: string,
  data: Record<string, unknown>,
) {
  return expectJson<T>(await context.put(url, { data }));
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

async function signIn(context: APIRequestContext, email: string, password: string) {
  await postJson<{ token: string }>(context, "/api/auth/login", { email, password });
}

async function signInPage(page: Page, email: string, password = PASSWORD) {
  await page.context().clearCookies();
  const response = await page.request.post("/api/auth/login", {
    data: { email, password },
  });
  const body = await expectJson<{ token: string }>(response);
  await page.context().addCookies([
    {
      name: "tessara_session",
      value: body.token,
      url: BASE_URL,
      httpOnly: true,
      sameSite: "Lax",
    },
  ]);
}

async function createRole(
  admin: APIRequestContext,
  name: string,
  capabilityKeys: string[],
) {
  const capabilities = await getJson<CapabilitySummary[]>(admin, "/api/admin/capabilities");
  const capabilityIds = capabilityKeys.map((key) => {
    const capability = capabilities.find((item) => item.key === key);
    expect(capability, `capability ${key} should exist`).toBeTruthy();
    return capability!.id;
  });
  return postJson<IdResponse>(admin, "/api/admin/roles", {
    name,
    capability_ids: capabilityIds,
  });
}

async function createActor(
  admin: APIRequestContext,
  identity: string,
  capabilityKeys: string[],
) {
  const role = await createRole(admin, `${RUN_ID}-${identity}-role`, capabilityKeys);
  const email = `${RUN_ID}-${identity}@tessara.local`;
  await postJson<IdResponse>(admin, "/api/admin/users", {
    email,
    display_name: `${RUN_ID}-${identity}`,
    password: PASSWORD,
    is_active: true,
    role_ids: [role.id],
  });
  const context = await newContext();
  await signIn(context, email, PASSWORD);
  return { context, email };
}

function cleanupPlaywrightEntities() {
  runPlaywrightSql(`
DELETE FROM core_control_plane_audit_events
WHERE actor_account_id IN (
  SELECT id
  FROM accounts
  WHERE email LIKE '${ENTITY_PREFIX}%'
     OR display_name LIKE '${ENTITY_PREFIX}%'
);

DELETE FROM accounts
WHERE email LIKE '${ENTITY_PREFIX}%'
   OR display_name LIKE '${ENTITY_PREFIX}%';

DELETE FROM roles
WHERE name LIKE '${ENTITY_PREFIX}%';

DELETE FROM nodes
WHERE name LIKE '${ENTITY_PREFIX}%';

DELETE FROM node_types
WHERE name LIKE '${ENTITY_PREFIX}%'
   OR slug LIKE '${ENTITY_PREFIX}%';
`);
}

function forceScopedAssignment(email: string) {
  const scopeName = `${RUN_ID}-scope`;
  runPlaywrightSql(`
INSERT INTO node_types (name, slug)
VALUES ('${scopeName}', '${scopeName}');

INSERT INTO nodes (node_type_id, name)
SELECT id, '${scopeName}'
FROM node_types
WHERE slug = '${scopeName}';

UPDATE role_assignments
SET node_id = (
  SELECT id
  FROM nodes
  WHERE name = '${scopeName}'
)
WHERE account_id = (
  SELECT id
  FROM accounts
  WHERE email = '${email}'
)
  AND node_id IS NULL;
`);
}

function policyUpdate(
  policy: NavigationPolicyResponse,
  groups = policy.groups,
  destinations = policy.destinations,
) {
  return {
    schema_version: 2,
    expected_revision: policy.revision,
    groups: groups.map((group) => ({
      id: group.id,
      label: group.label,
      order: group.order,
    })),
    destinations: destinations.map((entry) => ({
      id: entry.id,
      group_id: entry.group_id,
      visible: entry.visible,
      order: entry.order,
    })),
  };
}

function samePolicyValues(
  left: NavigationPolicyResponse,
  right: NavigationPolicyResponse,
) {
  const groupValues = (entries: NavigationPolicyGroup[]) =>
    entries.map(({ id, label, order }) => ({ id, label, order })).sort((a, b) => a.id.localeCompare(b.id));
  const destinationValues = (entries: NavigationPolicyDestination[]) =>
    entries
      .map((entry) => ({
        id: entry.id,
        group_id: entry.group_id,
        visible: entry.visible,
        order: entry.order,
      }))
      .sort((a, b) => a.id.localeCompare(b.id));
  return JSON.stringify({ groups: groupValues(left.groups), destinations: destinationValues(left.destinations) })
    === JSON.stringify({ groups: groupValues(right.groups), destinations: destinationValues(right.destinations) });
}

async function restoreOriginalPolicy() {
  if (!fixtures?.admin || !fixtures.originalPolicy) {
    return;
  }
  let current = await getJson<NavigationPolicyResponse>(
    fixtures.admin,
    "/api/admin/navigation-policy",
  );
  if (samePolicyValues(current, fixtures.originalPolicy)) {
    return;
  }
  const originalGroupIds = new Set(fixtures.originalPolicy.groups.map((group) => group.id));
  const populatedRemovedGroup = current.groups.some(
    (group) =>
      !originalGroupIds.has(group.id)
      && current.destinations.some((destination) => destination.group_id === group.id),
  );
  if (populatedRemovedGroup) {
    current = await putJson<NavigationPolicyResponse>(
      fixtures.admin,
      "/api/admin/navigation-policy",
      policyUpdate(current, current.groups, fixtures.originalPolicy.destinations),
    );
  }
  await putJson<NavigationPolicyResponse>(
    fixtures.admin,
    "/api/admin/navigation-policy",
    policyUpdate(current, fixtures.originalPolicy.groups, fixtures.originalPolicy.destinations),
  );
}

async function preparePolicyScenario() {
  const current = await getJson<NavigationPolicyResponse>(
    fixtures.admin,
    "/api/admin/navigation-policy",
  );
  const destinations = current.destinations.map((entry) => ({ ...entry }));
  const forms = destinations.find((entry) => entry.id === "tessara.forms.navigation");
  expect(forms, "Forms navigation contribution should exist").toBeTruthy();
  forms!.visible = true;

  const prepared = { ...current, destinations };
  if (samePolicyValues(current, prepared)) {
    return current;
  }
  return putJson<NavigationPolicyResponse>(
    fixtures.admin,
    "/api/admin/navigation-policy",
    policyUpdate(current, current.groups, destinations),
  );
}

function attachBrowserGuard(page: Page) {
  const errors: string[] = [];
  const bridgeRequests: string[] = [];
  const moduleDataRequests: string[] = [];
  const navigationPolicyRequests: string[] = [];
  let lastNavigationStartedAt = Number.NEGATIVE_INFINITY;
  let allowUnauthenticatedConsoleError = false;
  let expectedHttpErrorScope:
    | { consoleMessages: string[]; responseIdentities: string[] }
    | null = null;

  page.on("request", (browserRequest) => {
    const pathname = new URL(browserRequest.url()).pathname;
    if (pathname.startsWith("/bridge/")) {
      bridgeRequests.push(pathname);
    }
    if (pathname.startsWith("/api/admin/modules")) {
      moduleDataRequests.push(pathname);
    }
    if (pathname === "/api/admin/navigation-policy") {
      navigationPolicyRequests.push(pathname);
    }
    if (
      browserRequest.isNavigationRequest() &&
      browserRequest.frame() === page.mainFrame()
    ) {
      lastNavigationStartedAt = Date.now();
    }
  });
  page.on("response", (response) => {
    if (expectedHttpErrorScope !== null && response.status() >= 400) {
      const request = response.request();
      expectedHttpErrorScope.responseIdentities.push(
        `${request.method()} ${new URL(response.url()).pathname} ${response.status()}`,
      );
    }
  });
  page.on("console", (message) => {
    if (message.type() === "error" && expectedHttpErrorScope !== null) {
      expectedHttpErrorScope.consoleMessages.push(message.text());
    } else if (
      message.type() === "error" &&
      allowUnauthenticatedConsoleError &&
      message.text() === EXPECTED_UNAUTHENTICATED_CONSOLE
    ) {
      // Clearing the session can race the document redirect with the shell
      // navigation request. Either may emit the same expected 401 diagnostic.
    } else if (
      message.type() === "error" &&
      !(
        message.text().includes(BENIGN_NAVIGATION_ABORT) &&
        Date.now() - lastNavigationStartedAt < 5_000
      )
    ) {
      errors.push(`${page.url()}: ${message.text()}`);
    }
  });
  page.on("pageerror", (error) => {
    if (
      !(
        error.message.includes(BENIGN_NAVIGATION_ABORT) &&
        Date.now() - lastNavigationStartedAt < 5_000
      )
    ) {
      errors.push(`${page.url()}: ${error.message}`);
    }
  });

  async function whileExpectedHttpError(
    expectedConsoleMessages: string[],
    expectedResponseIdentities: string[],
    run: () => Promise<void>,
  ) {
    expect(expectedHttpErrorScope, "expected HTTP-error console scopes must not be nested").toBeNull();
    const scope = { consoleMessages: [] as string[], responseIdentities: [] as string[] };
    expectedHttpErrorScope = scope;
    try {
      await run();
    } finally {
      await page.waitForLoadState("networkidle", { timeout: 5_000 }).catch(() => {});
      let stableIntervals = 0;
      while (stableIntervals < 3) {
        const consoleCount = scope.consoleMessages.length;
        const responseCount = scope.responseIdentities.length;
        await page.waitForTimeout(50);
        stableIntervals =
          scope.consoleMessages.length === consoleCount &&
          scope.responseIdentities.length === responseCount
            ? stableIntervals + 1
            : 0;
      }
      expectedHttpErrorScope = null;
    }
    expect(
      scope.consoleMessages.every((message) =>
        expectedConsoleMessages.includes(message),
      ),
      "expected HTTP failures must not produce uncharacterized browser diagnostics",
    ).toBe(true);
    expect(
      scope.consoleMessages.length,
      "browsers may coalesce identical resource errors but must not emit more than the expected failures",
    ).toBeLessThanOrEqual(expectedConsoleMessages.length);
    expect(
      scope.responseIdentities.every((identity) =>
        expectedResponseIdentities.includes(identity),
      ),
      "the characterized browser diagnostics must correspond only to expected responses",
    ).toBe(true);
    expect(
      scope.responseIdentities.length,
      "browser response events may be omitted during navigation but must not exceed the expected failures",
    ).toBeLessThanOrEqual(expectedResponseIdentities.length);
  }

  return {
    bridgeRequests,
    moduleDataRequests,
    navigationPolicyRequests,
    resetDataRequests() {
      moduleDataRequests.length = 0;
      navigationPolicyRequests.length = 0;
    },
    async whileExpectedShellNavigationOutage(run: () => Promise<void>) {
      await whileExpectedHttpError(
        [EXPECTED_SHELL_OUTAGE_CONSOLE],
        ["GET /api/shell/navigation 503"],
        run,
      );
    },
    async whileExpectedFormsRouteDenial(run: () => Promise<void>) {
      await whileExpectedHttpError(
        [EXPECTED_PENDING_WORK_DENIAL_CONSOLE, EXPECTED_PENDING_WORK_DENIAL_CONSOLE],
        ["GET /api/forms 403", "GET /api/workflow-assignments/pending 403"],
        run,
      );
    },
    allowUnauthenticatedNavigationError() {
      allowUnauthenticatedConsoleError = true;
    },
    assertClean() {
      expect(bridgeRequests, "native Module Management must never request /bridge/*").toEqual([]);
      expect(errors, `browser console should stay clean: ${errors.join("\n")}`).toEqual([]);
    },
  };
}

function moduleIdentity(entry: ModuleInventoryEntry): ModuleIdentity {
  if (entry.kind === "independently_deployed") {
    return {
      definition_id: entry.definition.id,
      source_digest: entry.release.manifest_digest,
    };
  }
  return {
    definition_id: entry.descriptor.reserved_definition_id,
    source_digest: entry.source_digest,
  };
}

function sortedModuleIdentities(entries: ModuleIdentity[]) {
  return [...entries].sort((left, right) =>
    left.definition_id.localeCompare(right.definition_id),
  );
}

async function moduleBootstrap(page: Page) {
  const json = await page.locator("#tessara-module-management-bootstrap").textContent();
  expect(json, "SSR must emit one inert Module Management bootstrap").not.toBeNull();
  return JSON.parse(json!) as ModuleManagementBootstrap;
}

async function markerValues(
  page: Page,
  selector: string,
  attribute: string,
) {
  return page.locator(selector).evaluateAll(
    (elements, attributeName) =>
      elements.map((element) => element.getAttribute(attributeName)),
    attribute,
  );
}

async function expectRenderedModuleDetailMatchesProjection(
  page: Page,
  entry: TransitionalModuleInventoryEntry,
) {
  const descriptor = entry.descriptor;
  const availabilityLabels: Record<TransitionAvailability, string> = {
    active_in_process: "Active in Core process",
    unavailable: "Unavailable",
    retired: "Retired",
  };
  await expect(
    page.getByRole("heading", {
      level: 1,
      name: descriptor.display_name,
      exact: true,
    }),
  ).toHaveCount(1);
  const overview = page.locator(
    'section.organization-detail-card[aria-labelledby="module-overview-heading"]',
  );
  await expect(overview).toHaveCount(1);
  await expect(overview.locator(".module-detail__heading p")).toHaveText(
    descriptor.description,
  );
  const overviewRows = overview.locator("dl.organization-detail-list > div");
  const digestWithoutAlgorithm = entry.source_digest.replace(/^sha256:/, "");
  const sourceDigestPreview =
    digestWithoutAlgorithm.length > 13
      ? `${digestWithoutAlgorithm.slice(0, 8)}…${digestWithoutAlgorithm.slice(-5)}`
      : digestWithoutAlgorithm;
  await expect(overviewRows.nth(0).locator("dd code")).toHaveText(
    descriptor.reserved_definition_id,
  );
  await expect(overviewRows.nth(1).locator("dd code")).toHaveText(sourceDigestPreview);
  await expect(
    overview.getByRole("button", { name: "Copy complete source digest", exact: true }),
  ).toBeVisible();
  await expect(
    overview.getByRole("button", { name: "View complete source digest", exact: true }),
  ).toBeVisible();
  await expect(
    page
      .locator(".module-detail-page-heading__lifecycle")
      .getByText(availabilityLabels[descriptor.availability], { exact: true }),
  ).toBeVisible();
  await expect(overviewRows.nth(2).locator("dd")).toHaveText(
    descriptor.configuration_schema === null ? "Not declared" : "Declared",
  );

  const featureNodes = page.locator("[data-feature-id]");
  expect(await markerValues(page, "[data-feature-id]", "data-feature-id")).toEqual(
    descriptor.features.map((feature) => feature.id),
  );
  const featureFields: Array<[
    | "use_cases"
    | "inputs"
    | "outcomes"
    | "constraints"
    | "contracts"
    | "resource_types"
    | "destinations"
    | "capabilities"
    | "configuration_pointers",
    string,
  ]> = [
    ["use_cases", "Use cases"],
    ["inputs", "Inputs"],
    ["outcomes", "Outcomes"],
    ["constraints", "Constraints"],
    ["contracts", "Contracts"],
    ["resource_types", "Resource types"],
    ["destinations", "Destinations"],
    ["capabilities", "Capabilities"],
    ["configuration_pointers", "Configuration pointers"],
  ];
  for (const [index, feature] of descriptor.features.entries()) {
    const rendered = featureNodes.nth(index);
    await expect(rendered.locator("h3")).toHaveText(feature.name);
    await expect(rendered.locator(":scope > code")).toHaveText(feature.id);
    await expect(rendered.locator(":scope > p")).toHaveText(feature.description);
    for (const [field, label] of featureFields) {
      const values = feature[field];
      const list = rendered.locator(`[data-feature-field="${field}"]`);
      if (values.length === 0) {
        await expect(list).toHaveCount(0);
        continue;
      }
      await expect(list).toHaveCount(1);
      await expect(list.locator("h4")).toHaveText(label);
      expect(await list.locator("li").allTextContents()).toEqual(values);
    }
  }

  const contractKindLabels: Record<FunctionalContractKind, string> = {
    api: "API",
    event: "Event",
    resource: "Resource",
    behavior: "Behavior",
  };
  const contractNodes = page.locator("[data-contract-id]");
  expect(await markerValues(page, "[data-contract-id]", "data-contract-id")).toEqual(
    descriptor.provided_contracts.map((contract) => contract.id),
  );
  for (const [index, contract] of descriptor.provided_contracts.entries()) {
    const rendered = contractNodes.nth(index);
    await expect(rendered.locator(":scope > strong > code")).toHaveText(contract.id);
    await expect(rendered.locator(":scope > span")).toHaveText(
      `${contractKindLabels[contract.kind]} ${contract.version}`,
    );
    await expect(rendered.locator(":scope > p")).toHaveText(contract.description);
  }

  const dependencyNodes = page.locator("[data-dependency-binding]");
  expect(
    await markerValues(
      page,
      "[data-dependency-binding]",
      "data-dependency-binding",
    ),
  ).toEqual(descriptor.dependencies.map((dependency) => dependency.binding_key));
  for (const [index, dependency] of descriptor.dependencies.entries()) {
    const rendered = dependencyNodes.nth(index);
    await expect(rendered.locator("th code")).toHaveText(dependency.binding_key);
    await expect(rendered.locator("td").nth(0).locator("code")).toHaveText(
      dependency.contract_id,
    );
    await expect(rendered.locator("td").nth(1).locator("code")).toHaveText(
      dependency.version_requirement,
    );
    await expect(rendered.locator("td").nth(2)).toHaveText(
      dependency.optional ? "Optional" : "Required",
    );
  }

  const capabilityNodes = page.locator("[data-capability-id]");
  expect(
    await markerValues(page, "[data-capability-id]", "data-capability-id"),
  ).toEqual(descriptor.security_capabilities.map((capability) => capability.id));
  for (const [index, capability] of descriptor.security_capabilities.entries()) {
    const rendered = capabilityNodes.nth(index);
    await expect(rendered.locator(":scope > strong > code")).toHaveText(capability.id);
    await expect(rendered.locator(":scope > p")).toHaveText(capability.description);
  }

  const resourceNodes = page.locator("[data-resource-type-id]");
  expect(
    await markerValues(page, "[data-resource-type-id]", "data-resource-type-id"),
  ).toEqual(descriptor.resource_types.map((resource) => resource.id));
  for (const [index, resource] of descriptor.resource_types.entries()) {
    const rendered = resourceNodes.nth(index);
    await expect(rendered.locator(":scope > strong > code")).toHaveText(resource.id);
    await expect(rendered.locator(":scope > p")).toHaveText(resource.description);
  }

  const routeKindLabels: Record<RouteKind, string> = {
    product: "Product",
    administration: "Administration",
    configuration: "Configuration",
    diagnostics: "Diagnostics",
  };
  const parameterTypeLabels: Record<RouteParameterType, string> = {
    string: "string",
    integer: "integer",
    boolean: "boolean",
    uuid: "UUID",
  };
  const destinationNodes = page.locator("[data-destination-name]");
  expect(
    await markerValues(page, "[data-destination-name]", "data-destination-name"),
  ).toEqual(descriptor.routes.map((route) => route.name));
  for (const [index, route] of descriptor.routes.entries()) {
    const rendered = destinationNodes.nth(index);
    await expect(rendered.locator(":scope > strong > code")).toHaveText(route.name);
    await expect(rendered.locator(":scope > span").first()).toHaveText(
      routeKindLabels[route.kind],
    );

    const parameterNodes = rendered.locator("[data-destination-parameter]");
    expect(
      await parameterNodes.evaluateAll((elements) =>
        elements.map((element) => element.getAttribute("data-destination-parameter")),
      ),
    ).toEqual(route.parameters.map((parameter) => parameter.name));
    for (const [parameterIndex, parameter] of route.parameters.entries()) {
      const renderedParameter = parameterNodes.nth(parameterIndex);
      await expect(renderedParameter.locator("code")).toHaveText(parameter.name);
      await expect(renderedParameter).toContainText(
        `${parameterTypeLabels[parameter.value_type]} (${parameter.required ? "required" : "optional"})`,
      );
    }

    const destinationLink = rendered.locator(":scope > a");
    if (route.resolved_path === undefined) {
      await expect(destinationLink).toHaveCount(0);
    } else {
      await expect(destinationLink).toHaveCount(1);
      await expect(destinationLink).toHaveAttribute("href", route.resolved_path);
    }
  }

  const declaredNavigation = page.locator(
    'section[aria-labelledby="module-declared-navigation-heading"]',
  );
  if (descriptor.navigation.length === 0) {
    await expect(declaredNavigation).toHaveCount(0);
  } else {
    await expect(declaredNavigation).toHaveCount(1);
  }
  const navigationNodes = page.locator("[data-navigation-declaration]");
  expect(
    await markerValues(
      page,
      "[data-navigation-declaration]",
      "data-navigation-declaration",
    ),
  ).toEqual(descriptor.navigation.map((declaration) => declaration.id));
  for (const [index, declaration] of descriptor.navigation.entries()) {
    const rendered = navigationNodes.nth(index);
    await expect(rendered.locator(":scope > strong")).toHaveText(declaration.label);
    await expect(rendered.locator(":scope > code")).toHaveText(
      declaration.destination,
    );
    await expect(
      rendered.locator(":scope > .data-table__secondary-text").nth(0),
    ).toHaveText(
      `Discovery hint: ${declaration.group} group, source order ${declaration.order_hint}`,
    );
    expect(await rendered.locator("ul.module-navigation-eligibility li code").allTextContents())
      .toEqual(declaration.required_capabilities_any_of);
  }

  const findingsSection = page.locator('section[data-module-section="findings"]');
  const findingNodes = findingsSection.locator("[data-finding-code]");
  expect(await findingNodes.evaluateAll((elements) =>
    elements.map((element) => element.getAttribute("data-finding-code")),
  )).toEqual(
    entry.findings.map((finding) => finding.code),
  );
  expect(await findingNodes.evaluateAll((elements) =>
    elements.map((element) => element.getAttribute("data-finding-path")),
  )).toEqual(
    entry.findings.map((finding) => finding.path),
  );
  for (const [index, finding] of entry.findings.entries()) {
    const rendered = findingNodes.nth(index);
    await expect(rendered.locator(":scope > strong > code")).toHaveText(finding.code);
    await expect(rendered.locator(":scope > span > code")).toHaveText(finding.path);
    await expect(rendered.locator(":scope > p")).toHaveText(finding.message);
  }
}

async function gotoHydrated(page: Page, url: string) {
  await page.goto(url);
  await expect(page.locator("#app-root")).toHaveAttribute("data-hydration", "ready", {
    timeout: 10_000,
  });
}

function desktopNavigation(page: Page) {
  return page.locator(".sidebar .sidebar-nav");
}

async function navigationLabels(navigation: ReturnType<typeof desktopNavigation>) {
  return navigation.locator(".sidebar-link__label").allTextContents();
}

function expectBefore(labels: string[], first: string, second: string) {
  expect(labels, `${first} and ${second} should both be present`).toEqual(
    expect.arrayContaining([first, second]),
  );
  expect(labels.indexOf(first), `${first} should precede ${second}`).toBeLessThan(
    labels.indexOf(second),
  );
}

test.describe.serial("Sprint 6A Module Management", () => {
  test.beforeAll(async () => {
    cleanupPlaywrightEntities();
    const admin = await newContext();
    await signIn(admin, "admin@tessara.local", "tessara-dev-admin");
    await ensureDemoSeed(admin);

    const reader = await createActor(admin, "reader", ["modules:read"]);
    const manager = await createActor(admin, "manager", [
      "modules:manage_navigation",
    ]);
    const scopedReader = await createActor(admin, "scoped-reader", ["modules:read"]);
    const productOnly = await createActor(admin, "product-only", ["forms:read"]);
    const noAccess = await createActor(admin, "no-access", []);
    forceScopedAssignment(scopedReader.email);
    const originalPolicy = await getJson<NavigationPolicyResponse>(
      admin,
      "/api/admin/navigation-policy",
    );
    fixtures = {
      admin,
      reader,
      manager,
      scopedReader,
      productOnly,
      noAccess,
      originalPolicy,
    };
  });

  test.afterAll(async () => {
    try {
      await restoreOriginalPolicy();
    } finally {
      try {
        cleanupPlaywrightEntities();
      } finally {
        await Promise.all(contexts.map((context) => context.dispose()));
      }
    }
  });

  test("global read exposes protected Module Management without an aggregate Administration item and remains read-only", async ({
    page,
  }) => {
    const guard = attachBrowserGuard(page);
    await page.setViewportSize({ width: 1280, height: 900 });
    await signInPage(page, fixtures.reader.email);

    let releaseShellNavigation!: () => void;
    const heldShellNavigation = new Promise<void>((resolve) => {
      releaseShellNavigation = resolve;
    });
    await page.route("**/api/shell/navigation", async (route) => {
      await heldShellNavigation;
      await route.continue();
    });
    await gotoHydrated(page, "/administration/modules");

    const desktop = desktopNavigation(page);
    await expect(page.locator(".sidebar .account-card small")).toHaveText(
      fixtures.reader.email,
    );
    await expect(
      desktop.getByRole("link", { name: "Module Management" }),
      "authoritative global module read must retain protected Module Management while the shell projection is loading",
    ).toBeVisible();
    releaseShellNavigation();
    await expect(desktop.locator(".sidebar-section", { hasText: /^Admin$/ })).toBeVisible();
    await expect(desktop.getByRole("link", { name: "Module Management" })).toBeVisible();
    await expect(desktop.getByRole("link", { name: "Administration" })).toHaveCount(0);
    await page.unroute("**/api/shell/navigation");

    let shellOutageRequests = 0;
    await page.route("**/api/shell/navigation", async (route) => {
      shellOutageRequests += 1;
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: JSON.stringify({ message: "Synthetic shell navigation outage." }),
      });
    });
    await guard.whileExpectedShellNavigationOutage(async () => {
      await gotoHydrated(page, "/administration/modules");
    });
    expect(shellOutageRequests).toBe(1);
    await expect(page.locator(".sidebar .account-card small")).toHaveText(
      fixtures.reader.email,
    );
    await expect(
      desktopNavigation(page).getByRole("link", { name: "Module Management" }),
      "authoritative global module read must retain protected Module Management when shell composition fails",
    ).toBeVisible();
    await expect(
      desktopNavigation(page).getByRole("link", { name: "Administration" }),
    ).toHaveCount(0);
    await expect(
      desktopNavigation(page).locator(".sidebar-navigation-status"),
    ).toContainText(
      "Contribution navigation is temporarily unavailable.",
    );
    await page.unroute("**/api/shell/navigation");

    await page.setViewportSize({ width: 390, height: 844 });
    await page.reload();
    await expect(page.locator("#app-root")).toHaveAttribute("data-hydration", "ready");
    await page.getByRole("button", { name: "Open navigation" }).click();
    const mobile = page.locator(".mobile-nav__panel");
    await expect(mobile).toBeVisible();
    await expect(mobile.locator(".sidebar-section", { hasText: /^Admin$/ })).toBeVisible();
    await expect(mobile.getByRole("link", { name: "Module Management" })).toBeVisible();
    await expect(mobile.getByRole("link", { name: "Administration" })).toHaveCount(0);

    await page.setViewportSize({ width: 1280, height: 900 });
    await gotoHydrated(page, "/administration/modules");
    await expect(page.getByRole("heading", { level: 1, name: "Module Management" })).toBeVisible();
    await page.getByRole("tab", { name: "Navigation" }).click();
    const policy = page.locator(".module-navigation-policy");
    await expect(policy.getByText("Read-only", { exact: true })).toBeVisible();
    await expect(policy.getByRole("checkbox")).toHaveCount(0);
    await expect(policy.getByRole("button", { name: /Move .* (earlier|later)/ })).toHaveCount(0);
    await expect(policy.getByRole("button", { name: "Save navigation" })).toHaveCount(0);
    const moduleDestination = policy.locator(
      '[data-navigation-destination="core.admin.modules"]',
    );
    await expect(moduleDestination).toContainText("Module Management");
    await expect(
      moduleDestination.locator('[aria-label="Protected placement"]'),
    ).toBeVisible();
    await expect(moduleDestination.getByRole("button")).toHaveCount(0);

    const readerPolicy = await getJson<NavigationPolicyResponse>(
      fixtures.reader.context,
      "/api/admin/navigation-policy",
    );
    expect(readerPolicy.can_manage_navigation).toBe(false);
    const deniedWrite = await fixtures.reader.context.put("/api/admin/navigation-policy", {
      data: policyUpdate(readerPolicy),
    });
    expect(deniedWrite.status()).toBe(403);
    const deniedBody = (await deniedWrite.json()) as ApiErrorBody;
    expect(deniedBody.code).toBe("modules_manage_navigation_global_required");

    const descriptor = await fixtures.reader.context.get(
      `/api/admin/modules/${FORMS_DEFINITION}/descriptor`,
    );
    expect(descriptor.ok()).toBe(true);
    expect(descriptor.headers()["content-type"]).toContain("application/json");
    expect(descriptor.headers().etag).toMatch(/^"sha256:[0-9a-f]{64}"$/);
    expect((await descriptor.json()).reserved_definition_id).toBe(FORMS_DEFINITION);
    guard.assertClean();
  });

  test("scoped module authority, product-only authority, and no access stay hidden and restricted", async ({
    page,
  }) => {
    const guard = attachBrowserGuard(page);

    await signInPage(page, fixtures.scopedReader.email);
    let releaseScopedShellNavigation!: () => void;
    const heldScopedShellNavigation = new Promise<void>((resolve) => {
      releaseScopedShellNavigation = resolve;
    });
    await page.route("**/api/shell/navigation", async (route) => {
      await heldScopedShellNavigation;
      await route.continue();
    });
    await gotoHydrated(page, "/administration/modules");
    await expect(page.locator(".sidebar .account-card small")).toHaveText(
      fixtures.scopedReader.email,
    );
    await expect(
      desktopNavigation(page).getByRole("link", { name: "Module Management" }),
      "a scoped-only module grant must remain hidden while the authoritative shell projection is loading",
    ).toHaveCount(0);
    releaseScopedShellNavigation();
    await expect(
      desktopNavigation(page).getByRole("link", { name: "Module Management" }),
    ).toHaveCount(0);
    await page.unroute("**/api/shell/navigation");

    let scopedShellOutageRequests = 0;
    await page.route("**/api/shell/navigation", async (route) => {
      scopedShellOutageRequests += 1;
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: JSON.stringify({ message: "Synthetic shell navigation outage." }),
      });
    });
    await guard.whileExpectedShellNavigationOutage(async () => {
      await gotoHydrated(page, "/administration/modules");
    });
    expect(scopedShellOutageRequests).toBe(1);
    await expect(page.locator(".sidebar .account-card small")).toHaveText(
      fixtures.scopedReader.email,
    );
    await expect(
      desktopNavigation(page).getByRole("link", { name: "Module Management" }),
      "a failed shell projection must not promote a scoped-only module grant to global visibility",
    ).toHaveCount(0);
    await expect(
      desktopNavigation(page).locator(".sidebar-navigation-status"),
    ).toContainText(
      "Contribution navigation is temporarily unavailable.",
    );
    await page.unroute("**/api/shell/navigation");

    for (const [name, actor] of [
      ["scoped module read", fixtures.scopedReader],
      ["product only", fixtures.productOnly],
      ["no access", fixtures.noAccess],
    ] as const) {
      await signInPage(page, actor.email);
      await gotoHydrated(page, "/administration/modules");
      await expect(
        desktopNavigation(page).getByRole("link", { name: "Module Management" }),
      ).toHaveCount(0);

      const response = await actor.context.get("/api/admin/modules");
      expect(response.status(), `${name} inventory status`).toBe(403);
      expect(((await response.json()) as ApiErrorBody).code).toBe(
        "modules_read_global_required",
      );

      const deniedPolicyWrite = await actor.context.put("/api/admin/navigation-policy", {
        data: policyUpdate(fixtures.originalPolicy),
      });
      expect(deniedPolicyWrite.status(), `${name} policy write status`).toBe(403);
      expect(((await deniedPolicyWrite.json()) as ApiErrorBody).code).toBe(
        "modules_manage_navigation_global_required",
      );
    }

    await signInPage(page, fixtures.scopedReader.email);
    await gotoHydrated(page, "/administration/modules");
    await expect(
      page.getByRole("heading", { name: "Module Management restricted" }),
      "scoped module authority should render the restricted directory state",
    ).toBeVisible();
    await expect(page.getByText(/installation-global Module Management read access/)).toBeVisible();
    for (const definitionId of [FORMS_DEFINITION, UNKNOWN_DEFINITION]) {
      await gotoHydrated(page, `/administration/modules/${definitionId}`);
      await expect(page.getByRole("heading", { name: "Module Management restricted" })).toBeVisible();
      await expect(page.getByText("No Module Release", { exact: true })).toHaveCount(0);
      await expect(page.getByText("Source digest", { exact: true })).toHaveCount(0);
    }

    for (const actor of [fixtures.productOnly, fixtures.noAccess]) {
      await signInPage(page, actor.email);
      for (const path of [
        "/administration/modules",
        `/administration/modules/${FORMS_DEFINITION}`,
        `/administration/modules/${UNKNOWN_DEFINITION}`,
      ]) {
        await page.goto(path);
        expect(new URL(page.url()).pathname).toBe(path);
        await expect(
          page.getByRole("heading", { name: "Module Management restricted" }),
        ).toBeVisible();
        await expect(page.locator("tr[data-module-definition]")).toHaveCount(0);
        await expect(page.getByText("Source digest", { exact: true })).toHaveCount(0);
      }
    }

    await page.context().clearCookies();
    guard.allowUnauthenticatedNavigationError();
    await page.goto("/administration/modules");
    await expect(page).toHaveURL(/\/login$/);
    guard.assertClean();
  });

  test("directory and detail preserve human-machine parity and explicit route states", async ({
    page,
  }) => {
    const guard = attachBrowserGuard(page);
    await signInPage(page, fixtures.reader.email);
    const inventory = await getJson<ModuleInventoryResponse>(
      fixtures.reader.context,
      "/api/admin/modules",
    );
    const inventoryIdentities = sortedModuleIdentities(
      inventory.entries.map(moduleIdentity),
    );
    await gotoHydrated(page, "/administration/modules");

    await expect(page.locator("tr[data-module-definition]")).toHaveCount(
      inventory.entries.length,
    );
    const directoryBootstrap = await moduleBootstrap(page);
    expect(directoryBootstrap.route).toBe("directory");
    if (directoryBootstrap.route !== "directory") {
      throw new Error(`expected directory bootstrap, received ${directoryBootstrap.route}`);
    }
    expect(
      sortedModuleIdentities(directoryBootstrap.inventory.entries.map(moduleIdentity)),
      "SSR bootstrap identities and source digests must match the inventory API",
    ).toEqual(inventoryIdentities);
    const renderedIdentities = await page
      .locator("tr[data-module-definition]")
      .evaluateAll((rows) =>
        rows.map((row) => ({
          definition_id: row.getAttribute("data-module-definition") ?? "",
          source_digest:
            Array.from(row.querySelectorAll(".data-table__secondary-text"))
              .map((element) => element.textContent?.trim() ?? "")
              .find((value) => value.startsWith("sha256:")) ?? "",
        })),
      );
    expect(
      sortedModuleIdentities(renderedIdentities),
      "every rendered identity and source digest must match the inventory API",
    ).toEqual(inventoryIdentities);
    await expect(
      page.getByText("Transitional — not independently deployable", { exact: true }),
    ).toHaveCount(1);
    await expect(page.getByText("No Module Release", { exact: true })).toHaveCount(7);
    await expect(page.getByText("No Module Instance", { exact: true })).toHaveCount(7);
    await expect(
      page.locator(`tr[data-module-definition="${MIGRATION_DEFINITION}"]`),
    ).toContainText("Retired");
    await expect(page.getByText("Install module", { exact: true })).toHaveCount(0);
    await expect(page.getByText("Enable module", { exact: true })).toHaveCount(0);
    await expect(page.getByText("Create Module Release", { exact: true })).toHaveCount(0);
    await expect(page.getByText("Create Module Instance", { exact: true })).toHaveCount(0);

    await gotoHydrated(page, `/administration/modules/${FORMS_DEFINITION}`);
    await expect(page.getByRole("heading", { level: 1, name: "Forms" })).toBeVisible();
    for (const exactText of [
      "Transitional — not independently deployable",
      "No Module Release",
      "No Module Instance",
    ]) {
      await expect(page.getByText(exactText, { exact: true }).first()).toBeVisible();
    }
    const descriptorLink = page
      .locator(`a[href="/api/admin/modules/${FORMS_DEFINITION}/descriptor"]`)
      .first();
    await expect(descriptorLink).toContainText("View source descriptor (JSON)");
    await expect(descriptorLink).toHaveAttribute(
      "href",
      `/api/admin/modules/${FORMS_DEFINITION}/descriptor`,
    );
    await page.getByRole("tab", { name: "Declarations" }).click();
    for (const exactText of ["Feature Declarations", "Contracts"]) {
      await expect(page.getByText(exactText, { exact: true }).first()).toBeVisible();
    }
    await page.getByRole("tab", { name: "Dependencies" }).click();
    for (const dimension of ["dependency", "compatibility"]) {
      await expect(page.locator(`[data-module-dimension="${dimension}"]`)).toBeVisible();
    }
    await page.getByRole("tab", { name: "Findings" }).click();
    for (const dimension of ["configuration", "readiness", "health"]) {
      await expect(page.locator(`[data-module-dimension="${dimension}"]`)).toBeVisible();
    }
    await expect(page.locator('[aria-labelledby="module-findings-heading"]')).toBeVisible();
    await page.getByRole("tab", { name: "Capabilities" }).click();
    await expect(page.getByRole("heading", { name: "Capabilities", exact: true })).toBeVisible();
    await page.getByRole("tab", { name: "Resources" }).click();
    await expect(
      page.getByRole("heading", { name: "Resources/Destinations", exact: true }),
    ).toBeVisible();
    await page.getByRole("tab", { name: "Navigation" }).click();
    await expect(page.locator("section.module-navigation-policy")).toBeVisible();
    await page.getByRole("tab", { name: "Dependencies" }).click();
    for (const dimension of [
      "dependency",
      "compatibility",
      "configuration",
      "readiness",
      "health",
    ]) {
      await expect(page.locator(`[data-module-dimension="${dimension}"]`)).toHaveCount(1);
    }
    await expect(
      page
        .locator("[data-module-dimension]")
        .getByText("Not applicable — no Module Release/Instance", { exact: true }),
    ).toHaveCount(4);
    await expect(
      page
        .locator('[data-module-dimension="dependency"]')
        .getByText("No functional dependencies declared", { exact: true }),
    ).toBeVisible();
    const healthDimension = page.locator('[data-module-dimension="health"]');
    await expect(healthDimension).toContainText(
      "No Module Instance exists, so Core does not evaluate or infer health.",
    );
    expect(await healthDimension.textContent()).not.toMatch(/\b(?:healthy|unhealthy)\b/i);

    await gotoHydrated(page, `/administration/modules/${RESPONSES_DEFINITION}`);
    await expect(page.getByRole("heading", { level: 1, name: "Responses" })).toBeVisible();
    await page.getByRole("tab", { name: "Dependencies" }).click();
    const responseDependencies = page.locator(".module-detail-dependencies");
    await expect(
      responseDependencies.getByText("Transition-internal only", { exact: true }),
    ).toBeVisible();
    await expect(responseDependencies).toContainText(
      "2 declared relationships describe current in-process coupling and cannot be satisfied by a transition contribution provider.",
    );
    await expect(page.getByText("transition_internal_only", { exact: true }).first()).toBeVisible();

    await gotoHydrated(page, `/administration/modules/${MIGRATION_DEFINITION}`);
    await expect(page.getByRole("heading", { level: 1, name: "Migration" })).toBeVisible();
    await expect(page.getByRole("heading", { name: "Contribution retired" })).toBeVisible();
    await expect(
      page.getByText(
        "The roadmap identity is retired and no current in-process product surface exists.",
        { exact: true },
      ),
    ).toBeVisible();

    for (const inventoryEntry of inventory.entries) {
      const definitionId = moduleIdentity(inventoryEntry).definition_id;
      const detail = await getJson<ModuleDetailResponse>(
        fixtures.reader.context,
        `/api/admin/modules/${definitionId}`,
      );
      expect(detail.entry, `${definitionId} detail must match its inventory entry`).toEqual(
        inventoryEntry,
      );

      await gotoHydrated(page, `/administration/modules/${definitionId}`);
      const detailBootstrap = await moduleBootstrap(page);
      expect(detailBootstrap.route, `${definitionId} must emit a detail bootstrap`).toBe(
        "detail",
      );
      if (detailBootstrap.route !== "detail") {
        throw new Error(`expected detail bootstrap for ${definitionId}`);
      }
      expect(
        detailBootstrap.detail,
        `${definitionId} SSR bootstrap must match its detail API projection`,
      ).toEqual(detail);

      if (detail.entry.kind === "transitional_in_process") {
        const overview = page.locator(
          'section.organization-detail-card[aria-labelledby="module-overview-heading"]',
        );
        await expect(overview.getByText(definitionId, { exact: true })).toBeVisible();
        await expectRenderedModuleDetailMatchesProjection(page, detail.entry);
      } else {
        await expect(
          page.getByRole("heading", {
            level: 1,
            name: detail.entry.definition.display_name,
            exact: true,
          }),
        ).toBeVisible();
        await expect(page.getByText(definitionId, { exact: true }).first()).toBeVisible();
        await expect(
          page.getByText("Independently deployed", { exact: true }),
        ).toBeVisible();
        await expect(page.getByText("Healthy and enabled", { exact: true })).toBeVisible();
      }

      const descriptorResponse = await fixtures.reader.context.get(
        `/api/admin/modules/${definitionId}/descriptor`,
      );
      expect(descriptorResponse.ok(), `${definitionId} descriptor response`).toBe(true);
      const descriptorBytes = await descriptorResponse.body();
      const descriptorDigest = `sha256:${createHash("sha256")
        .update(descriptorBytes)
        .digest("hex")}`;
      expect(
        descriptorResponse.headers().etag,
        `${definitionId} ETag must quote the exact source digest as an HTTP entity tag`,
      ).toBe(`"${moduleIdentity(inventoryEntry).source_digest}"`);
      expect(
        descriptorDigest,
        `${definitionId} descriptor bytes must hash to the rendered source digest`,
      ).toBe(moduleIdentity(inventoryEntry).source_digest);
      if (detail.entry.kind === "transitional_in_process") {
        expect(
          JSON.parse(descriptorBytes.toString("utf8")),
          `${definitionId} descriptor bytes must decode to the detail descriptor`,
        ).toEqual(detail.entry.descriptor);
      } else {
        expect(
          JSON.parse(descriptorBytes.toString("utf8")),
          `${definitionId} descriptor bytes must decode to the persisted manifest`,
        ).toEqual(detail.entry.manifest);
      }
    }

    await gotoHydrated(page, `/administration/modules/${UNKNOWN_DEFINITION}`);
    await expect(page.getByRole("heading", { name: "Module definition not found" })).toBeVisible();
    await expect(
      page.getByText("No transition contribution exists for this definition identifier.", {
        exact: true,
      }),
    ).toBeVisible();
    expect(await page.content()).not.toContain("/bridge/");

    expect(
      guard.moduleDataRequests,
      "SSR hydration must not issue duplicate inventory or detail API loads",
    ).toEqual([]);
    expect(
      guard.navigationPolicyRequests,
      "SSR hydration must not issue a duplicate navigation-policy API load",
    ).toEqual([]);
    guard.assertClean();

    guard.resetDataRequests();
    await gotoHydrated(page, "/");
    let releaseInventory!: () => void;
    const heldInventory = new Promise<void>((resolve) => {
      releaseInventory = resolve;
    });
    await page.route("**/api/admin/modules", async (route) => {
      await heldInventory;
      await route.continue();
    });
    await desktopNavigation(page).getByRole("link", { name: "Module Management" }).click();
    const loadingState = page.locator('section[aria-busy="true"]');
    await expect(
      loadingState.getByRole("heading", { name: "Loading module inventory", exact: true }),
    ).toBeVisible();
    await expect(loadingState).toContainText(
      "Fetching the authorized installation inventory and navigation policy.",
    );
    releaseInventory();
    await expect(page.locator("tr[data-module-definition]")).toHaveCount(
      inventory.entries.length,
    );
    await page.getByRole("tab", { name: "Navigation" }).click();
    await expect(page.getByText("Read-only", { exact: true })).toBeVisible();
    await page.unroute("**/api/admin/modules");
    expect(guard.moduleDataRequests).toEqual(["/api/admin/modules"]);
    expect(guard.navigationPolicyRequests).toEqual(["/api/admin/navigation-policy"]);

    guard.resetDataRequests();
    await gotoHydrated(page, "/");
    await page.route("**/api/admin/modules", async (route) => {
      await route.fulfill({
        status: 503,
        contentType: "application/json",
        body: JSON.stringify({ message: "Synthetic module inventory outage." }),
      });
    });
    await desktopNavigation(page).getByRole("link", { name: "Module Management" }).click();
    const unavailableState = page.locator(
      'section.module-management-unavailable[role="status"]',
    );
    await expect(
      unavailableState.getByRole("heading", {
        name: "Module Management unavailable",
        exact: true,
      }),
    ).toBeVisible();
    await expect(unavailableState).toContainText("Synthetic module inventory outage.");
    await page.unroute("**/api/admin/modules");
    expect(guard.moduleDataRequests).toEqual(["/api/admin/modules"]);
    expect(guard.navigationPolicyRequests).toEqual([]);

    guard.resetDataRequests();
    await gotoHydrated(page, "/");
    await page.route("**/api/admin/modules", async (route) => {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: "{",
      });
    });
    await desktopNavigation(page).getByRole("link", { name: "Module Management" }).click();
    const errorState = page.locator('section.organization-state.is-error[role="alert"]');
    await expect(
      errorState.getByRole("heading", {
        name: "Unable to load Module Management",
        exact: true,
      }),
    ).toBeVisible();
    await expect(errorState).toContainText("Module inventory response could not be read.");
    await page.unroute("**/api/admin/modules");
    expect(guard.moduleDataRequests).toEqual(["/api/admin/modules"]);
    expect(guard.navigationPolicyRequests).toEqual([]);
    expect(
      guard.bridgeRequests,
      "fault rendering must remain native and must not fall back to /bridge/*",
    ).toEqual([]);
  });

  test("keyboard group and placement edits retain focus and persist in desktop and mobile shells", async ({
    page,
  }) => {
    const guard = attachBrowserGuard(page);
    await preparePolicyScenario();
    try {
      await page.setViewportSize({ width: 1280, height: 900 });

      await signInPage(page, fixtures.noAccess.email);
      await gotoHydrated(page, "/administration/modules");
      await expect(desktopNavigation(page).getByRole("link", { name: "Forms" })).toHaveCount(0);
      await guard.whileExpectedFormsRouteDenial(async () => {
        await page.goto("/forms");
        await expect(page).toHaveURL(/\/$/);
        await expect(page.getByRole("heading", { level: 1, name: "Home" })).toBeVisible();
      });

      await signInPage(page, fixtures.manager.email);
      await gotoHydrated(page, "/administration/modules");
      await page.getByRole("tab", { name: "Navigation" }).click();

      const policy = page.locator(".module-navigation-policy");
      await expect(policy.getByText("Read-only", { exact: true })).toHaveCount(0);
      await expect(policy.getByRole("button", { name: "Save navigation" })).toBeDisabled();
      await expect(policy.getByRole("button", { name: "Discard changes" })).toBeDisabled();
      const adminGroup = policy
        .locator("details.module-navigation-group")
        .filter({ hasText: "core.admin" });
      await expect(adminGroup).toHaveCount(1);
      await adminGroup.locator("summary").click();
      await expect(
        policy.locator('[data-navigation-destination="core.admin.modules"]'),
      ).toContainText("Module Management");
      await expect(
        policy.locator('[data-navigation-destination="core.admin.modules"]')
          .getByLabel("Protected placement"),
      ).toBeVisible();

      const policyBeforeCoreMutation = await getJson<NavigationPolicyResponse>(
        fixtures.manager.context,
        "/api/admin/navigation-policy",
      );
      const protectedDestinations = policyBeforeCoreMutation.destinations.map((item) => ({
        ...item,
        visible: item.id === "core.admin.modules" ? false : item.visible,
      }));
      const coreMutation = policyUpdate(
        policyBeforeCoreMutation,
        policyBeforeCoreMutation.groups,
        protectedDestinations,
      );
      const coreMutationResponse = await fixtures.manager.context.put(
        "/api/admin/navigation-policy",
        { data: coreMutation },
      );
      expect(coreMutationResponse.status()).toBe(400);
      expect(((await coreMutationResponse.json()) as ApiErrorBody).code).toBe(
        "navigation_policy_destination_protected",
      );
      const policyAfterCoreMutation = await getJson<NavigationPolicyResponse>(
        fixtures.manager.context,
        "/api/admin/navigation-policy",
      );
      expect(policyAfterCoreMutation.revision).toBe(policyBeforeCoreMutation.revision);
      expect(
        samePolicyValues(policyAfterCoreMutation, policyBeforeCoreMutation),
      ).toBe(true);

      await policy.getByRole("button", { name: "Add group" }).focus();
      await page.keyboard.press("Enter");
      const customGroup = policy.locator(
        "details.module-navigation-group",
      ).filter({ hasText: "Custom group" }).last();
      await expect(customGroup).toBeVisible();
      await customGroup.getByRole("button", { name: /Open actions for/ }).click();
      await page.getByRole("menuitem", { name: "Rename group" }).click();
      const groupName = page.getByLabel("Group name");
      await groupName.fill("Insights");
      await page.getByRole("button", { name: "Save name" }).click();

      const formsRow = policy.locator(
        '[data-navigation-destination="tessara.forms.navigation"]',
      );
      const moveFormsToGroup = formsRow.getByLabel("Move Forms to group");
      await moveFormsToGroup.selectOption({ label: "Insights" });
      await expect(
        moveFormsToGroup,
        "focus should remain on the cross-group control after moving Forms",
      ).toBeFocused();
      await expect(customGroup).toContainText("Forms");

      const save = policy.getByRole("button", { name: "Save navigation" });
      await expect(save).toBeEnabled();
      await save.focus();
      await page.keyboard.press("Enter");
      await expect(
        policy.getByRole("heading", { name: "Navigation saved", exact: true }),
      ).toBeVisible();
      await expect(save).toBeDisabled();

      await page.reload();
      await expect(page.locator("#app-root")).toHaveAttribute("data-hydration", "ready");
      await page.getByRole("tab", { name: "Navigation" }).click();
      const persistedCustomGroup = policy.locator(
        "details.module-navigation-group",
      ).filter({ hasText: "Custom group" });
      await expect(persistedCustomGroup).toContainText("Insights");
      await expect(persistedCustomGroup).toContainText("Forms");
      const persistedPolicy = await getJson<NavigationPolicyResponse>(
        fixtures.manager.context,
        "/api/admin/navigation-policy",
      );
      const persistedForms = persistedPolicy.destinations.find(
        (destination) => destination.id === "tessara.forms.navigation",
      );
      const insights = persistedPolicy.groups.find((group) => group.label === "Insights");
      expect(insights).toBeTruthy();
      expect(persistedForms?.group_id).toBe(insights?.id);

      await signInPage(page, "admin@tessara.local", "tessara-dev-admin");
      await gotoHydrated(page, "/");
      const adminDesktopNavigation = desktopNavigation(page);
      await expect(
        adminDesktopNavigation.locator(".sidebar-section", { hasText: /^Insights$/ }),
      ).toBeVisible();
      await expect(adminDesktopNavigation.getByRole("link", { name: "Forms" })).toBeVisible();

      await gotoHydrated(page, "/forms");
      await expect(page.getByRole("heading", { level: 1, name: "Forms" })).toBeVisible();
      await expect(desktopNavigation(page).getByRole("link", { name: "Forms" })).toBeVisible();

      await page.setViewportSize({ width: 390, height: 844 });
      await gotoHydrated(page, "/");
      await page.getByRole("button", { name: "Open navigation" }).click();
      const mobile = page.locator(".mobile-nav__panel");
      await expect(mobile).toBeVisible();
      await expect(
        mobile.locator(".sidebar-section", { hasText: /^Insights$/ }),
      ).toBeVisible();
      await expect(mobile.getByRole("link", { name: "Forms" })).toBeVisible();
      expect(
        await page.evaluate(
          () => document.documentElement.scrollWidth <= document.documentElement.clientWidth,
        ),
        "mobile Module Management shell should not overflow horizontally",
      ).toBe(true);
      guard.assertClean();
    } finally {
      await restoreOriginalPolicy();
    }
  });

  test("native directory and detail remain useful without JavaScript or bridge requests", async ({
    browser,
  }) => {
    const context = await browser.newContext({
      baseURL: BASE_URL,
      javaScriptEnabled: false,
      viewport: { width: 1280, height: 900 },
    });
    try {
      const page = await context.newPage();
      const assertNativeRouteGuard = attachNativeRouteGuard(page);
      const login = await page.request.post("/api/auth/login", {
        data: { email: fixtures.reader.email, password: PASSWORD },
      });
      const body = await expectJson<{ token: string }>(login);
      await context.addCookies([
        {
          name: "tessara_session",
          value: body.token,
          url: BASE_URL,
          httpOnly: true,
          sameSite: "Lax",
        },
      ]);

      await expectNoJavaScriptNativeRouteDirectLoadAndRefresh(page, {
        path: "/administration/modules",
        ready: async (routePage) => {
          const routeContent = routePage.locator(
            ".route-panel.module-management-page",
          );
          await expect(routeContent).toHaveCount(1);
          await expect(
            routeContent.getByRole("heading", {
              level: 1,
              name: "Module Management",
              exact: true,
            }),
          ).toBeVisible();
          await expect(
            routeContent.locator("tr[data-module-definition]"),
          ).toHaveCount(8);
          await expect(
            routeContent.getByText("Read-only", { exact: true }),
          ).toBeVisible();
        },
      });

      await expectNoJavaScriptNativeRouteDirectLoadAndRefresh(page, {
        path: `/administration/modules/${FORMS_DEFINITION}`,
        ready: async (routePage) => {
          const routeContent = routePage.locator(
            ".route-panel.module-management-detail-page",
          );
          await expect(routeContent).toHaveCount(1);
          await expect(
            routeContent.getByRole("heading", {
              level: 1,
              name: "Forms",
              exact: true,
            }),
          ).toBeVisible();
          await expect(
            routeContent.getByText(
              "Transitional — not independently deployable",
              { exact: true },
            ),
          ).toBeVisible();
        },
      });

      await expectNoJavaScriptNativeRouteDirectLoadAndRefresh(page, {
        path: `/administration/modules/${UNKNOWN_DEFINITION}`,
        ready: async (routePage) => {
          const routeContent = routePage.locator(
            ".route-panel.module-management-detail-page",
          );
          await expect(routeContent).toHaveCount(1);
          await expect(
            routeContent.getByRole("heading", {
              name: "Module definition not found",
              exact: true,
            }),
          ).toBeVisible();
        },
      });

      await assertNativeRouteGuard();
    } finally {
      await context.close();
    }
  });
});
