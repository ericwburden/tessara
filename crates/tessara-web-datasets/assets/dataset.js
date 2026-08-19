// Dataset release 1.0.0 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_dataset,
  hydrate_dataset,
  mount_dataset,
  navigate_dataset,
  resume_dataset,
  suspend_dataset,
  unmount_dataset,
} from "/_tessara/modules/tessara.datasets/1.0.0/sha256:98cf0aae3429c493ff5a2acc9cf44b7ba685d002f76c8f296d41c53f0215f1e8/dataset-bindings.js";

await init("/_tessara/modules/tessara.datasets/1.0.0/sha256:851f0906ba4dc217f36e8fd4c222e65a7d5d61118427bab25746b6ee2953fbd6/dataset.wasm");

if (document.getElementById("module-content")) {
  hydrate_dataset();
}

export async function createModule(host) {
  if (!host || host.lifecycleAbi !== "1.0.0") {
    throw new Error("Datasets requires Tessara browser lifecycle ABI 1.0.0");
  }
  let disposed = false;
  globalThis.__tessaraModuleHostV1 = host;
  const assertActive = () => {
    if (disposed) throw new Error("Datasets lifecycle instance is disposed");
  };
  return {
    async mount(input) {
      assertActive();
      mount_dataset(input.outletId, JSON.stringify(input.bootstrap.payload));
    },
    async navigate(input) {
      assertActive();
      navigate_dataset(JSON.stringify(input.bootstrap.payload));
    },
    async canDeactivate() {
      assertActive();
      return can_deactivate_dataset()
        ? { allowed: true }
        : { allowed: false, prompt: "Discard unsaved Dataset changes?" };
    },
    async suspend() { assertActive(); suspend_dataset(); },
    async resume() { assertActive(); resume_dataset(); },
    async unmount() { if (!disposed) unmount_dataset(); },
    async dispose() {
      if (!disposed) {
        unmount_dataset();
        if (globalThis.__tessaraModuleHostV1 === host) delete globalThis.__tessaraModuleHostV1;
        disposed = true;
      }
    },
  };
}
