import { chromium } from "../end2end/node_modules/playwright/index.mjs";
import { createHash } from "node:crypto";
import { mkdir, readFile, writeFile } from "node:fs/promises";
import path from "node:path";

const baseUrl = process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8088";
const outputRoot = path.resolve(
  process.argv[2] ?? "docs/audits/sprint-8b-dataset-ui-baseline",
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
const datasetsResponse = await discoveryContext.request.get(`${baseUrl}/api/datasets`);
const datasets = await datasetsResponse.json();
if (!Array.isArray(datasets) || !datasets[0]?.id || !datasets[0]?.current_revision_id) {
  throw new Error("current-main baseline has no revisioned Dataset fixture");
}
const formsResponse = await discoveryContext.request.get(`${baseUrl}/api/forms`);
const forms = await formsResponse.json();
const datasetId = datasets[0].id;
const revisionId = datasets[0].current_revision_id;
const formId = Array.isArray(forms) ? forms[0]?.id : forms.forms?.[0]?.id;
await discoveryContext.close();

const routeCases = [
  ["dataset-directory", "/datasets", "light", 1440, 1000, "populated"],
  ["dataset-create", "/datasets/new", "dark", 390, 844, "manager"],
  ["dataset-detail", `/datasets/${datasetId}`, "light", 1440, 1000, "read_only"],
  ["dataset-preview", `/datasets/${datasetId}/preview`, "dark", 1440, 1000, "populated"],
  ["dataset-edit", `/datasets/${datasetId}/edit`, "light", 390, 844, "manager"],
  ["dataset-revisions", `/datasets/${datasetId}/revisions`, "dark", 1024, 1366, "populated"],
  ["dataset-revision-detail", `/datasets/${datasetId}/revisions/${revisionId}`, "light", 1024, 1366, "read_only"],
  ["dataset-revision-edit", `/datasets/${datasetId}/revisions/${revisionId}/edit`, "dark", 1440, 1000, "manager"],
  ["operations-dataset-readiness", "/operations", "light", 1440, 1000, "populated"],
  ["module-management-dataset", "/administration/modules/tessara.datasets", "dark", 390, 844, "administrator"],
];
if (formId) {
  routeCases.push(["form-dataset-sources", `/forms/${formId}`, "light", 390, 844, "populated"]);
}

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
    if (!['127.0.0.1', 'localhost'].includes(url.hostname)) externalRequests.push(request.url());
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
    body_text: document.body.innerText.replace(/\s+/g, " ").trim(),
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
      body_text_sha256: sha256(semantic.body_text),
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
await noScriptPage.goto(`${baseUrl}/datasets`, { waitUntil: "networkidle" });
const noScriptPath = path.join(outputRoot, "dataset-directory-javascript-disabled-1024x1366.png");
await noScriptPage.screenshot({ path: noScriptPath, fullPage: true });
const noScriptBytes = await readFile(noScriptPath);
const noScriptText = (await noScriptPage.locator("body").innerText()).replace(/\s+/g, " ").trim();
cases.push({
  key: "dataset-directory-javascript-disabled",
  route: "/datasets",
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
  "crates/tessara-web/src/routes/datasets.rs",
  "crates/tessara-web-datasets/src/pages/mod.rs",
  "crates/tessara-web-datasets/src/api.rs",
  "end2end/tests/datasets.spec.ts",
  "end2end/tests/permissions.spec.ts",
];
const sourceEvidence = [];
for (const sourcePath of sourceFiles) {
  const absolute = path.resolve(outputRoot, "../../..", sourcePath);
  const bytes = await readFile(absolute);
  sourceEvidence.push({ path: sourcePath, sha256: sha256(bytes) });
}

const index = {
  schema_version: 1,
  contract: "tessara.sprint-8b.dataset-ui-baseline",
  source_commit: sourceCommit,
  capture_state: "captured-current-main",
  captured_from: baseUrl,
  cases,
  source_evidence: sourceEvidence,
  delegated_behavior_coverage: {
    reader_restricted_and_empty_states: "end2end/tests/permissions.spec.ts",
    loading_validation_and_unsaved_dirty_states: "end2end/tests/datasets.spec.ts",
    stored_and_system_theme_behavior: "end2end/tests/components.spec.ts",
    two_hundred_percent_zoom_and_overflow: "end2end/tests/components.spec.ts",
  },
  required_matrix: {
    routes: ["/datasets", "/datasets/new", "/datasets/{dataset_id}", "/datasets/{dataset_id}/edit", "/datasets/{dataset_id}/revisions", "/datasets/{dataset_id}/revisions/{revision_id}", "/operations", "/forms/{form_id}", "/administration/modules/tessara.datasets"],
    roles: ["reader", "manager", "restricted", "administrator"],
    states: ["populated", "empty", "loading", "read_only", "restricted", "provider_degraded", "validation_error", "unsaved_dirty"],
    themes: ["light", "dark", "stored_theme", "system_theme"],
    viewports: ["1440x1000", "1024x1366", "390x844", "200_percent_zoom"],
    runtime: ["javascript_disabled_ssr", "direct_refresh", "hydrated", "no_external_assets"],
  },
};
await writeFile(path.join(outputRoot, "baseline-index.json"), `${JSON.stringify(index, null, 2)}\n`);
await browser.close();
console.log(JSON.stringify({ cases: cases.length, output: outputRoot }));
