// Dataset release 1.0.0 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_dataset,
  hydrate_dataset,
  mount_dataset,
  navigate_dataset,
  resume_dataset,
  suspend_dataset,
  unmount_dataset,
} from "/_tessara/modules/tessara.datasets/1.0.0/sha256:f3e5d1671522c238a54628a7c44f110b9097fe9600e2ba8045635f09a308d7f8/dataset-bindings.js";

await init("/_tessara/modules/tessara.datasets/1.0.0/sha256:7c13c2d90bad64420979668dc1bd484bbbe59cd124eb31b5099514f92704abc7/dataset.wasm");

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
