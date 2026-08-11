// Components release 1.0.1 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_component,
  hydrate_component,
  mount_component,
  navigate_component,
  resume_component,
  suspend_component,
  unmount_component,
} from "/_tessara/modules/tessara.components/1.0.1/sha256:4cc5a22156729d153d374a217f88d89871fe7f92edff8a0230459b9f013abb95/component-bindings.js";

await init("/_tessara/modules/tessara.components/1.0.1/sha256:3c3ae2c8ed44cddd238de8e59d634343b5615221da729c163f44d848d4a84bf7/component.wasm");

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
