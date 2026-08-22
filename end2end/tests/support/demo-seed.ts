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

  // Acceptance always consumes the source-exact topology selected by its
  // materialization receipt. Neither a fresh Reference apply nor a restored
  // upgrade topology may be mutated through the legacy demo seed endpoint.
  return false;
}

export async function invokeDemoSeedEndpoint(
  request: Pick<APIRequestContext, "post">,
): Promise<APIResponse | null> {
  if (!shouldInvokeDemoSeedEndpoint()) {
    return null;
  }
  return request.post("/api/demo/seed", { data: {} });
}
