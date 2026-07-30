# Module SDK Canonical Ownership Inventory

Status: Sprint 6D implementation inventory, started 2026-07-30.

This inventory is the first implementation artifact for
[Sprint 6D](../sprints/sprint-6d-plan.md). It maps the current shared and
duplicated behavior to one canonical target owner before source moves begin.
It is an extraction ledger, not permission to broaden a shared package.

## Target Package Direction

| Target | Canonical responsibility | Forbidden responsibility |
| --- | --- | --- |
| `tessara-module-contract` | Serializable manifests, Shell Context, signed grant/decision envelopes, semantic destinations, typed resource references, stable public errors, configuration/control wire schemas, compatibility identifiers | Axum routing, Leptos rendering, SQLx, process startup, Core authorization decisions, product DTOs |
| `tessara-module-runtime` | Purpose-bound context verification adapters, configuration/control route plumbing, health/readiness/diagnostics orchestration, tracing/correlation, graceful shutdown, standard runtime errors | Product routes, product configuration rules, repositories, migrations, Core policy, module-definition branches |
| `tessara-module-ui` | Normalized shell presentation model, complete-document SSR, design tokens, shared primitives, accessibility behavior, standard route states, asset/hydration conventions | Session lookup, authorization decisions, root routing, Core application state, module product pages |
| `tessara-module-testkit` | Manifest/compatibility fixtures, signed context fixtures, runtime/UI conformance, package/source graph assertions, outage/shutdown helpers | Production policy, product fixtures, a second runtime implementation |
| Core application | Authentication, authorization and Organization decisions, Shell Context projection, navigation composition, semantic resolution, lifecycle/materialization orchestration, fallback routing | Module product rendering or data |
| Functional module | Product routes/UI, domain rules, configuration semantics, persistence, migrations, product diagnostics facts | Reimplementing platform shell/runtime or another module's behavior |

Names are provisional only where the Sprint 6D plan permits refinement. The
ownership and forbidden edges are mandatory.

## Current Source Inventory

| Behavior | Current canonical or duplicated source | Current consumers | Finding | Sprint 6D disposition |
| --- | --- | --- | --- | --- |
| Manifest, inventory, dependency, deployment, protocol, semantic destination, typed-reference, Shell Context, and signed-envelope wire types | `crates/tessara-module-contract/src/{inventory,dependency,deployment,protocol}.rs` | Core, installation control, Dashboard, Scoped Records | Correct policy-neutral nucleus; its public `sdk` module also contains presentation | Retain wire ownership here, make target/dependency audits explicit, and move rendering out |
| Shell Context signature and field validation | `tessara-module-contract::verify_shell_context` plus module-local key construction | Dashboard and Scoped Records | Verification algorithm is canonical, but environment decoding, purpose-bound verifier construction, and request extraction are repeated | Keep wire validation in contract; move runtime construction/extraction adapters to `tessara-module-runtime` |
| `ShellContentV1` remote body fragment | `tessara-module-contract/src/sdk.rs` | Core Scoped Records proxy and root `tessara-web` bridge | Transitional fragment contract conflicts with the target complete-document route-owner model if treated as the new SDK | Keep explicitly transitional for existing behavior; do not use it for the Sprint 6D reference module |
| Complete native module document | `render_native_module_document` in `tessara-module-contract/src/sdk.rs` with embedded CSS | Scoped Records | Presentation and CSS are incorrectly owned by the wire-contract crate; shell is visually separate from root `AppShell` | Move complete-document rendering and styles to `tessara-module-ui`; contract retains only normalized serializable inputs |
| Root authenticated shell | `crates/tessara-web/src/ui/shell/*`, `AppShell`, root auth guards, root navigation state | Core and Dashboard through root `tessara-web` | Presentation, authentication guard, router state, and Core application state are coupled | Extract pure presentation from normalized Shell Context into `tessara-module-ui`; keep Core guard/context projection in root adapters |
| Shared UI primitives | `crates/tessara-web-ui` | Root and feature web crates | Mostly policy-neutral; `placement_editor` imports grid behavior from `tessara-core` | Adopt/narrow into `tessara-module-ui`; split or explicitly isolate the placement editor/grid dependency rather than making it a general SDK dependency |
| Browser JSON transport | `crates/tessara-web-http` | Feature web crates | Already narrow and policy-neutral; hydrate-only dependencies are explicit | Retain as a narrow leaf or absorb behind a clearly named module UI transport layer; do not add endpoint or auth policy |
| Design tokens and CSS | `style/main.css` plus embedded native-module CSS in `tessara-module-contract::sdk` | Core site assets, Dashboard root-linked SSR, Scoped Records native document | Canonical styles are split between a monolithic Core build and an embedded second implementation; modules do not own versioned asset output | Extract token/base/shell sources and module asset conventions to `tessara-module-ui`; each image builds content-addressed assets from that source |
| Process logging, bind, serve, and shutdown | Near-identical `main.rs` in Dashboard and Scoped Records | Dashboard and Scoped Records binaries | `tracing_subscriber`, `TraceLayer`, listener binding, `axum::serve`, Ctrl-C shutdown, and verifier setup are duplicated | Provide runtime builder/start/serve helpers with module-supplied router/state and bind-variable declaration |
| Core verification-key environment decoding | Near-identical base64 and `PurposeBoundVerifyingKeyV1` setup in both module `main.rs` files | Dashboard and Scoped Records | Exact repeated integration code and stable error concerns | Canonical runtime configuration loader produces purpose-bound authorization and shell verifiers |
| Database pool and `migrate` command dispatch | Similar module `main.rs` code using SQLx | Dashboard and Scoped Records | Pattern repeats, but database choice, migrations, pool sizing, and identities are module-owned | Runtime may provide command/lifecycle hooks; SQLx pool and embedded migration ownership stay with the module or an optional database adapter, not the runtime nucleus |
| Configuration validation endpoint | Repeated routes and response plumbing in both module routers; validation rules are module-local | Core Module Management | Route/protocol mechanics repeat correctly; validation semantics must remain module-owned | Runtime owns the standard endpoint adapter and wire envelope; module implements a validator/provider interface |
| Configuration read/apply | Repeated paths and control-key enforcement with module-specific SQL | Core Module Management | Protocol and authorization plumbing repeat; storage and normalization differ | Runtime owns request/auth/error plumbing; module provider owns normalization, persistence, and read-back |
| Private security-state application | Repeated `/api/private/security-state` route and persistence behavior | Core module control client | Stable protocol is repeated; stored state and readiness impact are module-local | Runtime owns validation/auth/dispatch; module provider applies state and reports resulting readiness |
| Liveness and readiness | Repeated `/health/live` and `/health/ready` routes | Compose/Supervisor/Core observation | Endpoint/status vocabulary repeats; readiness inputs differ | Runtime owns route/status envelope and orchestration; module supplies readiness checks |
| Sanitized diagnostics | Dashboard JSON endpoint and Scoped Records HTML/detail behavior | Core Module Management and operators | Sanitization contract and correlation facts need one standard; product facts differ | Runtime/testkit owns redaction envelope and invariant tests; modules provide typed safe facts |
| Trace/correlation behavior | `TraceLayer::new_for_http` in both modules; Shell Context carries correlation ID | Module services and gateway | Trace layer setup repeats and correlation propagation is incomplete as a shared rule | Runtime owns standard trace setup and request correlation extraction/propagation |
| Module control registry and schema-driven Module Management | Core `modules` service/routes/pages and `TESSARA_MODULE_CONTROL_ENDPOINTS` | Dashboard and Scoped Records | Correct Core-owned lifecycle/policy behavior; not SDK source | Keep in Core; testkit supplies a conforming module fixture without copying Core control logic |
| Dashboard complete-document SSR | Dashboard service calls root `tessara_web::application_html_with_dashboard_bootstrap` | Dashboard route proxy | Known Sprint 6C source/build transition | Inventory and enforce as an allowed temporary edge; removal is Sprint 6E, not 6D |
| Scoped Records product document | Module-local HTML strings plus `render_native_module_document` | Scoped Records routes | Product markup is legitimately module-owned, but shared shell/styles come from the transitional renderer | Preserve product behavior; migrate shared shell/runtime source only when required for conformance, without product redesign |
| Docker image provenance | Core, Dashboard, installation-control, and Scoped Records Dockerfiles accept source commit/tree/dirty build args | Deployment evidence | Required convention is repeated correctly but verified through sprint-specific harnesses | Testkit/deployment helpers canonically validate labels and immutable digests; images continue to carry their own labels |
| Compose/bootstrap/conformance | Sprint 6C Compose, bootstrap, isolation/degraded-state scripts, smoke/UAT/Playwright | Reference installation | Strong baseline but feature-specific naming and evidence remain | Sprint 6D layers a reference fixture and generic SDK evidence onto the retained topology; it does not fork the full harness |

