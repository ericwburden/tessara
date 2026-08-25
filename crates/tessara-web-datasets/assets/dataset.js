// Dataset release 1.0.0 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_dataset,
  hydrate_dataset,
  mount_dataset,
  navigate_dataset,
  resume_dataset,
  suspend_dataset,
  unmount_dataset,
} from "/_tessara/modules/tessara.datasets/1.0.0/sha256:19cd615ea13d76fbde4644a03b10088647c18f5f11d5d2583733c348fbc693e6/dataset-bindings.js";

await init("/_tessara/modules/tessara.datasets/1.0.0/sha256:c62ceaebff98846bd4f39f32b12c1d72791c62cc6e79398fd723d781f99a8550/dataset.wasm");

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
