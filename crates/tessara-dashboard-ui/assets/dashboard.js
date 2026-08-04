// Dashboard release 2.1.0 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_dashboard,
  hydrate_dashboard,
  mount_dashboard,
  navigate_dashboard,
  resume_dashboard,
  suspend_dashboard,
  unmount_dashboard,
} from "/_tessara/modules/tessara.dashboards/2.1.0/sha256:ca2bca20d4495516457f431d82117517b0fb6fe31496d2c9b0f86409c7dc68cb/dashboard-bindings.js";

await init("/_tessara/modules/tessara.dashboards/2.1.0/sha256:8058f7895f1d72fe03a94bb2224ef8f83b0676a3e0a174a9d0ebb62bac183d28/dashboard.wasm");

if (document.getElementById("module-content")) {
  hydrate_dashboard();
}

export async function createModule(host) {
  if (!host || host.lifecycleAbi !== "1.0.0") {
    throw new Error("Dashboard requires Tessara browser lifecycle ABI 1.0.0");
  }
  let disposed = false;
  globalThis.__tessaraModuleHostV1 = host;
  const assertActive = () => {
    if (disposed) throw new Error("Dashboard lifecycle instance is disposed");
  };
  return {
    async mount(input) {
      assertActive();
      mount_dashboard(input.outletId, JSON.stringify(input.bootstrap.payload));
    },
    async navigate(input) {
      assertActive();
      navigate_dashboard(JSON.stringify(input.bootstrap.payload));
    },
    async canDeactivate() {
      assertActive();
      return can_deactivate_dashboard()
        ? { allowed: true }
        : {
            allowed: false,
            prompt: "Discard unsaved Dashboard layout changes?",
          };
    },
    async suspend() {
      assertActive();
      suspend_dashboard();
    },
    async resume() {
      assertActive();
      resume_dashboard();
    },
    async unmount() {
      if (!disposed) unmount_dashboard();
    },
    async dispose() {
      if (!disposed) {
        unmount_dashboard();
        if (globalThis.__tessaraModuleHostV1 === host) {
          delete globalThis.__tessaraModuleHostV1;
        }
        disposed = true;
      }
    },
  };
}