## Immediate Dependency Findings

1. `tessara-module-contract` has no Leptos, Axum, SQLx, JavaScript, or browser
   dependency, but `src/sdk.rs` mixes wire verification with HTML/CSS
   presentation. This is the first source split.
2. `tessara-web-ui` is not fully independent because
   `placement_editor.rs` imports `GridConstraints` and grid operations from
   `tessara-core`. Other UI primitives have no observed Core/root import.
3. `tessara-web-http` has no root or product dependency and is already a
   viable policy-neutral leaf.
4. Dashboard and Scoped Records duplicate process startup, verifier
   construction, tracing, graceful shutdown, and standard control/probe route
   plumbing.
5. Root `AppShell` cannot move as-is because it invokes a Core authentication
   guard and reads root shell/navigation state. The canonical UI boundary must
   accept a normalized presentation model derived from verified Shell Context.
6. Dashboard's `tessara-web` dependency is a known temporary edge and must
   remain visible in the Sprint 6D audit output. Sprint 6D must not declare the
   Dashboard graph conforming; Sprint 6E removes that edge.
7. The existing native module renderer duplicates shell CSS inside a Rust
   format string. It is useful evidence for no-JavaScript complete-document
   behavior, not the final design-system implementation.

## First Extraction Gates

Before moving shared implementation source:

- add a package-graph specification for the four canonical package roles and
  the Sprint 6D reference module on native and WASM targets where applicable;
- add source-pattern checks that reject Core/root/product imports from those
  packages;
- retain explicit temporary findings for Dashboard root-web coupling and the
  transitional `ShellContentV1` fragment path;
- characterize current `render_native_module_document`, root `AppShell`,
  Dashboard, and Scoped Records SSR/no-JavaScript output so extraction does not
  silently change product behavior;
- keep configuration, readiness, diagnostics facts, persistence, migrations,
  and product routes behind module-supplied interfaces rather than moving them
  into a shared implementation.

## Out Of Scope For This Inventory

- removing Dashboard's root-web dependency or moving its product assets
  (Sprint 6E);
- changing Dashboard, Scoped Records, or Core product UX;
- Blueprint/composition automation;
- physical Components or other feature extraction;
- turning transitional `ShellContentV1` fragments into the target module UI
  model;
- introducing a runtime-loaded browser microfrontend, iframe, or remote WASM
  contract.
