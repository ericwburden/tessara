import { expect, type Page, type Response } from "@playwright/test";

const DESKTOP_VIEWPORT = { width: 1440, height: 1000 };
const MOBILE_VIEWPORT = { width: 390, height: 844 };
const EXPECTED_FORBIDDEN_RESOURCE_CONSOLE =
  "Failed to load resource: the server responded with a status of 403 (Forbidden)";

type ExpectedForbiddenGet = {
  path: string;
  count: number;
};

type ExpectedHttpErrorScope = {
  consoleIdentities: string[];
  responseIdentities: string[];
};

type ReadyAssertion = (page: Page) => Promise<void>;

type ShellRouteExpectation = {
  path: string;
  shellTitle: string;
  activeHref: string;
  ready: ReadyAssertion;
};

type StandaloneRouteExpectation = {
  path: string;
  rootSelector: string;
  ready: ReadyAssertion;
};

type NativeRouteExpectation = {
  path: string;
  expectedRootMarkup?: string;
  documentRootSelector?: string;
  ready: ReadyAssertion;
};

type NoJavaScriptNativeRouteExpectation = {
  path: string;
  expectedRootMarkup?: string;
  documentRootSelector?: string;
  ready: ReadyAssertion;
};

function isHydrationDiagnostic(message: string) {
  return /\b(?:hydrat(?:e|ed|ing|ion)|mismatch)\b/i.test(message);
}

export function attachNativeRouteGuard(page: Page) {
  const consoleFailures: string[] = [];
  const bridgeRequests: string[] = [];
  let expectedHttpErrorScope: ExpectedHttpErrorScope | null = null;

  page.on("request", (request) => {
    if (new URL(request.url()).pathname.startsWith("/bridge/")) {
      bridgeRequests.push(request.url());
    }
  });
  page.on("response", (response) => {
    if (expectedHttpErrorScope !== null && response.status() >= 400) {
      const request = response.request();
      const url = new URL(response.url());
      expectedHttpErrorScope.responseIdentities.push(
        `${request.method()} ${url.pathname}${url.search} ${response.status()}`,
      );
    }
  });
  page.on("console", (message) => {
    const text = message.text();
    if (message.type() === "error" && expectedHttpErrorScope !== null) {
      const locationUrl = message.location().url;
      let location = "<unknown>";
      try {
        const url = new URL(locationUrl);
        location = `${url.pathname}${url.search}`;
      } catch {
        // Preserve an unparseable or missing location as an exact failing identity.
      }
      expectedHttpErrorScope.consoleIdentities.push(`${location} :: ${text}`);
    } else if (
      message.type() === "error" ||
      (message.type() === "warning" && isHydrationDiagnostic(text))
    ) {
      consoleFailures.push(`${message.type()}: ${text}`);
    }
  });
  page.on("pageerror", (error) => {
    consoleFailures.push(`pageerror: ${error.message}`);
  });

  const assertClean = async () => {
    expect(
      bridgeRequests,
      `native routes must not request /bridge/*: ${bridgeRequests.join("\n")}`,
    ).toEqual([]);
    expect(
      consoleFailures,
      `browser console and hydration diagnostics must stay clean: ${consoleFailures.join("\n")}`,
    ).toEqual([]);
  };
  assertClean.whileExpectedForbiddenGets = async (
    expected: ReadonlyArray<ExpectedForbiddenGet>,
    run: () => Promise<void>,
  ) => {
    expect(
      expectedHttpErrorScope,
      "expected HTTP-error scopes must not be nested",
    ).toBeNull();
    const scope: ExpectedHttpErrorScope = {
      consoleIdentities: [],
      responseIdentities: [],
    };
    expectedHttpErrorScope = scope;
    try {
      await run();
    } finally {
      await page.waitForLoadState("networkidle", { timeout: 5_000 }).catch(() => {});
      let stableIntervals = 0;
      while (stableIntervals < 3) {
        const consoleCount = scope.consoleIdentities.length;
        const responseCount = scope.responseIdentities.length;
        await page.waitForTimeout(50);
        stableIntervals =
          scope.consoleIdentities.length === consoleCount &&
          scope.responseIdentities.length === responseCount
            ? stableIntervals + 1
            : 0;
      }
      expectedHttpErrorScope = null;
    }

    const expectedResponses = expected
      .flatMap(({ path, count }) => Array(count).fill(`GET ${path} 403`))
      .sort();
    const expectedConsoleIdentities = expected
      .flatMap(({ path, count }) =>
        Array(count).fill(`${path} :: ${EXPECTED_FORBIDDEN_RESOURCE_CONSOLE}`),
      )
      .sort();
    expect(
      scope.responseIdentities.sort(),
      "expected forbidden GETs must be the only HTTP failures in the scope",
    ).toEqual(expectedResponses);
    expect(
      scope.consoleIdentities.sort(),
      "each expected forbidden GET must produce exactly one location-bound browser diagnostic",
    ).toEqual(expectedConsoleIdentities);
  };
  return assertClean;
}

