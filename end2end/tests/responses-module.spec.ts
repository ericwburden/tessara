import {
  expect,
  test,
  type APIResponse,
  type Page,
  type Response,
} from "@playwright/test";

const RESPONSES_DEFINITION = "tessara.responses";
const RETIRED_RESPONSE_BROWSER_PATHS = [
  "/api/submissions",
  "/api/responses/options",
  "/api/responses/start",
] as const;
const RESPONSE_DIAGNOSTIC_SECTIONS = [
  "Overview",
  "Configuration",
  "Declarations",
  "Contracts",
  "Capabilities",
  "Dependencies",
  "Resources",
  "Navigation",
  "Findings",
] as const;

type ResponseSummary = {
  id: string;
  form_name: string;
  node_name: string;
  status: "draft" | "submitted";
};

type ResponseDetail = ResponseSummary & {
  revision: number;
  values: Array<{ key: string; value: unknown }>;
};

type ResponseStartOptions = {
  assignments: Array<{
    workflow_assignment_id: string;
    form_name: string;
    node_name: string;
  }>;
};

type ModuleDetail = {
  entry: {
    kind: string;
    release: { version: string };
    instance: { ready: boolean; enabled: boolean; healthy: boolean };
    configuration: {
      declared: boolean;
      valid: boolean;
      values: Record<string, unknown>;
    };
    manifest: Record<string, unknown> | null;
    findings: unknown[];
  };
};

async function expectJson<T>(response: APIResponse | Response): Promise<T> {
  const body = await response.text();
  expect(
    response.ok(),
    `${response.status()} ${response.url()}: ${body}`,
  ).toBeTruthy();
  return JSON.parse(body) as T;
}

async function signIn(
  page: Page,
  email = "admin@tessara.local",
  password = "tessara-dev-admin",
) {
  const response = await page.request.post("/api/auth/login", {
    data: { email, password },
  });
  expect(response.ok(), await response.text()).toBeTruthy();
}

function requireResponse(
  responses: ResponseSummary[],
  predicate: (response: ResponseSummary) => boolean,
  label: string,
) {
  const response = responses.find(predicate);
  expect(response, label).toBeDefined();
  return response!;
}

function isRetiredResponseApiPath(path: string) {
  return RETIRED_RESPONSE_BROWSER_PATHS.some(
    (retired) => path === retired || path.startsWith(`${retired}/`),
  );
}

function isAllowedResponseDocumentApiPath(path: string) {
  return (
    path === "/api/auth/session" ||
    path === "/api/shell/navigation" ||
    path === "/api/responses" ||
    path === "/api/responses/start-options" ||
    /^\/api\/responses\/[0-9a-f-]{36}(?:\/(?:values|submit))?$/i.test(path)
  );
}

async function responseRouteParity(page: Page) {
  return page.evaluate(() => {
    const responseSurface = document.querySelector<HTMLElement>(".tessara-response");
    const activeResponseLinks = Array.from(
      document.querySelectorAll<HTMLAnchorElement>(
        '.sidebar-link.is-active[href="/responses"]',
      ),
    );
    return {
      title: document.title,
      topBarTitle:
        document.querySelector<HTMLElement>(".top-app-bar__title")?.innerText.trim() ??
        "",
      heading: responseSurface?.querySelector("h1")?.textContent?.trim() ?? "",
      responseText: responseSurface?.innerText.replace(/\s+/g, " ").trim() ?? "",
      activeResponseLinkCount: activeResponseLinks.length,
      hasApplicationCanvas:
        document.querySelector('.app-main[aria-label="Application content"]') !== null,
      moduleScope: document.body.classList.contains("module-scope--tessara-responses"),
      documentClientWidth: document.documentElement.clientWidth,
      documentScrollWidth: document.documentElement.scrollWidth,
      desktopTableVisible:
        document.querySelector<HTMLElement>(".responses-list .table-wrap")?.offsetParent !==
        null,
      mobileCardsVisible:
        document.querySelector<HTMLElement>(".responses-mobile-cards")?.offsetParent !==
        null,
    };
  });
}

