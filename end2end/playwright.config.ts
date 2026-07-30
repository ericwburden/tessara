import { defineConfig } from "@playwright/test";

const acceptance = process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE === "1";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  forbidOnly: acceptance,
  // The repository suite mutates installation-wide module enablement and
  // shared demo fixtures. Keep the documented direct runner state-safe too;
  // focused diagnostic runs can still override this explicitly on the CLI.
  workers: 1,
  retries: 0,
  reporter: acceptance
    ? [["list"], ["json"], ["junit", { includeProjectInTestName: true }]]
    : [["list"]],
  use: {
    baseURL: process.env.PLAYWRIGHT_BASE_URL ?? "http://127.0.0.1:8080",
    trace: "retain-on-failure",
  },
});
