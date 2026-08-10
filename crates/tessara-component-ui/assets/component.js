// Components release 1.0.1 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_component,
  hydrate_component,
  mount_component,
  navigate_component,
  resume_component,
  suspend_component,
  unmount_component,
} from "/_tessara/modules/tessara.components/1.0.1/sha256:5ced05ea4b5677db7aed50f26c651fa7c5c88a7ff400fc7ba6b21e5611fc3fe4/component-bindings.js";

await init("/_tessara/modules/tessara.components/1.0.1/sha256:169f860178f0d562f7531750d0d0394f478facce7e52350deefac3f6790cb5fe/component.wasm");

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