const consoleErrors = new WeakMap<Page, string[]>();

test.describe("Sprint 8C independent Response module", () => {
  test.beforeEach(async ({ page }) => {
    const errors: string[] = [];
    consoleErrors.set(page, errors);
    page.on("console", (message) => {
      if (message.type() === "error") errors.push(message.text());
    });
    page.on("pageerror", (error) => errors.push(error.message));
    await signIn(page);
  });

  test.afterEach(async ({ page }) => {
    const errors = consoleErrors.get(page) ?? [];
    expect(
      errors,
      `Response routes must keep the browser console clean:\n${errors.join("\n")}`,
    ).toEqual([]);
  });

  test("direct documents use only Response-owned public browser routes", async ({
    page,
  }) => {
    const responses = await expectJson<ResponseSummary[]>(
      await page.request.get("/api/responses"),
    );
    const draft = requireResponse(
      responses,
      (response) => response.status === "draft",
      "Reference Response draft",
    );
    const observedProductRequests = new Set<string>();
    const forbiddenProductRequests: string[] = [];
    const externalRequests: string[] = [];
    let applicationOrigin: string | undefined;

    page.on("request", (request) => {
      const url = new URL(request.url());
      if (request.isNavigationRequest() && request.frame() === page.mainFrame()) {
        applicationOrigin ??= url.origin;
      }
      if (
        applicationOrigin !== undefined &&
        (url.protocol === "http:" || url.protocol === "https:") &&
        url.origin !== applicationOrigin
      ) {
        externalRequests.push(request.url());
      }
      if (url.pathname.startsWith("/api/")) {
        observedProductRequests.add(url.pathname);
      }
      if (
        url.pathname.startsWith("/api/") &&
        (isRetiredResponseApiPath(url.pathname) ||
          !isAllowedResponseDocumentApiPath(url.pathname))
      ) {
        forbiddenProductRequests.push(url.pathname);
      }
    });

    for (const route of [
      "/responses",
      `/responses/${draft.id}`,
      `/responses/${draft.id}/edit`,
    ]) {
      await page.goto(route);
      await expect(page.locator("#module-content")).toHaveAttribute(
        "data-hydration",
        "ready",
      );
      await expect(page.locator(".top-app-bar__title")).toContainText(/Responses?|Edit Response/);
    }

    expect([...observedProductRequests]).toEqual(
      expect.arrayContaining([
        "/api/responses",
        `/api/responses/${draft.id}`,
      ]),
    );
    expect(
      [...observedProductRequests].every(isAllowedResponseDocumentApiPath),
      `Response documents used a non-owner product API: ${[
        ...observedProductRequests,
      ].join(", ")}`,
    ).toBe(true);
    expect(forbiddenProductRequests).toEqual([]);
    expect(externalRequests).toEqual([]);
  });

  test("assignment-only start options reject retired Core start routes", async ({
    page,
  }) => {
    const options = await expectJson<ResponseStartOptions>(
      await page.request.get("/api/responses/start-options"),
    );
    expect(Array.isArray(options.assignments)).toBe(true);
    for (const assignment of options.assignments) {
      expect(assignment.workflow_assignment_id).toMatch(
        /^[0-9a-f]{8}-[0-9a-f]{4}-[1-8][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i,
      );
    }

    const observed: string[] = [];
    const forbiddenObserved: string[] = [];
    page.on("request", (request) => {
      const path = new URL(request.url()).pathname;
      if (path.startsWith("/api/")) {
        observed.push(path);
        if (isRetiredResponseApiPath(path) || !isAllowedResponseDocumentApiPath(path)) {
          forbiddenObserved.push(path);
        }
      }
    });
    await page.goto("/responses/new");
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    await expect(
      page.getByRole("heading", { name: "Start Response", exact: true }),
    ).toBeVisible();
    expect(observed).toContain("/api/responses/start-options");
    expect(
      forbiddenObserved,
      `Response start used a retired or non-owner product API: ${forbiddenObserved.join(
        ", ",
      )}`,
    ).toEqual([]);

    const retired = await Promise.all([
      page.request.get(RETIRED_RESPONSE_BROWSER_PATHS[0]),
      page.request.get(RETIRED_RESPONSE_BROWSER_PATHS[1]),
      page.request.post(RETIRED_RESPONSE_BROWSER_PATHS[2], {
        data: {
          form_version_id: "00000000-0000-0000-0000-000000000000",
          node_id: "00000000-0000-0000-0000-000000000000",
        },
      }),
    ]);
    expect(retired[0].status()).toBe(404);
    expect([400, 404]).toContain(retired[1].status());
    expect([404, 405]).toContain(retired[2].status());
  });

  test("lifecycle navigation preserves unsaved draft state when discard is declined", async ({
    page,
  }) => {
    const responses = await expectJson<ResponseSummary[]>(
      await page.request.get("/api/responses"),
    );
    const draft = requireResponse(
      responses,
      (response) => response.status === "draft",
      "Reference Response draft",
    );

    await page.setViewportSize({ width: 1024, height: 768 });
    await page.goto("/responses");
    await expect(page.locator("#module-content")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    await expect(page.locator(".responses-list .table-wrap")).toBeVisible();
    await expect(page.locator(`a[href="/responses/${draft.id}"]`).first()).toBeVisible();
    const direct = await responseRouteParity(page);
    expect(direct).toMatchObject({
      title: "Responses · Tessara",
      topBarTitle: "Responses",
      heading: "Responses",
      hasApplicationCanvas: true,
      moduleScope: true,
      desktopTableVisible: true,
      mobileCardsVisible: false,
    });
    expect(direct.activeResponseLinkCount).toBeGreaterThan(0);
    expect(direct.documentScrollWidth).toBeLessThanOrEqual(
      direct.documentClientWidth + 1,
    );

    await page.goto("/");
    await expect(page.locator("#app-root")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    await page.getByRole("link", { name: "Responses", exact: true }).click();
    await expect(page).toHaveURL(/\/responses$/);
    await expect(page.locator("#tessara-module-outlet")).toBeVisible();
    await expect(page.locator(".responses-list .table-wrap")).toBeVisible();
    await expect(page.locator(`a[href="/responses/${draft.id}"]`).first()).toBeVisible();
    const lifecycleOutlet = page.locator("#tessara-module-outlet");
    await expect(lifecycleOutlet).toHaveAttribute("aria-busy", "false");
    await expect(lifecycleOutlet).toHaveAttribute(
      "data-module-definition",
      RESPONSES_DEFINITION,
    );
    await expect(lifecycleOutlet).toHaveAttribute("data-module-release", "1.0.0");
    const lifecycle = await responseRouteParity(page);
    expect(lifecycle).toEqual(direct);

    await page.locator(`a[href="/responses/${draft.id}"]`).first().click();
    await expect(page).toHaveURL(new RegExp(`/responses/${draft.id}$`));
    await page.getByRole("link", { name: "Edit Draft", exact: true }).click();
    await expect(page).toHaveURL(new RegExp(`/responses/${draft.id}/edit$`));

    const editor = page.locator(".response-edit-form");
    await expect(editor).toBeVisible();
    const editableField = editor.locator('input:not([type="checkbox"]), textarea').first();
    await expect(editableField).toBeEditable();
    const dirtyValue = `Unsaved Response ${Date.now()}`;
    await editableField.fill(dirtyValue);

    await Promise.all([
      page.waitForEvent("dialog").then(async (dialog) => {
        expect(dialog.type()).toBe("confirm");
        expect(dialog.message()).toBe("Discard unsaved Response changes?");
        await dialog.dismiss();
      }),
      page.getByRole("link", { name: "Home", exact: true }).click(),
    ]);
    await expect(page).toHaveURL(new RegExp(`/responses/${draft.id}/edit$`));
    await expect(editableField).toHaveValue(dirtyValue);
  });

  test("scoped review and module diagnostics remain explicit and nondisclosing", async ({
    page,
  }) => {
    const allResponses = await expectJson<ResponseSummary[]>(
      await page.request.get("/api/responses"),
    );
    const disjoint = requireResponse(
      allResponses,
      (response) => response.form_name === "Disjoint Responses",
      "Disjoint Response fixture",
    );

    await signIn(
      page,
      "response-manager@tessara.local",
      "sprint-8c-response-manager",
    );
    const scopedResponses = await expectJson<ResponseSummary[]>(
      await page.request.get("/api/responses"),
    );
    expect(scopedResponses.length).toBeGreaterThan(0);
    expect(scopedResponses.every((response) =>
      ["Restricted Division", "Confidential Team"].includes(response.node_name),
    )).toBe(true);
    expect(scopedResponses.some((response) => response.id === disjoint.id)).toBe(false);

    const knownForbidden = await page.request.get(`/api/responses/${disjoint.id}`);
    const randomForbidden = await page.request.get(
      "/api/responses/40000000-0000-4000-8000-00000000008c",
    );
    expect(knownForbidden.status()).toBe(404);
    expect(randomForbidden.status()).toBe(404);
    expect(await knownForbidden.text()).toBe(await randomForbidden.text());

    await signIn(page);
    const module = await expectJson<ModuleDetail>(
      await page.request.get(`/api/admin/modules/${RESPONSES_DEFINITION}`),
    );
    expect(module.entry.kind).toBe("independently_deployed");
    expect(module.entry.release.version).toBe("1.0.0");
    expect(module.entry.instance).toMatchObject({
      ready: true,
      enabled: true,
      healthy: true,
    });
    expect(module.entry.configuration).toEqual({
      declared: true,
      valid: true,
      values: {
        schema_version: 1,
        display_label: "Responses",
        provider_request_timeout_seconds: 5,
        workflow_event_page_size: 250,
      },
    });
    expect(module.entry.manifest).not.toBeNull();
    expect(module.entry.manifest?.browser_lifecycle).toEqual({
      lifecycle_abi: "1.0.0",
      entry_asset: "/response.js",
      stylesheet_assets: ["/response.css"],
      complete_document_fallback: true,
      capabilities: {
        navigation_guard: true,
        suspend_resume: true,
      },
    });
    expect(module.entry.findings).toEqual([]);
    const diagnosticProjection = JSON.stringify(module);
    const forbiddenDiagnosticValues = [
      disjoint.id,
      "response-manager@tessara.local",
      "postgres://",
      "local-response-runtime",
      "development-module-control-only",
      "MzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzM",
      "authorization digest",
      "opaque cursor",
    ];
    for (const forbidden of forbiddenDiagnosticValues) {
      expect(
        diagnosticProjection,
        `Response module API diagnostics disclosed '${forbidden}'`,
      ).not.toContain(forbidden);
    }

    await page.goto(`/administration/modules/${RESPONSES_DEFINITION}`);
    await expect(page.locator("#app-root")).toHaveAttribute(
      "data-hydration",
      "ready",
    );
    await expect(
      page.getByRole("heading", { level: 1, name: "Responses", exact: true }),
    ).toBeVisible();
    await expect(page.getByText("Independently deployed", { exact: true })).toBeVisible();
    await expect(page.getByText("Healthy and enabled", { exact: true })).toBeVisible();
    for (const section of RESPONSE_DIAGNOSTIC_SECTIONS) {
      await page.getByRole("tab", { name: section, exact: true }).click();
      const surface = page.locator(".module-detail-sections");
      await expect(surface).toHaveAttribute(
        "data-active-section",
        section.toLowerCase(),
      );
      const surfaceText = await surface.innerText();
      for (const forbidden of forbiddenDiagnosticValues) {
        expect(
          surfaceText,
          `Response Module Management ${section} disclosed '${forbidden}'`,
        ).not.toContain(forbidden);
      }
    }
    await page.getByRole("tab", { name: "Dependencies", exact: true }).click();
    const dependencyText = await page.locator(".module-detail-dependencies").innerText();
    expect(dependencyText).toContain("tessara.forms.form-version-schema");
    expect(dependencyText).toContain("tessara.workflows.response-context");
    expect(dependencyText).toContain("tessara.workflows.response-assignment-catalog");
  });
});