async function expectNativeDocument(
  page: Page,
  response: Response | null,
  path: string,
  expectedRootMarkup: string,
  documentRootSelector = "#app-root",
) {
  expect(response, `${path} should return a document response`).not.toBeNull();
  expect(response!.status(), `${path} should return a successful document`).toBe(200);
  expect(response!.request().resourceType()).toBe("document");
  expect(response!.headers()["content-type"]).toContain("text/html");
  expect(new URL(page.url()).pathname).toBe(path);

  const html = await response!.text();
  expect(html).toContain(
    documentRootSelector === "#module-content"
      ? 'id="module-content"'
      : 'id="app-root"',
  );
  expect(html).toContain(expectedRootMarkup);
  expect(html).not.toContain("/bridge/");

  await expect(page.locator(documentRootSelector)).toHaveAttribute(
    "data-hydration",
    "ready",
  );
}

async function expectNoJavaScriptNativeDocument(
  page: Page,
  response: Response | null,
  path: string,
  expectedRootMarkup: string,
  documentRootSelector = "#app-root",
) {
  expect(response, `${path} should return a document response`).not.toBeNull();
  expect(response!.status(), `${path} should return a successful document`).toBe(200);
  expect(response!.request().resourceType()).toBe("document");
  expect(response!.headers()["content-type"]).toContain("text/html");
  expect(new URL(page.url()).pathname).toBe(path);

  const html = await response!.text();
  expect(html).toContain(
    documentRootSelector === "#module-content"
      ? 'id="module-content"'
      : 'id="app-root"',
  );
  expect(html).toContain(expectedRootMarkup);
  expect(html).not.toContain("/bridge/");

  await expect(page.locator(documentRootSelector)).toHaveCount(1);
}

async function expectNoPageLevelHorizontalOverflow(page: Page, path: string) {
  await expect
    .poll(
      () =>
        page.evaluate(
          () =>
            document.documentElement.scrollWidth <=
            document.documentElement.clientWidth + 1,
        ),
      { message: `${path} should not create page-level horizontal overflow` },
    )
    .toBe(true);
}

async function expectShellOwnership(
  page: Page,
  expectation: ShellRouteExpectation,
  mobile: boolean,
) {
  await expect(page.locator(".app-shell")).toBeVisible();
  await expect(page.locator(".top-app-bar__title")).toHaveText(
    expectation.shellTitle,
  );

  const navigationScope = mobile
    ? page.locator(".mobile-nav__panel")
    : page.locator(".sidebar");
  if (mobile) {
    await page.getByRole("button", { name: "Open navigation" }).click();
    await expect(navigationScope).toBeVisible();
  } else {
    await expect(navigationScope).toBeVisible();
  }

  const activeLink = navigationScope.locator(
    `a.sidebar-link.is-active[href="${expectation.activeHref}"]`,
  );
  await expect(activeLink).toHaveCount(1);
  await expect(activeLink).toBeVisible();

  if (mobile) {
    await page.getByRole("button", { name: "Close navigation" }).click();
    await expect(navigationScope).toBeHidden();
  }
}

