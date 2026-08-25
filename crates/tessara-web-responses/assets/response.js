// Response release 1.0.0 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_response,
  hydrate_response,
  mount_response,
  navigate_response,
  resume_response,
  suspend_response,
  unmount_response,
} from "/_tessara/modules/tessara.responses/1.0.0/sha256:67df799cbf64eac17e122b7aa2e68f1f9bbc885e68a408d208b6dc3285bbaddf/response-bindings.js";

await init("/_tessara/modules/tessara.responses/1.0.0/sha256:4c825e13ac052c01cdd9445b2a2868c6b0025571689a6cffb653f608b7b3169a/response.wasm");

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
