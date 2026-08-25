// Dashboard release 3.0.2 complete-document and lifecycle-v1 entrypoint.
import init, {
  can_deactivate_dashboard,
  hydrate_dashboard,
  mount_dashboard,
  navigate_dashboard,
  resume_dashboard,
  suspend_dashboard,
  unmount_dashboard,
} from "/_tessara/modules/tessara.dashboards/3.0.2/sha256:442bdedbeba7f973a0fca3a5d61fee2e9fd62af3cda42cf331b6d6a6eea5f649/dashboard-bindings.js";

await init("/_tessara/modules/tessara.dashboards/3.0.2/sha256:d3ea9abac5e0ce0930c8013e0226d9a119c8f4804b49163dca5e1137825cab77/dashboard.wasm");

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