export async function expectShellRouteDirectLoadAndRefresh(
  page: Page,
  expectation: ShellRouteExpectation,
) {
  await page.setViewportSize(DESKTOP_VIEWPORT);
  await expectNativeDocument(
    page,
    await page.goto(expectation.path),
    expectation.path,
    'class="app-shell"',
  );
  await expectShellOwnership(page, expectation, false);
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);

  await page.setViewportSize(MOBILE_VIEWPORT);
  await expectNativeDocument(
    page,
    await page.reload(),
    expectation.path,
    'class="app-shell"',
  );
  await expectShellOwnership(page, expectation, true);
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);
}

async function expectStandaloneOwnership(
  page: Page,
  expectation: StandaloneRouteExpectation,
) {
  await expect(page.locator(expectation.rootSelector)).toBeVisible();
  await expect(page.locator(".app-shell")).toHaveCount(0);
  await expect(page.locator(".sidebar")).toHaveCount(0);
  await expect(page.locator(".mobile-nav")).toHaveCount(0);
}

export async function expectStandaloneRouteDirectLoadAndRefresh(
  page: Page,
  expectation: StandaloneRouteExpectation,
) {
  await page.setViewportSize(DESKTOP_VIEWPORT);
  await expectNativeDocument(
    page,
    await page.goto(expectation.path),
    expectation.path,
    `class="${expectation.rootSelector.replace(/^\./, "")}`,
  );
  await expectStandaloneOwnership(page, expectation);
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);

  await page.setViewportSize(MOBILE_VIEWPORT);
  await expectNativeDocument(
    page,
    await page.reload(),
    expectation.path,
    `class="${expectation.rootSelector.replace(/^\./, "")}`,
  );
  await expectStandaloneOwnership(page, expectation);
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);
}

export async function expectHydratedNativeRouteDirectLoadAndRefresh(
  page: Page,
  expectation: NativeRouteExpectation,
) {
  const documentRootSelector =
    expectation.documentRootSelector ?? "#app-root";
  const expectedRootMarkup =
    expectation.expectedRootMarkup ??
    (documentRootSelector === "#module-content"
      ? 'id="module-content"'
      : 'class="app-shell"');

  await page.setViewportSize(DESKTOP_VIEWPORT);
  await expectNativeDocument(
    page,
    await page.goto(expectation.path),
    expectation.path,
    expectedRootMarkup,
    documentRootSelector,
  );
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);

  await page.setViewportSize(MOBILE_VIEWPORT);
  await expectNativeDocument(
    page,
    await page.reload(),
    expectation.path,
    expectedRootMarkup,
    documentRootSelector,
  );
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);

  await page.setViewportSize(DESKTOP_VIEWPORT);
}

export async function expectNoJavaScriptNativeRouteDirectLoadAndRefresh(
  page: Page,
  expectation: NoJavaScriptNativeRouteExpectation,
) {
  const documentRootSelector =
    expectation.documentRootSelector ?? "#app-root";
  const expectedRootMarkup =
    expectation.expectedRootMarkup ??
    (documentRootSelector === "#module-content"
      ? 'id="module-content"'
      : 'class="app-shell"');

  await page.setViewportSize(DESKTOP_VIEWPORT);
  await expectNoJavaScriptNativeDocument(
    page,
    await page.goto(expectation.path),
    expectation.path,
    expectedRootMarkup,
    documentRootSelector,
  );
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);

  await page.setViewportSize(MOBILE_VIEWPORT);
  await expectNoJavaScriptNativeDocument(
    page,
    await page.reload(),
    expectation.path,
    expectedRootMarkup,
    documentRootSelector,
  );
  await expectation.ready(page);
  await expectNoPageLevelHorizontalOverflow(page, expectation.path);
}
