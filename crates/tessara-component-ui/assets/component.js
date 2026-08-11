// Components release 1.0.1 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_component,
  hydrate_component,
  mount_component,
  navigate_component,
  resume_component,
  suspend_component,
  unmount_component,
} from "/_tessara/modules/tessara.components/1.0.1/sha256:cb0bd057acdafc02eeb8c8318bbe0f3196c2629fab8a9ebacca2bb3c746c817a/component-bindings.js";

await init("/_tessara/modules/tessara.components/1.0.1/sha256:8f3bdee27a9ceaa28ff17a43f71630a39288ca7726ef23071becd68771f6a30e/component.wasm");

if (document.getElementById("module-content")) {
  hydrate_component();
}

export async function createModule(host) {
  if (!host || host.lifecycleAbi !== "1.0.0") {
    throw new Error("Components requires Tessara browser lifecycle ABI 1.0.0");
  }
  let disposed = false;
  globalThis.__tessaraModuleHostV1 = host;
  const assertActive = () => {
    if (disposed) throw new Error("Components lifecycle instance is disposed");
  };
  return {
    async mount(input) {
      assertActive();
      mount_component(input.outletId, JSON.stringify(input.bootstrap.payload));
    },
    async navigate(input) {
      assertActive();
      navigate_component(JSON.stringify(input.bootstrap.payload));
    },
    async canDeactivate() {
      assertActive();
      return can_deactivate_component()
        ? { allowed: true }
        : { allowed: false, prompt: "Discard unsaved Component changes?" };
    },
    async suspend() { assertActive(); suspend_component(); },
    async resume() { assertActive(); resume_component(); },
    async unmount() { if (!disposed) unmount_component(); },
    async dispose() {
      if (!disposed) {
        unmount_component();
        if (globalThis.__tessaraModuleHostV1 === host) delete globalThis.__tessaraModuleHostV1;
        disposed = true;
      }
    },
  };
}
