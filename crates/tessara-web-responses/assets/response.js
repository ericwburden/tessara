// Response release 1.0.0 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_response,
  hydrate_response,
  mount_response,
  navigate_response,
  resume_response,
  suspend_response,
  unmount_response,
} from "/_tessara/modules/tessara.responses/1.0.0/sha256:d09a3596639d1ffeec5443f019c996cb160fbf935efc9c68c3531d8d3ffaa130/response-bindings.js";

await init("/_tessara/modules/tessara.responses/1.0.0/sha256:f46b5ad579bf45891d7fd8ceb408435859c91c26c82865fb6f84bc0b2d6b446b/response.wasm");

if (document.getElementById("module-content")) {
  hydrate_response();
}

export async function createModule(host) {
  if (!host || host.lifecycleAbi !== "1.0.0") {
    throw new Error("Responses requires Tessara browser lifecycle ABI 1.0.0");
  }
  let disposed = false;
  globalThis.__tessaraModuleHostV1 = host;
  const assertActive = () => {
    if (disposed) throw new Error("Responses lifecycle instance is disposed");
  };
  return {
    async mount(input) {
      assertActive();
      mount_response(input.outletId, JSON.stringify(input.bootstrap.payload));
    },
    async navigate(input) {
      assertActive();
      navigate_response(JSON.stringify(input.bootstrap.payload));
    },
    async canDeactivate() {
      assertActive();
      return can_deactivate_response()
        ? { allowed: true }
        : { allowed: false, prompt: "Discard unsaved Response changes?" };
    },
    async suspend() { assertActive(); suspend_response(); },
    async resume() { assertActive(); resume_response(); },
    async unmount() { if (!disposed) unmount_response(); },
    async dispose() {
      if (!disposed) {
        unmount_response();
        if (globalThis.__tessaraModuleHostV1 === host) delete globalThis.__tessaraModuleHostV1;
        disposed = true;
      }
    },
  };
}
