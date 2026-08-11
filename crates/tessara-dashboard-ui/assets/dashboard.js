// Dashboard release 3.0.1 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_dashboard,
  hydrate_dashboard,
  mount_dashboard,
  navigate_dashboard,
  resume_dashboard,
  suspend_dashboard,
  unmount_dashboard,
} from "/_tessara/modules/tessara.dashboards/3.0.1/sha256:360df2c768d51f0edfccc41525fda152efc39c3548499ad6a45158f6494f88ae/dashboard-bindings.js";

await init("/_tessara/modules/tessara.dashboards/3.0.1/sha256:6b259b73b236c782f157014baeac1b6e14de0a1435b100c4bf42684542116ca4/dashboard.wasm");

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
