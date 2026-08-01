# Module Browser Lifecycle v1

Status: Sprint 6E implementation contract

Module Browser Lifecycle v1 lets an independently deployed module render inside
the persistent Core shell without coupling Core to the module's UI framework.
Complete HTML documents remain mandatory for direct requests, JavaScript-free
operation, and compatibility recovery.

## Manifest declaration

An interactive release declares `browser_lifecycle` alongside its existing
`browser_routes` and immutable `assets`:

```json
{
  "lifecycle_abi": "1.0.0",
  "entry_asset": "/module.js",
  "stylesheet_assets": ["/module.css"],
  "complete_document_fallback": true,
  "capabilities": {
    "navigation_guard": true,
    "suspend_resume": true
  }
}
```

The entry and stylesheet paths must name correctly typed assets in the same
manifest. Core binds those assets to the installed definition, release, and
content digest. Published release identities and assets are immutable.

Absence of `browser_lifecycle` means the module uses complete-document
navigation. Transitional contributions still rendered by Core remain native
shell routes.

## Route bootstrap

Core requests the ordinary browser route with:

```http
Accept: application/vnd.tessara.module-view+json; version=1
```

The module performs the same authorization and scope filtering as a complete
document request, then returns `BrowserLifecycleBootstrapV1`. Core validates
the generic envelope, installed release identity, route, and immutable assets.
The `payload` member is opaque to Core and owned by the module UI.

Without that media type, the route returns its complete HTML document. Both
representations are private, non-cacheable, and vary on `Accept` and actor
authorization.

## JavaScript ABI

Core imports the digest-addressed ECMAScript entry and calls:

```ts
createModule(host: ModuleHostV1): Promise<ModuleInstanceV1>
```

The returned instance implements:

```ts
mount(input): Promise<void>
navigate(input): Promise<void>
canDeactivate(input?): Promise<{ allowed: boolean; prompt?: string }>
suspend(): Promise<void>
resume(input?): Promise<void>
unmount(reason?): Promise<void>
dispose(): Promise<void>
```

`mount` renders once into the supplied Core-owned outlet. `navigate` applies a
same-release route bootstrap without re-importing the runtime. `unmount`
removes visible UI and cancels route-owned work. `dispose` permanently releases
the instance. Cleanup operations must be safe when repeated. Core may remove
the outlet after a bounded cleanup failure.

The v1 host exposes `lifecycleAbi` and `navigate(href)`. Module code uses the
host callback for same-origin navigation so Core can arbitrate dirty-state
guards, browser history, focus, and module transitions.

## Ownership

Core owns top-level history, shell DOM, session, theme, locale, runtime import,
stylesheets, navigation arbitration, focus restoration, document title, and
host failure states. A module owns everything inside its outlet, route payload
interpretation, data loading, dirty-state detection, and cleanup. Neither side
queries or mutates the other's private DOM.

Module styles are loaded and removed by Core and must be scoped to the module
outlet. Shared shell and design-system styles remain Core-owned.

## Release and failure behavior

A mounted instance is pinned to one definition, release, ABI, and digest set.
Deployment changes never hot-replace it. A new release activates on a clean
module activation or document load.

Authorization failures, malformed bootstraps, ABI mismatches, missing exports,
asset failures, and lifecycle exceptions fail closed to a Core-owned state with
an explicit complete-document recovery link. A failed soft transition does not
grant authority or expose an unfiltered payload.

## Rust/WASM adapter

`tessara-module-ui::LeptosLifecycleRoot` owns the Leptos unmount handle. Module
implementations retain that adapter for the lifetime of the JavaScript module
instance; replacing or dropping it disposes the reactive owner and removes the
view. Dashboards is the first conforming implementation.

## Conformance expectations

A lifecycle-v1 module must demonstrate:

- direct, refresh, deep-link, and JavaScript-disabled document behavior;
- soft Core-to-module, module-to-module, and module-to-Core navigation;
- Back and Forward behavior;
- dirty-state navigation arbitration;
- repeated mount, navigate, suspend, resume, unmount, and dispose calls;
- cleanup of requests, timers, observers, listeners, overlays, and styles;
- safe bootstrap, authorization, asset, ABI, timeout, and runtime failures;
- candidate activation and rollback without persisted-data loss.
