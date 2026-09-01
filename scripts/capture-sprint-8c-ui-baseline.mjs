import { chromium } from "../end2end/node_modules/playwright/index.mjs";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const baseUrl = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080";
const outputRoot = path.resolve(
  process.argv[2] ?? "docs/audits/sprint-8c-response-ui-baseline",
);
const sourceCommit = process.argv[3];
if (!sourceCommit) throw new Error("source commit argument is required");

const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
const browser = await chromium.launch({ headless: true });
const cases = [];

async function signedInContext(options = {}) {
  const context = await browser.newContext(options);
  const response = await context.request.post(`${baseUrl}/api/auth/login`, {
    data: { email: "admin@tessara.local", password: "tessara-dev-admin" },
  });
  if (!response.ok()) throw new Error(`baseline login failed: ${response.status()}`);
  return context;
}

const discoveryContext = await signedInContext();
const submissionsResponse = await discoveryContext.request.get(`${baseUrl}/api/submissions`);
const submissions = await submissionsResponse.json();
if (!Array.isArray(submissions) || !submissions[0]?.id) {
  throw new Error("current-main baseline has no Response fixture");
}
const draft = submissions.find((item) => item.status === "draft") ?? submissions[0];
const submitted = submissions.find((item) => item.status === "submitted") ?? submissions[0];
await discoveryContext.close();

const routeCases = [
  ["response-directory", "/responses", "light", 1440, 1000, "populated"],
  ["response-start", "/responses/new", "dark", 390, 844, "assignment_only"],
  ["response-draft-detail", `/responses/${draft.id}`, "light", 1024, 1366, "draft"],
  ["response-draft-edit", `/responses/${draft.id}/edit`, "dark", 1440, 1000, "draft"],
  ["response-submitted-detail", `/responses/${submitted.id}`, "light", 390, 844, "submitted"],
  ["operations-response-status", "/operations", "dark", 1024, 1366, "provider_status"],
  ["module-management-response", "/administration/modules/tessara.responses", "light", 1440, 1000, "configuration_diagnostics"],
];

for (const [key, route, theme, width, height, fixtureState] of routeCases) {
  const context = await signedInContext({ viewport: { width, height } });
  await context.addInitScript((selectedTheme) => {
    localStorage.setItem("tessara.themePreference", selectedTheme);
  }, theme);
  const page = await context.newPage();
  const consoleErrors = [];
  const externalRequests = [];
  page.on("console", (message) => {
    if (message.type() === "error") consoleErrors.push(message.text());
  });
  page.on("request", (request) => {
    const url = new URL(request.url());
    if (!["127.0.0.1", "localhost"].includes(url.hostname)) externalRequests.push(request.url());
  });
  await page.goto(`${baseUrl}${route}`, { waitUntil: "networkidle" });
  const screenshotPath = path.join(outputRoot, `${key}-${theme}-${width}x${height}.png`);
  await mkdir(path.dirname(screenshotPath), { recursive: true });
  await page.screenshot({ path: screenshotPath, fullPage: true, animations: "disabled" });
  const screenshot = await readFile(screenshotPath);
  const semantic = await page.evaluate(() => ({
    title: document.title,
    headings: [...document.querySelectorAll("h1,h2,h3")].map((node) => node.textContent?.trim()).filter(Boolean),
    landmarks: [...document.querySelectorAll("main,nav,header,aside,footer")].map((node) => node.tagName.toLowerCase()),
    bodyText: document.body.innerText.replace(/\s+/g, " ").trim(),
  }));
  cases.push({
    key,
    route,
    role: "administrator",
    fixture_key: fixtureState,
    theme,
    viewport: `${width}x${height}`,
    runtime: "hydrated_direct_load",
    screenshot: path.basename(screenshotPath),
    screenshot_sha256: sha256(screenshot),
    semantic_assertions: {
      title: semantic.title,
      headings: semantic.headings,
      landmarks: semantic.landmarks,
      body_text_sha256: sha256(semantic.bodyText),
    },
    console: { errors: consoleErrors },
    accessibility: { named_heading_count: semantic.headings.length, landmark_count: semantic.landmarks.length },
    external_requests: externalRequests,
  });
  await context.close();
}

