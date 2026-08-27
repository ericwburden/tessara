// Components release 1.0.1 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_component,
  hydrate_component,
  mount_component,
  navigate_component,
  resume_component,
  suspend_component,
  unmount_component,
} from "/_tessara/modules/tessara.components/1.1.0/sha256:eda4dbf4c50504041f4966d809ceb80afaaf5c713544b332ac9618d22e5b5039/component-bindings.js";

await init("/_tessara/modules/tessara.components/1.1.0/sha256:0a26e247986e233430da02b89dba36fdf101496824df5aa901b7dd5a4c8e7287/component.wasm");

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
