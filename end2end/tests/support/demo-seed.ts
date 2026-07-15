import type { APIRequestContext, APIResponse } from "@playwright/test";

export function shouldInvokeDemoSeedEndpoint(): boolean {
  if (process.env.TESSARA_PLAYWRIGHT_ACCEPTANCE !== "1") {
    return true;
  }

  const dataState = process.env.TESSARA_PLAYWRIGHT_DATA_STATE;
  if (dataState !== "upgraded" && dataState !== "fresh") {
    throw new Error(
      `Playwright acceptance requires exact upgraded|fresh data state; received ${JSON.stringify(dataState)}`,
    );
  }

  // Gate 4 proves the restored Sprint 5A demo rows survived migration 3. It
  // must never invoke a demo mutation path after that migration.
  return dataState === "fresh";
}

export async function invokeDemoSeedEndpoint(
  request: Pick<APIRequestContext, "post">,
): Promise<APIResponse | null> {
  if (!shouldInvokeDemoSeedEndpoint()) {
    return null;
  }
  return request.post("/api/demo/seed", { data: {} });
}
