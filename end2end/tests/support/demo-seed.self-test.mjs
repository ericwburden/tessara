import assert from "node:assert/strict";
import { readdir, readFile } from "node:fs/promises";
import { dirname, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { invokeDemoSeedEndpoint } from "./demo-seed.ts";

const supportDirectory = dirname(fileURLToPath(import.meta.url));
const testsDirectory = resolve(supportDirectory, "..");
const environmentNames = [
  "TESSARA_PLAYWRIGHT_ACCEPTANCE",
  "TESSARA_PLAYWRIGHT_DATA_STATE",
];
const savedEnvironment = new Map(
  environmentNames.map((name) => [name, process.env[name]]),
);

async function runGuardProbe({ acceptance, dataState, expectedCalls }) {
  if (acceptance === undefined) {
    delete process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE;
  } else {
    process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE = acceptance;
  }
  if (dataState === undefined) {
    delete process.env.TESSARA_PLAYWRIGHT_DATA_STATE;
  } else {
    process.env.TESSARA_PLAYWRIGHT_DATA_STATE = dataState;
  }

  let calls = 0;
  const response = { marker: "demo-seed-response" };
  const request = {
    async post(url, options) {
      calls += 1;
      assert.equal(url, "/api/demo/seed");
      assert.deepEqual(options, { data: {} });
      return response;
    },
  };
  const result = await invokeDemoSeedEndpoint(request);
  assert.equal(calls, expectedCalls);
  assert.equal(result, expectedCalls === 0 ? null : response);
}

async function findSeedEndpointLiterals(directory) {
  const matches = [];
  for (const entry of await readdir(directory, { withFileTypes: true })) {
    const path = resolve(directory, entry.name);
    if (entry.isDirectory()) {
      matches.push(...(await findSeedEndpointLiterals(path)));
      continue;
    }
    if (!entry.isFile() || !entry.name.endsWith(".ts")) {
      continue;
    }
    const source = await readFile(path, "utf8");
    const count = source.match(/["']\/api\/demo\/seed["']/g)?.length ?? 0;
    if (count > 0) {
      matches.push({ path: relative(testsDirectory, path).replaceAll("\\", "/"), count });
    }
  }
  return matches;
}

try {
  await runGuardProbe({ acceptance: "1", dataState: "upgraded", expectedCalls: 0 });
  await runGuardProbe({ acceptance: "1", dataState: "fresh", expectedCalls: 1 });
  await runGuardProbe({ acceptance: undefined, dataState: undefined, expectedCalls: 1 });

  process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE = "1";
  process.env.TESSARA_PLAYWRIGHT_DATA_STATE = "invalid";
  let invalidCalls = 0;
  await assert.rejects(
    invokeDemoSeedEndpoint({
      async post() {
        invalidCalls += 1;
        return {};
      },
    }),
    /requires exact upgraded\|fresh data state/,
  );
  assert.equal(invalidCalls, 0);

  assert.deepEqual(await findSeedEndpointLiterals(testsDirectory), [
    { path: "support/demo-seed.ts", count: 1 },
  ]);
} finally {
  for (const [name, value] of savedEnvironment) {
    if (value === undefined) {
      delete process.env[name];
    } else {
      process.env[name] = value;
    }
  }
}

console.log("Playwright demo seed guard and endpoint inventory self-test passed.");
