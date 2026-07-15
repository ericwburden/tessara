import { defineConfig } from "@playwright/test";

const acceptance = process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE === "1";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  forbidOnly: acceptance,
  workers: acceptance ? 1 : undefined,
  retries: 0,
  reporter: acceptance
    ? [["list"], ["json"], ["junit", { includeProjectInTestName: true }]]
    : [["list"]],
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080",
    trace: "retain-on-failure",
  },
});