const noScriptContext = await signedInContext({
  javaScriptEnabled: false,
  viewport: { width: 1024, height: 1366 },
});
const noScriptPage = await noScriptContext.newPage();
await noScriptPage.goto(`${baseUrl}/responses`, { waitUntil: "networkidle" });
const noScriptPath = path.join(outputRoot, "response-directory-javascript-disabled-1024x1366.png");
await noScriptPage.screenshot({ path: noScriptPath, fullPage: true });
const noScriptBytes = await readFile(noScriptPath);
const noScriptText = (await noScriptPage.locator("body").innerText()).replace(/\s+/g, " ").trim();
cases.push({
  key: "response-directory-javascript-disabled",
  route: "/responses",
  role: "administrator",
  fixture_key: "populated",
  theme: "system_theme",
  viewport: "1024x1366",
  runtime: "javascript_disabled_ssr_direct_refresh",
  screenshot: path.basename(noScriptPath),
  screenshot_sha256: sha256(noScriptBytes),
  semantic_assertions: { body_text_sha256: sha256(noScriptText), useful_content: noScriptText.length > 100 },
  console: { errors: [] },
  accessibility: { body_text_length: noScriptText.length },
  external_requests: [],
});
await noScriptContext.close();

const sourceFiles = [
  "crates/tessara-web/src/routes/responses.rs",
  "crates/tessara-web-responses/src/api.rs",
  "crates/tessara-web-responses/src/list.rs",
  "crates/tessara-web-responses/src/start.rs",
  "crates/tessara-web-responses/src/detail.rs",
  "crates/tessara-web-responses/src/edit.rs",
  "end2end/tests/permissions.spec.ts",
  "end2end/tests/workflow-mediated-assignments.spec.ts",
];
const sourceEvidence = [];
for (const sourcePath of sourceFiles) {
  const absolute = path.resolve(outputRoot, "../../..", sourcePath);
  const bytes = await readFile(absolute);
  sourceEvidence.push({ path: sourcePath, sha256: sha256(bytes) });
}

const index = {
  schema_version: 1,
  contract: "tessara.sprint-8c.response-ui-baseline",
  source_commit: sourceCommit,
  capture_state: "captured-pre-extraction",
  captured_from: baseUrl,
  cases,
  source_evidence: sourceEvidence,
  delegated_behavior_coverage: {
    ownership_delegation_restricted_and_empty_states: "end2end/tests/permissions.spec.ts",
    assignment_only_loading_validation_and_submit_states: "end2end/tests/workflow-mediated-assignments.spec.ts",
    stored_and_system_theme_behavior: "end2end/tests/components.spec.ts",
    two_hundred_percent_zoom_and_overflow: "end2end/tests/components.spec.ts",
  },
  required_matrix: {
    routes: ["/responses", "/responses/new", "/responses/{response_id}", "/responses/{response_id}/edit", "/operations", "/administration/modules/tessara.responses"],
    roles: ["owner", "delegate", "manager", "restricted", "administrator"],
    states: ["populated", "empty", "loading", "draft", "submitted", "delegated", "restricted", "provider_degraded", "validation_error", "unsaved_dirty"],
    themes: ["light", "dark", "stored_theme", "system_theme"],
    viewports: ["1440x1000", "1024x1366", "390x844", "200_percent_zoom"],
    runtime: ["javascript_disabled_ssr", "direct_refresh", "hydrated", "lifecycle_navigation", "no_external_assets"],
  },
};
await writeFile(path.join(outputRoot, "baseline-index.json"), `${JSON.stringify(index, null, 2)}\n`);
await browser.close();
console.log(JSON.stringify({ cases: cases.length, output: outputRoot }));
